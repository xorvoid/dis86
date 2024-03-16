use crate::segoff::SegOff;
use crate::bsl;
use crate::types::Type;

#[derive(Debug)]
pub struct Func {
  pub name: String,
  pub start: SegOff,
  pub end: Option<SegOff>,
  pub mode: CallMode,
  pub ret: Type,
  pub args: Option<u16>,  // None means "unknown", Some(0) means "no args"
  pub pop_args_after_call: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallMode {
  Near,
  Far,
}

#[derive(Debug)]
pub struct Global {
  pub name: String,
  pub offset: u16,
  pub typ: Type,
}

#[derive(Debug)]
pub struct TextSectionRegion {
  pub name: String,
  pub start: SegOff,
  pub end: SegOff,
  pub typ: Type,
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
  pub text_section: Vec<TextSectionRegion>,
  pub segmaps: Vec<Segmap>,
}

impl Config {
  pub fn func_lookup(&self, addr: SegOff) -> Option<&Func> {
    // TODO: Consider something better than linear search
    for f in &self.funcs {
      if addr == f.start {
        return Some(f)
      }
    }
    None
  }

  pub fn func_lookup_by_name(&self, name: &str) -> Option<&Func> {
    // TODO: Consider something better than linear search
    for f in &self.funcs {
      if name == f.name.as_str() {
        return Some(f)
      }
    }
    None
  }

  pub fn text_region_lookup_by_start_addr(&self, addr: SegOff) -> Option<&TextSectionRegion> {
    // TODO: Consider something better than linear search
    for r in &self.text_section {
      if addr == r.start {
        return Some(r)
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
      text_section: vec![],
      segmaps: vec![],
    };

    let dat = std::fs::read_to_string(path)
      .map_err(|err| format!("Failed to read file with: {}'", err))?;

    let root = bsl::parse(&dat)
      .ok_or_else(|| format!("Failed to parse config"))?;

    cfg.parse_functions(&root)?;
    cfg.parse_globals(&root)?;
    cfg.parse_text_section(&root)?;
    cfg.parse_segmap(&root)?;

    Ok(cfg)
  }

  fn parse_functions(&mut self, root: &bsl::Root) -> Result<(), String> {
    let func = root.get_node("dis86.functions")
      .ok_or_else(|| format!("Failed to get the functions node"))?;

    for (key, val) in func.iter() {
      let f = val.as_node()
        .ok_or_else(|| format!("Expected function properties"))?;

      let start_str = f.get_str("start")
        .ok_or_else(|| format!("No function 'start' property for '{}'", key))?;
      let end_str = f.get_str("end")
        .ok_or_else(|| format!("No function 'end' property for '{}'", key))?;
      let mode_str = f.get_str("mode")
        .ok_or_else(|| format!("No function 'mode' property for '{}'", key))?;
      let ret_str = f.get_str("ret")
        .ok_or_else(|| format!("No function 'ret' property for '{}'", key))?;
      let args_str = f.get_str("args")
        .ok_or_else(|| format!("No function 'args' property for '{}'", key))?;

      let pop_args_after_call = !f.get_str("dont_pop_args").is_some();

      let start: SegOff = start_str.parse()
        .map_err(|_| format!("Expected segoff for '{}.start', got '{}'", key, start_str))?;
      let end: Option<SegOff> = if end_str.len() == 0 { None } else {
        Some(end_str.parse()
             .map_err(|_| format!("Expected segoff for '{}.end', got '{}'", key, end_str))?)
      };
      let mode = match mode_str {
        "near" => CallMode::Near,
        "far" => CallMode::Far,
        _ => panic!("Unsupported mode '{}'", mode_str)
      };
      let args: i16 = args_str.parse()
        .map_err(|_| format!("Expected u16 for '{}.args', got '{}'", key, args_str))?;
      let ret: Type = ret_str.parse()
        .map_err(|err| format!("Expected type for '{}.ret', got '{}' | {}", key, ret_str, err))?;

      self.funcs.push(Func {
        name: key.to_string(),
        start,
        end,
        mode,
        ret,
        args: if args >= 0 { Some(args as u16) } else { None },
        pop_args_after_call,
      });
    }

    Ok(())
  }

  fn parse_globals(&mut self, root: &bsl::Root) -> Result<(), String> {
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
      let typ: Type = match type_str.parse() {
        Ok(typ) => typ,
        Err(err) => {
          // FIXME: Make this a hard error.. currently the configs have undefined struct names.. need to support that first :-(
          eprintln!("WRN: Expected type for '{}.type', got '{}' | {}", key, type_str, err);
          Type::Unknown
        }
      };

      self.globals.push(Global {
        name: key.to_string(),
        offset: off,
        typ,
      });
    }
    Ok(())
  }

  fn parse_text_section(&mut self, root: &bsl::Root) -> Result<(), String> {
    let func = root.get_node("dis86.text_section")
      .ok_or_else(|| format!("Failed to get the text_section node"))?;

    for (key, val) in func.iter() {
      let f = val.as_node()
        .ok_or_else(|| format!("Expected text_section properties"))?;

      let start_str = f.get_str("start")
        .ok_or_else(|| format!("No text_section 'start' property for '{}'", key))?;
      let end_str = f.get_str("end")
        .ok_or_else(|| format!("No text_section 'end' property for '{}'", key))?;
      let type_str = f.get_str("type")
        .ok_or_else(|| format!("No text_section 'type' property for '{}'", key))?;

      let start: SegOff = start_str.parse()
        .map_err(|_| format!("Expected segoff for '{}.start', got '{}'", key, start_str))?;
      let end: SegOff = end_str.parse()
        .map_err(|_| format!("Expected segoff for '{}.start', got '{}'", key, end_str))?;
      let typ: Type = type_str.parse()
        .map_err(|err| format!("Expected type for '{}.type', got '{}' | {}", key, type_str, err))?;

      self.text_section.push(TextSectionRegion {
        name: key.to_string(),
        start,
        end,
        typ,
      });
    }

    Ok(())
  }

  fn parse_segmap(&mut self, root: &bsl::Root) -> Result<(), String> {
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

      self.segmaps.push(Segmap {
        name: key.to_string(),
        from,
        to,
      });
    }
    Ok(())
  }
}
