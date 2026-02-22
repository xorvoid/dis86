use super::machine::*;

#[derive(Debug, Clone, Copy)]
pub struct Flag { pub mask: u16, pub shift: u16 }

pub const FLAG_CF: Flag = Flag { mask: 0x0001, shift: 0  };  // Carry
pub const FLAG_PF: Flag = Flag { mask: 0x0004, shift: 2  };  // Parity
pub const FLAG_AF: Flag = Flag { mask: 0x0010, shift: 4  };  // Auxilliary Carry
pub const FLAG_ZF: Flag = Flag { mask: 0x0040, shift: 6  };  // Zero
pub const FLAG_SF: Flag = Flag { mask: 0x0080, shift: 7  };  // Sign
pub const FLAG_TF: Flag = Flag { mask: 0x0100, shift: 8  };  // Trap
pub const FLAG_IF: Flag = Flag { mask: 0x0200, shift: 9  };  // Interrupt Enable
pub const FLAG_DF: Flag = Flag { mask: 0x0400, shift: 10 };  // Direction
pub const FLAG_OF: Flag = Flag { mask: 0x0800, shift: 11 };  // Overflow

impl Machine {
  pub fn flag_read(&self, f: Flag) -> bool {
    let cur = self.reg_read_u16(FLAGS);
    (cur & f.mask) != 0
  }

  pub fn flag_write(&mut self, f: Flag, set: bool) {
    let cur = self.reg_read_u16(FLAGS);
    let new = (cur & !f.mask) | ((set as u16) << f.shift);
    self.reg_write_u16(FLAGS, new);
  }

  // Overflow flag
  // -------------------------------------------------
  // OF is set when the operands' sign_u8s implied one result sign_u8, but the
  // actual result has the opposite sign_u8:
  //   positive - negative = negative?  -> OF=1 (should have been positive)
  //   negative - positive = positive?  -> OF=1 (should have been negative)

  pub fn flag_update_cmp(&mut self, lhs: Value, rhs: Value) {
    match (lhs, rhs) {
      (Value::U8(lhs),  Value::U8(rhs))  => self.flag_update_cmp8(lhs, rhs),
      (Value::U16(lhs), Value::U16(rhs)) => self.flag_update_cmp16(lhs, rhs),
      _ => panic!("Mismatched sizes"),
    }
  }

  pub fn flag_update_cmp8(&mut self, lhs: u8, rhs: u8) {
    let result = lhs.wrapping_sub(rhs);
    self.flag_write(FLAG_CF, lhs < rhs);
    self.flag_write(FLAG_ZF, result == 0);
    self.flag_write(FLAG_SF, (result & 0x80) != 0);
    self.flag_write(FLAG_OF, ((lhs ^ rhs) & (lhs ^ result) & 0x80) != 0);
    self.flag_write(FLAG_PF, result.count_ones() % 2 == 0);
    self.flag_write(FLAG_AF, (lhs & 0x0F) < (rhs & 0x0F));
  }

  pub fn flag_update_cmp16(&mut self, lhs: u16, rhs: u16) {
    let result = lhs.wrapping_sub(rhs);
    self.flag_write(FLAG_CF, lhs < rhs);
    self.flag_write(FLAG_ZF, result == 0);
    self.flag_write(FLAG_SF, (result & 0x8000) != 0);
    self.flag_write(FLAG_OF, ((lhs ^ rhs) & (lhs ^ result) & 0x8000) != 0);
    self.flag_write(FLAG_PF, (result as u8).count_ones() % 2 == 0); // PF uses low byte only
    self.flag_write(FLAG_AF, (lhs & 0x0F) < (rhs & 0x0F));
  }

  pub fn flag_update_bitwise(&mut self, result: Value) {
    match result {
      Value::U8(result)  => self.flag_update_bitwise8(result),
      Value::U16(result) => self.flag_update_bitwise16(result),
      _ => panic!("Mismatched sizes"),
    }
  }

