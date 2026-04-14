//! Property tests for the WXF codec. These complement the inline unit tests
//! in `src/wxf.rs` with broad random coverage asserting `decode ∘ encode = id`
//! (and a compressed-form variant).

#![cfg(feature = "wxf")]

use proptest::prelude::*;
use wolfram_expr::{
    wxf::{from_wxf_bytes, to_wxf_bytes, to_wxf_bytes_compressed},
    Expr, Symbol,
};

fn leaf() -> impl Strategy<Value = Expr> {
    prop_oneof![
        any::<i64>().prop_map(Expr::from),
        (-1.0e6_f64..1.0e6_f64)
            .prop_filter("no NaN", |r| !r.is_nan())
            .prop_map(Expr::real),
        "[a-zA-Z][a-zA-Z0-9]{0,8}".prop_map(|s| Expr::symbol(Symbol::new(&format!(
            "Global`{}",
            s
        )))),
        ".*".prop_map(Expr::string),
    ]
}

fn expr_strategy() -> impl Strategy<Value = Expr> {
    leaf().prop_recursive(3, 16, 4, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..4).prop_map(Expr::list),
            prop::collection::vec(inner.clone(), 0..3).prop_map(|args| Expr::normal(
                Symbol::new("Global`f"),
                args
            )),
        ]
    })
}

proptest! {
    #[test]
    fn roundtrip_uncompressed(e in expr_strategy()) {
        let bytes = to_wxf_bytes(&e).unwrap();
        let decoded = from_wxf_bytes(&bytes).unwrap();
        prop_assert_eq!(decoded, e);
    }

    #[test]
    fn roundtrip_compressed(e in expr_strategy()) {
        let bytes = to_wxf_bytes_compressed(&e).unwrap();
        let decoded = from_wxf_bytes(&bytes).unwrap();
        prop_assert_eq!(decoded, e);
    }
}
