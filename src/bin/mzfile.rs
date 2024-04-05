use dis86::binfmt::mz;
use dis86::segoff::{Seg, Off, SegOff};
use dis86::binary::Binary;
use dis86::asm;
use dis86::config::{self, Config};
use std::fs;
use std::collections::HashMap;

struct Command {
  name: &'static str,
  func: fn(args: &[String]),
  desc: &'static str
}

const COMMANDS: &'static [Command] = &[
  Command { name: "info",    func: cmd_info,    desc: "Decode the headers and print to stdout" },
  Command { name: "extract", func: cmd_extract, desc: "Extract the main exe region and all overlay regions" },
  Command { name: "map",     func: cmd_map,     desc: "Map addresses to destinations (useful for overlay stubs)" },
  Command { name: "dis",     func: cmd_dis,     desc: "Disassemble entire file (in so far as practical)" },
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
    if addr == stub.stub_addr() {
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

fn cmd_dis(args: &[String]) {
  if args.len() < 3 {
    eprintln!("usage: {} dis <path> [<config>]", args[0]);
    std::process::exit(1);
  }
  let path = &args[2];

  let mut cfg = None;
  if args.len() >= 4 {
    cfg = Some(Config::from_path(&args[3]).unwrap());
  }

  let Ok(data) = std::fs::read(path) else {
    panic!("Failed to read file: {}", path);
  };

  let exe = mz::Exe::decode(&data).unwrap();

  let Some(seginfo) = exe.seginfo else {
    panic!("Binary has no seginfo: needed to do full disassemble");
  };

  let binary = Binary::from_exe(&exe, cfg.as_ref());

  // Process normal segments
  for s in seginfo {
    let sz = s.size();
    if sz == 0 || sz == 0xffff {
      continue;
    }
    let start = SegOff { seg: Seg::Normal(s.seg), off: Off(s.minoff) };
    let end = SegOff { seg: Seg::Normal(s.seg), off: Off(s.maxoff) };
    if s.typ == mz::SegInfoType::CODE {
      disassemble_code(&binary, start, end, cfg.as_ref(), None);
    }
    else if s.typ == mz::SegInfoType::DATA {
      disassemble_data(&binary, start, end, cfg.as_ref());
    }
  }

  // Process overlay segments
  if let Some(ovr) = exe.ovr.as_ref() {
    let mut dest_to_stub_map = HashMap::new();
    for stub in &ovr.stubs {
      dest_to_stub_map.insert(stub.dest_addr(), stub.stub_addr());
    }

    for (i, seg) in ovr.segs.iter().enumerate() {
      let start = SegOff { seg: Seg::Overlay(i as u16), off: Off(0) };
      let end = SegOff { seg: Seg::Overlay(i as u16), off: Off(seg.segment_size) };
      disassemble_code(&binary, start, end, cfg.as_ref(), Some(&dest_to_stub_map));
    }
  }
}

fn cfg_func(cfg: Option<&Config>, addr: SegOff) -> Option<&config::Func> {
  cfg?.func_lookup(addr)
}

fn instr_is_callf(ins: &Option<asm::instr::Instr>) -> bool {
  let Some(ins) = ins.as_ref() else { return false };
  ins.opcode == asm::instr::Opcode::OP_CALLF
}

fn instr_is_calln(ins: &Option<asm::instr::Instr>) -> bool {
  let Some(ins) = ins.as_ref() else { return false };
  ins.opcode == asm::instr::Opcode::OP_CALL
}

fn instr_is_return(ins: &Option<asm::instr::Instr>) -> bool {
  let Some(ins) = ins.as_ref() else { return false };
  ins.opcode == asm::instr::Opcode::OP_IRET ||
    ins.opcode == asm::instr::Opcode::OP_RET ||
    ins.opcode == asm::instr::Opcode::OP_RETF
}

fn disassemble_code(binary: &Binary, start: SegOff, end: SegOff, cfg: Option<&Config>,
                    dest_to_stub_map: Option<&HashMap<SegOff, SegOff>>) {
  println!(";;; ===================== CODE SEGMENT {} =====================", start.seg);
  let mut region = binary.region_iter(start, end);
  loop {
    let (addr, instr, raw) = match asm::decode::decode_one(&mut region) {
      Ok(None) => break,
      Ok(Some((instr, raw))) => (instr.addr, Some(instr), raw),
      Err(_) => {
        // Failed to decode an instruction. We're probably in some inline data region and this is technically
        // unsolvable without proper metadata. As a work-around, we simply emit the current byte as "data" and
        // continue on. Eventually we'll decode another instruction and hopefully we'll eventually re-align.
        let addr = region.addr();
        let raw = region.slice(addr, 1);
        region.advance();
        (addr, None, raw)
      }
    };

    let mut emitted_divider = false;
    let mut emit_divider = || {
      if !emitted_divider {
        println!(";;; =============================================================");
        emitted_divider = true;
      }
    };

    if let Some(dest_to_stub_map) = dest_to_stub_map {
      if let Some(stub_addr) = dest_to_stub_map.get(&addr) {
        emit_divider();
        println!(";;; STUB TARGET FROM {}", stub_addr);
      }
    }

    if let Some(func) = cfg_func(cfg, addr) {
      emit_divider();
      println!(";;; FUNCTION: {}", func.name);
    } else {
      if let Some(instr) = instr.as_ref() {
        if instr.opcode == asm::instr::Opcode::OP_PUSH &&
          instr.operands[0] == asm::instr::Operand::Reg(asm::instr::OperandReg(asm::instr::Reg::BP)) {
            emit_divider();
            println!(";;; MAYBE UNKNOWN FUNCTION");
          }
      }
    }

    print!("{}", &asm::intel_syntax::format(addr, instr.as_ref(), raw, true).unwrap());

    if instr_is_callf(&instr) {
      if let asm::instr::Operand::Far(far) = &instr.as_ref().unwrap().operands[0] {
        let dest_addr = SegOff { seg: Seg::Normal(far.seg), off: Off(far.off) };
        if let Some(func) = binary.lookup_call(addr, dest_addr) {
          print!("  ; {}()", func.name);
        }
      }
    }

    if instr_is_calln(&instr) {
      if let asm::instr::Operand::Rel(rel) = &instr.as_ref().unwrap().operands[0] {
        let dest_addr = instr.as_ref().unwrap().rel_addr(rel);
        if let Some(func) = binary.lookup_call(addr, dest_addr) {
          print!("  ; {}()", func.name);
        }
      }
    }

    println!("");

    if instr_is_return(&instr) {
      println!("");
    }
  }
  println!("");
}

fn disassemble_data(binary: &Binary, start: SegOff, end: SegOff, _cfg: Option<&Config>) {
  println!(";;; =========== DATA SEGMENT {} ===========", start.seg);
  let mut region = binary.region_iter(start, end);
  while region.bytes_remaining() > 0 {
    let n = std::cmp::min(region.bytes_remaining(), 16);
    let addr = region.addr();
    let raw = region.slice(addr, n as u16);
    region.advance_by(n);
    println!("{}", &asm::intel_syntax::format(addr, None, raw, true).unwrap());
  }
  println!("");
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
