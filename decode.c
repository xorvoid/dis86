#include "dis86_private.h"

static void modrm_process(dis86_t *d, operand_t *operand1, operand_t *operand2, int has_seg_override, u8 seg_override)
{
  u8 modrm = bin_fetch_u8(d->b);
  u8 mod = modrm >> 6;
  u8 reg = (modrm >> 3) & 7;
  u8 rm = modrm & 7;

  if (mod == 0) {
    if (rm == 6) {
      /* Direct addressing: 16-bit address offset */
      u16 imm = bin_fetch_u16(d->b);
      *operand1 = operand_addr_imm(imm, has_seg_override, seg_override);
      *operand2 = operand_reg(reg);
    }
    else {
      /* Ordinary special mode */
      *operand1 = operand_addr_mode(rm, has_seg_override, seg_override);
      *operand2 = operand_reg(reg);
    }
  }

  /* Two register mode */
  else if(mod == 3) {
    *operand1 = operand_reg(rm);
    *operand2 = operand_reg(reg);
  }

  else {
    FAIL("Unsupported MOD/RM mode | mod=%u, rm=%u", mod, rm);
  }
}

static u8 binary_op[]  = { OP_ADD,  OP_OR,  OP_ADC, OP_SBB, OP_AND, OP_SUB,  OP_XOR, OP_CMP  };
static u8 unary_op[]   = { 0,       0,      OP_NOT, OP_NEG, OP_MUL, OP_IMUL, OP_DIV, OP_IDIV };
static u8 inc_dec_op[] = { OP_INC,  OP_DEC, 0,      0,      0,      0,      0,      0       };

static u8 arith_op(dis86_t *d, u8 *ops_tbl, operand_t *operand, int has_seg_override, u8 seg_override)
{
  u8 modrm = bin_fetch_u8(d->b);
  u8 mod = modrm >> 6;
  u8 opnum = (modrm >> 3) & 7;
  u8 rm = modrm & 7;

  u8 op = ops_tbl[opnum];
  if (!op) FAIL("Invalid instruction op encoding");

  // FIXME
  assert(mod == 3);

  *operand = operand_reg(rm);
  return op;
}

