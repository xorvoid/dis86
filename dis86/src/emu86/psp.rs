
#[repr(C, packed)]
#[derive(Debug, Clone)]
pub struct ProgramSegmentPrefix {
    pub int20:              [u8; 2],    // 0x00 - INT 20h instruction (CD 20), legacy CP/M exit
    pub mem_top:            u16,        // 0x02 - Segment of first byte beyond program's allocated memory
    pub reserved_04:        u8,         // 0x04 - Reserved (CP/M remnant)
    pub cpm_call:           [u8; 5],    // 0x05 - Far call to DOS function dispatcher (legacy CP/M compatibility)
    pub int22_vec:          [u8; 4],    // 0x0A - Saved INT 22h vector (terminate address), restored on exit
    pub int23_vec:          [u8; 4],    // 0x0E - Saved INT 23h vector (Ctrl-C handler), restored on exit
    pub int24_vec:          [u8; 4],    // 0x12 - Saved INT 24h vector (critical error handler), restored on exit
    pub parent_psp_seg:     u16,        // 0x16 - Segment of parent process PSP (e.g. COMMAND.COM)
    pub jft:                [u8; 20],   // 0x18 - Job File Table: 20 entries mapping handles to SFT indices
                                        //        0xFF = unused slot. Handles 0-4 pre-opened (stdin/out/err/aux/prn)
    pub env_seg:            u16,        // 0x2C - Segment of environment block (null-terminated VAR=VALUE pairs)
    pub caller_sp:          u16,        // 0x2E - Saved SP of caller (used internally by DOS during INT 21h)
    pub caller_ss:          u16,        // 0x30 - Saved SS of caller
    pub jft_size:           u16,        // 0x32 - Number of entries in the JFT (default 20, expandable via AH=67h)
    pub jft_ptr_off:        u16,        // 0x34 - Offset of JFT far pointer (normally PSP:0x18, can be redirected)
    pub jft_ptr_seg:        u16,        // 0x36 - Segment of JFT far pointer
    pub prev_psp:           [u8; 4],    // 0x38 - Previous PSP (used by SHARE.EXE and network redirectors)
    pub reserved_3c:        [u8; 4],    // 0x3C - Reserved
    pub dos_version:        u16,        // 0x40 - DOS version reported to this program (may be faked by SETVER)
                                        //        low byte = minor, high byte = major
    pub reserved_42:        [u8; 14],   // 0x42 - Reserved
    pub int21_retf:         [u8; 3],    // 0x50 - INT 21h / RETF stub (CD 21 CB), legacy CP/M call entry point
    pub reserved_53:        [u8; 9],    // 0x53 - Reserved
    pub fcb1:               [u8; 16],   // 0x5C - Default FCB 1, pre-filled from first command-line argument
    pub fcb2:               [u8; 16],   // 0x6C - Default FCB 2, pre-filled from second command-line argument
    pub reserved_7c:        [u8; 4],    // 0x7C - Reserved (may be overwritten if FCBs are opened)
    pub cmd_tail_len:       u8,         // 0x80 - Length of command tail (bytes following program name on command line)
    pub cmd_tail:           [u8; 127],  // 0x81 - Command tail: space-prefixed arguments, terminated by 0x0D (CR)
}
sa::const_assert!(std::mem::size_of::<ProgramSegmentPrefix>() == 256);

impl ProgramSegmentPrefix {
  pub fn zeroed() -> Self {
    unsafe { std::mem::zeroed() }
  }

  pub fn from_slice(slice: &[u8]) -> &ProgramSegmentPrefix {
    assert!(slice.len() == std::mem::size_of::<ProgramSegmentPrefix>());
    unsafe { &*(slice.as_ptr() as *const _) }
  }

  pub fn from_slice_mut(slice: &mut [u8]) -> &mut ProgramSegmentPrefix {
    assert!(slice.len() == std::mem::size_of::<ProgramSegmentPrefix>());
    unsafe { &mut *(slice.as_mut_ptr() as *mut _) }
  }
}
