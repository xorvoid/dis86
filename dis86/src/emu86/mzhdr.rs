use crate::binfmt::mz;

pub fn print_hdr(exe: &mz::Exe) {
  // Load everything to stack because rust thinks it's unaligned and complains otherwise...
  let magic    = exe.hdr.magic;
  let cblp     = exe.hdr.cblp;
  let cp       = exe.hdr.cp;
  let crlc     = exe.hdr.crlc;
  let cparhdr  = exe.hdr.cparhdr;
  let minalloc = exe.hdr.minalloc;
  let maxalloc = exe.hdr.maxalloc;
  let ss       = exe.hdr.ss;
  let sp       = exe.hdr.sp;
  let csum     = exe.hdr.csum;
  let ip       = exe.hdr.ip;
  let cs       = exe.hdr.cs;
  let lfarlc   = exe.hdr.lfarlc;
  let ovno     = exe.hdr.ovno;

  let declared_size = cp as u32 * 512;
  let actual_size = exe.rawdata.len() as u32;
  let header_size = cparhdr as u32 * 16;
  let image_offset = header_size;
  let reloc_table_size = crlc * 4;

  println!("MZ Header:");
  println!("  cblp       (Bytes on last page of file)             0x{:04x} ({})", cblp,     cblp);
  println!("  cp         (Pages in file)                          0x{:04x} ({})", cp,       cp);
  println!("  crlc       (Number of relocation entries)           0x{:04x} ({})", crlc,     crlc);
  println!("  cparhdr    (Size of header in paragraphs)           0x{:04x} ({})", cparhdr,  cparhdr);
  println!("  minalloc   (Minimum extra paragraphs needed)        0x{:04x} ({})", minalloc, minalloc);
  println!("  maxalloc   (Maximum extra paragraphs needed)        0x{:04x} ({})", maxalloc, maxalloc);
  println!("  ss         (Initial SS (relative to load segment))  0x{:04x} ({})", ss,       ss);
  println!("  sp         (Initial SP value)                       0x{:04x} ({})", sp,       sp);
  println!("  csum       (Checksum)                               0x{:04x} ({})", csum,     csum);
  println!("  ip         (Initial IP value)                       0x{:04x} ({})", ip,       ip);
  println!("  cs         (Initial CS (relative to load segment))  0x{:04x} ({})", cs,       cs);
  println!("  lfarlc     (File offset of relocation table)        0x{:04x} ({})", lfarlc,   lfarlc);
  println!("  ovno       (Overlay number (0 = main program))      0x{:04x} ({})", ovno,     ovno);
  println!("");
  println!("Derived:");
  println!("  declared size      {}", declared_size);
  println!("  actual size        {}", actual_size);
  println!("  header size        {}", header_size);
  println!("  image offset       0x{:04x}", image_offset);
  println!("  initial CS:IP      {:04x}:{:04x}", cs, ip);
  println!("  initial SS:SP      {:04x}:{:04x}", ss, sp);
  println!("  reloc table size   {}", reloc_table_size);
}
