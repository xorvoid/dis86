#include "header.h"

typedef struct binary binary_t;
struct binary
{
  u8 *   mem;
  size_t len;
  size_t idx;
  size_t base_addr;
};

static inline void binary_init(binary_t *b, size_t base_addr, char *mem, size_t len)
{
  b->mem = malloc(len);
  memcpy(b->mem, mem, len);
  b->len = len;
  b->idx = base_addr;
  b->base_addr = base_addr;
}

static inline u8 binary_byte_at(binary_t *b, size_t idx)
{
  if (idx < b->base_addr) FAIL("Binary access below start of region");
  if (idx >= b->base_addr + b->len) FAIL("Binary access beyond end of region");
  return b->mem[idx - b->base_addr];
}

static inline u8 binary_peek_u8(binary_t *b)
{
  u8 byte = binary_byte_at(b, b->idx);
  return byte;
}

static inline void binary_advance_u8(binary_t *b)
{
  b->idx++;
}

static inline u8 binary_fetch_u8(binary_t *b)
{
  u8 byte = binary_peek_u8(b);
  binary_advance_u8(b);
  return byte;
}

static inline u16 binary_fetch_u16(binary_t *b)
{
  u8 low = binary_fetch_u8(b);
  u8 high = binary_fetch_u8(b);
  return (u16)high << 8 | (u16)low;
}

static inline size_t binary_baseaddr(binary_t *b)
{
  return b->base_addr;
}

static inline size_t binary_location(binary_t *b)
{
  return b->idx;
}

static inline size_t binary_length(binary_t *b)
{
  return b->len;
}

static inline void binary_dump(binary_t *b)
{
  printf("BINARY DUMP LOCATION %zx: ", b->idx);
  size_t end = MIN(b->idx + 16, b->base_addr + b->len);
  for (size_t idx = b->idx; idx < end; idx++) {
    printf("%02x ", binary_byte_at(b, idx));
  }
  printf("\n");
}
