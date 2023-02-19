#pragma once
#include <stdlib.h>
#include <stdarg.h>

typedef struct str str_t;
struct str
{
  char *buf;
  size_t idx;
  size_t len;
};

static inline void str_init(str_t *s)
{
  s->buf = malloc(4);
  s->idx = 0;
  s->len = 4;
}

static inline char *str_to_cstr(str_t *s)
{
  assert(s->buf); // Sanity: cannot call this twice

  // extractg underlying c-str and ensure it's null-terminated
  assert(s->idx < s->len);
  char *ret = s->buf;
  ret[s->idx] = 0;

  // invalid it
  s->buf = NULL;
  s->idx = 0;
  s->len = 0;

  return ret;
}

static inline void str_fmt(str_t *s, const char *fmt, ...)
{
  while (1) {
    va_list va;

    va_start(va, fmt);
    size_t n = vsnprintf(s->buf + s->idx, s->len - s->idx, fmt, va);
    va_end(va);

    if (s->idx + n < s->len) {
      s->idx += n;
      return;
    }

    /* resize */
    s->len *= 2;
    s->buf = realloc(s->buf, s->len);
    if (s->buf == NULL) FAIL("Failed to realloc buffer");
  }
}

static inline void str_rstrip(str_t *s)
{
  while (s->idx > 0) {
    if (s->buf[s->idx - 1] != ' ') break;
    s->idx--;
  }
}
