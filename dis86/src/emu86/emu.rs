use crate::binfmt::mz;
use super::machine::Machine;

struct Emulator {
  #[allow(dead_code)]
  exe_path: String,
  exe: mz::Exe,
  machine: Machine,
}

impl Emulator {
  fn new(exe_path: &str) -> Result<Emulator, String> {
    let Ok(data) = std::fs::read(exe_path) else {
      panic!("Failed to read file: {}", exe_path);
    };

    let exe = mz::Exe::decode(&data).unwrap();
    let mut machine = Machine::default();

    machine.load_exe(&exe)?;

    let mut this = Emulator {
      exe_path: exe_path.to_string(),
      exe,
      machine,
    };

    Ok(this)
  }

  fn run(&mut self) -> Result<(), String> {
    while !self.machine.halted() {
      self.machine.step()?;
    }
    self.machine.cpu.dump_state();
    Ok(())
  }
}

pub fn run(exe_path: &str) -> Result<(), String> {
  let mut emu = Emulator::new(exe_path)?;
  emu.run()?;
  Ok(())
}
