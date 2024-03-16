use crate::instr;
use crate::decomp::ir::sym;
use crate::util::dvec::{DVec, DVecIndex};
use std::collections::HashMap;

// SSA IR Definitions

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)] pub struct ConstRef(pub usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)] pub struct BlockRef(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ref {
  //None,
  Const(ConstRef),
  Instr(BlockRef, DVecIndex),
  Init(instr::Reg),
  Block(BlockRef),
  Symbol(sym::SymbolRef),
  Func(usize),
}

impl Ref {
  pub fn unwrap_block(self) -> BlockRef {
    let Ref::Block(blkref) = self else { panic!("expected block ref") };
    blkref
  }

  pub fn unwrap_symbol(self) -> sym::SymbolRef {
    let Ref::Symbol(symref) = self else { panic!("expected symbol ref") };
    symref
  }

  pub fn unwrap_func(self) -> usize {
    let Ref::Func(idx) = self else { panic!("expected function ref") };
    idx
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Name {
  Reg(instr::Reg),
  Var(String),
}

impl From<instr::Reg> for Name {
  fn from(reg: instr::Reg) -> Self {
    Self::Reg(reg)
  }
}

impl From<&instr::Reg> for Name {
  fn from(reg: &instr::Reg) -> Self {
    Self::Reg(*reg)
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FullName(pub Name, pub usize);

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Instr {
  pub opcode: Opcode,
  pub operands: Vec<Ref>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Opcode {
  Nop,
  Pin,
  Ref,
  Phi,

  Add,
  Sub,
  Shl,
  Shr,    // signed
  UShr,   // unsigned
  And,
  Or,
  Xor,
  IDiv,
  UDiv,
  IMul,
  UMul,

  Neg,

  SignExtTo32,
  Load8,
  Load16,
  Load32,
  Store8,
  Store16,
  ReadVar8,
  ReadVar16,
  ReadVar32,
  WriteVar8,
  WriteVar16,
  ReadArr8,
  ReadArr16,
  ReadArr32,
  WriteArr8,
  WriteArr16,
  Lower16,     // |n: u32| => n as u16
  Upper16,     // |n: u32| => (n >> 16) as u16
  Make32,      // |high: u16, low: u16| => (high as u32) << 16 | (low as u32)

  UpdateFlags,
  EqFlags,     // Maps to: JE  / JZ
  NeqFlags,    // Maps to: JNE / JNZ
  GtFlags,     // Maps to: JG  / JNLE
  GeqFlags,    // Maps to: JGE / JNL
  LtFlags,     // Maps to: JL  / JNGE
  LeqFlags,    // Maps to: JLE / JNG
  UGtFlags,    // Maps to: JA  / JNBE
  UGeqFlags,   // Maps to: JAE / JNB  / JNC
  ULtFlags,    // Maps to: JB  / JNAE / JC
  ULeqFlags,   // Maps to: JBE / JNA

  Eq,          // Operator: == (any sign)
  Neq,         // Operator: != (any sign)
  Gt,          // Operator: >  (signed)
  Geq,         // Operator: >= (signed)
  Lt,          // Operator: <  (signed)
  Leq,         // Operator: <= (signed)
  UGt,         // Operator: >  (unsigned)
  UGeq,        // Operator: >= (unsigned)
  ULt,         // Operator: <  (unsigned)
  ULeq,        // Operator: <=  (unsigned)

  CallFar,
  CallNear,
  CallPtr,
  CallArgs,

  RetFar,
  RetNear,

  Jmp,
  Jne,
  JmpTbl,

  // TODO: HMMM.... Better Impl?
  AssertEven,
}


impl Opcode {
  pub fn as_str(&self) -> &'static str {
    match self {
      Opcode::Nop         => "nop",
      Opcode::Pin         => "pin",
      Opcode::Ref         => "ref",
      Opcode::Phi         => "phi",
      Opcode::Sub         => "sub",
      Opcode::Add         => "add",
      Opcode::Shl         => "shl",
      Opcode::Shr         => "shr",
      Opcode::UShr        => "ushr",
      Opcode::And         => "and",
      Opcode::Or          => "or",
      Opcode::Xor         => "xor",
      Opcode::IDiv        => "idiv",
      Opcode::UDiv        => "udiv",
      Opcode::IMul        => "imul",
      Opcode::UMul        => "umul",
      Opcode::Neg         => "neg",
      Opcode::SignExtTo32 => "signext32",
      //Opcode::AddrOf      => "addrof",
      Opcode::Load8       => "load8",
      Opcode::Load16      => "load16",
      Opcode::Load32      => "load32",
      Opcode::Store8      => "store8",
      Opcode::Store16     => "store16",
      Opcode::ReadVar8    => "readvar8",
      Opcode::ReadVar16   => "readvar16",
      Opcode::ReadVar32   => "readvar32",
      Opcode::WriteVar8   => "writevar8",
      Opcode::WriteVar16  => "writevar16",
      Opcode::ReadArr8    => "readarr8",
      Opcode::ReadArr16   => "readarr16",
      Opcode::ReadArr32   => "readarr32",
      Opcode::WriteArr8   => "writearr8",
      Opcode::WriteArr16  => "writearr16",
      Opcode::Lower16     => "lower16",
      Opcode::Upper16     => "upper16",
      Opcode::Make32      => "make32",
      Opcode::UpdateFlags => "updf",
      Opcode::EqFlags     => "eqf",
      Opcode::NeqFlags    => "neqf",
      Opcode::GtFlags     => "gtf",
      Opcode::GeqFlags    => "geqf",
      Opcode::LtFlags     => "gtf",
      Opcode::LeqFlags    => "geqf",
      Opcode::UGtFlags    => "ugtf",
      Opcode::UGeqFlags   => "ugeqf",
      Opcode::ULtFlags    => "ugtf",
      Opcode::ULeqFlags   => "ugeqf",
      Opcode::Eq          => "eq",
      Opcode::Neq         => "neq",
      Opcode::Gt          => "gt",
      Opcode::Geq         => "geq",
      Opcode::Lt          => "lt",
      Opcode::Leq         => "leq",
      Opcode::UGt         => "ugt",
      Opcode::UGeq        => "ugeq",
      Opcode::ULt         => "ult",
      Opcode::ULeq        => "uleq",
      Opcode::CallFar     => "callfar",
      Opcode::CallNear    => "callnear",
      Opcode::CallPtr     => "callptr",
      Opcode::CallArgs    => "callargs",
      Opcode::RetFar      => "retf",
      Opcode::RetNear     => "retn",
      Opcode::Jmp         => "jmp",
      Opcode::Jne         => "jne",
      Opcode::JmpTbl      => "jmptbl",

      Opcode::AssertEven => "assert_even",
    }
  }

  pub fn is_load(&self) -> bool {
    match self {
      Opcode::Load8 => true,
      Opcode::Load16 => true,
      Opcode::Load32 => true,
      _ => false,
    }
  }

  pub fn is_store(&self) -> bool {
    match self {
      Opcode::Store8 => true,
      Opcode::Store16 => true,
      _ => false,
    }
  }

  pub fn is_call(&self) -> bool {
    match self {
      Opcode::CallFar | Opcode::CallNear | Opcode::CallPtr | Opcode::CallArgs => true,
      _ => false,
    }
  }

  pub fn operation_size(&self) -> u32 {
    match self {
      Opcode::Load8 => 1,
      Opcode::Load16 => 2,
      Opcode::Load32 => 4,
      Opcode::Store8 => 1,
      Opcode::Store16 => 2,
      _ => unreachable!(),
    }
  }

  pub fn has_no_result(&self) -> bool {
    match self {
      Opcode::Nop => true,
      //Opcode::Pin => true,
      Opcode::Store8 => true,
      Opcode::Store16 => true,
      Opcode::WriteVar8 => true,
      Opcode::WriteVar16 => true,
      Opcode::RetFar => true,
      Opcode::RetNear => true,
      Opcode::Jmp => true,
      Opcode::Jne => true,
      Opcode::JmpTbl => true,
      _ => false,
    }
  }

  pub fn maybe_unused(&self) -> bool {
    match self {
      Opcode::Nop => true,
      Opcode::Pin => true,
      Opcode::Store8 => true,
      Opcode::Store16 => true,
      Opcode::WriteVar8 => true,
      Opcode::WriteVar16 => true,
      Opcode::CallFar => true,
      Opcode::CallNear => true,
      Opcode::CallPtr => true,
      Opcode::CallArgs => true,
      Opcode::RetFar => true,
      Opcode::RetNear => true,
      Opcode::Jmp => true,
      Opcode::Jne => true,
      Opcode::JmpTbl => true,
      _ => false,
    }
  }

  pub fn has_side_effects(&self) -> bool {
    match self {
      Opcode::Pin => true,
      Opcode::Store8 => true,
      Opcode::Store16 => true,
      Opcode::WriteVar8 => true,
      Opcode::WriteVar16 => true,
      Opcode::CallFar => true,
      Opcode::CallNear => true,
      Opcode::CallPtr => true,
      Opcode::CallArgs => true,
      Opcode::RetFar => true,
      Opcode::RetNear => true,
      Opcode::Jmp => true,
      Opcode::Jne => true,
      Opcode::JmpTbl => true,
      _ => false,
    }
  }
}

#[derive(Debug)]
pub struct Block {
  pub name: String,
  pub defs: HashMap<Name, Ref>,
  pub preds: Vec<BlockRef>,
  pub instrs: DVec<Instr>,
  pub sealed: bool, // has all predecessors?
  pub incomplete_phis: Vec<(Name, Ref)>,
}

#[derive(Debug)]
pub struct IR {
  pub consts: Vec<i32>,
  pub symbols: sym::SymbolMap,
  pub funcs: Vec<String>,
  pub names: HashMap<Ref, FullName>,
  pub name_next: HashMap<Name, usize>,
  pub blocks: Vec<Block>,
}

impl IR {
  pub fn new() -> Self {
    Self {
      consts: vec![],
      symbols: sym::SymbolMap::new(),
      funcs: vec![],
      names: HashMap::new(),
      name_next: HashMap::new(),
      blocks: vec![],
    }
  }

  pub fn instr(&self, r: Ref) -> Option<&Instr> {
    if let Ref::Instr(b, i) = r {
      Some(&self.blocks[b.0].instrs[i])
    } else {
      None
    }
  }

  pub fn instr_mut(&mut self, r: Ref) -> Option<&mut Instr> {
    if let Ref::Instr(b, i) = r {
      Some(&mut self.blocks[b.0].instrs[i])
    } else {
      None
    }
  }
}

impl IR {
  pub fn append_const(&mut self, val: i32) -> Ref {
    // Search existing constants for a duplicate (TODO: Make this into a hash-tbl if it gets slow)
    for (i, constval) in self.consts.iter().enumerate() {
      if val == *constval {
        return Ref::Const(ConstRef(i))
      }
    }

    // Add new constant
    let idx = self.consts.len();
    self.consts.push(val);
    Ref::Const(ConstRef(idx))
  }

  pub fn lookup_const(&self, k: Ref) -> Option<i32> {
    if let Ref::Const(ConstRef(i)) = k {
      Some(self.consts[i])
    } else {
      None
    }
  }

  fn phi_populate<S: Into<Name>>(&mut self, sym: S, phiref: Ref) {
    let sym: Name = sym.into();
    let Ref::Instr(blk, idx) = phiref else { panic!("Invalid ref") };

    let preds = self.blocks[blk.0].preds.clone(); // ARGH: Need to break borrow on 'self' so we can recurse
    assert!(self.blocks[blk.0].instrs[idx].opcode == Opcode::Phi);

    // recurse each pred
    let mut refs = vec![];
    for b in preds {
      refs.push(self.get_var(sym.clone(), b));
    }

    // update the phi with operands
    self.blocks[blk.0].instrs[idx].operands = refs;

    // TODO: Remove trivial phis
  }

  fn phi_create(&mut self, sym: Name, blk: BlockRef) -> Ref {
    // create phi node (without operands) to terminate recursion

    let idx = self.blocks[blk.0].instrs.push_front(Instr {
      opcode: Opcode::Phi,
      operands: vec![],
    });

    let vref = Ref::Instr(blk, idx);
    self.set_var(sym, blk, vref);

    vref
  }

  pub fn get_var<S: Into<Name>>(&mut self, sym: S, blk: BlockRef) -> Ref {
    let sym: Name = sym.into();

    // Defined locally in this block? Easy.
    match self.blocks[blk.0].defs.get(&sym) {
      Some(val) => return *val,
      None => (),
    }

    // Otherwise, search predecessors
    let b = &self.blocks[blk.0];
    if !b.sealed {
      // add an empty phi node and mark it for later population
      let phi = self.phi_create(sym.clone(), blk);
      self.blocks[blk.0].incomplete_phis.push((sym, phi));
      phi
    } else {
      let preds = &self.blocks[blk.0].preds;
      if preds.len() == 1 {
        let parent = preds[0];
        self.get_var(sym, parent)
      } else {
        // create a phi and immediately populate it
        let phi = self.phi_create(sym.clone(), blk);
        self.phi_populate(sym, phi);
        phi
      }
    }
  }

  pub fn set_var<S: Into<Name>>(&mut self, sym: S, blk: BlockRef, r: Ref) {
    let sym = sym.into();
    self.blocks[blk.0].defs.insert(sym.clone(), r);
    self.set_name(&sym, r);
  }

  pub fn seal_block(&mut self, r: BlockRef) {
    let b = &mut self.blocks[r.0];
    if b.sealed { panic!("block is already sealed!"); }
    b.sealed = true;
    for (sym, phi) in std::mem::replace(&mut b.incomplete_phis, vec![]) {
      self.phi_populate(sym, phi)
    }
  }

  pub fn unseal_all_blocks(&mut self) {
    for i in 0..self.blocks.len() {
      self.blocks[i].sealed = false;
    }
  }

  pub fn seal_all_blocks(&mut self) {
    for i in 0..self.blocks.len() {
      self.seal_block(BlockRef(i));
    }
  }

  fn set_name(&mut self, name: &Name, r: Ref) {
   let idx_ref = self.name_next.entry(name.clone()).or_insert(1);
   let idx = *idx_ref;
   *idx_ref = idx+1;
   self.names.insert(r, FullName(name.clone(), idx));
 }
}

impl Block {
  pub fn new(name: &str) -> Self {
    Self {
      name: name.to_string(),
      defs: HashMap::new(),
      preds: vec![],
      instrs: DVec::new(),
      sealed: false,
      incomplete_phis: vec![],
    }
  }
}
