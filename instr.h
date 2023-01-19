#pragma once
#include "header.h"
#include "operand.h"
#include "reg.h"
#include "oper.h"

#define REP_NONE  0
#define REP_EQ    1
#define REP_NE    2

#define SIZE_FLAG_NONE  0
#define SIZE_FLAG_8     1
#define SIZE_FLAG_16    2
#define SIZE_FLAG_32    3

enum {
  MODE_BX_PLUS_SI,
  MODE_BX_PLUS_DI,
  MODE_BP_PLUS_SI,
  MODE_BP_PLUS_DI,
  MODE_SI,
  MODE_DI,
  MODE_BP,
  MODE_BX,
};

struct dis86_instr
{
  int       rep;
  int       opcode;     /* operation enum (not 8086 opcode) */
  int       size_flag;  /* SIZE_FLAG_* */
  operand_t operand[2]; /* operands */
};
