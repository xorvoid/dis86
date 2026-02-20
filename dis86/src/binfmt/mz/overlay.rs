use crate::binfmt::mz::*;

const CODE_OVERLAY_SEG_INTERRUPT_CODE:  [u8; 4]  = [0xcd, 0x3f, 0x00, 0x00];
const CODE_OVERLAY_SEG_ZEROS:           [u8; 18] = [0; 18];
const CODE_OVERLAY_STUB_INTERRUPT_CODE: [u8; 2]  = [0xcd, 0x3f];
const CODE_OVERLAY_STUB_ZEROS:          [u8; 1]  = [0; 1];

#[repr(C, packed)]
#[derive(Debug)]
pub struct CodeOverlaySeg {
  interrupt_code: [u8; 4], // should be: cd 3f 00 00
  data_offset: u32,
  seg_size: u16,
  _unknown_1: u16,
  _unknown_2: u16,
  _zeros: [u8; 18],
  /* stubs follow: [CodeOverlaySeg] */
}
sa::const_assert!(std::mem::size_of::<CodeOverlaySeg>() == 32);

#[repr(C, packed)]
#[derive(Debug)]
pub struct CodeOverlayStub {
  interrupt_code: [u8; 2], // should be: cd 3f
  call_offset: u16,
  _zeros: [u8; 1],
}
sa::const_assert!(std::mem::size_of::<CodeOverlayStub>() == 5);

pub(super) fn decode_overlay_info(data: &[u8], exe_start: u32, fbov: &FBOV, seginfo: &[SegInfo]) -> Result<OverlayInfo, String> {
  let mut out_segs = vec![];
  let mut out_stubs = vec![];

  let exe_data = &data[exe_start as usize..];
  // let exe_data = exe.exe_data();
  // let Some(seginfo) = exe.seginfo else {
  //   return Ok(None);
  // };

  let mut next_seg = 0;
  for s in seginfo {
    // iterate all stubs
    if s.typ != SegInfoType::STUB { continue }

    // sanity check: might not be required but it's generally true how with
    // how this compiler liked to layout things
    assert!(next_seg == 0 || s.seg == next_seg);
    let n_segs = (s.maxoff + 15) >> 4;
    next_seg = s.seg + n_segs;

    // unpack the actual stub code
    let dat = &exe_data[16 * s.seg as usize..];
    let sz = s.maxoff as usize;
    assert!(sz >= 32); // each hdr section is 32-bytes
    assert!((sz-32)%5 == 0); // each launcher entry is 5 bytes
    let num_entries = (sz-32)/5;

    // get the seg struct
    let seg: &CodeOverlaySeg = unsafe { util::try_struct_from_bytes(dat) }.unwrap();
    if seg.interrupt_code != CODE_OVERLAY_SEG_INTERRUPT_CODE {
      return Err(format!("Invalid seg interrupt code, got {:?} expected {:?}", seg.interrupt_code, CODE_OVERLAY_SEG_INTERRUPT_CODE));
    }
    if seg._zeros != CODE_OVERLAY_SEG_ZEROS {
      return Err(format!("Zeros in seg aren't zero, got {:?} expected {:?}", seg._zeros, CODE_OVERLAY_SEG_ZEROS));
    }

    // create a user struct
    out_segs.push(OverlaySeg {
      stub_segment: s.seg,
      segment_size: seg.seg_size,
      data_offset:  seg.data_offset,
      _unknown_1: seg._unknown_1,
      _unknown_2: seg._unknown_2,
    });

    let seg_num: u16 = (out_segs.len() - 1).try_into().unwrap();

    // process each stub
    let stubs: &[CodeOverlayStub] = unsafe { util::try_slice_from_bytes(&dat[32..], num_entries as usize) }.unwrap();
    for (i, stub) in stubs.iter().enumerate() {
      if stub.interrupt_code != CODE_OVERLAY_STUB_INTERRUPT_CODE {
        return Err(format!("Invalid stub interrupt code, got {:?} expected {:?}", stub.interrupt_code, CODE_OVERLAY_STUB_INTERRUPT_CODE));
      }
      if stub._zeros != CODE_OVERLAY_STUB_ZEROS {
        return Err(format!("Zeros in stub aren't zero, got {:?} expected {:?}", stub._zeros, CODE_OVERLAY_STUB_ZEROS));
      }
      if stub.call_offset >= seg.seg_size {
        let call_offset = stub.call_offset; // unaligned
        let seg_size = seg.seg_size; // unaligned
        return Err(format!("Stub call offset exceeds the segment size, offset {} segsize: {}", call_offset, seg_size));
      }

      out_stubs.push(OverlayStub {
        overlay_seg_num: seg_num,
        stub_segment: s.seg,
        stub_offset: (32 + 5*i).try_into().unwrap(),
        dest_offset: stub.call_offset,
      });
    }
  }

  // Overlay data starts immediately after the FBOV header
  let file_offset =
    fbov as *const _ as usize
    + std::mem::size_of::<FBOV>()
    - data.as_ptr() as usize;

  Ok(OverlayInfo {
    file_offset: file_offset.try_into().unwrap(),
    segs: out_segs,
    stubs: out_stubs
  })
}
