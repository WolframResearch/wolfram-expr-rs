//! Representation of Wolfram Language symbols.
//!
//! This module provides four primary types:
//!
//! * [`Symbol`]
//! * [`SymbolName`]
//! * [`Context`]
//! * [`RelativeContext`]
//!
//! These types are used for storing a string value that has been validated to conform
//! to the syntax of Wolfram Language [symbols and contexts][ref/SymbolNamesAndContexts].
//!
//! In addition to the previous types, which own their string value, types are provided
//! that can be used to validate a borrowed `&str` value, without requiring another
//! allocation:
//!
//! * [`SymbolRef`]
//! * [`SymbolNameRef`]
//! * [`ContextRef`]
// * TODO: `RelativeContextRef`
//!
//! ## Related Links
//!
//! * [Input Syntax: Symbol Names and Contexts][ref/SymbolNamesAndContexts]
//!
//! [ref/SymbolNamesAndContexts]: https://reference.wolfram.com/language/tutorial/InputSyntax.html#6562

pub(crate) mod parse;

use std::{
    fmt::{self, Debug, Display},
    mem,
    sync::Arc,
};


/* Notes

Operations on Symbols

- Format (with conditional context path based on $Context)
- Test for equality
- Lookup symbol name in context path while parsing
- Remove / format Removed["..."]

*/

//==========================================================
// Types
//==========================================================

//======================================
// Owned Data
//======================================

// TODO: Change these types to be Arc<str>. This has the consequence of increasing the
//       size of these types from 64-bits to 128 bits, so first take care that they are
//       not passed through a C FFI anywhere as a pointer-sized type.

/// Wolfram Language symbol.
///
/// # PartialOrd sorting order
///
/// The comparison behavior of this type is **NOT** guaranteed to match the behavior of
/// `` System`Order `` for symbols (and does *not* match it at the moment).
///
/// This type implements `PartialOrd`/`Ord` primarily for the purposes of allowing
/// instances of this type to be included in ordered sets (e.g. `BTreeMap`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct Symbol(Arc<String>);

/// The identifier portion of a symbol. This contains no context marks ('`').
///
/// In the symbol `` Global`foo ``, the `SymbolName` is `"foo"`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymbolName(Arc<String>);

/// Wolfram Language context.
///
/// Examples: `` System` ``, `` Global` ``, `` MyPackage`Utils` ``, etc.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Context(Arc<String>);

/// Context begining with a `` ` ``.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RelativeContext(Arc<String>);

// By using `usize` here, we guarantee that we can later change this to be a pointer
// instead without changing the sizes of a lot of Expr types. This is good for FFI/ABI
// compatibility if I decide to change the way Symbol works.
const _: () = assert!(mem::size_of::<Symbol>() == mem::size_of::<usize>());
const _: () = assert!(mem::align_of::<Symbol>() == mem::align_of::<usize>());

//======================================
// Borrowed Data
//======================================

/// Borrowed string containing a valid symbol.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymbolRef<'s>(&'s str);

/// Borrowing string containing a valid symbol name.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymbolNameRef<'s>(&'s str);

/// Borrowed string containing a valid context.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContextRef<'s>(pub(super) &'s str);

//==========================================================
// Impls -- Owned Types
//==========================================================

impl From<&Symbol> for Symbol {
    fn from(sym: &Symbol) -> Self {
        sym.clone()
    }
}

impl Symbol {
    /// Attempt to parse `input` as an absolute symbol.
    ///
    /// An absolute symbol is a symbol with an explicit context path. ``"System`Plus"`` is
    /// an absolute symbol, ``"Plus"`` is a relative symbol and/or a [`SymbolName`].
    /// ``"`Plus"`` is also a relative symbol.
    pub fn try_new(input: &str) -> Option<Self> {
        let sym_ref = SymbolRef::try_new(input)?;

        Some(sym_ref.to_symbol())
    }

    /// Construct a symbol from `input`.
    ///
    /// # Panics
    ///
    /// This function will panic if `input` is not a valid Wolfram Language symbol.
    /// `Symbol::try_new(input)` must succeed.
    ///
    /// This method is intended to be used for convenient construction of symbols from
    /// string literals, where an error is unlikely to occur, e.g.:
    ///
    /// ```
    /// # use wolfram_expr::{Expr, Symbol};
    /// let expr = Expr::normal(Symbol::new("MyPackage`Foo"), vec![]);
    /// ```
    ///
    /// If not using a string literal as the argument, prefer to use [`Symbol::try_new`]
    /// and handle the error condition.
    #[track_caller]
    pub fn new(input: &str) -> Self {
        match Symbol::try_new(input) {
            Some(symbol) => symbol,
            None => panic!("string is not parseable as a symbol: {}", input),
        }
    }

    /// Get a borrowed [`SymbolRef`] from this [`Symbol`].
    pub fn as_symbol_ref(&self) -> SymbolRef {
        let Symbol(arc_string) = self;

        SymbolRef(arc_string.as_str())
    }

    /// Get the context path part of a symbol as an [`ContextRef`].
    pub fn context(&self) -> ContextRef {
        self.as_symbol_ref().context()
    }

