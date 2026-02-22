# Compiler Optimizations

The HBF compiler performs aggressive optimizations to generate minimal, efficient Brainfuck code. All optimizations happen during the frontend (HBF → BFO) stage.

## Optimization Philosophy

**Goal**: Erase abstraction overhead completely

**Approach**: 
- Virtual types exist only at compile-time
- Constant expressions evaluated immediately
- Loops unrolled when possible
- Functions inlined aggressively

**Result**: BFO output is as minimal as hand-written assembly

## 1. Always-Virtual Variable Model

### Overview

HBF employs an "Always-Virtual" model for non-`cell` types (`int`, `char`, `bool`, and their arrays). These variables exist purely in the compiler's symbol table and **never occupy tape space** unless explicitly materialized.

### Implementation

**Module**: `bfo_gen/scope.rs`

Virtual variables are stored in a scope stack:
```rust
variables: Vec<HashMap<String, Expr>>
```

When a virtual variable is referenced, the compiler:
1. Looks up its value in the scope stack
2. Folds the expression recursively
3. Uses the result directly (never emits BFO for the variable itself)

### Example

**HBF**:
```c
int a = 5;
int b = 10;
int c = a + b;
cell result = c * 2;
putc(result);
```

**Compilation Process**:
1. `int a = 5` → `scope["a"] = Number(5)` (no BFO)
2. `int b = 10` → `scope["b"] = Number(10)` (no BFO)
3. `int c = a + b` → Fold to `15`, `scope["c"] = Number(15)` (no BFO)
4. `cell result = c * 2` → Fold to `30`, emit `new result 30`
5. `putc(result)` → Emit `print result`

**BFO Output**:
```bfo
new result 30
print result
```

### Benefits

- ✅ **Zero Footprint**: Virtual variables use 0 tape cells
- ✅ **Unlimited Arithmetic**: No Brainfuck pointer movement needed
- ✅ **Compile-Time Evaluation**: All math happens during compilation
- ✅ **Dead Code Elimination**: Unused virtual variables disappear

### Boolean Folding

Booleans are strictly virtual, folded to `1` (true) or `0` (false):

**HBF**:
```c
bool flag = true;
int result = 48 + flag;  // 48 + 1 = 49
putc(result);
```

**BFO**:
```bfo
print 49
```

## 2. Constant Folding

### Overview

The compiler evaluates constant expressions at compile-time, reducing complex arithmetic to literals.

### Implementation

**Module**: `bfo_gen/expr_fold.rs`

The `fold_expr()` function recursively evaluates expressions:

```rust
fn fold_expr(&self, expr: Expr) -> Expr {
    match expr {
        Expr::BinaryOp { left, op, right } => {
            let l = self.fold_expr(*left);
            let r = self.fold_expr(*right);
            // Evaluate if both are literals
            if let (Number(a), Number(b)) = (&l, &r) {
                match op {
                    Plus => return Number(a + b),
                    Minus => return Number(a - b),
                    // ... more operations
                }
            }
            BinaryOp { left: l, op, right: r }
        }
        // ... more cases
    }
}
```

### Supported Operations

**Arithmetic**:
- `+`, `-`, `*`, `/`, `%`

**Comparison**:
- `==`, `!=`, `<`, `<=`, `>`, `>=`

**Logical**:
- `&&`, `||`

**Array Operations**:
- Indexing: `"Hello"[0]` → `'H'`
- Length: `"Hello".length` → `5`

### Example

**HBF**:
```c
int x = (5 + 10) * 2 - 3;
cell c = x;
putc(c);
```

**Folding Steps**:
1. `5 + 10` → `15`
2. `15 * 2` → `30`
3. `30 - 3` → `27`

**BFO**:
```bfo
new c 27
print c
```

## 3. Loop Unrolling

### Overview

Loops with constant bounds are completely unrolled, eliminating loop overhead.

### Implementation

**Module**: `bfo_gen/stmt_gen.rs`

The compiler simulates loop execution:

