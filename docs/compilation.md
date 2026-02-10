# Compilation Pipeline

The HBF compiler uses a multi-stage pipeline to transform high-level code into optimized Brainfuck.

## Pipeline Overview

```
┌──────────────────────────────────────────────────────────────┐
│                    FRONTEND: HBF → BFO                        │
├──────────────────────────────────────────────────────────────┤
│                                                               │
│  HBF Source (.hbf)                                           │
│       ↓                                                       │
│  Lexer (hbf_lexer.rs)         - Tokenization                │
│       ↓                                                       │
│  Parser (hbf_parser.rs)       - AST Construction            │
│       ↓                                                       │
│  BFO Generator (bfo_gen/)     - Optimization & Codegen      │
│    ├── expr_fold.rs           - Constant folding            │
│    ├── scope.rs               - Variable management         │
│    ├── stmt_gen.rs            - Code generation             │
│    ├── inline.rs              - Function inlining           │
│    └── emit.rs                - BFO emission                │
│       ↓                                                       │
│  BFO IR (.bfo)                                               │
│                                                               │
└──────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│                   BACKEND: BFO → Brainfuck                    │
├──────────────────────────────────────────────────────────────┤
│                                                               │
│  BFO IR (.bfo)                                               │
│       ↓                                                       │
│  BFO Lexer (bfo_lexer.rs)     - BFO Tokenization            │
│       ↓                                                       │
│  BFO Parser (bfo_parser.rs)   - BFO AST Construction        │
│       ↓                                                       │
│  BFO Compiler (bfo_compiler.rs) - Memory Allocation         │
│       ↓                                                       │
│  Internal IR (ir.rs)          - Brainfuck Operations        │
│       ↓                                                       │
│  Codegen (bf_codegen.rs)     - BF Character Generation     │
│       ↓                                                       │
│  Brainfuck (.bf)                                             │
│                                                               │
└──────────────────────────────────────────────────────────────┘
```

## Stage 1: HBF → BFO (Frontend)

The frontend performs aggressive optimizations while lowering HBF to BFO.

### 1.1 Lexical Analysis

**Module**: `hbf_lexer.rs`

Converts source text into tokens:
```c
int x = 10;
```
↓
```
[Keyword(Int), Identifier("x"), Equals, Number(10), Semicolon]
```

### 1.2 Parsing

**Module**: `hbf_parser.rs`

Builds Abstract Syntax Tree:
```
VarDecl {
    var_type: Type::Int,
    name: "x",
    value: Expr::Number(10)
}
```

### 1.3 Optimization & Code Generation

**Module**: `bfo_gen/` (modularized)

This is where the magic happens. The BFO generator performs multiple optimizations simultaneously:

#### Virtual Variable Elimination

**Input (HBF)**:
```c
int a = 5;
int b = 10;
cell c = a + b;
putc(c);
```

**Process**:
1. `int a = 5` → Store in scope stack (no BFO emitted)
2. `int b = 10` → Store in scope stack (no BFO emitted)
3. `cell c = a + b` → Fold `a + b` to `15`, emit `new c 15`
4. `putc(c)` → Emit `print c`

**Output (BFO)**:
```bfo
new c 15
print c
```

**Result**: Virtual variables `a` and `b` occupy **zero tape cells**.

#### Constant Folding

**Module**: `expr_fold.rs`

Evaluates expressions at compile-time:

```c
int result = (5 + 10) * 2 - 3;
// Folded to: 27
```

Supports:
- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Comparison: `==`, `!=`, `<`, `<=`, `>`, `>=`
- Logical: `&&`, `||`
- Array indexing: `"Hello"[0]` → `'H'`
- Member access: `"Hello".length` → `5`

#### Loop Unrolling

**Module**: `stmt_gen.rs`

Unrolls loops with constant bounds:

**Input (HBF)**:
```c
char[] msg = "Hi";
for (int i = 0; i < msg.length; i++) {
    putc(msg[i]);
}
```

**Process**:
1. Detect constant bound: `msg.length` = `2`
2. Simulate loop iterations:
   - i=0: `putc(msg[0])` → `putc('H')` → `print 'H'`
   - i=1: `putc(msg[1])` → `putc('i')` → `print 'i'`
3. Emit unrolled body

**Output (BFO)**:
```bfo
print 'H'
print 'i'
```

**Benefit**: Zero loop overhead, minimal BFO output.

#### Function Inlining

**Module**: `inline.rs`

Inlines functions with virtual parameters:

**Input (HBF)**:
```c
void print_digit(int n) {
    putc(48 + n);
}

print_digit(5);
```

**Process**:
1. Detect function call with virtual parameter
2. Evaluate argument: `5`
3. Inline function body with substitution
4. Fold expression: `48 + 5` → `53`
5. Emit optimized code

**Output (BFO)**:
```bfo
print 53
```

#### Scope Management

**Module**: `scope.rs`

Maintains variable scopes:
- Stack of hash maps for virtual variables
- Supports variable shadowing
- Tracks physical cells and arrays

