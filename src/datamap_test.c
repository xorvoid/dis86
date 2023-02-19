#include "header.h"
#include "datamap.h"

#define FMT_HDR  "%-10s %-6s %s\n"
#define FMT_DATA "%-10s %-6s 0x%x\n"

const char *type_str(int typ)
{
  switch (typ) {
    case DATAMAP_TYPE_U8: return "u8";
    case DATAMAP_TYPE_U16: return "u16";
    default: return "unknown";
  }
}

int main(int argc, char *argv[])
{
  if (argc < 2) {
    fprintf(stderr, "usage: %s <filename>\n", argv[0]);
    return 1;
  }
  const char *filename = argv[1];

  datamap_t *d = datamap_load(filename);
  if (!d) FAIL("Failed to load datamap");

  printf(FMT_HDR, "name", "type", "addr");
  printf("-----------------------------\n");

  for (size_t i = 0; i < d->n_entries; i++) {
    datamap_entry_t *ent = &d->entries[i];
    printf(FMT_DATA, ent->name, type_str(ent->type), ent->addr);
  }

  datamap_delete(d);
  return 0;
}
