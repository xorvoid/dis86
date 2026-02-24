#include "shmdata.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/mman.h>
#include <sys/stat.h>

shmdata_t *shmdata_create(const char *path)
{
  size_t size = (sizeof(shmdata_t) + 4095) & ~4095;

  int fd = open(path, O_RDWR | O_CREAT | O_TRUNC, 0600);
  if (fd < 0) {
    perror("open");
    return  NULL;
  }

  if (ftruncate(fd, (off_t)size) < 0) {
    perror("ftruncate");
    close(fd);
    unlink(path);
    return NULL;
  }

  void *addr = mmap(NULL, size, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);
  if (addr == MAP_FAILED) {
    perror("mmap");
    close(fd);
    unlink(path);
    return NULL;
  }

  close(fd);
  return (shmdata_t*)addr;
}

shmdata_t *shmdata_attach(const char *path)
{
  size_t size = (sizeof(shmdata_t) + 4095) & ~4095;

  int fd = open(path, O_RDWR, 0600);
  if (fd < 0) {
    perror("open");
    return NULL;
  }

  void *addr = mmap(NULL, size, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);
  if (addr == MAP_FAILED) {
    perror("mmap");
    close(fd);
    return NULL;
  }

  return (shmdata_t*)addr;
}
