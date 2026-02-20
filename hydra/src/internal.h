#pragma once
#include "dosbox-x/include/export/dosbox-x/hydra_machine.h"
#include <errno.h>
#include <pthread.h>
#include "header.h"
#include "addr.h"
#include "conf.h"
#include "dump.h"
#include "hooks.h"
#include "machine.h"
#include "overlay.h"
#include "callstack.h"
#include "functions.h"

// FIXME: REMOVE THIS HARDCODING
//#define CODE_START_SEG ((u16)0x823)

#define ENABLE_DEBUG_CALLSTACK 0
#define MAX_HOOKS 2048

// FIXME: FIND A GOOD HOME
enum {
  HYDRA_MODE_NORMAL = 0,
  HYDRA_MODE_CAPTURE = 1,
  HYDRA_MODE_RESTORE = 2,
};

typedef struct hydra_mode hydra_mode_t;
struct hydra_mode {
  int mode;
  addr_t capture_addr;       // valid only when mode == CAPTURE
  const char *state_path;
};
extern hydra_mode_t HYDRA_MODE[1];

/********************************************************************/
/* conf.c */

extern hydra_conf_t HYDRA_CONF[1];
#define CODE_START_SEG (HYDRA_CONF->code_load_offset)

/********************************************************************/
/* hooks.c */

typedef struct hydra_hook   hydra_hook_t;
struct hydra_hook
{
  const char *name;
  hydra_result_t (*func)(hydra_machine_t *);
  addr_t addr;
  int flags;
};

void hydra_hook_register(hydra_hook_t entry);
hydra_hook_t * hydra_hook_find(addr_t addr);
bool hydra_hook_entry(addr_t addr);

/********************************************************************/
/* exec.c */

enum {
  HYDRA_EXEC_STATE_UNINIT = 0,
  HYDRA_EXEC_STATE_IDLE,
  HYDRA_EXEC_STATE_ACTIVE,
  HYDRA_EXEC_STATE_DONE,
};

typedef struct hydra_exec_ctx hydra_exec_ctx_t;
struct hydra_exec_ctx
{
  pthread_t             thread;
  pthread_mutex_t       mutex[1];
  pthread_cond_t        cond_main[1];
  pthread_cond_t        cond_child[1];
  int                   state;  // HYDRA_EXEC_STATE_*

  hydra_hook_t *        hook;
  hydra_machine_t       machine;
  hydra_result_t        result;

  u16                   saved_cs;
  u16                   saved_ip;
  int                   maybe_reloc;
};

hydra_exec_ctx_t * execution_context_get(u16 *opt_exec_id);
void               execution_context_set(hydra_exec_ctx_t *ctx);

void hydra_exec_init(hydra_machine_hardware_t *hw, hydra_machine_audio_t *audio);
int hydra_exec_run(hydra_machine_t *m);

/********************************************************************/
/* callstack.c */

void hydra_callstack_init(void);
void hydra_callstack_trigger_enter(uint16_t seg, uint16_t off);
void hydra_callstack_dump(void);
void hydra_callstack_notify(hydra_machine_t *m);
void hydra_callstack_track(hydra_machine_t *m, size_t interrupt_count);
void hydra_callstack_ret(hydra_machine_t *m);

/********************************************************************/
/* datasection.c */

u8 * hydra_datasection_baseptr(void);
void hydra_datasection_baseptr_set(u8 *ptr);
