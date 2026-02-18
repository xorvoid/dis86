use crate::segoff::{Seg, Off, SegOff};
use crate::binfmt::mz;

#[derive(Debug, Clone)]
pub struct Region {
  pub seg: Seg,
  pub skip_off: u32, // The segment data might not start at 0
  pub size: u32,
}

#[derive(Debug)]
pub struct CodeSegment {
  pub primary: Region,
  pub stub: Option<Region>,
}

impl CodeSegment {
  pub fn start(&self) -> SegOff {
    SegOff { seg: self.primary.seg, off: Off(self.primary.skip_off as u16) }
  }
  pub fn end(&self) -> SegOff {
    let end_off: u16 = (self.primary.skip_off + self.primary.size).try_into().unwrap();
    SegOff { seg: self.primary.seg, off: Off(end_off) }
  }
}

// Should basically match those that were manually found in annotations.py
pub fn find_code_segments(exe_path: &str) -> Vec<CodeSegment> {
  let Ok(data) = std::fs::read(exe_path) else {
    panic!("Failed to read file: {}", exe_path);
  };
  let exe = mz::Exe::decode(&data).unwrap();
  let seginfo = exe.seginfo.unwrap(); // FIXME
  let ovr = exe.ovr.as_ref().unwrap(); // FIXME

  // Collect ordinary code segments and stub segments
  let mut code_segments = vec![];
  let mut stub_segments = vec![];
  for s in seginfo {
    let region = Region {
      seg: Seg::Normal(s.seg),
      skip_off: s.minoff as u32,
      size: s.size() as u32,
    };
    if s.typ == mz::SegInfoType::CODE && s.size() != 0 {
      code_segments.push(CodeSegment { primary: region, stub: None, });
    }
    else if s.typ == mz::SegInfoType::STUB {
      stub_segments.push(region);
    }
  }

  // Iterate all overlay segments and match them up with the stubs
  for (i, seg) in ovr.segs.iter().enumerate() {
    let end = seg.data_offset + seg.segment_size as u32;
    let region = Region {
      seg: Seg::Overlay(i as u16),
      skip_off: 0,
      size: seg.segment_size as u32,
    };
    let stub = stub_segments[i].clone();
    assert!(stub.skip_off == 0);
    code_segments.push(CodeSegment { primary: region, stub: Some(stub) });
  }

  code_segments
}

pub fn dump_code_segments(code_segments: &[CodeSegment]) {
  for (i, s) in code_segments.iter().enumerate() {
    let seg_str = format!("{},", s.primary.seg);
    let mut ex = "".to_string();
    if let Some(stub) = &s.stub {
      ex = format!("    entry-seg: {},   entry-seg-size: {}",
                   stub.seg, stub.size);
    }
    println!("{:3} | seg: {:<15} skip_off: 0x{:04x},   size: {:>6}{}",
             i, seg_str, s.primary.skip_off, s.primary.size, ex);
  }
}
