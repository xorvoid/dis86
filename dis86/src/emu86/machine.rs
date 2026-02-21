pub use super::mem::*;
pub use super::cpu::*;
pub use super::dos::Dos;
pub use crate::segoff:: SegOff;

#[derive(Debug)]
pub enum Value {
  U8(u8),
  U16(u16),
  U32(u32),
  Addr(SegOff),
}

impl Value {
  #[allow(dead_code)]
  pub fn unwrap_u8(&self) -> u8 {
    let Value::U8(val) = self else { panic!("expected Value::U8") };
    *val
  }

  #[allow(dead_code)]
  pub fn unwrap_u16(&self) -> u16 {
    let Value::U16(val) = self else { panic!("expected Value::U16") };
    *val
  }

  #[allow(dead_code)]
  pub fn unwrap_u32(&self) -> u32 {
    let Value::U32(val) = self else { panic!("expected Value::U32") };
    *val
  }

  #[allow(dead_code)]
  pub fn unwrap_addr(&self) -> SegOff {
    let Value::Addr(val) = self else { panic!("expected Value::Addr") };
    *val
  }
}

#[derive(Default)]
pub struct Machine {
  pub halted: bool,
  pub mem: Memory,
  pub cpu: Cpu,
  pub dos: Dos,
}

impl Machine {
  pub fn halted(&self) -> bool {
    self.halted
  }

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

  pub fn stack_push(&mut self, val: Value) {
    let mut addr = self.reg_read_addr(SS, SP);
    addr.off.0 -= 2;

    self.reg_write_u16(SP, addr.off.0);

    self.mem.write_u16(addr, val.unwrap_u16());
  }

  pub fn stack_pop(&mut self) -> Value {
    let mut addr = self.reg_read_addr(SS, SP);
    let val = self.mem.read_u16(addr);

    self.reg_write_u16(SP, addr.off.0 + 2);

    Value::U16(val)
  }
}
