use super::machine::*;

impl Machine {
  pub fn interrupt(&mut self, num: u8) {
    match num {
      0x21 => self.dos_interrupt_0x21(),
      _ => panic!("unimplmented interrupt"),
    }
  }
}
