use crate::binfmt::mz::*;
use crate::segoff::{Seg, Off, SegOff};

impl<'a> Exe<'a> {
  pub fn exe_data(&self) -> &[u8] {
    &self.rawdata[self.exe_start as usize..self.exe_end as usize]
  }

  pub fn overlay_data(&self, id: usize) -> &[u8] {
    let ovr = self.ovr.as_ref().unwrap();
    let seg = &ovr.segs[id];
    let start = ovr.file_offset as usize + seg.data_offset as usize;
    let end = start + seg.segment_size as usize;
    &self.rawdata[start..end]
  }

  pub fn num_overlay_segments(&self) -> usize {
    self.ovr.as_ref().map(|ovr| ovr.segs.len()).unwrap_or(0)
  }
}

impl SegInfo {
  pub fn size(&self) -> u16 {
    self.maxoff.wrapping_sub(self.minoff)
  }
}

impl OverlayStub {
  pub fn stub_addr(&self) -> SegOff {
    SegOff { seg: Seg::Normal(self.stub_segment), off: Off(self.stub_offset) }
  }

  pub fn dest_addr(&self) -> SegOff {
    SegOff { seg: Seg::Overlay(self.overlay_seg_num), off: Off(self.dest_offset) }
  }
}
