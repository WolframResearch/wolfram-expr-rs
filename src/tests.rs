use crate::symbol::{ContextRef, RelativeContext, SymbolNameRef, SymbolRef};
use crate::{Expr, ExprKind, Number, Symbol};
use num_bigint::BigInt;

//======================================
// BigInteger / BigReal variants
//======================================

fn big(s: &str) -> BigInt {
    s.parse().unwrap()
}

#[test]
fn bigint_basic_eq_hash_clone() {
    use std::collections::HashSet;
    let a = Expr::bigint(big("340282366920938463463374607431768211456"));
    let b = a.clone();
    assert_eq!(a, b);

    let mut set: HashSet<Expr> = HashSet::new();
    set.insert(a.clone());
    assert!(set.contains(&b));

    // Distinct magnitudes don't collide.
    let c = Expr::bigint(big("340282366920938463463374607431768211457"));
    assert_ne!(a, c);
    set.insert(c.clone());
    assert_eq!(set.len(), 2);
}

#[test]
fn bigint_negative_and_zero() {
    let neg = Expr::bigint(-BigInt::from(2u8).pow(300));
    let zero = Expr::bigint(BigInt::from(0));
    assert_ne!(neg, zero);
    // Display: bigints print their decimal form directly.
    assert!(format!("{}", neg).starts_with('-'));
    assert_eq!(format!("{}", zero), "0");
}

#[test]
fn bigint_is_not_integer_via_try_as_number() {
    // Arbitrary-precision integers are *not* reducible to `Number`, which is
    // machine-only. Callers have to dispatch on `ExprKind` explicitly.
    let big_expr = Expr::bigint(big("10000000000000000000000"));
    assert_eq!(big_expr.try_as_number(), None);
    assert!(matches!(big_expr.kind(), ExprKind::BigInteger(_)));
    assert_eq!(big_expr.tag(), None);
    assert_eq!(big_expr.normal_head(), None);
    assert_eq!(big_expr.normal_part(0), None);
    assert!(big_expr.try_as_normal().is_none());
    assert!(big_expr.try_as_symbol().is_none());
    assert!(big_expr.try_as_str().is_none());
    assert!(big_expr.try_as_bool().is_none());
}

#[test]
fn bigint_machine_value_stays_as_integer_when_constructed_explicitly() {
    // Invariant: if you explicitly call `Expr::bigint(42)`, the variant is
    // preserved as BigInteger even though it fits in i64. Folding is the
    // WXF decoder's concern (see `wxf::bigint_fits_in_i64_decodes_as_integer`).
    let e = Expr::bigint(BigInt::from(42));
    assert!(matches!(e.kind(), ExprKind::BigInteger(_)));
    // Machine Integer is a *different* variant.
    assert_ne!(e, Expr::from(42_i64));
}

#[test]
fn bigint_inside_normal() {
    // Normals should carry BigInteger children transparently.
    let inner = Expr::bigint(big("99999999999999999999"));
    let outer = Expr::list(vec![inner.clone(), Expr::from(1)]);
    assert_eq!(outer.normal_part(0), Some(&inner));
    assert_eq!(outer.normal_part(1), Some(&Expr::from(1)));
    assert_eq!(outer.normal_part(2), None);
    // `list` is a Normal with head `System\`List`.
    assert!(outer.has_normal_head(&Symbol::new("System`List")));
}

#[test]
fn bigreal_basic_eq_hash_display() {
    use std::collections::HashSet;
    let a = Expr::bigreal("3.14159265358979", 16);
    let b = a.clone();
    assert_eq!(a, b);

    let mut set: HashSet<Expr> = HashSet::new();
    set.insert(a.clone());
    assert!(set.contains(&b));

    let c = Expr::bigreal("3.14159265358979", 20); // different precision
    assert_ne!(a, c);

    // Display format: "digits`precision".
    assert_eq!(format!("{}", a), "3.14159265358979`16");
}

#[test]
fn bigreal_is_not_number_and_accessors_return_none() {
    let br = Expr::bigreal("1.2345", 8);
    assert_eq!(br.try_as_number(), None);
    assert!(matches!(br.kind(), ExprKind::BigReal { .. }));
    assert_eq!(br.tag(), None);
    assert_eq!(br.normal_head(), None);
    assert!(br.try_as_normal().is_none());
    assert!(br.try_as_symbol().is_none());
    assert!(br.try_as_str().is_none());
}

#[test]
fn bigreal_precision_zero_is_distinct_from_nonzero() {
    let zero_prec = Expr::bigreal("1.0", 0);
    let some_prec = Expr::bigreal("1.0", 1);
    assert_ne!(zero_prec, some_prec);
    match zero_prec.kind() {
        ExprKind::BigReal { precision, .. } => assert_eq!(*precision, 0),
        _ => panic!(),
    }
}

#[test]
fn machine_number_accessors_still_work() {
    // Regression: the variant additions must not break existing
    // `try_as_number` dispatch on Integer/Real.
    let i = Expr::from(7_i64);
    let r = Expr::real(2.5);
    assert_eq!(i.try_as_number(), Some(Number::Integer(7)));
    match r.try_as_number() {
        Some(Number::Real(nf)) => assert_eq!(nf.into_inner(), 2.5),
        _ => panic!(),
    }
}

/// `(input, is Symbol, is SymbolName, is Context, is RelativeContext)`
#[rustfmt::skip]
const DATA: &[(&str, bool, bool, bool, bool)] = &[
    // Symbol-like
    ("foo`bar",     true , false, false, false),
    ("foo`bar`baz", true , false, false, false),
    ("foo`bar5",    true , false, false, false),
    ("foo`5bar",    false, false, false, false),
    ("5foo`bar",    false, false, false, false),
    ("foo``bar",    false, false, false, false),
    ("foo`$bar",    true , false, false, false),
    ("$foo`$bar",   true , false, false, false),
    ("$foo`$$$",    true , false, false, false),
    ("$$$`$$$",     true , false, false, false),

    // SymbolName-like
    ("foo",         false, true,  false, false),
    ("foo5",        false, true,  false, false),
    ("foo5bar",     false, true,  false, false),
    ("$foo",        false, true,  false, false),
    ("5foo",        false, false, false, false),
    ("foo_bar",     false, false, false, false),
    ("_foo",        false, false, false, false),

    // TODO: RelativeSymbol-like
    ("`foo",        false, false, false, false),
    ("`foo`bar",    false, false, false, false),

    // Context-like
    ("foo`",        false, false, true,  false),
    ("foo`bar`",    false, false, true,  false),

    // RelativeContext-like
    ("`foo`",       false, false, false, true),
    ("`foo`bar`",   false, false, false, true),
];

#[test]
pub fn test_symbol_like_parsing() {
    for (input, is_symbol, is_symbol_name, is_context, is_rel_context) in
        DATA.iter().copied()
    {
        println!("input: {input}");
        assert_eq!(SymbolRef::try_new(input).is_some(), is_symbol);
        assert_eq!(SymbolNameRef::try_new(input).is_some(), is_symbol_name);
        assert_eq!(ContextRef::try_new(input).is_some(), is_context);
        assert_eq!(RelativeContext::try_new(input).is_some(), is_rel_context);
    }
}
