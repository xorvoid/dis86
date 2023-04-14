#include "decompile_private.h"

#define DEBUG_REPORT_SYMBOLS 0

static const char *n_bytes_as_type(u16 n_bytes)
{
  switch (n_bytes) {
    case 1: return "u8";
    case 2: return "u16";
    case 4: return "u32";
    default: FAIL("Unknown size type | n_bytes: %u", n_bytes);
  }
}

typedef struct decompiler decompiler_t;
struct decompiler
{
  dis86_t *                  dis;
  dis86_decompile_config_t * cfg;
  dis86_decompile_config_t * default_cfg;
  const char *               func_name;
  dis86_instr_t *            ins;
  size_t                     n_ins;

  symbols_t * symbols;
  labels_t    labels[1];

  meh_t *meh;
};

static decompiler_t * decompiler_new( dis86_t *                  dis,
                                      dis86_decompile_config_t * opt_cfg,
                                      const char *               func_name,
                                      dis86_instr_t *            ins_arr,
                                      size_t                     n_ins )

{
  decompiler_t *d = calloc(1, sizeof(decompiler_t));
  d->dis = dis;
  d->cfg = opt_cfg;
  if (!d->cfg) {
    d->default_cfg = config_default_new();
    d->cfg = d->default_cfg;
  }
  d->func_name = func_name;
  d->ins       = ins_arr;
  d->n_ins     = n_ins;

  d->symbols = symbols_new();
  d->meh = NULL;
  return d;
}

static void decompiler_delete(decompiler_t *d)
{
  if (d->meh) meh_delete(d->meh);
  if (d->default_cfg) config_delete(d->default_cfg);
  symbols_delete(d->symbols);
  free(d);
}

static void dump_symtab(symtab_t *symtab)
{
  symtab_iter_t it[1];
  symtab_iter_begin(it, symtab);
  while (1) {
    sym_t *var = symtab_iter_next(it);
    if (!var) break;

    static char buf[128];
    const char *size;
    if (var->len <= 4) {
      size = n_bytes_as_type(var->len);
    } else {
      size = "UNKNOWN";
    }
    LOG_INFO("  %-30s | %04x | %6u | %s", sym_name(var, buf, ARRAY_SIZE(buf)), (u16)var->off, var->len, size);
  }
}

static void decompiler_initial_analysis(decompiler_t *d)
{
  // Pass to find all labels
  find_labels(d->labels, d->ins, d->n_ins);

  // Populate registers
  for (int reg_id = 1; reg_id < _REG_LAST; reg_id++) {
    sym_t deduced_sym[1];
    sym_deduce_reg(deduced_sym, reg_id);
    if (deduced_sym->len != 2) continue; // skip the small overlap regs
    symbols_insert_deduced(d->symbols, deduced_sym);
  }

  // Load all global symbols from config into the symtab
  for (size_t i = 0; i < d->cfg->global_len; i++) {
    config_global_t *g = &d->cfg->global_arr[i];

    type_t type[1];
    if (!type_parse(type, g->type)) {
      LOG_WARN("For global '%s', failed to parse type '%s' ... skipping", g->name, g->type);
      continue;
    }

    symbols_add_global(d->symbols, g->name, g->offset, type_size(type));
  }

  // Pass to locate all symbols
  for (size_t i = 0; i < d->n_ins; i++) {
    dis86_instr_t *ins = &d->ins[i];

    for (size_t j = 0; j < ARRAY_SIZE(ins->operand); j++) {
      operand_t *o = &ins->operand[j];
      if (o->type != OPERAND_TYPE_MEM) continue;

      sym_t deduced_sym[1];
      if (!sym_deduce(deduced_sym, &o->u.mem)) continue;

      if (!symbols_insert_deduced(d->symbols, deduced_sym)) {
        static char buf[128];
        const char *name = sym_name(deduced_sym, buf, ARRAY_SIZE(buf));
        LOG_WARN("Unknown global | name: %s  off: 0x%04x  size: %u", name, (u16)deduced_sym->off, deduced_sym->len);
      }
    }
  }

  // Pass to convert to expression structures
  d->meh = meh_new(d->cfg, d->symbols, d->ins, d->n_ins);

  // Report the symbols
  if (DEBUG_REPORT_SYMBOLS) {
    LOG_INFO("Registers:");
    dump_symtab(d->symbols->registers);
    LOG_INFO("Globals:");
    dump_symtab(d->symbols->globals);
    LOG_INFO("Params:");
    dump_symtab(d->symbols->params);
    LOG_INFO("Locals:");
    dump_symtab(d->symbols->locals);
  }
}

