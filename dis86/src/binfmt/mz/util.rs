use std::mem::size_of;

pub(super) unsafe fn try_struct_from_bytes<T: Sized>(data: &[u8]) -> Result<&T, String> {
  let sz = size_of::<T>();
  if data.len() < sz {
    Err(format!("Data is too short for {}: got {}, expected {}", std::any::type_name::<T>(), data.len(), sz))
  } else {
    Ok(unsafe { &*(data.as_ptr() as *const T) })
  }
}

pub(super) unsafe fn try_slice_from_bytes<T: Sized>(data: &[u8], nelts: usize) -> Result<&[T], String> {
  let len = nelts * size_of::<T>();
  if data.len() < len {
    Err(format!("Data is too short for {}: got {}, expected {}", std::any::type_name::<[T]>(), data.len(), len))
  } else {
    Ok(unsafe { std::slice::from_raw_parts(data.as_ptr() as *const T, nelts) })
  }
}
