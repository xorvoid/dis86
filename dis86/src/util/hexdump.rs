
#[allow(dead_code)]
pub fn hexdump(data: &[u8]) -> String {
  let mut s = String::new();
  for (i, chunk) in data.chunks(16).enumerate() {
    let addr = i * 16;
    let hex: Vec<String> = chunk.iter().map(|b| format!("{:02x}", b)).collect();
    s.push_str(&format!("{:08x}  {}\n", addr, hex.join(" ")));
  }
  s
}
