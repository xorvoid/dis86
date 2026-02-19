use crate::binary::{Binary, Fmt};
use crate::config::Config;
use crate::segoff::{Seg, SegOff};
use crate::util::range_set::RangeSet;

use super::workqueue::WorkQueue;
use super::code_segment::{CodeSegment, CodeSegments, CodeDetail};
use super::func_details::FuncDetails;

use std::collections::BTreeMap;

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
    FuncDetails::build(func.start, code_seg, &self.binary).unwrap() // HAX FIXME
  }

  pub fn analyze_function_by_start(&self, start: SegOff) -> Result<FuncDetails, String> {
    let Some(code_seg) = self.code_segments.find_by_segment(start.seg) else {
      return Err(format!("Failed to find code segement"));
    };
    assert!(start >= code_seg.start());
    FuncDetails::build(start, code_seg, &self.binary)
  }

  pub fn analyze_code_segment_NEW(&self, seg: Seg) {
    let code_seg = self.code_segments.find_by_segment(seg).unwrap();
    let detail = CodeDetail::build(code_seg, &self.cfg);

    for func in &detail.function_entries {
      let func_detail = FuncDetails::build(func.start, code_seg, &self.binary).unwrap();  // HAX FIXME

      println!("function name: {}", func.name);
      println!("-----------------------------");
      println!("{}", func_detail);
    }
  }

  // Scan known functions to find new functions, then scan those, return a big list of all found functions
  pub fn scan_for_all_functions(&self) {
    let mut workqueue = WorkQueue::new();

    // init work queue with known config functions
    for f in &self.cfg.funcs {
      workqueue.insert(f.start);
    }

    let mut functions = BTreeMap::new();
    while let Some(addr) = workqueue.pop() {
      let result = self.analyze_function_by_start(addr);
      functions.insert(addr, result);
    }

    // Print out a report
    let mut current_seg = None;
    for (addr, result) in functions {
      let seg = addr.seg;
      if Some(seg) != current_seg {
        println!("");
        println!("Segment {}", seg);
        println!("--------------------------------------------------------------------------------------------------------------------------------------------------------");
        current_seg = Some(seg);
      }

      let name = match self.cfg.func_lookup(addr) {
        Some(func) => func.name.clone(),
        None => "UNKNOWN".to_string(),
      };

      print!("Function: {:<25} |  addr: {}  | ", name, addr);
      match result {
        Ok(details) => {
          println!("start: {}  end: {}", details.start_addr, details.end_addr_inferred);
          // Add all new calls to the work queue
          for call in &details.calls {
            workqueue.insert(*call);
          }
        }
        Err(err) => {
          println!("error: '{}'", err);
        }
      }
    }
  }
}
