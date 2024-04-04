use dis86::binfmt::mz;
use dis86::segoff::SegOff;
use std::fs;

fn find_segment_info<'a>(exe: &mz::Exe<'a>, addr: SegOff) -> &'a mz::SegInfo {
  let mut found = None;
  let Some(seginfo) = exe.seginfo else {
    panic!("Binary has no segment info, cannot map {}", addr);
  };
  for s in seginfo {
    if s.seg != addr.seg { continue; }
    if !(s.minoff <= addr.off && addr.off < s.maxoff) { continue; }
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
    if addr.seg == stub.stub_segment && addr.off == stub.stub_offset {
      return stub;
    }
  }
  panic!("Failed to find overlay stub entry for {}", addr);
}

fn main() {
  let args: Vec<String> = std::env::args().collect();
  if args.len() != 3 {
    eprintln!("usage: {} <exe-path> <seg:off>", args[0]);
    std::process::exit(1);
  }
  let path = &args[1];
  let addr: SegOff = args[2].parse().unwrap();

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
  // println!("addr: {}", addr);
  // println!("s: {:?}", s);
}
