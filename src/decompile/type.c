#include "type.h"

const char *basetype_str(int t)
{
  switch (t) {
    case BASETYPE_U8:  return "u8";
    case BASETYPE_U16: return "u16";
    case BASETYPE_U32: return "u32";
    default: FAIL("Unknown basetype: %d", t);
  }
}

static u16 basetype_size(int t)
{
  switch (t) {
    case BASETYPE_U8:  return 1;
    case BASETYPE_U16: return 2;
    case BASETYPE_U32: return 4;
    default: FAIL("Unknown basetype: %d", t);
  }
}

static bool basetype_parse(const char *s, size_t len, int *out)
{
  int t = -1;
  if (len == 2) {
    if (0 == memcmp(s, "u8", 2)) t = BASETYPE_U8;
  } else if (len == 3) {
    if (0 == memcmp(s, "u16", 3)) t = BASETYPE_U16;
    if (0 == memcmp(s, "u32", 3)) t = BASETYPE_U32;
  }

  if (t == -1) return false;

  if (out) *out = t;
  return true;
}

bool type_parse(type_t *typ, const char *str)
{
  const char *lbrace = str;
  while (*lbrace && *lbrace != '[') lbrace++;

  int base;
  if (!basetype_parse(str, lbrace-str, &base)) return false;

  if (!*lbrace) { // not an array?
    typ->basetype  = base;
    typ->is_array  = false;
    typ->array_len = 0;
    return true;
  }

  // is an array

  // find ending ']'
  const char *rbrace = lbrace+1;
  while (*rbrace && *rbrace != ']') rbrace++;
  if (*rbrace == 0 || *(rbrace+1) != 0) return false;

  const char * size_str = lbrace+1;
  size_t       size_len = rbrace - size_str;

  u64 size;
  if (!parse_bytes_u64(size_str, size_len, &size)) {
    return false;
  }

  typ->basetype  = base;
  typ->is_array  = true;
  typ->array_len = size;
  return true;
}

u16 type_size(type_t *typ)
{
  u16 sz = basetype_size(typ->basetype);
  if (typ->is_array) {
    sz *= typ->array_len;
  }
  return sz;
}
