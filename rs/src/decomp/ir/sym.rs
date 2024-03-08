use crate::decomp::config::Config;
use crate::decomp::ir::def::*;
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SymbolType {
  Param,
  Local,
  Global,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymbolRef {
  pub typ: SymbolType,
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
}

impl SymbolMap {
  fn get_table(&self, typ: SymbolType) -> &SymbolTable {
    match typ {
      SymbolType::Param  => &self.params,
      SymbolType::Local  => &self.locals,
      SymbolType::Global => &self.globals,
    }
  }

  pub fn find_ref(&self, typ: SymbolType, off: i32, sz: u32) -> Option<SymbolRef> {
    // FIXME: This is sorted: can use binary search
    let tbl = self.get_table(typ);
    for (i, sym) in tbl.symbols.iter().enumerate() {
      if sym.start() <= off && off < sym.end() {
        return Some(SymbolRef {
          typ,
          idx: i,
          off: off - sym.start(),
          sz,
        });
      }
    }
    None
  }

  pub fn symbol(&self, r: SymbolRef) -> &Symbol {
    &self.get_table(r.typ).symbols[r.idx]
  }

  pub fn symbol_type(&self, r: SymbolRef) -> SymbolType {
    r.typ
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
  let ss = Ref::Init("ss"); //ir.blocks[0].defs.get(&instr::Reg::SS.into()).unwrap();
  let sp = Ref::Init("sp"); //ir.blocks[0].defs.get(&instr::Reg::BP.into()).unwrap();

  // Detect locals and params
  let mut var_mem_refs = vec![];
  for b in 0..ir.blocks.len() {
    for i in ir.blocks[b].instrs.range() {
      let r = Ref::Instr(BlockRef(b), i);

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

      // let mut f = crate::decomp::ir::display::Formatter::new();
      // f.fmt_instr(ir, addr_ref, addr_instr).unwrap();
      // f.fmt_instr(ir, mem_ref, mem_instr).unwrap();
      // println!("{}", f.finish());

      if off > 0 {
        let name = format!("_param_{:04}", off+2);
        ir.symbols.params.append(&name, off, size);
      } else {
        let name = format!("_local_{:04}", -(off+2));
        ir.symbols.locals.append(&name, off, size);
      }
    }
  }

  // Coalesce any duplicates or overlaps
  ir.symbols.params.coalesce();
  ir.symbols.locals.coalesce();

  // Update the IR
  for (mem_ref, off, sz) in var_mem_refs {
    let typ = if off > 0 { SymbolType::Param } else { SymbolType::Local };
    //let size = ir.instr(mem_ref).unwrap().opcode.operation_size();
    let sym = ir.symbols.find_ref(typ, off, sz).unwrap();

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
    let size = match &g.typ as &str {
      "u8" => 1,
      "u16" => 2,
      "u32" => 4,
      _ => {
        eprintln!("WARN: Unsupported type '{}' for {} ... assuming u32", g.typ, g.name);
        4
      }
    };
    ir.symbols.globals.append(&g.name, g.offset.into(), size);
  }
  ir.symbols.globals.finalize_non_overlaping();
}

pub fn symbolize_globals(ir: &mut IR, cfg: &Config) {
  populate_globals(ir, cfg);

  let ds = Ref::Init("ds");

  for b in 0..ir.blocks.len() {
    for i in ir.blocks[b].instrs.range() {
      let r = Ref::Instr(BlockRef(b), i);
      let instr = ir.instr(r).unwrap();
      if !instr.opcode.is_load() && !instr.opcode.is_store() { continue; }
      if instr.operands[0] != ds { continue; }
      let off_ref = instr.operands[1];
      let size = instr.opcode.operation_size();
      let Some(off) = ir.lookup_const(off_ref) else { continue };
      let Some(sym) = ir.symbols.find_ref(SymbolType::Global, off, size) else {
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
