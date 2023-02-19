#!/bin/bash
set -e

CFLAGS="-std=c99 -Wall -Werror -Wno-unused-variable -Wno-unused-function -O2 -g"

LIB_SRC="dis86.c decode.c instr.c print.c"

SRC="main.c $LIB_SRC"
clang $CFLAGS -o dis86 $SRC

SRC="test.c $LIB_SRC"
clang $CFLAGS -o test $SRC
