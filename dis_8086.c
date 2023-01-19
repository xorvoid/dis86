#include "header.h"
#include "reg.h"
#include "oper.h"
#include "bin.h"

#define REP_NONE  0
#define REP_EQ    1
#define REP_NE    2

#define SIZE_FLAG_NONE  0
#define SIZE_FLAG_8     1
#define SIZE_FLAG_16    2
#define SIZE_FLAG_32    3

enum {
  OPERAND_TYPE_NONE,
  OPERAND_TYPE_VAL,
  OPERAND_TYPE_ADDR,
};

enum {
  MODE_BX_PLUS_SI,
  MODE_BX_PLUS_DI,
  MODE_BP_PLUS_SI,
  MODE_BP_PLUS_DI,
  MODE_SI,
  MODE_DI,
  MODE_BP,
  MODE_BX,
};

typedef struct operand operand_t;
struct operand {
  int type;
  int has_mode : 1;
  int has_reg : 1;
  int has_sreg : 1;
  int has_imm : 1;
  int has_rel : 1;
  int has_seg_override : 1;

  int mode;
  u8 reg;
  u8 sreg;
  u8 seg_override;
  u16 imm;
  u16 rel;
};

operand_t operand_reg(u8 reg)
{
  operand_t operand[1] = {{}};
  operand->type = OPERAND_TYPE_VAL;
  operand->has_reg = 1;
  operand->reg = reg;
  return operand[0];
}

operand_t operand_sreg(u8 sreg)
{
  operand_t operand[1] = {{}};
  operand->type = OPERAND_TYPE_VAL;
  operand->has_sreg = 1;
  operand->sreg = sreg;
  return operand[0];
}

operand_t operand_imm(u16 imm)
{
  operand_t operand[1] = {{}};
  operand->type = OPERAND_TYPE_VAL;
  operand->has_imm = 1;
  operand->imm = imm;
  return operand[0];
}

operand_t operand_rel16(u16 rel)
{
  operand_t operand[1] = {{}};
  operand->type = OPERAND_TYPE_VAL;
  operand->has_rel = 1;
  operand->rel = rel;
  return operand[0];
}

operand_t operand_rel8(u8 rel)
{
  operand_t operand[1] = {{}};
  operand->type = OPERAND_TYPE_VAL;
  operand->has_rel = 1;
  operand->rel = (u16)(i16)(i8)rel;
  return operand[0];
}

operand_t operand_addr_imm(u16 imm, int has_seg, u8 seg)
{
  operand_t operand[1] = {{}};
  operand->type = OPERAND_TYPE_ADDR;
  operand->has_imm = 1;
  operand->has_seg_override = has_seg;
  operand->imm = imm;
  operand->seg_override = seg;
  return operand[0];
}

operand_t operand_addr_reg(u8 seg, u8 reg)
{
  operand_t operand[1] = {{}};
  operand->type = OPERAND_TYPE_ADDR;
  operand->has_reg = 1;
  operand->has_seg_override = 1;
  operand->reg = reg;
  operand->seg_override = seg;
  return operand[0];
}

operand_t operand_addr_mode(int mode, int has_seg, u8 seg)
{
  operand_t operand[1] = {{}};
  operand->type = OPERAND_TYPE_ADDR;
  operand->has_mode = 1;
  operand->has_seg_override = 1;
  operand->mode = mode;
  operand->seg_override = seg;
  return operand[0];
}

typedef struct instr instr_t;
struct instr
{
  int       rep;
  int       opcode;     /* operation enum (not 8086 opcode) */
  int       size_flag;  /* SIZE_FLAG_* */
  operand_t operand[2]; /* operands */
};


static inline const char *reg_str(u8 r, u8 size_flag)
{
  if (size_flag == SIZE_FLAG_8) {
    return reg8_str(r);
  } else if (size_flag == SIZE_FLAG_16 || size_flag == SIZE_FLAG_32) {
    return reg16_str(r);
  } else {
    FAIL("Invalid sz flag: %u", size_flag);
  }
}

