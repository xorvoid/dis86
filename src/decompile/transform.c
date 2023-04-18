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

void transform_pass_or_jmp(meh_t *m)
{
  for (size_t i = 1; i < m->expr_len; i++) {
    expr_t *expr = &m->expr_arr[i];
    if (expr->kind != EXPR_KIND_BRANCH_FLAGS) continue;
    expr_branch_flags_t *k = expr->k.branch_flags;

    const char *cmp;
    if (0 == memcmp(k->op, "JE", 2)) cmp = "==";
    else if (0 == memcmp(k->op, "JNE", 3)) cmp = "!=";
    else continue;

    size_t prev_idx = i-1;
    expr_t *prev_expr = &m->expr_arr[i-1];
    if (prev_expr->kind != EXPR_KIND_OPERATOR2) continue;
    expr_operator2_t *p = prev_expr->k.operator2;
    if (0 != memcmp(p->operator.oper, "|=", 2)) continue;
    if (!value_matches(&p->dest, &p->src)) continue;

    // Save
    value_t src    = p->src;
    u32     target = k->target;

    // Rewrite
    prev_expr->kind = EXPR_KIND_BRANCH_COND;
    prev_expr->n_ins++;
    expr_branch_cond_t *b = prev_expr->k.branch_cond;
    b->operator.oper = cmp;
    b->operator.sign = 0;
    b->left     = src;
    b->right    = VALUE_IMM(0);
    b->target   = target;

    // Ignore the extra instruction
    m->expr_arr[i] = EXPR_NONE;
  }
}

void _synthesize_calls_one(meh_t *m, size_t i)
{
  expr_t *expr = &m->expr_arr[i];
  if (expr->kind != EXPR_KIND_CALL) return;

  expr_call_t *   k        = expr->k.call;
  addr_t          addr     = k->addr;
  bool            remapped = k->remapped;
  config_func_t * func     = k->func;

  if (!func || func->args < 0) return;
  if (i < (size_t)func->args) return;
  if (i+1 >= m->expr_len) return;

  // Check and extract arguments
  value_t args[MAX_ARGS];
  for (size_t j = 0; j < (size_t)func->args; j++) {
    size_t idx = i-1 - j;
    expr_t *arg_expr = &m->expr_arr[idx];
    if (arg_expr->kind != EXPR_KIND_ABSTRACT) return;
    expr_abstract_t *a = arg_expr->k.abstract;
    if (0 != memcmp(a->func_name, "PUSH", 4)) return;
    args[j] = a->args[0];
  }

  // Check for stack cleanup
  size_t num_cleanup_ins = 0;
  if (func->pop_args_after_call) {
    if (func->args > 1) {
      expr_t *cleanup_expr = &m->expr_arr[i+1];
      if (cleanup_expr->kind != EXPR_KIND_OPERATOR2) return;
      expr_operator2_t *c = cleanup_expr->k.operator2;
      if (0 != memcmp(c->operator.oper, "+=", 2)) return;
      if (c->dest.type != VALUE_TYPE_SYM) return;
      // FIXME!
      //if (!symref_matches(c->dest.u.sym->ref, symbols_find_reg(symbols, REG_SP))) return;
      if (c->src.type != VALUE_TYPE_IMM) return;
      u16 val = c->src.u.imm->value;
      if (val != 2*(size_t)func->args) return;
      num_cleanup_ins = 1;
    } else if (func->args == 1) {
      expr_t *cleanup_expr = &m->expr_arr[i+1];
      if (cleanup_expr->kind != EXPR_KIND_ABSTRACT) return;
      expr_abstract_t *a = cleanup_expr->k.abstract;
      if (0 != memcmp(a->func_name, "POP", 3)) return;
      num_cleanup_ins = 1;
    }
  }

  // Rewrite
  expr->kind = EXPR_KIND_CALL_WITH_ARGS;
  expr_call_with_args_t * a = expr->k.call_with_args;
  a->addr     = addr;
  a->remapped = remapped;
  a->func     = func;
  assert(ARRAY_SIZE(args) == ARRAY_SIZE(a->args));
  memcpy(a->args, args, sizeof(args));

  // Remove the old exprs
  dis86_instr_t *first_ins = NULL;
  size_t ins_count = 0;
  size_t n = (size_t)func->args + 1 + num_cleanup_ins;
  for (size_t j = 0; j < n; j++) {
    size_t idx = i - (size_t)func->args + j;
    if (!first_ins) {
      first_ins = m->expr_arr[idx].ins;
    }
    ins_count += m->expr_arr[idx].n_ins;
    if (i == idx) continue;
    m->expr_arr[idx] = EXPR_NONE;
  }

  // Update the ins array tracking
  expr->ins = first_ins;
  expr->n_ins = ins_count;
}

void transform_pass_synthesize_calls(meh_t *m)
{
  for (size_t i = 0; i < m->expr_len; i++) {
    _synthesize_calls_one(m, i);
  }
}
