use crate::instr;
use crate::decomp::ir::*;
use std::fmt::{self, Write};
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

impl fmt::Display for Name {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Name::Reg(r) => write!(f, "{}", reg_name(*r)),
      Name::Var(v) => write!(f, "{}", v),
    }
  }
}

pub struct Formatter {
  map: HashMap<Ref, usize>,
  next: usize,
  pub(crate) out: String, // HAX pub(crate)
}

impl Formatter {
  pub fn new() -> Self {
    Self {
      map: HashMap::new(),
      next: 0,
      out: String::new(),
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

  pub fn finish(self) -> String {
    self.out
  }

  pub fn ref_string(&mut self, ir: &IR, r: Ref) -> Result<String, fmt::Error> {
    let mut buf = String::new();
    let f = &mut buf;

    match r {
      Ref::Const(ConstRef(num)) => {
        let k = ir.consts[num] as i16;
        if -1024 <= k && k <= 16 {
          write!(f, "#{}", k)?;
        } else {
          write!(f, "#0x{:x}", k)?;
        }
      }
      Ref::Init(sym) => write!(f, "{}.0", sym)?,
      Ref::Block(blk) => write!(f, "b{}", blk.0)?,
      Ref::Instr(_, _) => {
        if let Some(FullName(sym, num)) = ir.names.get(&r) {
          write!(f, "{}.{}", sym, num)?;
        } else {
          write!(f, "t{}", self.map(r))?;
        }
      }
      Ref::Symbol(sym) => write!(f, "{}", ir.symbols.symbol_name(sym))?,
      Ref::Func(idx) => write!(f, "{}", ir.funcs[idx])?,
    }

    Ok(buf)
  }

  pub fn fmt_blkhdr(&mut self, blkref: BlockRef, blk: &Block) -> fmt::Result {
    writeln!(&mut self.out, "")?;
    write!(&mut self.out, "b{}: (", blkref.0)?;
    for (k, p) in blk.preds.iter().enumerate() {
      if k != 0 {
        write!(&mut self.out, " ")?;
      }
      write!(&mut self.out, "b{}", p.0)?;
    }
    writeln!(&mut self.out, ") {}", blk.name)?;
    Ok(())
  }


  pub fn fmt_instr(&mut self, ir: &IR, dst: Ref, instr: &Instr) -> fmt::Result {
    let s = self.ref_string(ir, dst)?;
    if !instr.opcode.has_no_result() {
      write!(&mut self.out, "  {:<8} = ", s)?;
    } else {
      write!(&mut self.out, "  {:<11}", "")?;
    }
    write!(&mut self.out, "{:<10}", instr.opcode.to_string())?;
    for oper in &instr.operands {
      let s = self.ref_string(ir, *oper)?;
      write!(&mut self.out, " {:<20}", s)?;
    }
    writeln!(&mut self.out, "")?;

    Ok(())
  }

  fn fmt_ir(&mut self, ir: &IR) -> fmt::Result {
    for (i, blk) in ir.blocks.iter().enumerate() {
      // if self.is_block_dead(BlockRef(i)) {
      //   continue;
      // }
      let bref = BlockRef(i);
      self.fmt_blkhdr(bref, blk)?;
      for idx in blk.instrs.range() {
        let iref = Ref::Instr(bref, idx);
        let instr = &blk.instrs[idx];
        if instr.opcode == Opcode::Nop { continue; }
        self.fmt_instr(ir, iref, instr)?;
      }
    }

    Ok(())
  }
}

impl fmt::Display for IR {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut r = Formatter::new();
    r.fmt_ir(self)?;
    write!(f, "{}", r.finish())
  }
}
