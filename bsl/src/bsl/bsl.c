#include "bsl.h"
#include <assert.h>
#include <stdalign.h>
#include <stdbool.h>
#include <string.h>

#define HAX_FAIL(...) do { fprintf(stderr, "FAIL (%s:%d): ", __FUNCTION__, __LINE__); fprintf(stderr, __VA_ARGS__); fprintf(stderr, "\n"); abort(); } while(0)

typedef struct bsl_parser  bsl_parser_t;
typedef struct bsl_keyval  bsl_keyval_t;
typedef struct bsl_node    bsl_node_t;

enum {
  TOKEN_EOF = 256,
  TOKEN_STR,
};

struct bsl_parser
{
  const char * buf;
  size_t       sz;
  size_t       idx;

  int          tok_type;
  const char * tok_buf;
  size_t       tok_len;
};

static void bsl_parser_init(bsl_parser_t *p, const char *buf, size_t sz)
{
  p->buf = buf;
  p->sz = sz;
  p->idx = 0;
}

static inline bool is_white(char c)   { return c == ' ' || c == '\t' || c == '\n'; }
static inline bool is_visible(char c) { return 33 <= c && c <= 126; }

static void bsl_parser_skip_white(bsl_parser_t *p)
{
  for (; p->idx < p->sz; p->idx++) {
    char c = p->buf[p->idx];
    if (!is_white(c)) break;
  }
}

static void bsl_parser_advance(bsl_parser_t *p)
{
  if (p->idx < p->sz) p->idx++;
}

static char bsl_parser_char(bsl_parser_t *p)
{
  return (p->idx == p->sz) ? '\0' : p->buf[p->idx];
}

static void bsl_parser_tok_next(bsl_parser_t *p)
{
  // skip all whitespace
  bsl_parser_skip_white(p);

  // token end?
  if (p->idx == p->sz) {
    p->tok_type = TOKEN_EOF;
    p->tok_len = 0;
    return;
  }

  p->tok_buf = &p->buf[p->idx];
  char c = *p->tok_buf;

  // token punctuation
  if (c == '{' || c == '}') {
    bsl_parser_advance(p);
    p->tok_type = c;
    p->tok_len = 1;
    return;
  }

  // token str (quoted)
  if (c == '"') {
    p->tok_type = TOKEN_STR;
    while (1) {
      bsl_parser_advance(p);
      c = bsl_parser_char(p);
      if (c == '\0') HAX_FAIL("REACHED EOF WHILE INSIDE A QUOTED STRING");
      if (c == '"') break; // Found!!
    }

    // Advance past the quote
    bsl_parser_advance(p);

    // Remove the quotes from the output
    p->tok_buf++;  // skip starting '"'
    p->tok_len = &p->buf[p->idx] - p->tok_buf - 1; // skip ending '"'
    return;
  }

  // token str
  if (is_visible(c)) {
    p->tok_type = TOKEN_STR;
    while (is_visible(c)) {
      bsl_parser_advance(p);
      c = bsl_parser_char(p);
    }
    p->tok_len = &p->buf[p->idx] - p->tok_buf;
    return;
  }

  HAX_FAIL("BAD TOK");
}

static char *bsl_parser_tok_str(bsl_parser_t *p)
{
  char *s = malloc(p->tok_len+1);
  memcpy(s, p->tok_buf, p->tok_len);
  s[p->tok_len] = '\0';
  return s;
}

struct bsl_keyval
{
  int    type; // BSL_TYPE_*
  char * key;
  void * val;
};

struct bsl_node
{
  bsl_keyval_t * kv_arr;
  size_t         kv_len;
  size_t         kv_cap;
  int            toplevel;
};

static bsl_node_t * bsl_node_new(void)
{
  bsl_node_t *n = malloc(sizeof(bsl_node_t));
  if (!n) return NULL;

  n->kv_len = 0;
  n->kv_cap = 8;
  n->kv_arr = malloc(n->kv_cap * sizeof(bsl_keyval_t));
  n->toplevel = 0;

  if (!n->kv_arr) {
    free(n);
    return NULL;
  }

  return n;
}

static void bsl_node_delete(bsl_node_t *n)
{
  for (size_t i = 0; i < n->kv_len; i++) {
    bsl_keyval_t *kv = &n->kv_arr[i];
    if (kv->type == BSL_TYPE_STR) {
      free((char*)kv->val);
    } else if (kv->type == BSL_TYPE_NODE) {
      bsl_node_delete((bsl_node_t*)kv->val);
    }
  }
  free(n->kv_arr);
  free(n);
}

static void bsl_node_append(bsl_node_t *n, bsl_keyval_t kv /* value moved into this node */)
{
  if (n->kv_len == n->kv_cap) { // realloc?
    n->kv_cap *= 2;
    n->kv_arr = realloc(n->kv_arr, n->kv_cap * sizeof(bsl_keyval_t));
    if (!n->kv_arr) HAX_FAIL("CANNOT REALLOC");
  }

  assert(n->kv_len < n->kv_cap);
  n->kv_arr[n->kv_len++] = kv;
}

