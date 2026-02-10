# HBF Compiler Documentation

Welcome to the comprehensive documentation for the **Higher-Level Brainfuck (HBF) Compiler** - a modern, optimizing compiler that transforms C-like code into efficient Brainfuck.

## Quick Navigation

### 📚 Core Documentation
- **[Language Reference](language.md)** - Complete HBF syntax and features guide
- **[BFO Format](bfo.md)** - Intermediate representation specification
- **[Architecture](architecture.md)** - Compiler design and module structure
- **[Compilation Pipeline](compilation.md)** - How HBF becomes Brainfuck
- **[Optimizations](optimizations.md)** - Compiler optimization techniques
- **[Compiler Internals](compiler_internals.md)** - Implementation details
- **[Naming Conventions](naming_convention.md)** - BFO naming rules

### 🚀 Quick Start

#### Installation
```bash
git clone https://github.com/Kunta-tech/hbf.git
cd hbf
cargo build --release
```

#### Your First Program

Create `hello.hbf`:
```c
putc("Hello, World!\n");
```

Compile and run:
```bash
cargo run -- build hello.hbf
# Run hello.bf in any Brainfuck interpreter
```

#### Understanding the Output

View the intermediate BFO:
```bash
cargo run -- compile hello.hbf
cat hello.bfo
```

Output:
```bfo
print 'H'
print 'e'
print 'l'
print 'l'
print 'o'
print ','
print ' '
print 'W'
print 'o'
print 'r'
print 'l'
print 'd'
print '!'
print '\n'
```

## Key Concepts

### Virtual vs Physical Types

**Virtual Types** (`int`, `char`, `bool`):
- Exist only at compile-time
- Zero tape footprint
- Fully optimized away

**Physical Types** (`cell`, `cell[]`):
- Occupy Brainfuck tape cells
- Persistent storage
- Used for I/O and runtime state

### Example
```c
int x = 10;        // Virtual - no tape space
cell c = x + 5;    // Physical - uses 1 tape cell
putc(c);           // Outputs: ASCII 15
```

Generated BFO:
```bfo
new c 15           // Only 'c' exists in BFO
print c
```

## Compiler Pipeline

```
HBF Source (.hbf)
    ↓ Lexer & Parser
HBF AST
    ↓ BFO Generator (optimized)
BFO IR (.bfo)
    ↓ BFO Compiler
Internal IR
    ↓ Codegen
Brainfuck (.bf)
```

## Major Features

### 1. Constant Folding
```c
int result = (5 + 10) * 2;
cell c = result;
putc(c);
```
→ `print 30`

### 2. Loop Unrolling
```c
for (int i = 0; i < 3; i++) {
    putc('A');
}
```
→ `print 'A'` × 3

### 3. Function Inlining
```c
void print_digit(int n) {
    putc(48 + n);
}
print_digit(5);
```
→ `print 53`

### 4. Virtual Arrays
```c
char[] msg = "Hi";
for (int i = 0; i < msg.length; i++) {
    putc(msg[i]);
}
```
→ `print 'H'` + `print 'i'`

## Examples

The `examples/` directory contains comprehensive test cases:

