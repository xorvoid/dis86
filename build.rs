fn main() {
    println!("cargo:rerun-if-changed=subprojects/bsl/src/bsl/bsl.c");
    println!("cargo:rerun-if-changed=subprojects/bsl/src/bsl/bsl.h");

  cc::Build::new()
    .cargo_warnings(false)
    .file("subprojects/bsl/src/bsl/bsl.c")
    .compile("bsl_c");
}
