use crate::decomp::ir;
use crate::decomp::control_flow::{self, ControlFlow, Detail, ElemId};
use std::collections::{HashMap, HashSet};
use std::fmt::Write;

type FlowIter<'a> = std::iter::Peekable<control_flow::ControlFlowIter<'a>>;

#[derive(Debug)]
pub enum Expr {
  None,
  Unary(Box<UnaryExpr>),
  Binary(Box<BinaryExpr>),
  Const(i64),
  Name(String),
  Call(Box<Expr>, Vec<Expr>),
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
  Add, Sub, And, Or, Xor, Shl, Shr, Eq, Neq, Gt, Geq, Lt, Leq,
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
  None,
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

enum Next {
  None,
  Return,
  UncondJump,
  CondJump(Expr),
}

struct Builder<'a> {
  ir: &'a ir::IR,
  cf: &'a ControlFlow,
  blkstack: Vec<Block>,
  n_uses: HashMap<ir::Ref, usize>,
  temp_names: HashMap<ir::Ref, String>,
  temp_count: usize,
}

impl<'a> Builder<'a> {
  fn new(ir: &'a ir::IR, cf: &'a ControlFlow) -> Self {
    Self {
      ir,
      cf,
      blkstack: vec![],
      n_uses: HashMap::new(),
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
      ir::Opcode::Phi => Expr::UnimplPhi,
      ir::Opcode::Pin => Expr::UnimplPin,
      _ => {
        Expr::None
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

  fn blockref_to_label(&self, bref: ir::BlockRef) -> Label {
    Label(format!("{}_b{}", self.ir.blocks[bref.0].name, bref.0))
  }

  fn push_stmt(&mut self, stmt: Stmt) {
    self.blkstack.as_mut_slice().last_mut().unwrap().0.push(stmt);
  }

  fn block_enter(&mut self) {
    self.blkstack.push(Block::default());
  }

  #[must_use]
  fn block_leave(&mut self) -> Block {
    self.blkstack.pop().unwrap()
  }

  #[must_use]
  fn convert_blk(&mut self, bref: ir::BlockRef) -> Next {
    let blk = &self.ir.blocks[bref.0];
    for i in blk.instrs.range() {
      let r = ir::Ref::Instr(bref, i);
      let instr = self.ir.instr(r).unwrap();
      match instr.opcode {
        ir::Opcode::Nop => continue,
        ir::Opcode::Ret => {
          self.push_stmt(Stmt::Return);
          return Next::Return;
        }
        ir::Opcode::Jmp => {
          // TODO: Handle phis!!
          return Next::UncondJump;
        }
        ir::Opcode::Jne => {
          // TODO: Handle phis!!
          let cond = self.ref_to_expr(instr.operands[0], 0);
          return Next::CondJump(cond); //, tgt_true, tgt_false);
        }
        ir::Opcode::WriteVar16 => {
          let lhs = self.symbol_to_expr(instr.operands[0].unwrap_symbol());
          let rhs = self.ref_to_expr(instr.operands[1], 0);
          self.push_stmt(Stmt::Assign(Assign { lhs, rhs }));
        }
        ir::Opcode::Store16 => {
          let seg = self.ref_to_expr(instr.operands[0], 1);
          let off = self.ref_to_expr(instr.operands[1], 1);
          let lhs = Expr::Deref(Box::new(Expr::Ptr16(Box::new(seg), Box::new(off))));
          let rhs = self.ref_to_expr(instr.operands[2], 0);
          self.push_stmt(Stmt::Assign(Assign { lhs, rhs }));
        }
        _ => {
          if self.n_uses.get(&r).unwrap_or(&0) == &1 { continue; }

          let name = self.ref_name(r);
          let rvalue = self.ref_to_expr(r, 0);

          self.push_stmt(Stmt::Assign(Assign {
            lhs: Expr::Name(name),
            rhs: rvalue,
          }));
        }
      }
    }
    unreachable!("IR Block Should End With A Branching Instr");
  }




  ////////////////////////////////////////////////////////////////////////////////////
  // NEW

  fn elemid_to_label(&self, id: ElemId) -> Label {
    let elem = self.cf.elem(id);
    let Detail::BasicBlock(bb) = &elem.detail else { panic!("Expected basic block") };
    self.blockref_to_label(bb.blkref)
  }

  fn generate_jump(&mut self, jump: control_flow::Jump, cond: Option<Expr>) {
    match jump {
      control_flow::Jump::None => (),
      control_flow::Jump::UncondFallthrough => (),
      control_flow::Jump::UncondTarget(tgt) => {
        let label = self.elemid_to_label(tgt);
        self.push_stmt(Stmt::Goto(Goto { label }));
      }
      control_flow::Jump::CondTargetTrue(tgt) => {
        let cond = cond.unwrap();
        let label = self.elemid_to_label(tgt);
        let goto = Stmt::Goto(Goto{label});
        let then_body = Block(vec![goto]);
        self.push_stmt(Stmt::If(If {cond, then_body }));
      }
      control_flow::Jump::CondTargetFalse(tgt) => {
        let cond = cond.unwrap();
        let label = self.elemid_to_label(tgt);
        let goto = Stmt::Goto(Goto{label});
        let then_body = Block(vec![goto]);
        let cond_inv = Expr::Unary(Box::new(UnaryExpr{op: UnaryOperator::Not, rhs: cond}));
        self.push_stmt(Stmt::If(If {cond: cond_inv, then_body }));
      }
      control_flow::Jump::CondTargetBoth(tgt_true, tgt_false) => {
        let cond = cond.unwrap();
        let label_true = self.elemid_to_label(tgt_true);
        let label_false = self.elemid_to_label(tgt_false);
        self.push_stmt(Stmt::CondGoto(CondGoto { cond, label_true, label_false }));
      }
    }
  }

  fn convert_basic_block(&mut self, iter: &mut FlowIter, depth: usize) {
    let Some(bb_elt) = iter.next() else { panic!("expected basic block element") };
    let Detail::BasicBlock(bb) = &bb_elt.elem.detail else { panic!("expected basic block element") };

    if bb.labeled {
      let label = self.elemid_to_label(bb_elt.id);
      self.push_stmt(Stmt::Label(label));
    }

    let next = self.convert_blk(bb.blkref);
    let cond = match next {
      Next::CondJump(cond) => Some(cond),
      _ => None,
    };
    self.generate_jump(bb_elt.elem.jump.unwrap(), cond);
  }

  fn convert_loop(&mut self, iter: &mut FlowIter, depth: usize) {
    let Some(loop_elt) = iter.next() else { panic!("expected loop element") };
    let Detail::Loop(lp) = &loop_elt.elem.detail else { panic!("expected loop element") };

    let body = self.convert_body(iter, depth+1);
    self.push_stmt(Stmt::Loop(Loop { body }));
    self.generate_jump(loop_elt.elem.jump.unwrap(), None);
  }

  fn convert_ifstmt(&mut self, iter: &mut FlowIter, depth: usize) {
    let Some(ifstmt_elt) = iter.next() else { panic!("expected ifstmt element") };
    let Detail::If(ifstmt) = &ifstmt_elt.elem.detail else { panic!("expected ifstmt element") };

    let Detail::BasicBlock(bb) = &self.cf.elem(ifstmt.entry).detail else { panic!("expected ifstmt entry to be a basic-block") };
    if bb.labeled {
      let label = self.elemid_to_label(ifstmt.entry);
      self.push_stmt(Stmt::Label(label));
    }
    let n = self.convert_blk(bb.blkref);
    let Next::CondJump(cond) = n else { panic!("expected ifstmt entry to end in a conditional jump") };

    let then_body = self.convert_body(iter, depth+1);
    self.push_stmt(Stmt::If(If { cond, then_body }));
    self.generate_jump(ifstmt_elt.elem.jump.unwrap(), None);
  }

  fn convert_body(&mut self, iter: &mut FlowIter, depth: usize) -> Block {
    self.block_enter();

    while let Some(elt) = iter.peek() {
      assert!(elt.depth <= depth);
      if elt.depth < depth {
        break;
      }
      match &elt.elem.detail {
        Detail::BasicBlock(_) => self.convert_basic_block(iter, depth),
        Detail::Loop(_) => self.convert_loop(iter, depth),
        Detail::If(_) => self.convert_ifstmt(iter, depth),
      };
    }

    self.block_leave()
  }

  fn build(&mut self, name: &str) -> Function {
    self.n_uses = compute_uses(self.ir);

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
  pub fn from_ir(name: &str, ir: &ir::IR) -> Self {
    //let s = display_ir_with_uses(ir).unwrap();
    //println!("{}", s);

    let ctrlflow = ControlFlow::from_ir(&ir);
    //println!("+=======================");
    //control_flow::print(&ctrlflow);

    Builder::new(ir, &ctrlflow).build(name)
  }
}


fn compute_uses(ir: &ir::IR) -> HashMap<ir::Ref, usize> {
  let mut n_uses = HashMap::new();
  for b in 0..ir.blocks.len() {
    for i in ir.blocks[b].instrs.range() {
      let r = ir::Ref::Instr(ir::BlockRef(b), i);
      let instr = ir.instr(r).unwrap();
      for oper in &instr.operands {
        *n_uses.entry(*oper).or_insert(0) += 1;
      }
    }
  }
  n_uses
}

fn display_ir_with_uses(ir: &ir::IR) -> Result<String, std::fmt::Error> {
  let n_uses = compute_uses(ir);
  let mut r = crate::decomp::ir::display::Formatter::new();
  for (i, blk) in ir.blocks.iter().enumerate() {
    let bref = ir::BlockRef(i);
    r.fmt_blkhdr(bref, blk)?;
    for idx in blk.instrs.range() {
      let iref = ir::Ref::Instr(bref, idx);
      let instr = &blk.instrs[idx];
      if instr.opcode == ir::Opcode::Nop { continue; }

      let n = n_uses.get(&iref).unwrap_or(&0);
      write!(&mut r.out, "{} | ", n)?;
      r.fmt_instr(ir, iref, instr)?;
    }
  }
  Ok(r.finish())
}
