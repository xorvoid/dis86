use super::mem::*;
use super::cpu::*;
use crate::asm::decode::Decoder;
use crate::asm::instr::{self, Instr, Opcode, Operand};
use crate::asm::intel_syntax::instr_str;

#[derive(Debug)]
enum Value {
  U8(u8),
  U16(u16),
  U32(u32),
}

#[derive(Default)]
pub struct Machine {
  pub halted: bool,
  pub mem: Memory,
  pub cpu: Cpu,
}

impl Machine {
  pub fn halted(&self) -> bool {
    self.halted
  }

  pub fn reg(&self, r: Register) -> u16 {
    assert!(r.size == 2);
    self.cpu.regs[r.idx as usize]
  }

  pub fn reg_set(&mut self, r: Register, val: u16) {
    assert!(r.size == 2);
    self.cpu.regs[r.idx as usize] = val;
  }

  pub fn read(instr: &Instr, oper: usize) -> Value {
    match &instr.operands[oper] {
      Operand::Imm(imm) => {
        match imm.sz {
          instr::Size::Size8  => Value::U8(imm.val as u8),
          instr::Size::Size16 => Value::U16(imm.val),
          _ => panic!("unsupported size"),
        }
      }
      _ => panic!("unsupported operand"),
    }
  }

  pub fn write(&mut self, instr: &Instr, oper: usize, val: Value) {
    match &instr.operands[oper] {
      Operand::Reg(r) => {
        let reg = convert_reg(r.0);
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
      _ => panic!("unsupported operand"),
    }
  }

  pub fn step(&mut self) -> Result<(), String> {
    let addr = SegOff::new_normal(self.reg(CS), self.reg(IP));
    println!("addr: {}", addr);

    // let slice = self.mem.slice_starting_at(addr);
    // //println!("mem: {:?}", &slice[..16]);
    // hexdump(&slice[..16]);

    let instr = decode_instr(&self.mem, addr)?;
    println!("{}   {}", addr, instr_str(&instr));
    println!("{:?}", instr);

    if instr.rep.is_some() { panic!("REP prefix is not yet implemented"); }

    match instr.opcode {
      Opcode::OP_MOV => {
        assert!(instr.operands.len() == 2);
        self.write(&instr, 0, Self::read(&instr, 1));
      }
      _ => {
        panic!("Unimplmented opcode: {}", instr.opcode.name());
      }
    }

    println!("Halting!");
    self.halted = true;

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

fn hexdump(data: &[u8]) {
  for (i, chunk) in data.chunks(16).enumerate() {
    let addr = i * 16;
    let hex: Vec<String> = chunk.iter().map(|b| format!("{:02x}", b)).collect();
    println!("{:08x}  {}", addr, hex.join(" "));
  }
}

// FIXME: Shouldn't have to remap this at all.. would love to use it directly or with a trivial offsetting
fn convert_reg(r: instr::Reg) -> Register {
  match r {
    instr::Reg::AX => AX,
    instr::Reg::BX => BX,
    instr::Reg::CX => CX,
    instr::Reg::DX => DX,
    instr::Reg::SI => SI,
    instr::Reg::DI => DI,
    instr::Reg::BP => BP,
    instr::Reg::SP => SP,
    instr::Reg::IP => IP,
    instr::Reg::CS => CS,
    instr::Reg::DS => DS,
    instr::Reg::ES => ES,
    instr::Reg::SS => SS,
    instr::Reg::FLAGS => FLAGS,
    _ => panic!("unimpl register"),
  }
}
