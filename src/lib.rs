use std::fmt;
use std::rc::Rc;
use std::ops::Deref;
use std::hash::{Hash, Hasher};

extern crate string_interner;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate static_assertions;
extern crate ordered_float;

mod symbol;

pub use self::symbol::{Symbol, SymbolTable};

// #[derive(Clone, PartialEq)]
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Expr {
    inner: Rc<ExprKind>,
}

// TODO: Remove this in favor of Expr::kind.
impl Deref for Expr {
    type Target = ExprKind;
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

/// Newtype around Expr, which calculates it's Hash value based on the pointer value,
/// not the ExprKind.
///
/// TODO: Add tests that `ExprRefHash` is working as expected
///
/// This is used in `wl_parser::source_map` to give unique source mapping, so that Expr's
/// which are equal according to the PartialEq impl for ExprKind (and whose hash values
/// are therefore the same) can be differenciated.
pub struct ExprRefHash {
    expr: Expr
}

impl ExprRefHash {
    pub fn new(expr: Expr) -> Self {
        ExprRefHash { expr }
    }

    pub fn to_expr(self) -> Expr {
        self.expr
    }
}

impl Hash for ExprRefHash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Clone `expr` to increase the strong count. Otherwise expr would be dropped
        // inside of `Rc::into_raw` and the `Expr` could be deallocated.
        let ptr = Rc::into_raw(self.expr.inner.clone());
        ptr.hash(state);
        let _ = unsafe { Rc::from_raw(ptr) };
    }
}

impl PartialEq for ExprRefHash {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.expr.inner, &other.expr.inner)
    }
}

impl Eq for ExprRefHash {}

impl Expr {
    pub fn new(kind: ExprKind) -> Expr {
        Expr {
            inner: Rc::new(kind)
        }
    }

    pub fn to_kind(self) -> ExprKind {
        match Rc::try_unwrap(self.inner) {
            Ok(kind) => kind,
            Err(self_) => (*self_).clone(),
        }
    }

    pub fn kind(&self) -> &ExprKind {
        &*self.inner
    }

    pub fn kind_mut(&mut self) -> &mut ExprKind {
        Rc::make_mut(&mut self.inner)
    }

    pub fn normal<H: Into<Expr>>(head: H, contents: Vec<Expr>) -> Expr {
        let head = head.into();
        // let contents = contents.into();
        Expr {
            inner: Rc::new(ExprKind::Normal(Box::new(Normal { head, contents }))),
        }
    }

    // TODO: Should Expr's be cached? Especially Symbol exprs? Would certainly save
    //       a lot of allocations.
    pub fn symbol<S: Into<Symbol>>(s: S) -> Expr {
        let s = s.into();
        Expr {
            inner: Rc::new(ExprKind::Symbol(s))
        }
    }

    pub fn number(num: Number) -> Expr {
        Expr {
            inner: Rc::new(ExprKind::Number(num))
        }
    }

    pub fn string<S: Into<String>>(s: S) -> Expr {
        Expr {
            inner: Rc::new(ExprKind::String(s.into()))
        }
    }

    // TODO: _[x] probably should return None, even though technically
    //       Blank[][x] has the tag Blank.
    // TODO: The above TODO is probably wrong -- tag() shouldn't have any language
    //       semantics built in to it.
    pub fn tag(&self) -> Option<Symbol> {
        match *self.inner {
            ExprKind::Number(_) | ExprKind::String(_) => None,
            ExprKind::Normal(ref normal) => normal.head.tag(),
            // TODO: Remove this clone when Symbol becomes a Copy/Interned string
            ExprKind::Symbol(ref sym) => Some(sym.clone())
        }
    }

    pub fn head(&self) -> Expr {
        match *self.inner {
            // TODO Test: >>> Head[Head[67]] -> Symbol
            ExprKind::Number(num) => match num {
                Number::Integer(_) => Expr::symbol(self.symbol_head().unwrap()),
                Number::Real(_) => Expr::symbol(self.symbol_head().unwrap()),
            },
            ExprKind::Symbol(_) => Expr::symbol(self.symbol_head().unwrap()),
            ExprKind::String(_) => Expr::symbol(self.symbol_head().unwrap()),
            // TODO Test: Head[Plus[1, 1]]
            ExprKind::Normal(ref normal) => normal.head.clone(),
        }
    }

