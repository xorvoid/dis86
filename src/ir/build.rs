use crate::asm::instr;
use crate::binary;
use crate::segoff::{Seg, Off, SegOff};
use crate::config::{self, Config};
use crate::spec;
use crate::ir::*;
use crate::types::{Type, ArraySize};
use std::collections::{HashSet, HashMap};

const DEBUG: bool = false;

fn instr_str(ins: &instr::Instr) -> String {
  crate::asm::intel_syntax::format(ins.addr, Some(ins), &[], false).unwrap()
}

fn operand_is_stack_reg(operand: &instr::Operand) -> bool {
  let instr::Operand::Reg(instr::OperandReg(reg)) = operand else { return false };
  reg == &instr::Reg::SP || reg == &instr::Reg::BP
}

fn simple_binary_operation(opcode: instr::Opcode) -> Option<Opcode> {
  match opcode {
    instr::Opcode::OP_ADD => Some(Opcode::Add),
    instr::Opcode::OP_SUB => Some(Opcode::Sub),
    instr::Opcode::OP_SHL => Some(Opcode::Shl),
    instr::Opcode::OP_SAR => Some(Opcode::Shr),
    instr::Opcode::OP_SHR => Some(Opcode::UShr),
    instr::Opcode::OP_AND => Some(Opcode::And),
    instr::Opcode::OP_OR => Some(Opcode::Or),
    instr::Opcode::OP_XOR => Some(Opcode::Xor),
    _ => None,
  }
}

fn simple_unary_operation(opcode: instr::Opcode) -> Option<Opcode> {
  match opcode {
    instr::Opcode::OP_NEG => Some(Opcode::Neg),
    instr::Opcode::OP_NOT => Some(Opcode::Not),
    _ => None,
  }
}

enum SpecialState {
  PushCS, // CS register was pushed in the last instruction
}

struct IRBuilder<'a> {
  instrs: &'a [instr::Instr],
  cfg: &'a Config,
  spec: &'a spec::Spec<'a>,
  binary: &'a binary::Binary<'a>,

  ir: IR,
  addrmap: HashMap<SegOff, BlockRef>,
  cur: BlockRef,
  special: Option<SpecialState>,

  overlay: bool,
  pin_all: bool,
}

impl<'a> IRBuilder<'a> {
  fn new(cfg: &'a Config, instrs: &'a [instr::Instr], spec: &'a spec::Spec, binary: &'a binary::Binary,
         overlay: bool, pin_all: bool) -> Self {
    let mut this = Self {
      instrs,
      cfg,
      spec,
      binary,

      ir: IR::new(),
      addrmap: HashMap::new(),
      cur: BlockRef(0),
      special: None,

      overlay,
      pin_all,
    };

    // Create and seal the entry block
    let entry = this.new_block("entry");
    this.ir.seal_block(entry);

    // Set initial register values
    this.ir.set_var(instr::Reg::AX, this.cur, Ref::Init(instr::Reg::AX));
    this.ir.set_var(instr::Reg::CX, this.cur, Ref::Init(instr::Reg::CX));
    this.ir.set_var(instr::Reg::DX, this.cur, Ref::Init(instr::Reg::DX));
    this.ir.set_var(instr::Reg::BX, this.cur, Ref::Init(instr::Reg::BX));

    this.ir.set_var(instr::Reg::SP, this.cur, Ref::Init(instr::Reg::SP));
    this.ir.set_var(instr::Reg::BP, this.cur, Ref::Init(instr::Reg::BP));
    this.ir.set_var(instr::Reg::SI, this.cur, Ref::Init(instr::Reg::SI));
    this.ir.set_var(instr::Reg::DI, this.cur, Ref::Init(instr::Reg::DI));

    this.ir.set_var(instr::Reg::ES, this.cur, Ref::Init(instr::Reg::ES));
    this.ir.set_var(instr::Reg::CS, this.cur, Ref::Init(instr::Reg::CS));
    this.ir.set_var(instr::Reg::SS, this.cur, Ref::Init(instr::Reg::SS));
    this.ir.set_var(instr::Reg::DS, this.cur, Ref::Init(instr::Reg::DS));

    this.ir.set_var(instr::Reg::IP, this.cur, Ref::Init(instr::Reg::IP));
    this.ir.set_var(instr::Reg::FLAGS, this.cur, Ref::Init(instr::Reg::FLAGS));

    this
  }

  fn new_block(&mut self, name: &str) -> BlockRef {
    self.ir.push_block(Block::new(name))
  }

  fn get_block(&mut self, effective: SegOff) -> BlockRef {
    *self.addrmap.get(&effective).unwrap()
  }

  fn jump_indirect_targets(&self, ins: &instr::Instr, m: &instr::OperandMem) -> Option<Vec<SegOff>> {
    // Matching jumps of the form: "jmp WORD PTR cs:[bx+0x6d7]"
    if m.sz != instr::Size::Size16 { return None; }
    if m.sreg != instr::Reg::CS    { return None; }
    let Some(_) = m.reg1 else      { return None; };
    if !m.reg2.is_none()           { return None; }
    let Some(off) = m.off else     { return None; };

    // Construct a segoff address to the memory operand
    let addr = SegOff {
      seg: self.spec.start.seg, // code segment
      off: Off(off),
    };

    // Try to find a matching text segment region in config
    let region = self.cfg.text_region_lookup(addr, ins.addr).unwrap_or_else(
      || panic!("Failed to find text section region ({}) for: '{}' at '{}'", addr, instr_str(ins), ins.addr));

    // Unpack the array type
    let Type::Array(basetype, ArraySize::Known(len)) = &region.typ else {
      panic!("Expected text segment region to be an array of known length ({}) for: '{}'", region.name, instr_str(ins));
    };

    // Sanity check type
    if basetype.as_ref() != &Type::U16 {
      panic!("Expected text segment region to be an array with basetype u16, got ({}) for: '{}'", region.name, instr_str(ins));
    }

    // Sanity check range
    if region.start.add_offset(2 * *len as u16) != region.end {
      panic!("Expected text segment region with a length that is consistent with the region size, got ({}) for: '{}'", region.name, instr_str(ins));
    }

    // Grab the bytes to the region from the raw binary
    let dat = self.binary.region(region.start, region.end);

    // Process them into branch targets
    let mut tgts = vec![];
    for i in 0..*len {
      let off = Off(u16::from_le_bytes(dat[2*i .. 2*i+2].try_into().unwrap()));
      tgts.push(SegOff { seg: self.spec.start.seg, off });
    }

    Some(tgts)
  }

