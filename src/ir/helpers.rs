use crate::asm::instr;
use crate::ir::def::*;
use crate::ir::sym;
use crate::types::Type;
use crate::util::dvec::{DVec, DVecIndex};
use std::collections::HashMap;

impl Ref {
  pub fn is_const(self) -> bool {
    let Ref::Const(_) = self else { return false };
    true
  }

  pub fn unwrap_instr(self) -> (BlockRef, DVecIndex) {
    let Ref::Instr(b, i) = self else { panic!("expected instr ref") };
    (b, i)
  }

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

impl Opcode {
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
      Opcode::Store32 => true,
      _ => false,
    }
  }

  pub fn is_mem_op(&self) -> bool {
    match self {
      Opcode::Load8 => true,
      Opcode::Load16 => true,
      Opcode::Load32 => true,
      Opcode::Store8 => true,
      Opcode::Store16 => true,
      Opcode::Store32 => true,
      Opcode::ReadVar8 => true,
      Opcode::ReadVar16 => true,
      Opcode::ReadVar32 => true,
      Opcode::WriteVar8 => true,
      Opcode::WriteVar16 => true,
      Opcode::WriteVar32 => true,
      _ => false,
    }
  }

  pub fn is_call(&self) -> bool {
    match self {
      Opcode::CallFar | Opcode::CallNear | Opcode::CallPtr | Opcode::CallArgs => true,
      _ => false,
    }
  }

  pub fn operation_size(&self) -> u16 {
    match self {
      Opcode::Load8 => 1,
      Opcode::Load16 => 2,
      Opcode::Load32 => 4,
      Opcode::Store8 => 1,
      Opcode::Store16 => 2,
      _ => panic!("invalid"),
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
      Opcode::WriteVar32 => true,
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
      Opcode::WriteVar32 => true,
      Opcode::CallFar => true,
      Opcode::CallNear => true,
      Opcode::CallPtr => true,
      Opcode::CallArgs => true,
      Opcode::Int => true,
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
      Opcode::WriteVar32 => true,
      Opcode::CallFar => true,
      Opcode::CallNear => true,
      Opcode::CallPtr => true,
      Opcode::CallArgs => true,
      Opcode::Int => true,
      Opcode::RetFar => true,
      Opcode::RetNear => true,
      Opcode::Jmp => true,
      Opcode::Jne => true,
      Opcode::JmpTbl => true,
      Opcode::AssertEven => true,
      Opcode::AssertPos => true,
      _ => false,
    }
  }
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

  pub fn block(&self, blkref: BlockRef) -> &Block {
    self.blocks[blkref.0].as_ref().unwrap()
  }

  pub fn block_mut(&mut self, blkref: BlockRef) -> &mut Block {
    self.blocks[blkref.0].as_mut().unwrap()
  }

  pub fn push_block(&mut self, blk: Block) -> BlockRef {
    let idx = self.blocks.len();
    self.blocks.push(Some(blk));
    BlockRef(idx)
  }

  pub fn remove_block(&mut self, blkref: BlockRef) {
    // Caller is responsible for ensuring there are no references to this block elsewhere in the IR
    assert!(self.blocks[blkref.0].is_some());
    self.blocks[blkref.0] = None;
  }

  pub fn iter_blocks(&self) -> impl Iterator<Item=BlockRef> {
    // FIXME: Can we avoid the intermediate vec?? (Without holding &self hostage?)
    let mut blkrefs = vec![];
    for i in 0..self.blocks.len() {
      if self.blocks[i].is_some() {
        blkrefs.push(BlockRef(i))
      }
    }
    blkrefs.into_iter()
  }

  pub fn iter_instrs(&self, blk: BlockRef) -> impl Iterator<Item=Ref> {
    self.block(blk).instrs.range().map(move |idx| Ref::Instr(blk, idx))
  }

  pub fn prev_ref_in_block(&self, r: Ref) -> Option<Ref> {
    let Ref::Instr(b, mut i) = r else { return None };
    let blk = self.block(b);
    while i > blk.instrs.start() {
      i -= 1;
      if blk.instrs[i].opcode != Opcode::Nop {
        return Some(Ref::Instr(b, i));
      }
    }
    None
  }

  pub fn next_ref_in_block(&self, r: Ref) -> Option<Ref> {
    let Ref::Instr(b, mut i) = r else { return None };
    let blk = self.block(b);
    loop {
      i += 1;
      if i >= blk.instrs.end() { return None }
      if blk.instrs[i].opcode != Opcode::Nop {
        return Some(Ref::Instr(b, i));
      }
    }
  }

  pub fn instr(&self, r: Ref) -> Option<&Instr> {
    if let Ref::Instr(b, i) = r {
      Some(&self.block(b).instrs[i])
    } else {
      None
    }
  }

  pub fn instr_mut(&mut self, r: Ref) -> Option<&mut Instr> {
    if let Ref::Instr(b, i) = r {
      Some(&mut self.block_mut(b).instrs[i])
    } else {
      None
    }
  }

  pub fn instr_matches(&self, r: Ref, op: Opcode) -> Option<(&Instr, Ref)> {
    let instr = self.instr(r)?;
    if instr.opcode != op { return None }
    Some((instr, r))
  }
}

