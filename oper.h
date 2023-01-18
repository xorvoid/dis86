#pragma once
#include "header.h"

#define OPERATIONS(_)    \
  _( OP_MOV,    "mov"  ) \
  _( OP_CALL,   "call" ) \
  _( OP_INT,    "int"  ) \
  _( OP_LES,    "les"  ) \
  _( OP_LDS,    "lds"  ) \
  _( OP_CLD,    "cld"  ) \
  _( OP_SCAS,   "scas" ) \
  _( OP_JCXZ,   "jcxz" ) \
  _( OP_INC,    "inc" ) \

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
