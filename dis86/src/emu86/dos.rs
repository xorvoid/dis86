use super::machine::*;
use super::dos_ivt::*;

pub struct Dos {
  pub interrupt_vectors: [SegOff; 256],
  pub mem_resize_call_count: usize,
}

impl Default for Dos {
  fn default() -> Dos {
    Dos {
      interrupt_vectors: [SegOff::new(0, 0); 256],
      mem_resize_call_count: 0,
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
      0x4a => self.dos_mem_resize(),
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

  fn dos_mem_resize(&mut self) {
    let segment_block = self.reg_read_u16(ES);
    let new_size_par = self.reg_read_u16(BX);

    if self.dos.mem_resize_call_count > 0 {
      panic!("Memory resize is limited to one call on the load seg");
    }
    if segment_block != PSP_SEGMENT.unwrap_normal() {
      panic!("Memory resize is limited to one call on the load seg");
    }

    self.dos.mem_resize_call_count += 1;

    // Success
    self.flag_write(FLAG_CF, false);
  }
}
