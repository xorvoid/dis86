use crate::instr;
use crate::util::dvec::DVec;
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
  symbol_count: HashMap<Symbol, usize>,
  // ref_to_symbol: HashMap<Ref, (Symbol, usize)>,
  cur: BlockRef,
}

struct BlockMeta {
  sealed: bool, // has all predecessors?
  incomplete_phis: Vec<(Symbol, Ref)>,
}

impl Block {
  fn new(name: &str) -> Self {
    Self {
      name: name.to_string(),
      defs: HashMap::new(),
      preds: vec![],
      instrs: DVec::new(),
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
      symbol_count: HashMap::new(),
      cur: BlockRef(0),
    };

    // Create and seal the entry block
    let entry = this.new_block("entry");
    this.seal_block(entry);

    // Set initial register values
    this.set_var(instr::Reg::AX, this.cur, Ref::Init("ax"));
    this.set_var(instr::Reg::CX, this.cur, Ref::Init("cx"));
    this.set_var(instr::Reg::DX, this.cur, Ref::Init("dx"));
    this.set_var(instr::Reg::BX, this.cur, Ref::Init("bx"));

    this.set_var(instr::Reg::SP, this.cur, Ref::Init("sp"));
    this.set_var(instr::Reg::BP, this.cur, Ref::Init("bp"));
    this.set_var(instr::Reg::SI, this.cur, Ref::Init("si"));
    this.set_var(instr::Reg::DI, this.cur, Ref::Init("di"));

    this.set_var(instr::Reg::ES, this.cur, Ref::Init("es"));
    this.set_var(instr::Reg::CS, this.cur, Ref::Init("cs"));
    this.set_var(instr::Reg::SS, this.cur, Ref::Init("ss"));
    this.set_var(instr::Reg::DS, this.cur, Ref::Init("ds"));

    this.set_var(instr::Reg::IP, this.cur, Ref::Init("ip"));
    this.set_var(instr::Reg::FLAGS, this.cur, Ref::Init("flags"));

    this
  }

  fn new_block(&mut self, name: &str) -> BlockRef {
    let idx = self.ir.blocks.len();
    self.ir.blocks.push(Block::new(name));
    self.blkmeta.push(BlockMeta {
      sealed: false,
      incomplete_phis: vec![],
    });
    BlockRef(idx)
  }

  fn seal_block(&mut self, r: BlockRef) {
    let b = &mut self.blkmeta[r.0];
    if b.sealed { panic!("block is already sealed!"); }
    b.sealed = true;
    for (sym, phi) in std::mem::replace(&mut b.incomplete_phis, vec![]) {
      self.phi_populate(sym, phi)
    }
  }

  fn phi_populate<S: Into<Symbol>>(&mut self, sym: S, phiref: Ref) {
    let sym: Symbol = sym.into();
    let Ref::Instr(blk, idx) = phiref else { panic!("Invalid ref") };

    let preds = self.ir.blocks[blk.0].preds.clone(); // ARGH: Need to break borrow on 'self' so we can recurse
    assert!(self.ir.blocks[blk.0].instrs[idx].opcode == Opcode::Phi);

    // recurse each pred
    let mut refs = vec![];
    for b in preds {
      refs.push(self.get_var(sym, b));
    }

    // update the phi with operands
    self.ir.blocks[blk.0].instrs[idx].operands = refs;

    // TODO: Remove trivial phis
  }

  fn phi_create(&mut self, sym: Symbol, blk: BlockRef) -> Ref {
    // create phi node (without operands) to terminate recursion

    let idx = self.ir.blocks[blk.0].instrs.push_front(Instr {
      debug: None,
      opcode: Opcode::Phi,
      operands: vec![],
    });

    let vref = Ref::Instr(blk, idx);
    self.set_var(sym, blk, vref);

    vref
  }

  fn get_var<S: Into<Symbol>>(&mut self, sym: S, blk: BlockRef) -> Ref {
    let sym: Symbol = sym.into();

    // Defined locally in this block? Easy.
    match self.ir.blocks[blk.0].defs.get(&sym) {
      Some(val) => return *val,
      None => (),
    }

    // Otherwise, search predecessors
    let b = &self.blkmeta[blk.0];
    if !b.sealed {
      // add an empty phi node and mark it for later population
      let phi = self.phi_create(sym, blk);
      self.blkmeta[blk.0].incomplete_phis.push((sym, phi));
      phi
    } else {
      let preds = &self.ir.blocks[blk.0].preds;
      if preds.len() == 1 {
        let parent = preds[0];
        self.get_var(sym, parent)
      } else {
        // create a phi and immediately populate it
        let phi = self.phi_create(sym, blk);
        self.phi_populate(sym, phi);
        phi
      }
    }
  }