    /// Gets the head of all non-sub-value form (_[___][___]) exprs as a symbol.
    ///
    /// symbol_head(10) => Integer
    /// symbol_head(f[x]) => f
    /// symbol_head(f[x][y]) => None
    pub fn symbol_head(&self) -> Option<Symbol> {
        // QUIRK
        // TODO: This is one of the few places where I'm not sure about using
        //       `Symbol::unchecked_new`. The observed behavior in a NB is:
        //           >>> Remove[System`Integer]
        //           >>> Head[5]
        //             | System`Integer
        //       This means that Head adds back in the System`Integer/System`Symbol/etc.
        //       symbols when it's executed. We can certainly simply call
        //       `SymbolTable::add_symbol` in `builtin_downvalue_Head`, but it's
        //       concerning there might be other places this happens, making that fix a
        //       a specific solution to a more general problem (and therefore leaving
        //       holes for bugs).
        unsafe {
            match **self {
                ExprKind::Number(num) => match num {
                    Number::Integer(_) => Some(Symbol::unchecked_new("System`Integer")),
                    Number::Real(_) => Some(Symbol::unchecked_new("System`Real")),
                },
                ExprKind::Symbol(_) => Some(Symbol::unchecked_new("System`Symbol")),
                ExprKind::String(_) => Some(Symbol::unchecked_new("System`String")),
                ExprKind::Normal(ref normal) => match *normal.head {
                    ExprKind::Symbol(sym) => Some(sym),
                    _ => None
                },
            }
        }
    }

    /// Returns `true` if `self` is a `Normal` expr with the head `sym`.
    pub fn has_normal_head(&self, sym: Symbol) -> bool {
        match *self.kind() {
            ExprKind::Normal(ref normal) => normal.has_head(sym),
            _ => false,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum ExprKind {
    Normal(Box<Normal>), // TODO: Remove the box here, this indirection isn't needed after
                         //       making Expr an Rc type.
    Number(Number),
    String(String),
    Symbol(Symbol),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Normal {
    pub head: Expr,
    pub contents: Vec<Expr>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash)]
pub enum Number {
    // TODO: Rename this to MachineInteger
    Integer(i64),
    Real(ordered_float::OrderedFloat<f64>),
}

//=======================================
// Type Impl's
//=======================================

impl Normal {
    pub fn new<E: Into<Expr>>(head: E, contents: Vec<Expr>) -> Self {
        Normal { head: head.into(), contents }
    }

    pub fn has_head(&self, sym: Symbol) -> bool {
        match *self.head {
            ExprKind::Symbol(self_head) => self_head == sym,
            _ => false
        }
    }
}

impl Number {
    pub fn real(r: f64) -> Self {
        Number::Real(ordered_float::OrderedFloat(r))
    }
}

//=======================================
// Display & Debug impl/s
//=======================================

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}


impl fmt::Display for ExprKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ExprKind::Normal(ref normal) => fmt::Display::fmt(normal, f),
            ExprKind::Number(ref number) => fmt::Display::fmt(number, f),
            ExprKind::String(ref string) => write!(f, "\"{}\"", string),
            ExprKind::Symbol(ref symbol) => fmt::Display::fmt(symbol, f),
        }
    }
}


impl fmt::Debug for ExprKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
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
            Number::Real(ref real) => write!(f, "{:?}", real.0),
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

impl From<Box<Normal>> for Expr {
    fn from(normal: Box<Normal>) -> Expr {
        Expr {
            inner: Rc::new(ExprKind::Normal(normal))
        }
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

// impl From<Number> for ExprKind {
//     fn from(number: Number) -> ExprKind {
//         ExprKind::Number(number)
//     }
// }
