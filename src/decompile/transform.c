#include "decompile_private.h"

void transform_pass_xor_rr(meh_t *m)
{
  for (size_t i = 0; i < m->expr_len; i++) {
    expr_t *expr = &m->expr_arr[i];
    if (expr->kind != EXPR_KIND_OPERATOR2) continue;

    expr_operator2_t *k = expr->k.operator2;
    if (0 != memcmp(k->operator.oper, "^=", 2)) continue;
    if (k->dest.type != VALUE_TYPE_SYM) continue;
    if (k->src.type != VALUE_TYPE_SYM) continue;
    if (!value_matches(&k->dest, &k->src)) continue;

    k->operator.oper = "=";
    k->src = VALUE_IMM(0);
  }
}
