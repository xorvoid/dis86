# Dis86

Dis86 is a decompiler for 16-bit real-mode x86 DOS binaries. It has been built for doing reverse-engineering work such as
analyzing and re-implementing old DOS videogames from the early 1990s. The project is a work-in-progress and the development team makes no gaurentees
it will work or be useful out-of-the-box for any applications other than their own. Features and improvements are made on-demand as needed.

## Goals and Non-goals

Goals:

- Support reverse-engineering 16-bit real-mode x86 DOS binaries
- Generate code that is semantically correct (in so far as practical)
- Generate code that integrates will with a hybrid-runtime system (Hdyra) [currently unreleased]
- Avoid making many assumptions or using heuristics that can lead to broken decompiled code
- Be hackable and easy to extend as required
- Automate away common manual transformations and let a human reverser focus on the subjective tasks a computer cannot do well (e.g. naming things)

Non-goals:

- Output code beauty (semantic correctness is more important)
- Re-compilable to eqivalent binaries

Also, we generally prefer manual configuration/annotation tables to flawed heuristics that will generate incorrect code.

## Discussion of Internals

Discussion of the internals will be published periodically on the author's blog: [xorvoid](https://www.xorvoid.com)

## Building

Assuming you have rust and cargo installed:

```
just build
```

## Some Commands

Emit Disassembly:

```
./target/debug/dis86 --config <your_config.bsl> --binary <raw_text_segment> --name <function_name> --emit-dis <output-file>
```

Emit initial Intermediate Representation (IR):

```
./target/debug/dis86 --config <your_config.bsl> --binary <raw_text_segment> --name <function_name> --emit-ir-initial <output-file>
```

Emit final (optimized) Intermediate Representation (IR):

```
./target/debug/dis86 --config <your_config.bsl> --binary <raw_text_segment> --name <function_name> --emit-ir-final <output-file>
```

Visualize the control-flow grpah with graphviz:

```
./target/debug/dis86 --config <your_config.bsl> --binary <raw_text_segment> --name <function_name> --emit-graph /tmp/ctrlflow.dot
dot -Tpng /tmp/ctrlflow.dot > /tmp/control_flow_graph.png
open /tmp/control_flow_graph.png
```

Emit inferred higher-level control-flow structure:

```
./target/debug/dis86 --config <your_config.bsl> --binary <raw_text_segment> --name <function_name> --emit-ctrlflow <output-file>
```

Emit an Abstract Syntax Tree (AST):

```
./target/debug/dis86 --config <your_config.bsl> --binary <raw_text_segment> --name <function_name> --emit-ast <output-file>
```

Emit C code:

```
./target/debug/dis86 --config <your_config.bsl> --binary <raw_text_segment> --name <function_name> --emit-code <output-file>
```

## Caveats & Limitations

Primary development goal is to support an ongoing reverse-engineering and reimplementation project. The decompilier is also designed to
emit code that integrates well with a hybrid runtime system (called Hydra) that is used to run a partially-decompiled / reimplemnented
project. As such, uses that fall out of this scope have been unconsidered and may have numerous unknown issues.

Some specific known limitations:

- The decompiler accepts only a flat binary region for the text segment. It doesn't handle common binary file-formats (e.g. MZ) at the moment.
- Handling of many 8086 opcodes are unimplemented in the assembly->ir build step. Implementations are added as needed.
- Handling of some IR ops are unimplemented in the ir->ast convert step. Implementations are added as needed.
- Control-flow synthesis is limited to while-loops, if-stmts, and switch-stmts. If-else is unimplemented.
- Block scheduling and placement is very unoptimal for more complicated control-flow.
- ... and many more ...

## Future Plans / Wishlist

Feature wishlist

- Array accesses
- Compound types (struct and unions)
- Synthesizing struct/union member access
- If-else statements
- Pointer analysis and arithmetic
- More "u16 pair -> u32" fusing
- Improved type-aware IR
- Less verbose output C code patterns for common operations (e.g. passing pointer as a function call arg)

## Prehistoric versions

Dis86 began life as a simple disassembler and 1-to-1 instruction => C-statement decompiler. Overtime it gained complexity to the point that
it was then rearchitected with a proper SSA IR and series of transformations.

The older versions remain in the repo under `old/`. In particular, `old/v2` was much less sophisticated albiet more complete in terms of the input machine-code it could handle.

These versions remain as sometimes they are still useful when the latest version is missing some feature.
