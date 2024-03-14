#!/bin/bash
cd ..
./target/debug/dis86 decomp --config ../gizmo/build/src/hooklib/dis86_config.bsl --binary ../gizmo/dis/exe.bin --start-addr 0622:0922 --end-addr 0622:09e5 --emit-code -
