
#[derive(Debug, Clone, Copy)]
pub struct Register { pub idx: u8, pub size: u8 }

pub const AX:    Register = Register { idx:  0, size: 2 };
pub const BX:    Register = Register { idx:  1, size: 2 };
pub const CX:    Register = Register { idx:  2, size: 2 };
pub const DX:    Register = Register { idx:  3, size: 2 };
pub const SI:    Register = Register { idx:  4, size: 2 };
pub const DI:    Register = Register { idx:  5, size: 2 };
pub const BP:    Register = Register { idx:  6, size: 2 };
pub const SP:    Register = Register { idx:  7, size: 2 };
pub const IP:    Register = Register { idx:  8, size: 2 };
pub const CS:    Register = Register { idx:  9, size: 2 };
pub const DS:    Register = Register { idx: 10, size: 2 };
pub const ES:    Register = Register { idx: 11, size: 2 };
pub const SS:    Register = Register { idx: 12, size: 2 };
pub const FLAGS: Register = Register { idx: 13, size: 2 };

pub struct Cpu {
  pub regs: [u16; 14],
}

impl Default for Cpu {
  fn default() -> Cpu {
    Cpu { regs: [0; 14] }
  }
}

impl Cpu {
  pub fn dump_state(&self) {
    println!("CPU State:");
    println!("  AX     0x{:x}", self.regs[AX.idx as usize]);
    println!("  BX     0x{:x}", self.regs[BX.idx as usize]);
    println!("  CX     0x{:x}", self.regs[CX.idx as usize]);
    println!("  DX     0x{:x}", self.regs[DX.idx as usize]);
    println!("  SI     0x{:x}", self.regs[SI.idx as usize]);
    println!("  DI     0x{:x}", self.regs[DI.idx as usize]);
    println!("  BP     0x{:x}", self.regs[BP.idx as usize]);
    println!("  SP     0x{:x}", self.regs[SP.idx as usize]);
    println!("  IP     0x{:x}", self.regs[IP.idx as usize]);
    println!("  CS     0x{:x}", self.regs[CS.idx as usize]);
    println!("  DS     0x{:x}", self.regs[DS.idx as usize]);
    println!("  ES     0x{:x}", self.regs[ES.idx as usize]);
    println!("  SS     0x{:x}", self.regs[SS.idx as usize]);
    println!("  FLAGS  0x{:x}", self.regs[FLAGS.idx as usize]);
  }
}
