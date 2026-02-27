# DIV Instruction Flag Behavior in core_normal

Source: `src/cpu/instructions.h` lines 635–782

## Execution Flow

All DIV/IDIV variants (DIVB/DIVW/DIVD, IDIVB/IDIVW/IDIVD) follow this pattern:

1. Compute quotient and remainder
2. Check for divide-by-zero or quotient overflow → `EXCEPTION(0)` if triggered
3. Store results into registers (AL/AH, AX/DX, EAX/EDX)
4. `FillFlags()` — flushes any pending lazy flags from the *previous* instruction
5. Explicitly set all six status flags via `SETFLAGBIT`

## Flag Values Set

| Flag | Value | Notes |
|------|-------|-------|
| **AF** | Always **0** | Marked `/*FIXME*/` |
| **SF** | Always **0** | Marked `/*FIXME*/` |
| **OF** | Always **0** | Marked `/*FIXME*/` |
| **ZF** | `(rem==0) && ((quo&1)!=0)` | Set iff remainder is zero AND quotient is odd |
| **CF** | `((rem&3)>=1 && (rem&3)<=2)` | Set iff low 2 bits of remainder are `01` or `10` |
| **PF** | `parity(rem) XOR parity(quo) XOR FLAG_PF` | Set iff rem and quo have the **same** parity |

For 16-bit and 32-bit, parity is computed over the full width:

```c
// instructions.h:591-592
#define PARITY16(x)  (parity_lookup[((x)>>8)&0xff] ^ parity_lookup[(x)&0xff] ^ FLAG_PF)
#define PARITY32(x)  (PARITY16((x)&0xffff) ^ PARITY16(((x)>>16)&0xffff) ^ FLAG_PF)
```

`parity_lookup[byte]` returns `FLAG_PF` (0x4) if the byte has **even** bit-count, `0` if odd.

## Example: DIVB (8-bit)

```c
// instructions.h:635-652
#define DIVB(op1,load,save)
{
    Bitu val=load(op1);
    if (val==0) EXCEPTION(0);
    Bitu quo=reg_ax / val;
    uint8_t rem=(uint8_t)(reg_ax % val);
    uint8_t quo8=(uint8_t)(quo&0xff);
    if (quo>0xff) EXCEPTION(0);
    reg_ah=rem;
    reg_al=quo8;
    FillFlags();
    SETFLAGBIT(AF,0);/*FIXME*/
    SETFLAGBIT(SF,0);/*FIXME*/
    SETFLAGBIT(OF,0);/*FIXME*/
    SETFLAGBIT(ZF,(rem==0)&&((quo8&1)!=0));
    SETFLAGBIT(CF,((rem&3) >= 1 && (rem&3) <= 2));
    SETFLAGBIT(PF,parity_lookup[rem&0xff]^parity_lookup[quo8&0xff]^FLAG_PF);
}
```

## Notes

- All flags after DIV are officially **undefined** on real x86 hardware. The `/*FIXME*/`
  comments on AF, SF, OF indicate the author knows these are uncertain approximations.
- The ZF and CF formulas are non-standard approximations of observed real-hardware behavior.
- `FillFlags()` has a `case t_DIV: break;` — it does nothing for DIV-type lazy flags.
  It only serves to commit any previous instruction's pending lazy flags before the
  `SETFLAGBIT` calls overwrite them.
- IDIV variants use identical flag-setting logic, just with signed arithmetic for the
  quotient/remainder computation.
