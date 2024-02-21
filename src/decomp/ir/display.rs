use crate::instr;
use crate::decomp::ir::*;
use std::fmt;
use std::collections::HashMap;

impl fmt::Display for Opcode {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.as_str())
  }
}

fn reg_name(r: instr::Reg) -> &'static str {
  match r {
    instr::Reg::AX => "ax",
    instr::Reg::CX => "cx",
    instr::Reg::DX => "dx",
    instr::Reg::BX => "bx",
    instr::Reg::SP => "sp",
    instr::Reg::BP => "bp",
    instr::Reg::SI => "si",
    instr::Reg::DI => "di",
    instr::Reg::AL => "al",
    instr::Reg::CL => "cl",
    instr::Reg::DL => "dl",
    instr::Reg::BL => "bl",
    instr::Reg::AH => "ah",
    instr::Reg::CH => "ch",
    instr::Reg::DH => "dh",
    instr::Reg::BH => "bh",
    instr::Reg::ES => "es",
    instr::Reg::CS => "cs",
    instr::Reg::SS => "ss",
    instr::Reg::DS => "ds",
    instr::Reg::IP => "ip",
    instr::Reg::FLAGS => "flags",
  }
}

struct RefMapper {
  map: HashMap<Ref, usize>,
  next: usize,
}

impl RefMapper {
  fn new(ir: &IR) -> Self {
    let _ = ir;
    Self {
      map: HashMap::new(),
      next: 0,
    }
  }

  fn map(&mut self, r: Ref) -> usize {
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

  fn fmt(&mut self, ir: &IR, r: Ref) -> String {
    match r {
      Ref::Const(ConstRef(num)) => {
        let k = ir.consts[num] as i16;
        if -1024 <= k && k <= 16 {
          format!("#{}", k)
        } else {
          format!("#0x{:x}", k)
        }
      }
      Ref::Init(sym) => format!("{}.0", sym),
      Ref::Block(blk) => format!("b{}", blk.0),
      Ref::Instr(_, _) => {
        if let Some((sym, num)) = ir.instr(r).unwrap().debug {
          format!("{}.{}", reg_name(sym.0), num)
        } else {
          format!("t{}", self.map(r))
        }
      }
    }
  }
}

impl fmt::Display for IR {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut r = RefMapper::new(self);

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

      for idx in blk.instrs.range() {
        let iref = Ref::Instr(BlockRef(i), idx);
        let instr = &blk.instrs[idx];
        if instr.opcode == Opcode::Nop { continue; }

        let s = r.fmt(self, iref);
        if !instr.opcode.has_no_result() {
          write!(f, "  {:<8} = ", s)?;
        } else {
          write!(f, "  {:<11}", "")?;
        }
        write!(f, "{:<10}", instr.opcode.to_string())?;
        for oper in &instr.operands {
          let s = r.fmt(self, *oper);
          write!(f, " {:<12}", s)?;
        }
        writeln!(f, "")?;
      }
    }
    Ok(())
  }
}
