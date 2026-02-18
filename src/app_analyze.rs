use crate::binary::{Binary, Fmt};
use crate::config::{Config, Func};
use crate::segoff::{Seg, Off, SegOff};
use crate::util::range_set::RangeSet;
use crate::binfmt::mz;

use crate::analyze::code_segment::{self, CodeSegment};
use crate::analyze::instr_details::{self, InstrDetails};
use crate::analyze::func_details::{self, FuncDetails};

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

  fn analyze_function(&self, name: &str) -> FuncDetails {
    let func = self.cfg.func_lookup_by_name(name).unwrap(); // FIXME
    let code_seg = self.find_code_segment_for_function(func).unwrap(); // FIXME
    assert!(func.start >= code_seg.start());

    let fmt = Fmt::Exe(self.exe_path.to_string());
    let binary = Binary::from_fmt(&fmt, Some(&self.cfg)).unwrap();

    func_details::func_details(func, code_seg, &binary)
  }
}

pub fn run(cfg: &Config, exe_path: &str) -> i32 {
  let mut a = Analyze::new(cfg, exe_path);
  a.code_segments = Some(code_segment::find_code_segments(&a.exe_path)); // KLUDGY FIXME

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
