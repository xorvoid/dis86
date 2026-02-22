use super::value::Value;
use super::alu;
use super::cpu_flags::*;

macro_rules! check_sub8 {
  ($lhs:expr, $rhs:expr, cf=$cf:expr, zf=$zf:expr, sf=$sf:expr, of=$of:expr, pf=$pf:expr, af=$af:expr) => {
    let (_result, f) = alu::binary(alu::BinaryOp::Sub, Value::U8($lhs), Value::U8($rhs), Flags(0));
    assert_eq!(f.get(FLAG_CF), $cf != 0, "CF mismatch");
    assert_eq!(f.get(FLAG_ZF), $zf != 0, "ZF mismatch");
    assert_eq!(f.get(FLAG_SF), $sf != 0, "SF mismatch");
    assert_eq!(f.get(FLAG_OF), $of != 0, "OF mismatch");
    assert_eq!(f.get(FLAG_PF), $pf != 0, "PF mismatch");
    assert_eq!(f.get(FLAG_AF), $af != 0, "AF mismatch");
  };
}

macro_rules! check_sub16 {
  ($lhs:expr, $rhs:expr, cf=$cf:expr, zf=$zf:expr, sf=$sf:expr, of=$of:expr, pf=$pf:expr, af=$af:expr) => {
    let (_result, f) = alu::binary(alu::BinaryOp::Sub, Value::U16($lhs), Value::U16($rhs), Flags(0));
    assert_eq!(f.get(FLAG_CF), $cf != 0, "CF mismatch");
    assert_eq!(f.get(FLAG_ZF), $zf != 0, "ZF mismatch");
    assert_eq!(f.get(FLAG_SF), $sf != 0, "SF mismatch");
    assert_eq!(f.get(FLAG_OF), $of != 0, "OF mismatch");
    assert_eq!(f.get(FLAG_PF), $pf != 0, "PF mismatch");
    assert_eq!(f.get(FLAG_AF), $af != 0, "AF mismatch");
  };
}

macro_rules! check_bitwise8 {
  ($lhs:expr, $rhs:expr, zf=$zf:expr, sf=$sf:expr, pf=$pf:expr) => {
    let (_result, f) = alu::binary(alu::BinaryOp::And, Value::U8($lhs), Value::U8($rhs), Flags(0));
    assert_eq!(f.get(FLAG_ZF), $zf != 0, "ZF mismatch");
    assert_eq!(f.get(FLAG_SF), $sf != 0, "SF mismatch");
    assert_eq!(f.get(FLAG_PF), $pf != 0, "PF mismatch");
  };
}

macro_rules! check_bitwise16 {
  ($lhs:expr, $rhs:expr, zf=$zf:expr, sf=$sf:expr, pf=$pf:expr) => {
    let (_result, f) = alu::binary(alu::BinaryOp::And, Value::U16($lhs), Value::U16($rhs), Flags(0));
    assert_eq!(f.get(FLAG_ZF), $zf != 0, "ZF mismatch");
    assert_eq!(f.get(FLAG_SF), $sf != 0, "SF mismatch");
    assert_eq!(f.get(FLAG_PF), $pf != 0, "PF mismatch");
  };
}

macro_rules! check_add8 {
  ($lhs:expr, $rhs:expr, cf=$cf:expr, zf=$zf:expr, sf=$sf:expr, of=$of:expr, pf=$pf:expr, af=$af:expr) => {{
    let (_result, f) = alu::binary(alu::BinaryOp::Add, Value::U8($lhs), Value::U8($rhs), Flags(0));
    assert_eq!(f.get(FLAG_CF), $cf != 0, "CF mismatch");
    assert_eq!(f.get(FLAG_ZF), $zf != 0, "ZF mismatch");
    assert_eq!(f.get(FLAG_SF), $sf != 0, "SF mismatch");
    assert_eq!(f.get(FLAG_OF), $of != 0, "OF mismatch");
    assert_eq!(f.get(FLAG_PF), $pf != 0, "PF mismatch");
    assert_eq!(f.get(FLAG_AF), $af != 0, "AF mismatch");
  }};
}

macro_rules! check_add16 {
  ($lhs:expr, $rhs:expr, cf=$cf:expr, zf=$zf:expr, sf=$sf:expr, of=$of:expr, pf=$pf:expr, af=$af:expr) => {{
    let (_result, f) = alu::binary(alu::BinaryOp::Add, Value::U16($lhs), Value::U16($rhs), Flags(0));
    assert_eq!(f.get(FLAG_CF), $cf != 0, "CF mismatch");
    assert_eq!(f.get(FLAG_ZF), $zf != 0, "ZF mismatch");
    assert_eq!(f.get(FLAG_SF), $sf != 0, "SF mismatch");
    assert_eq!(f.get(FLAG_OF), $of != 0, "OF mismatch");
    assert_eq!(f.get(FLAG_PF), $pf != 0, "PF mismatch");
    assert_eq!(f.get(FLAG_AF), $af != 0, "AF mismatch");
  }};
}

