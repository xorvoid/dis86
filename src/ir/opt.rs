use crate::ir::def::*;
use crate::ir::sym;
use crate::types::Type;
use std::collections::{hash_map, HashMap, HashSet, VecDeque};

// Propagate operand through any ref opcodes
fn operand_propagate(ir: &IR, mut r: Ref) -> Ref {
  loop {
    let Some(instr) = ir.instr(r) else { return r };
    if instr.opcode != Opcode::Ref { return r; }
    r = instr.operands[0];
  }
}

/*
From:
--------------------------------------------
  t2 = xor t1 t1

To:
--------------------------------------------
  t2 = #0
*/
pub fn reduce_xor(ir: &mut IR) {
  for b in ir.iter_blocks() {
    for r in ir.iter_instrs(b) {
      let instr = ir.instr(r).unwrap();
      if instr.opcode != Opcode::Xor || instr.operands[0] != instr.operands[1] {
        continue;
      }
      let k = ir.append_const(0);
      let instr = ir.instr_mut(r).unwrap();
      instr.opcode = Opcode::Ref;
      instr.operands = vec![k];
    }
  }
}

/*
From:
--------------------------------------------
  t2 = xor t1 t1

To:
--------------------------------------------
  t2 = ref t1
*/
pub fn reduce_trivial_or(ir: &mut IR) {
  for b in ir.iter_blocks() {
    for r in ir.iter_instrs(b) {
      let instr = ir.instr(r).unwrap();
      if instr.opcode != Opcode::Or || instr.operands[0] != instr.operands[1] {
        continue;
      }
      let instr = ir.instr_mut(r).unwrap();
      instr.opcode = Opcode::Ref;
      instr.operands = vec![instr.operands[0]];
    }
  }
}

/*
From:
--------------------------------------------
  t36      = signext32  t34
  dx.2     = upper16    t36
  t37      = make32     dx.2                 t34

To:
--------------------------------------------
  t37      = signext32  t34
*/
pub fn reduce_make_32_signext_32(ir: &mut IR) {
  for b in ir.iter_blocks() {
    for r in ir.iter_instrs(b) {
      let make32_ref = r;
      let make32 = ir.instr(make32_ref).unwrap();
      if make32.opcode != Opcode::Make32 { continue; }

      let upper16_ref = make32.operands[0];
      let Some(upper16) = ir.instr(upper16_ref) else { continue };
      if upper16.opcode != Opcode::Upper16 { continue; }

      let signext32_ref = upper16.operands[0];
      let Some(signext32) = ir.instr(signext32_ref) else { continue };
      if signext32.opcode != Opcode::SignExtTo32 { continue; }

      if make32.operands[1] == signext32.operands[0] {
        let instr = ir.instr_mut(make32_ref).unwrap();
        instr.opcode = Opcode::Ref;
        instr.operands = vec![signext32_ref];
      }
    }
  }
}

/*
From:
--------------------------------------------
  t5 = phi t5 t1 t1 t5 t5
To:
--------------------------------------------
  t5 = ref t1
*/
pub fn reduce_phi_single_ref(ir: &mut IR) {
  for b in ir.iter_blocks() {
    for r in ir.iter_instrs(b) {
      if ir.instr(r).unwrap().opcode != Opcode::Phi { continue; }

      // propagate while checking conditions
      let mut operands = ir.instr(r).unwrap().operands.clone();
      let mut trivial = true;
      let mut single_ref = None;
      for j in 0..operands.len() {
        operands[j] = operand_propagate(ir, operands[j]);
        if operands[j] == r { continue; }
        match &single_ref {
          None => single_ref = Some(operands[j]),
          Some(s) => if *s != operands[j] {
            trivial = false;
          }
        }
      }
      ir.instr_mut(r).unwrap().operands = operands;

      // all operands the same? reduce to a mov
      if trivial && single_ref.is_some() {
        let vref = single_ref.unwrap();
        let instr = ir.instr_mut(r).unwrap();
        instr.opcode = Opcode::Ref;
        instr.operands = vec![vref];
      }
    }
  }
}

