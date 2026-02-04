# HBF Compiler: Loose HBF → Strict BFO

A Higher-Level Language to Brainfuck Compiler platform written in Rust. HBF provides an expressive, C-like source language that lowers into a strictly predetermined, ultra-optimized intermediate representation (BFO).

## The Stability Contract

HBF is designed as a stable platform for Brainfuck developers:
- **Loose HBF Source**: Write flexible code with global variables, nested expressions, and function calls.
- **Strict BFO Output**: The compiler erases abstraction overhead, resolving variables and loops into a minimalist BFO instruction set that is 100% ready for Brainfuck backends.

## Key Features

- **Always-Virtual Types**: `int` and `char` variables exist only during compilation, allowing for complex intermediate logic with **zero tape footprint**.
- **Global Virtual Folding**: Automatically resolves global constants and arithmetic into function-local literals at compile-time.
- **Loop Unrolling**: Erases constant-bound `for` loops, converting array indexing into fixed cell references.
- **Direct I/O Printing**: Prints literals and named cells directly, bypassing redundant materialization to temporary cells.
- **Add-to-Zero Materialization**: Implements a robust variable copy pattern (`set target 0; add target src`) to satisfy BFO's literal-only `set` constraint.

## Usage

### Compile to BFO
```bash
cargo run -- compile examples/01_hello_world.hbf
```

### Batch Test All Examples
```bash
cargo run -- test-all
```

## Refined Example Suite

The `examples/` directory contains a structured set of test cases for verifying the compiler:

1.  `01_hello_world.hbf`: Direct literal printing.
2.  `02_virtual_vars.hbf`: Virtual type folding.
3.  `03_global_folding.hbf`: Global constant resolution in functions.
4.  `04_loop_unrolling.hbf`: Unrolling with `.length` and array indexing.
5.  `05_native_loops.hbf`: High-efficiency native countdowns.
6.  `06_function_inlining.hbf`: Inlining + Unrolling of array-heavy functions.
7.  `07_physical_arrays.hbf`: Tape cell management and indexing.
8.  `08_shorthand_assign.hbf`: `A = A + i` -> `add A i` optimization.
9.  `09_complex_expressions.hbf`: Deep virtual arithmetic resolution.

## Future Prospects

- **BFO to BF Backend**: Lowering Brainfuck Object files to raw Brainfuck using efficient pointer-based tape management and cell allocation.
- **Global Dead Code Elimination**: Post-compilation optimization to remove unused functions and global variables from the final output.
- **Tape Cell Reuse**: Implementing a more sophisticated stack/heap model to minimize Brainfuck tape footprint by reusing cells for intermediate locals.
- **Standard Library**: A set of pre-compiled HBF modules for string math, and advanced memory management.
- **Inter-module Linking**: A dedicated linker to combine multiple `.bfo` files into a single optimized `.bf` executable.
