//! `serde::Serialize` / `serde::Deserialize` implementations for [`Expr`]
//! and a lossy [`serde_json::Value`] bridge.
//!
//! Wire format: an **externally-tagged** single-key JSON object per
//! variant. This is verbose but round-trip-lossless and unambiguous
//! across every serde format (JSON, bincode, MessagePack, YAML, …):
//!
//! ```text
//! {"integer":      42}
//! {"real":         3.14}
//! {"string":       "hi"}
//! {"symbol":       "System`List"}
//! {"bigInteger":   "123..."}                 // decimal string, not JSON number
//! {"bigReal":      {"digits":"3.14","precision":30}}
//! {"normal":       {"head": <expr>, "args": [<expr>, ...]}}
//! ```
//!
//! The `serde_json::Value ↔ Expr` bridge below is a **separate**, lossy
//! mapping for web-interop style use cases: JSON numbers collapse to
//! `Integer` or `Real`, booleans and null map to `System\`True/False/Null`,
//! objects to `Association` with string keys, arrays to `List`. Use
//! this when talking to non-WL services; use the native `Serialize`
//! form above for lossless Rust-to-Rust exchange.

use std::fmt;

use serde::{
    de::{self, DeserializeSeed, MapAccess, Visitor},
    ser::SerializeMap,
    Deserialize, Deserializer, Serialize, Serializer,
};

use crate::{Expr, ExprKind, Symbol};

//======================================
// Serialize
//======================================

impl Serialize for Expr {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(1))?;
        match self.kind() {
            ExprKind::Integer(i) => map.serialize_entry("integer", i)?,
            ExprKind::Real(r) => map.serialize_entry("real", &r.into_inner())?,
            ExprKind::String(s) => map.serialize_entry("string", s)?,
            ExprKind::Symbol(sym) => map.serialize_entry("symbol", sym.as_str())?,
            ExprKind::BigInteger(bi) => {
                map.serialize_entry("bigInteger", &bi.to_str_radix(10))?
            },
            ExprKind::BigReal { digits, precision } => {
                #[derive(Serialize)]
                struct BigRealWire<'a> {
                    digits: &'a str,
                    precision: u32,
                }
                map.serialize_entry(
                    "bigReal",
                    &BigRealWire {
                        digits: digits.as_str(),
                        precision: *precision,
                    },
                )?
            },
            ExprKind::Normal(normal) => {
                #[derive(Serialize)]
                struct NormalWire<'a> {
                    head: &'a Expr,
                    args: &'a [Expr],
                }
                map.serialize_entry(
                    "normal",
                    &NormalWire {
                        head: normal.head(),
                        args: normal.elements(),
                    },
                )?
            },
        }
        map.end()
    }
}

//======================================
// Deserialize
//======================================

impl<'de> Deserialize<'de> for Expr {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(ExprVisitor)
    }
}

struct ExprVisitor;

impl<'de> Visitor<'de> for ExprVisitor {
    type Value = Expr;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a single-key map tagging a wolfram-expr variant")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Expr, A::Error> {
        let tag: String = map
            .next_key()?
            .ok_or_else(|| de::Error::custom("empty map, expected variant tag"))?;
        let expr = match tag.as_str() {
            "integer" => Expr::from(map.next_value::<i64>()?),
            "real" => Expr::real(map.next_value::<f64>()?),
            "string" => Expr::string(map.next_value::<String>()?),
            "symbol" => {
                let s: String = map.next_value()?;
                Expr::symbol(Symbol::try_new(&s).ok_or_else(|| {
                    de::Error::custom(format!("not a valid symbol: {:?}", s))
                })?)
            },
            "bigInteger" => {
                let s: String = map.next_value()?;
                let bi: num_bigint::BigInt = s.parse().map_err(|_| {
                    de::Error::custom(format!("not a valid bigint: {:?}", s))
                })?;
                Expr::bigint(bi)
            },
            "bigReal" => {
                #[derive(Deserialize)]
                struct BigRealWire {
                    digits: String,
                    precision: u32,
                }
                let br: BigRealWire = map.next_value()?;
                Expr::bigreal(br.digits, br.precision)
            },
            "normal" => {
                #[derive(Deserialize)]
                struct NormalWire {
                    head: Expr,
                    args: Vec<Expr>,
                }
                let n: NormalWire = map.next_value()?;
                Expr::normal(n.head, n.args)
            },
            other => {
                return Err(de::Error::unknown_variant(
                    other,
                    &["integer", "real", "string", "symbol", "bigInteger", "bigReal", "normal"],
                ))
            },
        };
        // Refuse trailing keys — the format is single-key.
        if map.next_key::<String>()?.is_some() {
            return Err(de::Error::custom(
                "expected single-key map, found additional keys",
            ));
        }
        Ok(expr)
    }
}