  pub fn flag_update_bitwise8(&mut self, result: u8) {
    self.flag_write(FLAG_CF, false);
    self.flag_write(FLAG_ZF, result == 0);
    self.flag_write(FLAG_SF, (result & 0x80) != 0);
    self.flag_write(FLAG_OF, false);
    self.flag_write(FLAG_PF, result.count_ones() % 2 == 0);
  }

  pub fn flag_update_bitwise16(&mut self, result: u16) {
    self.flag_write(FLAG_CF, false);
    self.flag_write(FLAG_ZF, result == 0);
    self.flag_write(FLAG_SF, (result & 0x8000) != 0);
    self.flag_write(FLAG_OF, false);
    self.flag_write(FLAG_PF, (result as u8).count_ones() % 2 == 0); // PF uses low byte only
  }

  pub fn flag_update_add(&mut self, lhs: Value, rhs: Value) {
    match (lhs, rhs) {
      (Value::U8(lhs), Value::U8(rhs))   => self.flag_update_add8(lhs, rhs),
      (Value::U16(lhs), Value::U16(rhs)) => self.flag_update_add16(lhs, rhs),
      _ => panic!("Mismatched sizes"),
    }
  }

  pub fn flag_update_add8 (&mut self, lhs: u8,  rhs: u8)  { self._impl_flag_update_add8 (lhs, rhs, true); }
  pub fn flag_update_add16(&mut self, lhs: u16, rhs: u16) { self._impl_flag_update_add16(lhs, rhs, true); }

  pub fn flag_update_inc(&mut self, lhs: Value) {
    match lhs {
      Value::U8(lhs)  => self.flag_update_inc8(lhs),
      Value::U16(lhs) => self.flag_update_inc16(lhs),
      _ => panic!("Mismatched sizes"),
    }
  }

  pub fn flag_update_inc8 (&mut self, lhs: u8)  { self._impl_flag_update_add8 (lhs, 1, false); }
  pub fn flag_update_inc16(&mut self, lhs: u16) { self._impl_flag_update_add16(lhs, 1, false); }

  fn _impl_flag_update_add8(&mut self, lhs: u8, rhs: u8, update_cf: bool) {
    let result = lhs.wrapping_add(rhs);
    let wide   = (lhs as u16) + (rhs as u16);
    self.flag_write(FLAG_CF, wide > 0xFF);
    self.flag_write(FLAG_ZF, result == 0);
    self.flag_write(FLAG_SF, (result & 0x80) != 0);
    self.flag_write(FLAG_OF, ((lhs ^ result) & (rhs ^ result) & 0x80) != 0);
    self.flag_write(FLAG_PF, result.count_ones() % 2 == 0);
    self.flag_write(FLAG_AF, (lhs & 0x0F) + (rhs & 0x0F) > 0x0F);
  }

  fn _impl_flag_update_add16(&mut self, lhs: u16, rhs: u16, update_cf: bool) {
    let result = lhs.wrapping_add(rhs);
    let wide   = (lhs as u32) + (rhs as u32);
    if update_cf { self.flag_write(FLAG_CF, wide > 0xFFFF) };
    self.flag_write(FLAG_ZF, result == 0);
    self.flag_write(FLAG_SF, (result & 0x8000) != 0);
    self.flag_write(FLAG_OF, ((lhs ^ result) & (rhs ^ result) & 0x8000) != 0);
    self.flag_write(FLAG_PF, (result as u8).count_ones() % 2 == 0);  // PF uses low byte only
    self.flag_write(FLAG_AF, (lhs & 0x0F) + (rhs & 0x0F) > 0x0F);
  }

  // TODO FLAG UPDATE FOR SHIFT/ROTATE
}

#[cfg(test)]
mod tests {
  use super::*;

