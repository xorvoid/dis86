#pragma once

#include <assert.h>
#include <stdint.h>

#define IS_U8(var)  _Generic((var), uint8_t:  1, default: 0)
#define IS_U16(var) _Generic((var), uint16_t: 1, default: 0)
#define IS_U32(var) _Generic((var), uint32_t: 1, default: 0)
#define IS_U64(var) _Generic((var), uint64_t: 1, default: 0)
#define IS_I8(var)  _Generic((var), int8_t:   1, default: 0)
#define IS_I16(var) _Generic((var), int16_t:  1, default: 0)
#define IS_I32(var) _Generic((var), int32_t:  1, default: 0)
#define IS_I64(var) _Generic((var), int64_t:  1, default: 0)

#define STATIC_ASSERT_U8(var)  static_assert(IS_U8(var), "")
#define STATIC_ASSERT_U16(var) static_assert(IS_U16(var), "")
#define STATIC_ASSERT_U32(var) static_assert(IS_U32(var), "")
#define STATIC_ASSERT_U64(var) static_assert(IS_U64(var), "")
#define STATIC_ASSERT_I8(var)  static_assert(IS_I8(var), "")
#define STATIC_ASSERT_I16(var) static_assert(IS_I16(var), "")
#define STATIC_ASSERT_I32(var) static_assert(IS_I32(var), "")
#define STATIC_ASSERT_I64(var) static_assert(IS_I64(var), "")
