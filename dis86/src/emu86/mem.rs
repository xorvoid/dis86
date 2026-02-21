//use crate::emu86::psp::ProgramSegmentPrefix;
pub use crate::segoff::{Seg, SegOff};

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

  pub fn read_u8(&self, addr: SegOff) -> u8  {
    self.0[addr.abs_normal()]
  }

  pub fn read_u16(&self, addr: SegOff) -> u16 {
    let idx = addr.abs_normal();
    u16::from_le_bytes(self.0[idx..idx+2].try_into().unwrap())
  }

  pub fn read_u32(&self, addr: SegOff) -> u32 {
    let idx = addr.abs_normal();
    u32::from_le_bytes(self.0[idx..idx+4].try_into().unwrap())
  }

  pub fn write_u8(&mut self, addr: SegOff, val: u8) {
    self.0[addr.abs_normal()] = val;
  }

  pub fn write_u16(&mut self, addr: SegOff, val: u16) {
    let idx = addr.abs_normal();
    self.0[idx..idx+2].copy_from_slice(&val.to_le_bytes());
  }

  pub fn write_u32(&mut self, addr: SegOff, val: u32) {
    let idx = addr.abs_normal();
    self.0[idx..idx+4].copy_from_slice(&val.to_le_bytes());
  }

  pub fn slice_starting_at(&self, addr: SegOff) -> &[u8] {
    &self.0[addr.abs_normal()..]
  }

  // pub fn program_segment_prefix_mut(&mut self) -> &mut ProgramSegmentPrefix {
  //   let off = PSP_SEGMENT.abs_normal();
  //   let slice = &mut self.0[off..off+256];
  //   ProgramSegmentPrefix::from_slice_mut(slice)
  // }

  // pub fn program_segment_prefix(&self) -> &ProgramSegmentPrefix {
  //   let off = PSP_SEGMENT.abs_normal();
  //   let slice = &self.0[off..off+256];
  //   ProgramSegmentPrefix::from_slice(slice)
  // }
}
