use super::machine::*;

impl Machine {
  pub fn io_port_inb(&mut self, port: u16) -> u8 {
    self.adlib.tick_us(3);
    //println!("IO PORT IN | port: 0x{:x}", port);
    match port {
      0x388 => self.adlib.read_status(),
      _ => panic!("Unsupported IO port on read: 0x{:x}", port),
    }
  }

  pub fn io_port_outb(&mut self, port: u16, data: u8) {
    self.adlib.tick_us(3);
    //println!("IO PORT OUT | port: 0x{:x}, data: 0x{:x}", port, data);
    match port {
      0x40 => (), // FIXME: WHAT IS THIS? Related to Keyboard or PIT??
      0x43 => (), // FIXME: WHAT IS THIS? Related to Keyboard or PIT??
      0x388 => self.adlib.write_addr(data as u16),
      0x389 => self.adlib.write_register(data),
      _ => panic!("Unsupported IO port on write: 0x{:x}", port),
    }
  }
}
