use super::code_segment::CodeSegment;
use crate::segoff::SegOff;
use crate::config::Func;
use crate::binary::Binary;
use crate::asm::instr::Instr;
use crate::asm::decode::Decoder;
use crate::asm::intel_syntax::instr_str;
use std::collections::{BTreeSet, HashSet};
use std::fmt;

use crate::analyze::instr_details::{self, ReturnKind, Next};

pub struct FuncDetails {
  pub inferred_end_addr: SegOff,
  pub return_kind:       ReturnKind,
}

impl fmt::Display for FuncDetails {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "inferred_end_addr: {}", self.inferred_end_addr)?;
    writeln!(f, "return_kind:       {}", self.return_kind)?;
    Ok(())
  }
}

struct Block {
  start: SegOff,
  exits: Vec<SegOff>,
  instrs: Vec<Instr>,
}

impl fmt::Display for Block {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "Block {} | exits: [", self.start)?;
    for (i, exit) in self.exits.iter().enumerate() {
      if i != 0 { write!(f, ", ")?; }
      write!(f, "{}", exit)?;
    }
    writeln!(f, "]")?;
    writeln!(f, "------------------------------------")?;
    for instr in &self.instrs {
      writeln!(f, "{} | {}", instr.addr, instr_str(instr))?;
    }
    Ok(())
  }
}

impl FuncDetails {
  pub fn build(func: &Func, code_seg: &CodeSegment, binary: &Binary) -> FuncDetails {
    assert!(func.start >= code_seg.start());
    let code_seg_end = code_seg.end();

    let mut discovered = HashSet::new();
    let mut queued = BTreeSet::new();

    discovered.insert(func.start);
    queued.insert(func.start);

    let mut largest_addr = func.start;
    let mut return_kind = None;

    // Iterate over blocks
    while let Some(loc) = queued.pop_first() {
      let mut block = Block {
        start: loc,
        exits: vec![], // not yet known
        instrs: vec![],
      };

      let mut addr = loc;

      // Iterate over instructions in the block (until we reach a terminator)
      let mut block_complete = false;
      while !block_complete {
        // Decode next instruction
        let instr = decode_one_instr(&binary, addr, code_seg_end).unwrap();
        assert!(instr.rep.is_none()); // UNIMPL
        let end_addr = instr.end_addr();
        if end_addr > largest_addr { largest_addr = end_addr; }

        // Add instr to the block
        block.instrs.push(instr);

        // Compute instr details
        let mut details = instr_details::instr_details(&instr);

        // Figure out what to do next
        match details.next {
          Next::Fallthrough(target) => {
            addr = target;
            continue;
          }
          Next::Return(ret) => {
            if return_kind.is_none() {
              return_kind = Some(ret);
            }
            block.exits = vec![];
            block_complete = true;
          }
          Next::Jump(targets) => {
            for tgt in &targets {
              if discovered.get(tgt).is_some() { continue; }
              discovered.insert(*tgt);
              queued.insert(*tgt);
            }
            // Add exits to block
            block.exits = targets;
            block_complete = true;
          }
        }
      }

      println!("{}", block);
    }

    FuncDetails {
      inferred_end_addr: largest_addr,
      return_kind: return_kind.unwrap(),
    }
  }
}

// FIXME: THIS FUNCTION IS WAY TOO COMPLICATED FOR ITS SIMPLE TASK: APIs NEED IMPROVEMENT
fn decode_one_instr(binary: &Binary, loc: SegOff, end: SegOff) -> Option<Instr> {
  let mut decoder = Decoder::new(binary.region_iter(loc, end));
  let (instr, _raw) = decoder.try_next().unwrap()?;
  Some(instr)
}
