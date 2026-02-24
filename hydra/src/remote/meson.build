
SRC = [
  'api.c',
  'remote.c',
  'shmdata.c',
] + HYDRA_SRC

libhydraremote = shared_library(
  'hydraremote', SRC,
  link_with: libhydra,
)
