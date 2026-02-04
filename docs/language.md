# HBF Language Reference

HBF (Higher Brainfuck) is a C-like compiled language that targets Brainfuck through an intermediate representation (BFO).

## Types

### Primitive Types
- **`int`**: Integer type (8-bit, mod 256)
- **`cell`**: Raw memory cell type (native memory block)
- **`char`**: Character type (stores values like 'A')
- **`void`**: Function return type (no return value)

### Combined Types
- **`type[]`**: Array of the specified type (e.g., `cell[]`, `int[]`, `char[]`)

### Type Semantics
- **Physical vs Virtual Types**:
  - **`cell` / `cell[]`**: **Physical**. These mapped directly to Brainfuck tape cells with stable addresses (`name`, `arr_1`, etc.).
  - **`int` / `char` / `int[]` / `char[]`**: **Virtual**. These exist only in the compiler's symbol table during compilation. They do not occupy tape space by default.
- **Lazy Materialization**: Virtual variables are "materialized" into BFO instructions only when they are needed for I/O (`putc`) or when assigned to a Physical `cell`.
- All numeric values are stored as 8-bit values (0-255).

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
String literals like `"Hello"` are automatically converted to an `ArrayLiteral` of `char` (cell) values at compile time.

```c
char[] s = "Hello"; 
// Equivalent to char[] s = {'H', 'e', 'l', 'l', 'o'};
```

### Array Literals
Arrays can be initialized using curly braces:
```c
cell[] arr = {65, 66, 67};
```

## I/O Optimization: Direct Literal Printing

HBF avoids wasting Brainfuck tape space for transient I/O. When printing literals or virtual variables, the compiler skips variable allocation entirely.

**HBF:**
```c
putc('A');
```

**BFO:**
```
print 'A'      ; No cell used
```

If a value is complex (e.g., a runtime expression assigned to a local `int`), the compiler may use an internal materialization strategy to ensure efficient output.

## Operators

### Arithmetic
- `+` Addition
- `-` Subtraction

### Comparison (in loops)
- `<` Less than
- `>` Greater than

### Assignment
- `=` Variable assignment
- `arr[i] = val` Indexed assignment (constant index only)

### Array Access
- `arr[i]` Array indexing (constant index or unrollable loop variable)

## Control Flow

### For Loops (Loop Unrolling)

Standard for loops with **constant bounds** (including `.length`) are **unrolled** at compile time. This allows the use of loop variables as array indices.

```c
char[] s = "Hi";
for (int i = 0; i < s.length; i++) {
    putc(s[i]);
}
// Compiler substitutes 'i' with 0 then 1, and resolves s[0], s[1]:
// putc('H');
// putc('i');
```

**Requirements for unrolling:**
- Constant iteration count known at compile time
- Standard pattern: `for (int i = 0; i < CONSTANT; i++)`

**Compiled BFO:**
```
set s_0 'H'
set s_1 'i'
print s_0
print s_1
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
- **Variable**: Must be `cell` type (physical counter).
- **Value**: Can be a literal (`10`), a virtual variable (`int a`), or another physical `cell`.
- **Initialization**: If `value` is a variable, the compiler automatically uses the **Add-to-Zero** pattern (`set i 0; add i src`) to copy the value.

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

### Deterministic Inlining
To ensure maximal optimization, HBF uses a deterministic inlining strategy:
- **Virtual Inlining**: Functions taking strictly virtual parameters (`int`, `char`) are **automatically inlined**. This allows the compiler to resolve runtime arithmetic and folding at each specific call site.
- **Physical BFO Functions**: Only functions taking strictly physical `cell` parameters are preserved as `func` definitions in the BFO output.

### Function Calls
```c
function_name(arg1, arg2);
```

### Return Type
Currently only `void` functions are supported.

## Built-in Functions

### `putc(expr)`
Outputs a single character to stdout. 

- **Physical**: If passed a `cell` or a physical `cell[]` element, it emits `print name` (Direct Printing).
- **Virtual/Literal**: If passed an `int`, `char`, or literal, it resolves to `print literal` (Direct Printing).
- **Complex**: Complex expressions are materialized into a temporary cell before printing.

## Array Operations

### `.length`
Returns the length of an array as a compile-time constant:
```c
for (int i = 0; i < s.length; i++) {
    // ...
}
```

### Array Indexing `[i]`
Access individual elements:
```c
cell c = arr[0];
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

### Example 3: Array & String Processing
```c
void print_string(char[] s) {
    for (int i = 0; i < s.length; i++) {
        cell c = s[i];
        putc(c);
    }
}

print_string("Hello, World!\n");
```
