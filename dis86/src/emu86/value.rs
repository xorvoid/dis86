use crate::segoff::SegOff;

#[derive(Debug, Clone, Copy)]
pub enum Value {
  U8(u8),
  U16(u16),
  U32(u32),
  Addr(SegOff),
}

impl From<u8>     for Value { fn from(val: u8)     -> Value { Value::U8(val)   } }
impl From<u16>    for Value { fn from(val: u16)    -> Value { Value::U16(val)  } }
impl From<u32>    for Value { fn from(val: u32)    -> Value { Value::U32(val)  } }
impl From<SegOff> for Value { fn from(val: SegOff) -> Value { Value::Addr(val) } }

impl Value {
  #[allow(dead_code)]
  pub fn unwrap_u8(&self) -> u8 {
    let Value::U8(val) = self else { panic!("expected Value::U8") };
    *val
  }

  #[allow(dead_code)]
  pub fn unwrap_u16(&self) -> u16 {
    let Value::U16(val) = self else { panic!("expected Value::U16") };
    *val
  }

  #[allow(dead_code)]
  pub fn unwrap_u32(&self) -> u32 {
    let Value::U32(val) = self else { panic!("expected Value::U32") };
    *val
  }

  #[allow(dead_code)]
  pub fn unwrap_addr(&self) -> SegOff {
    let Value::Addr(val) = self else { panic!("expected Value::Addr") };
    *val
  }

  pub fn arith_add(self, rhs: Value) -> Value {
    match (self, rhs) {
      (Value::U8(lhs),  Value::U8(rhs))  => Value::U8(lhs.wrapping_add(rhs)),
      (Value::U16(lhs), Value::U16(rhs)) => Value::U16(lhs.wrapping_add(rhs)),
      _ => panic!("Mismatched sizes"),
    }
  }

  pub fn arith_inc(self) -> Value {
    match self {
      Value::U8(lhs)  => Value::U8(lhs.wrapping_add(1)),
      Value::U16(lhs) => Value::U16(lhs.wrapping_add(1)),
      _ => panic!("Mismatched sizes"),
    }
  }

  pub fn arith_neg(self) -> Value {
    match self {
      Value::U8(lhs)  => Value::U8(-(lhs as i8) as u8),
      Value::U16(lhs) => Value::U16(-(lhs as i16) as u16),
      _ => panic!("Mismatched sizes"),
    }
  }

  pub fn arith_sub(self, rhs: Value) -> Value {
    match (self, rhs) {
      (Value::U8(lhs),  Value::U8(rhs))  => Value::U8(lhs.wrapping_sub(rhs)),
      (Value::U16(lhs), Value::U16(rhs)) => Value::U16(lhs.wrapping_sub(rhs)),
      _ => panic!("Mismatched sizes"),
    }
  }

  pub fn bitwise_and(self, rhs: Value) -> Value {
    match (self, rhs) {
      (Value::U8(lhs),  Value::U8(rhs))  => Value::U8(lhs & rhs),
      (Value::U16(lhs), Value::U16(rhs)) => Value::U16(lhs & rhs),
      _ => panic!("Mismatched sizes"),
    }
  }

  pub fn bitwise_or(self, rhs: Value) -> Value {
    match (self, rhs) {
      (Value::U8(lhs),  Value::U8(rhs))  => Value::U8(lhs | rhs),
      (Value::U16(lhs), Value::U16(rhs)) => Value::U16(lhs | rhs),
      _ => panic!("Mismatched sizes"),
    }
  }

  pub fn bitwise_xor(self, rhs: Value) -> Value {
    match (self, rhs) {
      (Value::U8(lhs),  Value::U8(rhs))  => Value::U8(lhs ^ rhs),
      (Value::U16(lhs), Value::U16(rhs)) => Value::U16(lhs ^ rhs),
      _ => panic!("Mismatched sizes"),
    }
  }

  pub fn shift_shl(self, lhs: Value, count: u8) -> Value {
    match lhs {
      Value::U8(lhs)  => Value::U8((lhs as u32).wrapping_shl(count as u32) as u8),
      Value::U16(lhs) => Value::U16((lhs as u32).wrapping_shl(count as u32) as u16),
      _ => panic!("Mismatched sizes"),
    }
  }
}
