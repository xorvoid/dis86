use crate::ir;
use crate::types::*;
use crate::config::Config;
use crate::control_flow::{self, ControlFlow, Detail, ElemId};
use std::collections::{HashMap, HashSet};

const OPT_DEFINE_TEMPS_AT_USE: bool = false;

type FlowIter<'a> = std::iter::Peekable<control_flow::ControlFlowIter<'a>>;

#[derive(Debug, Clone)]
pub struct VarDecl {
  pub typ: Type,
  pub names: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct VarMap {
  pub typ: Type,
  pub name: String,
  pub mapping_expr: Expr,
}

#[derive(Debug, Clone)]
pub enum Expr {
  Unary(Box<UnaryExpr>),
  Binary(Box<BinaryExpr>),
  HexConst(u16),
  DecimalConst(i16),
  Name(String),
  Call(Box<Expr>, Vec<Expr>),
  Abstract(&'static str, Vec<Expr>),
  ArrayAccess(Box<Expr>, Box<Expr>),
  StructAccess(Box<Expr>, Box<Expr>),
  Deref(Box<Expr>),
  Cast(Type, Box<Expr>),
  UnimplPhi,
  UnimplPin,
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
  Addr, Neg, LogicalNot, BitwiseNot,
}

impl UnaryOperator {
  pub fn as_operator_str(&self) -> &'static str {
    match self {
      UnaryOperator::Addr => "(u8*)&",
      UnaryOperator::Neg => "-",
      UnaryOperator::LogicalNot => "!",
      UnaryOperator::BitwiseNot => "~",
    }
  }
}

#[derive(Debug, Clone)]
pub struct UnaryExpr {
  pub op: UnaryOperator,
  pub rhs: Expr,
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOperator {
  Add, Sub, Shl, Shr, Mul, Div, Mod, And, Or, Xor, Eq, Neq, Gt, Geq, Lt, Leq,
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
      BinaryOperator::Mod => "%",
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

#[derive(Debug, Clone)]
pub struct BinaryExpr {
  pub op: BinaryOperator,
  pub lhs: Expr,
  pub rhs: Expr,
}

#[derive(Debug, Clone)]
pub struct Assign {
  pub decltype: Option<Type>,
  pub lhs: Expr,
  pub rhs: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Label(pub String); // fixme??

#[derive(Debug, Clone)]
pub struct CondGoto {
  pub cond: Expr,
  pub label_true: Label,
  pub label_false: Label,
}

#[derive(Debug, Clone)]
pub struct Goto {
  pub label: Label,
}

#[derive(Debug, Clone)]
pub enum ReturnType {
  Far,
  Near,
}

#[derive(Debug, Clone)]
pub struct Return {
  pub rt: ReturnType,
  pub vals: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct Loop {
  pub body: Block,
}

#[derive(Debug, Clone)]
pub struct If {
  pub cond: Expr,
  pub then_body: Block,
}

#[derive(Debug, Clone)]
pub struct Switch {
  pub switch_val: Expr,
  pub cases: Vec<SwitchCase>,
  pub default: Option<Block>,
}

#[derive(Debug, Clone)]
pub struct SwitchCase {
  pub cases: Vec<Expr>,
  pub body: Block,
}

#[derive(Debug, Clone)]
pub enum Stmt {
  Label(Label),
  Instr(ir::Ref),
  Expr(Expr),
  Assign(Assign),
  CondGoto(CondGoto),
  Goto(Goto),
  Return(Return),
  Loop(Loop),
  If(If),
  Switch(Switch),
  Unreachable,
}

#[derive(Debug, Clone)]
pub struct Function {
  pub name: String,
  pub ret: Option<Type>,
  pub vardecls: Vec<VarDecl>,
  pub varmaps: Vec<VarMap>,
  pub body: Block,
}

#[derive(Debug, Default, Clone)]
pub struct Block(pub Vec<Stmt>);

struct Builder<'a> {
  cfg: &'a Config,
  ir: &'a ir::IR,
  cf: &'a ControlFlow,
  n_uses: HashMap<ir::Ref, usize>,
  temp_names: HashMap<ir::Ref, String>,
  temp_count: usize,

  assigns: Vec<(String, Type)>,
  assigned: HashSet<String>,
  mappings: HashMap<String, (Type, Expr)>,
}

fn unary_expr(op: UnaryOperator, rhs: Expr) -> Expr {
  Expr::Unary(Box::new(UnaryExpr { op, rhs }))
}

fn binary_expr(op: BinaryOperator, lhs: Expr, rhs: Expr) -> Expr {
  Expr::Binary(Box::new(BinaryExpr { op, lhs, rhs }))
}

impl Block {
  fn push_stmt(&mut self, stmt: Stmt) {
    self.0.push(stmt);
  }
}

impl<'a> Builder<'a> {
  fn new(cfg: &'a Config, ir: &'a ir::IR, cf: &'a ControlFlow) -> Self {
    let n_uses = ir::util::compute_uses(ir);
    Self {
      cfg,
      ir,
      cf,
      n_uses,
      temp_names: HashMap::new(),
      temp_count: 0,

      assigns: vec![],
      assigned: HashSet::new(),
      mappings: HashMap::new(),
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

  fn ref_to_unary_expr(&mut self, r: ir::Ref, depth: usize, hex_const: bool, _inverted: &mut bool) -> Option<Expr> {
    let instr = self.ir.instr(r).unwrap();

    let (ast_op, signed) = match instr.opcode {
      ir::Opcode::Neg  => (UnaryOperator::Neg, false),
      ir::Opcode::Not  => (UnaryOperator::BitwiseNot, false),
      _ => return None,
    };

    // TODO: IMPLEMENT INVERTED FOR UNARY EXPR

    let mut rhs = self.ref_to_expr_hex(instr.operands[0], depth+1, hex_const);

    if signed {
      rhs = Expr::Cast(Type::I16, Box::new(rhs));
    }

    Some(unary_expr(ast_op, rhs))
  }

  fn ref_to_binary_expr(&mut self, r: ir::Ref, depth: usize, hex_const: bool, inverted: &mut bool) -> Option<Expr> {
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

    let mut lhs = self.ref_to_expr_hex(instr.operands[0], depth+1, hex_const);
    let mut rhs = self.ref_to_expr_hex(instr.operands[1], depth+1, hex_const);

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

  fn ref_to_expr_hex(&mut self, r: ir::Ref, depth: usize, hex_const: bool) -> Expr {
    let mut inverted = false;
    self.ref_to_expr_impl(r, depth, hex_const, &mut inverted)
  }

  // FIXME: CLEANUP AND RENAME
  fn ref_to_expr_2(&mut self, r: ir::Ref, depth: usize, mut inverted: bool) -> Expr {
    let expr = self.ref_to_expr_impl(r, depth, false, &mut inverted);
    let expr = if inverted {
      Expr::Unary(Box::new(UnaryExpr{op: UnaryOperator::LogicalNot, rhs: expr}))
    } else {
      expr
    };
    expr
  }

  // depth==0 instruction itself (must generate)
  // depth==1 operand of another instruction (may generate)
  // FIXME: CLEANUP AND RENAME
  fn ref_to_expr_impl(&mut self, r: ir::Ref, depth: usize, hex_const: bool, inverted: &mut bool) -> Expr {
    match self.ir.lookup_const(r) {
      Some(k) => {
        if hex_const || k >= 256 || k <= -256 {
          return Expr::HexConst(k as u16);
        } else {
          return Expr::DecimalConst(k as i16);
        }
      }
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
    if let Some(expr) = self.ref_to_unary_expr(r, depth, hex_const, inverted) {
      return expr;
    }
    if let Some(expr) = self.ref_to_binary_expr(r, depth, hex_const, inverted) {
      return expr;
    }

    match instr.opcode {
      ir::Opcode::Ref => {
        self.ref_to_expr(instr.operands[0], depth+1)
      }
      ir::Opcode::Load16 => {
        let seg = self.ref_to_expr_hex(instr.operands[0], depth+1, true);
        let off = self.ref_to_expr_hex(instr.operands[1], depth+1, true);
        Expr::Deref(Box::new(Expr::Abstract("PTR_16", vec![seg, off])))
      }
      ir::Opcode::Load32 => {
        let seg = self.ref_to_expr_hex(instr.operands[0], depth+1, true);
        let off = self.ref_to_expr_hex(instr.operands[1], depth+1, true);
        Expr::Deref(Box::new(Expr::Abstract("PTR_32", vec![seg, off])))
      }
      ir::Opcode::Upper16 => {
        let lhs = self.ref_to_expr_hex(instr.operands[0], depth+1, hex_const);
        Expr::Cast(Type::U16, Box::new(Expr::Binary(Box::new(BinaryExpr {
          op: BinaryOperator::Shr,
          lhs,
          rhs: Expr::DecimalConst(16),
        }))))
      }
      ir::Opcode::Lower16 => {
        let lhs = self.ref_to_expr_hex(instr.operands[0], depth+1, hex_const);
        Expr::Cast(Type::U16, Box::new(lhs))
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
        let exprs: Vec<_> = instr.operands.iter().map(|r| self.ref_to_expr_hex(*r, depth+1, true)).collect();
        Expr::Abstract("CALL_FAR", exprs)
      }
      ir::Opcode::CallNear => {
        let exprs: Vec<_> = instr.operands.iter().map(|r| self.ref_to_expr_hex(*r, depth+1, true)).collect();
        Expr::Abstract("CALL_NEAR", exprs)
      }
      ir::Opcode::CallPtr => {
        let exprs: Vec<_> = instr.operands.iter().map(|r| self.ref_to_expr_hex(*r, depth+1, true)).collect();
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
      ir::Opcode::Sign => {
        let lhs = self.ref_to_expr(instr.operands[0], depth+1);
        binary_expr(BinaryOperator::Neq,
                    binary_expr(BinaryOperator::Shr, lhs, Expr::DecimalConst(15)),
                    Expr::DecimalConst(0))
      }
      ir::Opcode::NotSign => {
        let lhs = self.ref_to_expr(instr.operands[0], depth+1);
        binary_expr(BinaryOperator::Eq,
                    binary_expr(BinaryOperator::Shr, lhs, Expr::DecimalConst(15)),
                    Expr::DecimalConst(0))
      }
      ir::Opcode::Unimpl => {
        let exprs: Vec<_> = instr.operands.iter().map(|r| self.ref_to_expr(*r, depth+1)).collect();
        Expr::Abstract("UNIMPL", exprs)
      }
      opcode @ _ => {
        panic!("Unimplemented {:?} in ast converter", opcode);
      }
    }
  }

  fn symbol_to_expr(&mut self, symref: ir::sym::SymbolRef) -> Expr {
    let sym = symref.def(&self.ir.symbols);
    if (symref.table() == ir::sym::Table::Local || symref.table() == ir::sym::Table::Param) &&
      self.mappings.get(&sym.name).is_none()
    {
      let ss = crate::asm::instr::Reg::SS;
      let sp = crate::asm::instr::Reg::SP;
      let seg = Expr::Name(ss.info().name.to_string());
      let off = Expr::Binary(Box::new(BinaryExpr {
        op: BinaryOperator::Add,
        lhs: Expr::Name(sp.info().name.to_string()),
        rhs: Expr::HexConst(sym.off as u16),
      }));

      // self.decls.push(VarDecl {
      //   typ: symref.to_type(),
      //   names: vec![sym.name.clone()],
      //   mem_mapping: Some(Expr::Deref(Box::new(Expr::Abstract("PTR_16", vec![seg, off])))),
      // })

      let typ = symref.get_type(&self.ir.symbols);
      let ptr_sz = match typ {
        Type::U16 => "PTR_16",
        Type::U32 => "PTR_32",
        _ => panic!("Unsupported type: {:?}", typ),
      };

      let impl_expr = Expr::Deref(Box::new(Expr::Abstract(ptr_sz, vec![seg, off])));
      let typ = symref.get_type(&self.ir.symbols).clone();
      self.mappings.insert(sym.name.clone(), (typ, impl_expr));
    }

    let expr = Expr::Name(sym.name.clone());
    let typ = symref.get_type(&self.ir.symbols);
    self.symbol_to_expr_recurse(expr, typ, symref.access_region)
  }

  fn symbol_to_expr_recurse(&self, mut expr: Expr, typ: &Type, mut access: ir::sym::Region) -> Expr {
    if !typ.is_primitive() {
      match typ {
        Type::Array(basetype, len) => {
          let ArraySize::Known(len) = len else { panic!("Expected datatype to have known array length") };
          let basetype_sz = basetype.size_in_bytes().unwrap();
          let idx = access.off as usize / basetype_sz;
          if idx > *len { panic!("Access out of range"); }
          if access.sz as usize > basetype_sz { panic!("Access exceeds basetype size"); }

          let expr = Expr::ArrayAccess(
            Box::new(expr),
            Box::new(Expr::DecimalConst(idx as i16)));

          access.off -= (idx * basetype_sz) as i32;

          // recurse
          return self.symbol_to_expr_recurse(expr, basetype, access);
        }
        Type::Struct(struct_ref) => {
          let access_start = access.off as usize;
          let access_end = access_start + access.sz as usize;
          let s = self.cfg.type_builder.lookup_struct(*struct_ref).unwrap();
          for mbr in &s.members {
            let mbr_start = mbr.off as usize;
            let mbr_end = mbr_start + mbr.typ.size_in_bytes().unwrap();
            if !(mbr_start <= access_start && access_end <= mbr_end) { continue; }

            // Found!!
            let expr = Expr::StructAccess(
              Box::new(expr),
              Box::new(Expr::Name(mbr.name.clone())));

            access.off -= mbr.off as i32;

            // recurse
            return self.symbol_to_expr_recurse(expr, &mbr.typ, access);
          }
          panic!("Failed to find member");
        }
        _ => {
          panic!("Unknown ... {:?}", typ);
        }
      }
    }

    // Base-case of a primitive
    if access.off != 0 ||  access.sz as usize != typ.size_in_bytes().unwrap() {
      expr = Expr::Unary(Box::new(UnaryExpr {
        op: UnaryOperator::Addr,
        rhs: expr,
      }));
      if access.off != 0 {
        expr = Expr::Binary(Box::new(BinaryExpr {
          op: BinaryOperator::Add,
          lhs: expr,
          rhs: Expr::HexConst(access.off as u16),
        }));
      }
      let t = match access.sz {
        1 => Type::U8,
        2 => Type::U16,
        4 => Type::U32,
        _ => panic!("Unknown access size: {}", access.sz),
      };
      expr = Expr::Cast(Type::ptr(t), Box::new(expr));
      expr = Expr::Deref(Box::new(expr));
    }
    expr
  }

  fn make_label(&self, id: ElemId) -> Label {
    let elem = self.cf.elem(id);
    let Detail::BasicBlock(bb) = &elem.detail else { panic!("Expected basic block") };
    Label(format!("{}", self.ir.block(bb.blkref).name))
  }

  fn assign(&mut self, blk: &mut Block, typ: Type, name: &str, rhs: Expr) {
    let decltype = if OPT_DEFINE_TEMPS_AT_USE {
      Some(typ)
    } else {
      if self.assigned.get(name).is_none() {
        self.assigns.push((name.to_string(), typ));
        self.assigned.insert(name.to_string());
      }
      None
    };

    blk.push_stmt(Stmt::Assign(Assign {
      decltype,
      lhs: Expr::Name(name.to_string()),
      rhs,
    }));
  }

  fn emit_phis(&mut self, blk: &mut Block, src: ir::BlockRef, dst: ir::BlockRef) {
    // first, which pred is the src block?
    let mut idx = None;
    for (i, pred) in self.ir.block(dst).preds.iter().enumerate() {
      if *pred == src {
        idx = Some(i);
        break;
      }
    }
    let idx = idx.unwrap();

    // next, for each phi, generate code for the pred idx
    for r in self.ir.iter_instrs(dst) {
      let instr = self.ir.instr(r).unwrap();
      if instr.opcode != ir::Opcode::Phi { continue };

      let name = self.ref_name(r);
      let rvalue = self.ref_to_expr(instr.operands[idx], 1);
      self.assign(blk, instr.typ.clone(), &name, rvalue);
    }
  }

  // Returns a jump condition expr if the block ends in a conditional branch
  #[must_use]
  fn emit_blk(&mut self, blk: &mut Block, bref: ir::BlockRef, inverted_cond: bool) -> Option<Expr> {
    for r in self.ir.iter_instrs(bref) {
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
        ir::Opcode::JmpTbl => {
          // TODO: Maybe verify that there are no phis in the target block? This should be gaurenteed by
          // the ir finalize, but it's probably good to do sanity checks
          let idx = self.ref_to_expr(instr.operands[0], 1);
          return Some(idx);
        }
        ir::Opcode::WriteVar16 => {
          let lhs = self.symbol_to_expr(instr.operands[0].unwrap_symbol());
          let rhs = self.ref_to_expr(instr.operands[1], 1);
          blk.push_stmt(Stmt::Assign(Assign { decltype: None, lhs, rhs }));
        }
        ir::Opcode::WriteVar32 => {
          let lhs = self.symbol_to_expr(instr.operands[0].unwrap_symbol());
          let rhs = self.ref_to_expr(instr.operands[1], 1);
          blk.push_stmt(Stmt::Assign(Assign { decltype: None, lhs, rhs }));
        }
        ir::Opcode::Store8 => {
          let seg = self.ref_to_expr_hex(instr.operands[0], 1, true);
          let off = self.ref_to_expr_hex(instr.operands[1], 1, true);
          let lhs = Expr::Deref(Box::new(Expr::Abstract("PTR_8", vec![seg, off])));
          let rhs = self.ref_to_expr(instr.operands[2], 1);
          blk.push_stmt(Stmt::Assign(Assign { decltype: None, lhs, rhs }));
        }
        ir::Opcode::Store16 => {
          let seg = self.ref_to_expr_hex(instr.operands[0], 1, true);
          let off = self.ref_to_expr_hex(instr.operands[1], 1, true);
          let lhs = Expr::Deref(Box::new(Expr::Abstract("PTR_16", vec![seg, off])));
          let rhs = self.ref_to_expr(instr.operands[2], 1);
          blk.push_stmt(Stmt::Assign(Assign { decltype: None, lhs, rhs }));
        }
        ir::Opcode::AssertEven => {
          let val = self.ref_to_expr(instr.operands[0], 1);
          let cond = Expr::Binary(Box::new(BinaryExpr {
            op: BinaryOperator::Eq,
            lhs: Expr::Binary(Box::new(BinaryExpr {
              op: BinaryOperator::Mod,
              lhs: val,
              rhs: Expr::DecimalConst(2),
            })),
            rhs: Expr::DecimalConst(0),
          }));
          blk.push_stmt(Stmt::Expr(Expr::Abstract("assert", vec![cond])));
        }
        ir::Opcode::AssertPos => {
          let val = self.ref_to_expr(instr.operands[0], 1);
          let cond = Expr::Binary(Box::new(BinaryExpr {
            op: BinaryOperator::Geq,
            lhs: Expr::Cast(Type::I16, Box::new(val)),
            rhs: Expr::DecimalConst(0),
          }));
          blk.push_stmt(Stmt::Expr(Expr::Abstract("assert", vec![cond])));
        }
        ir::Opcode::Int => {
          let num = self.ref_to_expr_hex(instr.operands[0], 0, true);
          blk.push_stmt(Stmt::Expr(Expr::Abstract("INT", vec![num])));
        }
        _ => {
          let uses = self.n_uses.get(&r).cloned().unwrap_or(0);
          if uses != 1 || instr.opcode.is_call() {
            let rvalue = self.ref_to_expr(r, 0);
            let typ = self.ir.instr(r).unwrap().typ.clone();
            if typ == Type::Void {
              blk.push_stmt(Stmt::Expr(rvalue));
            } else {
              let name = self.ref_name(r);
              self.assign(blk, typ, &name, rvalue);
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
        // NOTE: cond already inverted before call!
        blk.push_stmt(Stmt::If(If {cond: cond, then_body }));
      }
      control_flow::Jump::CondTargetBoth(tgt_true, tgt_false) => {
        let cond = cond.unwrap();
        let label_true = self.make_label(tgt_true);
        let label_false = self.make_label(tgt_false);
        blk.push_stmt(Stmt::CondGoto(CondGoto { cond, label_true, label_false }));
      }
      control_flow::Jump::Table(_tgts) => {
        panic!("All JumpTable should be converted to Switch in control flow analysis");
      }
      //control_flow::Jump::Continue => {}
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

  fn convert_switch(&mut self, blk: &mut Block, iter: &mut FlowIter, depth: usize) {
    let Some(sw_elt) = iter.next() else { panic!("expected switch element") };
    let Detail::Switch(sw) = &sw_elt.elem.detail else { panic!("expected switch element") };

    let Detail::BasicBlock(bb) = &self.cf.elem(sw.entry).detail else { panic!("expected switch entry to be a basic-block") };
    if bb.labeled {
      let label = self.make_label(sw.entry);
      blk.push_stmt(Stmt::Label(label));
    }
    let Some(select) = self.emit_blk(blk, bb.blkref, false) else {
      panic!("expected switch entry to end in a jump table idx expr");
    };

    let mut cases = vec![];
    let mut map: HashMap<Label, usize> = HashMap::new(); // Label -> case-idx

    let mut idx = 0;
    while let Some(elt) = iter.peek() {
      if elt.depth <= depth {
        break;
      }
      assert!(elt.depth == depth+1);
      let elt = iter.next().unwrap();

      match &elt.elem.detail {
        Detail::Goto(g) => {
          let label = self.make_label(g.target);
          let case_idx = map.get(&label).cloned().unwrap_or_else(|| {
            let case_idx = cases.len();
            map.insert(label.clone(), case_idx);
            let mut body = Block::default();
            body.push_stmt(Stmt::Goto(Goto { label }));
            cases.push(SwitchCase {
              cases: vec![],
              body,
            });
            case_idx
          });
          cases[case_idx].cases.push(Expr::DecimalConst(idx as i16));
        }
        Detail::ElemBlock(_) => {
          let body = self.convert_body(iter, elt.depth+1);
          cases.push(SwitchCase {
            cases: vec![Expr::DecimalConst(idx as i16)],
            body,
          });
        }
        _ => panic!("Unexpected elem detail type in switch body: {:?}", elt.elem),
      }

      idx += 1;
    }

    let mut default = Block::default();
    default.push_stmt(Stmt::Unreachable);

    blk.push_stmt(Stmt::Switch(Switch {
      switch_val: select,
      cases,
      default: Some(default),
    }));
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
        Detail::Switch(_) => self.convert_switch(&mut blk, iter, depth),
        _ => panic!("Unknown detail type: {:?}", elt.elem.detail),
      };
    }

    blk
  }

  fn build(&mut self, name: &str, ret: Option<Type>) -> Function {
    let mut iter = self.cf.iter().peekable();
    let body = self.convert_body(&mut iter, 0);
    assert!(iter.next().is_none());

    // Group all decls by type to save codegen space
    // let mut type_map: HashMap<Type, usize> = HashMap::new();
    // let mut decls = vec![];
    // let mut mem_mapped: HashMap<String, VarDecl> = HashMap::new();
    // for d in &self.decls {
    //   if d.mem_mapping.is_some() {
    //     assert!(d.names.len() == 1);
    //     let name = &d.names[0];
    //     if mem_mapped.get(name).is_none() {
    //       mem_mapped.insert(name.clone(), d.clone());
    //     }
    //     continue;
    //   }
    //   let idx = match type_map.get(&d.typ) {
    //     Some(idx) => *idx,
    //     None => {
    //       let idx = decls.len();
    //       decls.push(VarDecl { typ: d.typ.clone(), names: vec![], mem_mapping: None });
    //       type_map.insert(d.typ.clone(), idx);
    //       idx
    //     }
    //   };
    //   for n in &d.names {
    //     decls[idx].names.push(n.clone());
    //   }
    // }
    // for n in itertools::sorted(mem_mapped.keys()) {
    //   decls.push(mem_mapped.get(n).unwrap().clone());
    // }

    let mut vardecls = vec![];
    let mut type_map: HashMap<Type, usize> = HashMap::new();
    for (name, typ) in std::mem::replace(&mut self.assigns, vec![]) {
      let idx = match type_map.get(&typ) {
        Some(idx) => *idx,
        None => {
          let idx = vardecls.len();
          vardecls.push(VarDecl { typ: typ.clone(), names: vec![] });
          type_map.insert(typ, idx);
          idx
        }
      };
      vardecls[idx].names.push(name);
    }

    let mut varmaps = vec![];
    for name in itertools::sorted(self.mappings.keys()) {
      let (typ, expr) = self.mappings.get(name).unwrap();
      varmaps.push(VarMap {
        typ: typ.clone(),
        name: name.to_string(),
        mapping_expr: expr.clone(),
      });
    }

    Function {
      name: name.to_string(),
      ret,
      vardecls,
      varmaps,
      body,
    }
  }
}

impl Function {
  pub fn from_ir(cfg: &Config, name: &str, ret: Option<Type>, ir: &ir::IR, ctrlflow: &ControlFlow) -> Self {
    Builder::new(cfg, ir, ctrlflow).build(name, ret)
  }
}
