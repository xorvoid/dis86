use crate::segoff::SegOff;
use crate::decomp::config::{self, Config};

pub struct Spec<'a> {
  pub func: Option<&'a config::Func>,
  pub name: String,
  pub start: SegOff,
  pub end: SegOff,
}

impl<'a> Spec<'a> {
  pub fn from_config_name(cfg: &'a Config, name: &str) -> Self {
    let Some(func) = cfg.func_lookup_by_name(name) else {
      panic!("Failed to lookup function named: {}", name);
    };
    let Some(end) = func.end else {
      panic!("Function has no 'end' addr defined in config");
    };
    Spec {
      name: name.to_string(),
      func: Some(func),
      start: func.start,
      end,
    }
  }

  pub fn from_start_and_end(start: Option<SegOff>, end: Option<SegOff>) -> Self {
    let Some(start) = start else { panic!("No start address provided") };
    let Some(end) = end else { panic!("No end address provided") };
    let name = format!("func_{:08x}__{:04x}_{:04x}", start.abs(), start.seg, start.off);
    Self { name, func: None, start, end }
  }
}
