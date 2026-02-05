# HBF Language Reference
HBF (Higher Brainfuck) is a C-like compiled language that targets Brainfuck through an intermediate representation (BFO).

## Types

### Primitive Types
- **`int`**: Integer type (8-bit, mod 256). **Virtual**.
- **`char`**: Character type. **Virtual**.
- **`cell`**: Raw memory cell type. **Physical**.
- **`void`**: Function return type.

### Combined Types
- **`type[]`**: Array of the specified type (e.g., `cell[]`, `int[]`, `char[]`).

### Virtual vs Physical Types
- **Virtual Types (`int`, `char`, `int[]`, `char[]`)**: Exist only in the compiler's symbol table. They are folded into BFO instructions and occupy NO tape space.
- **Physical Types (`cell`, `cell[]`)**: Map directly to Brainfuck tape cells with stable addresses.

## Variables

Variables must be declared with explicit types:

```c
int x = 10;
cell c = 65;
```

### Scope
- Variables are scoped to their enclosing block `{}`.
- Function parameters are local to the function.
- Global/Top-level variables are accessible everywhere.

## Literals & Strings

### String Literals
String literals are automatically converted to `char[]` arrays. When used with `putc`, they generate sequence of direct `print` instructions.

```c
putc("Hello");
// Resulting BFO:
// print 'H'
// print 'e'
// ...
```

### Array Literals
Arrays can be initialized using curly braces:
```c
cell[] arr = {65, 66, 67};
```

## Control Flow

### For Loops (Loop Unrolling)

Standard for loops with **constant bounds** are **unrolled** at compile time.

```c
char[] s = "Hi";
for (int i = 0; i < s.length; i++) {
    putc(s[i]);
}
```

**Resulting BFO (Zero Footprint):**
```
print 'H'
print 'i'
```

### forn Loops (Native Countdown)

For runtime-variable iteration counts, use `forn`:

```c
forn(cell n = 10) {
    putc('B');
}
```

**Resulting BFO:**
```
set n 10
while n {
    print 'B'
    sub n 1
}
```

## Functions

### Deterministic Inlining
Functions taking only **virtual** parameters are automatically inlined.

```c
void print_digit(int n) {
    putc(48 + n);
}

print_digit(1); // Resulting BFO: print 49
```

### Top-Level Code
HBF supports top-level statements and function calls, which are processed sequentially.

## Array Operations

### `.length`
Returns the length of an array as a compile-time constant.

### Array Indexing `[i]`
Access individual elements using a constant index (or a loop variable in an unrollable loop).

## Complete Examples

### Example: String Processing
```c
void print_string(char[] s) {
    for (int i = 0; i < s.length; i++) {
        cell c = s[i];
        putc(c);
    }
}

print_string("Hello, World!\n");
```
