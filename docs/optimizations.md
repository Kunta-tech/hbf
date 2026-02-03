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

## 2. Countdown Loop Optimization

## Overview

The HBF compiler automatically optimizes `for (int i = 0; i < n; i++)` loops into a countdown pattern that eliminates comparison operations.

## The Problem

BFO's `while` instruction only checks if a variable is **non-zero**, not complex comparisons like `i < n`. This creates a challenge for standard counting loops.

## The Solution: Countdown Pattern

### Transformation

**Before (HBF):**
```c
for (int i = 0; i < 5; i++) {
    putc('A');
}
```

**After (Optimized BFO):**
```
set i 0
sub i 5        ; i = -5
while i {      ; Loop while i != 0
    print 'A'
    add i 1    ; -5 → -4 → -3 → -2 → -1 → 0 (stops)
}
```

### How It Works

1. **Initialize to negative**: `i = 0 - n = -n`
2. **Loop condition**: `while i` checks `i != 0`
3. **Increment**: `i++` counts toward zero
4. **Termination**: Loop stops when `i` reaches `0`

### Why This Works

| Iteration | i value | i != 0? | Action |
|-----------|---------|---------|--------|
| Start     | -5      | ✓       | Run loop |
| 1         | -4      | ✓       | Run loop |
| 2         | -3      | ✓       | Run loop |
| 3         | -2      | ✓       | Run loop |
| 4         | -1      | ✓       | Run loop |
| 5         | 0       | ✗       | **Exit** |

## Pattern Detection

The compiler automatically applies this optimization when it detects:

```c
for (int i = 0; i < CONSTANT; i++)
```

**Requirements:**
- ✅ Init: `int i = 0` (must start at zero)
- ✅ Condition: `i < n` (less-than with constant)
- ✅ Update: `i++` (increment by one)

**Not optimized:**
- ❌ `for (int i = 1; i < 10; i++)` - doesn't start at 0
- ❌ `for (int i = 0; i < n; i += 2)` - increment not 1
- ❌ `for (int i = 0; i <= 5; i++)` - uses `<=` instead of `<`

## Benefits

### 1. No Comparison Operations
Eliminates the need for complex comparison logic in BFO/BF.

### 2. Brainfuck-Native
Maps perfectly to BF's `[...]` loop which runs while cell ≠ 0.

### 3. Efficient
Countdown is as fast as count-up but requires no comparison overhead.

## Example: Before vs After

### Without Optimization (Broken)
```
set i 0
while i {      ; BUG: i=0 initially, loop never runs!
    print 'A'
    add i 1
}
```

### With Optimization (Correct)
```
set i 0
sub i 5        ; i = -5
while i {      ; Works: i != 0 for 5 iterations
    print 'A'
    add i 1
}
```

## Implementation

The optimization is implemented in [`src/bfo_gen.rs`](file:///s:/vs%20code/projects/hbf/src/bfo_gen.rs):

```rust
fn can_optimize_for_loop(&self, init: &Stmt, condition: &Expr, update: &Stmt) 
    -> Option<(String, i32)>
```

This function pattern-matches the for loop structure and returns the variable name and loop count if the pattern is detected.

## Credit

This optimization was contributed by the user and demonstrates the power of thinking in terms of the target platform's constraints (Brainfuck's zero-check loops).