macro_rules! check_shl8 {
  ($lhs:expr, $count:expr, cf=$cf:expr, zf=$zf:expr, sf=$sf:expr, of=$of:expr, pf=$pf:expr) => {{
    let (_result, f) = alu::shift(alu::ShiftOp::Shl, Value::U8($lhs), $count, Flags(0));
    assert_eq!(f.get(FLAG_CF), $cf != 0, "CF mismatch");
    assert_eq!(f.get(FLAG_ZF), $zf != 0, "ZF mismatch");
    assert_eq!(f.get(FLAG_SF), $sf != 0, "SF mismatch");
    assert_eq!(f.get(FLAG_OF), $of != 0, "OF mismatch");
    assert_eq!(f.get(FLAG_PF), $pf != 0, "PF mismatch");
  }};
}

macro_rules! check_shl16 {
  ($lhs:expr, $count:expr, cf=$cf:expr, zf=$zf:expr, sf=$sf:expr, of=$of:expr, pf=$pf:expr) => {{
    let (_result, f) = alu::shift(alu::ShiftOp::Shl, Value::U16($lhs), $count, Flags(0));
    assert_eq!(f.get(FLAG_CF), $cf != 0, "CF mismatch");
    assert_eq!(f.get(FLAG_ZF), $zf != 0, "ZF mismatch");
    assert_eq!(f.get(FLAG_SF), $sf != 0, "SF mismatch");
    assert_eq!(f.get(FLAG_OF), $of != 0, "OF mismatch");
    assert_eq!(f.get(FLAG_PF), $pf != 0, "PF mismatch");
  }};
}

// --- sub8 ---

#[test]
fn sub8_equal() {
  // 5 - 5 = 0: ZF set, nothing else
  check_sub8!(0x05, 0x05, cf=0, zf=1, sf=0, of=0, pf=1, af=0);
}

#[test]
fn sub8_dst_greater() {
  // 7 - 3 = 4: no flags
  check_sub8!(0x07, 0x03, cf=0, zf=0, sf=0, of=0, pf=0, af=0);
}

#[test]
fn sub8_dst_less_unsigned() {
  // 3 - 5 = 0xFE: CF set (borrow), SF set, 0xFE = 1111_1110 (odd parity)
  check_sub8!(0x03, 0x05, cf=1, zf=0, sf=1, of=0, pf=0, af=1);
}

#[test]
fn sub8_signed_overflow_positive() {
  // 0x7F (127) - 0xFF (-1) = 0x80 (-128): OF set (127-(-1)=128 overflows i8)
  check_sub8!(0x7F, 0xFF, cf=1, zf=0, sf=1, of=1, pf=0, af=0);
}

#[test]
fn sub8_signed_overflow_negative() {
  // 0x80 (-128) - 0x01 (1) = 0x7F (127): OF set (-128-1=-129 overflows i8)
  check_sub8!(0x80, 0x01, cf=0, zf=0, sf=0, of=1, pf=0, af=1);
}

#[test]
fn sub8_zero_result_max() {
  // 0xFF - 0xFF = 0: ZF set, parity even
  check_sub8!(0xFF, 0xFF, cf=0, zf=1, sf=0, of=0, pf=1, af=0);
}

#[test]
fn sub8_auxiliary_carry() {
  // 0x10 - 0x01 = 0x0F: AF set (low nibble 0 < 1, borrow from bit 4)
  check_sub8!(0x10, 0x01, cf=0, zf=0, sf=0, of=0, pf=1, af=1);
}

#[test]
fn sub8_parity_odd() {
  // 0x09 - 0x02 = 0x07 = 0000_0111: 3 ones = odd parity, PF=0
  check_sub8!(0x09, 0x02, cf=0, zf=0, sf=0, of=0, pf=0, af=0);
}

#[test]
fn sub8_zero_minus_one() {
  // 0x00 - 0x01 = 0xFF: CF set, SF set, 0xFF = 8 ones = even parity
  check_sub8!(0x00, 0x01, cf=1, zf=0, sf=1, of=0, pf=1, af=1);
}

// --- sub16 ---

#[test]
fn sub16_equal() {
  // 0x1234 - 0x1234 = 0: ZF set
  check_sub16!(0x1234, 0x1234, cf=0, zf=1, sf=0, of=0, pf=1, af=0);
}

#[test]
fn sub16_dst_greater() {
  // 0x0100 - 0x0001 = 0x00FF: SF clear, PF: 0xFF = 8 ones = even
  check_sub16!(0x0100, 0x0001, cf=0, zf=0, sf=0, of=0, pf=1, af=1);
}

#[test]
fn sub16_dst_less_unsigned() {
  // 0x0001 - 0x0002 = 0xFFFF: CF set, SF set, low byte 0xFF = even parity
  check_sub16!(0x0001, 0x0002, cf=1, zf=0, sf=1, of=0, pf=1, af=1);
}

