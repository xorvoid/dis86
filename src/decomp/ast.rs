use crate::decomp::ir;
use crate::decomp::control_flow::{self, ControlFlow, Detail, ElemId};
use std::collections::HashMap;

type FlowIter<'a> = std::iter::Peekable<control_flow::ControlFlowIter<'a>>;

#[derive(Debug)]
pub enum Expr {
  Unary(Box<UnaryExpr>),
  Binary(Box<BinaryExpr>),
  Const(i64),
  Name(String),
  Call(Box<Expr>, Vec<Expr>),
  Abstract(&'static str, Vec<Expr>),
  Deref(Box<Expr>),
  Cast(&'static str, Box<Expr>),
  Ptr16(Box<Expr>, Box<Expr>),
  Ptr32(Box<Expr>, Box<Expr>),
  UnimplPhi,
  UnimplPin,
}

#[derive(Debug)]
pub enum UnaryOperator {
  Addr, Not,
}

#[derive(Debug)]
pub struct UnaryExpr {
  pub op: UnaryOperator,
  pub rhs: Expr,
}

#[derive(Debug)]
pub enum BinaryOperator {
  Add, Sub, UDiv, And, Or, Xor, Shl, Shr, Eq, Neq, Gt, Geq, Lt, Leq,
}

#[derive(Debug)]
pub struct BinaryExpr {
  pub op: BinaryOperator,
  pub lhs: Expr,
  pub rhs: Expr,
}

#[derive(Debug)]
pub struct Assign {
  pub lhs: Expr,
  pub rhs: Expr,
}

#[derive(Debug)]
pub struct Label(pub String); // fixme??

#[derive(Debug)]
pub struct CondGoto {
  pub cond: Expr,
  pub label_true: Label,
  pub label_false: Label,
}

#[derive(Debug)]
pub struct Goto {
  pub label: Label,
}

#[derive(Debug)]
pub struct Loop {
  pub body: Block,
}

#[derive(Debug)]
pub struct If {
  pub cond: Expr,
  pub then_body: Block,
}

#[derive(Debug)]
pub enum Stmt {
  Label(Label),
  Instr(ir::Ref),
  Expr(Expr),
  Assign(Assign),
  CondGoto(CondGoto),
  Goto(Goto),
  Return,
  Loop(Loop),
  If(If),
}

#[derive(Debug)]
pub struct Function {
  pub name: String,
  pub body: Block,
}

#[derive(Debug, Default)]
pub struct Block(pub Vec<Stmt>);

struct Builder<'a> {
  ir: &'a ir::IR,
  cf: &'a ControlFlow,
  n_uses: HashMap<ir::Ref, usize>,
  temp_names: HashMap<ir::Ref, String>,
  temp_count: usize,
}

impl Block {
  fn push_stmt(&mut self, stmt: Stmt) {
    self.0.push(stmt);
  }
}

impl<'a> Builder<'a> {
  fn new(ir: &'a ir::IR, cf: &'a ControlFlow) -> Self {
    let n_uses = ir::util::compute_uses(ir);
    Self {
      ir,
      cf,
      n_uses,
      temp_names: HashMap::new(),
      temp_count: 0,
    }
  }

  fn lookup_uses(&self, r: ir::Ref) -> usize {
    *self.n_uses.get(&r).unwrap_or(&0)
  }

  fn ref_name(&mut self, r: ir::Ref) -> String {
    if let Some(n) = self.ir.names.get(&r) {
      return format!("{}_{}", n.0, n.1);
    }
    if let Some(n) = self.temp_names.get(&r) {
      return n.clone();
    }
    let name = format!("tmp_{}", self.temp_count);
    self.temp_count += 1;
    self.temp_names.insert(r, name.clone());
    name
  }

