
pub fn hex_u64(s: &str) -> Result<u64, &'static str> {
  if s.len() > 16 {
    return Err("Hex string too long to fit in u64");
  }
  let mut ret: u64 = 0;
  for c in s.chars() {
    let d = c.to_digit(16).ok_or_else(|| "Non-hexdigit char in string")?;
    ret = ret*16 + d as u64;
  }
  Ok(ret)
}

#[allow(unused)]
pub fn hex_u32(s: &str) -> Result<u32, &'static str> {
  if s.len() > 8 {
    return Err("Hex string too long to fit in u32");
  }
  let n = hex_u64(s)?;
  Ok(n as u32)
}

pub fn hex_u16(s: &str) -> Result<u16, &'static str> {
  if s.len() > 4 {
    return Err("Hex string too long to fit in u16");
  }
  let n = hex_u64(s)?;
  Ok(n as u16)
}

#[allow(unused)]
pub fn hex_u8(s: &str) -> Result<u8, &'static str> {
  if s.len() > 2 {
    return Err("Hex string too long to fit in u8");
  }
  let n = hex_u64(s)?;
  Ok(n as u8)
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test() {
    assert_eq!(hex_u64("5"), Ok(5));
    assert_eq!(hex_u64("a"), Ok(10));
    assert_eq!(hex_u64("deadbeaf"), Ok(3735928495));
  }
}
