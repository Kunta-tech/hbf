# Compiler Optimizations

## 1. Always-Virtual Variable Model

HBF employs an "Always-Virtual" model for all non-`cell` scalar and array types (`int`, `bool`, `char`, `int[]`, `char[]`). These variables exist purely in the compiler's symbol table and do not occupy a fixed slot on the Brainfuck tape by default.

### The Solution: Virtual Folding

**HBF:**
```c
int a = 5;
int b = 10;
cell c = a + b;
putc(c);
```

**BFO (generated):**
```
set c 15      ; Evaluated 5 + 10 = 15
print c
```

### Benefits
- **Zero Footprint**: Virtual variables use 0 tape cells.
- **Limitless Arithmetic**: Since virtual math happens at compile-time, it isn't restricted by Brainfuck's pointer movement or destructive copy limits.

### Boolean Folding

Boolean types are strictly virtual. They are folded into integer constants (`1` for `true`, `0` for `false`) during the compilation process.

**HBF:**
```c
bool flag = true;
int result = 48 + flag; // 48 + 1 = 49 ('1')
putc(result);
```

**BFO:**
```
print 49
```

---

## 2. Loop Unrolling

The HBF compiler automatically **unrolls** `for` loops with constant bounds. This works in tandem with the Virtual Variable model to resolve array indexing at compile-time.

### Direct Literal Unrolling

**HBF:**
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

### How It Works
1. **Detect constant bounds**: Compiler identifies patterns like `i < 5` or `i < s.length`.
2. **Literal Substitution**: The compiler resolves `s[i]` to its literal value (e.g., `'H'`) for each iteration.
3. **Loop Erasure**: The loop overhead is completely removed, leaving only the materialized body instructions.

---

## 3. Native Countdown Loops (`forn`)

For runtime-variable iteration counts, HBF provides the `forn` construct which generates native BFO countdown loops.

**HBF:**
```c
forn(cell n = 10) {
    putc('B');
}
```

**Resulting BFO (native countdown):**
```
set n 10
while n {
    print 'B'
    sub n 1
}
```

### Why Countdown?


**BFO:**
```
func repeat_char(count, c) {
    set i 0
    add i count
    while i {
        print c
        sub i 1
    }
}
```

## 4. Deterministic Inlining

### Overview

To satisfy the "Loose HBF -> Strict BFO" contract, the compiler uses deterministic inlining for all functions using only virtual parameters (`int`, `char`).

### Literal Resolution at Call-Sites

By inlining these functions, the compiler can substitute arguments with their call-site values and resolve complex arithmetic to zero-cost literals.

**HBF:**
```c
void print_digit(int n) {
    putc(48 + n);
}
void main() {
    print_digit(1);
}
```

**Optimized BFO:**
```
print 49      ; Resolved (48 + 1) during inlining
```

### Physical Modular Interface

Only functions taking strictly physical `cell` parameters are preserved in BFO. This provides a clear, modular interface for tape-resident data while ensuring abstraction overhead is completely erased for everything else.

## 5. Shorthand Binary Operations

### Overview

When updating a physical `cell` using its own current value (e.g., `A = A + i`), the compiler avoids redundant reconstruction.

### Optimized Lowering

Instead of clearing the variable and rebuilding it (`set A 0; add A A; add A i`), the compiler detects the shorthand pattern and emits a single, atomic BFO instruction.

**HBF:**
```c
cell A = 65;
A = A + 5;
```

**BFO:**
```
set A 65
add A 5       ; Atomic update, no redundancy
```

**Supported Patterns:**
- `A = A + B` -> `add A B`
- `A = B + A` -> `add A B`
- `A = A - B` -> `sub A B`

## 6. Property & Access Folding

### Overview

Beyond basic arithmetic, the HBF compiler can resolve array indexing (`arr[i]`) and property access (`arr.length`) at compile-time for all virtual types.

### Virtual Array Indexing

When a virtual array (like a string or `int[]`) is accessed with a constant index, the compiler retrieves the literal value directly from its metadata storage.

**HBF:**
```c
char[] s = "Hello";
cell c = s[0];
putc(c);
```

**BFO:**
```
print 'H'      ; s[0] resolved to 'H' at compile-time
```

### Property Folding

The `.length` property of any array is treated as a compile-time constant, allowing for efficient loop unrolling and literal substitution.

**Benefits:**
- ✅ **Zero Runtime Access**: No need to store length on the tape or perform runtime lookups.
- ✅ **Predictable Unrolling**: Enables the compiler to determine loop boundaries without runtime interaction.

## Implementation

Both optimizations are implemented in [`src/bfo_gen.rs`](file:///s:/vs%20code/projects/hbf/src/bfo_gen.rs):

- **Loop unrolling**: Pattern-matches `for` loops and repeats body statements
- **`forn` loops**: Generates `set` + `while` + `sub` pattern
