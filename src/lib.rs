extern crate static_assertions as sa;

// Helper libraries
mod util;
mod bsl;

// Core support libraries
pub mod binfmt;
pub mod binary;
pub mod region;
pub mod segoff;
pub mod spec;
pub mod config;
pub mod types;

// Subsystems
pub mod asm;
pub mod ir;
pub mod ast;
pub mod control_flow;
pub mod gen;

// Main application glue
pub mod app;
