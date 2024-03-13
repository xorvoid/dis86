#pragma once
#include "header.h"

#define INITIAL_CAP 32

typedef struct array array_t;
struct array
{
  void * mem;
  size_t len;
  size_t cap;
  size_t elt_sz;
};

static inline array_t *array_new(size_t elt_sz)
{
  array_t *arr = calloc(1, sizeof(array_t));
  arr->mem = malloc(INITIAL_CAP * elt_sz);
  arr->len = 0;
  arr->cap = INITIAL_CAP;
  arr->elt_sz = elt_sz;
  return arr;
}

static inline void array_delete(array_t *arr)
{
  free(arr->mem);
  free(arr);
}

static inline size_t array_len(array_t *arr)
{
  return arr->len;
}

static inline void *array_at(array_t *arr, size_t idx)
{
  assert(idx < arr->len);
  return arr->mem + idx * arr->elt_sz;
}

static inline void *array_append_dst(array_t *arr)
{
  if (arr->len == arr->cap) {
    arr->cap *= 2;
    arr->mem = realloc(arr->mem, arr->cap * arr->elt_sz);
  }
  arr->len++;
  return array_at(arr, arr->len-1);
}

static inline void array_append(array_t *arr, void *elt)
{
  void *p = array_append_dst(arr);
  memcpy(p, elt, arr->elt_sz);
}

static inline void *array_borrow(array_t *arr, size_t *_len)
{
  *_len = arr->len;
  return arr->mem;
}
