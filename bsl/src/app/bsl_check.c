#include "bsl/bsl.h"
#include <stdio.h>
#include <stdlib.h>

#define FAIL(...) do { fprintf(stderr, "FAIL: "); fprintf(stderr, __VA_ARGS__); fprintf(stderr, "\n"); exit(42); } while(0)

static inline char *file_read(const char *name, size_t *_len)
{
  FILE *fp = fopen(name, "r");
  if (!fp) FAIL("Failed to open file: %s", name);

  fseek(fp, 0, SEEK_END);
  size_t len = ftell(fp);
  fseek(fp, 0, SEEK_SET);

  char *mem = malloc(len);
  if (!mem) FAIL("Failed to allocate file buffer");

  size_t n = fread(mem, 1, len, fp);
  if (n != len) FAIL("Failed to read everything from file: %s", name);

  fclose(fp);

  *_len = len;
  return mem;
}


int main(int argc, char *argv[])
{
  if (argc != 2) {
    fprintf(stderr, "usage: %s <filename>\n", argv[0]);
    return 1;
  }
  const char *filename = argv[1];

  size_t data_len;
  char *data = file_read(filename, &data_len);

  bsl_t *b = bsl_parse_new(data, data_len, NULL);

  int ret = 0;
  if (!b) {
    fprintf(stderr, "Failed to parse bsl from '%s'\n", filename);
    ret = 1;
  }

  if (b) bsl_delete(b);
  free(data);

  return ret;
}