    /// Get the symbol name part of a symbol as a [`SymbolNameRef`].
    pub fn symbol_name(&self) -> SymbolNameRef {
        self.as_symbol_ref().symbol_name()
    }
}

impl SymbolName {
    /// Attempt to parse `input` as a symbol name.
    ///
    /// A symbol name is a symbol without any context marks.
    pub fn try_new(input: &str) -> Option<SymbolName> {
        SymbolNameRef::try_new(input)
            .as_ref()
            .map(SymbolNameRef::to_symbol_name)
    }

    /// Get a borrowed [`SymbolNameRef`] from this `SymbolName`.
    pub fn as_symbol_name_ref(&self) -> SymbolNameRef {
        SymbolNameRef(self.as_str())
    }
}

impl Context {
    /// Attempt to parse `input` as a context.
    pub fn try_new(input: &str) -> Option<Self> {
        let context_ref = ContextRef::try_new(input)?;

        Some(context_ref.to_context())
    }

    /// Construct a context from `input`.
    ///
    /// # Panics
    ///
    /// This function will panic if `input` is not a valid Wolfram Language context.
    /// `Context::try_new(input)` must succeed.
    ///
    /// This method is intended to be used for convenient construction of contexts from
    /// string literals, where an error is unlikely to occur, e.g.:
    ///
    /// ```
    /// use wolfram_expr::symbol::Context;
    ///
    /// let context = Context::new("MyPackage`");
    /// ```
    ///
    /// If not using a string literal as the argument, prefer to use [`Context::try_new`]
    /// and handle the error condition.
    #[track_caller]
    pub fn new(input: &str) -> Self {
        match Context::try_new(input) {
            Some(context) => context,
            None => panic!("string is not parseable as a context: {}", input),
        }
    }

    /// The `` Global` `` context.
    pub fn global() -> Self {
        Context(Arc::new(String::from("Global`")))
    }

    /// The `` System` `` context.
    pub fn system() -> Self {
        Context(Arc::new(String::from("System`")))
    }

    /// Construct a new [`Context`] by appending a new context component to this
    /// context.
    ///
    /// ```
    /// use wolfram_expr::symbol::{Context, SymbolName, SymbolNameRef};
    ///
    /// let context = Context::from_symbol_name(&SymbolName::try_new("MyContext").unwrap());
    /// let private = context.join(SymbolNameRef::try_new("Private").unwrap());
    ///
    /// assert!(private.as_str() == "MyContext`Private`");
    /// ```
    pub fn join(&self, name: SymbolNameRef) -> Context {
        let Context(context) = self;
        Context::try_new(&format!("{}{}`", context, name.as_str()))
            .expect("Context::join(): invalid Context")
    }

    /// Return the components of this [`Context`].
    ///
    /// ```
    /// use wolfram_expr::symbol::Context;
    ///
    /// let context = Context::new("MyPackage`Sub`Module`");
    ///
    /// let components = context.components();
    ///
    /// assert!(components.len() == 3);
    /// assert!(components[0].as_str() == "MyPackage");
    /// assert!(components[1].as_str() == "Sub");
    /// assert!(components[2].as_str() == "Module");
    /// ```
    pub fn components(&self) -> Vec<SymbolNameRef> {
        let Context(string) = self;

        let comps: Vec<SymbolNameRef> = string
            .split('`')
            // Remove the last component, which will always be the empty string
            .filter(|comp| !comp.is_empty())
            .map(|comp| {
                SymbolNameRef::try_new(comp)
                    .expect("Context::components(): invalid context component")
            })
            .collect();

        comps
    }

    /// Get a borrowed [`ContextRef`] from this `Context`.
    pub fn as_context_ref(&self) -> ContextRef {
        ContextRef(self.as_str())
    }

    /// Create the context `` name` ``.
    pub fn from_symbol_name(name: &SymbolName) -> Self {
        Context::try_new(&format!("{}`", name)).unwrap()
    }
}

impl RelativeContext {
    /// Attempt to parse `input` as a relative context.
    pub fn try_new(input: &str) -> Option<Self> {
        crate::symbol::parse::RelativeContext_try_new(input)
    }

    /// Return the components of this [`RelativeContext`].
    ///
    /// ```
    /// use wolfram_expr::symbol::RelativeContext;
    ///
    /// let context = RelativeContext::try_new("`Sub`Module`").unwrap();
    ///
    /// let components = context.components();
    ///
    /// assert!(components.len() == 2);
    /// assert!(components[0].as_str() == "Sub");
    /// assert!(components[1].as_str() == "Module");
    /// ```
    pub fn components(&self) -> Vec<SymbolNameRef> {
        let RelativeContext(string) = self;

        let comps: Vec<SymbolNameRef> = string
            .split('`')
            // Remove the last component, which will always be the empty string
            .filter(|comp| !comp.is_empty())
            .map(|comp| {
                SymbolNameRef::try_new(comp)
                    .expect("RelativeContext::components(): invalid context component")
            })
            .collect();

        comps
    }
}

