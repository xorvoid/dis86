use crate::decomp::ir;
use crate::decomp::types::*;
use crate::decomp::control_flow::{self, ControlFlow, Detail, ElemId};
use std::collections::{HashMap, HashSet};

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
  Cast(Type, Box<Expr>),
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

#[derive(Debug, Clone, Copy)]
pub enum BinaryOperator {
  Add, Sub, Shl, Shr, Mul, Div, And, Or, Xor, Eq, Neq, Gt, Geq, Lt, Leq,
}

impl BinaryOperator {
  pub fn as_operator_str(&self) -> &'static str {
    match self {
      BinaryOperator::Add => "+",
      BinaryOperator::Sub => "-",
      BinaryOperator::Shl => "<<",
      BinaryOperator::Shr => ">>",
      BinaryOperator::Mul => "*",
      BinaryOperator::Div => "/",
      BinaryOperator::And => "&",
      BinaryOperator::Or  => "|",
      BinaryOperator::Xor => "^",
      BinaryOperator::Eq  => "==",
      BinaryOperator::Neq => "!=",
      BinaryOperator::Gt  => ">",
      BinaryOperator::Geq => ">=",
      BinaryOperator::Lt  => "<",
      BinaryOperator::Leq => "<=",
    }
  }

  fn invert(self) -> Option<Self> {
    match self {
      BinaryOperator::Eq => Some(BinaryOperator::Neq),
      BinaryOperator::Neq => Some(BinaryOperator::Eq),
      BinaryOperator::Gt => Some(BinaryOperator::Leq),
      BinaryOperator::Geq => Some(BinaryOperator::Lt),
      BinaryOperator::Lt => Some(BinaryOperator::Geq),
      BinaryOperator::Leq => Some(BinaryOperator::Gt),
      _ => None,
    }
  }
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
pub enum ReturnType {
  Far,
  Near,
}

#[derive(Debug)]
pub struct Return {
  pub rt: ReturnType,
  pub vals: Vec<Expr>,
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
  VarDecl(Type, String),
  Label(Label),
  Instr(ir::Ref),
  Expr(Expr),
  Assign(Assign),
  CondGoto(CondGoto),
  Goto(Goto),
  Return(Return),
  Loop(Loop),
  If(If),
}

#[derive(Debug)]
pub struct Function {
  pub name: String,
  pub ret: Option<Type>,
  pub decls: Block,
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
  assigned_names: HashSet<String>,
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
      assigned_names: HashSet::new(),
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

  // depth==0 instruction itself (must generate)
  // depth==1 operand of another instruction (may generate)
  fn ref_to_binary_expr(&mut self, r: ir::Ref, depth: usize, inverted: &mut bool) -> Option<Expr> {
    let instr = self.ir.instr(r).unwrap();

    let (mut ast_op, signed) = match instr.opcode {
      ir::Opcode::Add  => (BinaryOperator::Add,  false),
      ir::Opcode::Sub  => (BinaryOperator::Sub,  false),
      ir::Opcode::IMul => (BinaryOperator::Mul,  true),
      ir::Opcode::UMul => (BinaryOperator::Mul,  false),
      ir::Opcode::IDiv => (BinaryOperator::Div,  true),
      ir::Opcode::UDiv => (BinaryOperator::Div,  false),
      ir::Opcode::And  => (BinaryOperator::And,  false),
      ir::Opcode::Or   => (BinaryOperator::Or,   false),
      ir::Opcode::Xor  => (BinaryOperator::Xor,  false),
      ir::Opcode::Shl  => (BinaryOperator::Shl,  false),
      ir::Opcode::Shr  => (BinaryOperator::Shr,  true),
      ir::Opcode::UShr => (BinaryOperator::Shr,  false),
      ir::Opcode::Eq   => (BinaryOperator::Eq,   false),
      ir::Opcode::Neq  => (BinaryOperator::Neq,  false),
      ir::Opcode::Gt   => (BinaryOperator::Gt,   true),
      ir::Opcode::Geq  => (BinaryOperator::Geq,  true),
      ir::Opcode::Lt   => (BinaryOperator::Lt,   true),
      ir::Opcode::Leq  => (BinaryOperator::Leq,  true),
      ir::Opcode::UGt  => (BinaryOperator::Gt,   false),
      ir::Opcode::UGeq => (BinaryOperator::Geq,  false),
      ir::Opcode::ULt  => (BinaryOperator::Lt,   false),
      ir::Opcode::ULeq => (BinaryOperator::Leq,  false),
      _ => return None,
    };

    // Try to invert the operation if requested
    if *inverted {
      if let Some(op) = ast_op.invert() {
        *inverted = false;
        ast_op = op;
      }
    }

    let mut lhs = self.ref_to_expr(instr.operands[0], depth+1);
    let mut rhs = self.ref_to_expr(instr.operands[1], depth+1);

    if signed {
      lhs = Expr::Cast(Type::I16, Box::new(lhs));
      rhs = Expr::Cast(Type::I16, Box::new(rhs));
    }

    Some(Expr::Binary(Box::new(BinaryExpr {
      op: ast_op,
      lhs,
      rhs,
    })))
  }

