use crate::binfmt::mz::*;

impl<'a> Exe<'a> {
  pub fn print_hdr(&self) {
    // Load everything to stack because rust thinks it's unaligned and complains otherwise...
    let magic    = self.hdr.magic;
    let cblp     = self.hdr.cblp;
    let cp       = self.hdr.cp;
    let crlc     = self.hdr.crlc;
    let cparhdr  = self.hdr.cparhdr;
    let minalloc = self.hdr.minalloc;
    let maxalloc = self.hdr.maxalloc;
    let ss       = self.hdr.ss;
    let sp       = self.hdr.sp;
    let csum     = self.hdr.csum;
    let ip       = self.hdr.ip;
    let cs       = self.hdr.cs;
    let lfarlc   = self.hdr.lfarlc;
    let ovno     = self.hdr.ovno;

    let magic_str = std::str::from_utf8(&magic).unwrap_or("??");

    println!("MZ Header:");
    println!("  magic:    0x{:02x}{:02x} (\"{}\")", magic[0], magic[1], magic_str);
    println!("  cblp      0x{:04x} ({})", cblp,     cblp);
    println!("  cp        0x{:04x} ({})", cp,       cp);
    println!("  crlc      0x{:04x} ({})", crlc,     crlc);
    println!("  cparhdr   0x{:04x} ({})", cparhdr,  cparhdr);
    println!("  minalloc  0x{:04x} ({})", minalloc, minalloc);
    println!("  maxalloc  0x{:04x} ({})", maxalloc, maxalloc);
    println!("  ss        0x{:04x} ({})", ss,       ss);
    println!("  sp        0x{:04x} ({})", sp,       sp);
    println!("  csum      0x{:04x} ({})", csum,     csum);
    println!("  ip        0x{:04x} ({})", ip,       ip);
    println!("  cs        0x{:04x} ({})", cs,       cs);
    println!("  lfarlc    0x{:04x} ({})", lfarlc,   lfarlc);
    println!("  ovno      0x{:04x} ({})", ovno,     ovno);
    println!("");
    println!("Exe Region:");
    println!("  start     0x{:08x}", self.exe_start);
    println!("  end       0x{:08x}", self.exe_end);
    println!("");
  }

  pub fn print_relocs(relocs: &[Reloc]) {
    print!("Relocations:");
    for (i, r) in relocs.iter().enumerate() {
      if i%16 == 0 { println!(""); }
      // Load everything to stack because rust thinks it's unaligned and complains otherwise...
      let segment = r.segment;
      let offset  = r.offset;
      print!("  {:04x}:{:04x}", segment, offset);
    }
    println!("");
    println!("");
  }

  pub fn print_fbov(fbov: &FBOV) {
    // Load everything to stack because rust thinks it's unaligned and complains otherwise...
    let magic   = fbov.magic;
	  let ovrsize = fbov.ovrsize;
	  let exeinfo = fbov.exeinfo;
	  let segnum  = fbov.segnum;

    let magic_str = std::str::from_utf8(&magic).unwrap_or("????");

    println!("FBOV Header:");
    println!("  magic:    0x{:02x}{:02x}{:02x}{:02x} (\"{}\")", magic[0], magic[1], magic[2], magic[3], magic_str);
    println!("  ovrsize   0x{:08x} ({})",   ovrsize, ovrsize);
    println!("  exeinfo   0x{:08x} ({})",   exeinfo, exeinfo);
    println!("  segnum    0x{:08x} ({})",   segnum, segnum);
    println!("");
  }

  pub fn print_seginfo(seginfo: &[SegInfo]) {
    println!("Segment Information:");
    println!("  {:<4}  {:<8}  {:<12}  {:<8}  {:<8}  {:<10}",
             "num", "seg", "type", "minoff", "maxoff", "size");

    for (i, s) in seginfo.iter().enumerate() {
      // Load everything to stack because rust thinks it's unaligned and complains otherwise...
      let seg    = s.seg;
      let maxoff = s.maxoff;
      let typ    = s.typ;
      let minoff = s.minoff;

      let typ_str = match typ {
        SegInfoType::DATA => "DATA",
        SegInfoType::CODE => "CODE",
        SegInfoType::STUB => "STUB",
        SegInfoType::OVERLAY => "OVERLAY",
        _ => "UNKNOWN",
      };
      let typ_str = format!("{}({})", typ_str, typ);
      let n = s.size();
      println!(" {:4}   0x{:04x}    {:<12}  0x{:04x}    0x{:04x}    {:5} (0x{:04x})",
               i, seg, typ_str, minoff, maxoff, n, n);
    }
    println!("");
  }


  fn print_overlayinfo(ovr: &OverlayInfo) {
    println!("Overlay File Offset: 0x{:x}", ovr.file_offset);
    println!("");

    println!("Overlay Segments:");
    println!("   num      data_off    data_end    seg_size    _unknown_1    _unknown_2");
    for (i, seg) in ovr.segs.iter().enumerate() {
      let end = seg.data_offset + seg.segment_size as u32;
      println!("   {:3}   0x{:08x}   0x{:08x}   {:9}        0x{:04x}        0x{:04x}",
               i, seg.data_offset, end, seg.segment_size, seg._unknown_1, seg._unknown_2);
    }
    println!("");

    println!("Overlay Stubs:");
    for stub in &ovr.stubs {
      println!("  {} => {}", stub.stub_addr(), stub.dest_addr());
    }
  }

  pub fn print(&self) {
    Self::print_hdr(self);
    Self::print_relocs(&self.relocs);
    if let Some(fbov) = self.fbov {
      Self::print_fbov(fbov);
    }
    if let Some(seginfo) = self.seginfo {
      Self::print_seginfo(seginfo);
    }
    if let Some(ovr) = self.ovr.as_ref() {
      Self::print_overlayinfo(ovr);
    }
  }
}
