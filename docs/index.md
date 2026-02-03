# HBF Compiler Documentation

Welcome to the documentation for the Higher-Level Brainfuck (HBF) Compiler.

## Table of Contents

- [Language Reference](language.md): Learn the HBF syntax and features.
- [BFO Format](bfo.md): Understand the intermediate representation.
- [Compilation Pipeline](compilation.md): How HBF compiles to Brainfuck.
- [Optimizations](optimizations.md): Compiler optimizations explained.
- [Architecture & Internals](architecture.md): Understand how the compiler works under the hood.

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
