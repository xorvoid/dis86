
TBL = [
  "(  OP_ADD,   0x00,    -1,       OPER_RM8,      OPER_R8     ,  -1          )",
  "(  OP_ADD,   0x01,    -1,       OPER_RM16,     OPER_R16    ,  -1          )",
  "(  OP_ADD,   0x02,    -1,       OPER_R8,       OPER_RM8    ,  -1          )",
  "(  OP_ADD,   0x03,    -1,       OPER_R16,      OPER_RM16   ,  -1          )",
  "(  OP_ADD,   0x04,    -1,       OPER_AL,       OPER_IMM8   ,  -1          )",
  "(  OP_ADD,   0x05,    -1,       OPER_AX,       OPER_IMM16  ,  -1          )",
  "(  OP_PUSH,  0x06,    -1,       OPER_ES,       -1          ,  -1          )",
  "(  OP_POP,   0x07,    -1,       OPER_ES,       -1          ,  -1          )",
  "(  OP_OR,    0x08,    -1,       OPER_RM8,      OPER_R8     ,  -1          )",
  "(  OP_OR,    0x09,    -1,       OPER_RM16,     OPER_R16    ,  -1          )",
  "(  OP_OR,    0x0a,    -1,       OPER_R8,       OPER_RM8    ,  -1          )",
  "(  OP_OR,    0x0b,    -1,       OPER_R16,      OPER_RM16   ,  -1          )",
  "(  OP_OR,    0x0c,    -1,       OPER_AL,       OPER_IMM8   ,  -1          )",
  "(  OP_OR,    0x0d,    -1,       OPER_AX,       OPER_IMM16  ,  -1          )",
  "(  OP_PUSH,  0x0e,    -1,       OPER_CS,       -1          ,  -1          )",
  "(  OP_INVAL, 0x0f,    -1,       -1,            -1          ,  -1          )",
  "(  OP_ADC,   0x10,    -1,       OPER_RM8,      OPER_R8     ,  -1          )",
  "(  OP_ADC,   0x11,    -1,       OPER_RM16,     OPER_R16    ,  -1          )",
  "(  OP_ADC,   0x12,    -1,       OPER_R8,       OPER_RM8    ,  -1          )",
  "(  OP_ADC,   0x13,    -1,       OPER_R16,      OPER_RM16   ,  -1          )",
  "(  OP_ADC,   0x14,    -1,       OPER_AL,       OPER_IMM8   ,  -1          )",
  "(  OP_ADC,   0x15,    -1,       OPER_AX,       OPER_IMM16  ,  -1          )",
  "(  OP_PUSH,  0x16,    -1,       OPER_SS,       -1          ,  -1          )",
  "(  OP_POP,   0x17,    -1,       OPER_SS,       -1          ,  -1          )",
  "(  OP_SBB,   0x18,    -1,       OPER_RM8,      OPER_R8     ,  -1          )",
  "(  OP_SBB,   0x19,    -1,       OPER_RM16,     OPER_R16    ,  -1          )",
  "(  OP_SBB,   0x1a,    -1,       OPER_R8,       OPER_RM8    ,  -1          )",
  "(  OP_SBB,   0x1b,    -1,       OPER_R16,      OPER_RM16   ,  -1          )",
  "(  OP_SBB,   0x1c,    -1,       OPER_AL,       OPER_IMM8   ,  -1          )",
  "(  OP_SBB,   0x1d,    -1,       OPER_AX,       OPER_IMM16  ,  -1          )",
  "(  OP_PUSH,  0x1e,    -1,       OPER_DS,       -1          ,  -1          )",
  "(  OP_POP,   0x1f,    -1,       OPER_DS,       -1          ,  -1          )",
  "(  OP_AND,   0x20,    -1,       OPER_RM8,      OPER_R8     ,  -1          )",
  "(  OP_AND,   0x21,    -1,       OPER_RM16,     OPER_R16    ,  -1          )",
  "(  OP_AND,   0x22,    -1,       OPER_R8,       OPER_RM8    ,  -1          )",
  "(  OP_AND,   0x23,    -1,       OPER_R16,      OPER_RM16   ,  -1          )",
  "(  OP_AND,   0x24,    -1,       OPER_AL,       OPER_IMM8   ,  -1          )",
  "(  OP_AND,   0x25,    -1,       OPER_AX,       OPER_IMM16  ,  -1          )",
  "(  OP_INVAL, 0x26,    -1,       -1,            -1          ,  -1          )", #/* ES OVERRIDE */
  "(  OP_DAA,   0x27,    -1,       OPER_AL,       -1          ,  -1          )",
  "(  OP_SUB,   0x28,    -1,       OPER_RM8,      OPER_R8     ,  -1          )",
  "(  OP_SUB,   0x29,    -1,       OPER_RM16,     OPER_R16    ,  -1          )",
  "(  OP_SUB,   0x2a,    -1,       OPER_R8,       OPER_RM8    ,  -1          )",
  "(  OP_SUB,   0x2b,    -1,       OPER_R16,      OPER_RM16   ,  -1          )",
  "(  OP_SUB,   0x2c,    -1,       OPER_AL,       OPER_IMM8   ,  -1          )",
  "(  OP_SUB,   0x2d,    -1,       OPER_AX,       OPER_IMM16  ,  -1          )",
  "(  OP_INVAL, 0x2e,    -1,       -1,            -1          ,  -1          )", #/* CS OVERRIDE */
  "(  OP_DAS,   0x2f,    -1,       OPER_AL,       -1          ,  -1          )",
  "(  OP_XOR,   0x30,    -1,       OPER_RM8,      OPER_R8     ,  -1          )",
  "(  OP_XOR,   0x31,    -1,       OPER_RM16,     OPER_R16    ,  -1          )",
  "(  OP_XOR,   0x32,    -1,       OPER_R8,       OPER_RM8    ,  -1          )",
  "(  OP_XOR,   0x33,    -1,       OPER_R16,      OPER_RM16   ,  -1          )",
  "(  OP_XOR,   0x34,    -1,       OPER_AL,       OPER_IMM8   ,  -1          )",
  "(  OP_XOR,   0x35,    -1,       OPER_AX,       OPER_IMM16  ,  -1          )",
  "(  OP_INVAL, 0x36,    -1,       -1,            -1          ,  -1          )", #/* SS OVERRIDE */
  "(  OP_AAA,   0x37,    -1,       OPER_AL,       OPER_AH     ,  -1          )",
  "(  OP_CMP,   0x38,    -1,       OPER_RM8,      OPER_R8     ,  -1          )",
  "(  OP_CMP,   0x39,    -1,       OPER_RM16,     OPER_R16    ,  -1          )",
  "(  OP_CMP,   0x3a,    -1,       OPER_R8,       OPER_RM8    ,  -1          )",
  "(  OP_CMP,   0x3b,    -1,       OPER_R16,      OPER_RM16   ,  -1          )",
  "(  OP_CMP,   0x3c,    -1,       OPER_AL,       OPER_IMM8   ,  -1          )",
  "(  OP_CMP,   0x3d,    -1,       OPER_AX,       OPER_IMM16  ,  -1          )",
  "(  OP_INVAL, 0x3e,    -1,       -1,            -1          ,  -1          )", #/* DS OVERRIDE */
  "(  OP_AAS,   0x3f,    -1,       OPER_AL,       OPER_AH     ,  -1          )",
  "(  OP_INC,   0x40,    -1,       OPER_AX,       -1          ,  -1          )",
  "(  OP_INC,   0x41,    -1,       OPER_CX,       -1          ,  -1          )",
  "(  OP_INC,   0x42,    -1,       OPER_DX,       -1          ,  -1          )",
  "(  OP_INC,   0x43,    -1,       OPER_BX,       -1          ,  -1          )",
  "(  OP_INC,   0x44,    -1,       OPER_SP,       -1          ,  -1          )",
  "(  OP_INC,   0x45,    -1,       OPER_BP,       -1          ,  -1          )",
  "(  OP_INC,   0x46,    -1,       OPER_SI,       -1          ,  -1          )",
  "(  OP_INC,   0x47,    -1,       OPER_DI,       -1          ,  -1          )",
  "(  OP_DEC,   0x48,    -1,       OPER_AX,       -1          ,  -1          )",
  "(  OP_DEC,   0x49,    -1,       OPER_CX,       -1          ,  -1          )",
  "(  OP_DEC,   0x4a,    -1,       OPER_DX,       -1          ,  -1          )",
  "(  OP_DEC,   0x4b,    -1,       OPER_BX,       -1          ,  -1          )",
  "(  OP_DEC,   0x4c,    -1,       OPER_SP,       -1          ,  -1          )",
  "(  OP_DEC,   0x4d,    -1,       OPER_BP,       -1          ,  -1          )",
  "(  OP_DEC,   0x4e,    -1,       OPER_SI,       -1          ,  -1          )",
  "(  OP_DEC,   0x4f,    -1,       OPER_DI,       -1          ,  -1          )",
  "(  OP_PUSH,  0x50,    -1,       OPER_AX,       -1          ,  -1          )",
  "(  OP_PUSH,  0x51,    -1,       OPER_CX,       -1          ,  -1          )",
  "(  OP_PUSH,  0x52,    -1,       OPER_DX,       -1          ,  -1          )",
  "(  OP_PUSH,  0x53,    -1,       OPER_BX,       -1          ,  -1          )",
  "(  OP_PUSH,  0x54,    -1,       OPER_SP,       -1          ,  -1          )",
  "(  OP_PUSH,  0x55,    -1,       OPER_BP,       -1          ,  -1          )",
  "(  OP_PUSH,  0x56,    -1,       OPER_SI,       -1          ,  -1          )",
  "(  OP_PUSH,  0x57,    -1,       OPER_DI,       -1          ,  -1          )",
  "(  OP_POP,   0x58,    -1,       OPER_AX,       -1          ,  -1          )",
  "(  OP_POP,   0x59,    -1,       OPER_CX,       -1          ,  -1          )",
  "(  OP_POP,   0x5a,    -1,       OPER_DX,       -1          ,  -1          )",
  "(  OP_POP,   0x5b,    -1,       OPER_BX,       -1          ,  -1          )",
  "(  OP_POP,   0x5c,    -1,       OPER_SP,       -1          ,  -1          )",
  "(  OP_POP,   0x5d,    -1,       OPER_BP,       -1          ,  -1          )",
  "(  OP_POP,   0x5e,    -1,       OPER_SI,       -1          ,  -1          )",
  "(  OP_POP,   0x5f,    -1,       OPER_DI,       -1          ,  -1          )",
  "(  OP_PUSHA, 0x60,    -1,       -1,            -1          ,  -1          )",
  "(  OP_POPA,  0x61,    -1,       -1,            -1          ,  -1          )",
  "(  OP_INVAL, 0x62,    -1,       -1,            -1          ,  -1          )",
  "(  OP_INVAL, 0x63,    -1,       -1,            -1          ,  -1          )",
  "(  OP_INVAL, 0x64,    -1,       -1,            -1          ,  -1          )",
  "(  OP_INVAL, 0x65,    -1,       -1,            -1          ,  -1          )",
  "(  OP_INVAL, 0x66,    -1,       -1,            -1          ,  -1          )",
  "(  OP_INVAL, 0x67,    -1,       -1,            -1          ,  -1          )",
  "(  OP_PUSH,  0x68,    -1,       OPER_IMM16,    -1          ,  -1          )",
  #// FIXME: IMUL
  "(  OP_INVAL, 0x69,    -1,       -1,            -1          ,  -1          )",
  "(  OP_PUSH,  0x6a,    -1,       OPER_IMM8,     -1          ,  -1          )",
  #// FIXME: IMUL
  "(  OP_INVAL, 0x6b,    -1,       -1,            -1          ,  -1          )",
  "(  OP_INS,   0x6c,    -1,       OPER_M8,       OPER_DX     ,  -1          )",
  "(  OP_INS,   0x6d,    -1,       OPER_M16,      OPER_DX     ,  -1          )",
  "(  OP_OUTS,  0x6e,    -1,       OPER_DX,       OPER_M8     ,  -1          )",
  "(  OP_OUTS,  0x6f,    -1,       OPER_DX,       OPER_M16    ,  -1          )",
  "(  OP_JO,    0x70,    -1,       -1,            -1          ,  -1          )",
  "(  OP_JNO,   0x71,    -1,       -1,            -1          ,  -1          )",
  "(  OP_JB,    0x72,    -1,       -1,            -1          ,  -1          )",
  "(  OP_JAE,   0x73,    -1,       -1,            -1          ,  -1          )",
  "(  OP_JE,    0x74,    -1,       -1,            -1          ,  -1          )",
  "(  OP_JNE,   0x75,    -1,       -1,            -1          ,  -1          )",
  "(  OP_JBE,   0x76,    -1,       -1,            -1          ,  -1          )",
  "(  OP_JA,    0x77,    -1,       -1,            -1          ,  -1          )",
  "(  OP_JS,    0x78,    -1,       -1,            -1          ,  -1          )",
  "(  OP_JNS,   0x79,    -1,       -1,            -1          ,  -1          )",
  "(  OP_JP,    0x7a,    -1,       -1,            -1          ,  -1          )",
  "(  OP_JNP,   0x7b,    -1,       -1,            -1          ,  -1          )",
  "(  OP_JL,    0x7c,    -1,       -1,            -1          ,  -1          )",
  "(  OP_JGE,   0x7d,    -1,       -1,            -1          ,  -1          )",
  "(  OP_JLE,   0x7e,    -1,       -1,            -1          ,  -1          )",
  "(  OP_JG,    0x7f,    -1,       -1,            -1          ,  -1          )",
  "(  OP_ADD,   0x80,     0,       OPER_RM8,      OPER_IMM8   ,  -1          )",
  "(  OP_OR,    0x80,     1,       OPER_RM8,      OPER_IMM8   ,  -1          )",
  "(  OP_ADC,   0x80,     2,       OPER_RM8,      OPER_IMM8   ,  -1          )",
  "(  OP_SBB,   0x80,     3,       OPER_RM8,      OPER_IMM8   ,  -1          )",
  "(  OP_AND,   0x80,     4,       OPER_RM8,      OPER_IMM8   ,  -1          )",
  "(  OP_SUB,   0x80,     5,       OPER_RM8,      OPER_IMM8   ,  -1          )",
  "(  OP_XOR,   0x80,     6,       OPER_RM8,      OPER_IMM8   ,  -1          )",
  "(  OP_CMP,   0x80,     7,       OPER_RM8,      OPER_IMM8   ,  -1          )",
  "(  OP_ADD,   0x81,     0,       OPER_RM16,     OPER_IMM16  ,  -1          )",
  "(  OP_OR,    0x81,     1,       OPER_RM16,     OPER_IMM16  ,  -1          )",
  "(  OP_ADC,   0x81,     2,       OPER_RM16,     OPER_IMM16  ,  -1          )",
  "(  OP_SBB,   0x81,     3,       OPER_RM16,     OPER_IMM16  ,  -1          )",
  "(  OP_AND,   0x81,     4,       OPER_RM16,     OPER_IMM16  ,  -1          )",
  "(  OP_SUB,   0x81,     5,       OPER_RM16,     OPER_IMM16  ,  -1          )",
  "(  OP_XOR,   0x81,     6,       OPER_RM16,     OPER_IMM16  ,  -1          )",
  "(  OP_CMP,   0x81,     7,       OPER_RM16,     OPER_IMM16  ,  -1          )",
  "(  OP_ADD,   0x82,     0,       OPER_RM8,      OPER_IMM8   ,  -1          )",
  "(  OP_OR,    0x82,     1,       OPER_RM8,      OPER_IMM8   ,  -1          )",
  "(  OP_ADC,   0x82,     2,       OPER_RM8,      OPER_IMM8   ,  -1          )",
  "(  OP_SBB,   0x82,     3,       OPER_RM8,      OPER_IMM8   ,  -1          )",
  "(  OP_AND,   0x82,     4,       OPER_RM8,      OPER_IMM8   ,  -1          )",
  "(  OP_SUB,   0x82,     5,       OPER_RM8,      OPER_IMM8   ,  -1          )",
  "(  OP_XOR,   0x82,     6,       OPER_RM8,      OPER_IMM8   ,  -1          )",
  "(  OP_CMP,   0x82,     7,       OPER_RM8,      OPER_IMM8   ,  -1          )",
  "(  OP_ADD,   0x83,     0,       OPER_RM16,     OPER_IMM8   ,  -1          )",
  "(  OP_OR,    0x83,     1,       OPER_RM16,     OPER_IMM8   ,  -1          )",
  "(  OP_ADC,   0x83,     2,       OPER_RM16,     OPER_IMM8   ,  -1          )",
  "(  OP_SBB,   0x83,     3,       OPER_RM16,     OPER_IMM8   ,  -1          )",
  "(  OP_AND,   0x83,     4,       OPER_RM16,     OPER_IMM8   ,  -1          )",
  "(  OP_SUB,   0x83,     5,       OPER_RM16,     OPER_IMM8   ,  -1          )",
  "(  OP_XOR,   0x83,     6,       OPER_RM16,     OPER_IMM8   ,  -1          )",
  "(  OP_CMP,   0x83,     7,       OPER_RM16,     OPER_IMM8   ,  -1          )",
  "(  OP_TEST,  0x84,    -1,       OPER_RM8,      OPER_R8     ,  -1          )",
  "(  OP_TEST,  0x85,    -1,       OPER_RM16,     OPER_R16    ,  -1          )",
  "(  OP_XCHG,  0x86,    -1,       OPER_R8,       OPER_RM8    ,  -1          )",
  "(  OP_XCHG,  0x87,    -1,       OPER_R16,      OPER_RM16   ,  -1          )",
  "(  OP_MOV,   0x88,    -1,       OPER_RM8,      OPER_R8     ,  -1          )",
  "(  OP_MOV,   0x89,    -1,       OPER_RM16,     OPER_R16    ,  -1          )",
  "(  OP_MOV,   0x8a,    -1,       OPER_R8,       OPER_RM8    ,  -1          )",
  "(  OP_MOV,   0x8b,    -1,       OPER_R16,      OPER_RM16   ,  -1          )",
  "(  OP_MOV,   0x8c,    -1,       OPER_RM16,     OPER_SREG   ,  -1          )",
  "(  OP_LEA,   0x8d,    -1,       OPER_R16,      OPER_M16    ,  -1          )",
  "(  OP_MOV,   0x8e,    -1,       OPER_SREG,     OPER_RM16   ,  -1          )",
  "(  OP_POP,   0x8f,    -1,       OPER_RM16,     -1          ,  -1          )",
  "(  OP_NOP,   0x90,    -1,       -1,            -1          ,  -1          )",
  "(  OP_XCHG,  0x91,    -1,       OPER_CX,       OPER_AX     ,  -1          )",
  "(  OP_XCHG,  0x92,    -1,       OPER_DX,       OPER_AX     ,  -1          )",
  "(  OP_XCHG,  0x93,    -1,       OPER_BX,       OPER_AX     ,  -1          )",
  "(  OP_XCHG,  0x94,    -1,       OPER_SP,       OPER_AX     ,  -1          )",
  "(  OP_XCHG,  0x95,    -1,       OPER_BP,       OPER_AX     ,  -1          )",
  "(  OP_XCHG,  0x96,    -1,       OPER_SI,       OPER_AX     ,  -1          )",
  "(  OP_XCHG,  0x97,    -1,       OPER_DI,       OPER_AX     ,  -1          )",
  "(  OP_CBW,   0x98,    -1,       OPER_AX,       OPER_AL     ,  -1          )",
  "(  OP_CWD,   0x99,    -1,       OPER_DX,       OPER_AX     ,  -1          )",
  "(  OP_CALLF, 0x9a,    -1,       OPER_IMM32,    -1          ,  -1          )",
  "(  OP_INVAL, 0x9b,    -1,       -1,            -1          ,  -1          )",
  "(  OP_PUSHF, 0x9c,    -1,       OPER_FLAGS,    -1          ,  -1          )",
  "(  OP_POPF,  0x9d,    -1,       OPER_FLAGS,    -1          ,  -1          )",
  "(  OP_SAHF,  0x9e,    -1,       OPER_AH,       -1          ,  -1          )",
  "(  OP_LAHF,  0x9f,    -1,       OPER_AH,       -1          ,  -1          )",
  "(  OP_MOV,   0xa0,    -1,       OPER_AL,       OPER_MOFF8  ,  -1          )",
  "(  OP_MOV,   0xa1,    -1,       OPER_AX,       OPER_MOFF16 ,  -1          )",
  "(  OP_MOV,   0xa2,    -1,       OPER_MOFF8,    OPER_AL     ,  -1          )",
  "(  OP_MOV,   0xa3,    -1,       OPER_MOFF16,   OPER_AX     ,  -1          )",
  "(  OP_MOVS,  0xa4,    -1,       OPER_DST8,     OPER_SRC8   ,  -1          )",
  "(  OP_MOVS,  0xa5,    -1,       OPER_DST16,    OPER_SRC16  ,  -1          )",
  "(  OP_CMPS,  0xa6,    -1,       OPER_DST8,     OPER_SRC8   ,  -1          )",
  "(  OP_CMPS,  0xa7,    -1,       OPER_DST16,    OPER_SRC16  ,  -1          )",
  "(  OP_TEST,  0xa8,    -1,       OPER_AL,       OPER_IMM8   ,  -1          )",
  "(  OP_TEST,  0xa9,    -1,       OPER_AX,       OPER_IMM16  ,  -1          )",
  "(  OP_STOS,  0xaa,    -1,       OPER_DST8,     OPER_AL     ,  -1          )",
  "(  OP_STOS,  0xab,    -1,       OPER_DST16,    OPER_AX     ,  -1          )",
  "(  OP_LODS,  0xac,    -1,       OPER_AL,       OPER_SRC8   ,  -1          )",
  "(  OP_LODS,  0xad,    -1,       OPER_AX,       OPER_SRC16  ,  -1          )",
  "(  OP_SCAS,  0xae,    -1,       OPER_DST8,     OPER_AL     ,  -1          )",
  "(  OP_SCAS,  0xaf,    -1,       OPER_DST16,    OPER_AX     ,  -1          )",
  "(  OP_MOV,   0xb0,    -1,       OPER_AL,       OPER_IMM8   ,  -1          )",
  "(  OP_MOV,   0xb1,    -1,       OPER_CL,       OPER_IMM8   ,  -1          )",
  "(  OP_MOV,   0xb2,    -1,       OPER_DL,       OPER_IMM8   ,  -1          )",
  "(  OP_MOV,   0xb3,    -1,       OPER_BL,       OPER_IMM8   ,  -1          )",
  "(  OP_MOV,   0xb4,    -1,       OPER_AH,       OPER_IMM8   ,  -1          )",
  "(  OP_MOV,   0xb5,    -1,       OPER_CH,       OPER_IMM8   ,  -1          )",
  "(  OP_MOV,   0xb6,    -1,       OPER_DH,       OPER_IMM8   ,  -1          )",
  "(  OP_MOV,   0xb7,    -1,       OPER_BH,       OPER_IMM8   ,  -1          )",
  "(  OP_MOV,   0xb8,    -1,       OPER_AX,       OPER_IMM16  ,  -1          )",
  "(  OP_MOV,   0xb9,    -1,       OPER_CX,       OPER_IMM16  ,  -1          )",
  "(  OP_MOV,   0xba,    -1,       OPER_DX,       OPER_IMM16  ,  -1          )",
  "(  OP_MOV,   0xbb,    -1,       OPER_BX,       OPER_IMM16  ,  -1          )",
  "(  OP_MOV,   0xbc,    -1,       OPER_SP,       OPER_IMM16  ,  -1          )",
  "(  OP_MOV,   0xbd,    -1,       OPER_BP,       OPER_IMM16  ,  -1          )",
  "(  OP_MOV,   0xbe,    -1,       OPER_SI,       OPER_IMM16  ,  -1          )",
  "(  OP_MOV,   0xbf,    -1,       OPER_DI,       OPER_IMM16  ,  -1          )",
  "(  OP_ROL,   0xc0,     0,       OPER_RM8,      OPER_IMM8,     -1          )",
  "(  OP_ROR,   0xc0,     1,       OPER_RM8,      OPER_IMM8,     -1          )",
  "(  OP_RCL,   0xc0,     2,       OPER_RM8,      OPER_IMM8,     -1          )",
  "(  OP_RCR,   0xc0,     3,       OPER_RM8,      OPER_IMM8,     -1          )",
  "(  OP_SHL,   0xc0,     4,       OPER_RM8,      OPER_IMM8,     -1          )",
  "(  OP_SHR,   0xc0,     5,       OPER_RM8,      OPER_IMM8,     -1          )",
  "(  OP_SHL,   0xc0,     6,       OPER_RM8,      OPER_IMM8,     -1          )",
  "(  OP_SAR,   0xc0,     7,       OPER_RM8,      OPER_IMM8,     -1          )",
  "(  OP_ROL,   0xc1,     0,       OPER_RM16,     OPER_IMM8,     -1          )",
  "(  OP_ROR,   0xc1,     1,       OPER_RM16,     OPER_IMM8,     -1          )",
  "(  OP_RCL,   0xc1,     2,       OPER_RM16,     OPER_IMM8,     -1          )",
  "(  OP_RCR,   0xc1,     3,       OPER_RM16,     OPER_IMM8,     -1          )",
  "(  OP_SHL,   0xc1,     4,       OPER_RM16,     OPER_IMM8,     -1          )",
  "(  OP_SHR,   0xc1,     5,       OPER_RM16,     OPER_IMM8,     -1          )",
  "(  OP_SHL,   0xc1,     6,       OPER_RM16,     OPER_IMM8,     -1          )",
  "(  OP_SAR,   0xc1,     7,       OPER_RM16,     OPER_IMM8,     -1          )",
  "(  OP_RET,   0xc2,    -1,       OPER_IMM16,    -1,            -1          )",
  "(  OP_RET,   0xc3,    -1,       -1,            -1,            -1          )",
  "(  OP_LES,   0xc4,    -1,       OPER_ES,       OPER_R16,      OPER_M32    )",
  "(  OP_LDS,   0xc5,    -1,       OPER_DS,       OPER_R16,      OPER_M32    )",
  "(  OP_MOV,   0xc6,     0,       OPER_RM8,      OPER_IMM8,     -1          )",
  "(  OP_MOV,   0xc7,     0,       OPER_RM16,     OPER_IMM16,    -1          )",
  "(  OP_ENTER, 0xc8,    -1,       OPER_BP,       OPER_IMM16,    OPER_IMM8   )",
  "(  OP_LEAVE, 0xc9,    -1,       OPER_BP,       -1,            -1          )",
  "(  OP_RETF,  0xca,    -1,       OPER_IMM16,    -1,            -1          )",
  "(  OP_RETF,  0xcb,    -1,       -1,            -1,            -1          )",
  "(  OP_INT,   0xcc,    -1,       OPER_LIT3,     OPER_FLAGS,    -1          )",
  "(  OP_INT,   0xcd,    -1,       OPER_IMM8,     OPER_FLAGS,    -1          )",
  "(  OP_INTO,  0xce,    -1,       OPER_FLAGS,    -1,            -1          )",
  "(  OP_IRET,  0xcf,    -1,       OPER_FLAGS,    -1,            -1          )",
  "(  OP_ROL,   0xd0,     0,       OPER_RM8,      OPER_LIT1,     -1          )",
  "(  OP_ROR,   0xd0,     1,       OPER_RM8,      OPER_LIT1,     -1          )",
  "(  OP_RCL,   0xd0,     2,       OPER_RM8,      OPER_LIT1,     -1          )",
  "(  OP_RCR,   0xd0,     3,       OPER_RM8,      OPER_LIT1,     -1          )",
  "(  OP_SHL,   0xd0,     4,       OPER_RM8,      OPER_LIT1,     -1          )",
  "(  OP_SHR,   0xd0,     5,       OPER_RM8,      OPER_LIT1,     -1          )",
  "(  OP_SHL,   0xd0,     6,       OPER_RM8,      OPER_LIT1,     -1          )",
  "(  OP_SAR,   0xd0,     7,       OPER_RM8,      OPER_LIT1,     -1          )",
  "(  OP_ROL,   0xd1,     0,       OPER_RM16,     OPER_LIT1,     -1          )",
  "(  OP_ROR,   0xd1,     1,       OPER_RM16,     OPER_LIT1,     -1          )",
  "(  OP_RCL,   0xd1,     2,       OPER_RM16,     OPER_LIT1,     -1          )",
  "(  OP_RCR,   0xd1,     3,       OPER_RM16,     OPER_LIT1,     -1          )",
  "(  OP_SHL,   0xd1,     4,       OPER_RM16,     OPER_LIT1,     -1          )",
  "(  OP_SHR,   0xd1,     5,       OPER_RM16,     OPER_LIT1,     -1          )",
  "(  OP_SHL,   0xd1,     6,       OPER_RM16,     OPER_LIT1,     -1          )",
  "(  OP_SAR,   0xd1,     7,       OPER_RM16,     OPER_LIT1,     -1          )",
  "(  OP_ROL,   0xd2,     0,       OPER_RM8,      OPER_CL,       -1          )",
  "(  OP_ROR,   0xd2,     1,       OPER_RM8,      OPER_CL,       -1          )",
  "(  OP_RCL,   0xd2,     2,       OPER_RM8,      OPER_CL,       -1          )",
  "(  OP_RCR,   0xd2,     3,       OPER_RM8,      OPER_CL,       -1          )",
  "(  OP_SHL,   0xd2,     4,       OPER_RM8,      OPER_CL,       -1          )",
  "(  OP_SHR,   0xd2,     5,       OPER_RM8,      OPER_CL,       -1          )",
  "(  OP_SHL,   0xd2,     6,       OPER_RM8,      OPER_CL,       -1          )",
  "(  OP_SAR,   0xd2,     7,       OPER_RM8,      OPER_CL,       -1          )",
  "(  OP_ROL,   0xd3,     0,       OPER_RM16,     OPER_CL,       -1          )",
  "(  OP_ROR,   0xd3,     1,       OPER_RM16,     OPER_CL,       -1          )",
  "(  OP_RCL,   0xd3,     2,       OPER_RM16,     OPER_CL,       -1          )",
  "(  OP_RCR,   0xd3,     3,       OPER_RM16,     OPER_CL,       -1          )",
  "(  OP_SHL,   0xd3,     4,       OPER_RM16,     OPER_CL,       -1          )",
  "(  OP_SHR,   0xd3,     5,       OPER_RM16,     OPER_CL,       -1          )",
  "(  OP_SHL,   0xd3,     6,       OPER_RM16,     OPER_CL,       -1          )",
  "(  OP_SAR,   0xd3,     7,       OPER_RM16,     OPER_CL,       -1          )",
  "(  OP_INVAL, 0xd4,    -1,       -1,            -1,            -1          )",
  "(  OP_INVAL, 0xd5,    -1,       -1,            -1,            -1          )",
  "(  OP_INVAL, 0xd6,    -1,       -1,            -1,            -1          )",
  "(  OP_XLAT,  0xd7,    -1,       OPER_AL,       OPER_M8,        -1          )",
  #/* x87 floating point instruction: 0xd8 - 0xdf */
  "(  OP_INVAL, 0xd8,    -1,       -1,            -1,            -1          )",
  "(  OP_INVAL, 0xd9,    -1,       -1,            -1,            -1          )",
  "(  OP_INVAL, 0xda,    -1,       -1,            -1,            -1          )",
  "(  OP_INVAL, 0xdb,    -1,       -1,            -1,            -1          )",
  "(  OP_INVAL, 0xdc,    -1,       -1,            -1,            -1          )",
  "(  OP_INVAL, 0xdd,    -1,       -1,            -1,            -1          )",
  "(  OP_INVAL, 0xde,    -1,       -1,            -1,            -1          )",
  "(  OP_INVAL, 0xdf,    -1,       -1,            -1,            -1          )",
  "(  OP_LOOPNE,0xe0,    -1,       OPER_CX,       OPER_REL8,     -1          )",
  "(  OP_LOOPE, 0xe1,    -1,       OPER_CX,       OPER_REL8,     -1          )",
  "(  OP_LOOP,  0xe2,    -1,       OPER_CX,       OPER_REL8,     -1          )",
  "(  OP_JCXZ,  0xe3,    -1,       OPER_CX,       OPER_REL8,     -1          )",
  "(  OP_IN,    0xe4,    -1,       OPER_AL,       OPER_IMM8,     -1          )",
  "(  OP_IN,    0xe5,    -1,       OPER_AX,       OPER_IMM8,     -1          )",
  "(  OP_OUT,   0xe6,    -1,       OPER_IMM8,     OPER_AL,       -1          )",
  "(  OP_OUT,   0xe7,    -1,       OPER_IMM8,     OPER_AX,       -1          )",
  "(  OP_CALL,  0xe8,    -1,       OPER_REL16,    -1,            -1          )",
  "(  OP_JMP,   0xe9,    -1,       OPER_REL16,    -1,            -1          )",
  "(  OP_JMPF,  0xea,    -1,       OPER_IMM32,    -1,            -1          )",
  "(  OP_JMP,   0xeb,    -1,       OPER_REL8,     -1,            -1          )",
  "(  OP_IN,    0xec,    -1,       OPER_AL,       OPER_DX,       -1          )",
  "(  OP_IN,    0xed,    -1,       OPER_AX,       OPER_DX,       -1          )",
  "(  OP_OUT,   0xee,    -1,       OPER_DX,       OPER_AL,       -1          )",
  "(  OP_OUT,   0xef,    -1,       OPER_DX,       OPER_AX,       -1          )",
  #// LOCK PREFIX: 0xf0
  "(  OP_INVAL, 0xf0,    -1,       -1,            -1,            -1          )",
  "(  OP_INVAL, 0xf1,    -1,       -1,            -1,            -1          )",
  #// REPNE, REPE: 0xf2, 0xf3
  "(  OP_INVAL, 0xf2,    -1,       -1,            -1,            -1          )",
  "(  OP_INVAL, 0xf3,    -1,       -1,            -1,            -1          )",
  "(  OP_HLT,   0xf4,    -1,       -1,            -1,            -1          )",
  "(  OP_CMC,   0xf5,    -1,       -1,            -1,            -1          )",
  "(  OP_TEST,  0xf6,     0,       OPER_RM8,      OPER_IMM8,     -1          )",
  "(  OP_TEST,  0xf6,     1,       OPER_RM8,      OPER_IMM8,     -1          )",
  "(  OP_NOT,   0xf6,     2,       OPER_RM8,      -1,            -1          )",
  "(  OP_NEG,   0xf6,     3,       OPER_RM8,      -1,            -1          )",
  "(  OP_MUL,   0xf6,     4,       OPER_AX,       OPER_AL,       OPER_RM8    )",
  "(  OP_IMUL,  0xf6,     5,       OPER_AX,       OPER_AL,       OPER_RM8    )",
  "(  OP_DIV,   0xf6,     6,       OPER_AH,       OPER_AL,       OPER_RM8    )",
  "(  OP_DIV,   0xf6,     7,       OPER_AH,       OPER_AL,       OPER_RM8    )",
  "(  OP_TEST,  0xf7,     0,       OPER_RM16,     OPER_IMM16,    -1          )",
  "(  OP_TEST,  0xf7,     1,       OPER_RM16,     OPER_IMM16,    -1          )",
  "(  OP_NOT,   0xf7,     2,       OPER_RM16,     -1,            -1          )",
  "(  OP_NEG,   0xf7,     3,       OPER_RM16,     -1,            -1          )",
  "(  OP_MUL,   0xf7,     4,       OPER_DX,       OPER_AX,       OPER_RM16   )",
  "(  OP_IMUL,  0xf7,     5,       OPER_DX,       OPER_AX,       OPER_RM16   )",
  "(  OP_DIV,   0xf7,     6,       OPER_DX,       OPER_AX,       OPER_RM16   )",
  "(  OP_DIV,   0xf7,     7,       OPER_DX,       OPER_AX,       OPER_RM16   )",
  "(  OP_CLC,   0xf8,    -1,       -1,            -1,            -1          )",
  "(  OP_STC,   0xf9,    -1,       -1,            -1,            -1          )",
  "(  OP_CLI,   0xfa,    -1,       -1,            -1,            -1          )",
  "(  OP_STI,   0xfb,    -1,       -1,            -1,            -1          )",
  "(  OP_CLD,   0xfc,    -1,       -1,            -1,            -1          )",
  "(  OP_STD,   0xfd,    -1,       -1,            -1,            -1          )",
  "(  OP_INC,   0xfe,     0,       OPER_RM8,      -1,            -1          )",
  "(  OP_DEC,   0xfe,     1,       OPER_RM8,      -1,            -1          )",
  "(  OP_INC,   0xff,     0,       OPER_RM16,     -1,            -1          )",
  "(  OP_DEC,   0xff,     1,       OPER_RM16,     -1,            -1          )",
  "(  OP_CALL,  0xff,     2,       OPER_RM16,     -1,            -1          )",
  "(  OP_CALLF, 0xff,     3,       OPER_RM32,     -1,            -1          )",
  "(  OP_JMP,   0xff,     4,       OPER_RM16,     -1,            -1          )",
  "(  OP_JMPF,  0xff,     5,       OPER_RM32,     -1,            -1          )",
  "(  OP_PUSH,  0xff,     6,       OPER_RM16,     -1,            -1          )",
]

