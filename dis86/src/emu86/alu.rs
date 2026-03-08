use super::machine::*;

// goal:
//   take two values (generic)
//   perform some operation
//   update flags

pub enum BinaryOp {
  Add,
  Adc,
  Sub,
  Sbb,
  And,
  Or,
  Xor,
}

pub enum MultiplyOp {
  Unsigned,
  Signed,
}

pub enum UnaryOp {
  Neg,
  Inc,
  Dec,
  Not,
}

pub enum ShiftOp {
  Shl,
  Shr,
  Sar,
  Rol,
}

fn flag_generic_sf(r: u16, sign_mask: u16)  -> bool { (r & sign_mask) != 0 }
fn flag_generic_zf(r: u16, value_mask: u16) -> bool { (r & value_mask) == 0 }
fn flag_generic_pf(r: u16)                  -> bool { (r as u8).count_ones() % 2 == 0 } // PF uses low byte only

fn update_flags_sub(f: &mut Flags, a: u16, b: u16, carry_in: u16, r: u16, sign_mask: u16, value_mask: u16, update_cf: bool) {
  assert!(carry_in <= 1);
  let cf = (a as u32) < (b as u32) + (carry_in as u32);

  if update_cf { f.set(FLAG_CF, cf) };
  f.set(FLAG_ZF, flag_generic_zf(r, value_mask));
  f.set(FLAG_SF, flag_generic_sf(r, sign_mask));
  f.set(FLAG_PF, flag_generic_pf(r));
  f.set(FLAG_AF, ((a as u32) & 0x0F) < ((b as u32) & 0x0F) + (carry_in as u32));

  // Overflow cases
  // -------------------------------------------------
  //   positive - negative - cf = negative?  -> OF=1 (should have been positive)
  //   negative - positive - cf = positive?  -> OF=1 (should have been negative)
  f.set(FLAG_OF, ((a ^ b) & (a ^ r) & sign_mask) != 0);
}

fn update_flags_bitwise(f: &mut Flags, r: u16, sign_mask: u16, value_mask: u16) {
  f.set(FLAG_CF, false);
  f.set(FLAG_ZF, flag_generic_zf(r, value_mask));
  f.set(FLAG_SF, flag_generic_sf(r, sign_mask));
  f.set(FLAG_OF, false);
  f.set(FLAG_PF, flag_generic_pf(r));
}

fn update_flags_add(f: &mut Flags, a: u16, b: u16, carry_in: u16, r32: u32, sign_mask: u16, value_mask: u16, update_cf: bool) {
  assert!(carry_in <= 1);
  let r = r32 as u16;
  let cf = ((r32 >> 1) & (sign_mask as u32)) != 0;

  if update_cf { f.set(FLAG_CF, cf) };
  f.set(FLAG_ZF, flag_generic_zf(r, value_mask));
  f.set(FLAG_SF, flag_generic_sf(r, sign_mask));
  f.set(FLAG_PF, flag_generic_pf(r));
  f.set(FLAG_AF, (a & 0x0F) + (b & 0x0F) + carry_in > 0x0F);

  // Overflow cases
  // -------------------------------------------------
  //   positive + positive + cf = negative?  -> OF=1 (should have been positive)
  //   negative + negative + cf = positive?  -> OF=1 (should have been negative)
  //
  // NOTICE: carry flag is 0 or 1, so it cannot the result sign (we can ignore it)
  f.set(FLAG_OF, ((a ^ r) & (b ^ r) & sign_mask) != 0);
}

fn update_flags_shl(f: &mut Flags, _a: u16, n: u8, r32: u32, sign_mask: u16, value_mask: u16) {
  if n == 0 { return; } // No update to flags if no shift happens

  let r = r32 as u16;

  let old_sign = ((r32 >> 1) & (sign_mask as u32)) != 0;
  let new_sign = (r & sign_mask) != 0;

  f.set(FLAG_CF, old_sign);
  f.set(FLAG_ZF, flag_generic_zf(r, value_mask));
  f.set(FLAG_SF, flag_generic_sf(r, sign_mask));
  f.set(FLAG_OF, old_sign ^ new_sign);  // sign bit changed?
  f.set(FLAG_PF, flag_generic_pf(r));
  f.set(FLAG_AF, false);
}

