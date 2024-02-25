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
  Init(&'static str),  // FIXME: Don't use a String
  Block(BlockRef),
  Symbol(sym::SymbolRef),
  Func(usize),
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

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Instr {
  pub debug: Option<(Symbol, usize)>,
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
  And,
  Xor,
  //AddrOf,
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
  CallArgs,
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
      Opcode::CallArgs    => "callargs",
      Opcode::Ret         => "ret",
      Opcode::Jmp         => "jmp",
      Opcode::Jne         => "jne",
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
      Opcode::Pin => true,
      Opcode::Store8 => true,
      Opcode::Store16 => true,
      Opcode::WriteVar8 => true,
      Opcode::WriteVar16 => true,
      Opcode::Ret => true,
      Opcode::Jmp => true,
      Opcode::Jne => true,
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
      Opcode::Call => true,
      Opcode::CallArgs => true,
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
  pub symbols: sym::SymbolMap,
  pub funcs: Vec<String>,
  pub blocks: Vec<Block>,
}

impl IR {
  pub fn new() -> Self {
    Self {
      consts: vec![],
      symbols: sym::SymbolMap::new(),
      funcs: vec![],
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
}
