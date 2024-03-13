use std::env;
use dis86::decomp::config::Config;

fn main() {
  let args: Vec<_> = env::args().collect();
  let cfg = Config::from_path(&args[1]).unwrap();
  println!("{:#?}", cfg);
}
