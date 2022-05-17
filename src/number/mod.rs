/// 64-bit floating-point real number. Not NaN.
pub type F64 = ordered_float::NotNan<f64>;
/// 32-bit floating-point real number. Not NaN.
pub type F32 = ordered_float::NotNan<f32>;

/// Subset of [`ExprKind`] that covers number-type expression values.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash)]
pub enum Number {
    // TODO: Rename this to MachineInteger
    Integer(i64),
    // TODO: Make an explicit MachineReal type which hides the inner f64, so that other
    //       code can make use of WL machine reals with a guaranteed type. In
    //       particular, change wl_compile::mir::Constant to use that type.
    Real(F64),
}

impl Number {
    /// # Panics
    ///
    /// This function will panic if `r` is NaN.
    ///
    /// TODO: Change this function to take `NotNan` instead, so the caller doesn't have to
    ///       worry about panics.
    pub fn real(r: f64) -> Self {
        let r = match ordered_float::NotNan::new(r) {
            Ok(r) => r,
            Err(_) => panic!("Number::real: got NaN"),
        };
        Number::Real(r)
    }
}


macro_rules! integer_like {
    ($($t:ty),*) => {
        $(
            impl From<$t> for Number {
                fn from(n: $t) -> Self {
                    Number::Integer(n as i64)
                }
            }
            impl From<&$t> for Number {
                fn from(n: &$t) -> Self {
                    Number::Integer(*n as i64)
                }
            }
        )*
    }
}

// except range not safe
integer_like![u8, u16, u32];
integer_like![i8, i16, i32, i64];
