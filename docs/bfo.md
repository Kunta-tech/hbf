# BFO (Brainfuck Object) Format

BFO is the intermediate representation between HBF source code and Brainfuck output. It is a strict, assembly-like text format.

## Purpose

BFO serves as a deterministic, tape-ready contract:
1. **Erasure**: All high-level abstractions (virtual types, nested math, loops) are erased.
2. **Deterministic IR**: Uses only literal assignments and atomic cell operations.
3. **Debuggable**: A human-readable representation of the optimized code.

## Syntax

### Comments
```
; This is a comment
```

### Instructions

#### `set <var> <literal>`
Set a variable to a literal value (integer or character):
```
set x 10
set c 'H'
```
> [!IMPORTANT]
> BFO `set` strictly supports **literals only**. Variable-to-variable moves must use the Add-to-Zero pattern.

#### `add <var> <value>` / `sub <var> <value>`
Perform arithmetic on a variable using a literal or another variable:
```
add c 'a'  ; c = c + 97
sub x y    ; x = x - y
```

#### `print <value>`
Output a value (literal or variable) as a character:
```
print 'A'
print msg_1
```

#### `while <var> { ... }`
Loop while the variable is non-zero.
```
while count {
    print 'B'
    sub count 1
}
```

#### `func <name>(<params>) { ... }`
Define a physical function. Only used for functions taking `cell` parameters.
```
func repeat(c) {
    while i {
        print c
        sub i 1
    }
}
```

#### Function Call
```
}

set fff_i 0   ; making i global
func fff(c) {
    while fff_i {
        print c
        sub fff_i 1
    }
}

; Main program
add_cells(5, 10)
set fff_i 5
fff('H')
```

This outputs:
- ASCII 15 (from 5+10)
- 'H' printed 5 times

## Optimizations

### Always-Virtual Variables

**Principle:** Variables of type `int` and `char` are **Virtual**. They exist only in the compiler's symbol table and are folded/evaluated at compile-time. They are only "materialized" into BFO `set` instructions when needed for I/O.

**HBF:**
```c
int a = 5;
int b = 10;
cell c = a + b;
putc(c);
```

**BFO:**
```
set c 15      ; HBF compiler evaluates 5 + 10 = 15
print c
```

**Workflow:**
1. **Silent Updates**: `int a = 5` and `int b = 10` update internal state but emit NO BFO.
2. **Compile-Time Evaluation**: `a + b` is evaluated by the compiler.
3. **Materialization**: The result `15` is emitted only when assigned to the physical `cell c`.

**Benefits:**
- ✅ Eliminates unnecessary variables
- ✅ Reduces memory usage
- ✅ Simplifies generated Brainfuck code

### Loop Unrolling

**Pattern:** `for (int i = 0; i < CONSTANT; i++)`

**Behavior:** Compiler **unrolls** the loop, repeating the body statements inline.

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

**How it works:**
1. Detect constant iteration count at compile time
2. Repeat loop body N times inline
3. Eliminate loop variable and overhead

**Benefits:**
- ✅ Zero loop overhead
- ✅ Simpler BFO output
- ✅ Faster execution

### Native Countdown Loops (`forn`)

**Pattern:** `forn(cell n = value)`

**Behavior:** Generates native BFO `while` loop with countdown pattern.

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

**How it works:**
1. Initialize counter to iteration count
2. Loop while counter ≠ 0
3. Decrement by 1 each iteration
4. Loop terminates when counter reaches 0

**Benefits:**
- ✅ Works with runtime variables
- ✅ Maps perfectly to Brainfuck's `[...]` loop
- ✅ No comparison operations needed

**Use cases:**
- Use `for` when iteration count is compile-time constant
- Use `forn` when iteration count is runtime variable