1. **[01_hello_world.hbf](file:///s:/vs%20code/projects/hbf/examples/01_hello_world.hbf)** - Basic output
2. **[02_virtual_vars.hbf](file:///s:/vs%20code/projects/hbf/examples/02_virtual_vars.hbf)** - Virtual variable folding
3. **[03_global_folding.hbf](file:///s:/vs%20code/projects/hbf/examples/03_global_folding.hbf)** - Global constant resolution
4. **[04_loop_unrolling.hbf](file:///s:/vs%20code/projects/hbf/examples/04_loop_unrolling.hbf)** - Loop optimization
5. **[05_native_loops.hbf](file:///s:/vs%20code/projects/hbf/examples/05_native_loops.hbf)** - Runtime loops with `forn`
6. **[06_function_inlining.hbf](file:///s:/vs%20code/projects/hbf/examples/06_function_inlining.hbf)** - Function optimization
7. **[07_physical_arrays.hbf](file:///s:/vs%20code/projects/hbf/examples/07_physical_arrays.hbf)** - Tape-resident arrays
8. **[08_shorthand_assign.hbf](file:///s:/vs%20code/projects/hbf/examples/08_shorthand_assign.hbf)** - Optimized updates
9. **[09_complex_expressions.hbf](file:///s:/vs%20code/projects/hbf/examples/09_complex_expressions.hbf)** - Nested arithmetic
10. **[10_hello_world.hbf](file:///s:/vs%20code/projects/hbf/examples/10_hello_world.hbf)** - Complete program
11. **[11_bools.hbf](file:///s:/vs%20code/projects/hbf/examples/11_bools.hbf)** - Boolean logic
12. **[12_test_else_if.hbf](file:///s:/vs%20code/projects/hbf/examples/12_test_else_if.hbf)** - Conditional chains
13. **[13_multi_decl.hbf](file:///s:/vs%20code/projects/hbf/examples/13_multi_decl.hbf)** - Multi-declarations

## CLI Commands

### Compile to BFO
```bash
cargo run -- compile <file.hbf>
```
Generates `<file>.bfo` intermediate representation.

### Build to Brainfuck
```bash
cargo run -- build <file.hbf>
```
Generates both `<file>.bfo` and `<file>.bf`.

### Run Tests
```bash
cargo test
```

## Module Structure

The compiler is organized into focused modules:

```
src/
├── hbf_lexer.rs       - HBF tokenization
├── hbf_parser.rs      - HBF parsing
├── hbf_ast.rs         - HBF AST definitions
├── bfo_gen/           - BFO generator (modularized)
│   ├── mod.rs         - Main generator & API
│   ├── scope.rs       - Variable management
│   ├── emit.rs        - Code emission
│   ├── expr_fold.rs   - Constant folding
│   ├── stmt_gen.rs    - Statement generation
│   └── inline.rs      - Function inlining
├── bfo_lexer.rs       - BFO tokenization
├── bfo_parser.rs      - BFO parsing
├── bfo_compiler.rs    - BFO compilation
├── ir.rs              - Internal IR
├── bf_codegen.rs     - Brainfuck generation
└── main.rs            - CLI entry point
```

## Learning Path

### Beginner
1. Read [Language Reference](language.md) - Learn HBF syntax
2. Try examples 01-04 - Understand basic features
3. View BFO output - See how code is optimized

### Intermediate
4. Read [BFO Format](bfo.md) - Understand intermediate representation
5. Read [Compilation Pipeline](compilation.md) - Learn compilation stages
6. Try examples 05-09 - Explore advanced features

### Advanced
7. Read [Architecture](architecture.md) - Understand compiler design
8. Read [Compiler Internals](compiler_internals.md) - Study implementation
9. Read [Optimizations](optimizations.md) - Deep dive into optimizations
10. Contribute to the project!

## Common Patterns

### String Output
```c
putc("Hello\n");
```

### Loops
```c
// Compile-time unrolling
for (int i = 0; i < 5; i++) {
    putc('*');
}

// Runtime countdown
forn(10) {
    putc('!');
}
```

### Functions
```c
void greet(char[] name) {
    putc("Hello, ");
    for (int i = 0; i < name.length; i++) {
        putc(name[i]);
    }
    putc('!');
}

greet("World");
```

### Arrays
```c
// Virtual array (compile-time)
char[] msg = "Hi";
putc(msg[0]);  // Outputs: 'H'

// Physical array (runtime)
cell[] data = {1, 2, 3};
putc(data[0]);
```

## Debugging Tips

### View Intermediate Output
```bash
cargo run -- compile example.hbf
cat example.bfo
```

### Check Optimization
Compare source and BFO line counts:
```bash
wc -l example.hbf example.bfo
```

### Trace Compilation
Add debug prints in generator modules (temporary):
```rust
eprintln!("Folding: {:?}", expr);
```

## Performance

| Metric | Value |
|--------|-------|
| Compilation Speed | O(n) linear |
| Virtual Variable Overhead | 0 bytes |
| Loop Unrolling Limit | 10,000 iterations |
| Optimization Passes | Single-pass |

## Contributing

We welcome contributions! Areas of interest:
- Additional optimizations
- Better error messages
- More examples
- Documentation improvements
- Bug fixes

## Resources

- **Source Code**: [GitHub Repository](https://github.com/Kunta-tech/hbf)
- **Issue Tracker**: [GitHub Issues](https://github.com/Kunta-tech/hbf/issues)
- **Brainfuck Reference**: [Esolangs Wiki](https://esolangs.org/wiki/Brainfuck)

## License

[MIT License](https://github.com/Kunta-tech/hbf/blob/main/LICENSE)

---

**Happy Compiling!** 🦀🧠
