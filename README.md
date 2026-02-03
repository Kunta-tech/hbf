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

```bash
cargo run -- build examples/test.hbf
```
