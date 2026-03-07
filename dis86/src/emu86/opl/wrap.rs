use super::bind::*;

pub struct Opl3(*mut opl3_chip);

impl Opl3 {
  pub fn new(samplerate: u32) -> Opl3 {
    let ctx = unsafe { OPL3_New() };
    unsafe { OPL3_Reset(ctx, samplerate) };
    Opl3(ctx)
  }

  pub fn write_reg(&mut self, reg: u16, val: u8) {
    unsafe{ OPL3_WriteReg(self.0, reg, val) };
  }

  pub fn write_reg_buffered(&mut self, reg: u16, val: u8) {
    unsafe{ OPL3_WriteRegBuffered(self.0, reg, val) };
  }
}

impl Drop for Opl3 {
  fn drop(&mut self) {
    unsafe { OPL3_Delete(self.0) };
  }
}
