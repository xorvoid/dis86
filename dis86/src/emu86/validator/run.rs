use super::super::emu::{Emu, Emulator};
use super::super::cpu::*;
use super::hydra_process::HydraProcess;
use crate::segoff::SegOff;

struct Validator {
  hydra: Box<dyn Emu>,
  emu86: Box<dyn Emu>,
}

impl Validator {
  fn new(exe_path: &str) -> Result<Self, String> {
    let mut emu86_impl = Emulator::new(exe_path)?;

    let mut hydra_impl = HydraProcess::spawn(exe_path)?;
    hydra_impl.begin();

    Ok(Self {
      hydra: Box::new(hydra_impl),
      emu86: Box::new(emu86_impl),
    })
  }

  fn run(&mut self) -> Result<(), String> {
    let mut hydra_state_prev = self.hydra.cpu_state();
    let mut emu86_state_prev = self.emu86.cpu_state();

    loop {
      let hydra_addr = self.hydra.instr_addr();
      let emu86_addr = self.emu86.instr_addr();

      self.hydra.step()?;
      self.emu86.step()?;

      let (hydra_state, emu86_state) = check(
        hydra_addr, emu86_addr,
        &hydra_state_prev, &emu86_state_prev,
        self.hydra.as_mut(), self.emu86.as_mut());

      hydra_state_prev = hydra_state;
      emu86_state_prev = emu86_state;
    }
  }
}

pub fn run(exe_path: &str) -> Result<(), String> {
  Validator::new(exe_path)?.run()
}

fn check(hydra_addr: SegOff, emu86_addr: SegOff,
         hydra_state_prev: &Cpu, emu86_state_prev: &Cpu,
         hydra: &mut dyn Emu, emu86: &mut dyn Emu) -> (Cpu, Cpu) {

  let mut hydra_state = hydra.cpu_state();

  // Check stack (near stack pointer)
  let tos_addr = hydra_state.reg_read_addr(SS, SP);
  //let s_addr = tos_addr.add_offset((-16 as i16) as u16);

  super::mirroring::apply_overrides(hydra_addr, emu86, &hydra_state);
  let mut emu86_state = emu86.cpu_state();

  // Clear the AF flag... It just creates problems... its behavior is undefined in
  // a number of cases
  hydra_state.regs[FLAGS.idx as usize]   &= !(1<<4);
  emu86_state.regs[FLAGS.idx as usize] &= !(1<<4);

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
  print_changes(&hydra_state_prev, &hydra_state);
  eprintln!("");
  eprintln!("emu86 changes:");
  print_changes(&emu86_state_prev, &emu86_state);
  eprintln!("");
  eprintln!("hydra state:");
  eprintln!("{}", hydra_state);
  eprintln!("");
  eprintln!("emu86 state:");
  eprintln!("{}", emu86_state);
  eprintln!("");
  panic!("STOP");
}

fn print_change_reg(name: &str, reg: Register, prev: &Cpu, cur: &Cpu) {
  let prev_val = prev.regs[reg.idx as usize];
  let cur_val  = cur.regs[reg.idx as usize];
  if prev_val != cur_val {
    eprintln!("  {} | 0x{:04x} => 0x{:04x}", name, prev_val, cur_val);
  }
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
