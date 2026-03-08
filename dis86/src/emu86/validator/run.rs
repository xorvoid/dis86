use super::super::emu::{Emu, Emulator};
use super::super::cpu::*;
use super::hydra_process::HydraProcess;
use crate::segoff::SegOff;
use super::mirroring::apply_overrides;

struct Validator {
  hydra: Box<dyn Emu>,
  emu86: Box<dyn Emu>,
}

impl Validator {
  fn new(exe_path: &str) -> Result<Self, String> {
    let emu86_impl = Emulator::new(exe_path)?;
    let hydra_impl = HydraProcess::spawn(exe_path)?;

    Ok(Self {
      hydra: Box::new(hydra_impl),
      emu86: Box::new(emu86_impl),
    })
  }

  fn run(&mut self) -> Result<(), String> {
    loop {
      let hydra_addr = self.hydra.instr_addr();
      let emu86_addr = self.emu86.instr_addr();

      self.hydra.step()?;
      self.emu86.step()?;

      apply_overrides(hydra_addr, self.hydra.as_mut(), self.emu86.as_mut());

      if self.hydra.cpu_state() != self.emu86.cpu_state() {
        self.failure(hydra_addr, emu86_addr);
      }
    }
  }

  fn failure(&mut self, hydra_addr: SegOff, emu86_addr: SegOff) {
    eprintln!("");
    eprintln!("State divergence:");
    eprintln!("  hydra  @  {}", hydra_addr);
    eprintln!("  emu86  @  {}", emu86_addr);
    eprintln!("");
    eprintln!("hydra changes:");
    print_changes(&self.hydra.last_cpu_state(), &self.hydra.cpu_state());
    eprintln!("");
    eprintln!("emu86 changes:");
    print_changes(&self.emu86.last_cpu_state(), &self.emu86.cpu_state());
    eprintln!("");
    eprintln!("hydra state:");
    eprintln!("{}", self.hydra.cpu_state());
    eprintln!("");
    eprintln!("emu86 state:");
    eprintln!("{}", self.emu86.cpu_state());
    eprintln!("");
    panic!("STOP");
  }
}

pub fn run(exe_path: &str) -> Result<(), String> {
  Validator::new(exe_path)?.run()
}


fn print_changes(prev: &Cpu, cur: &Cpu) {
  print_change_reg("AX", AX, prev, cur);
  print_change_reg("BX", BX, prev, cur);
  print_change_reg("CX", CX, prev, cur);
  print_change_reg("DX", DX, prev, cur);
  print_change_reg("SI", SI, prev, cur);
  print_change_reg("DI", DI, prev, cur);
  print_change_reg("BP", BP, prev, cur);
  print_change_reg("SP", SP, prev, cur);
  print_change_reg("IP", IP, prev, cur);
  print_change_reg("CS", CS, prev, cur);
  print_change_reg("DS", DS, prev, cur);
  print_change_reg("ES", ES, prev, cur);
  print_change_reg("SS", SS, prev, cur);
  print_change_reg("FLAGS", FLAGS, prev, cur);
}

fn print_change_reg(name: &str, reg: Register, prev: &Cpu, cur: &Cpu) {
  let prev_val = prev.regs[reg.idx as usize];
  let cur_val  = cur.regs[reg.idx as usize];
  if prev_val != cur_val {
    eprintln!("  {} | 0x{:04x} => 0x{:04x}", name, prev_val, cur_val);
  }
}

#[allow(dead_code)]
fn dump_mem(msg: &str, emu: &dyn Emu, addr: SegOff, len: u32) {
  let mem = emu.mem_slice(addr, len);
  let hex = crate::util::hexdump::hexdump(mem);

  println!("Memdump for '{}'", msg);
  println!("----------------------------------------------");
  println!("{}", hex);
}
