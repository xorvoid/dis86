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
    rawdata: data,
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

  pub fn overlay_info(&self) -> Result<Option<OverlayInfo>, String> {
    OK(None)
  }
}
