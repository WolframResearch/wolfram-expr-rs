//! Conversions between [`num_complex::Complex<f64>`] and Wolfram
//! `Complex[re, im]` expressions.
//!
//! Mirrors the chrono bridge: gated behind the `num_complex` feature,
//! uses `System\`Complex` as the Normal head, and drops precision loss
//! on the user's lap via machine-`f64`.

use std::convert::{TryFrom, TryInto};

use num_complex::Complex;

use crate::{Expr, ExprKind, Symbol};

impl From<Complex<f64>> for Expr {
    fn from(c: Complex<f64>) -> Expr {
        Expr::normal(
            Symbol::new("System`Complex"),
            vec![Expr::real(c.re), Expr::real(c.im)],
        )
    }
}

/// Error returned when an `Expr` doesn't match `Complex[re, im]` with
/// machine-numeric components.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComplexConversionError(pub &'static str);

impl std::fmt::Display for ComplexConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Complex conversion: {}", self.0)
    }
}

impl std::error::Error for ComplexConversionError {}

fn part_as_f64(expr: &Expr) -> Option<f64> {
    match expr.kind() {
        &ExprKind::Integer(i) => Some(i as f64),
        &ExprKind::Real(r) => Some(r.into_inner()),
        _ => None,
    }
}

impl TryFrom<&Expr> for Complex<f64> {
    type Error = ComplexConversionError;

    fn try_from(expr: &Expr) -> Result<Self, Self::Error> {
        let normal = expr
            .try_as_normal()
            .ok_or(ComplexConversionError("not a Normal expression"))?;
        if !normal.has_head(&Symbol::new("System`Complex")) {
            return Err(ComplexConversionError("head is not System`Complex"));
        }
        if normal.elements().len() != 2 {
            return Err(ComplexConversionError("Complex needs exactly [re, im]"));
        }
        let re = part_as_f64(&normal.elements()[0]).ok_or(
            ComplexConversionError("real part not a machine number"),
        )?;
        let im = part_as_f64(&normal.elements()[1]).ok_or(
            ComplexConversionError("imaginary part not a machine number"),
        )?;
        Ok(Complex::new(re, im))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_real_parts() {
        let c = Complex::new(3.0_f64, 4.0_f64);
        let e: Expr = c.into();
        let back: Complex<f64> = (&e).try_into().unwrap();
        assert_eq!(c, back);
    }

    #[test]
    fn accepts_integer_components() {
        // WL produces Complex[3, 4] (Integer parts) for Gaussian integers.
        // Our bridge should still yield f64 components.
        let e = Expr::normal(
            Symbol::new("System`Complex"),
            vec![Expr::from(3_i64), Expr::from(4_i64)],
        );
        let c: Complex<f64> = (&e).try_into().unwrap();
        assert_eq!(c, Complex::new(3.0, 4.0));
    }

    #[test]
    fn rejects_wrong_head() {
        let e = Expr::list(vec![Expr::from(1), Expr::from(2)]);
        let r: Result<Complex<f64>, _> = (&e).try_into();
        assert!(r.is_err());
    }

    #[test]
    fn rejects_wrong_arity() {
        let e = Expr::normal(
            Symbol::new("System`Complex"),
            vec![Expr::from(1_i64)],
        );
        let r: Result<Complex<f64>, _> = (&e).try_into();
        assert!(r.is_err());
    }

    #[test]
    fn rejects_non_numeric_part() {
        let e = Expr::normal(
            Symbol::new("System`Complex"),
            vec![Expr::string("not a number"), Expr::from(4_i64)],
        );
        let r: Result<Complex<f64>, _> = (&e).try_into();
        assert!(r.is_err());
    }
}
