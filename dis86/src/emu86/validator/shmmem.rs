pub use crate::segoff::SegOff;
use std::ffi::CString;
use std::ptr;

pub struct ShmMem {
  pub raw: *mut u8,
  pub len: usize,
}

impl ShmMem {
  #[allow(dead_code)]
  pub fn slice(&self) -> &[u8] {
    unsafe { std::slice::from_raw_parts(self.raw as *const u8, self.len) }
  }

  #[allow(dead_code)]
  pub fn slice_mut(&mut self) -> &mut [u8] {
    unsafe { std::slice::from_raw_parts_mut(self.raw, self.len) }
  }

  #[allow(dead_code)]
  pub fn slice_starting_at(&self, addr: SegOff) -> &[u8] {
    &self.slice()[addr.abs_normal()..]
  }

  #[allow(dead_code)]
  pub fn read_u8(&self, addr: SegOff) -> u8 {
    self.slice()[addr.abs_normal()]
  }

  #[allow(dead_code)]
  pub fn read_u16(&self, addr: SegOff) -> u16 {
    let idx = addr.abs_normal();
    u16::from_le_bytes(self.slice()[idx..idx+2].try_into().unwrap())
  }
}

impl ShmMem {
  pub fn attach(path: &str) -> Result<Self, String> {
    let cpath = CString::new(path).map_err(|e| e.to_string())?;

    let size = std::fs::metadata(path).unwrap().len() as usize;
    assert!(size % 4096 == 0);

    let fd = unsafe { libc::open(cpath.as_ptr(), libc::O_RDWR, 0o600u32) };
    if fd < 0 {
      return Err(last_os_error("open"));
    }

    let addr = unsafe {
      libc::mmap(
        ptr::null_mut(),
        size,
        libc::PROT_READ | libc::PROT_WRITE,
        libc::MAP_SHARED,
        fd,
        0,
      )
    };
    if addr == libc::MAP_FAILED {
      unsafe { libc::close(fd); }
      return Err(last_os_error("mmap"));
    }

    unsafe { libc::close(fd); }

    let raw = addr as *mut u8;
    Ok(Self { raw, len: size })
  }
}

impl Drop for ShmMem {
  fn drop(&mut self) {
    unsafe {
      let raw: *mut libc::c_void = self.raw as *mut _;
      if !raw.is_null() && raw != libc::MAP_FAILED {
        libc::munmap(raw, self.len);
      }
    }
  }
}

fn last_os_error(context: &str) -> String {
  let err = std::io::Error::last_os_error();
  format!("{}: {}", context, err)
}
