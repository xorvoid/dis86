
#define MAX_LABELS 256

typedef struct labels labels_t;
struct labels
{
  u32 addr[MAX_LABELS];
  size_t n_addr;
};

// FIXME: O(n) search
static bool is_label(labels_t *labels, u32 addr)
{
  for (size_t i = 0; i < labels->n_addr; i++) {
    if (labels->addr[i] == addr) return true;
  }
  return false;
}

static u32 branch_destination(dis86_instr_t *ins)
{
  i16 rel = 0;
  switch (ins->opcode) {
    case OP_JO:  rel = (i16)ins->operand[0].u.rel.val; break;
    case OP_JNO: rel = (i16)ins->operand[0].u.rel.val; break;
    case OP_JB:  rel = (i16)ins->operand[0].u.rel.val; break;
    case OP_JAE: rel = (i16)ins->operand[0].u.rel.val; break;
    case OP_JE:  rel = (i16)ins->operand[0].u.rel.val; break;
    case OP_JNE: rel = (i16)ins->operand[0].u.rel.val; break;
    case OP_JBE: rel = (i16)ins->operand[0].u.rel.val; break;
    case OP_JA:  rel = (i16)ins->operand[0].u.rel.val; break;
    case OP_JS:  rel = (i16)ins->operand[0].u.rel.val; break;
    case OP_JNS: rel = (i16)ins->operand[0].u.rel.val; break;
    case OP_JP:  rel = (i16)ins->operand[0].u.rel.val; break;
    case OP_JNP: rel = (i16)ins->operand[0].u.rel.val; break;
    case OP_JL:  rel = (i16)ins->operand[0].u.rel.val; break;
    case OP_JGE: rel = (i16)ins->operand[0].u.rel.val; break;
    case OP_JLE: rel = (i16)ins->operand[0].u.rel.val; break;
    case OP_JG:  rel = (i16)ins->operand[0].u.rel.val; break;
    case OP_JMP: rel = (i16)ins->operand[0].u.rel.val; break;
    case OP_LOOP:rel = (i16)ins->operand[1].u.rel.val; break;
    default: return 0;
  }

  u16 effective = ins->addr + ins->n_bytes + rel;
  return effective;
}

static void find_labels(labels_t *labels, dis86_instr_t *ins_arr, size_t n_ins)
{
  labels->n_addr = 0;

  for (size_t i = 0; i < n_ins; i++) {
    dis86_instr_t *ins = &ins_arr[i];
    u16 dst = branch_destination(ins);
    if (!dst) continue;

    assert(labels->n_addr < ARRAY_SIZE(labels->addr));
    labels->addr[labels->n_addr++] = dst;
  }
}
