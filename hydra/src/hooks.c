#include "internal.h"

static hydra_hook_t hooks[MAX_HOOKS];
static size_t num_hooks = 0;

void hydra_hook_register(hydra_hook_t ent)
{
  assert(num_hooks < ARRAY_SIZE(hooks));
  hooks[num_hooks++] = ent;
}

bool hydra_hook_entry(addr_t addr)
{
  // FIXME: ADD TO THE USER CONFIG
  // navigator
  u16 main_seg = 0x02e0 + CODE_START_SEG;
  u16 main_off = 0x000f;
  return addr_equal(addr, ADDR_MAKE(main_seg, main_off));
}

hydra_hook_t * hydra_hook_find(addr_t addr)
{
  // Subtract off CODE_START_SEG
  assert(!addr_is_overlay(addr));
  u16 seg = addr_seg(addr);
  u16 off = addr_off(addr);
  if (seg < CODE_START_SEG) return NULL;
  addr = ADDR_MAKE(seg - CODE_START_SEG, off);

  // Remap to an overlay address, if possible
  addr = hydra_overlay_segment_remap_from_physical(addr);

  for (size_t i = 0; i < num_hooks; i++) {
    hydra_hook_t *ent = &hooks[i];
    if (addr_equal(addr, ent->addr)) {
      return ent;
    }
  }
  return NULL;
}

void hydra_impl_register_addr(hydra_result_t (*func)(hydra_machine_t *m), u16 seg, u16 off, int flags)
{
  hydra_hook_t ent = {NULL, func, ADDR_MAKE(seg, off), flags};
  hydra_hook_register(ent);
}

void hydra_impl_register(const char *name, hydra_result_t (*func)(hydra_machine_t *m), int flags)
{
  const hydra_function_def_t *def = hydra_function_find(name);
  if (!def) FAIL("Cannot find function '%s' to register", name);

  hydra_hook_t ent = {name, func, def->addr, flags};
  hydra_hook_register(ent);
}

hydra_result_t hydra_impl_dead(hydra_machine_t *m)
{
  hydra_callstack_dump();
  hydra_cpu_dump(m->registers);
  FAIL("DEADCODE NOT SO DEAD at CS: %x IP: %x", m->registers->cs, m->registers->ip);
  return HYDRA_RESULT_RESUME();
}
