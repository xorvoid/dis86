#include "dis86_private.h"
#include "instr_tbl.h"

// Register number to register enum
static inline int reg8(u8 num)  { assert(num <= 7); return REG_AL + num; }
static inline int reg16(u8 num) { assert(num <= 7); return REG_AX + num; }
static inline int sreg16(u8 num) { assert(num <= 3); return REG_ES + num; }

// Mode is the 2-bits from [6..7] in the ModRM byte
static inline u8 modrm_mode(u8 modrm) { return modrm>>6; }

// Reg is the 3-bits from [3..5] in the ModRM byte
static inline u8 modrm_reg(u8 modrm) { return (modrm>>3)&7; }

// Opcode2 is the same as the reg-field in the ModRM byte
// That is: the 3-bits from [3..5] in the ModRM byte
static inline u8 modrm_op2(u8 modrm) { return (modrm>>3)&7; }

// RM is the 3-bits from [0..2] in the ModRM byte
static inline u8 modrm_rm(u8 modrm) { return modrm&7; }

static inline operand_t operand_reg(int id)
{
  operand_t o = {};
  o.type = OPERAND_TYPE_REG;
  o.u.reg.id = id;
  return o;
}

static inline operand_t operand_imm8(u8 val)
{
  operand_t o = {};
  o.type = OPERAND_TYPE_IMM;
  o.u.imm.sz = SIZE_8;
  o.u.imm.val = val;
  return o;
}

static inline operand_t operand_imm16(u16 val)
{
  operand_t o = {};
  o.type = OPERAND_TYPE_IMM;
  o.u.imm.sz = SIZE_16;
  o.u.imm.val = val;
  return o;
}

static inline operand_t operand_rel(binary_t *b, int sz)
{
  operand_t o = {};
  o.type = OPERAND_TYPE_REL;
  if (sz == SIZE_8) {
    o.u.rel.val = (i8)binary_fetch_u8(b);
  } else if (sz == SIZE_16) {
    o.u.rel.val = binary_fetch_u16(b);
  } else {
    FAIL("Invalid size: %d", sz);
  }
  return o;
}

static inline operand_t operand_far(binary_t *b)
{
  u16 off = binary_fetch_u16(b);
  u16 seg = binary_fetch_u16(b);

  operand_t o = {};
  o.type = OPERAND_TYPE_FAR;
  o.u.far.seg = seg;
  o.u.far.off = off;
  return o;
}

static inline operand_t operand_moff(binary_t *b, int sz, int sreg)
{
  operand_t o = {};
  o.type = OPERAND_TYPE_MEM;
  o.u.mem.sz = sz;
  o.u.mem.sreg = sreg ? sreg : REG_DS;
  o.u.mem.reg1 = REG_INVAL;
  o.u.mem.reg2 = REG_INVAL;
  o.u.mem.off  = binary_fetch_u16(b);
  return o;
}

static inline operand_t _operand_rm(binary_t *b, int sz, u8 modrm, int sreg)
{
  u8 mode = modrm_mode(modrm);
  u8 rm = modrm_rm(modrm);

  // Handle special cases first
  if (mode == 3) { /* Register mode */
    if (sz == SIZE_8)       return operand_reg(reg8(rm));
    else if (sz == SIZE_16) return operand_reg(reg16(rm));
    else FAIL("Only 8-bit and 16-bit registers are allowed");
  }
  if (mode == 0 && rm == 6) { /* Direct addressing mode: 16-bit */
    return operand_moff(b, sz, sreg);
  }

  // Everything else uses some inderiect register mode
  operand_t o = {};
  o.type = OPERAND_TYPE_MEM;

  operand_mem_t *m = &o.u.mem;
  m->sz = sz;

  switch (rm) {
    case 0:  m->sreg = REG_DS;  m->reg1 = REG_BX;  m->reg2 = REG_SI;    break;
    case 1:  m->sreg = REG_DS;  m->reg1 = REG_BX;  m->reg2 = REG_DI;    break;
    case 2:  m->sreg = REG_SS;  m->reg1 = REG_BP;  m->reg2 = REG_SI;    break;
    case 3:  m->sreg = REG_SS;  m->reg1 = REG_BP;  m->reg2 = REG_DI;    break;
    case 4:  m->sreg = REG_DS;  m->reg1 = REG_SI;  m->reg2 = REG_INVAL; break;
    case 5:  m->sreg = REG_DS;  m->reg1 = REG_DI;  m->reg2 = REG_INVAL; break;
    case 6:  m->sreg = REG_SS;  m->reg1 = REG_BP;  m->reg2 = REG_INVAL; break;
    case 7:  m->sreg = REG_DS;  m->reg1 = REG_BX;  m->reg2 = REG_INVAL; break;
  }

  // Handle immediate dispacements
  if      (mode == 0)  m->off = 0;  /* none */
  else if (mode == 1)  m->off = (i8)binary_fetch_u8(b);
  else if (mode == 2)  m->off = binary_fetch_u16(b);

  // Apply sreg override (if required)
  if (sreg) m->sreg = sreg;

  return o;
}

