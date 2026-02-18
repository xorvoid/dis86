//use crate::binary::Binary;
use crate::binary::{Binary, Fmt};
use crate::binfmt::mz;
use crate::config::{Config, Func};
use crate::segoff::{Seg, Off, SegOff};
use crate::util::range_set::RangeSet;
use crate::asm;
use crate::asm::instr::{self, Instr, Opcode};
use crate::asm::decode::Decoder;
use std::collections::HashSet;
use std::fmt;

// FIXME: THIS WRAPPER IS CRAZY
fn instr_str(ins: &instr::Instr) -> String {
  crate::asm::intel_syntax::format(ins.addr, Some(ins), &[], false).unwrap()
}

// FIXME: UNIFY BACK WITH ir_build::jump_targets
fn jump_targets(ins: &instr::Instr) -> Option<Vec<SegOff>> {
  // Filter for branch instructions
  let (oper_num, fallthrough) = match &ins.opcode {
    instr::Opcode::OP_JA   => (0, true),
    instr::Opcode::OP_JAE  => (0, true),
    instr::Opcode::OP_JB   => (0, true),
    instr::Opcode::OP_JBE  => (0, true),
    instr::Opcode::OP_JCXZ => (1, true),
    instr::Opcode::OP_JE   => (0, true),
    instr::Opcode::OP_JG   => (0, true),
    instr::Opcode::OP_JGE  => (0, true),
    instr::Opcode::OP_JL   => (0, true),
    instr::Opcode::OP_JLE  => (0, true),
    instr::Opcode::OP_JMP  => (0, false),
    instr::Opcode::OP_JMPF => (0, false),
    instr::Opcode::OP_JNE  => (0, true),
    instr::Opcode::OP_JNO  => (0, true),
    instr::Opcode::OP_JNP  => (0, true),
    instr::Opcode::OP_JNS  => (0, true),
    instr::Opcode::OP_JO   => (0, true),
    instr::Opcode::OP_JP   => (0, true),
    instr::Opcode::OP_JS   => (0, true),
    instr::Opcode::OP_LOOP => (1, true),
    _ => return None,
  };

  let mut targets = vec![];

  match &ins.operands[oper_num] {
    instr::Operand::Rel(rel) => {
      targets.push(ins.rel_addr(rel));
    }
    _ => panic!("Unsupported branch instruction: '{}' | {:?}", instr_str(ins), ins.operands[oper_num]),
  };

  if fallthrough {
    targets.push(ins.end_addr());
  }

  Some(targets)
}

enum ReturnKind {
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

struct FunctionDetails {
  inferred_end_addr: SegOff,
  return_kind: ReturnKind,
}

struct InstrDetails {
  next: Vec<SegOff>,
  fallthrough: bool,
  ret_near: bool,
}

fn instr_details(ins: &Instr) -> InstrDetails {
  if let Some(tgts) = jump_targets(ins) {
    return InstrDetails {
      next: tgts,
      fallthrough: false,
      ret_near: false,
    };
  }

  let mut is_ret = false;
  let mut ret_near = false;

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

    Opcode::OP_RET        => { is_ret = true; ret_near = true; }
    Opcode::OP_RETF       => is_ret = true,

    _ => panic!("UNIMPL OPCODE"),
  };

  if is_ret {
    return InstrDetails {
      next: vec![],
      fallthrough: false,
      ret_near,
    };
  }

  InstrDetails {
    next: vec![ins.end_addr()],
    fallthrough: true,
    ret_near: false,
  }
}

#[derive(Debug, Clone)]
struct Region {
  seg: Seg,
  // minoff: u32,
  // maxoff: u32,
  skip_off: u32, // The segment data might not start at 0
  size: u32,
}

#[derive(Debug)]
struct CodeSegment {
  primary: Region,
  stub: Option<Region>,
}

impl CodeSegment {
  fn start(&self) -> SegOff {
    SegOff { seg: self.primary.seg, off: Off(self.primary.skip_off as u16) }
  }
  fn end(&self) -> SegOff {
    let end_off: u16 = (self.primary.skip_off + self.primary.size).try_into().unwrap();
    SegOff { seg: self.primary.seg, off: Off(end_off) }
  }
}

fn dump_code_segments(code_segments: &[CodeSegment]) {
  for (i, s) in code_segments.iter().enumerate() {
    let seg_str = format!("{},", s.primary.seg);
    let mut ex = "".to_string();
    if let Some(stub) = &s.stub {
      ex = format!("    entry-seg: {},   entry-seg-size: {}",
                   stub.seg, stub.size);
    }
    println!("{:3} | seg: {:<15} skip_off: 0x{:04x},   size: {:>6}{}",
             i, seg_str, s.primary.skip_off, s.primary.size, ex);
  }
}


struct Analyze {
  cfg: Config,
  exe_path: String,
  code_segments: Option<Vec<CodeSegment>>,
}

impl Analyze {
  fn new(cfg: &Config, exe_path: &str) -> Self {
    Self {
      cfg: cfg.clone(),
      exe_path: exe_path.to_string(),
      code_segments: None,
    }
  }

