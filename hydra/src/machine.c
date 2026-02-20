#include "internal.h"

void hydra_impl_unknown(const char *func, int line)
{
  fprintf(stderr, "FAIL: UNKNOWN INSTRUCTION: UNIMPL AT %s:%d\n", func, line);
  abort();                                                              \
}

#define U32_MAKE(upper, lower) ((u32)(upper) << 16 | (u32)(lower))

u32 hydra_impl_call_far(u16 seg, u16 off)
{
  // Grab the current execution context
  u16 exec_id = 0;
  hydra_exec_ctx_t *exec = execution_context_get(&exec_id);

  // Save a return reference to this executor on the 8086 stack
  hydra_machine_t *m = &exec->machine;
  u32 addr_off = (u32)m->registers->ss * 16 + (u16)(m->registers->sp - 2);
  u32 addr_seg = (u32)m->registers->ss * 16 + (u16)(m->registers->sp - 4);
  m->hardware->mem_write16(m->hardware->ctx, addr_off, 0xffff);
  m->hardware->mem_write16(m->hardware->ctx, addr_seg, exec_id);
  m->registers->sp -= 4;

  // Save the real CS:IP
  exec->saved_cs = m->registers->cs;
  exec->saved_ip = m->registers->ip;
  exec->maybe_reloc = 0;

  // Update the cpu to the call site
  exec->result.type = HYDRA_RESULT_TYPE_CALL;
  exec->result.new_cs = seg;
  exec->result.new_ip = off;

  // Wake up the main thread
  pthread_cond_signal(exec->cond_main);

  // Wait for main to resume this function
  pthread_cond_wait(exec->cond_child, exec->mutex);

  // Restore the execution context (another function may have modified it)
  execution_context_set(exec);

  // Restore the real CS
  m->registers->cs = exec->saved_cs;
  m->registers->ip = exec->saved_ip;

  // Return into the hook impl

  return U32_MAKE(m->registers->dx, m->registers->ax);
}

u32 hydra_impl_call_near_off(u16 off, int maybe_reloc)
{
  // Grab the current execution context
  u16 exec_id = 0;
  hydra_exec_ctx_t *exec = execution_context_get(&exec_id);

  // We only support a very small number of near call exec ids
  assert(exec_id <= 255);

  // Save a return reference to this executor on the 8086 stack
  hydra_machine_t *m = &exec->machine;
  u32 addr = (u32)m->registers->ss * 16 + m->registers->sp;
  m->hardware->mem_write16(m->hardware->ctx, addr - 2, 0xff00 + exec_id);
  //printf("exec: 0x%x\n", 0xff00 + exec_id);
  m->registers->sp -= 2;

  // Save the real IP
  exec->saved_cs = m->registers->cs;
  exec->saved_ip = m->registers->ip;
  exec->maybe_reloc = maybe_reloc;

  // Update the cpu to the call site
  assert(m->registers->cs >= CODE_START_SEG);
  exec->result.type = HYDRA_RESULT_TYPE_CALL_NEAR;
  exec->result.new_ip = off;

  // Wake up the main thread
  pthread_cond_signal(exec->cond_main);

  // Wait for main to resume this function
  pthread_cond_wait(exec->cond_child, exec->mutex);

  // Sanity
  //printf("after sp: 0x%x\n", m->registers->sp);
  if (!maybe_reloc) {
    assert(m->registers->cs == exec->saved_cs);
  }

  // Restore the exec context (another function may have modified it)
  execution_context_set(exec);

  // Restore the real IP
  //m->registers->cs = exec->saved_cs;
  m->registers->ip = exec->saved_ip;

  // Return into the hook impl

  return U32_MAKE(m->registers->dx, m->registers->ax);
}


u32 hydra_impl_call_near_abs(u16 abs_off)
{
  // Grab the current execution context
  u16 exec_id = 0;
  hydra_exec_ctx_t *exec = execution_context_get(&exec_id);
  hydra_machine_t *m = &exec->machine;

  return hydra_impl_call_near_off(abs_off - 16*(m->registers->cs - CODE_START_SEG), 0);
}

u32 hydra_impl_call_far_cs(u16 cs_reg_value, u16 off)
{
  assert(cs_reg_value >= CODE_START_SEG);
  return hydra_impl_call_far(cs_reg_value - CODE_START_SEG, off);
}

u32 hydra_impl_call_far_indirect(u32 addr)
{
  u16 seg = addr>>16;
  u16 off = addr;
  assert(seg >= CODE_START_SEG);
  return hydra_impl_call_far(seg - CODE_START_SEG, off);
}

// FIXME
/* void hydra_impl_call_func(const char *name) */
/* { */
/*   addr_t addr = {}; */
/*   if (!function_addr(name, &addr)) FAIL("Failed to find function to call: %s", name); */
/*   hydra_impl_call_far(addr.seg, addr.off); */
/* } */

