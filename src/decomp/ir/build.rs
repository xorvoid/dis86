use crate::instr;
use crate::segoff::SegOff;
use crate::decomp::config::Config;
use crate::decomp::ir::*;
use crate::decomp::types::Type;
use std::collections::{HashSet, HashMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Address(usize);

fn simple_binary_operation(opcode: instr::Opcode) -> Option<Opcode> {
  match opcode {
    instr::Opcode::OP_ADD => Some(Opcode::Add),
    instr::Opcode::OP_SUB => Some(Opcode::Sub),
    instr::Opcode::OP_SHL => Some(Opcode::Shl),
    instr::Opcode::OP_AND => Some(Opcode::And),
    instr::Opcode::OP_OR => Some(Opcode::Or),
    instr::Opcode::OP_XOR => Some(Opcode::Xor),
    _ => None,
  }
}

fn jump_target(ins: &instr::Instr) -> Option<Address> {
  // Filter for branch instructions
  match &ins.opcode {
    instr::Opcode::OP_JA => (),
    instr::Opcode::OP_JAE => (),
    instr::Opcode::OP_JB => (),
    instr::Opcode::OP_JBE => (),
    instr::Opcode::OP_JCXZ => (),
    instr::Opcode::OP_JE => (),
    instr::Opcode::OP_JG => (),
    instr::Opcode::OP_JGE => (),
    instr::Opcode::OP_JL => (),
    instr::Opcode::OP_JLE => (),
    instr::Opcode::OP_JMP => (),
    instr::Opcode::OP_JMPF => (),
    instr::Opcode::OP_JNE => (),
    instr::Opcode::OP_JNO => (),
    instr::Opcode::OP_JNP => (),
    instr::Opcode::OP_JNS => (),
    instr::Opcode::OP_JO => (),
    instr::Opcode::OP_JP => (),
    instr::Opcode::OP_JS => (),
    _ => return None,
  }

  // target should be first operand
  let tgt = match &ins.operands[0] {
    instr::Operand::Rel(rel) => {
      let effective = ins.rel_addr(rel);
      Address(effective.abs().into())
    }
    _ => panic!("Unsupported branch operand"),
  };

  Some(tgt)
}

enum SpecialState {
  PushCS, // CS register was pushed in the last instruction
}

struct IRBuilder<'a> {
  ir: IR,
  blkmeta: Vec<BlockMeta>,
  addrmap: HashMap<Address, BlockRef>,
  cur: BlockRef,
  cfg: &'a Config,
  special: Option<SpecialState>,
}

struct BlockMeta {
  //sealed: bool, // has all predecessors?
  //incomplete_phis: Vec<(Name, Ref)>,
}

impl<'a> IRBuilder<'a> {
  fn new(cfg: &'a Config) -> Self {
    let mut this = Self {
      ir: IR::new(),
      blkmeta: vec![],
      addrmap: HashMap::new(),
      cur: BlockRef(0),
      cfg,
      special: None,
    };

    // Create and seal the entry block
    let entry = this.new_block("entry");
    this.ir.seal_block(entry);

    // Set initial register values
    this.ir.set_var(instr::Reg::AX, this.cur, Ref::Init("ax"));
    this.ir.set_var(instr::Reg::CX, this.cur, Ref::Init("cx"));
    this.ir.set_var(instr::Reg::DX, this.cur, Ref::Init("dx"));
    this.ir.set_var(instr::Reg::BX, this.cur, Ref::Init("bx"));

    this.ir.set_var(instr::Reg::SP, this.cur, Ref::Init("sp"));
    this.ir.set_var(instr::Reg::BP, this.cur, Ref::Init("bp"));
    this.ir.set_var(instr::Reg::SI, this.cur, Ref::Init("si"));
    this.ir.set_var(instr::Reg::DI, this.cur, Ref::Init("di"));

    this.ir.set_var(instr::Reg::ES, this.cur, Ref::Init("es"));
    this.ir.set_var(instr::Reg::CS, this.cur, Ref::Init("cs"));
    this.ir.set_var(instr::Reg::SS, this.cur, Ref::Init("ss"));
    this.ir.set_var(instr::Reg::DS, this.cur, Ref::Init("ds"));

    this.ir.set_var(instr::Reg::IP, this.cur, Ref::Init("ip"));
    this.ir.set_var(instr::Reg::FLAGS, this.cur, Ref::Init("flags"));

    this
  }

