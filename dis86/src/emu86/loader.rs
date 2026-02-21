use super::machine::*;
use super::mem::*;
use super::cpu::*;
use crate::binfmt::mz;

impl Machine {
  pub fn load_exe(&mut self, exe: &mz::Exe) -> Result<(), String> {
    let load_seg = PSP_SEGMENT;
    let code_seg = Seg::Normal(load_seg.unwrap_normal() + 0x10);

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

    // TODO PERFORM RELOCATIONS

    // Set up CS:IP
    self.reg_set(CS, code_seg.unwrap_normal() + exe.hdr.cs as u16);
    self.reg_set(IP, exe.hdr.ip as u16);

    // Set up SS:SP
    self.reg_set(SS, code_seg.unwrap_normal() + exe.hdr.ss as u16);
    self.reg_set(SP, exe.hdr.sp as u16);

    // Set up DS and ES to point at the PSP
    self.reg_set(DS, PSP_SEGMENT.unwrap_normal());
    self.reg_set(ES, PSP_SEGMENT.unwrap_normal());

    Ok(())
  }
}
