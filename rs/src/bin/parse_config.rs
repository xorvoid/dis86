use std::env;
use dis::decomp::config::Config;

fn main() {
  let args: Vec<_> = env::args().collect();
  let cfg = Config::from_file(&args[1]).unwrap();
  println!("{:#?}", cfg);
}