  fn jump_targets(&self, ins: &instr::Instr) -> Option<Vec<SegOff>> {
    // Special handling for some indirect jumps
    if ins.opcode == instr::Opcode::OP_JMP {
      if let instr::Operand::Mem(m) = &ins.operands[0] {
        if let Some(targets) = self.jump_indirect_targets(ins, m) {
          return Some(targets);
        }
        // If none.. we want to fall through so we hit the panic below
      }
    }

    // Filter for branch instructions
    let (oper_num, fallthrough) = match &ins.opcode {
      instr::Opcode::OP_JA   => (0, true),
      instr::Opcode::OP_JAE  => (0, true),
      instr::Opcode::OP_JB   => (0, true),
      instr::Opcode::OP_JBE  => (0, true),
      instr::Opcode::OP_JCXZ => (1, true),
      instr::Opcode::OP_JE   => (0, true),
      instr::Opcode::OP_JG   => (0, true),
      instr::Opcode::OP_JGE  => (0, true),
      instr::Opcode::OP_JL   => (0, true),
      instr::Opcode::OP_JLE  => (0, true),
      instr::Opcode::OP_JMP  => (0, false),
      instr::Opcode::OP_JMPF => (0, false),
      instr::Opcode::OP_JNE  => (0, true),
      instr::Opcode::OP_JNO  => (0, true),
      instr::Opcode::OP_JNP  => (0, true),
      instr::Opcode::OP_JNS  => (0, true),
      instr::Opcode::OP_JO   => (0, true),
      instr::Opcode::OP_JP   => (0, true),
      instr::Opcode::OP_JS   => (0, true),
      instr::Opcode::OP_LOOP => (1, true),
      _ => return None,
    };

    let mut targets = vec![];

    match &ins.operands[oper_num] {
      instr::Operand::Rel(rel) => {
        targets.push(ins.rel_addr(rel));
      }
      _ => panic!("Unsupported branch instruction: '{}' | {:?}", instr_str(ins), ins.operands[oper_num]),
    };

    if fallthrough {
      targets.push(ins.end_addr());
    }

    Some(targets)
  }

  fn append_instr(&mut self, typ: Type, opcode: Opcode, operands: Vec<Ref>) -> Ref {
    self.append_instr_with_attrs(typ, Attribute::NONE, opcode, operands)
  }

  fn append_instr_with_attrs(&mut self, typ: Type, attrs: u8, opcode: Opcode, operands: Vec<Ref>) -> Ref {
    let instr = Instr {
      typ,
      attrs,
      opcode,
      operands,
    };
    let idx = self.ir.block_mut(self.cur).instrs.push_back(instr);
    Ref::Instr(self.cur, idx)
  }

  fn append_jmp(&mut self, next: BlockRef) {
    self.ir.block_mut(next).preds.push(self.cur);
    self.append_instr(Type::Void, Opcode::Jmp, vec![Ref::Block(next)]);
  }

  fn append_jne(&mut self, cond: Ref, true_blk: BlockRef, false_blk: BlockRef) {
    self.ir.block_mut(true_blk).preds.push(self.cur);
    self.ir.block_mut(false_blk).preds.push(self.cur);
    self.append_instr(Type::Void, Opcode::Jne, vec![
      cond,
      Ref::Block(true_blk),
      Ref::Block(false_blk)]);
  }

  fn append_jmptbl(&mut self, reg_ref: Ref, targets: Vec<SegOff>) {
    // NOTE: The reg value will have been scaled up to do the memory access, we need to "de-scale" it
    // This is technically not gaurenteed to be safe so we insert some "asserts"
    self.append_instr(Type::Void, Opcode::AssertPos, vec![reg_ref]);
    self.append_instr(Type::Void, Opcode::AssertEven, vec![reg_ref]);
    // Then scale it down..
    let k = self.ir.append_const(1);
    let typ = self.deduce_type_binary(reg_ref, k);
    let idx = self.append_instr(typ, Opcode::UShr, vec![reg_ref, k]);
    let mut opers = vec![idx];
    for tgt in targets {
      let blkref = self.get_block(tgt);
      self.ir.block_mut(blkref).preds.push(self.cur);
      opers.push(Ref::Block(blkref));
    }
    self.append_instr(Type::Void, Opcode::JmpTbl, opers);
  }

  fn switch_blk(&mut self, bref: BlockRef) {
    self.cur = bref;
  }

  fn start_next_blk(&mut self, next: SegOff) {
    let next_bref = self.get_block(next);

    // Make sure the last instruction is a jump or ret
    match self.ir.block(self.cur).instrs.last().map(|ins| ins.opcode) {
      Some(Opcode::Jmp) => (),
      Some(Opcode::Jne) => (),
      Some(Opcode::JmpTbl) => (),
      Some(Opcode::RetFar) => (),
      Some(Opcode::RetNear) => (),
      _ => self.append_jmp(next_bref), // need to append a trailing jump
    }

    // Switch to the next block
    self.switch_blk(next_bref);
    assert!(self.ir.block(self.cur).instrs.empty());
  }
}

/////////////////////////////////////////////////////////////////////////////////////

