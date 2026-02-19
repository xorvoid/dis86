use crate::analyze::analyze::Analyze;
use crate::config::Config;
use crate::segoff::Seg;

pub fn run(cfg: &Config, exe_path: &str) -> i32 {
  let a = Analyze::new(cfg, exe_path);
  //a.scan_for_all_functions(true);

  a.analyze_code_segments_and_report();

  1
}
