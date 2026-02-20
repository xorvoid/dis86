#!/usr/bin/env just --justfile

# List all available recipies
list:
  just --list
  
# Build the repository
build: build-dis86 build-hydra

# Build the dis86 component only
build-dis86:
  #!/bin/bash
  cd {{justfile_directory()}}
  (cd dis86 && cargo build)
  mkdir -p build/bin
  cp dis86/target/debug/dis86 build/bin/
  cp dis86/target/debug/mzfile build/bin/

# Build the hydra component only
build-hydra:
  #!/bin/bash
  cd {{justfile_directory()}}
  (cd hydra && just build)

# Test the repository
test *opts:
  (cd dis86 && cargo test {{opts}})
  (cd hydra && just test)