  fn ref_to_expr(&mut self, r: ir::Ref, depth: usize) -> Expr {
    self.ref_to_expr_2(r, depth, false)
  }

  // FIXME: CLEANUP AND RENAME
  fn ref_to_expr_2(&mut self, r: ir::Ref, depth: usize, mut inverted: bool) -> Expr {
    let expr = self.ref_to_expr_3(r, depth, &mut inverted);
    let expr = if inverted {
      Expr::Unary(Box::new(UnaryExpr{op: UnaryOperator::Not, rhs: expr}))
    } else {
      expr
    };
    expr
  }

  // FIXME: CLEANUP AND RENAME
  fn ref_to_expr_3(&mut self, r: ir::Ref, depth: usize, inverted: &mut bool) -> Expr {
    match self.ir.lookup_const(r) {
      Some(k) => return Expr::Const(k as i64),
      None => (),
    }
    if let ir::Ref::Init(reg) = r {
      return Expr::Name(reg.info().name.to_string());
    }

    let instr = self.ir.instr(r).unwrap();
    if depth != 0 && (self.lookup_uses(r) != 1 || instr.opcode.is_call()) {
      let name = self.ref_name(r);
      return Expr::Name(name);
    }

    assert!(matches!(r, ir::Ref::Instr(_, _)));
    if let Some(expr) = self.ref_to_binary_expr(r, depth, inverted) {
      return expr;
    }

    match instr.opcode {
      ir::Opcode::Load16 => {
        let seg = self.ref_to_expr(instr.operands[0], depth+1);
        let off = self.ref_to_expr(instr.operands[1], depth+1);
        Expr::Deref(Box::new(Expr::Abstract("PTR_16", vec![seg, off])))
      }
      ir::Opcode::Load32 => {
        let seg = self.ref_to_expr(instr.operands[0], depth+1);
        let off = self.ref_to_expr(instr.operands[1], depth+1);
        Expr::Deref(Box::new(Expr::Abstract("PTR_32", vec![seg, off])))
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
      ir::Opcode::CallFar => {
        let exprs: Vec<_> = instr.operands.iter().map(|r| self.ref_to_expr(*r, depth+1)).collect();
        Expr::Abstract("CALL_FAR", exprs)
      }
      ir::Opcode::CallNear => {
        let exprs: Vec<_> = instr.operands.iter().map(|r| self.ref_to_expr(*r, depth+1)).collect();
        Expr::Abstract("CALL_NEAR", exprs)
      }
      ir::Opcode::CallPtr => {
        let exprs: Vec<_> = instr.operands.iter().map(|r| self.ref_to_expr(*r, depth+1)).collect();
        Expr::Abstract("CALL_FAR_INDIRECT", exprs)
      }
      ir::Opcode::Phi => {
        // generally handled by jmp, but other expressions that use a phi might end up here
        // so, we can simply return our refname
        Expr::Name(self.ref_name(r))
      }
      ir::Opcode::Make32 => {
        let exprs: Vec<_> = instr.operands.iter().map(|r| self.ref_to_expr(*r, depth+1)).collect();
        Expr::Abstract("MAKE_32", exprs)
      }
      ir::Opcode::SignExtTo32 => {
        assert!(instr.operands.len() == 1);
        let rhs = self.ref_to_expr(instr.operands[0], depth+1);
        // TODO: VERIFY THIS IS A U16
        Expr::Cast(Type::I32, Box::new(Expr::Cast(Type::I16, Box::new(rhs))))
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
      expr = Expr::Cast(Type::ptr(Type::U16), Box::new(expr));
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
      self.assigned_names.insert(name.clone());
      blk.push_stmt(Stmt::Assign(Assign {
        lhs: Expr::Name(name),
        rhs: rvalue,
      }));
    }
  }

  // Returns a jump condition expr if the block ends in a conditional branch
  #[must_use]
  fn emit_blk(&mut self, blk: &mut Block, bref: ir::BlockRef, inverted_cond: bool) -> Option<Expr> {
    for i in self.ir.blocks[bref.0].instrs.range() {
      let r = ir::Ref::Instr(bref, i);
      let instr = self.ir.instr(r).unwrap();
      match instr.opcode {
        ir::Opcode::Nop => continue,
        ir::Opcode::Phi => continue, // handled by jmp
        ir::Opcode::Pin => continue, // ignored
        ir::Opcode::RetFar => {
          let vals: Vec<_> = instr.operands.iter().map(|r| self.ref_to_expr(*r, 1)).collect();
          blk.push_stmt(Stmt::Return(Return{rt: ReturnType::Far, vals}));
          return None;
        }
        ir::Opcode::RetNear => {
          let vals: Vec<_> = instr.operands.iter().map(|r| self.ref_to_expr(*r, 1)).collect();
          blk.push_stmt(Stmt::Return(Return{rt: ReturnType::Near, vals}));
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
          let cond = self.ref_to_expr_2(instr.operands[0], 1, inverted_cond);
          return Some(cond);
        }
        ir::Opcode::WriteVar16 => {
          let lhs = self.symbol_to_expr(instr.operands[0].unwrap_symbol());
          let rhs = self.ref_to_expr(instr.operands[1], 1);
          blk.push_stmt(Stmt::Assign(Assign { lhs, rhs }));
        }
        ir::Opcode::Store16 => {
          let seg = self.ref_to_expr(instr.operands[0], 1);
          let off = self.ref_to_expr(instr.operands[1], 1);
          let lhs = Expr::Deref(Box::new(Expr::Abstract("PTR_16", vec![seg, off])));
          let rhs = self.ref_to_expr(instr.operands[2], 1);
          blk.push_stmt(Stmt::Assign(Assign { lhs, rhs }));
        }
        _ => {
          let uses = self.n_uses.get(&r).cloned().unwrap_or(0);
          if uses >= 2 || instr.opcode.is_call() {
            let rvalue = self.ref_to_expr(r, 0);
            if uses > 0 {
              let name = self.ref_name(r);
              self.assigned_names.insert(name.clone());
              blk.push_stmt(Stmt::Assign(Assign {
                lhs: Expr::Name(name),
                rhs: rvalue,
              }));
            } else {
              blk.push_stmt(Stmt::Expr(rvalue));
            }
          }
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
        // NOTE: cond already inverted befroe call!
        //let cond_inv = Expr::Unary(Box::new(UnaryExpr{op: UnaryOperator::Not, rhs: cond}));
        blk.push_stmt(Stmt::If(If {cond: cond, then_body }));
      }
      control_flow::Jump::CondTargetBoth(tgt_true, tgt_false) => {
        let cond = cond.unwrap();
        let label_true = self.make_label(tgt_true);
        let label_false = self.make_label(tgt_false);
        blk.push_stmt(Stmt::CondGoto(CondGoto { cond, label_true, label_false }));
      }
      control_flow::Jump::Table(_) => panic!("Jump::Table Unimpl"),
    }
  }

  fn convert_basic_block(&mut self, blk: &mut Block, iter: &mut FlowIter, _depth: usize) {
    let Some(bb_elt) = iter.next() else { panic!("expected basic block element") };
    let Detail::BasicBlock(bb) = &bb_elt.elem.detail else { panic!("expected basic block element") };

    if bb.labeled {
      let label = self.make_label(bb_elt.id);
      blk.push_stmt(Stmt::Label(label));
    }

    let jump = bb_elt.elem.jump.clone().unwrap();
    let cond = self.emit_blk(blk, bb.blkref, jump.cond_inverted());
    self.emit_jump(blk, jump, cond);
  }

  fn convert_loop(&mut self, blk: &mut Block, iter: &mut FlowIter, depth: usize) {
    let Some(loop_elt) = iter.next() else { panic!("expected loop element") };
    let Detail::Loop(_) = &loop_elt.elem.detail else { panic!("expected loop element") };

    let body = self.convert_body(iter, depth+1);
    blk.push_stmt(Stmt::Loop(Loop { body }));
    self.emit_jump(blk, loop_elt.elem.jump.clone().unwrap(), None);
  }

  fn convert_ifstmt(&mut self, blk: &mut Block, iter: &mut FlowIter, depth: usize) {
    let Some(ifstmt_elt) = iter.next() else { panic!("expected ifstmt element") };
    let Detail::If(ifstmt) = &ifstmt_elt.elem.detail else { panic!("expected ifstmt element") };

    let Detail::BasicBlock(bb) = &self.cf.elem(ifstmt.entry).detail else { panic!("expected ifstmt entry to be a basic-block") };
    if bb.labeled {
      let label = self.make_label(ifstmt.entry);
      blk.push_stmt(Stmt::Label(label));
    }
    let Some(cond) = self.emit_blk(blk, bb.blkref, ifstmt.inverted) else {
      panic!("expected ifstmt entry to end in a conditional jump");
    };

    let then_body = self.convert_body(iter, depth+1);
    blk.push_stmt(Stmt::If(If { cond, then_body }));
    self.emit_jump(blk, ifstmt_elt.elem.jump.clone().unwrap(), None);
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

  fn build(&mut self, name: &str, ret: Option<Type>) -> Function {
    let mut iter = self.cf.iter().peekable();
    let body = self.convert_body(&mut iter, 0);
    assert!(iter.next().is_none());

    let mut decls = Block(vec![]);
    let mut names: Vec<_> = self.assigned_names.iter().cloned().collect();
    names.sort();
    for name in names {
      decls.0.push(Stmt::VarDecl(Type::U16, name));
    }

    Function {
      name: name.to_string(),
      ret,
      decls,
      body,
    }
  }
}

impl Function {
  pub fn from_ir(name: &str, ret: Option<Type>, ir: &ir::IR, ctrlflow: &ControlFlow) -> Self {
    Builder::new(ir, ctrlflow).build(name, ret)
  }
}
