#include "internal.h"
#include <dlfcn.h>

typedef struct call       call_t;
typedef struct callstack  callstack_t;

struct call
{
  addr_t src;
  addr_t dst;
};

enum {
  CALL_EVENT_NONE = 0,
  CALL_EVENT_CALL,
  CALL_EVENT_RET,
  CALL_EVENT_JMP_RET,
};

struct callstack
{
  size_t last_interrupt_count;
  addr_t last_code;

  call_t call_stack[1024];
  size_t call_idx;
  int    call_event;

  const hydra_callstack_metadata_t *md;
};
static callstack_t c[1];

void hydra_callstack_init(void)
{
  c->last_interrupt_count = 0;
  c->last_code            = ADDR_MAKE(0, 0);
  c->call_idx             = 0;
  c->call_event           = CALL_EVENT_NONE;
  c->md                   = NULL;

  // Try to load user-provided callstack configuration data
  const hydra_callstack_metadata_t *(*user_fn)(void) = NULL;
  *(void**)&user_fn = dlsym(RTLD_DEFAULT, "hydra_user_callstack");
  if (!user_fn) FAIL("Failed to find user metadata: hydra_user_callstack()");
  c->md = user_fn();
}

void hydra_callstack_trigger_enter(u16 seg, u16 off)
{
  assert(c->call_event == CALL_EVENT_NONE);
  c->last_code = ADDR_MAKE(seg, off);
  c->call_event = CALL_EVENT_CALL;
}

void hydra_callstack_dump(void)
{
  printf("Call Stack:\n");
  for (size_t i = 0; i < c->call_idx; i++) {
    call_t *call = &c->call_stack[i];
    addr_t src = addr_relative_to_segment(call->src, CODE_START_SEG);
    addr_t dst = addr_relative_to_segment(call->dst, CODE_START_SEG);
    const char *src_name = hydra_function_name(src);
    const char *dst_name = hydra_function_name(dst);
    printf("  %zu  " ADDR_FMT " => " ADDR_FMT " | %s => %s\n", i, ADDR_ARG(src), ADDR_ARG(dst), src_name, dst_name);
  }
}


static call_t *callstack_push(addr_t src, addr_t dst, size_t *_depth)
{
  if (c->call_idx >= ARRAY_SIZE(c->call_stack)) {
    hydra_callstack_dump();
    FAIL("Aborting due to callstack overflow!");
  }

  call_t *call = &c->call_stack[c->call_idx++];
  call->src = src;
  call->dst = dst;

  *_depth = c->call_idx;
  return call;
}

static call_t *callstack_pop(size_t *_depth)
{
  if (c->call_idx == 0) {
    if (ENABLE_DEBUG_CALLSTACK) printf("WARN: Call stack underflow!\n");
    *_depth = 0;
    return NULL;
  }

  size_t depth = c->call_idx;
  assert(c->call_idx > 0);
  c->call_idx = depth - 1;

  *_depth = depth;
  return &c->call_stack[depth - 1];
}

static void callstack_enter(const char *type, hydra_machine_registers_t *registers, addr_t _from)
{
  addr_t cur = ADDR_MAKE(registers->cs, registers->ip);
  size_t depth = 0;
  call_t *call = callstack_push(_from, cur, &depth);

  if (!ENABLE_DEBUG_CALLSTACK) return;

  // Adjust all the addresses for the base load segment
  addr_t from = addr_relative_to_segment(_from, CODE_START_SEG);
  addr_t to = addr_relative_to_segment(cur, CODE_START_SEG);

  // Emit!
  for (size_t i = 0; i < depth; i++) printf("  ");

  printf(ADDR_FMT " => " ADDR_FMT " | %s",
         ADDR_ARG(from), ADDR_ARG(to), type);

  const char *from_name = hydra_function_name(from);
  const char *to_name = hydra_function_name(to);
  if (from_name) {
    printf(" [%s]", from_name);
  }
  if (from_name || to_name) {
    printf(" => ");
  }
  if (to_name) {
    printf("[%s]", to_name);
  }
  printf("\n");

  /* if (depth < 4) { */
  /*   callstack_dump(); */
  /* } */
}

