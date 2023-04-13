#pragma once
#include "header.h"

enum {
  BASETYPE_U8,
  BASETYPE_U16,
  BASETYPE_U32,
};

const char *basetype_str(int t);

typedef struct type type_t;
struct type
{
  int    basetype;
  int    is_array;
  size_t array_len;
};

bool type_parse(type_t *typ, const char *str);
u16  type_size(type_t *typ);
