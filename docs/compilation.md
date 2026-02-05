# Compilation Pipeline

## Overview

HBF uses a multi-stage compilation pipeline:

```
HBF Source (.hbf) → BFO Object (.bfo) → [Brainfuck (.bf)]
```

> [!NOTE]
> Brainfuck generation (.bf) is currently the final backend stage and is a work-in-progress.

## Stage 1: HBF → BFO (Frontend)

The frontend compiler (`hbf compile`) performs:
1. **Lexical Analysis**: Tokenizes source code into `lexer::Token`s.
2. **Parsing**: Constructs an Abstract Syntax Tree (AST).
3. **Semantic Analysis**: Type checking and scope resolution.
4. **Optimization**: Constant folding and virtual variable elimination.
5. **Code Generation**: Emits assembly-like BFO instructions.

### Variable Materialization

HBF makes a strict distinction between **Virtual** (`int`, `char`) and **Physical** (`cell`) types.
- **Virtual variables** are "Always-Virtual": they exist only in the compiler's symbol table and are folded into BFO instructions. They occupy ZERO tape space.
- **Physical variables** (`cell`) are materialized into BFO `set` and `add` instructions.

### Function Handling

#### Deterministic Inlining
Functions taking strictly **virtual** parameters (`int`, `char`, `int[]`, `char[]`) are **automatically inlined**. This allows the compiler to resolve runtime arithmetic to literals at each specific call site.

#### BFO Functions
Only functions taking strictly **physical** `cell` parameters are preserved as `func` definitions in the BFO output.

**HBF:**
```c
void add_cells(cell val1, cell val2) {
    cell res = val1 + val2;
    putc(res);
}
```
↓
**BFO:**
```
func add_cells(val1, val2) {
    set res 0      ; Add-to-Zero pattern for variable moves
    add res val1
    add res val2
    print res
}
```

## Stage 2: BFO → BF (Backend)

The backend handles the low-level Brainfuck mapping:
1. **Memory Allocation**: Assigns tape cells to BFO variables.
2. **Stack Management**: Implements the function call stack.
3. **Instruction Translation**: Translates `set`, `add`, `sub`, `while`, and `print` into raw `+`, `-`, `>`, `<`, `[`, `]`, `.` characters.

### Instruction Translation Example

| BFO | Brainfuck (Conceptual) |
|-----|-------------------|
| `set x 10` | `>>>>>[-]++++++++++` |
| `add x y` | `>>[-<+>]<` |
| `print x` | `>.` |
| `while x { ... }` | `> [...]` |

## Optimization Highlights

- **Constant Folding**: `int a = 5 + 10;` results in NO code; the compiler simply tracks `a` as `15`.
- **Loop Unrolling**: `for (int i=0; i<5; i++)` is replaced by the body statements repeated 5 times in the BFO.
- **Dead Variable Elimination**: Virtual variables that aren't materialized for I/O or Physical assignments are silently removed.

## Debugging

To inspect the intermediate representation, use the `compile` command:
```bash
cargo run -- compile example.hbf
cat example.bfo
```
