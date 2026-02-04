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

#### `set <var> <literal>`
Set a variable to a literal value (integer or character):
```
set g1 'H'      ; Set g1 to ASCII 'H' (72)
set x 10        ; Set x to 10
```
> [!IMPORTANT]
> BFO `set` does **not** support variable-to-variable assignment. All variable movement and folding must be handled by the HBF compiler's Virtual Variable model before generating BFO.

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

### Variable Initialization (Add-to-Zero)

Since `set` only supports literals, moving a value from one variable to another (e.g., in function parameters or loop counters) is achieved via the **Add-to-Zero** pattern:

```
set target 0    ; Clear target
add target src  ; Copy src value into target
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

