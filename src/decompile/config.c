#include "config.h"
#include "dis86_private.h"
#include "bsl/bsl.h"

config_t * config_default_new(void)
{
  config_t * cfg = calloc(1, sizeof(config_t));
  return cfg;
}

config_t * config_read_new(const char *path)
{
  config_t * cfg = calloc(1, sizeof(config_t));

  size_t sz;
  char * data = read_file(path, &sz);
  if (!data) FAIL("Failed to read file: '%s'", path);

  bsl_t *root = bsl_parse_new(data, sz, NULL);
  if (!root) FAIL("Failed to read the config");

  bsl_t *func = bsl_get_node(root, "dis86.functions");
  if (!func) FAIL("Failed to get functions node");

  bsl_iter_t   it[1];
  int          type;
  const char * key;
  void *       val;

  bsl_iter_begin(it, func);
  while (bsl_iter_next(it, &type, &key, &val)) {
    if (type != BSL_TYPE_NODE) FAIL("Expected function properties");
    bsl_t *f = (bsl_t*)val;

    const char *addr_str = bsl_get_str(f, "addr");
    if (!addr_str) FAIL("No function addr property");

    assert(cfg->func_len < ARRAY_SIZE(cfg->func_arr));
    config_func_t *cf = &cfg->func_arr[cfg->func_len++];
    cf->name = strdup(key);
    cf->addr = parse_segoff(addr_str);
  }

  bsl_t *segmap = bsl_get_node(root, "dis86.segmap");
  if (!segmap) FAIL("Failed to get segmap node");

  bsl_iter_begin(it, segmap);
  while (bsl_iter_next(it, &type, &key, &val)) {
    if (type != BSL_TYPE_NODE) FAIL("Expected segmap properties");
    bsl_t *s = (bsl_t*)val;

    const char *from_str = bsl_get_str(s, "from");
    if (!from_str) FAIL("No segmap 'from' property");
    u16 from = parse_hex_u16(from_str, strlen(from_str));

    const char *to_str = bsl_get_str(s, "to");
    if (!to_str) FAIL("No segmap 'to' property");
    u16 to = parse_hex_u16(to_str, strlen(to_str));

    assert(cfg->segmap_len < ARRAY_SIZE(cfg->segmap_arr));
    config_segmap_t *sm = &cfg->segmap_arr[cfg->segmap_len++];
    sm->name = strdup(key);
    sm->from = from;
    sm->to = to;
  }


  bsl_delete(root);
  free(data);

  return cfg;
}

void config_delete(config_t *cfg)
{
  if (!cfg) return;
  for (size_t i = 0; i < cfg->func_len; i++) {
    free(cfg->func_arr[i].name);
  }
  for (size_t i = 0; i < cfg->segmap_len; i++) {
    free(cfg->segmap_arr[i].name);
  }
  free(cfg);
}

void config_print(config_t *cfg)
{
  printf("functions:\n");
  for (size_t i = 0; i < cfg->func_len; i++) {
    config_func_t *f = &cfg->func_arr[i];
    printf("  %-30s  %04x:%04x\n", f->name, f->addr.seg, f->addr.off);
  }
  printf("\n");
  printf("segmap:\n");
  for (size_t i = 0; i < cfg->segmap_len; i++) {
    config_segmap_t *s = &cfg->segmap_arr[i];
    printf("  %-30s  %04x => %04x\n", s->name, s->from, s->to);
  }
}

const char * config_func_lookup(config_t *cfg, segoff_t s)
{
  for (size_t i = 0; i < cfg->func_len; i++) {
    config_func_t *f = &cfg->func_arr[i];
    if (f->addr.seg == s.seg && f->addr.off == s.off) {
      return f->name;
    }
  }
  return NULL;
}

bool config_seg_remap(config_t *cfg, u16 *_seg)
{
  u16 seg = *_seg;
  for (size_t i = 0; i < cfg->segmap_len; i++) {
    config_segmap_t *sm = &cfg->segmap_arr[i];
    if (seg == sm->from) {
      *_seg = sm->to;
      return true;
    }
  }
  return false;
}

dis86_decompile_config_t * dis86_decompile_config_read_new(const char *path)
{ return config_read_new(path); }

void dis86_decompile_config_delete(dis86_decompile_config_t *cfg)
{ config_delete(cfg); }
