use crate::ir::def::*;
use std::collections::HashMap;

struct Finalizer {
  num: usize,
}

impl Finalizer {
  fn new() -> Self {
    Self { num: 0 }
  }

  // Insert blocks between jne and phi so that astgen can
  // implement phi's in the intermediate block.
  // This is required only when a block ends with a jne and
  // one or more target block contain phis
  fn insert_intermediate_phi_blocks(&mut self, ir: &mut IR) {
    for blkref in ir.iter_blocks() {
      let r = Ref::Instr(blkref, ir.block(blkref).instrs.last_idx().unwrap());
      let exits = ir.block(blkref).exits();
      if exits.len() <= 1 { continue; }
      let mut old_to_new = HashMap::new();
      for (i, exit) in exits.into_iter().enumerate() {
        if let Some(new_exit) = old_to_new.get(&exit) {
          // Already generated phi block.. rewrite it
          ir.instr_mut(r).unwrap().operands[i+1] = Ref::Block(*new_exit);
          continue;
        }
        if target_has_phis(ir, exit) {
          let new_exit = self.insert_block(ir, blkref, r, i+1);
          old_to_new.insert(exit, new_exit);
        }
      }
    }
  }

  fn insert_block(&mut self, ir: &mut IR, blkref: BlockRef, r: Ref, oper_idx: usize) -> BlockRef {
    // unique number for block name
    let num = self.num;
    self.num += 1;

    // fetch the dest_blkref
    let instr = ir.instr(r).unwrap();
    let dest_blkref = instr.operands[oper_idx];

    // generate new block
    let mut new_blk = Block::new(&format!("phi_{:04}", num));
    new_blk.sealed = true;

    // have the new block jump to the original destination
    new_blk.instrs.push_back(Instr {
      typ: crate::types::Type::Void,
      attrs: Attribute::NONE,
      opcode: Opcode::Jmp,
      operands: vec![dest_blkref],
    });

    // add preds to new blk
    new_blk.preds.push(blkref);

    // append the block to the ir
    let new_blkref = ir.push_block(new_blk);

    // update the jne instruction to jump to the new blk
    let instr = ir.instr_mut(r).unwrap();
    let dest_blkref = instr.operands[oper_idx];
    instr.operands[oper_idx] = Ref::Block(new_blkref);

    // update the dest block preds to be the new block instead of the old block
    let dest_blk = ir.block_mut(dest_blkref.unwrap_block());
    for pred in &mut dest_blk.preds {
      if *pred == blkref {
        *pred = new_blkref;
      }
    }

    new_blkref
  }
}

fn target_has_phis(ir: &IR, b: BlockRef) -> bool {
  for r in ir.iter_instrs(b) {
    if ir.instr(r).unwrap().opcode == Opcode::Phi {
      return true;
    }
  }
  false
}

pub fn finalize(ir: &mut IR) {
  let mut fin = Finalizer::new();
  fin.insert_intermediate_phi_blocks(ir);
}
