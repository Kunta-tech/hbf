# HBF Language Reference

HBF (Higher Brainfuck) is a C-like compiled language that targets Brainfuck through an intermediate representation (BFO).

## Types

### Primitive Types
- **`int`**: Integer type (8-bit, mod 256)
- **`cell`**: Raw memory cell type (8-bit, mod 256)
- **`string`**: String type (compile-time constant)
- **`void`**: Function return type (no return value)

### Type Semantics
- All numeric values are stored as 8-bit values (0-255)
- `int` and `cell` are functionally equivalent at runtime but semantically distinct
- `cell` is used for raw byte manipulation and I/O
- `int` is used for arithmetic and loop counters

## Variables

Variables must be declared with explicit types:

```c
int x = 10;
cell c = 65;
```

### Scope
- Variables are scoped to their enclosing block `{}`
- Function parameters are local to the function
- Global variables can be declared at file scope

## Literals

### Integer Literals
```c
int x = 42;
```

### Character Literals
```c
cell c = 'A';      // ASCII value 65
cell newline = '\n'; // Escape sequences supported
```

### String Literals
```c
"Hello, World!\n"
```

## Operators

### Arithmetic
- `+` Addition
- `-` Subtraction

### Comparison (in loops)
- `<` Less than
- `>` Greater than

### Assignment
- `=` Assignment

### Array Access
- `s[i]` String/array indexing

## Control Flow

### For Loops (Loop Unrolling)

Standard for loops with **constant bounds** are **unrolled** at compile time:

```c
for (int i = 0; i < 5; i++) {
    putc('A');
}
// Compiler generates 5 sequential putc('A') statements
```

**Requirements for unrolling:**
- Constant iteration count known at compile time
- Standard pattern: `for (int i = 0; i < CONSTANT; i++)`

**Compiled BFO:**
```
print 'A'
print 'A'
print 'A'
print 'A'
print 'A'
```

### forn Loops (Native Countdown)

For runtime-variable iteration counts or when you need a native loop, use `forn`:

```c
forn(cell n = 10) {
    putc('B');
}
// Generates a while loop that counts down from n to 0
```

**Syntax:**
- `forn(cell variable = value) { ... }`
- Variable must be `cell` type
- Value can be a constant or runtime variable
- Loop executes while variable is non-zero, decrementing each iteration

**Compiled BFO:**
```
set n 10
while n {
    print 'B'
    sub n 1
}
```

### While Loops

```c
while (condition) {
    // body
}
```

## Functions

### Function Declaration
```c
void function_name(type param1, type param2) {
    // body
}
```

### Function Calls
```c
function_name(arg1, arg2);
```

### Return Type
Currently only `void` functions are supported.

## Built-in Functions

### `putc(cell c)`
Outputs a single character to stdout. **Only accepts `cell` type.**

```c
cell c = 'H';
putc(c);
```

## String Operations

### `.length`
Returns the length of a string:
```c
for (int i = 0; i < s.length; i++) {
    // ...
}
```

### Array Indexing `[i]`
Access individual characters:
```c
cell c = s[i];
```

## Comments

```c
// Single-line comment
```

## Complete Examples

### Example 1: Basic Arithmetic
```c
int a1 = 5;
int a2 = 10;
int a3 = a1 + a2;
cell c = a3;
putc(c);
```

### Example 2: Functions
```c
void add_cells(cell a, cell b) {
    cell c = a + b;
    putc(c);
}

void repeat_char(int n, cell c) {
    for (int i = 0; i < n; i++) {
        putc(c);
    }
}

add_cells(5, 10);
repeat_char(5, 'H');
```

### Example 3: String Processing
```c
void print_string(string s) {
    for (int i = 0; i < s.length; i++) {
        cell c = s[i];
        putc(c);
    }
}

print_string("Hello, World!\n");
```
