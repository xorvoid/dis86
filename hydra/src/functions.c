#include "internal.h"
#include <dlfcn.h>

// Must be configured at init-time
static const hydra_function_metadata_t *md = NULL;

void hydra_function_metadata_init(void)
{
  const hydra_function_metadata_t *(*user_fn)(void) = NULL;
  *(void**)&user_fn = dlsym(RTLD_DEFAULT, "hydra_user_functions");
  if (!user_fn) FAIL("Failed to find user metadata: hydra_user_functions()");
  md = user_fn();
}


const hydra_function_def_t * hydra_function_find(const char *name)
{
  for (size_t i = 0; i < md->n_defs; i++) {
    if (0 == strcmp(name, md->defs[i].name)) {
      return &md->defs[i];
    }
  }
  return NULL;
}

const char *hydra_function_name(addr_t s)
{
 u32 addr = addr_abs(s);
  for (size_t i = 0; i < md->n_defs; i++) {
    const hydra_function_def_t *f = &md->defs[i];
    if (addr_is_overlay(f->addr)) continue; // ignore overlays
    if (addr == addr_abs(f->addr)) {
      return f->name;
    }
  }
  return NULL;
}

bool hydra_function_addr(const char *name, addr_t *_out)
{
  for (size_t i = 0; i < md->n_defs; i++) {
    const hydra_function_def_t *f = &md->defs[i];
    if (0 == strcmp(name, f->name)) {
      if (_out) *_out = f->addr;
      return true;
    }
  }
  return false;
}
