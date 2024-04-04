
#[repr(C, packed)]
#[derive(Debug, Clone)]
pub struct Header {
  pub magic:    [u8; 2], /* "MZ" */
  pub cblp:     u16,
  pub cp:       u16,
  pub crlc:     u16,
  pub cparhdr:  u16,
  pub minalloc: u16,
  pub maxalloc: u16,
  pub ss:       i16,
  pub sp:       u16,
  pub csum:     u16,
  pub ip:       u16,
  pub cs:       i16,
  pub lfarlc:   u16,
  pub ovno:     u16,
}

#[repr(C, packed)]
#[derive(Debug, Clone)]
pub struct Reloc {
  pub offset: u16,
  pub segment: u16,
}

// Borland C/C++ FBOV Header for Overlays (ZROOM?)
#[repr(C, packed)]
#[derive(Debug, Clone)]
pub struct FBOV {
  pub magic: [u8; 4], /* "FBOV" */
	pub ovrsize: u32,
	pub exeinfo: u32,  /* points to mz_seginfo array in binary */
	pub segnum: i32,   /* number of entries in the mz_seginfo array */
}

#[allow(non_snake_case)]
pub mod SegInfoFlags {
  pub const DATA: u16 = 0;
  pub const CODE: u16 = 1;
  pub const STUB: u16 = 3;
  pub const OVERLAY: u16 = 4;
}

#[repr(C, packed)]
#[derive(Debug, Clone)]
pub struct SegInfo
{
	pub seg: u16,
	pub maxoff: u16,
	pub flags: u16, // SegInfoFlags::*
	pub minoff: u16,
}

#[derive(Debug, Clone)]
pub struct Exe<'a> {
  pub hdr: &'a Header,
  pub exe_start: u32,
  pub exe_end: u32,
  pub relocs: &'a [Reloc],
  pub fbov: Option<&'a FBOV>,
  pub seginfo: Option<&'a [SegInfo]>,
}
