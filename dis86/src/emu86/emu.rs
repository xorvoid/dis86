use crate::binfmt::mz;
use super::machine::Machine;
use super::sdl;
use std::path::Path;

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

pub fn run(exe_path: &str) -> Result<(), String> {
  let mut emu = Emulator::new(exe_path)?;
  emu.run()?;
  Ok(())
}
