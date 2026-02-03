# Compilation Pipeline

## Overview

HBF uses a three-stage compilation pipeline:

```
HBF Source (.hbf) → BFO Object (.bfo) → Brainfuck (.bf)
```

## Stage 1: HBF → BFO (Frontend)

The frontend compiler:
1. **Lexical Analysis**: Tokenizes source code
2. **Parsing**: Builds Abstract Syntax Tree (AST)
3. **Semantic Analysis**: Type checking, scope resolution
4. **Code Generation**: Emits BFO instructions

### Function Handling

#### Predictable Functions
Functions with compile-time analyzable behavior are preserved:
```c
void add_cells(cell a, cell b) {
    cell c = a + b;
    putc(c);
}
```
↓
```
func add_cells(a, b) {
    set c a
    add c b
    print c
}
```

#### Unpredictable Functions
Functions with runtime-dependent behavior are **inlined**:
```c
void print_string(string s) {
    for (int i = 0; i < s.length; i++) {
        cell c = s[i];
        putc(c);
    }
}
print_string("Hi");
```
↓
```
; Inlined due to string parameter
set g1 'H'
print g1
set g1 'i'
print g1
```

### Variable Scoping
- Local variables → Stack allocation in BFO
- Loop counters → May become global if needed across function boundaries
- Global variables → Declared at BFO file scope

## Stage 2: BFO → BF (Backend)

The backend compiler:
1. **Memory Allocation**: Assigns tape cells to variables
2. **Stack Management**: Implements function call stack
3. **Code Generation**: Translates BFO to raw Brainfuck

### Memory Layout
```
[global vars][stack frame 1][stack frame 2]...
```

### Instruction Translation

#### `set x 10`
```bf
>>>>>[-]++++++++++ ; Move to x's cell, clear, add 10
```

#### `add x y`
```bf
>>[-<+>]< ; Copy y to x (destructive)
```

#### `print x`
```bf
>. ; Move to x, output
```

#### `while x { ... }`
```bf
>[...] ; Loop on x's cell
```

## Optimization Opportunities

### BFO Level
- **Constant folding**: Evaluate constant expressions at compile time
- **Dead variable elimination**: Remove unused `int` variables
- **Countdown loop optimization**: `for (i=0; i<n; i++)` → countdown pattern
- Constant propagation
- Common subexpression elimination

### BF Level
- Pointer movement optimization (`>>>` instead of `>` `>` `>`)
- Loop unrolling
- Cell reuse

## Example: Full Pipeline

**HBF:**
```c
int a = 5;
int b = 10;
cell c = a + b;
putc(c);
```

**BFO:**
```
set a 5
set b 10
set c a
add c b
print c
```

**BF:**
```bf
+++++>++++++++++>[-]<<[->>+<<]>>.
```

## Debugging

### View BFO Output
```bash
cargo run -- build example.hbf
cat example.bfo  # Inspect intermediate representation
```

### Trace Execution
BFO is human-readable, making it easier to debug than raw Brainfuck.
