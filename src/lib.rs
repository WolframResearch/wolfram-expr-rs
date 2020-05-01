#![allow(clippy::let_and_return)]

mod symbol;
mod symbol_table;

use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use static_assertions::{assert_eq_align, assert_eq_size};


pub use self::symbol::{AbsoluteContext, RelativeContext, Symbol, SymbolName};
pub use self::symbol_table::SymbolTable;

#[cfg(feature = "unstable_parse")]
pub mod parse {
    pub use crate::symbol::parse::*;
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Expr {
    inner: Arc<ExprKind>,
}

assert_eq_size!(Expr, usize);
assert_eq_align!(Expr, usize);

assert_eq_size!(Expr, *const ());
assert_eq_align!(Expr, *const ());

/// Newtype around Expr, which calculates it's Hash value based on the pointer value,
/// not the ExprKind.
///
/// TODO: Add tests that `ExprRefCmp` is working as expected
///
/// This is used in `wl_parse::source_map` to give unique source mapping, so that Expr's
/// which are equal according to the PartialEq impl for ExprKind (and whose hash values
/// are therefore the same) can be differenciated.
///
/// TODO: Rename this to ExprRefCmp
#[derive(Debug)]
pub struct ExprRefCmp {
    expr: Expr,
}

impl ExprRefCmp {
    pub fn new(expr: Expr) -> Self {
        ExprRefCmp { expr }
    }

    pub fn into_expr(self) -> Expr {
        self.expr
    }
}

impl Hash for ExprRefCmp {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Clone `expr` to increase the strong count. Otherwise expr would be dropped
        // inside of `Arc::into_raw` and the `Expr` could be deallocated.
        let ptr = Arc::into_raw(self.expr.inner.clone());
        ptr.hash(state);
        let _ = unsafe { Arc::from_raw(ptr) };
    }
}

impl PartialEq for ExprRefCmp {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.expr.inner, &other.expr.inner)
    }
}

impl Eq for ExprRefCmp {}

impl Expr {
    pub fn new(kind: ExprKind) -> Expr {
        Expr {
            inner: Arc::new(kind),
        }
    }

    /// Consume `self` and return an owned [`ExprKind`].
    ///
    /// If the reference count of `self` is equal to 1 this function will *not* perform
    /// a clone of the stored `ExprKind`, making this operation very cheap in that case.
    // Silence the clippy warning about this method. While this method technically doesn't
    // follow the Rust style convention of using `into` to prefix methods which take
    // `self` by move, I think using `to` is more appropriate given the expected
    // performance characteristics of this method. `into` implies that the method is
    // always returning data already owned by this type, and as such should be a very
    // cheap operation. This method can make no such guarantee; if the reference count is
    // 1, then performance is very good, but if the reference count is >1, a deeper clone
    // must be done.
    #[allow(clippy::wrong_self_convention)]
    pub fn to_kind(self) -> ExprKind {
        match Arc::try_unwrap(self.inner) {
            Ok(kind) => kind,
            Err(self_) => (*self_).clone(),
        }
    }

    pub fn kind(&self) -> &ExprKind {
        &*self.inner
    }

    pub fn kind_mut(&mut self) -> &mut ExprKind {
        Arc::make_mut(&mut self.inner)
    }

