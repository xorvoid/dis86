use crate::segoff::{Seg, SegOff};
use crate::region::RegionIter;
use crate::config::{self, Config};
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

pub struct Binary<'a> {
  main: Data,
  overlays: Vec<Data>,
  config: Option<&'a Config>,
  segmap: Option<Vec<u16>>,
}

fn build_segmap(exe: &binfmt::mz::Exe) -> Option<Vec<u16>> {
  let segmap = exe.seginfo?;
  let mut out = vec![];
  for s in segmap {
    out.push(s.seg);
  }
  Some(out)
}

impl<'a> Binary<'a> {
  pub fn from_fmt(fmt: &Fmt, config: Option<&'a Config>) -> Result<Self, String> {
    let path = fmt.path();

    let data = std::fs::read(path).map_err(
      |err| format!("Failed to read file: '{}': {:?}", path, err))?;

    let binary = match fmt {
      Fmt::Raw(_) => {
        Binary::from_raw(&data, config)
      }
      Fmt::Exe(_) => {
        let exe = binfmt::mz::Exe::decode(&data).unwrap();
        Self::from_exe(&exe, config)
      }
    };

    Ok(binary)
  }

  pub fn from_exe(exe: &binfmt::mz::Exe, config: Option<&'a Config>) -> Self {
    let main = Data(exe.exe_data().to_vec());
    let mut overlays = vec![];
    for i in 0..exe.num_overlay_segments() {
      overlays.push(Data(exe.overlay_data(i).to_vec()));
    }
    let segmap = build_segmap(&exe);
    Binary { main, overlays, config, segmap, }
  }

  pub fn from_raw(data: &[u8], config: Option<&'a Config>) -> Self {
    Self { main: Data(data.to_vec()), overlays: vec![], config, segmap: None }
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

  pub fn remap_to_segment(&self, old: u16) -> Seg {
    let Some(segmap) = self.segmap.as_ref() else {
      panic!("Cannot remap segments when binary has no seginfo table");
    };
    assert!(old%8 == 0);
    Seg::Normal(segmap[(old/8) as usize])
  }

  pub fn lookup_call(&self, from: SegOff, to: SegOff) -> Option<&'a config::Func> {
    match &from.seg {
      Seg::Normal(_) => cfg_func(self.config, to),
      Seg::Overlay(_) => {
        // We're calling from an overlay, so we need to remap the dest seg before making the call...
        let Seg::Normal(seg) = to.seg else { return None; /*panic!("Unexpected destination segment as overlay!") */ };
        let remapped_seg = self.remap_to_segment(seg);
        let to_modified = SegOff { seg: remapped_seg, off: to.off };
        cfg_func(self.config, to_modified)
      }
    }
  }
}

fn cfg_func(cfg: Option<&Config>, addr: SegOff) -> Option<&config::Func> {
  cfg?.func_lookup(addr)
}
