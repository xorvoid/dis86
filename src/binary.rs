use crate::segoff::SegOff;

pub struct Binary<'a> {
  mem: &'a [u8],
  base_addr: SegOff,
  off: usize
}

impl<'a> Binary<'a> {
  pub fn new(mem: &'a [u8], base_addr: SegOff) -> Self {
    Self { mem, base_addr, off: 0 }
  }

  pub fn get(&self, addr: SegOff) -> u8 {
    let addr = addr.abs();
    let base = self.base_addr.abs();
    if addr < base { panic!("Binary access below start of region"); }
    if addr >= base + self.mem.len() { panic!("Binary access beyond end of region"); }
    self.mem[addr - base]
  }

  pub fn slice(&self, addr: SegOff, len: u16) -> &'a [u8] {
    let addr = addr.abs();
    let base = self.base_addr.abs();
    let len = len as usize;
    if addr < base { panic!("Binary access below start of region"); }
    if addr+len > base + self.mem.len() { panic!("Binary access beyond end of region"); }
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
    let mut b = Binary::new(&[0x12, 0x34, 0x56, 0x78, 0x9a], 10);
    assert_eq!(b.peek(), 0x12);
    assert_eq!(b.peek(), 0x12);

    b.advance();
    assert_eq!(b.peek(), 0x34);
    assert_eq!(b.get(10), 0x12);

    let v = b.fetch();
    assert_eq!(v, 0x34);
    assert_eq!(b.peek(), 0x56);

    let v = b.fetch_u16();
    assert_eq!(v, 0x7856);

    assert_eq!(b.peek(), 0x9a);
  }
}
