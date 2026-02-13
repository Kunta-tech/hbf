# HBF Language Reference
HBF (Higher Brainfuck) is a C-like compiled language that targets Brainfuck through an intermediate representation (BFO).

## Types

### Primitive Types
- **`int`**: Integer type (8-bit, mod 256). **Virtual**.
- **`bool`**: Boolean type (`true`/`false`). **Virtual**.
- **`char`**: Character type. **Virtual**.
- **`cell`**: Raw memory cell type. **Physical**.
- **`void`**: Function return type or non-returning function.

### Combined Types
- **`type[]`**: Array of the specified type (e.g., `cell[]`, `int[]`, `char[]`).

### Virtual vs Physical Types

| Property | Virtual Types (`int`, `bool`, `char`) | Physical Types (`cell`, `cell[]`) |
|----------|---------------------------------------|-----------------------------------|
| **Location** | Compiler symbol table (compile-time) | Brainfuck tape (runtime) |
| **Footprint** | **Zero cells** | 1 or more cells |
| **Indexing** | Constant only | Constant (direct) |
| **Use Case** | Offsets, loop counters, constants | Runtime state, user input, buffers |

> [!IMPORTANT]
> **Turing Completeness**: While HBF requires constant indexing for direct cell access (simulating fixed-size buffers), the inclusion of `while` loops and `getc()` ensures that HBF is **Turing-complete**. Complex algorithms can be implemented by using cells as tape pointers or through procedural primitives.

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

### Multi-Variable Declaration
Multiple variables of the same type can be declared in a single statement, separated by commas:
```c
int a, b = 12, c;
```
If an initializer is omitted, variables are automatically initialized to `0` (for scalars) or an empty literal `[]` (for arrays).

### Flexible Array Syntax
Arrays can be declared with brackets on either the type or the name:
- **Java-style**: `int[] a, b;`
- **C-style**: `int a[], b;` (Only `a` is an array)

## Literals & Strings

### Boolean Literals
The keywords `true` and `false` are available. Since `bool` is a virtual type, these values are folded to integer constants (`1` and `0` respectively) during compilation.

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
// Unrolls to: print 'H', print 'i'
```

### while Loops
While loops allow for conditional iteration. Like `for` loops, they are **unrolled at compile-time** if the condition depends on virtual variables. If the condition depends on physical cells, they generate native Brainfuck loops.

**Virtual Unrolling:**
```c
int i = 0;
while (i < 3) {
    putc('A');
    i++;
}
// Generates 3 print 'A' instructions
```

**Physical (Native):**
```c
while (c) {
    putc('!');
    sub(c, 1);
}
```

### if / else Statements
HBF supports compile-time evaluated `if`/`else` statements. The condition must be a virtual expression (resolvable at compile-time).
```c
bool flag = true;
if (flag) {
    putc('Y');
} else {
    putc('N');
}
```
`else if` chains are also supported. Only the branch satisfying the condition is emitted to the output BFO.

**Resulting BFO (Zero Footprint):**
```
print 'H'
print 'i'
```

### forn Loops (Native Countdown)

For runtime-variable iteration counts, use `forn`:

```c
forn(10) {
    putc('B');
}
```

**Resulting BFO:**
```
set _forn_0 10
while _forn_0 {
    print 'B'
    sub _forn_0 1
}
```

## Functions

### Recursive and Virtual Inlining
Functions taking only **virtual** parameters are automatically inlined. HBF now supports functions returning virtual types (`int`, `char`, `bool`), allowing them to be used in constant expressions.

```c
int square(int n) {
    return n * n;
}

void print_digit(int n) {
    putc(48 + n);
}

print_digit(square(3)); // Resulting BFO: print 57 (9 + 48)
```

### Top-Level Code
HBF supports top-level statements and function calls, which are processed sequentially.

## Array Operations

### `.length`
Returns the length of an array as a compile-time constant.

### Array Indexing `[i]`
Access individual elements using a constant index (or a loop variable in an unrollable loop).


## Procedural Primitives (Physical Math)

To ensure maximum efficiency and tape control, **infix math** (like `+`, `-`, `*`) is **restricted** for `cell` types. Instead, HBF provides built-in procedural primitives:

| Primitive | Description | BF Mapping |
|-----------|-------------|------------|
| `add(target, value)` | Adds `value` to `target` cell. | `[-] > [+] <` (optimized) |
| `sub(target, value)` | Subtracts `value` from `target` cell. | `[-] > [-] <` |
| `set(target, value)` | Sets `target` to `value`. | `[-] +++` |
| `copy(dest, src)` | Copies `src` to `dest` (preserves `src`). | Two-loop copy |
| `move(dest, src)` | Destructively moves `src` to `dest` (`src` becomes 0). | `[-] [ - > + < ]` |
| `clear(target)` | Resets `target` cell to 0. | `[-]` |

### Post-Fix Ergonomics
HBF supports `++` and `--` on both virtual and physical types. These are unified in the parser into standard assignments (`a++` becomes `a = a + 1`). This ensures they work seamlessly with the constant folder for virtual types while mapping to optimized `add`/`sub` primitives for `cell` types.

---

### Example: String Processing
```c
void print_string(char[] s) {
    for (int i = 0; i < s.length; i++) {
        cell c;
        set(c, s[i]);
        putc(c);
    }
}

print_string("Hello, World!\n");
```
