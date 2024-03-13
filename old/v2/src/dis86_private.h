#pragma once

#include "dis86.h"
#include "header.h"
#include "binary.h"
#include "instr.h"

enum {
  RESULT_SUCCESS = 0,
  RESULT_NEED_OPCODE2,
  RESULT_NOT_FOUND,
};

struct dis86
{
  binary_t b[1];
  dis86_instr_t ins[1];
};