    pub fn ref_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }

    pub fn normal<H: Into<Expr>>(head: H, contents: Vec<Expr>) -> Expr {
        let head = head.into();
        // let contents = contents.into();
        Expr {
            inner: Arc::new(ExprKind::Normal(Normal { head, contents })),
        }
    }

    // TODO: Should Expr's be cached? Especially Symbol exprs? Would certainly save
    //       a lot of allocations.
    pub fn symbol<S: Into<Symbol>>(s: S) -> Expr {
        let s = s.into();
        Expr {
            inner: Arc::new(ExprKind::Symbol(s)),
        }
    }

    pub fn number(num: Number) -> Expr {
        Expr {
            inner: Arc::new(ExprKind::Number(num)),
        }
    }

    pub fn string<S: Into<String>>(s: S) -> Expr {
        Expr {
            inner: Arc::new(ExprKind::String(s.into())),
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
            ExprKind::Symbol(ref sym) => Some(sym.clone()),
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

    pub fn normal_head(&self) -> Option<Expr> {
        match *self.inner {
            ExprKind::Normal(ref normal) => Some(normal.head.clone()),
            ExprKind::Symbol(_) | ExprKind::Number(_) | ExprKind::String(_) => None,
        }
    }

    /// Attempt to get the element at `index` of a `Normal` expression.
    ///
    /// Return `None` if this is not a `Normal` expression, or the given index is out of
    /// bounds.
    ///
    /// `index` is 0-based. The 0th index is the first element, not the head.
    ///
    /// This function does not panic.
    pub fn normal_part(&self, index_0: usize) -> Option<&Expr> {
        match self.kind() {
            ExprKind::Normal(ref normal) => normal.contents.get(index_0),
            ExprKind::Symbol(_) | ExprKind::Number(_) | ExprKind::String(_) => None,
        }
    }

    pub fn try_normal(&self) -> Option<&Normal> {
        match self.kind() {
            ExprKind::Normal(ref normal) => Some(normal),
            ExprKind::Symbol(_) | ExprKind::String(_) | ExprKind::Number(_) => None,
        }
    }

    pub fn try_symbol(&self) -> Option<&Symbol> {
        match self.kind() {
            ExprKind::Symbol(ref symbol) => Some(symbol),
            ExprKind::Normal(_) | ExprKind::String(_) | ExprKind::Number(_) => None,
        }
    }

    /// Gets the head of all non-sub-value form `(_[___][___])` exprs as a symbol.
    ///
    /// ```text
    /// symbol_head(10) => Integer
    /// symbol_head(f[x]) => f
    /// symbol_head(f[x][y]) => None
    /// symbol_head(10[x]) => None
    /// ```
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
            match self.kind() {
                ExprKind::Number(num) => match num {
                    Number::Integer(_) => Some(Symbol::unchecked_new("System`Integer")),
                    Number::Real(_) => Some(Symbol::unchecked_new("System`Real")),
                },
                ExprKind::Symbol(_) => Some(Symbol::unchecked_new("System`Symbol")),
                ExprKind::String(_) => Some(Symbol::unchecked_new("System`String")),
                ExprKind::Normal(ref normal) => match normal.head.kind() {
                    ExprKind::Symbol(sym) => Some(sym.clone()),
                    _ => None,
                },
            }
        }
    }

    /// Returns `true` if `self` is a `Normal` expr with the head `sym`.
    pub fn has_normal_head(&self, sym: &Symbol) -> bool {
        match *self.kind() {
            ExprKind::Normal(ref normal) => normal.has_head(sym),
            _ => false,
        }
    }

    pub fn is_symbol(&self, sym: &Symbol) -> bool {
        match self.kind() {
            ExprKind::Symbol(ref self_sym) => self_sym == sym,
            _ => false,
        }
    }

    //==================================
    // Common values
    //==================================

    pub fn null() -> Expr {
        Expr::symbol(unsafe { Symbol::unchecked_new("System`Null") })
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum ExprKind<E = Expr> {
    Normal(Normal<E>),
    Number(Number),
    String(String),
    Symbol(Symbol),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Normal<E = Expr> {
    pub head: E,
    pub contents: Vec<E>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash)]
pub enum Number {
    // TODO: Rename this to MachineInteger
    Integer(i64),
    // TODO: Make an explicit MachineReal type which hides the inner f64, so that other
    //       code can make use of WL machine reals with a guaranteed type. In
    //       particular, change wl_compile::mir::Constant to use that type.
    Real(F64),
}

pub type F64 = ordered_float::NotNan<f64>;
pub type F32 = ordered_float::NotNan<f32>;

//=======================================
// Type Impl's
//=======================================

impl Normal {
    pub fn new<E: Into<Expr>>(head: E, contents: Vec<Expr>) -> Self {
        Normal {
            head: head.into(),
            contents,
        }
    }

    pub fn has_head(&self, sym: &Symbol) -> bool {
        self.head.is_symbol(sym)
    }
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

//=======================================
// Display & Debug impl/s
//=======================================

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Expr { inner } = self;
        write!(f, "{:?}", inner)
    }
}

/// By default, this should generate a string which can be unambiguously parsed to
/// reconstruct the `Expr` being displayed. This means symbols will always include their
/// contexts, special characters in String's will always be properly escaped, and numeric
/// literals needing precision and accuracy marks will have them.
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
            ExprKind::String(ref string) => {
                // Escape any '"' which appear in the string.
                // Using the Debug implementation will cause \n, \t, etc. to appear in
                // place of the literal character they are escapes for. This is necessary
                // when printing expressions in a way that they can be read back in as a
                // string, such as with ToExpression.
                write!(f, "{:?}", string)
            },
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
        write!(f, "{}[", self.head)?;
        for (idx, elem) in self.contents.iter().enumerate() {
            write!(f, "{}", elem)?;
            if idx != self.contents.len() - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "]")
    }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Number::Integer(ref int) => write!(f, "{}", int),
            Number::Real(ref real) => {
                // Make sure we're not printing NotNan (which surprisingly implements
                // Display)
                let real: f64 = **real;
                write!(f, "{:?}", real)
            },
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

// impl From<Number> for ExprKind {
//     fn from(number: Number) -> ExprKind {
//         ExprKind::Number(number)
//     }
// }
