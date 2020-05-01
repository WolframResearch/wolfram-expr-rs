pub(crate) mod parse;

use std::fmt::{self, Debug, Display};

use std::sync::Arc;

use static_assertions::assert_eq_size;

/* Notes

Operations on Symbols

- Format (with conditional context path based on $Context)
- Test for equality
- Lookup symbol name in context path while parsing
- Remove / format Removed["..."]

*/

/// Create a Deref'erencable lazy_static! accessor for a symbol.
///
/// NOTE: This function will not check that the string given as the value of a symbol
///       is a syntactically correct absolute symbol. Care should be taken that this is
///       the case; a mistake could lead to a difficult bug.
///
/// ## Example:
///
/// ```no_test
/// cache_symbol!(
///     Internal_Value: "Internal`Value";
///     _counter: "MyContext`$counter");
///
/// let sym: Symbol = *Internal_Value;
/// let counter: Symbol = *_counter;
/// ```
#[macro_export]
macro_rules! cache_symbol {
    ($($name:ident: $symbol_str:expr);+ $(;)*) => {
        ::lazy_static::lazy_static! {
            $(
            #[allow(non_upper_case_globals)]
            pub static ref $name: $crate::Symbol = {
                // TODO: This should check the symbol on debug builds
                $crate::Symbol::new($symbol_str).expect("cache_symbol!: invalid symbol syntax")
            };
            )*
        }
    }
}

/// Represents a WL symbol.
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

/// A context path which does not begin with `.
/// E.g.: Global`A`
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AbsoluteContext(Arc<String>);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RelativeContext(Arc<String>);

// By using `usize` here, we guarantee that we can later change this to be a pointer
// instead without changing the sizes of a lot of Expr types. This is good for FFI/ABI
// compatibility if I decide to change the way Symbol works.
assert_eq_size!(Symbol, usize);

impl From<&Symbol> for Symbol {
    fn from(sym: &Symbol) -> Self {
        sym.clone()
    }
}

impl Symbol {
    /// Get the context path part of a symbol as a &str.
    pub fn context_path(&self) -> String {
        let mut s = self.to_string();
        let last_grave = s
            .rfind("`")
            .expect("Failed to find grave '`' character in symbol");
        // Slicing is [a..b), non inclusive of the 2nd index
        s.truncate(last_grave + 1);
        s
    }

    // Get the symbol name part of a symbol as a &str.
    pub fn symbol_name(&self) -> String {
        let mut s = self.to_string();
        let last_grave = s
            .rfind("`")
            .expect("Failed to find grave '`' character in symbol");
        // We assume the grave character is encoded as a single byte
        let substr = s.split_off(last_grave + 1);
        substr
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
            pub(crate) unsafe fn unchecked_new<S: AsRef<str> + Into<String>>(
                input: S,
            ) -> $ty {
                let inner: Arc<String> = Arc::new(input.into());
                $ty(inner)
            }
        }
    };
}

common_impls! { Symbol }
common_impls!(SymbolName);
common_impls!(AbsoluteContext);
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
