use dis86::binfmt::mz;
use std::fs;

fn main() {
  let args: Vec<String> = std::env::args().collect();
  if args.len() != 3 {
    eprintln!("usage: {} <path> <outdir>", args[0]);
    std::process::exit(1);
  }
  let path = &args[1];
  let outdir = &args[2];

  // Read and decode the exe
  let Ok(data) = fs::read(path) else {
    panic!("Failed to read file: {}", path);
  };
  let exe = mz::Exe::decode(&data).unwrap();

  // Prepare outdir
  fs::create_dir_all(outdir).unwrap();

  // Write exe data
  fs::write(&format!("{}/exe.bin", outdir), exe.exe_data()).unwrap();

  // Write overlay data (if it exists)
  if let Some(ovr) = exe.ovr.as_ref() {
    for i in 0..ovr.segs.len() {
      let dat = exe.overlay_data(i);
      fs::write(&format!("{}/overlay_{:04}.bin", outdir, i), dat).unwrap();
    }
  }

  //println!("{:#?}", exe);
  //exe.print();
}
