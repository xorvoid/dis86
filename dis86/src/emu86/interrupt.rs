use super::machine::*;

impl Machine {
  pub fn interrupt(&mut self, num: u8) {
    match num {
      0x21 => self.interrupt_0x21(),
      _ => panic!("unimplmented interrupt"),
    }
  }

  pub fn interrupt_0x21(&mut self) {
    let func = self.reg(AH);
    match func {
      0x30 => self.dos_get_version(),
      _ => panic!("unimplemented DOS function: {}", func),
    }
  }

  fn dos_get_version(&mut self) {
    self.reg_set(AL, 1);
    self.reg_set(AH, 0);
    // NOTE: Missing other fields
  }
}
