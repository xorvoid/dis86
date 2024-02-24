fn main() {
    println!("cargo:rerun-if-changed=c/bsl/src/bsl/bsl.c");

  cc::Build::new()
    .cargo_warnings(false)
    .file("c/bsl/src/bsl/bsl.c")
    .compile("bsl_c");
}
