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

  bsl_iter_t it[1];
  bsl_iter_begin(it, func);

  int type;
  const char *key;
  void *val;
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
  free(cfg);
}

void config_print(config_t *cfg)
{
  printf("functions: %p\n", cfg);
  for (size_t i = 0; i < cfg->func_len; i++) {
    config_func_t *f = &cfg->func_arr[i];
    printf("  %-30s  %04x:%04x\n", f->name, f->addr.seg, f->addr.off);
  }
}

const char * config_lookup_func(config_t *cfg, segoff_t s)
{
  for (size_t i = 0; i < cfg->func_len; i++) {
    config_func_t *f = &cfg->func_arr[i];
    if (f->addr.seg == s.seg && f->addr.off == s.off) {
      return f->name;
    }
  }
  return NULL;
}

dis86_decompile_config_t * dis86_decompile_config_read_new(const char *path)
{ return config_read_new(path); }

void dis86_decompile_config_delete(dis86_decompile_config_t *cfg)
{ config_delete(cfg); }