  macro_rules! check_cmp8 {
    ($lhs:expr, $rhs:expr, cf=$cf:expr, zf=$zf:expr, sf=$sf:expr, of=$of:expr, pf=$pf:expr, af=$af:expr) => {
      let mut m = Machine::default();
      m.flag_update_cmp8($lhs, $rhs);
      assert_eq!(m.flag_read(FLAG_CF), $cf != 0, "CF mismatch");
      assert_eq!(m.flag_read(FLAG_ZF), $zf != 0, "ZF mismatch");
      assert_eq!(m.flag_read(FLAG_SF), $sf != 0, "SF mismatch");
      assert_eq!(m.flag_read(FLAG_OF), $of != 0, "OF mismatch");
      assert_eq!(m.flag_read(FLAG_PF), $pf != 0, "PF mismatch");
      assert_eq!(m.flag_read(FLAG_AF), $af != 0, "AF mismatch");
    };
  }

  macro_rules! check_cmp16 {
    ($lhs:expr, $rhs:expr, cf=$cf:expr, zf=$zf:expr, sf=$sf:expr, of=$of:expr, pf=$pf:expr, af=$af:expr) => {
      let mut m = Machine::default();
      m.flag_update_cmp16($lhs, $rhs);
      assert_eq!(m.flag_read(FLAG_CF), $cf != 0, "CF mismatch");
      assert_eq!(m.flag_read(FLAG_ZF), $zf != 0, "ZF mismatch");
      assert_eq!(m.flag_read(FLAG_SF), $sf != 0, "SF mismatch");
      assert_eq!(m.flag_read(FLAG_OF), $of != 0, "OF mismatch");
      assert_eq!(m.flag_read(FLAG_PF), $pf != 0, "PF mismatch");
      assert_eq!(m.flag_read(FLAG_AF), $af != 0, "AF mismatch");
    };
  }

  macro_rules! check_bitwise8 {
    ($lhs:expr, $rhs:expr, zf=$zf:expr, sf=$sf:expr, pf=$pf:expr) => {
      let mut m = Machine::default();
      m.flag_update_bitwise8($lhs & $rhs);
      assert_eq!(m.flag_read(FLAG_ZF), $zf != 0, "ZF mismatch");
      assert_eq!(m.flag_read(FLAG_SF), $sf != 0, "SF mismatch");
      assert_eq!(m.flag_read(FLAG_PF), $pf != 0, "PF mismatch");
    };
  }

  macro_rules! check_bitwise16{
    ($lhs:expr, $rhs:expr, zf=$zf:expr, sf=$sf:expr, pf=$pf:expr) => {
      let mut m = Machine::default();
      m.flag_update_bitwise16($lhs & $rhs);
      assert_eq!(m.flag_read(FLAG_ZF), $zf != 0, "ZF mismatch");
      assert_eq!(m.flag_read(FLAG_SF), $sf != 0, "SF mismatch");
      assert_eq!(m.flag_read(FLAG_PF), $pf != 0, "PF mismatch");
    };
  }

  macro_rules! check_add8 {
    ($lhs:expr, $rhs:expr, cf=$cf:expr, zf=$zf:expr, sf=$sf:expr, of=$of:expr, pf=$pf:expr, af=$af:expr) => {{
      let mut m = Machine::default();
      m.flag_update_add8($lhs, $rhs);
      assert_eq!(m.flag_read(FLAG_CF), $cf != 0, "CF mismatch");
      assert_eq!(m.flag_read(FLAG_ZF), $zf != 0, "ZF mismatch");
      assert_eq!(m.flag_read(FLAG_SF), $sf != 0, "SF mismatch");
      assert_eq!(m.flag_read(FLAG_OF), $of != 0, "OF mismatch");
      assert_eq!(m.flag_read(FLAG_PF), $pf != 0, "PF mismatch");
      assert_eq!(m.flag_read(FLAG_AF), $af != 0, "AF mismatch");
    }};
  }

