#pragma once
#include "header.h"

enum {
  REG8_AL = 0,
  REG8_CL,
  REG8_DL,
  REG8_BL,
  REG8_AH,
  REG8_CH,
  REG8_DH,
  REG8_BH,
};

enum {
  REG16_AX = 0,
  REG16_CX,
  REG16_DX,
  REG16_BX,
  REG16_SP,
  REG16_BP,
  REG16_SI,
  REG16_DI,
};

enum {
  SREG_ES = 0,
  SREG_CS,
  SREG_SS,
  SREG_DS,
  SREG_INVAL4,
  SREG_INVAL5,
  SREG_INVAL6,
  SREG_INVAL7,
};

static inline const char *reg8_str(u8 r)
{
  static const char *REG8[8] = {
    /* 0 */ "al",
    /* 1 */ "cl",
    /* 2 */ "dl",
    /* 3 */ "bl",
    /* 4 */ "ah",
    /* 5 */ "ch",
    /* 6 */ "dh",
    /* 7 */ "bh",
  };
  if (r >= ARRAY_SIZE(REG8)) FAIL("Invalid 8-bit register number: %u", r);
  return REG8[r];
}

static inline const char *reg16_str(u8 r)
{
  static const char *REG16[8] = {
    /* 0 */ "ax",
    /* 1 */ "cx",
    /* 2 */ "dx",
    /* 3 */ "bx",
    /* 4 */ "sp",
    /* 5 */ "bp",
    /* 6 */ "si",
    /* 7 */ "di",
  };
  if (r >= ARRAY_SIZE(REG16)) FAIL("Invalid 16-bit register number: %u", r);
  return REG16[r];
}

static inline const char *sreg_str(u8 r)
{
  static const char *SREG[8] = {
    /* 0 */ "es",
    /* 1 */ "cs",
    /* 2 */ "ss",
    /* 3 */ "ds",
    /* 4 */ 0,
    /* 5 */ 0,
    /* 6 */ 0,
    /* 7 */ 0,
  };
  if (r >= ARRAY_SIZE(SREG)) FAIL("Invalid segment register number: %u", r);

  const char *s = SREG[r];
  if (!s) FAIL("Invalid segment register number: %u", r);

  return s;
}
