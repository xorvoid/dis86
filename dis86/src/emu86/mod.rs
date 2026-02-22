mod value;

mod mem;
mod cpu;
#[allow(dead_code)]
mod cpu_flags;
mod cpu_scas;
mod cpu_stos;

pub mod alu;

mod machine;
mod loader;
mod interrupt;
mod step;

mod dos;
#[allow(dead_code)]
mod dos_ivt;
mod dos_structs;

mod mzhdr;
mod emu;

pub use emu::run;

// Tests
#[cfg(test)] mod alu_test;
