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
visualize-ctrlflow:
     just run --emit-graph /tmp/ctrlflow.dot && dot -Tpng /tmp/ctrlflow.dot > /tmp/control_flow_graph.png && open /tmp/control_flow_graph.png

# A temporary command for dev build-test-cycle
run-dis: build
     ./target/debug/dis86 dis --binary ../gizmo/dis/exe.bin --start-addr 0622:0922 --end-addr 0622:09e5

# A temporary command for dev build-test-cycle
run *opts: build
     ./target/debug/dis86 decomp --config ../gizmo/build/src/hooklib/dis86_config.bsl --binary ../gizmo/dis/exe.bin --start-addr 0622:0922 --end-addr 0622:09e5 {{opts}}

# A temporary command for dev build-test-cycle
run-decomp *opts: build
     ./target/debug/dis86 decomp --config ../gizmo/build/src/hooklib/dis86_config.bsl --binary ../gizmo/dis/exe.bin --start-addr 0622:0922 --end-addr 0622:09e5 --emit-code -

# A temporary command for dev build-test-cycle
run-old:
     ./old/v2/build/src/app/dis86 decomp --config ../gizmo/build/src/hooklib/dis86_config.bsl --binary ../gizmo/dis/exe.bin --start-addr 0622:0922 --end-addr 0622:09e5
