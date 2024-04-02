pub use crate::ir::opcode::Opcode;
use crate::asm::instr;
use crate::ir::sym;
use crate::types::Type;
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Name {
  Reg(instr::Reg),
  Var(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FullName(pub Name, pub usize);

#[allow(non_snake_case)]
pub mod Attribute {
  pub const NONE: u8 = 0;
  pub const MAY_ESCAPE: u8 = 1<<0;
  pub const STACK_PTR: u8 = 1<<1;
  pub const PIN: u8 = 1<<2;
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Instr {
  pub typ: Type,
  pub attrs: u8,
  pub opcode: Opcode,
  pub operands: Vec<Ref>,
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
  pub consts: Vec<i16>,
  pub symbols: sym::SymbolMap,
  pub funcs: Vec<String>,
  pub names: HashMap<Ref, FullName>,
  pub name_next: HashMap<Name, usize>,
  pub blocks: Vec<Option<Block>>,  // Optional because dead blocks can be pruned out
}
