use crate::instr;
use std::collections::HashMap;

// SSA IR Definitions

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)] pub struct ConstRef(pub usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)] pub struct BlockRef(pub usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)] pub struct InstrRef(pub usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)] pub struct PhiRef(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ref {
  //None,
  Const(ConstRef),
  Instr(BlockRef, InstrRef),
  Phi(BlockRef, PhiRef),
  Init(&'static str),  // FIXME: Don't use a String
  Block(BlockRef),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Symbol(pub instr::Reg);

impl From<instr::Reg> for Symbol {
  fn from(reg: instr::Reg) -> Self {
    Self(reg)
  }
}

impl From<&instr::Reg> for Symbol {
  fn from(reg: &instr::Reg) -> Self {
    Self(*reg)
  }
}

#[derive(Debug)]
pub struct Instr {
  pub opcode: Opcode,
  pub operands: Vec<Ref>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
  Nop,
  Phi,
  Push,
  Pop,
  //Mov,
  Add,
  Sub,
  Shl,
  And,
  //Or,
  Xor,
  //Eq,
  //Neq,
  Load8,
  Load16,
  Load32,
  Store8,
  Store16,
  Lower16,     // |n: u32| => n as u16
  Upper16,     // |n: u32| => (n >> 16) as u16
  UpdateFlags,
  EqFlags,
  NeqFlags,
  GtFlags,
  GeqFlags,
  LtFlags,
  LeqFlags,
  Call,
  Ret,
  Jmp,
  Jne,
}


impl Opcode {
  pub fn as_str(&self) -> &'static str {
    match self {
      Opcode::Nop         => "nop",
      Opcode::Phi         => "phi",
      Opcode::Push        => "push",
      Opcode::Pop         => "pop",
      //Opcode::Mov         => "mov",
      Opcode::Sub         => "sub",
      Opcode::Add         => "add",
      Opcode::Shl         => "shl",
      Opcode::And         => "and",
      //Opcode::Or          => "or",
      Opcode::Xor         => "xor",
      //Opcode::Eq          => "eq",
      //Opcode::Neq         => "neq",
      Opcode::Load8       => "load8",
      Opcode::Load16      => "load16",
      Opcode::Load32      => "load32",
      Opcode::Store8      => "store8",
      Opcode::Store16     => "store16",
      Opcode::Lower16     => "lower16",
      Opcode::Upper16     => "upper16",
      Opcode::UpdateFlags => "updf",
      Opcode::EqFlags     => "eqf",
      Opcode::NeqFlags    => "neqf",
      Opcode::GtFlags     => "gtf",
      Opcode::GeqFlags    => "geqf",
      Opcode::LtFlags     => "gtf",
      Opcode::LeqFlags    => "geqf",
      Opcode::Call        => "call",
      Opcode::Ret         => "ret",
      Opcode::Jmp         => "jmp",
      Opcode::Jne         => "jne",
    }
  }
}

#[derive(Debug)]
pub struct Block {
  pub name: String,
  pub defs: HashMap<Symbol, Ref>,
  pub preds: Vec<BlockRef>,
  pub phis: Vec<Instr>,
  pub instrs: Vec<Instr>,
}

#[derive(Debug)]
pub struct IR {
  pub consts: Vec<i32>,
  pub blocks: Vec<Block>,
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
}
