# Compiler Optimizations

## 1. Always-Virtual Variable Model

### Overview

HBF employs an "Always-Virtual" model for all non-`cell` types (`int`, `char`, and their arrays). These variables exist purely in the compiler's symbol table during compilation and do not occupy a fixed slot on the Brainfuck tape by default.

### The Problem

In traditional compilers, every variable maps to a memory address. In Brainfuck, memory (the tape) is a precious resource. Allocating tape cells for intermediate computation counters or constants is inefficient.

### The Solution: Virtual Variables

**HBF:**
```c
int a = 5;
int b = 10;
cell c = a + b;
putc(c);
```

**BFO (2 instructions):**
```
set c 15      ; Materialized literal result
print c
```

### How It Works

1. **Silent Initialization**: `int a = 5` and `int b = 10` are recorded in the compiler's memory. No BFO code is generated.
2. **Compile-Time Evaluation**: When `a + b` is encountered, the compiler resolves it to `15` using its internal state.
3. **Lazy Materialization**: A virtual value is only "materialized" (emitted as a BFO `set` instruction) when it is:
   - Assigned to a physical `cell`.
   - Used as an operand in an I/O operation (`putc`).
4. **Direct Literal Printing**: If a value is used purely for I/O, the compiler skips variable assignment entirely and emits `print <literal>` (e.g., `print 'H'`).

### Global Virtual Folding

Virtual variables defined at the global scope are also tracked and folded into function bodies.

**HBF:**
```c
int a = 2;
char b = 'a';

void main() {
    putc(a + b);
}
```

**BFO:**
```
func main() {
    print 99    ; Evaluated (2 + 97) at compile-time
}
```

### Benefits

| Metric | Non-Virtual | Always-Virtual | Improvement |
|--------|-------------|----------------|-------------|
| Variables Used | 3 | 0 | **100% reduction** |
| Tape Cells Used| 3 | 0 | **100% reduction** |
| Instructions | 6 | 1 | **83% reduction** |

---

## 2. Loop Unrolling & Materialization

### Overview

The HBF compiler automatically **unrolls** `for` loops with constant iteration counts. This works in tandem with the Virtual Variable model to resolve array indexing at compile-time.

### The Solution: Direct Literal Unrolling

**HBF:**
```c
char[] s = "Hi";
for (int i = 0; i < s.length; i++) {
    putc(s[i]);
}
```

**Optimized BFO (2 instructions):**
```
print 'H'
print 'i'
```

### How It Works

1. **Detect constant bounds**: Compiler identifies `for (int i = 0; i < CONSTANT; i++)` or `i < arr.length`.
2. **Virtual Array Resolution**: The compiler looks up the array `s` in its memory-only symbol table.
3. **Literal Substitution**: For each iteration `i`, the compiler resolves `s[i]` to its literal value (e.g., `'H'`).
4. **Direct Printing**: Instead of moving the pointer and setting values, the compiler emits direct `print` instructions for the substitution results.
5. **Zero Footprint**: The loop variable `i` and the array `s` never touch the physical Brainfuck tape.

### Pattern Detection

The compiler automatically unrolls when it detects:

```c
for (int i = 0; i < CONSTANT; i++)
// OR
for (int i = 0; i < arr.length; i++)
```

**Requirements:**
- ✅ Init: `int i = 0` (must start at zero)
- ✅ Condition: `i < CONSTANT` or `i < arr.length` (where array size is known)
- ✅ Update: `i++` (increment by one)

**Not unrolled:**
- ❌ `for (int i = 0; i < n; i++)` - `n` is a runtime variable
- ❌ `for (int i = 1; i < 10; i++)` - doesn't start at 0
- ❌ `for (int i = 0; i <= 5; i++)` - uses `<=` instead of `<`

### Benefits

| Benefit | Description |
|---------|-------------|
| **Zero overhead** | No loop counter, no comparisons, no branching |
| **Efficient Array Access** | Converts variable indices `arr[i]` into fixed cell references `arr_0`, `arr_1` |
| **Constant-Time Memory** | Allows indexing into memory cells that aren't natively addressable via variables in BF |
| **Faster execution** | Direct execution without loop management |

### Example: Array Processing

**HBF:**
```c
char[] s = "OK";
for (int i = 0; i < s.length; i++) {
    putc(s[i]);
}
```

**BFO (unrolled and substituted):**
```
set s_0 'O'
set s_1 'K'
print s_0
print s_1
```
**4 instructions, zero loop overhead, constant-time indexing.**

---

## 3. Native Countdown Loops (`forn`)

### Overview

For runtime-variable iteration counts, HBF provides the `forn` construct which generates native BFO countdown loops.

### The Problem

When the iteration count isn't known at compile time, loop unrolling isn't possible. We need a native loop construct.

### The Solution: `forn` Construct & Add-to-Zero Init

**HBF:**
```c
forn(cell n = 10) {
    putc('B');
}
```

**BFO (native countdown loop):**
```
set n 10
while n {
    print 'B'
    sub n 1
}
```

### Add-to-Zero Initialization

When the loop counter is initialized from a runtime variable (like a function parameter), the compiler uses an **Add-to-Zero** pattern to satisfy the BFO `set` (literal-only) restriction:

**HBF:**
```c
void fff(int count, cell c) {
    forn(cell i = count) {
        putc(c);
    }
}
```

**BFO:**
```
func fff(count, c) {
    set i 0      ; Clear/initialize i
    add i count  ; "Copy" count to i
    while i {
        print c
        sub i 1
    }
}
```

### How It Works

1. **Initialize counter**: For literals, uses `set n literal`. For variables, uses `set n 0` + `add n var`.
2. **Loop while non-zero**: BFO `while n` checks `n != 0`.
3. **Decrement**: Each iteration subtracts 1 from `n`.
4. **Terminate at zero**: Loop stops when `n` reaches 0.

### Why Countdown?

BFO's `while` instruction only checks if a variable is **non-zero**, not complex comparisons like `i < n`. Counting down to zero maps perfectly to this constraint.

| Iteration | n value | n != 0? | Action |
|-----------|---------|---------|--------|
| Start     | 10      | ✓       | Run loop |
| 1         | 9       | ✓       | Run loop |
| ...       | ...     | ✓       | Run loop |
| 9         | 1       | ✓       | Run loop |
| 10        | 0       | ✗       | **Exit** |

### Syntax

```c
forn(cell variable = value) {
    // body
}
```

**Requirements:**
- Variable must be `cell` type
- Value can be constant or runtime variable
- Loop executes `value` times

### Benefits

- ✅ **Runtime flexibility**: Works with variable iteration counts
- ✅ **Brainfuck-native**: Maps perfectly to BF's `[...]` loop
- ✅ **No comparison overhead**: Uses simple zero-check

### Use Cases

**Use `for` when:**
- Iteration count is a compile-time constant
- You want zero loop overhead

**Use `forn` when:**
- Iteration count is a runtime variable
- You need a native loop in the BFO output

### Example: Runtime Variable

```c
void repeat_char(int count, cell c) {
    forn(cell i = count) {
        putc(c);
    }
}

repeat_char(5, 'H');  // Prints 'H' 5 times
```

**BFO:**
```
func repeat_char(count, c) {
    set i count
    while i {
        print c
        sub i 1
    }
}
```

## Implementation

Both optimizations are implemented in [`src/bfo_gen.rs`](file:///s:/vs%20code/projects/hbf/src/bfo_gen.rs):

- **Loop unrolling**: Pattern-matches `for` loops and repeats body statements
- **`forn` loops**: Generates `set` + `while` + `sub` pattern