  fn new_block(&mut self, name: &str) -> BlockRef {
    let idx = self.ir.blocks.len();
    self.ir.blocks.push(Block::new(name));
    self.blkmeta.push(BlockMeta {
      // sealed: false,
      // incomplete_phis: vec![],
    });
    BlockRef(idx)
  }

  fn get_block(&mut self, effective: Address) -> BlockRef {
    *self.addrmap.get(&effective).unwrap()
  }

  fn append_instr(&mut self, opcode: Opcode, operands: Vec<Ref>) -> Ref {
    let instr = Instr {
      opcode,
      operands,
    };
    let blk = &mut self.ir.blocks[self.cur.0];
    let idx = blk.instrs.push_back(instr);
    Ref::Instr(self.cur, idx)
  }

  fn append_jmp(&mut self, next: BlockRef) {
    self.ir.blocks[next.0].preds.push(self.cur);
    self.append_instr(Opcode::Jmp, vec![Ref::Block(next)]);
  }

  fn append_jne(&mut self, cond: Ref, true_blk: BlockRef, false_blk: BlockRef) {
    self.ir.blocks[true_blk.0].preds.push(self.cur);
    self.ir.blocks[false_blk.0].preds.push(self.cur);
    self.append_instr(Opcode::Jne, vec![
      cond,
      Ref::Block(true_blk),
      Ref::Block(false_blk)]);
  }

  fn switch_blk(&mut self, bref: BlockRef) {
    self.cur = bref;
  }

  fn start_next_blk(&mut self, next: Address) {
    let next_bref = self.get_block(next);

    // Make sure the last instruction is a jump
    match self.ir.blocks[self.cur.0].instrs.last().unwrap().opcode {
      Opcode::Jmp => (),
      Opcode::Jne => (),
      _ => self.append_jmp(next_bref), // need to append a trailing jump
    }

    // Switch to the next block
    self.switch_blk(next_bref);
    assert!(self.ir.blocks[self.cur.0].instrs.empty());
  }
}

/////////////////////////////////////////////////////////////////////////////////////

