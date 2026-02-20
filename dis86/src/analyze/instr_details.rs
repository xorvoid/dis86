use crate::asm::instr::{Instr, Opcode, Operand};
use crate::asm::intel_syntax::instr_str;
use crate::segoff::{SegOff, Seg, Off};
use crate::binary::Binary;
use std::fmt;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ReturnKind {
  Near,
  Far,
  Interrupt,
}

impl fmt::Display for ReturnKind {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ReturnKind::Near      => write!(f, "near"),
      ReturnKind::Far       => write!(f, "far"),
      ReturnKind::Interrupt => write!(f, "interrupt"),
    }
  }
}

pub enum Next {
  Fallthrough(SegOff),     // ordinary instruction fallthrough
  Return(ReturnKind),      // return instr (block terminator)
  Jump(Vec<SegOff>),       // jump targets (block terminator)
}

pub enum Call {
  Direct(SegOff),
  Indirect,
}

fn determine_callf(ins: &Instr, binary: &Binary) -> Result<Call, String> {
  // FIXME: UNIFY WITH ir_build::process_callf
  if let Operand::Far(far) = &ins.operands[0] {
    let seg = if ins.addr.seg.is_overlay() {
      // Far calls from overlays need to be remapped
      binary.remap_to_segment(far.seg)
    } else {
      // Otherwise: Normal
      Seg::Normal(far.seg)
    };
    let addr = SegOff { seg, off: Off(far.off) };
    Ok(Call::Direct(addr))
  } else if let Operand::Mem(_) = &ins.operands[0] {
    Ok(Call::Indirect)
  } else {
    Err(format!("Unsupported operand to CALLF for '{}'", instr_str(ins)))
  }
}

fn determine_calln(ins: &Instr, binary: &Binary) -> Result<Call, String> {
  if let Operand::Rel(rel) = &ins.operands[0] {
    let addr = ins.rel_addr(rel);
    Ok(Call::Direct(addr))
  } else if let Operand::Mem(_) = &ins.operands[0] {
    Ok(Call::Indirect)
  } else {
    Err(format!("Unsupported operand to CALL for '{}'", instr_str(ins)))
  }
}

pub struct InstrDetails {
  pub next: Next,
  pub call: Option<Call>,
}

// FIXME: UNIFY BACK WITH ir_build::jump_targets
fn jump_targets(ins: &Instr) -> Result<Option<Vec<SegOff>>, String> {
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
    _ => return Ok(None),
  };

  let mut targets = vec![];

  match &ins.operands[oper_num] {
    Operand::Rel(rel) => {
      targets.push(ins.rel_addr(rel));
    }
    _ => {
      return Err(format!("Unsupported branch instruction: '{}' | {:?}", instr_str(ins), ins.operands[oper_num]));
    }
  };

  if fallthrough {
    targets.push(ins.end_addr());
  }

  Ok(Some(targets))
}

