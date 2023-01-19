#include "header.h"

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
#include "test_cases.c"
};

int main()
{
  for (size_t i = 0; i < ARRAY_SIZE(TESTS); i++) {
    test_t *t = &TESTS[i];
    printf("Testing: '%s'\n", t->code);
  }
  return 0;
}