def pre_process():
    ret = []
    for t in TBL:
        parts = [x.strip() for x in t[1:-1].split(',')]
        assert(len(parts) == 6)
        ret.append(parts)
    return ret

def fmt_row(fmts, elts):
    assert(len(fmts) == len(elts))
    s = ''
    for f,e in zip(fmts[:-1], elts[:-1]):
        s += f % (e+',')
    s += fmts[-1] % elts[-1]
    return s


def emit_c_ops(tbl):
    all_ops = sorted(list(set([row[0] for row in tbl])))
    fmts = ['%-12s', '%-10s']

    print('#define INSTR_OP_ARRAY(_) \\')
    for op in all_ops:
        mneumonic = op.split('_')[1].lower()
        row = fmt_row(fmts, [op, f'"{mneumonic}"'])
        print('  _(  %s) \\' % row)
    print('')

    print('enum {')
    print('#define ELT(enum_symbol, _) enum_symbol,')
    print('  INSTR_OP_ARRAY(ELT)')
    print('#undef ELT')
    print('};')
    print('')

    print('static inline const char *instr_op_mneumonic(int op)')
    print('{')
    print('  static const char *arr[] = {')
    print('    #define ELT(_, str) str,')
    print('    INSTR_OP_ARRAY(ELT)')
    print('    #undef ELT')
    print('  };')
    print('  if ((size_t)op >= ARRAY_SIZE(arr)) return NULL;')
    print('  return arr[op];')
    print('}')
    print('')

def emit_c_opers(tbl):
    all_opers = []
    for row in tbl:
        if row[3] != '-1': all_opers.append(row[3])
        if row[4] != '-1': all_opers.append(row[4])
        if row[5] != '-1': all_opers.append(row[5])
    all_opers = sorted(list(set(all_opers)))

    print('enum {')
    for oper in all_opers:
        print('  %s,' % oper)
    print('};')

def emit_c_fmt(tbl):
    fmts = ['%-14s', '%-9s', '%4s', '   %-14s', '%-14s', '%-14s']
    print('static instr_fmt_t instr_tbl[] = {')
    for row in tbl:
        row = fmt_row(fmts, row)
        print('  {  %s},' % row)
    print('};')

tbl = pre_process()

emit_c_ops(tbl)
emit_c_opers(tbl)
emit_c_fmt(tbl)
