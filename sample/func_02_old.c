#define _param_0006 ARG_16(0x6)
#define _local_0002 LOCAL_16(0x2)
#define _local_0008 LOCAL_16(0x8)
#define _local_0006 LOCAL_32(0x6)
#define _local_000a LOCAL_16(0xa)
void func_00006b42__0622_0922(void)
{
  PUSH(BP);                                          // push   bp
  BP = SP;                                           // mov    bp,sp
  SP -= 0xa;                                         // sub    sp,0xa
  PUSH(SI);                                          // push   si
  PUSH(DI);                                          // push   di
  DI = _param_0006;                                  // mov    di,WORD PTR ss:[bp+0x6]
  BX = DI;                                           // mov    bx,di
  BX <<= 0x2;                                        // shl    bx,0x2
  AX = *PTR_16(DS, BX+0x946);                        // mov    ax,WORD PTR ds:[bx+0x946]
  DX = *PTR_16(DS, BX+0x944);                        // mov    dx,WORD PTR ds:[bx+0x944]
  *(u16*)((u8*)&_local_0006 + 2) = AX;               // mov    WORD PTR ss:[bp-0x4],ax
  *(u16*)((u8*)&_local_0006 + 0) = DX;               // mov    WORD PTR ss:[bp-0x6],dx
  SI = 0;                                            // xor    si,si
  goto label_00006b93;                               // jmp    0x6b93

 label_00006b64:
  LOAD_SEG_OFF(ES, BX, _local_0006);                 // les    bx,DWORD PTR ss:[bp-0x6]
                                                     // test   WORD PTR es:[bx],0x40
  if (*PTR_16(ES, BX) == 0x40) goto label_00006b72;  // je     0x6b72
  *PTR_16(ES, BX) ^= 0x50;                           // xor    WORD PTR es:[bx],0x50

 label_00006b72:
  LOAD_SEG_OFF(ES, BX, _local_0006);                 // les    bx,DWORD PTR ss:[bp-0x6]
  AX = *PTR_16(ES, BX);                              // mov    ax,WORD PTR es:[bx]
  _local_0002 = AX;                                  // mov    WORD PTR ss:[bp-0x2],ax
                                                     // test   WORD PTR ss:[bp-0x2],0x14
  if (_local_0002 != 0x14) goto label_00006b8e;      // jne    0x6b8e
                                                     // push   WORD PTR ss:[bp-0x4]
                                                     // push   bx
                                                     // callf  0x581:0x496
  F_vga_dyn_append(m, BX, (u16)(_local_0006>>16));   // add    sp,0x4

 label_00006b8e:
  *(u16*)((u8*)&_local_0006 + 0) += 0x20;            // add    WORD PTR ss:[bp-0x6],0x20
  SI += 1 ;                                          // inc    si

 label_00006b93:
  BX = DI;                                           // mov    bx,di
  BX <<= 0x1;                                        // shl    bx,0x1
                                                     // cmp    WORD PTR ds:[bx+0x8c8],si
  if ((i16)*PTR_16(DS, BX+0x8c8) > (i16)SI) goto label_00006b64; // jg     0x6b64
                                                     // cmp    di,0x3
  if (DI != 0x3) goto label_00006c01;                // jne    0x6c01
  BX = G_data_08c2;                                  // mov    bx,WORD PTR ds:0x8c2
  BX <<= 0x2;                                        // shl    bx,0x2
  LOAD_SEG_OFF(ES, BX, *PTR_32(DS, BX+0x7a04));      // les    bx,DWORD PTR ds:[bx+0x7a04]
                                                     // push   WORD PTR es:[bx+0xa2]
                                                     // push   WORD PTR es:[bx+0xa0]
                                                     // callf  0x9c0:0xf
  F_list_load_next(m, *PTR_16(ES, BX+0xa0), *PTR_16(ES, BX+0xa2)); // add    sp,0x4
  _local_0008 = DX;                                  // mov    WORD PTR ss:[bp-0x8],dx
  _local_000a = AX;                                  // mov    WORD PTR ss:[bp-0xa],ax
  SI = 0;                                            // xor    si,si
                                                     // cmp    si,WORD PTR ds:0x964
  if ((i16)SI >= (i16)G_data_0964) goto label_00006c01; // jge    0x6c01

 label_00006bcd:
  AX = _local_0008;                                  // mov    ax,WORD PTR ss:[bp-0x8]
  DX = _local_000a;                                  // mov    dx,WORD PTR ss:[bp-0xa]
  DX += 0xc;                                         // add    dx,0xc
  *(u16*)((u8*)&_local_0006 + 2) = AX;               // mov    WORD PTR ss:[bp-0x4],ax
  *(u16*)((u8*)&_local_0006 + 0) = DX;               // mov    WORD PTR ss:[bp-0x6],dx
                                                     // push   ax
                                                     // push   dx
                                                     // callf  0x581:0x496
  F_vga_dyn_append(m, DX, AX);                       // add    sp,0x4
  SI += 1 ;                                          // inc    si
                                                     // push   WORD PTR ss:[bp-0x8]
                                                     // push   WORD PTR ss:[bp-0xa]
                                                     // callf  0x9c0:0x2e
  F_list_load_next_2(m, _local_000a, _local_0008);   // add    sp,0x4
  _local_0008 = DX;                                  // mov    WORD PTR ss:[bp-0x8],dx
  _local_000a = AX;                                  // mov    WORD PTR ss:[bp-0xa],ax
                                                     // cmp    si,WORD PTR ds:0x964
  if ((i16)SI < (i16)G_data_0964) goto label_00006bcd; // jl     0x6bcd

 label_00006c01:
  DI = POP();                                        // pop    di
  SI = POP();                                        // pop    si
  LEAVE(BP, SP);                                     // leave
  RETURN_FAR();                                      // retf
}
#undef _param_0006
#undef _local_0002
#undef _local_0008
#undef _local_0006
#undef _local_000a

