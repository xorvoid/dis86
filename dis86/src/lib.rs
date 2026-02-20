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
pub mod access;

// Subsystems
pub mod asm;
pub mod analyze;
pub mod decompile;

// Main application glue
pub mod app;
pub mod app_analyze;
