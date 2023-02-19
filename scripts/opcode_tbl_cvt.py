
TBL = '''\
OP_AAA,     "aaa"     ) \
  _(  OP_AAS,     "aas"     ) \
  _(  OP_ADC,     "adc"     ) \
  _(  OP_ADD,     "add"     ) \
  _(  OP_AND,     "and"     ) \
  _(  OP_CALL,    "call"    ) \
  _(  OP_CALLF,   "callf"   ) \
  _(  OP_CBW,     "cbw"     ) \
  _(  OP_CLC,     "clc"     ) \
  _(  OP_CLD,     "cld"     ) \
  _(  OP_CLI,     "cli"     ) \
  _(  OP_CMC,     "cmc"     ) \
  _(  OP_CMP,     "cmp"     ) \
  _(  OP_CMPS,    "cmps"    ) \
  _(  OP_CWD,     "cwd"     ) \
  _(  OP_DAA,     "daa"     ) \
  _(  OP_DAS,     "das"     ) \
  _(  OP_DEC,     "dec"     ) \
  _(  OP_DIV,     "div"     ) \
  _(  OP_ENTER,   "enter"   ) \
  _(  OP_HLT,     "hlt"     ) \
  _(  OP_IMUL,    "imul"    ) \
  _(  OP_IN,      "in"      ) \
  _(  OP_INC,     "inc"     ) \
  _(  OP_INS,     "ins"     ) \
  _(  OP_INT,     "int"     ) \
  _(  OP_INTO,    "into"    ) \
  _(  OP_INVAL,   "inval"   ) \
  _(  OP_IRET,    "iret"    ) \
  _(  OP_JA,      "ja"      ) \
  _(  OP_JAE,     "jae"     ) \
  _(  OP_JB,      "jb"      ) \
  _(  OP_JBE,     "jbe"     ) \
  _(  OP_JCXZ,    "jcxz"    ) \
  _(  OP_JE,      "je"      ) \
  _(  OP_JG,      "jg"      ) \
  _(  OP_JGE,     "jge"     ) \
  _(  OP_JL,      "jl"      ) \
  _(  OP_JLE,     "jle"     ) \
  _(  OP_JMP,     "jmp"     ) \
  _(  OP_JMPF,    "jmpf"    ) \
  _(  OP_JNE,     "jne"     ) \
  _(  OP_JNO,     "jno"     ) \
  _(  OP_JNP,     "jnp"     ) \
  _(  OP_JNS,     "jns"     ) \
  _(  OP_JO,      "jo"      ) \
  _(  OP_JP,      "jp"      ) \
  _(  OP_JS,      "js"      ) \
  _(  OP_LAHF,    "lahf"    ) \
  _(  OP_LDS,     "lds"     ) \
  _(  OP_LEA,     "lea"     ) \
  _(  OP_LEAVE,   "leave"   ) \
  _(  OP_LES,     "les"     ) \
  _(  OP_LODS,    "lods"    ) \
  _(  OP_LOOP,    "loop"    ) \
  _(  OP_LOOPE,   "loope"   ) \
  _(  OP_LOOPNE,  "loopne"  ) \
  _(  OP_MOV,     "mov"     ) \
  _(  OP_MOVS,    "movs"    ) \
  _(  OP_MUL,     "mul"     ) \
  _(  OP_NEG,     "neg"     ) \
  _(  OP_NOP,     "nop"     ) \
  _(  OP_NOT,     "not"     ) \
  _(  OP_OR,      "or"      ) \
  _(  OP_OUT,     "out"     ) \
  _(  OP_OUTS,    "outs"    ) \
  _(  OP_POP,     "pop"     ) \
  _(  OP_POPA,    "popa"    ) \
  _(  OP_POPF,    "popf"    ) \
  _(  OP_PUSH,    "push"    ) \
  _(  OP_PUSHA,   "pusha"   ) \
  _(  OP_PUSHF,   "pushf"   ) \
  _(  OP_RCL,     "rcl"     ) \
  _(  OP_RCR,     "rcr"     ) \
  _(  OP_RET,     "ret"     ) \
  _(  OP_RETF,    "retf"    ) \
  _(  OP_ROL,     "rol"     ) \
  _(  OP_ROR,     "ror"     ) \
  _(  OP_SAHF,    "sahf"    ) \
  _(  OP_SAR,     "sar"     ) \
  _(  OP_SBB,     "sbb"     ) \
  _(  OP_SCAS,    "scas"    ) \
  _(  OP_SHL,     "shl"     ) \
  _(  OP_SHR,     "shr"     ) \
  _(  OP_STC,     "stc"     ) \
  _(  OP_STD,     "std"     ) \
  _(  OP_STI,     "sti"     ) \
  _(  OP_STOS,    "stos"    ) \
  _(  OP_SUB,     "sub"     ) \
  _(  OP_TEST,    "test"    ) \
  _(  OP_XCHG,    "xchg"    ) \
  _(  OP_XLAT,    "xlat"    ) \
  _(  OP_XOR,     "xor"
'''.rstrip()

def fmt_row(fmts, elts):
    assert(len(fmts) == len(elts))
    s = ''
    for f,e in zip(fmts[:-1], elts[:-1]):
        s += f % (e+',')
    s += fmts[-1] % elts[-1]
    return s

fmts = ['%-12s', '%-12s', '%-20s', '%-7s']

rows = [x.strip() for x in TBL.split(')   _(')]
for r in rows:
    enum, name = [x.strip() for x in r.split(',')]
    r = [enum, name, 'CODE_C_UNKNOWN', '""']
    print('  _(  %s  )\\' % fmt_row(fmts, r))