impl IRBuilder<'_> {
  fn append_asm_src_reg(&mut self, reg: &instr::OperandReg) -> Ref {
    self.ir.get_var(reg.0, self.cur)
  }

  fn append_asm_dst_reg(&mut self, reg: &instr::OperandReg, vref: Ref) {
    self.ir.set_var(reg.0, self.cur, vref);
  }

  fn compute_mem_address(&mut self, mem: &instr::OperandMem) -> Ref {
    assert!(mem.reg2.is_none());

    let addr = match (&mem.reg1, &mem.off) {
      (Some(reg1), Some(off)) => {
        let reg = self.ir.get_var(reg1, self.cur);
        let off = self.ir.append_const((*off as i16).into());
        self.append_instr(Opcode::Add, vec![reg, off])
      }
      (Some(reg1), None) => {
        self.ir.get_var(reg1, self.cur)
      }
      (None, Some(off)) => {
        self.ir.append_const((*off as i16).into())
      }
      (None, None) => {
        panic!("Invalid addressing mode");
      }
    };

    addr
  }

  fn append_asm_src_mem(&mut self, mem: &instr::OperandMem) -> Ref {
    let addr = self.compute_mem_address(mem);
    let seg = self.ir.get_var(mem.sreg, self.cur);

    let opcode = match mem.sz {
      instr::Size::Size8 => Opcode::Load8,
      instr::Size::Size16 => Opcode::Load16,
      instr::Size::Size32 => Opcode::Load32,
    };
    self.append_instr(opcode, vec![seg, addr])
  }

  fn append_asm_dst_mem(&mut self, mem: &instr::OperandMem, vref: Ref) {
    let addr = self.compute_mem_address(mem);
    let seg = self.ir.get_var(mem.sreg, self.cur);

    let opcode = match mem.sz {
      instr::Size::Size8 => Opcode::Store8,
      instr::Size::Size16 => Opcode::Store16,
      _ => panic!("32-bit stores not supported"),
    };

    self.append_instr(opcode, vec![seg, addr, vref]);
  }

  fn append_asm_src_imm(&mut self, imm: &instr::OperandImm) -> Ref {
    // TODO: Is it okay to sign-ext away the size here??
    let k: i32 = match imm.sz {
      instr::Size::Size8 => (imm.val as i8).into(),
      instr::Size::Size16 => (imm.val as i16).into(),
      _ => panic!("32-bit immediates not supported"),
    };
    self.ir.append_const(k)
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
    let new_flags = self.append_instr(Opcode::UpdateFlags, vec![old_flags, vref]);
    self.set_flags(new_flags);
  }

  fn pin_register(&mut self, reg: instr::Reg) {
    let vref = self.ir.get_var(reg, self.cur);
    self.append_instr(Opcode::Pin, vec![vref]);
  }

  fn pin_registers(&mut self) {
    // Caller-saved: so don't bother..
    //self.pin_register(instr::Reg::AX);
    //self.pin_register(instr::Reg::CX);
    //self.pin_register(instr::Reg::DX);
    //self.pin_register(instr::Reg::BX);
    //self.pin_register(instr::Reg::ES);

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
    let sp = self.append_instr(Opcode::Sub, vec![sp, k]);
    self.ir.set_var(instr::Reg::SP, self.cur, sp);

    // store to SS:SP
    self.append_instr(Opcode::Store16, vec![ss, sp, vref]);
  }

  fn append_pop(&mut self) -> Ref {
    let ss = self.ir.get_var(instr::Reg::SS, self.cur);
    let sp = self.ir.get_var(instr::Reg::SP, self.cur);
    let k = self.ir.append_const(2);

    let val = self.append_instr(Opcode::Load16, vec![ss, sp]);
    let sp = self.append_instr(Opcode::Add, vec![sp, k]);
    self.ir.set_var(instr::Reg::SP, self.cur, sp);

    val
  }

  fn process_callf_indirect(&mut self, ins: &instr::Instr) {
    let addr = self.append_asm_src_operand(&ins.operands[0]);
    self.append_instr(Opcode::CallPtr, vec![addr]);
  }

  fn process_callf(&mut self, ins: &instr::Instr) {
    let instr::Operand::Far(far) = &ins.operands[0] else {
      return self.process_callf_indirect(ins);
    };
    let addr = SegOff { seg: far.seg, off: far.off };
    if let Some(func) = self.cfg.func_lookup(addr) {
      // Known function
      let idx = self.ir.funcs.len();
      self.ir.funcs.push(func.name.to_string());
      let ret;
      if let Some(nargs) = &func.args {
        // Known args
        let mut operands = vec![Ref::Func(idx)];
        let ss = self.ir.get_var(instr::Reg::SS, self.cur);
            let sp = self.ir.get_var(instr::Reg::SP, self.cur);
        for i in 0..(*nargs as i32) {
          let mut off = sp;
          if i != 0 {
            let k = self.ir.append_const(2*i);
            off = self.append_instr(Opcode::Add, vec![sp, k]);
          }
          let val = self.append_instr(Opcode::Load16, vec![ss, off]);
          operands.push(val);
        }
        ret = self.append_instr(Opcode::CallArgs, operands);
      } else {
        // Unknown args
        ret = self.append_instr(Opcode::CallFar, vec![Ref::Func(idx)]);
      }
      match &func.ret {
        Type::Void => (), // nothing to do
        Type::U16 => {
          self.ir.set_var(instr::Reg::AX, self.cur, ret);
        }
        Type::U32 => {
          let upper = self.append_instr(Opcode::Upper16, vec![ret]);
          self.ir.set_var(instr::Reg::DX, self.cur, upper);

          let lower = self.append_instr(Opcode::Lower16, vec![ret]);
          self.ir.set_var(instr::Reg::AX, self.cur, lower);
        }
        _ => panic!("Unsupported function return type: {}", func.ret),
      }
    } else {
      // Unknown function
          let seg = self.ir.append_const(far.seg.into());
      let off = self.ir.append_const(far.off.into());
      let ret = self.append_instr(Opcode::CallFar, vec![seg, off]);
      self.ir.set_var(instr::Reg::AX, self.cur, ret);
    }
  }

  fn process_call(&mut self, ins: &instr::Instr, cs_pushed: bool) {
    println!("cs_pushed: {}", cs_pushed);
    let instr::Operand::Rel(rel) = &ins.operands[0] else {
      panic!("Expected near call to have relative operand");
    };
    if cs_pushed {
      // pop CS back off and use it to do a far call
      self.append_pop();
      // TODO: IMPL FAR CALL
    }
    // FIXME: THIS IS DEFINITELY BROKEN
    let k = self.ir.append_const(rel.val.into());
    let ret = self.append_instr(Opcode::CallNear, vec![k]);
    self.ir.set_var(instr::Reg::AX, self.cur, ret);
  }

  fn append_asm_instr(&mut self, ins: &instr::Instr) {
    //println!("## {}", intel_syntax::format(ins, &[], false).unwrap());
    assert!(ins.rep.is_none());

    let special = self.special.take();

    // process simple binary operations
    if let Some(opcode) = simple_binary_operation(ins.opcode) {
      let a = self.append_asm_src_operand(&ins.operands[0]);
      let b = self.append_asm_src_operand(&ins.operands[1]);
      let vref = self.append_instr(opcode, vec![a, b]);
      self.append_asm_dst_operand(&ins.operands[0], vref);
      return;
    }

    // handle less standard operations
    match &ins.opcode {
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
        // FIXME: WHEN RETURN A VALUE OR NOT?
        self.append_instr(Opcode::RetFar, vec![]);

      }
      instr::Opcode::OP_RET => {
        self.pin_registers();
        // FIXME: WHEN RETURN A VALUE OR NOT?
        self.append_instr(Opcode::RetNear, vec![]);

      }
      instr::Opcode::OP_MOV => {
        let vref = self.append_asm_src_operand(&ins.operands[1]);
        //let vref = self.append_instr(Opcode::Ref, vec![vref]);
        self.append_asm_dst_operand(&ins.operands[0], vref);
      }
      instr::Opcode::OP_INC => {
        let one = self.ir.append_const(1);
        let vref = self.append_asm_src_operand(&ins.operands[0]);
        let vref = self.append_instr(Opcode::Add, vec![vref, one]);
        self.append_asm_dst_operand(&ins.operands[0], vref);
      }
      instr::Opcode::OP_DEC => {
        let one = self.ir.append_const(1);
        let vref = self.append_asm_src_operand(&ins.operands[0]);
        let vref = self.append_instr(Opcode::Sub, vec![vref, one]);
        self.append_asm_dst_operand(&ins.operands[0], vref);
      }
      instr::Opcode::OP_JMP => {
        let instr::Operand::Rel(rel) = &ins.operands[0] else { panic!("Expected relative offset operand for JMP") };
        let effective = Address(ins.rel_addr(rel).abs());
        let blkref = self.get_block(effective);
        self.append_jmp(blkref);
      }
      instr::Opcode::OP_JE => {
        let instr::Operand::Rel(rel) = &ins.operands[0] else { panic!("Expected relative offset operand for JMP") };
        let effective = Address(ins.rel_addr(rel).abs());

        let false_blk = self.get_block(Address(ins.end_addr().abs()));
        let true_blk = self.get_block(effective);

        let flags = self.get_flags();
        let cond = self.append_instr(Opcode::EqFlags, vec![flags]);

        self.append_jne(cond, true_blk, false_blk);
      }
      instr::Opcode::OP_JNE => {
        let instr::Operand::Rel(rel) = &ins.operands[0] else { panic!("Expected relative offset operand for JMP") };
        let effective = Address(ins.rel_addr(rel).abs());

        let false_blk = self.get_block(Address(ins.end_addr().abs()));
        let true_blk = self.get_block(effective);

        let flags = self.get_flags();
        let cond = self.append_instr(Opcode::NeqFlags, vec![flags]);

        self.append_jne(cond, true_blk, false_blk);
      }
      instr::Opcode::OP_JG => {
        let instr::Operand::Rel(rel) = &ins.operands[0] else { panic!("Expected relative offset operand for JMP") };
        let effective = Address(ins.rel_addr(rel).abs());

        let false_blk = self.get_block(Address(ins.end_addr().abs()));
        let true_blk = self.get_block(effective);

        let flags = self.get_flags();
        let cond = self.append_instr(Opcode::GtFlags, vec![flags]);

        self.append_jne(cond, true_blk, false_blk);
      }
      instr::Opcode::OP_JGE => {
        let instr::Operand::Rel(rel) = &ins.operands[0] else { panic!("Expected relative offset operand for JMP") };
        let effective = Address(ins.rel_addr(rel).abs());

        let false_blk = self.get_block(Address(ins.end_addr().abs()));
        let true_blk = self.get_block(effective);

        let flags = self.get_flags();
        let cond = self.append_instr(Opcode::GeqFlags, vec![flags]);

        self.append_jne(cond, true_blk, false_blk);
      }
      instr::Opcode::OP_JL => {
        let instr::Operand::Rel(rel) = &ins.operands[0] else { panic!("Expected relative offset operand for JMP") };
        let effective = Address(ins.rel_addr(rel).abs());

        let false_blk = self.get_block(Address(ins.end_addr().abs()));
        let true_blk = self.get_block(effective);

        let flags = self.get_flags();
        let cond = self.append_instr(Opcode::LtFlags, vec![flags]);

        self.append_jne(cond, true_blk, false_blk);
      }
      instr::Opcode::OP_JLE => {
        let instr::Operand::Rel(rel) = &ins.operands[0] else { panic!("Expected relative offset operand for JMP") };
        let effective = Address(ins.rel_addr(rel).abs());

        let false_blk = self.get_block(Address(ins.end_addr().abs()));
        let true_blk = self.get_block(effective);

        let flags = self.get_flags();
        let cond = self.append_instr(Opcode::LeqFlags, vec![flags]);

        self.append_jne(cond, true_blk, false_blk);
      }
      instr::Opcode::OP_CALLF => {
        self.process_callf(ins);
      }
      instr::Opcode::OP_CALL => {
        let cs_pushed = matches!(special, Some(SpecialState::PushCS));
        self.process_call(ins, cs_pushed);
      }
      instr::Opcode::OP_LES => {
        let vref = self.append_asm_src_operand(&ins.operands[2]);

        let upper = self.append_instr(Opcode::Upper16, vec![vref]);
        self.append_asm_dst_operand(&ins.operands[0], upper);

        let lower = self.append_instr(Opcode::Lower16, vec![vref]);
        self.append_asm_dst_operand(&ins.operands[1], lower);
      }
      instr::Opcode::OP_TEST => {
        let a = self.append_asm_src_operand(&ins.operands[0]);
        let b = self.append_asm_src_operand(&ins.operands[1]);
        let vref = self.append_instr(Opcode::And, vec![a, b]);
        self.append_update_flags(vref);
      }
      instr::Opcode::OP_CMP => {
        let a = self.append_asm_src_operand(&ins.operands[0]);
        let b = self.append_asm_src_operand(&ins.operands[1]);
        let vref = self.append_instr(Opcode::Sub, vec![a, b]);
        self.append_update_flags(vref);
      }
      instr::Opcode::OP_CWD => {
        let src = self.append_asm_src_operand(&ins.operands[1]);
        let vref = self.append_instr(Opcode::SignExtTo32, vec![src]);
        let upper = self.append_instr(Opcode::Upper16, vec![vref]);
        self.append_asm_dst_operand(&ins.operands[0], upper);
      }
      instr::Opcode::OP_DIV => {
        let upper_in = self.append_asm_src_operand(&ins.operands[0]);
        let lower_in = self.append_asm_src_operand(&ins.operands[1]);
        let divisor = self.append_asm_src_operand(&ins.operands[2]);
        let dividend = self.append_instr(Opcode::Make32, vec![upper_in, lower_in]);
        let quotient = self.append_instr(Opcode::UDiv, vec![dividend, divisor]);
        let upper_out = self.append_instr(Opcode::Upper16, vec![quotient]);
        let lower_out = self.append_instr(Opcode::Lower16, vec![quotient]);
        self.append_asm_dst_operand(&ins.operands[0], upper_out);
        self.append_asm_dst_operand(&ins.operands[1], lower_out);
      }
      _ => panic!("Unimpl opcode: {:?}", ins.opcode),
    }
  }

  fn build_from_instrs(&mut self, instrs: &[instr::Instr]) {
    // Step 1: Infer basic-block boundaries
    let mut block_start = HashSet::new();
    for ins in instrs {
      let Some(target) = jump_target(ins) else { continue };
      block_start.insert(target.0);
      block_start.insert(ins.end_addr().abs());
    }

    // Step 2: Create all the blocks we should encounter
    let mut addr_ordered: Vec<_> = block_start.iter().collect();
    addr_ordered.sort();
    for addr in addr_ordered {
      let bref = self.new_block(&format!("addr_{:x}", addr));
      self.addrmap.insert(Address(*addr), bref);
    }

    // Step 3: iterate each instruction, building each block
    for ins in instrs {
      if block_start.get(&ins.addr.abs()).is_some() {
        self.start_next_blk(Address(ins.addr.abs()));
      }
      self.append_asm_instr(ins);
    }

    // Step 4: walk blocks and seal them
    for i in 0..self.ir.blocks.len() {
      if !self.ir.blocks[i].sealed {
        self.ir.seal_block(BlockRef(i));
      }
    }
  }
}

pub fn from_instrs(instrs: &[instr::Instr], cfg: &Config) -> IR {
  let mut bld = IRBuilder::new(cfg);
  bld.build_from_instrs(instrs);
  bld.ir
}
