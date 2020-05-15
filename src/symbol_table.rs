use std::collections::{HashMap, HashSet};

use crate::{AbsoluteContext, Symbol, SymbolName};

#[derive(Debug)]
pub struct SymbolTable {
    pub context: AbsoluteContext,
    pub context_path: Vec<AbsoluteContext>,

    /// Keep track of all symbols which have been explicitly `Removed[_Symbol]`.
    symbols: HashSet<Symbol>,
    /// Keep a record of all symbols which have a common symbol name (but different
    /// context paths). The only difference between symbols stored in the same HashSet is
    /// their context path.
    common_symbol_names: HashMap<SymbolName, HashSet<Symbol>>,
}

impl SymbolTable {
    /// Construct a new symbol table from a context and context path.
    pub fn new<S, I, C>(context: AbsoluteContext, context_path: C) -> Self
    where
        S: Into<String>,
        I: AsRef<str>,
        C: IntoIterator<Item = AbsoluteContext>,
    {
        let context_path: Vec<AbsoluteContext> = context_path.into_iter().collect();
        SymbolTable {
            context: context.into(),
            context_path,
            symbols: HashSet::new(),
            common_symbol_names: HashMap::new(),
        }
    }

    /// Add `sym` to this symbol table.
    ///
    /// Returns `true` if `sym` was already a part of this symbol table.
    pub fn add_symbol(&mut self, sym: Symbol) -> bool {
        self.symbols.insert(sym.clone());

        let symbol_name = sym.symbol_name();
        self.common_symbol_names
            .entry(symbol_name.to_symbol_name())
            .or_insert_with(HashSet::new)
            .insert(sym)
    }

    /// Used by `Remove[_Symbol]`.
    pub fn remove_symbol(&mut self, sym: &Symbol) {
        self.symbols.remove(sym);
    }

    /// Returns true if `sym` resides in $Context or an element of $ContextPath.
    pub fn is_visible(&self, sym: &Symbol) -> bool {
        sym.context().as_str() == self.context.as_str()
            || self
                .context_path
                .iter()
                .any(|context| sym.context().as_str() == context.as_str())
    }

    /// This function assumes that `symbol` matches the syntax of symbol as defined
    /// in the parser. It will likely panic!() if malformed input is given.
    /// FIXME: It won't panic, it will just call Symbol::unchecked_new(), fix this.
    /// TODO: Change the type of `symbol` to enforce syntax
    pub fn parse_from_source(&mut self, symbol: &str) -> Symbol {
        let sym = if let Some(name) = SymbolName::new(symbol) {
            self.parse_symbol_name(&name)
        } else if symbol.starts_with('`') {
            // This is a relative symbol, e.g.: `y`x in the source.
            // So if $Context is "Internal`", the full symbol is Internal`y`x
            // `context` ($Context) should always end in a grave character, so strip
            // that out before concatenating with `symbol`. We assume the grave
            // character is encoded as a single byte
            let full_symbol = format!("{}{}", self.context, &symbol[1..]);
            unsafe { Symbol::unchecked_new(full_symbol) }
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
    /// is the [`SymbolName`] part of a symbol (or it other words the part of the symbol
    /// that remains when you remove the context path).
    fn parse_symbol_name(&mut self, symbol_name: &SymbolName) -> Symbol {
        // println!("parse_symbol_name: {} {:?} {}", self.context, self.context_path,
        //                                           symbol_name);

        // let common_names = if self.common_symbol_names.contains_key(symbol_name) {
        //     self.common_symbol_names.get(symbol_name).unwrap()
        // } else {
        //     return self.unchecked_new_symbol(&format!("{}{}", context, symbol_name));
        // };

        let common_names = match self.common_symbol_names.get(symbol_name) {
            Some(common_names) => common_names,
            None => {
                return unsafe {
                    Symbol::unchecked_new(format!("{}{}", self.context, symbol_name))
                }
            },
        };

        for context_path_entry in &self.context_path {
            // println!("CPE: {}", context_path_entry);
            for common_name in common_names {
                // println!("CN: {}", common_name);
                // println!("  | {}", common_name.context_path());
                if common_name.context().as_str() == context_path_entry.as_str() {
                    return common_name.clone();
                }
            }
        }

        // We didn't find a symbol in $ContextPath with the name `symbol_name`, so create
        // a symbol symbol in the current context: $Context`<symbol_name>
        let sym =
            unsafe { Symbol::unchecked_new(format!("{}{}", self.context, symbol_name)) };
        self.add_symbol(sym.clone());
        sym
    }
}
