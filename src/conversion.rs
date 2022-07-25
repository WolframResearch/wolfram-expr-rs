use super::*;


impl Expr {
    /// If this is a [`Normal`] expression, return that. Otherwise return None.
    pub fn try_as_normal(&self) -> Option<&Normal> {
        match self.kind() {
            ExprKind::Normal(ref normal) => Some(normal),
            ExprKind::Symbol(_)
            | ExprKind::String(_)
            | ExprKind::Integer(_)
            | ExprKind::Real(_) => None,
        }
    }

    /// If this is a [`True`](http://reference.wolfram.com/language/ref/True.html)
    /// or [`False`](http://reference.wolfram.com/language/ref/False.html) symbol,
    /// return that. Otherwise return None.
    pub fn try_as_bool(&self) -> Option<bool> {
        let s = self.try_as_symbol()?;
        if s.as_str() == "System`True" {
            return Some(true);
        }
        if s.as_str() == "System`False" {
            return Some(false);
        }
        None
    }

    /// If this is a [`ExprKind::String`] expression, return that. Otherwise return None.
    pub fn try_as_str(&self) -> Option<&str> {
        match self.kind() {
            ExprKind::String(ref string) => Some(string.as_str()),
            _ => None,
        }
    }

    /// If this is a [`Symbol`] expression, return that. Otherwise return None.
    pub fn try_as_symbol(&self) -> Option<&Symbol> {
        match self.kind() {
            ExprKind::Symbol(ref symbol) => Some(symbol),
            ExprKind::Normal(_)
            | ExprKind::String(_)
            | ExprKind::Integer(_)
            | ExprKind::Real(_) => None,
        }
    }

    /// If this is a [`Number`] expression, return that. Otherwise return None.
    pub fn try_as_number(&self) -> Option<Number> {
        match self.kind() {
            ExprKind::Integer(int) => Some(Number::Integer(*int)),
            ExprKind::Real(real) => Some(Number::Real(*real)),
            ExprKind::Normal(_) | ExprKind::String(_) | ExprKind::Symbol(_) => None,
        }
    }

    //---------------------------------------------------------------------
    // SEMVER: This methods have been replaced, remove in a future version.
    //---------------------------------------------------------------------

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
    fn from(sym: Symbol) -> Expr {
        Expr::symbol(sym)
    }
}

impl From<&Symbol> for Expr {
    fn from(sym: &Symbol) -> Expr {
        Expr::symbol(sym)
    }
}

impl From<Normal> for Expr {
    fn from(normal: Normal) -> Expr {
        Expr {
            inner: Arc::new(ExprKind::Normal(normal)),
        }
    }
}

impl From<bool> for Expr {
    fn from(value: bool) -> Expr {
        match value {
            true => Expr::symbol(Symbol::new("System`True")),
            false => Expr::symbol(Symbol::new("System`False")),
        }
    }
}

macro_rules! string_like {
    ($($t:ty),*) => {
        $(
            impl From<$t> for Expr {
                fn from(s: $t) -> Expr {
                    Expr::string(s)
                }
            }
        )*
    }
}

string_like!(&str, &String, String);

//--------------------
// Integer conversions
//--------------------

impl From<u8> for Expr {
    fn from(int: u8) -> Expr {
        Expr::from(i64::from(int))
    }
}

impl From<i8> for Expr {
    fn from(int: i8) -> Expr {
        Expr::from(i64::from(int))
    }
}

impl From<u16> for Expr {
    fn from(int: u16) -> Expr {
        Expr::from(i64::from(int))
    }
}

impl From<i16> for Expr {
    fn from(int: i16) -> Expr {
        Expr::from(i64::from(int))
    }
}

impl From<u32> for Expr {
    fn from(int: u32) -> Expr {
        Expr::from(i64::from(int))
    }
}

impl From<i32> for Expr {
    fn from(int: i32) -> Expr {
        Expr::from(i64::from(int))
    }
}

impl From<i64> for Expr {
    fn from(int: i64) -> Expr {
        Expr::number(Number::Integer(int))
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
    fn from(number: Number) -> ExprKind {
        match number {
            Number::Integer(int) => ExprKind::Integer(int),
            Number::Real(real) => ExprKind::Real(real),
        }
    }
}
