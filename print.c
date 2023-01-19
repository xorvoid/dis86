#include "dis86_private.h"
#include <stdarg.h>

typedef struct str str_t;
struct str
{
  char *buf;
  size_t idx;
  size_t len;
};

static inline void str_init(str_t *s)
{
  s->buf = malloc(4);
  s->idx = 0;
  s->len = 4;
}

static inline char *str_to_cstr(str_t *s)
{
  char *ret = s->buf;
  s->buf = NULL;
  s->idx = 0;
  s->len = 0;
  return ret;
}

static inline void str_fmt(str_t *s, const char *fmt, ...)
{
  while (1) {
    va_list va;

    va_start(va, fmt);
    size_t n = vsnprintf(s->buf + s->idx, s->len - s->idx, fmt, va);
    va_end(va);

    if (s->idx + n < s->len) {
      s->idx += n;
      return;
    }

    /* resize */
    s->len *= 2;
    s->buf = realloc(s->buf, s->len);
    if (s->buf == NULL) FAIL("Failed to realloc buffer");
  }
}

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

char *dis86_print_intel_syntax(dis86_t *d, dis86_instr_t *ins, size_t addr, size_t n_bytes, bool with_detail)
{
  str_t s[1];
  str_init(s);

  if (with_detail) {
    str_fmt(s, "%8zx:\t", addr);
    for (size_t i = 0; i < n_bytes; i++) {
      u8 b = bin_byte_at(d->b, addr + i);
      str_fmt(s, "%02x ", b);
    }
    size_t used = n_bytes * 3;
    size_t remain = (used <= 21) ? 21 - used : 0;
    str_fmt(s, "%*s\t", (int)remain, " ");
  }

  str_t op_buf[1];
  str_init(op_buf);
  if (ins->rep == REP_EQ) {
    str_fmt(op_buf, "rep ");
  } else if (ins->rep == REP_NE) {
    str_fmt(op_buf, "repnz ");
  }
  str_fmt(op_buf, "%s", opcode_str(ins->opcode));

  char *op_str = str_to_cstr(op_buf);
  str_fmt(s, "%-6s", op_str);
  free(op_str);

  for (size_t i = 0; i < ARRAY_SIZE(ins->operand); i++) {
    operand_t *o = &ins->operand[i];
    if (o->type == OPERAND_TYPE_NONE) {
      break;
    }

    if (i == 0) {
      str_fmt(s, " ");
    } else {
      str_fmt(s, ",");
    }

    if (o->type == OPERAND_TYPE_VAL) {
      if (o->has_reg) {
        str_fmt(s, "%s", reg_str(o->reg, ins->size_flag));
      }
      if (o->has_sreg) {
        str_fmt(s, "%s", sreg_str(o->sreg));
      }
      if (o->has_imm) {
        str_fmt(s, "0x%x", o->imm);
      }
      if (o->has_rel) {
        str_fmt(s, "0x%x", (u16)(addr + n_bytes + o->rel));
      }
    }

    if (o->type == OPERAND_TYPE_ADDR) {
      if (ins->size_flag == SIZE_FLAG_8) {
        str_fmt(s, "BYTE PTR ");
      } else if (ins->size_flag == SIZE_FLAG_16) {
        str_fmt(s, "WORD PTR ");
      } else if (ins->size_flag == SIZE_FLAG_32) {
        str_fmt(s, "DWORD PTR ");
      } else {
        FAIL("Expected size flag to be set");
      }
      if (o->has_seg_override) {
        str_fmt(s, "%s:", sreg_str(o->seg_override));
      } else {
        str_fmt(s, "ds:");
      }
      if (o->has_mode) {
        switch (o->mode) {
          case MODE_BX_PLUS_SI: str_fmt(s, "[bx+si]"); break;
          case MODE_BX_PLUS_DI: str_fmt(s, "[bx+di]"); break;
          case MODE_BP_PLUS_SI: str_fmt(s, "[bp+si]"); break;
          case MODE_BP_PLUS_DI: str_fmt(s, "[bp+di]"); break;
          case MODE_SI:         str_fmt(s, "[si]"); break;
          case MODE_DI:         str_fmt(s, "[di]"); break;
          case MODE_BP:         str_fmt(s, "[bp]"); break;
          case MODE_BX:         str_fmt(s, "[bx]"); break;
        }
      }
      if (o->has_reg) {
        str_fmt(s, "[%s]", reg_str(o->reg, SIZE_FLAG_16));
      }
      if (o->has_imm) {
        str_fmt(s, "0x%x", o->imm);
      }
    }
  }

  /* remove any trailing space */
  char *ret = str_to_cstr(s);
  size_t len = strlen(ret);
  while (len > 0) {
    if (ret[len-1] != ' ') break;
    len--;
  }
  ret[len] = '\0';

  return ret;
}
