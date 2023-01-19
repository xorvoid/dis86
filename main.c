#include "dis86.h"
#include "dis86_private.h"

static dis86_t *dis_exit = NULL;
static void on_fail()
{
  if (!dis_exit) return;
  bin_dump(dis_exit->b);
}

int main(int argc, char *argv[])
{
  atexit(on_fail);

  if (argc != 2) {
    fprintf(stderr, "usage: %s <binary>\n", argv[0]);
    return 1;
  }
  const char *filename = argv[1];

  size_t mem_sz = 0;
  char *mem = read_file(filename, &mem_sz);

  dis86_t *d = dis86_new(0, mem, mem_sz);
  if (!d) FAIL("Failed to allocate dis86 instance");
  free(mem);
  dis_exit = d;

  while (1) {
    size_t addr, n_bytes;
    dis86_instr_t *ins = dis86_next(d, &addr, &n_bytes);
    if (!ins) break;

    char *s = dis86_print_intel_syntax(d, ins, addr, n_bytes, true);
    printf("%s\n", s);
    free(s);
  }

  dis_exit = NULL;
  dis86_delete(d);
  return 0;
}