static void callstack_leave(const char *type, hydra_machine_registers_t *registers)
{
  addr_t cur = ADDR_MAKE(registers->cs, registers->ip);

  size_t depth = 0;
  call_t *call = callstack_pop(&depth);

  // Adjust all the addresses for the base load segment
  addr_t from = addr_relative_to_segment(c->last_code, CODE_START_SEG);
  addr_t to = addr_relative_to_segment(cur, CODE_START_SEG);
  addr_t src = ADDR_MAKE(0, 0);
  bool unexpected_return = false;
  if (call) {
    src = addr_relative_to_segment(call->src, CODE_START_SEG);
    unexpected_return = addr_difference(to, src) > 5; //(to._i._seg - src._i._seg != 0 || to._i._off - src._i._off > 5);
  }

  if (unexpected_return) {
    printf("WARN: Unexpected return location, expected " ADDR_FMT " but got " ADDR_FMT "\n",
           ADDR_ARG(to), ADDR_ARG(src));
  }

  if (!ENABLE_DEBUG_CALLSTACK) return;

  // Emit!
  for (size_t i = 0; i < depth; i++) printf("  ");

  printf(ADDR_FMT " <= " ADDR_FMT " | %s",
         ADDR_ARG(to), ADDR_ARG(from), type);

  const char *from_name = hydra_function_name(from);
  const char *to_name = hydra_function_name(to);
  if (to_name) {
    printf(" [%s]", to_name);
  }
  if (from_name || to_name) {
    printf(" <= ");
  }
  if (from_name) {
    printf(" [%s]", from_name);
  }

  if (!call) {
    printf(" (UNDERFLOW)");
  }
  if (unexpected_return) {
    printf(" (UNEXPECTED LOC: expected [" ADDR_FMT "])", ADDR_ARG(src));
  }
  printf("\n");

  /* if (depth < 4) { */
  /*   callstack_dump(); */
  /* } */
}

static u32 skip_prefixes(hydra_machine_t *m)
{
  u32 addr = m->registers->cs*16 + m->registers->ip;
  while (1) {
    u8 b = m->hardware->mem_read8(m->hardware->ctx, addr);
    if (b == 0x26 || b == 0x2e || b == 0x36 || b == 0x3e ||
        b == 0xf2 || b == 0xf3 || b == 0xf0) {
      addr++;
      continue; // this is a prefix
    }
    return addr;
  }
}

static bool is_instr_call(hydra_machine_t *m)
{
  u32 addr = skip_prefixes(m);
  u8  op   = m->hardware->mem_read8(m->hardware->ctx, addr);
  u8  op2  = (m->hardware->mem_read8(m->hardware->ctx, addr+1) >> 3) & 7; // sometimes needed, othertimes ignored

  if      (op == 0xe8)                return true;  /* Near call */
  else if (op == 0x9a)                return true;  /* Far call */
  else if (op == 0xff && op2 == 0x02) return true;  /* Near call (indirect) */
  else if (op == 0xff && op2 == 0x03) return true;  /* Far call (indirect) */

  return false;
}

static bool is_instr_ret(hydra_machine_t *m)
{
  u32 addr = skip_prefixes(m);
  u8  op   = m->hardware->mem_read8(m->hardware->ctx, addr);
  u8  op2  = (m->hardware->mem_read8(m->hardware->ctx, addr+1) >> 3) & 7; // sometimes needed, othertimes ignored

  if      (op == 0xc2) return true;  /* Near ret (with leave) */
  else if (op == 0xc3) return true;  /* Near ret */
  else if (op == 0xca) return true;  /* Far ret (with leave) */
  else if (op == 0xcb) return true;  /* Far ret */
  else if (op == 0xcf) return true;  /* Interrupt ret */

  return false;
}

static hydra_callstack_conf_t * conf_find(hydra_machine_t *m, int type)
{
  if (!c->md) return NULL;

  for (size_t i = 0; i < c->md->n_confs; i++) {
    hydra_callstack_conf_t *conf = &c->md->confs[i];
    if (conf->type != type) continue;
    if (m->registers->cs == addr_seg(conf->addr) + CODE_START_SEG && m->registers->ip == addr_off(conf->addr)) {
      return conf;
    }
  }
  return NULL;
}

