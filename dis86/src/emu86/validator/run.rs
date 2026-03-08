use super::super::emu::{Emu, Emulator};
use super::super::cpu::*;
use super::hydra_process::HydraProcess;
use crate::segoff::SegOff;

struct Validator {
  hydra: Box<dyn Emu>,
  emu86: Box<dyn Emu>,
  hydra_state_prev: Cpu,
  emu86_state_prev: Cpu,
}

impl Validator {
  fn new(exe_path: &str) -> Result<Self, String> {
    let mut emu86_impl = Emulator::new(exe_path)?;

    let mut hydra_impl = HydraProcess::spawn(exe_path)?;
    hydra_impl.begin();

    let hydra_state_prev = hydra_impl.cpu_state();
    let emu86_state_prev = emu86_impl.cpu_state();

    Ok(Self {
      hydra: Box::new(hydra_impl),
      emu86: Box::new(emu86_impl),
      hydra_state_prev,
      emu86_state_prev,
    })
  }

  fn run(&mut self) -> Result<(), String> {
    loop {
      let hydra_addr = self.hydra.instr_addr();
      let emu86_addr = self.emu86.instr_addr();

      // let addr = SegOff::new(0, 0);
      // dump_mem("hydra", self.hydra.as_ref(), addr, 16);
      // dump_mem("emu86", self.emu86.as_ref(), addr, 16);

      self.hydra.step()?;
      self.emu86.step()?;

      let (hydra_state, emu86_state) = self.check(
        hydra_addr, emu86_addr);

      self.hydra_state_prev = hydra_state;
      self.emu86_state_prev = emu86_state;
    }
  }

  fn check(&mut self, hydra_addr: SegOff, emu86_addr: SegOff) -> (Cpu, Cpu) {
    let hydra = self.hydra.as_mut();
    let emu86 = self.emu86.as_mut();

    //let mut hydra_state = hydra.cpu_state();

    // Check stack (near stack pointer)
    //let tos_addr = hydra_state.reg_read_addr(SS, SP);
    //let s_addr = tos_addr.add_offset((-16 as i16) as u16);

    let (hydra_state, emu86_state) =
      super::mirroring::apply_overrides(hydra_addr, hydra, emu86);

    // All Good?
    if &hydra_state == &emu86_state {
      return (hydra_state, emu86_state);
    }

    // Failure?
    eprintln!("");
    eprintln!("State divergence:");
    eprintln!("  hydra  @  {}", hydra_addr);
    eprintln!("  emu86  @  {}", emu86_addr);
    eprintln!("");
    eprintln!("hydra changes:");
    print_changes(&self.hydra_state_prev, &hydra_state);
    eprintln!("");
    eprintln!("emu86 changes:");
    print_changes(&self.emu86_state_prev, &emu86_state);
    eprintln!("");
    eprintln!("hydra state:");
    eprintln!("{}", hydra_state);
    eprintln!("");
    eprintln!("emu86 state:");
    eprintln!("{}", emu86_state);
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

fn dump_mem(msg: &str, emu: &dyn Emu, addr: SegOff, len: u32) {
  let mem = emu.mem_slice(addr, 16);
  let hex = crate::util::hexdump::hexdump(mem);

  println!("Memdump for '{}'", msg);
  println!("----------------------------------------------");
  println!("{}", hex);
}
