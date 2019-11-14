use std::fmt;
use std::rc::Rc;
use std::sync::Arc;
use std::hash::{Hash, Hasher};

extern crate string_interner;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate static_assertions;
extern crate ordered_float;

mod symbol;

pub use self::symbol::{Symbol, SymbolTable, InternedString};

// #[derive(Clone, PartialEq)]
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Expr {
    inner: Rc<ExprKind>,
}

assert_eq_size!(expr_size; Expr, usize);

/// A version of Expr which is shareable across threads.
#[derive(Clone)]
pub struct ArcExpr {
    inner: Arc<ExprKind<ArcExpr>>,
}

impl From<Expr> for ArcExpr {
    fn from(expr: Expr) -> Self {
        expr.to_arc_expr()
    }
}

impl From<ArcExpr> for Expr {
    fn from(expr: ArcExpr) -> Self {
        expr.to_rc_expr()
    }
}

/// Newtype around Expr, which calculates it's Hash value based on the pointer value,
/// not the ExprKind.
///
/// TODO: Add tests that `ExprRefHash` is working as expected
///
/// This is used in `wl_parse::source_map` to give unique source mapping, so that Expr's
/// which are equal according to the PartialEq impl for ExprKind (and whose hash values
/// are therefore the same) can be differenciated.
///
/// TODO: Rename this to ExprRefCmp
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
            inner: Rc::new(ExprKind::Normal(Normal { head, contents })),
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
    pub fn normal_part(&self, index: usize) -> Option<&Expr> {
        match self.kind() {
            ExprKind::Normal(ref normal) => normal.contents.get(index),
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

    /// Gets the head of all non-sub-value form (_[___][___]) exprs as a symbol.
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
                    _ => None
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

    pub fn to_arc_expr(&self) -> ArcExpr {
        let Expr { inner } = self;
        let kind: ExprKind<Expr> = inner.as_ref().clone();
        let kind: ExprKind<ArcExpr> = match kind {
            ExprKind::Normal(Normal { head, contents }) => {
                let contents = contents.iter().map(Expr::to_arc_expr).collect();
                let normal = Normal { head: head.to_arc_expr(), contents };
                ExprKind::Normal(normal)
            },
            ExprKind::Number(num) => ExprKind::Number(num),
            ExprKind::String(string) => ExprKind::String(string),
            ExprKind::Symbol(symbol) => ExprKind::Symbol(symbol),
        };
        ArcExpr::new(kind)
    }
}

impl ArcExpr {
    fn new(kind: ExprKind<ArcExpr>) -> ArcExpr {
        ArcExpr {
            inner: Arc::new(kind)
        }
    }

    pub fn to_rc_expr(&self) -> Expr {
        let ArcExpr { inner } = self;
        let kind: ExprKind<ArcExpr> = inner.as_ref().clone();
        let kind: ExprKind<Expr> = match kind {
            ExprKind::Normal(Normal { head, contents }) => {
                let contents = contents.iter().map(ArcExpr::to_rc_expr).collect();
                let normal = Normal { head: head.to_rc_expr(), contents };
                ExprKind::Normal(normal)
            },
            ExprKind::Number(num) => ExprKind::Number(num),
            ExprKind::String(string) => ExprKind::String(string),
            ExprKind::Symbol(symbol) => ExprKind::Symbol(symbol),
        };
        Expr::new(kind)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum ExprKind<E=Expr> {
    Normal(Normal<E>),
    Number(Number),
    String(String),
    Symbol(Symbol),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Normal<E=Expr> {
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
        Normal { head: head.into(), contents }
    }

    pub fn has_head(&self, sym: &Symbol) -> bool {
        match self.head.kind() {
            ExprKind::Symbol(self_head) => self_head == sym,
            _ => false
        }
    }
}

impl Number {
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
