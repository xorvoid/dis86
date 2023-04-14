#pragma once
#include "header.h"

#define OPERAND_MAX 3

typedef struct operand       operand_t;
typedef struct operand_reg   operand_reg_t;
typedef struct operand_mem   operand_mem_t;
typedef struct operand_imm   operand_imm_t;
typedef struct operand_rel   operand_rel_t;
typedef struct operand_far   operand_far_t;
typedef struct instr_fmt     instr_fmt_t;

#define REGISTER_ARRAY(_)\
  /* Standard 16-bit registers */ \
  _( REG_AX,    16, "ax",    "AX"    )\
  _( REG_CX,    16, "cx",    "CX"    )\
  _( REG_DX,    16, "dx",    "DX"    )\
  _( REG_BX,    16, "bx",    "BX"    )\
  _( REG_SP,    16, "sp",    "SP"    )\
  _( REG_BP,    16, "bp",    "BP"    )\
  _( REG_SI,    16, "si",    "SI"    )\
  _( REG_DI,    16, "di",    "DI"    )\
  /* Standard 8-bit registers (may overlap with above) */\
  _( REG_AL,     8, "al",    "AL"    )\
  _( REG_CL,     8, "cl",    "CL"    )\
  _( REG_DL,     8, "dl",    "DL"    )\
  _( REG_BL,     8, "bl",    "BL"    )\
  _( REG_AH,     8, "ah",    "AH"    )\
  _( REG_CH,     8, "ch",    "CH"    )\
  _( REG_DH,     8, "dh",    "DH"    )\
  _( REG_BH,     8, "bh",    "BH"    )\
  /* Segment registers */\
  _( REG_ES,    16, "es",    "ES"    )\
  _( REG_CS,    16, "cs",    "CS"    )\
  _( REG_SS,    16, "ss",    "SS"    )\
  _( REG_DS,    16, "ds",    "DS"    )\
  /* Other registers */\
  _( REG_IP,    16, "ip",    "IP"    )\
  _( REG_FLAGS, 16, "flags", "FLAGS" )\

enum {
  REG_INVAL = 0,
#define ELT(r, _1, _2, _3) r,
  REGISTER_ARRAY(ELT)
#undef ELT
  _REG_LAST,
};

static inline const char *reg_name(int reg)
{
  static const char *arr[] = {
    NULL,
#define ELT(_1, _2, s, _3) s,
  REGISTER_ARRAY(ELT)
#undef ELT
  };
  if ((size_t)reg >= ARRAY_SIZE(arr)) return NULL;
  return arr[reg];
}

static inline const char *reg_name_upper(int reg)
{
  static const char *arr[] = {
    NULL,
#define ELT(_1, _2, _3, s) s,
  REGISTER_ARRAY(ELT)
#undef ELT
  };
  if ((size_t)reg >= ARRAY_SIZE(arr)) return NULL;
  return arr[reg];
}

enum {
  OPERAND_TYPE_NONE = 0,
  OPERAND_TYPE_REG,
  OPERAND_TYPE_MEM,
  OPERAND_TYPE_IMM,
  OPERAND_TYPE_REL,
  OPERAND_TYPE_FAR,
};

struct operand_reg
{
  int id;
};

enum {
  SIZE_8,
  SIZE_16,
  SIZE_32,
};

struct operand_mem
{
  int sz;   // SIZE_
  int sreg; // always must be populated
  int reg1; // 0 if unused
  int reg2; // 0 if unused
  u16 off;  // 0 if unused
};

struct operand_imm
{
  int sz;
  u16 val;
};

struct operand_rel
{
  u16 val;
};

struct operand_far
{
  u16 seg;
  u16 off;
};

struct operand
{
  int type;
  union {
    operand_reg_t reg;
    operand_mem_t mem;
    operand_imm_t imm;
    operand_rel_t rel;
    operand_far_t far;
  } u;
};

enum {
  REP_NONE = 0,
  REP_NE,
  REP_E,
};

struct dis86_instr
{
  int       rep;
  int       opcode;
  operand_t operand[OPERAND_MAX];
  size_t    addr;
  size_t    n_bytes;
  int       intel_hidden;   /* bitmap of operands hidden in intel assembly */
};

const char *instr_op_mneumonic(int op);

struct instr_fmt
{
  int op;             /* OP_ */
  int opcode1;        /* first byte: opcode */
  int opcode2;        /* 3-bit modrm reg field: sometimes used as level 2 opcode */
  int operand1;       /* OPER_ */
  int operand2;       /* OPER_ */
  int operand3;       /* OPER_ */
  int intel_hidden;   /* bitmap of operands hidden in intel assembly */
};

int instr_fmt_lookup(int opcode1, int opcode2, instr_fmt_t **fmt);