static inline operand_t operand_rm8(binary_t *b, u8 modrm, int sreg)  { return _operand_rm(b, SIZE_8, modrm, sreg);  }
static inline operand_t operand_rm16(binary_t *b, u8 modrm, int sreg) { return _operand_rm(b, SIZE_16, modrm, sreg); }

static inline operand_t operand_m8(binary_t *b, u8 modrm, int sreg)
{
  operand_t o = _operand_rm(b, SIZE_8, modrm, sreg);
  if (o.type != OPERAND_TYPE_MEM) FAIL("Register used where memory operand was required");
  return o;
}

static inline operand_t operand_m16(binary_t *b, u8 modrm, int sreg)
{
  operand_t o = _operand_rm(b, SIZE_16, modrm, sreg);
  if (o.type != OPERAND_TYPE_MEM) FAIL("Register used where memory operand was required");
  return o;
}

static inline operand_t operand_m32(binary_t *b, u8 modrm, int sreg)
{
  operand_t o = _operand_rm(b, SIZE_32, modrm, sreg);
  if (o.type != OPERAND_TYPE_MEM) FAIL("Register used where memory operand was required");
  return o;
}

static inline operand_t operand_src(int sz)
{
  operand_t o = {};
  o.type = OPERAND_TYPE_MEM;
  o.u.mem.sz = sz;
  o.u.mem.sreg = REG_DS; // TODO FIMXE: ARE SEG OVERRIDES ALLOWED FOR THESE??
  o.u.mem.reg1 = REG_SI;
  o.u.mem.reg2 = REG_INVAL;
  o.u.mem.off  = 0;
  return o;
}

static inline operand_t operand_dst(int sz)
{
  operand_t o = {};
  o.type = OPERAND_TYPE_MEM;
  o.u.mem.sz = sz;
  o.u.mem.sreg = REG_ES; // TODO FIMXE: ARE SEG OVERRIDES ALLOWED FOR THESE??
  o.u.mem.reg1 = REG_DI;
  o.u.mem.reg2 = REG_INVAL;
  o.u.mem.off  = 0;
  return o;
}

