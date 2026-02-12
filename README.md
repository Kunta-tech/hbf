# HBF Compiler

A **staged partial-evaluation language with a cell-based IR targeting Brainfuck**. **HBF** (**Highlevel Brainfuck**) provides a C-like programming language that compiles to Brainfuck through an optimized intermediate representation called **BFO** (**Brainfuck Object**).

## Overview

HBF bridges the gap between high-level programming and Brainfuck by providing:
- **Familiar Syntax**: C-like language with variables, functions, loops, and arrays
- **Virtual Types**: Zero-footprint `int`, `char`, and `bool` types that exist only at compile-time
- **Aggressive Optimization**: Constant folding, polymorphic loop unrolling, and function inlining
- **Intermediate Representation**: Human-readable BFO format for debugging and optimization

## Quick Start

### Installation

```bash
git clone https://github.com/Kunta-tech/hbf.git
cd hbf
cargo build --release
```

### Usage

**Build complete Brainfuck executable:**
```bash
cargo run -- example.hbf
```

**Compile to BFO only:**
```bash
cargo run -- -c example.hbf
```

**Specify output filename:**
```bash
cargo run -- example.hbf -o custom.bf
```

**Compile BFO to BF:**
```bash
cargo run -- example.bfo
```

### Hello World Example

```c
// hello.hbf
putc("Hello, World!\n");
```

Compile and run:
```bash
cargo run -- hello.hbf
# Run hello.bf in any Brainfuck interpreter
```

## Key Features

### 1. Virtual Types (Zero Tape Footprint)

Variables of type `int`, `char`, and `bool` are **virtual** - they exist only during compilation and occupy zero Brainfuck tape cells.

```c
int a = 5;
int b = 10;
cell c = a + b;  // Only 'c' uses tape space
putc(c);         // Outputs: ASCII 15
```

**Generated BFO:**
```
new c 15    ; Compiler evaluated 5 + 10 at compile-time
print c
```

### 2. Constant Folding & Loop Unrolling

The compiler evaluates constant expressions and unrolls loops with known bounds:

```c
char[] msg = "Hi";
for (int i = 0; i < msg.length; i++) {
    putc(msg[i]);
}
```

**Generated BFO:**
```
print 'H'
print 'i'
```

### 3. Function Inlining

Functions with virtual parameters are automatically inlined and optimized:

```c
void print_digit(int n) {
    putc(48 + n);
}

print_digit(5);  // Outputs: '5'
```

**Generated BFO:**
```
print 53    ; Compiler evaluated 48 + 5
```

### 4. Physical Types & Procedural Math

Use `cell` and `cell[]` for variables that need persistent tape storage. To ensure efficiency, infix math (like `a + b`) is restricted for cells. Instead, use procedural primitives:

```c
cell a = 10;
cell b = 5;

add(a, b);       // a = a + b
sub(a, 1);       // a = a - 1
a++;             // Optimized increment (unified for virtual/physical)

cell c;
move(c, a);      // Efficient destructive transfer (a becomes 0)
```

Procedural primitives map to optimized Brainfuck loops, giving you fine-grained control over tape operations.


## Compilation Pipeline

```
┌─────────────┐
│ HBF Source  │  .hbf file with C-like syntax
│   (.hbf)    │
└──────┬──────┘
       │ hbf_lexer → hbf_parser
       ↓
┌─────────────┐
│  HBF AST    │  Abstract Syntax Tree
└──────┬──────┘
       │ bfo_gen/ (modularized)
       │  ├── scope.rs      - Variable management
       │  ├── expr_fold.rs  - Constant folding
       │  ├── stmt_gen.rs   - Code generation
       │  ├── inline.rs     - Function inlining
       │  └── emit.rs       - BFO emission
       ↓
┌─────────────┐
│  BFO IR     │  Brainfuck Object intermediate representation
│   (.bfo)    │
└──────┬──────┘
       │ bfo_parser → bfo_compiler
       ↓
┌─────────────┐
│  BF IR      │  Internal representation
└──────┬──────┘
       │ bf_codegen
       ↓
┌─────────────┐
│ Brainfuck   │  Final executable
│   (.bf)     │
└─────────────┘
```

## Project Structure

```
src/
├── hbf_lexer.rs       - Tokenizes HBF source
├── hbf_parser.rs      - Parses tokens into AST
├── hbf_ast.rs         - AST definitions
├── hbf_token.rs       - Token types
├── bfo_gen/           - BFO generator (modularized)
│   ├── mod.rs         - Main generator & public API
│   ├── scope.rs       - Variable scope management
│   ├── emit.rs        - BFO code emission
│   ├── expr_fold.rs   - Expression constant folding
│   ├── stmt_gen.rs    - Statement code generation
│   └── inline.rs      - Function inlining logic
├── bfo_lexer.rs       - Tokenizes BFO
├── bfo_parser.rs      - Parses BFO into AST
├── bfo_ast.rs         - BFO AST definitions
├── bfo_compiler.rs    - Compiles BFO to IR
├── ir.rs              - Internal representation
├── bf_codegen.rs     - Generates Brainfuck from IR
└── main.rs            - CLI entry point
```

## Language Features

### Types
- **Virtual**: `int`, `char`, `bool`, `int[]`, `char[]` - Compile-time only
- **Physical**: `cell`, `cell[]` - Occupy Brainfuck tape cells

### Control Flow
- `for` loops - Unrolled at compile-time for constant bounds
- `while` loops - Unrolled for virtual conditions; otherwise native Brainfuck loops
- `forn(n)` - Runtime countdown loops
- `if/else` - Compile-time evaluation when possible

### Functions
- Automatic inlining for virtual-parameter functions
- Physical functions preserved in BFO for `cell` parameters

## Examples

See the `examples/` directory for comprehensive examples:
- `01_hello_world.hbf` - Basic output
- `02_virtual_vars.hbf` - Virtual variable folding
- `03_global_folding.hbf` - Global constant resolution
- `04_loop_unrolling.hbf` - Loop optimization
- `05_native_loops.hbf` - Runtime loops with `forn`
- `06_function_inlining.hbf` - Function optimization
- `07_physical_arrays.hbf` - Tape-resident arrays
- And more...

## Documentation

Comprehensive documentation is available in the `docs/` directory:
- [Language Reference](docs/language.md) - Complete HBF syntax guide
- [BFO Format](docs/bfo.md) - Intermediate representation specification
- [Architecture](docs/architecture.md) - Compiler design and structure
- [Compilation Pipeline](docs/compilation.md) - How HBF becomes Brainfuck
- [Optimizations](docs/optimizations.md) - Compiler optimization techniques
- [Compiler Internals](docs/compiler_internals.md) - Implementation details

## Future Roadmap

- **Enhanced BFO Backend**: Improved tape allocation and pointer management
- **Dead Code Elimination**: Remove unused functions and variables
- **Cell Reuse**: Minimize tape footprint through intelligent allocation
- **Standard Library**: Pre-compiled modules for common operations
- **Module System**: Link multiple `.bfo` files into single executable

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## License

[MIT License](https://github.com/Kunta-tech/hbf/blob/main/LICENSE)

## Acknowledgments

Built with Rust 🦀
