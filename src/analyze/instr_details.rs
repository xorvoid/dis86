use crate::asm::instr::{Instr, Opcode, Operand};
use crate::asm::intel_syntax::instr_str;
use crate::segoff::SegOff;
use std::fmt;

pub enum ReturnKind {
  Near,
  Far,
}

impl fmt::Display for ReturnKind {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ReturnKind::Near => write!(f, "near"),
      ReturnKind::Far  => write!(f, "far"),
    }
  }
}

pub enum Next {
  Fallthrough(SegOff),     // ordinary instruction fallthrough
  Return(ReturnKind),      // return instr (block terminator)
  Jump(Vec<SegOff>),       // jump targets (block terminator)
}

pub struct InstrDetails {
  pub next: Next,
}

// FIXME: UNIFY BACK WITH ir_build::jump_targets
fn jump_targets(ins: &Instr) -> Option<Vec<SegOff>> {
  // Filter for branch instructions
  let (oper_num, fallthrough) = match &ins.opcode {
    Opcode::OP_JA   => (0, true),
    Opcode::OP_JAE  => (0, true),
    Opcode::OP_JB   => (0, true),
    Opcode::OP_JBE  => (0, true),
    Opcode::OP_JCXZ => (1, true),
    Opcode::OP_JE   => (0, true),
    Opcode::OP_JG   => (0, true),
    Opcode::OP_JGE  => (0, true),
    Opcode::OP_JL   => (0, true),
    Opcode::OP_JLE  => (0, true),
    Opcode::OP_JMP  => (0, false),
    Opcode::OP_JMPF => (0, false),
    Opcode::OP_JNE  => (0, true),
    Opcode::OP_JNO  => (0, true),
    Opcode::OP_JNP  => (0, true),
    Opcode::OP_JNS  => (0, true),
    Opcode::OP_JO   => (0, true),
    Opcode::OP_JP   => (0, true),
    Opcode::OP_JS   => (0, true),
    Opcode::OP_LOOP => (1, true),
    _ => return None,
  };

  let mut targets = vec![];

  match &ins.operands[oper_num] {
    Operand::Rel(rel) => {
      targets.push(ins.rel_addr(rel));
    }
    _ => panic!("Unsupported branch instruction: '{}' | {:?}", instr_str(ins), ins.operands[oper_num]),
  };

  if fallthrough {
    targets.push(ins.end_addr());
  }

  Some(targets)
}

pub fn instr_details(ins: &Instr) -> InstrDetails {
  if let Some(tgts) = jump_targets(ins) {
    return InstrDetails { next: Next::Jump(tgts) };
  }

  let mut ret = None;

  // TODO COMPLETE THIS
  match ins.opcode {
    // Ordinary instr
    Opcode::OP_PUSH       => (),
    Opcode::OP_POP        => (),
    Opcode::OP_MOV        => (),
    Opcode::OP_ADD        => (),
    Opcode::OP_SUB        => (),
    Opcode::OP_MUL        => (),
    Opcode::OP_IMUL       => (),
    Opcode::OP_IMUL_TRUNC => (),
    Opcode::OP_XOR        => (),
    Opcode::OP_AND        => (),
    Opcode::OP_OR         => (),
    Opcode::OP_LES        => (),
    Opcode::OP_LEA        => (),
    Opcode::OP_CBW        => (),
    Opcode::OP_CWD        => (),
    Opcode::OP_SHL        => (),
    Opcode::OP_SHR        => (),
    Opcode::OP_ENTER      => (),
    Opcode::OP_LEAVE      => (),
    Opcode::OP_CALL       => (),
    Opcode::OP_CALLF      => (),
    Opcode::OP_CMP        => (),
    Opcode::OP_INC        => (),
    Opcode::OP_DEC        => (),
    Opcode::OP_INT        => (),
    Opcode::OP_CLD        => (),

    Opcode::OP_RET        => ret = Some(ReturnKind::Near),
    Opcode::OP_RETF       => ret = Some(ReturnKind::Far),

    _ => panic!("UNIMPL OPCODE"),
  };

  if let Some(ret) = ret {
    return InstrDetails { next: Next::Return(ret) };
  }

  InstrDetails { next: Next::Fallthrough(ins.end_addr()) }
}
