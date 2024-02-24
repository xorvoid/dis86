use crate::segoff::SegOff;
use crate::bsl;

#[derive(Debug)]
pub struct Func {
  pub name: String,
  pub addr: SegOff,
  pub ret:  String,
  pub args: Option<u16>,  // None means "unknown", Some(0) means "no args"
  pub pop_args_after_call: bool,
}

#[derive(Debug)]
pub struct Global {
  pub name: String,
  pub offset: u16,
  pub typ: String,
}

#[derive(Debug)]
pub struct Segmap {
  pub name: String,
  pub from: u16,
  pub to: u16,
}

#[derive(Debug, Default)]
pub struct Config {
  pub funcs: Vec<Func>,
  pub globals: Vec<Global>,
  pub segmaps: Vec<Segmap>,
}

impl Config {
  pub fn func_lookup(&self, addr: SegOff) -> Option<&Func> {
    // TODO: Consider something better than linear search
    for f in &self.funcs {
      if addr == f.addr {
        return Some(f)
      }
    }
    None
  }
}

impl Config {
  pub fn from_path(path: &str) -> Result<Config, String> {
    let mut cfg = Config {
      funcs: vec![],
      globals: vec![],
      segmaps: vec![],
    };

    let dat = std::fs::read_to_string(path)
      .map_err(|err| format!("Failed to read file with: {}'", err))?;

    let root = bsl::parse(&dat)
      .ok_or_else(|| format!("Failed to parse config"))?;

    let func = root.get_node("dis86.functions")
      .ok_or_else(|| format!("Failed to get the functions node"))?;

    for (key, val) in func.iter() {
      let f = val.as_node()
        .ok_or_else(|| format!("Expected function properties"))?;

      let addr_str = f.get_str("addr")
        .ok_or_else(|| format!("No function 'addr' property for '{}'", key))?;
      let ret_str = f.get_str("ret")
        .ok_or_else(|| format!("No function 'ret' property for '{}'", key))?;
      let args_str = f.get_str("args")
        .ok_or_else(|| format!("No function 'args' property for '{}'", key))?;

      let pop_args_after_call = !f.get_str("dont_pop_args").is_some();

      let addr: SegOff = addr_str.parse()
        .map_err(|_| format!("Expected segoff for '{}.addr', got '{}'", key, addr_str))?;
      let args: i16 = args_str.parse()
        .map_err(|_| format!("Expected u16 for '{}.args', got '{}'", key, args_str))?;

      cfg.funcs.push(Func {
        name: key.to_string(),
        addr,
        ret: ret_str.to_string(),
        args: if args >= 0 { Some(args as u16) } else { None },
        pop_args_after_call,
      });
    }

    let glob = root.get_node("dis86.globals")
      .ok_or_else(|| format!("Failed to get the globals node"))?;

    for (key, val) in glob.iter() {
      let g = val.as_node()
        .ok_or_else(|| format!("Expected global properties"))?;

      let off_str = g.get_str("off")
        .ok_or_else(|| format!("No global 'off' property for '{}'", key))?;
      let type_str = g.get_str("type")
        .ok_or_else(|| format!("No global 'type' property for '{}'", key))?;

      let off = crate::util::parse::hex_u16(off_str)
        .map_err(|_| format!("Expected u16 hex for '{}.off', got '{}'", key, off_str))?;

      cfg.globals.push(Global {
        name: key.to_string(),
        offset: off,
        typ: type_str.to_string(),
      });
    }

    let segmap = root.get_node("dis86.segmap")
      .ok_or_else(|| format!("Failed to get the segmap node"))?;

    for (key, val) in segmap.iter() {
      let g = val.as_node()
        .ok_or_else(|| format!("Expected global properties"))?;

      let from_str = g.get_str("from")
        .ok_or_else(|| format!("No segmap 'from' property for '{}'", key))?;
      let to_str = g.get_str("to")
        .ok_or_else(|| format!("No segmap 'to' property for '{}'", key))?;

      let from = crate::util::parse::hex_u16(from_str)
        .map_err(|_| format!("Expected u16 hex for '{}.from', got '{}'", key, from_str))?;
      let to = crate::util::parse::hex_u16(to_str)
        .map_err(|_| format!("Expected u16 hex for '{}.to', got '{}'", key, to_str))?;

      cfg.segmaps.push(Segmap {
        name: key.to_string(),
        from,
        to,
      });
    }

    Ok(cfg)
  }
}
