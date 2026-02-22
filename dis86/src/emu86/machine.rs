pub use super::mem::*;
pub use super::cpu::*;
pub use super::cpu_flags::*;
pub use super::dos::Dos;
pub use crate::segoff:: SegOff;

#[derive(Default)]
pub struct Machine {
  pub halted: bool,
  pub mem: Memory,
  pub cpu: Cpu,
  pub dos: Dos,
}

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

impl Machine {
  pub fn halted(&self) -> bool {
    self.halted
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
