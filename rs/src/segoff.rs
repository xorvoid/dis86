use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub struct SegOff {
  pub seg: u16,
  pub off: u16,
}

impl SegOff {
  pub fn abs(&self) -> usize {
    (self.seg as usize) * 16 + (self.off as usize)
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
