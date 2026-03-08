use crate::binfmt::mz;
use super::machine::Machine;
use super::cpu::{Cpu, Register};
use super::value::Value;
use super::sdl;
use std::path::Path;
use crate::segoff::{Seg, SegOff};

pub struct Emulator {
  #[allow(dead_code)]
  exe_path: String,
  #[allow(dead_code)]
  exe: mz::Exe,
  pub machine: Machine,
  app: sdl::App,
  step_count: u64,
}

impl Emulator {
  pub fn new(exe_path: &str) -> Result<Emulator, String> {
    let Ok(data) = std::fs::read(exe_path) else {
      panic!("Failed to read file: {}", exe_path);
    };
    let exe = mz::Exe::decode(&data).unwrap();

    // As the filesystem rootdir, use the root dir of the exe
    let root_dir = Path::new(exe_path).parent().unwrap().to_str().unwrap();

    // Init the machine and load up the program
    let mut machine = Machine::new(Some(root_dir));
    machine.load_exe(&exe)?;

    let app = sdl::App::new();

    Ok(Emulator {
      exe_path: exe_path.to_string(),
      exe,
      machine,
      app,
      step_count: 0,
    })
  }

  pub fn step(&mut self) -> Result<(), String> {
    // Avoid updating SDL on every instruction ... too slow
    // FIXME: Add a proper time-based update (try to maintain some update Hz)
    if self.step_count % (1<<8) == 0 {
      let quit = self.app.update()?;
      if quit { return Err(format!("SDL Exited")) };
    }
    self.machine.step()?;
    self.step_count += 1;
    Ok(())
  }

  fn run(&mut self) -> Result<(), String> {
    while !self.machine.halted() {
      self.step()?;
    }
    println!("CPU State:");
    println!("{}", self.machine.cpu);
    Ok(())
  }
}

// A generic trait to make a unified interface for doing validation / comparisons of two very
// different implementations
pub trait Emu {
  fn step(&mut self) -> Result<(), String>;
  fn cpu_state(&self) -> Cpu;
  fn instr_addr(&self) -> SegOff;

  fn reg_read(&self, reg: Register) -> Value;
  fn reg_write(&mut self, reg: Register, val: Value);

  fn mem_slice(&self, addr: SegOff, len: u32) -> &[u8];

  fn interrupt_handler(&self, vector: u8) -> Option<SegOff>;

  fn code_load_seg(&self) -> Seg {
    Seg::Normal(0x823)
  }
}

impl Emu for Emulator {
  fn step(&mut self) -> Result<(), String> {
    Self::step(self)
  }
  fn cpu_state(&self) -> Cpu {
    self.machine.cpu.clone()
  }
  fn instr_addr(&self) -> SegOff {
    self.machine.instr_addr()
  }
  fn reg_read(&self, reg: Register) -> Value {
    self.machine.reg_read(reg)
  }
  fn reg_write(&mut self, reg: Register, val: Value) {
    self.machine.reg_write(reg, val)
  }
  fn mem_slice(&self, addr: SegOff, len: u32) -> &[u8] {
    &self.machine.mem.slice_starting_at(addr)[..len as usize]
  }
  fn interrupt_handler(&self, vector: u8) -> Option<SegOff> {
    self.machine.interrupt_vectors[vector as usize]
  }
}

pub fn run(exe_path: &str) -> Result<(), String> {
  let mut emu = Emulator::new(exe_path)?;
  emu.run()?;
  Ok(())
}
