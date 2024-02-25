use crate::decomp::ir::def::*;
use crate::decomp::ir::sym;
use std::collections::{hash_map, HashMap, HashSet};

// Propagate operand through any ref opcodes
fn operand_propagate(ir: &IR, mut r: Ref) -> Ref {
  loop {
    let Some(instr) = ir.instr(r) else { return r };
    if instr.opcode != Opcode::Ref { return r; }
    r = instr.operands[0];
  }
}

pub fn reduce_xor(ir: &mut IR) {
  for b in 0..ir.blocks.len() {
    for i in ir.blocks[b].instrs.range() {
      let r = Ref::Instr(BlockRef(b), i);
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

pub fn reduce_phi(ir: &mut IR) {
  for b in 0..ir.blocks.len() {
    for i in ir.blocks[b].instrs.range() {
      let r = Ref::Instr(BlockRef(b), i);
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
      if trivial {
        let vref = single_ref.unwrap();
        let instr = ir.instr_mut(r).unwrap();
        instr.opcode = Opcode::Ref;
        instr.operands = vec![vref];
      }
    }
  }
}

fn arith_const_oper(ir: &IR, vref: Ref) -> Option<(Ref, i32)> {
  let instr = ir.instr(vref)?;
  if instr.operands.len() != 2 { return None; }

  let (nref, cref) = (instr.operands[0], instr.operands[1]);
  let Ref::Const(_) = cref else { return None };

  match instr.opcode {
    Opcode::Add => Some((nref, ir.lookup_const(cref).unwrap())),
    Opcode::Sub => Some((nref, -ir.lookup_const(cref).unwrap())),
    _ => None,
  }
}

pub fn arithmetic_accumulation(ir: &mut IR) {
  for b in 0..ir.blocks.len() {
    for i in ir.blocks[b].instrs.range() {
      let vref = Ref::Instr(BlockRef(b), i);
      let Some((_, a)) = arith_const_oper(ir, vref) else { continue };


      let instr = ir.instr(vref).unwrap();
      let Some((nref, b)) = arith_const_oper(ir, instr.operands[0]) else { continue };

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
  for b in 0..ir.blocks.len() {
    for i in ir.blocks[b].instrs.range() {
      let r = Ref::Instr(BlockRef(b), i);
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
  // pass 1: find all uses
  let mut used = HashSet::new();
  for b in 0..ir.blocks.len() {
    for i in ir.blocks[b].instrs.range() {
      let r = Ref::Instr(BlockRef(b), i);
      let instr = ir.instr(r).unwrap();
      // Add instructions with side-effects
      if instr.opcode != Opcode::Nop && instr.opcode.maybe_unused() {
        used.insert(r);
      }
      // Add operands
      for oper in &instr.operands {
        used.insert(*oper);
      }
    }
  }
  // pass 2: remove unused
  for b in 0..ir.blocks.len() {
    for i in ir.blocks[b].instrs.range() {
      let r = Ref::Instr(BlockRef(b), i);
      if used.get(&r).is_some() {
        continue;
      }
      // Overwrite the instr to nop
      let instr = ir.instr_mut(r).unwrap();
      instr.opcode = Opcode::Nop;
      instr.operands = vec![];
    }
  }
}

fn allow_cse(opcode: Opcode) -> bool {
  match opcode {
    Opcode::Add => true,
    Opcode::Sub => true,
    _ => false,
  }
}

pub fn common_subexpression_elimination(ir: &mut IR) {
  let mut prev = HashMap::new();
  for b in 0..ir.blocks.len() {
    for i in ir.blocks[b].instrs.range() {
      let r = Ref::Instr(BlockRef(b), i);
      let instr = ir.instr(r).unwrap();
      if !allow_cse(instr.opcode) { continue; }

      // FIXME: Hacky due to the "debug" field which has always felt out of place
      let mut key = instr.clone();
      key.debug = None;

      let prev_ref = match prev.entry(key) {
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
  let mut prev_stores = vec![];

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

  for b in 0..ir.blocks.len() {
    for i in ir.blocks[b].instrs.range() {
      let r = Ref::Instr(BlockRef(b), i);
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

fn get_var(ir: &IR, name: &Name, blk: BlockRef) -> Ref {
  // TODO: Unify with build::IRBuilder::get_var

  // Defined locally in this block? Easy.
  match ir.blocks[blk.0].defs.get(name) {
    Some(val) => return *val,
    None => (),
  }

  // Otherwise, search predecessors
  let preds = &ir.blocks[blk.0].preds;
  if preds.len() == 1 {
    let parent = preds[0];
    get_var(ir, name, parent)
  } else {
    panic!("Unimpl | Need PHI!!");
    // // create a phi and immediately populate it
    // let phi = self.phi_create(sym.clone(), blk);
    // self.phi_populate(sym, phi);
    // phi
  }
}

pub fn mem_symbol_to_ref(ir: &mut IR) {
  // FIXME: Need to pessimize with escape-analysis
  // TODO: Expand the scope of this.. only handing 16-bit symbols and operations
  for b in 0..ir.blocks.len() {
    for i in ir.blocks[b].instrs.range() {
      let r = Ref::Instr(BlockRef(b), i);
      let instr = ir.instr(r).unwrap();

      if instr.opcode == Opcode::WriteVar16 {
        let Ref::Symbol(symref) = instr.operands[0] else { continue };
        if ir.symbols.symbol_type(symref) != sym::SymbolType::Local { continue; }
        if ir.symbols.symbol(symref).size != 2 { continue; }

        let name = Name::Var(ir.symbols.symbol_name(symref));

        let instr = ir.instr_mut(r).unwrap();
        instr.debug = Some((name.clone(), 42));
        instr.opcode = Opcode::Ref;
        instr.operands = vec![instr.operands[1]];

        // Add the def
        ir.blocks[b].defs.insert(name, r);
      }

      else if instr.opcode == Opcode::ReadVar16 {
        let Ref::Symbol(symref) = instr.operands[0] else { continue };
        if ir.symbols.symbol_type(symref) != sym::SymbolType::Local { continue; }
        if ir.symbols.symbol(symref).size != 2 { continue; }

        let name = Name::Var(ir.symbols.symbol_name(symref));
        let vref = get_var(ir, &name, BlockRef(b));

        let instr = ir.instr_mut(r).unwrap();
        instr.debug = Some((name.clone(), 42));
        instr.opcode = Opcode::Ref;
        instr.operands = vec![vref];
      }
    }
  }

}

const N_OPT_PASSES: usize = 3;

pub fn optimize(ir: &mut IR) {
  for _ in 0..N_OPT_PASSES {
    // constant_fold(ir);
    // reduce_jne(ir);
    reduce_xor(ir);
    reduce_phi(ir);
    arithmetic_accumulation(ir);
    value_propagation(ir);
    common_subexpression_elimination(ir);
    value_propagation(ir);
    // jump_propagation(ir);
  }
  deadcode_elimination(ir);
}
