use crate::util::arrayvec::ArrayVec;
use crate::region::RegionIter;
use crate::asm::instr::*;
use crate::asm::instr_fmt;

// Mode is the 2-bits from [6..7] in the ModRM byte
fn modrm_mode(modrm: u8) -> u8 { modrm>>6 }

// Reg is the 3-bits from [3..5] in the ModRM byte
fn modrm_reg(modrm: u8) -> u8 { (modrm>>3)&7 }

// Opcode2 is the same as the reg-field in the ModRM byte
// That is: the 3-bits from [3..5] in the ModRM byte
fn modrm_op2(modrm: u8) -> u8 { (modrm>>3)&7 }

// RM is the 3-bits from [0..2] in the ModRM byte
fn modrm_rm(modrm: u8) -> u8 { modrm&7 }


fn operand_reg(r: Reg) -> Result<Operand, String> {
  Ok(Operand::Reg(OperandReg(r)))
}

fn operand_imm8(imm: u8) -> Result<Operand, String> {
  Ok(Operand::Imm(OperandImm {
    sz: Size::Size8,
    val: imm as u16,
  }))
}

fn operand_imm16(imm: u16) -> Result<Operand, String> {
  Ok(Operand::Imm(OperandImm {
    sz: Size::Size16,
    val: imm,
  }))
}

fn operand_rel(bin: &mut RegionIter, sz: Size) -> Result<Operand, String> {
  let val = match sz {
    Size::Size8 => bin.fetch_sext()?,
    Size::Size16 => bin.fetch_u16()?,
    _ => return Err(format!("Invalid relative addressing size: {:?}", sz)),
  };
  Ok(Operand::Rel(OperandRel { val }))
}

fn operand_src(sz: Size) -> Result<Operand, String> {
  Ok(Operand::Mem(OperandMem {
    sz: sz,
    sreg: Reg::DS, // TODO FIMXE: ARE SEG OVERRIDES ALLOWED FOR THESE??
    reg1: Some(Reg::SI),
    reg2: None,
    off: None,
  }))
}

fn operand_dst(sz: Size) -> Result<Operand, String> {
  Ok(Operand::Mem(OperandMem {
    sz: sz,
    sreg: Reg::ES, // TODO FIMXE: ARE SEG OVERRIDES ALLOWED FOR THESE??
    reg1: Some(Reg::DI),
    reg2: None,
    off: None,
  }))
}

fn operand_far(bin: &mut RegionIter) -> Result<Operand, String> {
  let off = bin.fetch_u16()?;
  let seg = bin.fetch_u16()?;
  Ok(Operand::Far(OperandFar { seg, off }))
}

fn operand_moff(bin: &mut RegionIter, sz: Size, prefix_sreg: Option<Reg>) -> Result<Operand, String> {
  Ok(Operand::Mem(OperandMem {
    sz,
    sreg: prefix_sreg.unwrap_or(Reg::DS),
    reg1: None,
    reg2: None,
    off: Some(bin.fetch_u16()?),
  }))
}

fn operand_rm(bin: &mut RegionIter, sz: Size, modrm: u8, prefix_sreg: Option<Reg>) -> Result<Operand, String> {
  let mode = modrm_mode(modrm);
  let rm = modrm_rm(modrm);

  // Handle special cases first
  if mode == 3 { /* Register mode */
    if sz == Size::Size8 { return operand_reg(Reg::reg8(rm)); }
    else if sz == Size::Size16 { return operand_reg(Reg::reg16(rm)); }
    else { return Err(format!("Only 8-bit and 16-bit registers are allowed")); }
  }
  if mode == 0 && rm == 6 { /* Direct addressing mode: 16-bit */
    return operand_moff(bin, sz, prefix_sreg);
  }

  // Everything else uses some indirect register mode

  // Determine the register set
  let (sreg, reg1, reg2) = match rm {
    0 => (Reg::DS, Reg::BX, Some(Reg::SI)),
    1 => (Reg::DS, Reg::BX, Some(Reg::DI)),
    2 => (Reg::SS, Reg::BP, Some(Reg::SI)),
    3 => (Reg::SS, Reg::BP, Some(Reg::DI)),
    4 => (Reg::DS, Reg::SI, None),
    5 => (Reg::DS, Reg::DI, None),
    6 => (Reg::SS, Reg::BP, None),
    7 => (Reg::DS, Reg::BX, None),
    _ => unreachable!(),
  };

  // Handle immediate dispacements
  let off = match mode {
    0 => None,
    1 => Some(bin.fetch_sext()?),
    2 => Some(bin.fetch_u16()?),
    _ => unreachable!(),
  };

  // Construct the resulting operand
  Ok(Operand::Mem(OperandMem {
    sz,
    sreg: prefix_sreg.unwrap_or(sreg),
    reg1: Some(reg1),
    reg2,
    off,
  }))
}

fn operand_rm8(bin: &mut RegionIter, modrm: u8, sreg: Option<Reg>) -> Result<Operand, String> { operand_rm(bin, Size::Size8, modrm, sreg) }
fn operand_rm16(bin: &mut RegionIter, modrm: u8, sreg: Option<Reg>) -> Result<Operand, String> { operand_rm(bin, Size::Size16, modrm, sreg) }

fn operand_m8(bin: &mut RegionIter, modrm: u8, sreg: Option<Reg>) -> Result<Operand, String> {
  let oper = operand_rm(bin, Size::Size8, modrm, sreg)?;
  if !matches!(oper, Operand::Mem(_)) { return Err(format!("Register used where memory operand was required")); }
  Ok(oper)
}

fn operand_m16(bin: &mut RegionIter, modrm: u8, sreg: Option<Reg>) -> Result<Operand, String> {
  let oper = operand_rm(bin, Size::Size16, modrm, sreg)?;
  if !matches!(oper, Operand::Mem(_)) { return Err(format!("Register used where memory operand was required")); }
  Ok(oper)
}