  macro_rules! check_add16 {
    ($lhs:expr, $rhs:expr, cf=$cf:expr, zf=$zf:expr, sf=$sf:expr, of=$of:expr, pf=$pf:expr, af=$af:expr) => {{
      let mut m = Machine::default();
      m.flag_update_add16($lhs, $rhs);
      assert_eq!(m.flag_read(FLAG_CF), $cf != 0, "CF mismatch");
      assert_eq!(m.flag_read(FLAG_ZF), $zf != 0, "ZF mismatch");
      assert_eq!(m.flag_read(FLAG_SF), $sf != 0, "SF mismatch");
      assert_eq!(m.flag_read(FLAG_OF), $of != 0, "OF mismatch");
      assert_eq!(m.flag_read(FLAG_PF), $pf != 0, "PF mismatch");
      assert_eq!(m.flag_read(FLAG_AF), $af != 0, "AF mismatch");
    }};
  }

  // --- cmp8 ---

  #[test]
  fn cmp8_equal() {
    // 5 - 5 = 0: ZF set, nothing else
    check_cmp8!(0x05, 0x05, cf=0, zf=1, sf=0, of=0, pf=1, af=0);
  }

  #[test]
  fn cmp8_dst_greater() {
    // 7 - 3 = 4: no flags
    check_cmp8!(0x07, 0x03, cf=0, zf=0, sf=0, of=0, pf=0, af=0);
  }

  #[test]
  fn cmp8_dst_less_unsigned() {
    // 3 - 5 = 0xFE: CF set (borrow), SF set, 0xFE = 1111_1110 (odd parity)
    check_cmp8!(0x03, 0x05, cf=1, zf=0, sf=1, of=0, pf=0, af=1);
  }

  #[test]
  fn cmp8_signed_overflow_positive() {
    // 0x7F (127) - 0xFF (-1) = 0x80 (-128): OF set (127-(-1)=128 overflows i8)
    check_cmp8!(0x7F, 0xFF, cf=1, zf=0, sf=1, of=1, pf=0, af=0);
  }

  #[test]
  fn cmp8_signed_overflow_negative() {
    // 0x80 (-128) - 0x01 (1) = 0x7F (127): OF set (-128-1=-129 overflows i8)
    check_cmp8!(0x80, 0x01, cf=0, zf=0, sf=0, of=1, pf=0, af=1);
  }

  #[test]
  fn cmp8_zero_result_max() {
    // 0xFF - 0xFF = 0: ZF set, parity even
    check_cmp8!(0xFF, 0xFF, cf=0, zf=1, sf=0, of=0, pf=1, af=0);
  }

  #[test]
  fn cmp8_auxiliary_carry() {
    // 0x10 - 0x01 = 0x0F: AF set (low nibble 0 < 1, borrow from bit 4)
    check_cmp8!(0x10, 0x01, cf=0, zf=0, sf=0, of=0, pf=1, af=1);
  }

  #[test]
  fn cmp8_parity_odd() {
    // 0x09 - 0x02 = 0x07 = 0000_0111: 3 ones = odd parity, PF=0
    check_cmp8!(0x09, 0x02, cf=0, zf=0, sf=0, of=0, pf=0, af=0);
  }

  #[test]
  fn cmp8_zero_minus_one() {
    // 0x00 - 0x01 = 0xFF: CF set, SF set, 0xFF = 8 ones = even parity
    check_cmp8!(0x00, 0x01, cf=1, zf=0, sf=1, of=0, pf=1, af=1);
  }

  // --- cmp16 ---

  #[test]
  fn cmp16_equal() {
    // 0x1234 - 0x1234 = 0: ZF set
    check_cmp16!(0x1234, 0x1234, cf=0, zf=1, sf=0, of=0, pf=1, af=0);
  }

  #[test]
  fn cmp16_dst_greater() {
    // 0x0100 - 0x0001 = 0x00FF: SF clear, PF: 0xFF = 8 ones = even
    check_cmp16!(0x0100, 0x0001, cf=0, zf=0, sf=0, of=0, pf=1, af=1);
  }

