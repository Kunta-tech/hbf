# Compiler Optimizations

## 1. Constant Folding & Dead Variable Elimination

### Overview

Since I/O in HBF only works with `cell` types (`putc` requires `cell`), intermediate `int` variables used purely for computation can be eliminated and their values folded directly into the `cell` variables.

### The Problem

**Unoptimized code:**
```c
int a1 = 5;
int a2 = 10;
int a3 = a1 + a2;
cell c = a3;
putc(c);
```

**Naive BFO (6 instructions):**
```
set a1 5
set a2 10
set a3 a1
add a3 a2
set c a3
print c
```

### The Solution

**Optimized BFO (2 instructions):**
```
set c 15      ; 5 + 10 = 15 (constant folded)
print c
```

### How It Works

1. **Track variable values**: As declarations are processed, track constant values
2. **Substitute variables**: Replace variable references with their known values
3. **Evaluate expressions**: Compute constant arithmetic (`5 + 10` → `15`)
4. **Eliminate dead variables**: Remove `int` variables that are never used for I/O
5. **Fold into cell**: Assign final computed value directly to `cell` variable

### Optimization Rules

**Variables eliminated:**
- ✅ `int` variables with constant values
- ✅ `int` variables only used in expressions
- ✅ Intermediate `int` variables assigned to `cell`

**Variables kept:**
- ❌ `cell` variables (used for I/O)
- ❌ `int` variables with non-constant values
- ❌ Variables used in multiple places

### Example: Complex Expression

**HBF:**
```c
int x = 10;
int y = 20;
int z = x + y - 5;
cell result = z;
putc(result);
```

**Optimized BFO:**
```
set result 25    ; 10 + 20 - 5 = 25
print result
```

### Benefits

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Variables | 4 | 1 | **75% reduction** |
| Instructions | 6 | 2 | **67% reduction** |
| Memory cells | 4 | 1 | **75% reduction** |

---

## 2. Loop Unrolling Optimization

### Overview

The HBF compiler automatically **unrolls** `for` loops with constant iteration counts, eliminating loop overhead entirely by repeating the loop body inline.

### The Problem

Loops have overhead in Brainfuck - they require loop counter management, comparisons, and branching. For small, constant iteration counts, this overhead is wasteful.

### The Solution: Loop Unrolling

**HBF:**
```c
for (int i = 0; i < 5; i++) {
    putc('A');
}
```

**Optimized BFO (unrolled):**
```
print 'A'
print 'A'
print 'A'
print 'A'
print 'A'
```

### How It Works

1. **Detect constant bounds**: Compiler identifies `for (int i = 0; i < CONSTANT; i++)`
2. **Evaluate iteration count**: Calculates how many times the loop runs
3. **Repeat body**: Generates the loop body statements N times inline
4. **Eliminate loop variable**: No loop counter needed in BFO

### Pattern Detection

The compiler automatically unrolls when it detects:

```c
for (int i = 0; i < CONSTANT; i++)
```

**Requirements:**
- ✅ Init: `int i = 0` (must start at zero)
- ✅ Condition: `i < CONSTANT` (less-than with compile-time constant)
- ✅ Update: `i++` (increment by one)

**Not unrolled:**
- ❌ `for (int i = 0; i < n; i++)` - `n` is runtime variable
- ❌ `for (int i = 1; i < 10; i++)` - doesn't start at 0
- ❌ `for (int i = 0; i <= 5; i++)` - uses `<=` instead of `<`

### Benefits

| Benefit | Description |
|---------|-------------|
| **Zero overhead** | No loop counter, no comparisons, no branching |
| **Smaller code** | For small iteration counts, unrolled code is simpler |
| **Faster execution** | Direct execution without loop management |

### Example: Before vs After

**Before (with loop):**
```
set i 0
sub i 5
while i {
    print 'A'
    add i 1
}
```
**5 instructions + loop overhead**

**After (unrolled):**
```
print 'A'
print 'A'
print 'A'
print 'A'
print 'A'
```
**5 instructions, zero overhead**

---

## 3. Native Countdown Loops (`forn`)

### Overview

For runtime-variable iteration counts, HBF provides the `forn` construct which generates native BFO countdown loops.

### The Problem

When the iteration count isn't known at compile time, loop unrolling isn't possible. We need a native loop construct.

### The Solution: `forn` Construct

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

### How It Works

1. **Initialize counter**: Set `n` to the iteration count
2. **Loop while non-zero**: BFO `while n` checks `n != 0`
3. **Decrement**: Each iteration subtracts 1 from `n`
4. **Terminate at zero**: Loop stops when `n` reaches 0

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
