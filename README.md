# wolfram-expr

<h4>
  <a href="https://docs.rs/wolfram-expr">API Documentation</a>
  <span> | </span>
  <a href="https://github.com/WolframResearch/wolfram-expr-rs/blob/master/docs/CHANGELOG.md">Changelog</a>
  <span> | </span>
  <a href="https://github.com/WolframResearch/wolfram-expr-rs/blob/master/docs/CONTRIBUTING.md">Contributing</a>
</h4>

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

Pattern match over different expression variants:

```rust
use wolfram_expr::{Expr, ExprKind};

let expr = Expr::from("some arbitrary expression");

match expr.kind() {
    ExprKind::Integer(1) => println!("got 1"),
    ExprKind::Integer(n) => println!("got {}", n),
    ExprKind::Real(_) => println!("got a real number"),
    ExprKind::String(s) => println!("got string: {}", s),
    ExprKind::Symbol(sym) => println!("got symbol named {}", sym.symbol_name()),
    ExprKind::Normal(e) => println!(
        "got expr with head {} and length {}",
        e.head(),
        e.elements().len()
    ),
}
```

## Related Links

#### Related crates

* [`wolfram-library-link`][wolfram-library-link] — author libraries that can be
  dynamically loaded by the Wolfram Language.
* [`wstp`][wstp] — bindings to the Wolfram Symbolic Transport Protocol, used for passing
  arbitrary Wolfram expressions between programs.
* [`wolfram-app-discovery`][wolfram-app-discovery] — utility for locating local
  installations of Wolfram applications and the Wolfram Language.


[wstp]: https://github.com/WolframResearch/wstp-rs
[wolfram-app-discovery]: https://crates.io/crates/wolfram-app-discovery
[wolfram-library-link]: https://github.com/WolframResearch/wolfram-library-link-rs

## License

 Licensed under either of

  * Apache License, Version 2.0
    ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
  * MIT license
    ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

 at your option.

 ## Contribution

 Unless you explicitly state otherwise, any contribution intentionally submitted
 for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
 dual licensed as above, without any additional terms or conditions.

 See [CONTRIBUTING.md](./CONTRIBUTING.md) for more information.