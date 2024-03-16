use std::str::FromStr;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
  Void, U8, U16, U32, I8, I16, I32,
  Array(Box<Type>, ArraySize),
  Ptr(Box<Type>),
  Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArraySize {
  Known(usize),
  Unknown,
}

impl Type {
  pub fn ptr(base: Type) -> Type {
    Type::Ptr(Box::new(base))
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
      Type::Unknown => None,
    }
  }
}

impl FromStr for Type {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if false { unreachable!() }
    else if s == "void" { Ok(Type::Void) }
    else if s == "u8"   { Ok(Type::U8) }
    else if s == "u16"  { Ok(Type::U16) }
    else if s == "u32"  { Ok(Type::U32) }
    else if s == "i8"   { Ok(Type::I8) }
    else if s == "i16"  { Ok(Type::I16) }
    else if s == "i32"  { Ok(Type::I32) }
    else {
      parse_array_type(s).ok_or_else(
        || format!("Failed to parse type: '{}'", s))
    }
  }
}

fn parse_array_type(s: &str) -> Option<Type> {
  let array_start = s.find('[')?;
  let array_end = s.find(']')?;
  if array_end != s.len() - 1 { return None; }

  let base_str = &s[..array_start];
  let size_str = &s[array_start+1..array_end];

  let base: Type = base_str.parse().ok()?;
  let size = if size_str.len() > 0 {
    let n: usize = size_str.parse().ok()?;
    ArraySize::Known(n)
  } else {
    ArraySize::Unknown
  };

  Some(Type::Array(Box::new(base), size))
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
      Type::Unknown => write!(f, "?unknown_type?"),
    }
  }
}
