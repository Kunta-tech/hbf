# HBF Compiler

A Higher-Level Language to Brainfuck Compiler written in Rust.

## User Preferences & Architecture

- **Language**: Rust
- **Pipeline**:
  1.  `.hbf` (Higher Brainfuck) Source
  2.  `.bfo` (Brainfuck Object) Intermediate Representation
  3.  `.bf` (Brainfuck) Final Output
- **Output**: The compiler generates both intermediate and final artifacts.

## Usage

### Compile to BFO
```bash
cargo run -- compile examples/test_array.hbf
```

### Batch Test All Examples
```bash
cargo run -- test-all
```

## Future Prospects

- **BFO to BF Backend**: Lowering Brainfuck Object files to raw Brainfuck using efficient pointer-based tape management and cell allocation.
- **Global Dead Code Elimination**: Post-compilation optimization to remove unused functions and global variables from the final output.
- **Tape Cell Reuse**: Implementing a more sophisticated stack/heap model to minimize Brainfuck tape footprint by reusing cells for intermediate locals.
- **Standard Library**: A set of pre-compiled HBF modules for string math, and advanced memory management.
- **Inter-module Linking**: A dedicated linker to combine multiple `.bfo` files into a single optimized `.bf` executable.
