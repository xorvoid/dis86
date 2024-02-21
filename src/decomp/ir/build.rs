use crate::instr;
use std::collections::{HashSet, HashMap};
use crate::decomp::ir::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Address(usize);

// const FLAGS_BIT_CF: u32 = 1<<0;
// const FLAGS_BIT_PF: u32 = 1<<2;
// const FLAGS_BIT_AF: u32 = 1<<4;
// const FLAGS_BIT_ZF: u32 = 1<<6;
// const FLAGS_BIT_SF: u32 = 1<<7;
// const FLAGS_BIT_TF: u32 = 1<<8;
// const FLAGS_BIT_IF: u32 = 1<<9;
// const FLAGS_BIT_DF: u32 = 1<<10;
// const FLAGS_BIT_OF: u32 = 1<<11;

// fn supported_instruction(ins: &instr::Instr) -> bool {
//   // NOT TRUE - FIXME
//   true
// }

fn simple_binary_operation(opcode: instr::Opcode) -> Option<Opcode> {
  match opcode {
    instr::Opcode::OP_ADD => Some(Opcode::Add),
    instr::Opcode::OP_SUB => Some(Opcode::Sub),
    instr::Opcode::OP_SHL => Some(Opcode::Shl),
    instr::Opcode::OP_XOR => Some(Opcode::Xor),
    _ => None,
  }
}

fn reg_name(reg: instr::Reg) -> &'static str {
  // INEFFICIENT AND COMPLICATED
  match reg {
    instr::Reg::AX => "AX",
    instr::Reg::CX => "CX",
    instr::Reg::DX => "DX",
    instr::Reg::BX => "BX",
    instr::Reg::AL => "AX",  // use the full register
    instr::Reg::CL => "CX",  // use the full register
    instr::Reg::DL => "DX",  // use the full register
    instr::Reg::BL => "BX",  // use the full register
    instr::Reg::AH => "AX",  // use the full register
    instr::Reg::CH => "CX",  // use the full register
    instr::Reg::DH => "DX",  // use the full register
    instr::Reg::BH => "BX",  // use the full register
    instr::Reg::SP => "SP",
    instr::Reg::BP => "BP",
    instr::Reg::SI => "SI",
    instr::Reg::DI => "DI",
    instr::Reg::ES => "ES",
    instr::Reg::CS => "CS",
    instr::Reg::SS => "SS",
    instr::Reg::DS => "DS",
    instr::Reg::IP => "IP",
    instr::Reg::FLAGS => "FLAGS",
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
      let end_addr = (ins.addr + ins.n_bytes) as u16;
      let effective = end_addr.wrapping_add(rel.val);
      Address(effective.into())
    }
    _ => panic!("Unsupported branch operand"),
  };

  Some(tgt)
}

struct IRBuilder {
  ir: IR,
  blkmeta: Vec<BlockMeta>,
  addrmap: HashMap<Address, BlockRef>,
  cur: BlockRef,
}
//let mut block_map = HashMap::new();



struct BlockMeta {
  sealed: bool, // has all predecessors?
  defs: HashMap<String, ValueRef>,
  incomplete_phis: Vec<(String, PhiRef)>,
}

impl Block {
  fn new(name: &str) -> Self {
    Self {
      name: name.to_string(),
      preds: vec![],
      phis: vec![],
      instrs: vec![],
    }
  }
}

