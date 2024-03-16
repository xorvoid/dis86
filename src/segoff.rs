use std::str::FromStr;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SegOff {
  pub seg: u16,
  pub off: u16,
}

impl SegOff {
  pub fn abs(&self) -> usize {
    (self.seg as usize) * 16 + (self.off as usize)
  }

  pub fn add_offset(&self, off: u16) -> SegOff {
    SegOff { seg: self.seg, off: self.off.wrapping_add(off) }
  }

  pub fn offset_to(&self, other: SegOff) -> u16 {
    if self.seg != other.seg { panic!("Cannot take difference of different segments"); }
    if self.off > other.off { panic!("Not a positive offset"); }
    other.off - self.off
  }
}

impl FromStr for SegOff {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, String> {
    // format: 'xxxx:yyyy' where xxxx and yyyy are 16-bit hexdecimal
    let idx = s.find(':').ok_or_else(|| format!("Invalid segoff: '{}'", s))?;
    Ok(SegOff {
      seg: u16::from_str_radix(&s[..idx], 16).map_err(|_| format!("Invalid segoff: '{}'", s))?,
      off: u16::from_str_radix(&s[idx+1..], 16).map_err(|_| format!("Invalid segoff: '{}'", s))?,
    })
  }
}

impl fmt::Display for SegOff {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{:04x}:{:04x}", self.seg, self.off)
  }
}