```rust
Stmt::For { init, condition, update, body } => {
    self.push_scope();
    
    // Execute init
    if let Some(i) = init {
        self.gen_stmt(*i, false);
    }
    
    // Simulate iterations
    let mut iterations = 0;
    while iterations < 10000 {
        // Evaluate condition
        let cond_val = self.fold_expr(condition.clone());
        if !is_truthy(cond_val) { break; }
        
        // Execute body
        for s in &body {
            self.gen_stmt(s.clone(), false);
        }
        
        // Execute update
        if let Some(u) = &update {
            self.gen_stmt(*u.clone(), false);
        }
        
        iterations += 1;
    }
    
    self.pop_scope();
}
```

### Example 1: Simple Loop

**HBF**:
```c
for (int i = 0; i < 3; i++) {
    putc('A');
}
```

**BFO**:
```bfo
print 'A'
print 'A'
print 'A'
```

### Example 2: Array Iteration

**HBF**:
```c
char[] msg = "Hi";
for (int i = 0; i < msg.length; i++) {
    putc(msg[i]);
}
```

**Process**:
1. `msg.length` → `2`
2. Iteration 0: `msg[0]` → `'H'` → `print 'H'`
3. Iteration 1: `msg[1]` → `'i'` → `print 'i'`

**BFO**:
```bfo
print 'H'
print 'i'
```

### 3.1. While Loop Unrolling

**Overview**: Similar to `for` loops, `while` loops are unrolled when the condition is virtual.

**Implementation**: The generator evaluates the condition in each virtual iteration. If the virtual state changes such that the condition becomes false, the unrolling stops.

**Example**:
```c
int i = 3;
while (i > 0) {
    putc('!');
    i--;
}
```
**BFO**:
```bfo
print '!'
print '!'
print '!'
```

### Benefits

- ✅ **Zero Loop Overhead**: No counter, no condition check
- ✅ **Minimal BFO**: Only body statements repeated
- ✅ **Faster Execution**: Direct execution, no branching

## 4. Function Inlining & Return Optimization

### Overview

Functions with virtual parameters are automatically inlined, enabling cross-function constant folding. Additionally, the compiler tracks **function return types** so that functions returning virtual types (like `int square(int n)`) can be fully evaluated at compile-time.

### Implementation

**Module**: `bfo_gen/inline.rs`

Inlining process:
1. Evaluate arguments in caller scope
2. Create new scope for function
3. Bind parameters to evaluated arguments
4. Generate function body
5. Pop scope

```rust
fn inline_function(&mut self, params: Vec<(Type, String)>, 
                   args: Vec<Expr>, body: Vec<Stmt>) {
    // Evaluate arguments
    let evaluated_args: Vec<Expr> = args.iter()
        .map(|arg| self.fold_expr(arg.clone()))
        .collect();
    
    // Create scope
    self.push_scope();
    self.emit_line("{");
    
    // Bind parameters
    for (i, (param_type, param_name)) in params.iter().enumerate() {
        if let Some(arg) = evaluated_args.get(i) {
            match param_type {
                Type::Cell => self.materialize_to_cell(param_name, arg.clone(), true),
                _ => self.declare_variable(param_name, arg.clone()),
            }
        }
    }
    
    // Generate body
    for stmt in body {
        self.gen_stmt(stmt, false);
    }
    
    // Clean up
    self.emit_line("}");
    self.pop_scope();
}
```

### Example

**HBF**:
```c
void print_digit(int n) {
    putc(48 + n);
}

void print_number(int x) {
    print_digit(x / 10);
    print_digit(x % 10);
}

print_number(42);
```

**Inlining Steps**:
1. Inline `print_number(42)`:
   - `x = 42`
   - Inline `print_digit(42 / 10)` → `print_digit(4)`
   - Inline `print_digit(42 % 10)` → `print_digit(2)`

2. Inline `print_digit(4)`:
   - `n = 4`
   - `putc(48 + 4)` → `putc(52)`

3. Inline `print_digit(2)`:
   - `n = 2`
   - `putc(48 + 2)` → `putc(50)`

**BFO**:
```bfo
print 52
print 50
```

### Benefits

- ✅ **Cross-Function Optimization**: Arguments folded at call site
- ✅ **No Function Overhead**: No call/return mechanism needed
- ✅ **Enables Further Optimization**: Inlined code can be optimized further