/*
From:
--------------------------------------------
b1:
  t1 = op r1 r2 r3
       jmp b3

b2:
  t2 = op r1 r2 r3
       jmp b3

b3: (b1 b2 b3)
  t5 = phi t1 t2 t5

To:
--------------------------------------------
b3: (b1, b2)
  t5 = op r1 r2 r3
*/
pub fn reduce_phi_common_subexpr(ir: &mut IR) {
  for b in ir.iter_blocks() {
    for r in ir.iter_instrs(b) {
      if ir.instr(r).unwrap().opcode != Opcode::Phi { continue; }

      let mut operands = ir.instr(r).unwrap().operands.clone();

      // propagate all operands
      for oper in &mut operands {
        *oper = operand_propagate(ir, *oper);
      }

      // find first non-trivial to act as common
      let mut common = None;
      for oper in &operands {
        if *oper == r { continue; }
        common = Some(*oper);
        break;
      }
      let Some(common) = common else { continue };
      let Some(common_instr) = ir.instr(common).cloned() else { continue };

      // need to pessimize around side-effecting operations
      if common_instr.opcode.has_side_effects() { continue; }

      // see if all non-trivial operands match
      let mut all_match = true;
      for oper in &operands {
        if *oper == r { continue; }
        let instr = ir.instr(*oper);
        if instr.is_none() || &common_instr != instr.unwrap() {
          all_match = false;
          break;
        }
      }

      // re-write the phi
      if all_match {
        *ir.instr_mut(r).unwrap() = common_instr;
      }
    }
  }
}

fn stack_ptr_const_oper(ir: &IR, vref: Ref) -> Option<(Ref, i16)> {
  let instr = ir.instr(vref)?;
  if instr.operands.len() != 2 { return None; }
  if (instr.attrs & Attribute::STACK_PTR) == 0 { return None; }

  let (nref, cref) = (instr.operands[0], instr.operands[1]);
  let Ref::Const(_) = cref else { return None };

  match instr.opcode {
    Opcode::Add => Some((nref, ir.lookup_const(cref).unwrap())),
    Opcode::Sub => Some((nref, -ir.lookup_const(cref).unwrap())),
    _ => None,
  }
}

pub fn stack_ptr_accumulation(ir: &mut IR) {
  for b in ir.iter_blocks() {
    for vref in ir.iter_instrs(b) {
      let Some((_, a)) = stack_ptr_const_oper(ir, vref) else { continue };

      let instr = ir.instr(vref).unwrap();
      let Some((nref, b)) = stack_ptr_const_oper(ir, instr.operands[0]) else { continue };

      let k = a+b;
      if k > 0 {
        let cref = ir.append_const(k);
        let instr = ir.instr_mut(vref).unwrap();
        instr.opcode = Opcode::Add;
        instr.operands = vec![nref, cref];
      } else if k < 0 {
        let cref = ir.append_const(-k);
        let instr = ir.instr_mut(vref).unwrap();
        instr.opcode = Opcode::Sub;
        instr.operands = vec![nref, cref];
      } else {
        let instr = ir.instr_mut(vref).unwrap();
        instr.opcode = Opcode::Ref;
        instr.operands = vec![nref];
      }
    }
  }
}

pub fn value_propagation(ir: &mut IR) {
  for b in ir.iter_blocks() {
    for r in ir.iter_instrs(b) {
      // Propagate all operands
      let mut operands = ir.instr(r).unwrap().operands.clone();
      for j in 0..operands.len() {
        operands[j] = operand_propagate(ir, operands[j]);
      }
      ir.instr_mut(r).unwrap().operands = operands;
    }
  }
}

