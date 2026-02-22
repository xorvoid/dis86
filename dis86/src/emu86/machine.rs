pub use super::value::*;
pub use super::mem::*;
pub use super::cpu::*;
pub use super::cpu_flags::*;
pub use super::alu;

pub use super::dos::Dos;
pub use crate::segoff:: SegOff;

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

  pub fn stack_push(&mut self, val: Value) {
    let mut addr = self.reg_read_addr(SS, SP);
    addr.off.0 -= 2;

    self.reg_write_u16(SP, addr.off.0);

    self.mem.write_u16(addr, val.unwrap_u16());
  }

  pub fn stack_pop(&mut self) -> Value {
    let addr = self.reg_read_addr(SS, SP);
    let val = self.mem.read_u16(addr);

    self.reg_write_u16(SP, addr.off.0 + 2);

    Value::U16(val)
  }
}
