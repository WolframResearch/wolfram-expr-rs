use crate::symbol::{ContextRef, RelativeContext, SymbolNameRef, SymbolRef};

/// `(input, is Symbol, is SymbolName, is Context, is RelativeContext)`
#[rustfmt::skip]
const DATA: &[(&str, bool, bool, bool, bool)] = &[
    // Symbol-like
    ("foo`bar",     true , false, false, false),
    ("foo`bar`baz", true , false, false, false),
    ("foo`bar5",    true , false, false, false),
    ("foo`5bar",    false, false, false, false),
    ("5foo`bar",    false, false, false, false),
    ("foo``bar",    false, false, false, false),
    ("foo`$bar",    true , false, false, false),
    ("$foo`$bar",   true , false, false, false),
    ("$foo`$$$",    true , false, false, false),
    ("$$$`$$$",     true , false, false, false),

    // SymbolName-like
    ("foo",         false, true,  false, false),
    ("foo5",        false, true,  false, false),
    ("foo5bar",     false, true,  false, false),
    ("$foo",        false, true,  false, false),
    ("5foo",        false, false, false, false),
    ("foo_bar",     false, false, false, false),
    ("_foo",        false, false, false, false),

    // TODO: RelativeSymbol-like
    ("`foo",        false, false, false, false),
    ("`foo`bar",    false, false, false, false),

    // Context-like
    ("foo`",        false, false, true,  false),
    ("foo`bar`",    false, false, true,  false),

    // RelativeContext-like
    ("`foo`",       false, false, false, true),
    ("`foo`bar`",   false, false, false, true),
];

#[test]
pub fn test_symbol_like_parsing() {
    for (input, is_symbol, is_symbol_name, is_context, is_rel_context) in
        DATA.iter().copied()
    {
        println!("input: {input}");
        assert_eq!(SymbolRef::try_new(input).is_some(), is_symbol);
        assert_eq!(SymbolNameRef::try_new(input).is_some(), is_symbol_name);
        assert_eq!(ContextRef::try_new(input).is_some(), is_context);
        assert_eq!(RelativeContext::try_new(input).is_some(), is_rel_context);
    }
}