impl IRBuilder<'_> {
  fn append_asm_src_reg(&mut self, reg: &instr::OperandReg) -> Ref {
    self.ir.get_var(reg.0, self.cur)
  }

  fn append_asm_dst_reg(&mut self, reg: &instr::OperandReg, mut vref: Ref) {
    if self.pin_all {
      vref = self.append_instr_with_attrs(Type::U16, Attribute::PIN, Opcode::Ref, vec![vref]);
    }
    self.ir.set_var(reg.0, self.cur, vref);
  }

  fn compute_mem_address(&mut self, mem: &instr::OperandMem) -> Ref {
    let mut refs = vec![];
    if let Some(reg) = mem.reg2 {
      refs.push(self.ir.get_var(reg, self.cur));
    }
    if let Some(reg) = mem.reg1 {
      refs.push(self.ir.get_var(reg, self.cur));
    }
    if let Some(off) = mem.off {
      refs.push(self.ir.append_const(off as i16));
    }

    let attr = if mem.sreg == instr::Reg::SS { Attribute::STACK_PTR } else { Attribute::NONE };

    if refs.len() == 1 {
      refs[0]
    } else if refs.len() == 2 {
      let typ = self.deduce_type_binary(refs[0], refs[1]);
      self.append_instr_with_attrs(typ, attr, Opcode::Add, vec![refs[0], refs[1]])
    } else {
      assert!(refs.len() == 3);
      let typ = self.deduce_type_binary(refs[0], refs[1]);
      let lhs = self.append_instr_with_attrs(typ, attr, Opcode::Add, vec![refs[0], refs[1]]);
      let typ = self.deduce_type_binary(lhs, refs[2]);
      self.append_instr_with_attrs(typ, attr, Opcode::Add, vec![lhs, refs[2]])
    }
  }

  fn append_asm_src_mem(&mut self, mem: &instr::OperandMem) -> Ref {
    let addr = self.compute_mem_address(mem);
    let seg = self.ir.get_var(mem.sreg, self.cur);

    let (typ, opcode) = match mem.sz {
      instr::Size::Size8 => (Type::U8, Opcode::Load8),
      instr::Size::Size16 => (Type::U16, Opcode::Load16),
      instr::Size::Size32 => (Type::U32, Opcode::Load32),
    };
    self.append_instr_with_attrs(typ, Attribute::MAY_ESCAPE, opcode, vec![seg, addr])
  }

  fn append_asm_dst_mem(&mut self, mem: &instr::OperandMem, vref: Ref) {
    let addr = self.compute_mem_address(mem);
    let seg = self.ir.get_var(mem.sreg, self.cur);

    let opcode = match mem.sz {
      instr::Size::Size8 => Opcode::Store8,
      instr::Size::Size16 => Opcode::Store16,
      _ => panic!("32-bit stores not supported"),
    };

    let mut attr = Attribute::MAY_ESCAPE;
    if self.pin_all {
      attr |= Attribute::PIN;
    }
    self.append_instr_with_attrs(Type::Void, attr, opcode, vec![seg, addr, vref]);
  }

  fn append_asm_src_imm(&mut self, imm: &instr::OperandImm) -> Ref {
    // TODO: Is it okay to sign-ext away the size here??
    // let k: i32 = match imm.sz {
    //   instr::Size::Size8 => (imm.val as i8).into(),
    //   instr::Size::Size16 => (imm.val as i16).into(),
    //   _ => panic!("32-bit immediates not supported"),
    // };
    self.ir.append_const(imm.val as i16)
  }

  fn append_asm_src_rel(&mut self, _rel: &instr::OperandRel) -> Ref {
    panic!("unimpl");
  }

  fn append_asm_src_far(&mut self, _far: &instr::OperandFar) -> Ref {
    panic!("unimpl");
  }

  fn append_asm_src_operand(&mut self, oper: &instr::Operand) -> Ref {
    match oper {
      instr::Operand::Reg(reg) => self.append_asm_src_reg(reg),
      instr::Operand::Mem(mem) => self.append_asm_src_mem(mem),
      instr::Operand::Imm(imm) => self.append_asm_src_imm(imm),
      instr::Operand::Rel(rel) => self.append_asm_src_rel(rel),
      instr::Operand::Far(far) => self.append_asm_src_far(far),
    }
  }

  fn append_asm_dst_operand(&mut self, oper: &instr::Operand, vref: Ref) {
    match oper {
      instr::Operand::Reg(reg) => self.append_asm_dst_reg(reg, vref),
      instr::Operand::Mem(mem) => self.append_asm_dst_mem(mem, vref),
      instr::Operand::Imm(_)   => panic!("Should never have a destination imm"),
      instr::Operand::Rel(_)   => panic!("Should never have a destination rel"),
      instr::Operand::Far(_)   => panic!("Should never have a destination far"),
    };
  }

  fn get_flags(&mut self) -> Ref {
    self.ir.get_var(instr::Reg::FLAGS, self.cur)
  }

  fn set_flags(&mut self, vref: Ref) {
    self.ir.set_var(instr::Reg::FLAGS, self.cur, vref);
  }

  fn append_update_flags(&mut self, vref: Ref) {
    let old_flags = self.get_flags();
    let new_flags = self.append_instr(Type::U16, Opcode::UpdateFlags, vec![old_flags, vref]);
    self.set_flags(new_flags);
  }

  fn pin_register(&mut self, reg: instr::Reg) {
    let vref = self.ir.get_var(reg, self.cur);
    let pin = self.append_instr(Type::Void, Opcode::Pin, vec![vref]);
    self.ir.set_var(reg, self.cur, pin);
  }

  fn return_vals(&mut self) -> Vec<Ref> {
    let ax = self.ir.get_var(instr::Reg::AX, self.cur);
    let dx = self.ir.get_var(instr::Reg::DX, self.cur);
    if let Some(func) = self.spec.func {
      // Use the retval in the config defn
      match &func.ret {
        Type::Void => vec![], // no return value
        Type::U8 | Type::I8 | Type::U16 | Type::I16 => vec![ax],
        Type::U32 | Type::I32  => vec![ax, dx],
        _ => panic!("Unsupported function return type: {}", func.ret),
      }
    } else {
      // Assume worst case: DX:AX return
      vec![ax, dx]
    }
  }

  fn pin_registers(&mut self) {
    // Caller-saved: so don't bother.. unless pin_all
    if self.pin_all {
      self.pin_register(instr::Reg::AX);
      self.pin_register(instr::Reg::CX);
      self.pin_register(instr::Reg::DX);
      self.pin_register(instr::Reg::BX);
      self.pin_register(instr::Reg::ES);
    }

    self.pin_register(instr::Reg::SP);
    self.pin_register(instr::Reg::BP);
    self.pin_register(instr::Reg::SI);
    self.pin_register(instr::Reg::DI);

    self.pin_register(instr::Reg::SS);
    self.pin_register(instr::Reg::DS);

    // ??? Technically we should have this ???
    //self.pin_register(instr::Reg::FLAGS);

    // What would this even mean??
    //self.pin_register(instr::Reg::CS);
    //self.pin_register(instr::Reg::IP);
  }

  fn append_push(&mut self, vref: Ref) {
    let ss = self.ir.get_var(instr::Reg::SS, self.cur);
    let sp = self.ir.get_var(instr::Reg::SP, self.cur);
    let k = self.ir.append_const(2);

    // decrement SP
    let sp = self.append_instr_with_attrs(Type::U16, Attribute::STACK_PTR, Opcode::Sub, vec![sp, k]);
    self.ir.set_var(instr::Reg::SP, self.cur, sp);

    // store to SS:SP
    self.append_instr(Type::Void, Opcode::Store16, vec![ss, sp, vref]);
  }

  fn append_pop(&mut self) -> Ref {
    let ss = self.ir.get_var(instr::Reg::SS, self.cur);
    let sp = self.ir.get_var(instr::Reg::SP, self.cur);
    let k = self.ir.append_const(2);

    let val = self.append_instr(Type::U16, Opcode::Load16, vec![ss, sp]);
    let sp = self.append_instr_with_attrs(Type::U16, Attribute::STACK_PTR, Opcode::Add, vec![sp, k]);
    self.ir.set_var(instr::Reg::SP, self.cur, sp);

    val
  }

  // Try to infer the number of args by looking at the surrounding context for
  // typical stack manipulation associated with the 8086 dos calling conventions
  fn heuristic_infer_call_arguments_by_context(&self, ins: &instr::Instr, warn_msg: &str) -> u16 {
    let idx = offset_from(self.instrs, ins);

    let mut bytes_pushed_before = 0;
    for i in (0..idx).rev() {
      if self.instrs[i].opcode != instr::Opcode::OP_PUSH { break; }
      bytes_pushed_before += 2;
    }

    let mut bytes_cleanup_after = 0;
    if idx+1 < self.instrs.len() {
      let cleanup = &self.instrs[idx+1];
      match cleanup.opcode {
        instr::Opcode::OP_POP => {
          if matches!(cleanup.operands[0], instr::Operand::Reg(instr::OperandReg(instr::Reg::CX))) {
            bytes_cleanup_after = 2;
          }
        }
        instr::Opcode::OP_ADD => {
        let matches =
            matches!(cleanup.operands[0], instr::Operand::Reg(instr::OperandReg(instr::Reg::SP))) &&
            matches!(cleanup.operands[1], instr::Operand::Imm(_));
          if matches {
            let instr::Operand::Imm(imm) = cleanup.operands[1] else { unreachable!() };
            bytes_cleanup_after = imm.val;
          }
        }
        _ => (),
      }
    }

    if bytes_pushed_before >= bytes_cleanup_after {
      assert!(bytes_cleanup_after%2 == 0);
      let n = bytes_cleanup_after/2;
      eprintln!("WARN: {}, inferred {} arg(s)... possibly erroneously", warn_msg, n);
      n
    } else {
      // If more bytes are cleaned up then pushed.. pessimize
      let n = 0;
      eprintln!("WARN: {}, failed to infer, assuming {} arg(s)...very likely erroneously", warn_msg, n);
      n
    }
  }

  fn load_args_from_stack(&mut self, n: u16) -> Vec<Ref> {
    let mut args = vec![];
    let ss = self.ir.get_var(instr::Reg::SS, self.cur);
    let sp = self.ir.get_var(instr::Reg::SP, self.cur);
    for i in 0..n {
      let mut off = sp;
      if i != 0 {
        let k = self.ir.append_const((2*i) as i16);
        off = self.append_instr_with_attrs(Type::U16, Attribute::STACK_PTR, Opcode::Add, vec![sp, k]);
      }
      let val = self.append_instr(Type::U16, Opcode::Load16, vec![ss, off]);
      args.push(val);
    }
    args
  }

  fn save_return_value(&mut self, ret_type: &Type, ret_ref: Ref) {
    match ret_type {
      Type::Void => (), // nothing to do
      Type::U16 => {
        self.ir.set_var(instr::Reg::AX, self.cur, ret_ref);
      }
      Type::U32 | Type::Unknown => {  // Assume worst-case u32 for unknown
        let (upper, lower) = self.append_upper_lower_split(ret_ref);
        self.ir.set_var(instr::Reg::DX, self.cur, upper);
        self.ir.set_var(instr::Reg::AX, self.cur, lower);
      }
      _ => panic!("Unsupported function return type: {}", ret_type),
    }
  }

  fn process_callf_indirect(&mut self, ins: &instr::Instr) {
    let (ret_type, args) = if let Some(indirect) = self.cfg.indirect_lookup(ins.addr) {
      (&indirect.ret, indirect.args)
    } else {
      let nargs = self.heuristic_infer_call_arguments_by_context(ins, &format!(
        "Unknown ptr call from '{}' as binary loc {}", instr_str(ins), ins.addr));
      (&Type::Unknown, nargs)
    };

    let addr = self.append_asm_src_operand(&ins.operands[0]);
    let mut operands = vec![addr];
    operands.append(&mut self.load_args_from_stack(args));
    let ret_ref = self.append_instr(ret_type.clone(), Opcode::CallPtr, operands);
    self.save_return_value(ret_type, ret_ref);
  }

  fn process_call_known(&mut self, func: &config::Func, ins: &instr::Instr) {
    let idx = self.ir.funcs.len();
    self.ir.funcs.push(func.name.to_string());

    let nargs = func.args.unwrap_or_else(|| {
      self.heuristic_infer_call_arguments_by_context(ins,
        &format!("Far call to {} with unknown args", func.name))
    });

    let mut operands = vec![Ref::Func(idx)];
    operands.append(&mut self.load_args_from_stack(nargs));

    let ret_ref = self.append_instr(func.ret.clone(), Opcode::CallArgs, operands);
    self.save_return_value(&func.ret, ret_ref);

    if func.dont_pop_args {
      let sp = self.ir.get_var(instr::Reg::SP, self.cur);
      let k = self.ir.append_const((2*nargs) as i16);
      let sp = self.append_instr_with_attrs(Type::U16, Attribute::STACK_PTR, Opcode::Add, vec![sp, k]);
      self.ir.set_var(instr::Reg::SP, self.cur, sp);
    }
  }

  fn append_regargs(&mut self, regargs: &Option<Vec<instr::Reg>>) {
    let Some(regargs) = regargs else { return };
    for reg in regargs {
      let vref = self.ir.get_var(reg, self.cur);
      let sym = self.ir.symbols.find_ref_by_name(sym::Table::Register, reg.info().name).unwrap();
      self.append_instr_with_attrs(Type::U16, Attribute::PIN, Opcode::WriteVar16, vec![Ref::Symbol(sym), vref]);
    }
  }

  fn process_call_segoff(&mut self, addr: SegOff, mode: config::CallMode, ins: &instr::Instr) {
    if let Some(func) = self.cfg.func_lookup(addr) {
      // Known function
      if func.mode != mode {
        panic!("Found function but it's call mode doesn't match! Expected {:?}, Got {:?}", mode, func.mode);
      }
      self.append_regargs(&func.regargs);
      self.process_call_known(func, ins);
    } else {
      // Unknown function
      let ret_type = Type::Unknown;
      let nargs = self.heuristic_infer_call_arguments_by_context(ins,
                    &format!("Unknown call to {}", addr));

      // FIXME: Can we do unwrap_normal() ??
      println!("addr: {}", addr);
      let seg = match addr.seg {
        Seg::Normal(num) => self.ir.append_const(num as i16),
        Seg::Overlay(num) => self.ir.append_const(-(num as i16)),
      };
      //let seg = self.ir.append_const(addr.seg.unwrap_normal() as i16);
      let off = self.ir.append_const(addr.off.0 as i16);

      let mut operands = vec![seg, off];
      operands.append(&mut self.load_args_from_stack(nargs));
      let opcode = match mode {
        config::CallMode::Far => Opcode::CallFar,
        config::CallMode::Near => Opcode::CallNear,
      };
      let ret_ref = self.append_instr(ret_type.clone(), opcode, operands);
      self.save_return_value(&ret_type, ret_ref);
    }
  }

  fn process_callf(&mut self, ins: &instr::Instr) {
    let instr::Operand::Far(far) = &ins.operands[0] else {
      return self.process_callf_indirect(ins);
    };
    let seg = if self.overlay {
      // Far calls from overlays need to be remapped
      self.binary.remap_to_segment(far.seg)
    } else {
      // Otherwise: Normal
      Seg::Normal(far.seg)
    };
    //let seg = Seg::Normal(far.seg);
    let addr = SegOff { seg, off: Off(far.off) };
    self.process_call_segoff(addr, config::CallMode::Far, ins);
  }

  fn process_calln(&mut self, ins: &instr::Instr, cs_pushed: bool) {
    let instr::Operand::Rel(rel) = &ins.operands[0] else {
      panic!("Expected near call to have relative operand");
    };
    let addr = ins.rel_addr(rel);
    if cs_pushed {
      // If CS was previously pushed, then the function returns
      // via "retf", which means its juat a far-call in disguise.
      // So, POP CS back off the stack and let's pretend it's a normal
      // far call
      self.append_pop();
      self.process_call_segoff(addr, config::CallMode::Far, ins);
    } else {
      // Otherwise, we assume it's actually a near call
      self.process_call_segoff(addr, config::CallMode::Near, ins);
    }
  }

  fn append_cond_jump(&mut self, ins: &instr::Instr, compare_opcode: Opcode) {
    let instr::Operand::Rel(rel) = &ins.operands[0] else { panic!("Expected relative offset operand for JMP") };

    let false_blk = self.get_block(ins.end_addr());
    let true_blk = self.get_block(ins.rel_addr(rel));

    let flags = self.get_flags();
    let cond = self.append_instr(Type::U16, compare_opcode, vec![flags]);

    self.append_jne(cond, true_blk, false_blk);
  }

  fn append_cond_set(&mut self, ins: &instr::Instr, compare_opcode: Opcode) {
    let flags = self.get_flags();
    let cond = self.append_instr(Type::U8, compare_opcode, vec![flags]);
    self.append_asm_dst_operand(&ins.operands[0], cond);
  }

  fn append_upper_lower_split(&mut self, r: Ref) -> (Ref, Ref) {
    let upper = self.append_instr(Type::U16, Opcode::Upper16, vec![r]);
    let lower = self.append_instr(Type::U16, Opcode::Lower16, vec![r]);
    (upper, lower)
  }

  fn deduce_type_unary(&mut self, a: Ref) -> Type {
    //println!("a: {:?}", a);
    match a {
      Ref::Instr(_, _) => self.ir.instr(a).unwrap().typ.clone(),
      Ref::Init(_) => Type::U16,
      _ => Type::Unknown,
    }
  }

  fn deduce_type_binary(&mut self, a: Ref, b: Ref) -> Type {
    let a_typ = self.deduce_type_unary(a);
    let b_typ = self.deduce_type_unary(b);
    if a_typ == b_typ { a_typ }
    else if a_typ == Type::Unknown { b_typ }
    else if b_typ == Type::Unknown { a_typ }
    else { Type::Unknown }
  }

  fn append_asm_instr(&mut self, ins: &instr::Instr) {
    //println!("## {}", intel_syntax::format(ins, &[], false).unwrap());
    assert!(ins.rep.is_none());

    let special = self.special.take();

    // process simple unary operations
    if let Some(opcode) = simple_unary_operation(ins.opcode) {
      let a = self.append_asm_src_operand(&ins.operands[0]);
      let typ = self.deduce_type_unary(a);
      let vref = self.append_instr(typ, opcode, vec![a]);
      self.append_asm_dst_operand(&ins.operands[0], vref);
      return;
    }

    // process simple binary operations
    if let Some(opcode) = simple_binary_operation(ins.opcode) {
      let attr = if operand_is_stack_reg(&ins.operands[0]) { Attribute::STACK_PTR } else { Attribute::NONE };
      let a = self.append_asm_src_operand(&ins.operands[0]);
      let b = self.append_asm_src_operand(&ins.operands[1]);
      let typ = self.deduce_type_binary(a, b);
      let vref = self.append_instr_with_attrs(typ, attr, opcode, vec![a, b]);
      self.append_asm_dst_operand(&ins.operands[0], vref);
      self.append_update_flags(vref);
      return;
    }

    // handle less standard operations
    match &ins.opcode {
      instr::Opcode::OP_NOP => (),
      instr::Opcode::OP_SBB => {
        // FIXME: WE DON'T HAVE  A GREAT WAY TO IMPL SBC AT THE MOMENT BECAUSE IT NEEDS TO CORRECTLY
        // CONSUME THE CARRY FLAG FROM A PREVIOUS SUBTRACT THAT GENERATED A BORROW. BUT, AT THE MOMENT,
        // WE USE "UpdateFlags" AS A PLACEHOLDER FOR ARBITRARY FLAGS UPDATES WITHOUT BREAKING DOWN INTO
        // THE SPECIFIC FLAGS AFFECTED. SO FOR NOW, WE'LL JAM IT THROUGH "Unimpl" AND MAKE THE USER
        // DEAL WITH IT MANUALLY
        let a = self.append_asm_src_operand(&ins.operands[0]);
        let b = self.append_asm_src_operand(&ins.operands[1]);
        let typ = self.deduce_type_binary(a, b);
        let vref = self.append_instr(typ, Opcode::Unimpl, vec![a, b]);
        self.append_asm_dst_operand(&ins.operands[0], vref);
        self.append_update_flags(vref);
      }
      instr::Opcode::OP_RCL => {
        // FIXME: SIMILAR FOR RCL ... WE SIMPLY DON'T HAVE A GREAT WAY TO CAPTURE THE SEMANTICS OF ROTATION
        // INCLUDING THE CARRY FLAG!!!
        let a = self.append_asm_src_operand(&ins.operands[0]);
        let b = self.append_asm_src_operand(&ins.operands[1]);
        let typ = self.deduce_type_binary(a, b);
        let vref = self.append_instr(typ, Opcode::Unimpl, vec![a, b]);
        self.append_asm_dst_operand(&ins.operands[0], vref);
        self.append_update_flags(vref);
      }
      instr::Opcode::OP_ADC => {
        // FIXME: SIMILAR FOR ADC ... WE SIMPLY DON'T HAVE A GREAT WAY TO CAPTURE THE SEMANTICS
        let a = self.append_asm_src_operand(&ins.operands[0]);
        let b = self.append_asm_src_operand(&ins.operands[1]);
        let typ = self.deduce_type_binary(a, b);
        let vref = self.append_instr(typ, Opcode::Unimpl, vec![a, b]);
        self.append_asm_dst_operand(&ins.operands[0], vref);
        self.append_update_flags(vref);
      }
      instr::Opcode::OP_LOOP => {
        // Step 1: Update CX := CX - 1
        let cx = self.append_asm_src_operand(&ins.operands[0]);
        let one = self.ir.append_const(1);
        let cx = self.append_instr(Type::U16, Opcode::Sub, vec![cx, one]);
        self.append_asm_dst_operand(&ins.operands[0], cx);

        // Step 2: Jump when CX != 0
        let instr::Operand::Rel(rel) = &ins.operands[1] else { panic!("Expected relative offset operand for LOOP") };

        let false_blk = self.get_block(ins.end_addr());
        let true_blk = self.get_block(ins.rel_addr(rel));

        let z = self.ir.append_const(0);
        let cond = self.append_instr(Type::U16, Opcode::Neq, vec![cx, z]);

        self.append_jne(cond, true_blk, false_blk);
      }
      instr::Opcode::OP_XCHG => {
        let a = self.append_asm_src_operand(&ins.operands[0]);
        let b = self.append_asm_src_operand(&ins.operands[1]);
        self.append_asm_dst_operand(&ins.operands[0], b);
        self.append_asm_dst_operand(&ins.operands[1], a);
      }
      instr::Opcode::OP_PUSH => {
        if matches!(ins.operands[0], instr::Operand::Reg(instr::OperandReg(instr::Reg::CS))) {
          assert!(self.special.is_none());
          self.special = Some(SpecialState::PushCS);
        }
        let a = self.append_asm_src_operand(&ins.operands[0]);
        self.append_push(a);
      }
      instr::Opcode::OP_POP => {
        let vref = self.append_pop();
        self.append_asm_dst_operand(&ins.operands[0], vref);
      }
      instr::Opcode::OP_LEAVE => {
        // mov sp, bp
        let vref = self.ir.get_var(instr::Reg::BP, self.cur);
        self.ir.set_var(instr::Reg::SP, self.cur, vref);
        // pop bp
        let vref = self.append_pop();
        self.ir.set_var(instr::Reg::BP, self.cur, vref);
      }
      instr::Opcode::OP_RETF => {
        self.pin_registers();
        let ret_vals = self.return_vals();
        self.append_instr(Type::Void, Opcode::RetFar, ret_vals);

      }
      instr::Opcode::OP_RET => {
        self.pin_registers();
        let ret_vals = self.return_vals();
        self.append_instr(Type::Void, Opcode::RetNear, ret_vals);
      }
      instr::Opcode::OP_MOV => {
        let vref = self.append_asm_src_operand(&ins.operands[1]);
        self.append_asm_dst_operand(&ins.operands[0], vref);
      }
      instr::Opcode::OP_IMUL_TRUNC => {
        // IMUL has many forms, let's be conservative for now and hard sanity check the version we need
        assert!(
          matches!(ins.operands[0], instr::Operand::Reg(_)) &&
          matches!(ins.operands[1], instr::Operand::Reg(_)) &&
          matches!(ins.operands[2], instr::Operand::Imm(_)));

        let lhs = self.append_asm_src_operand(&ins.operands[1]);
        let rhs = self.append_asm_src_operand(&ins.operands[2]);
        let typ = self.deduce_type_binary(lhs, rhs);
        let res = self.append_instr(typ, Opcode::IMul, vec![lhs, rhs]);
        self.append_asm_dst_operand(&ins.operands[0], res);
      }
      instr::Opcode::OP_IMUL => {
        // IMUL has many forms, let's be conservative for now and hard sanity check the version we need
        assert!(
          ins.operands.len() == 3 &&
          matches!(ins.operands[0], instr::Operand::Reg(instr::OperandReg(instr::Reg::DX))) &&
          matches!(ins.operands[1], instr::Operand::Reg(instr::OperandReg(instr::Reg::AX))));

        let lhs = self.append_asm_src_operand(&ins.operands[1]);
        let rhs = self.append_asm_src_operand(&ins.operands[2]);
        let res = self.append_instr(Type::U32, Opcode::IMul, vec![lhs, rhs]);
        let (upper, lower) = self.append_upper_lower_split(res);
        self.append_asm_dst_operand(&ins.operands[0], upper);
        self.append_asm_dst_operand(&ins.operands[1], lower);
      }
      instr::Opcode::OP_MUL => {
        // Sanity check the form we have
        assert!(
          matches!(ins.operands[0], instr::Operand::Reg(instr::OperandReg(instr::Reg::DX))) &&
          matches!(ins.operands[1], instr::Operand::Reg(instr::OperandReg(instr::Reg::AX))) &&
          matches!(ins.operands[2], instr::Operand::Reg(_)));

        let lhs = self.append_asm_src_operand(&ins.operands[1]);
        let rhs = self.append_asm_src_operand(&ins.operands[2]);
        let res = self.append_instr(Type::U32, Opcode::UMul, vec![lhs, rhs]);
        let (upper, lower) = self.append_upper_lower_split(res);
        self.append_asm_dst_operand(&ins.operands[0], upper);
        self.append_asm_dst_operand(&ins.operands[1], lower);
     }
      instr::Opcode::OP_INC => {
        let attr = if operand_is_stack_reg(&ins.operands[0]) { Attribute::STACK_PTR } else { Attribute::NONE };
        let one = self.ir.append_const(1);
        let vref = self.append_asm_src_operand(&ins.operands[0]);
        let typ = self.deduce_type_binary(vref, one);
        let vref = self.append_instr_with_attrs(typ, attr, Opcode::Add, vec![vref, one]);
        self.append_asm_dst_operand(&ins.operands[0], vref);
        self.append_update_flags(vref);
      }
      instr::Opcode::OP_DEC => {
        let attr = if operand_is_stack_reg(&ins.operands[0]) { Attribute::STACK_PTR } else { Attribute::NONE };
        let one = self.ir.append_const(1);
        let vref = self.append_asm_src_operand(&ins.operands[0]);
        let typ = self.deduce_type_binary(vref, one);
        let vref = self.append_instr_with_attrs(typ, attr, Opcode::Sub, vec![vref, one]);
        self.append_asm_dst_operand(&ins.operands[0], vref);
        self.append_update_flags(vref);
      }
      instr::Opcode::OP_JMP => {
        match &ins.operands[0] {
          instr::Operand::Mem(m) => {
            // Special handling for some indirect jumps
            let Some(targets) = self.jump_indirect_targets(ins, m) else {
              panic!("Indirect jump form not currently supported for '{}'", instr_str(ins));
            };
            let Some(reg) = m.reg1 else {
              panic!("Shouldn't fail.. should be checked by jump_indirect_targets already");
            };
            let reg_ref = self.ir.get_var(reg, self.cur);
            self.append_jmptbl(reg_ref, targets);
          }
          instr::Operand::Rel(rel) => {
            let blkref = self.get_block(ins.rel_addr(rel));
            self.append_jmp(blkref);
          }
          _ => panic!("Unsupported JMP operand for '{}'", instr_str(ins)),
        }
      }

      instr::Opcode::OP_JCXZ  => {
        let instr::Operand::Rel(rel) = &ins.operands[1] else { panic!("Expected relative offset operand for JCXZ") };

        let false_blk = self.get_block(ins.end_addr());
        let true_blk = self.get_block(ins.rel_addr(rel));

        let cx = self.append_asm_src_operand(&ins.operands[0]);
        let z = self.ir.append_const(0);
        let cond = self.append_instr(Type::U16, Opcode::Eq, vec![cx, z]);

        self.append_jne(cond, true_blk, false_blk);
      }

      instr::Opcode::OP_JS  => {
        let instr::Operand::Rel(rel) = &ins.operands[0] else { panic!("Expected relative offset operand for JS") };

        let false_blk = self.get_block(ins.end_addr());
        let true_blk = self.get_block(ins.rel_addr(rel));

        let flags = self.get_flags();
        let sign = self.append_instr(Type::U16, Opcode::SignFlags, vec![flags]);
        let z = self.ir.append_const(0);
        let cond = self.append_instr(Type::U16, Opcode::Neq, vec![sign, z]);

        self.append_jne(cond, true_blk, false_blk);
      }

      instr::Opcode::OP_JNS  => {
        let instr::Operand::Rel(rel) = &ins.operands[0] else { panic!("Expected relative offset operand for JNS") };

        let false_blk = self.get_block(ins.end_addr());
        let true_blk = self.get_block(ins.rel_addr(rel));

        let flags = self.get_flags();
        let sign = self.append_instr(Type::U16, Opcode::SignFlags, vec![flags]);
        let z = self.ir.append_const(0);
        let cond = self.append_instr(Type::U16, Opcode::Eq, vec![sign, z]);

        self.append_jne(cond, true_blk, false_blk);
      }

      instr::Opcode::OP_JE  => self.append_cond_jump(ins, Opcode::EqFlags),
      instr::Opcode::OP_JNE => self.append_cond_jump(ins, Opcode::NeqFlags),
      instr::Opcode::OP_JG  => self.append_cond_jump(ins, Opcode::GtFlags),
      instr::Opcode::OP_JGE => self.append_cond_jump(ins, Opcode::GeqFlags),
      instr::Opcode::OP_JL  => self.append_cond_jump(ins, Opcode::LtFlags),
      instr::Opcode::OP_JLE => self.append_cond_jump(ins, Opcode::LeqFlags),
      instr::Opcode::OP_JA  => self.append_cond_jump(ins, Opcode::UGtFlags),
      instr::Opcode::OP_JAE => self.append_cond_jump(ins, Opcode::UGeqFlags),
      instr::Opcode::OP_JB  => self.append_cond_jump(ins, Opcode::ULtFlags),
      instr::Opcode::OP_JBE => self.append_cond_jump(ins, Opcode::ULeqFlags),

      instr::Opcode::OP_SETE  => self.append_cond_set(ins, Opcode::EqFlags),
      instr::Opcode::OP_SETNE => self.append_cond_set(ins, Opcode::NeqFlags),
      instr::Opcode::OP_SETG  => self.append_cond_set(ins, Opcode::GtFlags),
      instr::Opcode::OP_SETGE => self.append_cond_set(ins, Opcode::GeqFlags),
      instr::Opcode::OP_SETL  => self.append_cond_set(ins, Opcode::LtFlags),
      instr::Opcode::OP_SETLE => self.append_cond_set(ins, Opcode::LeqFlags),
      instr::Opcode::OP_SETA  => self.append_cond_set(ins, Opcode::UGtFlags),
      instr::Opcode::OP_SETAE => self.append_cond_set(ins, Opcode::UGeqFlags),
      instr::Opcode::OP_SETB  => self.append_cond_set(ins, Opcode::ULtFlags),
      instr::Opcode::OP_SETBE => self.append_cond_set(ins, Opcode::ULeqFlags),

      instr::Opcode::OP_CALLF => {
        self.process_callf(ins);
      }
      instr::Opcode::OP_CALL => {
        let cs_pushed = matches!(special, Some(SpecialState::PushCS));
        self.process_calln(ins, cs_pushed);
      }
      instr::Opcode::OP_INT => {
        let num = self.append_asm_src_operand(&ins.operands[0]);
        self.append_instr(Type::Void, Opcode::Int, vec![num]);
      }
      instr::Opcode::OP_LEA => {
        let instr::Operand::Mem(mem) = &ins.operands[1] else {
          panic!("Expected LEA to have a mem operand");
        };

        let addr = self.compute_mem_address(mem);

        // NOTE: Another weird 8086 quirk, the sreg here is meaningless (lol).
        //       LEA is only computing a 16-bit offset, not a full seg:off address
        //       This is only the case because LEA strictly DOES NOT dereference.

        self.append_asm_dst_operand(&ins.operands[0], addr);
      }
      instr::Opcode::OP_LES => {
        let vref = self.append_asm_src_operand(&ins.operands[2]);
        let (upper, lower) = self.append_upper_lower_split(vref);
        self.append_asm_dst_operand(&ins.operands[0], upper);
        self.append_asm_dst_operand(&ins.operands[1], lower);
      }
      instr::Opcode::OP_TEST => {
        let a = self.append_asm_src_operand(&ins.operands[0]);
        let b = self.append_asm_src_operand(&ins.operands[1]);
        let typ = self.deduce_type_binary(a, b);
        let vref = self.append_instr(typ, Opcode::And, vec![a, b]);
        self.append_update_flags(vref);
      }
      instr::Opcode::OP_CMP => {
        let a = self.append_asm_src_operand(&ins.operands[0]);
        let b = self.append_asm_src_operand(&ins.operands[1]);
        let typ = self.deduce_type_binary(a, b);
        let vref = self.append_instr(typ, Opcode::Sub, vec![a, b]);
        self.append_update_flags(vref);
      }
      instr::Opcode::OP_CWD => {
        let src = self.append_asm_src_operand(&ins.operands[1]);
        let vref = self.append_instr(Type::U32, Opcode::SignExtTo32, vec![src]);
        let upper = self.append_instr(Type::U16, Opcode::Upper16, vec![vref]);
        self.append_asm_dst_operand(&ins.operands[0], upper);
      }
      instr::Opcode::OP_DIV => {
        let upper_in = self.append_asm_src_operand(&ins.operands[0]);
        let lower_in = self.append_asm_src_operand(&ins.operands[1]);
        let divisor = self.append_asm_src_operand(&ins.operands[2]);
        let dividend = self.append_instr(Type::U32, Opcode::Make32, vec![upper_in, lower_in]);
        let quotient = self.append_instr(Type::U32, Opcode::UDiv, vec![dividend, divisor]);
        let (upper_out, lower_out) = self.append_upper_lower_split(quotient);
        self.append_asm_dst_operand(&ins.operands[0], upper_out);
        self.append_asm_dst_operand(&ins.operands[1], lower_out);
      }
      instr::Opcode::OP_STOS => {
        let src = self.append_asm_src_operand(&ins.operands[1]);
        self.append_asm_dst_operand(&ins.operands[0], src);
      }
      instr::Opcode::OP_LODS => {
        let src = self.append_asm_src_operand(&ins.operands[1]);
        self.append_asm_dst_operand(&ins.operands[0], src);
      }
      instr::Opcode::OP_STI => {
        self.append_instr(Type::Void, Opcode::Unimpl, vec![]);
      }
      instr::Opcode::OP_CLI => {
        self.append_instr(Type::Void, Opcode::Unimpl, vec![]);
      }
      instr::Opcode::OP_IN => {
        self.append_instr(Type::U8, Opcode::Unimpl, vec![]);
      }
      instr::Opcode::OP_OUT => {
        self.append_instr(Type::Void, Opcode::Unimpl, vec![]);
      }
      instr::Opcode::OP_CLD => {
        self.append_instr(Type::Void, Opcode::Unimpl, vec![]);
      }
      _ => panic!("Unimpl opcode: {:?}", ins.opcode),
    }
  }

  fn build(&mut self) {
    // Step 1: Infer basic-block boundaries
    let mut block_start = HashSet::new();
    let mut last_was_jump = false;
    for ins in self.instrs {
      if last_was_jump {
        block_start.insert(ins.addr);
        last_was_jump = false;
      }
      let Some(targets) = self.jump_targets(ins) else { continue };
      for tgt in targets {
        block_start.insert(tgt);
      }
      last_was_jump = true;
    }

    // Step 2: Create all the blocks we should encounter
    let mut addr_ordered: Vec<_> = block_start.iter().collect();
    addr_ordered.sort();
    for addr in addr_ordered {
      let bref = self.new_block(&format!("addr_{}", addr.off));
      self.addrmap.insert(*addr, bref);
    }

    // Step 3: iterate each instruction, building each block
    for ins in self.instrs {
      if DEBUG { println!("DEBUG: {}", instr_str(ins)); }
      if block_start.get(&ins.addr).is_some() {
        self.start_next_blk(ins.addr);
      }
      self.append_asm_instr(ins);
    }

    // Step 4: walk blocks and seal them
    for blkref in self.ir.iter_blocks() {
      if !self.ir.block(blkref).sealed {
        self.ir.seal_block(blkref);
      }
    }
  }
}

/// Returns the offset of `elt` into `slice`.
fn offset_from<T>(slice: &[T], elt: &T) -> usize {
  let elt_addr = elt as *const T as usize;
  let slice_addr = slice.as_ptr() as usize;
  if elt_addr < slice_addr { panic!("elt not in slice!"); }

  let off = elt_addr - slice_addr;
  if (off % std::mem::size_of::<T>()) != 0 { panic!("offset is not a multiple of element size"); }

  let n = off / std::mem::size_of::<T>();
  if n >= slice.len() { panic!("index is out of range"); }

  // sanity
  debug_assert!(&slice[n] as *const _ == elt as *const _);

  n
}

impl IR {
  pub fn from_instrs(instrs: &[instr::Instr], cfg: &Config, spec: &spec::Spec<'_>, binary: &binary::Binary, overlay: bool, pin_all: bool) -> IR {
    let mut bld = IRBuilder::new(cfg, instrs, spec, binary, overlay, pin_all);
    bld.build();
    bld.ir
  }
}
