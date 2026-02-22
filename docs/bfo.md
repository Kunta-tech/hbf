# BFO (Brainfuck Object) Format

BFO is the intermediate representation between HBF source code and Brainfuck output. It provides a human-readable, assembly-like format that serves as a stable contract between the frontend optimizer and the backend compiler.

## Purpose

BFO serves three critical roles:

1. **Abstraction Erasure**: All high-level constructs (virtual types, nested expressions, complex loops) are eliminated
2. **Deterministic IR**: Uses only literal assignments and atomic cell operations
3. **Debuggable Output**: Human-readable format for understanding compiler optimizations

## Syntax

### Comments

```bfo
; This is a comment
; Comments start with semicolon and continue to end of line
```

### Instructions

#### `new <var> <literal>`

Create and initialize a new variable with a literal value:

```bfo
new x 10        ; Create x and set to 10
new c 'H'       ; Create c and set to 'H' (ASCII 72)
new flag 1      ; Create flag and set to 1 (true)
```

> [!IMPORTANT]
> `new` creates a variable in the current scope. The variable is freed when the scope exits.

#### `set <var> <literal>`

Set an existing variable to a literal value:

```bfo
set x 10        ; Set x to 10
set c 'A'       ; Set c to 'A' (ASCII 65)
set flag 0      ; Set flag to 0 (false)
```

> [!IMPORTANT]
> BFO `set` **strictly supports literals only**. To copy one variable to another, use the Add-to-Zero pattern:
> ```bfo
> set target 0
> add target source
> ```

#### `add <var> <value>`

Add a value to a variable:

```bfo
add x 5         ; x = x + 5
add x y         ; x = x + y (add variable y to x)
add c 'a'       ; c = c + 97
```

#### `sub <var> <value>`

Subtract a value from a variable:

```bfo
sub x 1         ; x = x - 1
sub x y         ; x = x - y
sub count 5     ; count = count - 5
```

#### `scan <var>`

Read a single character from standard input and store it in `<var>`:

```bfo
scan x          ; x = read_char()
```

#### `goto <var>`

Move the Brainfuck pointer to the start of variable `<var>`:

```bfo
goto x          ; Pointer moves to x
set 10          ; Operates on x
```

#### `@ <const>`

Move the Brainfuck pointer to an absolute address:

```bfo
@ 10            ; Pointer moves to cell 10
set 5           ; Cell 10 = 5
```

> [!NOTE]
> Absolute jumps are mostly used by the compiler and standard library for low-level memory management.

#### `print <value>`

Output a value as a character:

```bfo
print 'A'       ; Print literal 'A'
print x         ; Print value of variable x
print 72        ; Print ASCII 72 ('H')
```

#### `while <var> { ... }`

Loop while the variable is non-zero:

```bfo
while count {
    print 'B'
    sub count 1
}
```

> [!NOTE]
> BFO while loops map directly to Brainfuck's `[...]` construct. The loop continues as long as the variable is non-zero.

#### `{ ... }` Block Scope

Create a new scope for variables:

```bfo
{
    new x 10
    print x
}
; x is freed here
```

> [!NOTE]
> Block scopes enable cell reuse. Variables declared with `new` inside a block are automatically freed when the block exits.

#### `alias <new_name> <old_name>`

Create an alternative name for an existing variable or segment. Does not allocate new memory.

```bfo
alias flag status   ; flag now refers to the same cell as status
```

#### `ref <new_name> <old_name>`

Similar to `alias`, but often used to pass variables into function scopes as references.

```bfo
ref p x             ; p refers to x
```

#### `free <var>`

Explicitly free a variable:

```bfo
new temp 5
add result temp
free temp
```

> [!NOTE]
> `free` is typically used for temporary variables that are no longer needed. Variables in block scopes are automatically freed.

## Instruction Semantics

### new vs set

| Instruction | Purpose | Scope | Example |
|-------------|---------|-------|---------|
| `new` | Create and initialize | Current scope | `new x 10` |
| `set` | Update existing | Any scope | `set x 20` |

**When to use `new`**:
- First declaration of a variable
- Creating function parameters
- Temporary variables

**When to use `set`**:
- Updating existing variables
- Resetting counters
- Modifying state

### Literal-Only Constraint

BFO enforces that `set` and `new` only accept **literal values** (numbers or characters). This constraint:

1. **Simplifies Backend**: No need to handle variable-to-variable moves in set/new
2. **Explicit Copies**: Forces use of Add-to-Zero pattern for clarity
3. **Optimization Friendly**: Literals enable better code generation

**Variable Copy Pattern**:
```bfo
; WRONG: set target source  (not allowed!)

; CORRECT: Add-to-Zero pattern
set target 0
add target source
```

## Common Patterns

### Countdown Loop

```bfo
new counter 10
while counter {
    print 'X'
    sub counter 1
}
free counter
```

### Variable Swap

```bfo
new temp 0
add temp a      ; temp = a
set a 0
add a b         ; a = b
set b 0
add b temp      ; b = temp (original a)
free temp
```

