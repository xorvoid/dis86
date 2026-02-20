use crate::segoff::{Seg, Off, SegOff};

pub struct RegionIter<'a> {
  mem: &'a [u8],
  base_seg: Seg,
  base_off: Off,
  off: usize
}

impl<'a> RegionIter<'a> {
  pub fn new(mem: &'a [u8], base_addr: SegOff) -> Self {
    Self { mem, base_seg: base_addr.seg, base_off: base_addr.off, off: 0 }
  }

  pub fn get_checked(&self, addr: SegOff) -> Result<u8, String> {
    if addr.seg != self.base_seg { return Err(format!("Mismatching segments")); }
    let addr = addr.off.0 as usize;
    let base = self.base_off.0 as usize;
    if addr < base { return Err(format!("RegionIter access below start of region")); }
    if addr >= base + self.mem.len() { return Err(format!("RegionIter access beyond end of region")); }
    Ok(self.mem[addr - base])
  }

  pub fn get(&self, addr: SegOff) -> u8 {
    self.get_checked(addr).unwrap()
  }

  pub fn slice(&self, addr: SegOff, len: u16) -> &'a [u8] {
    if addr.seg != self.base_seg { panic!("Mismatching segments"); }
    let addr = addr.off.0 as usize;
    let base = self.base_off.0 as usize;
    let len = len as usize;
    if addr < base { panic!("RegionIter access below start of region"); }
    if addr+len > base + self.mem.len() { panic!("RegionIter access beyond end of region"); }
    &self.mem[addr - base .. addr - base + len]
  }

  pub fn peek_checked(&self) -> Result<u8, String> {
    self.get_checked(self.addr())
  }

  pub fn peek(&self) -> u8 {
    self.peek_checked().unwrap()
  }

  pub fn advance(&mut self) {
    self.off += 1;
  }

  pub fn advance_by(&mut self, n: usize) {
    self.off += n;
  }

  pub fn fetch(&mut self) -> Result<u8, String> {
    let b = self.peek_checked()?;
    self.advance();
    Ok(b)
  }

  pub fn fetch_sext(&mut self) -> Result<u16, String> {
    let b = self.fetch()?;
    Ok(b as i8 as i16 as u16)
  }

  pub fn fetch_u16(&mut self) -> Result<u16, String> {
    let low = self.fetch()?;
    let high = self.fetch()?;
    Ok((high as u16) << 8 | (low as u16))
  }

  pub fn addr(&self) -> SegOff {
    let off: u16 = self.off.try_into().unwrap();
    self.base_addr().add_offset(off)
  }

  pub fn reset_addr(&mut self, addr: SegOff) {
    assert!(self.base_addr() <= addr && addr < self.end_addr());
    self.off = (addr.off.0 - self.base_off.0) as usize;
  }

  pub fn base_addr(&self) -> SegOff {
    SegOff { seg: self.base_seg, off: self.base_off }
  }

  pub fn end_addr(&self) -> SegOff {
    let off: u16 = self.mem.len().try_into().unwrap();
    self.base_addr().add_offset(off)
  }

  pub fn bytes_remaining(&self) -> usize {
    self.mem.len() - self.off
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test() {
    let addr: SegOff = "0000:000a".parse().unwrap();
    let mut b = RegionIter::new(&[0x12, 0x34, 0x56, 0x78, 0x9a], addr);
    assert_eq!(b.peek(), 0x12);
    assert_eq!(b.peek(), 0x12);

    b.advance();
    assert_eq!(b.peek(), 0x34);
    assert_eq!(b.get(addr), 0x12);

    let v = b.fetch().unwrap();
    assert_eq!(v, 0x34);
    assert_eq!(b.peek(), 0x56);

    let v = b.fetch_u16().unwrap();
    assert_eq!(v, 0x7856);

    assert_eq!(b.peek(), 0x9a);
  }
}
