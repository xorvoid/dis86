pub use super::value::*;
pub use super::mem::*;
pub use super::cpu::*;
pub use super::cpu_flags::*;
pub use super::io::*;
pub use super::adlib::*;
pub use super::alu;

pub use super::dos::Dos;
pub use crate::segoff:: SegOff;

pub struct Machine {
  pub halted: bool,
  pub mem: Memory,
  pub cpu: Cpu,
  pub dos: Dos,
  pub interrupt_vectors: [Option<SegOff>; 256],
  pub adlib: Adlib,
  pub exec_count: u64,
}

impl Machine {
  pub fn new(root_dir: Option<&str>) -> Machine {
    let mut mem = Memory::default();
    let cpu = Cpu::default();

    let dos = Dos::new(root_dir, &mut mem);
    let adlib = Adlib::new();

    Machine {
      halted: false,
      mem, cpu, dos,
      interrupt_vectors: [None; 256],
      adlib,
      exec_count: 0,
    }
  }

  pub fn halted(&self) -> bool {
    self.halted
  }

  pub fn instr_addr(&mut self) -> SegOff {
    SegOff::new(self.reg_read_u16(CS), self.reg_read_u16(IP))
  }

  pub fn stack_push(&mut self, val: Value) {
    self.stack_push_u16(val.unwrap_u16());
  }

  pub fn stack_pop(&mut self) -> Value {
    Value::U16(self.stack_pop_u16())
  }

  pub fn stack_push_u16(&mut self, val: u16) {
    let mut addr = self.reg_read_addr(SS, SP);
    addr.off.0 -= 2;

    self.reg_write_u16(SP, addr.off.0);

    self.mem.write_u16(addr, val);
  }

  pub fn stack_pop_u16(&mut self) -> u16 {
    let addr = self.reg_read_addr(SS, SP);
    let val = self.mem.read_u16(addr);

    self.reg_write_u16(SP, addr.off.0 + 2);

    val
  }
}
