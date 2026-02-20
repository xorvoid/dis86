#pragma once
#include "addr.h"

#define HYDRA_MAYBE_UNUSED __attribute__((unused))

#define DONT_POP_ARGS 1
#define NEAR 2

#define HYDRA_DEFINE_CALLSTUB(name, ret, nargs, addr, flags)  \
  HYDRA_DEFINE_CALLSTUB_ ## nargs(name, ret, addr, flags)

#define HYDRA_DEFINE_CALLSTUB_IGNORE(name, ret, addr, flags)

#define HYDRA_DEFINE_CALLSTUB_0(name, ret, addr, flags) static HYDRA_MAYBE_UNUSED ret name(hydra_machine_t *m) { return (ret)hydra_impl_callstub_0(m, addr, flags); }

#define HYDRA_DEFINE_CALLSTUB_1(name, ret, addr, flags) static HYDRA_MAYBE_UNUSED ret name(hydra_machine_t *m, u16 arg1) { return (ret)hydra_impl_callstub_1(m, addr, flags, arg1); }

#define HYDRA_DEFINE_CALLSTUB_2(name, ret, addr, flags) static HYDRA_MAYBE_UNUSED ret name(hydra_machine_t *m, u16 arg1, u16 arg2) { return (ret)hydra_impl_callstub_2(m, addr, flags, arg1, arg2); }

#define HYDRA_DEFINE_CALLSTUB_3(name, ret, addr, flags) static HYDRA_MAYBE_UNUSED ret name(hydra_machine_t *m, u16 arg1, u16 arg2, u16 arg3) { return (ret)hydra_impl_callstub_3(m, addr, flags, arg1, arg2, arg3); }

#define HYDRA_DEFINE_CALLSTUB_4(name, ret, addr, flags) static HYDRA_MAYBE_UNUSED ret name(hydra_machine_t *m, u16 arg1, u16 arg2, u16 arg3, u16 arg4) { return (ret)hydra_impl_callstub_4(m, addr, flags, arg1, arg2, arg3, arg4); }

#define HYDRA_DEFINE_CALLSTUB_5(name, ret, addr, flags) static HYDRA_MAYBE_UNUSED ret name(hydra_machine_t *m, u16 arg1, u16 arg2, u16 arg3, u16 arg4, u16 arg5) { return (ret)hydra_impl_callstub_5(m, addr, flags, arg1, arg2, arg3, arg4, arg5); }

#define HYDRA_DEFINE_CALLSTUB_6(name, ret, addr, flags) static HYDRA_MAYBE_UNUSED ret name(hydra_machine_t *m, u16 arg1, u16 arg2, u16 arg3, u16 arg4, u16 arg5, u16 arg6) { return (ret)hydra_impl_callstub_6(m, addr, flags, arg1, arg2, arg3, arg4, arg5, arg6); }

#define HYDRA_DEFINE_CALLSTUB_7(name, ret, addr, flags) static HYDRA_MAYBE_UNUSED ret name(hydra_machine_t *m, u16 arg1, u16 arg2, u16 arg3, u16 arg4, u16 arg5, u16 arg6, u16 arg7) { return (ret)hydra_impl_callstub_7(m, addr, flags, arg1, arg2, arg3, arg4, arg5, arg6, arg7); }

static void hydra_impl_call(hydra_machine_t *m, addr_t addr, int flags)
{
  u16 seg;
  if (addr_is_overlay(addr)) {
    seg = hydra_overlay_segment_lookup(addr_overlay_num(addr));
  } else {
    seg = addr_seg(addr);
  }

  u16 off = addr_off(addr);

  if (flags & NEAR) CALL_NEAR_ABS(16*(u32)seg + off);
  else CALL_FAR(seg, off);
}

static HYDRA_MAYBE_UNUSED u32 hydra_impl_callstub_0(hydra_machine_t *m, addr_t addr, int flags)
{
  hydra_impl_call(m, addr, flags);
  return (u32)DX << 16 | AX;
}

static HYDRA_MAYBE_UNUSED u32 hydra_impl_callstub_1(hydra_machine_t *m, addr_t addr, int flags, u16 arg1)
{
  PUSH(arg1);
  hydra_impl_call(m, addr, flags);
  if (!(flags & DONT_POP_ARGS)) SP += 0x2;
  return (u32)DX << 16 | AX;
}

static HYDRA_MAYBE_UNUSED u32 hydra_impl_callstub_2(hydra_machine_t *m, addr_t addr, int flags, u16 arg1, u16 arg2)
{
  PUSH(arg2);
  PUSH(arg1);
  hydra_impl_call(m, addr, flags);
  if (!(flags & DONT_POP_ARGS)) SP += 0x4;
  return (u32)DX << 16 | AX;
}

static HYDRA_MAYBE_UNUSED u32 hydra_impl_callstub_3(hydra_machine_t *m, addr_t addr, int flags, u16 arg1, u16 arg2, u16 arg3)
{
  PUSH(arg3);
  PUSH(arg2);
  PUSH(arg1);
  hydra_impl_call(m, addr, flags);
  if (!(flags & DONT_POP_ARGS)) SP += 0x6;
  return (u32)DX << 16 | AX;
}

static HYDRA_MAYBE_UNUSED u32 hydra_impl_callstub_4(hydra_machine_t *m, addr_t addr, int flags, u16 arg1, u16 arg2, u16 arg3, u16 arg4)
{
  PUSH(arg4);
  PUSH(arg3);
  PUSH(arg2);
  PUSH(arg1);
  hydra_impl_call(m, addr, flags);
  if (!(flags & DONT_POP_ARGS)) SP += 0x8;
  return (u32)DX << 16 | AX;
}

static HYDRA_MAYBE_UNUSED u32 hydra_impl_callstub_5(hydra_machine_t *m, addr_t addr, int flags, u16 arg1, u16 arg2, u16 arg3, u16 arg4, u16 arg5)
{
  PUSH(arg5);
  PUSH(arg4);
  PUSH(arg3);
  PUSH(arg2);
  PUSH(arg1);
  hydra_impl_call(m, addr, flags);
  if (!(flags & DONT_POP_ARGS)) SP += 0xa;
  return (u32)DX << 16 | AX;
}

static HYDRA_MAYBE_UNUSED u32 hydra_impl_callstub_6(hydra_machine_t *m, addr_t addr, int flags, u16 arg1, u16 arg2, u16 arg3, u16 arg4, u16 arg5, u16 arg6)
{
  PUSH(arg6);
  PUSH(arg5);
  PUSH(arg4);
  PUSH(arg3);
  PUSH(arg2);
  PUSH(arg1);
  hydra_impl_call(m, addr, flags);
  if (!(flags & DONT_POP_ARGS)) SP += 0xc;
  return (u32)DX << 16 | AX;
}

static HYDRA_MAYBE_UNUSED u32 hydra_impl_callstub_7(hydra_machine_t *m, addr_t addr, int flags, u16 arg1, u16 arg2, u16 arg3, u16 arg4, u16 arg5, u16 arg6, u16 arg7)
{
  PUSH(arg7);
  PUSH(arg6);
  PUSH(arg5);
  PUSH(arg4);
  PUSH(arg3);
  PUSH(arg2);
  PUSH(arg1);
  hydra_impl_call(m, addr, flags);
  if (!(flags & DONT_POP_ARGS)) SP += 0xe;
  return (u32)DX << 16 | AX;
}
