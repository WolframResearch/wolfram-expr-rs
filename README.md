# wolfram-expr

Representation of Wolfram Language expressions.

## Examples

Construct the expression `{1, 2, 3}`:

```rust
use wolfram_expr::{Expr, Symbol};

let expr = Expr::normal(Symbol::new("System`List"), vec![
    Expr::from(1),
    Expr::from(2),
    Expr::from(3)
]);
```