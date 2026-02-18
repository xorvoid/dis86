use crate::binary::{Binary, Fmt};
use crate::config::Config;
use crate::segoff::Seg;
use crate::util::range_set::RangeSet;

use crate::analyze::code_segment::{CodeSegment, CodeSegments};
use crate::analyze::func_details::FuncDetails;

pub struct Analyze {
  cfg: Config,
  binary: Binary,
  pub code_segments: CodeSegments,
}

impl Analyze {
  pub fn new(cfg: &Config, exe_path: &str) -> Self {
    let fmt = Fmt::Exe(exe_path.to_string());
    let binary = Binary::from_fmt(&fmt, Some(cfg)).unwrap();
    let code_segments = CodeSegments::from_binary(&binary);

    Self {
      cfg: cfg.clone(),
      binary,
      code_segments,
    }
  }

  pub fn dump_info(&self) {
    self.binary.exe().unwrap().print();
  }

  pub fn analyze_code_segment(&self, seg: Seg) {
    let code_seg = self.code_segments.find_by_segment(seg).unwrap();

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

  // pub fn analyze_code_segments_and_report(&self) {
  //   self.code_segments.dump();
  //   for c in &self.code_segments.0 {
  //     println!("Segment {}", c.primary.seg);
  //     println!("===============================");
  //     self.analyze_code_segment(c);
  //     println!("");
  //   }
  // }

  pub fn analyze_function(&self, name: &str) -> FuncDetails {
    let func = self.cfg.func_lookup_by_name(name).unwrap(); // FIXME
    let code_seg = self.code_segments.find_for_function(func).unwrap(); // FIXME
    assert!(func.start >= code_seg.start());
    FuncDetails::build(func, code_seg, &self.binary)
  }
}
