#include "dis86.h"
#include "header.h"

int main(int argc, char *argv[])
{
  if (argc != 2) {
    fprintf(stderr, "usage: %s <binary>\n", argv[0]);
    return 1;
  }
  const char *filename = argv[1];

  size_t mem_sz = 0;
  char *mem = read_file(filename, &mem_sz);

  dis86_t *d = dis86_new(mem, mem_sz);
  if (!d) FAIL("Failed to allocate dis86 instance");
  free(mem);

  while (1) {
    size_t addr, n_bytes;
    dis86_instr_t *ins = dis86_next(d, &addr, &n_bytes);
    if (!ins) break;
    dis86_print_intel_syntax(d, ins, addr, n_bytes);
  }

  return 0;
}