// forward decl needed for mutually recursively dependent parser
static bsl_node_t *parse_node(bsl_parser_t *p);

// value = str | "{" node "}"
static void *parse_value(bsl_parser_t *p, int *out_type)
{
  if (p->tok_type == TOKEN_STR) {
    char *val = bsl_parser_tok_str(p);
    bsl_parser_tok_next(p);
    *out_type = BSL_TYPE_STR;
    return val;
  }

  else if (p->tok_type == '{') {
    bsl_parser_tok_next(p);
    bsl_node_t *node = parse_node(p);
    if (p->tok_type != '}') HAX_FAIL("Expected closing '}'");
    bsl_parser_tok_next(p);
    *out_type = BSL_TYPE_NODE;
    return node;
  }

  else {
    HAX_FAIL("Expected value to start with either a string or '{', got [0x%x]", p->tok_type);
  }

}

// keyval = str value
static bool parse_keyval(bsl_parser_t *p, bsl_keyval_t *out_kv)
{
  if (p->tok_type != TOKEN_STR) return false;
  char *key = bsl_parser_tok_str(p);
  bsl_parser_tok_next(p);

  int type;
  void *val = parse_value(p, &type);

  out_kv->type = type;
  out_kv->key  = key;
  out_kv->val  = val;

  return true;
}

// node = keyval*
static bsl_node_t *parse_node(bsl_parser_t *p)
{
  bsl_node_t * node = bsl_node_new();

  while (1) {
    bsl_keyval_t kv;
    if (!parse_keyval(p, &kv)) break;
    bsl_node_append(node, kv);
  }

  return node;
}

bsl_t * bsl_parse_new(const char *buf, size_t sz, int *opt_err)
{
  bsl_parser_t p[1];
  bsl_parser_init(p, buf, sz);
  bsl_parser_tok_next(p);

  bsl_node_t * node = parse_node(p);
  if (p->tok_type != TOKEN_EOF) HAX_FAIL("EXPECTED EOF");

  if (opt_err) *opt_err = BSL_SUCCESS;
  node->toplevel = 1;
  return (bsl_t*)node;
}

void bsl_delete(bsl_t *bsl)
{
  bsl_node_t *node = (bsl_node_t*)bsl;
  if (!node->toplevel) {
    fprintf(stderr, "ERR: FATAL CODING BUG DETECTED. INVALID TO DELETE INTERNAL NODES.");
    abort();
  }

  bsl_node_delete(node);
}

static void * node_get(bsl_node_t *node, const char *key, size_t key_len, int *opt_type)
{
  for (size_t i = 0; i < node->kv_len; i++) {
    bsl_keyval_t *kv = &node->kv_arr[i];

    if (strlen(kv->key) != key_len) continue;
    if (0 != memcmp(kv->key, key, key_len)) continue;

    // Found!
    if (opt_type) *opt_type = kv->type;
    return kv->val;
  }
  return NULL;
}

void * bsl_get_generic(bsl_t *bsl, const char *key, int *opt_type)
{
  bsl_node_t *node = (bsl_node_t*)bsl;

  if (!key || !*key) return NULL;

  const char * ptr = key;
  while (1) {
    const char * end = ptr;
    while (*end && *end != '.') end++;
    size_t len = end - ptr;

    int type = -1;
    void *val = node_get(node, ptr, len, &type);
    if (!val) return NULL; // Not Found

    if (*end == '\0') {
      if (opt_type) *opt_type = type;
      return val;
    }

    if (type != BSL_TYPE_NODE) return NULL; // Not a node type

    node = (bsl_node_t*)val;
    ptr = end+1;
  }
}

const char * bsl_get_str(bsl_t *bsl, const char *key)
{
  int type = -1;
  const char * val = bsl_get_generic(bsl, key, &type);
  if (!val || type != BSL_TYPE_STR) return NULL;
  return val;
}

bsl_t * bsl_get_node(bsl_t *bsl, const char *key)
{
  int type = -1;
  bsl_t * val = bsl_get_generic(bsl, key, &type);
  if (!val || type != BSL_TYPE_NODE) return NULL;
  return val;
}

typedef struct iter_impl iter_impl_t;
struct __attribute__((aligned(16))) iter_impl
{
  bsl_node_t * node;
  size_t       idx;
  char         _extra[16];
};
static_assert(sizeof(iter_impl_t) == sizeof(bsl_iter_t), "");
static_assert(alignof(iter_impl_t) == alignof(bsl_iter_t), "");

void bsl_iter_begin(bsl_iter_t *_it, bsl_t *bsl)
{
  iter_impl_t *it = (iter_impl_t*)_it;
  it->node = (bsl_node_t*)bsl;
  it->idx  = 0;
}

bool bsl_iter_next(bsl_iter_t *_it, int *_type, const char **_key, void **_val)
{
  iter_impl_t * it   = (iter_impl_t*)_it;
  bsl_node_t *  node = it->node;

  if (it->idx == node->kv_len) return false;

  bsl_keyval_t *kv = &node->kv_arr[it->idx++];

  *_type = kv->type;
  *_key  = kv->key;
  *_val  = kv->val;

  return true;
}
