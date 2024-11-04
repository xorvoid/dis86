use crate::segoff::SegOff;
use crate::util::arrayvec::ArrayVec;
pub use crate::asm::instr_fmt::Opcode;

#[derive(Debug, Clone, Copy)]
pub struct Instr {
  pub rep: Option<Rep>,
  pub opcode: Opcode,
  pub operands: ArrayVec<Operand, 3>,
  pub addr: SegOff,
  pub n_bytes: u16,
  pub intel_hidden_operand_bitmask: u8,
}

impl Instr {
  pub fn end_addr(&self) -> SegOff {
    self.addr.add_offset(self.n_bytes)
  }
  pub fn rel_addr(&self, rel: &OperandRel) -> SegOff {
    self.end_addr().add_offset(rel.val)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operand {
  Reg(OperandReg),
  Mem(OperandMem),
  Imm(OperandImm),
  Rel(OperandRel),
  Far(OperandFar),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OperandReg(pub Reg);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OperandMem {
  pub sz: Size,
  pub sreg: Reg,
  pub reg1: Option<Reg>,
  pub reg2: Option<Reg>,
  pub off: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OperandImm {
  pub sz: Size,
  pub val: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OperandRel {
  pub val: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OperandFar {
  pub seg: u16,
  pub off: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Size {
  Size8,
  Size16,
  Size32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rep {
  NE,
  EQ,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Reg {
  AX,
  CX,
  DX,
  BX,
  SP,
  BP,
  SI,
  DI,
  AL,
  CL,
  DL,
  BL,
  AH,
  CH,
  DH,
  BH,
  ES,
  CS,
  SS,
  DS,
  IP,
  FLAGS,
}

impl Reg {
  pub fn reg8(num: u8) -> Reg {
    assert!(num <= 7);
    unsafe { std::mem::transmute(Reg::AL as u8 + num) }
  }

  pub fn reg16(num: u8) -> Reg {
    assert!(num <= 7);
    unsafe { std::mem::transmute(Reg::AX as u8 + num) }
  }

  pub fn sreg16(num: u8) -> Reg {
    assert!(num <= 3);
    unsafe { std::mem::transmute(Reg::ES as u8 + num) }
  }

  pub fn name(&self) -> &'static str {
    match self {
      Reg::AX => "ax",
      Reg::CX => "cx",
      Reg::DX => "dx",
      Reg::BX => "bx",
      Reg::SP => "sp",
      Reg::BP => "bp",
      Reg::SI => "si",
      Reg::DI => "di",
      Reg::AL => "al",
      Reg::CL => "cl",
      Reg::DL => "dl",
      Reg::BL => "bl",
      Reg::AH => "ah",
      Reg::CH => "ch",
      Reg::DH => "dh",
      Reg::BH => "bh",
      Reg::ES => "es",
      Reg::CS => "cs",
      Reg::SS => "ss",
      Reg::DS => "ds",
      Reg::IP => "ip",
      Reg::FLAGS => "flags",
    }
  }
}

pub struct RegInfo {
  pub name: &'static str,
  pub sz: Size,
  pub seg: bool,
}

const REG_INFO: &[RegInfo] = &[
  RegInfo { name: "AX",    sz: Size::Size16, seg: false },
  RegInfo { name: "CX",    sz: Size::Size16, seg: false },
  RegInfo { name: "DX",    sz: Size::Size16, seg: false },
  RegInfo { name: "BX",    sz: Size::Size16, seg: false },
  RegInfo { name: "SP",    sz: Size::Size16, seg: false },
  RegInfo { name: "BP",    sz: Size::Size16, seg: false },
  RegInfo { name: "SI",    sz: Size::Size16, seg: false },
  RegInfo { name: "DI",    sz: Size::Size16, seg: false },
  RegInfo { name: "AL",    sz: Size::Size8,  seg: false },
  RegInfo { name: "CL",    sz: Size::Size8,  seg: false },
  RegInfo { name: "DL",    sz: Size::Size8,  seg: false },
  RegInfo { name: "BL",    sz: Size::Size8,  seg: false },
  RegInfo { name: "AH",    sz: Size::Size8,  seg: false },
  RegInfo { name: "CH",    sz: Size::Size8,  seg: false },
  RegInfo { name: "DH",    sz: Size::Size8,  seg: false },
  RegInfo { name: "BH",    sz: Size::Size8,  seg: false },
  RegInfo { name: "ES",    sz: Size::Size16, seg: true  },
  RegInfo { name: "CS",    sz: Size::Size16, seg: true  },
  RegInfo { name: "SS",    sz: Size::Size16, seg: true  },
  RegInfo { name: "DS",    sz: Size::Size16, seg: true  },
  RegInfo { name: "IP",    sz: Size::Size16, seg: false },
  RegInfo { name: "FLAGS", sz: Size::Size16, seg: false },
];

impl Reg {
  pub fn info(&self) -> &RegInfo {
    let idx = *self as usize;
    assert!(idx < REG_INFO.len());
    &REG_INFO[idx]
  }

  pub fn from_str_upper(s: &str) -> Option<Reg> {
    match s {
      "AX" => Some(Reg::AX),
      "CX" => Some(Reg::CX),
      "DX" => Some(Reg::DX),
      "BX" => Some(Reg::BX),
      "SP" => Some(Reg::SP),
      "BP" => Some(Reg::BP),
      "SI" => Some(Reg::SI),
      "DI" => Some(Reg::DI),
      "AL" => Some(Reg::AL),
      "CL" => Some(Reg::CL),
      "DL" => Some(Reg::DL),
      "BL" => Some(Reg::BL),
      "AH" => Some(Reg::AH),
      "CH" => Some(Reg::CH),
      "DH" => Some(Reg::DH),
      "BH" => Some(Reg::BH),
      "ES" => Some(Reg::ES),
      "CS" => Some(Reg::CS),
      "SS" => Some(Reg::SS),
      "DS" => Some(Reg::DS),
      "IP" => Some(Reg::IP),
      "FLAGS" => Some(Reg::FLAGS),
      _ => None,
    }
  }
}
