use super::machine::*;

// goal:
//   take two values (generic)
//   perform some operation
//   update flags

pub enum BinaryOp {
  // Add,
  Sub,
  And,
  Or,
  Xor,
}

pub enum UnaryOp {
  Neg,
  //Inc,
  //Not,
}

// enum ShiftOp {
//   Shl,
//   Shr,
//   Sar,
// }

fn flag_generic_af(a: u16, b: u16)          -> bool { (a & 0x0F) < (b & 0x0F) }
fn flag_generic_sf(r: u16, sign_mask: u16) -> bool { (r & sign_mask) != 0 }
fn flag_generic_zf(r: u16)                 -> bool { r == 0 }
fn flag_generic_pf(r: u16)                 -> bool { (r as u8).count_ones() % 2 == 0 } // PF uses low byte only

fn update_flags_sub(f: &mut Flags, a: u16, b: u16, r: u16, sign_mask: u16) {
  f.set(FLAG_CF, a < b);
  f.set(FLAG_ZF, flag_generic_zf(r));
  f.set(FLAG_SF, flag_generic_sf(r, sign_mask));
  f.set(FLAG_PF, flag_generic_pf(r));
  f.set(FLAG_AF, flag_generic_af(a, b));

  // Overflow cases
  // -------------------------------------------------
  //   positive - negative = negative?  -> OF=1 (should have been positive)
  //   negative - positive = positive?  -> OF=1 (should have been negative)
  f.set(FLAG_OF, ((a ^ b) & (a ^ r) & sign_mask) != 0);
}

fn update_flags_bitwise(f: &mut Flags, r: u16, sign_mask: u16) {
  f.set(FLAG_CF, false);
  f.set(FLAG_ZF, flag_generic_zf(r));
  f.set(FLAG_SF, flag_generic_sf(r, sign_mask));
  f.set(FLAG_OF, false);
  f.set(FLAG_PF, flag_generic_pf(r));
}

pub fn binary(op: BinaryOp, a: Value, b: Value, mut f: Flags) -> (Value, Flags) {
  // Unpack
  let (size, sign_mask, a, b) = match (a, b) {
    (Value::U8(a),  Value::U8(b))  => (1, 0x80,   a as u16, b as u16),
    (Value::U16(a), Value::U16(b)) => (2, 0x8000, a, b),
    _ => panic!("Mismatched sizes"),
  };

  let result;
  match op {
    // BinaryOp::Add => {
    //   result = a.wrapping_add(b);
    // }
    BinaryOp::Sub => {
      result = a.wrapping_sub(b);
      update_flags_sub(&mut f, a, b, result, sign_mask);
    }
    BinaryOp::And => {
      result = a & b;
      update_flags_bitwise(&mut f, result, sign_mask);
    }
    BinaryOp::Or => {
      result = a | b;
      update_flags_bitwise(&mut f, result, sign_mask);
    }
    BinaryOp::Xor => {
      result = a ^ b;
      update_flags_bitwise(&mut f, result, sign_mask);
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
  let (size, sign_mask, a) = match a {
    Value::U8(a)  => (1, 0x80,   a as u16),
    Value::U16(a) => (2, 0x8000, a),
    _ => panic!("Mismatched sizes"),
  };

  let result;
  match op {
    UnaryOp::Neg => {
      result = -(a as i16) as u16;
      update_flags_sub(&mut f, 0, a, result, sign_mask);
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

//fn alu_shift(op: BinaryOp, a: Value, n: u8, f: Flags) -> (Value, Flags);
