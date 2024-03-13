#pragma once
#include "header.h"

typedef struct datamap       datamap_t;
typedef struct datamap_entry datamap_entry_t;

enum {
  DATAMAP_TYPE_U8,
  DATAMAP_TYPE_U16,
};

struct datamap
{
  datamap_entry_t * entries;
  size_t n_entries;
};

struct datamap_entry
{
  char * name;
  int    type; /* DATAMAP_TYPE_ */
  u16    addr;
};

datamap_t *datamap_load_from_mem(const char *str, size_t n);
datamap_t *datamap_load_from_file(const char *filename);
void datamap_delete(datamap_t *d);
