use crate::decomp::ir::*;
use std::collections::HashMap;
use std::fmt;

struct Emit<'a> {
  ir: &'a IR,
  f: &'a mut dyn fmt::Write,
  vars: HashMap<Ref, usize>,
  next: usize,
}

impl Emit<'_> {
  fn refstr(&mut self, r: Ref) -> String {
    if let Some(k) = self.ir.lookup_const(r) {
      if k > 128 {
        return format!("0x{:x}", k);
      } else {
        return format!("{}", k);
      }
    }
    if let Some((name, num)) = self.ir.names.get(&r) {
      return format!("{}{}", name, num);
    }
    let num = match self.vars.get(&r) {
      Some(num) => *num,
      None => {
        let n = self.next;
        self.next += 1;
        self.vars.insert(r, n);
        n
      }
    };
    return format!("t{}", num);
  }

  fn symbol(&mut self, sref: Ref, size: u32) -> String {
    let Ref::Symbol(symref) = sref else { panic!("Expected symbol ref") };
    let sym = self.ir.symbols.symbol(symref);
    if symref.off != 0 {
      format!("*(u{}*)((u8*)&{}+{})", 8*size, sym.name, symref.off)
    } else if sym.size != size {
      format!("*(u{}*)&{}", 8*size, sym.name)
    } else {
      format!("{}", sym.name)
    }
  }

  fn label(&mut self, name: &str) -> fmt::Result {
    writeln!(self.f, "\n{}:", name)
  }

  fn jmp(&mut self, name: &str) -> fmt::Result {
    writeln!(self.f, "  goto {};", name)
  }

  fn jne(&mut self, cond: Ref, tblk: &str, fblk: &str) -> fmt::Result {
    let c = self.refstr(cond);
    writeln!(self.f, "  if ({}) goto {};", c, tblk)?;
    writeln!(self.f, "  else goto {};", fblk)?;
    Ok(())
  }

  fn func(&mut self, name: &str, r: Ref, operands: &[Ref]) -> fmt::Result {
    let dst = self.refstr(r);
    write!(self.f, "  {} = {}(", dst, name)?;
    for (i, arg) in operands.iter().enumerate() {
      if i != 0 { write!(self.f, ", ")? }
      let s = self.refstr(*arg);
      write!(self.f, "{}", s)?;
    }
    writeln!(self.f, ");")
  }

  fn store(&mut self, name: &str, r: Ref) -> fmt::Result {
    let instr = self.ir.instr(r).unwrap();
    let n = instr.operands.len();
    write!(self.f, "  {}(", name)?;
    for (i, arg) in instr.operands[..n-1].iter().enumerate() {
      if i != 0 { write!(self.f, ", ")? }
      let s = self.refstr(*arg);
      write!(self.f, "{}", s)?;
    }
    let s = self.refstr(instr.operands[n-1]);
    writeln!(self.f, ") = {};", s)
  }

  fn binop(&mut self, operator: &str, r: Ref) -> fmt::Result {
    let dst = self.refstr(r);
    let instr = self.ir.instr(r).unwrap();
    let lhs = self.refstr(instr.operands[0]);
    let rhs = self.refstr(instr.operands[1]);
    writeln!(self.f, "  {} = {} {} {};", dst, lhs, operator, rhs)
  }
}

