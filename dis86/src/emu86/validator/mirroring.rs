use super::super::emu::Emu;
use super::super::cpu::*;
use crate::segoff::{SegOff, Seg, Off};

// FIXME: BUILD THIS UP STATICALLY, RATHER THAN ON EACH ITER
fn build_entries(code_seg: Seg) -> Vec<Entry> {
  let code = code_seg.unwrap_normal();
  vec![
    // Reads the timestamp, non-deterministic
    Entry::new_addr(code+0x0000, 0x010b, vec![CX, DX]),

    // overlay_0005 (HACKY, FIXME)
    Entry::new_addr(code+0x1d10, 0x001e, vec![BX]),

    // Ignore result of "in al,dx", used to verify that opl hardware exists
    // Timing can very significantly and non-deterministically
    Entry::new_range(code+0x0bb4, 0x00ae, 0x00b3, vec![AX]),
    Entry::new_range(code+0x0bb4, 0x00b9, 0x00db, vec![AX]),
    Entry::new_addr( code+0x0bb4, 0x0165, vec![AX]),
    Entry::new_addr( code+0x0bb4, 0x0168, vec![AX]),

    // HACKY work around for mouse interrupt... TODO: IMPL PROPERLY
    Entry::new_addr( code+0x0454, 0x000e, vec![AX, BX]),

    // Reads current system data (non-deterministic)
    Entry::new_addr( code+0x0000, 0x0391, vec![AX, CX, DX]),

    // Reads current system time (non-deterministic)
    Entry::new_addr( code+0x0000, 0x03a4, vec![CX, DX]),

    // Confusing setting of PF on an unsigned multiplys (FIXME)
    Entry::new_addr( code+0x0000, 0x067f, vec![FLAGS]),
    Entry::new_addr( code+0x0000, 0x0689, vec![FLAGS]),
  ]
}

#[derive(Clone, Copy)]
enum Loc {
  Addr(SegOff),
  Range(Seg, Off, Off), // Inclusive
}

struct Entry {
  loc: Loc,
  regs: Vec<Register>,
}

impl Entry {
  fn new_addr(seg: u16, off: u16, regs: Vec<Register>) -> Entry {
    Entry { loc: Loc::Addr(SegOff::new(seg, off)), regs }
  }
  fn new_range(seg: u16, start_off: u16, end_off: u16, regs: Vec<Register>) -> Entry {
    Entry { loc: Loc::Range(Seg::Normal(seg), Off(start_off), Off(end_off)), regs }
  }
}

impl Loc {
  fn matches(&self, addr: SegOff) -> bool {
    match *self {
      Loc::Addr(segoff) => {
        segoff == addr
      }
      Loc::Range(seg, start_off, end_off) => {
        if seg != addr.seg { return false; }
        start_off <= addr.off && addr.off <= end_off
      }
    }
  }
}

pub fn apply_overrides(addr: SegOff, emu: &mut dyn Emu, hydra_state: &Cpu) {
  let entries = build_entries(emu.code_load_seg());
  for entry in &entries {
    if entry.loc.matches(addr) {
      for reg in entry.regs.iter().cloned() {
        emu.reg_write(reg, hydra_state.reg_read(reg));
      }
      return;
    }
  }
}
