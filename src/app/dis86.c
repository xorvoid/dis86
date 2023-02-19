#include "dis86.h"
#include "dis86_private.h"
#include "segoff.h"

static dis86_t *dis_exit = NULL;
static void on_fail()
{
  if (!dis_exit) return;
  binary_dump(dis_exit->b);
}

#include "dis.c"
#include "decomp.c"

int main(int argc, char *argv[])
{
  atexit(on_fail);

  if (argc != 5) {
    fprintf(stderr, "usage: %s <mode> <binary> <start-seg-off> <end-seg-off>\n", argv[0]);
    fprintf(stderr, "\n");
    fprintf(stderr, "MODES:\n");
    fprintf(stderr, "  dis       disassemble the binary and emit intel syntax\n");
    fprintf(stderr, "  decomp    decompile the binary\n");
    return 1;
  }
  const char *mode = argv[1];
  const char *filename = argv[2];
  segoff_t start = parse_segoff(argv[3]);
  segoff_t end = parse_segoff(argv[4]);

  if (0) {}
  else if(0 == strcmp(mode, "dis"))     return exec_mode_dis(filename, start, end);
  else if(0 == strcmp(mode, "decomp"))  return exec_mode_decomp(filename, start, end);
  else FAIL("Unknown mode: '%s'", mode);
}