// `DeserializeSeed` not needed since `Expr` isn't generic; wiring it up
// here just to silence unused-import lints when serde features
// cross-compile.
#[allow(dead_code)]
fn _seed_marker<'de, T: DeserializeSeed<'de>>() {}

//======================================
// serde_json::Value <-> Expr bridge
//======================================

impl From<&serde_json::Value> for Expr {
    fn from(v: &serde_json::Value) -> Expr {
        use serde_json::Value;
        match v {
            Value::Null => Expr::symbol(Symbol::new("System`Null")),
            Value::Bool(true) => Expr::symbol(Symbol::new("System`True")),
            Value::Bool(false) => Expr::symbol(Symbol::new("System`False")),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Expr::from(i)
                } else if let Some(f) = n.as_f64() {
                    Expr::real(f)
                } else {
                    // u64 > i64::MAX — preserve as BigInteger.
                    Expr::bigint(n.to_string().parse::<num_bigint::BigInt>().unwrap())
                }
            },
            Value::String(s) => Expr::string(s.clone()),
            Value::Array(items) => Expr::list(items.iter().map(Expr::from).collect()),
            Value::Object(obj) => {
                let rules: Vec<Expr> = obj
                    .iter()
                    .map(|(k, val)| {
                        Expr::rule(Expr::string(k.clone()), Expr::from(val))
                    })
                    .collect();
                Expr::normal(Symbol::new("System`Association"), rules)
            },
        }
    }
}

/// Error returned when an [`Expr`] shape doesn't fit the JSON data model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonBridgeError(pub String);

impl fmt::Display for JsonBridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JSON bridge: {}", self.0)
    }
}

impl std::error::Error for JsonBridgeError {}

/// Convert an [`Expr`] to a [`serde_json::Value`]. Lossy: `BigReal`
/// precision is dropped, non-string `Association` keys are rejected,
/// bignums round-trip as decimal strings.
pub fn expr_to_json(expr: &Expr) -> Result<serde_json::Value, JsonBridgeError> {
    use serde_json::Value;

    Ok(match expr.kind() {
        &ExprKind::Integer(i) => Value::from(i),
        &ExprKind::Real(r) => {
            // NaN was rejected at construction time via F64; any finite
            // f64 is representable in a serde_json number.
            serde_json::Number::from_f64(r.into_inner())
                .map(Value::Number)
                .unwrap_or(Value::Null)
        },
        ExprKind::String(s) => Value::from(s.as_str()),
        ExprKind::Symbol(sym) => match sym.as_str() {
            "System`True" => Value::Bool(true),
            "System`False" => Value::Bool(false),
            "System`Null" => Value::Null,
            other => Value::from(other),
        },
        ExprKind::BigInteger(bi) => Value::from(bi.to_str_radix(10)),
        ExprKind::BigReal { digits, .. } => Value::from(digits.as_str()),
        ExprKind::Normal(normal) => {
            let head = normal.head();
            let args = normal.elements();
            if let Some(sym) = head.try_as_symbol() {
                match sym.as_str() {
                    "System`List" => {
                        let items: Result<Vec<_>, _> =
                            args.iter().map(expr_to_json).collect();
                        return Ok(Value::Array(items?));
                    },
                    "System`Association" => {
                        let mut map = serde_json::Map::new();
                        for entry in args {
                            let n = entry.try_as_normal().ok_or_else(|| {
                                JsonBridgeError(
                                    "Association entry is not a Normal".into(),
                                )
                            })?;
                            if n.elements().len() != 2 {
                                return Err(JsonBridgeError(
                                    "Association entry has wrong arity".into(),
                                ));
                            }
                            let key = n.elements()[0].try_as_str().ok_or_else(|| {
                                JsonBridgeError(
                                    "Association key is not a String".into(),
                                )
                            })?;
                            map.insert(key.to_string(), expr_to_json(&n.elements()[1])?);
                        }
                        return Ok(Value::Object(map));
                    },
                    _ => {},
                }
            }
            // Fallback: encode generic Normal as {"normal": {"head": ..., "args": [...]}}
            // to stay JSON-valid without being lossy.
            let mut map = serde_json::Map::new();
            map.insert("head".into(), expr_to_json(head)?);
            let arg_vals: Result<Vec<_>, _> = args.iter().map(expr_to_json).collect();
            map.insert("args".into(), Value::Array(arg_vals?));
            let mut outer = serde_json::Map::new();
            outer.insert("normal".into(), Value::Object(map));
            Value::Object(outer)
        },
    })
}

