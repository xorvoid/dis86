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
  info:     u16,           // Device Info
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
    // FIXME: INFO MIGHT BE WRONG
    fs.handles[0] = Some(HandleData { filename: "<stdin>".to_string(),  handle: Handle(0), info: 0x80d3, file: None });
    fs.handles[1] = Some(HandleData { filename: "<stdout>".to_string(), handle: Handle(1), info: 0x80d3, file: None });
    fs.handles[2] = Some(HandleData { filename: "<stderr>".to_string(), handle: Handle(2), info: 0x80d3, file: None });
    fs.handles[3] = Some(HandleData { filename: "<stdaux>".to_string(), handle: Handle(3), info: 0x80d3, file: None });
    fs.handles[4] = Some(HandleData { filename: "<stdprn>".to_string(), handle: Handle(4), info: 0x80d3, file: None });

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

  fn create_handle(&mut self, filename: &str, file: File, info: u16) -> Option<Handle> {
    let handle = self.find_unused_handle()?;

    self.handles[handle.0 as usize] = Some(HandleData {
      filename: filename.to_string(),
      handle,
      info,
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

struct DosPath {
  drive: Option<char>,
  components: Vec<String>,
}

impl DosPath {
  fn parse(s: &str) -> Result<DosPath, String> {
    if s.is_empty() {
      return Err("Path cannot be empty".to_string());
    }

    let mut drive = None;
    let mut rest = s;
    if s.len() >= 2 && s.as_bytes()[1] == b':' {
      let d = s.chars().next().unwrap();
      if !d.is_ascii_alphabetic() {
        return Err(format!("Invalid drive letter: '{}'", d));
      }
      drive = Some(d.to_ascii_uppercase());
      rest = &s[2..];
    }

    // Normalize separators: DOS allows both \ and /
    let rest = rest.replace('/', "\\");

    let components: Vec<String> = rest
      .split('\\')
      .filter(|c| !c.is_empty())
      .map(|c| {
        // Validate each component: no forbidden characters
        // DOS forbidden: < > : " / \ | ? *
        for ch in c.chars() {
          if matches!(ch, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') {
            return Err(format!("Invalid character '{}' in path component '{}'", ch, c));
          }
          if (ch as u32) < 32 {
            return Err(format!("Control character in path component '{}'", c));
          }
        }
        Ok(c.to_string())
      })
      .collect::<Result<Vec<_>, _>>()?;

    Ok(DosPath { drive, components })
  }

  fn to_hostpath(&self, host_rootdir: &str) -> String {
    match self.drive {
      None => (), // Relative path
      Some('D') => (), // Abs path
      _ => panic!("Only drive D is supported currently"),
    };

    let mut path = host_rootdir.to_string();
    for component in &self.components {
      path += "/";
      path += &component.to_lowercase();
    }
    path
  }
}

// Filesystem API implementations
impl Machine {
  // func: 0x3d
  pub fn dos_open_file(&mut self) {
    let filename_addr = self.reg_read_addr(DS, DX);
    let filename = self.mem.asciiz(filename_addr);
    let path = DosPath::parse(filename).unwrap();

    println!("open file | '{}'", filename);

    let Some(root_dir) = &self.dos.filesystem.root_dir else {
      panic!("Expected a root dir configuration");
    };

    let dos_path = DosPath::parse(filename).unwrap();
    let host_path = dos_path.to_hostpath(root_dir);

    let file = File::open(host_path).unwrap();
    let info = 0x3;

    let Some(handle) = self.dos.filesystem.create_handle(filename, file, info) else {
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

    let file = handle_data.file.as_mut().unwrap();

    let len = match file.read(&mut buffer[..num_bytes as usize]) {
      Ok(len) => len,
      Err(err) => panic!("Failed to read with {:?}", err),
    };

    self.reg_write_u16(AX, len as u16);
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


  // func: 0x44
  pub fn dos_ioctl(&mut self) {
    let func = self.reg_read_u8(AL);
    match func {
      0 => self.dos_ioctl_get_device_info(),
      _ => panic!("unimplmented ioctl"),
    }
  }

  fn dos_ioctl_get_device_info(&mut self) {
    let handle = Handle(self.reg_read_u16(BX));
    println!("ioctl handle: {}", handle.0);

    let Some(handle_data) = self.dos.filesystem.lookup_handle(handle) else {
      panic!("No open handle {}", handle.0);
    };

    let info = handle_data.info;

    self.reg_write_u16(DX, info);
    self.reg_write_u16(AX, info);
  }
}