pub fn deadcode_elimination(ir: &mut IR) {
  // Mark and Sweep DCE
  //   DCE has the same sort of problem as garbage-collection. If you implement it by
  //   removing code only when n_uses == 0, then you can never remove dead-cycles.
  //   This is the same problem as using a refcnt-based GC. By contrast, mark-and-sweep
  //   "just works"

  // First we populate the "root set" which we'll consider to be any side-effecting
  // operation. This may be a little pessimistic, but we consider it the responsibility
  // of other opt passes to "prove" that side-effects are not required and reduce them to
  // code that DCE can eliminate.

  let mut unprocessed = VecDeque::new();
  for b in ir.iter_blocks() {
    for r in ir.iter_instrs(b) {
      let instr = ir.instr(r).unwrap();
      if instr.opcode.has_side_effects() {
        unprocessed.push_back(r);
      }
    }
  }

  // Next, we build up the live-set by recursively processing the deps
  // of any live ref.. adding each to the liveset until we're done
  let mut live_refs = HashSet::new();
  while let Some(r) = unprocessed.pop_front() {
    if live_refs.get(&r).is_some() { continue; } // already processed
    live_refs.insert(r);
    // add all operands to the unprocessed lise
    if let Some(instr) = ir.instr(r) {
      for oper_ref in &instr.operands {
        unprocessed.push_back(*oper_ref);
      }
    }
  }

  // Lastly, use the live set to remove dead-code
  for b in ir.iter_blocks() {
    for r in ir.iter_instrs(b) {
      if live_refs.get(&r).is_some() { continue; } // live
      let instr = ir.instr_mut(r).unwrap();
      instr.opcode = Opcode::Nop;
      instr.operands = vec![];
    }
  }
}

pub fn deadblock_elimination(ir: &mut IR) {
  // A dead block is one with no preds
  for blkref in ir.iter_blocks() {
    let blk = ir.block(blkref);
    if blkref == BlockRef(0) { continue; } // entry block is always alive
    if blk.preds.len() > 0 { continue; }

    // Need to remove ourself as a pred from any target blocks
    for exit in blk.exits() {
      // Find pred_idx
      let exit_blk = ir.block_mut(exit);
      let mut pred_idx = None;
      for (i, p) in exit_blk.preds.iter().enumerate() {
        if *p == blkref {
          pred_idx = Some(i);
          break;
        }
      }
      let pred_idx = pred_idx.unwrap();

      // Remove index from pred and all phis
      exit_blk.preds.remove(pred_idx);
      for r in ir.iter_instrs(exit) {
        let instr = ir.instr_mut(r).unwrap();
        if instr.opcode != Opcode::Phi { continue; }
        instr.operands.remove(pred_idx);
      }
    }

    ir.remove_block(blkref);
  }
}

fn allow_cse(opcode: Opcode) -> bool {
  match opcode {
    Opcode::Add => true,
    Opcode::Sub => true,
    Opcode::Shl => true,
    Opcode::Shr => true,
    Opcode::UShr => true,
    Opcode::And => true,
    Opcode::Or => true,
    Opcode::Xor => true,
    Opcode::IMul => true,
    Opcode::UMul => true,
    Opcode::IDiv => true,
    Opcode::UDiv => true,
    Opcode::Neg => true,
    Opcode::SignExtTo32 => true,
    Opcode::Lower16 => true,
    Opcode::Upper16 => true,
    Opcode::Make32 => true,
    Opcode::UpdateFlags => true,
    Opcode::EqFlags => true,
    Opcode::NeqFlags => true,
    Opcode::GtFlags => true,
    Opcode::GeqFlags => true,
    Opcode::LtFlags => true,
    Opcode::LeqFlags => true,
    Opcode::UGtFlags => true,
    Opcode::UGeqFlags => true,
    Opcode::ULtFlags => true,
    Opcode::ULeqFlags => true,
    Opcode::Eq => true,
    Opcode::Neq => true,
    Opcode::Gt => true,
    Opcode::Geq => true,
    Opcode::Lt => true,
    Opcode::Leq => true,
    Opcode::UGt => true,
    Opcode::UGeq => true,
    Opcode::ULt => true,
    Opcode::ULeq => true,
    _ => false,
  }
}

