use crate::asm::instr;
use crate::config::Config;
use crate::ir::def::*;
use crate::types::Type;
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Table {
  Param,
  Local,
  Global,
  Register,
}

// Id is a reference to lookup the cooresponding Symbol
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Id {
  table: Table,
  idx: usize,
}

// Region is information about a byte range region
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Region {
  pub off: i32,
  pub sz: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymbolRef {
  id: Id,
  pub access_region: Region,   // Access using this region with the symbol
}

impl SymbolRef {
  pub fn def<'a>(&self, map: &'a SymbolMap) -> &'a SymbolDef {
    &map.get_table(self.id.table).symbols[self.id.idx]
  }

  pub fn table(&self) -> Table {
    self.id.table
  }

  pub fn name(&self, map: &SymbolMap) -> String {
    let name = &self.def(map).name;
    if self.off() == 0 {
      format!("{}", name)
    } else {
      format!("{}@+{}", name, self.off())
    }
  }

  pub fn off(&self) -> i32 {
    self.access_region.off
  }

  pub fn sz(&self) -> u16 {
    self.access_region.sz
  }

  pub fn get_type<'a>(&self, map: &'a SymbolMap) -> &'a Type {
    &self.def(map).typ
  }

  pub fn join_adjacent(map: &SymbolMap, low: SymbolRef, high: SymbolRef) -> Option<SymbolRef> {
    let low_sym = low.def(map);
    let high_sym = high.def(map);
    if low_sym as *const _ != high_sym as *const _ {
      return None;
    }
    let low_endoff = low.off() + low.sz() as i32;
    if high.off() as i32 != low_endoff {
      return None;
    }
    Some(SymbolRef {
      id: low.id,
      access_region: Region {
        off: low.access_region.off,
        sz: low.access_region.sz + high.access_region.sz,
      }
    })
  }


}

#[derive(Debug, Clone)]
pub struct SymbolDef {
  pub name: String,
  pub typ: Type,
  pub off: i16,
  pub size: u16,
}

impl SymbolDef {
  fn start(&self) -> i32 {
    self.off.into()
  }
  fn end(&self) -> i32 {
    let off: i32 = self.off.into();
    let size: i32 = self.size.into();
    off + size
  }
}

#[derive(Debug)]
struct SymbolTable {
  symbols: Vec<SymbolDef>, // Ordered by offset
}

#[derive(Debug)]
pub struct SymbolMap {
  params: SymbolTable,
  locals: SymbolTable,
  globals: SymbolTable,
  registers: SymbolTable,
}

impl SymbolMap {
  pub fn new() -> Self {
    let mut this = Self {
      params: SymbolTable::new(),
      locals: SymbolTable::new(),
      globals: SymbolTable::new(),
      registers: SymbolTable::new(),
    };

    // Add all registers
    this.registers.append("AX",    Type::U16,  0, 2);
    this.registers.append("CX",    Type::U16,  2, 2);
    this.registers.append("DX",    Type::U16,  4, 2);
    this.registers.append("BX",    Type::U16,  6, 2);
    this.registers.append("SP",    Type::U16,  8, 2);
    this.registers.append("BP",    Type::U16, 10, 2);
    this.registers.append("SI",    Type::U16, 12, 2);
    this.registers.append("DI",    Type::U16, 14, 2);
    this.registers.append("ES",    Type::U16, 16, 2);
    this.registers.append("CS",    Type::U16, 18, 2);
    this.registers.append("SS",    Type::U16, 20, 2);
    this.registers.append("DS",    Type::U16, 22, 2);
    this.registers.append("IP",    Type::U16, 24, 2);
    this.registers.append("FLAGS", Type::U16, 26, 2);

    this
  }
}

impl SymbolTable {
  fn new() -> Self {
    Self {
      symbols: Vec::new(),
    }
  }

  fn append(&mut self, name: &str, typ: Type, off: i16, size: u16) {
    self.symbols.push(SymbolDef {
      name: name.to_string(),
      typ,
      off,
      size,
    });
  }

  fn coalesce(&mut self) {
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
        // also update the type
        last.typ = infer_type_from_size(last.size);
      } else { // disjoint?
        new_symbols.push(sym.clone());
      }
    }

    self.symbols = new_symbols;
  }

  fn finalize_non_overlaping(&mut self) {
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
  fn get_table(&self, table: Table) -> &SymbolTable {
    match table {
      Table::Param  => &self.params,
      Table::Local  => &self.locals,
      Table::Global => &self.globals,
      Table::Register => &self.registers,
    }
  }

  pub fn find_ref(&self, table: Table, off: i16, sz: u16) -> Option<SymbolRef> {
    // FIXME: This is sorted: can use binary search
    let tbl = self.get_table(table);
    let off: i32 = off.into();
    for (i, sym) in tbl.symbols.iter().enumerate() {
      if sym.start() <= off && off < sym.end() {
        return Some(SymbolRef {
          id: Id {
            table,
            idx: i,
          },
          access_region: Region {
            off: off - sym.start(),
            sz,
          }
        });
      }
    }
    None
  }

  pub fn find_ref_by_name(&self, table: Table, name: &str) -> Option<SymbolRef> {
    let tbl = self.get_table(table);
    for (i, sym) in tbl.symbols.iter().enumerate() {
      if &sym.name == name {
        return Some(SymbolRef {
          id: Id {
            table,
            idx: i,
          },
          access_region: Region {
            off: 0,
            sz: sym.size,
          }
        });
      }
    }
    None
  }
}

fn infer_type_from_size(size: u16) -> Type {
  match size {
    1 => Type::U8,
    2 => Type::U16,
    4 => Type::U32,
    _ => panic!("Unsupported size"),
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

      // let mut f = crate::ir::display::Formatter::new();
      // f.fmt_instr(ir, addr_ref, addr_instr).unwrap();
      // f.fmt_instr(ir, mem_ref, mem_instr).unwrap();
      // println!("{}", f.finish());

      let typ = infer_type_from_size(size);

      let frame_offset = 2;
      if off > 0 {
        let name = format!("_param_{:04x}", off+frame_offset);
        ir.symbols.params.append(&name, typ, off, size);
        var_mem_refs.push((mem_ref, Table::Param, off, size));
      } else {
        let name = format!("_local_{:04x}", -(off+frame_offset));
        ir.symbols.locals.append(&name, typ, off, size);
        var_mem_refs.push((mem_ref, Table::Local, off, size));
      }
    }
  }

  // Coalesce any duplicates or overlaps
  ir.symbols.params.coalesce();
  ir.symbols.locals.coalesce();

  // Update the IR
  for (mem_ref, region, off, sz) in var_mem_refs {
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
    ir.symbols.globals.append(&g.name, g.typ.clone(), g.offset as i16, size as u16);
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
      let Some(sym) = ir.symbols.find_ref(Table::Global, off, size) else {
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