impl IRBuilder {
  fn new() -> Self {
    let mut this = Self {
      ir: IR {
        consts: vec![],
        blocks: vec![],
      },
      blkmeta: vec![],
      addrmap: HashMap::new(),
      cur: BlockRef(0),
    };

    // Create and seal the entry block
    let entry = this.new_block("entry");
    this.seal_block(entry);

    // Set initial register values
    this.set_var("AX", this.cur, ValueRef::Init("AX"));
    this.set_var("CX", this.cur, ValueRef::Init("CX"));
    this.set_var("DX", this.cur, ValueRef::Init("DX"));
    this.set_var("BX", this.cur, ValueRef::Init("BX"));

    this.set_var("SP", this.cur, ValueRef::Init("SP"));
    this.set_var("BP", this.cur, ValueRef::Init("BP"));
    this.set_var("SI", this.cur, ValueRef::Init("SI"));
    this.set_var("DI", this.cur, ValueRef::Init("DI"));

    this.set_var("ES", this.cur, ValueRef::Init("ES"));
    this.set_var("CS", this.cur, ValueRef::Init("CS"));
    this.set_var("SS", this.cur, ValueRef::Init("SS"));
    this.set_var("DS", this.cur, ValueRef::Init("DS"));

    this.set_var("IP", this.cur, ValueRef::Init("IP"));
    this.set_var("FLAGS", this.cur, ValueRef::Init("FLAGS"));

    this
  }

  fn new_block(&mut self, name: &str) -> BlockRef {
    let idx = self.ir.blocks.len();
    self.ir.blocks.push(Block::new(name));
    self.blkmeta.push(BlockMeta {
      sealed: false,
      defs: HashMap::new(),
      incomplete_phis: vec![],
    });
    BlockRef(idx)
  }

  fn seal_block(&mut self, r: BlockRef) {
    let b = &mut self.blkmeta[r.0];
    if b.sealed { panic!("block is already sealed!"); }
    b.sealed = true;
    // TODO
    // for (name, phi) in std::mem::replace(&mut b.incomplete_phis, vec![]) {
    //   self.phi_populate(&name, BlockRef(r.0), phi);
    // }
    std::mem::forget(r);
  }

  fn phi_create(&mut self, name: &str, blk: BlockRef) -> PhiRef {
    // create phi node (without operands) to terminate recursion
    let phi = PhiRef(self.ir.blocks[blk.0].phis.len());
    self.ir.blocks[blk.0].phis.push(Instr {
      opcode: Opcode::Phi,
      operands: vec![],
    });
    let val = ValueRef::Phi(blk, phi);
    self.set_var(name, blk, val);
    phi
  }

  fn get_var(&mut self, name: &str, blk: BlockRef) -> ValueRef {
    let b = &mut self.blkmeta[blk.0];

    // Defined locally in this block? Easy.
    match b.defs.get(name) {
      Some(val) => return *val,
      None => (),
    }

    // FIXME: BROKEN!!
    //assert!(b.sealed);

    let preds = &self.ir.blocks[blk.0].preds;
    if preds.len() == 1 {
      let parent = preds[0];
      self.get_var(name, parent)
    } else {
      // add an empty phi node and mark it for later population
      let phi = self.phi_create(name, blk);
      self.blkmeta[blk.0].incomplete_phis.push((name.to_string(), phi));
      ValueRef::Phi(blk, phi)
    }

    // // Otherwise, search predecessors
    // if !b.sealed {
    //   // add an empty phi node and mark it for later population
    //   let phi = self.phi_create(name, blk);
    //   self.blkmeta[blk.0].incomplete_phis.push((name.to_string(), phi));
    //   ValueRef::Phi(blk, phi)
    // } else {
    //   let preds = &self.ir.blocks[blk.0].preds;
    //   if preds.len() == 1 {
    //     let parent = preds[0];
    //     self.get_var(name, parent)
    //   } else {
    //     // create a phi and immediately populate it
    //     let phi = self.phi_create(name, blk);
    //     self.phi_populate(name, blk, phi);
    //     ValueRef::Phi(blk, phi)
    //   }
    // }

    // // Otherwise, search predecessors
    // panic!("UNIMPL");
  }

  fn set_var(&mut self, name: &str, blk: BlockRef, r: ValueRef) {
    self.blkmeta[blk.0].defs.insert(name.to_string(), r);
  }

  fn get_block(&mut self, effective: Address) -> BlockRef {
    *self.addrmap.get(&effective).unwrap()
  }

  fn append_instr(&mut self, instr: Instr) -> ValueRef {
    let blk = &mut self.ir.blocks[self.cur.0];
    let idx = blk.instrs.len();
    blk.instrs.push(instr);
    ValueRef::Instr(self.cur, InstrRef(idx))
  }

