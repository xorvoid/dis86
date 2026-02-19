use crate::analyze::analyze::Analyze;
use crate::config::Config;
use crate::segoff::Seg;

pub fn run(cfg: &Config, exe_path: &str) -> i32 {
  let a = Analyze::new(cfg, exe_path);

  // let d = a.analyze_function("F_jawn_unknown_17");
  // println!("========================");
  // println!("{}", d);

  // let seg: Seg = "overlay_0000".parse().unwrap();
  // println!("Segment: {}", seg);
  // println!("-----------------------------------");
  // a.analyze_code_segment_NEW(seg);

  a.scan_for_all_functions();

  1
}
