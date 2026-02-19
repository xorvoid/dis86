use super::code_segment::CodeSegment;
use super::workqueue::WorkQueue;
use crate::segoff::SegOff;
use crate::binary::Binary;
use crate::asm::instr::Instr;
use crate::asm::decode::Decoder;
use crate::asm::intel_syntax::instr_str;
use std::collections::BTreeSet;
use std::fmt;

use crate::analyze::instr_details::{self, ReturnKind, Next, Call};

const DEBUG: bool = false;
//const DEBUG: bool = true;

pub struct FuncDetails {
  pub start_addr:        SegOff,
  pub end_addr_inferred: SegOff,
  pub direct_calls:      BTreeSet<SegOff>,
  pub indirect_calls:    usize,
  pub return_kind:       ReturnKind,
}

impl fmt::Display for FuncDetails {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "start_addr:        {}", self.start_addr)?;
    writeln!(f, "end_addr_inferred: {}", self.end_addr_inferred)?;
    write!(f,   "direct_calls:      [")?;
    for (i, c) in self.direct_calls.iter().enumerate() {
      if i != 0 { write!(f, ", ")?; }
      write!(f, "{}", c)?;
    }
    writeln!(f, "]")?;
    writeln!(f, "indirect_calls:    {}", self.indirect_calls)?;
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
  pub fn build(func_start: SegOff, code_seg: &CodeSegment, binary: &Binary) -> Result<FuncDetails, String> {
    assert!(func_start >= code_seg.start());
    let code_seg_end = code_seg.end();

    let mut workqueue = WorkQueue::new();
    workqueue.insert(func_start);

    let mut largest_addr = func_start;
    let mut direct_calls = BTreeSet::new();
    let mut indirect_calls = 0;
    let mut return_kind = None;

    // Iterate over blocks
    while let Some(loc) = workqueue.pop() {
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
        let end_addr = instr.end_addr();
        if end_addr > largest_addr { largest_addr = end_addr; }

        if DEBUG { println!("INSTR | {} | {}", instr.addr, instr_str(&instr)); }

        // Add instr to the block
        block.instrs.push(instr);

        // Compute instr details
        let details = instr_details::instr_details(&instr)?;

        // Handle calls
        match &details.call {
          Some(Call::Direct(addr)) => {
            if DEBUG { println!("Call to {}", addr); }
            direct_calls.insert(*addr);
          }
          Some(Call::Indirect) => {
            if DEBUG { println!("Indirect call"); }
            indirect_calls += 1;
          }
          None => (),
        }

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
              workqueue.insert(*tgt);
            }
            // Add exits to block
            block.exits = targets;
            block_complete = true;
          }
        }
      }

      if DEBUG { println!("{}", block); }
    }

    Ok(FuncDetails {
      start_addr: func_start,
      end_addr_inferred: largest_addr,
      direct_calls,
      indirect_calls,
      return_kind: return_kind.unwrap(),
    })
  }
}

// FIXME: THIS FUNCTION IS WAY TOO COMPLICATED FOR ITS SIMPLE TASK: APIs NEED IMPROVEMENT
fn decode_one_instr(binary: &Binary, loc: SegOff, end: SegOff) -> Option<Instr> {
  let mut decoder = Decoder::new(binary.region_iter(loc, end));
  let (instr, _raw) = decoder.try_next().unwrap()?;
  Some(instr)
}
