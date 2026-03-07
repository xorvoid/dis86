use super::machine::*;
use super::opl::Opl3;

const DEBUG: bool = false;

// ADLIB Handling
pub struct Adlib {
  opl: Opl3,
  reg_addr: u16,
  timer1: Timer<80>,
  timer2: Timer<320>,
}

impl Adlib {
  pub fn new() -> Adlib {
    Adlib {
      opl: Opl3::new(1440), // FIXME sample rate
      reg_addr: 0,
      timer1: Timer::new(),
      timer2: Timer::new(),
    }
  }

  pub fn tick_us(&mut self, us: u32) {
    self.timer1.tick_us(us);
    self.timer2.tick_us(us);
  }

  pub fn write_addr(&mut self, addr: u16) {
    self.reg_addr = addr;
  }

  pub fn write_register(&mut self, dat: u8) {
    if DEBUG { println!("adlib_write | reg: 0x{:x}, dat: 0x{:x}", self.reg_addr, dat); }
    match self.reg_addr {
      0x2 => self.timer1.reload = dat,
      0x3 => self.timer2.reload = dat,
      0x4 => self.timer_configure(dat),
      _ => {
        self.opl.write_reg(self.reg_addr, dat);
      }
    }
  }

  fn timer_configure(&mut self, dat: u8) {
    // IRQ reset
    if bit_is_set(dat, 7) {
      self.timer1.expired = false;
      self.timer2.expired = false;
      return;
    }

    // Timer 1 configure
    self.timer1.masked = bit_is_set(dat, 6);
    if bit_is_set(dat, 0) {
      if !self.timer1.running {
        self.timer1.counter = self.timer1.reload;
        self.timer1.running = true;
      }
    } else {
      self.timer1.running = false;
    }

    // Timer 2 configure
    self.timer2.masked = bit_is_set(dat, 5);
    if bit_is_set(dat, 1) {
      if !self.timer2.running {
        self.timer2.counter = self.timer2.reload;
        self.timer2.running = true;
      }
    } else {
      self.timer2.running = false;
    }
  }

  pub fn read_status(&mut self) -> u8 {
    let mut status = 0;
    if self.timer1.expired { status |= 0xc0; } // bit 6 & 7
    if self.timer2.expired { status |= 0xa0; } // bit 5 & 7

    if DEBUG { println!("adlib_read_status | 0x{:x}", status); }

    status
  }

}

struct Timer<const PERIOD_US: u32> {
  accum_us: u32,   // accumulated time (in micro-seconds)
  counter: u8,     // current timer count
  reload: u8,      // timer count to reload on overflow
  masked: bool,    // when masked, don't update the "expired" feild on overflow
  running: bool,   // only update the count when started / running
  expired: bool,   // has the timer expired / rolled-over ?
}

impl<const PERIOD_US: u32> Timer<PERIOD_US> {
  fn new() -> Self {
    Self {
      accum_us: 0,
      counter: 0,
      reload: 0,
      masked: false,
      running: false,
      expired: false,
    }
  }

  fn tick_us(&mut self, us: u32) {
    self.accum_us += us;
    while self.accum_us >= PERIOD_US {
      if DEBUG {
        println!("");
        println!("Timer tick:");
        println!("---------------------------------------");
        println!("  period:  {} micros", PERIOD_US);
        println!("  counter: {}", self.counter);
        println!("  reload:  {}", self.reload);
        println!("  masked:  {}", self.masked);
        println!("  running: {}", self.running);
        println!("  expired: {}", self.expired);
        println!("");
      }
      self.accum_us -= PERIOD_US;
      if self.running {
        self.counter = self.counter.wrapping_add(1);
        if !self.masked && self.counter == 0 {
          self.expired = true;
        }
      }
    }
  }
}

#[inline(always)]
fn bit_is_set(data: u8, bitno: u8) -> bool {
  ((data >> bitno) & 1) != 0
}
