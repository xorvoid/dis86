#include "cmdarg.h"
#include <string.h>

static inline bool parse_u64(const char *s, uint64_t *_num)
{
  uint64_t num = 0;
  while (1) {
    char c = *s++;
    if (!c) break;
    if (!('0' <= c && c <= '9')) return false; // not a decimal digit

    uint64_t next_num = 10*num + (uint64_t)(c-'0');
    if (next_num < num) return false; // overflow!
    num = next_num;
  }

  *_num = num;
  return true;
}

static inline bool parse_i64(const char *s, int64_t *_num)
{
  bool neg = false;
  if (*s == '-') {
    neg = true;
    s++;
  }

  uint64_t unum = 0;
  if (!parse_u64(s, &unum)) return false;

  int64_t num;
  if (neg) {
    if (unum > (1ull<<63)) return false; // overflow
    num = -(int64_t)unum;
  } else {
    if (unum >= (1ull<<63)) return false; // overflow
    num = (int64_t)unum;
  }

  *_num = num;
  return true;
}

bool cmdarg_option(int * _argc, char *** _argv, const char * name, bool *_out)
{
  char ** argv = *_argv;
  int     argc = *_argc;

  // Search for the option name
  int  found_idx = -1;
  for (int i = 0; i < argc; i++) {
    if (0 == strcmp(name, argv[i])) {
      found_idx = i;
      break;
    }
  }

  // Not found
  if (found_idx == -1) return false;

  // On success, remove from the arg list
  for (int i = found_idx+1; i < argc; i++) {
    argv[i-1] = argv[i];
  }
  argc--;

  *_argc = argc;
  if (_out) *_out = true;
  return true;
}


bool cmdarg_string(int * _argc, char *** _argv, const char * name, const char ** _out)
{
  char ** argv = *_argv;
  int     argc = *_argc;

  // Search for the option name
  int  found_idx = -1;
  for (int i = 0; i < argc; i++) {
    if (0 == strcmp(name, argv[i])) {
      found_idx = i;
      break;
    }
  }

  // Failure
  if (found_idx == -1 || found_idx+1 == argc) return false;

  // Capture return value
  const char *ret =  argv[found_idx+1];

  // On success, remove from the arg list
  for (int i = found_idx+2; i < argc; i++) {
    argv[i-2] = argv[i];
  }
  argc -= 2;

  *_argc = argc;
  if (_out) *_out = ret;
  return true;
}

bool cmdarg_u64(int * _argc, char *** _argv, const char * name, uint64_t *_out)
{
  char ** argv = *_argv;
  int     argc = *_argc;

  // Search for the option name
  int  found_idx = -1;
  for (int i = 0; i < argc; i++) {
    if (0 == strcmp(name, argv[i])) {
      found_idx = i;
      break;
    }
  }

  // Failure
  if (found_idx == -1 || found_idx+1 == argc) return false;

  // Try to parse that value
  uint64_t ret = 0;
  const char *data =  argv[found_idx+1];
  if (!parse_u64(data, &ret)) return false;

  // On success, remove from the arg list
  for (int i = found_idx+2; i < argc; i++) {
    argv[i-2] = argv[i];
  }
  argc -= 2;

  *_argc = argc;
  if (_out) *_out = ret;
  return true;
}

bool cmdarg_i64(int * _argc, char *** _argv, const char * name, int64_t *_out)
{
  char ** argv = *_argv;
  int     argc = *_argc;

  // Search for the option name
  int  found_idx = -1;
  for (int i = 0; i < argc; i++) {
    if (0 == strcmp(name, argv[i])) {
      found_idx = i;
      break;
    }
  }

  // Failure
  if (found_idx == -1 || found_idx+1 == argc) return false;

  // Try to parse that value
  int64_t ret = 0;
  const char *data =  argv[found_idx+1];
  if (!parse_i64(data, &ret)) return false;

  // On success, remove from the arg list
  for (int i = found_idx+2; i < argc; i++) {
    argv[i-2] = argv[i];
  }
  argc -= 2;

  *_argc = argc;
  if (_out) *_out = ret;
  return true;
}
