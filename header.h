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

#define ARRAY_SIZE(arr) (sizeof(arr)/sizeof((arr)[0]))
#define FAIL(...) do { fprintf(stderr, "FAIL: "); fprintf(stderr, __VA_ARGS__); fprintf(stderr, "\n"); abort(); } while(0)
