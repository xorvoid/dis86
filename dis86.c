#include "dis86_private.h"

dis86_t *dis86_new(size_t base_addr, char *mem, size_t mem_sz)
{
  dis86_t *d = calloc(1, sizeof(dis86_t));
  binary_init(d->b, base_addr, mem, mem_sz);
  return d;
}

void dis86_delete(dis86_t *d)
{
  free(d);
}

size_t dis86_position(dis86_t *d) { return binary_location(d->b); }
size_t dis86_baseaddr(dis86_t *d) { return binary_baseaddr(d->b); }
size_t dis86_length(dis86_t *d)   { return binary_length(d->b);   }
