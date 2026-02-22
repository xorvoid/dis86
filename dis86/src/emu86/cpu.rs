use super::machine::*;

#[derive(Debug, Clone, Copy)]
pub struct Register { pub idx: u8, pub off: u8, pub size: u8 }

pub const AX:    Register = Register { idx:  0, off: 0, size: 2 };
pub const BX:    Register = Register { idx:  1, off: 0, size: 2 };
pub const CX:    Register = Register { idx:  2, off: 0, size: 2 };
pub const DX:    Register = Register { idx:  3, off: 0, size: 2 };
pub const SI:    Register = Register { idx:  4, off: 0, size: 2 };
pub const DI:    Register = Register { idx:  5, off: 0, size: 2 };
pub const BP:    Register = Register { idx:  6, off: 0, size: 2 };
pub const SP:    Register = Register { idx:  7, off: 0, size: 2 };
pub const IP:    Register = Register { idx:  8, off: 0, size: 2 };
pub const CS:    Register = Register { idx:  9, off: 0, size: 2 };
pub const DS:    Register = Register { idx: 10, off: 0, size: 2 };
pub const ES:    Register = Register { idx: 11, off: 0, size: 2 };
pub const SS:    Register = Register { idx: 12, off: 0, size: 2 };
pub const FLAGS: Register = Register { idx: 13, off: 0, size: 2 };
pub const AL:    Register = Register { idx:  0, off: 0, size: 1 };
pub const AH:    Register = Register { idx:  0, off: 1, size: 1 };
pub const BL:    Register = Register { idx:  1, off: 0, size: 1 };
pub const BH:    Register = Register { idx:  1, off: 1, size: 1 };
pub const CL:    Register = Register { idx:  2, off: 0, size: 1 };
pub const CH:    Register = Register { idx:  2, off: 1, size: 1 };
pub const DL:    Register = Register { idx:  3, off: 0, size: 1 };
pub const DH:    Register = Register { idx:  3, off: 1, size: 1 };

pub struct Cpu {
  pub regs: [u16; 14],
}

impl Default for Cpu {
  fn default() -> Cpu {
    Cpu { regs: [0; 14] }
  }
}

impl Machine {
  // OLD
  pub fn reg(&self, r: Register) -> u16 {
    match self.reg_read(r) {
      Value::U8(val) => val as u16,
      Value::U16(val) => val,
      _ => panic!("unimpl"),
    }
  }

  // OLD
  pub fn reg_set(&mut self, r: Register, val: u16) {
    let v = match r.size {
      1 => Value::U8(val as u8),
      2 => Value::U16(val),
      _ => panic!("unimpl"),
    };
    self.reg_write(r, v)
  }

  pub fn reg_read_u8(&self, r: Register) -> u8 {
    self.reg_read(r).unwrap_u8()
  }

  pub fn reg_read_u16(&self, r: Register) -> u16 {
    self.reg_read(r).unwrap_u16()
  }

  pub fn reg_read_addr(&self, seg: Register, off: Register) -> SegOff {
    let seg = self.reg_read_u16(seg);
    let off = self.reg_read_u16(off);
    SegOff::new(seg, off)
  }

  pub fn reg_read(&self, r: Register) -> Value {
    if r.size == 2 {
      Value::U16(self.cpu.regs[r.idx as usize])
    } else {
      assert!(r.size == 1);
      let val = self.cpu.regs[r.idx as usize];
      let res = if r.off == 0 { val as u8 } else { (val >> 8) as u8 };
      Value::U8(res)
    }
  }

  pub fn reg_write_u8(&mut self, r: Register, val: u8) {
    self.reg_write(r, Value::U8(val))
  }

  pub fn reg_write_u16(&mut self, r: Register, val: u16) {
    self.reg_write(r, Value::U16(val))
  }

  pub fn reg_write_addr(&mut self, seg: Register, off: Register, addr: SegOff) {
    self.reg_write_u16(seg, addr.seg.unwrap_normal());
    self.reg_write_u16(off, addr.off.0);
  }

  pub fn reg_write(&mut self, r: Register, val: Value) {
    if r.size == 2 {
      self.cpu.regs[r.idx as usize] = val.unwrap_u16();
    } else {
      // partial register write combine
      assert!(r.size == 1);
      let val = val.unwrap_u8();
      let cur = self.cpu.regs[r.idx as usize];
      let new = if r.off == 0 {
        (cur & 0xff00) | (val as u8 as u16)
      } else {
        (cur & 0x00ff) | (val as u8 as u16) << 8
      };
      self.cpu.regs[r.idx as usize] = new;
    }
  }
}


impl Cpu {
  pub fn dump_state(&self) {
    println!("CPU State:");
    println!("  AX     0x{:04x}", self.regs[AX.idx as usize]);
    println!("  BX     0x{:04x}", self.regs[BX.idx as usize]);
    println!("  CX     0x{:04x}", self.regs[CX.idx as usize]);
    println!("  DX     0x{:04x}", self.regs[DX.idx as usize]);
    println!("  SI     0x{:04x}", self.regs[SI.idx as usize]);
    println!("  DI     0x{:04x}", self.regs[DI.idx as usize]);
    println!("  BP     0x{:04x}", self.regs[BP.idx as usize]);
    println!("  SP     0x{:04x}", self.regs[SP.idx as usize]);
    println!("  IP     0x{:04x}", self.regs[IP.idx as usize]);
    println!("  CS     0x{:04x}", self.regs[CS.idx as usize]);
    println!("  DS     0x{:04x}", self.regs[DS.idx as usize]);
    println!("  ES     0x{:04x}", self.regs[ES.idx as usize]);
    println!("  SS     0x{:04x}", self.regs[SS.idx as usize]);
    println!("  FLAGS  0x{:04x}", self.regs[FLAGS.idx as usize]);
  }
}
