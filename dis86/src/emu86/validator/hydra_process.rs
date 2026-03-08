use std::process::{Command, Child, Stdio};
use super::super::emu::Emu;
use super::super::cpu::*;
use super::super::value::Value;
use super::shmdata::ShmData;
use super::shmmem::ShmMem;
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
  #[allow(dead_code)]
  pub mem: ShmMem,
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

    let data = ShmData::attach("/dev/shm/hydra_remote").unwrap();
    let mem = ShmMem::attach("/dev/shm/dosbox_mem").unwrap();

    let mut this = HydraProcess {
      hydra,
      data,
      mem,
    };

    this.wait_for_init();

    Ok(this)
  }

  fn wait_for_init(&mut self) {
    loop {
      mem_barrier();
      let init = shmdata_read!(self.data, init);
      if init != 0 { break };
    }
  }

  pub fn step(&mut self) {
    self.wait_for_init();

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

impl Emu for HydraProcess {
  fn step(&mut self) -> Result<(), String> {
    Self::step(self);
    Ok(())
  }
  fn cpu_state(&self) -> Cpu {
    let mut cpu = Cpu::default();
    cpu.regs[AX.idx as usize]    = shmdata_read!(self.data, ax);
    cpu.regs[BX.idx as usize]    = shmdata_read!(self.data, bx);
    cpu.regs[CX.idx as usize]    = shmdata_read!(self.data, cx);
    cpu.regs[DX.idx as usize]    = shmdata_read!(self.data, dx);
    cpu.regs[SI.idx as usize]    = shmdata_read!(self.data, si);
    cpu.regs[DI.idx as usize]    = shmdata_read!(self.data, di);
    cpu.regs[BP.idx as usize]    = shmdata_read!(self.data, bp);
    cpu.regs[SP.idx as usize]    = shmdata_read!(self.data, sp);
    cpu.regs[IP.idx as usize]    = shmdata_read!(self.data, ip);
    cpu.regs[CS.idx as usize]    = shmdata_read!(self.data, cs);
    cpu.regs[DS.idx as usize]    = shmdata_read!(self.data, ds);
    cpu.regs[ES.idx as usize]    = shmdata_read!(self.data, es);
    cpu.regs[SS.idx as usize]    = shmdata_read!(self.data, ss);
    cpu.regs[FLAGS.idx as usize] = shmdata_read!(self.data, flags);

    cpu
  }
  fn instr_addr(&self) -> SegOff {
    mem_barrier();
    let cs = shmdata_read!(self.data, cs);
    let ip = shmdata_read!(self.data, ip);
    SegOff::new(cs, ip)
  }
  fn reg_read(&self, reg: Register) -> Value {
    // FIXME: Inefficent
    self.cpu_state().reg_read(reg)
  }
  fn reg_write(&mut self, _reg: Register, _val: Value) {
    panic!("reg_write unimpl for hydra process");
  }
  fn mem_slice(&self, addr: SegOff, len: u32) -> &[u8] {
    &self.mem.slice_starting_at(addr)[..len as usize]
  }
  fn interrupt_handler(&self, _vector: u8) -> Option<SegOff> {
    panic!("interrupt_handler unimpl for hydra process");
  }
}
