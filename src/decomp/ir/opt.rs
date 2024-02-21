use crate::decomp::ir::def::*;
use std::collections::HashSet;

// Propagate operand through any ref opcodes
fn operand_propagate(ir: &IR, mut r: Ref) -> Ref {
  loop {
    let Some(instr) = ir.instr(r) else { return r };
    if instr.opcode != Opcode::Ref { return r; }
    r = instr.operands[0];
  }
}

pub fn reduce_phi(ir: &mut IR) {
  for b in 0..ir.blocks.len() {
    for i in ir.blocks[b].instrs.range() {
      let r = Ref::Instr(BlockRef(b), i);
      if ir.instr(r).unwrap().opcode != Opcode::Phi { continue; }

      // propagate while checking conditions
      let mut operands = ir.instr(r).unwrap().operands.clone();
      let mut trivial = true;
      let mut single_ref = None;
      for j in 0..operands.len() {
        operands[j] = operand_propagate(ir, operands[j]);
        if operands[j] == r { continue; }
        match &single_ref {
          None => single_ref = Some(operands[j]),
          Some(s) => if *s != operands[j] {
            trivial = false;
          }
        }
      }
      ir.instr_mut(r).unwrap().operands = operands;

      // all operands the same? reduce to a mov
      if trivial {
        let vref = single_ref.unwrap();
        let instr = ir.instr_mut(r).unwrap();
        instr.opcode = Opcode::Ref;
        instr.operands = vec![vref];
      }
    }
  }
}

pub fn reduce_phi_old(ir: &mut IR) {
  for b in 0..ir.blocks.len() {
    for i in ir.blocks[b].instrs.range() {
      let r = Ref::Instr(BlockRef(b), i);
      if ir.instr(r).unwrap().opcode != Opcode::Phi { continue; }

      // propagate while checking conditions
      let mut operands = ir.instr(r).unwrap().operands.clone();
      let mut same = true;
      for j in 0..operands.len() {
        operands[j] = operand_propagate(ir, operands[j]);
        if operands[j] != operands[0] {
          same = false;
        }
      }
      let first = operands[0];
      ir.instr_mut(r).unwrap().operands = operands;

      // all operands the same? reduce to a mov
      if same {
        let instr = ir.instr_mut(r).unwrap();
        instr.opcode = Opcode::Ref;
        instr.operands = vec![first];
      }
    }
  }
}

pub fn value_propagation(ir: &mut IR) {
  for b in 0..ir.blocks.len() {
    for i in ir.blocks[b].instrs.range() {
      let r = Ref::Instr(BlockRef(b), i);
      // Propagate all operands
      let mut operands = ir.instr(r).unwrap().operands.clone();
      for j in 0..operands.len() {
        operands[j] = operand_propagate(ir, operands[j]);
      }
      ir.instr_mut(r).unwrap().operands = operands;
    }
  }
}

pub fn deadcode_elimination(ir: &mut IR) {
  // pass 1: find all uses
  let mut used = HashSet::new();
  for b in 0..ir.blocks.len() {
    for i in ir.blocks[b].instrs.range() {
      let r = Ref::Instr(BlockRef(b), i);
      let instr = ir.instr(r).unwrap();
      // Add instructions with side-effects
      if instr.opcode != Opcode::Nop && instr.opcode.has_no_result() {
        used.insert(r);
      }
      // Add operands
      for oper in &instr.operands {
        used.insert(*oper);
      }
    }
  }
  // pass 2: remove unused
  for b in 0..ir.blocks.len() {
    for i in ir.blocks[b].instrs.range() {
      let r = Ref::Instr(BlockRef(b), i);
      if used.get(&r).is_some() {
        continue;
      }
      // Overwrite the instr to nop
      let instr = ir.instr_mut(r).unwrap();
      instr.opcode = Opcode::Nop;
      instr.operands = vec![];
    }
  }
}

pub fn optimize(ir: &mut IR) {
  // constant_fold(ir);
  // reduce_jne(ir);
  reduce_phi(ir);
  value_propagation(ir);
  deadcode_elimination(ir);
  // jump_propagation(ir);
}