  #[test]
  fn cmp16_dst_less_unsigned() {
    // 0x0001 - 0x0002 = 0xFFFF: CF set, SF set, low byte 0xFF = even parity
    check_cmp16!(0x0001, 0x0002, cf=1, zf=0, sf=1, of=0, pf=1, af=1);
  }

  #[test]
  fn cmp16_signed_overflow_positive() {
    // 0x7FFF (32767) - 0xFFFF (-1) = 0x8000: OF set, SF set, PF set (lower byte only used)
    check_cmp16!(0x7FFF, 0xFFFF, cf=1, zf=0, sf=1, of=1, pf=1, af=0);
  }

  #[test]
  fn cmp16_signed_overflow_negative() {
    // 0x8000 (-32768) - 0x0001 (1) = 0x7FFF: OF set, SF clear
    check_cmp16!(0x8000, 0x0001, cf=0, zf=0, sf=0, of=1, pf=1, af=1);
  }

  #[test]
  fn cmp16_zero_result_max() {
    // 0xFFFF - 0xFFFF = 0: ZF set, parity even
    check_cmp16!(0xFFFF, 0xFFFF, cf=0, zf=1, sf=0, of=0, pf=1, af=0);
  }

  #[test]
  fn cmp16_parity_uses_low_byte_only() {
    // 0x0103 - 0x0001 = 0x0102: low byte 0x02 = 0000_0010 = 1 one = odd parity
    check_cmp16!(0x0103, 0x0001, cf=0, zf=0, sf=0, of=0, pf=0, af=0);
  }

  #[test]
  fn cmp16_zero_minus_one() {
    // 0x0000 - 0x0001 = 0xFFFF: CF set, SF set, low byte 0xFF = even parity
    check_cmp16!(0x0000, 0x0001, cf=1, zf=0, sf=1, of=0, pf=1, af=1);
  }

  // --- bitwise8 ---

  #[test]
  fn bitwise8_zero_and_zero() {
    // 0x00 & 0x00 = 0x00: ZF set
    check_bitwise8!(0x00, 0x00, zf=1, sf=0, pf=1);
  }

  #[test]
  fn bitwise8_no_overlap() {
    // 0xF0 & 0x0F = 0x00: bits don't overlap, ZF set
    check_bitwise8!(0xF0, 0x0F, zf=1, sf=0, pf=1);
  }

  #[test]
  fn bitwise8_alternating_no_overlap() {
    // 0x55 & 0xAA = 0x00: alternating bits, no overlap
    check_bitwise8!(0x55, 0xAA, zf=1, sf=0, pf=1);
  }

  #[test]
  fn bitwise8_all_ones() {
    // 0xFF & 0xFF = 0xFF: SF set, 8 ones = even parity
    check_bitwise8!(0xFF, 0xFF, zf=0, sf=1, pf=1);
  }

  #[test]
  fn bitwise8_high_bit_only() {
    // 0x80 & 0x80 = 0x80: SF set, 1 one = odd parity
    check_bitwise8!(0x80, 0x80, zf=0, sf=1, pf=0);
  }

  #[test]
  fn bitwise8_low_bit_only() {
    // 0x01 & 0x01 = 0x01: SF clear, 1 one = odd parity
    check_bitwise8!(0x01, 0x01, zf=0, sf=0, pf=0);
  }

  #[test]
  fn bitwise8_parity_even() {
    // 0x33 & 0xFF = 0x33 = 0011_0011: 4 ones = even parity
    check_bitwise8!(0x33, 0xFF, zf=0, sf=0, pf=1);
  }

  #[test]
  fn bitwise8_parity_odd() {
    // 0x07 & 0xFF = 0x07 = 0000_0111: 3 ones = odd parity
    check_bitwise8!(0x07, 0xFF, zf=0, sf=0, pf=0);
  }

  #[test]
  fn bitwise8_mask_low_nibble() {
    // 0xAB & 0x0F = 0x0B = 0000_1011: 3 ones = odd parity, SF clear
    check_bitwise8!(0xAB, 0x0F, zf=0, sf=0, pf=0);
  }

