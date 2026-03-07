use super::machine::*;

impl Machine {
  pub fn video_interrupt_0x10(&mut self) {
    let func = self.reg_read_u8(AH);
    match func {
      0x00 => self.video_set_mode(),
      0x1a => self.video_get_or_set_display_combination_code(),
      _ => panic!("unsupported video function: 0x{:x}", func),
    }
  }

  fn video_set_mode(&mut self) {
    let mode = self.reg_read_u8(AL);
    if mode != 0x13 { panic!("Only video mode 0x13 is supported"); }
  }

  fn video_get_or_set_display_combination_code(&mut self) {
    let get_or_set = self.reg_read_u8(AL);
    match get_or_set {
      0 => { // get
        // NOTE: JUST TO MATCH DOSBOX
        self.reg_write_u16(AX, 0x1a);
        self.reg_write_u16(BX, 0x8);
      }
      _ => panic!("unimpl"),
    }
  }
}
