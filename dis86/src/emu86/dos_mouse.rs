use super::machine::*;
use crate::segoff::SegOff;

pub struct Mouse {
  button_status: u8,
  position_x: u16,
  position_y: u16,
  call_mask: u16,
  handler: Option<SegOff>,
}

impl Mouse {
  pub fn new() -> Mouse {
    Mouse {
      button_status: 0,
      position_x: 320,
      position_y: 100,
      call_mask: 0,
      handler: None,
    }
  }
}

impl Machine {
  pub fn mouse_interrupt_0x33(&mut self) {
    let func = self.reg_read_u16(AX);
    match func {
      0x00 => self.mouse_reset_driver(),
      0x03 => self.mouse_position_and_button_status(),
      0x0c => self.mouse_setup_handler(),
      _ => panic!("Unsupported mouse function: {}", func),
    }
  }

  fn mouse_reset_driver(&mut self) {
    self.reg_write_u16(AX, 0xffff); // hardware/driver installed
    self.reg_write_u16(BX, 0x3);    // 3 buttons
  }

  fn mouse_position_and_button_status(&mut self) {
    self.reg_write_u16(BX, self.dos.mouse.button_status as u16);
    self.reg_write_u16(CX, self.dos.mouse.position_x);
    self.reg_write_u16(DX, self.dos.mouse.position_y);
  }

  fn mouse_setup_handler(&mut self) {
    let call_mask = self.reg_read_u16(CX);
    let handler_addr = self.reg_read_addr(ES, DX);
    self.dos.mouse.call_mask = call_mask;
    self.dos.mouse.handler = Some(handler_addr);
  }
}
