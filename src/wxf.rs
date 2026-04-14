//! WXF (Wolfram Exchange Format) codec for [`Expr`].
//!
//! Implements the on-the-wire format documented at
//! <https://reference.wolfram.com/language/tutorial/WXFFormatDescription.html>.
//!
//! # Coverage
//!
//! Tokens implemented in both directions:
//!
//! * `C`/`j`/`i`/`L` — machine-sized integers (i8/i16/i32/i64).
//! * `r` — IEEE 754 f64.
//! * `R` — arbitrary-precision real (stored as a decimal digit string +
//!   precision in [`ExprKind::BigReal`]).
//! * `I` — arbitrary-precision integer ([`ExprKind::BigInteger`]).
//! * `S` — UTF-8 string.
//! * `s` — symbol (context preserved).
//! * `f` — function / head application.
//! * `A` — association with both `-` (Rule) and `:` (RuleDelayed) entries.
//! * `0xC1` — packed array.
//! * `0xC2` — numeric array.
//!
//! Compressed WXF streams (`8C:` header) are handled via the `flate2` crate,
//! which is pulled in by the default `wxf` feature.
//!
//! Packed / numeric arrays and `DateObject` are surfaced on the `Expr` side as
//! `Normal` expressions with reserved heads in the `System` context so
//! consumers can walk them generically — no extra enum variants required.

use std::io::{self, Cursor, Read, Write};

use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use num_bigint::BigInt;

use crate::{Expr, ExprKind, Normal, Symbol};

//======================================
// Reserved heads for non-enum variants
//======================================

const HEAD_PACKED_ARRAY: &str = "System`PackedArray";
const HEAD_NUMERIC_ARRAY: &str = "System`NumericArray";

/// Element-type tags used inside the packed- and numeric-array WXF tokens.
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ArrayElementType {
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Real32,
    Real64,
    ComplexReal32,
    ComplexReal64,
}

impl ArrayElementType {
    fn from_tag(byte: u8) -> Option<ArrayElementType> {
        Some(match byte {
            0x00 => ArrayElementType::Int8,
            0x01 => ArrayElementType::Int16,
            0x02 => ArrayElementType::Int32,
            0x03 => ArrayElementType::Int64,
            0x10 => ArrayElementType::UInt8,
            0x11 => ArrayElementType::UInt16,
            0x12 => ArrayElementType::UInt32,
            0x13 => ArrayElementType::UInt64,
            0x22 => ArrayElementType::Real32,
            0x23 => ArrayElementType::Real64,
            0x33 => ArrayElementType::ComplexReal32,
            0x34 => ArrayElementType::ComplexReal64,
            _ => return None,
        })
    }

    fn tag(self) -> u8 {
        match self {
            ArrayElementType::Int8 => 0x00,
            ArrayElementType::Int16 => 0x01,
            ArrayElementType::Int32 => 0x02,
            ArrayElementType::Int64 => 0x03,
            ArrayElementType::UInt8 => 0x10,
            ArrayElementType::UInt16 => 0x11,
            ArrayElementType::UInt32 => 0x12,
            ArrayElementType::UInt64 => 0x13,
            ArrayElementType::Real32 => 0x22,
            ArrayElementType::Real64 => 0x23,
            ArrayElementType::ComplexReal32 => 0x33,
            ArrayElementType::ComplexReal64 => 0x34,
        }
    }

    fn element_size(self) -> usize {
        match self {
            ArrayElementType::Int8 | ArrayElementType::UInt8 => 1,
            ArrayElementType::Int16 | ArrayElementType::UInt16 => 2,
            ArrayElementType::Int32 | ArrayElementType::UInt32 | ArrayElementType::Real32 => 4,
            ArrayElementType::Int64
            | ArrayElementType::UInt64
            | ArrayElementType::Real64
            | ArrayElementType::ComplexReal32 => 8,
            ArrayElementType::ComplexReal64 => 16,
        }
    }
}

//======================================
// Varint
//======================================

