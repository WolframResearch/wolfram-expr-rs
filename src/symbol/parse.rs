use crate::{
    symbol::{AbsoluteContext, RelativeContext, SymbolName},
    Symbol,
};

use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{complete, recognize},
    multi::{many0, many1},
    sequence::{pair, tuple},
    IResult, InputLength,
};
use nom_locate::LocatedSpan;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymbolRef<'s>(&'s str);
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymbolNameRef<'s>(&'s str);
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AbsoluteContextRef<'s>(&'s str);

impl<'s> SymbolRef<'s> {
    pub fn new(string: &'s str) -> Option<Self> {
        let input = LocatedSpan::new(string);

        let (rem, (_span, sym)) = absolute_symbol_ref_ty(input).ok()?;

        // Check that the input didn't contain any trailing characters after the symbol.
        if rem.input_len() == 0 {
            Some(sym)
        } else {
            None
        }
    }

    pub fn as_str(&self) -> &'s str {
        let SymbolRef(string) = self;
        string
    }

    pub fn to_symbol(&self) -> Symbol {
        let SymbolRef(string) = self;
        unsafe { Symbol::unchecked_new(string.to_owned()) }
    }

    pub unsafe fn unchecked_new(string: &'s str) -> Self {
        SymbolRef(string)
    }
}

impl<'s> SymbolNameRef<'s> {
    pub fn new(string: &'s str) -> Option<Self> {
        let input = LocatedSpan::new(string);

        let (rem, (_span, sym)) = symbol_name_ref_ty(input).ok()?;

        // Check that the input didn't contain any trailing characters after the symbol.
        if rem.input_len() == 0 {
            Some(sym)
        } else {
            None
        }
    }

    pub fn as_str(&self) -> &'s str {
        let SymbolNameRef(string) = self;
        string
    }

    pub fn to_symbol_name(&self) -> SymbolName {
        let SymbolNameRef(string) = self;
        unsafe { SymbolName::unchecked_new(string.to_owned()) }
    }

    pub unsafe fn unchecked_new(string: &'s str) -> Self {
        SymbolNameRef(string)
    }
}

impl<'s> AbsoluteContextRef<'s> {
    pub fn new(string: &'s str) -> Option<Self> {
        let input = LocatedSpan::new(string);

        let (remaining, _) = absolute_context_path(input).ok()?;

        // Check that the input didn't contain any trailing characters after the symbol.
        if remaining.input_len() == 0 {
            Some(AbsoluteContextRef(input.fragment()))
        } else {
            None
        }
    }

    pub fn as_str(&self) -> &'s str {
        let AbsoluteContextRef(string) = self;
        string
    }

    pub fn to_absolute_context(&self) -> AbsoluteContext {
        let AbsoluteContextRef(string) = self;
        unsafe { AbsoluteContext::unchecked_new(string.to_owned()) }
    }

    pub unsafe fn unchecked_new(string: &'s str) -> Self {
        AbsoluteContextRef(string)
    }
}

impl Symbol {
    /// Attempt to parse `input` as an absolute symbol.
    ///
    /// An absolute symbol is a symbol with an explicit context path. "System`Plus" is an
    /// absolute symbol, "Plus" is a relative symbol. "`Plus" is also a relative symbol.
    ///
    /// If a symbol needs to be created multiple times, consider using cache_symbol!()
    /// instead.
    pub fn new<I: AsRef<str>>(input: I) -> Option<Self> {
        SymbolRef::new(input.as_ref())
            .as_ref()
            .map(SymbolRef::to_symbol)
    }
}

impl SymbolName {
    /// Attempt to parse `input` as a symbol name.
    ///
    /// A symbol name is a symbol without any context marks.
    pub fn new<I: AsRef<str>>(input: I) -> Option<SymbolName> {
        SymbolNameRef::new(input.as_ref())
            .as_ref()
            .map(SymbolNameRef::to_symbol_name)
    }

    pub fn as_symbol_name_ref(&self) -> SymbolNameRef {
        SymbolNameRef(self.as_str())
    }
}

impl AbsoluteContext {
    pub fn new<I: AsRef<str>>(input: I) -> Option<Self> {
        AbsoluteContextRef::new(input.as_ref())
            .as_ref()
            .map(AbsoluteContextRef::to_absolute_context)
    }

    pub fn as_absolute_context_ref(&self) -> AbsoluteContextRef {
        AbsoluteContextRef(self.as_str())
    }
}

impl RelativeContext {
    pub fn new<I: AsRef<str>>(input: I) -> Option<Self> {
        let input = LocatedSpan::new(input.as_ref());

        let (remaining, _) = relative_context_path(input).ok()?;

        if remaining.input_len() == 0 {
            Some(unsafe { RelativeContext::unchecked_new(input.fragment().to_owned()) })
        } else {
            None
        }
    }
}

impl From<SymbolName> for AbsoluteContext {
    fn from(name: SymbolName) -> AbsoluteContext {
        AbsoluteContext::new(format!("{}`", name)).unwrap()
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
