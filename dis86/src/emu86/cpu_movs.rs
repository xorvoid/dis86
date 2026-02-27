use super::machine::*;
use crate::asm::instr::{self, Instr, Opcode, Operand};

impl Machine {
  pub fn opcode_movs(&mut self, instr: &Instr) -> Result<(), String> {
    assert_eq!(instr.opcode, Opcode::OP_MOVS);

    let Operand::Mem(mem) = instr.operands[1] else { panic!("Expected memory operand") };
    let size = match mem.sz {
      instr::Size::Size8  => 1,
      instr::Size::Size16 => 2,
      _ => panic!("unsupported size"),
    };

    let dir = self.flag_read(FLAG_DF);
    let inc = if !dir { size } else { (-(size as i16)) as u16 };
    let rep = instr.rep;

    let mut count = if rep.is_some() {
      self.reg_read_u16(CX)
    } else {
      1
    };

    while count != 0 {
      let rhs = self.operand_read(&instr, 1);
      self.operand_write(&instr, 0, rhs);

      let si = self.reg_read_u16(SI);
      self.reg_write_u16(SI, si.wrapping_add(inc));

      let di = self.reg_read_u16(DI);
      self.reg_write_u16(DI, di.wrapping_add(inc));

      count -= 1;
    }

    if rep.is_some() {
      self.reg_write_u16(CX, count);
    }

    Ok(())
  }
}