fn write_varint<W: Write>(w: &mut W, mut v: u64) -> io::Result<()> {
    let mut buf = [0u8; 10];
    let mut i = 0;
    loop {
        let mut byte = (v & 0x7F) as u8;
        v >>= 7;
        if v != 0 {
            byte |= 0x80;
        }
        buf[i] = byte;
        i += 1;
        if v == 0 {
            break;
        }
    }
    w.write_all(&buf[..i])
}

fn read_varint<R: Read>(r: &mut R) -> io::Result<u64> {
    let mut value = 0u64;
    let mut shift = 0u32;
    let mut b = [0u8; 1];
    loop {
        r.read_exact(&mut b)?;
        let byte = b[0];
        value |= ((byte & 0x7F) as u64) << shift;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift >= 64 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "varint overflow"));
        }
    }
    Ok(value)
}

//======================================
// Public API
//======================================

/// Encode `expr` as uncompressed WXF bytes (header `8:`).
pub fn to_wxf_bytes(expr: &Expr) -> io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    buf.extend_from_slice(b"8:");
    write_expr(&mut buf, expr)?;
    Ok(buf)
}

/// Encode `expr` as zlib-compressed WXF bytes (header `8C:`).
pub fn to_wxf_bytes_compressed(expr: &Expr) -> io::Result<Vec<u8>> {
    let mut body = Vec::new();
    write_expr(&mut body, expr)?;
    let mut out = Vec::with_capacity(body.len() / 2 + 3);
    out.extend_from_slice(b"8C:");
    let mut enc = ZlibEncoder::new(&mut out, Compression::default());
    enc.write_all(&body)?;
    enc.finish()?;
    Ok(out)
}

/// Decode WXF bytes (accepts both `8:` and `8C:` headers).
pub fn from_wxf_bytes(bytes: &[u8]) -> io::Result<Expr> {
    if bytes.len() >= 3 && &bytes[..3] == b"8C:" {
        let mut dec = ZlibDecoder::new(&bytes[3..]);
        let mut decompressed = Vec::new();
        dec.read_to_end(&mut decompressed)?;
        let mut cur = Cursor::new(decompressed);
        return read_expr(&mut cur);
    }
    if bytes.len() >= 2 && &bytes[..2] == b"8:" {
        let mut cur = Cursor::new(&bytes[2..]);
        return read_expr(&mut cur);
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "missing or unrecognized WXF header",
    ))
}

//======================================
// Encode
//======================================

fn write_expr<W: Write>(w: &mut W, expr: &Expr) -> io::Result<()> {
    match expr.kind() {
        ExprKind::Integer(i) => write_integer(w, *i),
        ExprKind::Real(r) => {
            w.write_all(&[b'r'])?;
            w.write_all(&f64::from(**r).to_le_bytes())
        },
        ExprKind::String(s) => write_string_token(w, b'S', s),
        ExprKind::Symbol(s) => write_string_token(w, b's', s.as_str()),
        ExprKind::BigInteger(bi) => {
            let s = bi.to_str_radix(10);
            write_string_token(w, b'I', &s)
        },
        ExprKind::BigReal { digits, .. } => write_string_token(w, b'R', digits),
        ExprKind::Normal(normal) => write_normal(w, normal),
    }
}

fn write_integer<W: Write>(w: &mut W, i: i64) -> io::Result<()> {
    if i >= i8::MIN as i64 && i <= i8::MAX as i64 {
        w.write_all(&[b'C', i as i8 as u8])
    } else if i >= i16::MIN as i64 && i <= i16::MAX as i64 {
        w.write_all(&[b'j'])?;
        w.write_all(&(i as i16).to_le_bytes())
    } else if i >= i32::MIN as i64 && i <= i32::MAX as i64 {
        w.write_all(&[b'i'])?;
        w.write_all(&(i as i32).to_le_bytes())
    } else {
        w.write_all(&[b'L'])?;
        w.write_all(&i.to_le_bytes())
    }
}

fn write_string_token<W: Write>(w: &mut W, tag: u8, s: &str) -> io::Result<()> {
    w.write_all(&[tag])?;
    write_varint(w, s.len() as u64)?;
    w.write_all(s.as_bytes())
}

