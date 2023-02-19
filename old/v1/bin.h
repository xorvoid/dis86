#include "header.h"

typedef struct bin bin_t;
struct bin
{
  u8 *   mem;
  size_t len;
  size_t idx;
  size_t base_addr;
};

static inline void bin_init(bin_t *b, size_t base_addr, char *mem, size_t len)
{
  b->mem = malloc(len);
  memcpy(b->mem, mem, len);
  b->len = len;
  b->idx = base_addr;
  b->base_addr = base_addr;
}

static inline u8 bin_byte_at(bin_t *b, size_t idx)
{
  if (idx < b->base_addr) FAIL("Binary access below start of region");
  if (idx >= b->base_addr + b->len) FAIL("Binary access beyond end of region");
  return b->mem[idx - b->base_addr];
}

static inline u8 bin_fetch_u8(bin_t *b)
{
  u8 byte = bin_byte_at(b, b->idx);
  b->idx++;
  return byte;
}

static inline u16 bin_fetch_u16(bin_t *b)
{
  u8 low = bin_fetch_u8(b);
  u8 high = bin_fetch_u8(b);
  return (u16)high << 8 | (u16)low;
}

static inline size_t bin_baseaddr(bin_t *b)
{
  return b->base_addr;
}

static inline size_t bin_location(bin_t *b)
{
  return b->idx;
}

static inline size_t bin_length(bin_t *b)
{
  return b->len;
}

static inline void bin_dump(bin_t *b)
{
  printf("BIN DUMP LOCATION %zx: ", b->idx);
  size_t end = MIN(b->idx + 16, b->base_addr + b->len);
  for (size_t idx = b->idx; idx < end; idx++) {
    printf("%02x ", bin_byte_at(b, idx));
  }
  printf("\n");
}
