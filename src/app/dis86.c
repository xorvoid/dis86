#include "dis86.h"
#include "dis86_private.h"

static dis86_t *dis_exit = NULL;
static void on_fail()
{
  if (!dis_exit) return;
  binary_dump(dis_exit->b);
}

typedef struct segoff segoff_t;
struct segoff
{
  u16 seg;
  u16 off;
};

static u16 parse_hex_u16(const char *s, size_t len)
{
  if (len > 4) FAIL("Hex string too long to fit in u16");

  u16 ret = 0;
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

static segoff_t parse_segoff(const char *s)
{
  const char *end = s + strlen(s);

  const char *colon = strchr(s, ':');
  if (!colon) FAIL("Invalid segoff: '%s'", s);

  segoff_t ret;
  ret.seg = parse_hex_u16(s, colon-s);
  ret.off = parse_hex_u16(colon+1, end-(colon+1));
  return ret;
}

static size_t segoff_abs(segoff_t s)
{
  return (size_t)s.seg * 16 + (size_t)s.off;
}

int main(int argc, char *argv[])
{
  atexit(on_fail);

  if (argc != 4) {
    fprintf(stderr, "usage: %s <binary> <start-seg-off> <end-seg-off>\n", argv[0]);
    return 1;
  }
  const char *filename = argv[1];
  segoff_t start = parse_segoff(argv[2]);
  segoff_t end = parse_segoff(argv[3]);

  size_t start_idx = segoff_abs(start);
  size_t end_idx = segoff_abs(end);

  size_t mem_sz = 0;
  char *mem = read_file(filename, &mem_sz);

  char *region = &mem[start_idx];
  size_t region_sz = end_idx - start_idx;

  dis86_t *d = dis86_new(start_idx, region, region_sz);
  if (!d) FAIL("Failed to allocate dis86 instance");
  free(mem);
  dis_exit = d;

  char *s;
  while (1) {
    dis86_instr_t *ins = dis86_next(d);
    if (!ins) break;

    s = dis86_print_intel_syntax(d, ins, true);
    printf("%s\n", s);
    free(s);
  }

  dis_exit = NULL;
  dis86_delete(d);
  return 0;
}
