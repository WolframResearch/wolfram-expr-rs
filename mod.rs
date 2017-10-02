use std::fmt;

mod symbol;

pub use self::symbol::Symbol;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    Normal(Box<Normal>),
    Number(Number),
    String(String),
    Symbol(Symbol),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Normal {
    pub head: Expr,
    pub contents: Vec<Expr>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Number {
    Integer(i64),
    // Real(f64),
}

//=======================================
// Type Impl's
//=======================================

impl Normal {
    pub fn new(head: Expr, contents: Vec<Expr>) -> Self {
        Normal { head, contents }
    }

    pub fn has_head(&self, sym: Symbol) -> bool {
        match self.head {
            Expr::Symbol(self_head) => self_head == sym,
            _ => false
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Expr::Normal(ref normal) => fmt::Display::fmt(normal, f),
            Expr::Number(ref number) => fmt::Display::fmt(number, f),
            Expr::String(ref string) => write!(f, "\"{}\"", string),
            Expr::Symbol(ref symbol) => fmt::Display::fmt(symbol, f),
        }
    }
}

impl fmt::Display for Normal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{}[", self.head));
        for (idx, elem) in self.contents.iter().enumerate() {
            try!(write!(f, "{}", elem));
            if idx != self.contents.len() - 1 {
                try!(write!(f, ", "));
            }
        }
        write!(f, "]")
    }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Number::Integer(ref int) => write!(f, "{}", int),
            // Number::Real(ref real) => write!(f, "{}",  real),
        }
    }
}

impl From<Normal> for Expr {
    fn from(normal: Normal) -> Expr {
        Expr::Normal(Box::new(normal))
    }
}

impl From<Symbol> for Expr {
    fn from(symbol: Symbol) -> Expr {
        Expr::Symbol(symbol)
    }
}

impl From<Number> for Expr {
    fn from(number: Number) -> Expr {
        Expr::Number(number)
    }
}

pub(crate) fn column_offset_append(base: &mut String, s: &str, column: Option<usize>) {
    let column = column.unwrap_or_else(|| base.lines().last().unwrap_or("").chars().count());

    let mut lines = s.lines();
    match lines.next() {
        Some(line) => base.push_str(line),
        None => return
    };
    for line in lines {
        base.push_str("\n");
        indent_amount(base, column);
        base.push_str(line);
    }
}

fn indent_amount(base: &mut String, indent: usize) {
    base.reserve(indent);
    for _ in 0..indent {
        *base += " ";
    }
}
