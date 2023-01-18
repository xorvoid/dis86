#!/bin/bash
SRC="dis_8086.c"
clang -std=c99 -Wall -Werror -Wno-unused-variable -Wno-unused-function -O2 -g -o dis_8086 $SRC
