#include "internal.h"

static u8 *baseptr = NULL;

u8 * hydra_datasection_baseptr(void)
{
  if (!baseptr) FAIL("Data Section baseptr must be set on init with hydra_datasection_baseptr_set()");
  return baseptr;
}

void hydra_datasection_baseptr_set(u8 *ptr)
{
  baseptr = ptr;
}
