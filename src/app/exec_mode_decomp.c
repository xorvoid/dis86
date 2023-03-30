#include "exec_mode.h"
#include "dis86.h"
#include "dis86_private.h"
#include "segoff.h"
#include "cmdarg/cmdarg.h"
#include "array.h"

static dis86_t *dis_exit = NULL;
static void on_fail()
{
  if (!dis_exit) return;
  binary_dump(dis_exit->b);
}

static void print_help(FILE *f, const char *appname)
{
  fprintf(f, "usage: %s decomp OPTIONS\n", appname);
  fprintf(stderr, "\n");
  fprintf(stderr, "OPTIONS:\n");
  fprintf(stderr, "  --binary       path to binary on the filesystem (required)\n");
  fprintf(stderr, "  --start-addr   start seg:off address (required)\n");
  fprintf(stderr, "  --end-addr     end seg:off address (required)\n");
}

static bool cmdarg_segoff(int * argc, char *** argv, const char * name, segoff_t *_out)
{
  const char *s;
  if (!cmdarg_string(argc, argv, name, &s)) return false;

  *_out = parse_segoff(s);
  return true;
}

static int _legacy_exec(const char *filename, segoff_t start, segoff_t end);

int exec_mode_decomp(int argc, char *argv[])
{
  atexit(on_fail);

  const char * binary = NULL;
  segoff_t     start  = {};
  segoff_t     end    = {};

  bool found;

  found = cmdarg_string(&argc, &argv, "--binary", &binary);
  if (!found) { print_help(stderr, argv[0]); return 3; }

  found = cmdarg_segoff(&argc, &argv, "--start-addr", &start);
  if (!found) { print_help(stderr, argv[0]); return 3; }

  found = cmdarg_segoff(&argc, &argv, "--end-addr", &end);
  if (!found) { print_help(stderr, argv[0]); return 3; }

  return _legacy_exec(binary, start, end);
}

static int _legacy_exec(const char *filename, segoff_t start, segoff_t end)
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

  char func_name[256];
  sprintf(func_name, "func_%08x__%04x_%04x", (u32)start_idx, start.seg, start.off);

  const char *s = dis86_decompile(d, func_name, instr, n_instr);
  printf("%-30s\n", s);
  free((void*)s);

  dis_exit = NULL;
  array_delete(ins_arr);
  dis86_delete(d);
  return 0;
}