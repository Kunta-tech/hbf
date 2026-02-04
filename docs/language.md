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

## Internal Variables

The compiler uses a special `tmp` variable to materialize virtual literals for I/O:

**HBF:**
```c
putc('A');
```

**BFO:**
```
set tmp 'A'
print tmp
```

This prevents virtual variables from needing a dedicated slot on the Brainfuck tape when they are only used for transient operations.

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
// Compiler substitutes 'i' with 0 then 1, generating:
// putc(s_0);
// putc(s_1);
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