static void update(hydra_machine_t *m, size_t interrupt_count)
{
  // Report interrupts
  if (interrupt_count != c->last_interrupt_count) {
    if (m->registers->cs != 0xc000 && m->registers->cs != 0xf000) {
      addr_t cur = ADDR_MAKE(m->registers->cs - CODE_START_SEG, m->registers->ip);
      addr_t stack_ptr = ADDR_MAKE(m->registers->ss, m->registers->sp);
      addr_t src = ADDR_MAKE(m->hardware->mem_read16(m->hardware->ctx, addr_abs(stack_ptr)+2), m->hardware->mem_read16(m->hardware->ctx, addr_abs(stack_ptr)));
      src = addr_relative_to_segment(src, CODE_START_SEG);
      if (ENABLE_DEBUG_CALLSTACK) printf("INTERRUPT to " ADDR_FMT "(src: " ADDR_FMT ")\n", ADDR_ARG(cur), ADDR_ARG(src));
    }
    c->last_interrupt_count = interrupt_count;
    c->call_event = CALL_EVENT_NONE; // Interrupt happened before any call/ret could exec
  }

  // Check for handler entry
  hydra_callstack_conf_t *h = conf_find(m, HYDRA_CALLSTACK_CONF_TYPE_HANDLER);
  if (h) {
    // Found! Pull the save return address from the satck and "enter"
    addr_t stack_ptr = ADDR_MAKE(m->registers->ss, m->registers->sp);
    addr_t src = ADDR_MAKE(m->hardware->mem_read16(m->hardware->ctx, addr_abs(stack_ptr)+2), m->hardware->mem_read16(m->hardware->ctx, addr_abs(stack_ptr)));
    callstack_enter(h->name, m->registers, src);
  }

  // Ignore ROM and above (DOS Reserved Mem)
  if (m->registers->cs >= 0xc000) return;

  // Ignore certain addrs
  if (conf_find(m, HYDRA_CALLSTACK_CONF_TYPE_IGNORE_ADDR)) {
    return;
  }

  // Special jump ret locations
  //hydra_callstack_conf_t *j =
  if (conf_find(m, HYDRA_CALLSTACK_CONF_TYPE_JUMPRET)) {
    c->call_event = CALL_EVENT_JMP_RET;
  }

  /* for (size_t i = 0; i < ARRAY_SIZE(ignore_addrs); i++) { */
  /*   addr_t *s = &jmp_ret[i]; */
  /*   if (m->registers->cs == s->seg + CODE_START_SEG && m->registers->ip == s->off) { */
  /*     c->call_event = CALL_EVENT_JMP_RET; */
  /*   } */
  /* } */

  // Check for instructions that manipulate the callstack
  if (is_instr_call(m))  c->call_event = CALL_EVENT_CALL;
  if (is_instr_ret(m))   c->call_event = CALL_EVENT_RET;
}

void hydra_callstack_notify(hydra_machine_t *m)
{
  // Handle call event actions that are defered from previous instr
  // because we need post-execution information
  switch (c->call_event) {
    case CALL_EVENT_NONE:  break;

    case CALL_EVENT_CALL: {
      callstack_enter("CALL", m->registers, c->last_code);
    } break;

    case CALL_EVENT_RET: {
      callstack_leave("RETURN", m->registers);
    } break;

    case CALL_EVENT_JMP_RET: {
      callstack_leave("JMP_RET", m->registers);
    } break;
}
  c->call_event = CALL_EVENT_NONE;
  c->last_code = ADDR_MAKE(m->registers->cs, m->registers->ip);
}

void hydra_callstack_track(hydra_machine_t *m, size_t interrupt_count)
{
  update(m, interrupt_count);
  c->last_code = ADDR_MAKE(m->registers->cs, m->registers->ip);
}

void hydra_callstack_ret(hydra_machine_t *m)
{
  assert(c->call_event == CALL_EVENT_NONE);
  c->call_event = CALL_EVENT_RET;
}
