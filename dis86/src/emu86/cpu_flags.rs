use super::machine::*;


#[derive(Debug, Clone, Copy)]
pub struct Flag { pub mask: u16, pub shift: u16 }

pub const FLAG_CF: Flag = Flag { mask: 0x0001, shift: 0  };  // Carry
pub const FLAG_PF: Flag = Flag { mask: 0x0004, shift: 2  };  // Parity
pub const FLAG_AF: Flag = Flag { mask: 0x0010, shift: 4  };  // Auxilliary Carry
pub const FLAG_ZF: Flag = Flag { mask: 0x0040, shift: 6  };  // Zero
pub const FLAG_SF: Flag = Flag { mask: 0x0080, shift: 7  };  // Sign
pub const FLAG_TF: Flag = Flag { mask: 0x0100, shift: 8  };  // Trap
pub const FLAG_IF: Flag = Flag { mask: 0x0200, shift: 9  };  // Interrupt Enable
pub const FLAG_DF: Flag = Flag { mask: 0x0400, shift: 10 };  // Direction
pub const FLAG_OF: Flag = Flag { mask: 0x0800, shift: 11 };  // Overflow

#[derive(Debug, Clone, Copy)]
pub struct Flags(pub u16);

impl Flags {
  pub fn get(self, f: Flag) -> bool {
    (self.0 & f.mask) != 0
  }
  pub fn set(&mut self, f: Flag, set: bool) {
    self.0 = (self.0 & !f.mask) | ((set as u16) << f.shift);
  }
}

impl Machine {
  pub fn flag_read_all(&self) -> Flags {
    Flags(self.reg_read_u16(FLAGS))
  }

  pub fn flag_write_all(&mut self, f: Flags) {
    self.reg_write_u16(FLAGS, f.0)
  }

  pub fn flag_read(&self, f: Flag) -> bool {
    let flags = self.flag_read_all();
    flags.get(f)
  }

  pub fn flag_write(&mut self, f: Flag, set: bool) {
    let mut flags = self.flag_read_all();
    flags.set(f, set);
    self.flag_write_all(flags);
  }
}
