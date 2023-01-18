#pragma once
#include <assert.h>
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

static inline void bin_dump_and_abort();

#define MIN(a, b) (((a)<(b))?(a):(b))
#define MAX(a, b) (((a)>(b))?(a):(b))
#define ARRAY_SIZE(arr) (sizeof(arr)/sizeof((arr)[0]))
#define FAIL(...) do { fprintf(stderr, "FAIL: "); fprintf(stderr, __VA_ARGS__); fprintf(stderr, "\n"); bin_dump_and_abort(); abort(); } while(0)

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
