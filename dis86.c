#include "dis86_private.h"

dis86_t *dis86_new(char *mem, size_t mem_sz)
{
  dis86_t *d = calloc(1, sizeof(dis86_t));
  bin_init(d->b, mem, mem_sz);
  return d;
}

void dis86_delete(dis86_t *d)
{
  free(d);
}