//======================================
// Tests
//======================================

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigInt;

    fn json_round_trip(e: &Expr) -> Expr {
        let s = serde_json::to_string(e).unwrap();
        serde_json::from_str(&s).unwrap()
    }

    #[test]
    fn serde_integer() {
        let e = Expr::from(42_i64);
        assert_eq!(serde_json::to_string(&e).unwrap(), r#"{"integer":42}"#);
        assert_eq!(json_round_trip(&e), e);
    }

    #[test]
    fn serde_real() {
        let e = Expr::real(3.25);
        assert_eq!(serde_json::to_string(&e).unwrap(), r#"{"real":3.25}"#);
        assert_eq!(json_round_trip(&e), e);
    }

    #[test]
    fn serde_string_and_symbol() {
        assert_eq!(json_round_trip(&Expr::string("hi")), Expr::string("hi"));
        let sym = Expr::symbol(Symbol::new("Global`foo"));
        assert_eq!(json_round_trip(&sym), sym);
    }

    #[test]
    fn serde_bigint_as_string() {
        let bi: BigInt = "123456789012345678901234567890".parse().unwrap();
        let e = Expr::bigint(bi);
        let s = serde_json::to_string(&e).unwrap();
        assert!(s.contains("123456789012345678901234567890"));
        assert_eq!(json_round_trip(&e), e);
    }

    #[test]
    fn serde_bigreal() {
        let e = Expr::bigreal("3.14159", 10);
        let s = serde_json::to_string(&e).unwrap();
        assert!(s.contains("\"digits\":\"3.14159\""));
        assert!(s.contains("\"precision\":10"));
        assert_eq!(json_round_trip(&e), e);
    }

    #[test]
    fn serde_normal_nested() {
        let e = Expr::list(vec![
            Expr::from(1),
            Expr::from(2),
            Expr::list(vec![Expr::string("x"), Expr::from(3)]),
        ]);
        assert_eq!(json_round_trip(&e), e);
    }

    #[test]
    fn serde_rejects_unknown_variant() {
        let bad = r#"{"foo": 1}"#;
        let r: Result<Expr, _> = serde_json::from_str(bad);
        assert!(r.is_err());
    }

    #[test]
    fn serde_rejects_trailing_keys() {
        let bad = r#"{"integer": 1, "real": 2.0}"#;
        let r: Result<Expr, _> = serde_json::from_str(bad);
        assert!(r.is_err());
    }

    // ----- json bridge (lossy) -----

    #[test]
    fn json_value_number_to_expr() {
        let v: serde_json::Value = serde_json::from_str("42").unwrap();
        assert_eq!(Expr::from(&v), Expr::from(42_i64));
        let v: serde_json::Value = serde_json::from_str("3.14").unwrap();
        assert_eq!(Expr::from(&v), Expr::real(3.14));
    }

    #[test]
    fn json_value_null_bool() {
        let v: serde_json::Value = serde_json::from_str("null").unwrap();
        assert_eq!(Expr::from(&v), Expr::symbol(Symbol::new("System`Null")));
        let v: serde_json::Value = serde_json::from_str("true").unwrap();
        assert_eq!(Expr::from(&v), Expr::symbol(Symbol::new("System`True")));
    }

    #[test]
    fn json_value_object_to_association() {
        let v: serde_json::Value =
            serde_json::from_str(r#"{"a": 1, "b": "x"}"#).unwrap();
        let expr = Expr::from(&v);
        // Should be Association[Rule["a", 1], Rule["b", "x"]]
        let normal = expr.try_as_normal().unwrap();
        assert!(normal.has_head(&Symbol::new("System`Association")));
        assert_eq!(normal.elements().len(), 2);
    }

    #[test]
    fn json_value_array_to_list() {
        let v: serde_json::Value = serde_json::from_str("[1, 2, 3]").unwrap();
        let expr = Expr::from(&v);
        assert_eq!(
            expr,
            Expr::list(vec![
                Expr::from(1_i64),
                Expr::from(2_i64),
                Expr::from(3_i64),
            ])
        );
    }

    #[test]
    fn expr_to_json_bridge() {
        let e = Expr::list(vec![
            Expr::from(1_i64),
            Expr::normal(
                Symbol::new("System`Association"),
                vec![Expr::rule(Expr::string("k"), Expr::string("v"))],
            ),
            Expr::symbol(Symbol::new("System`True")),
        ]);
        let v = expr_to_json(&e).unwrap();
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!(s, r#"[1,{"k":"v"},true]"#);
    }

    #[test]
    fn expr_to_json_rejects_non_string_assoc_keys() {
        let e = Expr::normal(
            Symbol::new("System`Association"),
            vec![Expr::rule(Expr::from(1_i64), Expr::string("v"))],
        );
        assert!(expr_to_json(&e).is_err());
    }
}