static void decompiler_emit_preamble(decompiler_t *d, str_t *s)
{
  static char buf[128];
  symtab_iter_t it[1];

  // Emit params
  symtab_iter_begin(it, d->symbols->params);
  while (1) {
    sym_t *var = symtab_iter_next(it);
    if (!var) break;

    const char *name = sym_name(var, buf, ARRAY_SIZE(buf));
    str_fmt(s, "#define %s ARG_%zu(0x%x)\n", name, 8*sym_size_bytes(var), var->off);
  }

  // Emit locals
  symtab_iter_begin(it, d->symbols->locals);
  while (1) {
    sym_t *var = symtab_iter_next(it);
    if (!var) break;

    const char *name = sym_name(var, buf, ARRAY_SIZE(buf));
    str_fmt(s, "#define %s LOCAL_%zu(0x%x)\n", name, 8*sym_size_bytes(var), -var->off);
  }

  str_fmt(s, "void %s(void)\n", d->func_name);
  str_fmt(s, "{\n");
}

static void decompiler_emit_postamble(decompiler_t *d, str_t *s)
{
  static char buf[128];
  symtab_iter_t it[1];

  str_fmt(s, "}\n");

  // Cleanup params
  symtab_iter_begin(it, d->symbols->params);
  while (1) {
    sym_t *var = symtab_iter_next(it);
    if (!var) break;
    str_fmt(s, "#undef %s\n", sym_name(var, buf, ARRAY_SIZE(buf)));
  }

  // Cleanup locals
  symtab_iter_begin(it, d->symbols->locals);
  while (1) {
    sym_t *var = symtab_iter_next(it);
    if (!var) break;
    str_fmt(s, "#undef %s\n", sym_name(var, buf, ARRAY_SIZE(buf)));
  }
}

static void symref_lvalue_str(symref_t ref, const char *name, str_t *s)
{
  assert(ref.symbol);

  if (ref.off == 0 && ref.len == ref.symbol->len) {
    str_fmt(s, "%s", name);
  }

  else {
    str_fmt(s, "*(%s*)((u8*)&%s + %u)", n_bytes_as_type(ref.len), name, ref.off);
  }
}

static void symref_rvalue_str(symref_t ref, const char *name, str_t *s)
{
  assert(ref.symbol);
  if (ref.off == 0) {
    if (ref.len == ref.symbol->len) {
      str_fmt(s, "%s", name);
    } else {
      // Offset is the same, so just truncate it down
      str_fmt(s, "(%s)%s", n_bytes_as_type(ref.len), name);
    }
  }

  else {
    u16 bits = 8 * ref.off;
    str_fmt(s, "(%s)(%s>>%u)", n_bytes_as_type(ref.len), name, bits);
  }
}

static void value_str(value_t *v, str_t *s, bool as_lvalue)
{
  static char buf[128];

  switch (v->type) {
    case VALUE_TYPE_SYM: {
      const char *name = sym_name(v->u.sym->ref.symbol, buf, ARRAY_SIZE(buf));
      if (as_lvalue) {
        symref_lvalue_str(v->u.sym->ref, name, s);
      } else {
        symref_rvalue_str(v->u.sym->ref, name, s);
      }
    } break;
    case VALUE_TYPE_MEM: {
      value_mem_t *m = v->u.mem;
      switch (m->sz) {
        case SIZE_8:  str_fmt(s, "*PTR_8("); break;
        case SIZE_16: str_fmt(s, "*PTR_16("); break;
        case SIZE_32: str_fmt(s, "*PTR_32("); break;
      }
      str_fmt(s, "%s, ", sym_name(m->sreg.symbol, buf, ARRAY_SIZE(buf)));
      // FIXME: THIS IS ALL BROKEN BECAUSE IT ASSUMES THE SYMREF ARE NEVER PARTIAL REFS
      if (!m->reg1.symbol && !m->reg2.symbol) {
        if (m->off) str_fmt(s, "0x%x", m->off);
      } else {
        if (m->reg1.symbol) str_fmt(s, "%s", sym_name(m->reg1.symbol, buf, ARRAY_SIZE(buf)));
        if (m->reg2.symbol) str_fmt(s, "+%s", sym_name(m->reg2.symbol, buf, ARRAY_SIZE(buf)));
        if (m->off) {
          i16 disp = (i16)m->off;
          if (disp >= 0) str_fmt(s, "+0x%x", (u16)disp);
          else           str_fmt(s, "-0x%x", (u16)-disp);
        }
      }
      str_fmt(s, ")");
    } break;
    case VALUE_TYPE_IMM: {
      str_fmt(s, "0x%x", v->u.imm->value);
    } break;
    default: FAIL("Unknown value type: %d\n", v->type);
  }
}