static inline void instr_print(size_t start_loc, instr_t *ins)
{
  printf("%8zx:\t", start_loc);
  for (size_t i = start_loc; i < bin_location(); i++) {
    u8 b = bin_byte_at(i);
    printf("%02x ", b);
  }
  size_t used = (bin_location() - start_loc) * 3;
  size_t remain = (used <= 21) ? 21 - used : 0;
  printf("%*s\t", (int)remain, " ");

  if (ins->rep == REP_EQ) {
    printf("rep ");
  } else if (ins->rep == REP_NE) {
    printf("repnz ");
  }

  printf("%-6s", opcode_str(ins->opcode));
  for (size_t i = 0; i < ARRAY_SIZE(ins->operand); i++) {
    operand_t *o = &ins->operand[i];
    if (o->type == OPERAND_TYPE_NONE) {
      break;
    }

    if (i == 0) {
      printf(" ");
    } else {
      printf(",");
    }

    if (o->type == OPERAND_TYPE_VAL) {
      if (o->has_reg) {
        printf("%s", reg_str(o->reg, ins->size_flag));
      }
      if (o->has_sreg) {
        printf("%s", sreg_str(o->sreg));
      }
      if (o->has_imm) {
        printf("0x%x", o->imm);
      }
      if (o->has_rel) {
        u16 loc = bin_location() + o->rel;
        printf("0x%x", loc);
      }
    }

    if (o->type == OPERAND_TYPE_ADDR) {
      if (ins->size_flag == SIZE_FLAG_8) {
        printf("BYTE PTR ");
      } else if (ins->size_flag == SIZE_FLAG_16) {
        printf("WORD PTR ");
      } else if (ins->size_flag == SIZE_FLAG_32) {
        printf("DWORD PTR ");
      } else {
        FAIL("Expected size flag to be set");
      }
      if (o->has_seg_override) {
        printf("%s:", sreg_str(o->seg_override));
      } else {
        printf("ds:");
      }
      if (o->has_mode) {
        switch (o->mode) {
          case MODE_BX_PLUS_SI: printf("[bx+si]"); break;
          case MODE_BX_PLUS_DI: printf("[bx+di]"); break;
          case MODE_BP_PLUS_SI: printf("[bp+si]"); break;
          case MODE_BP_PLUS_DI: printf("[bp+di]"); break;
          case MODE_SI:         printf("[si]"); break;
          case MODE_DI:         printf("[di]"); break;
          case MODE_BP:         printf("[bp]"); break;
          case MODE_BX:         printf("[bx]"); break;
        }
      }
      if (o->has_reg) {
        printf("[%s]", reg_str(o->reg, SIZE_FLAG_16));
      }
      if (o->has_imm) {
        printf("0x%x", o->imm);
      }
    }
  }
  printf("\n");
}

static void modrm_process(operand_t *operand1, operand_t *operand2, int has_seg_override, u8 seg_override)
{
  u8 modrm = bin_fetch_u8();
  u8 mod = modrm >> 6;
  u8 reg = (modrm >> 3) & 7;
  u8 rm = modrm & 7;

  if (mod == 0) {
    if (rm == 6) {
      /* Direct addressing: 16-bit address offset */
      u16 imm = bin_fetch_u16();
      *operand1 = operand_addr_imm(imm, has_seg_override, seg_override);
      *operand2 = operand_reg(reg);
    }
    else {
      /* Ordinary special mode */
      *operand1 = operand_addr_mode(rm, has_seg_override, seg_override);
      *operand2 = operand_reg(reg);
    }
  }

  /* Two register mode */
  else if(mod == 3) {
    *operand1 = operand_reg(rm);
    *operand2 = operand_reg(reg);
  }

  else {
    FAIL("Unsupported MOD/RM mode | mod=%u, rm=%u", mod, rm);
  }
}

static u8 binary_op[]  = { OP_ADD,  OP_OR,  OP_ADC, OP_SBB, OP_AND, OP_SUB,  OP_XOR, OP_CMP  };
static u8 unary_op[]   = { 0,       0,      OP_NOT, OP_NEG, OP_MUL, OP_IMUL, OP_DIV, OP_IDIV };
static u8 inc_dec_op[] = { OP_INC,  OP_DEC, 0,      0,      0,      0,      0,      0       };