dis86_instr_t *dis86_next(dis86_t *d, size_t *addr, size_t *n_bytes)
{
  dis86_instr_t *ins = d->ins;
  memset(ins, 0, sizeof(*ins));

  size_t start_loc = bin_location(d->b);
  u8 b = bin_fetch_u8(d->b);

  /* handle any prefixes first */
  int has_seg_override;
  u8 seg_override;
  if (b == 0x2e) {
    has_seg_override = 1;
    seg_override = SREG_CS;
  } else if (b == 0x3e) {
    has_seg_override = 1;
    seg_override = SREG_DS;
  } else if (b == 0x26) {
    has_seg_override = 1;
    seg_override = SREG_ES;
  } else if (b == 0x36) {
    has_seg_override = 1;
    seg_override = SREG_SS;
  } else {
    has_seg_override = 0;
  }

  /* advance byte? */
  if (has_seg_override) {
    b = bin_fetch_u8(d->b);
  }

  /* handle rep and repne */
  ins->rep = REP_NONE;
  if (b == 0xf3) {
    ins->rep = REP_EQ;
  } else if (b == 0xf2) {
    ins->rep = REP_NE;
  }

  /* advance byte? */
  if (ins->rep) {
    b = bin_fetch_u8(d->b);
  }

  /* parse opcode now */
  u8 op = b;

  /* common layout unpack: <6-bit op> <1-bit d> <1-bit w> */
  u8 op_prefix = op & ~3;
  u8 op_d = (op>>1)&1;
  u8 op_w = op&1;

  /****************************************************************
   * Common artihmetic:
   *
   *   ADD  | +0x00 | 0x00 - 0x05 | 0
   *   OR   | +0x08 | 0x08 - 0x0d | 1
   *   ADC  | +0x10 | 0x10 - 0x15 | 2
   *   SBB  | +0x18 | 0x18 - 0x1d | 3
   *   AND  | +0x20 | 0x20 - 0x25 | 4
   *   SUB  | +0x28 | 0x28 - 0x2d | 5
   *   XOR  | +0x30 | 0x30 - 0x35 | 6
   *   CMP  | +0x38 | 0x38 - 0x3d | 7
   *
   * TEST:
   *  let (op_idx, op_off) = (op / 8, op % 8)
   *  assert(op_idx <= 7)
   *  assert(op_off <= 5)
   ****************************************************************/

  u8 arith_op_idx = op >> 3;
  u8 arith_op_off = op & 7;
  if (arith_op_idx <= 7 && arith_op_off <= 3) {
    // ARITH REG, R/M
    // NOTE: NOT HANDLING THE 0x4 and 0x5 cases yet! FIXME!
    ins->opcode = binary_op[arith_op_idx];
    ins->size_flag = op_w ? SIZE_FLAG_16 : SIZE_FLAG_8;
    modrm_process(d, &ins->operand[op_d], &ins->operand[1-op_d], has_seg_override, seg_override);

  /****************************************************************
   * Special arthmetic cases: 0x80, 0x81, 0x83 */

  } else if (op == 0x80) {
    /* BINARY_OP R/M8 IMM8 */
    ins->opcode = arith_op(d, binary_op, &ins->operand[0], has_seg_override, seg_override);
    ins->size_flag = SIZE_FLAG_8;
    ins->operand[1] = operand_imm(bin_fetch_u8(d->b));

  } else if (op == 0x81) {
    /* BINARY_OP R/M16 IMM16 */
    ins->opcode = arith_op(d, binary_op, &ins->operand[0], has_seg_override, seg_override);
    ins->size_flag = SIZE_FLAG_16;
    ins->operand[1] = operand_imm(bin_fetch_u16(d->b));

  } else if (op == 0x83) {
    /* BINARY_OP R/M16 IMM8 */
    ins->opcode = arith_op(d, binary_op, &ins->operand[0], has_seg_override, seg_override);
    ins->size_flag = SIZE_FLAG_16;
    ins->operand[1] = operand_imm((i16)(i8)bin_fetch_u8(d->b));

  /****************************************************************
   * Conditional jumps */

  } else if (0x72 <= op && op <= 0x7f) {
    /* JUMP_PREDICATE REL8 */
    u8 opcode;
    switch (op) {
      case 0x72: opcode = OP_JB;    break;
      case 0x73: opcode = OP_JAE;   break;
      case 0x74: opcode = OP_JE;    break;
      case 0x75: opcode = OP_JNE;   break;
      case 0x76: opcode = OP_JBE;   break;
      case 0x77: opcode = OP_JA;    break;
      case 0x78: opcode = OP_INVAL; break;
      case 0x79: opcode = OP_INVAL; break;
      case 0x7a: opcode = OP_INVAL; break;
      case 0x7b: opcode = OP_INVAL; break;
      case 0x7c: opcode = OP_JL;    break;
      case 0x7d: opcode = OP_JGE;   break;
      case 0x7e: opcode = OP_JLE;   break;
      case 0x7f: opcode = OP_JG;    break;
    }
    if (!opcode) FAIL("Invalid opcode: 0x%x", op);

    u8 rel = bin_fetch_u8(d->b);
    ins->opcode = opcode;
    ins->operand[0] = operand_rel8(rel);

  /****************************************************************
   * Aboslute jumps */
  } else if (op == 0xe9) {
    u16 rel = bin_fetch_u16(d->b);
    ins->opcode = OP_JMP;
    ins->operand[0] = operand_rel16(rel);

  /****************************************************************/
  /* Less common layouts */
  } else if (0x40 <= op && op <= 0x47) {
    /* INC REG16 */
    u8 reg = op - 0x40;

    ins->opcode = OP_INC;
    ins->size_flag = SIZE_FLAG_16;
    ins->operand[0] = operand_reg(reg);

  } else if ((op & ~3) == 0x88) { /* all sizes and directions */
    /* MOV REG, R/M */
    u8 op_w = (op&1);
    u8 op_d = ((op>>1)&1);

    ins->opcode = OP_MOV;
    ins->size_flag = op_w ? SIZE_FLAG_16 : SIZE_FLAG_8;
    modrm_process(d, &ins->operand[op_d], &ins->operand[1-op_d], has_seg_override, seg_override);

  } else if ((op & ~3) == 0x8c) { /* only one size, all directions */
    /* MOV REG, SREG */
    u8 op_w = (op&1);
    u8 op_d = ((op>>1)&1);
    assert(op_w == 0);

    ins->opcode = OP_MOV;
    ins->size_flag = SIZE_FLAG_16;
    modrm_process(d, &ins->operand[op_d], &ins->operand[1-op_d], has_seg_override, seg_override);

    /* SREG, not REG (sort of a hack) */
    ins->operand[1-op_d].sreg = ins->operand[1-op_d].reg;
    ins->operand[1-op_d].has_reg = 0;
    ins->operand[1-op_d].has_sreg = 1;

  } else if ((op & ~3) == 0xa0) { /* all sizes, all directions */
    /* MOV MEM16, AX  or  MOV MEM8, AL */
    u8 op_w = (op&1);
    u8 op_d = ((op>>1)&1);

    u16 imm = op_w ? bin_fetch_u16(d->b) : bin_fetch_u8(d->b);

    ins->opcode = OP_MOV;
    ins->size_flag = op_w ? SIZE_FLAG_16 : SIZE_FLAG_8;
    ins->operand[op_d] = operand_reg(0); /* AX or AL implied by opcode */
    ins->operand[1-op_d] = operand_addr_imm(imm, has_seg_override, seg_override);

  } else if ((op & ~1) == 0xae) { /* all sizes, one direction */
    /* SCAS AX, WORD PTR ES:[DI] (implied) */
    u8 op_w = (op&1);

    ins->opcode = OP_SCAS;
    ins->size_flag = op_w ? SIZE_FLAG_16 : SIZE_FLAG_8;
    ins->operand[0] = operand_reg(0); /* AX or AL implied by opcode */
    ins->operand[1] = operand_addr_reg(SREG_ES, REG16_DI); /* Always ES:[DI] */

  } else if (op == 0xcd) {
    /* INT IMM8 */
    u8 imm = bin_fetch_u8(d->b);
    ins->opcode = OP_INT;
    ins->size_flag = SIZE_FLAG_8;
    ins->operand[0] = operand_imm(imm);

  } else if (0xb0 <= op && op <= 0xb7) {
    /* MOV REG8 IMM8 */
    u8 reg = op - 0xb0;
    u8 imm = bin_fetch_u8(d->b);

    ins->opcode = OP_MOV;
    ins->size_flag = SIZE_FLAG_8;
    ins->operand[0] = operand_reg(reg);
    ins->operand[1] = operand_imm(imm);

  } else if (0xb8 <= op && op <= 0xbf) {
    /* MOV REG16 IMM16 */
    u8 reg = op - 0xb8;
    u16 imm = bin_fetch_u16(d->b);

    ins->opcode = OP_MOV;
    ins->size_flag = SIZE_FLAG_16;
    ins->operand[0] = operand_reg(reg);
    ins->operand[1] = operand_imm(imm);

  } else if (op == 0xc4) {
    // LES REG MEM32

    /* u8 b = bin_fetch_u8(); */
    /* printf("b: %x (%u)\n", b, b); */

    ins->opcode = OP_LES;
    ins->size_flag = SIZE_FLAG_32;
    modrm_process(d, &ins->operand[1], &ins->operand[0], has_seg_override, seg_override);

  /* } else if (op == 0xc5) { */
  /*   // LDS REG MEM32 */

  } else if (op == 0xe3) {
    // JCXZ REL8
    u8 imm = bin_fetch_u8(d->b);
    ins->opcode = OP_JCXZ;
    ins->operand[0] = operand_rel8(imm);

  } else if (op == 0xe8) {
    // CALL REL16
    u16 imm = bin_fetch_u16(d->b);
    ins->opcode = OP_CALL;
    ins->operand[0] = operand_rel16(imm);

  } else if (op == 0xd3) {
    // SKIP FIXME HAX
    // SHL instruction
    u8 _ = bin_fetch_u8(d->b);

  } else if (op == 0xf7) {
    // UNARY_OP R/M16
    ins->opcode = arith_op(d, unary_op, &ins->operand[0], has_seg_override, seg_override);
    ins->size_flag = SIZE_FLAG_16;

  } else if (op == 0xfc) {
    // CLD
    ins->opcode = OP_CLD;

  } else {
    FAIL("Unknown opcode: %x", op);
  }

  size_t end_loc = bin_location(d->b);

  *addr = start_loc;
  *n_bytes = end_loc - start_loc;
  return ins;
}
