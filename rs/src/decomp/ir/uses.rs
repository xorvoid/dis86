use crate::decomp::ir;
use std::collections::HashMap;

pub fn compute_uses(ir: &ir::IR) -> HashMap<ir::Ref, usize> {
  let mut n_uses = HashMap::new();
  for b in 0..ir.blocks.len() {
    for i in ir.blocks[b].instrs.range() {
      let r = ir::Ref::Instr(ir::BlockRef(b), i);
      let instr = ir.instr(r).unwrap();
      for oper in &instr.operands {
        *n_uses.entry(*oper).or_insert(0) += 1;
      }
    }
  }
  n_uses
}
