#pragma once
#include <stdlib.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/*****************************************************************/
/* CORE TYPES */
/*****************************************************************/
typedef struct dis86        dis86_t;
typedef struct dis86_instr  dis86_instr_t;

/*****************************************************************/
/* CORE ROUTINES */
/*****************************************************************/

/* Create new instance: deep copies the memory */
dis86_t *dis86_new(size_t base_addr, char *mem, size_t mem_sz);

/* Destroys an instance */
void dis86_delete(dis86_t *d);

/* Get next instruction */
dis86_instr_t *dis86_next(dis86_t *d);

/* Get Position */
size_t dis86_position(dis86_t *d);

/* Get Baseaddr */
size_t dis86_baseaddr(dis86_t *d);

/* Get Length */
size_t dis86_length(dis86_t *d);

/*****************************************************************/
/* INSTR ROUTINES */
/*****************************************************************/

/* Get the address where the instruction resides */
size_t dis86_instr_addr(dis86_instr_t *ins);

/* Get the number of bytes used in the encoding */
size_t dis86_instr_n_bytes(dis86_instr_t *ins);

/* Copy the instruction */
void dis86_instr_copy(dis86_instr_t *dst, dis86_instr_t *src);

/*****************************************************************/
/* PRINT ROUTINES */
/*****************************************************************/

/* Print */
char *dis86_print_intel_syntax(dis86_t *d, dis86_instr_t *ins, bool with_detail);

/*****************************************************************/
/* DECOMPILE ROUTINES */
/*****************************************************************/

/* Decompile to C code */
char *dis86_decompile(dis86_t *d, const char *func_name, dis86_instr_t *ins, size_t n_ins);

#ifdef __cplusplus
}
#endif
