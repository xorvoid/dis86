use super::machine::*;

impl Machine {
  pub fn interrupt(&mut self, num: u8) {
    self.interrupt_save();

    // Is there a user-configured handler?
    if let Some(addr) = self.interrupt_vectors[num as usize] {
      return self.reg_write_addr(CS, IP, addr);
    }

    // Otherwise use the default handler
    self.interrupt_restore();
    self.interrupt_default_handler(num);
  }

  pub fn interrupt_save(&mut self) {
    // Push flags
    let mut flags = self.reg_read_u16(FLAGS);
    flags |= 1<<1; // NOTE: JUST TO MATCH DOSBOX ... 1-bit always seems to be set
    self.stack_push_u16(flags);

    // Clear IF, TF
    self.flag_write(FLAG_IF, false);
    self.flag_write(FLAG_TF, false);

    // Push CS
    let cs = self.reg_read(CS);
    self.stack_push(cs);

    // Push IP
    let ip = self.reg_read(IP);
    self.stack_push(ip);
  }

  pub fn interrupt_restore(&mut self) {
    let ip = self.stack_pop();
    let cs = self.stack_pop();
    let flags = self.stack_pop_u16();
    self.reg_write_u16(FLAGS, flags);
    self.reg_write(CS, cs);
    self.reg_write(IP, ip);
  }

  fn interrupt_default_handler(&mut self, num: u8) {
    // Handle default behavior
    match num {
      0x1a => self.bios_time_of_day(),
      0x10 => self.video_interrupt_0x10(),
      0x21 => self.dos_interrupt_0x21(),
      0x33 => self.mouse_interrupt_0x33(),
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
