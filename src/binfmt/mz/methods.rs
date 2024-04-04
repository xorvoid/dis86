use crate::binfmt::mz::*;

impl<'a> Exe<'a> {
  pub fn exe_data(&self) -> &[u8] {
    &self.rawdata[self.exe_start as usize..self.exe_end as usize]
  }
}