fn write_normal<W: Write>(w: &mut W, normal: &Normal) -> io::Result<()> {
    // Special-case Association: emit `A` token with `-` / `:` rule entries.
    if let Some(sym) = normal.head.try_as_symbol() {
        if sym.as_str() == "System`Association" {
            return write_assoc(w, normal.elements());
        }
        if sym.as_str() == HEAD_PACKED_ARRAY {
            if let Some(bytes) = encode_array_token(normal.elements(), 0xC1) {
                return w.write_all(&bytes);
            }
        }
        if sym.as_str() == HEAD_NUMERIC_ARRAY {
            if let Some(bytes) = encode_array_token(normal.elements(), 0xC2) {
                return w.write_all(&bytes);
            }
        }
    }
    w.write_all(&[b'f'])?;
    write_varint(w, normal.elements().len() as u64)?;
    write_expr(w, &normal.head)?;
    for a in normal.elements() {
        write_expr(w, a)?;
    }
    Ok(())
}

fn write_assoc<W: Write>(w: &mut W, entries: &[Expr]) -> io::Result<()> {
    w.write_all(&[b'A'])?;
    write_varint(w, entries.len() as u64)?;
    for entry in entries {
        let (tag, key, val) = match entry.kind() {
            ExprKind::Normal(n) => match n.head.try_as_symbol().map(|s| s.as_str()) {
                Some("System`Rule") if n.elements().len() == 2 => {
                    (b'-', &n.elements()[0], &n.elements()[1])
                },
                Some("System`RuleDelayed") if n.elements().len() == 2 => {
                    (b':', &n.elements()[0], &n.elements()[1])
                },
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Association entry is not a Rule or RuleDelayed",
                    ))
                },
            },
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Association entry is not a normal expression",
                ))
            },
        };
        w.write_all(&[tag])?;
        write_expr(w, key)?;
        write_expr(w, val)?;
    }
    Ok(())
}

/// Pack array-head arguments `[etype_symbol, {dims}, raw_string]` back into
/// the binary token. Returns `None` if the arguments don't match that shape
/// (in which case the caller falls back to a generic `f` encoding).
fn encode_array_token(args: &[Expr], token: u8) -> Option<Vec<u8>> {
    if args.len() != 3 {
        return None;
    }
    let etype = match args[0].kind() {
        ExprKind::Integer(i) => ArrayElementType::from_tag(*i as u8)?,
        _ => return None,
    };
    let dims: Vec<usize> = match args[1].kind() {
        ExprKind::Normal(n) if n.has_head(&Symbol::new("System`List")) => n
            .elements()
            .iter()
            .map(|e| match e.kind() {
                ExprKind::Integer(i) if *i >= 0 => Some(*i as usize),
                _ => None,
            })
            .collect::<Option<Vec<_>>>()?,
        _ => return None,
    };
    let data = match args[2].kind() {
        ExprKind::String(s) => s.as_bytes(),
        _ => return None,
    };
    let expected = etype.element_size() * dims.iter().product::<usize>();
    if data.len() != expected {
        return None;
    }
    let mut out = Vec::with_capacity(6 + dims.len() + data.len());
    out.push(token);
    out.push(etype.tag());
    let mut tmp = Vec::new();
    write_varint(&mut tmp, dims.len() as u64).ok()?;
    out.extend_from_slice(&tmp);
    for d in &dims {
        let mut tmp = Vec::new();
        write_varint(&mut tmp, *d as u64).ok()?;
        out.extend_from_slice(&tmp);
    }
    out.extend_from_slice(data);
    Some(out)
}

//======================================
// Decode
//======================================