#### Code Emission

**Module**: `emit.rs`

Generates BFO instructions:
- `materialize_to_cell()` - Converts expressions to cells
- Optimizes patterns like `A = A + B` → `add A B`
- Manages indentation and formatting

### 1.4 BFO Output

The result is a clean, optimized BFO file:

```bfo
; Virtual variables eliminated
; Loops unrolled
; Functions inlined
; Expressions folded

new result 42
print result
```

## Stage 2: BFO → Brainfuck (Backend)

The backend compiles BFO to executable Brainfuck.

### 2.1 BFO Parsing

**Modules**: `bfo_lexer.rs`, `bfo_parser.rs`

Parse BFO instructions into AST for compilation.

### 2.2 Memory Allocation

**Module**: `bfo_compiler.rs`

Key responsibilities:
1. **Cell Allocation**: Assign tape positions to variables
2. **Scope Management**: Handle block scopes `{ }`
3. **Pointer Tracking**: Track current tape position
4. **Cell Reuse**: Reuse freed cells within scopes

**Example**:
```bfo
new x 10    ; Allocate cell 0 for x
new y 5     ; Allocate cell 1 for y
{
    new temp 3  ; Allocate cell 2 for temp
    free temp   ; Mark cell 2 as free
}
new z 7     ; Reuse cell 2 for z
```

### 2.3 IR Generation

**Module**: `ir.rs`

Converts BFO to internal representation:

```bfo
new x 10
add x 5
print x
```
↓
```rust
[
    MoveRight(1),   // Move to cell 0
    Add(10),        // Set to 10
    Add(5),         // Add 5
    Output,         // Print
]
```

### 2.4 Brainfuck Generation

**Module**: `bf_codegen.rs`

Expands IR to Brainfuck characters:

```rust
MoveRight(3) → ">>>"
Add(10)      → "++++++++++"
Output       → "."
Loop([...])  → "[...]"
```

**Final Output**:
```brainfuck
>+++++++++++++.
```

## Optimization Summary

| Optimization | Stage | Module | Benefit |
|--------------|-------|--------|---------|
| Virtual Variables | Frontend | scope.rs | Zero tape footprint |
| Constant Folding | Frontend | expr_fold.rs | Compile-time evaluation |
| Loop Unrolling | Frontend | stmt_gen.rs | Zero loop overhead |
| Function Inlining | Frontend | inline.rs | Cross-function optimization |
| Shorthand Ops | Frontend | emit.rs | Atomic updates |
| Cell Reuse | Backend | bfo_compiler.rs | Minimal tape usage |

## Complete Example

**HBF Source**:
```c
void greet(char[] name) {
    putc("Hello, ");
    for (int i = 0; i < name.length; i++) {
        cell c = name[i];
        putc(c);
    }
    putc('!');
    putc('\n');
}

greet("World");
```

**BFO Output**:
```bfo
; Inlined greet("World")
{
    ; putc("Hello, ") - string literal unrolled
    print 'H'
    print 'e'
    print 'l'
    print 'l'
    print 'o'
    print ','
    print ' '
    
    ; Loop unrolled (name.length = 5)
    new c 'W'
    print c
    free c
    
    new c 'o'
    print c
    free c
    
    new c 'r'
    print c
    free c
    
    new c 'l'
    print c
    free c
    
    new c 'd'
    print c
    free c
    
    ; After loop
    print '!'
    print '\n'
}
```

**Brainfuck Output** (conceptual):
```brainfuck
+++++++++[>++++++++<-]>.  ; 'H'
>+++++++[>+++++++<-]>.    ; 'e'
...
```

## Debugging the Pipeline

### View BFO Output

```bash
cargo run -- compile example.hbf
cat example.bfo
```

### View Brainfuck Output

```bash
cargo run -- build example.hbf
cat example.bf
```

### Trace Compilation

Add debug prints in generator modules:
```rust
// In stmt_gen.rs
eprintln!("Generating statement: {:?}", stmt);
```

## Performance Characteristics

| Stage | Time Complexity | Space Complexity |
|-------|----------------|------------------|
| Lexing | O(n) | O(n) |
| Parsing | O(n) | O(n) |
| BFO Generation | O(n) | O(n) |
| BFO Compilation | O(n) | O(v) where v = variables |
| Codegen | O(n) | O(n) |

**Overall**: O(n) linear time, where n is source code size.

## Error Handling

Errors are reported at different stages:

1. **Lexer**: Invalid characters, malformed literals
2. **Parser**: Syntax errors with line numbers
3. **BFO Generator**: Type errors, undefined variables
4. **BFO Compiler**: Invalid instructions, undefined variables
5. **Codegen**: IR validation errors

## Future Improvements

1. **Separate Type Checker**: Validate types before code generation
2. **Multi-Pass Optimization**: Dedicated optimization passes
3. **Better Error Messages**: More context and suggestions
4. **Incremental Compilation**: Cache intermediate results
5. **Parallel Compilation**: Compile independent modules concurrently
