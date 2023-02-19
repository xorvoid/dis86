#include "dis86_private.h"
#include "instr_tbl.h"

// Reg is the 3-bits from [3..5] in the ModRM byte
static inline u8 decode_reg(u8 b) { return (b>>3)&7; }
static inline int decode_reg8(u8 b) { return REG_AL + (b>>3)&7; }
static inline int decode_reg16(u8 b) { return REG_AX + (b>>3)&7; }

// Opcode2 is the same as the reg-field in the ModRM byte
// That is: the 3-bits from [3..5] in the ModRM byte
static inline u8 decode_opcode2(u8 b) { return (b>>3)&7; }


static inline operand_t operand_reg(int id)
{
  operand_t o = {};
  o.type = OPERAND_TYPE_REG;
  o.u.reg.id = id;
  return o;
}

static inline operand_t operand_imm(u16 val)
{
  operand_t o = {};
  o.type = OPERAND_TYPE_IMM;
  o.u.imm.val = val;
  return o;
}

dis86_instr_t *dis86_next(dis86_t *d)
{
  dis86_instr_t *ins = d->ins;
  memset(ins, 0, sizeof(*ins));

  size_t start_loc = binary_location(d->b);
  int opcode1 = binary_fetch_u8(d->b);
  int opcode2 = -1;

  instr_fmt_t *fmt = NULL;
  int ret = instr_fmt_lookup(opcode1, opcode2, &fmt);

  // Need a level 2 opcode to do lookup?
  if (ret == RESULT_NEED_OPCODE2) {
    u8 b = binary_peek_u8(d->b);
    opcode2 = decode_opcode2(b);
    ret = instr_fmt_lookup(opcode1, opcode2, &fmt); // lookup again
  }

  // Verify everything
  if (ret != RESULT_SUCCESS) {
    FAIL("Failed to find instruction fmt for opcode1=0x%02x, opcode2=0x%02x\n", opcode1, opcode2);
  }

  int need_modrm = 0;
  operand_t * oper_reg8  = NULL;
  operand_t * oper_reg16 = NULL;
  //operand_t * oper_rm  = NULL;
  operand_t * oper_imm8 = NULL;
  operand_t * oper_imm16 = NULL;

  // Decode everything else
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
      case OPER_LIT1:  ins->operand[i] = operand_imm(1); break;
      case OPER_LIT3:  ins->operand[i] = operand_imm(3); break;

      // Implied string operations operands
      case OPER_SRC8:  UNIMPL(); break;
      case OPER_SRC16: UNIMPL(); break;
      case OPER_DST8:  UNIMPL(); break;
      case OPER_DST16: UNIMPL(); break;

      // Explicit register operands
      case OPER_R8:   need_modrm = 1; oper_reg8  = &ins->operand[i]; break;
      case OPER_R16:  need_modrm = 1; oper_reg16 = &ins->operand[i]; break;
      case OPER_SREG: UNIMPL(); break;

      // Explicit memory operands
      case OPER_M8:  UNIMPL(); break;
      case OPER_M16: UNIMPL(); break;
      case OPER_M32: UNIMPL(); break;

      // Explicit register or memory operands (modrm)
      case OPER_RM8:  UNIMPL(); break;
      case OPER_RM16: UNIMPL(); break;

      // Explicit immediate data
      case OPER_IMM8:  oper_imm8  = &ins->operand[i]; break;
      case OPER_IMM16: oper_imm16 = &ins->operand[i]; break;
      case OPER_IMM32: UNIMPL(); break;

      // Explicit relative offsets (branching / calls)
      case OPER_REL8:  UNIMPL(); break;
      case OPER_REL16: UNIMPL(); break;

      // Explicit 16-bit immediate used as a memory offset into DS
      case OPER_MOFF8:  UNIMPL(); break;
      case OPER_MOFF16: UNIMPL(); break;

      default: UNIMPL();
    }
  }


  // Need ModRM
  u8 modrm = need_modrm ? binary_fetch_u8(d->b) : 0;

  // Process all that the modrm implies
  if (oper_reg8)  *oper_reg8  = operand_reg(decode_reg8(modrm));
  if (oper_reg16) *oper_reg16 = operand_reg(decode_reg16(modrm));

  // Process any immediate data
  if (oper_imm8)  *oper_imm8  = operand_imm((i8)binary_fetch_u8(d->b));
  if (oper_imm16) *oper_imm16 = operand_imm(binary_fetch_u16(d->b));

  ins->addr = start_loc;
  ins->n_bytes = 1;

  return ins;
}
