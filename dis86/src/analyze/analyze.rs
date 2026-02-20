use crate::binary::{Binary, Fmt};
use crate::config::Config;
use crate::segoff::{Seg, SegOff};
use crate::util::range_set::RangeSet;

use super::workqueue::WorkQueue;
use super::code_segment::{CodeSegments};
use super::func_details::{FuncDetails, ReturnKind};

use std::collections::{BTreeMap, HashSet};

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

  pub fn analyze_code_segment(&self, seg: Seg) -> (u32, u32) {
    let code_seg = self.code_segments.find_by_segment(seg).unwrap();

    let mut r = RangeSet::new();

    // Add all ranges implied by function config
    for f in &self.cfg.funcs {
      if f.start.seg != code_seg.primary.seg { continue };
      let Some(end) = &f.end else {
        println!("Unknown end address for {}", f.name);
        continue;
      };
      r.insert(f.start.off.0 as u32, end.off.0 as u32);
    }

    // Add all ranges implied by the text section data
    for t in self.cfg.text_regions_matching_segment(code_seg.primary.seg) {
      r.insert(t.start.off.0 as u32, t.end.off.0 as u32);
    }

    let seg_start = code_seg.primary.skip_off;
    let seg_end = seg_start + code_seg.primary.size;

    if let Some(span_end) = r.span_end() {
      if span_end > seg_end {
        println!("WARN: Function ranges exceed the segment! (expected: {}, got: {})", seg_end, span_end);
      }
    }

    let gaps = r.gaps_within(seg_start, seg_end);
    let mut total_gap = 0;
    for gap in &gaps {
      total_gap += gap.end - gap.start;
    }

    let total_size = code_seg.primary.size;
    let perc = if total_size > 0 {
      100.0 * (1.0 - (total_gap as f64) / (total_size as f64))
    } else {
      100.0
    };
    println!("Percent annotated: {:.2}", perc);

    if gaps.len() > 0 {
      println!("Gaps:");
      for gap in &gaps {
        println!("   [ 0x{:04x}, 0x{:04x} )   size: {}", gap.start, gap.end, gap.end - gap.start);
      }
    }

    (total_gap, total_size)
  }

  pub fn analyze_code_segments_and_report(&self) {
    //self.code_segments.dump();
    let mut total_gap = 0;
    let mut total_size = 0;
    for c in &self.code_segments.0 {
      let seg = c.primary.seg;
      println!("Segment {}", seg);
      println!("===============================");
      let (gap, size) = self.analyze_code_segment(seg);
      total_gap += gap;
      total_size += size;
      println!("");
    }

    let perc = 100.0 * (1.0 - (total_gap as f64) / (total_size as f64));
    println!("Total completion: {} / {} = {:.2} %", total_size - total_gap, total_size, perc);

  }

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

  // Scan known functions to find new functions, then scan those, return a big list of all found functions
  pub fn scan_for_all_functions(&self, emit_annotation_format: bool) {
    let mut workqueue = WorkQueue::new();

    // init work queue with known config functions
    for f in &self.cfg.funcs {
      workqueue.insert(f.start);
    }

    let mut functions = BTreeMap::new();
    while let Some(addr) = workqueue.pop() {
      let result = self.analyze_function_by_start(addr);

      // Add all new calls to the work queue
      if let Ok(details) = &result {
        for call in &details.direct_calls {
          workqueue.insert(*call);
        }
      }

      functions.insert(addr, result);
    }

    if emit_annotation_format {
      // Synthesize annotations
      generate_annotations(&functions, &self.cfg);
    } else {
      // Print out a report
      dump_functions(&functions, &self.cfg);
    }
  }
}

fn dump_functions(functions: &BTreeMap<SegOff, Result<FuncDetails, String>>, cfg: &Config) {
  let mut current_seg = None;
  for (addr, result) in functions {
    let seg = addr.seg;
    if Some(seg) != current_seg {
      println!("");
      println!("Segment {}", seg);
      println!("--------------------------------------------------------------------------------------------------------------------------------------------------------");
      current_seg = Some(seg);
    }

    let name = match cfg.func_lookup(*addr) {
      Some(func) => func.name.clone(),
      None => "UNKNOWN".to_string(),
    };

    print!("Function: {:<35} |  addr: {}  | ", name, addr);
    match result {
      Ok(details) => {
        println!("start: {}  end: {}  indirect_calls: {}",
                 details.start_addr, details.end_addr_inferred, details.indirect_calls);
      }
      Err(err) => {
        println!("error: '{}'", err);
      }
    }
  }
}

struct FunctionNames {
  used: HashSet<String>,
}

impl FunctionNames {
  fn from_cfg(cfg: &Config) -> FunctionNames {
    let mut used = HashSet::new();
    for func in &cfg.funcs {
      if used.get(&func.name).is_some() {
        panic!("Duplicate function name in the config: {}", func.name);
      }
      used.insert(func.name.clone());
    }
    FunctionNames { used }
  }

  fn compute_unique(&mut self, base: &str) -> String {
    // Try a bunch of names until we find a unique one
    // NOTE: THIS IS VERY INEFFICENT... Falls apart to O(n^2) over many calls
    let mut n = 1;
    loop {
      let name = format!("F_{}_unknown_{}", base, n);
      if self.used.get(&name).is_none() {
        self.used.insert(name.clone());
        return name;
      }
      n += 1;
    }
  }
}

fn generate_annotations(functions: &BTreeMap<SegOff, Result<FuncDetails, String>>, cfg: &Config) {
  let mut function_names = FunctionNames::from_cfg(cfg);

  let mut current_seg = None;
  for (addr, result) in functions {
    let seg = addr.seg;

    let seg_name = match cfg.code_seg_lookup(seg) {
      Some(cs) => cs.name.clone(),
      None => format!("_{}", seg),
    };

    if Some(seg) != current_seg {
      println!("");
      println!("    ##################################################################################################################");
      println!("    ## Section {}: {}", seg, seg_name);
      current_seg = Some(seg);
    }


    let (name, ret, args) = match cfg.func_lookup(*addr) {
      Some(func) => (func.name.clone(), func.ret.clone(), func.args),
      None       => (function_names.compute_unique(&seg_name), None, None),
    };

    match result {
      Ok(details) => {
        if details.indirect_calls > 0 {
          print!("    # IGNORED INDIRECT CALLS | {} | {} | ", name, addr);
          println!("start: {}  end: {}  indirect_calls: {}",
                   details.start_addr, details.end_addr_inferred, details.indirect_calls);
        } else {
          let name     = format!("\"{}\",", name);
          let start    = format!("\"{}\",", details.start_addr);
          let end      = format!("\"{}\"", details.end_addr_inferred);
          let flags    = if details.return_kind == ReturnKind::Near {
            ", flags = \"NEAR\""
          } else {
            ""
          };

          let ret_str  = match ret {
            Some(ret) => format!("\"{}\",", ret),
            None      => "None,".to_string(),
          };

          let args_str  = match args {
            Some(args) => format!("{},", args),
            None       => "None,".to_string(),
          };

          println!("    F( {:<30} {:<7} {:<12} {} {}{} ),", name, ret_str, args_str, start, end, flags);
        }
      }
      Err(err) => {
        println!("    # IGNORED ERROR | {} | {} | error: '{}'", name, addr, err);
      }
    }
  }
}