  fn set_var<S: Into<Symbol>>(&mut self, sym: S, blk: BlockRef, r: Ref) {
    let sym = sym.into();
    self.ir.blocks[blk.0].defs.insert(sym, r);

    // set up debug symbol
    if let Ref::Instr(b, i) = r {
      let instr = &mut self.ir.blocks[b.0].instrs[i];
      if instr.debug.is_none() {
        let num_ptr = self.symbol_count.entry(sym).or_insert(1);
        let num = *num_ptr;
        *num_ptr += 1;
        instr.debug = Some((sym, num));
      }
    }
  }

  fn get_block(&mut self, effective: Address) -> BlockRef {
    *self.addrmap.get(&effective).unwrap()
  }

  fn append_instr(&mut self, opcode: Opcode, operands: Vec<Ref>) -> Ref {
    let instr = Instr {
      debug: None,
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

impl IRBuilder {
  fn append_asm_src_reg(&mut self, reg: &instr::OperandReg) -> Ref {
    self.get_var(reg.0, self.cur)
  }

  fn append_asm_dst_reg(&mut self, reg: &instr::OperandReg, vref: Ref) {
    self.set_var(reg.0, self.cur, vref);
  }

  fn compute_mem_address(&mut self, mem: &instr::OperandMem) -> Ref {
    assert!(mem.reg2.is_none());

    let addr = match (&mem.reg1, &mem.off) {
      (Some(reg1), Some(off)) => {
        let reg = self.get_var(reg1, self.cur);
        let off = self.ir.append_const((*off as i16).into());
        self.append_instr(Opcode::Add, vec![reg, off])
      }
      (Some(reg1), None) => {
        self.get_var(reg1, self.cur)
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
    let seg = self.get_var(mem.sreg, self.cur);

    let opcode = match mem.sz {
      instr::Size::Size8 => Opcode::Load8,
      instr::Size::Size16 => Opcode::Load16,
      instr::Size::Size32 => Opcode::Load32,
    };
    self.append_instr(opcode, vec![seg, addr])
  }

  fn append_asm_dst_mem(&mut self, mem: &instr::OperandMem, vref: Ref) {
    let addr = self.compute_mem_address(mem);
    let seg = self.get_var(mem.sreg, self.cur);

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
    self.get_var(instr::Reg::FLAGS, self.cur)
  }

  fn set_flags(&mut self, vref: Ref) {
    self.set_var(instr::Reg::FLAGS, self.cur, vref);
  }

  fn append_update_flags(&mut self, vref: Ref) {
    let old_flags = self.get_flags();
    let new_flags = self.append_instr(Opcode::UpdateFlags, vec![old_flags, vref]);
    self.set_flags(new_flags);
  }

  fn append_asm_instr(&mut self, ins: &instr::Instr) {
    //println!("## {}", intel_syntax::format(ins, &[], false).unwrap());
    assert!(ins.rep.is_none());

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
        let a = self.append_asm_src_operand(&ins.operands[0]);
        self.append_instr(Opcode::Push, vec![a]);
      }
      instr::Opcode::OP_POP => {
        let vref = self.append_instr(Opcode::Pop, vec![]);
        self.append_asm_dst_operand(&ins.operands[0], vref);
      }
      instr::Opcode::OP_LEAVE => {
        // mov sp, bp
        let vref = self.get_var(instr::Reg::BP, self.cur);
        self.set_var(instr::Reg::SP, self.cur, vref);
        // pop bp
        let vref = self.append_instr(Opcode::Pop, vec![]);
        self.set_var(instr::Reg::BP, self.cur, vref);
      }
      instr::Opcode::OP_RETF => {
        let vref = self.get_var(instr::Reg::AX, self.cur);
        self.append_instr(Opcode::Ret, vec![vref]);
      }
      instr::Opcode::OP_MOV => {
        let vref = self.append_asm_src_operand(&ins.operands[1]);
        self.append_asm_dst_operand(&ins.operands[0], vref);
      }
      instr::Opcode::OP_INC => {
        let one = self.ir.append_const(1);
        let vref = self.append_asm_src_operand(&ins.operands[0]);
        let vref = self.append_instr(Opcode::Add, vec![vref, one]);
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
        let cond = self.append_instr(Opcode::EqFlags, vec![flags]);

        self.append_jne(cond, true_blk, false_blk);
      }
      instr::Opcode::OP_JNE => {
        let instr::Operand::Rel(rel) = &ins.operands[0] else { panic!("Expected relative offset operand for JMP") };
        let end_addr = (ins.addr + ins.n_bytes) as u16;
        let effective = Address(end_addr.wrapping_add(rel.val).into());

        let false_blk = self.get_block(Address(ins.addr + ins.n_bytes));
        let true_blk = self.get_block(effective);

        let flags = self.get_flags();
        let cond = self.append_instr(Opcode::NeqFlags, vec![flags]);

        self.append_jne(cond, true_blk, false_blk);
      }
      instr::Opcode::OP_JG => {
        let instr::Operand::Rel(rel) = &ins.operands[0] else { panic!("Expected relative offset operand for JMP") };
        let end_addr = (ins.addr + ins.n_bytes) as u16;
        let effective = Address(end_addr.wrapping_add(rel.val).into());

        let false_blk = self.get_block(Address(ins.addr + ins.n_bytes));
        let true_blk = self.get_block(effective);

        let flags = self.get_flags();
        let cond = self.append_instr(Opcode::GtFlags, vec![flags]);

        self.append_jne(cond, true_blk, false_blk);
      }
      instr::Opcode::OP_JGE => {
        let instr::Operand::Rel(rel) = &ins.operands[0] else { panic!("Expected relative offset operand for JMP") };
        let end_addr = (ins.addr + ins.n_bytes) as u16;
        let effective = Address(end_addr.wrapping_add(rel.val).into());

        let false_blk = self.get_block(Address(ins.addr + ins.n_bytes));
        let true_blk = self.get_block(effective);

        let flags = self.get_flags();
        let cond = self.append_instr(Opcode::GeqFlags, vec![flags]);

        self.append_jne(cond, true_blk, false_blk);
      }
      instr::Opcode::OP_JL => {
        let instr::Operand::Rel(rel) = &ins.operands[0] else { panic!("Expected relative offset operand for JMP") };
        let end_addr = (ins.addr + ins.n_bytes) as u16;
        let effective = Address(end_addr.wrapping_add(rel.val).into());

        let false_blk = self.get_block(Address(ins.addr + ins.n_bytes));
        let true_blk = self.get_block(effective);

        let flags = self.get_flags();
        let cond = self.append_instr(Opcode::LtFlags, vec![flags]);

        self.append_jne(cond, true_blk, false_blk);
      }
      instr::Opcode::OP_JLE => {
        let instr::Operand::Rel(rel) = &ins.operands[0] else { panic!("Expected relative offset operand for JMP") };
        let end_addr = (ins.addr + ins.n_bytes) as u16;
        let effective = Address(end_addr.wrapping_add(rel.val).into());

        let false_blk = self.get_block(Address(ins.addr + ins.n_bytes));
        let true_blk = self.get_block(effective);

        let flags = self.get_flags();
        let cond = self.append_instr(Opcode::LeqFlags, vec![flags]);

        self.append_jne(cond, true_blk, false_blk);
      }
      instr::Opcode::OP_CALLF => {
        let instr::Operand::Far(far) = &ins.operands[0] else { panic!("Unsupported OP_CALLF operand") };
        let seg = self.ir.append_const(far.seg.into());
        let off = self.ir.append_const(far.off.into());
        let ret = self.append_instr(Opcode::Call, vec![seg, off]);
        self.set_var(instr::Reg::AX, self.cur, ret);
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

    // Step 4: walk blocks and seal them
    for i in 0..self.ir.blocks.len() {
      if !self.blkmeta[i].sealed {
        self.seal_block(BlockRef(i));
      }
    }
  }
}

pub fn from_instrs(instrs: &[instr::Instr]) -> IR {
  let mut bld = IRBuilder::new();
  bld.build_from_instrs(instrs);
  bld.ir
}
