use super::super::emu::Emu;
use super::super::cpu::*;
use crate::segoff::SegOff;

struct Entry {
  seg: u16,
  off: u16,
  regs: Vec<Register>,
}

// FIXME: BUILD THIS UP STATICALLY, RATHER THAN ON EACH ITER
fn get_entries() -> Vec<Entry> {
  vec![
    // Reads the timestamp, non-deterministic
    Entry { seg: 0x0000, off: 0x010b, regs: vec![CX, DX] },

    // overlay_0005 (HACKY, FIXME)
    Entry { seg: 0x1d10, off: 0x1e, regs: vec![BX] },
  ]
}

pub fn apply_overrides(addr: SegOff, emu: &mut dyn Emu, hydra_state: &Cpu) {
  let code_seg = emu.code_load_seg().unwrap_normal();
  for entry in &get_entries() {
    let mirror_addr = SegOff::new(entry.seg + code_seg, entry.off);
    if mirror_addr == addr {
      for reg in entry.regs.iter().cloned() {
        emu.reg_write(reg, hydra_state.reg_read(reg));
      }
      return;
    }
  }

  // // Reads the timestamp, non-deterministic
  // if addr == SegOff::new(0x823, 0x010d) {
  //   emu.machine.reg_write_u16(CX, hydra_state.reg_read_u16(CX));
  //   emu.machine.reg_write_u16(DX, hydra_state.reg_read_u16(DX));
  // }

  // // overlay_0005 (HACKY, FIXME)
  // if addr == SegOff::new(0x2533, 0x21) {
  //   emu.reg_write_u16(BX, 0xeb); // WHY??
  // }

  // Ignore result of "in al,dx", used to verify that opl hardware exists
  // Timing can very significantly and non-deterministically
  if addr.seg.unwrap_normal() == 0xbb4+0x823 && 0xb9 <= addr.off.0 && addr.off.0 <= 0xdb {
    emu.reg_write(AX, hydra_state.reg_read(AX));
  }
  if addr == SegOff::new(0xbb4+0x823, 0x165) {
    emu.reg_write(AX, hydra_state.reg_read(AX));
  }
  if addr == SegOff::new(0xbb4+0x823, 0x168) {
    emu.reg_write(AX, hydra_state.reg_read(AX));
  }
  if addr.seg.unwrap_normal() == 0xbb4+0x823 && 0xae <= addr.off.0 && addr.off.0 <= 0xb3 {
    emu.reg_write(AX, hydra_state.reg_read(AX));
  }

  // HACKY work around for mouse interrupt... TODO: IMPL PROPERLY
  if addr == SegOff::new(0x454+0x823, 0x000e) {
    emu.reg_write(AX, hydra_state.reg_read(AX));
    emu.reg_write(BX, hydra_state.reg_read(BX));
  }

  // Reads current system data (non-deterministic)
  if addr == SegOff::new(0x000+0x823, 0x391) {
    emu.reg_write(AX, hydra_state.reg_read(AX));
    emu.reg_write(CX, hydra_state.reg_read(CX));
    emu.reg_write(DX, hydra_state.reg_read(DX));
  }

  // Reads current system time (non-deterministic)
  if addr == SegOff::new(0x000+0x823, 0x3a4) {
    emu.reg_write(CX, hydra_state.reg_read(CX));
    emu.reg_write(DX, hydra_state.reg_read(DX));
  }
}
