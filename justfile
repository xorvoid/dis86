#!/usr/bin/env just --justfile

# List all available recipies
list:
  just --list

# Build the repository
build:
  cargo build

# Test the repository
test:
  cargo test

# Show control-flow-graph using graphviz
vis name:
     just run {{name}} --emit-graph /tmp/ctrlflow.dot && dot -Tpng /tmp/ctrlflow.dot > /tmp/control_flow_graph.png && open /tmp/control_flow_graph.png

# A temporary command for dev build-test-cycle
run name *opts: build
     ./target/debug/dis86 --config ../gizmo/build/src/hooklib/dis86_config.bsl --binary ../gizmo/dis/exe.bin --name {{name}} {{opts}}

# A temporary command for dev build-test-cycle
run-old:
     ./old/v2/build/src/app/dis86 decomp --config ../gizmo/build/src/hooklib/dis86_config.bsl --binary ../gizmo/dis/exe.bin --start-addr 0622:0922 --end-addr 0622:09e5

rundiff a b: build
     just run --emit-{{a}} /tmp/a
     just run --emit-{{b}} /tmp/b
     opendiff /tmp/a /tmp/b