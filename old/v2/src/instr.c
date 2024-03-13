#include "dis86_private.h"
#include "instr_tbl.h"

size_t dis86_instr_addr(dis86_instr_t *ins)
{
  return ins->addr;
}

size_t dis86_instr_n_bytes(dis86_instr_t *ins)
{
  return ins->n_bytes;
}

int instr_fmt_lookup(int opcode1, int opcode2, instr_fmt_t **_fmt)
{
  // TODO FIXME .. VERY INEFFICENT O(N) SEARCH
  // WE COULD DO A BINARY SEARCH, BUT REALLY WE SHOULD JUST USE A TABLE
  // THAT'S MUCH MORE EFFICENT FOR O(1) LOOKUPS. WE COULD EITHER BUILD
  // ANOTHER TABLE AT RUNTIME.. OR MORE IDEALLY, JUST REFORMT THE CURRENT
  // TABLE. THE PRIMARY CHALLENGE IS THE OPCODE2 THE WE OCCASIONALLY HAVE..
  // BUT NOT ALWAYS..

  int opcode1_found = 0;
  for (size_t i = 0; i < ARRAY_SIZE(instr_tbl); i++) {
    instr_fmt_t *fmt = &instr_tbl[i];
    if (opcode1 == fmt->opcode1) {
      if (fmt->op == OP_INVAL) return RESULT_NOT_FOUND;
      opcode1_found = 1;
      if (opcode2 == fmt->opcode2) {
        *_fmt = fmt;
        return RESULT_SUCCESS;
      }
    }
  }

  if (opcode1_found && opcode2 == -1) {
    return RESULT_NEED_OPCODE2;
  }

  return RESULT_NOT_FOUND;
}

const char *instr_op_mneumonic(int op)
{
  static const char *arr[] = {
#define ELT(_1, str) str,
    INSTR_OP_ARRAY(ELT)
    #undef ELT
  };
  if ((size_t)op >= ARRAY_SIZE(arr)) return NULL;
  return arr[op];
}

void dis86_instr_copy(dis86_instr_t *dst, dis86_instr_t *src)
{
  *dst = *src;
}
