#pragma once
#include "header.h"
#include "dis86_private.h"
#include "instr_tbl.h"
#include "util.h"
#include "symbols.h"
#include "config.h"
#include "labels.h"
#include "type.h"
#include "value.h"
#include "expr.h"
#include "transform.h"
#include "str.h"

#define LOG_INFO(fmt, ...) do { \
    fprintf(stderr, "INFO: "); \
    fprintf(stderr, fmt, ##__VA_ARGS__); \
    fprintf(stderr, "\n"); \
  } while(0)

#define LOG_WARN(fmt, ...) do { \
    fprintf(stderr, "WARN: "); \
    fprintf(stderr, fmt, ##__VA_ARGS__); \
    fprintf(stderr, "\n"); \
  } while(0)
