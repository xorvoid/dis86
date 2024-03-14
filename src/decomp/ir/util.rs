use crate::decomp::ir;
use std::collections::HashMap;
use std::fmt::Write;

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

pub fn gen_graphviz_dotfile(ir: &ir::IR) -> Result<String, std::fmt::Error> {
  let mut buf = String::new();
  let f = &mut buf;
  writeln!(f, "digraph control_flow {{")?;
  for b in 0..ir.blocks.len() {
    let blk = &ir.blocks[b];
    let src = &blk.name;
    let instr = blk.instrs.last().unwrap();
    match instr.opcode {
      ir::Opcode::Ret => {
        writeln!(f, "  {}_{} -> exit;", src, b)?;
      }
      ir::Opcode::Jmp => {
        let tgt = instr.operands[0].unwrap_block().0;
        let tgt_name = &ir.blocks[tgt].name;
        writeln!(f, "  {}_{} -> {}_{};", src, b, tgt_name, tgt)?;
      }
      ir::Opcode::Jne => {
        let true_tgt = instr.operands[1].unwrap_block().0;
        let true_tgt_name = &ir.blocks[true_tgt].name;
        let false_tgt = instr.operands[2].unwrap_block().0;
        let false_tgt_name = &ir.blocks[false_tgt].name;
        writeln!(f, "  {}_{} -> {}_{};", src, b, true_tgt_name, true_tgt)?;
        writeln!(f, "  {}_{} -> {}_{};", src, b, false_tgt_name, false_tgt)?;
      }
      _ => panic!("Expected last instruction to be a branching instruction: {:?}", instr),
    }
  }
  writeln!(f, "}}\n")?;
  Ok(buf)
}
