#pragma once
#include <assert.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

// printf formatting
#define ADDR_FMT      "%s%04x:%04x"
#define ADDR_ARG(s)   addr_fmtarg_overlay(s), addr_fmtarg_seg(s), addr_fmtarg_off(s)
#define ADDR_MAKE(seg, off) ({ addr_t a; a._i._overlay = 0; a._i._seg = (seg); a._i._off = (off); a; })
#define ADDR_MAKE_EXT(ovr, seg, off) ({ addr_t a; a._i._overlay = (ovr); a._i._seg = (seg); a._i._off = (off); a; })

struct _inner {
  uint16_t _overlay;
  uint16_t _seg;
  uint16_t _off;
};

typedef struct addr addr_t;
struct addr
{
  struct _inner _i;
};

// Parse string in the form "xxxx:yyyy"
addr_t parse_addr(const char *s);

// Compute absoulte address
size_t addr_abs(addr_t s);

// Compute relative addr from some base segment
addr_t addr_relative_to_segment(addr_t s, uint16_t seg);

// Accessors
static inline uint16_t addr_is_overlay(addr_t s) { return s._i._overlay; }
static inline uint16_t addr_overlay_num(addr_t s) { assert(s._i._overlay); return s._i._seg; }
static inline uint16_t addr_seg(addr_t s) { assert(!s._i._overlay); return s._i._seg; }
static inline uint16_t addr_off(addr_t s) { return s._i._off; }

// Fmt args
static inline const char * addr_fmtarg_overlay(addr_t s) { return s._i._overlay ? "overlay_" : ""; }
static inline uint16_t     addr_fmtarg_seg(addr_t s)     { return s._i._seg; }
static inline uint16_t     addr_fmtarg_off(addr_t s)     { return s._i._off; }

// Equal??
static inline bool addr_equal(addr_t a, addr_t b)
{
  return
    a._i._overlay == b._i._overlay &&
    a._i._seg == b._i._seg &&
    a._i._off == b._i._off;
}

// Difference
static inline int32_t addr_difference(addr_t a, addr_t b)
{
  return (int32_t)addr_abs(a) - (int32_t)addr_abs(b);
}
