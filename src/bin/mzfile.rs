use dis86::binfmt::mz;

fn main() {
  let args: Vec<String> = std::env::args().collect();
  if args.len() != 2 {
    eprintln!("usage: {} <path>", args[0]);
    std::process::exit(1);
  }
  let path = &args[1];

  let Ok(data) = std::fs::read(path) else {
    panic!("Failed to read file: {}", path);
  };

  let exe = mz::Exe::decode(&data).unwrap();
  //println!("{:#?}", exe);
  exe.print();
}