  #[test]
  fn bitwise8_mask_high_bit() {
    // common idiom: test if sign bit set
    // 0x81 & 0x80 = 0x80: SF set
    check_bitwise8!(0x81, 0x80, zf=0, sf=1, pf=0);
  }

  // --- bitwise16 ---

  #[test]
  fn bitwise16_zero() {
    // 0x0000 & 0x0000 = 0x0000: ZF set
    check_bitwise16!(0x0000, 0x0000, zf=1, sf=0, pf=1);
  }

  #[test]
  fn bitwise16_no_overlap() {
    // 0xFF00 & 0x00FF = 0x0000: ZF set
    check_bitwise16!(0xFF00, 0x00FF, zf=1, sf=0, pf=1);
  }

  #[test]
  fn bitwise16_all_ones() {
    // 0xFFFF & 0xFFFF = 0xFFFF: SF set, low byte 0xFF = 8 ones = even parity
    check_bitwise16!(0xFFFF, 0xFFFF, zf=0, sf=1, pf=1);
  }

  #[test]
  fn bitwise16_high_bit_only() {
    // 0x8000 & 0x8000 = 0x8000: SF set, low byte 0x00 = 0 ones = even parity
    check_bitwise16!(0x8000, 0x8000, zf=0, sf=1, pf=1);
  }

  #[test]
  fn bitwise16_low_bit_only() {
    // 0x0001 & 0x0001 = 0x0001: SF clear, low byte 0x01 = 1 one = odd parity
    check_bitwise16!(0x0001, 0x0001, zf=0, sf=0, pf=0);
  }

  #[test]
  fn bitwise16_parity_uses_low_byte_only() {
    // 0xFF03 & 0xFFFF = 0xFF03: low byte 0x03 = 0000_0011 = 2 ones = even parity
    // high byte 0xFF ignored for parity
    check_bitwise16!(0xFF03, 0xFFFF, zf=0, sf=1, pf=1);
  }

  #[test]
  fn bitwise16_parity_odd_low_byte() {
    // 0x0102 & 0xFFFF = 0x0102: low byte 0x02 = 0000_0010 = 1 one = odd parity
    check_bitwise16!(0x0102, 0xFFFF, zf=0, sf=0, pf=0);
  }

  #[test]
  fn bitwise16_mask_high_byte() {
    // 0x1234 & 0xFF00 = 0x1200: SF clear, low byte 0x00 = even parity
    check_bitwise16!(0x1234, 0xFF00, zf=0, sf=0, pf=1);
  }

  #[test]
  fn bitwise16_mask_sign_bit() {
    // common idiom: test if sign bit set
    // 0x8001 & 0x8000 = 0x8000: SF set
    check_bitwise16!(0x8001, 0x8000, zf=0, sf=1, pf=1);
  }

  // --- add8 ---

  #[test]
  fn add8_zero_plus_zero() {
    // 0 + 0 = 0: ZF set, parity even
    check_add8!(0x00, 0x00, cf=0, zf=1, sf=0, of=0, pf=1, af=0);
  }

  #[test]
  fn add8_no_flags() {
    // 1 + 1 = 2: no flags
    check_add8!(0x01, 0x01, cf=0, zf=0, sf=0, of=0, pf=0, af=0);
  }

  #[test]
  fn add8_signed_overflow_positive() {
    // 0x7F + 0x01 = 0x80: OF set (127+1=128 overflows i8), SF set
    // 0x80 = 1000_0000: 1 one = odd parity
    check_add8!(0x7F, 0x01, cf=0, zf=0, sf=1, of=1, pf=0, af=1);
  }

  #[test]
  fn add8_carry_and_zero() {
    // 0xFF + 0x01 = 0x00: CF set, ZF set, OF=0 (-1+1=0, no signed overflow)
    check_add8!(0xFF, 0x01, cf=1, zf=1, sf=0, of=0, pf=1, af=1);
  }

