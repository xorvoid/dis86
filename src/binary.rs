

pub struct Binary<'a> {
  mem: &'a [u8],
  addr: usize,
  base_addr: usize,
}

impl<'a> Binary<'a> {
  pub fn new(mem: &'a [u8], base_addr: usize) -> Self {
    Self { mem, addr: base_addr, base_addr }
  }

  pub fn get(&self, addr: usize) -> u8 {
    if addr < self.base_addr { panic!("Binary access below start of region"); }
    if addr >= self.base_addr + self.mem.len() { panic!("Binary access beyond end of region"); }
    self.mem[addr - self.base_addr]
  }

  pub fn slice(&self, addr: usize, len: usize) -> &'a [u8] {
    if addr < self.base_addr { panic!("Binary access below start of region"); }
    if addr+len > self.base_addr + self.mem.len() { panic!("Binary access beyond end of region"); }
    &self.mem[addr - self.base_addr .. addr - self.base_addr + len]
  }

  pub fn peek(&self) -> u8 {
    self.get(self.addr)
  }

  pub fn advance(&mut self) {
    self.addr += 1;
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

  pub fn addr(&self) -> usize {
    self.addr
  }

  pub fn base_addr(&self) -> usize {
    self.base_addr
  }

  pub fn end_addr(&self) -> usize {
    self.base_addr + self.mem.len()
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
