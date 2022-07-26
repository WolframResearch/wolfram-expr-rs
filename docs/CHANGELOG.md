# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]



## [0.1.2] – 2022-07-25

### Added

* Added new convenience methods for constructing and converting `Expr`s. ([#7])

  The following operations are new:

  * [`Expr::try_as_bool`](https://docs.rs/wolfram-expr/0.1.2/wolfram_expr/struct.Expr.html#method.try_as_bool)
  * [`Expr::try_as_str`](https://docs.rs/wolfram-expr/0.1.2/wolfram_expr/struct.Expr.html#method.try_as_str)
  * [`Expr::rule_delayed`](https://docs.rs/wolfram-expr/0.1.2/wolfram_expr/struct.Expr.html#method.rule_delayed)
  * `impl From<bool> for Expr`

  The following methods were renamed, and the previous method marked
  `#[deprecated(..)]`:

  * [`Expr::try_as_normal`](https://docs.rs/wolfram-expr/0.1.2/wolfram_expr/struct.Expr.html#method.try_as_normal)
    (was `Expr::try_normal`)
  * [`Expr::try_as_symbol`](https://docs.rs/wolfram-expr/0.1.2/wolfram_expr/struct.Expr.html#method.try_as_symbol)
    (was `Expr::try_symbol`)
  * [`Expr::try_as_number`](https://docs.rs/wolfram-expr/0.1.2/wolfram_expr/struct.Expr.html#method.try_as_number)
    (was `Expr::try_number`)

* Add unstable [`ExprRefCmp`] type behind the `"unstable_parse"` feature flag. ([#12])



## [0.1.1] – 2022-02-18

### Added

* Added [`Expr::rule()`](https://docs.rs/wolfram-expr/0.1.1/wolfram_expr/struct.Expr.html#method.rule)
  and [`Expr::list()`](https://docs.rs/wolfram-expr/0.1.1/wolfram_expr/struct.Expr.html#method.list)
  methods for more convenient construction of `Rule` and `List` expressions. ([#5])

  Construct the expression `FontFamily -> "Courier New"`:

  ```rust
  use wolfram_expr::{Expr, Symbol};

  let option = Expr::rule(Symbol::new("System`FontFamily"), Expr::string("Courier New"));
  ```

  Construct the expression `{1, 2, 3}`:

  ```rust
  use wolfram_expr::Expr;

  let list = Expr::list(vec![Expr::from(1), Expr::from(2), Expr::from(3)]);
  ```



## [0.1.0] – 2022-02-08

### Added

* The [`Expr`](https://docs.rs/wolfram-expr/0.1.0/wolfram_expr/struct.Expr.html) type, for
  representing Wolfram Language expressions in an efficient and easy-to-process structure.

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




[#5]: https://github.com/WolframResearch/wolfram-expr-rs/pull/5

<!-- v0.1.2 -->
[#7]: https://github.com/WolframResearch/wolfram-expr-rs/pull/7
[#12]: https://github.com/WolframResearch/wolfram-expr-rs/pull/12


<!-- This needs to be updated for each tagged release. -->
[Unreleased]: https://github.com/WolframResearch/wolfram-expr-rs/compare/v0.1.2...HEAD

[0.1.2]: https://github.com/WolframResearch/wolfram-expr-rs/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/WolframResearch/wolfram-expr-rs/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/WolframResearch/wolfram-expr-rs/releases/tag/v0.1.0