#[test]
fn sub16_signed_overflow_positive() {
  // 0x7FFF (32767) - 0xFFFF (-1) = 0x8000: OF set, SF set, PF set (lower byte only used)
  check_sub16!(0x7FFF, 0xFFFF, cf=1, zf=0, sf=1, of=1, pf=1, af=0);
}

#[test]
fn sub16_signed_overflow_negative() {
  // 0x8000 (-32768) - 0x0001 (1) = 0x7FFF: OF set, SF clear
  check_sub16!(0x8000, 0x0001, cf=0, zf=0, sf=0, of=1, pf=1, af=1);
}

#[test]
fn sub16_zero_result_max() {
  // 0xFFFF - 0xFFFF = 0: ZF set, parity even
  check_sub16!(0xFFFF, 0xFFFF, cf=0, zf=1, sf=0, of=0, pf=1, af=0);
}

#[test]
fn sub16_parity_uses_low_byte_only() {
  // 0x0103 - 0x0001 = 0x0102: low byte 0x02 = 0000_0010 = 1 one = odd parity
  check_sub16!(0x0103, 0x0001, cf=0, zf=0, sf=0, of=0, pf=0, af=0);
}

#[test]
fn sub16_zero_minus_one() {
  // 0x0000 - 0x0001 = 0xFFFF: CF set, SF set, low byte 0xFF = even parity
  check_sub16!(0x0000, 0x0001, cf=1, zf=0, sf=1, of=0, pf=1, af=1);
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

// --- shl8 ---

#[test]
fn shl8_no_flags() {
  // 0x01 << 1 = 0x02: no flags
  check_shl8!(0x01, 1, cf=0, zf=0, sf=0, of=0, pf=0);
}

#[test]
fn shl8_cf_from_msb() {
  // 0x80 << 1 = 0x00: CF=1 (old MSB), ZF=1, OF=1 (sign changed 1->0)
  check_shl8!(0x80, 1, cf=1, zf=1, sf=0, of=1, pf=1);
}

#[test]
fn shl8_of_sign_changes_to_one() {
  // 0x40 << 1 = 0x80: CF=0, SF=1, OF=1 (sign changed 0->1)
  check_shl8!(0x40, 1, cf=0, zf=0, sf=1, of=1, pf=0);
}

#[test]
fn shl8_no_of_sign_unchanged() {
  // 0xFF << 1 = 0xFE: CF=1, SF=1, OF=0 (sign stayed 1)
  // 0xFE = 1111_1110: 7 ones = odd parity
  check_shl8!(0xFF, 1, cf=1, zf=0, sf=1, of=0, pf=0);
}

#[test]
fn shl8_count_4() {
  // 0x01 << 4 = 0x10: CF=0, OF undefined (false), 0x10 = 1 one = odd parity
  check_shl8!(0x01, 4, cf=0, zf=0, sf=0, of=0, pf=0);
}

#[test]
fn shl8_count_8_cf_from_bit0() {
  // 0x01 << 8 = 0x00: CF=1 (old bit 0), ZF=1
  check_shl8!(0x01, 8, cf=1, zf=1, sf=0, of=0, pf=1);
}

#[test]
fn shl8_count_9_cf_zero() {
  // count > 8: result=0, CF=0
  check_shl8!(0xFF, 9, cf=0, zf=1, sf=0, of=0, pf=1);
}

// --- shl8 ---
#[test]
fn shl16_no_flags() {
  // 0x0001 << 1 = 0x0002: no flags
  check_shl16!(0x0001, 1, cf=0, zf=0, sf=0, of=0, pf=0);
}

#[test]
fn shl16_cf_from_msb() {
  // 0x8000 << 1 = 0x0000: CF=1 (old MSB), ZF=1, OF=1 (sign changed 1->0)
  check_shl16!(0x8000, 1, cf=1, zf=1, sf=0, of=1, pf=1);
}

#[test]
fn shl16_of_sign_changes_to_one() {
  // 0x4000 << 1 = 0x8000: CF=0, SF=1, OF=1 (sign changed 0->1)
  check_shl16!(0x4000, 1, cf=0, zf=0, sf=1, of=1, pf=1);
}

#[test]
fn shl16_no_of_sign_unchanged() {
  // 0xFFFF << 1 = 0xFFFE: CF=1, SF=1, OF=0 (sign stayed 1)
  check_shl16!(0xFFFF, 1, cf=1, zf=0, sf=1, of=0, pf=0);
}

#[test]
fn shl16_count_4() {
  // 0x0100 << 4 = 0x1000: CF=0, OF undefined (false)
  check_shl16!(0x0100, 4, cf=0, zf=0, sf=0, of=0, pf=1);
}

#[test]
fn shl16_count_16_cf_from_bit0() {
  // 0x01 << 16 = 0x00: CF=1 (old bit 0), ZF=1
  check_shl16!(0x0001, 16, cf=1, zf=1, sf=0, of=0, pf=1);
}

#[test]
fn shl16_count_17_cf_zero() {
  // count > 16: result=0, CF=0
  check_shl16!(0xFFFF, 17, cf=0, zf=1, sf=0, of=0, pf=1);
}
