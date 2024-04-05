use std::str::FromStr;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Seg {
  Normal(u16),
  Overlay(u16),
}

impl Seg {
  pub fn unwrap_normal(self) -> u16 {
    let Seg::Normal(seg) = self else { panic!("Expected Seg::Normal") };
    seg
  }

  pub fn unwrap_overlay(self) -> u16 {
    let Seg::Overlay(seg) = self else { panic!("Expected Seg::Overlay") };
    seg
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Off(pub u16);

// FIXME: SegOff probably shouldn't be PartialOrd/Ord
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SegOff {
  pub seg: Seg,
  pub off: Off,
}

impl SegOff {
  pub fn abs_normal(&self) -> usize {
    (self.seg.unwrap_normal() as usize) * 16 + (self.off.0 as usize)
  }

  pub fn is_overlay_addr(&self) -> bool {
    matches!(self.seg, Seg::Overlay(_))
  }

  pub fn add_offset(&self, off: u16) -> SegOff {
    SegOff { seg: self.seg, off: Off(self.off.0.wrapping_add(off)) }
  }

  pub fn offset_to(&self, other: SegOff) -> u16 {
    if self.seg != other.seg { panic!("Cannot take difference of different segments"); }
    if self.off > other.off { panic!("Not a positive offset"); }
    other.off.0 - self.off.0
  }
}

impl FromStr for SegOff {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, String> {
    // FIXME: EXTEND TO PARSE OVERLAYS? "ovrxx:yyyy"
    // format: 'xxxx:yyyy' where xxxx and yyyy are 16-bit hexdecimal
    let idx = s.find(':').ok_or_else(|| format!("Invalid segoff: '{}'", s))?;
    Ok(SegOff {
      seg: Seg::Normal(u16::from_str_radix(&s[..idx], 16).map_err(|_| format!("Invalid segoff: '{}'", s))?),
      off: Off(u16::from_str_radix(&s[idx+1..], 16).map_err(|_| format!("Invalid segoff: '{}'", s))?),
    })
  }
}

impl fmt::Display for Seg {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Seg::Normal(seg) => write!(f, "{:04x}", seg),
      Seg::Overlay(seg) => write!(f, "ovr{:02x}", seg),
    }
  }
}

impl fmt::Display for Off {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{:04x}", self.0)
  }
}

impl fmt::Display for SegOff {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}:{}", self.seg, self.off)
  }
}
