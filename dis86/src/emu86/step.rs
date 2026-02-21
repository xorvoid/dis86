use super::machine::*;
use crate::asm::decode::Decoder;
use crate::asm::instr::{self, Instr, Opcode, Operand, OperandReg, OperandMem, OperandImm, OperandRel, OperandFar};
use crate::asm::intel_syntax::instr_str;

const DEBUG: bool = true;

impl Machine {
  pub fn operand_imm_read(&self, imm: &OperandImm) -> Value {
    match imm.sz {
      instr::Size::Size8  => Value::U8(imm.val as u8),
      instr::Size::Size16 => Value::U16(imm.val),
      _ => panic!("unsupported size"),
    }
  }

  pub fn operand_reg_read(&self, reg: &OperandReg) -> Value {
    let reg = convert_reg(reg.0);
    let v = self.reg(reg);
    assert!(reg.size == 2);
    Value::U16(v)
  }

  pub fn operand_reg_write(&mut self, reg: &OperandReg, val: Value) {
    let reg = convert_reg(reg.0);
    match val {
      Value::U8(val) => {
        assert_eq!(reg.size, 1);
        self.reg_set(reg, val as u16);
      }
      Value::U16(val) => {
        assert_eq!(reg.size, 2);
        self.reg_set(reg, val);
      }
      _ => panic!("unsupported size"),
    }
  }

  pub fn operand_mem_read(&self, mem: &OperandMem) -> Value {
    let seg = self.reg(convert_reg(mem.sreg));

    let mut offset = 0;
    if let Some(reg) = mem.reg1 {
      offset += self.reg(convert_reg(reg));
    }
    if let Some(reg) = mem.reg2 {
      offset += self.reg(convert_reg(reg));
    }
    if let Some(off) = mem.off {
      offset += off;
    }

    let addr = SegOff::new_normal(seg, offset);
    match mem.sz {
      instr::Size::Size8  => Value::U8(self.mem.read_u8(addr)),
      instr::Size::Size16 => Value::U16(self.mem.read_u16(addr)),
      instr::Size::Size32 => Value::U32(self.mem.read_u32(addr)),
    }
  }

  pub fn operand_mem_write(&mut self, mem: &OperandMem, val: Value) {
    let seg = self.reg(convert_reg(mem.sreg));

    let mut offset = 0;
    if let Some(reg) = mem.reg1 {
      offset += self.reg(convert_reg(reg));
    }
    if let Some(reg) = mem.reg2 {
      offset += self.reg(convert_reg(reg));
    }
    if let Some(off) = mem.off {
      offset += off;
    }

    let addr = SegOff::new_normal(seg, offset);
    match val {
      Value::U8(val)  => self.mem.write_u8(addr, val),
      Value::U16(val) => self.mem.write_u16(addr, val),
      Value::U32(val) => self.mem.write_u32(addr, val),
      Value::Addr(_) => panic!("Inavlid value type: {:?}", val)
    }
  }

  pub fn operand_read(&self, instr: &Instr, oper: usize) -> Value {
    let operand = &instr.operands[oper];
    match operand {
      Operand::Imm(imm) => self.operand_imm_read(imm),
      Operand::Reg(reg) => self.operand_reg_read(reg),
      Operand::Mem(mem) => self.operand_mem_read(mem),
      Operand::Rel(rel) => Value::Addr(instr.rel_addr(rel)),
      _ => panic!("unsupported operand: {:?}", operand),
    }
  }

  pub fn operand_read_u8(&self, instr: &Instr, oper: usize) -> u8 {
    self.operand_read(instr, oper).unwrap_u8()
  }

  pub fn operand_read_u16(&self, instr: &Instr, oper: usize) -> u16 {
    self.operand_read(instr, oper).unwrap_u16()
  }

  pub fn operand_read_addr(&self, instr: &Instr, oper: usize) -> SegOff {
    self.operand_read(instr, oper).unwrap_addr()
  }

  pub fn operand_write(&mut self, instr: &Instr, oper: usize, val: Value) {
    let operand = &instr.operands[oper];
    match operand {
      Operand::Reg(reg) => self.operand_reg_write(reg, val),
      Operand::Mem(mem) => self.operand_mem_write(mem, val),
      _ => panic!("unsupported operand: {:?}", operand),
    }
  }

  pub fn step(&mut self) -> Result<(), String> {
    // Get instr addr

    // Fetch and Decode
    let instr_addr = SegOff::new_normal(self.reg(CS), self.reg(IP));
    let instr = decode_instr(&self.mem, instr_addr)?;

    // Update IP
    self.reg_set(IP, instr.end_addr().off.0);

    // Report
    if DEBUG {
      println!("{}   {}", instr_addr, instr_str(&instr));
      //println!("{:?}", instr);
    }

    if instr.rep.is_some() { panic!("REP prefix is not yet implemented"); }

    match instr.opcode {
      Opcode::OP_MOV  => self.operand_write(&instr, 0, self.operand_read(&instr, 1)),
      Opcode::OP_PUSH => self.stack_push(self.operand_read(&instr, 0)),
      Opcode::OP_POP  => { let val = self.stack_pop(); self.operand_write(&instr, 0, val) }
      Opcode::OP_INT  => self.interrupt(self.operand_read_u8(&instr, 0)),
      Opcode::OP_CALL => {
        let tgt = self.operand_read_addr(&instr, 0);
        self.stack_push(self.reg_read(IP));
        self.reg_set(IP, tgt.off.0);
      }
      Opcode::OP_RET => { let off = self.stack_pop(); self.reg_write(IP, off); }
      _ => {
        panic!("Unimplmented opcode: {}", instr.opcode.name());
      }
    }

    //println!("Halting!");
    //self.halted = true;

    Ok(())
  }
}

// FIXME: THIS IS KLUDGY AS HELL... THE INSTR DECODE API IS BAD AND CAUSES ISSUES EVERYWHERE
fn decode_instr(mem: &Memory, addr: SegOff) -> Result<Instr, String> {
  let slice = &mem.slice_starting_at(addr)[..16]; // HAX: 16 bytes is arbitrary
  let region = crate::region::RegionIter::new(slice, addr);
  let mut decoder = Decoder::new(region);
  let (instr, _raw) = decoder.try_next()?.unwrap();
  Ok(instr)
}

// FIXME: Shouldn't have to remap this at all.. would love to use it directly or with a trivial offsetting
fn convert_reg(r: instr::Reg) -> Register {
  match r {
    instr::Reg::AX    => AX,
    instr::Reg::BX    => BX,
    instr::Reg::CX    => CX,
    instr::Reg::DX    => DX,
    instr::Reg::SI    => SI,
    instr::Reg::DI    => DI,
    instr::Reg::BP    => BP,
    instr::Reg::SP    => SP,
    instr::Reg::IP    => IP,
    instr::Reg::CS    => CS,
    instr::Reg::DS    => DS,
    instr::Reg::ES    => ES,
    instr::Reg::SS    => SS,
    instr::Reg::FLAGS => FLAGS,
    instr::Reg::AH    => AH,
    instr::Reg::AL    => AL,
    instr::Reg::BH    => BH,
    instr::Reg::BL    => BL,
    instr::Reg::CH    => CH,
    instr::Reg::CL    => CL,
    instr::Reg::DH    => DH,
    instr::Reg::DL    => DL,
  }
}
