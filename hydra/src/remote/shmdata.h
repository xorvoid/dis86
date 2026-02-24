#pragma once
#include "header.h"

typedef struct shmdata shmdata_t;

struct __attribute__((packed)) shmdata
{
  u32 init;
  u32 end;
  u32 pid;
  u64 req;  // request step by incrementing
  u64 ack;  // ack step by matching 'req' value

  // registers
  u16 ax;
  u16 bx;
  u16 cx;
  u16 dx;
  u16 si;
  u16 di;
  u16 bp;
  u16 sp;
  u16 ip;
  u16 cs;
  u16 ds;
  u16 es;
  u16 ss;
  u16 flags;

  // memory
  // TODO...
};

shmdata_t *shmdata_create(const char *path);
shmdata_t *shmdata_attach(const char *path);