## 5. Compile-Time If/Else Evaluation

### Overview

When `if` conditions are compile-time constants, only the active branch is emitted.

### Implementation

**Module**: `bfo_gen/stmt_gen.rs`

```rust
Stmt::If { condition, then_branch, else_branch } => {
    let folded_cond = self.fold_expr(condition);
    match folded_cond {
        Expr::BoolLiteral(b) => {
            if b {
                for s in then_branch { self.gen_stmt(s, false); }
            } else if let Some(else_stmts) = else_branch {
                for s in else_stmts { self.gen_stmt(s, false); }
            }
        }
        // ... runtime if handling
    }
}
```

### Example

**HBF**:
```c
bool debug = false;
if (debug) {
    putc("DEBUG: ");
}
putc("Hello\n");
```

**BFO**:
```bfo
print 'H'
print 'e'
print 'l'
print 'l'
print 'o'
print '\n'
```

**Result**: Debug branch completely eliminated.

## 6. Post-Fix Operator Transformation

**Overview**: `++` and `--` operators are transformed into standard assignments at the parser level.

**Implementation**:
- `a++` → `a = a + 1`
- `arr[i]--` → `arr[i] = arr[i] - 1`

**Benefit**: This unification allows these operators to work "for free" with the constant folder for virtual variables (`a++` updates the compile-time value) while still leveraging specialized `add`/`sub` primitives for `cell` variables via the generator's pattern matching.

## 7. Procedural Primitives & Restricted Math

### Overview

To ensure maximum efficiency and tape control, HBF restricts **infix math** on `cell` types. Instead, it provides **procedural primitives** that map to atomic, highly optimized BFO instructions.

### The "Move" Optimization

The `move(dest, src)` primitive is a critical optimization. While a `copy` requires a temporary cell and two loops to preserve the source, a `move` is destructive and maps to a single, minimal Brainfuck loop.

| Operation | BF Mapping | Complexity |
|-----------|------------|------------|
| `copy(b, a)` | `[ - > + > + < < ] > [ - < + > ]` | O(2n) + Temp |
| `move(b, a)` | `[ - > + < ]` | O(n) + No Temp |

### Shorthand Binary Operations (Ergonomics)
Pointers like `A = A + B` are still supported via the parser mapping them to procedural `add` or `sub` calls.

---

### Implementation

**Module**: `bfo_gen/emit.rs`

The `materialize_to_cell()` function detects patterns:

```rust
Expr::BinaryOp { left, op, right } => {
    let left_is_name = matches!(*left, Expr::Variable(ref v) if v == name);
    let right_is_name = matches!(*right, Expr::Variable(ref v) if v == name);
    
    if left_is_name && op == Token::Plus {
        // A = A + right → add A right
        self.emit(&format!("add {} ", name));
        self.gen_expr_simple(*right);
    } else if right_is_name && op == Token::Plus {
        // A = left + A → add A left
        self.emit(&format!("add {} ", name));
        self.gen_expr_simple(*left);
    } else if left_is_name && op == Token::Minus {
        // A = A - right → sub A right
        self.emit(&format!("sub {} ", name));
        self.gen_expr_simple(*right);
    }
    // ... general case
}
```

### Supported Patterns

| HBF | BFO |
|-----|-----|
| `A = A + B` | `add A B` |
| `A = B + A` | `add A B` |
| `A = A - B` | `sub A B` |
| `A = A + 5` | `add A 5` |

