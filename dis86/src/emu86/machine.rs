pub use super::mem::*;
pub use super::cpu::*;
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
}

impl Machine {
  pub fn halted(&self) -> bool {
    self.halted
  }

  // OLD
  pub fn reg(&self, r: Register) -> u16 { self.reg_read(r) }

  // OLD
  pub fn reg_set(&mut self, r: Register, val: u16) { self.reg_write(r, val) }

  pub fn reg_read(&self, r: Register) -> u16 {
    if r.size == 2 {
      self.cpu.regs[r.idx as usize]
    } else {
      assert!(r.size == 1);
      let val = self.cpu.regs[r.idx as usize];
      if r.off == 0 { val as u8 as u16 } else { val >> 8 }
    }
  }

  pub fn reg_write(&mut self, r: Register, val: u16) {
    if r.size == 2 {
      self.cpu.regs[r.idx as usize] = val;
    } else {
      // partial register write combine
      assert!(r.size == 1);
      let cur = self.cpu.regs[r.idx as usize];
      let new = if r.off == 0 {
        (cur & 0xff00) | (val as u8 as u16)
      } else {
        (cur & 0x00ff) | (val as u8 as u16) << 8
      };
      self.cpu.regs[r.idx as usize] = new;
    }
  }

  pub fn stack_push(&mut self, val: u16) {
    let ss = self.reg_read(SS);
    let sp = self.reg_read(SP) - 2;
    self.reg_write(SP, sp);

    let addr = SegOff::new_normal(ss, sp);
    self.mem.write_u16(addr, val);
  }

  pub fn stack_pop(&mut self) -> u16 {
    let ss = self.reg_read(SS);
    let sp = self.reg_read(SP);

    let addr = SegOff::new_normal(ss, sp);
    let val = self.mem.read_u16(addr);

    self.reg_write(SP, sp+2);

    val
  }
}
