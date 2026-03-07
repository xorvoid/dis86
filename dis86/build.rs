fn main() {
  println!("cargo:rerun-if-changed=bsl/src/bsl/bsl.c");
  println!("cargo:rerun-if-changed=bsl/src/bsl/bsl.h");
  println!("cargo:rerun-if-changed=src/emu86/opl/c/nuked_opl/opl3.c");
  println!("cargo:rerun-if-changed=src/emu86/opl/c/nuked_opl/opl3.h");

  cc::Build::new()
    .cargo_warnings(false)
    .file("../bsl/src/bsl/bsl.c")
    .compile("bsl_c");

  cc::Build::new()
    .cargo_warnings(false)
    .file("src/emu86/opl/c/nuked_opl/opl3.c")
    .compile("opl3_c");
}
