use crate::binfmt::mz;
use super::machine::Machine;
use std::path::Path;

pub struct Emulator {
  #[allow(dead_code)]
  exe_path: String,
  #[allow(dead_code)]
  exe: mz::Exe,
  pub machine: Machine,
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
    let mut machine = Machine::new(root_dir);
    machine.load_exe(&exe)?;

    Ok(Emulator {
      exe_path: exe_path.to_string(),
      exe,
      machine,
    })
  }

  fn run(&mut self) -> Result<(), String> {
    while !self.machine.halted() {
      self.machine.step()?;
    }
    println!("CPU State:");
    println!("{}", self.machine.cpu);
    Ok(())
  }
}

pub fn run(exe_path: &str) -> Result<(), String> {
  let mut emu = Emulator::new(exe_path)?;
  emu.run()?;
  Ok(())
}
