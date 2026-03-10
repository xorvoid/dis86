#include "internal.h"
#include "remote.h"

#define ENABLE_ITRACE 0

const hydra_function_metadata_t * hydra_user_functions(void)
{
  static hydra_function_metadata_t md[1];
  md->n_defs = 0;
  md->defs = NULL;
  return md;
}

const hydra_callstack_metadata_t * hydra_user_callstack(void)
{
  static hydra_callstack_metadata_t md[1];
  md->n_confs = 0;
  md->confs = NULL;
  return md;
}

void hydra_user_init(hydra_conf_t *conf, hydra_machine_hardware_t *hw, hydra_machine_audio_t *audio)
{
  remote_init();

  // tell hydra where important segments are located
  conf->code_load_offset = 0x823;
  conf->data_section_seg = 0xea7;
}

static void capture_itrace(hydra_machine_t *m)
{
  static addr_t last_addr = {};

  u16 cs = m->registers->cs;
  u16 ip = m->registers->ip;
  addr_t addr = ADDR_MAKE(cs, ip);

  if (cs < 0x823) return;
  if (cs & 0x8000) return;
  if (addr_equal(last_addr, addr)) return;

  printf("ITRACE | %04x:%04x\n", cs - 0x823, ip);
  last_addr = addr;
}

void hydra_user_step_hook(hydra_machine_t *m)
{
  if (ENABLE_ITRACE) capture_itrace(m);
  remote_step_hook(m);
}
