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
}
