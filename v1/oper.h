#pragma once
#include "header.h"

#define OPER_C_UNAVAIL 0
#define OPER_C_FUNC    1
#define OPER_C_RFUNC   2
#define OPER_C_INFIX   3
#define OPER_C_LITERAL 4

#define OPERATIONS(_)     \
  _( OP_INVAL,  "inval", "",          OPER_C_UNAVAIL ) \
  _( OP_ADC,    "adc",   "",          OPER_C_UNAVAIL ) \
  _( OP_ADD,    "add",   "+=",        OPER_C_INFIX   ) \
  _( OP_AND,    "and",   "&=",        OPER_C_INFIX   ) \
  _( OP_CALL,   "call",  "CALL",      OPER_C_FUNC    ) \
  _( OP_CLI,    "cli",   "",          OPER_C_UNAVAIL ) \
  _( OP_CLD,    "cld",   "",          OPER_C_UNAVAIL ) \
  _( OP_CMP,    "cmp",   "",          OPER_C_UNAVAIL ) \
  _( OP_DEC,    "dec",   "-= 1",      OPER_C_INFIX   ) \
  _( OP_DIV,    "div",   "",          OPER_C_UNAVAIL ) \
  _( OP_HLT,    "hlt",   "",          OPER_C_UNAVAIL ) \
  _( OP_IDIV,   "idiv",  "",          OPER_C_UNAVAIL ) \
  _( OP_IMUL,   "imul",  "",          OPER_C_UNAVAIL ) \
  _( OP_IN,     "in",    "",          OPER_C_UNAVAIL ) \
  _( OP_INC,    "inc",   "+= 1",      OPER_C_INFIX   ) \
  _( OP_INT,    "int",   "",          OPER_C_UNAVAIL ) \
  _( OP_IRET,   "iret",  "",          OPER_C_UNAVAIL ) \
  _( OP_JA,     "ja",    "",          OPER_C_UNAVAIL ) \
  _( OP_JAE,    "jae",   "",          OPER_C_UNAVAIL ) \
  _( OP_JB,     "jb",    "",          OPER_C_UNAVAIL ) \
  _( OP_JBE,    "jbe",   "",          OPER_C_UNAVAIL ) \
  _( OP_JCXZ,   "jcxz",  "",          OPER_C_UNAVAIL ) \
  _( OP_JE,     "je",    "",          OPER_C_UNAVAIL ) \
  _( OP_JG,     "jg",    "",          OPER_C_UNAVAIL ) \
  _( OP_JGE,    "jge",   "",          OPER_C_UNAVAIL ) \
  _( OP_JL,     "jl",    "",          OPER_C_UNAVAIL ) \
  _( OP_JLE,    "jle",   "",          OPER_C_UNAVAIL ) \
  _( OP_JMP,    "jmp",   "",          OPER_C_UNAVAIL ) \
  _( OP_JNE,    "jne",   "",          OPER_C_UNAVAIL ) \
  _( OP_JZ,     "jz",    "",          OPER_C_UNAVAIL ) \
  _( OP_LDS,    "lds",   "",          OPER_C_UNAVAIL ) \
  _( OP_LEAVE,  "leave", "sp = bp; bp = POP()", OPER_C_LITERAL ) \
  _( OP_LES,    "les",   "",          OPER_C_UNAVAIL ) \
  _( OP_MOV,    "mov",   "=",         OPER_C_INFIX   ) \
  _( OP_MUL,    "mul",   "",          OPER_C_UNAVAIL ) \
  _( OP_NEG,    "neg",   "",          OPER_C_UNAVAIL ) \
  _( OP_NOP,    "nop",   "",          OPER_C_UNAVAIL ) \
  _( OP_NOT,    "not",   "",          OPER_C_UNAVAIL ) \
  _( OP_OR,     "or",    "|=",        OPER_C_INFIX   ) \
  _( OP_OUT,    "out",   "",          OPER_C_UNAVAIL ) \
  _( OP_POP,    "pop",   "POP",       OPER_C_RFUNC   ) \
  _( OP_PUSH,   "push",  "PUSH",      OPER_C_FUNC    ) \
  _( OP_REP,    "rep",   "",          OPER_C_UNAVAIL ) \
  _( OP_REPNE,  "repne", "",          OPER_C_UNAVAIL ) \
  _( OP_RET,    "ret",   "return",    OPER_C_LITERAL ) \
  _( OP_RETF,   "retf",  "",          OPER_C_UNAVAIL ) \
  _( OP_ROL,    "rol",   "",          OPER_C_UNAVAIL ) \
  _( OP_ROR,    "ror",   "",          OPER_C_UNAVAIL ) \
  _( OP_SBB,    "sbb",   "",          OPER_C_UNAVAIL ) \
  _( OP_SCAS,   "scas",  "",          OPER_C_UNAVAIL ) \
  _( OP_SAR,    "sar",   "",          OPER_C_UNAVAIL ) \
  _( OP_SHL,    "shl",   "<<=",       OPER_C_INFIX   ) \
  _( OP_SHR,    "shr",   ">>=",       OPER_C_INFIX   ) \
  _( OP_STD,    "std",   "",          OPER_C_UNAVAIL ) \
  _( OP_STI,    "sti",   "",          OPER_C_UNAVAIL ) \
  _( OP_STOS,   "stos",  "",          OPER_C_UNAVAIL ) \
  _( OP_SUB,    "sub",   "-=",        OPER_C_INFIX   ) \
  _( OP_XOR,    "xor",   "^=",        OPER_C_INFIX   ) \

enum {
#define ELT(x, _1, _2, _3) x,
  OPERATIONS(ELT)
#undef ELT
};

static inline const char *opcode_str(int op)
{
  static const char *str[] = {
#define ELT(_1, x, _2, _3) x,
    OPERATIONS(ELT)
#undef ELT
  };
  if (op >= ARRAY_SIZE(str)) FAIL("Invalid operation number: %u", op);
  return str[op];
}

static inline const char *opcode_c(int op)
{
  static const char *str[] = {
#define ELT(_1, _2, x, _3) x,
    OPERATIONS(ELT)
#undef ELT
  };
  if (op >= ARRAY_SIZE(str)) FAIL("Invalid operation number: %u", op);
  return str[op];
}

static inline int opcode_c_type(int op)
{
  static int ty[] = {
#define ELT(_1, _2, _3, x) x,
    OPERATIONS(ELT)
#undef ELT
  };
  if (op >= ARRAY_SIZE(ty)) FAIL("Invalid operation number: %u", op);
  return ty[op];
}
