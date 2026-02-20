#pragma once

/* BSL: Barebones Specification Language */

#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>

typedef struct bsl      bsl_t;
typedef struct bsl_iter bsl_iter_t;
struct __attribute__((aligned(16))) bsl_iter { char _opaque[32]; };

enum {
  BSL_SUCCESS,
  BSL_ERR_PARSE,
};

enum {
  BSL_TYPE_STR  = 0,  // char *
  BSL_TYPE_NODE = 1,  // bsl_t *
};

bsl_t *      bsl_parse_new(const char *buf, size_t sz, int *opt_err);
void         bsl_delete(bsl_t *bsl);

void *       bsl_get_generic(bsl_t *bsl, const char *key, int *opt_type);
const char * bsl_get_str(bsl_t *bsl, const char *key);
bsl_t *      bsl_get_node(bsl_t *bsl, const char *key);

void         bsl_iter_begin(bsl_iter_t *it, bsl_t *bsl);
bool         bsl_iter_next(bsl_iter_t *it, int *_type, const char **_key, void **_val);
