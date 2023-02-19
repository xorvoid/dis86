#include "datamap.h"
#include <stdbool.h>

static inline bool is_white(char c)
{
  return c == ' ' || c == '\t' || c == '\n';
}

typedef struct parser parser_t;
struct parser
{
  const char *line;
  const char *cur;
};

static inline void parser_skip(parser_t *p)
{
  bool in_comment = false;
  while (*p->cur) {
    if (*p->cur == '#') {
      in_comment = true;
    }
    if (!in_comment && !is_white(*p->cur)) {
      break;
    }
    p->cur++;
  }
}

static void parser_init(parser_t *p, const char *line)
{
  p->line = line;
  p->cur = line;
  parser_skip(p);
}

static inline bool parser_is_end(parser_t *p)
{
  return *p->cur == '\0';
}

static inline void parse_tok(parser_t *p, const char **tok, size_t *tok_len)
{
  parser_skip(p);

  // consume non-white tok
  *tok = p->cur;
  while (*p->cur && !is_white(*p->cur)) p->cur++;
  *tok_len = p->cur - *tok;
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

datamap_t *datamap_load(const char *filename)
{
  size_t cap = 32;

  datamap_t *d = calloc(1, sizeof(datamap_t));
  d->entries = malloc(cap * sizeof(datamap_entry_t));
  d->n_entries = 0;

  FILE *fp = fopen(filename, "r");
  if (!fp) FAIL("Failed to open file: '%s'", filename);

  char *line = NULL;
  size_t len = 0;

  while (1) {
    ssize_t nread = getline(&line, &len, fp);
    if (nread == -1) break;

    size_t s_len = strlen(line);
    if (line[s_len-1] == '\n') line[s_len-1] = '\0';

    parser_t p[1];
    parser_init(p, line);

    if (parser_is_end(p)) {
      // allow and ignore empty lines
      continue;
    }

    datamap_entry_t *ent = entry_begin(d, &cap);
    ent->name = parse_name(p);
    ent->type = parse_type(p);
    ent->addr = parse_addr(p);
    parse_end(p);
    entry_commit(d, ent);
  }

  free(line);
  fclose(fp);

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
