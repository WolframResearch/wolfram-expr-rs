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

impl From<u8> for Number {
    fn from(n: u8) -> Self {
        Number::Integer(n as i64)
    }
}

impl From<u16> for Number {
    fn from(n: u16) -> Self {
        Number::Integer(n as i64)
    }
}

impl From<u32> for Number {
    fn from(n: u32) -> Self {
        Number::Integer(n as i64)
    }
}

// range not safe
// impl From<u64> for Number {
//     fn from(n: u64) -> Self {
//         Number::Integer(n as i64)
//     }
// }

impl From<i8> for Number {
    fn from(n: u32) -> Self {
        Number::Integer(n as i64)
    }
}

impl From<i16> for Number {
    fn from(n: i16) -> Self {
        Number::Integer(n as i64)
    }
}

impl From<i32> for Number {
    fn from(n: u32) -> Self {
        Number::Integer(n as i64)
    }
}

impl From<i64> for Number {
    fn from(n: u32) -> Self {
        Number::Integer(n as i64)
    }
}