  fn ref_to_binary_expr(&mut self, r: ir::Ref, depth: usize) -> Option<Expr> {
    let instr = self.ir.instr(r).unwrap();

    let ast_op = match instr.opcode {
      ir::Opcode::Add => BinaryOperator::Add,
      ir::Opcode::Sub => BinaryOperator::Sub,
      ir::Opcode::UDiv => BinaryOperator::UDiv,
      ir::Opcode::And => BinaryOperator::And,
      ir::Opcode::Or => BinaryOperator::Or,
      ir::Opcode::Xor => BinaryOperator::Xor,
      ir::Opcode::Shl => BinaryOperator::Shl,
      ir::Opcode::Shr => BinaryOperator::Shr,
      ir::Opcode::Eq => BinaryOperator::Eq,
      ir::Opcode::Neq => BinaryOperator::Neq,
      ir::Opcode::Gt => BinaryOperator::Gt,
      ir::Opcode::Geq => BinaryOperator::Geq,
      ir::Opcode::Lt => BinaryOperator::Lt,
      ir::Opcode::Leq => BinaryOperator::Leq,
      _ => return None,
    };

    let lhs = self.ref_to_expr(instr.operands[0], depth+1);
    let rhs = self.ref_to_expr(instr.operands[1], depth+1);

    Some(Expr::Binary(Box::new(BinaryExpr {
      op: ast_op,
      lhs,
      rhs,
    })))
  }

  fn ref_to_expr(&mut self, r: ir::Ref, depth: usize) -> Expr {
    match self.ir.lookup_const(r) {
      Some(k) => return Expr::Const(k as i64),
      None => (),
    }
    if depth > 0 && self.lookup_uses(r) != 1 {
      let name = self.ref_name(r);
      return Expr::Name(name);
    }

    if let Some(expr) = self.ref_to_binary_expr(r, depth) {
      return expr;
    }

    let instr = self.ir.instr(r).unwrap();
    match instr.opcode {
      ir::Opcode::Load16 => {
        let seg = self.ref_to_expr(instr.operands[0], depth+1);
        let off = self.ref_to_expr(instr.operands[1], depth+1);
        Expr::Deref(Box::new(Expr::Ptr16(Box::new(seg), Box::new(off))))
      }
      ir::Opcode::Load32 => {
        let seg = self.ref_to_expr(instr.operands[0], depth+1);
        let off = self.ref_to_expr(instr.operands[1], depth+1);
        Expr::Deref(Box::new(Expr::Ptr32(Box::new(seg), Box::new(off))))
      }
      ir::Opcode::Upper16 => {
        let lhs = self.ref_to_expr(instr.operands[0], depth+1);
        Expr::Binary(Box::new(BinaryExpr {
          op: BinaryOperator::Shr,
          lhs,
          rhs: Expr::Const(16),
        }))
      }
      ir::Opcode::Lower16 => {
        let lhs = self.ref_to_expr(instr.operands[0], depth+1);
        lhs
      }
      ir::Opcode::ReadVar16 => {
        self.symbol_to_expr(instr.operands[0].unwrap_symbol())
      }
      ir::Opcode::ReadVar32 => {
        self.symbol_to_expr(instr.operands[0].unwrap_symbol())
      }
      ir::Opcode::CallArgs => {
        let funcidx = instr.operands[0].unwrap_func();
        let funcname = self.ir.funcs[funcidx].clone();
        let mut args = vec![];
        for a in &instr.operands[1..] {
          args.push(self.ref_to_expr(*a, depth+1));
        }
        Expr::Call(Box::new(Expr::Name(funcname)), args)
      }
      ir::Opcode::Phi => {
        // generally handled by jmp, but other expressions that use a phi might end up here
        // so, we can simply return our refname
        Expr::Name(self.ref_name(r))
      }
      ir::Opcode::Pin => Expr::UnimplPin,
      ir::Opcode::Make32 => {
        let exprs: Vec<_> = instr.operands.iter().map(|r| self.ref_to_expr(*r, depth+1)).collect();
        Expr::Abstract("MAKE_32", exprs)
      }
      ir::Opcode::SignExtTo32 => {
        let exprs: Vec<_> = instr.operands.iter().map(|r| self.ref_to_expr(*r, depth+1)).collect();
        Expr::Abstract("SIGNEXT_32", exprs)
      }
      opcode @ _ => {
        panic!("Unimplemented {:?} in ast converter", opcode);
      }
    }
  }