### Conditional Execution (Single-Shot While)

```bfo
new condition 1
while condition {
    ; This executes once if condition is non-zero
    print 'Y'
    set condition 0  ; Prevent re-execution
}
free condition
```

### Function Call Pattern

```bfo
; Function definition
func repeat_char(count, ch) {
    while count {
        print ch
        sub count 1
    }
}

; Function call (conceptual - actual calling handled by compiler)
new arg1 5
new arg2 'A'
repeat_char(arg1, arg2)
free arg1
free arg2
```

## Optimization Examples

### Virtual Variable Elimination

**HBF:**
```c
int a = 5;
int b = 10;
cell c = a + b;
putc(c);
```

**BFO:**
```bfo
new c 15        ; Compiler evaluated 5 + 10
print c
```

**Explanation**: Virtual variables `a` and `b` never appear in BFO. The compiler folded `a + b` to `15` at compile-time.

### Loop Unrolling

**HBF:**
```c
for (int i = 0; i < 3; i++) {
    putc('A');
}
```

**BFO:**
```bfo
print 'A'
print 'A'
print 'A'
```

**Explanation**: Constant-bound loops are completely unrolled, eliminating loop overhead.

### Function Inlining

**HBF:**
```c
void print_digit(int n) {
    putc(48 + n);
}

print_digit(5);
```

**BFO:**
```bfo
print 53        ; Compiler evaluated 48 + 5 during inlining
```

**Explanation**: Functions with virtual parameters are inlined and their arithmetic is resolved at the call site.

### Shorthand Binary Operations

**HBF:**
```c
cell x = 10;
x = x + 5;
```

**BFO:**
```bfo
new x 10
add x 5         ; Atomic update, no redundant set
```

**Explanation**: The compiler detects `x = x + 5` pattern and emits a single `add` instead of `set x 0; add x x; add x 5`.

## Complete Example

**HBF Source:**
```c
void print_line(char[] msg) {
    for (int i = 0; i < msg.length; i++) {
        cell c = msg[i];
        putc(c);
    }
    putc('\n');
}

print_line("Hi");
```

**Generated BFO:**
```bfo
; Inlined print_line("Hi")
{
    ; Loop unrolled (i = 0)
    new c 'H'
    print c
    free c
    
    ; Loop unrolled (i = 1)
    new c 'i'
    print c
    free c
    
    ; After loop
    print '\n'
}
```

**Explanation**:
1. Function `print_line` is inlined
2. `msg` is resolved to `"Hi"` (virtual array)
3. Loop is unrolled (2 iterations)
4. Array indexing `msg[i]` is resolved to literals `'H'` and `'i'`
5. Block scope manages temporary `c` variable

## BFO to Brainfuck Mapping

| BFO | Brainfuck (Conceptual) | Notes |
|-----|------------------------|-------|
| `new x 10` | `>++++++++++` | Move to new cell, add 10 |
| `set x 5` | `[-]+++++` | Clear cell, add 5 |
| `add x 3` | `+++` | Add 3 to current cell |
| `sub x 2` | `--` | Subtract 2 from current cell |
| `print x` | `.` | Output current cell |
| `while x { ... }` | `[...]` | Loop while cell non-zero |
| `free x` | - | Mark cell as available |

> [!NOTE]
> Actual Brainfuck generation includes pointer movement (`<>`) to navigate between cells. The BFO compiler handles cell allocation and pointer tracking.

## Design Principles

1. **Simplicity**: Minimal instruction set
2. **Explicitness**: No implicit operations
3. **Literal-First**: Prefer literals over variable references
4. **Scope-Aware**: Support block scoping for cell reuse
5. **Debuggable**: Human-readable for inspection

## Limitations

1. **No Arithmetic in Instructions**: `add x (y + 5)` is not allowed. Must be pre-computed.
2. **No Nested Expressions**: `print (a + b)` is not allowed. Must materialize to variable first.
3. **No Direct Variable Copy**: `set x y` is not allowed. Must use Add-to-Zero pattern.
4. **Single Condition in While**: `while (x && y)` is not allowed. Must use single variable.

These limitations are **by design** to keep BFO simple and to force the frontend (HBF compiler) to perform all optimizations.

## Future Extensions

Potential future additions to BFO:

- **Input Instruction**: `input <var>` for reading user input
- **Multiply/Divide**: `mul <var> <val>`, `div <var> <val>` for efficient operations
- **Copy Instruction**: `copy <dest> <src>` as syntactic sugar for Add-to-Zero
- **Labels and Jumps**: For more complex control flow
- **Type Annotations**: Optional type hints for better error messages

## Validation

Valid BFO programs must:
1. Use only defined instructions
2. Reference only declared variables
3. Use literals in `new` and `set`
4. Balance block scopes (`{` and `}`)
5. Free variables before scope exit (or use block scopes)

The BFO parser validates these constraints and reports errors during compilation.
