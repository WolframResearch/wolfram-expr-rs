/*!
 * Contains the global symbol string interner. The intention is that Symbol can be treated
 * as if it was a string, without actually having every symbol be a String allocation.
 *
 * TODO: Possibly switch this module to using *const pointers to never-freed data, rather
 * than the identifying usize "tokens" which are used now. This would prevent ever having
 * to aquire a lock to Display symbols.
 */
use std::fmt;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

mod interner;

pub use self::interner::InternedString;

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
            pub static ref $name: ::wl_expr::Symbol = unsafe {
                // TODO: This should check the symbol on debug builds
                ::wl_expr::Symbol::unchecked_new($symbol_str)
            };
            )*
        }
    }
}

#[derive(Debug)]
pub struct SymbolTable {
    // TODO: Change these String's into AbsoluteContext?
    pub context: String,
    pub context_path: Vec<String>,

    /// Keep track of all symbols which have been explicitly `Removed[_Symbol]`.
    symbols: HashSet<Symbol>,
    /// Keep a record of all symbols which have a common symbol name (but different
    /// context paths). The only difference between symbols stored in the same HashSet is
    /// their context path.
    common_symbol_names: HashMap<String, HashSet<Symbol>>,
}

impl SymbolTable {
    // TODO: Add methods for manipulating `context` and `context_path` safely. I'd like
    //       to use AbsoluteContext here, but there's also a very strong argument that
    //       that stays in the parser, and it would be worse to make wl-expr depend on
    //       wl-parse than to do nothing.

    pub fn new<'a, S, I, C>(context: S, context_path: C) -> Self
            where S: Into<String>, I: AsRef<str>, C: IntoIterator<Item=I> {
        let context_path: Vec<String> = context_path.into_iter().map(|s| {
            s.as_ref().to_owned()
        }).collect();
        SymbolTable {
            context: context.into(),
            context_path,
            symbols: HashSet::new(),
            common_symbol_names: HashMap::new(),
        }
    }

    /// Returns true if `sym` was already a part of this symbol table.
    pub fn add_symbol(&mut self, sym: Symbol) -> bool {
        self.symbols.insert(sym.clone());

        let symbol_name = sym.symbol_name();
        self.common_symbol_names.entry(symbol_name).or_insert(HashSet::new()).insert(sym)
    }

    /// Used by Remove[_Symbol]
    pub fn remove_symbol(&mut self, sym: &Symbol) {
        self.symbols.remove(sym);
    }

    // Returns true if `sym` belongs in $Context or an element of $ContextPath.
    pub fn is_visible(&self, sym: &Symbol) -> bool {
        let context = sym.context_path();
        context == self.context || self.context_path.contains(&context)
    }

    /// This function assumes that `symbol` matches the syntax of symbol as defined
    /// in the parser. It will likely panic!() if malformed input is given.
    /// FIXME: It won't panic, it will just call Symbol::unchecked_new(), fix this.
    /// TODO: Change the type of `symbol` to enforce syntax
    pub fn parse_from_source(&mut self, symbol: &str) -> Symbol {
        let sym = if !symbol.contains("`") {
            self.parse_symbol_name(symbol)
        } else if symbol.starts_with("`") {
            // This is a relative symbol, e.g.: `y`x in the source.
            // So if $Context is "Internal`", the full symbol is Internal`y`x
            // `context` ($Context) should always end in a grave character, so strip
            // that out before concatenating with `symbol`. We assume the grave
            // character is encoded as a single byte
            let full_symbol = format!("{}{}", self.context, &symbol[1..]);
            unsafe {
                Symbol::unchecked_new(full_symbol)
            }
        } else {
            unsafe {
                // This must be an absolute symbol.
                Symbol::unchecked_new(symbol)
            }
        };

        self.add_symbol(sym.clone());
        sym
    }

    /// `symbol_name` should be a symbol that does NOT contains and "`" characters. It
    /// is the SymbolName part of a symbol (or it other words the part of the symbol that
    /// remains when you remove the context path).
    ///
    /// TODO(!): verify that `symbol_name` actually legal as a symbol name. This will have
    ///          to involve wl-parse somehow.
    fn parse_symbol_name(&mut self, symbol_name: &str) -> Symbol {
        // println!("parse_symbol_name: {} {:?} {}", self.context, self.context_path,
        //                                           symbol_name);
        // TODO: This brace should be fixed by NLL.
        {
            // let common_names = if self.common_symbol_names.contains_key(symbol_name) {
            //     self.common_symbol_names.get(symbol_name).unwrap()
            // } else {
            //     return self.unchecked_new_symbol(&format!("{}{}", context, symbol_name));
            // };

            let common_names = match self.common_symbol_names.get(symbol_name) {
                Some(common_names) => common_names,
                None => return unsafe {
                    Symbol::unchecked_new(format!("{}{}", self.context, symbol_name))
                },
            };

            for context_path_entry in &self.context_path {
                // println!("CPE: {}", context_path_entry);
                for common_name in common_names {
                    // println!("CN: {}", common_name);
                    // println!("  | {}", common_name.context_path());
                    if common_name.context_path() == *context_path_entry {
                        return common_name.clone();
                    }
                }
            }
        }
        // We didn't find a symbol in $ContextPath with the name `symbol_name`, so create
        // a symbol symbol in the current context: $Context`<symbol_name>
        let sym = unsafe {
            Symbol::unchecked_new(format!("{}{}", self.context, symbol_name))
        };
        self.add_symbol(sym.clone());
        sym
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
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct Symbol(Arc<String>);

// By using `usize` here, we gurantee that we can later change this to be a pointer
// instead without changing the sizes of a lot of Expr types. This is good for FFI/ABI
// compatibility if I decide to change the way Symbol works.
assert_eq_size!(symbol; Symbol, usize);

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Symbol(interned) = self;
        write!(f, "Symbol({})", interned)
    }
}

impl From<&Symbol> for Symbol {
    fn from(sym: &Symbol) -> Self {
        sym.clone()
    }
}

impl Symbol {
    /// Get the context path part of a symbol as a &str.
    pub fn context_path(&self) -> String {
        let mut s = self.to_string();
        let last_grave = s.rfind("`")
            .expect("Failed to find grave '`' character in symbol");
        // Slicing is [a..b), non inclusive of the 2nd index
        s.truncate(last_grave + 1);
        s
    }

    // Get the symbol name part of a symbol as a &str.
    pub fn symbol_name(&self) -> String {
        let mut s = self.to_string();
        let last_grave = s.rfind("`")
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
    // /// Takes a &'static str (as opposed to a &str) to help gurantee that no user input
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
    // /// Takes a &'static str (as opposed to a &str) to help gurantee that no user input
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

// impl PartialEq<str> for Symbol {
//     fn eq(&self, other: &str) -> bool {
//         let mut lock = acquire_lock();
//         let other_sym: usize = lock.get_or_intern(other);
//         self.0 == other_sym
//     }
// }