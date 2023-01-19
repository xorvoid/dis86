#pragma once
#include "header.h"

enum {
  OPERAND_TYPE_NONE,
  OPERAND_TYPE_VAL,
  OPERAND_TYPE_ADDR,
};

typedef struct operand operand_t;
struct operand {
  int type;
  int has_mode : 1;
  int has_reg : 1;
  int has_sreg : 1;
  int has_imm : 1;
  int has_rel : 1;
  int has_abs32 : 1;
  int has_seg_override : 1;

  int mode;
  u8 reg;
  u8 sreg;
  u8 seg_override;
  u16 imm;
  u16 rel;
  u32 abs32;
};

static inline operand_t operand_reg(u8 reg)
{
  operand_t operand[1] = {{}};
  operand->type = OPERAND_TYPE_VAL;
  operand->has_reg = 1;
  operand->reg = reg;
  return operand[0];
}

static inline operand_t operand_sreg(u8 sreg)
{
  operand_t operand[1] = {{}};
  operand->type = OPERAND_TYPE_VAL;
  operand->has_sreg = 1;
  operand->sreg = sreg;
  return operand[0];
}

static inline operand_t operand_imm(u16 imm)
{
  operand_t operand[1] = {{}};
  operand->type = OPERAND_TYPE_VAL;
  operand->has_imm = 1;
  operand->imm = imm;
  return operand[0];
}

static inline operand_t operand_rel16(u16 rel)
{
  operand_t operand[1] = {{}};
  operand->type = OPERAND_TYPE_VAL;
  operand->has_rel = 1;
  operand->rel = rel;
  return operand[0];
}

static inline operand_t operand_rel8(u8 rel)
{
  operand_t operand[1] = {{}};
  operand->type = OPERAND_TYPE_VAL;
  operand->has_rel = 1;
  operand->rel = (u16)(i16)(i8)rel;
  return operand[0];
}

static inline operand_t operand_abs32(u32 abs32)
{
  operand_t operand[1] = {{}};
  operand->type = OPERAND_TYPE_VAL;
  operand->has_abs32 = 1;
  operand->abs32 = abs32;
  return operand[0];
}

static inline operand_t operand_addr_imm(u16 imm, int has_seg, u8 seg)
{
  operand_t operand[1] = {{}};
  operand->type = OPERAND_TYPE_ADDR;
  operand->has_imm = 1;
  operand->has_seg_override = has_seg;
  operand->imm = imm;
  operand->seg_override = seg;
  return operand[0];
}

static inline operand_t operand_addr_reg(u8 seg, u8 reg)
{
  operand_t operand[1] = {{}};
  operand->type = OPERAND_TYPE_ADDR;
  operand->has_reg = 1;
  operand->has_seg_override = 1;
  operand->reg = reg;
  operand->seg_override = seg;
  return operand[0];
}

static inline operand_t operand_addr_mode(int mode, int has_seg, u8 seg)
{
  operand_t operand[1] = {{}};
  operand->type = OPERAND_TYPE_ADDR;
  operand->has_mode = 1;
  operand->has_seg_override = 1;
  operand->mode = mode;
  operand->seg_override = seg;
  return operand[0];
}

static inline operand_t operand_addr_mode_imm(int mode, u16 imm, int has_seg, u8 seg)
{
  operand_t operand[1] = {{}};
  operand->type = OPERAND_TYPE_ADDR;
  operand->has_mode = 1;
  operand->has_imm = 1;
  operand->has_seg_override = 1;
  operand->mode = mode;
  operand->imm = imm;
  operand->seg_override = seg;
  return operand[0];
}