void hydra_impl_raw_code(u8 *code, size_t code_sz)
{
  /* So this is a fun trick. Occasionally, we need to execute actual 8086
     code.. for example "sti".

     To do this, we write the instructions we want to the start of the code segment
     and then execute it as a function call. We save the previous code
     and restore it after the execution
  */

#define MAX_RAW_CODE 128
  assert(code_sz <= MAX_RAW_CODE);

  hydra_exec_ctx_t *exec = execution_context_get(NULL);
  hydra_machine_t *m = &exec->machine;

  u8   code_saved[MAX_RAW_CODE];
  u16  code_seg  = CODE_START_SEG;
  u32  code_addr = (u32)code_seg << 4;
  u8 * code_ptr  = m->hardware->mem_hostaddr(m->hardware->ctx, code_addr);

  // save
  memcpy(code_saved, code_ptr, MAX_RAW_CODE);

  // write new
  memcpy(code_ptr, code, code_sz);

  // call
  hydra_impl_call_far(0, 0);

  // restore
  memcpy(code_ptr, code_saved, MAX_RAW_CODE);
}

void hydra_impl_nop(void)
{
  // NO-OP
  u8 code[] = {0x90};
  hydra_impl_raw_code(code, 1);
}

// cld
void hydra_impl_cld(void)
{
  u8 machine_code[] = {0xfc, 0xcb}; /* cld; retf; */
  hydra_impl_raw_code(machine_code, ARRAY_SIZE(machine_code));
}

// std
void hydra_impl_std(void)
{
  u8 machine_code[] = {0xfd, 0xcb}; /* std; retf; */
  hydra_impl_raw_code(machine_code, ARRAY_SIZE(machine_code));
}

// cli
void hydra_impl_cli(void)
{
  u8 machine_code[] = {0xfa, 0xcb}; /* cli; retf; */
  hydra_impl_raw_code(machine_code, ARRAY_SIZE(machine_code));
}

// sti
void hydra_impl_sti(void)
{
  u8 machine_code[] = {0xfb, 0xcb}; /* sti; retf; */
  hydra_impl_raw_code(machine_code, ARRAY_SIZE(machine_code));
}

// inb al,PORT
u8 hydra_impl_inb(u16 port)
{
  hydra_exec_ctx_t *exec = execution_context_get(NULL);
  hydra_machine_t *m = &exec->machine;

  u8 save_ax = m->registers->ax;
  u8 save_dx = m->registers->dx;
  m->registers->dx = port;

  /* in al,dx; retf; */
  u8 machine_code[] = {0xec, 0xcb};
  hydra_impl_raw_code(machine_code, ARRAY_SIZE(machine_code));

  u8 ret = (u8)m->registers->ax;
  m->registers->ax = save_ax;
  m->registers->dx = save_dx;
  return ret;
}

// outb PORT, al
void hydra_impl_outb(u16 port, u8 val)
{
  hydra_exec_ctx_t *exec = execution_context_get(NULL);
  hydra_machine_t *m = &exec->machine;

  u8 save_ax = m->registers->ax;
  u8 save_dx = m->registers->dx;
  m->registers->ax = (u16)val;
  m->registers->dx = port;

  /* out dx,al; retf; */
  u8 machine_code[] = {0xee, 0xcb};
  hydra_impl_raw_code(machine_code, ARRAY_SIZE(machine_code));

  m->registers->ax = save_ax;
  m->registers->dx = save_dx;
}

void hydra_impl_int(u8 num)
{
  /* int <num>; retf; */
  u8 machine_code[] = {0xcd, num, 0xcb};
  hydra_impl_raw_code(machine_code, ARRAY_SIZE(machine_code));
}

uint32_t hydra_impl_ptr_to_flataddr(hydra_machine_t *m, void *_ptr)
{
  uint8_t * ptr = (uint8_t*)_ptr;

  uint32_t min_addr = 0x8000;
  uint32_t max_addr = 0x9f000;
  uint8_t * min_ptr = m->hardware->mem_hostaddr(m->hardware->ctx, min_addr);
  uint8_t * max_ptr = min_ptr - min_addr + max_addr;
  if (!(min_ptr <= ptr && ptr < max_ptr)) FAIL("Invalid pointer in PTR_TO_ADDR: %p\n", ptr);

  return min_addr + (ptr - min_ptr);
}

addr_t hydra_impl_ptr_to_addr(hydra_machine_t *m, void *ptr)
{
  uint32_t addr = hydra_impl_ptr_to_flataddr(m, ptr);
  assert(addr <= 1<<20);

  addr_t ret = ADDR_MAKE(addr>>4, addr&15);
  return ret;
}

uint16_t hydra_impl_ptr_to_off(hydra_machine_t *m, void *ptr, uint16_t seg)
{
  uint32_t addr = hydra_impl_ptr_to_flataddr(m, ptr);
  assert(addr <= 1<<20);

  uint32_t seg_start = (uint32_t)seg * 16;
  uint32_t seg_end   = seg_start + (1<<16);

  if (!(seg_start <= addr && addr < seg_end)) {
    FAIL("Address 0x%08x is not in segment 0x%04x", addr, seg_start);
  }

  return (uint16_t)(addr - seg_start);
}

uint32_t hydra_impl_ptr_to_32(hydra_machine_t *m, void *ptr)
{
  addr_t s = hydra_impl_ptr_to_addr(m, ptr);
  return (uint32_t)addr_seg(s) << 16 | addr_off(s);
}
