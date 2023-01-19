#include "header.h"

typedef struct bin bin_t;
struct bin
{
  u8 *   mem;
  size_t len;
  size_t idx;
};

static inline void bin_init(bin_t *b, char *mem, size_t len)
{
  b->mem = malloc(len);
  memcpy(b->mem, mem, len);
  b->len = len;
  b->idx = 0;
}

static inline u8 bin_fetch_u8(bin_t *b)
{
  if (b->idx >= b->len) FAIL("Fetch beyond end of region");
  return b->mem[b->idx++];
}

static inline u16 bin_fetch_u16(bin_t *b)
{
  u8 low = bin_fetch_u8(b);
  u8 high = bin_fetch_u8(b);
  return (u16)high << 8 | (u16)low;
}

static inline u8 bin_byte_at(bin_t *b, size_t idx)
{
  if (idx >= b->len) FAIL("Binary access beyond end of region");
  return b->mem[idx];
}

static inline size_t bin_location(bin_t *b)
{
  return b->idx;
}

static inline void bin_dump_and_abort(bin_t *b)
{
  printf("ABORTING AT LOCATION %zx: ", b->idx);
  size_t end = MIN(b->idx + 16, b->len);
  for (size_t idx = b->idx; idx < end; idx++) {
    printf("%02x ", bin_byte_at(b, idx));
  }
  printf("\n");

  abort();
}
