#include "datamap.h"
#include <stdbool.h>

#define INITIAL_CAP 32

static inline bool is_white(char c)
{
  return c == ' ' || c == '\t' || c == '\n';
}

typedef struct parser parser_t;
struct parser
{
  const char *line;
  size_t len;
  size_t idx;
};

static inline void parser_skip(parser_t *p);

static void parser_init(parser_t *p, const char *line, size_t len)
{
  p->line = line;
  p->len = len;
  p->idx = 0;
  parser_skip(p);
}

static inline bool parser_is_end(parser_t *p)
{
  return p->idx == p->len;
}

static inline void parser_skip(parser_t *p)
{
  bool in_comment = false;
  while (p->idx < p->len) {
    char c = p->line[p->idx];
    if (c == '#') {
      in_comment = true;
    }
    if (!in_comment && !is_white(c)) {
      break;
    }
    p->idx++;
  }
}

static inline void parse_tok(parser_t *p, const char **tok, size_t *tok_len)
{
  parser_skip(p);

  // consume non-white tok
  *tok = &p->line[p->idx];
  while (p->idx < p->len && !is_white(p->line[p->idx])) p->idx++;
  *tok_len = &p->line[p->idx] - *tok;
}

static inline char *parse_name(parser_t *p)
{
  const char *tok;
  size_t tok_len;
  parse_tok(p, &tok, &tok_len);
  if (tok_len == 0) FAIL("Reached end while parsing name in line: '%s'", p->line);

  char *s = malloc(tok_len+1);
  memcpy(s, tok, tok_len);
  s[tok_len] = '\0';
  return s;
}

static inline int parse_type(parser_t *p)
{
  const char *tok;
  size_t tok_len;
  parse_tok(p, &tok, &tok_len);
  if (tok_len == 0) FAIL("Reached end while parsing type in line: '%s'", p->line);

  if (tok_len == 2 && 0 == memcmp(tok, "u8", 2))  return DATAMAP_TYPE_U8;
  else if (tok_len == 3 && 0 == memcmp(tok, "u16", 3)) return DATAMAP_TYPE_U16;
  else FAIL("Unknown type '%.*s' in line: '%s'", (int)tok_len, tok, p->line);
}

static inline u16 parse_addr(parser_t *p)
{
  const char *tok;
  size_t tok_len;
  parse_tok(p, &tok, &tok_len);
  if (tok_len == 0) FAIL("Reached end while parsing type in line: '%s'", p->line);

  if (tok_len < 2 || tok[0] != '0' || tok[1] != 'x') {
    FAIL("Expected hex number for addr in line: '%s'", p->line);
  }
  if (tok_len > 6) {
    FAIL("Hex number too long for addr in line: '%s'", p->line);
  }

  u16 num = 0;
  for (size_t i = 2; i < tok_len; i++) {
    char c = tok[i];
    if ('0' <= c && c <= '9') num = num*16 + (c-'0');
    else if ('a' <= c && c <= 'z') num = num*16 + (c-'a'+10);
    else if ('A' <= c && c <= 'Z') num = num*16 + (c-'A'+10);
    else FAIL("Invalid hex number for addr in line: '%s'", p->line);
  }

  return num;
}

static inline void parse_end(parser_t *p)
{
  const char *tok;
  size_t tok_len;
  parse_tok(p, &tok, &tok_len);
  if (tok_len != 0) FAIL("Expected end of line in line: '%s'", p->line);
}

static inline datamap_entry_t *entry_begin(datamap_t *d, size_t *_cap)
{
  size_t cap = *_cap;
  if (d->n_entries+1 > cap) {
    cap *= 2;
    d->entries = realloc(d->entries, cap * sizeof(datamap_entry_t));
    *_cap = cap;
  }
  return &d->entries[d->n_entries];
}

static inline void entry_commit(datamap_t *d, datamap_entry_t *ent)
{
  assert(ent == &d->entries[d->n_entries]);
  d->n_entries++;
}

datamap_t *datamap_load_from_mem(const char *str, size_t n)
{
  size_t cap = INITIAL_CAP;

  datamap_t *d = calloc(1, sizeof(datamap_t));
  d->entries = malloc(cap * sizeof(datamap_entry_t));
  d->n_entries = 0;


  const char *line = str;
  const char *line_end = str;
  while (*line) {
    // Find next line
    while (*line_end && *line_end != '\n') line_end++;

    // Init parser
    parser_t p[1];
    parser_init(p, line, line_end - line);

    // Advance the line
    if (*line_end) line_end++;
    line = line_end;

    // Allow and ignore empty lines
    if (parser_is_end(p)) {
      continue;
    }

    // Parse the entry
    datamap_entry_t *ent = entry_begin(d, &cap);
    ent->name = parse_name(p);
    ent->type = parse_type(p);
    ent->addr = parse_addr(p);
    parse_end(p);
    entry_commit(d, ent);
  }

  return d;
}

datamap_t *datamap_load_from_file(const char *filename)
{
  size_t mem_sz = 0;
  char * mem = read_file(filename, &mem_sz);

  datamap_t *d = datamap_load_from_mem(mem, mem_sz);

  free(mem);
  return d;
}

void datamap_delete(datamap_t *d)
{
  for (size_t i = 0; i < d->n_entries; i++) {
    datamap_entry_t *ent = &d->entries[i];
    free(ent->name);
  }
  free(d);
}
