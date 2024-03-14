use std::ops::{Index, IndexMut, Range};

pub type DVecIndex = i64;

// Double-ended Vector
#[derive(Debug)]
pub struct DVec<T> {
  neg: Vec<T>,
  pos: Vec<T>,
}

impl<T> DVec<T> {
  pub fn new() -> Self {
    Self {
      neg: vec![],
      pos: vec![],
    }
  }

  pub fn start(&self) -> DVecIndex {
    -(self.neg.len() as i64)
  }

  pub fn end(&self) -> DVecIndex {
    self.pos.len() as i64
  }

  pub fn range(&self) -> Range<DVecIndex> {
    self.start()..self.end()
  }

  pub fn empty(&self) -> bool {
    self.start() == 0 && self.end() == 0
  }

  pub fn push_front(&mut self, val: T) -> DVecIndex {
    self.neg.push(val);
    self.start()
  }

  pub fn push_back(&mut self, val: T) -> DVecIndex {
    let idx = self.end();
    self.pos.push(val);
    idx
  }

  pub fn last_idx(&self) -> Option<DVecIndex> {
    if self.pos.len() > 0 {
      Some((self.pos.len()-1) as i64)
    } else if self.neg.len() > 0 {
      Some(-1)
    } else {
      None
    }
  }

  pub fn last(&self) -> Option<&T> {
    let idx = self.last_idx()?;
    Some(&self[idx])
  }
}

impl<T> Index<DVecIndex> for DVec<T> {
  type Output = T;
  fn index(&self, index: DVecIndex) -> &Self::Output {
    if index < 0 {
      &self.neg[-(index+1) as usize]
    } else {
      &self.pos[index as usize]
    }
  }
}

impl<T> IndexMut<DVecIndex> for DVec<T> {
  fn index_mut(&mut self, index: DVecIndex) -> &mut Self::Output {
    if index < 0 {
      &mut self.neg[-(index+1) as usize]
    } else {
      &mut self.pos[index as usize]
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test() {
    let mut d = DVec::new();
    assert_eq!(d.start(), 0);
    assert_eq!(d.end(), 0);

    d.push_back(4);
    assert_eq!(d.start(), 0);
    assert_eq!(d.end(), 1);

    d.push_front(3);
    assert_eq!(d.start(), -1);
    assert_eq!(d.end(), 1);

    d.push_front(5);
    assert_eq!(d.start(), -2);
    assert_eq!(d.end(), 1);

    let idx: Vec<_> = d.range().collect();
    assert_eq!(idx, vec![-2, -1, 0]);

    let elts: Vec<_> = d.range().map(|i| d[i]).collect();
    assert_eq!(elts, vec![5, 3, 4]);

    d[-1] = 42;
    let elts: Vec<_> = d.range().map(|i| d[i]).collect();
    assert_eq!(elts, vec![5, 42, 4]);
  }
}
