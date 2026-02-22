use super::machine::*;
use crate::asm::instr::{self, Instr, Opcode, Operand};

impl Machine {
  pub fn opcode_stos(&mut self, instr: &Instr) -> Result<(), String> {
    assert_eq!(instr.opcode, Opcode::OP_STOS);

    let Operand::Reg(reg) = instr.operands[1] else { panic!("Expected register operand") };
    let size = match reg.0.info().sz {
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

    let rhs = self.operand_read(&instr, 1);
    while count != 0 {
      self.operand_write(&instr, 0, rhs);

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

#[cfg(test)]
mod test {
  use super::*;

  struct Test {
    data_range: (u16, u16),
    df:    u8,
    di:    u16,
    ax:    u16,
    cx:    u16,
  }

  struct Result {
    data: Vec<u8>,
    di:   u16,
    cx:   u16,
  }

  fn mem_write_data(m: &mut Machine, addr: SegOff, data: &[u8]) {
    for i in 0..data.len() {
      m.mem.write_u8(addr.add_offset(i as u16), data[i]);
    }
  }

  fn run_impl(test: Test, code: &[u8]) -> Result {
    let mut m = Machine::default();

    let code_addr = SegOff::new(0x0000, 0x0000);
    mem_write_data(&mut m, code_addr, code);

    let data_addr = SegOff::new(0x1000, 0x0000);
    m.reg_write_addr(ES, DI, data_addr);

    m.flag_write(FLAG_DF, test.df != 0);
    m.reg_write_u16(DI, test.di);
    m.reg_write_u16(CX, test.cx);
    m.reg_write_u16(AX, test.ax);

    m.step().unwrap();

    let mut data = vec![];
    let mut off = test.data_range.0;
    let off_end = test.data_range.1;
    while off < off_end {
      data.push(m.mem.read_u8(data_addr.add_offset(off)));
      off += 1;
    }

    Result {
      data,
      di: m.reg_read_u16(DI),
      cx: m.reg_read_u16(CX),
    }
  }

  fn run_stosb(test: Test) -> Result {
    run_impl(test, &[0xaa]) // stosb BYTE PTR es:[di], al
  }

  fn run_rep_stosb(test: Test) -> Result {
    run_impl(test, &[0xf3, 0xaa]) // rep stosb BYTE PTR es:[di], al
  }

  fn run_stosw(test: Test) -> Result {
    run_impl(test, &[0xab]) // stosw WORD PTR es:[di], ax
  }

  fn run_rep_stosw(test: Test) -> Result {
    run_impl(test, &[0xf3, 0xab]) // rep stosw WORD PTR es:[di], ax
  }

  //////////////////////////////////////////////////////////////////////
  // STOSB
  //////////////////////////////////////////////////////////////////////

  #[test]
  fn test_stosb() {
    let result = run_stosb(Test {
      data_range: (0, 5),
      df: 0, di: 1, cx: 1, ax: 0xabcd,
    });
    assert_eq!(&result.data, &[0x00, 0xcd, 0x00, 0x00, 0x00]);
    assert_eq!(result.di, 2);
    assert_eq!(result.cx, 1);
  }

  #[test]
  fn test_stosb_backwards() {
    let result = run_stosb(Test {
      data_range: (0, 5),
      df: 1, di: 1, cx: 1, ax: 0xabcd,
    });
    assert_eq!(&result.data, &[0x00, 0xcd, 0x00, 0x00, 0x00]);
    assert_eq!(result.di, 0);
    assert_eq!(result.cx, 1);
  }


  #[test]
  fn test_rep_stosb() {
    let result = run_rep_stosb(Test {
      data_range: (0, 5),
      df: 0, di: 1, cx: 3, ax: 0xabcd,
    });
    assert_eq!(&result.data, &[0x00, 0xcd, 0xcd, 0xcd, 0x00]);
    assert_eq!(result.di, 4);
    assert_eq!(result.cx, 0);
  }

  #[test]
  fn test_rep_stosb_backwards() {
    let result = run_rep_stosb(Test {
      data_range: (0, 5),
      df: 1, di: 2, cx: 3, ax: 0xabcd,
    });
    assert_eq!(&result.data, &[0xcd, 0xcd, 0xcd, 0x00, 0x00]);
    assert_eq!(result.di, 0xffff);
    assert_eq!(result.cx, 0);
  }

  //////////////////////////////////////////////////////////////////////
  // STOSW
  //////////////////////////////////////////////////////////////////////

  #[test]
  fn test_stosw() {
    let result = run_stosw(Test {
      data_range: (0, 10),
      df: 0, di: 2, cx: 1, ax: 0xabcd,
    });
    assert_eq!(&result.data, &[0x00,0x00,  0xcd,0xab,  0x00,0x00,  0x00,0x00,  0x00,0x00]);
    assert_eq!(result.di, 4);
    assert_eq!(result.cx, 1);
  }

  #[test]
  fn test_stosw_backwards() {
    let result = run_stosw(Test {
      data_range: (0, 10),
      df: 1, di: 2, cx: 1, ax: 0xabcd,
    });
    assert_eq!(&result.data, &[0x00,0x00,  0xcd,0xab,  0x00,0x00,  0x00,0x00,  0x00,0x00]);
    assert_eq!(result.di, 0);
    assert_eq!(result.cx, 1);
  }

  #[test]
  fn test_rep_stosw() {
    let result = run_rep_stosw(Test {
      data_range: (0, 10),
      df: 0, di: 2, cx: 3, ax: 0xabcd,
    });
    assert_eq!(&result.data, &[0x00,0x00,  0xcd,0xab,  0xcd,0xab,  0xcd,0xab,  0x00,0x00]);
    assert_eq!(result.di, 8);
    assert_eq!(result.cx, 0);
  }

  #[test]
  fn test_rep_stosw_backwards() {
    let result = run_rep_stosw(Test {
      data_range: (0, 10),
      df: 1, di: 4, cx: 3, ax: 0xabcd,
    });
    assert_eq!(&result.data, &[0xcd,0xab,  0xcd,0xab,  0xcd,0xab,  0x00,0x00,  0x00,0x00]);
    assert_eq!(result.di, 0xfffe);
    assert_eq!(result.cx, 0);
  }
}
