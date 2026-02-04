# BFO (Brainfuck Object) Format

BFO is the intermediate representation between HBF source code and Brainfuck output.

## Purpose

BFO serves as:
1. **Optimization target**: Easier to optimize than raw Brainfuck
2. **Debug format**: Human-readable representation of compiled code
3. **Linking format**: Allows separate compilation and linking

## Syntax

BFO uses an assembly-like syntax with instructions and operands.

### Comments
```
; This is a comment
```

### Instructions

#### `set <var> <value>`
Set a variable to a value:
```
set g1 'H'      ; Set g1 to ASCII 'H' (72)
set x 10        ; Set x to 10
set y x         ; Copy x to y
```

#### `add <var> <value>`
Add to a variable:
```
add c b         ; c = c + b
add x 5         ; x = x + 5
```

#### `sub <var> <value>`
Subtract from a variable:
```
sub fff_i 1     ; fff_i = fff_i - 1
```

#### `print <var>`
Output a variable as a character:
```
print g1
```

#### `while <var> { ... }`
Loop while variable is non-zero:
```
while fff_i {
    print c
    sub fff_i 1
}
```

#### `func <name>(<params>) { ... }`
Define a function:
```
func add_cells(a, b) {
    set c a
    add c b
    print c
}
```

#### Function Call
```
add_cells(5, 10)
fff('H')
```

## Compilation Strategy

### Predictable Functions
Functions that can be analyzed at compile-time are preserved in BFO:
```
func add_cells(a, b) {
    set c a
    add c b
    print c
}
```

### Unpredictable Functions
Functions with runtime-dependent behavior (e.g., string parameters) are **inlined**:

**HBF:**
```c
void print_string(string s) {
    for (int i = 0; i < s.length; i++) {
        cell c = s[i];
        putc(c);
    }
}
print_string("Hello");
```

**BFO (inlined):**
```
; function print_string breaks down due to not using string in parameter
set g1 'H'
print g1
set g1 'e'
print g1
set g1 'l'
print g1
set g1 'l'
print g1
set g1 'o'
print g1
```

## Memory Model

### Variables as Addresses
Every variable is a memory address pointing to a cell on the Brainfuck tape.

### Stack Allocation
- Local variables are allocated on a stack
- Memory is deallocated when out of scope
- Global variables persist for the program lifetime

### Scoping
Variables can be made global by declaring them outside functions:
```
set fff_i 0   ; Global variable
func fff(c) {
    while fff_i {
        print c
        sub fff_i 1
    }
}
```

## Value Range
All variables store integer values **mod 256** (0-255) or ASCII character values.

## Example: Complete BFO Program

```
func add_cells(a, b) {
    set c a
    add c b
    print c
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

### Constant Folding & Dead Variable Elimination

**Principle:** Since I/O (`putc`) only works with `cell` types, intermediate `int` variables can be folded away.

**HBF:**
```c
int a1 = 5;
int a2 = 10;
int a3 = a1 + a2;
cell c = a3;
putc(c);
```

**Naive BFO:**
```
set a1 5
set a2 10
set a3 a1
add a3 a2
set c a3
print c
```

**Optimized BFO:**
```
set c 15      ; Constant folded: 5 + 10 = 15
print c
```

**Optimizations applied:**
1. **Constant propagation**: `a1=5`, `a2=10` tracked
2. **Expression evaluation**: `a1 + a2` → `5 + 10` → `15`
3. **Dead code elimination**: `a1`, `a2`, `a3` removed (never used for I/O)
4. **Direct assignment**: Result folded into `c`

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

