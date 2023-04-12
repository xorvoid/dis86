#include "decompile_private.h"
#include "str.h"

static int code_c_type[] = {
#define ELT(_1, _2, ty, _4) ty,
  INSTR_OP_ARRAY(ELT)
#undef ELT
};

static const char *code_c_str[] = {
#define ELT(_1, _2, _3, s) s,
  INSTR_OP_ARRAY(ELT)
#undef ELT
};


typedef struct decompiler decompiler_t;
struct decompiler
{
  dis86_t *                  dis;
  dis86_decompile_config_t * cfg;
  dis86_decompile_config_t * default_cfg;
  const char *               func_name;
  dis86_instr_t *            ins;
  size_t                     n_ins;

  symtab_t *sym;
  labels_t labels[1];
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

  d->sym = symtab_new();
  return d;
}

static void decompiler_delete(decompiler_t *d)
{
  if (d->default_cfg) config_delete(d->default_cfg);
  symtab_delete(d->sym);
  free(d);
}

static void decompiler_initial_analysis(decompiler_t *d)
{
  // Pass to find all labels
  find_labels(d->labels, d->ins, d->n_ins);

  // Pass to locate all symbols
  for (size_t i = 0; i < d->n_ins; i++) {
    dis86_instr_t *ins = &d->ins[i];

    for (size_t j = 0; j < ARRAY_SIZE(ins->operand); j++) {
      operand_t *o = &ins->operand[j];
      if (o->type != OPERAND_TYPE_MEM) continue;

      symtab_lookup_or_create(d->sym, &o->u.mem, NULL);
    }
  }

  // Report the symbols
  symtab_iter_t it[1];
  symtab_iter_begin(it, d->sym);
  while (1) {
    variable_t *var = symtab_iter_next(it);
    if (!var) break;

    static char buf[128];
    LOG_INFO("Symbol: %s", variable_name(var, buf, ARRAY_SIZE(buf)));
  }
}

static void decompiler_emit_preamble(decompiler_t *d, str_t *s)
{
  str_fmt(s, "void %s(void)\n", d->func_name);
  str_fmt(s, "{\n");
}

static void decompiler_emit_postamble(decompiler_t *d, str_t *s)
{
  str_fmt(s, "}\n");
}

static void operand_str(decompiler_t *d, str_t *s, dis86_instr_t *ins, operand_t *o)
{
  static char buf[128];

  switch (o->type) {
    case OPERAND_TYPE_REG: str_fmt(s, "%s", as_upper(reg_name(o->u.reg.id))); break;
    case OPERAND_TYPE_MEM: {
      operand_mem_t *m = &o->u.mem;

      bool created = false;
      variable_t *var = symtab_lookup_or_create(d->sym, m, &created);
      assert(!created);

      if (var) {
        str_fmt(s, "%s", variable_name(var, buf, ARRAY_SIZE(buf)));
        break; // all done
      }
      switch (m->sz) {
        case SIZE_8:  str_fmt(s, "*PTR_8("); break;
        case SIZE_16: str_fmt(s, "*PTR_16("); break;
        case SIZE_32: str_fmt(s, "*PTR_32("); break;
      }
      str_fmt(s, "%s, ", as_upper(reg_name(m->sreg)));
      if (!m->reg1 && !m->reg2) {
        if (m->off) str_fmt(s, "0x%x", m->off);
      } else {
        if (m->reg1) str_fmt(s, "%s", as_upper(reg_name(m->reg1)));
        if (m->reg2) str_fmt(s, "+%s", as_upper(reg_name(m->reg2)));
        if (m->off) {
          i16 disp = (i16)m->off;
          if (disp >= 0) str_fmt(s, "+0x%x", (u16)disp);
          else           str_fmt(s, "-0x%x", (u16)-disp);
        }
      }
      str_fmt(s, ")");
    } break;
    case OPERAND_TYPE_IMM: str_fmt(s, "0x%x", o->u.imm.val); break;
    case OPERAND_TYPE_REL: {
      u16 effective = ins->addr + ins->n_bytes + o->u.rel.val;
      str_fmt(s, "0x%x", effective);
    } break;
      //case OPERAND_TYPE_FAR: break;
    default: FAIL("INVALID OPERAND TYPE: %d", o->type);
  }
}