  fn append_jmp(&mut self, next: BlockRef) {
    self.ir.blocks[next.0].preds.push(self.cur);

    self.append_instr(Instr {
      opcode: Opcode::Jmp,
      operands: vec![ValueRef::Instr(next, InstrRef(0))],
    });
  }

  fn append_jne(&mut self, cond: ValueRef, true_blk: BlockRef, false_blk: BlockRef) {
    self.ir.blocks[true_blk.0].preds.push(self.cur);
    self.ir.blocks[false_blk.0].preds.push(self.cur);

    self.append_instr(Instr {
      opcode: Opcode::Jne,
      operands: vec![
        cond,
        ValueRef::Instr(true_blk, InstrRef(0)),
        ValueRef::Instr(false_blk, InstrRef(0))],
    });
  }

  fn switch_blk(&mut self, bref: BlockRef) {
    self.cur = bref;
  }

  fn start_next_blk(&mut self, next: Address) {
    let next_bref = self.get_block(next);

    // Make sure the last instruction is a jump
    let blk = &self.ir.blocks[self.cur.0];
    assert!(blk.instrs.len() > 0);
    match blk.instrs[blk.instrs.len()-1].opcode {
      Opcode::Jmp => (),
      Opcode::Jne => (),
      _ => { // no trailing jump, insert one
        self.append_instr(Instr{
          opcode: Opcode::Jmp,
          operands: vec![ValueRef::Instr(next_bref, InstrRef(0))],
        });
      }
    }

    // Switch to the next block
    self.switch_blk(next_bref);
    let blk = &self.ir.blocks[self.cur.0];
    assert!(blk.instrs.len() == 0);
  }
}

/////////////////////////////////////////////////////////////////////////////////////

impl IRBuilder {
  fn append_asm_src_reg(&mut self, reg: &instr::OperandReg) -> ValueRef {
    self.get_var(reg_name(reg.0), self.cur)
  }

  fn append_asm_dst_reg(&mut self, reg: &instr::OperandReg, vref: ValueRef) {
    self.set_var(reg_name(reg.0), self.cur, vref);
  }

