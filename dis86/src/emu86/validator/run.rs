use crate::emu86::emu::Emulator;
use super::hydra_process::HydraProcess;

pub fn run(exe_path: &str) -> Result<(), String> {
  let mut hydra = HydraProcess::spawn(exe_path)?;

  loop {
    hydra.step();
  }

  Ok(())
}
