#pragma once
#include <stdbool.h>
#include <stdint.h>

bool         cmdarg_option (int * argc, char *** argv, const char * name);
const char * cmdarg_string (int * argc, char *** argv, const char * name, const char * default_);
uint64_t     cmdarg_u64    (int * argc, char *** argv, const char * name, uint64_t default_);
int64_t      cmdarg_i64    (int * argc, char *** argv, const char * name, int64_t  default_);
