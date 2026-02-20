#pragma once
#include "addr.h"
#include "typecheck.h"

#ifndef ARRAY_SIZE
# define ARRAY_SIZE(arr) (sizeof(arr)/sizeof((arr)[0]))
#endif

/* Macros to be able to decompile to 8086-like C and compile it all into
   working code. Goal is then to re-write to more sane C */

#define MACHINE hydra_machine_t *m

#define AX ((m)->registers->ax)
#define BX ((m)->registers->bx)
#define CX ((m)->registers->cx)
#define DX ((m)->registers->dx)
#define SI ((m)->registers->si)
#define DI ((m)->registers->di)
#define BP ((m)->registers->bp)
#define SP ((m)->registers->sp)
#define IP ((m)->registers->ip)
#define CS ((m)->registers->cs)
#define DS ((m)->registers->ds)
#define ES ((m)->registers->es)
#define SS ((m)->registers->ss)

#define AL (*(((u8*)&(AX))))
#define AH (*(((u8*)&(AX))+1))
#define BL (*(((u8*)&(BX))))
#define BH (*(((u8*)&(BX))+1))
#define CL (*(((u8*)&(CX))))
#define CH (*(((u8*)&(CX))+1))
#define DL (*(((u8*)&(DX))))
#define DH (*(((u8*)&(DX))+1))

#define FLAGS ((m)->registers->flags)
#define FLAGS_CF (!!(FLAGS & (1<<0)))
#define FLAGS_PF (!!(FLAGS & (1<<2)))
#define FLAGS_AF (!!(FLAGS & (1<<4)))
#define FLAGS_ZF (!!(FLAGS & (1<<6)))
#define FLAGS_SF (!!(FLAGS & (1<<7)))
#define FLAGS_TF (!!(FLAGS & (1<<8)))
#define FLAGS_IF (!!(FLAGS & (1<<9)))
#define FLAGS_DF (!!(FLAGS & (1<<10)))
#define FLAGS_OF (!!(FLAGS & (1<<11)))

#define FLAGS_CF_ON() do { FLAGS |= 1<<0; } while(0)
#define FLAGS_PF_ON() do { FLAGS |= 1<<2; } while(0)
#define FLAGS_AF_ON() do { FLAGS |= 1<<4; } while(0)
#define FLAGS_ZF_ON() do { FLAGS |= 1<<6; } while(0)
#define FLAGS_SF_ON() do { FLAGS |= 1<<7; } while(0)
#define FLAGS_TF_ON() do { FLAGS |= 1<<8; } while(0)
#define FLAGS_IF_ON() do { FLAGS |= 1<<9; } while(0)
#define FLAGS_DF_ON() do { FLAGS |= 1<<10); } while(0)
#define FLAGS_OF_ON() do { FLAGS |= 1<<11); } while(0)

#define FLAGS_CF_OFF() do { FLAGS &= ~(1<<0); } while(0)
#define FLAGS_PF_OFF() do { FLAGS &= ~(1<<2); } while(0)
#define FLAGS_AF_OFF() do { FLAGS &= ~(1<<4); } while(0)
#define FLAGS_ZF_OFF() do { FLAGS &= ~(1<<6); } while(0)
#define FLAGS_SF_OFF() do { FLAGS &= ~(1<<7); } while(0)
#define FLAGS_TF_OFF() do { FLAGS &= ~(1<<8); } while(0)
#define FLAGS_IF_OFF() do { FLAGS &= ~(1<<9); } while(0)
#define FLAGS_DF_OFF() do { FLAGS &= ~(1<<10); } while(0)
#define FLAGS_OF_OFF() do { FLAGS &= ~(1<<11); } while(0)

#define FLAGS_CF_VAL(v) do { if (v) FLAGS_CF_ON(); else FLAGS_CF_OFF(); } while(0)
#define FLAGS_PF_VAL(v) do { if (v) FLAGS_PF_ON(); else FLAGS_PF_OFF(); } while(0)
#define FLAGS_AF_VAL(v) do { if (v) FLAGS_AF_ON(); else FLAGS_AF_OFF(); } while(0)
#define FLAGS_ZF_VAL(v) do { if (v) FLAGS_ZF_ON(); else FLAGS_ZF_OFF(); } while(0)
#define FLAGS_SF_VAL(v) do { if (v) FLAGS_SF_ON(); else FLAGS_SF_OFF(); } while(0)
#define FLAGS_TF_VAL(v) do { if (v) FLAGS_TF_ON(); else FLAGS_TF_OFF(); } while(0)
#define FLAGS_IF_VAL(v) do { if (v) FLAGS_IF_ON(); else FLAGS_IF_OFF(); } while(0)
#define FLAGS_DF_VAL(v) do { if (v) FLAGS_DF_ON(); else FLAGS_DF_OFF(); } while(0)
#define FLAGS_OF_VAL(v) do { if (v) FLAGS_OF_ON(); else FLAGS_OF_OFF(); } while(0)

#define JB(_)  (FLAGS_CF)
#define JAE(_) (!FLAGS_CF)
#define JL(_)  (FLAGS_SF != FLAGS_OF)