static u8 arith_op(u8 *ops_tbl, operand_t *operand, int has_seg_override, u8 seg_override)
{
  u8 modrm = bin_fetch_u8();
  u8 mod = modrm >> 6;
  u8 opnum = (modrm >> 3) & 7;
  u8 rm = modrm & 7;

  u8 op = ops_tbl[opnum];
  if (!op) FAIL("Invalid instruction op encoding");

  // FIXME
  assert(mod == 3);

  *operand = operand_reg(rm);
  return op;
}

static void instr_fetch(instr_t *ins)
{
  memset(ins, 0, sizeof(*ins));

  u8 b = bin_fetch_u8();

  /* handle any prefixes first */
  int has_seg_override;
  u8 seg_override;
  if (b == 0x2e) {
    has_seg_override = 1;
    seg_override = SREG_CS;
  } else if (b == 0x3e) {
    has_seg_override = 1;
    seg_override = SREG_DS;
  } else if (b == 0x26) {
    has_seg_override = 1;
    seg_override = SREG_ES;
  } else if (b == 0x36) {
    has_seg_override = 1;
    seg_override = SREG_SS;
  } else {
    has_seg_override = 0;
  }

  /* advance byte? */
  if (has_seg_override) {
    b = bin_fetch_u8();
  }

  /* handle rep and repne */
  ins->rep = REP_NONE;
  if (b == 0xf3) {
    ins->rep = REP_EQ;
  } else if (b == 0xf2) {
    ins->rep = REP_NE;
  }

  /* advance byte? */
  if (ins->rep) {
    b = bin_fetch_u8();
  }

  /* parse opcode now */
  u8 op = b;
  if (0) {
  } else if (op == 0x38) {
    /* CMP R/M REG8 */
    ins->opcode = OP_CMP;
    ins->size_flag = SIZE_FLAG_8;
    modrm_process(&ins->operand[0], &ins->operand[1], has_seg_override, seg_override);

    /* // HAX: WEIRD STUFF HAPPENING HERE WITH PREFIX BYTES AND OPCODE COLLISIONS */
    /* u8 _ = bin_fetch_u8(); */

  } else if (0x40 <= op && op <= 0x47) {
    /* INC REG16 */
    u8 reg = op - 0x40;

    ins->opcode = OP_INC;
    ins->size_flag = SIZE_FLAG_16;
    ins->operand[0] = operand_reg(reg);

  } else if (op == 0x75) {
    /* JNE REL8 */
    u8 rel = bin_fetch_u8();

    ins->opcode = OP_JNE;
    ins->operand[0] = operand_rel8(rel);

  } else if (op == 0x80) {
    /* BINARY_OP R/M8 IMM8 */
    ins->opcode = arith_op(binary_op, &ins->operand[0], has_seg_override, seg_override);
    ins->size_flag = SIZE_FLAG_8;
    ins->operand[1] = operand_imm(bin_fetch_u8());

  } else if (op == 0x81) {
    /* BINARY_OP R/M16 IMM16 */
    ins->opcode = arith_op(binary_op, &ins->operand[0], has_seg_override, seg_override);
    ins->size_flag = SIZE_FLAG_16;
    ins->operand[1] = operand_imm(bin_fetch_u16());

  } else if (op == 0x83) {
    /* BINARY_OP R/M16 IMM8 */
    ins->opcode = arith_op(binary_op, &ins->operand[0], has_seg_override, seg_override);
    ins->size_flag = SIZE_FLAG_16;
    ins->operand[1] = operand_imm((i16)(i8)bin_fetch_u8());

  } else if ((op & ~3) == 0x88) { /* all sizes and directions */
    /* MOV REG, R/M */
    u8 w = (op&1);
    u8 d = ((op>>1)&1);

    ins->opcode = OP_MOV;
    ins->size_flag = w ? SIZE_FLAG_16 : SIZE_FLAG_8;
    modrm_process(&ins->operand[d], &ins->operand[1-d], has_seg_override, seg_override);

  } else if ((op & ~3) == 0x8c) { /* only one size, all directions */
    /* MOV REG, SREG */
    u8 w = (op&1);
    u8 d = ((op>>1)&1);
    assert(w == 0);

    ins->opcode = OP_MOV;
    ins->size_flag = SIZE_FLAG_16;
    /* ins->operand[d] = operand_reg(rm); */
    /* ins->operand[1-d] = operand_sreg(reg); */
    modrm_process(&ins->operand[d], &ins->operand[1-d], has_seg_override, seg_override);

    /* SREG, not REG (sort of a hack) */
    ins->operand[1-d].sreg = ins->operand[1-d].reg;
    ins->operand[1-d].has_reg = 0;
    ins->operand[1-d].has_sreg = 1;

  } else if ((op & ~3) == 0xa0) { /* all sizes, all directions */
    /* MOV MEM16, AX  or  MOV MEM8, AL */
    u8 w = (op&1);
    u8 d = ((op>>1)&1);

    u16 imm = w ? bin_fetch_u16() : bin_fetch_u8();

    ins->opcode = OP_MOV;
    ins->size_flag = w ? SIZE_FLAG_16 : SIZE_FLAG_8;
    ins->operand[d] = operand_reg(0); /* AX or AL implied by opcode */
    ins->operand[1-d] = operand_addr_imm(imm, has_seg_override, seg_override);

  } else if ((op & ~1) == 0xae) { /* all sizes, one direction */
    /* SCAS AX, WORD PTR ES:[DI] (implied) */
    u8 w = (op&1);

    ins->opcode = OP_SCAS;
    ins->size_flag = w ? SIZE_FLAG_16 : SIZE_FLAG_8;
    ins->operand[0] = operand_reg(0); /* AX or AL implied by opcode */
    ins->operand[1] = operand_addr_reg(SREG_ES, REG16_DI); /* Always ES:[DI] */

  } else if (op == 0xcd) {
    /* INT IMM8 */
    u8 imm = bin_fetch_u8();
    ins->opcode = OP_INT;
    ins->size_flag = SIZE_FLAG_8;
    ins->operand[0] = operand_imm(imm);

  } else if (0xb0 <= op && op <= 0xb7) {
    /* MOV REG8 IMM8 */
    u8 reg = op - 0xb0;
    u8 imm = bin_fetch_u8();

    ins->opcode = OP_MOV;
    ins->size_flag = SIZE_FLAG_8;
    ins->operand[0] = operand_reg(reg);
    ins->operand[1] = operand_imm(imm);

  } else if (0xb8 <= op && op <= 0xbf) {
    /* MOV REG16 IMM16 */
    u8 reg = op - 0xb8;
    u16 imm = bin_fetch_u16();

    ins->opcode = OP_MOV;
    ins->size_flag = SIZE_FLAG_16;
    ins->operand[0] = operand_reg(reg);
    ins->operand[1] = operand_imm(imm);

  } else if (op == 0xc4) {
    // LES REG MEM32

    /* u8 b = bin_fetch_u8(); */
    /* printf("b: %x (%u)\n", b, b); */

    ins->opcode = OP_LES;
    ins->size_flag = SIZE_FLAG_32;
    modrm_process(&ins->operand[1], &ins->operand[0], has_seg_override, seg_override);

  /* } else if (op == 0xc5) { */
  /*   // LDS REG MEM32 */

  } else if (op == 0xe3) {
    // JCXZ REL8
    u8 imm = bin_fetch_u8();
    ins->opcode = OP_JCXZ;
    ins->operand[0] = operand_rel8(imm);

  } else if (op == 0xe8) {
    // CALL REL16
    u16 imm = bin_fetch_u16();
    ins->opcode = OP_CALL;
    ins->operand[0] = operand_rel16(imm);

  } else if (op == 0xd3) {
    // SKIP FIXME HAX
    // SHL instruction
    u8 _ = bin_fetch_u8();

  } else if (op == 0xf7) {
    // UNARY_OP R/M16
    ins->opcode = arith_op(unary_op, &ins->operand[0], has_seg_override, seg_override);
    ins->size_flag = SIZE_FLAG_16;

  } else if (op == 0xfc) {
    // CLD
    ins->opcode = OP_CLD;

  } else {
    FAIL("Unknown opcode: %x", op);
  }
}

int main(int argc, char *argv[])
{
  if (argc != 2) {
    fprintf(stderr, "usage: %s <binary>\n", argv[0]);
    return 1;
  }
  const char *filename = argv[1];

  bin_init(filename);

  instr_t ins[1];
  //for (size_t i = 0; i < 10; i++) {
  while (1) {
    size_t start_loc = bin_location();
    instr_fetch(ins);
    size_t end_loc = bin_location();
    instr_print(start_loc, ins);
  }

  return 0;
}
