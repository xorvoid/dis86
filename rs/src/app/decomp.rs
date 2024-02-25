use pico_args;
use crate::segoff::SegOff;
use crate::decode::Decoder;
use crate::decomp::ir::{build, opt, sym};
use crate::decomp::config::Config;

fn print_help(appname: &str) {
  println!("usage: {} dis OPTIONS", appname);
  println!("");
  println!("OPTIONS:");
  println!("  --binary       path to binary on the filesystem (required)");
  println!("  --start-addr   start seg:off address (required)");
  println!("  --end-addr     end seg:off address (required)");
}

#[derive(Debug)]
struct Args {
  binary: String,
  config: String,
  start_addr: SegOff,
  end_addr: SegOff,
}

fn parse_args(appname: &str) -> Result<Args, pico_args::Error> {
  let mut pargs = pico_args::Arguments::from_env();

  // Help has a higher priority and should be handled separately.
  if pargs.contains(["-h", "--help"]) {
    print_help(appname);
    std::process::exit(0);
  }

  // Parse out all args
  let args = Args {
    config: pargs.value_from_str("--config")?,
    binary: pargs.value_from_str("--binary")?,
    start_addr: pargs.value_from_str("--start-addr")?,
    end_addr: pargs.value_from_str("--end-addr")?,
  };

  // It's up to the caller what to do with the remaining arguments.
  let remaining = pargs.finish();
  if remaining != &["decomp"] {
    eprintln!("Error: unused arguments left: {:?}.", remaining);
    std::process::exit(1);
  }

  Ok(args)
}

pub fn run(appname: &str) {
  let args = match parse_args(appname) {
    Ok(v) => v,
    Err(e) => {
      eprintln!("Error: {}.", e);
      std::process::exit(1);
    }
  };

  let cfg = Config::from_path(&args.config).unwrap();

  let start_idx = args.start_addr.abs();
  let end_idx = args.end_addr.abs();

  let dat = match std::fs::read(&args.binary) {
    Ok(dat) => dat,
    Err(err) => panic!("Failed to read file: '{}': {:?}", args.binary, err),
  };

  let decoder = Decoder::new(&dat[start_idx..end_idx], start_idx);
  let mut instr_list = vec![];
  for (instr, _raw) in decoder {
    instr_list.push(instr);
  }

  let mut ir = build::from_instrs(&instr_list, &cfg);
  opt::optimize(&mut ir);
  // sym::symbolize(&mut ir, &cfg);
  // opt::forward_store_to_load(&mut ir);
  // opt::optimize(&mut ir);

  println!("{}", ir);
  //println!("{:#?}", ir.symbols);
}