//#define ADDR(seg, off) ({ STATIC_ASSERT_U16(seg);  STATIC_ASSERT_U16(off); ((u32)(u16)(seg))*16 + (u32)(u16)(off); })
#define ADDR(seg, off) ({ ((u32)(u16)(seg))*16 + (u32)(u16)(off); })
#define LOWER_16(_u32) ({ STATIC_ASSERT_U32(_u32); u32 v = (_u32); (u16)v; })
#define UPPER_16(_u32) ({ STATIC_ASSERT_U32(_u32); u32 v = (_u32); (u16)(v>>16); })

#define LOAD_8(seg, off) m->hardware->mem_read8(m->hardware->ctx, ADDR(seg, off))
#define LOAD_16(seg, off) m->hardware->mem_read16(m->hardware->ctx, ADDR(seg, off))
#define LOAD_32(seg, off) ((u32)LOAD_16(seg, (off)+2) << 16 | (u32)LOAD_16(seg, off))
#define LOAD_ADDR(seg, off) ADDR(LOAD_16(seg, (off)+2), LOAD_16(seg, off))

#define STORE_8(seg, off, val) m->hardware->mem_write8(m->hardware->ctx, ADDR(seg, off), val)
#define STORE_16(seg, off, val) m->hardware->mem_write16(m->hardware->ctx, ADDR(seg, off), val)
#define STORE_32(seg, off, val) do { STORE_16(seg, (off)+2, (u16)((u32)(val)>>16)); STORE_16(seg, off, (u16)val); } while(0)

#define PTR_8(seg, off)  ((u8*)m->hardware->mem_hostaddr(m->hardware->ctx, ADDR(seg, off)))
#define PTR_16(seg, off) ((u16*)m->hardware->mem_hostaddr(m->hardware->ctx, ADDR(seg, off)))
#define PTR_32(seg, off) ((u32*)m->hardware->mem_hostaddr(m->hardware->ctx, ADDR(seg, off)))

#define PTR_8_FROM_32(_u32)  ({ STATIC_ASSERT_U32(_u32); u32 v = (_u32); PTR_8(v>>16, v); })
#define PTR_16_FROM_32(_u32) ({ STATIC_ASSERT_U32(_u32); u32 v = (_u32); PTR_16(v>>16, v); })
#define PTR_32_FROM_32(_u32) ({ STATIC_ASSERT_U32(_u32); u32 v = (_u32); PTR_32(v>>16, v); })

#define TYPE_FROM_32(type, _u32) ((type*)PTR_8_FROM_32(_u32))

#define VAR_8(seg, off)  (*PTR_8(seg, off))
#define VAR_16(seg, off) (*PTR_16(seg, off))
#define VAR_32(seg, off) (*PTR_32(seg, off))

#define ARG_8(off)    (*PTR_8(SS, BP+(off)))
#define ARG_16(off)   (*PTR_16(SS, BP+(off)))
#define ARG_32(off)   (*PTR_32(SS, BP+(off)))

#define LOCAL_8(off)   (*PTR_8(SS, BP-(off)))
#define LOCAL_16(off)  (*PTR_16(SS, BP-(off)))
#define LOCAL_32(off)  (*PTR_32(SS, BP-(off)))

#define PUSH(val) do { SP -= 2; STORE_16(SS, SP, val); } while(0)
#define POP() ({ u16 val = LOAD_16(SS, SP); SP += 2; val; })
#define LEAVE(...) do { SP = BP; BP = POP(); } while(0)

#define SET_32(reg_high, reg_low, _val32) do { \
    static_assert(sizeof(_val32) == 4, ""); \
    u32 val32 = _val32; \
    (reg_high) = (u16)((val32)>>16); \
    (reg_low) = (u16)(val32); \
  } while(0)

#define LOAD_SEG_OFF(sreg, reg, val32) SET_32(sreg, reg, val32)

#define REP_MOVS_8() do { \
    for (; CX; CX--) { \
      STORE_8(ES, DI, LOAD_8(DS, SI)); \
      SI += 1; \
      DI += 1; \
    } \
  } while(0)

#define REP_MOVS_16() do { \
    for (; CX; CX--) { \
      STORE_16(ES, DI, LOAD_16(DS, SI)); \
      SI += 2; \
      DI += 2; \
    } \
  } while(0)

#define CALL_FAR(seg, off, ...) CALL_FAR_ARGS(seg, off, __VA_ARGS__)

#define CALL_FAR_ARGS(seg, off, ...)  ({        \
  u16 args[] = {__VA_ARGS__}; \
  PUSH_ARGS(args); \
  u32 ret = hydra_impl_call_far(seg, off);     \
  POP_ARGS(args);                \
  ret; })

#define CALL_FAR_CS(off) hydra_impl_call_far_cs(CS, off);

#define CALL_FAR_INDIRECT(addr, ...) ({         \
  u16 args[] = {__VA_ARGS__}; \
  PUSH_ARGS(args); \
  hydra_impl_call_far_indirect(addr); \
  POP_ARGS(args);                \
  AX; })

