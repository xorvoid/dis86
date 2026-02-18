use super::code_segment::CodeSegment;
use crate::segoff::SegOff;
use crate::config::Func;
use crate::binary::Binary;
use crate::asm::instr::{self, Instr, Opcode};
use crate::asm::decode::Decoder;
use crate::asm::intel_syntax::instr_str;
use std::collections::HashSet;
use std::fmt;

use crate::analyze::instr_details;

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

pub struct FuncDetails {
  pub inferred_end_addr: SegOff,
  pub return_kind: ReturnKind,
}

pub fn func_details(func: &Func, code_seg: &CodeSegment, binary: &Binary) -> FuncDetails {
  assert!(func.start >= code_seg.start());

  let mut work = vec![]; // used as a stack
  work.push(func.start);

  let mut known_targets = HashSet::new();
  known_targets.insert(func.start);

  let mut block_active = false;
  let mut largest_addr = func.start;
  let mut ret_near = false;

  let end = code_seg.end();
  while let Some(loc) = work.pop() {
    if !block_active {
      println!("Block: {}", loc);
      println!("-------------------------------");
      block_active = true;
    }

    let instr = decode_one_instr(&binary, loc, end).unwrap();
    assert!(instr.rep.is_none()); // UNIMPL
    let end_addr = instr.end_addr();
      if end_addr > largest_addr { largest_addr = end_addr; }

    // FIXME: THIS API IS INSANE
    println!("{} | {}", loc, instr_str(&instr));

    let mut d = instr_details::instr_details(&instr);
    if !ret_near && d.ret_near {
      ret_near = true;
    }
    if d.fallthrough {
      work.append(&mut d.next);
      continue;
    }

    // block done
    block_active = false;
    println!("");

    for tgt in d.next {
      if known_targets.get(&tgt).is_none() {
        known_targets.insert(tgt);
        work.push(tgt);
      }
    }
  }

  FuncDetails {
    inferred_end_addr: largest_addr,
    return_kind: if ret_near { ReturnKind::Near } else { ReturnKind::Far },
  }
}

// FIXME: THIS FUNCTION IS WAY TOO COMPLICATED FOR ITS SIMPLE TASK: APIs NEED IMPROVEMENT
fn decode_one_instr(binary: &Binary, loc: SegOff, end: SegOff) -> Option<Instr> {
  let mut decoder = Decoder::new(binary.region_iter(loc, end));
  let (instr, raw) = decoder.try_next().unwrap()?;
  Some(instr)
}
