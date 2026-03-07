
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct opl3_chip {
    _unused: [u8; 0], // FIXME: Get actual size??
}

extern "C" {
  pub fn OPL3_New(
  ) -> *mut opl3_chip;
}

extern "C" {
  pub fn OPL3_Delete(
    chip: *mut opl3_chip,
  );
}

extern "C" {
  pub fn OPL3_Reset(
    chip: *mut opl3_chip,
    samplerate: u32,
  );
}

extern "C" {
  pub fn OPL3_WriteReg(
    chip: *mut opl3_chip,
    reg: u16,
    v: u8,
  );
}

extern "C" {
  pub fn OPL3_WriteRegBuffered(
    chip: *mut opl3_chip,
    reg: u16,
    v: u8,
  );
}

extern "C" {
  pub fn OPL3_GenerateStream(
    chip: *mut opl3_chip,
    sndptr: *mut i16,
    numsamples: u32,
  );
}
