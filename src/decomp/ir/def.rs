use crate::instr;
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
  pub debug: Option<(Symbol, usize)>,
  pub opcode: Opcode,
  pub operands: Vec<Ref>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
  Nop,
  Pin,
  Ref,
  Phi,
  Add,
  Sub,
  Shl,
  And,
  Xor,
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
      Opcode::Pin         => "pin",
      Opcode::Ref         => "ref",
      Opcode::Phi         => "phi",
      Opcode::Sub         => "sub",
      Opcode::Add         => "add",
      Opcode::Shl         => "shl",
      Opcode::And         => "and",
      Opcode::Xor         => "xor",
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

  pub fn has_no_result(&self) -> bool {
    match self {
      Opcode::Nop => true,
      Opcode::Pin => true,
      Opcode::Store8 => true,
      Opcode::Store16 => true,
      Opcode::Ret => true,
      Opcode::Jmp => true,
      Opcode::Jne => true,
      _ => false,
    }
  }
}

#[derive(Debug)]
pub struct Block {
  pub name: String,
  pub defs: HashMap<Symbol, Ref>,
  pub preds: Vec<BlockRef>,
  pub instrs: DVec<Instr>,
}

#[derive(Debug)]
pub struct IR {
  pub consts: Vec<i32>,
  pub blocks: Vec<Block>,
}

impl IR {
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
}