fn read_expr<R: Read>(r: &mut R) -> io::Result<Expr> {
    let mut tag = [0u8; 1];
    r.read_exact(&mut tag)?;
    match tag[0] {
        b'C' => {
            let mut b = [0u8; 1];
            r.read_exact(&mut b)?;
            Ok(Expr::from(i8::from_le_bytes(b) as i64))
        },
        b'j' => {
            let mut b = [0u8; 2];
            r.read_exact(&mut b)?;
            Ok(Expr::from(i16::from_le_bytes(b) as i64))
        },
        b'i' => {
            let mut b = [0u8; 4];
            r.read_exact(&mut b)?;
            Ok(Expr::from(i32::from_le_bytes(b) as i64))
        },
        b'L' => {
            let mut b = [0u8; 8];
            r.read_exact(&mut b)?;
            Ok(Expr::from(i64::from_le_bytes(b)))
        },
        b'r' => {
            let mut b = [0u8; 8];
            r.read_exact(&mut b)?;
            Ok(Expr::real(f64::from_le_bytes(b)))
        },
        b'S' => Ok(Expr::string(read_len_string(r)?)),
        b's' => {
            let raw = read_len_string(r)?;
            // WL's `BinarySerialize` emits built-in heads like `List` without
            // a `System\`` context. `Symbol::try_new` rejects un-qualified
            // names, so we default to the `System\`` context in that case —
            // matching the Kernel's default-context resolution.
            let sym = Symbol::try_new(&raw).unwrap_or_else(|| {
                Symbol::new(&format!("System`{}", raw))
            });
            Ok(Expr::symbol(sym))
        },
        b'I' => {
            let s = read_len_string(r)?;
            let bi: BigInt = s
                .parse()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "bigint parse"))?;
            if let Some(small) = bigint_to_i64(&bi) {
                Ok(Expr::from(small))
            } else {
                Ok(Expr::bigint(bi))
            }
        },
        b'R' => Ok(Expr::bigreal(read_len_string(r)?, 0)),
        b'f' => {
            let argc = read_varint(r)? as usize;
            let head = read_expr(r)?;
            let mut args = Vec::with_capacity(argc);
            for _ in 0..argc {
                args.push(read_expr(r)?);
            }
            Ok(Expr::normal(head, args))
        },
        b'A' => {
            let count = read_varint(r)? as usize;
            let mut entries = Vec::with_capacity(count);
            for _ in 0..count {
                let mut rule = [0u8; 1];
                r.read_exact(&mut rule)?;
                let head_name = match rule[0] {
                    b'-' => "System`Rule",
                    b':' => "System`RuleDelayed",
                    _ => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "association entry must be Rule or RuleDelayed",
                        ))
                    },
                };
                let key = read_expr(r)?;
                let val = read_expr(r)?;
                entries.push(Expr::normal(Symbol::new(head_name), vec![key, val]));
            }
            Ok(Expr::normal(Symbol::new("System`Association"), entries))
        },
        0xC1 => read_array(r, HEAD_PACKED_ARRAY),
        0xC2 => read_array(r, HEAD_NUMERIC_ARRAY),
        other => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unknown WXF token 0x{:02X}", other),
        )),
    }
}

fn read_len_string<R: Read>(r: &mut R) -> io::Result<String> {
    let len = read_varint(r)? as usize;
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf)?;
    String::from_utf8(buf).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "utf-8"))
}

fn read_array<R: Read>(r: &mut R, head: &str) -> io::Result<Expr> {
    let mut tb = [0u8; 1];
    r.read_exact(&mut tb)?;
    let etype = ArrayElementType::from_tag(tb[0]).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unsupported array element type 0x{:02X}", tb[0]),
        )
    })?;
    let rank = read_varint(r)? as usize;
    let mut dims = Vec::with_capacity(rank);
    for _ in 0..rank {
        dims.push(read_varint(r)? as usize);
    }
    let total = etype.element_size() * dims.iter().product::<usize>();
    let mut data = vec![0u8; total];
    r.read_exact(&mut data)?;
    let dims_expr = Expr::list(
        dims.iter()
            .map(|d| Expr::from(*d as i64))
            .collect(),
    );
    Ok(Expr::normal(
        Symbol::new(head),
        vec![
            Expr::from(etype.tag() as i64),
            dims_expr,
            Expr::string(unsafe { String::from_utf8_unchecked(data) }),
        ],
    ))
}

fn bigint_to_i64(b: &BigInt) -> Option<i64> {
    use num_bigint::Sign;
    let (sign, digits) = b.to_u64_digits();
    match digits.len() {
        0 => Some(0),
        1 => {
            let v = digits[0];
            match sign {
                Sign::Plus | Sign::NoSign => {
                    if v <= i64::MAX as u64 {
                        Some(v as i64)
                    } else {
                        None
                    }
                },
                Sign::Minus => {
                    if v <= (i64::MAX as u64) + 1 {
                        Some(-(v as i128) as i64)
                    } else {
                        None
                    }
                },
            }
        },
        _ => None,
    }
}