  fn symbol_to_expr(&self, symref: ir::sym::SymbolRef) -> Expr {
    let sym = self.ir.symbols.symbol(symref);
    let mut expr = Expr::Name(sym.name.clone());
    if symref.off != 0 ||  symref.sz != sym.size {
      expr = Expr::Unary(Box::new(UnaryExpr {
        op: UnaryOperator::Addr,
        rhs: expr,
      }));
      if symref.off != 0 {
        expr = Expr::Binary(Box::new(BinaryExpr {
          op: BinaryOperator::Add,
          lhs: expr,
          rhs: Expr::Const(symref.off as i64),
        }));
      }
      expr = Expr::Cast("u16*", Box::new(expr));
      expr = Expr::Deref(Box::new(expr));
    }
    expr
  }

  fn make_label(&self, id: ElemId) -> Label {
    let elem = self.cf.elem(id);
    let Detail::BasicBlock(bb) = &elem.detail else { panic!("Expected basic block") };
    Label(format!("{}", self.ir.blocks[bb.blkref.0].name))
  }

  fn emit_phis(&mut self, blk: &mut Block, src: ir::BlockRef, dst: ir::BlockRef) {
    // first, which pred is the src block?
    let mut idx = None;
    for (i, pred) in self.ir.blocks[dst.0].preds.iter().enumerate() {
      if *pred == src {
        idx = Some(i);
        break;
      }
    }
    let idx = idx.unwrap();

    // next, for each phi, generate code for the pred idx
    for i in self.ir.blocks[dst.0].instrs.range() {
      let r = ir::Ref::Instr(dst, i);
      let instr = self.ir.instr(r).unwrap();
      if instr.opcode != ir::Opcode::Phi { continue };

      let name = self.ref_name(r);
      let rvalue = self.ref_to_expr(instr.operands[idx], 1);
      blk.push_stmt(Stmt::Assign(Assign {
        lhs: Expr::Name(name),
        rhs: rvalue,
      }));
    }
  }

  // Returns a jump condition expr if the block ends in a conditional branch
  #[must_use]
  fn emit_blk(&mut self, blk: &mut Block, bref: ir::BlockRef) -> Option<Expr> {
    for i in self.ir.blocks[bref.0].instrs.range() {
      let r = ir::Ref::Instr(bref, i);
      let instr = self.ir.instr(r).unwrap();
      match instr.opcode {
        ir::Opcode::Nop => continue,
        ir::Opcode::Phi => continue, // handled by jmp
        ir::Opcode::Ret => {
          blk.push_stmt(Stmt::Return);
          return None;
        }
        ir::Opcode::Jmp => {
          let ir::Ref::Block(dst) = instr.operands[0] else { panic!("Expected block ref for jmp instr") };
          self.emit_phis(blk, bref, dst);
          return None;
        }
        ir::Opcode::Jne => {
          // TODO: Maybe verify that there are no phis in the target block? This should be gaurenteed by
          // the ir finalize, but it's probably good to do sanity checks
          let cond = self.ref_to_expr(instr.operands[0], 0);
          return Some(cond);
        }
        ir::Opcode::WriteVar16 => {
          let lhs = self.symbol_to_expr(instr.operands[0].unwrap_symbol());
          let rhs = self.ref_to_expr(instr.operands[1], 0);
          blk.push_stmt(Stmt::Assign(Assign { lhs, rhs }));
        }
        ir::Opcode::Store16 => {
          let seg = self.ref_to_expr(instr.operands[0], 1);
          let off = self.ref_to_expr(instr.operands[1], 1);
          let lhs = Expr::Deref(Box::new(Expr::Ptr16(Box::new(seg), Box::new(off))));
          let rhs = self.ref_to_expr(instr.operands[2], 0);
          blk.push_stmt(Stmt::Assign(Assign { lhs, rhs }));
        }
        _ => {
          if self.n_uses.get(&r).unwrap_or(&0) == &1 { continue; }

          let name = self.ref_name(r);
          let rvalue = self.ref_to_expr(r, 0);

          blk.push_stmt(Stmt::Assign(Assign {
            lhs: Expr::Name(name),
            rhs: rvalue,
          }));
        }
      }
    }
    unreachable!("IR Block Should End With A Branching Instr");
  }

