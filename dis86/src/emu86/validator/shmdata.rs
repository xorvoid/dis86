use std::ffi::CString;
use std::ptr;

//// IMPORTANT!! THIS MUST MATCH THE STRUCT DEFINED IN hydra/src/remote/shmdata.h
#[repr(C, packed)]
#[derive(Debug)]
pub struct ShmDataRaw {
  pub init: u32,
  pub end: u32,
  pub pid: u32,
  pub req: u64,  // request step by incrementing
  pub ack: u64,  // ack step by matching 'req' value

  // registers
  pub ax: u16,
  pub bx: u16,
  pub cx: u16,
  pub dx: u16,
  pub si: u16,
  pub di: u16,
  pub bp: u16,
  pub sp: u16,
  pub ip: u16,
  pub cs: u16,
  pub ds: u16,
  pub es: u16,
  pub ss: u16,
  pub flags: u16,

  // memory
  // TODO...
}

#[macro_export]
macro_rules! shmdata_read {
  ($dat:expr, $field:ident) => {
    unsafe {
      let _typecheck: &ShmData = &$dat; // type check
      std::ptr::read_unaligned(std::ptr::addr_of!((*$dat.raw).$field))
    }
  };
}

#[macro_export]
macro_rules! shmdata_write {
  ($dat:expr, $field:ident, $val:expr) => {
    unsafe {
      let _typecheck: &ShmData = &$dat; // type check
      std::ptr::write_unaligned(std::ptr::addr_of_mut!((*$dat.raw).$field), $val)
    }
  };
}

pub struct ShmData {
  pub raw: *mut ShmDataRaw,
}

impl ShmData {
  fn size() -> usize {
    (std::mem::size_of::<ShmData>() + 4095) & !4095
  }

  pub fn attach(path: &str) -> Result<Self, String> {
    let size = Self::size();
    let cpath = CString::new(path).map_err(|e| e.to_string())?;

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

    let raw = addr as *mut ShmDataRaw;
    Ok(Self { raw })
  }

  // /// Get a typed pointer into the shared region at a byte offset.
  // /// Caller must ensure alignment, bounds, and that no conflicting references exist.
  // pub unsafe fn as_ptr<T>(&self, offset: usize) -> *mut T {
  //     (self.addr as *mut u8).add(offset) as *mut T
  // }

  // /// Get the region as a mutable byte slice.
  // pub unsafe fn as_bytes_mut(&mut self) -> &mut [u8] {
  //     std::slice::from_raw_parts_mut(self.addr as *mut u8, self.size)
  // }
}

impl Drop for ShmData {
  fn drop(&mut self) {
    unsafe {
      let raw: *mut libc::c_void = self.raw as *mut _;
      let size = Self::size();
      if !raw.is_null() && raw != libc::MAP_FAILED {
        libc::munmap(raw, size);
      }
    }
  }
}

fn last_os_error(context: &str) -> String {
  let err = std::io::Error::last_os_error();
  format!("{}: {}", context, err)
}
