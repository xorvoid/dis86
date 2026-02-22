use super::machine::*;

impl Machine {
  pub fn interrupt(&mut self, num: u8) {
    match num {
      0x1a => self.bios_time_of_day(),
      0x21 => self.dos_interrupt_0x21(),
      _ => panic!("unimplemnted interrupt"),
    }
  }
}

// BIOS
impl Machine {
  fn bios_time_of_day(&mut self) {
    let ah = self.reg_read_u8(AH);
    match ah {
      0x00 => self.bios_get_system_time(),
      _ => panic!("unimplemented time function (int 0x1f): ah=0x{:x}", ah),
    }
  }

  fn bios_get_system_time(&mut self) {
    // FIXME: IMPLEMENT CLOCK SO WE CAN ACTUALLY KEEP TIME!
    self.reg_write_u8(AL, 0);
    self.reg_write_u32(CX, DX, 0);
  }
}
