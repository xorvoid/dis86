use dis86::emu86;

fn print_help() {
  let appname = std::env::args().next().unwrap();
  println!("usage: {} OPTIONS", appname);
  println!("");
  println!("REQUIRED OPTIONS:");
  println!("  --exe             path to MZ format exe on the filesystem (required)");
}

#[derive(Debug)]
struct Args {
  exe: String,
}

fn parse_args() -> Result<Args, pico_args::Error> {
  let mut pargs = pico_args::Arguments::from_env();

  // Help has a higher priority and should be handled separately.
  if pargs.contains(["-h", "--help"]) {
    print_help();
    std::process::exit(0);
  }

  // Parse out all args
  let args = Args {
    exe: pargs.value_from_str("--exe")?,
  };

  let remaining = pargs.finish();

  // It's up to the caller what to do with the remaining arguments.
  if remaining.len() != 0 {
    eprintln!("Error: unused arguments left: {:?}.", remaining);
    std::process::exit(1);
  }

  Ok(args)
}

pub fn run() -> i32 {
  let args = match parse_args() {
    Ok(v) => v,
    Err(e) => {
      eprintln!("Error: {}.", e);
      return 1;
    }
  };

  match emu86::run(&args.exe) {
    Ok(_) => (),
    Err(err) => {
      eprintln!("Error: {}", err);
      return 1;
    }
  }

  0
}

fn main() {
  std::process::exit(run());
}
