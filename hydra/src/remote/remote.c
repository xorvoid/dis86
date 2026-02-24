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

void update_shmdata(hydra_machine_t *m)
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

void remote_notify(hydra_machine_t *m)
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
        printf("init\n");
        update_shmdata(m);
        BARRIER();
        shm->init = 1;
        BARRIER();
        state = STATE_WAIT;
      } break;
      case STATE_WAIT: {
        printf("wait\n");
        while (1) {
          BARRIER();
          u64 req = shm->req;
          u64 ack = shm->ack;
          if (req <= ack) continue;
          printf("run %04x:%04x\n", cs, ip);
          state = STATE_RUN;
          return; // return to hydra to run instruction
        }
      } break;
      case STATE_RUN: {
        printf("run\n");
        update_shmdata(m);
        BARRIER();
        shm->ack = shm->req;
        BARRIER();
       state = STATE_WAIT;
      } break;
    }
  }
}
