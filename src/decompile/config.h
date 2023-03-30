#pragma once
#include "header.h"
#include "segoff.h"

#define MAX_CONFIG_FUNCS   1024
#define MAX_CONFIG_SEGMAPS 1024

typedef struct dis86_decompile_config config_t;
typedef struct config_func            config_func_t;
typedef struct config_segmap          config_segmap_t;

struct config_func
{
  char *   name;
  segoff_t addr;
};

struct config_segmap
{
  char * name;
  u16    from;
  u16    to;
};

struct dis86_decompile_config
{
  size_t          func_len;
  config_func_t   func_arr[MAX_CONFIG_FUNCS];

  size_t          segmap_len;
  config_segmap_t segmap_arr[MAX_CONFIG_SEGMAPS];
};

config_t *   config_read_new(const char *path);
config_t *   config_default_new(void);
void         config_delete(config_t *cfg);

void         config_print(config_t *cfg);
const char * config_func_lookup(config_t *cfg, segoff_t s);
bool         config_seg_remap(config_t *cfg, u16 *inout_seg);
