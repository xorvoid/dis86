#include "decompile_private.h"

value_t value_from_operand(operand_t *o, symbols_t *symbols)
{
  value_t val[1] = {{}};

  switch (o->type) {
    case OPERAND_TYPE_REG: {
      sym_t deduced_sym[1];
      sym_deduce_reg(deduced_sym, o->u.reg.id);

      val->type = VALUE_TYPE_SYM;
      val->u.sym->ref = symbols_find_ref(symbols, deduced_sym);
      assert(val->u.sym->ref.symbol);
    } break;
    case OPERAND_TYPE_MEM: {
      operand_mem_t *m = &o->u.mem;
      symref_t ref = symbols_find_mem(symbols, m);
      if (ref.symbol) {
        val->type = VALUE_TYPE_SYM;
        val->u.sym->ref = ref;
      } else {
        val->type = VALUE_TYPE_MEM;
        val->u.mem->sz   = m->sz;
        val->u.mem->sreg = symbols_find_reg(symbols, m->sreg);
        val->u.mem->reg1 = symbols_find_reg(symbols, m->reg1);
        val->u.mem->reg2 = symbols_find_reg(symbols, m->reg2);
        val->u.mem->off  = m->off;
      }
    } break;
    case OPERAND_TYPE_IMM: {
      val->type         = VALUE_TYPE_IMM;
      val->u.imm->sz    = o->u.imm.sz;
      val->u.imm->value = o->u.imm.val;
    } break;
    case OPERAND_TYPE_REL: {
      FAIL("OPERAND_TYPE_REL UNIMPL");
    } break;
    case OPERAND_TYPE_FAR: {
      FAIL("OPERAND_TYPE_FAR UNIMPL");
    } break;
    default: FAIL("INVALID OPERAND TYPE: %d", o->type);
  }

  return *val;
}