  fn compute_mem_address(&mut self, mem: &instr::OperandMem) -> ValueRef {
    assert!(mem.reg2.is_none());

    let addr = match (&mem.reg1, &mem.off) {
      (Some(reg1), Some(off)) => {
        let reg = self.get_var(reg_name(*reg1), self.cur);
        let off = self.ir.append_const((*off as i16).into());

        self.append_instr(Instr {
          opcode: Opcode::Add,
          operands: vec![reg, off],
        })
      }
      (Some(reg1), None) => {
        self.get_var(reg_name(*reg1), self.cur)
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

  fn append_asm_src_mem(&mut self, mem: &instr::OperandMem) -> ValueRef {
    let addr = self.compute_mem_address(mem);
    let seg = self.get_var(reg_name(mem.sreg), self.cur);

    let opcode = match mem.sz {
      instr::Size::Size8 => Opcode::Load8,
      instr::Size::Size16 => Opcode::Load16,
      instr::Size::Size32 => Opcode::Load32,
    };

    self.append_instr(Instr {
      opcode,
      operands: vec![seg, addr],
    })
  }

  fn append_asm_dst_mem(&mut self, mem: &instr::OperandMem, vref: ValueRef) {
    let addr = self.compute_mem_address(mem);
    let seg = self.get_var(reg_name(mem.sreg), self.cur);

    let opcode = match mem.sz {
      instr::Size::Size8 => Opcode::Store8,
      instr::Size::Size16 => Opcode::Store16,
      _ => panic!("32-bit stores not supported"),
    };

    self.append_instr(Instr {
      opcode,
      operands: vec![seg, addr, vref],
    });
  }

  fn append_asm_src_imm(&mut self, imm: &instr::OperandImm) -> ValueRef {
    // TODO: Is it okay to sign-ext away the size here??
    let k: i32 = match imm.sz {
      instr::Size::Size8 => (imm.val as i8).into(),
      instr::Size::Size16 => (imm.val as i16).into(),
      _ => panic!("32-bit immediates not supported"),
    };
    self.ir.append_const(k)
  }

  fn append_asm_src_rel(&mut self, _rel: &instr::OperandRel) -> ValueRef {
    panic!("unimpl");
  }

  fn append_asm_src_far(&mut self, _far: &instr::OperandFar) -> ValueRef {
    panic!("unimpl");
  }

  fn append_asm_src_operand(&mut self, oper: &instr::Operand) -> ValueRef {
    match oper {
      instr::Operand::Reg(reg) => self.append_asm_src_reg(reg),
      instr::Operand::Mem(mem) => self.append_asm_src_mem(mem),
      instr::Operand::Imm(imm) => self.append_asm_src_imm(imm),
      instr::Operand::Rel(rel) => self.append_asm_src_rel(rel),
      instr::Operand::Far(far) => self.append_asm_src_far(far),
    }
  }

  fn append_asm_dst_operand(&mut self, oper: &instr::Operand, vref: ValueRef) {
    match oper {
      instr::Operand::Reg(reg) => self.append_asm_dst_reg(reg, vref),
      instr::Operand::Mem(mem) => self.append_asm_dst_mem(mem, vref),
      instr::Operand::Imm(_)   => panic!("Should never have a destination imm"),
      instr::Operand::Rel(_)   => panic!("Should never have a destination rel"),
      instr::Operand::Far(_)   => panic!("Should never have a destination far"),
    };
  }

  fn get_flags(&mut self) -> ValueRef {
    self.get_var("FLAGS", self.cur)
  }

  fn set_flags(&mut self, vref: ValueRef) {
    self.set_var("FLAGS", self.cur, vref);
  }

  fn append_update_flags(&mut self, vref: ValueRef) {
    let old_flags = self.get_flags();
    let new_flags = self.append_instr(Instr {
      opcode: Opcode::UpdateFlags,
      operands: vec![old_flags, vref],
    });
    self.set_flags(new_flags);
  }

  fn append_asm_instr(&mut self, ins: &instr::Instr) {
    //println!("## {}", intel_syntax::format(ins, &[], false).unwrap());
    assert!(ins.rep.is_none());

    // process simple binary operations
    if let Some(opcode) = simple_binary_operation(ins.opcode) {
      let a = self.append_asm_src_operand(&ins.operands[0]);
      let b = self.append_asm_src_operand(&ins.operands[1]);
      let vref = self.append_instr(Instr {
        opcode,
        operands: vec![a, b],
      });
      self.append_asm_dst_operand(&ins.operands[0], vref);
      return;
    }

    // handle less standard operations
    match &ins.opcode {
      instr::Opcode::OP_PUSH => {
        let a = self.append_asm_src_operand(&ins.operands[0]);
        self.append_instr(Instr {
          opcode: Opcode::Push,
          operands: vec![a],
        });
      }
      instr::Opcode::OP_POP => {
        let vref = self.append_instr(Instr {
          opcode: Opcode::Pop,
          operands: vec![],
        });
        self.append_asm_dst_operand(&ins.operands[0], vref);
      }
      instr::Opcode::OP_LEAVE => {
        // mov sp, bp
        let vref = self.get_var(reg_name(instr::Reg::BP), self.cur);
        self.set_var(reg_name(instr::Reg::SP), self.cur, vref);
        // pop bp
        let vref = self.append_instr(Instr {
          opcode: Opcode::Pop,
          operands: vec![],
        });
        self.set_var(reg_name(instr::Reg::BP), self.cur, vref);
      }
      instr::Opcode::OP_RETF => {
        let vref = self.get_var(reg_name(instr::Reg::AX), self.cur);
        self.append_instr(Instr {
          opcode: Opcode::Ret,
          operands: vec![vref],
        });
      }
      instr::Opcode::OP_MOV => {
        let vref = self.append_asm_src_operand(&ins.operands[1]);
        self.append_asm_dst_operand(&ins.operands[0], vref);
      }
      instr::Opcode::OP_INC => {
        let one = self.ir.append_const(1);
        let vref = self.append_asm_src_operand(&ins.operands[0]);
        let vref = self.append_instr(Instr {
          opcode: Opcode::Add,
          operands: vec![vref, one],
        });
        self.append_asm_dst_operand(&ins.operands[0], vref);
      }
      instr::Opcode::OP_JMP => {
        let instr::Operand::Rel(rel) = &ins.operands[0] else { panic!("Expected relative offset operand for JMP") };
        let end_addr = (ins.addr + ins.n_bytes) as u16;
        let effective = Address(end_addr.wrapping_add(rel.val).into());
        let blkref = self.get_block(effective);
        self.append_jmp(blkref);
      }
      instr::Opcode::OP_JE => {
        let instr::Operand::Rel(rel) = &ins.operands[0] else { panic!("Expected relative offset operand for JMP") };
        let end_addr = (ins.addr + ins.n_bytes) as u16;
        let effective = Address(end_addr.wrapping_add(rel.val).into());

        let false_blk = self.get_block(Address(ins.addr + ins.n_bytes));
        let true_blk = self.get_block(effective);

        let flags = self.get_flags();
        let cond = self.append_instr(Instr {
          opcode: Opcode::EqFlags,
          operands: vec![flags],
        });

        self.append_jne(cond, true_blk, false_blk);
      }
      instr::Opcode::OP_JNE => {
        let instr::Operand::Rel(rel) = &ins.operands[0] else { panic!("Expected relative offset operand for JMP") };
        let end_addr = (ins.addr + ins.n_bytes) as u16;
        let effective = Address(end_addr.wrapping_add(rel.val).into());

        let false_blk = self.get_block(Address(ins.addr + ins.n_bytes));
        let true_blk = self.get_block(effective);

        let flags = self.get_flags();
        let cond = self.append_instr(Instr {
          opcode: Opcode::NeqFlags,
          operands: vec![flags],
        });

        self.append_jne(cond, true_blk, false_blk);
      }
      instr::Opcode::OP_JG => {
        let instr::Operand::Rel(rel) = &ins.operands[0] else { panic!("Expected relative offset operand for JMP") };
        let end_addr = (ins.addr + ins.n_bytes) as u16;
        let effective = Address(end_addr.wrapping_add(rel.val).into());

        let false_blk = self.get_block(Address(ins.addr + ins.n_bytes));
        let true_blk = self.get_block(effective);

        let flags = self.get_flags();
        let cond = self.append_instr(Instr {
          opcode: Opcode::GtFlags,
          operands: vec![flags],
        });

        self.append_jne(cond, true_blk, false_blk);
      }
      instr::Opcode::OP_JGE => {
        let instr::Operand::Rel(rel) = &ins.operands[0] else { panic!("Expected relative offset operand for JMP") };
        let end_addr = (ins.addr + ins.n_bytes) as u16;
        let effective = Address(end_addr.wrapping_add(rel.val).into());

        let false_blk = self.get_block(Address(ins.addr + ins.n_bytes));
        let true_blk = self.get_block(effective);

        let flags = self.get_flags();
        let cond = self.append_instr(Instr {
          opcode: Opcode::GeqFlags,
          operands: vec![flags],
        });

        self.append_jne(cond, true_blk, false_blk);
      }
      instr::Opcode::OP_JL => {
        let instr::Operand::Rel(rel) = &ins.operands[0] else { panic!("Expected relative offset operand for JMP") };
        let end_addr = (ins.addr + ins.n_bytes) as u16;
        let effective = Address(end_addr.wrapping_add(rel.val).into());

        let false_blk = self.get_block(Address(ins.addr + ins.n_bytes));
        let true_blk = self.get_block(effective);

        let flags = self.get_flags();
        let cond = self.append_instr(Instr {
          opcode: Opcode::LtFlags,
          operands: vec![flags],
        });

        self.append_jne(cond, true_blk, false_blk);
      }
      instr::Opcode::OP_JLE => {
        let instr::Operand::Rel(rel) = &ins.operands[0] else { panic!("Expected relative offset operand for JMP") };
        let end_addr = (ins.addr + ins.n_bytes) as u16;
        let effective = Address(end_addr.wrapping_add(rel.val).into());

        let false_blk = self.get_block(Address(ins.addr + ins.n_bytes));
        let true_blk = self.get_block(effective);

        let flags = self.get_flags();
        let cond = self.append_instr(Instr {
          opcode: Opcode::LeqFlags,
          operands: vec![flags],
        });

        self.append_jne(cond, true_blk, false_blk);
      }
      instr::Opcode::OP_CALLF => {
        let instr::Operand::Far(far) = &ins.operands[0] else { panic!("Unsupported OP_CALLF operand") };
        let seg = self.ir.append_const(far.seg.into());
        let off = self.ir.append_const(far.off.into());
        let ret = self.append_instr(Instr {
          opcode: Opcode::Call,
          operands: vec![seg, off],
        });
        self.set_var(reg_name(instr::Reg::AX), self.cur, ret);
      }
      instr::Opcode::OP_LES => {
        let vref = self.append_asm_src_operand(&ins.operands[2]);
        let upper = self.append_instr(Instr {
          opcode: Opcode::Upper16,
          operands: vec![vref],
        });
        self.append_asm_dst_operand(&ins.operands[0], upper);
        let lower = self.append_instr(Instr {
          opcode: Opcode::Lower16,
          operands: vec![vref],
        });
        self.append_asm_dst_operand(&ins.operands[1], lower);
      }
      instr::Opcode::OP_TEST => {
        let a = self.append_asm_src_operand(&ins.operands[0]);
        let b = self.append_asm_src_operand(&ins.operands[1]);
        let vref = self.append_instr(Instr {
          opcode: Opcode::And,
          operands: vec![a, b],
        });
        self.append_update_flags(vref);
      }
      instr::Opcode::OP_CMP => {
        let a = self.append_asm_src_operand(&ins.operands[0]);
        let b = self.append_asm_src_operand(&ins.operands[1]);
        let vref = self.append_instr(Instr {
          opcode: Opcode::Sub,
          operands: vec![a, b],
        });
        self.append_update_flags(vref);
      }
      _ => panic!("Unimpl opcode: {:?}", ins.opcode),
    }
  }

  fn build_from_instrs(&mut self, instrs: &[instr::Instr]) {
    // // Step 0: Sanity
    // for ins in instrs {
    //   if !supported_instruction(ins) {
    //     panic!("Unsupported instruction: '{}'", intel_syntax::format(ins, &[], false).unwrap());
    //   }
    // }

    // Step 1: Infer basic-block boundaries
    let mut block_start = HashSet::new();
    for ins in instrs {
      let Some(target) = jump_target(ins) else { continue };
      block_start.insert(target.0);
      block_start.insert(ins.addr + ins.n_bytes);
    }

    // Step 2: Create all the blocks we should encounter
    let mut addr_ordered: Vec<_> = block_start.iter().collect();
    addr_ordered.sort();
    for addr in addr_ordered {
      let bref = self.new_block(&format!("addr_{:x}", addr));
      self.addrmap.insert(Address(*addr), bref);
    }

    // for ins in instrs {
    //   if block_start.get(&ins.addr).is_some() {
    //     println!("");
    //     println!("##################################################");
    //     println!("# Block 0x{:x}", ins.addr);
    //     println!("##################################################");
    //   }
    //   println!("{}", intel_syntax::format(ins, &[], false).unwrap());
    // }

    // Step 3: iterate each instruction, building each block
    for ins in instrs {
      if block_start.get(&ins.addr).is_some() {
        self.start_next_blk(Address(ins.addr));
      }
      self.append_asm_instr(ins);
    }
  }
}

pub fn from_instrs(instrs: &[instr::Instr]) -> IR {
  let mut bld = IRBuilder::new();
  bld.build_from_instrs(instrs);
  bld.ir
}
