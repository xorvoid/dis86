
typedef struct hydra_callstack_conf      hydra_callstack_conf_t;
typedef struct hydra_callstack_metadata  hydra_callstack_metadata_t;

enum {
  HYDRA_CALLSTACK_CONF_TYPE_HANDLER,
  HYDRA_CALLSTACK_CONF_TYPE_IGNORE_ADDR,
  HYDRA_CALLSTACK_CONF_TYPE_IGNORE_ABOVE,
  HYDRA_CALLSTACK_CONF_TYPE_JUMPRET,
};

struct hydra_callstack_conf
{
  int          type; /* HYDRA_CALLSTACK_CONF_TYPE_* */
  const char * name;
  addr_t       addr;
};

struct hydra_callstack_metadata
{
  size_t                   n_confs;
  hydra_callstack_conf_t * confs;
};