pub fn generate(ir: &IR, f: &mut dyn fmt::Write) -> fmt::Result {
  let mut emit = Emit {
    ir, f, vars: HashMap::new(), next: 0,
  };

  for b in 0..ir.blocks.len() {
    let blk = &ir.blocks[b];
    emit.label(&blk.name)?;
    for i in blk.instrs.range() {
      let r = Ref::Instr(BlockRef(b), i);
      let instr = ir.instr(r).unwrap();
      match instr.opcode {
        Opcode::Nop => (),
        Opcode::Pin => panic!("Unimpl Pin"),
        Opcode::Ref => panic!("Unimpl Ref"),
        Opcode::Phi => {
          //return Ok(());
          panic!("Unimpl Phi");
        }
        Opcode::Add => emit.binop("+", r)?,
        Opcode::Sub => emit.binop("-", r)?,
        Opcode::Shl => emit.binop("<<", r)?,
        Opcode::And => emit.binop("&", r)?,
        Opcode::Xor => emit.binop("^", r)?,
        Opcode::Load8 => panic!("Unimpl Load8"),
        Opcode::Load16 => emit.func("*PTR_16", r, &instr.operands)?,
        Opcode::Load32 => panic!("Unimpl Load32"),
        Opcode::Store8 => panic!("Unimpl Store8"),
        Opcode::Store16 => emit.store("*PTR_16", r)?,

        Opcode::ReadVar8 => panic!("Unimpl ReadVar8"),
        Opcode::ReadVar16 => {
          let dst = emit.refstr(r);
          let src = emit.symbol(instr.operands[0], 2);
          writeln!(emit.f, "  {} = {};", dst, src)?;
        }
        Opcode::ReadVar32 => {
          let dst = emit.refstr(r);
          let src = emit.symbol(instr.operands[0], 4);
          writeln!(emit.f, "  {} = {};", dst, src)?;
        }
        Opcode::WriteVar8 => panic!("Unimpl WriteVar8"),
        Opcode::WriteVar16 => {
          let dst = emit.symbol(instr.operands[0], 2);
          let src = emit.refstr(instr.operands[1]);
          writeln!(emit.f, "  {} = {};", dst, src)?;
        }
        Opcode::ReadArr8 => panic!("Unimpl ReadArr8"),
        Opcode::ReadArr16 => panic!("Unimpl ReadArr16"),
        Opcode::ReadArr32 => panic!("Unimpl ReadArr32"),
        Opcode::WriteArr8 => panic!("Unimpl WriteArr8"),
        Opcode::WriteArr16 => panic!("Unimpl WriteArr16"),
        Opcode::Lower16 => {
          let dst = emit.refstr(r);
          let arg = emit.refstr(instr.operands[0]);
          writeln!(emit.f, "  {} = (u16)({});", dst, arg)?;
        }
        Opcode::Upper16 => {
          let dst = emit.refstr(r);
          let arg = emit.refstr(instr.operands[0]);
          writeln!(emit.f, "  {} = (u16)({} >> 16);", dst, arg)?;
        }
        Opcode::UpdateFlags => panic!("Unimpl UpdateFlags"),
        Opcode::EqFlags => panic!("Unimpl EqFlags"),
        Opcode::NeqFlags => panic!("Unimpl NeqFlags"),
        Opcode::GtFlags => panic!("Unimpl GtFlags"),
        Opcode::GeqFlags => panic!("Unimpl GeqFlags"),
        Opcode::LtFlags => panic!("Unimpl LtFlags"),
        Opcode::LeqFlags => panic!("Unimpl LeqFlags"),
        Opcode::Eq => emit.binop("==", r)?,
        Opcode::Neq => emit.binop("!=", r)?,
        Opcode::Gt => panic!("Unimpl Gt"),
        Opcode::Geq => panic!("Unimpl Geq"),
        Opcode::Lt => panic!("Unimpl Lt"),
        Opcode::Leq => panic!("Unimpl Leq"),
        Opcode::Call => panic!("Unimpl Call"),
        Opcode::CallArgs => {
          let Ref::Func(idx) = instr.operands[0] else { panic!("Expected function reference for first operand of callargs") };
          let name = &ir.funcs[idx];
          emit.func(name, r, &instr.operands[1..])?;
        }
        Opcode::Ret => panic!("Unimpl Ret"),
        Opcode::Jmp => {
          let Ref::Block(BlockRef(b)) = instr.operands[0] else { panic!("Expected blockref for first operand of Jmp") };
          emit.jmp(&ir.blocks[b].name)?;
        }
        Opcode::Jne => {
          let Ref::Block(BlockRef(t)) = instr.operands[1] else { panic!("Expected blockref for second operand of Jne") };
          let Ref::Block(BlockRef(f)) = instr.operands[2] else { panic!("Expected blockref for third operand of Jne") };
          emit.jne(instr.operands[0], &ir.blocks[t].name, &ir.blocks[f].name)?;
        }
      }
    }
  }
  Ok(())
}
