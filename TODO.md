# HBF Roadmap & TODO

The goal of HBF is to become a high-level language capable of implementing complex algorithms from the ground up, supported by a rich standard library, while BFO serves as a consolidated intermediate object format.

## Core Language & Architecture
- [ ] **Turing Completeness**: Implement runtime dynamic indexing for arrays (`cell[cell]`).
- [x] **BFO Consolidation**: Consolidated IR/AST, implemented auto-memory management and modular function support.
- [ ] **Pointer Support**: Allow raw pointer manipulation for more direct tape control.
- [ ] **Better Dead Code Elimination**: Ensure BFO only contains instructions for code that is actually reached.

## Standard Library (stdlib)
- [ ] **Math Module**: Implement optimized `mul`, `div`, and `mod` for `cell` types.
- [ ] **String Manipulation**: Advanced `print_str`, `copy_str`, and `concat` (using cell recycling).
- [ ] **Memory Management**: A lightweight "allocator" logic for managing tape regions.
- [ ] **I/O Helpers**: Buffered input and formatted output.

## Tooling & Ecosystem
- [ ] **Linker**: A tool to consolidate multiple `.bfo` files into a single Brainfuck product.
- [ ] **Debugger**: A simulator that maps BFO/BF execution back to HBF source lines.
- [ ] **BFO Backend Optimizations**: Intelligent tape layout to minimize pointer movement.

## Documentation & Examples
- [x] Document the BFO object file specification.
- [ ] Create examples for complex algorithms (e.g., sorting, fibonacci).
