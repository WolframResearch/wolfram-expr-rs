// TODO(!): Replace all of this symbol parsing logic with functionality from
//          wolfram-code-parse, once that is available.

use crate::symbol::{ContextRef, RelativeContext, SymbolNameRef, SymbolRef};

#[allow(non_snake_case)]
pub(super) fn SymbolRef_try_new<'s>(string: &'s str) -> Option<SymbolRef<'s>> {
    if parse_symbol_like(string)? == SymbolLike::AbsoluteSymbol {
        Some(SymbolRef(string))
    } else {
        None
    }
}

#[allow(non_snake_case)]
pub(super) fn SymbolNameRef_try_new<'s>(string: &'s str) -> Option<SymbolNameRef<'s>> {
    if parse_symbol_like(string)? == SymbolLike::SymbolName {
        Some(SymbolNameRef(string))
    } else {
        None
    }
}

#[allow(non_snake_case)]
pub(super) fn ContextRef_try_new<'s>(string: &'s str) -> Option<ContextRef<'s>> {
    if parse_symbol_like(string)? == SymbolLike::AbsoluteContext {
        Some(ContextRef(string))
    } else {
        None
    }
}

// TODO(cleanup): Add RelativeContextRef type, use here instead.
#[allow(non_snake_case)]
pub(super) fn RelativeContext_try_new(input: &str) -> Option<RelativeContext> {
    if parse_symbol_like(input)? == SymbolLike::RelativeContext {
        Some(unsafe { RelativeContext::unchecked_new(input.to_owned()) })
    } else {
        None
    }
}

#[derive(Debug, PartialEq)]
enum SymbolLike {
    /// `` ctx`foo ``
    AbsoluteSymbol,
    /// `foo`
    SymbolName,
    /// `` `foo ``
    RelativeSymbol,
    /// `` ctx` ``
    AbsoluteContext,
    /// `` `ctx` ``
    RelativeContext,
}

fn parse_symbol_like(input: &str) -> Option<SymbolLike> {
    if input.is_empty() {
        return None;
    }

    let components: Vec<&str> = input.split("`").collect();

    let like = match components.as_slice() {
        [only] if is_symbol_component(*only) => SymbolLike::SymbolName,
        // "`...`"
        ["", inner @ .., ""] if inner.iter().copied().all(is_symbol_component) => {
            SymbolLike::RelativeContext
        },
        // "`..."
        ["", rest @ ..] if rest.iter().copied().all(is_symbol_component) => {
            SymbolLike::RelativeSymbol
        },
        // "...`"
        [most @ .., ""] if most.iter().copied().all(is_symbol_component) => {
            SymbolLike::AbsoluteContext
        },

        components if components.iter().copied().all(is_symbol_component) => {
            SymbolLike::AbsoluteSymbol
        },

        _ => return None,
    };

    Some(like)
}


fn is_symbol_component(str: &str) -> bool {
    if str.is_empty() {
        return false;
    }

    debug_assert!(!str.contains('`'));

    let mut chars = str.chars();

    let first_char = chars.next().unwrap();

    if !first_char.is_alphabetic() && first_char != '$' {
        return false;
    }

    for char in chars {
        match char {
            '_' | '-' => return false,
            _ if char.is_alphabetic() => (),
            _ if char.is_digit(10) => (),
            '$' => (),
            _ => return false,
        }
    }

    true
}

#[test]
fn test_is_symbol_component() {
    assert!(is_symbol_component("foo"));
    assert!(is_symbol_component("$bar"));
}

#[test]
fn test_parse_symbol_like() {
    assert_eq!(parse_symbol_like("foo"), Some(SymbolLike::SymbolName));
}
