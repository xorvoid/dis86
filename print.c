#include "dis86_private.h"

static inline const char *reg_str(u8 r, u8 size_flag)
{
  if (size_flag == SIZE_FLAG_8) {
    return reg8_str(r);
  } else if (size_flag == SIZE_FLAG_16 || size_flag == SIZE_FLAG_32) {
    return reg16_str(r);
  } else {
    FAIL("Invalid sz flag: %u", size_flag);
  }
}

void dis86_print_intel_syntax(dis86_t *d, dis86_instr_t *ins, size_t addr, size_t n_bytes)
{
  printf("%8zx:\t", addr);
  for (size_t i = 0; i < n_bytes; i++) {
    u8 b = bin_byte_at(d->b, addr + i);
    printf("%02x ", b);
  }
  size_t used = n_bytes * 3;
  size_t remain = (used <= 21) ? 21 - used : 0;
  printf("%*s\t", (int)remain, " ");

  if (ins->rep == REP_EQ) {
    printf("rep ");
  } else if (ins->rep == REP_NE) {
    printf("repnz ");
  }

  printf("%-6s", opcode_str(ins->opcode));
  for (size_t i = 0; i < ARRAY_SIZE(ins->operand); i++) {
    operand_t *o = &ins->operand[i];
    if (o->type == OPERAND_TYPE_NONE) {
      break;
    }

    if (i == 0) {
      printf(" ");
    } else {
      printf(",");
    }

    if (o->type == OPERAND_TYPE_VAL) {
      if (o->has_reg) {
        printf("%s", reg_str(o->reg, ins->size_flag));
      }
      if (o->has_sreg) {
        printf("%s", sreg_str(o->sreg));
      }
      if (o->has_imm) {
        printf("0x%x", o->imm);
      }
      if (o->has_rel) {
        printf("0x%x", (u16)(addr + n_bytes + o->rel));
      }
    }

    if (o->type == OPERAND_TYPE_ADDR) {
      if (ins->size_flag == SIZE_FLAG_8) {
        printf("BYTE PTR ");
      } else if (ins->size_flag == SIZE_FLAG_16) {
        printf("WORD PTR ");
      } else if (ins->size_flag == SIZE_FLAG_32) {
        printf("DWORD PTR ");
      } else {
        FAIL("Expected size flag to be set");
      }
      if (o->has_seg_override) {
        printf("%s:", sreg_str(o->seg_override));
      } else {
        printf("ds:");
      }
      if (o->has_mode) {
        switch (o->mode) {
          case MODE_BX_PLUS_SI: printf("[bx+si]"); break;
          case MODE_BX_PLUS_DI: printf("[bx+di]"); break;
          case MODE_BP_PLUS_SI: printf("[bp+si]"); break;
          case MODE_BP_PLUS_DI: printf("[bp+di]"); break;
          case MODE_SI:         printf("[si]"); break;
          case MODE_DI:         printf("[di]"); break;
          case MODE_BP:         printf("[bp]"); break;
          case MODE_BX:         printf("[bx]"); break;
        }
      }
      if (o->has_reg) {
        printf("[%s]", reg_str(o->reg, SIZE_FLAG_16));
      }
      if (o->has_imm) {
        printf("0x%x", o->imm);
      }
    }
  }
  printf("\n");
}
