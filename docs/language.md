# HBF Language Reference

HBF is a C-like high-level language designed to compile down to Brainfuck.

## Variables
HBF supports integer variables. Variables are allocated on the tape automatically.

```rust
let x = 10; // Declaration
x = x + 1;  // Assignment
```

## Input / Output
Currently, HBF supports outputting byte values and string literals.

### Print numbers (as ASCII characters)
```rust
let val = 65;
print(val); // Prints 'A'
```

### Print Strings
```rust
print("Hello World!");
```

## Control Flow
HBF provides `while` loops, which map directly to Brainfuck's `[ ... ]` construct.

```rust
let i = 5;
while (i) {
    print(".");
    i = i - 1;
}
```
*Note: Condition evaluates to true if the variable is non-zero.*

## Expressions & Operators
- `+` (Addition)
- `-` (Subtraction)
- `(` `)` Grouping

```rust
let y = (5 + 3) - 2;
```
