#include "addr.h"
#include "header.h"

addr_t parse_addr(const char *s)
{
  const char *end = s + strlen(s);

  const char *colon = strchr(s, ':');
  if (!colon) FAIL("Invalid addr: '%s'", s);

  addr_t ret;
  ret._i._overlay = 0;
  ret._i._seg = parse_hex_u16(s, colon-s);
  ret._i._off = parse_hex_u16(colon+1, end-(colon+1));
  return ret;
}

size_t addr_abs(addr_t s)
{
  assert(!s._i._overlay);
  return (size_t)s._i._seg * 16 + (size_t)s._i._off;
}

addr_t addr_relative_to_segment(addr_t s, u16 seg)
{
  assert(!s._i._overlay);

  if (s._i._seg < seg) {
    FAIL("Cannot compute relative segment, expected >= %04x, got %04x", seg, s._i._seg);
  }

  assert(s._i._seg >= seg);
  s._i._seg -= seg;
  return s;
}
