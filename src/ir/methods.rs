use crate::asm::instr;
use crate::ir::def::*;
use crate::sym;
use crate::types::Type;
use crate::ir::block_data::{self, InstrData};
use std::collections::HashMap;

////////////////////////////////////////////////////////////////////////////////////
// Creation
////////////////////////////////////////////////////////////////////////////////////
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

  pub fn add_block(&mut self, name: &str) -> BlockRef {
    let idx = self.blocks.len();
    let blkref = BlockRef(idx);

    let blk = Block {
      name: name.to_string(),
      defs: HashMap::new(),
      preds: vec![],
      data: InstrData::new(blkref),
      sealed: false,
      incomplete_phis: vec![],
    };

    self.blocks.push(Some(blk));

    blkref
  }
}

////////////////////////////////////////////////////////////////////////////////////
// Iteration
////////////////////////////////////////////////////////////////////////////////////
impl IR {
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
    // NOTE: Not the most efficent, but this prevents holding an immutable reference
    // which would prevent mutation during the iteration
    let data = &self.block(blk).data;
    let mut refs = vec![];
    let mut next = data.first();
    while let Some(cur) = next {
      refs.push(cur);
      next = data.next(cur);
    }
    refs.into_iter()
  }
}

////////////////////////////////////////////////////////////////////////////////////
// Blocks
////////////////////////////////////////////////////////////////////////////////////
impl IR {
  pub fn block(&self, blkref: BlockRef) -> &Block {
    self.blocks[blkref.0].as_ref().unwrap()
  }

  pub fn block_mut(&mut self, blkref: BlockRef) -> &mut Block {
    self.blocks[blkref.0].as_mut().unwrap()
  }

  pub fn block_remove(&mut self, blkref: BlockRef) {
    // Caller is responsible for ensuring there are no references to this block elsewhere in the IR
    assert!(self.blocks[blkref.0].is_some());
    self.blocks[blkref.0] = None;
  }

  pub fn block_last(&self, blkref: BlockRef) -> Ref {
    self.block(blkref).data.last().unwrap()
  }

  pub fn block_last_instr(&self, blkref: BlockRef) -> Option<&Instr> {
    self.instr(self.block_last(blkref))
  }

  pub fn block_exits(&self, blkref: BlockRef) -> Vec<BlockRef> {
    let instr = self.block_last_instr(blkref).unwrap();

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

  pub fn block_instr_count(&mut self, blkref: BlockRef) -> usize {
    self.block(blkref).data.count()
  }

  pub fn block_instr_append(&mut self, blkref: BlockRef, instr: Instr) -> Ref {
    self.block_mut(blkref).data.insert(instr, block_data::Loc::Last)
  }

  pub fn block_instr_prepend(&mut self, blkref: BlockRef, instr: Instr) -> Ref {
    self.block_mut(blkref).data.insert(instr, block_data::Loc::First)
  }
}

////////////////////////////////////////////////////////////////////////////////////
// Instructions
////////////////////////////////////////////////////////////////////////////////////
impl IR {
  pub fn instr_prev(&self, r: Ref) -> Option<Ref> {
    let Ref::Instr(b, _) = r else { return None };
    let blk = self.block(b);

    let mut iref = Some(r);
    while let Some(r) = iref {
      if self.instr(r).unwrap().opcode != Opcode::Nop {
        return Some(r);
      }
      iref = blk.data.prev(r);
    }
    None
  }

  pub fn instr_next(&self, r: Ref) -> Option<Ref> {
    let Ref::Instr(b, _) = r else { return None };
    let blk = self.block(b);

    let mut iref = Some(r);
    while let Some(r) = iref {
      if self.instr(r).unwrap().opcode != Opcode::Nop {
        return Some(r);
      }
      iref = blk.data.next(r);
    }
    None
  }

  pub fn instr(&self, r: Ref) -> Option<&Instr> {
    if let Ref::Instr(b, _) = r {
      Some(self.block(b).data.lookup(r))
    } else {
      None
    }
  }

  pub fn instr_mut(&mut self, r: Ref) -> Option<&mut Instr> {
    if let Ref::Instr(b, _) = r {
      Some(self.block_mut(b).data.lookup_mut(r))
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

////////////////////////////////////////////////////////////////////////////////////
// Constants
////////////////////////////////////////////////////////////////////////////////////
impl IR {
  pub fn const_new(&mut self, val: i16) -> Ref {
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

  pub fn const_lookup(&self, k: Ref) -> Option<i16> {
    if let Ref::Const(ConstRef(i)) = k {
      Some(self.consts[i])
    } else {
      None
    }
  }

}

////////////////////////////////////////////////////////////////////////////////////
// Phi
////////////////////////////////////////////////////////////////////////////////////
impl IR {
  fn phi_populate<S: Into<Name>>(&mut self, sym: S, phiref: Ref) {
    let sym: Name = sym.into();
    let Ref::Instr(blk, _) = phiref else { panic!("Invalid ref") };

    let preds = self.block(blk).preds.clone(); // ARGH: Need to break borrow on 'self' so we can recurse
    assert!(self.instr(phiref).unwrap().opcode == Opcode::Phi);

    // recurse each pred
    let mut refs = vec![];
    for b in preds {
      refs.push(self.get_var(sym.clone(), b));
    }

    // update the phi with operands
    self.instr_mut(phiref).unwrap().operands = refs;

    // TODO: Remove trivial phis
  }

  fn phi_create(&mut self, sym: Name, blk: BlockRef) -> Ref {
    // create phi node (without operands) to terminate recursion

    let vref = self.block_instr_prepend(blk, Instr {
      typ: Type::U16, // TODO: SANITY CHECK THAT NO OTHER SIZES CAN GO THROUGH A PHI!!
      attrs: Attribute::NONE,
      opcode: Opcode::Phi,
      operands: vec![],
    });

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

impl IR {
  pub fn compute_uses(&self) -> HashMap<Ref, usize> {
    let mut n_uses = HashMap::new();
    for b in self.iter_blocks() {
      for r in self.iter_instrs(b) {
        let instr = self.instr(r).unwrap();
        for oper in &instr.operands {
          *n_uses.entry(*oper).or_insert(0) += 1;
        }
      }
    }
    n_uses
  }
}

impl Ref {
  pub fn is_instr(self) -> bool {
    let Ref::Instr(_, _) = self else { return false };
    true
  }

  pub fn is_const(self) -> bool {
    let Ref::Const(_) = self else { return false };
    true
  }

  pub fn unwrap_instr(self) -> (BlockRef, usize) {
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

impl From<instr::Reg> for Name { fn from(reg: instr::Reg) -> Self { Self::Reg(reg) } }
impl From<&instr::Reg> for Name { fn from(reg: &instr::Reg) -> Self { Self::Reg(*reg) } }
