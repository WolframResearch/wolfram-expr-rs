use crate::{Symbol, SymbolName};

use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{complete, recognize},
    multi::{many0, many1},
    sequence::{pair, tuple},
    IResult,
};
use nom_locate::LocatedSpan;

impl Symbol {
    /// Attempt to parse `input` as an absolute symbol.
    ///
    /// An absolute symbol is a symbol with an explicit context path. "System`Plus" is an
    /// absolute symbol, "Plus" is a relative symbol. "`Plus" is also a relative symbol.
    pub fn new<I: AsRef<str>>(input: I) -> Option<Self> {
        use nom::InputLength;

        let input = LocatedSpan::new(input.as_ref());

        let (rem, (_span, sym)) = absolute_symbol_ty(input).ok()?;

        // Check that the input didn't contain any trailing characters after the symbol.
        if rem.input_len() == 0 {
            Some(sym)
        } else {
            None
        }
    }
}

impl SymbolName {
    /// Attempt to parse `input` as a symbol name.
    ///
    /// A symbol name is a symbol without any context marks.
    pub fn new<I: AsRef<str>>(input: I) -> Option<SymbolName> {
        use nom::InputLength;

        let input = LocatedSpan::new(input.as_ref());

        let (remaining, (_span, symname)) = symbol_name_ty(input).ok()?;

        // Check that the input didn't contain any trailing characters after the symbol.
        if remaining.input_len() == 0 {
            Some(symname)
        } else {
            None
        }
    }
}


pub(super) type StrSpan<'a> = LocatedSpan<&'a str>;

pub fn symbol(i: StrSpan) -> IResult<StrSpan, StrSpan> {
    alt((absolute_symbol, relative_symbol, symbol_name))(i)
}

pub fn symbol_name(i: StrSpan) -> IResult<StrSpan, StrSpan> {
    use nom::character::complete::{alpha1, digit1};

    let (i, res) = recognize(tuple((
        many1(alt((alpha1, tag("$")))),
        many0(alt((digit1, alpha1, tag("$")))),
    )))(i)?;
    Ok((i, res))
}

pub fn absolute_context_path(i: StrSpan) -> IResult<StrSpan, StrSpan> {
    recognize(many1(complete(pair(symbol_name, tag("`")))))(i)
}

pub fn relative_context_path(i: StrSpan) -> IResult<StrSpan, StrSpan> {
    // ` (<symbol_name>`)+
    recognize(pair(tag("`"), many1(complete(pair(symbol_name, tag("`"))))))(i)
}

pub fn absolute_symbol(i: StrSpan) -> IResult<StrSpan, StrSpan> {
    recognize(pair(absolute_context_path, symbol_name))(i)
}

pub fn relative_symbol(i: StrSpan) -> IResult<StrSpan, StrSpan> {
    recognize(pair(relative_context_path, symbol_name))(i)
}

//======================================
// Parsers which also return the safe wrappers.
// These are meant to be used outside of this crate.
//======================================

// These return a (StrSpan, <type>) so that the consumer can get line / extent information
// for the consumed input.

fn absolute_symbol_ty(i: StrSpan) -> IResult<StrSpan, (StrSpan, Symbol)> {
    let (i, span) = absolute_symbol(i)?;
    let sym = unsafe { Symbol::unchecked_new(span.fragment) };
    Ok((i, (span, sym)))
}

fn symbol_name_ty(i: StrSpan) -> IResult<StrSpan, (StrSpan, SymbolName)> {
    let (i, span) = symbol_name(i)?;
    let symname = unsafe { SymbolName::unchecked_new(span.fragment) };
    Ok((i, (span, symname)))
}