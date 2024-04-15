use crate::ir::def::*;
use crate::ir::sym;
use crate::types::Type;

pub fn fuse_adjacent_writevar16_to_writevar32(ir: &mut IR) {
  // FIXME: THIS FUNCTION IS MESSY... CAN WE MAKE IT CLEANER???
  for b in ir.iter_blocks() {
    for r in ir.iter_instrs(b) {

      // Find low write16: E.g. 'writevar16 _local_0028          dx.3' where _local_0028 is u32
      let low_ref = r;
      let low_instr = ir.instr(low_ref).unwrap();
      if low_instr.opcode != Opcode::WriteVar16 { continue; }
      let Ref::Symbol(low_symref) = &low_instr.operands[0] else { continue };
      let low_sym = low_symref.def(&ir.symbols);
      if low_symref.off() != 0 { continue; }
      if low_symref.sz() != 2 { continue; }
      if low_sym.size != 4 { continue; }

      // Find high write16: E.g. 'writevar16 _local_0028@2        ax.3' where _local_0028 is u32
      let Some(high_ref) = ir.prev_ref_in_block(low_ref) else { continue };
      let high_instr = ir.instr(high_ref).unwrap();
      if high_instr.opcode != Opcode::WriteVar16 { continue; }
      let Ref::Symbol(high_symref) = &high_instr.operands[0] else { continue };
      let high_sym = high_symref.def(&ir.symbols);
      if high_symref.off() != 2 { continue; }
      if high_symref.sz() != 2 { continue; }
      if low_sym as *const _ != high_sym as *const _ { continue; }

      // New sequence: Make32 and WriteVar32
      let symref = sym::SymbolRef::join_adjacent(&ir.symbols, *low_symref, *high_symref).unwrap();

      let low_val = low_instr.operands[1];
      let high_val = high_instr.operands[1];
      *ir.instr_mut(high_ref).unwrap() = Instr {
        typ: Type::U32,
        attrs: Attribute::NONE,
        opcode: Opcode::Make32,
        operands: vec![high_val, low_val],
      };
      *ir.instr_mut(low_ref).unwrap() = Instr {
        typ: Type::Void,
        attrs: Attribute::MAY_ESCAPE,
        opcode: Opcode::WriteVar32,
        operands: vec![Ref::Symbol(symref), high_ref],
      };
    }
  }
}

fn is_fusable_load16_to_load32(ir: &IR, high_ref: Ref, low_ref: Ref) -> bool {
  let Some(high_instr) = ir.instr(high_ref) else { return false };
  let Some(low_instr) = ir.instr(low_ref) else { return false };
  if high_instr.opcode != Opcode::Load16 { return false } // high instr is load16
  if low_instr.opcode != Opcode::Load16 { return false } // low instr is load16
  if high_instr.operands[0] != low_instr.operands[0] { return false } // matching segments?

  let high_ref_ref = high_instr.operands[1];
  let low_ref_ref = low_instr.operands[1];
  let (high_k, low_k) = match (high_ref_ref, low_ref_ref) {
    //(Ref::Const(_), Ref::Const(_)) => (high_ref_ref, low_ref_ref),
    (Ref::Instr(_, _), Ref::Instr(_, _)) => {
      let high_off = ir.instr(high_ref_ref).unwrap();
      let low_off = ir.instr(low_ref_ref).unwrap();
      if high_off.opcode != Opcode::Add { return false }

      //if false { unreachable!();
      if high_off.operands[0] == low_ref_ref && high_off.operands[1].is_const() {
        (ir.lookup_const(high_off.operands[1]).unwrap(), 0)
      } else {
        if low_off.opcode != Opcode::Add { return false }
        if high_off.operands[0] != low_off.operands[0] { return false }
        let Ref::Const(_) = &high_off.operands[1] else { return false };
        let Ref::Const(_) = &low_off.operands[1] else { return false };
        (
          ir.lookup_const(high_off.operands[1]).unwrap(),
          ir.lookup_const(low_off.operands[1]).unwrap(),
        )
      }
    }
    _ => return false,
  };

  high_k == low_k+2
}

pub fn fuse_make32_load16_to_load32(ir: &mut IR) {
  // FIXME: THIS FUNCTION IS MESSY... CAN WE MAKE IT CLEANER???
  for b in ir.iter_blocks() {
    for r in ir.iter_instrs(b) {
      let make32_ref = r;
      let make32_instr = ir.instr(make32_ref).unwrap();
      if make32_instr.opcode != Opcode::Make32 { continue }

      let high_ref = make32_instr.operands[0];
      let low_ref = make32_instr.operands[1];
      if !is_fusable_load16_to_load32(ir, high_ref, low_ref) { continue }

      // Need to check that the load16's are in the same block and no other memory references
      // are between them and the make32 (otherwise we could have aliasing and break everything)
      let Ref::Instr(make32_b, _) = make32_ref else { unreachable!() };
      let Ref::Instr(high_b, high_i) = high_ref else { unreachable!() };
      let Ref::Instr(low_b, low_i) = low_ref else { unreachable!() };
      if make32_b != high_b { continue }
      if make32_b != low_b { continue }
      let start_i = std::cmp::min(high_i, low_i);
      let mut cur = Ref::Instr(make32_b, start_i);
      let mut allowed = true;
      loop {
        cur = ir.next_ref_in_block(cur).unwrap();
        if cur == make32_ref { break }
        if cur == high_ref || cur == low_ref { continue };
        let instr = ir.instr(cur).unwrap();
        if instr.opcode.is_mem_op() {
          allowed = false;
          break;
        }
      }
      if !allowed { continue }

      // Okay, do the rewrite!
      let low_instr = ir.instr(low_ref).unwrap();
      let (seg, off) = (low_instr.operands[0], low_instr.operands[1]);
      *ir.instr_mut(make32_ref).unwrap() = Instr {
        typ: Type::U32,
        attrs: Attribute::MAY_ESCAPE,
        opcode: Opcode::Load32,
        operands: vec![seg, off],
      };
      //println!("FUSEABLE: {}", crate::ir::display::instr_to_string(ir, make32_ref));
    }
  }
}

pub fn fuse_mem(ir: &mut IR) {
  fuse_adjacent_writevar16_to_writevar32(ir);
  fuse_make32_load16_to_load32(ir);
}
