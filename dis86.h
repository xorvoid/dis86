#pragma once
#include <stdlib.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct dis86        dis86_t;
typedef struct dis86_instr  dis86_instr_t;

/* Create new instance: deep copies the memory */
dis86_t *dis86_new(char *mem, size_t mem_sz);

/* Destroys an instance */
void dis86_delete(dis86_t *d);

/* Get next instruction */
dis86_instr_t *dis86_next(dis86_t *d, size_t *addr, size_t *n_bytes);

/* Print */
void dis86_print_intel_syntax(dis86_t *d, dis86_instr_t *ins, size_t addr, size_t n_bytes);

#ifdef __cplusplus
}
#endif
