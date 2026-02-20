typedef struct hydra_function_metadata  hydra_function_metadata_t;
typedef struct hydra_function_def       hydra_function_def_t;

struct hydra_function_def
{
  const char *name;
  addr_t addr;
};


struct hydra_function_metadata
{
  size_t                 n_defs;
  hydra_function_def_t * defs;
};

void                         hydra_function_metadata_init(void);
const hydra_function_def_t * hydra_function_find(const char *name);
const char *                 hydra_function_name(addr_t s);
bool                         hydra_function_addr(const char *name, addr_t *_out);
