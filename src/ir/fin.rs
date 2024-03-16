use crate::ir::def::*;

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
    for b in 0..ir.blocks.len() {
      let blkref = BlockRef(b);
      let r = Ref::Instr(blkref, ir.blocks[b].instrs.last_idx().unwrap());
      let instr = ir.instr(r).unwrap().clone();
      if instr.opcode != Opcode::Jne { continue; }
      if target_has_phis(ir, instr.operands[1].unwrap_block()) {
        self.insert_block(ir, blkref, r, 1);
      }
      if target_has_phis(ir, instr.operands[2].unwrap_block()) {
        self.insert_block(ir, blkref, r, 2);
      }
    }
  }

  fn insert_block(&mut self, ir: &mut IR, blkref: BlockRef, r: Ref, oper_idx: usize) {
    // unique number for block name
    let num = self.num;
    self.num += 1;

    // generate new block
    let mut new_blk = Block::new(&format!("phi_{:04}", num));
    new_blk.sealed = true;

    // determine the blkref
    let new_blkref = BlockRef(ir.blocks.len());

    // update the jne instruction to jump to the new blk
    let instr = ir.instr_mut(r).unwrap();
    let dest_blkref = instr.operands[oper_idx];
    instr.operands[oper_idx] = Ref::Block(new_blkref);

    // have the new block jump to the original destination
    new_blk.instrs.push_back(Instr {
      opcode: Opcode::Jmp,
      operands: vec![dest_blkref],
    });

    // add preds to new blk
    new_blk.preds.push(blkref);

    // append the block to the ir
    ir.blocks.push(new_blk);

    // update the dest block preds to be the new block instead of the old block
    let dest_blk = &mut ir.blocks[dest_blkref.unwrap_block().0];
    for pred in &mut dest_blk.preds {
      if *pred == blkref {
        *pred = new_blkref;
      }
    }
  }
}

fn target_has_phis(ir: &IR, b: BlockRef) -> bool {
  for i in ir.blocks[b.0].instrs.range() {
    let r = Ref::Instr(b, i);
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
