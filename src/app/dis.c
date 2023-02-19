
static int exec_mode_dis(const char *filename, segoff_t start, segoff_t end)
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
