use super::machine::*;
use super::dos_filesystem::*;

// NOTE: JUST TO MATCH DOSBOX
pub const MEM_TOP: u16 = 0x9fff;
pub const ENV_SEG: u16 = 0x07ca;


pub struct Dos {
  // interrupts
  pub default_interrupt_vectors: [SegOff; 256],

  // file i/op
  pub filesystem: Filesystem,

  // memory
  pub mem_resize_call_count: usize,
}

// Init
impl Dos {
  pub fn new(root_dir: Option<&str>, mem: &mut Memory) -> Dos {
    let mut dos = Dos {
      default_interrupt_vectors: [SegOff::new(0, 0); 256],
      filesystem: Filesystem::new(root_dir),
      mem_resize_call_count: 0,
    };

    // NOTE: JUST TO MATCH DOSBOX
    dos.default_interrupt_vectors[0x00] = SegOff::new(0xf000, 0xca60); // Divide by zero
    dos.default_interrupt_vectors[0x04] = SegOff::new(0x0070, 0x00f4); // Overflow (INTO Instruction)
    dos.default_interrupt_vectors[0x05] = SegOff::new(0xf000, 0xff54); // BOUND range exceeded
    dos.default_interrupt_vectors[0x06] = SegOff::new(0xf000, 0xca60); // Invalid opcode
    dos.default_interrupt_vectors[0x08] = SegOff::new(0xf000, 0xfea5); // PIT timer
    dos.default_interrupt_vectors[0x09] = SegOff::new(0xf000, 0xe987); // Keyboard
    dos.default_interrupt_vectors[0x33] = SegOff::new(0xc402, 0x0010); // Mouse Handler
    dos.default_interrupt_vectors[0x3f] = SegOff::new(0xf000, 0xca60); // Overlay load interrupt

    // NOTE: JUST TO MATCH DOSBOX (FIXME: MAKE THIS LESS HACKY / DO IT RIGHT)
    let env_addr = SegOff::new(ENV_SEG, 0);
    mem.slice_mut_starting_at(env_addr)[..10*16].copy_from_slice(&[
      0x43, 0x4f, 0x4d, 0x53, 0x50, 0x45, 0x43, 0x3d, 0x5a, 0x3a, 0x5c, 0x43, 0x4f, 0x4d, 0x4d, 0x41,
      0x4e, 0x44, 0x2e, 0x43, 0x4f, 0x4d, 0x00, 0x50, 0x41, 0x54, 0x48, 0x3d, 0x5a, 0x3a, 0x5c, 0x3b,
      0x5a, 0x3a, 0x5c, 0x53, 0x59, 0x53, 0x54, 0x45, 0x4d, 0x3b, 0x5a, 0x3a, 0x5c, 0x42, 0x49, 0x4e,
      0x3b, 0x5a, 0x3a, 0x5c, 0x44, 0x4f, 0x53, 0x3b, 0x5a, 0x3a, 0x5c, 0x34, 0x44, 0x4f, 0x53, 0x3b,

      0x5a, 0x3a, 0x5c, 0x44, 0x45, 0x42, 0x55, 0x47, 0x3b, 0x5a, 0x3a, 0x5c, 0x54, 0x45, 0x58, 0x54,
      0x55, 0x54, 0x49, 0x4c, 0x00, 0x50, 0x52, 0x4f, 0x4d, 0x50, 0x54, 0x3d, 0x24, 0x50, 0x24, 0x47,
      0x00, 0x42, 0x4c, 0x41, 0x53, 0x54, 0x45, 0x52, 0x3d, 0x41, 0x32, 0x32, 0x30, 0x20, 0x49, 0x37,
      0x20, 0x44, 0x31, 0x20, 0x48, 0x35, 0x20, 0x50, 0x33, 0x33, 0x30, 0x20, 0x54, 0x36, 0x00, 0x00,

      0x01, 0x00, 0x44, 0x3a, 0x5c, 0x53, 0x53, 0x47, 0x2e, 0x45, 0x58, 0x45, 0x00, 0x4f, 0x55, 0x4e,
      0x54, 0x2e, 0x43, 0x4f, 0x4d, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ]);

    dos
  }
}

impl Machine {
  pub fn dos_interrupt_0x21(&mut self) {
    let func = self.reg_read_u8(AH);
    match func {
      0x25 => self.dos_set_interrupt_vector(),
      0x30 => self.dos_get_version(),
      0x35 => self.dos_get_interrupt_vector(),
      0x3d => self.dos_open_file(),
      0x3e => self.dos_close_file(),
      0x3f => self.dos_read_file(),
      0x40 => self.dos_write_file(),
      0x42 => self.dos_seek_file(),
      0x43 => self.dos_get_or_set_file_attrs(),
      0x44 => self.dos_ioctl(),
      0x4a => self.dos_mem_resize(),
      0x4c => self.dos_exit_program(),
      _ => panic!("unimplemented DOS function: {}", func),
    }
  }

  // func: 0x25
  fn dos_set_interrupt_vector(&mut self) {
    let idx = self.reg_read_u8(AL);
    println!("set_interrupt_vector | AL=0x{:x}", idx);
    let addr = self.reg_read_addr(DS, DX);
    self.interrupt_vectors[idx as usize] = Some(addr);
  }

  // func: 0x30
  fn dos_get_version(&mut self) {
    self.reg_write_u8(AL, 5); // major version
    self.reg_write_u8(AH, 0); // minor version

    // NOTE: JUST TO MATCH DOSBOX
    self.reg_write_u16(BX, 0xff00);
    self.reg_write_u16(CX, 0);

    // NOTE: Missing other fields
  }

  // func: 0x35
  fn dos_get_interrupt_vector(&mut self) {
    let idx = self.reg_read_u8(AL);
    let addr = match self.interrupt_vectors[idx as usize] {
      Some(addr) => addr,
      None => self.dos.default_interrupt_vectors[idx as usize],
    };
    self.reg_write_addr(ES, BX, addr);
  }

  // func: 0x4a
  fn dos_mem_resize(&mut self) {
    let segment_block = self.reg_read_u16(ES);
    let _new_size_par = self.reg_read_u16(BX);

    // FIXME: DON"T JUST ACCEPT ALL RESIZES

    // if self.dos.mem_resize_call_count > 0 {
    //   panic!("Memory resize is limited to one call on the load seg");
    // }
    // if segment_block != PSP_SEGMENT.unwrap_normal() {
    //   panic!("Memory resize is limited to one call on the load seg");
    // }

    // self.dos.mem_resize_call_count += 1;

    // NOTE: JUST TO MATCH DOSBOX
    // It seems to return the segment in AX on success
    self.reg_write_u16(AX, segment_block);

    // Success
    self.flag_write(FLAG_CF, false);
  }

  // func: 0x4c
  fn dos_exit_program(&mut self) {
    let al = self.reg_read_u8(AL);
    panic!("Exited with {}", al);
  }
}
