use crate::segoff::SegOff;

pub struct Binary {
  mem: Vec<u8>,
}

impl Binary {
  pub fn from_data(data: &[u8]) -> Self {
    Self { mem: data.to_vec() }
  }

  pub fn from_file(path: &str) -> Result<Self, String> {
    let mem = std::fs::read(path).map_err(
      |err| format!("Failed to read file: '{}': {:?}", path, err))?;
    Ok(Self { mem })
  }

  pub fn region(&self, start: SegOff, end: SegOff) -> &[u8] {
    assert!(start <= end);
    &self.mem[start.abs()..end.abs()]
  }

  pub fn region_iter(&self, start: SegOff, end: SegOff) -> RegionIter<'_> {
    RegionIter::new(self.region(start, end), start)
  }
}


pub struct RegionIter<'a> {
  mem: &'a [u8],
  base_addr: SegOff,
  off: usize
}

impl<'a> RegionIter<'a> {
  pub fn new(mem: &'a [u8], base_addr: SegOff) -> Self {
    Self { mem, base_addr, off: 0 }
  }

  pub fn get(&self, addr: SegOff) -> u8 {
    let addr = addr.abs();
    let base = self.base_addr.abs();
    if addr < base { panic!("RegionIter access below start of region"); }
    if addr >= base + self.mem.len() { panic!("RegionIter access beyond end of region"); }
    self.mem[addr - base]
  }

  pub fn slice(&self, addr: SegOff, len: u16) -> &'a [u8] {
    let addr = addr.abs();
    let base = self.base_addr.abs();
    let len = len as usize;
    if addr < base { panic!("RegionIter access below start of region"); }
    if addr+len > base + self.mem.len() { panic!("RegionIter access beyond end of region"); }
    &self.mem[addr - base .. addr - base + len]
  }

  pub fn peek(&self) -> u8 {
    self.get(self.addr())
  }

  pub fn advance(&mut self) {
    self.off += 1;
  }

  pub fn fetch(&mut self) -> u8 {
    let b = self.peek();
    self.advance();
    b
  }

  pub fn fetch_sext(&mut self) -> u16 {
    let b = self.fetch();
    b as i8 as i16 as u16
  }

  pub fn fetch_u16(&mut self) -> u16 {
    let low = self.fetch();
    let high = self.fetch();
    (high as u16) << 8 | (low as u16)
  }

  pub fn addr(&self) -> SegOff {
    let off: u16 = self.off.try_into().unwrap();
    self.base_addr.add_offset(off)
  }

  pub fn base_addr(&self) -> SegOff {
    self.base_addr
  }

  pub fn end_addr(&self) -> SegOff {
    let off: u16 = self.mem.len().try_into().unwrap();
    self.base_addr.add_offset(off)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test() {
    let addr = SegOff { seg: 0, off: 10 };
    let mut b = RegionIter::new(&[0x12, 0x34, 0x56, 0x78, 0x9a], addr);
    assert_eq!(b.peek(), 0x12);
    assert_eq!(b.peek(), 0x12);

    b.advance();
    assert_eq!(b.peek(), 0x34);
    assert_eq!(b.get(addr), 0x12);

    let v = b.fetch();
    assert_eq!(v, 0x34);
    assert_eq!(b.peek(), 0x56);

    let v = b.fetch_u16();
    assert_eq!(v, 0x7856);

    assert_eq!(b.peek(), 0x9a);
  }
}
