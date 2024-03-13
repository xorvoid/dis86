use dis86::app;

fn print_help(appname: &str) {
  eprintln!("usage: {} <mode> [<MODE-SPECIFIC-OPTIONS>]", appname);
  eprintln!("");
  eprintln!("MODES:");
  eprintln!("  dis       disassemble the binary and emit intel syntax");
  eprintln!("  decomp    decompile the binary");
}

fn run_main() -> i32 {
  let args: Vec<_> = std::env::args().collect();
  if args.len() < 2 {
    print_help(&args[0]);
    return 1;
  }
  let mode = &args[1];

  if      mode == "dis"    { app::dis::run(&args[0]); return 0; }
  else if mode == "decomp" { app::decomp::run(&args[0]); return 0; }

  else {
    eprintln!("Error: Unknown mode '{}'", mode);
    print_help(&args[0]);
    return 2;
  }
}

fn main() {
  std::process::exit(run_main());
}
