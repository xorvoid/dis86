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

    let exits = blk.exits();
    if exits.len() == 0 { // block returns
      writeln!(f, "  {}_{} -> exit;", src, b.0)?;
      continue;
    }
    for exit in exits {
      let exit_name = &ir.block(exit).name;
      writeln!(f, "  {}_{} -> {}_{};", src, b.0, exit_name, exit.0)?;
    }
  }
  writeln!(f, "}}\n")?;
  Ok(buf)
}
