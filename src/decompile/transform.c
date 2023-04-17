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

    // Rewrite
    k->operator.oper = "=";
    k->src = VALUE_IMM(0);
  }
}

static operator_t jump_operation(const char *op)
{
  operator_t o = {};
  if (0 == strcmp(op, "JB"))  { o.oper = "<";  o.sign = 0; return o; }
  if (0 == strcmp(op, "JBE")) { o.oper = "<="; o.sign = 0; return o; }
  if (0 == strcmp(op, "JA"))  { o.oper = ">";  o.sign = 0; return o; }
  if (0 == strcmp(op, "JAE")) { o.oper = ">="; o.sign = 0; return o; }
  if (0 == strcmp(op, "JE"))  { o.oper = "=="; o.sign = 0; return o; }
  if (0 == strcmp(op, "JNE")) { o.oper = "!="; o.sign = 0; return o; }
  if (0 == strcmp(op, "JL"))  { o.oper = "<";  o.sign = 1; return o; }
  if (0 == strcmp(op, "JLE")) { o.oper = "<="; o.sign = 1; return o; }
  if (0 == strcmp(op, "JG"))  { o.oper = ">";  o.sign = 1; return o; }
  if (0 == strcmp(op, "JGE")) { o.oper = ">="; o.sign = 1; return o; }

  FAIL("Unexpected jump operation: '%s'", op);
}

void transform_pass_cmp_jmp(meh_t *m)
{
  for (size_t i = 1; i < m->expr_len; i++) {
    expr_t *expr = &m->expr_arr[i];
    if (expr->kind != EXPR_KIND_BRANCH_FLAGS) continue;

    expr_branch_flags_t *k = expr->k.branch_flags;
    size_t prev_idx = i-1;
    expr_t *prev_expr = &m->expr_arr[i-1];
    value_t prev_dest = expr_destination(prev_expr);
    if (!value_matches(&k->flags, &prev_dest)) continue;

    if (prev_expr->kind != EXPR_KIND_ABSTRACT) continue;
    expr_abstract_t *p = prev_expr->k.abstract;
    if (p->n_args != 2) continue;

    // Unpack values
    const char * name   = k->op;
    value_t      left   = p->args[0];
    value_t      right  = p->args[1];
    u32          target = k->target;

    // Rewrite
    prev_expr->kind = EXPR_KIND_BRANCH_COND;
    prev_expr->n_ins++;
    expr_branch_cond_t *b = prev_expr->k.branch_cond;
    b->operator = jump_operation(name);
    b->left     = left;
    b->right    = right;
    b->target   = target;

    // Ignore the extra instruction
    m->expr_arr[i] = EXPR_NONE;
  }
}

/* void transform_pass_or_jmp(meh_t *m) */
/* { */
/*   for (size_t i = 1; i < m->expr_len; i++) { */
/*     expr_t *expr = &m->expr_arr[i]; */
/*     if (expr->kind != EXPR_KIND_BRANCH_FLAGS) continue; */

/*     expr_branch_flags_t *k = expr->k.branch_flags; */
/*     size_t prev_idx = i-1; */
/*     expr_t *prev_expr = &m->expr_arr[i-1]; */
/*     value_t prev_dest = expr_destination(prev_expr); */
/*     if (!value_matches(&k->flags, &prev_dest)) continue; */

/*     if (prev_expr->kind != EXPR_KIND_ABSTRACT) continue; */
/*     expr_abstract_t *p = prev_expr->k.abstract; */
/*     if (p->n_args != 2) continue; */

/*     // Unpack values */
/*     const char * name   = k->op; */
/*     value_t      left   = p->args[0]; */
/*     value_t      right  = p->args[1]; */
/*     u32          target = k->target; */

/*     // Rewrite */
/*     prev_expr->kind = EXPR_KIND_BRANCH_COND; */
/*     prev_expr->n_ins++; */
/*     expr_branch_cond_t *b = prev_expr->k.branch_cond; */
/*     b->operator = jump_operation(name); */
/*     b->left     = left; */
/*     b->right    = right; */
/*     b->target   = target; */

/*     // Ignore the extra instruction */
/*     m->expr_arr[i] = EXPR_NONE; */
/*   } */
/* } */
