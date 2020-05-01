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
            pub static ref $name: $crate::Symbol = unsafe {
                // TODO: This should check the symbol on debug builds
                $crate::Symbol::unchecked_new($symbol_str)
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

// By using `usize` here, we guarantee that we can later change this to be a pointer
// instead without changing the sizes of a lot of Expr types. This is good for FFI/ABI
// compatibility if I decide to change the way Symbol works.
assert_eq_size!(Symbol, usize);

impl Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for SymbolName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&Symbol> for Symbol {
    fn from(sym: &Symbol) -> Self {
        sym.clone()
    }
}

impl Symbol {
    pub fn as_str(&self) -> &str {
        let Symbol(arc_string) = self;

        arc_string.as_str()
    }

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

    /// Create a symbol.
    ///
    /// Strongly prefer using `wl_parse::parse_symbol()` over this function, unless you
    /// are absolutely certain the string passed in will always be a valid absolute
    /// symbol.
    ///
    /// If a symbol needs to be created multiple times, consider using cache_symbol!()
    /// instead.
    ///
    /// NOTE: Adding an entry in the lang::sym::builtin_symbols! is almost always what's
    ///       needed instead of `Symbol::unchecked_new`.
    ///
    /// ## Safety
    ///
    /// This function actually does not do anything that would be rejected by rustc were
    /// the function not marked unsafe. However, this funciton is so often *not* what is
    /// really needed, it's marked unsafe as a deterent to possible users.
    ///
    /// NOTE: This function bypasses adding the new symbol to the evaluator's symbol table,
    ///       which is ALMOST ALWAYS not what is wanted (this means the new symbol could
    ///       not be found on the $ContextPath).
    ///
    /// NOTE: This function does NOT validate it's input. It's up to the caller to check
    ///       that the passed `str` matches the syntax of a symbol:
    ///           `` <context path>`<symbol_name> ``.
    ///
    /// The passed in string should be in the form of a context path followed by a symbol
    /// name.
    ///
    /// Example:
    ///
    /// ```norun
    /// Symbol::unchecked_new("Internal`x")
    /// Symbol::unchecked_new("A`B`mySymbol")
    /// ```
    ///
    // /// Takes a &'static str (as opposed to a &str) to help guarantee that no user input
    // /// is ever fed to this function. This function is only intended to be used as a
    // /// helper in the kernel.
    pub unsafe fn unchecked_new<S: Into<String> + AsRef<str>>(s: S) -> Symbol {
        let inner = Arc::new(s.into());
        // TODO: Add a debug_assert! here to validate `s`.
        Symbol(inner)
    }

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

impl SymbolName {
    pub fn as_str(&self) -> &str {
        let SymbolName(arc_name) = self;

        arc_name.as_str()
    }

    pub unsafe fn unchecked_new<S: Into<String> + AsRef<str>>(s: S) -> SymbolName {
        let inner = Arc::new(s.into());
        // TODO: Add a debug_assert! here to validate `s`.
        SymbolName(inner)
    }
}

// impl PartialEq<str> for Symbol {
//     fn eq(&self, other: &str) -> bool {
//         let mut lock = acquire_lock();
//         let other_sym: usize = lock.get_or_intern(other);
//         self.0 == other_sym
//     }
// }
