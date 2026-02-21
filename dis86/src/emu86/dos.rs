use super::machine::*;
use super::dos_ivt::*;

pub struct Dos {
  pub interrupt_vectors: [SegOff; 256],
}

impl Default for Dos {
  fn default() -> Dos {
    Dos {
      interrupt_vectors: [SegOff::new(0, 0); 256],
    }
  }
}

impl Machine {
  pub fn dos_interrupt_0x21(&mut self) {
    let func = self.reg_read_u8(AH);
    match func {
      0x25 => self.dos_get_version(),
      0x30 => self.dos_get_version(),
      0x35 => self.dos_get_interrupt_vector(),
      _ => panic!("unimplemented DOS function: {}", func),
    }
  }

  fn dos_set_interrupt_vector(&mut self) {
    let idx = self.reg_read_u8(AL);
    let addr = self.reg_read_addr(DS, DX);
    self.dos.interrupt_vectors[idx as usize] = addr;
  }

  fn dos_get_version(&mut self) {
    self.reg_write_u8(AL, 2); // major version
    self.reg_write_u8(AH, 0); // minor version
    // NOTE: Missing other fields
  }

  fn dos_get_interrupt_vector(&mut self) {
    let idx = self.reg_read_u8(AL);
    let addr = self.dos.interrupt_vectors[idx as usize];
    self.reg_write_addr(ES, BX, addr);
  }
}
