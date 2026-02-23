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
      0x25 => self.dos_set_interrupt_vector(),
      0x30 => self.dos_get_version(),
      0x35 => self.dos_get_interrupt_vector(),
      0x40 => self.dos_write_to_file(),
      0x4a => self.dos_mem_resize(),
      0x4c => self.dos_exit_program(),
      _ => panic!("unimplemented DOS function: {}", func),
    }
  }

  // func: 0x25
  fn dos_set_interrupt_vector(&mut self) {
    let idx = self.reg_read_u8(AL);
    let addr = self.reg_read_addr(DS, DX);
    self.dos.interrupt_vectors[idx as usize] = addr;
  }

  // func: 0x30
  fn dos_get_version(&mut self) {
    self.reg_write_u8(AL, 2); // major version
    self.reg_write_u8(AH, 0); // minor version
    // NOTE: Missing other fields
  }

  // func: 0x35
  fn dos_get_interrupt_vector(&mut self) {
    let idx = self.reg_read_u8(AL);
    let addr = self.dos.interrupt_vectors[idx as usize];
    self.reg_write_addr(ES, BX, addr);
  }

  // func: 0x40
  fn dos_write_to_file(&mut self) {
    let bx = self.reg_read_u16(BX);  // Handle
    let cx = self.reg_read_u16(CX);  // Num Bytes To Write
    let ds_dx = self.reg_read_addr(DS, DX); // Buffer Address

    if bx != 2 {
      panic!("expected stderr");
    }

    for i in 0..(cx as usize) {
      let addr = ds_dx.add_offset(i as u16);
      let byte = self.mem.read_u8(addr);
      let ch = char::from_u32(byte as u32).unwrap();
      eprint!("{}", ch);
    }
    eprintln!("");
  }

  // func: 0x4a
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

  // func: 0x4c
  fn dos_exit_program(&mut self) {
    let al = self.reg_read_u8(AL);
    panic!("Exited with {}", al);
  }
}
