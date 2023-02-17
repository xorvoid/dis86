#pragma once

#include "dis86.h"
#include "header.h"
#include "bin.h"
#include "instr.h"

struct dis86
{
  bin_t b[1];
  dis86_instr_t ins[1];
};
