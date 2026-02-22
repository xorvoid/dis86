use super::machine::*;
use crate::asm::instr::{self, Instr, Opcode, Operand};

impl Machine {
  pub fn opcode_scas(&mut self, instr: &Instr) -> Result<(), String> {
    assert_eq!(instr.opcode, Opcode::OP_SCAS);

    let Operand::Reg(reg) = instr.operands[0] else { panic!("Expected register operand") };
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

    while count != 0 {
      let lhs = self.operand_read(&instr, 0);
      let rhs = self.operand_read(&instr, 1);
      self.flag_update_cmp(lhs, rhs);

      let di = self.reg_read_u16(DI);
      self.reg_write_u16(DI, di.wrapping_add(inc));

      count -= 1;

      if let Some(rep) = rep {
        let zf = self.flag_read(FLAG_ZF);
        match rep {
          instr::Rep::NE => if zf  { break },
          instr::Rep::EQ => if !zf { break },
        }
      }
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
    data:  Vec<u8>,
    df:    u8,
    di:    u16,
    ax:    u16,
    cx:    u16,
  }

  struct Result {
    sf: u8,
    zf: u8,
    cf: u8,
    of: u8,
    di: u16,
    cx: u16,
  }

  fn mem_write_slice(m: &mut Machine, addr: SegOff, data: &[u8]) {
    for i in 0..data.len() {
      m.mem.write_u8(addr.add_offset(i as u16), data[i]);
    }
  }

  fn run_impl(test: Test, code: &[u8]) -> Result {
    let mut m = Machine::default();

    let code_addr = SegOff::new(0x0000, 0x0000);
    mem_write_slice(&mut m, code_addr, code);

    let data_addr = SegOff::new(0x1000, 0x0000);
    mem_write_slice(&mut m, data_addr, &test.data);
    m.reg_write_addr(ES, DI, data_addr);

    m.flag_write(FLAG_DF, test.df != 0);
    m.reg_write_u16(DI, test.di);
    m.reg_write_u16(CX, test.cx);
    m.reg_write_u16(AX, test.ax);

    m.step().unwrap();

    Result {
      sf: m.flag_read(FLAG_SF) as u8,
      zf: m.flag_read(FLAG_ZF) as u8,
      cf: m.flag_read(FLAG_CF) as u8,
      of: m.flag_read(FLAG_OF) as u8,
      di: m.reg_read_u16(DI),
      cx: m.reg_read_u16(CX),
    }
  }

  fn run_scasb(test: Test) -> Result {
    run_impl(test, &[0xae]) // scasb al, BYTE PTR es:[di]
  }

  fn run_repe_scasb(test: Test) -> Result {
    run_impl(test, &[0xf3, 0xae]) // repe scasb al, BYTE PTR es:[di]
  }

  fn run_repne_scasb(test: Test) -> Result {
    run_impl(test, &[0xf2, 0xae]) // repne scasb al, BYTE PTR es:[di]
  }

  fn run_scasw(test: Test) -> Result {
    run_impl(test, &[0xaf]) // scasw ax, WORD PTR es:[di]
  }

  fn run_repe_scasw(test: Test) -> Result {
    run_impl(test, &[0xf3, 0xaf]) // repe scasw ax, WORD PTR es:[di]
  }

  fn run_repne_scasw(test: Test) -> Result {
    run_impl(test, &[0xf2, 0xaf]) // repne scasw ax, WORD PTR es:[di]
  }

  //////////////////////////////////////////////////////////////////////
  // SCASB
  //////////////////////////////////////////////////////////////////////

  #[test]
  fn test_scasb_found() {
    let result = run_scasb(Test {
      data: vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
      df: 0, di: 0, cx: 0, ax: 1,
    });
    assert_eq!(result.zf, 1);
    assert_eq!(result.di, 1);
  }

  #[test]
  fn test_scasb_not_found() {
    let result = run_scasb(Test {
      data: vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
      df: 0, di: 0, cx: 0, ax: 2,
    });
    assert_eq!(result.zf, 0);
    assert_eq!(result.di, 1);
  }

  #[test]
  fn test_scasb_forward_advances_di() {
    // DF=0: DI goes 3 -> 4
    let result = run_scasb(Test {
      data: vec![0, 0, 0, 0, 0],
      df: 0, di: 3, cx: 0, ax: 0,
    });
    assert_eq!(result.di, 4);
    assert_eq!(result.zf, 1);
  }

  #[test]
  fn test_scasb_backward_decrements_di() {
    // DF=1: DI goes 3 -> 2
    let result = run_scasb(Test {
      data: vec![0, 0, 0, 0, 0],
      df: 1, di: 3, cx: 0, ax: 0,
    });
    assert_eq!(result.di, 2);
    assert_eq!(result.zf, 1);
  }

  #[test]
  fn test_scasb_cf_set_when_mem_greater() {
    // AL=0x03, ES:[di]=0x05: 0x03 - 0x05 borrows, CF=1
    let result = run_scasb(Test {
      data: vec![0x05],
      df: 0, di: 0, cx: 0, ax: 0x03,
    });
    assert_eq!(result.cf, 1);
    assert_eq!(result.zf, 0);
  }

  #[test]
  fn test_scasb_cf_clear_when_al_greater() {
    // AL=0x05, ES:[di]=0x03: no borrow
    let result = run_scasb(Test {
      data: vec![0x03],
      df: 0, di: 0, cx: 0, ax: 0x05,
    });
    assert_eq!(result.cf, 0);
    assert_eq!(result.zf, 0);
  }

  #[test]
  fn test_scasb_sf_set_when_result_negative() {
    // AL=0x01, ES:[di]=0x02: result=0xFF, SF=1
    let result = run_scasb(Test {
      data: vec![0x02],
      df: 0, di: 0, cx: 0, ax: 0x01,
    });
    assert_eq!(result.sf, 1);
  }

  #[test]
  fn test_scasb_signed_overflow() {
    // AL=0x7F, ES:[di]=0xFF: 127 - (-1) = 128, overflows i8
    let result = run_scasb(Test {
      data: vec![0xff],
      df: 0, di: 0, cx: 0, ax: 0x7f,
    });
    assert_eq!(result.of, 1);
    assert_eq!(result.sf, 1);
    assert_eq!(result.cf, 1);
  }

  #[test]
  fn test_scasb_mid_buffer() {
    // Start at di=2, ES:[2]=0xab: match
    let result = run_scasb(Test {
      data: vec![0x00, 0x00, 0xab, 0x00],
      df: 0, di: 2, cx: 0, ax: 0xab,
    });
    assert_eq!(result.zf, 1);
    assert_eq!(result.di, 3);
  }

  //////////////////////////////////////////////////////////////////////
  // SCASW
  //////////////////////////////////////////////////////////////////////

  #[test]
  fn test_scasw_found() {
    let result = run_scasw(Test {
      data: vec![6,0, 7,0, 5,0, 1,0, 9,0],
      df: 0, di: 0, cx: 0, ax: 6,
    });
    assert_eq!(result.zf, 1);
    assert_eq!(result.di, 2);
  }

  #[test]
  fn test_scasw_not_found() {
    let result = run_scasw(Test {
      data: vec![6,0, 7,0, 5,0, 1,0, 9,0],
      df: 0, di: 0, cx: 0, ax: 2,
    });
    assert_eq!(result.zf, 0);
    assert_eq!(result.di, 2);
  }

  #[test]
  fn test_scasw_backward_decrements_di() {
    // DF=1: DI goes 2 -> 1
    let result = run_scasw(Test {
      data: vec![0,0, 0,0, 0,0],
      df: 1, di: 2, cx: 0, ax: 0,
    });
    assert_eq!(result.di, 0);
  }

  //////////////////////////////////////////////////////////////////////
  // REPE SCASB
  //////////////////////////////////////////////////////////////////////

  #[test]
  fn test_repe_scasb_found() {
    let result = run_repe_scasb(Test {
      data: vec![1, 1, 2, 3, 4, 5, 6, 7, 8, 9],
      df: 0, di: 0, cx: 10, ax: 1,
    });
    assert_eq!(result.zf, 0);
    assert_eq!(result.di, 3);
    assert_eq!(result.cx, 7);
  }

  #[test]
  fn test_repe_scasb_all_match() {
    // All bytes equal AL: CX exhausted, ZF=1
    let result = run_repe_scasb(Test {
      data: vec![0, 0, 0, 0],
      df: 0, di: 0, cx: 4, ax: 0,
    });
    assert_eq!(result.zf, 1);
    assert_eq!(result.cx, 0);
    assert_eq!(result.di, 4);
  }

  #[test]
  fn test_repe_scasb_mismatch_at_start() {
    // First byte doesn't match: stops immediately
    let result = run_repe_scasb(Test {
      data: vec![0xff, 0, 0, 0],
      df: 0, di: 0, cx: 4, ax: 0,
    });
    assert_eq!(result.zf, 0);
    assert_eq!(result.di, 1);
    assert_eq!(result.cx, 3);
  }

  #[test]
  fn test_repe_scasb_cx_zero_no_iteration() {
    let result = run_repe_scasb(Test {
      data: vec![0],
      df: 0, di: 0, cx: 0, ax: 0,
    });
    assert_eq!(result.di, 0);
    assert_eq!(result.cx, 0);
  }

  //////////////////////////////////////////////////////////////////////
  // REPNE SCASB
  //////////////////////////////////////////////////////////////////////

  #[test]
  fn test_repne_scasb_found() {
    let result = run_repne_scasb(Test {
      data: vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
      df: 0, di: 0, cx: 9, ax: 4,
    });
    assert_eq!(result.zf, 1);
    assert_eq!(result.di, 4);
    assert_eq!(result.cx, 5);
  }

  #[test]
  fn test_repne_scasb_byte_at_start() {
    let result = run_repne_scasb(Test {
      data: vec![0xab, 0, 0, 0],
      df: 0, di: 0, cx: 4, ax: 0xab,
    });
    assert_eq!(result.zf, 1);
    assert_eq!(result.di, 1);
    assert_eq!(result.cx, 3);
  }

  #[test]
  fn test_repne_scasb_byte_at_end() {
    let result = run_repne_scasb(Test {
      data: vec![0, 0, 0, 0xff],
      df: 0, di: 0, cx: 4, ax: 0xff,
    });
    assert_eq!(result.zf, 1);
    assert_eq!(result.di, 4);
    assert_eq!(result.cx, 0);
  }

  #[test]
  fn test_repne_scasb_not_found() {
    let result = run_repne_scasb(Test {
      data: vec![0, 1, 2, 3],
      df: 0, di: 0, cx: 4, ax: 0xab,
    });
    assert_eq!(result.zf, 0);
    assert_eq!(result.cx, 0);
    assert_eq!(result.di, 4);
  }

  #[test]
  fn test_repne_scasb_cx_zero_no_iteration() {
    let result = run_repne_scasb(Test {
      data: vec![0, 0],
      df: 0, di: 0, cx: 0, ax: 0,
    });
    assert_eq!(result.di, 0);
    assert_eq!(result.cx, 0);
  }

  #[test]
  fn test_repne_scasb_strlen() {
    // Find null terminator in "HI\0"
    let result = run_repne_scasb(Test {
      data: vec![b'H', b'I', 0x00],
      df: 0, di: 0, cx: 0xffff, ax: 0,
    });
    assert_eq!(result.zf, 1);
    assert_eq!(result.di, 3);
    assert_eq!(result.cx, 0xffff - 3);
  }

  //////////////////////////////////////////////////////////////////////
  // REPE SCASW
  //////////////////////////////////////////////////////////////////////

  #[test]
  fn test_repe_scasw_found() {
    let result = run_repe_scasw(Test {
      data: vec![6,0, 6,0, 5,0, 1,0, 9,0],
      df: 0, di: 0, cx: 5, ax: 6,
    });
    assert_eq!(result.zf, 0);
    assert_eq!(result.di, 6);
    assert_eq!(result.cx, 2);
  }

  #[test]
  fn test_repe_scasw_found_backwards() {
    let result = run_repe_scasw(Test {
      data: vec![6,0, 6,0, 5,0, 0,0, 9,0],
      df: 1, di: 8, cx: 5, ax: 9,
    });
    assert_eq!(result.zf, 0);
    assert_eq!(result.di, 4);
    assert_eq!(result.cx, 3);
  }

  //////////////////////////////////////////////////////////////////////
  // REPNE SCASW
  //////////////////////////////////////////////////////////////////////

  #[test]
  fn test_repne_scasw_found() {
    let result = run_repne_scasw(Test {
      data: vec![6,0, 6,0, 5,0, 1,0, 9,0],
      df: 0, di: 0, cx: 5, ax: 1,
    });
    assert_eq!(result.zf, 1);
    assert_eq!(result.di, 8);
    assert_eq!(result.cx, 1);
  }

  #[test]
  fn test_repne_scasw_found_backwards() {
    let result = run_repne_scasw(Test {
      data: vec![6,0, 6,0, 5,0, 0,0, 9,0],
      df: 1, di: 8, cx: 5, ax: 0,
    });
    assert_eq!(result.zf, 1);
    assert_eq!(result.di, 4);
    assert_eq!(result.cx, 3);
  }
}
