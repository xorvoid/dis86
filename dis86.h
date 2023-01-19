#pragma once
#include <stdlib.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct dis86        dis86_t;
typedef struct dis86_instr  dis86_instr_t;

/* Create new instance: deep copies the memory */
dis86_t *dis86_new(size_t base_addr, char *mem, size_t mem_sz);

/* Destroys an instance */
void dis86_delete(dis86_t *d);

/* Get next instruction */
dis86_instr_t *dis86_next(dis86_t *d, size_t *addr, size_t *n_bytes);

/* Get Position */
size_t dis86_position(dis86_t *d);

/* Get Baseaddr */
size_t dis86_baseaddr(dis86_t *d);

/* Get Length */
size_t dis86_length(dis86_t *d);

/* Print */
char *dis86_print_intel_syntax(dis86_t *d, dis86_instr_t *ins, size_t addr, size_t n_bytes, bool with_detail);

#ifdef __cplusplus
}
#endif
