#include "shmdata.h"
#include "internal.h"

enum {
  STATE_INIT,
  STATE_INIT2,
  STATE_WAIT,
  STATE_RUN,
};

static shmdata_t * shm;
static int         state = STATE_INIT;
static addr_t      last_addr;

void remote_init(void)
{
  shm = shmdata_create("/dev/shm/hydra_remote");
  if (!shm) FAIL("Failed to create shmdata");

  shm->init = 0;
  shm->pid = getpid();
  shm->req = 0;
  shm->ack = 1;

  printf("waiting for init\n");
}

void update_shmdata_from_hydra(hydra_machine_t *m)
{
  shm->ax    = m->registers->ax;
  shm->bx    = m->registers->bx;
  shm->cx    = m->registers->cx;
  shm->dx    = m->registers->dx;
  shm->si    = m->registers->si;
  shm->di    = m->registers->di;
  shm->bp    = m->registers->bp;
  shm->sp    = m->registers->sp;
  shm->ip    = m->registers->ip;
  shm->cs    = m->registers->cs;
  shm->ds    = m->registers->ds;
  shm->es    = m->registers->es;
  shm->ss    = m->registers->ss;
  shm->flags = m->registers->flags;
}

void update_hydra_from_shmdata(hydra_machine_t *m)
{
  // Copy over register values
  m->registers->ax    = shm->ax;
  m->registers->bx    = shm->bx;
  m->registers->cx    = shm->cx;
  m->registers->dx    = shm->dx;
  m->registers->si    = shm->si;
  m->registers->di    = shm->di;
  m->registers->bp    = shm->bp;
  m->registers->sp    = shm->sp;
  m->registers->ip    = shm->ip;
  m->registers->cs    = shm->cs;
  m->registers->ds    = shm->ds;
  m->registers->es    = shm->es;
  m->registers->ss    = shm->ss;
  m->registers->flags = shm->flags;

  // Perform update in dosbox state
  m->hardware->update_registers(m->hardware->ctx, m->registers);
}

void post_init_state(hydra_machine_t *m)
{
  // Set registers to determined values. Currently, they'll have the DOSBOX setup values.
  // But this breaks comparison, so we need to bring them into alignment.
  m->registers->ax = 0;
  m->registers->bx = 0;
  m->registers->cx = 0;
  m->registers->dx = 0;
  m->registers->si = 0;
  m->registers->di = 0;
  m->registers->bp = 0;
  m->registers->flags = 1<<9; // IF

  // Perform update in dosbox state
  m->hardware->update_registers(m->hardware->ctx, m->registers);
}

void remote_step_hook(hydra_machine_t *m)
{
  u16 cs = m->registers->cs;
  u16 ip = m->registers->ip;
  addr_t addr = ADDR_MAKE(cs, ip);

  if (cs < 0x823) return;
  if (cs & 0x8000) return;
  if (addr_equal(last_addr, addr)) return;
  last_addr = addr;

  while (1) {
    switch (state) {
      /* case STATE_INIT: { */
      /*   if (!(cs == 0x823 && ip == 0)) return; */
      /*   state = STATE_INIT2; */
      /*   return; */
      /* } break; */
      case STATE_INIT: {
        if (!(cs == 0x823 && ip == 0)) return;
        //printf("init\n");
        post_init_state(m);
        update_shmdata_from_hydra(m);
        BARRIER();
        shm->init = 1;
        BARRIER();
        state = STATE_WAIT;
      } break;
      case STATE_WAIT: {
        //printf("wait\n");
        while (1) {
          BARRIER();
          u64 req = shm->req;
          u64 ack = shm->ack;
          if (req <= ack) continue;
          //printf("run %04x:%04x\n", cs, ip);
          update_hydra_from_shmdata(m);
          state = STATE_RUN;
          return; // return to hydra to run instruction
        }
      } break;
      case STATE_RUN: {
        //printf("run\n");
        update_shmdata_from_hydra(m);
        BARRIER();
        shm->ack = shm->req;
        BARRIER();
       state = STATE_WAIT;
      } break;
    }
  }
}
