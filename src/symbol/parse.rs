use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{complete, recognize},
    multi::{many0, many1},
    sequence::{pair, tuple},
    IResult, InputLength,
};
use nom_locate::LocatedSpan;

use crate::symbol::{ContextRef, RelativeContext, SymbolNameRef, SymbolRef};

#[allow(non_snake_case)]
pub(super) fn SymbolRef_try_new<'s>(string: &'s str) -> Option<SymbolRef<'s>> {
    let input = LocatedSpan::new(string);

    let (rem, (_span, sym)) = absolute_symbol_ref_ty(input).ok()?;

    // Check that the input didn't contain any trailing characters after the symbol.
    if rem.input_len() == 0 {
        Some(sym)
    } else {
        None
    }
}

#[allow(non_snake_case)]
pub(super) fn SymbolNameRef_try_new<'s>(string: &'s str) -> Option<SymbolNameRef<'s>> {
    let input = LocatedSpan::new(string);

    let (rem, (_span, sym)) = symbol_name_ref_ty(input).ok()?;

    // Check that the input didn't contain any trailing characters after the symbol.
    if rem.input_len() == 0 {
        Some(sym)
    } else {
        None
    }
}

#[allow(non_snake_case)]
pub(super) fn ContextRef_try_new<'s>(string: &'s str) -> Option<ContextRef<'s>> {
    let input = LocatedSpan::new(string);

    let (remaining, _) = absolute_context_path(input).ok()?;

    // Check that the input didn't contain any trailing characters after the symbol.
    if remaining.input_len() == 0 {
        Some(ContextRef(input.fragment()))
    } else {
        None
    }
}

// TODO(cleanup): Add RelativeContextRef type, use here instead.
#[allow(non_snake_case)]
pub(super) fn RelativeContext_try_new(input: &str) -> Option<RelativeContext> {
    let input = LocatedSpan::new(input);

    let (remaining, _) = relative_context_path(input).ok()?;

    if remaining.input_len() == 0 {
        Some(unsafe { RelativeContext::unchecked_new(input.fragment().to_owned()) })
    } else {
        None
    }
}

//======================================
// Compound combinators -- these conceptually still only parse single tokens, and are used
// directly by wl-parse.
//======================================

type StrSpan<'a> = LocatedSpan<&'a str>;

#[cfg_attr(not(feature = "unstable_parse"), allow(dead_code))]
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

fn absolute_context_path(i: StrSpan) -> IResult<StrSpan, StrSpan> {
    recognize(many1(complete(pair(symbol_name, tag("`")))))(i)
}

fn relative_context_path(i: StrSpan) -> IResult<StrSpan, StrSpan> {
    // ` (<symbol_name>`)+
    recognize(pair(tag("`"), many1(complete(pair(symbol_name, tag("`"))))))(i)
}

fn absolute_symbol(i: StrSpan) -> IResult<StrSpan, StrSpan> {
    recognize(pair(absolute_context_path, symbol_name))(i)
}

fn relative_symbol(i: StrSpan) -> IResult<StrSpan, StrSpan> {
    recognize(pair(relative_context_path, symbol_name))(i)
}

//======================================
// Parsers which also return the safe wrappers.
// These are meant to be used outside of this crate.
//======================================

// These return a (StrSpan, <type>) so that the consumer can get line / extent information
// for the consumed input.

fn absolute_symbol_ref_ty(i: StrSpan) -> IResult<StrSpan, (StrSpan, SymbolRef)> {
    let (i, span) = absolute_symbol(i)?;
    let sym = SymbolRef(span.fragment());
    Ok((i, (span, sym)))
}

fn symbol_name_ref_ty(i: StrSpan) -> IResult<StrSpan, (StrSpan, SymbolNameRef)> {
    let (i, span) = symbol_name(i)?;
    let symname = SymbolNameRef(span.fragment());
    Ok((i, (span, symname)))
}