impl IR {
  pub fn append_const(&mut self, val: i16) -> Ref {
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

  pub fn lookup_const(&self, k: Ref) -> Option<i16> {
    if let Ref::Const(ConstRef(i)) = k {
      Some(self.consts[i])
    } else {
      None
    }
  }

  fn phi_populate<S: Into<Name>>(&mut self, sym: S, phiref: Ref) {
    let sym: Name = sym.into();
    let Ref::Instr(blk, idx) = phiref else { panic!("Invalid ref") };

    let preds = self.block(blk).preds.clone(); // ARGH: Need to break borrow on 'self' so we can recurse
    assert!(self.block_mut(blk).instrs[idx].opcode == Opcode::Phi);

    // recurse each pred
    let mut refs = vec![];
    for b in preds {
      refs.push(self.get_var(sym.clone(), b));
    }

    // update the phi with operands
    self.block_mut(blk).instrs[idx].operands = refs;

    // TODO: Remove trivial phis
  }

  fn phi_create(&mut self, sym: Name, blk: BlockRef) -> Ref {
    // create phi node (without operands) to terminate recursion

    let idx = self.block_mut(blk).instrs.push_front(Instr {
      typ: Type::U16, // TODO: SANITY CHECK THAT NO OTHER SIZES CAN GO THROUGH A PHI!!
      attrs: Attribute::NONE,
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
    match self.block_mut(blk).defs.get(&sym) {
      Some(val) => return *val,
      None => (),
    }

    // Otherwise, search predecessors
    if !self.block(blk).sealed {
      // add an empty phi node and mark it for later population
      let phi = self.phi_create(sym.clone(), blk);
      self.block_mut(blk).incomplete_phis.push((sym, phi));
      phi
    } else {
      let preds = &self.block(blk).preds;
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
    self.block_mut(blk).defs.insert(sym.clone(), r);
    self.set_name(&sym, r);
  }

  pub fn seal_block(&mut self, r: BlockRef) {
    let b = self.block_mut(r);
    if b.sealed { panic!("block is already sealed!"); }
    b.sealed = true;
    for (sym, phi) in std::mem::replace(&mut b.incomplete_phis, vec![]) {
      self.phi_populate(sym, phi)
    }
  }

  pub fn unseal_all_blocks(&mut self) {
    for b in self.iter_blocks() {
      self.block_mut(b).sealed = false;
    }
  }

  pub fn seal_all_blocks(&mut self) {
    for b in self.iter_blocks() {
      self.seal_block(b);
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

  pub fn exits(&self) -> Vec<BlockRef> {
    let instr = self.instrs.last().unwrap();
    match instr.opcode {
      Opcode::RetFar | Opcode::RetNear => vec![],
      Opcode::Jmp => vec![
        instr.operands[0].unwrap_block()
      ],
      Opcode::Jne => vec![
        instr.operands[1].unwrap_block(),
        instr.operands[2].unwrap_block()
      ],
      Opcode::JmpTbl => {
        instr.operands[1..].iter().map(|oper| oper.unwrap_block()).collect()
      },
      _ => panic!("Expected last instruction to be a branching instruction: {:?}", instr),
    }
  }
}