pub fn common_subexpression_elimination(ir: &mut IR) {
  for b in ir.iter_blocks() {
    let mut prev = HashMap::new();
    for r in ir.iter_instrs(b) {
      let instr = ir.instr(r).unwrap();
      if !allow_cse(instr.opcode) { continue; }

      let prev_ref = match prev.entry(instr.clone()) {
        hash_map::Entry::Vacant(x) => {
          x.insert(r);
          continue;
        }
        hash_map::Entry::Occupied(x) => *x.get(),
      };

      let instr = ir.instr_mut(r).unwrap();
      instr.opcode = Opcode::Ref;
      instr.operands = vec![prev_ref];
    }
  }
}

pub fn forward_store_to_load(ir: &mut IR) {
  let prev_lookup = |ir: &IR, prev_stores: &[Ref], seg, off| -> Option<Ref> {
    for store_ref in prev_stores.iter().rev() {
      let store_instr = ir.instr(*store_ref).unwrap();
      // FIXME: Need to pessimize to account for possible aliasing
      if seg == store_instr.operands[0] && off == store_instr.operands[1] {
        return Some(store_instr.operands[2]);
      }
    }
    None
  };

  for b in ir.iter_blocks() {
    // Don't forward across blocks!!
    let mut prev_stores = vec![];
    for r in ir.iter_instrs(b) {
      let instr = ir.instr(r).unwrap();
      if instr.opcode.is_store() {
        prev_stores.push(r);
      }
      if !instr.opcode.is_load() { continue; }
      let seg = instr.operands[0];
      let off = instr.operands[1];
      let Some(store_val) = prev_lookup(ir, &prev_stores, seg, off) else {continue };

      let instr = ir.instr_mut(r).unwrap();
      instr.opcode = Opcode::Ref;
      instr.operands = vec![store_val];
    }
  }
}

pub fn mem_symbol_to_ref(ir: &mut IR) {
  // FIXME: Need to pessimize with escape-analysis
  // TODO: Expand the scope of this.. only handing 16-bit symbols and operations

  // Unseal all the blocks so phi nodes generate correctly
  ir.unseal_all_blocks();

  // Pass 1: Write -> Ref
  for b in ir.iter_blocks() {
    for r in ir.iter_instrs(b) {
      let instr = ir.instr(r).unwrap();

      // FIXME: THIS IS WRONG.. WE SHOULD USE THESE TO PROVE NON-ESCAPE... E.G. IF A STACK REFERENCE
      // IS NOT MARKED "MAY_ESCAPE" IT MIGHT STILL ESCAPE IF IT ANOTHER INSTR USES THE ADDRESS AND
      // IS MARKED "MAY_ESCAPE" ... BUT FOR NOW IT'S AN EASY WAY TO SEPERATE LOCAL VARS FROM TEMPORARY
      // PUSH/POP STACK SLOTS (WE WANT TO PESSIMIZE THE FORMER AND OPTIMIZE THE LATER). THIS BIG
      // COMMENT EXISTS TO REMIND THE FUTURE DEBUGGER OF PAST LAZINESS
      if (instr.attrs & Attribute::MAY_ESCAPE) != 0 { continue }; // don't life any memory ref that might escape

      if instr.opcode == Opcode::WriteVar16 {
        let Ref::Symbol(symref) = instr.operands[0] else { continue };
        if ir.symbols.symbol_region(symref) != sym::SymbolRegion::Local { continue; }
        if ir.symbols.symbol(symref).size != 2 { continue; }

        let name = Name::Var(ir.symbols.symbol_name(symref));

        let instr = ir.instr_mut(r).unwrap();
        instr.opcode = Opcode::Ref;
        instr.operands = vec![instr.operands[1]];

        // Add the def
        ir.set_var(name, b, r);
      } else if instr.opcode == Opcode::ReadVar16 {
        let Ref::Symbol(symref) = instr.operands[0] else { continue };
        if ir.symbols.symbol_region(symref) != sym::SymbolRegion::Local { continue; }
        if ir.symbols.symbol(symref).size != 2 { continue; }

        let name = Name::Var(ir.symbols.symbol_name(symref));
        let vref = ir.get_var(name, b);

        let instr = ir.instr_mut(r).unwrap();
        instr.opcode = Opcode::Ref;
        instr.operands = vec![vref];
      }
    }
  }

  // Re-seal all the blocks so phi nodes generate correctly
  ir.seal_all_blocks();
}

