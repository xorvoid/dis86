
void hydra_overlay_segment_set(u16 overlay_num, u16 segment);
void hydra_overlay_segment_clear(u16 overlay_num);

u16    hydra_overlay_segment_lookup(u16 overlay_num);
addr_t hydra_overlay_segment_remap_from_physical(addr_t addr);
