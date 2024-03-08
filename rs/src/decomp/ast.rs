use crate::decomp::ir;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Write;

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
  Addr,
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
  pub tgt_true: Label,
  pub tgt_false: Label,
}

#[derive(Debug)]
pub struct Goto {
  pub tgt: Label,
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
}

#[derive(Debug)]
pub struct Function {
  pub name: String,
  //vars: // todo
  pub body: Vec<Stmt>,
}

enum Next {
  Return,
  Fallthrough(ir::BlockRef), /* passive */
  Branch(ir::BlockRef /* active */, ir::BlockRef /* passive */),
}

struct Builder<'a> {
  ir: &'a ir::IR,
  func: Function,
  n_uses: HashMap<ir::Ref, usize>,
  temp_names: HashMap<ir::Ref, String>,
  temp_count: usize,
}

impl<'a> Builder<'a> {
  fn new(name: &str, ir: &'a ir::IR) -> Self {
    Self {
      ir,
      func: Function {
        name: name.to_string(),
        body: vec![],
      },
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

  fn compute_uses(&mut self) {
    for b in 0..self.ir.blocks.len() {
      for i in self.ir.blocks[b].instrs.range() {
        let r = ir::Ref::Instr(ir::BlockRef(b), i);
        let instr = self.ir.instr(r).unwrap();
        for oper in &instr.operands {
          *self.n_uses.entry(*oper).or_insert(0) += 1;
        }
      }
    }
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
    Label(self.ir.blocks[bref.0].name.clone())
  }

  fn convert_blk(&mut self, bref: ir::BlockRef) -> Next {
    let blk = &self.ir.blocks[bref.0];
    if bref.0 != 0 { // for all except the entry block
      let label = self.blockref_to_label(bref);
      self.func.body.push(Stmt::Label(label));
    }

    for i in blk.instrs.range() {
      let r = ir::Ref::Instr(bref, i);
      let instr = self.ir.instr(r).unwrap();
      match instr.opcode {
        ir::Opcode::Nop => continue,
        ir::Opcode::Ret => {
          self.func.body.push(Stmt::Return);
          return Next::Return;
        }
        ir::Opcode::Jmp => {
          // TODO: Handle phis!!
          let passive = instr.operands[0].unwrap_block();
          let tgt = self.blockref_to_label(passive);
          self.func.body.push(Stmt::Goto(Goto {
            tgt,
          }));
          return Next::Fallthrough(passive);
        }
        ir::Opcode::Jne => {
          // TODO: Handle phis!!
          let active = instr.operands[1].unwrap_block();
          let passive = instr.operands[2].unwrap_block();
          let tgt_true = self.blockref_to_label(active);
          let tgt_false = self.blockref_to_label(passive);
          let s = Stmt::CondGoto(CondGoto {
            cond: self.ref_to_expr(instr.operands[0], 0),
            tgt_true,
            tgt_false,
          });
          self.func.body.push(s);
          return Next::Branch(active, passive);
        }
        ir::Opcode::WriteVar16 => {
          let lhs = self.symbol_to_expr(instr.operands[0].unwrap_symbol());
          let rhs = self.ref_to_expr(instr.operands[1], 0);
          self.func.body.push(Stmt::Assign(Assign { lhs, rhs }));
        }
        ir::Opcode::Store16 => {
          let seg = self.ref_to_expr(instr.operands[0], 1);
          let off = self.ref_to_expr(instr.operands[1], 1);
          let lhs = Expr::Deref(Box::new(Expr::Ptr16(Box::new(seg), Box::new(off))));
          let rhs = self.ref_to_expr(instr.operands[2], 0);
          self.func.body.push(Stmt::Assign(Assign { lhs, rhs }));
        }
        _ => {
          if self.n_uses.get(&r).unwrap_or(&0) == &1 { continue; }

          let name = self.ref_name(r);
          let rvalue = self.ref_to_expr(r, 0);

          self.func.body.push(Stmt::Assign(Assign {
            lhs: Expr::Name(name),
            rhs: rvalue,
          }));
        }
      }
    }
    unreachable!("IR Block Should End With A Branching Instr");
  }

  fn build(&mut self) {
    self.compute_uses();
    // let mut converted = HashSet::new();
    // let mut queue = VecDeque::new();
    // queue.push_back(ir::BlockRef(0));

    // while queue.len() > 0 {
    //   let bref = queue.pop_front().unwrap();
    //   if converted.get(&bref).is_some() {
    //     continue;
    //   }
    //   converted.insert(bref);
    //   let next = self.convert_blk(bref);
    //   match next {
    //     Next::Return => continue,
    //     Next::Fallthrough(passive) => queue.push_back(passive),
    //     Next::Branch(active, passive) => {
    //       queue.push_back(active);
    //       queue.push_back(passive);
    //     }
    //   }
    // }

    for b in 0..self.ir.blocks.len() {
      self.convert_blk(ir::BlockRef(b));
    }
  }
}

impl Function {
  pub fn from_ir(name: &str, ir: &ir::IR) -> Self {
    let mut bld = Builder::new(name, ir);
    bld.build();

    let s = display_ir_with_uses(ir, &bld.n_uses).unwrap();
    println!("{}", s);

    bld.func
  }
}


fn display_ir_with_uses(ir: &ir::IR, n_uses: &HashMap<ir::Ref, usize>) -> Result<String, std::fmt::Error> {
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
