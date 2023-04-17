subdir('cmdarg')

SRC = [
  'core.c',
  'decode.c',
  'instr.c',
  'datamap.c',
  'print_intel_syntax.c',
  'decompile/decompile.c',
  'decompile/config.c',
  'decompile/type.c',
  'decompile/symbols.c',
  'decompile/value.c',
  'decompile/expr.c',
  'decompile/transform.c',
]

libdis86 = static_library(
  'dis86', SRC,
  link_with: libcmdarg,
  dependencies : libbsl_dep)

subdir('app')
subdir('test')
