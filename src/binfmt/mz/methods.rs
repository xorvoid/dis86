use crate::binfmt::mz::*;

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
}
