# HBF Compiler Documentation

Welcome to the documentation for the Higher-Level Brainfuck (HBF) Compiler.

## Table of Contents

- [Language Reference](language.md): Learn the HBF syntax and features.
- [BFO Format](bfo.md): Understand the intermediate representation.
- [Compilation Pipeline](compilation.md): How HBF compiles to Brainfuck.
- [Optimizations](optimizations.md): Compiler optimizations explained.
- [Architecture & Internals](compiler_internals.md): Understand how the compiler works under the hood.

## Quick Start
1. **Write code** in a `.hbf` file:
   ```c
   int x = 10;
   cell c = x;
   putc(c);
   ```
2. **Compile** using the CLI:
   ```bash
   cargo run -- build your_file.hbf
   ```
3. **Run** the generated `.bf` file in any Brainfuck interpreter.

## Examples
The compiler includes several examples demonstrating its features:
1.  **[01_hello_world.hbf](file:///s:/vs%20code/projects/hbf/examples/01_hello_world.hbf)**: Simple output.
2.  **[02_virtual_vars.hbf](file:///s:/vs%20code/projects/hbf/examples/02_virtual_vars.hbf)**: Virtual variable folding.
3.  **[03_global_folding.hbf](file:///s:/vs%20code/projects/hbf/examples/03_global_folding.hbf)**: Top-level constant folding.
4.  **[04_loop_unrolling.hbf](file:///s:/vs%20code/projects/hbf/examples/04_loop_unrolling.hbf)**: Constant loop erasure.
5.  **[05_native_loops.hbf](file:///s:/vs%20code/projects/hbf/examples/05_native_loops.hbf)**: Runtime counting with `forn`.
6.  **[06_function_inlining.hbf](file:///s:/vs%20code/projects/hbf/examples/06_function_inlining.hbf)**: Deterministic inlining.
7.  **[07_physical_arrays.hbf](file:///s:/vs%20code/projects/hbf/examples/07_physical_arrays.hbf)**: Tape-resident cell arrays.
8.  **[08_shorthand_assign.hbf](file:///s:/vs%20code/projects/hbf/examples/08_shorthand_assign.hbf)**: Optimized atomic updates.
9.  **[09_complex_expressions.hbf](file:///s:/vs%20code/projects/hbf/examples/09_complex_expressions.hbf)**: Nested math resolution.
10. **[10_hello_world.hbf](file:///s:/vs%20code/projects/hbf/examples/10_hello_world.hbf)**: Complete program example.
11. **[11_bools.hbf](file:///s:/vs%20code/projects/hbf/examples/11_bools.hbf)**: Boolean type and logic.
12. **[12_test_else_if.hbf](file:///s:/vs%20code/projects/hbf/examples/12_test_else_if.hbf)**: Compile-time `if`/`else` chains.
13. **[13_multi_decl.hbf](file:///s:/vs%20code/projects/hbf/examples/13_multi_decl.hbf)**: Multi-declaration and C-style arrays.