fn operand_m32(bin: &mut RegionIter, modrm: u8, sreg: Option<Reg>) -> Result<Operand, String> {
  let oper = operand_rm(bin, Size::Size32, modrm, sreg)?;
  if !matches!(oper, Operand::Mem(_)) { return Err(format!("Register used where memory operand was required")); }
  Ok(oper)
}

pub fn decode_one<'a>(bin: &mut RegionIter<'a>) -> Result<Option<(Instr, &'a [u8])>, String> {
  let save_addr = bin.addr();
  let ret = decode_one_impl(bin);
  if ret.is_err() {
    // if error, roll-back any bytes consumed
    bin.reset_addr(save_addr)
  }
  ret
}

pub fn decode_one_impl<'a>(bin: &mut RegionIter<'a>) -> Result<Option<(Instr, &'a [u8])>, String> {
  let start_addr = bin.addr();
  if start_addr == bin.end_addr() {
    return Ok(None);
  }

  // First parse any prefixes
  let mut sreg = None;
  let mut rep = None;
  loop {
    match bin.peek() {
      0x26 => sreg = Some(Reg::ES),
      0x2e => sreg = Some(Reg::CS),
      0x36 => sreg = Some(Reg::SS),
      0x3e => sreg = Some(Reg::DS),
      0xf2 => rep = Some(Rep::NE),
      0xf3 => rep = Some(Rep::EQ),
      _ => break,
    }
    bin.advance();
  }

  // Now parse the main level1 opcode
  let opcode1 = bin.fetch()?;
  let mut opcode2 = None;
  let mut ret = instr_fmt::lookup(opcode1, opcode2);

  // Need a level 2 opcode to do the lookup?
  if ret.err() == Some(instr_fmt::Error::NeedOpcode2) {
    let b = bin.peek_checked()?;
    opcode2 = Some(modrm_op2(b));
    ret = instr_fmt::lookup(opcode1, opcode2);
  } else if ret.err() == Some(instr_fmt::Error::NeedOpcode2Ext0F) {
    let b = bin.peek_checked()?;
    bin.advance();
    opcode2 = Some(b);
    ret = instr_fmt::lookup(opcode1, opcode2);
  }

  // Unpack
  let fmt = match ret {
    Ok(fmt) => fmt,
    Err(_) => return Err(format!("Failed to find instruction fmt for opcode1={:?}, opcode2={:?} at {}", opcode1, opcode2, start_addr)),
  };

  if fmt.op == Opcode::OP_INVAL {
    return Err(format!("Unsupported or invalid instruction at {}", start_addr));
  }

  // Do we need a modrm?
  let mut modrm = 0;
  if fmt.requires_modrm() {
    modrm = bin.fetch()?;
  }

  // Process the format and build up the instruction
  let mut operands = ArrayVec::new();
  for oper in &fmt.oper {
    let operand = match oper {
      // Sentinel
      instr_fmt::Oper::OPER_NONE => break,

      // Implied 16-bit register operands
      instr_fmt::Oper::OPER_AX => operand_reg(Reg::AX),
      instr_fmt::Oper::OPER_CX => operand_reg(Reg::CX),
      instr_fmt::Oper::OPER_DX => operand_reg(Reg::DX),
      instr_fmt::Oper::OPER_BX => operand_reg(Reg::BX),
      instr_fmt::Oper::OPER_SP => operand_reg(Reg::SP),
      instr_fmt::Oper::OPER_BP => operand_reg(Reg::BP),
      instr_fmt::Oper::OPER_SI => operand_reg(Reg::SI),
      instr_fmt::Oper::OPER_DI => operand_reg(Reg::DI),

      // Implied 8-bit register operands
      instr_fmt::Oper::OPER_AL => operand_reg(Reg::AL),
      instr_fmt::Oper::OPER_CL => operand_reg(Reg::CL),
      instr_fmt::Oper::OPER_DL => operand_reg(Reg::DL),
      instr_fmt::Oper::OPER_BL => operand_reg(Reg::BL),
      instr_fmt::Oper::OPER_AH => operand_reg(Reg::AH),
      instr_fmt::Oper::OPER_CH => operand_reg(Reg::CH),
      instr_fmt::Oper::OPER_DH => operand_reg(Reg::DH),
      instr_fmt::Oper::OPER_BH => operand_reg(Reg::BH),

      // Implied segment register operands
      instr_fmt::Oper::OPER_ES => operand_reg(Reg::ES),
      instr_fmt::Oper::OPER_CS => operand_reg(Reg::CS),
      instr_fmt::Oper::OPER_SS => operand_reg(Reg::SS),
      instr_fmt::Oper::OPER_DS => operand_reg(Reg::DS),

      // Implied segment register operands
      instr_fmt::Oper::OPER_FLAGS => operand_reg(Reg::FLAGS),
      instr_fmt::Oper::OPER_LIT1  => operand_imm8(1),
      instr_fmt::Oper::OPER_LIT3  => operand_imm8(3),

      // Implied string operations operands
      instr_fmt::Oper::OPER_SRC8  => operand_src(Size::Size8),
      instr_fmt::Oper::OPER_SRC16 => operand_src(Size::Size16),
      instr_fmt::Oper::OPER_DST8  => operand_dst(Size::Size8),
      instr_fmt::Oper::OPER_DST16 => operand_dst(Size::Size16),

      // Explicit register operands
      instr_fmt::Oper::OPER_R8    => operand_reg(Reg::reg8(modrm_reg(modrm))),
      instr_fmt::Oper::OPER_R16   => operand_reg(Reg::reg16(modrm_reg(modrm))),
      instr_fmt::Oper::OPER_SREG  => operand_reg(Reg::sreg16(modrm_reg(modrm))),

      // Explicit memory operands
      instr_fmt::Oper::OPER_M8    => operand_m8(bin, modrm, sreg),
      instr_fmt::Oper::OPER_M16   => operand_m16(bin, modrm, sreg),
      instr_fmt::Oper::OPER_M32   => operand_m32(bin, modrm, sreg),

      // Explicit register or memory operands (modrm)
      instr_fmt::Oper::OPER_RM8   => operand_rm8(bin, modrm, sreg),
      instr_fmt::Oper::OPER_RM16  => operand_rm16(bin, modrm, sreg),

      // Explicit immediate data
      instr_fmt::Oper::OPER_IMM8      => operand_imm8(bin.fetch()?),
      instr_fmt::Oper::OPER_IMM8_EXT  => operand_imm16(bin.fetch_sext()?),
      instr_fmt::Oper::OPER_IMM16     => operand_imm16(bin.fetch_u16()?),

      // Explicit far32 jump immediate
      instr_fmt::Oper::OPER_FAR32     => operand_far(bin),

      // Explicit 16-bit immediate used as a memory offset into DS
      instr_fmt::Oper::OPER_MOFF8     => operand_moff(bin, Size::Size8, sreg),
      instr_fmt::Oper::OPER_MOFF16    => operand_moff(bin, Size::Size16, sreg),

      // Explicit relative offsets (branching / calls)
      instr_fmt::Oper::OPER_REL8      => operand_rel(bin, Size::Size8),
      instr_fmt::Oper::OPER_REL16     => operand_rel(bin, Size::Size16),
    };
    let operand = operand?; // unwrap and forward any errors
    operands.push(operand);
  }

  let cur_addr = bin.addr();
  let n_bytes = start_addr.offset_to(cur_addr);

  let instr = Instr {
    rep,
    opcode: fmt.op,
    operands,
    addr: start_addr,
    n_bytes,
    intel_hidden_operand_bitmask: fmt.hidden,
  };

  let raw = bin.slice(start_addr, n_bytes);

  Ok(Some((instr, raw)))
}


