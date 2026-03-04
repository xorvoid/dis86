use super::machine::*;

impl Machine {
  pub fn mouse_interrupt_0x33(&mut self) {
    // NOTE: JUST TO MATCH DOSBOX
    self.reg_write_u16(AX, 0xffff);
    self.reg_write_u16(BX, 0x3);
  }
}
