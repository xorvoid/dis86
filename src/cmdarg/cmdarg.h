#pragma once
#include <stdbool.h>
#include <stdint.h>

bool cmdarg_option (int * argc, char *** argv, const char * name, bool *        _out);
bool cmdarg_string (int * argc, char *** argv, const char * name, const char ** _out);
bool cmdarg_u64    (int * argc, char *** argv, const char * name, uint64_t *    _out);
bool cmdarg_i64    (int * argc, char *** argv, const char * name, int64_t *     _out);