fn update_flags_shr(f: &mut Flags, a: u16, n: u8, r: u16, sign_mask: u16, value_mask: u16) {
  if n == 0 { return; } // No update to flags if no shift happens

  let cf_bit = 1 << (n-1);
  let cf = (a & cf_bit) != 0;

  f.set(FLAG_CF, cf);
  f.set(FLAG_ZF, flag_generic_zf(r, value_mask));
  f.set(FLAG_SF, flag_generic_sf(r, sign_mask));
  // I think this flag is ignored? Hard to tell...
  //f.set(FLAG_OF, (a & sign_mask) != 0);
  f.set(FLAG_PF, flag_generic_pf(r));
  f.set(FLAG_AF, false);
}

// Returns (quotient, remainder, flags)
pub fn divmod(a: Value, b: Value, mut f: Flags) -> (Value, Value, Flags) {
  let Value::U32(a) = a else { panic!("expected u32 for lhs") };
  let Value::U16(b) = b else { panic!("expected u16 for rhs") };
  let b = b as u32;

  let quotient = a / b;
  let remainder = a % b;

  if quotient > 0xffff {
    panic!("Divide Error"); // What should be done about this??
  }

  // Mirroring the behaviour of dosbox-x
  f.set(FLAG_CF, (remainder&3) >= 1 && (remainder&3) <= 2);  // Set iff low 2 bits of remainder are 01 or 10 )
  f.set(FLAG_ZF, remainder == 0 && (quotient&1) != 0);       // Set iff remainder is zero AND quotient is odd
  f.set(FLAG_SF, false);
  f.set(FLAG_OF, false);
  f.set(FLAG_AF, false);

  // Set iff rem and quo have the same parity
  let rem_parity = remainder.count_ones() % 2 != 0;
  let quo_parity = quotient.count_ones() % 2 != 0;
  f.set(FLAG_PF, rem_parity == quo_parity);

  (Value::U16(quotient as u16), Value::U16(remainder as u16), f)
}

pub fn multiply(op: MultiplyOp, a: Value, b: Value, mut f: Flags) -> (Value, Flags) {
  // Unpack common case
  let (size, _sign_mask, value_mask, a, b) = match (a, b) {
    (Value::U8(a),  Value::U8(b))  => (1, 0x80,   0xff,   a as u16, b as u16),
    (Value::U16(a), Value::U16(b)) => (2, 0x8000, 0xffff, a, b),
    _ => panic!("Mismatched sizes"),
  };

  let result: u32;
  match op {
    MultiplyOp::Unsigned => {
      result = (a as u32) * (b as u32);

      let ovf = (result & (value_mask as u32)) != result;
      f.set(FLAG_CF, ovf);
      f.set(FLAG_OF, ovf);
      f.set(FLAG_ZF, result == 0);
    }
    MultiplyOp::Signed => {
      result = match size {
        1 => ((a as i8 as i16)  * (b as i8 as i16))  as u16 as u32,
        2 => ((a as i16 as i32) * (b as i16 as i32)) as u32,
        _ => unreachable!(),
      };

      let ovf = (result & (value_mask as u32)) != result;
      f.set(FLAG_CF, ovf);
      f.set(FLAG_OF, ovf);
      f.set(FLAG_ZF, result == 0);
    }
  }

  // Special re-pack because they return larger types
  let val = match size {
    1 => Value::U16(result as u16),
    2 => Value::U32(result),
    _ => unreachable!(),
  };

  (val, f)
}

