#include "array.h"

static int exec_mode_decomp(const char *filename, segoff_t start, segoff_t end)
{
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

  array_t *ins_arr = array_new(sizeof(dis86_instr_t));
  while (1) {
    dis86_instr_t *ins = dis86_next(d);
    if (!ins) break;

    dis86_instr_t *ins_ptr = array_append_dst(ins_arr);
    dis86_instr_copy(ins_ptr, ins);
  }

  size_t n_instr = 0;
  dis86_instr_t *instr = (dis86_instr_t*)array_borrow(ins_arr, &n_instr);

  const char *s = dis86_decompile(d, instr, n_instr);
  printf("%-30s\n", s);
  free((void*)s);

  dis_exit = NULL;
  array_delete(ins_arr);
  dis86_delete(d);
  return 0;
}