pub struct Decoder<'a> {
  bin: RegionIter<'a>,
}

impl<'a> Decoder<'a> {
  pub fn new(it: RegionIter<'a>) -> Self {
    Self { bin: it }
  }

  pub fn try_next(&mut self) -> Result<Option<(Instr, &'a [u8])>, String> {
    decode_one(&mut self.bin)
  }
}

impl<'a> Iterator for Decoder<'a> {
  type Item = (Instr, &'a [u8]);
  fn next(&mut self) -> Option<(Instr, &'a [u8])> {
    self.try_next().unwrap()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  struct TestCase {
    addr: usize,
    dat: &'static [u8],
    asm: &'static str
  }

  const TESTS: &[TestCase] = &[
    TestCase { addr: 0x0000, dat: &[0xba, 0xa7, 0x0e],             asm: "mov    dx,0xea7" },
    TestCase { addr: 0x0008, dat: &[0xb4, 0x30],                   asm: "mov    ah,0x30" },
    TestCase { addr: 0x000a, dat: &[0xcd, 0x21],                   asm: "int    0x21" },
    TestCase { addr: 0x000c, dat: &[0x8b, 0x2e, 0x02, 0x00],       asm: "mov    bp,WORD PTR ds:0x2" },
    TestCase { addr: 0x0010, dat: &[0x8b, 0x1e, 0x2c, 0x00],       asm: "mov    bx,WORD PTR ds:0x2c" },
    TestCase { addr: 0x0016, dat: &[0xa3, 0x7d, 0x00],             asm: "mov    WORD PTR ds:0x7d,ax" },
    TestCase { addr: 0x0019, dat: &[0x8c, 0x06, 0x7b, 0x00],       asm: "mov    WORD PTR ds:0x7b,es" },
    TestCase { addr: 0x001d, dat: &[0x89, 0x1e, 0x77, 0x00],       asm: "mov    WORD PTR ds:0x77,bx" },
    TestCase { addr: 0x0021, dat: &[0x89, 0x2e, 0x91, 0x00],       asm: "mov    WORD PTR ds:0x91,bp" },
    TestCase { addr: 0x0025, dat: &[0xe8, 0x52, 0x01],             asm: "call   0x17a" },
    TestCase { addr: 0x0028, dat: &[0xc4, 0x3e, 0x75, 0x00],       asm: "les    di,DWORD PTR ds:0x75" },
    TestCase { addr: 0x002c, dat: &[0x8b, 0xc7],                   asm: "mov    ax,di" },
    TestCase { addr: 0x002e, dat: &[0x8b, 0xd8],                   asm: "mov    bx,ax" },
    TestCase { addr: 0x0030, dat: &[0xb9, 0xff, 0x7f],             asm: "mov    cx,0x7fff" },
    TestCase { addr: 0x0033, dat: &[0xfc],                         asm: "cld" },
    TestCase { addr: 0x0003, dat: &[0x2e, 0x89, 0x16, 0x60, 0x02], asm: "mov    WORD PTR cs:0x260,dx" },
    TestCase { addr: 0x0014, dat: &[0x8e, 0xda],                   asm: "mov    ds,dx" },
    TestCase { addr: 0x0036, dat: &[0xe3, 0x43],                   asm: "jcxz   0x7b" },
    TestCase { addr: 0x0038, dat: &[0x43],                         asm: "inc    bx" },
    TestCase { addr: 0x0039, dat: &[0x26, 0x38, 0x05],             asm: "cmp    BYTE PTR es:[di],al" },
    TestCase { addr: 0x003c, dat: &[0x75, 0xf6],                   asm: "jne    0x34" },
    TestCase { addr: 0x003e, dat: &[0x80, 0xcd, 0x80],             asm: "or     ch,0x80" },
    TestCase { addr: 0x0041, dat: &[0xf7, 0xd9],                   asm: "neg    cx" },
    TestCase { addr: 0x0043, dat: &[0x89, 0x0e, 0x75, 0x00],       asm: "mov    WORD PTR ds:0x75,cx" },
    TestCase { addr: 0x0047, dat: &[0xb9, 0x02, 0x00],             asm: "mov    cx,0x2" },
    TestCase { addr: 0x004a, dat: &[0xd3, 0xe3],                   asm: "shl    bx,cl" },
    TestCase { addr: 0x004c, dat: &[0x83, 0xc3, 0x10],             asm: "add    bx,0x10" },
    TestCase { addr: 0x004f, dat: &[0x83, 0xe3, 0xf0],             asm: "and    bx,0xfff0" },
    TestCase { addr: 0x0052, dat: &[0x89, 0x1e, 0x79, 0x00],       asm: "mov    WORD PTR ds:0x79,bx" },
    TestCase { addr: 0x0056, dat: &[0x8c, 0xd2],                   asm: "mov    dx,ss" },
    TestCase { addr: 0x0058, dat: &[0x2b, 0xea],                   asm: "sub    bp,dx" },
    TestCase { addr: 0x005a, dat: &[0xbf, 0xa7, 0x0e],             asm: "mov    di,0xea7" },
    TestCase { addr: 0x005d, dat: &[0x8e, 0xc7],                   asm: "mov    es,di" },
    TestCase { addr: 0x005f, dat: &[0x26, 0x8b, 0x3e, 0x2e, 0x44], asm: "mov    di,WORD PTR es:0x442e" },
    TestCase { addr: 0x0064, dat: &[0x81, 0xff, 0x00, 0x02],       asm: "cmp    di,0x200" },
    TestCase { addr: 0x0068, dat: &[0x73, 0x08],                   asm: "jae    0x72" },
    TestCase { addr: 0x006a, dat: &[0xbf, 0x00, 0x02],             asm: "mov    di,0x200" },
    TestCase { addr: 0x006d, dat: &[0x26, 0x89, 0x3e, 0x2e, 0x44], asm: "mov    WORD PTR es:0x442e,di" },
    TestCase { addr: 0x0072, dat: &[0xb1, 0x04],                   asm: "mov    cl,0x4" },
    TestCase { addr: 0x0074, dat: &[0xd3, 0xef],                   asm: "shr    di,cl" },
    TestCase { addr: 0x0076, dat: &[0x47],                         asm: "inc    di" },
    TestCase { addr: 0x0077, dat: &[0x3b, 0xef],                   asm: "cmp    bp,di" },
    TestCase { addr: 0x0079, dat: &[0x73, 0x03],                   asm: "jae    0x7e" },
    TestCase { addr: 0x007b, dat: &[0xe9, 0xcb, 0x01],             asm: "jmp    0x249" },
    TestCase { addr: 0x007e, dat: &[0x8b, 0xdf],                   asm: "mov    bx,di" },
    TestCase { addr: 0x0080, dat: &[0x03, 0xda],                   asm: "add    bx,dx" },
    TestCase { addr: 0x0082, dat: &[0x89, 0x1e, 0x89, 0x00],       asm: "mov    WORD PTR ds:0x89,bx" },
    TestCase { addr: 0x0086, dat: &[0x89, 0x1e, 0x8d, 0x00],       asm: "mov    WORD PTR ds:0x8d,bx" },
    TestCase { addr: 0x008a, dat: &[0xa1, 0x7b, 0x00],             asm: "mov    ax,WORD PTR ds:0x7b" },
    TestCase { addr: 0x008d, dat: &[0x2b, 0xd8],                   asm: "sub    bx,ax" },
    TestCase { addr: 0x008f, dat: &[0x8e, 0xc0],                   asm: "mov    es,ax" },
    TestCase { addr: 0x0091, dat: &[0xb4, 0x4a],                   asm: "mov    ah,0x4a" },
    TestCase { addr: 0x0093, dat: &[0x57],                         asm: "push   di" },
    TestCase { addr: 0x0094, dat: &[0xcd, 0x21],                   asm: "int    0x21" },
    TestCase { addr: 0x0096, dat: &[0x5f],                         asm: "pop    di" },
    TestCase { addr: 0x0097, dat: &[0xd3, 0xe7],                   asm: "shl    di,cl" },
    TestCase { addr: 0x0099, dat: &[0xfa],                         asm: "cli" },
    TestCase { addr: 0x009a, dat: &[0x8e, 0xd2],                   asm: "mov    ss,dx" },
    TestCase { addr: 0x009c, dat: &[0x8b, 0xe7],                   asm: "mov    sp,di" },
    TestCase { addr: 0x009e, dat: &[0xfb],                         asm: "sti" },
    TestCase { addr: 0x009f, dat: &[0xb8, 0xa7, 0x0e],             asm: "mov    ax,0xea7" },
    TestCase { addr: 0x00a2, dat: &[0x8e, 0xc0],                   asm: "mov    es,ax" },
    TestCase { addr: 0x00a4, dat: &[0x26, 0x89, 0x3e, 0x2e, 0x44], asm: "mov    WORD PTR es:0x442e,di" },
    TestCase { addr: 0x00a9, dat: &[0x33, 0xc0],                   asm: "xor    ax,ax" },
    TestCase { addr: 0x00ab, dat: &[0x2e, 0x8e, 0x06, 0x60, 0x02], asm: "mov    es,WORD PTR cs:0x260" },
    TestCase { addr: 0x00b0, dat: &[0xbf, 0x52, 0x45],             asm: "mov    di,0x4552" },
    TestCase { addr: 0x00b3, dat: &[0xb9, 0x04, 0xbd],             asm: "mov    cx,0xbd04" },
    TestCase { addr: 0x00b6, dat: &[0x2b, 0xcf],                   asm: "sub    cx,di" },
    TestCase { addr: 0x00b8, dat: &[0xfc],                         asm: "cld" },
    TestCase { addr: 0x00bb, dat: &[0x83, 0x3e, 0xa0, 0x43, 0x14], asm: "cmp    WORD PTR ds:0x43a0,0x14" },
    TestCase { addr: 0x00c0, dat: &[0x76, 0x47],                   asm: "jbe    0x109" },
    TestCase { addr: 0x00c2, dat: &[0x80, 0x3e, 0x7d, 0x00, 0x03], asm: "cmp    BYTE PTR ds:0x7d,0x3" },
    TestCase { addr: 0x00c7, dat: &[0x72, 0x40],                   asm: "jb     0x109" },
    TestCase { addr: 0x00c9, dat: &[0x77, 0x07],                   asm: "ja     0xd2" },
    TestCase { addr: 0x00cb, dat: &[0x80, 0x3e, 0x7e, 0x00, 0x1e], asm: "cmp    BYTE PTR ds:0x7e,0x1e" },
    TestCase { addr: 0x00d0, dat: &[0x72, 0x37],                   asm: "jb     0x109" },
    TestCase { addr: 0x00d2, dat: &[0xb8, 0x01, 0x58],             asm: "mov    ax,0x5801" },
    TestCase { addr: 0x00d5, dat: &[0xbb, 0x02, 0x00],             asm: "mov    bx,0x2" },
    TestCase { addr: 0x00d8, dat: &[0xcd, 0x21],                   asm: "int    0x21" },
    TestCase { addr: 0x00da, dat: &[0x72, 0x2a],                   asm: "jb     0x106" },
    TestCase { addr: 0x00dc, dat: &[0xb4, 0x67],                   asm: "mov    ah,0x67" },
    TestCase { addr: 0x00de, dat: &[0x8b, 0x1e, 0xa0, 0x43],       asm: "mov    bx,WORD PTR ds:0x43a0" },
    TestCase { addr: 0x00e2, dat: &[0xcd, 0x21],                   asm: "int    0x21" },
    TestCase { addr: 0x00e4, dat: &[0x72, 0x20],                   asm: "jb     0x106" },
    TestCase { addr: 0x00e6, dat: &[0xb4, 0x48],                   asm: "mov    ah,0x48" },
    TestCase { addr: 0x00e8, dat: &[0xbb, 0x01, 0x00],             asm: "mov    bx,0x1" },
    TestCase { addr: 0x00eb, dat: &[0xcd, 0x21],                   asm: "int    0x21" },
    TestCase { addr: 0x00ed, dat: &[0x72, 0x17],                   asm: "jb     0x106" },
    TestCase { addr: 0x00ef, dat: &[0x40],                         asm: "inc    ax" },
    TestCase { addr: 0x00f0, dat: &[0xa3, 0x91, 0x00],             asm: "mov    WORD PTR ds:0x91,ax" },
    TestCase { addr: 0x00f3, dat: &[0x48],                         asm: "dec    ax" },
    TestCase { addr: 0x00f4, dat: &[0x8e, 0xc0],                   asm: "mov    es,ax" },
    TestCase { addr: 0x00f6, dat: &[0xb4, 0x49],                   asm: "mov    ah,0x49" },
    TestCase { addr: 0x00f8, dat: &[0xcd, 0x21],                   asm: "int    0x21" },
    TestCase { addr: 0x00fa, dat: &[0x72, 0x0a],                   asm: "jb     0x106" },
    TestCase { addr: 0x00fc, dat: &[0xb8, 0x01, 0x58],             asm: "mov    ax,0x5801" },
    TestCase { addr: 0x00ff, dat: &[0xbb, 0x00, 0x00],             asm: "mov    bx,0x0" },
    TestCase { addr: 0x0102, dat: &[0xcd, 0x21],                   asm: "int    0x21" },
    TestCase { addr: 0x0104, dat: &[0x73, 0x03],                   asm: "jae    0x109" },
    TestCase { addr: 0x0106, dat: &[0xe9, 0x40, 0x01],             asm: "jmp    0x249" },
    TestCase { addr: 0x0109, dat: &[0xb4, 0x00],                   asm: "mov    ah,0x0" },
    TestCase { addr: 0x010b, dat: &[0xcd, 0x1a],                   asm: "int    0x1a" },
    TestCase { addr: 0x010d, dat: &[0x89, 0x16, 0x81, 0x00],       asm: "mov    WORD PTR ds:0x81,dx" },
    TestCase { addr: 0x0111, dat: &[0x89, 0x0e, 0x83, 0x00],       asm: "mov    WORD PTR ds:0x83,cx" },
    TestCase { addr: 0x0115, dat: &[0x0a, 0xc0],                   asm: "or     al,al" },
    TestCase { addr: 0x0117, dat: &[0x74, 0x0c],                   asm: "je     0x125" },
    TestCase { addr: 0x0119, dat: &[0xb8, 0x40, 0x00],             asm: "mov    ax,0x40" },
    TestCase { addr: 0x011c, dat: &[0x8e, 0xc0],                   asm: "mov    es,ax" },
    TestCase { addr: 0x011e, dat: &[0xbb, 0x70, 0x00],             asm: "mov    bx,0x70" },
    TestCase { addr: 0x0121, dat: &[0x26, 0xc6, 0x07, 0x01],       asm: "mov    BYTE PTR es:[bx],0x1" },
    TestCase { addr: 0x0125, dat: &[0x33, 0xed],                   asm: "xor    bp,bp" },
    TestCase { addr: 0x0127, dat: &[0x2e, 0x8e, 0x06, 0x60, 0x02], asm: "mov    es,WORD PTR cs:0x260" },
    TestCase { addr: 0x012c, dat: &[0xbe, 0x2e, 0x45],             asm: "mov    si,0x452e" },
    TestCase { addr: 0x012f, dat: &[0xbf, 0x4c, 0x45],             asm: "mov    di,0x454c" },
    TestCase { addr: 0x0132, dat: &[0xe8, 0xb5, 0x00],             asm: "call   0x1ea" },
    TestCase { addr: 0x0135, dat: &[0xff, 0x36, 0x73, 0x00],       asm: "push   WORD PTR ds:0x73" },
    TestCase { addr: 0x0139, dat: &[0xff, 0x36, 0x71, 0x00],       asm: "push   WORD PTR ds:0x71" },
    TestCase { addr: 0x013d, dat: &[0xff, 0x36, 0x6f, 0x00],       asm: "push   WORD PTR ds:0x6f" },
    TestCase { addr: 0x0141, dat: &[0xff, 0x36, 0x6d, 0x00],       asm: "push   WORD PTR ds:0x6d" },
    TestCase { addr: 0x0145, dat: &[0xff, 0x36, 0x6b, 0x00],       asm: "push   WORD PTR ds:0x6b" },
    TestCase { addr: 0x0149, dat: &[0x9a, 0x38, 0x0b, 0xe0, 0x02], asm: "callf  0x2e0:0xb38" },
    TestCase { addr: 0x014e, dat: &[0x50],                         asm: "push   ax" },
    TestCase { addr: 0x014f, dat: &[0x90],                         asm: "nop" },
    TestCase { addr: 0x0150, dat: &[0x0e],                         asm: "push   cs" },
    TestCase { addr: 0x0151, dat: &[0xe8, 0xca, 0x01],             asm: "call   0x31e" },
    TestCase { addr: 0x0154, dat: &[0x2e, 0x8e, 0x06, 0x60, 0x02], asm: "mov    es,WORD PTR cs:0x260" },
    TestCase { addr: 0x0159, dat: &[0x56],                         asm: "push   si" },
    TestCase { addr: 0x015a, dat: &[0x57],                         asm: "push   di" },
    TestCase { addr: 0x015b, dat: &[0xbe, 0x4c, 0x45],             asm: "mov    si,0x454c" },
    TestCase { addr: 0x015e, dat: &[0xbf, 0x52, 0x45],             asm: "mov    di,0x4552" },
    TestCase { addr: 0x0161, dat: &[0xe8, 0x86, 0x00],             asm: "call   0x1ea" },
    TestCase { addr: 0x0164, dat: &[0x5f],                         asm: "pop    di" },
    TestCase { addr: 0x0165, dat: &[0x5e],                         asm: "pop    si" },
    TestCase { addr: 0x0166, dat: &[0xcb],                         asm: "retf" },
    TestCase { addr: 0x0167, dat: &[0xcb],                         asm: "retf" },
    TestCase { addr: 0x0168, dat: &[0x8b, 0xec],                   asm: "mov    bp,sp" },
    TestCase { addr: 0x016a, dat: &[0xb4, 0x4c],                   asm: "mov    ah,0x4c" },
    TestCase { addr: 0x016c, dat: &[0x8a, 0x46, 0x04],             asm: "mov    al,BYTE PTR ss:[bp+0x4]" },
    TestCase { addr: 0x016f, dat: &[0xcd, 0x21],                   asm: "int    0x21" },
    TestCase { addr: 0x0171, dat: &[0xb9, 0x0e, 0x00],             asm: "mov    cx,0xe" },
    TestCase { addr: 0x0174, dat: &[0xba, 0x2f, 0x00],             asm: "mov    dx,0x2f" },
    TestCase { addr: 0x0177, dat: &[0xe9, 0xd5, 0x00],             asm: "jmp    0x24f" },
    TestCase { addr: 0x017a, dat: &[0x1e],                         asm: "push   ds" },
    TestCase { addr: 0x017b, dat: &[0xb8, 0x00, 0x35],             asm: "mov    ax,0x3500" },
    TestCase { addr: 0x017e, dat: &[0xcd, 0x21],                   asm: "int    0x21" },
    TestCase { addr: 0x0180, dat: &[0x89, 0x1e, 0x5b, 0x00],       asm: "mov    WORD PTR ds:0x5b,bx" },
    TestCase { addr: 0x0184, dat: &[0x8c, 0x06, 0x5d, 0x00],       asm: "mov    WORD PTR ds:0x5d,es" },
    TestCase { addr: 0x0188, dat: &[0xb8, 0x04, 0x35],             asm: "mov    ax,0x3504" },
    TestCase { addr: 0x018b, dat: &[0xcd, 0x21],                   asm: "int    0x21" },
    TestCase { addr: 0x018d, dat: &[0x89, 0x1e, 0x5f, 0x00],       asm: "mov    WORD PTR ds:0x5f,bx" },
    TestCase { addr: 0x0191, dat: &[0x8c, 0x06, 0x61, 0x00],       asm: "mov    WORD PTR ds:0x61,es" },
    TestCase { addr: 0x0195, dat: &[0xb8, 0x05, 0x35],             asm: "mov    ax,0x3505" },
    TestCase { addr: 0x0198, dat: &[0xcd, 0x21],                   asm: "int    0x21" },
    TestCase { addr: 0x019a, dat: &[0x89, 0x1e, 0x63, 0x00],       asm: "mov    WORD PTR ds:0x63,bx" },
    TestCase { addr: 0x019e, dat: &[0x8c, 0x06, 0x65, 0x00],       asm: "mov    WORD PTR ds:0x65,es" },
    TestCase { addr: 0x01a2, dat: &[0xb8, 0x06, 0x35],             asm: "mov    ax,0x3506" },
    TestCase { addr: 0x01a5, dat: &[0xcd, 0x21],                   asm: "int    0x21" },
    TestCase { addr: 0x01a7, dat: &[0x89, 0x1e, 0x67, 0x00],       asm: "mov    WORD PTR ds:0x67,bx" },
    TestCase { addr: 0x01ab, dat: &[0x8c, 0x06, 0x69, 0x00],       asm: "mov    WORD PTR ds:0x69,es" },
    TestCase { addr: 0x01af, dat: &[0xb8, 0x00, 0x25],             asm: "mov    ax,0x2500" },
    TestCase { addr: 0x01b2, dat: &[0x8c, 0xca],                   asm: "mov    dx,cs" },
    TestCase { addr: 0x01b4, dat: &[0x8e, 0xda],                   asm: "mov    ds,dx" },
    TestCase { addr: 0x01b6, dat: &[0xba, 0x71, 0x01],             asm: "mov    dx,0x171" },
    TestCase { addr: 0x01b9, dat: &[0xcd, 0x21],                   asm: "int    0x21" },
    TestCase { addr: 0x01bb, dat: &[0x1f],                         asm: "pop    ds" },
    TestCase { addr: 0x01bc, dat: &[0xc3],                         asm: "ret" },
    TestCase { addr: 0x01bd, dat: &[0x1e],                         asm: "push   ds" },
    TestCase { addr: 0x01be, dat: &[0xb8, 0x00, 0x25],             asm: "mov    ax,0x2500" },
    TestCase { addr: 0x01c1, dat: &[0xc5, 0x16, 0x5b, 0x00],       asm: "lds    dx,DWORD PTR ds:0x5b" },
    TestCase { addr: 0x01c5, dat: &[0xcd, 0x21],                   asm: "int    0x21" },
    TestCase { addr: 0x01c7, dat: &[0x1f],                         asm: "pop    ds" },
    TestCase { addr: 0x01c8, dat: &[0x1e],                         asm: "push   ds" },
    TestCase { addr: 0x01c9, dat: &[0xb8, 0x04, 0x25],             asm: "mov    ax,0x2504" },
    TestCase { addr: 0x01cc, dat: &[0xc5, 0x16, 0x5f, 0x00],       asm: "lds    dx,DWORD PTR ds:0x5f" },
    TestCase { addr: 0x01d0, dat: &[0xcd, 0x21],                   asm: "int    0x21" },
    TestCase { addr: 0x01d2, dat: &[0x1f],                         asm: "pop    ds" },
    TestCase { addr: 0x01d3, dat: &[0x1e],                         asm: "push   ds" },
    TestCase { addr: 0x01d4, dat: &[0xb8, 0x05, 0x25],             asm: "mov    ax,0x2505" },
    TestCase { addr: 0x01d7, dat: &[0xc5, 0x16, 0x63, 0x00],       asm: "lds    dx,DWORD PTR ds:0x63" },
    TestCase { addr: 0x01db, dat: &[0xcd, 0x21],                   asm: "int    0x21" },
    TestCase { addr: 0x01dd, dat: &[0x1f],                         asm: "pop    ds" },
    TestCase { addr: 0x01de, dat: &[0x1e],                         asm: "push   ds" },
    TestCase { addr: 0x01df, dat: &[0xb8, 0x06, 0x25],             asm: "mov    ax,0x2506" },
    TestCase { addr: 0x01e2, dat: &[0xc5, 0x16, 0x67, 0x00],       asm: "lds    dx,DWORD PTR ds:0x67" },
    TestCase { addr: 0x01e6, dat: &[0xcd, 0x21],                   asm: "int    0x21" },
    TestCase { addr: 0x01e8, dat: &[0x1f],                         asm: "pop    ds" },
    TestCase { addr: 0x01e9, dat: &[0xcb],                         asm: "retf" },
    TestCase { addr: 0x01ea, dat: &[0x81, 0xfe, 0x2e, 0x45],       asm: "cmp    si,0x452e" },
    TestCase { addr: 0x01ee, dat: &[0x74, 0x04],                   asm: "je     0x1f4" },
    TestCase { addr: 0x01f0, dat: &[0x32, 0xe4],                   asm: "xor    ah,ah" },
    TestCase { addr: 0x01f2, dat: &[0xeb, 0x02],                   asm: "jmp    0x1f6" },
    TestCase { addr: 0x01f4, dat: &[0xb4, 0xff],                   asm: "mov    ah,0xff" },
    TestCase { addr: 0x01f6, dat: &[0x8b, 0xd7],                   asm: "mov    dx,di" },
    TestCase { addr: 0x01f8, dat: &[0x8b, 0xde],                   asm: "mov    bx,si" },
    TestCase { addr: 0x01fa, dat: &[0x3b, 0xdf],                   asm: "cmp    bx,di" },
    TestCase { addr: 0x01fc, dat: &[0x74, 0x23],                   asm: "je     0x221" },
    TestCase { addr: 0x01fe, dat: &[0x26, 0x80, 0x3f, 0xff],       asm: "cmp    BYTE PTR es:[bx],0xff" },
    TestCase { addr: 0x0202, dat: &[0x74, 0x18],                   asm: "je     0x21c" },
    TestCase { addr: 0x0204, dat: &[0x81, 0xfe, 0x2e, 0x45],       asm: "cmp    si,0x452e" },
    TestCase { addr: 0x0208, dat: &[0x74, 0x06],                   asm: "je     0x210" },
    TestCase { addr: 0x020a, dat: &[0x26, 0x3a, 0x67, 0x01],       asm: "cmp    ah,BYTE PTR es:[bx+0x1]" },
    TestCase { addr: 0x020e, dat: &[0xeb, 0x04],                   asm: "jmp    0x214" },
    TestCase { addr: 0x0210, dat: &[0x26, 0x38, 0x67, 0x01],       asm: "cmp    BYTE PTR es:[bx+0x1],ah" },
    TestCase { addr: 0x0214, dat: &[0x77, 0x06],                   asm: "ja     0x21c" },
    TestCase { addr: 0x0216, dat: &[0x26, 0x8a, 0x67, 0x01],       asm: "mov    ah,BYTE PTR es:[bx+0x1]" },
    TestCase { addr: 0x021a, dat: &[0x8b, 0xd3],                   asm: "mov    dx,bx" },
    TestCase { addr: 0x021c, dat: &[0x83, 0xc3, 0x06],             asm: "add    bx,0x6" },
    TestCase { addr: 0x021f, dat: &[0xeb, 0xd9],                   asm: "jmp    0x1fa" },
    TestCase { addr: 0x0221, dat: &[0x3b, 0xd7],                   asm: "cmp    dx,di" },
    TestCase { addr: 0x0223, dat: &[0x74, 0x1b],                   asm: "je     0x240" },
    TestCase { addr: 0x0225, dat: &[0x8b, 0xda],                   asm: "mov    bx,dx" },
    TestCase { addr: 0x0227, dat: &[0x26, 0x80, 0x3f, 0x00],       asm: "cmp    BYTE PTR es:[bx],0x0" },
    TestCase { addr: 0x022b, dat: &[0x26, 0xc6, 0x07, 0xff],       asm: "mov    BYTE PTR es:[bx],0xff" },
    TestCase { addr: 0x022f, dat: &[0x06],                         asm: "push   es" },
    TestCase { addr: 0x0230, dat: &[0x74, 0x07],                   asm: "je     0x239" },
    TestCase { addr: 0x0232, dat: &[0x26, 0xff, 0x5f, 0x02],       asm: "callf  DWORD PTR es:[bx+0x2]" },
    TestCase { addr: 0x0236, dat: &[0x07],                         asm: "pop    es" },
    TestCase { addr: 0x0237, dat: &[0xeb, 0xb1],                   asm: "jmp    0x1ea" },
    TestCase { addr: 0x0239, dat: &[0x26, 0xff, 0x57, 0x02],       asm: "call   WORD PTR es:[bx+0x2]" },
    TestCase { addr: 0x023d, dat: &[0x07],                         asm: "pop    es" },
    TestCase { addr: 0x023e, dat: &[0xeb, 0xaa],                   asm: "jmp    0x1ea" },
    TestCase { addr: 0x0240, dat: &[0xc3],                         asm: "ret" },
    TestCase { addr: 0x0241, dat: &[0xb4, 0x40],                   asm: "mov    ah,0x40" },
    TestCase { addr: 0x0243, dat: &[0xbb, 0x02, 0x00],             asm: "mov    bx,0x2" },
    TestCase { addr: 0x0246, dat: &[0xcd, 0x21],                   asm: "int    0x21" },
    TestCase { addr: 0x0248, dat: &[0xc3],                         asm: "ret" },
    TestCase { addr: 0x0249, dat: &[0xb9, 0x1e, 0x00],             asm: "mov    cx,0x1e" },
    TestCase { addr: 0x024c, dat: &[0xba, 0x3d, 0x00],             asm: "mov    dx,0x3d" },
    TestCase { addr: 0x024f, dat: &[0x2e, 0x8e, 0x1e, 0x60, 0x02], asm: "mov    ds,WORD PTR cs:0x260" },
    TestCase { addr: 0x0254, dat: &[0xe8, 0xea, 0xff],             asm: "call   0x241" },
    TestCase { addr: 0x0257, dat: &[0xb8, 0x03, 0x00],             asm: "mov    ax,0x3" },
    TestCase { addr: 0x025a, dat: &[0x50],                         asm: "push   ax" },
    TestCase { addr: 0x025b, dat: &[0x90],                         asm: "nop" },
    TestCase { addr: 0x025c, dat: &[0x0e],                         asm: "push   cs" },
    TestCase { addr: 0x025d, dat: &[0xe8, 0xcd, 0x00],             asm: "call   0x32d" },
    TestCase { addr: 0x0034, dat: &[0xf2, 0xae],                   asm: "repne scas   al,BYTE PTR es:[di]" },
    TestCase { addr: 0x00b9, dat: &[0xf3, 0xaa],                   asm: "rep stos   BYTE PTR es:[di],al" },
    TestCase { addr: 0x0000, dat: &[0x6b, 0xc0, 0x06],             asm: "imul   ax,ax,0x6" },
    TestCase { addr: 0x0000, dat: &[0x6b, 0xff, 0x07],             asm: "imul   di,di,0x7" },
    TestCase { addr: 0x0000, dat: &[0x6b, 0x1e, 0x79, 0x1e, 0x6b], asm: "imul   bx,WORD PTR ds:0x1e79,0x6b" },
    TestCase { addr: 0x0000, dat: &[0x69, 0xf6, 0xa0, 0x00],       asm: "imul   si,si,0xa0" },
    TestCase { addr: 0x0000, dat: &[0x69, 0x01, 0x79, 0x01],       asm: "imul   ax,WORD PTR ds:[bx+di],0x179" },
  ];

  #[test]
  fn test() {
    use crate::segoff::*;
    for (n, test) in TESTS.iter().enumerate() {
      let addr = SegOff { seg: Seg::Normal(0), off: Off(test.addr.try_into().unwrap()) };
      let mut bin = RegionIter::new(test.dat, addr);
      let (ins, bytes) = decode_one(&mut bin).unwrap().unwrap();
      println!("{:?}", ins);
      let asm = crate::asm::intel_syntax::format(addr, Some(&ins), &bytes, false).unwrap();
      if asm != test.asm {
        panic!("Failed ({}/{}) | Expected: '{}' | Got: '{}'\n\nRAW:\n{:?}", n, TESTS.len(), test.asm, asm, ins);
      }
    }
  }
}
