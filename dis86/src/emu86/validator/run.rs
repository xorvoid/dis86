use super::super::emu::Emulator;
use super::super::cpu::*;
use super::hydra_process::HydraProcess;
use crate::segoff::SegOff;

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

fn apply_mirroring_overrides(addr: SegOff, emu: &mut Emulator, hydra_state: &Cpu) {
  // Reads the timestamp, non-deterministic
  if addr == SegOff::new(0x823, 0x010d) {
    emu.machine.reg_write_u16(CX, hydra_state.reg_read_u16(CX));
    emu.machine.reg_write_u16(DX, hydra_state.reg_read_u16(DX));
  }

  // overlay_0005 (HACKY, FIXME)
  if addr == SegOff::new(0x2533, 0x21) {
    emu.machine.reg_write_u16(BX, 0xeb); // WHY??
  }

  // Ignore result of "in al,dx", used to verify that opl hardware exists
  // Timing can very significantly and non-deterministically
  if addr.seg.unwrap_normal() == 0xbb4+0x823 && 0xba <= addr.off.0 && addr.off.0 <= 0xdc {
    emu.machine.reg_write_u16(AX, hydra_state.reg_read_u16(AX));
  }
  if addr == SegOff::new(0xbb4+0x823, 0x166) {
    emu.machine.reg_write_u16(AX, hydra_state.reg_read_u16(AX));
  }
  if addr == SegOff::new(0xbb4+0x823, 0x169) {
    emu.machine.reg_write_u16(AX, hydra_state.reg_read_u16(AX));
  }
  if addr.seg.unwrap_normal() == 0xbb4+0x823 && 0xaf <= addr.off.0 && addr.off.0 <= 0xb4 {
    emu.machine.reg_write_u16(AX, hydra_state.reg_read_u16(AX));
  }

  // HACKY work around for mouse interrupt... TODO: IMPL PROPERLY
  if addr == SegOff::new(0x454+0x823, 0x0010) {
    emu.machine.reg_write_u16(AX, hydra_state.reg_read_u16(AX));
    emu.machine.reg_write_u16(BX, hydra_state.reg_read_u16(BX));
  }

  // Reads current system data (non-deterministic)
  if addr == SegOff::new(0x000+0x823, 0x393) {
    emu.machine.reg_write_u16(AX, hydra_state.reg_read_u16(AX));
    emu.machine.reg_write_u16(CX, hydra_state.reg_read_u16(CX));
    emu.machine.reg_write_u16(DX, hydra_state.reg_read_u16(DX));
  }

  // Reads current system time (non-deterministic)
  if addr == SegOff::new(0x000+0x823, 0x3a6) {
    emu.machine.reg_write_u16(CX, hydra_state.reg_read_u16(CX));
    emu.machine.reg_write_u16(DX, hydra_state.reg_read_u16(DX));
  }
}

pub fn run(exe_path: &str) -> Result<(), String> {
  let mut emu = Emulator::new(exe_path)?;
  let mut hydra = HydraProcess::spawn(exe_path)?;

  hydra.begin();

  // loop {
  //   hydra.step();
  // }

  let mut hydra_state_prev = hydra.cpu_state();
  let mut machine_state_prev = emu.machine.cpu.clone();

  loop {
    let hydra_addr = hydra.instr_addr();
    let machine_addr = emu.machine.instr_addr();

    let mut hydra_state = hydra.cpu_state();

    // Check stack (near stack pointer)
    let tos_addr = hydra_state.reg_read_addr(SS, SP);
    //let s_addr = tos_addr.add_offset((-16 as i16) as u16);

    // println!("Top of Stack | hydra: 0x{:x} | emu86: 0x{:x}\n",
    //          hydra.mem.read_u16(tos_addr), machine.mem.read_u16(tos_addr));

    // let hydra_stack = crate::util::hexdump::hexdump(&hydra.mem.slice_starting_at(s_addr)[..32]);
    // let emu86_stack = crate::util::hexdump::hexdump(&machine.mem.slice_starting_at(s_addr)[..32]);
    // if hydra_stack != emu86_stack {
    //   panic!("stack mismatch");
    // }

    // let m = machine.mem.slice_starting_at(addr);
    // println!("Emu86 Stack |\n{}", crate::util::hexdump::hexdump(&m[..32]));

    // Handle mirroring overrides
    // if hydra_addr == SegOff::new(0x823, 0x010d) {
    //   emu.machine.reg_write_u16(CX, hydra_state.reg_read_u16(CX));
    //   emu.machine.reg_write_u16(DX, hydra_state.reg_read_u16(DX));
    // }
    apply_mirroring_overrides(hydra_addr, &mut emu, &hydra_state);
    let mut machine_state = emu.machine.cpu.clone();

    // Clear the AF flag... It just creates problems... its behavior is undefined in
    // a number of cases
    hydra_state.regs[FLAGS.idx as usize]   &= !(1<<4);
    machine_state.regs[FLAGS.idx as usize] &= !(1<<4);

    if machine_state != hydra_state {
      eprintln!("");
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

    emu.step()?;
    hydra.step();
  }
}