dis86_instr_t *dis86_next(dis86_t *d)
{
  dis86_instr_t *ins = d->ins;
  memset(ins, 0, sizeof(*ins));

  size_t start_loc = binary_location(d->b);
  if (start_loc == binary_baseaddr(d->b) + binary_length(d->b)) {
    return NULL; // Reached the end
  }

  // First parse any prefixes
  int sreg = REG_INVAL;
  int rep = REP_NONE;
  while (1) {
    int b = binary_peek_u8(d->b);

    if      (b == 0x26) sreg = REG_ES;
    else if (b == 0x2e) sreg = REG_CS;
    else if (b == 0x36) sreg = REG_SS;
    else if (b == 0x3e) sreg = REG_DS;
    else if (b == 0x3e) sreg = REG_DS;
    else if (b == 0xf2) rep = REP_NE;
    else if (b == 0xf3) rep = REP_E;
    else break; // not a prefix!

    binary_advance_u8(d->b);
  }

  // Now parse the main level1 opcode
  int opcode1 = binary_fetch_u8(d->b);
  int opcode2 = -1;

  instr_fmt_t *fmt = NULL;
  int ret = instr_fmt_lookup(opcode1, opcode2, &fmt);

  // Need a level 2 opcode to do lookup?
  if (ret == RESULT_NEED_OPCODE2) {
    u8 b = binary_peek_u8(d->b);
    opcode2 = modrm_op2(b);
    ret = instr_fmt_lookup(opcode1, opcode2, &fmt); // lookup again
  }

  // Verify everything
  if (ret != RESULT_SUCCESS) {
    FAIL("Failed to find instruction fmt for opcode1=0x%02x, opcode2=0x%02x\n", opcode1, opcode2);
  }

  int need_modrm = 0;
  operand_t * oper_reg8     = NULL;
  operand_t * oper_reg16    = NULL;
  operand_t * oper_sreg     = NULL;
  operand_t * oper_rm8      = NULL;
  operand_t * oper_rm16     = NULL;
  operand_t * oper_m8       = NULL;
  operand_t * oper_m16      = NULL;
  operand_t * oper_m32      = NULL;
  operand_t * oper_imm8     = NULL;
  operand_t * oper_imm8_ext = NULL;
  operand_t * oper_imm16    = NULL;
  operand_t * oper_moff8    = NULL;
  operand_t * oper_moff16   = NULL;
  operand_t * oper_rel8     = NULL;
  operand_t * oper_rel16    = NULL;
  operand_t * oper_far32    = NULL;

  // Decode everything else
  ins->rep = rep;
  ins->opcode = fmt->op;
  for (size_t i = 0; i < OPERAND_MAX; i++) {
    int operand = (&fmt->operand1)[i];
    if (operand == -1) break; // done

    // Process and locate all the operands
    switch (operand) {
      // Implied 16-bit register operands
      case OPER_AX: ins->operand[i] = operand_reg(REG_AX); break;
      case OPER_CX: ins->operand[i] = operand_reg(REG_CX); break;
      case OPER_DX: ins->operand[i] = operand_reg(REG_DX); break;
      case OPER_BX: ins->operand[i] = operand_reg(REG_BX); break;
      case OPER_SP: ins->operand[i] = operand_reg(REG_SP); break;
      case OPER_BP: ins->operand[i] = operand_reg(REG_BP); break;
      case OPER_SI: ins->operand[i] = operand_reg(REG_SI); break;
      case OPER_DI: ins->operand[i] = operand_reg(REG_DI); break;

      // Implied 8-bit register operands
      case OPER_AL: ins->operand[i] = operand_reg(REG_AL); break;
      case OPER_CL: ins->operand[i] = operand_reg(REG_CL); break;
      case OPER_DL: ins->operand[i] = operand_reg(REG_DL); break;
      case OPER_BL: ins->operand[i] = operand_reg(REG_BL); break;
      case OPER_AH: ins->operand[i] = operand_reg(REG_AH); break;
      case OPER_CH: ins->operand[i] = operand_reg(REG_CH); break;
      case OPER_DH: ins->operand[i] = operand_reg(REG_DH); break;
      case OPER_BH: ins->operand[i] = operand_reg(REG_BH); break;

      // Implied segment regsiter operands
      case OPER_ES: ins->operand[i] = operand_reg(REG_ES); break;
      case OPER_CS: ins->operand[i] = operand_reg(REG_CS); break;
      case OPER_SS: ins->operand[i] = operand_reg(REG_SS); break;
      case OPER_DS: ins->operand[i] = operand_reg(REG_DS); break;

      // Implied others
      case OPER_FLAGS: ins->operand[i] = operand_reg(REG_FLAGS); break;
      case OPER_LIT1:  ins->operand[i] = operand_imm8(1); break;
      case OPER_LIT3:  ins->operand[i] = operand_imm8(3); break;

      // Implied string operations operands
      case OPER_SRC8:  ins->operand[i] = operand_src(SIZE_8);  break;
      case OPER_SRC16: ins->operand[i] = operand_src(SIZE_16); break;
      case OPER_DST8:  ins->operand[i] = operand_dst(SIZE_8);  break;
      case OPER_DST16: ins->operand[i] = operand_dst(SIZE_16); break;

      // Explicit register operands
      case OPER_R8:   need_modrm = 1; oper_reg8  = &ins->operand[i]; break;
      case OPER_R16:  need_modrm = 1; oper_reg16 = &ins->operand[i]; break;
      case OPER_SREG: need_modrm = 1; oper_sreg  = &ins->operand[i]; break;

      // Explicit memory operands
      case OPER_M8:  need_modrm = 1; oper_m8  = &ins->operand[i]; break;
      case OPER_M16: need_modrm = 1; oper_m16 = &ins->operand[i]; break;
      case OPER_M32: need_modrm = 1; oper_m32 = &ins->operand[i]; break;

      // Explicit register or memory operands (modrm)
      case OPER_RM8:  need_modrm = 1; oper_rm8  = &ins->operand[i]; break;
      case OPER_RM16: need_modrm = 1; oper_rm16 = &ins->operand[i]; break;

      // Explicit immediate data
      case OPER_IMM8:     oper_imm8     = &ins->operand[i]; break;
      case OPER_IMM8_EXT: oper_imm8_ext = &ins->operand[i]; break;
      case OPER_IMM16:    oper_imm16    = &ins->operand[i]; break;

      // Explicit far32 jump immediate
      case OPER_FAR32: oper_far32 = &ins->operand[i]; break;

      // Explicit 16-bit immediate used as a memory offset into DS
      case OPER_MOFF8:  oper_moff8  = &ins->operand[i]; break;
      case OPER_MOFF16: oper_moff16 = &ins->operand[i]; break;

      // Explicit relative offsets (branching / calls)
      case OPER_REL8:  oper_rel8  = &ins->operand[i]; break;
      case OPER_REL16: oper_rel16 = &ins->operand[i]; break;

      default:
        FAIL("Unexpected operand!");
    }
  }


  // Need ModRM
  u8 modrm = need_modrm ? binary_fetch_u8(d->b) : 0;

  // Process normal modrm reg
  if (oper_reg8)  *oper_reg8  = operand_reg(reg8(modrm_reg(modrm)));
  if (oper_reg16) *oper_reg16 = operand_reg(reg16(modrm_reg(modrm)));
  if (oper_sreg)  *oper_sreg  = operand_reg(sreg16(modrm_reg(modrm)));

  // Process modrm rm operands
  if (oper_rm8)  *oper_rm8  = operand_rm8(d->b, modrm, sreg);
  if (oper_rm16) *oper_rm16 = operand_rm16(d->b, modrm, sreg);
  if (oper_m8)   *oper_m8   = operand_m8(d->b, modrm, sreg);
  if (oper_m16)  *oper_m16  = operand_m16(d->b, modrm, sreg);
  if (oper_m32)  *oper_m32  = operand_m32(d->b, modrm, sreg);

  // Process any immediate data
  if (oper_imm8)     *oper_imm8     = operand_imm8(binary_fetch_u8(d->b));
  if (oper_imm8_ext) *oper_imm8_ext = operand_imm16((i8)binary_fetch_u8(d->b));
  if (oper_imm16)    *oper_imm16    = operand_imm16(binary_fetch_u16(d->b));

  // Process any memory offset immediates
  if (oper_moff8)  *oper_moff8  = operand_moff(d->b, SIZE_8, sreg);
  if (oper_moff16) *oper_moff16 = operand_moff(d->b, SIZE_16, sreg);

  // Process any relative offset immediates
  if (oper_rel8)  *oper_rel8  = operand_rel(d->b, SIZE_8);
  if (oper_rel16) *oper_rel16 = operand_rel(d->b, SIZE_16);

  // Process any far32 offset immediates
  if (oper_far32) *oper_far32 = operand_far(d->b);

  ins->addr = start_loc;
  ins->n_bytes = binary_location(d->b) - start_loc;
  ins->intel_hidden = fmt->intel_hidden;

  return ins;
}
