#pragma once
#include "header.h"

typedef struct segoff segoff_t;
struct segoff
{
  u16 seg;
  u16 off;
};

static segoff_t parse_segoff(const char *s)
{
  const char *end = s + strlen(s);

  const char *colon = strchr(s, ':');
  if (!colon) FAIL("Invalid segoff: '%s'", s);

  segoff_t ret;
  ret.seg = parse_hex_u16(s, colon-s);
  ret.off = parse_hex_u16(colon+1, end-(colon+1));
  return ret;
}

static size_t segoff_abs(segoff_t s)
{
  return (size_t)s.seg * 16 + (size_t)s.off;
}
