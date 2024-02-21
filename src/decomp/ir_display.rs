use crate::decomp::ir::*;
use std::fmt;
use std::collections::HashMap;

impl fmt::Display for Opcode {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.as_str())
  }
}

struct RefMapper {
  map: HashMap<ValueRef, usize>,
  next: usize,
}

impl RefMapper {
  fn new() -> Self {
    Self {
      map: HashMap::new(),
      next: 0,
    }
  }

  fn map(&mut self, r: ValueRef) -> usize {
    match self.map.get(&r) {
      Some(val) => *val,
      None => {
        let val = self.next;
        self.map.insert(r, val);
        self.next += 1;
        val
      }
    }
  }

  fn fmt(&mut self, ir: &IR, r: ValueRef) -> String {
    match r {
      ValueRef::Const(ConstRef(num)) => format!("#0x{:x}", ir.consts[num]),
      ValueRef::Init(sym) => format!("init({})", sym),
      _ => format!("r{}", self.map(r)),
    }
  }

  fn write_instr(&mut self, f: &mut fmt::Formatter<'_>, ir: &IR, instr: &Instr, iref: ValueRef) -> fmt::Result {
    if instr.opcode == Opcode::Nop {
      return Ok(());
    }
    let s = self.fmt(ir, iref);
    write!(f, "  {:<3} = ", s)?;
    write!(f, "{:<8}", instr.opcode.to_string())?;
    for oper in &instr.operands {
      let s = self.fmt(ir, *oper);
      write!(f, " {:<12}", s)?;
    }
    writeln!(f, "")?;
    Ok(())
  }
}

impl fmt::Display for IR {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut r = RefMapper::new();

    // writeln!(f, "Consts:")?;
    // for (i, val) in self.consts.iter().enumerate() {
    //   writeln!(f, "  c{:<2} = {}", i, val)?;
    // }

    for (i, blk) in self.blocks.iter().enumerate() {
      // if self.is_block_dead(BlockRef(i)) {
      //   continue;
      // }
      writeln!(f, "")?;
      write!(f, "b{i}: (")?;
      for (k, p) in blk.preds.iter().enumerate() {
        if k != 0 {
          write!(f, " ")?;
        }
        write!(f, "b{}", p.0)?;
      }
      writeln!(f, ") {}", blk.name)?;

      for (j, phi) in blk.phis.iter().enumerate() {
        r.write_instr(f, self, phi, ValueRef::Phi(BlockRef(i), PhiRef(j)))?;
      }

      for (j, instr) in blk.instrs.iter().enumerate() {
        r.write_instr(f, self, instr, ValueRef::Instr(BlockRef(i), InstrRef(j)))?;
        //writeln!(f, "{:?}", instr)?;
      }
    }
    Ok(())
  }
}
