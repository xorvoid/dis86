use std::mem::MaybeUninit;
use std::ops::Index;

#[derive(Clone, Copy)]
pub struct ArrayVec<T: Copy, const N: usize> {
  mem: [MaybeUninit<T>; N],
  len: usize,
}

impl<T: Copy, const N: usize> ArrayVec<T, N> {
  pub fn new() -> Self {
    Self {
      mem: [MaybeUninit::uninit(); N],
      len: 0,
    }
  }

  pub fn as_slice(&self) -> &[T] {
    // SAFETY: we maintain the invariant that `len` specifies the valid
    // region of elements 0..len
    unsafe { slice_assume_init_ref(&self.mem[..self.len]) }
  }

  pub fn len(&self) -> usize {
    self.len
  }

  pub fn push(&mut self, obj: T) {
    if self.len >= N {
      panic!("ArrayVec capacity overflow");
    }
    self.mem[self.len].write(obj);
    self.len += 1;
  }
}

pub unsafe fn slice_assume_init_ref<T>(slice: &[MaybeUninit<T>]) -> &[T] {
  // SAFETY: casting `slice` to a `*const [T]` is safe since the caller guarantees that
  // `slice` is initialized, and `MaybeUninit` is guaranteed to have the same layout as `T`.
  // The pointer obtained is valid since it refers to memory owned by `slice` which is a
  // reference and thus guaranteed to be valid for reads.
  unsafe { &*(slice as *const [MaybeUninit<T>] as *const [T]) }
}

impl<T: Copy + std::fmt::Debug, const N: usize> std::fmt::Debug for ArrayVec<T, N> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
    let mut first = true;
    write!(f, "[")?;
    for val in self.as_slice() {
      if !first {
        write!(f, ", ")?;
      }
      first = false;
      write!(f, "{:?}", val)?;
    }
    write!(f, "]")
  }
}

impl<T: Copy, const N: usize> Index<usize> for ArrayVec<T, N> {
  type Output = T;
  fn index(&self, idx: usize) -> &Self::Output {
    &self.as_slice()[idx]
  }
}

impl<T: Copy, const N: usize> AsRef<[T]> for ArrayVec<T, N> {
  fn as_ref(&self) -> &[T] {
    self.as_slice()
  }
}

impl<'a, T: Copy, const N: usize> IntoIterator for &'a ArrayVec<T, N> {
  type Item = &'a T;
  type IntoIter = std::slice::Iter<'a, T>;
  fn into_iter(self) -> <&'a ArrayVec<T, N> as IntoIterator>::IntoIter {
    self.as_slice().into_iter()
  }
}
