use super::*;


impl Expr {
    /// If this is a [`Normal`] expression, return that. Otherwise return None.
    pub fn try_as_normal(&self) -> Option<&Normal> {
        let ExprKind::Normal(ref normal) = self.kind() else {
            return None;
        };
        Some(normal)
    }

    /// If this is a [`True`](http://reference.wolfram.com/language/ref/True.html)
    /// or [`False`](http://reference.wolfram.com/language/ref/False.html) symbol,
    /// return that. Otherwise return None.
    pub fn try_as_bool(&self) -> Option<bool> {
        match self.try_as_symbol()?.as_str() {
            "System`True" => Some(true),
            "System`False" => Some(false),
            _ => None,
        }
    }

    /// If this is a [`ExprKind::String`] expression, return that. Otherwise return None.
    pub fn try_as_str(&self) -> Option<&str> {
        let ExprKind::String(ref string) = self.kind() else {
            return None;
        };
        Some(string.as_str())
    }

    /// If this is a [`Symbol`] expression, return that. Otherwise return None.
    pub fn try_as_symbol(&self) -> Option<&Symbol> {
        let ExprKind::Symbol(ref symbol) = self.kind() else {
            return None;
        };
        Some(symbol)
    }

    /// If this is a [`Number`] expression, return that. Otherwise return None.
    pub fn try_as_number(&self) -> Option<Number> {
        match self.kind() {
            ExprKind::Integer(int) => Some(Number::Integer(*int)),
            ExprKind::Real(real) => Some(Number::Real(*real)),
            ExprKind::Normal(_) | ExprKind::String(_) | ExprKind::Symbol(_) => None,
        }
    }

    //---------------------------------------------------------------------------
    // SEMVER: These methods have been replaced; remove them in a future version.
    //---------------------------------------------------------------------------

    #[deprecated(note = "Use Expr::try_as_normal() instead")]
    #[allow(missing_docs)]
    pub fn try_normal(&self) -> Option<&Normal> {
        self.try_as_normal()
    }

    #[deprecated(note = "Use Expr::try_as_symbol() instead")]
    #[allow(missing_docs)]
    pub fn try_symbol(&self) -> Option<&Symbol> {
        self.try_as_symbol()
    }

    #[deprecated(note = "Use Expr::try_as_number() instead")]
    #[allow(missing_docs)]
    pub fn try_number(&self) -> Option<Number> {
        self.try_as_number()
    }
}

//=======================================
// Conversion trait impl's
//=======================================

impl From<Symbol> for Expr {
    fn from(sym: Symbol) -> Self {
        Self::symbol(sym)
    }
}

impl From<&Symbol> for Expr {
    fn from(sym: &Symbol) -> Self {
        Self::symbol(sym)
    }
}

impl From<Normal> for Expr {
    fn from(normal: Normal) -> Self {
        Self {
            inner: Arc::new(ExprKind::Normal(normal)),
        }
    }
}

impl From<bool> for Expr {
    fn from(value: bool) -> Self {
        Self::symbol(Symbol::new(if value { "System`True" } else { "System`False" }))
    }
}

macro_rules! string_like {
    ($($t:ty),*) => {
        $(
            impl From<$t> for Expr {
                fn from(s: $t) -> Self {
                    Self::string(s)
                }
            }
        )*
    }
}

string_like!(&str, &String, String);

//--------------------
// Integer conversions
//--------------------

macro_rules! u64_convertible {
    ($($t:ty),*) => {
        $(
            impl From<$t> for Expr {
                fn from(s: $t) -> Self {
                    Self::from(i64::from(s))
                }
            }
        )*
    }
}

u64_convertible!(u8, i8, u16, i16, u32, i32);

impl From<i64> for Expr {
    fn from(int: i64) -> Self {
        Self::number(Number::Integer(int))
    }
}

// impl From<Normal> for ExprKind {
//     fn from(normal: Normal) -> ExprKind {
//         ExprKind::Normal(Box::new(normal))
//     }
// }

// impl From<Symbol> for ExprKind {
//     fn from(symbol: Symbol) -> ExprKind {
//         ExprKind::Symbol(symbol)
//     }
// }

impl From<Number> for ExprKind {
    fn from(number: Number) -> Self {
        match number {
            Number::Integer(int) => Self::Integer(int),
            Number::Real(real) => Self::Real(real),
        }
    }
}