#define CALL_NEAR_OFF(off) hydra_impl_call_near_off(off, 0)
#define CALL_NEAR_OFF_RELOC(off) hydra_impl_call_near_off(off, 1)
#define CALL_NEAR_ABS(off) hydra_impl_call_near_abs(off)
#define CALL_NEAR(_seg, off, ...) ({                \
  u16 args[] = {__VA_ARGS__}; \
  PUSH_ARGS(args); \
  u32 ret = hydra_impl_call_near_off(off, 0);         \
  POP_ARGS(args);                \
  ret; })

#define CALL_FUNC(name) hydra_impl_call_func(#name)

#define PUSH_ARGS(args) do { \
    for (size_t i = ARRAY_SIZE(args); i > 0; i--) { PUSH(args[i-1]); } \
} while(0)

#define POP_ARGS(args) do { \
    for (size_t i = 0; i < ARRAY_SIZE(args); i++) { SP += 2; } \
} while(0)

#define NOP() hydra_impl_nop()
#define CLD() hydra_impl_cld()
#define STD() hydra_impl_std()
#define CLI() hydra_impl_cli()
#define STI() hydra_impl_sti()

#define INB(port) hydra_impl_inb(port);
#define OUTB(port, val) hydra_impl_outb(port, val)

#define INT(num) hydra_impl_int(num);

#define FRAME_ENTER(n) ({ PUSH(BP); BP = SP; SP -= n; })
#define FRAME_LEAVE()  ({ SP = BP; BP = POP(); })

#define REMOVE_ARGS_FAR(n) ({ \
  u16 ret_off = POP(); \
  u16 ret_seg = POP(); \
  SP += n; \
  PUSH(ret_seg); \
  PUSH(ret_off); \
})

#define MAKE_32(upper, lower) ({ \
  STATIC_ASSERT_U16(upper); \
  STATIC_ASSERT_U16(lower); \
  (u32)(upper) << 16 | (u32)(lower); \
})

#define MAKE_16(upper, lower) ({ \
  STATIC_ASSERT_U8(upper); \
  STATIC_ASSERT_U8(lower); \
  (u16)(upper) << 8 | (u16)(lower); \
})

#define UPPER(_u32) ({ STATIC_ASSERT_U32(_u32); (u16)((_u32) >> 16); })
#define LOWER(_u32) ({ STATIC_ASSERT_U32(_u32); (u16)((_u32)); })

#define PTR_TO_ADDR(_ptr) hydra_impl_ptr_to_addr(m, (_ptr))
#define PTR_TO_OFF(_ptr, seg) hydra_impl_ptr_to_off(m, (_ptr), (seg))
#define PTR_TO_32(_ptr) hydra_impl_ptr_to_32(m, (_ptr))

#define PTR_TO_ARGS(ptr) ADDR_TO_ARGS(PTR_TO_ADDR(ptr))
#define U32_TO_ARGS(_u32) LOWER(_u32), UPPER(_u32)

#define ADDR_TO_ARGS(addr) addr_off(addr), addr_seg(addr)

#define RETURN_RESUME()       return HYDRA_RESULT_RESUME()
#define RETURN_JUMP(seg, off) return HYDRA_RESULT_JUMP(seg, off)
#define RETURN_FAR()          return HYDRA_RESULT_RET_FAR()
#define RETURN_NEAR()         return HYDRA_RESULT_RET_NEAR()

#define RETURN_FAR_N(n) ({\
  REMOVE_ARGS_FAR(n); \
  RETURN_FAR(); \
})

#define UNKNOWN() hydra_impl_unknown(__FUNCTION__, __LINE__)

#define MACHINE_STATE_SAVE(path)    m->hardware->state_save(m->hardware->ctx, path)
#define MACHINE_STATE_RESTORE(path) m->hardware->state_restore(m->hardware->ctx, path)

/* Implementations provided "out-of-line" */
void     hydra_impl_unknown(const char *func, int line);
u32      hydra_impl_call_far(u16 seg, u16 off);
u32      hydra_impl_call_far_cs(u16 cs_reg_value, u16 off);
u32      hydra_impl_call_far_indirect(u32 addr);
u32      hydra_impl_call_near_off(u16 off, int maybe_reloc);
u32      hydra_impl_call_near_abs(u16 abs_off);
u32      hydra_impl_call_func(const char *name);
void     hydra_impl_cld(void);
void     hydra_impl_std(void);
void     hydra_impl_cli(void);
void     hydra_impl_sti(void);
u8       hydra_impl_inb(u16 port);
void     hydra_impl_outb(u16 port, u8 val);
void     hydra_impl_int(u8 num);
void     hydra_impl_nop(void);
addr_t   hydra_impl_ptr_to_addr(hydra_machine_t *m, void *ptr);
u16      hydra_impl_ptr_to_off(hydra_machine_t *m, void *ptr, u16 seg);
u32      hydra_impl_ptr_to_32(hydra_machine_t *m, void *ptr);
