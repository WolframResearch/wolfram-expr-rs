use super::*;


impl Expr {
    /// If this is a [`Normal`] expression, return that. Otherwise return None.
    pub fn try_as_normal(&self) -> Option<&Normal> {
        match self.kind() {
            ExprKind::Normal(normal) => Some(normal),
            ExprKind::Symbol(_)
            | ExprKind::String(_)
            | ExprKind::Integer(_)
            | ExprKind::Real(_) => None,
        }
    }

    /// If this is a [True](http://reference.wolfram.com/language/ref/True.html) or [False](http://reference.wolfram.com/language/ref/False.html) value, return that. Otherwise return None.
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

impl From<Number> for ExprKind {
    fn from(number: Number) -> ExprKind {
        match number {
            Number::Integer(int) => ExprKind::Integer(int),
            Number::Real(real) => ExprKind::Real(real),
        }
    }
}

macro_rules! number_like {
    ($($t:ty),*) => {
        $(
            impl From<$t> for Expr {
                fn from(v: $t) -> Self {
                    Expr::number(Number::from(v))
                }
            }
        )*
    }
}

number_like![u8, u16, u32];
number_like![i8, i16, i32, i64];
