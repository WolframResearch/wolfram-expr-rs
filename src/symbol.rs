//! Representation of Wolfram Language symbols.

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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RelativeContext(Arc<String>);

pub use crate::symbol::parse::{ContextRef, SymbolNameRef, SymbolRef};

// By using `usize` here, we guarantee that we can later change this to be a pointer
// instead without changing the sizes of a lot of Expr types. This is good for FFI/ABI
// compatibility if I decide to change the way Symbol works.
const _: () = assert!(mem::size_of::<Symbol>() == mem::size_of::<usize>());
const _: () = assert!(mem::align_of::<Symbol>() == mem::align_of::<usize>());

impl From<&Symbol> for Symbol {
    fn from(sym: &Symbol) -> Self {
        sym.clone()
    }
}

impl Symbol {
    /// Get the context path part of a symbol as an [`ContextRef`].
    pub fn context(&self) -> ContextRef {
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

    // Get the symbol name part of a symbol as a [`SymbolNameRef`].
    pub fn symbol_name(&self) -> SymbolNameRef {
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

impl Context {
    pub fn global() -> Self {
        Context(Arc::new(String::from("Global`")))
    }

    pub fn system() -> Self {
        Context(Arc::new(String::from("System`")))
    }

    /// Construct a new [`Context`] by appending a new context component to this
    /// context.
    ///
    /// ```
    /// use wolfram_expr::symbol::{Context, SymbolName, SymbolNameRef};
    ///
    /// let context = Context::from_symbol_name(&SymbolName::new("MyContext").unwrap());
    /// let private = context.join(SymbolNameRef::new("Private").unwrap());
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
    /// let context = Context::new("MyPackage`Sub`Module`").unwrap();
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

    /// Create the context `` name` ``.
    pub fn from_symbol_name(name: &SymbolName) -> Self {
        Context::try_new(&format!("{}`", name)).unwrap()
    }
}

impl RelativeContext {
    /// Return the components of this [`RelativeContext`].
    ///
    /// ```
    /// use wolfram_expr::symbol::RelativeContext;
    ///
    /// let context = RelativeContext::new("`Sub`Module`").unwrap();
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
    ($ty:ident) => {
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

common_impls! { Symbol }
common_impls!(SymbolName);
common_impls!(Context);
common_impls!(RelativeContext);

/*
impl Symbol {
    // /// Create a symbol in the System` context.
    // ///
    // /// This function has the same problems with regards to the symbol table as
    // /// `Symbol::unchecked_new`.
    // ///
    // /// NOTE: This function does NOT validate it's input. It's up to the caller to check
    // ///       that the passed str matches the syntax of a symbol.
    // ///
    // /// Example: The symbol System`Integer is created with Symbol::system("Integer")
    // ///
    // /// Take care, a call like `Symbol::system("System`x")` will produce the symbol
    // /// "System`System`x".
    // ///
    // /// Takes a &'static str (as opposed to a &str) to help guarantee that no user input
    // /// is ever fed to this function. This function is only intended to be used as a
    // /// helper.
    // pub fn system(s: &'static str) -> Symbol {
    //     // println!("Symbol::system: {}", s);
    //     // TODO: Add a debug_assert! here to validate `s`.
    //     let s = format!("System`{}", s);
    //     unsafe {
    //         Symbol::unchecked_new(s)
    //     }
    // }

    // pub fn global(s: &'static str) -> Symbol {
    //     // println!("Symbol::global: {}", s);
    //     // TODO: Add a debug_assert! here to validate `s`.
    //     let s = format!("Global`{}", s);
    //     Symbol::unchecked_new(&s)
    // }
}
*/

// impl PartialEq<str> for Symbol {
//     fn eq(&self, other: &str) -> bool {
//         let mut lock = acquire_lock();
//         let other_sym: usize = lock.get_or_intern(other);
//         self.0 == other_sym
//     }
// }