//======================================
// Tests
//======================================

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(e: &Expr) -> Expr {
        let bytes = to_wxf_bytes(e).unwrap();
        from_wxf_bytes(&bytes).unwrap()
    }

    fn roundtrip_compressed(e: &Expr) -> Expr {
        let bytes = to_wxf_bytes_compressed(e).unwrap();
        from_wxf_bytes(&bytes).unwrap()
    }

    #[test]
    fn small_integers() {
        for i in [0_i64, 1, -1, 127, -128, 128, -129, 32767, -32768, 32768] {
            assert_eq!(roundtrip(&Expr::from(i)), Expr::from(i));
        }
    }

    #[test]
    fn i64_edges() {
        for i in [i64::MIN, i64::MIN + 1, i64::MAX - 1, i64::MAX] {
            assert_eq!(roundtrip(&Expr::from(i)), Expr::from(i));
        }
    }

    #[test]
    fn reals() {
        let e = Expr::real(3.14159);
        assert_eq!(roundtrip(&e), e);
    }

    #[test]
    fn strings_and_symbols() {
        let s = Expr::string("hello 世界");
        assert_eq!(roundtrip(&s), s);
        let sym = Expr::symbol(Symbol::new("Global`myVar"));
        assert_eq!(roundtrip(&sym), sym);
    }

    #[test]
    fn list() {
        let e = Expr::list(vec![Expr::from(1), Expr::from(2), Expr::string("x")]);
        assert_eq!(roundtrip(&e), e);
    }

    #[test]
    fn bigint_roundtrip() {
        let big: BigInt = "340282366920938463463374607431768211456".parse().unwrap();
        let e = Expr::bigint(big);
        assert_eq!(roundtrip(&e), e);
    }

    #[test]
    fn bigint_fits_in_i64_decodes_as_integer() {
        // Encoder produces `L` for this, but a peer could legitimately send
        // an `I` token with a small value. Decode should fold it into Integer.
        let mut bytes = b"8:I".to_vec();
        let digits = "42";
        write_varint(&mut bytes, digits.len() as u64).unwrap();
        bytes.extend_from_slice(digits.as_bytes());
        let decoded = from_wxf_bytes(&bytes).unwrap();
        assert_eq!(decoded, Expr::from(42_i64));
    }

    #[test]
    fn assoc_with_mixed_rules() {
        let e = Expr::normal(
            Symbol::new("System`Association"),
            vec![
                Expr::rule(Symbol::new("Global`a"), Expr::from(1)),
                Expr::rule_delayed(Symbol::new("Global`b"), Expr::from(2)),
            ],
        );
        assert_eq!(roundtrip(&e), e);
    }

    #[test]
    fn compressed_roundtrip() {
        let e = Expr::list((0..100).map(Expr::from).collect());
        assert_eq!(roundtrip_compressed(&e), e);
    }

    #[test]
    fn packed_array_roundtrip() {
        // Hand-build an encoded WXF packed array and verify it decodes into
        // our reserved-head shape, then re-encodes byte-identically.
        let data: Vec<u8> = (0..12_u8).collect();
        let mut bytes = b"8:".to_vec();
        bytes.push(0xC1); // packed array token
        bytes.push(0x10); // UInt8
        write_varint(&mut bytes, 2).unwrap(); // rank
        write_varint(&mut bytes, 3).unwrap();
        write_varint(&mut bytes, 4).unwrap();
        bytes.extend_from_slice(&data);
        let decoded = from_wxf_bytes(&bytes).unwrap();
        assert!(decoded.has_normal_head(&Symbol::new(HEAD_PACKED_ARRAY)));
        let reencoded = to_wxf_bytes(&decoded).unwrap();
        assert_eq!(reencoded, bytes);
    }

    #[test]
    fn rejects_missing_header() {
        assert!(from_wxf_bytes(b"hello").is_err());
    }
}