pub fn binary(op: BinaryOp, a: Value, b: Value, mut f: Flags) -> (Value, Flags) {
  // Unpack common case
  let (size, sign_mask, value_mask, a, b) = match (a, b) {
    (Value::U8(a),  Value::U8(b))  => (1, 0x80,   0xff,   a as u16, b as u16),
    (Value::U16(a), Value::U16(b)) => (2, 0x8000, 0xffff, a, b),
    _ => panic!("Mismatched sizes"),
  };

  let result;
  match op {
    BinaryOp::Add => {
      let r32 = (a as u32) + (b as u32);
      result = r32 as u16;
      update_flags_add(&mut f, a, b, 0, r32, sign_mask, value_mask, true);
    }
    BinaryOp::Adc => {
      let carry_in = f.get(FLAG_CF) as u16;
      let r32 = (a as u32) + (b as u32) + (carry_in as u32);
      result = r32 as u16;
      update_flags_add(&mut f, a, b, carry_in, r32, sign_mask, value_mask, true);
    }
    BinaryOp::Sub => {
      result = a.wrapping_sub(b);
      update_flags_sub(&mut f, a, b, 0, result, sign_mask, value_mask, true);
    }
    BinaryOp::Sbb => {
      let carry_in = f.get(FLAG_CF) as u16;
      result = a.wrapping_sub(b).wrapping_sub(carry_in);
      update_flags_sub(&mut f, a, b, carry_in, result, sign_mask, value_mask, true);
    }
    BinaryOp::And => {
      result = a & b;
      update_flags_bitwise(&mut f, result, sign_mask, value_mask);
    }
    BinaryOp::Or => {
      result = a | b;
      update_flags_bitwise(&mut f, result, sign_mask, value_mask);
    }
    BinaryOp::Xor => {
      result = a ^ b;
      update_flags_bitwise(&mut f, result, sign_mask, value_mask);
    }
  };

  // Re-pack
  let result_value = match size {
    1 => Value::U8(result as u8),
    2 => Value::U16(result),
    _ => unreachable!(),
  };

  (result_value, f)
}

pub fn unary(op: UnaryOp, a: Value, mut f: Flags) -> (Value, Flags) {
  // Unpack
  let (size, sign_mask, value_mask, a) = match a {
    Value::U8(a)  => (1, 0x80,   0xff,   a as u16),
    Value::U16(a) => (2, 0x8000, 0xffff, a),
    _ => panic!("Mismatched sizes"),
  };

  let result;
  match op {
    UnaryOp::Neg => {
      result = -(a as i16) as u16;
      update_flags_sub(&mut f, 0, a, 0, result, sign_mask, value_mask, true);
    }
    UnaryOp::Inc => {
      let r32 = (a as u32) + (1 as u32);
      result = r32 as u16;
      update_flags_add(&mut f, a, 1, 0, r32, sign_mask, value_mask, false);
    }
    UnaryOp::Dec => {
      result = a.wrapping_sub(1);
      update_flags_sub(&mut f, a, 1, 0, result, sign_mask, value_mask, false);
    }
    UnaryOp::Not => {
      result = !a;
    }
  };


  // Re-pack
  let result_value = match size {
    1 => Value::U8(result as u8),
    2 => Value::U16(result),
    _ => unreachable!(),
  };

  (result_value, f)
}

pub fn shift(op: ShiftOp, a: Value, n: u8, mut f: Flags) -> (Value, Flags) {
  // Unpack
  let (size, sign_mask, value_mask, a) = match a {
    Value::U8(a)  => (1, 0x80,   0xff,   a as u16),
    Value::U16(a) => (2, 0x8000, 0xffff, a),
    _ => panic!("Mismatched sizes"),
  };

  let result;
  match op {
    ShiftOp::Shl => {
      let r32 = if n < 32 {
        (a as u32).wrapping_shl(n as u32)
      } else {
        0
      };
      result = r32 as u16;
      update_flags_shl(&mut f, a, n, r32, sign_mask, value_mask);
    }
    ShiftOp::Shr => {
      let r32 = if n < 16 {
        (a as u32).wrapping_shr(n as u32)
      } else {
        0
      };
      result = r32 as u16;
      update_flags_shr(&mut f, a, n, result, sign_mask, value_mask);
    }
    ShiftOp::Sar => {
      result = match size {
        1 => (a as i8).wrapping_shr(n as u32) as u8 as u16,
        2 => (a as i16).wrapping_shr(n as u32) as u16,
        _ => unreachable!(),
      };
      update_flags_shr(&mut f, a, n, result, sign_mask, value_mask);
    }
    ShiftOp::Rol => {
      result = match size {
        1 => (a as u8).rotate_left(n as u32) as u16,
        2 => a.rotate_left(n as u32),
        _ => unreachable!(),
      };
      // TODO SET FLAGS ?
    }
  };


  // Re-pack
  let result_value = match size {
    1 => Value::U8(result as u8),
    2 => Value::U16(result),
    _ => unreachable!(),
  };

  (result_value, f)
}