static void decompiler_emit_expr(decompiler_t *d, expr_t *expr, str_t *ret_s)
{
  str_t s[1];
  str_init(s);

  switch (expr->kind) {
    case EXPR_KIND_UNKNOWN: {
      str_fmt(s, "UNKNOWN();");
    } break;
    case EXPR_KIND_OPERATOR1: {
      expr_operator1_t *k = expr->k.operator1;
      assert(!k->operator.sign); // not sure what this would mean...
      value_str(&k->dest, s, true);
      str_fmt(s, " %s ", k->operator);
      str_fmt(s, ";");
    } break;
    case EXPR_KIND_OPERATOR2: {
      expr_operator2_t *k = expr->k.operator2;
      if (k->operator.sign) str_fmt(s, "(i16)");
      value_str(&k->dest, s, true);
      str_fmt(s, " %s ", k->operator);
      if (k->operator.sign) str_fmt(s, "(i16)");
      value_str(&k->src, s, false);
      str_fmt(s, ";");
    } break;
    case EXPR_KIND_OPERATOR3: {
      expr_operator3_t *k = expr->k.operator3;
      value_str(&k->dest, s, true);
      str_fmt(s, " = ");
      if (k->operator.sign) str_fmt(s, "(i16)");
      value_str(&k->left, s, false);
      str_fmt(s, " %s ", k->operator);
      if (k->operator.sign) str_fmt(s, "(i16)");
      value_str(&k->right, s, false);
      str_fmt(s, ";");
    } break;
    case EXPR_KIND_FUNCTION: {
      expr_function_t *k = expr->k.function;
      if (!VALUE_IS_NONE(k->ret)) {
        value_str(&k->ret, s, true);
        str_fmt(s, " = ");
      }
      str_fmt(s, "%s(", k->func_name);
      for (size_t i = 0; i < k->n_args; i++) {
        if (i != 0) str_fmt(s, ", ");
        value_str(&k->args[i], s, false);
      }
      str_fmt(s, ");");
    } break;
    case EXPR_KIND_LITERAL: {
      expr_literal_t *k = expr->k.literal;
      str_fmt(s, "%s", k->text);
    } break;
    case EXPR_KIND_BRANCH_COND: {
      expr_branch_cond_t *k = expr->k.branch_cond;
      str_fmt(s, "if (");
      if (k->operator.sign) str_fmt(s, "(i16)");
      value_str(&k->left, s, false);
      str_fmt(s, " %s ", k->operator);
      if (k->operator.sign) str_fmt(s, "(i16)");
      value_str(&k->right, s, false);
      str_fmt(s, ") goto label_%08x;", k->target);
    } break;
    case EXPR_KIND_BRANCH: {
      expr_branch_t *k = expr->k.branch;
      str_fmt(s, "goto label_%08x;", k->target);
    } break;
    case EXPR_KIND_CALL: {
      expr_call_t *k = expr->k.call;
      if (k->name) {
        str_fmt(s, "CALL_FUNC(%s);", k->name);
      } else {
        switch (k->addr.type) {
          case ADDR_TYPE_FAR: {
            str_fmt(s, "CALL_FAR(0x%04x, 0x%04x);", k->addr.u.far.seg, k->addr.u.far.off);
          } break;
          case ADDR_TYPE_NEAR: {
            str_fmt(s, "CALL_NEAR(0x%04x);", k->addr.u.near);
          } break;
          default: {
            FAIL("Unknonw address type: %d", k->addr.type);
          } break;
        }
      }
      if (k->remapped) str_fmt(s, " /* remapped */");
    } break;
    case EXPR_KIND_LEA: {
      expr_lea_t *k = expr->k.lea;
      value_str(&k->dest, s, false);
      str_fmt(s, " = %s - 0x%x;", as_upper(reg_name(k->addr_base_reg)), -(i16)k->addr_offset);
    } break;
    default: {
      str_fmt(s, "UNIMPL();");
    } break;
  }

  const char *code_str = str_to_cstr(s);
  for (size_t i = 0; i < expr->n_ins; i++) {
    const char *as = dis86_print_intel_syntax(d->dis, &expr->ins[i], false);
    const char *cs = i+1 == expr->n_ins ? code_str : "";
    str_fmt(ret_s, "  %-50s // %s\n", cs, as);
    free((void*)as);
  }
  free((void*)code_str);
}

char *dis86_decompile( dis86_t *                  dis,
                       dis86_decompile_config_t * opt_cfg,
                       const char *               func_name,
                       dis86_instr_t *            ins_arr,
                       size_t                     n_ins )
{
  str_t ret_s[1];
  str_init(ret_s);

  decompiler_t *d = decompiler_new(dis, opt_cfg, func_name, ins_arr, n_ins);
  decompiler_initial_analysis(d);
  decompiler_emit_preamble(d, ret_s);

  for (size_t i = 0; i < d->meh->expr_len; i++) {
    expr_t *expr = &d->meh->expr_arr[i];
    if (is_label(d->labels, (u32)expr->ins->addr)) {
      str_fmt(ret_s, "\n label_%08x:\n", (u32)expr->ins->addr);
    }
    decompiler_emit_expr(d, expr, ret_s);
  }

  decompiler_emit_postamble(d, ret_s);
  return str_to_cstr(ret_s);
}
