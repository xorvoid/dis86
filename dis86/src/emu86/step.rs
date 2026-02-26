use super::machine::*;
use crate::asm::decode::Decoder;
use crate::asm::instr::{self, Instr, Opcode, Operand, OperandReg, OperandMem, OperandImm};
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
    self.reg_read(convert_reg(reg.0))
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

  pub fn operand_mem_addr(&self, mem: &OperandMem) -> SegOff {
    let seg = self.reg(convert_reg(mem.sreg));

    let mut offset: u16 = 0;
    if let Some(reg) = mem.reg1 {
      offset = offset.wrapping_add(self.reg(convert_reg(reg)));
    }
    if let Some(reg) = mem.reg2 {
      offset = offset.wrapping_add(self.reg(convert_reg(reg)));
    }
    if let Some(off) = mem.off {
      offset = offset.wrapping_add(off);
    }

    SegOff::new_normal(seg, offset)
  }

  pub fn operand_mem_read(&self, mem: &OperandMem) -> Value {
    let addr = self.operand_mem_addr(mem);
    match mem.sz {
      instr::Size::Size8  => Value::U8(self.mem.read_u8(addr)),
      instr::Size::Size16 => Value::U16(self.mem.read_u16(addr)),
      instr::Size::Size32 => Value::U32(self.mem.read_u32(addr)),
    }
  }

  pub fn operand_mem_write(&mut self, mem: &OperandMem, val: Value) {
    let addr = self.operand_mem_addr(mem);
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
      Operand::Far(far) => Value::Addr(SegOff::new(far.seg, far.off)),
    }
  }

  pub fn operand_read_u8(&self, instr: &Instr, oper: usize) -> u8 {
    self.operand_read(instr, oper).unwrap_u8()
  }

  pub fn operand_read_u16(&self, instr: &Instr, oper: usize) -> u16 {
    self.operand_read(instr, oper).unwrap_u16()
  }

  pub fn operand_read_addr(&self, instr: &Instr, oper: usize) -> SegOff {
    let value = self.operand_read(instr, oper);
    match value {
      Value::Addr(addr) => addr,
      Value::U32(val) => SegOff::from_u32(val),  // allow 32-bit loaded values to be used as addresses
      _ => panic!("invalid value: {:?}", value),
    }
  }

  pub fn operand_write(&mut self, instr: &Instr, oper: usize, val: Value) {
    let operand = &instr.operands[oper];
    match operand {
      Operand::Reg(reg) => self.operand_reg_write(reg, val),
      Operand::Mem(mem) => self.operand_mem_write(mem, val),
      _ => panic!("unsupported operand: {:?}", operand),
    }
  }

  pub fn op_unary(&mut self, instr: &Instr, op: alu::UnaryOp) {
    let val = self.operand_read(instr, 0);
    let (result, flags) = alu::unary(op, val, self.flag_read_all());
    self.flag_write_all(flags);
    self.operand_write(instr, 0, result);
  }

  pub fn op_binary(&mut self, instr: &Instr, op: alu::BinaryOp) {
    let lhs = self.operand_read(instr, 0);
    let rhs = self.operand_read(instr, 1);
    let (result, flags) = alu::binary(op, lhs, rhs, self.flag_read_all());
    self.flag_write_all(flags);
    self.operand_write(instr, 0, result);
  }

  pub fn op_shift(&mut self, instr: &Instr, op: alu::ShiftOp) {
    let lhs = self.operand_read(instr, 0);
    let rhs = self.operand_read(instr, 1);
    let count = rhs.unwrap_u8();
    let (result, flags) = alu::shift(op, lhs, count, self.flag_read_all());
    self.flag_write_all(flags);
    self.operand_write(instr, 0, result);
  }

  pub fn op_jump_cond(&mut self, instr: &Instr, cond: bool) {
    let tgt = self.operand_read_addr(&instr, 0);
    if cond { self.reg_write_addr(CS, IP, tgt); }
  }

  pub fn step(&mut self) -> Result<(), String> {
    // Get instr addr

    // Fetch and Decode
    let cs = self.reg_read_u16(CS);
    let ip = self.reg_read_u16(IP);
    let instr_addr = SegOff::new_normal(cs, ip);
    let instr = decode_instr(&self.mem, instr_addr)?;

    // Update IP
    self.reg_set(IP, instr.end_addr().off.0);

    // Report
    if DEBUG {
      let instr_addr_adj = SegOff::new(cs - (PSP_SEGMENT.unwrap_normal() + 0x10), ip);
      println!("{:6} | {}   {}", self.exec_count, instr_addr_adj, instr_str(&instr));
      //println!("{:?}", instr);
    }

    // Special Ops (rep aware)
    match instr.opcode {
      Opcode::OP_SCAS => return self.opcode_scas(&instr),
      Opcode::OP_STOS => return self.opcode_stos(&instr),
      _ => (),
    }

    if instr.rep.is_some() { panic!("REP prefix is not yet implemented"); }

    let f = self.flag_read_all();
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
      Opcode::OP_CALLF => {
        let tgt = self.operand_read_addr(&instr, 0);
        self.stack_push(self.reg_read(CS));
        self.stack_push(self.reg_read(IP));
        self.reg_write_addr(CS, IP, tgt);
      }

      Opcode::OP_RET => {
        let off = self.stack_pop();
        if instr.operands.len() == 1 {
          // handle stack args removal
          let adj = self.operand_read(&instr, 0).unwrap_u16();
          self.reg_write_u16(SP, self.reg_read_u16(SP) + adj);
        }
        self.reg_write(IP, off);
      }

      Opcode::OP_RETF => {
        let off = self.stack_pop();
        if instr.operands.len() == 1 {
          // handle stack args removal
          let adj = self.operand_read(&instr, 0).unwrap_u16();
          self.reg_write_u16(SP, self.reg_read_u16(SP) + adj);
        }
        let seg = self.stack_pop();
        self.reg_write(CS, seg);
        self.reg_write(IP, off);
      }

      Opcode::OP_LDS |
      Opcode::OP_LES => {
        let val = self.operand_read(&instr, 2);
        let addr = SegOff::from_u32(val.unwrap_u32());
        self.operand_write(&instr, 0, Value::U16(addr.seg.unwrap_normal()));
        self.operand_write(&instr, 1, Value::U16(addr.off.0));
      }
      Opcode::OP_LEA => {
        let off = match &instr.operands[1] {
          Operand::Mem(mem) => self.operand_mem_addr(mem).off.0,
          _ => panic!("expected memory operand"),
        };
        self.operand_write(&instr, 0, Value::U16(off));
      }

      Opcode::OP_CLD => self.flag_write(FLAG_DF, false),
      Opcode::OP_STD => self.flag_write(FLAG_DF, true),
      Opcode::OP_CLI => self.flag_write(FLAG_IF, false),
      Opcode::OP_STI => self.flag_write(FLAG_IF, true),

      ////////////////////////////////////////////////////////////////////////////////
      // Jumps

      Opcode::OP_JMP |
      Opcode::OP_JMPF => {
        let tgt = self.operand_read_addr(&instr, 0);
        self.reg_write_addr(CS, IP, tgt);
      }

      Opcode::OP_JCXZ => {
        let tgt = self.operand_read_addr(&instr, 1);
        if self.reg_read_u16(CX) == 0 { self.reg_write_addr(CS, IP, tgt); }
      }

      Opcode::OP_JE   => self.op_jump_cond(&instr, f.get(FLAG_ZF)),
      Opcode::OP_JNE  => self.op_jump_cond(&instr, !f.get(FLAG_ZF)),
      Opcode::OP_JB   => self.op_jump_cond(&instr, f.get(FLAG_CF)),
      Opcode::OP_JAE  => self.op_jump_cond(&instr, !f.get(FLAG_CF)),
      Opcode::OP_JA   => self.op_jump_cond(&instr, !f.get(FLAG_CF) && !f.get(FLAG_ZF)),
      Opcode::OP_JBE  => self.op_jump_cond(&instr, f.get(FLAG_CF) || f.get(FLAG_ZF)),
      Opcode::OP_JL   => self.op_jump_cond(&instr, f.get(FLAG_SF) != f.get(FLAG_OF)),
      Opcode::OP_JGE  => self.op_jump_cond(&instr, f.get(FLAG_SF) == f.get(FLAG_OF)),
      Opcode::OP_JG   => self.op_jump_cond(&instr, !f.get(FLAG_ZF) && f.get(FLAG_SF) == f.get(FLAG_OF)),
      Opcode::OP_JLE  => self.op_jump_cond(&instr, f.get(FLAG_SF) || f.get(FLAG_SF) != f.get(FLAG_OF)),
      Opcode::OP_JS   => self.op_jump_cond(&instr, f.get(FLAG_SF)),
      Opcode::OP_JNS  => self.op_jump_cond(&instr, !f.get(FLAG_SF)),
      Opcode::OP_JO   => self.op_jump_cond(&instr, f.get(FLAG_OF)),
      Opcode::OP_JNO  => self.op_jump_cond(&instr, !f.get(FLAG_OF)),
      Opcode::OP_JP   => self.op_jump_cond(&instr, f.get(FLAG_PF)),
      Opcode::OP_JNP  => self.op_jump_cond(&instr, !f.get(FLAG_PF)),

      Opcode::OP_CMP => {
        let lhs = self.operand_read(&instr, 0);
        let rhs = self.operand_read(&instr, 1);
        let (_result, flags) = alu::binary(alu::BinaryOp::Sub, lhs, rhs, self.flag_read_all());
        self.flag_write_all(flags);
      }

      Opcode::OP_TEST => {
        let lhs = self.operand_read(&instr, 0);
        let rhs = self.operand_read(&instr, 1);
        let (_result, flags) = alu::binary(alu::BinaryOp::And, lhs, rhs, self.flag_read_all());
        self.flag_write_all(flags);
      }

      Opcode::OP_INC => self.op_unary(&instr, alu::UnaryOp::Inc),
      Opcode::OP_NEG => self.op_unary(&instr, alu::UnaryOp::Neg),
      Opcode::OP_AND => self.op_binary(&instr, alu::BinaryOp::And),
      Opcode::OP_OR  => self.op_binary(&instr, alu::BinaryOp::Or),
      Opcode::OP_XOR => self.op_binary(&instr, alu::BinaryOp::Xor),
      Opcode::OP_ADD => self.op_binary(&instr, alu::BinaryOp::Add),
      Opcode::OP_ADC => self.op_binary(&instr, alu::BinaryOp::Adc),
      Opcode::OP_SUB => self.op_binary(&instr, alu::BinaryOp::Sub),
      Opcode::OP_SHL => self.op_shift(&instr, alu::ShiftOp::Shl),
      Opcode::OP_SHR => self.op_shift(&instr, alu::ShiftOp::Shr),

      Opcode::OP_MUL => {
        let lhs = self.operand_read(&instr, 1);
        let rhs = self.operand_read(&instr, 2);

        // Special-cased 16-bit
        // FIXME: Add 8-bit
        assert!(lhs.is_u16());
        assert!(rhs.is_u16());

        let (result, flags) = alu::binary(alu::BinaryOp::Mul, lhs, rhs, self.flag_read_all());
        self.flag_write_all(flags);

        let result = result.unwrap_u32();
        self.operand_write(&instr, 0, Value::U16((result>>16) as u16));
        self.operand_write(&instr, 1, Value::U16(result as u16));
      }

      Opcode::OP_XCHG => {
        let lhs = self.operand_read(&instr, 0);
        let rhs = self.operand_read(&instr, 1);
        self.operand_write(&instr, 0, rhs);
        self.operand_write(&instr, 1, lhs);
      }

      Opcode::OP_CBW => {
        let lower = self.operand_read(&instr, 1).unwrap_u8();
        let upper = ((lower as i8) >> 7) as u8; // fill the word with the sign-bit
        let value = (upper as u16) << 8 | (lower as u16);
        self.operand_write(&instr, 0, Value::U16(value));
      }

      Opcode::OP_CWD => {
        let lower = self.operand_read(&instr, 1).unwrap_u16();
        let upper = ((lower as i16) >> 15) as u16; // fill the word with the sign-bit
        self.operand_write(&instr, 0, Value::U16(upper));
      }

      Opcode::OP_NOP => {}

      _ => {
        panic!("Unimplemnted opcode: {}", instr.opcode.name());
      }
    }

    //println!("Halting!");
    //self.halted = true;

    self.exec_count += 1;
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