pub fn simplify_branch_conds(ir: &mut IR) {
  for b in ir.iter_blocks() {
    for r in ir.iter_instrs(b) {
      let instr = ir.instr(r).unwrap();

      let opcode_new = match instr.opcode {
        Opcode::EqFlags  => Opcode::Eq,
        Opcode::NeqFlags => Opcode::Neq,
        Opcode::GtFlags  => Opcode::Gt,
        Opcode::GeqFlags => Opcode::Geq,
        Opcode::LtFlags  => Opcode::Lt,
        Opcode::LeqFlags => Opcode::Leq,
        Opcode::UGtFlags  => Opcode::UGt,
        Opcode::UGeqFlags => Opcode::UGeq,
        Opcode::ULtFlags  => Opcode::ULt,
        Opcode::ULeqFlags => Opcode::ULeq,
        _ => continue,
      };

      let opcode_eq = opcode_new == Opcode::Eq || opcode_new == Opcode::Neq;
      let opcode_above = opcode_new == Opcode::UGt;

      let upd_ref = instr.operands[0];
      let upd_instr = ir.instr(upd_ref).unwrap();
      if upd_instr.opcode != Opcode::UpdateFlags { continue; }

      let pred_ref = upd_instr.operands[1];
      let pred_instr = ir.instr(pred_ref).unwrap();

      if pred_instr.opcode == Opcode::Sub {
        // cmp ax, bx
        // jg <tgt>
        let lhs = pred_instr.operands[0];
        let rhs = pred_instr.operands[1];

        let instr = ir.instr_mut(r).unwrap();
        instr.opcode = opcode_new;
        instr.operands = vec![lhs, rhs];
      }

      else if pred_instr.opcode == Opcode::And && opcode_eq {
        // test ax, bx
        // je <tgt>
        let z = ir.append_const(0);
        let instr = ir.instr_mut(r).unwrap();
        instr.opcode = opcode_new;
        instr.operands = vec![pred_ref, z];
      }

      else if pred_instr.opcode == Opcode::Or && opcode_eq && pred_instr.operands[0] == pred_instr.operands[1] {
        // or ax, ax
        // je <tgt>
        let z = ir.append_const(0);
        let instr = ir.instr_mut(r).unwrap();
        instr.opcode = opcode_new;
        instr.operands = vec![pred_ref, z];
      }

      else if pred_instr.opcode == Opcode::Or && opcode_above && pred_instr.operands[0] == pred_instr.operands[1] {
        // or ax, ax
        // ja <tgt>   (equivalent to "jne <tgt>" after the or)
        let z = ir.append_const(0);
        let instr = ir.instr_mut(r).unwrap();
        instr.opcode = Opcode::Neq;
        instr.operands = vec![pred_ref, z];
      }
    }
  }
}

const N_OPT_PASSES: usize = 5;
pub fn optimize(ir: &mut IR) {
  deadblock_elimination(ir);
  for _ in 0..N_OPT_PASSES {
    reduce_xor(ir);
    reduce_make_32_signext_32(ir);
    reduce_phi_single_ref(ir);
    reduce_phi_common_subexpr(ir);
    simplify_branch_conds(ir);
    // note: reduce_trivial_or() after simplify_branch_conds() is important
    reduce_trivial_or(ir);
    stack_ptr_accumulation(ir);
    value_propagation(ir);
    common_subexpression_elimination(ir);
    value_propagation(ir);
  }
  deadcode_elimination(ir);
}
