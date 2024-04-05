use crate::segoff::{Seg, Off, SegOff};
use crate::region::RegionIter;
use crate::binfmt;

#[derive(Debug)]
pub enum Fmt {
  Raw(String),
  Exe(String),
}

impl Fmt {
  pub fn path(&self) -> &str {
    match self {
      Fmt::Raw(path) => path,
      Fmt::Exe(path) => path,
    }
  }
}

struct Data(Vec<u8>);

pub struct Binary {
  main: Data,
  overlays: Vec<Data>,
}

impl Binary {
  pub fn from_fmt(fmt: &Fmt) -> Result<Self, String> {
    let path = fmt.path();

    let data = std::fs::read(path).map_err(
      |err| format!("Failed to read file: '{}': {:?}", path, err))?;

    let binary = match fmt {
      Fmt::Raw(_) => {
        Binary::from_data(&data)
      }
      Fmt::Exe(_) => {
        let exe = binfmt::mz::Exe::decode(&data).unwrap();
        let main = Data(exe.exe_data().to_vec());
        let mut overlays = vec![];
        for i in 0..exe.num_overlay_segments() {
          overlays.push(Data(exe.overlay_data(i).to_vec()));
        }
        Binary { main, overlays }
      }
    };

    Ok(binary)
  }

  pub fn from_data(data: &[u8]) -> Self {
    Self { main: Data(data.to_vec()), overlays: vec![] }
  }

  pub fn from_file(path: &str) -> Result<Self, String> {
    let mem = std::fs::read(path).map_err(
      |err| format!("Failed to read file: '{}': {:?}", path, err))?;
    Ok(Self { main: Data(mem), overlays: vec![] })
  }

  pub fn region(&self, start: SegOff, end: SegOff) -> &[u8] {
    assert!(start.seg == end.seg);
    match start.seg {
      Seg::Normal(_) => &self.main.0[start.abs_normal() .. end.abs_normal()],
      Seg::Overlay(seg) => &self.overlays[seg as usize].0[start.off.0 as usize .. end.off.0 as usize],
    }
  }

  pub fn region_iter(&self, start: SegOff, end: SegOff) -> RegionIter<'_> {
    RegionIter::new(self.region(start, end), start)
  }
}
