#pragma once
#include "header.h"

#define OPERATIONS(_)     \
  _( OP_INVAL,  "inval" ) \
  _( OP_ADC,    "adc"   ) \
  _( OP_ADD,    "add"   ) \
  _( OP_AND,    "and"   ) \
  _( OP_CALL,   "call"  ) \
  _( OP_CLI,    "cli"   ) \
  _( OP_CLD,    "cld"   ) \
  _( OP_CMP,    "cmp"   ) \
  _( OP_DEC,    "dec"   ) \
  _( OP_DIV,    "div"   ) \
  _( OP_HLT,    "hlt"   ) \
  _( OP_IDIV,   "idiv"  ) \
  _( OP_IMUL,   "imul"  ) \
  _( OP_IN,     "in"    ) \
  _( OP_INC,    "inc"   ) \
  _( OP_INT,    "int"   ) \
  _( OP_IRET,   "iret"  ) \
  _( OP_JA,     "ja"    ) \
  _( OP_JAE,    "jae"   ) \
  _( OP_JB,     "jb"    ) \
  _( OP_JBE,    "jbe"   ) \
  _( OP_JCXZ,   "jcxz"  ) \
  _( OP_JE,     "je"    ) \
  _( OP_JG,     "jg"    ) \
  _( OP_JGE,    "jge"   ) \
  _( OP_JL,     "jl"    ) \
  _( OP_JLE,    "jle"   ) \
  _( OP_JMP,    "jmp"   ) \
  _( OP_JNE,    "jne"   ) \
  _( OP_JZ,     "jz"    ) \
  _( OP_LDS,    "lds"   ) \
  _( OP_LES,    "les"   ) \
  _( OP_MOV,    "mov"   ) \
  _( OP_MUL,    "mul"   ) \
  _( OP_NEG,    "neg"   ) \
  _( OP_NOP,    "nop"   ) \
  _( OP_NOT,    "not"   ) \
  _( OP_OR,     "or"    ) \
  _( OP_OUT,    "out"   ) \
  _( OP_POP,    "pop"   ) \
  _( OP_PUSH,   "push"  ) \
  _( OP_REP,    "rep"   ) \
  _( OP_REPNE,  "repne" ) \
  _( OP_RET,    "ret"   ) \
  _( OP_RETF,   "retf"  ) \
  _( OP_ROL,    "rol"   ) \
  _( OP_ROR,    "ror"   ) \
  _( OP_SBB,    "sbb"   ) \
  _( OP_SCAS,   "scas"  ) \
  _( OP_SAR,    "sar"   ) \
  _( OP_SHL,    "shl"   ) \
  _( OP_SHR,    "shr"   ) \
  _( OP_STD,    "std"   ) \
  _( OP_STI,    "sti"   ) \
  _( OP_STOS,   "stos"  ) \
  _( OP_SUB,    "sub"   ) \
  _( OP_XOR,    "xor"   ) \

enum {
#define ELT(x, _) x,
  OPERATIONS(ELT)
#undef ELT
};

static inline const char *opcode_str(int op)
{
  static const char *str[] = {
#define ELT(_, x) x,
    OPERATIONS(ELT)
#undef ELT
  };
  if (op >= ARRAY_SIZE(str)) FAIL("Invalid operation number: %u", op);
  return str[op];
}
