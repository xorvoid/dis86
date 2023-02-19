#include "header.h"
#include "dis86.h"

typedef struct binary_data binary_data_t;
struct binary_data
{
  uint8_t n_mem;
  uint8_t mem[16];
};

typedef struct test test_t;
struct test
{
  uint32_t       address;
  binary_data_t  data;
  const char *   code;
};

#define TEST(...) __VA_ARGS__,

static test_t TESTS[] = {
#include "test_decode_cases.inc"
};

static int run_test(size_t num, bool verbose)
{
  if (num >= ARRAY_SIZE(TESTS)) {
    FAIL("Invalid test number: %zu", num);
  }

  test_t *t = &TESTS[num];
  printf("TEST %zu: %-40s | ", num, t->code);
  fflush(stdout);

  dis86_t *d = dis86_new(t->address, (char*)t->data.mem, t->data.n_mem);
  if (!d) FAIL("Failed to allocate instance");

  dis86_instr_t *ins = dis86_next(d);
  if (!ins) FAIL("Failed to decode instruction");

  char *s = dis86_print_intel_syntax(d, ins, false);
  bool pass = (0 == strcmp(s, t->code));
  printf("%s", pass ? "PASS" : "FAIL");
  printf(" | '%s'\n", s);
  free(s);

  if (verbose) {
    printf("ADDRESS: 0x%08x\n", t->address);
    printf("BINARY DATA: ");
    for (size_t i = 0; i < t->data.n_mem; i++) {
      printf("%02x ", t->data.mem[i]);
    }
    printf("\n");
  }

  // Did we consume all of the input?
  assert(dis86_position(d) == dis86_baseaddr(d) + dis86_length(d));

  dis86_delete(d);
  return pass ? 0 : 1;
}

static int run_all()
{
  int ret = 0;
  for (size_t i = 0; i < ARRAY_SIZE(TESTS); i++) {
    int r = run_test(i, false);
    if (!ret) ret = r;
  }
  return ret;
}

int main(int argc, char *argv[])
{
  if (argc > 2) {
    fprintf(stderr, "usage: %s [<test-num>]\n", argv[0]);
    return 1;
  }

  if (argc >= 2) {
    return run_test(atoi(argv[1]), true);
  } else {
    return run_all();
  }
}
