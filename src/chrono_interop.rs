//! Conversions between [`chrono`] date/time types and Wolfram `DateObject`
//! expressions.
//!
//! The Wolfram `DateObject` is a normal expression of the form
//! `DateObject[{y, m, d, h, m, s}, granularity, calendar, timezone]` where
//! any suffix of the arg list is optional. We only bridge the two most
//! common shapes:
//!
//! | Rust type                | WL shape                                                           |
//! | ------------------------ | ------------------------------------------------------------------ |
//! | [`chrono::NaiveDate`]    | `DateObject[{y, m, d}]`                                            |
//! | [`chrono::DateTime<Utc>`]| `DateObject[{y, m, d, h, m, s}, "Instant", "Gregorian", "UTC"]`    |
//!
//! Sub-second precision is lost on the `DateTime<Utc>` round-trip because
//! WL's `DateObject` only stores integer-second components in the leaf list
//! (fractional seconds are carried as a `Real` in the sixth slot — we don't
//! emit that form). Callers that need millisecond/nanosecond precision should
//! serialize via [`chrono::DateTime::to_rfc3339`] and bridge the string
//! instead.

use std::convert::{TryFrom, TryInto};

use chrono::{DateTime, Datelike, NaiveDate, TimeZone, Timelike, Utc};

use crate::{Expr, ExprKind, Symbol};

//======================================
// Encode (Rust → Expr)
//======================================

impl From<NaiveDate> for Expr {
    fn from(d: NaiveDate) -> Expr {
        let components = Expr::list(vec![
            Expr::from(d.year() as i64),
            Expr::from(d.month() as i64),
            Expr::from(d.day() as i64),
        ]);
        Expr::normal(Symbol::new("System`DateObject"), vec![components])
    }
}

impl From<DateTime<Utc>> for Expr {
    fn from(dt: DateTime<Utc>) -> Expr {
        let components = Expr::list(vec![
            Expr::from(dt.year() as i64),
            Expr::from(dt.month() as i64),
            Expr::from(dt.day() as i64),
            Expr::from(dt.hour() as i64),
            Expr::from(dt.minute() as i64),
            Expr::from(dt.second() as i64),
        ]);
        Expr::normal(
            Symbol::new("System`DateObject"),
            vec![
                components,
                Expr::string("Instant"),
                Expr::string("Gregorian"),
                Expr::string("UTC"),
            ],
        )
    }
}

//======================================
// Decode (Expr → Rust)
//======================================

/// Error returned when an `Expr` doesn't match a supported `DateObject` shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateConversionError(pub &'static str);

impl std::fmt::Display for DateConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DateObject conversion: {}", self.0)
    }
}

impl std::error::Error for DateConversionError {}

fn components(expr: &Expr) -> Option<Vec<i64>> {
    let normal = expr.try_as_normal()?;
    if !normal.has_head(&Symbol::new("System`DateObject")) {
        return None;
    }
    let first = normal.elements().first()?;
    let list = first.try_as_normal()?;
    if !list.has_head(&Symbol::new("System`List")) {
        return None;
    }
    list.elements()
        .iter()
        .map(|e| match e.kind() {
            &ExprKind::Integer(i) => Some(i),
            _ => None,
        })
        .collect()
}

impl TryFrom<&Expr> for NaiveDate {
    type Error = DateConversionError;

    fn try_from(expr: &Expr) -> Result<Self, Self::Error> {
        let parts = components(expr)
            .ok_or(DateConversionError("not a DateObject[{...}] expression"))?;
        if parts.len() < 3 {
            return Err(DateConversionError("DateObject needs {y, m, d} at minimum"));
        }
        NaiveDate::from_ymd_opt(parts[0] as i32, parts[1] as u32, parts[2] as u32)
            .ok_or(DateConversionError("out-of-range date components"))
    }
}

impl TryFrom<&Expr> for DateTime<Utc> {
    type Error = DateConversionError;

    fn try_from(expr: &Expr) -> Result<Self, Self::Error> {
        let parts = components(expr)
            .ok_or(DateConversionError("not a DateObject[{...}] expression"))?;
        if parts.len() < 6 {
            return Err(DateConversionError(
                "DateTime requires {y, m, d, h, m, s}",
            ));
        }
        Utc.with_ymd_and_hms(
            parts[0] as i32,
            parts[1] as u32,
            parts[2] as u32,
            parts[3] as u32,
            parts[4] as u32,
            parts[5] as u32,
        )
        .single()
        .ok_or(DateConversionError("ambiguous / invalid date-time components"))
    }
}

//======================================
// Tests
//======================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn naivedate_round_trip() {
        let d = NaiveDate::from_ymd_opt(2026, 4, 14).unwrap();
        let e: Expr = d.into();
        let back: NaiveDate = (&e).try_into().unwrap();
        assert_eq!(d, back);
    }

    #[test]
    fn naivedate_produces_correct_shape() {
        let d = NaiveDate::from_ymd_opt(1999, 12, 31).unwrap();
        let e: Expr = d.into();
        assert_eq!(
            format!("{}", e),
            r#"System`DateObject[System`List[1999, 12, 31]]"#,
        );
    }

    #[test]
    fn datetime_round_trip() {
        let dt = Utc.with_ymd_and_hms(2026, 4, 14, 12, 30, 45).single().unwrap();
        let e: Expr = dt.into();
        let back: DateTime<Utc> = (&e).try_into().unwrap();
        assert_eq!(dt, back);
    }

    #[test]
    fn datetime_shape_includes_utc_qualifiers() {
        let dt = Utc.with_ymd_and_hms(2026, 4, 14, 0, 0, 0).single().unwrap();
        let e: Expr = dt.into();
        let s = format!("{}", e);
        assert!(s.contains("\"Instant\""));
        assert!(s.contains("\"Gregorian\""));
        assert!(s.contains("\"UTC\""));
    }

    #[test]
    fn rejects_non_dateobject() {
        let e = Expr::list(vec![Expr::from(1), Expr::from(2), Expr::from(3)]);
        let r: Result<NaiveDate, _> = (&e).try_into();
        assert!(r.is_err());
    }

    #[test]
    fn rejects_too_short_components_for_datetime() {
        let d = NaiveDate::from_ymd_opt(2026, 4, 14).unwrap();
        let e: Expr = d.into(); // only {y, m, d}
        let r: Result<DateTime<Utc>, _> = (&e).try_into();
        assert!(r.is_err());
    }

    #[test]
    fn rejects_out_of_range_components() {
        // Month 13 is invalid.
        let e = Expr::normal(
            Symbol::new("System`DateObject"),
            vec![Expr::list(vec![
                Expr::from(2026),
                Expr::from(13),
                Expr::from(1),
            ])],
        );
        let r: Result<NaiveDate, _> = (&e).try_into();
        assert!(r.is_err());
    }
}
