use crate::analyze::analyze::Analyze;
use crate::config::Config;

pub fn run(cfg: &Config, exe_path: &str) -> i32 {
  let a = Analyze::new(cfg, exe_path);

  let d = a.analyze_function("F_jawn_unknown_17");
  println!("========================");
  println!("{}", d);

  1
}
