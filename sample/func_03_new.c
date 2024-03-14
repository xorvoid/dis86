void func_00006b42__0622_0922(void)
{
  bx_2 = _param_0006;
  bx_3 = bx_2 << 2;
  *(u16*)((u8*)&_local_0006 + 2) = *PTR_16(ds_1, bx_3 + 0x946);
  *(u16*)(u8*)&_local_0006 = *PTR_16(ds_1, bx_3 + 0x944);
  si_5 = 0;
  while (1) {
addr_6b93:;
    if (!(*PTR_16(ds_1, (bx_2 << 1) + 0x8c8) > si_5)) {
      goto addr_6b9d;
    }
    tmp_0 = _local_0006;
    es_2 = tmp_0 >> 16;
    bx_4 = tmp_0;
    if ((*PTR_16(es_2, bx_4) & 64) == 0) {
      *PTR_16(es_2, bx_4) = *PTR_16(es_2, bx_4) ^ 80;
    }
    tmp_1 = _local_0006;
    bx_6 = tmp_1;
    if ((*PTR_16(tmp_1 >> 16, bx_6) & 20) != 0) {
      F_vga_dyn_append(bx_6, *(u16*)((u8*)&_local_0006 + 2));
    }
    *(u16*)(u8*)&_local_0006 = *(u16*)(u8*)&_local_0006 + 32;
    si_5 = si_5 + 1;
    goto addr_6b93;
  }
addr_6b9d:;
  if (bx_2 != 3) {
    tmp_2 = *PTR_32(ds_1, (G_data_08c2 << 2) + 0x7a04);
    es_5 = tmp_2 >> 16;
    bx_12 = tmp_2;
    tmp_3 = F_list_load_next(*PTR_16(es_5, bx_12 + 0xa0), *PTR_16(es_5, bx_12 + 0xa2));
    if (0 >= G_data_0964) {
      _local_0010_2 = tmp_3;
      _local_0008_2 = tmp_3 >> 16;
      si_8 = 0;
      while (1) {
addr_6bcd:;
        dx_5 = _local_0010_2 + 12;
        *(u16*)((u8*)&_local_0006 + 2) = _local_0008_2;
        *(u16*)(u8*)&_local_0006 = dx_5;
        F_vga_dyn_append(dx_5, _local_0008_2);
        si_9 = si_8 + 1;
        tmp_4 = F_list_load_next_2(_local_0010_2, _local_0008_2);
        if (!(si_9 < G_data_0964)) {
          goto addr_6c01;
        }
        _local_0010_2 = tmp_4;
        _local_0008_2 = tmp_4 >> 16;
        si_8 = si_9;
        goto addr_6bcd;
      }
    }
  }
addr_6c01:;
  return;
}
