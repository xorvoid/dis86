use super::machine::*;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

// Configuration
const FILE_HANDLES_MAX: usize = 128;

// Wrapper type for file handle ids. Not strictly required, but it makes
// the API type-signatures more clear
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Handle(pub u16);

// Standard File Handles
#[allow(dead_code)] const FILE_HANDLE_STDIN:  Handle = Handle(0);
#[allow(dead_code)] const FILE_HANDLE_STDOUT: Handle = Handle(1);
#[allow(dead_code)] const FILE_HANDLE_STDERR: Handle = Handle(2);
#[allow(dead_code)] const FILE_HANDLE_STDAUX: Handle = Handle(3);
#[allow(dead_code)] const FILE_HANDLE_STDPRN: Handle = Handle(4);

// FIesystem data
pub struct Filesystem {
  pub root_dir: Option<String>,         // Host director
  pub handles: Vec<Option<HandleData>>, // None means "closed"
}

// Handle data
#[allow(dead_code)]
pub struct HandleData {
  filename: String,        // Dos filename, e.g. 'D:\SSG.EXE'
  handle:   Handle,        // Handle number
  file:     Option<File>,  // Host system file (for actual I/O) [May be None for standard handles]
}

// Filesystem helper functions (mostly around the file handle table)
impl Filesystem {
  pub fn new(root_dir: Option<&str>) -> Filesystem {
    let mut handles = vec![];
    handles.resize_with(FILE_HANDLES_MAX, || None);

    let mut fs = Filesystem {
      root_dir: root_dir.map(|dir| dir.to_string()),
      handles,
    };

    // init the standard handles
    fs.handles[0] = Some(HandleData { filename: "<stdin>".to_string(),  handle: Handle(0), file: None });
    fs.handles[1] = Some(HandleData { filename: "<stdout>".to_string(), handle: Handle(1), file: None });
    fs.handles[2] = Some(HandleData { filename: "<stderr>".to_string(), handle: Handle(2), file: None });
    fs.handles[3] = Some(HandleData { filename: "<stdaux>".to_string(), handle: Handle(3), file: None });
    fs.handles[4] = Some(HandleData { filename: "<stdprn>".to_string(), handle: Handle(4), file: None });

    fs
  }

  fn find_unused_handle(&self) -> Option<Handle> {
    for (idx, h) in self.handles.iter().enumerate() {
      if h.is_none() {
        return Some(Handle(idx as u16));
      }
    }
    None
  }

  fn create_handle(&mut self, filename: &str, file: File) -> Option<Handle> {
    let handle = self.find_unused_handle()?;

    self.handles[handle.0 as usize] = Some(HandleData {
      filename: filename.to_string(),
      handle,
      file: Some(file),
    });

    Some(handle)
  }

  fn lookup_handle(&mut self, id: Handle) -> Option<&mut HandleData> {
    let idx = id.0 as usize;
    assert!(idx < self.handles.len());
    self.handles[idx].as_mut()
  }

  fn remove_handle(&mut self, id: Handle) {
    let idx = id.0 as usize;
    assert!(idx < self.handles.len());
    self.handles[idx] = None;
  }
}

// Filesystem API implementations
impl Machine {
  // func: 0x3d
  pub fn dos_open_file(&mut self) {
    let filename_addr = self.reg_read_addr(DS, DX);
    let filename = self.mem.asciiz(filename_addr);

    let host_path = {
      let Some(filename) = filename.strip_prefix("D:\\") else {
        panic!("Expected all file opens to be in 'D:\'");
      };
      let Some(root_dir) = &self.dos.filesystem.root_dir else {
        panic!("Expected a root dir configuration");
      };
      format!("{}/{}", root_dir, filename.to_lowercase())
    };

    let file = File::open(host_path).unwrap();

    let Some(handle) = self.dos.filesystem.create_handle(filename, file) else {
      panic!("Failed to allocate a new file handle");
    };

    self.reg_write_u16(AX, handle.0);
    self.flag_write(FLAG_CF, false);
  }

  // func: 0x3e
  pub fn dos_close_file(&mut self) {
    let handle = Handle(self.reg_read_u16(BX));
    self.dos.filesystem.remove_handle(handle);
    self.flag_write(FLAG_CF, false);
  }

  // func: 0x3f
  pub fn dos_read_file(&mut self) {
    let handle = Handle(self.reg_read_u16(BX));
    let num_bytes = self.reg_read_u16(CX);
    let buffer_addr = self.reg_read_addr(DS, DX);

    let Some(handle_data) = self.dos.filesystem.lookup_handle(handle) else {
      panic!("No open handle {}", handle.0);
    };

    let buffer = self.mem.slice_mut_starting_at(buffer_addr);
    if buffer.len() < num_bytes as usize{
      panic!("Buffer length overruns memory!");
    }

    // FIXME: DOS API ALLOWS PARTIAL LENGTH READS
    let file = handle_data.file.as_mut().unwrap();
    file.read_exact(&mut buffer[..num_bytes as usize]).unwrap();

    self.reg_write_u16(AX, num_bytes);
  }

  // func: 0x40
  pub fn dos_write_file(&mut self) {
    panic!("UNIMPL");
    // // FIXME FINISH IMPL!!!
    // let bx = self.reg_read_u16(BX);  // Handle
    // let cx = self.reg_read_u16(CX);  // Num Bytes To Write
    // let ds_dx = self.reg_read_addr(DS, DX); // Buffer Address

    // if bx != 2 {
    //   panic!("expected stderr");
    // }

    // for i in 0..(cx as usize) {
    //   let addr = ds_dx.add_offset(i as u16);
    //   let byte = self.mem.read_u8(addr);
    //   let ch = char::from_u32(byte as u32).unwrap();
    //   eprint!("{}", ch);
    // }
    // eprintln!("");
  }

  // func: 0x42
  pub fn dos_seek_file(&mut self) {
    let handle = Handle(self.reg_read_u16(BX));
    let offset = self.reg_read_u32(CX, DX) as i32;  // Signed offset
    let method = self.reg_read_u8(AL); // Method

    let Some(handle_data) = self.dos.filesystem.lookup_handle(handle) else {
      panic!("No open handle {}", handle.0);
    };

    let pos = match method {
      0 => {
        assert!(offset >= 0);
        SeekFrom::Start(offset as u64)
      }
      1 => SeekFrom::End(offset as i64),
      2 => SeekFrom::Current(offset as i64),
      _ => panic!("invalid seek method: {}", method),
    };

    let file = handle_data.file.as_mut().unwrap();
    let new_pos = file.seek(pos).unwrap();
    assert!(new_pos as u32 as u64 == new_pos);

    self.reg_write_u32(DX, AX, new_pos as u32);
    self.flag_write(FLAG_CF, false);
  }

  // func: 0x43
  pub fn dos_get_or_set_file_attrs(&mut self) {
    let get_or_set = self.reg_read_u8(AL);
    match get_or_set {
      // get
      0 => self.dos_get_file_attrs(),
      // set
      1 => panic!("Attr set is unimplmented"),
      _ => panic!("Invalid value for get_or_set: {}", get_or_set),
    }
  }

  // func: 0x43, AL=0
  pub fn dos_get_file_attrs(&mut self) {
    let filename_addr = self.reg_read_addr(DS, DX);
    let filename = self.mem.asciiz(filename_addr);

    // TODO: Use the actual filename and actually impl attr info

    // NOTE: JUST TO MATCH DOSBOX
    self.reg_write_u16(AX, 0x20);
    self.reg_write_u16(CX, 0x20);
    self.flag_write(FLAG_CF, false);
  }
}