macro_rules! common_impls {
    (impl $ty:ident) => {
        impl Display for $ty {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                let $ty(string) = self;

                write!(f, "{}", string)
            }
        }

        impl $ty {
            /// Get the underlying `&str` representation of this type.
            pub fn as_str(&self) -> &str {
                let $ty(string) = self;

                string.as_str()
            }

            /// Create a new instance of this type from a string, without validating the
            /// string contents.
            ///
            /// It's up to the caller to ensure that the passed `input` has the correct
            /// syntax.
            ///
            /// ## Safety
            ///
            /// This function actually does not do anything that would be rejected by
            /// rustc were the function not marked `unsafe`. However, this function is so
            /// often *not* what is really needed, it's marked unsafe as a deterent to
            /// possible users.
            pub(crate) unsafe fn unchecked_new<S: Into<String>>(input: S) -> $ty {
                let inner: Arc<String> = Arc::new(input.into());
                $ty(inner)
            }
        }
    };
}

common_impls!(impl Symbol);
common_impls!(impl SymbolName);
common_impls!(impl Context);
common_impls!(impl RelativeContext);

//==========================================================
// Impls -- Borrowed Types
//==========================================================

impl<'s> SymbolRef<'s> {
    /// Attempt to parse `string` as an absolute symbol.
    ///
    /// # Examples
    ///
    /// ```
    /// use wolfram_expr::symbol::SymbolRef;
    ///
    /// assert!(matches!(SymbolRef::try_new("System`List"), Some(_)));
    /// assert!(matches!(SymbolRef::try_new("List"), None));
    /// assert!(matches!(SymbolRef::try_new("123"), None));
    /// ```
    pub fn try_new(string: &'s str) -> Option<Self> {
        crate::symbol::parse::SymbolRef_try_new(string)
    }

    /// Get the borrowed string data.
    pub fn as_str(&self) -> &'s str {
        let SymbolRef(string) = self;
        string
    }

    /// Convert this borrowed string into an owned [`Symbol`].
    pub fn to_symbol(&self) -> Symbol {
        let SymbolRef(string) = self;
        unsafe { Symbol::unchecked_new(string.to_owned()) }
    }

    // TODO: Document this method
    #[doc(hidden)]
    pub const unsafe fn unchecked_new(string: &'s str) -> Self {
        SymbolRef(string)
    }

    /// Get the context path part of a symbol as an [`ContextRef`].
    pub fn context(&self) -> ContextRef<'s> {
        let string = self.as_str();

        let last_grave = string
            .rfind('`')
            .expect("Failed to find grave '`' character in symbol");

        // SAFETY: All valid Symbol's will contain at least one grave mark '`', will
        //         have at least 1 character after that grave mark, and the string up
        //         to and including the last grave mark will be a valid absolute context.
        let (context, _) = string.split_at(last_grave + 1);

        unsafe { ContextRef::unchecked_new(context) }
    }

    /// Get the symbol name part of a symbol as a [`SymbolNameRef`].
    pub fn symbol_name(&self) -> SymbolNameRef<'s> {
        let string = self.as_str();

        let last_grave = string
            .rfind('`')
            .expect("Failed to find grave '`' character in symbol");

        // SAFETY: All valid Symbol's will contain at least one grave mark '`', will
        //         have at least 1 character after that grave mark, and the string up
        //         to and including the last grave mark will be a valid absolute context.
        let (_, name) = string.split_at(last_grave + 1);
        unsafe { SymbolNameRef::unchecked_new(name) }
    }
}

impl<'s> SymbolNameRef<'s> {
    /// Attempt to parse `string` as a symbol name.
    pub fn try_new(string: &'s str) -> Option<Self> {
        crate::symbol::parse::SymbolNameRef_try_new(string)
    }

    /// Get the borrowed string data.
    pub fn as_str(&self) -> &'s str {
        let SymbolNameRef(string) = self;
        string
    }

    /// Convert this borrowed string into an owned [`SymbolName`].
    pub fn to_symbol_name(&self) -> SymbolName {
        let SymbolNameRef(string) = self;
        unsafe { SymbolName::unchecked_new(string.to_owned()) }
    }

    #[doc(hidden)]
    pub unsafe fn unchecked_new(string: &'s str) -> Self {
        SymbolNameRef(string)
    }
}

impl<'s> ContextRef<'s> {
    /// Attempt to parse `string` as a context.
    pub fn try_new(string: &'s str) -> Option<Self> {
        crate::symbol::parse::ContextRef_try_new(string)
    }

    /// Get the borrowed string data.
    pub fn as_str(&self) -> &'s str {
        let ContextRef(string) = self;
        string
    }

    /// Convert this borrowed string into an owned [`Context`].
    pub fn to_context(&self) -> Context {
        let ContextRef(string) = self;
        unsafe { Context::unchecked_new(string.to_owned()) }
    }

    #[doc(hidden)]
    pub unsafe fn unchecked_new(string: &'s str) -> Self {
        ContextRef(string)
    }
}

//======================================
// Formatting impls
//======================================

impl Display for SymbolNameRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
