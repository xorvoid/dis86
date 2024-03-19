use crate::asm::instr;
use crate::config::Config;
use crate::ir::def::*;
use crate::types::Type;
use std::cmp::Ordering;

// FIXME THIS WHOLE MODULE IS A MESS OF BAD DESIGN

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SymbolRegion {
  Param,
  Local,
  Global,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymbolRef {
  pub region: SymbolRegion,
  pub idx: usize /* id */,
  pub off: i32, /* off-adjust */
  pub sz: u32, /* size */
}

#[derive(Debug, Clone)]
pub struct Symbol {
  pub name: String,
  pub off: i32,
  pub size: u32,
}

impl Symbol {
  fn start(&self) -> i32 {
    self.off
  }
  fn end(&self) -> i32 {
    self.off + (self.size as i32)
  }
}

#[derive(Debug)]
pub struct SymbolTable {
  symbols: Vec<Symbol>, // Ordered by offset
}

#[derive(Debug)]
pub struct SymbolMap {
  pub params: SymbolTable,
  pub locals: SymbolTable,
  pub globals: SymbolTable,
}

impl SymbolMap {
  pub fn new() -> Self {
    Self {
      params: SymbolTable::new(),
      locals: SymbolTable::new(),
      globals: SymbolTable::new(),
    }
  }
}

impl SymbolRef {
  pub fn to_type(&self) -> Type {
    match self.sz {
      1 => Type::U8,
      2 => Type::U16,
      4 => Type::U32,
      _ => panic!("Unsupported type size: {}", self.sz),
    }
  }
}

impl SymbolTable {
  pub fn new() -> Self {
    Self {
      symbols: Vec::new(),
    }
  }

  pub fn append(&mut self, name: &str, off: i32, size: u32) {
    self.symbols.push(Symbol {
      name: name.to_string(),
      off,
      size,
    });
  }

  pub fn coalesce(&mut self) {
    if self.symbols.len() == 0 {
      return;
    }

    self.symbols.sort_by(|a, b| match a.off.cmp(&b.off) {
      Ordering::Less => Ordering::Less,
      Ordering::Greater => Ordering::Greater,
      Ordering::Equal => a.size.cmp(&b.size),
    });

    let mut new_symbols = vec![self.symbols[0].clone()];

    for sym in &self.symbols[1..] {
      let last_idx = new_symbols.len() - 1;
      let last = &mut new_symbols[last_idx];
      if sym.start() < last.end() { // overlapping?
        // simply update the last size
        last.size = (sym.end() - last.start()).try_into().unwrap();
      } else { // disjoint?
        new_symbols.push(sym.clone());
      }
    }

    self.symbols = new_symbols;
  }

  pub fn finalize_non_overlaping(&mut self) {
    self.symbols.sort_by(|a, b| match a.off.cmp(&b.off) {
      Ordering::Less => Ordering::Less,
      Ordering::Greater => Ordering::Greater,
      Ordering::Equal => a.size.cmp(&b.size),
    });

    // FIXME: ADD THIS BACK
    // for idx in 1..self.symbols.len() {
    //   if self.symbols[idx].start() < self.symbols[idx-1].end() { // overlapping
    //     panic!("Overlapping symbols: {} and {}", self.symbols[idx-1].name, self.symbols[idx].name);
    //   }
    // }
  }

  pub fn find_by_name(&self, name: &str) -> Option<(usize, &Symbol)> {
    // FIXME: Avoid O(n) search
    for (i, sym) in self.symbols.iter().enumerate() {
      if name == sym.name.as_str() {
        return Some((i, sym));
      }
    }
    None
  }
}

impl SymbolMap {
  fn region(&self, region: SymbolRegion) -> &SymbolTable {
    match region {
      SymbolRegion::Param  => &self.params,
      SymbolRegion::Local  => &self.locals,
      SymbolRegion::Global => &self.globals,
    }
  }

  pub fn find_by_name(&self, name: &str) -> Option<SymbolRef> {
    if let Some((idx, sym)) = self.params.find_by_name(name) {
      return Some(SymbolRef { region: SymbolRegion::Param, idx, off: 0, sz: sym.size })
    }
    if let Some((idx, sym)) = self.locals.find_by_name(name) {
      return Some(SymbolRef { region: SymbolRegion::Local, idx, off: 0, sz: sym.size })
    }
    if let Some((idx, sym)) = self.globals.find_by_name(name) {
      return Some(SymbolRef { region: SymbolRegion::Global, idx, off: 0, sz: sym.size })
    }
    None
  }

  pub fn find_ref(&self, region: SymbolRegion, off: i32, sz: u32) -> Option<SymbolRef> {
    // FIXME: This is sorted: can use binary search
    let tbl = self.region(region);
    for (i, sym) in tbl.symbols.iter().enumerate() {
      if sym.start() <= off && off < sym.end() {
        return Some(SymbolRef {
          region,
          idx: i,
          off: off - sym.start(),
          sz,
        });
      }
    }
    None
  }

  pub fn symbol(&self, r: SymbolRef) -> &Symbol {
    &self.region(r.region).symbols[r.idx]
  }

  pub fn symbol_region(&self, r: SymbolRef) -> SymbolRegion {
    r.region
  }

  pub fn symbol_name(&self, r: SymbolRef) -> String {
    let name = &self.symbol(r).name;
    if r.off == 0 {
      format!("{}", name)
    } else {
      format!("{}@+{}", name, r.off)
    }
  }
}

pub fn symbolize_stack(ir: &mut IR) {
  let ss = Ref::Init(instr::Reg::SS);
  let sp = Ref::Init(instr::Reg::SP);

  // Detect locals and params
  let mut var_mem_refs = vec![];
  for b in ir.iter_blocks() {
    for r in ir.iter_instrs(b) {

      let mem_ref = r;
      let mem_instr = ir.instr(mem_ref).unwrap();
      if !mem_instr.opcode.is_load() && !mem_instr.opcode.is_store() { continue; }
      if mem_instr.operands[0] != ss { continue; }

      let addr_ref = mem_instr.operands[1];
      let addr_instr = ir.instr(addr_ref).unwrap();
      if addr_instr.operands[0] != sp { continue; }

      let off = match addr_instr.opcode {
        Opcode::Add => ir.lookup_const(addr_instr.operands[1]),
        Opcode::Sub => ir.lookup_const(addr_instr.operands[1]).map(|x| -x),
        _ => None,
      };
      let Some(off) = off else { continue };

      let size = mem_instr.opcode.operation_size();

      var_mem_refs.push((mem_ref, off, size));

      // let mut f = crate::ir::display::Formatter::new();
      // f.fmt_instr(ir, addr_ref, addr_instr).unwrap();
      // f.fmt_instr(ir, mem_ref, mem_instr).unwrap();
      // println!("{}", f.finish());

      let frame_offset = 2;
      if off > 0 {
        let name = format!("_param_{:04x}", off+frame_offset);
        ir.symbols.params.append(&name, off, size);
      } else {
        let name = format!("_local_{:04x}", -(off+frame_offset));
        ir.symbols.locals.append(&name, off, size);
      }
    }
  }

  // Coalesce any duplicates or overlaps
  ir.symbols.params.coalesce();
  ir.symbols.locals.coalesce();

  // Update the IR
  for (mem_ref, off, sz) in var_mem_refs {
    let region = if off > 0 { SymbolRegion::Param } else { SymbolRegion::Local };
    //let size = ir.instr(mem_ref).unwrap().opcode.operation_size();
    let sym = ir.symbols.find_ref(region, off, sz).unwrap();

    let instr = ir.instr_mut(mem_ref).unwrap();
    if instr.opcode.is_load() {
      instr.opcode = match instr.opcode {
        Opcode::Load8 => Opcode::ReadVar8,
        Opcode::Load16 => Opcode::ReadVar16,
        Opcode::Load32 => Opcode::ReadVar32,
        _ => unreachable!(),
      };
      instr.operands = vec![Ref::Symbol(sym)];
    } else {
      instr.opcode = match instr.opcode {
        Opcode::Store8 => Opcode::WriteVar8,
        Opcode::Store16 => Opcode::WriteVar16,
        _ => unreachable!(),
      };
      instr.operands = vec![Ref::Symbol(sym), instr.operands[2]];
    }
  }
}

pub fn populate_globals(ir: &mut IR, cfg: &Config) {
  for g in &cfg.globals {
    // FIXME: Remove the Type::Unknown
    let size = g.typ.size_in_bytes().unwrap_or_else(|| {
      eprintln!("WARN: Unsupported type '{}' for {} ... assuming u32", g.typ, g.name);
      Type::U32.size_in_bytes().unwrap()
    });
    ir.symbols.globals.append(&g.name, g.offset.into(), size as u32);
  }
  ir.symbols.globals.finalize_non_overlaping();
}

pub fn symbolize_globals(ir: &mut IR, cfg: &Config) {
  populate_globals(ir, cfg);

  let ds = Ref::Init(instr::Reg::DS);

  for b in ir.iter_blocks() {
    for r in ir.iter_instrs(b) {
      let instr = ir.instr(r).unwrap();
      if !instr.opcode.is_load() && !instr.opcode.is_store() { continue; }
      if instr.operands[0] != ds { continue; }
      let off_ref = instr.operands[1];
      let size = instr.opcode.operation_size();
      let Some(off) = ir.lookup_const(off_ref) else { continue };
      let Some(sym) = ir.symbols.find_ref(SymbolRegion::Global, off, size) else {
        eprintln!("WARN: Could not find global for DS:{:04x}", off);
        continue;
      };

      let instr = ir.instr_mut(r).unwrap();
      if instr.opcode.is_load() {
        instr.opcode = match instr.opcode {
          Opcode::Load8 => Opcode::ReadVar8,
          Opcode::Load16 => Opcode::ReadVar16,
          Opcode::Load32 => Opcode::ReadVar32,
          _ => unreachable!(),
        };
        instr.operands = vec![Ref::Symbol(sym)];
      } else {
        instr.opcode = match instr.opcode {
          Opcode::Store8 => Opcode::WriteVar8,
          Opcode::Store16 => Opcode::WriteVar16,
          _ => unreachable!(),
        };
        instr.operands = vec![Ref::Symbol(sym), instr.operands[2]];
      }
    }
  }
}

pub fn symbolize(ir: &mut IR, cfg: &Config) {
  symbolize_stack(ir);
  symbolize_globals(ir, cfg);
}
