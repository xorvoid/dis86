use crate::emu86::psp::ProgramSegmentPrefix;
pub use crate::segoff::{Seg, Off, SegOff};

// Large enough to allow address ffff:ffff
pub const MEM_SIZE: usize = 0x10fff0;

// Always use a fixed PSP segment
pub const PSP_SEGMENT: Seg = Seg::Normal(0x800);

pub struct Memory(pub Vec<u8>);

impl Default for Memory {
  fn default() -> Self { Self::new() }
}

impl Memory {
  pub fn new() -> Memory {
    let mut raw = vec![];
    raw.resize(MEM_SIZE, 0);
    Memory(raw)
  }

  pub fn slice_starting_at(&self, addr: SegOff) -> &[u8] {
    &self.0[addr.abs_normal()..]
  }

  pub fn program_segment_prefix_mut(&mut self) -> &mut ProgramSegmentPrefix {
    let off = PSP_SEGMENT.abs_normal();
    let slice = &mut self.0[off..off+256];
    ProgramSegmentPrefix::from_slice_mut(slice)
  }

  pub fn program_segment_prefix(&self) -> &ProgramSegmentPrefix {
    let off = PSP_SEGMENT.abs_normal();
    let slice = &self.0[off..off+256];
    ProgramSegmentPrefix::from_slice(slice)
  }
}
