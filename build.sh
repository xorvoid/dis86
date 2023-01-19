#!/bin/bash
set -e

CFLAGS="-std=c99 -Wall -Werror -Wno-unused-variable -Wno-unused-function -O2 -g"

SRC="main.c"
clang $CFLAGS -o dis_8086 $SRC

clang $CFLAGS -o test test.c