  #[test]
  fn add8_carry_and_signed_overflow() {
    // 0x80 + 0x80 = 0x00: CF set, ZF set, OF set (-128+-128=-256 overflows i8)
    check_add8!(0x80, 0x80, cf=1, zf=1, sf=0, of=1, pf=1, af=0);
  }

  #[test]
  fn add8_auxiliary_carry() {
    // 0x0F + 0x01 = 0x10: AF set (low nibble carries into high)
    // 0x10 = 0001_0000: 1 one = odd parity
    check_add8!(0x0F, 0x01, cf=0, zf=0, sf=0, of=0, pf=0, af=1);
  }

  #[test]
  fn add8_signed_overflow_both_positive() {
    // 0x55 + 0x55 = 0xAA: OF set (85+85=170 overflows i8), SF set
    // 0xAA = 1010_1010: 4 ones = even parity
    check_add8!(0x55, 0x55, cf=0, zf=0, sf=1, of=1, pf=1, af=0);
  }

  #[test]
  fn add8_carry_no_signed_overflow() {
    // 0xFF + 0xFF = 0xFE: CF set, no OF (-1+-1=-2, fits in i8)
    // 0xFE = 1111_1110: 7 ones = odd parity
    check_add8!(0xFF, 0xFF, cf=1, zf=0, sf=1, of=0, pf=0, af=1);
  }

  // --- add16 ---

  #[test]
  fn add16_zero_plus_zero() {
    check_add16!(0x0000, 0x0000, cf=0, zf=1, sf=0, of=0, pf=1, af=0);
  }

  #[test]
  fn add16_signed_overflow_positive() {
    // 0x7FFF + 0x0001 = 0x8000: OF set (32767+1 overflows i16), SF set
    // low byte 0x00 = even parity
    check_add16!(0x7FFF, 0x0001, cf=0, zf=0, sf=1, of=1, pf=1, af=1);
  }

  #[test]
  fn add16_carry_and_zero() {
    // 0xFFFF + 0x0001 = 0x0000: CF set, ZF set, OF=0 (-1+1=0)
    check_add16!(0xFFFF, 0x0001, cf=1, zf=1, sf=0, of=0, pf=1, af=1);
  }

  #[test]
  fn add16_carry_and_signed_overflow() {
    // 0x8000 + 0x8000 = 0x0000: CF set, ZF set, OF set (-32768+-32768 overflows)
    check_add16!(0x8000, 0x8000, cf=1, zf=1, sf=0, of=1, pf=1, af=0);
  }

  #[test]
  fn add16_carry_no_signed_overflow() {
    // 0xFFFF + 0xFFFF = 0xFFFE: CF set, SF set, OF=0 (-1+-1=-2, fits in i16)
    // low byte 0xFE = 1111_1110: 7 ones = odd parity
    check_add16!(0xFFFF, 0xFFFF, cf=1, zf=0, sf=1, of=0, pf=0, af=1);
  }

  #[test]
  fn add16_parity_from_low_byte_only() {
    // 0x0100 + 0x0003 = 0x0103: low byte 0x03 = 0000_0011 = 2 ones = even parity
    check_add16!(0x0100, 0x0003, cf=0, zf=0, sf=0, of=0, pf=1, af=0);
  }

  #[test]
  fn add16_auxiliary_carry() {
    // 0x000F + 0x0001 = 0x0010: AF set
    // low byte 0x10 = 0001_0000: 1 one = odd parity
    check_add16!(0x000F, 0x0001, cf=0, zf=0, sf=0, of=0, pf=0, af=1);
  }

  #[test]
  fn add16_signed_overflow_both_negative() {
    // 0x8001 + 0x8001 = 0x0002: CF set, OF set (-32767+-32767 overflows)
    check_add16!(0x8001, 0x8001, cf=1, zf=0, sf=0, of=1, pf=0, af=0);
  }
}
