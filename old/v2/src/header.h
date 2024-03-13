#pragma once
#include <assert.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>

typedef uint8_t   u8;
typedef  int8_t   i8;
typedef uint16_t  u16;
typedef  int16_t  i16;
typedef uint32_t  u32;
typedef  int32_t  i32;
typedef uint64_t  u64;
typedef  int64_t  i64;

//static inline void bin_dump_and_abort();

#define MIN(a, b) (((a)<(b))?(a):(b))
#define MAX(a, b) (((a)>(b))?(a):(b))
#define ARRAY_SIZE(arr) (sizeof(arr)/sizeof((arr)[0]))
#define FAIL(...) do { fprintf(stderr, "FAIL: "); fprintf(stderr, __VA_ARGS__); fprintf(stderr, "\n"); exit(42); } while(0)
#define UNIMPL() FAIL("UNIMPLEMENTED: %s:%d", __FILE__, __LINE__)

static inline char *read_file(const char *filename, size_t *out_sz)
{
  FILE *fp = fopen(filename, "r");
  if (!fp) FAIL("Failed to open: '%s'", filename);

  fseek(fp, 0, SEEK_END);
  size_t file_sz = ftell(fp);
  fseek(fp, 0, SEEK_SET);

  char *mem = malloc(file_sz);
  if (!mem) FAIL("Failed to allocate");

  size_t n = fread(mem, 1, file_sz, fp);
  if (n != file_sz) FAIL("Failed to read");
  fclose(fp);

  if (out_sz) *out_sz = file_sz;
  return mem;
}

static inline void hexdump(u8 *mem, size_t len)
{
  size_t idx = 0;
  while (idx < len) {
    size_t line_end = MIN(idx+16, len);
    for (; idx < line_end; idx++) {
      printf("%02x ", mem[idx]);
    }
    printf("\n");
  }
}

static u64 parse_hex_u64(const char *s, size_t len)
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

static u32 parse_hex_u32(const char *s, size_t len)
{
  if (len > 8) FAIL("Hex string too long to fit in u16");
  return (u32)parse_hex_u64(s, len);
}

static u16 parse_hex_u16(const char *s, size_t len)
{
  if (len > 4) FAIL("Hex string too long to fit in u16");
  return (u16)parse_hex_u64(s, len);
}

static u8 parse_hex_u8(const char *s, size_t len)
{
  if (len > 2) FAIL("Hex string too long to fit in u16");
  return (u16)parse_hex_u64(s, len);
}

static inline bool parse_bytes_u64(const char *buf, size_t len, uint64_t *_num)
{
  if (len == 0) return false;

  uint64_t num = 0;
  for (size_t i = 0; i < len; i++) {
    char c = buf[i];
    if (!('0' <= c && c <= '9')) return false; // not a decimal digit

    uint64_t next_num = 10*num + (uint64_t)(c-'0');
    if (next_num < num) return false; // overflow!
    num = next_num;
  }

  *_num = num;
  return true;
}

static inline bool parse_bytes_u32(const char *buf, size_t len, uint32_t *_num)
{
  u64 num;
  if (!parse_bytes_u64(buf, len, &num)) return false;
  if ((u64)(u32)num != num) return false;
  *_num = (u32)num;
  return true;
}

static inline bool parse_bytes_u16(const char *buf, size_t len, uint16_t *_num)
{
  u64 num;
  if (!parse_bytes_u64(buf, len, &num)) return false;
  if ((u64)(u16)num != num) return false;
  *_num = (u16)num;
  return true;
}

static inline bool parse_bytes_u8(const char *buf, size_t len, uint8_t *_num)
{
  u64 num;
  if (!parse_bytes_u64(buf, len, &num)) return false;
  if ((u64)(u8)num != num) return false;
  *_num = (u8)num;
  return true;
}

static inline bool parse_bytes_i64(const char *buf, size_t len, int64_t *_num)
{
  if (len == 0) return false;

  bool neg = false;
  if (buf[0] == '-') {
    neg = true;
    buf++;
    len--;
  }

  uint64_t unum = 0;
  if (!parse_bytes_u64(buf, len, &unum)) return false;

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

static inline bool parse_bytes_i32(const char *buf, size_t len, int32_t *_num)
{
  i64 num;
  if (!parse_bytes_i64(buf, len, &num)) return false;
  if ((i64)(i32)num != num) return false;
  *_num = (i32)num;
  return true;
}

static inline bool parse_bytes_i16(const char *buf, size_t len, int16_t *_num)
{
  i64 num;
  if (!parse_bytes_i64(buf, len, &num)) return false;
  if ((i64)(i16)num != num) return false;
  *_num = (i16)num;
  return true;
}

static inline bool parse_bytes_i8(const char *buf, size_t len, int8_t *_num)
{
  i64 num;
  if (!parse_bytes_i64(buf, len, &num)) return false;
  if ((i64)(i8)num != num) return false;
  *_num = (i8)num;
  return true;
}
