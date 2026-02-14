use crate::ir;
use std::fmt::Write;

pub fn gen_graphviz_dotfile(ir: &ir::IR) -> Result<String, std::fmt::Error> {
  let mut buf = String::new();
  let f = &mut buf;
  writeln!(f, "strict digraph control_flow {{")?;
  for b in ir.iter_blocks() {
    let blk = ir.block(b);
    let src = &blk.name;

    let exits = ir.block_exits(b);
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