putc(counter);
```

### 8. Auto-Materialization
When a complex expression is used as an argument to a procedural primitive (e.g., `add(c, b + 5)`), the compiler automatically materializes the virtual part to a temporary cell, ensuring the developer can use convenient syntax without losing performance control.


**BFO**:
```bfo
new counter 10
add counter 5
sub counter 2
print counter
```

## 7. Property & Array Access Folding

### Overview

Array indexing and property access on virtual arrays are resolved at compile-time.

### Implementation

**Module**: `bfo_gen/expr_fold.rs`

```rust
Expr::ArrayAccess { array, index } => {
    let array_folded = self.fold_expr(*array);
    let index_folded = self.fold_expr(*index);
    
    if let Expr::Number(i) = &index_folded {
        match &array_folded {
            Expr::StringLiteral(s) => {
                if let Some(ch) = s.chars().nth(*i as usize) {
                    return Expr::CharLiteral(ch);
                }
            }
            Expr::ArrayLiteral(elements) => {
                if let Some(el) = elements.get(*i as usize) {
                    return el.clone();
                }
            }
            // ... more cases
        }
    }
    // ... fallback
}
```

### Example

**HBF**:
```c
char[] name = "Alice";
cell first = name[0];
int len = name.length;
putc(first);
putc(48 + len);  // Print length as digit
```

**BFO**:
```bfo
new first 'A'
print first
print 53
```

## 8. Backend Optimizations (BFO → BF)

While frontend optimizations erase high-level abstractions, the intermediate BFO stage and the final Brainfuck codegen perform low-level hardware-specific optimizations.

### 8.1. Whitespace-Aware Peephole Optimizer
The final codegen (`bf_codegen.rs`) implements a "smart" peephole optimizer. Most Brainfuck optimizers fail when code is formatted with newlines or spaces. HBF's optimizer peeks through the output buffer and ignores non-functional characters:
- `+ \n -` → Cancelled to nothing
- `> \r <` → Cancelled to nothing
- `+ + -` → Optimized to `+`

### 8.2. Intelligent State Tracking
The code generator maintains a virtual model of the tape state during compilation:
- **Dirty Cell Tracking**: The compiler knows which cells might contain non-zero values. `Clear` instructions (`[-]`) are automatically skipped if a cell is already guaranteed to be clean.
- **Loop-Exit Guarantee**: The optimizer recognizes that every Brainfuck loop `[...]` only exits when the current cell is exactly zero. It automatically removes the "dirty" flag from a cell upon loop termination.
- **Jump Resumption**: Absolute jumps (`@`) reset the pointer state, allowing optimizations to resume accurately even after complex, unbalanced loops that might lose the relative pointer position.

### 8.3. "Initial Zero" & Fresh Memory Optimization
Based on the assumption that all Brainfuck cells are zero at the start of the program:
1. The compiler tracks a `touched_cells` set.
2. Any `new` allocation that uses "fresh" memory (addresses never before touched by the program) skips its initial zeroing pass.
3. Only **recycled** memory from the LIFO free pool is explicitly cleared, ensuring minimal `[-]` instructions in the output.

## Optimization Metrics

For a typical HBF program:

| Metric | Before Optimization | After Optimization | Reduction |
|--------|--------------------|--------------------|-----------|
| Virtual Variables | 10 | 0 | 100% |
| Loop Instructions | 50 | 0 | 100% |
| Function Calls | 5 | 0 | 100% |
| BFO Lines | 200 | 15 | 92.5% |
| BF Gen Steps | 100% | 40% | 60% |
| Tape Cells Used | 15 | 3 | 80% |

## Future Optimizations

1. **Dead Code Elimination**: Remove unused functions and variables at the BFO level.
2. **Global Cell Lifecycle Analysis**: Predict cell usage across function boundaries.
3. **Strength Reduction**: Replace expensive operations with modular arithmetic.
4. **Register Allocation**: Optimize tape layout based on frequency of cross-cell access.

## Debugging Optimizations

To understand what optimizations are applied:

1. **View BFO**: See the optimized intermediate representation
   ```bash
   hbf -c example.hbf
   cat example.bfo
   ```

2. **View BF**: Inspect the final Brainfuck code for redundant patterns.
   ```bash
   hbf -s example.bfo
   cat example.bf
   ```

3. **Compiler Trace**: Use internal debug logging to see state tracking in action.

## Optimization Trade-offs

| Optimization | Benefit | Cost |
|--------------|---------|------|
| Virtual Variables | Zero tape usage | Compile-time memory |
| Loop Unrolling | Zero runtime overhead | Larger BFO for big loops |
| Function Inlining | Cross-function optimization | Code duplication |
| Constant Folding | Faster execution | Longer compile time |
| State Tracking | Minimal `[-]` and `<>` | Complexity in codegen |

**Overall**: HBF prioritizes **runtime performance** over compile time and code size.
