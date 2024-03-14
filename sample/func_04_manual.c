void my_function()
{
  u32 _local_0006 = PTR_32(ds_1, 0x944)[_param_0006];
  for (u16 i = 0;; i++) {
    u16 val = PTR_16(ds_1, 0x8c8)[_param_0006];
    if (i >= val) {
      break;
    }
    u16 *ptr = PTR_16_FROM_32(_local_0006);
    if ((*ptr & 64) == 0) {
      *ptr = *ptr ^ 80;
    }
    if ((*ptr & 20) != 0) {
      F_vga_dyn_append((u16)_local_0006, (u16)(_local_0006>>16));
    }
    _local_0006 += 32;
  }

  if (_param_0006 != 3) {
    u32 tmp_3 = PTR_32(ds_1, 0x7a04)[G_data_08c2];
    u8 *ptr = PTR_8_FROM_32(tmp_3);
    u32 addr = *(u32*)(ptr + 0xa0);

    u32 _local_0010 = F_list_load_next((u16)addr, (u16)(addr>>16));
    for (u16 i = 0; i < G_data_0964; i++) {
      _local_0006 = _local_0010 + 12;
      F_vga_dyn_append((u16)_local_0010 + 12, (u16)(_local_0010>>16));
      _local_0010 = F_list_load_next_2((u16)_local_0010, (u16)(_local_0010>>16));
    }
  }
}
