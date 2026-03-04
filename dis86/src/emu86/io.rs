use super::machine::*;

impl Machine {
  pub fn io_port_inb(&mut self, port: u16) -> u8 {
    println!("IO PORT IN | port: 0x{:x}", port);
    // FIXME
    0x00
  }

  pub fn io_port_outb(&mut self, port: u16, data: u8) {
    println!("IO PORT OUT | port: 0x{:x}, data: 0x{:x}", port, data);
  }
}
