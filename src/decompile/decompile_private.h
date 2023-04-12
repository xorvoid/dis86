#pragma once
#include "dis86_private.h"
#include "instr_tbl.h"
#include "util.h"
#include "var.h"
#include "config.h"
#include "labels.h"

#define LOG_INFO(fmt, ...) do { \
    fprintf(stderr, "INFO: "); \
    fprintf(stderr, fmt, __VA_ARGS__); \
    fprintf(stderr, "\n"); \
  } while(0)
