use super::machine::*;
use super::dos;
use crate::binfmt::mz;

impl Machine {
  pub fn load_exe(&mut self, exe: &mz::Exe) -> Result<(), String> {
    let load_seg = PSP_SEGMENT;
    let code_seg = Seg::Normal(load_seg.unwrap_normal() + 0x10);
    let code_seg_u16 = code_seg.unwrap_normal();

    // Configure the PSP
    let psp = self.mem.program_segment_prefix_mut();
    // NOTE: JUST TO MATCH DOSBOX
    psp.mem_top = dos::MEM_TOP;
    psp.env_seg = dos::ENV_SEG;
    psp.cmd_tail[0] = 0x0d;
    // ... missing fields ...

    // Determine image region to copy
    let image_start  = exe.hdr.cparhdr as usize * 16;
    let image_end    = exe.hdr.cp as usize * 512;
    let image_length = image_end - image_start;
    let image        = &exe.rawdata[image_start..image_end];

    // println!("image_start:   0x{:x}", image_start);
    // println!("image_end:     0x{:x}", image_end);
    // println!("image_length:  0x{:x}", image_length);

    // Copy into memory
    let mem_start = code_seg.abs_normal();
    let mem_end   = mem_start + image_length;
    self.mem.0[mem_start..mem_end].copy_from_slice(image);

    // Perform relocations
    for reloc in &exe.relocs {
      let addr = SegOff::new(code_seg_u16 + reloc.segment, reloc.offset);
      let val = self.mem.read_u16(addr);
      self.mem.write_u16(addr, code_seg_u16 + val);
    }

    // Set up CS:IP
    self.reg_set(CS, code_seg_u16 + exe.hdr.cs as u16);
    self.reg_set(IP, exe.hdr.ip as u16);

    // Set up SS:SP
    self.reg_set(SS, code_seg_u16 + exe.hdr.ss as u16);
    self.reg_set(SP, exe.hdr.sp as u16);

    // Set up DS and ES to point at the PSP
    self.reg_set(DS, PSP_SEGMENT.unwrap_normal());
    self.reg_set(ES, PSP_SEGMENT.unwrap_normal());

    // IF flag should be set
    self.reg_set(FLAGS, 1<<9); // IF

    Ok(())
  }
}
