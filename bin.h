#include "header.h"

static u8 *BIN_MEM = NULL;
static size_t BIN_LEN = 0;
static size_t BIN_IDX = 0;

static void bin_init(const char *filename)
{
  if (BIN_MEM) {
    free(BIN_MEM);
  }
  BIN_MEM = (u8*)read_file(filename, &BIN_LEN);
  BIN_IDX = 0;
}

static u8 bin_fetch_u8()
{
  if (BIN_IDX >= BIN_LEN) FAIL("Fetch beyond end of region");
  return BIN_MEM[BIN_IDX++];
}

static u16 bin_fetch_u16()
{
  u8 low = bin_fetch_u8();
  u8 high = bin_fetch_u8();
  return (u16)high << 8 | (u16)low;
}

static u8 bin_byte_at(size_t idx)
{
  if (idx >= BIN_LEN) FAIL("Binary access beyond end of region");
  return BIN_MEM[idx];
}

static size_t bin_location()
{
  return BIN_IDX;
}

static inline void bin_dump_and_abort()
{
  printf("ABORTING AT LOCATION %zx: ", BIN_IDX);
  size_t end = MIN(BIN_IDX + 16, BIN_LEN);
  for (size_t idx = BIN_IDX; idx < end; idx++) {
    printf("%02x ", bin_byte_at(idx));
  }
  printf("\n");

  abort();
}
