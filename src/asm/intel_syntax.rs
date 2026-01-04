use crate::asm::instr::*;
use crate::segoff::SegOff;
use std::fmt::Write;

type Result<T> = std::result::Result<T, std::fmt::Error>;

fn format_operand(s: &mut String, ins: &Instr, oper: &Operand) -> Result<()> {
  match oper {
    Operand::Reg(o) => write!(s, "{}", o.0.name())?,
    Operand::Mem(o) => {
      match o.sz {
        Size::Size8  => write!(s, "BYTE PTR ")?,
        Size::Size16 => write!(s, "WORD PTR ")?,
        Size::Size32 => write!(s, "DWORD PTR ")?,
      };
      write!(s, "{}:", o.sreg.name())?;

      if o.reg1.is_none() && o.reg2.is_none() {
        if o.off.is_some() {
          write!(s, "0x{:x}", o.off.unwrap())?;
        }
      } else {
        write!(s, "[")?;
        if o.reg1.is_some() { write!(s, "{}", o.reg1.unwrap().name())?; }
        if o.reg2.is_some() { write!(s, "+{}", o.reg2.unwrap().name())?; }
        if o.off.is_some() {
          let disp = o.off.unwrap() as i16;
          if disp >= 0 { write!(s, "+0x{:x}", disp as u16)?; }
          else         { write!(s, "-0x{:x}", (-disp) as u16)?; }
        }
        write!(s, "]")?;
      }
    }
    Operand::Imm(o) => write!(s, "0x{:x}", o.val)?,
    Operand::Rel(o) => {
      let effective = ins.rel_addr(o);
      write!(s, "0x{:x}", effective.off.0)?;
    }
    Operand::Far(o) => write!(s, "0x{:x}:0x{:x}", o.seg, o.off)?,
  };

  Ok(())
}

fn format_instr_impl(s: &mut String, ins: &Instr, bytes: &[u8], with_detail: bool) -> Result<()> {
  if with_detail {
    write!(s, "{}:\t", ins.addr)?;
    for b in bytes {
      write!(s, "{:02x} ", b)?;
    }
    let used = ins.n_bytes as usize * 3;
    let remain = if used <= 21 { 21 - used } else { 0 };
    write!(s, "{:1$}\t", "", remain)?;
  }

  match ins.rep {
    None => (),
    Some(Rep::NE) => write!(s, "repne ")?,
    Some(Rep::EQ) => write!(s, "rep ")?,
  }

  write!(s, "{:<5}", ins.opcode.name())?;

  let mut first = true;
  for (i, oper) in ins.operands.as_slice().iter().enumerate() {
    if ((1u8<<i) & ins.intel_hidden_operand_bitmask) != 0 {
      continue;
    }
    if first { write!(s, "  ")?; }
    else { write!(s, ",")?; }
    format_operand(s, ins, oper)?;
    first = false;
  }
  Ok(())
}

fn format_data_impl(s: &mut String, addr: SegOff, bytes: &[u8], with_detail: bool) -> Result<()> {
  if with_detail {
    write!(s, "{}:\t", addr)?;
    for b in bytes {
      write!(s, "{:02x} ", b)?;
    }
    let used = bytes.len() as usize * 3;
    let remain = if used <= 21 { 21 - used } else { 0 };
    write!(s, "{:1$}\t", "", remain)?;
  }
  write!(s, "(data)")?;
  Ok(())
}

// FIXME: THIS IS KLUDGY
pub fn format(addr: SegOff, ins: Option<&Instr>, bytes: &[u8], with_detail: bool) -> Result<String> {
  let mut s = String::new();
  match ins {
    Some(ins) => format_instr_impl(&mut s, ins, bytes, with_detail)?,
    None => format_data_impl(&mut s, addr, bytes, with_detail)?,
  }
  Ok(s.trim_end().to_string())
}
