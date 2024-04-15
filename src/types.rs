use crate::config;
use std::fmt;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StructRef {
  idx: usize,
  size: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
  Void, U8, U16, U32, I8, I16, I32,
  Array(Box<Type>, ArraySize),
  Ptr(Box<Type>),
  Struct(StructRef),
  Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ArraySize {
  Known(usize),
  Unknown,
}

impl Type {
  pub fn ptr(base: Type) -> Type {
    Type::Ptr(Box::new(base))
  }

  pub fn is_primitive(&self) -> bool {
    match self {
      Type::Void => true,
      Type::U8  => true,
      Type::U16 => true,
      Type::U32 => true,
      Type::I8  => true,
      Type::I16 => true,
      Type::I32 => true,
      Type::Unknown => true,
      _ => false,
    }
  }

  pub fn size_in_bytes(&self) -> Option<usize> {
    match self {
      Type::Void => None,
      Type::U8 => Some(1),
      Type::U16 => Some(2),
      Type::U32 => Some(4),
      Type::I8 => Some(1),
      Type::I16 => Some(2),
      Type::I32 => Some(4),
      Type::Array(typ, sz) => {
        let elt_sz = typ.size_in_bytes()?;
        let count = match sz {
          ArraySize::Known(n) => Some(n),
          ArraySize::Unknown => None,
        }?;
        Some(elt_sz * count)
      }
      Type::Ptr(_) => None, // Not sure what to do here.. on 8086 this a (seg:off) pair, but on a modern machine (like in hydra) it'll be 8 bytes... Hmmm
      Type::Struct(r) => Some(r.size as usize),
      Type::Unknown => None,
    }
  }
}

impl fmt::Display for Type {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Type::Void => write!(f, "void"),
      Type::U8   => write!(f, "u8"),
      Type::U16  => write!(f, "u16"),
      Type::U32  => write!(f, "u32"),
      Type::I8   => write!(f, "i8"),
      Type::I16  => write!(f, "i16"),
      Type::I32  => write!(f, "i32"),
      Type::Array(typ, sz)  => {
        write!(f, "{}[", typ)?;
        if let ArraySize::Known(n) = sz {
          write!(f, "{}", n)?;
        }
        write!(f, "]")
      }
      Type::Ptr(base)  => write!(f, "{}*", base),
      Type::Struct(r)  => write!(f, "struct_id_{}", r.idx),
      Type::Unknown => write!(f, "?unknown_type?"),
    }
  }
}

#[derive(Debug)]
pub struct Builder {
  structs: Vec<config::Struct>,
  basetypes: HashMap<String, Type>,
}

impl Builder {
  pub fn new() -> Self {
    let mut basetypes = HashMap::new();
    basetypes.insert("void".to_string(), Type::Void);
    basetypes.insert("u8".to_string(),   Type::U8);
    basetypes.insert("u16".to_string(),  Type::U16);
    basetypes.insert("u32".to_string(),  Type::U32);
    basetypes.insert("i8".to_string(),   Type::I8);
    basetypes.insert("i16".to_string(),  Type::I16);
    basetypes.insert("i32".to_string(),  Type::I32);

    Self { structs: vec![], basetypes }
  }

  pub fn append_struct(&mut self, s: &config::Struct) {
    let r = StructRef { idx: self.structs.len(), size: s.size };
    self.structs.push(s.clone());
    self.basetypes.insert(s.name.to_string(), Type::Struct(r));
  }

  pub fn lookup_struct(&self, r: StructRef) -> Option<&config::Struct> {
    self.structs.get(r.idx)
  }

  fn parse_array_type(&self, s: &str) -> Result<Type, String> {
    let array_start = s.find('[')
      .ok_or_else(|| format!("No opening an array bracket"))?;
    let array_end = s.find(']')
      .ok_or_else(|| format!("No closing an array bracket"))?;
    if array_end != s.len() - 1 {
      return Err(format!("Array closing bracket isn't at the end of the type"));
    }

    let base_str = &s[..array_start];
    let size_str = &s[array_start+1..array_end];

    let base = self.parse_type(base_str)?;
    let size = if size_str.len() > 0 {
      let n: usize = size_str.parse()
        .map_err(|_| format!("Cannot parse array size: {}", size_str))?;
      ArraySize::Known(n)
    } else {
      ArraySize::Unknown
    };

    Ok(Type::Array(Box::new(base), size))
  }

  pub fn parse_type(&self, s: &str) -> Result<Type, String> {
    if let Some(typ) = self.basetypes.get(s) {
      return Ok(typ.clone())
    }
    self.parse_array_type(s)
      .map_err(|_| format!("Failed to parse type: '{}'", s))
  }
}