pub fn instr_details(ins: &Instr, binary: &Binary) -> Result<InstrDetails, String> {
  let is_overlay = ins.addr.seg.is_overlay();

  if let Some(tgts) = jump_targets(ins)? {
    return Ok(InstrDetails { next: Next::Jump(tgts), call: None });
  }

  let mut call = None;
  let mut ret = None;

  // NOTE: Need to complete this. We don't support everything because we're being overly conservative.
  // Need to carefully think about the other instructions before adding them.
  match ins.opcode {
    Opcode::OP_AAA        => (),
    Opcode::OP_AAS        => (),
    Opcode::OP_ADC        => (),
    Opcode::OP_ADD        => (),
    Opcode::OP_AND        => (),
    Opcode::OP_CALL       => call = Some(determine_calln(ins, binary)?),
    Opcode::OP_CALLF      => call = Some(determine_callf(ins, binary)?),
    Opcode::OP_CBW        => (),
    Opcode::OP_CLC        => (),
    Opcode::OP_CLD        => (),
    Opcode::OP_CLI        => (),
    Opcode::OP_CMC        => (),
    Opcode::OP_CMP        => (),
    Opcode::OP_CMPS       => (),
    Opcode::OP_CWD        => (),
    Opcode::OP_DAA        => (),
    Opcode::OP_DAS        => (),
    Opcode::OP_DEC        => (),
    Opcode::OP_DIV        => (),
    Opcode::OP_ENTER      => (),
    // Opcode::OP_HLT     => (),
    Opcode::OP_IMUL       => (),
    Opcode::OP_IMUL_TRUNC => (),
    Opcode::OP_IN         => (),
    Opcode::OP_INC        => (),
    Opcode::OP_INS        => (),
    Opcode::OP_INT        => (),
    // Opcode::OP_INTO    => (),
    // Opcode::OP_INVAL   => (),
    Opcode::OP_IRET       => (),
    // Opcode::OP_JA      => (),
    // Opcode::OP_JAE     => (),
    // Opcode::OP_JB      => (),
    // Opcode::OP_JBE     => (),
    // Opcode::OP_JCXZ    => (),
    // Opcode::OP_JE      => (),
    // Opcode::OP_JG      => (),
    // Opcode::OP_JGE     => (),
    // Opcode::OP_JL      => (),
    // Opcode::OP_JLE     => (),
    // Opcode::OP_JMP     => (),
    // Opcode::OP_JMPF    => (),
    // Opcode::OP_JNE     => (),
    // Opcode::OP_JNO     => (),
    // Opcode::OP_JNP     => (),
    // Opcode::OP_JNS     => (),
    // Opcode::OP_JO      => (),
    // Opcode::OP_JP      => (),
    // Opcode::OP_JS      => (),
    Opcode::OP_LAHF       => (),
    Opcode::OP_LDS        => (),
    Opcode::OP_LEA        => (),
    Opcode::OP_LEAVE      => (),
    Opcode::OP_LES        => (),
    Opcode::OP_LODS       => (),
    // Opcode::OP_LOOP    => (),
    // Opcode::OP_LOOPE   => (),
    // Opcode::OP_LOOPNE  => (),
    Opcode::OP_MOV        => (),
    Opcode::OP_MOVS       => (),
    Opcode::OP_MUL        => (),
    Opcode::OP_NEG        => (),
    Opcode::OP_NOP        => (),
    Opcode::OP_NOT        => (),
    Opcode::OP_OR         => (),
    Opcode::OP_OUT        => (),
    Opcode::OP_OUTS       => (),
    Opcode::OP_POP        => (),
    Opcode::OP_POPA       => (),
    Opcode::OP_POPF       => (),
    Opcode::OP_PUSH       => (),
    Opcode::OP_PUSHA      => (),
    Opcode::OP_PUSHF      => (),
    Opcode::OP_RCL        => (),
    Opcode::OP_RCR        => (),
    Opcode::OP_RET        => ret = Some(ReturnKind::Near),
    Opcode::OP_RETF       => ret = Some(ReturnKind::Far),
    Opcode::OP_ROL        => (),
    Opcode::OP_ROR        => (),
    Opcode::OP_SAHF       => (),
    Opcode::OP_SAR        => (),
    Opcode::OP_SBB        => (),
    Opcode::OP_SCAS       => (),
    Opcode::OP_SETO       => (),
    Opcode::OP_SETNO      => (),
    Opcode::OP_SETA       => (),
    Opcode::OP_SETAE      => (),
    Opcode::OP_SETB       => (),
    Opcode::OP_SETBE      => (),
    Opcode::OP_SETE       => (),
    Opcode::OP_SETG       => (),
    Opcode::OP_SETGE      => (),
    Opcode::OP_SETL       => (),
    Opcode::OP_SETLE      => (),
    Opcode::OP_SETP       => (),
    Opcode::OP_SETS       => (),
    Opcode::OP_SETNE      => (),
    Opcode::OP_SETNP      => (),
    Opcode::OP_SETNS      => (),
    Opcode::OP_SHL        => (),
    Opcode::OP_SHR        => (),
    Opcode::OP_STC        => (),
    Opcode::OP_STD        => (),
    Opcode::OP_STI        => (),
    Opcode::OP_STOS       => (),
    Opcode::OP_SUB        => (),
    Opcode::OP_TEST       => (),
    Opcode::OP_XCHG       => (),
    Opcode::OP_XLAT       => (),
    Opcode::OP_XOR        => (),

    _ => panic!("UNIMPL Opcode for {}", instr_str(ins)),
  };

  if let Some(ret) = ret {
    return Ok(InstrDetails { next: Next::Return(ret), call: None });
  }

  Ok(InstrDetails { next: Next::Fallthrough(ins.end_addr()), call })
}
