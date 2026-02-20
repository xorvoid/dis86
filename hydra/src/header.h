#pragma once

#include <assert.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>
#include "typedefs.h"

#define FAIL(...) do { fprintf(stderr, "FAIL: "); fprintf(stderr, __VA_ARGS__); fprintf(stderr, "\n"); abort(); } while(0)
#define UNIMPL() FAIL("UNIMPLEMENTED at %s:%d", __FILE__, __LINE__)
#define ASSUME(cond) do { if (!(cond)) FAIL("ASSUMPTION FAILED: (" #cond ") at %s:%d", __FILE__, __LINE__); } while(0)

#define ARRAY_SIZE(arr) (sizeof(arr)/sizeof((arr)[0]))
#define MIN(a, b) (((a)<(b))?(a):(b))
#define MAX(a, b) (((a)>(b))?(a):(b))

static inline i64 wallclock(void)
{
  struct timespec ts[1] = {{}};
  clock_gettime(CLOCK_REALTIME, ts);
  return (i64)ts->tv_sec * 1000000000ul + (i64)ts->tv_nsec;
}

static inline u16 load_unaligned_u16(u8 *mem)
{
  u16 n;
  memcpy(&n, mem, sizeof(n));
  return n;
}

static inline u32 load_unaligned_32(u8 *mem)
{
  u32 n;
  memcpy(&n, mem, sizeof(n));
  return n;
}

static inline u64 load_unaligned_u64(u8 *mem)
{
  u64 n;
  memcpy(&n, mem, sizeof(n));
  return n;
}

static inline void hexdump(void *_mem, size_t len)
{
  u8 *mem = (u8*)_mem;

  size_t idx = 0;
  while (idx < len) {
    size_t line_end = MIN(idx+16, len);
    for (; idx < line_end; idx++) {
      printf("%02x ", mem[idx]);
    }
    printf("\n");
  }
}

static inline char *file_read(const char *name, size_t *_len)
{
  FILE *fp = fopen(name, "r");
  if (!fp) FAIL("Failed to open file: %s", name);

  fseek(fp, 0, SEEK_END);
  size_t len = ftell(fp);
  fseek(fp, 0, SEEK_SET);

  char *mem = malloc(len);
  if (!mem) FAIL("Failed to allocate file buffer");

  size_t n = fread(mem, 1, len, fp);
  if (n != len) FAIL("Failed to read everything from file: %s", name);

  fclose(fp);

  *_len = len;
  return mem;
}

static inline void file_write(const char *name, u8 *mem, size_t len)
{
  FILE *fp = fopen(name, "w");
  if (!fp) FAIL("Failed to open file: %s", name);

  size_t n = fwrite(mem, 1, len, fp);
  if (n != len) FAIL("Failed to write everything to file: %s", name);

  fclose(fp);
}

static inline u64 parse_hex_u64(const char *s, size_t len)
{
  if (len > 16) FAIL("Hex string too long to fit in u64");

  u64 ret = 0;
  for (size_t i = 0; i < len; i++) {
    char c = s[i];
    if ('0' <= c && c <= '9') {
      ret = ret*16 + (c-'0');
    } else if ('a' <= c && c <= 'f') {
      ret = ret*16 + (c-'a'+10);
    } else if ('A' <= c && c <= 'F') {
      ret = ret*16 + (c-'A'+10);
    } else {
      FAIL("Invalid hex string: '%.*s'", (int)len, s);
    }
  }

  return ret;
}

static inline u32 parse_hex_u32(const char *s, size_t len)
{
  if (len > 8) FAIL("Hex string too long to fit in u16");
  return (u32)parse_hex_u64(s, len);
}

static inline u16 parse_hex_u16(const char *s, size_t len)
{
  if (len > 4) FAIL("Hex string too long to fit in u16");
  return (u16)parse_hex_u64(s, len);
}

static inline u8 parse_hex_u8(const char *s, size_t len)
{
  if (len > 2) FAIL("Hex string too long to fit in u16");
  return (u16)parse_hex_u64(s, len);
}

static inline bool parse_u64(const char *s, uint64_t *_num)
{
  uint64_t num = 0;
  while (1) {
    char c = *s++;
    if (!c) break;
    if (!('0' <= c && c <= '9')) return false; // not a decimal digit

    uint64_t next_num = 10*num + (uint64_t)(c-'0');
    if (next_num < num) return false; // overflow!
    num = next_num;
  }

  *_num = num;
  return true;
}

static inline bool parse_u32(const char *s, u32 *_num)
{
  u64 num;
  if (!parse_u64(s, &num)) return false;
  if (num != (u64)(u32)num) return false; // overflowed this size
  *_num = (u32)num;
  return true;
}

static inline bool parse_u16(const char *s, u16 *_num)
{
  u64 num;
  if (!parse_u64(s, &num)) return false;
  if (num != (u64)(u16)num) return false; // overflowed this size
  *_num = (u16)num;
  return true;
}

static inline bool parse_u8(const char *s, u8 *_num)
{
  u64 num;
  if (!parse_u64(s, &num)) return false;
  if (num != (u64)(u8)num) return false; // overflowed this size
  *_num = (u8)num;
  return true;
}

static inline bool parse_i64(const char *s, int64_t *_num)
{
  bool neg = false;
  if (*s == '-') {
    neg = true;
    s++;
  }

  uint64_t unum = 0;
  if (!parse_u64(s, &unum)) return false;

  int64_t num;
  if (neg) {
    if (unum > (1ull<<63)) return false; // overflow
    num = -(int64_t)unum;
  } else {
    if (unum >= (1ull<<63)) return false; // overflow
    num = (int64_t)unum;
  }

  *_num = num;
  return true;
}

static inline bool parse_i32(const char *s, i32 *_num)
{
  i64 num;
  if (!parse_i64(s, &num)) return false;
  if (num != (i64)(i32)num) return false; // overflowed this size
  *_num = (i32)num;
  return true;
}

static inline bool parse_i16(const char *s, i16 *_num)
{
  i64 num;
  if (!parse_i64(s, &num)) return false;
  if (num != (i64)(i16)num) return false; // overflowed this size
  *_num = (i16)num;
  return true;
}

static inline bool parse_i8(const char *s, i8 *_num)
{
  i64 num;
  if (!parse_i64(s, &num)) return false;
  if (num != (i64)(i8)num) return false; // overflowed this size
  *_num = (i8)num;
  return true;
}

static inline bool starts_with(const char *s, const char *prefix) {
  size_t s_len = strlen(s);
  size_t prefix_len = strlen(prefix);
  if (s_len < prefix_len) return false;
  return 0 == memcmp(s, prefix, prefix_len);
}