  fn dump_info(&self) {
    let Ok(data) = std::fs::read(&self.exe_path) else {
      panic!("Failed to read file: {}", self.exe_path);
    };
    let exe = mz::Exe::decode(&data).unwrap();
    exe.print();
  }

  // Should basically match those that were manually found in annotations.py
  fn find_code_segments(&mut self) {
    let Ok(data) = std::fs::read(&self.exe_path) else {
      panic!("Failed to read file: {}", self.exe_path);
    };
    let exe = mz::Exe::decode(&data).unwrap();
    let seginfo = exe.seginfo.unwrap(); // FIXME
    let ovr = exe.ovr.as_ref().unwrap(); // FIXME

    // Collect ordinary code segments and stub segments
    let mut code_segments = vec![];
    let mut stub_segments = vec![];
    for s in seginfo {
      let region = Region {
        seg: Seg::Normal(s.seg),
        skip_off: s.minoff as u32,
        size: s.size() as u32,
      };
      if s.typ == mz::SegInfoType::CODE && s.size() != 0 {
        code_segments.push(CodeSegment { primary: region, stub: None, });
      }
      else if s.typ == mz::SegInfoType::STUB {
        stub_segments.push(region);
      }
    }

    // Iterate all overlay segments and match them up with the stubs
    for (i, seg) in ovr.segs.iter().enumerate() {
      let end = seg.data_offset + seg.segment_size as u32;
      let region = Region {
        seg: Seg::Overlay(i as u16),
        skip_off: 0,
        size: seg.segment_size as u32,
      };
      let stub = stub_segments[i].clone();
      assert!(stub.skip_off == 0);
      code_segments.push(CodeSegment { primary: region, stub: Some(stub) });
    }

    self.code_segments = Some(code_segments);
  }

  fn analyze_code_segment(&self, code_seg: &CodeSegment) {
    let mut r = RangeSet::new();
    for f in &self.cfg.funcs {
      if f.start.seg != code_seg.primary.seg { continue };
      let Some(end) = &f.end else {
        println!("Unknown end address for {}", f.name);
        continue;
      };
      r.insert(f.start.off.0 as u32, end.off.0 as u32);
    }

    let seg_start = code_seg.primary.skip_off;
    let seg_end = seg_start + code_seg.primary.size;

    if let Some(span_end) = r.span_end() {
      if span_end > seg_end {
        println!("WARN: Function ranges exceed the segment! (expected: {}, got: {})", seg_end, span_end);
      }
    }

    let gaps = r.gaps_within(seg_start, seg_end);

    if gaps.len() == 0 {
      println!("Complete!");
      return;
    }

    println!("");
    println!("Gaps:");
    println!("-------------------------------");
    for gap in gaps {
      println!("  [ 0x{:04x}, 0x{:04x} )   size: {}", gap.start, gap.end, gap.end - gap.start);
    }
  }

  fn find_code_segment_for_function(&self, func: &Func) -> Option<&CodeSegment> {
    let code_segments = self.code_segments.as_ref().unwrap();
    let func_seg = func.start.seg;
    for c in code_segments {
      if c.primary.seg == func.start.seg {
        return Some(c);
      }
    }
    None
  }

  fn analyze_function(&self, name: &str) -> FunctionDetails {
    let func = self.cfg.func_lookup_by_name(name).unwrap(); // FIXME
    let code_seg = self.find_code_segment_for_function(func).unwrap(); // FIXME
    assert!(func.start >= code_seg.start());

    let fmt = Fmt::Exe(self.exe_path.to_string());
    let binary = Binary::from_fmt(&fmt, Some(&self.cfg)).unwrap();

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

      let mut d = instr_details(&instr);
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

    FunctionDetails {
      inferred_end_addr: largest_addr,
      return_kind: if ret_near { ReturnKind::Near } else { ReturnKind::Far },
    }
  }
}


fn decode_one_instr(binary: &Binary, loc: SegOff, end: SegOff) -> Option<Instr> {
  // FIXME: THIS FUNCTION IS WAY TOO COMPLICATED FOR ITS SIMPLE TASK: APIs NEED IMPROVEMENT
  let mut decoder = Decoder::new(binary.region_iter(loc, end));
  let (instr, raw) = decoder.try_next().unwrap()?;
  Some(instr)
}

pub fn run(cfg: &Config, exe_path: &str) -> i32 {
  let mut a = Analyze::new(cfg, exe_path);
  a.find_code_segments();

  let d = a.analyze_function("F_jawn_unknown_17");

  println!("========================");
  println!("infered end:   {}", d.inferred_end_addr);
  println!("return method: {}", d.return_kind);

  //dump_info(exe_path);

  // let code_segments = find_code_segements(exe_path);
  // //dump_code_segments(&code_segments);
  // for c in &code_segments[..5] {
  //   println!("Segment {}", c.primary.seg);
  //   println!("===============================");
  //   analyze_code_segment(cfg, c);
  //   println!("");
  // }

  //analyze_function(cfg, exe_path, "F_navigator");

  1
}
