use crate::binfmt::mz::*;

fn decode_exe<'a>(data: &'a [u8]) -> Result<Exe<'a>, String> {
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
  let relocs = unsafe { util::try_slice_from_bytes(&data[hdr.lfarlc as usize..], hdr.crlc as usize) }?;

  // Optional FBOV
  let data_rem = &data[exe_end as usize..];
  let fbov = decode_fbov(data_rem);

  // Optional seginfo
  let mut seginfo = None;
  if let Some(fbov) = fbov {
    // Decode seginfo
    if fbov.segnum < 0 {
      let segnum = fbov.segnum; // unaligned
      return Err(format!("Negative FBOV segnum: {}", segnum));
    }
    seginfo = Some(unsafe { util::try_slice_from_bytes(&data[fbov.exeinfo as usize..], fbov.segnum as usize) }?);
  }

  // Optional overlay info
  let mut ovr = None;
  if let Some(fbov) = fbov {
    ovr = Some(overlay::decode_overlay_info(data, exe_start, fbov, seginfo.unwrap())?);
  }

  Ok(Exe {
    hdr,
    exe_start,
    exe_end,
    relocs,
    fbov,
    seginfo,
    ovr,
    rawdata: data,
  })
}

fn decode_hdr<'a>(data: &'a [u8]) -> Result<&'a Header, String> {
  // Get the header and perform magic check
  let hdr: &Header = unsafe { util::try_struct_from_bytes(data) }?;
  let magic_expect = ['M' as u8, 'Z' as u8];
  if hdr.magic != magic_expect {
    return Err(format!("Magic number mismatch: got {:?}, expected {:?}", hdr.magic, magic_expect));
  }

  Ok(hdr)
}

fn decode_fbov<'a>(data: &'a [u8]) -> Option<&'a FBOV> {
  // Get the struct and perform magic check
  let fbov: &FBOV = unsafe { util::try_struct_from_bytes(data) }.ok()?;
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
}