  fn emit_jump(&mut self, blk: &mut Block, jump: control_flow::Jump, cond: Option<Expr>) {
    match jump {
      control_flow::Jump::None => (),
      control_flow::Jump::UncondFallthrough => (),
      control_flow::Jump::UncondTarget(tgt) => {
        let label = self.make_label(tgt);
        blk.push_stmt(Stmt::Goto(Goto { label }));
      }
      control_flow::Jump::CondTargetTrue(tgt) => {
        let cond = cond.unwrap();
        let label = self.make_label(tgt);
        let goto = Stmt::Goto(Goto{label});
        let then_body = Block(vec![goto]);
        blk.push_stmt(Stmt::If(If {cond, then_body }));
      }
      control_flow::Jump::CondTargetFalse(tgt) => {
        let cond = cond.unwrap();
        let label = self.make_label(tgt);
        let goto = Stmt::Goto(Goto{label});
        let then_body = Block(vec![goto]);
        let cond_inv = Expr::Unary(Box::new(UnaryExpr{op: UnaryOperator::Not, rhs: cond}));
        blk.push_stmt(Stmt::If(If {cond: cond_inv, then_body }));
      }
      control_flow::Jump::CondTargetBoth(tgt_true, tgt_false) => {
        let cond = cond.unwrap();
        let label_true = self.make_label(tgt_true);
        let label_false = self.make_label(tgt_false);
        blk.push_stmt(Stmt::CondGoto(CondGoto { cond, label_true, label_false }));
      }
    }
  }

  fn convert_basic_block(&mut self, blk: &mut Block, iter: &mut FlowIter, _depth: usize) {
    let Some(bb_elt) = iter.next() else { panic!("expected basic block element") };
    let Detail::BasicBlock(bb) = &bb_elt.elem.detail else { panic!("expected basic block element") };

    if bb.labeled {
      let label = self.make_label(bb_elt.id);
      blk.push_stmt(Stmt::Label(label));
    }

    let cond = self.emit_blk(blk, bb.blkref);
    self.emit_jump(blk, bb_elt.elem.jump.unwrap(), cond);
  }

  fn convert_loop(&mut self, blk: &mut Block, iter: &mut FlowIter, depth: usize) {
    let Some(loop_elt) = iter.next() else { panic!("expected loop element") };
    let Detail::Loop(_) = &loop_elt.elem.detail else { panic!("expected loop element") };

    let body = self.convert_body(iter, depth+1);
    blk.push_stmt(Stmt::Loop(Loop { body }));
    self.emit_jump(blk, loop_elt.elem.jump.unwrap(), None);
  }

  fn convert_ifstmt(&mut self, blk: &mut Block, iter: &mut FlowIter, depth: usize) {
    let Some(ifstmt_elt) = iter.next() else { panic!("expected ifstmt element") };
    let Detail::If(ifstmt) = &ifstmt_elt.elem.detail else { panic!("expected ifstmt element") };

    let Detail::BasicBlock(bb) = &self.cf.elem(ifstmt.entry).detail else { panic!("expected ifstmt entry to be a basic-block") };
    if bb.labeled {
      let label = self.make_label(ifstmt.entry);
      blk.push_stmt(Stmt::Label(label));
    }
    let Some(cond) = self.emit_blk(blk, bb.blkref) else {
      panic!("expected ifstmt entry to end in a conditional jump");
    };

    let then_body = self.convert_body(iter, depth+1);
    blk.push_stmt(Stmt::If(If { cond, then_body }));
    self.emit_jump(blk, ifstmt_elt.elem.jump.unwrap(), None);
  }

  fn convert_body(&mut self, iter: &mut FlowIter, depth: usize) -> Block {
    let mut blk = Block::default();

    while let Some(elt) = iter.peek() {
      assert!(elt.depth <= depth);
      if elt.depth < depth {
        break;
      }
      match &elt.elem.detail {
        Detail::BasicBlock(_) => self.convert_basic_block(&mut blk, iter, depth),
        Detail::Loop(_) => self.convert_loop(&mut blk, iter, depth),
        Detail::If(_) => self.convert_ifstmt(&mut blk, iter, depth),
      };
    }

    blk
  }

  fn build(&mut self, name: &str) -> Function {
    let mut iter = self.cf.iter().peekable();
    let body = self.convert_body(&mut iter, 0);
    assert!(iter.next().is_none());

    Function {
      name: name.to_string(),
      body,
    }
  }
}

impl Function {
  pub fn from_ir(name: &str, ir: &ir::IR, ctrlflow: &ControlFlow) -> Self {
    Builder::new(ir, ctrlflow).build(name)
  }
}
