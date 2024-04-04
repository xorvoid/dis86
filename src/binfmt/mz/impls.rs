use crate::binfmt::mz::*;
use std::mem::size_of;

unsafe fn try_struct_from_bytes<T: Sized>(data: &[u8]) -> Result<&T, String> {
  let sz = size_of::<T>();
  if data.len() < sz {
    Err(format!("Data is too short for {}: got {}, expected {}", std::any::type_name::<T>(), data.len(), sz))
  } else {
    Ok(unsafe { &*(data.as_ptr() as *const T) })
  }
}

unsafe fn try_slice_from_bytes<T: Sized>(data: &[u8], nelts: usize) -> Result<&[T], String> {
  let len = nelts * size_of::<T>();
  if data.len() < len {
    Err(format!("Data is too short for {}: got {}, expected {}", std::any::type_name::<[T]>(), data.len(), len))
  } else {
    Ok(unsafe { std::slice::from_raw_parts(data.as_ptr() as *const T, nelts) })
  }
}

pub fn decode_exe<'a>(data: &'a [u8]) -> Result<Exe<'a>, String> {
  // Decode the header
  let hdr = decode_hdr(data)?;

  // Compute the EXE Region
  let exe_start = hdr.cparhdr as u32 * 16;
  let mut exe_end = hdr.cp as u32 * 512;
  if hdr.cblp != 0 { exe_end -= 512 - hdr.cblp as u32; }
  if exe_end as usize > data.len() {
    return Err(format!("End of exe region is beyond the end of data"));
  }

  // Determine the relocs array
  let relocs = unsafe { try_slice_from_bytes(&data[hdr.lfarlc as usize..], hdr.crlc as usize) }?;

  // Optional regions
  let data_rem = &data[exe_end as usize..];
  let fbov = decode_fbov(data_rem);
  let mut seginfo = None;
  if let Some(fbov) = fbov {
    // Decode seginfo
    if fbov.segnum < 0 {
      let segnum = fbov.segnum; // unaligned
      return Err(format!("Negative FBOV segnum: {}", segnum));
    }
    seginfo = Some(unsafe { try_slice_from_bytes(&data[fbov.exeinfo as usize..], fbov.segnum as usize) }?);
  }

  Ok(Exe {
    hdr,
    exe_start,
    exe_end,
    relocs,
    fbov,
    seginfo,
  })
}

pub fn decode_hdr<'a>(data: &'a [u8]) -> Result<&'a Header, String> {
  // let hdr_sz = size_of::<Header>();
  // if data.len() < hdr_sz {
  //   return Err(format!("Data is too short for header: got {}, expected {}", data.len(), hdr_sz));
  // }

  // Get the header and perform magic check
  let hdr: &Header = unsafe { try_struct_from_bytes(data) }?;
  let magic_expect = ['M' as u8, 'Z' as u8];
  if hdr.magic != magic_expect {
    return Err(format!("Magic number mismatch: got {:?}, expected {:?}", hdr.magic, magic_expect));
  }

  Ok(hdr)
}

pub fn decode_fbov<'a>(data: &'a [u8]) -> Option<&'a FBOV> {
  // Get the struct and perform magic check
  let fbov: &FBOV = unsafe { try_struct_from_bytes(data) }.ok()?;
  let magic_expect = ['F' as u8, 'B' as u8, 'O' as u8, 'V' as u8];
  if fbov.magic != magic_expect {
    return None;
  }

  // All good
  Some(fbov)
}

impl<'a> Exe<'a> {
  #[cfg(target_endian = "big")]
  pub fn decode(data: &'a [u8]) -> Result<Self, String> {
    panic!("MZ decoding only works on little-endian machines");
  }

  #[cfg(target_endian = "little")]
  pub fn decode(data: &'a [u8]) -> Result<Self, String> {
    decode_exe(data)
  }

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
    println!("  {:<8}  {:<12}  {:<8}  {:<8}  {:<10}",
           "seg", "flags", "minoff", "maxoff", "size");

    for s in seginfo {
      // Load everything to stack because rust thinks it's unaligned and complains otherwise...
      let seg    = s.seg;
      let maxoff = s.maxoff;
      let flags  = s.flags;
      let minoff = s.minoff;

      let n = maxoff.wrapping_sub(minoff);
      let typ = match flags {
        SegInfoFlags::DATA => "DATA",
        SegInfoFlags::CODE => "CODE",
        SegInfoFlags::STUB => "STUB",
        SegInfoFlags::OVERLAY => "OVERLAY",
        _ => "UNKNOWN",
      };
      let flags_str = format!("{}({})", typ, flags);
      println!("  0x{:04x}    {:<12}  0x{:04x}    0x{:04x}    {:5} (0x{:04x})",
               seg, flags_str, minoff, maxoff, n, n);
    }
  }

  // for (u32 i = 0; i < n_seginfo; i++) {
  //   mz_seginfo_t *s = &seginfo[i];

  //   u16 n = (u16)s->maxoff - (u16)s->minoff;

  //   const char *type = "UNKNOWN";
  //   if (0) {}
  //   else if (s->flags == 0) type = "DATA";
  //   else if (s->flags == 1) type = "CODE";
  //   else if (s->flags == 3) type = "STUB";  // OVERLAY ENTRY STUB
  //   else if (s->flags == 4) type = "OVERLAY";

  //   char flags[32];
  //   snprintf(flags, sizeof(flags), "%s(%u)", type, s->flags);

  //   printf("  0x%04x    %-12s  0x%04x    0x%04x    %5u (0x%04x)\n",
  //          s->seg, flags, s->minoff, s->maxoff, n, n);
  // }
  // printf("\n");
  // }

  pub fn print(&self) {
    Self::print_hdr(self);
    Self::print_relocs(&self.relocs);
    if let Some(fbov) = self.fbov {
      Self::print_fbov(fbov);
    }
    if let Some(seginfo) = self.seginfo {
      Self::print_seginfo(seginfo);
    }
  }
}
