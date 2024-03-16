use crate::ir;
use std::collections::HashMap;
use std::fmt::Write;

pub fn compute_uses(ir: &ir::IR) -> HashMap<ir::Ref, usize> {
  let mut n_uses = HashMap::new();
  for b in ir.iter_blocks() {
    for r in ir.iter_instrs(b) {
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
  writeln!(f, "strict digraph control_flow {{")?;
  for b in ir.iter_blocks() {
    let blk = ir.block(b);
    let src = &blk.name;
    let instr = blk.instrs.last().unwrap();
    match instr.opcode {
      ir::Opcode::RetFar | ir::Opcode::RetNear  => {
        writeln!(f, "  {}_{} -> exit;", src, b.0)?;
      }
      ir::Opcode::Jmp => {
        let tgt = instr.operands[0].unwrap_block();
        let tgt_name = &ir.block(tgt).name;
        writeln!(f, "  {}_{} -> {}_{};", src, b.0, tgt_name, tgt.0)?;
      }
      ir::Opcode::Jne => {
        let true_tgt = instr.operands[1].unwrap_block();
        let true_tgt_name = &ir.block(true_tgt).name;
        let false_tgt = instr.operands[2].unwrap_block();
        let false_tgt_name = &ir.block(false_tgt).name;
        writeln!(f, "  {}_{} -> {}_{};", src, b.0, true_tgt_name, true_tgt.0)?;
        writeln!(f, "  {}_{} -> {}_{};", src, b.0, false_tgt_name, false_tgt.0)?;
      }
      ir::Opcode::JmpTbl => {
        for dst in &instr.operands[1..] {
          let tgt = dst.unwrap_block();
          let tgt_name = &ir.block(tgt).name;
          writeln!(f, "  {}_{} -> {}_{};", src, b.0, tgt_name, tgt.0)?;
        }
      }
      _ => panic!("Expected last instruction to be a branching instruction: {:?}", instr),
    }
  }
  writeln!(f, "}}\n")?;
  Ok(buf)
}
