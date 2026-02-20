#include "internal.h"

void hydra_cpu_dump(hydra_machine_registers_t *cpu)
{
  printf("CPU STATE:\n");
  printf("  AX: %04x  BX: %04x  CX: %04x  DX: %04x\n", cpu->ax, cpu->bx, cpu->cx, cpu->dx);
  printf("  SI: %04x  DI: %04x  BP: %04x  SP: %04x  IP: %04x\n", cpu->si, cpu->di, cpu->bp, cpu->sp, cpu->ip);
  printf("  CS: %04x  DS: %04x  ES: %04x  SS: %04x\n", cpu->cs, cpu->ds, cpu->es, cpu->ss);
  printf("  FLAGS: %04x\n", cpu->flags);
}
