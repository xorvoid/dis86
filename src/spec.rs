use crate::segoff::SegOff;
use crate::config::{self, Config, Func};

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
    Self::from_func(func)
  }

  pub fn from_func(func: &'a Func) -> Self {
    let start = func.start;
    let Some(end) = func.end else {
      panic!("Function has no 'end' addr defined in config");
    };
    Spec {
      name: func.name.to_string(),
      func: Some(func),
      start,
      end,
    }
  }

  pub fn from_start_and_end(start: Option<SegOff>, end: Option<SegOff>) -> Self {
    let Some(start) = start else { panic!("No start address provided") };
    let Some(end) = end else { panic!("No end address provided") };
    let name = format!("func_{}_{}", start.seg, start.off);
    Self { name, func: None, start, end }
  }
}

pub fn specs_from_codeseg_name<'a>(cfg: &'a Config, name: &str) -> Vec<Spec<'a>> {
  let Some(codeseg) = cfg.code_seg_lookup_by_name(name) else {
    panic!("Failed to find code segment with name '{}'", name);
  };

  let mut ret = vec![];
  for func in cfg.func_lookup_by_seg(codeseg.seg) {
    ret.push(Spec::from_func(func));
  }
  ret
}
