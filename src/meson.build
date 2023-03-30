subdir('cmdarg')

SRC = [
  'core.c',
  'decode.c',
  'instr.c',
  'datamap.c',
  'print_intel_syntax.c',
  'decompile/decompile.c',
]

libdis86 = static_library(
  'dis86', SRC,
  link_with: libcmdarg)

subdir('app')
subdir('test')
