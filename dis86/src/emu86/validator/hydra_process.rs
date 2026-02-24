use std::process::{Command, Child, Stdio};
use super::shmdata::ShmData;
use crate::segoff::SegOff;
use crate::{shmdata_read, shmdata_write};
use std::path::Path;

use std::arch::asm;
fn mem_barrier() {
  unsafe { asm!("dsb sy", options(nostack, preserves_flags)) };
}

pub struct HydraProcess {
  hydra: Child,
  data: ShmData,
}

impl HydraProcess {
  pub fn spawn(exe_path: &str) -> Result<HydraProcess, String> {
    let current_exe = std::env::current_exe().unwrap();
    let dir = current_exe.parent().unwrap().parent().unwrap().parent().unwrap().parent().unwrap();
    let exe = Path::new(exe_path);
    let hydra = Command::new(&format!("{}/hydra/src/dosbox-x/src/dosbox-x", dir.display()))
      .args(&[
        "-conf", &format!("{}/hydra/conf/dosbox.conf", dir.display()),
        "-hydra", &format!("{}/hydra/build/src/remote/libhydraremote.so", dir.display()),
        "-hydra-conf", "normal",
        "-c", &format!("mount d {}", exe.parent().unwrap().display()),
        "-c", "D:",
        "-c", &format!("{}", exe.file_name().unwrap().display()),
        "-c", "exit"
      ])
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .spawn()
      .map_err(|_| format!("Failed to execute"))?;

    std::thread::sleep(std::time::Duration::from_millis(1000));

    let mut data = ShmData::attach("/dev/shm/hydra_remote").unwrap();

    Ok(HydraProcess {
      hydra,
      data,

    })
  }

  #[inline(never)]
  fn ensure_init(&mut self) {
    loop {
      mem_barrier();
      let init = shmdata_read!(self.data, init);
      if init != 0 { break };
    }
  }

  pub fn step(&mut self) {
    self.ensure_init();

    let cs = shmdata_read!(self.data, cs);
    let ip = shmdata_read!(self.data, ip);
    let addr = SegOff::new(cs, ip);

    let ack = shmdata_read!(self.data, ack);
    let next_ack = ack + 1;

    mem_barrier();
    shmdata_write!(self.data, req, next_ack);
    mem_barrier();

    loop {
      if next_ack == shmdata_read!(self.data, ack) {
        break;
      }
    }
  }
}

impl Drop for HydraProcess {
  fn drop(&mut self) {
    shmdata_write!(self.data, end, 1);
    let _ = self.hydra.kill();
  }
}