static void decompiler_process_ins(decompiler_t *d, size_t *ins_idx, str_t *ret_s)
{
  char buf[32];
  const char *cs, *as;

  str_t s[1];
  str_init(s);

  dis86_instr_t * ins      = &d->ins[*ins_idx];
  size_t          next_idx = *ins_idx + 1;
  dis86_instr_t * next_ins = next_idx < d->n_ins ? &d->ins[next_idx] : NULL;

  if (is_label(d->labels, (u32)ins->addr)) {
    str_fmt(ret_s, "\n label_%08x:\n", (u32)ins->addr);
  }

  // Special handling for cmp+jmp
  if (ins->opcode == OP_CMP) {
    const char *oper = NULL;
    int sign = 0;
    if (next_ins) {
      switch (next_ins->opcode) {
        case OP_JB:  oper = "<";  break;
        case OP_JBE: oper = "<="; break;
        case OP_JA:  oper = ">";  break;
        case OP_JAE: oper = ">="; break;
        case OP_JE:  oper = "=="; break;
        case OP_JNE: oper = "!="; break;
        case OP_JL:  oper = "<";  sign = 1; break;
        case OP_JLE: oper = "<="; sign = 1; break;
        case OP_JG:  oper = ">";  sign = 1; break;
        case OP_JGE: oper = ">="; sign = 1; break;
      }
      if (oper) {
        u32 dest = branch_destination(next_ins);
        str_fmt(s, "if (");
        if (sign) str_fmt(s, "(i16)");
        assert(ins->operand[0].type != OPERAND_TYPE_NONE);
        operand_str(d, s, ins, &ins->operand[0]);
        str_fmt(s, " %s ", oper);
        if (sign) str_fmt(s, "(i16)");
        assert(ins->operand[1].type != OPERAND_TYPE_NONE);
        operand_str(d, s, ins, &ins->operand[1]);
        str_fmt(s, ") goto label_%08x;", dest);

        as = dis86_print_intel_syntax(d->dis, ins, false);
        str_fmt(ret_s, "  %-50s // %s\n", "", as);
        free((void*)as);

        cs = str_to_cstr(s);
        as = dis86_print_intel_syntax(d->dis, next_ins, false);
        str_fmt(ret_s, "  %-50s // %s\n", cs, as);
        free((void*)as);
        free((void*)cs);

        (*ins_idx)++; // advance one extra
        return;
      }
    }
  }

  // Special handling for or reg,reg + je / jne
  if (ins->opcode == OP_OR &&
      ins->operand[0].type == OPERAND_TYPE_REG &&
      ins->operand[1].type == OPERAND_TYPE_REG &&
      ins->operand[0].u.reg.id == ins->operand[1].u.reg.id
      ) {
    const char *oper = NULL;
    if (next_ins) {
      switch (next_ins->opcode) {
        case OP_JE:  oper = "=="; break;
        case OP_JNE: oper = "!="; break;
      }
      if (oper) {
        u32 dest = branch_destination(next_ins);
        str_fmt(s, "if (");
        operand_str(d, s, ins, &ins->operand[0]);
        str_fmt(s, " %s 0) goto label_%08x;", oper, dest);

        as = dis86_print_intel_syntax(d->dis, ins, false);
        str_fmt(ret_s, "  %-50s // %s\n", "", as);
        free((void*)as);

        cs = str_to_cstr(s);
        as = dis86_print_intel_syntax(d->dis, next_ins, false);
        str_fmt(ret_s, "  %-50s // %s\n", cs, as);
        free((void*)as);
        free((void*)cs);

        (*ins_idx)++; // advance one extra
        return;
      }
    }
  }

  if (ins->opcode == OP_JMP) {
    u32 dest = branch_destination(ins);
    str_fmt(s, "goto label_%08x;", dest);

    cs = str_to_cstr(s);
    as = dis86_print_intel_syntax(d->dis, ins, false);
    str_fmt(ret_s, "  %-50s // %s\n", cs, as);
    free((void*)as);
    free((void*)cs);

    return;
  }

  if (ins->opcode == OP_XOR &&
      ins->operand[0].type == OPERAND_TYPE_REG &&
      ins->operand[1].type == OPERAND_TYPE_REG &&
      ins->operand[0].u.reg.id == ins->operand[1].u.reg.id) {

    operand_str(d, s, ins, &ins->operand[0]);
    str_fmt(s, " = 0;");

    cs = str_to_cstr(s);
    as = dis86_print_intel_syntax(d->dis, ins, false);
    str_fmt(ret_s, "  %-50s // %s\n", cs, as);
    free((void*)as);
    free((void*)cs);

    return;
  }

  if (ins->opcode == OP_LDS || ins->opcode == OP_LES) {
    str_fmt(s, "LOAD_SEG_OFF(");
    operand_str(d, s, ins, &ins->operand[0]);
    str_fmt(s, ", ");
    operand_str(d, s, ins, &ins->operand[1]);
    str_fmt(s, ", ");
    operand_str(d, s, ins, &ins->operand[2]);
    str_fmt(s, ");");

    cs = str_to_cstr(s);
    as = dis86_print_intel_syntax(d->dis, ins, false);
    str_fmt(ret_s, "  %-50s // %s\n", cs, as);
    free((void*)as);
    free((void*)cs);

    return;
  }

  if (ins->opcode == OP_CALLF) {
    if (ins->operand[0].type == OPERAND_TYPE_FAR) {
      operand_far_t *far = &ins->operand[0].u.far;
      segoff_t addr = {far->seg, far->off};
      bool remapped = config_seg_remap(d->cfg, &addr.seg);
      const char *name = config_func_lookup(d->cfg, addr);
      if (name) {
        str_fmt(s, "CALL_FUNC(%s);", name);
      } else {
        str_fmt(s, "CALL_FAR(0x%04x, 0x%04x);", addr.seg, addr.off);
      }
      if (remapped) str_fmt(s, " /* remapped */");
    }
    // HAX
    else {
      str_fmt(s, "UNKNOWN_CALL_FAR()");
    }

    cs = str_to_cstr(s);
    as = dis86_print_intel_syntax(d->dis, ins, false);
    str_fmt(ret_s, "  %-50s // %s\n", cs, as);
    free((void*)as);
    free((void*)cs);

    return;
  }

  if (ins->opcode == OP_CALL) {
    assert(ins->operand[0].type == OPERAND_TYPE_REL);
    u16 effective = ins->addr + ins->n_bytes + ins->operand[0].u.rel.val;
    str_fmt(s, "CALL_NEAR(0x%04x);", effective);

    cs = str_to_cstr(s);
    as = dis86_print_intel_syntax(d->dis, ins, false);
    str_fmt(ret_s, "  %-50s // %s\n", cs, as);
    free((void*)as);
    free((void*)cs);

    return;
  }

  if (ins->opcode == OP_LEA) {
    assert(ins->operand[0].type == OPERAND_TYPE_REG);
    operand_reg_t *reg = &ins->operand[0].u.reg;

    assert(ins->operand[1].type == OPERAND_TYPE_MEM);
    operand_mem_t *mem = &ins->operand[1].u.mem;
    assert(mem->sz == SIZE_16);
    assert(mem->reg1);
    assert(!mem->reg2);
    assert(mem->off);

    str_fmt(s, "%s", as_upper(reg_name(reg->id)));
    str_fmt(s, " = %s - 0x%x;", as_upper(reg_name(mem->reg1)), -(i16)mem->off);

    cs = str_to_cstr(s);
    as = dis86_print_intel_syntax(d->dis, ins, false);
    str_fmt(ret_s, "  %-50s // %s\n", cs, as);
    free((void*)as);
    free((void*)cs);

    return;
  }

  if (ins->opcode == OP_IMUL) {
    assert(ins->operand[0].type == OPERAND_TYPE_REG);
    operand_reg_t *reg1 = &ins->operand[0].u.reg;
    assert(ins->operand[1].type == OPERAND_TYPE_REG);
    operand_reg_t *reg2 = &ins->operand[1].u.reg;
    assert(ins->operand[2].type == OPERAND_TYPE_IMM);
    operand_imm_t *imm = &ins->operand[2].u.imm;

    str_fmt(s, "%s = (i16)%s * (i16)0x%x;", as_upper(reg_name(reg1->id)),
            as_upper(reg_name(reg2->id)), imm->val);

    cs = str_to_cstr(s);
    as = dis86_print_intel_syntax(d->dis, ins, false);
    str_fmt(ret_s, "  %-50s // %s\n", cs, as);
    free((void*)as);
    free((void*)cs);

    return;
  }

  /////////////////
  // GENERIC

  int type = code_c_type[ins->opcode];
  const char *str = code_c_str[ins->opcode];

  bool unknown = false;
  switch (type) {
    case CODE_C_UNKNOWN: {
      unknown = true;
      str_fmt(s, "UNKNOWN();");
    } break;
    case CODE_C_OPERATOR: {
      assert(ins->operand[0].type != OPERAND_TYPE_NONE);
      operand_str(d, s, ins, &ins->operand[0]);
      str_fmt(s, " %s ", str);
      if (ins->operand[1].type != OPERAND_TYPE_NONE) {
        operand_str(d, s, ins, &ins->operand[1]);
      }
      str_fmt(s, ";");
    } break;
    case CODE_C_FUNCTION: {
      str_fmt(s, "%s(", str);
      for (size_t i = 0; i < ARRAY_SIZE(ins->operand); i++) {
        operand_t *o = &ins->operand[i];
        if (o->type == OPERAND_TYPE_NONE) break;
        if (i != 0) str_fmt(s, ", ");
        operand_str(d, s, ins, o);
      }
      str_fmt(s, ");", str);
    } break;
    case CODE_C_RFUNCTION: {
      assert(ins->operand[0].type != OPERAND_TYPE_NONE);
      operand_str(d, s, ins, &ins->operand[0]);
      str_fmt(s, " = %s(", str);
      for (size_t i = 1; i < ARRAY_SIZE(ins->operand); i++) {
        operand_t *o = &ins->operand[i];
        if (o->type == OPERAND_TYPE_NONE) break;
        if (i != 1) str_fmt(s, ", ");
        operand_str(d, s, ins, o);
      }
      str_fmt(s, ");", str);
    } break;
    case CODE_C_LITERAL: {
      str_fmt(s, "%s", str);
    } break;
    default:
      FAIL("Unknown code type: %d\n", type);
  }

  cs = str_to_cstr(s);
  as = dis86_print_intel_syntax(d->dis, ins, false);

  if (unknown) {
    fprintf(stderr, "WARN: UNKNOWN C CONVERSION FOR INSTRUCTION '%s'\n", as);
  }

  str_fmt(ret_s, "  %-50s // %s\n", cs, as);
  free((void*)as);
  free((void*)cs);
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
  for (size_t i = 0; i < n_ins; i++) {
    decompiler_process_ins(d, &i, ret_s);
  }
  decompiler_emit_postamble(d, ret_s);
  return str_to_cstr(ret_s);
}
