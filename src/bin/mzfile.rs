use dis86::binfmt::mz;
use dis86::segoff::SegOff;
use std::fs;

struct Command {
  name: &'static str,
  func: fn(args: &[String]),
  desc: &'static str
}

const COMMANDS: &'static [Command] = &[
  Command { name: "info",    func: cmd_info,    desc: "Decode the headers and print to stdout" },
  Command { name: "extract", func: cmd_extract, desc: "Extract the main exe region and all overlay regions" },
  Command { name: "map",     func: cmd_map,     desc: "Map addresses to destinations (useful for overlay stubs)" },
];

fn cmd_info(args: &[String]) {
  if args.len() != 3 {
    eprintln!("usage: {} info <path>", args[0]);
    std::process::exit(1);
  }
  let path = &args[2];

  let Ok(data) = std::fs::read(path) else {
    panic!("Failed to read file: {}", path);
  };

  let exe = mz::Exe::decode(&data).unwrap();
  //println!("{:#?}", exe);
  exe.print();
}

fn cmd_extract(args: &[String]) {
  if args.len() != 4 {
    eprintln!("usage: {} <path> <outdir>", args[0]);
    std::process::exit(1);
  }
  let path = &args[2];
  let outdir = &args[3];

  // Read and decode the exe
  let Ok(data) = fs::read(path) else {
    panic!("Failed to read file: {}", path);
  };
  let exe = mz::Exe::decode(&data).unwrap();

  // Prepare outdir
  fs::create_dir_all(outdir).unwrap();

  // Write exe data
  fs::write(&format!("{}/exe.bin", outdir), exe.exe_data()).unwrap();

  // Write overlay data (if it exists)
  if let Some(ovr) = exe.ovr.as_ref() {
    for i in 0..ovr.segs.len() {
      let dat = exe.overlay_data(i);
      fs::write(&format!("{}/overlay_{:04}.bin", outdir, i), dat).unwrap();
    }
  }
}

fn find_segment_info<'a>(exe: &mz::Exe<'a>, addr: SegOff) -> &'a mz::SegInfo {
  let mut found = None;
  let Some(seginfo) = exe.seginfo else {
    panic!("Binary has no segment info, cannot map {}", addr);
  };
  for s in seginfo {
    if s.seg != addr.seg.unwrap_normal() { continue; }
    if !(s.minoff <= addr.off.0 && addr.off.0 < s.maxoff) { continue; }
    if found.is_some() { panic!("Found multiple matching segments"); }
    found = Some(s);
  }
  let Some(ret) = found else {
    panic!("Failed to find a segment for {}", addr);
  };
  ret
}

fn find_stub_info<'a>(exe: &'a mz::Exe<'_>, addr: SegOff) -> &'a mz::OverlayStub {
  let Some(ovr) = exe.ovr.as_ref() else {
    panic!("Binary has no overlay info, cannot map {}", addr);
  };
  for stub in &ovr.stubs {
    if addr.seg.unwrap_normal() == stub.stub_segment && addr.off.0 == stub.stub_offset {
      return stub;
    }
  }
  panic!("Failed to find overlay stub entry for {}", addr);
}

fn cmd_map(args: &[String]) {
  if args.len() != 4 {
    eprintln!("usage: {} <exe-path> <seg:off>", args[0]);
    std::process::exit(1);
  }
  let path = &args[2];
  let addr: SegOff = args[3].parse().unwrap();

  // Read and decode the exe
  let Ok(data) = fs::read(path) else {
    panic!("Failed to read file: {}", path);
  };
  let exe = mz::Exe::decode(&data).unwrap();

  // Map to a SegInfo
  let s = find_segment_info(&exe, addr);

  match s.typ {
    mz::SegInfoType::CODE | mz::SegInfoType::DATA => {
      // Ordinary stuff, nothing to do
      println!("{}", addr);
    }
    mz::SegInfoType::STUB => {
      // Overlay stub
      let stub = find_stub_info(&exe, addr);
      println!("overlay_{:04x}:{:04x}", stub.overlay_seg_num, stub.dest_offset);
    }
    mz::SegInfoType::OVERLAY => {
      println!("OVERLAY: UNIMPL");
    }
    _ => {
      println!("UNKNOWN");
    }
  }
}

fn main() {
  let args: Vec<String> = std::env::args().collect();
  if args.len() < 2 {
    eprintln!("usage: {} [CMD]", args[0]);
    eprintln!("");
    eprintln!("Commands:");
    for c in COMMANDS {
      eprintln!("  {:<10} {}", c.name, c.desc);
    }
    std::process::exit(1);
  }

  let cmd = &args[1];
  for c in COMMANDS {
    if cmd == c.name {
      (c.func)(&args);
      std::process::exit(0);
    }
  }

  panic!("Unknown command: {}", cmd);
}
