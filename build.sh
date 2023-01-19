#!/bin/bash
set -e

CFLAGS="-std=c99 -Wall -Werror -Wno-unused-variable -Wno-unused-function -O2 -g"

SRC="main.c dis86.c decode.c print.c"
clang $CFLAGS -o dis86 $SRC

clang $CFLAGS -o test test.c
