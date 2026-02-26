use super::super::emu::Emulator;
use super::super::cpu::*;
use super::hydra_process::HydraProcess;

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

pub fn run(exe_path: &str) -> Result<(), String> {
  let mut emu = Emulator::new(exe_path)?;
  let     machine = &mut emu.machine;
  let mut hydra = HydraProcess::spawn(exe_path)?;

  hydra.begin();

  // loop {
  //   hydra.step();
  // }

  let mut hydra_state_prev = hydra.cpu_state();
  let mut machine_state_prev = machine.cpu.clone();

  loop {
    let mut hydra_state = hydra.cpu_state();
    let mut machine_state = machine.cpu.clone();

    // Clear the AF flag... It just creates problems... its behavior is undefined in
    // a number of cases
    hydra_state.regs[FLAGS.idx as usize]   &= !(1<<4);
    machine_state.regs[FLAGS.idx as usize] &= !(1<<4);

    if machine_state != hydra_state {
      let hydra_addr = hydra.instr_addr();
      let machine_addr = machine.instr_addr();
      eprintln!("State divergence:");
      eprintln!("  hydra  @  {}", hydra_addr);
      eprintln!("  emu86  @  {}", machine_addr);
      eprintln!("");
      eprintln!("hydra changes:");
      print_changes(&hydra_state_prev, &hydra_state);
      eprintln!("");
      eprintln!("emu86 changes:");
      print_changes(&machine_state_prev, &machine_state);
      eprintln!("");
      eprintln!("hydra state:");
      eprintln!("{}", hydra_state);
      eprintln!("");
      eprintln!("emu86 state:");
      eprintln!("{}", machine_state);
      eprintln!("");
      panic!("STOP");
    }
    hydra_state_prev = hydra_state;
    machine_state_prev = machine_state;

    machine.step()?;
    hydra.step();
  }
}
