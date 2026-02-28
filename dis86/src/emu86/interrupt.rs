use super::machine::*;

impl Machine {
  pub fn interrupt(&mut self, num: u8) {
    // Handle custom behavior
    if let Some(addr) = self.interrupt_vectors[num as usize] {
      self.interrupt_custom_handler(addr)
    } else {
      self.interrupt_default_handler(num)
    }
  }

  fn interrupt_custom_handler(&mut self, handler_addr: SegOff) {
    // Push flags
    let mut flags = self.reg_read_u16(FLAGS);
    flags |= 1<<1; // NOTE: JUST TO MATCH DOSBOX ... 1-bit always seems to be set
    self.stack_push_u16(flags);

    // Clear IF, TF
    self.flag_write(FLAG_IF, false);
    self.flag_write(FLAG_TF, false);

    // Push CS
    let cs = self.reg_read(CS);
    //println!("pushing cs: 0x{:x}", cs.unwrap_u16());
    self.stack_push(cs);

    // Push IP
    let ip = self.reg_read(IP);
    //println!("pushing ip: 0x{:x}", ip.unwrap_u16());
    self.stack_push(ip);

    // Set handler CS:IP
    self.reg_write_addr(CS, IP, handler_addr);
  }

  fn interrupt_default_handler(&mut self, num: u8) {
    // Handle default behavior
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
