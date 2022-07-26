use crate::{Expr, ExprKind, Normal, Symbol};
use flate2::{write::ZlibEncoder, Compression};
use integer_encoding::VarInt;
use std::io::Write;

impl Expr {
    /// Export as wxf format.
    pub fn as_wxf(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(b"8:");
        self.write_internal(&mut out);
        out
    }
    /// Export as compressed wxf format.
    pub fn as_wxf_compressed(&self) -> Vec<u8> {
        let mut input = Vec::new();
        let mut e = ZlibEncoder::new(vec![], Compression::new(9));
        self.write_internal(&mut input);
        let mut out = Vec::with_capacity(input.len());
        if e.write_all(&input).is_ok() {
            out.extend_from_slice(b"8C:")
        };
        match e.finish() {
            Ok(o) => out.extend_from_slice(&o),
            Err(..) => {
                panic!("unknown error when compress");
            },
        };
        out
    }

    fn write_internal(&self, out: &mut Vec<u8>) {
        match self.kind() {
            ExprKind::Integer(n) => {
                out.push(b'L');
                out.extend_from_slice(&n.to_le_bytes());
            },
            ExprKind::Real(n) => {
                out.push(b'r');
                out.extend_from_slice(&n.to_le_bytes());
            },
            ExprKind::String(s) => {
                let len = s.len().encode_var_vec();
                out.push(b'S');
                out.extend_from_slice(&len);
                out.extend_from_slice(s.as_bytes());
            },
            ExprKind::Symbol(s) => s.write_internal(out),
            ExprKind::Normal(fx) => fx.write_internal(out),
        }
    }
}


impl Symbol {
    fn write_internal(&self, out: &mut Vec<u8>) {
        let s = self.as_str();
        let len = s.len().encode_var_vec();
        out.push(b's');
        out.extend_from_slice(&len);
        out.extend_from_slice(s.as_bytes());
    }
    #[allow(dead_code)]
    fn is_system_symbol(&self) -> bool {
        self.context().as_str().starts_with("System`")
    }
}

impl Normal {
    fn write_internal(&self, out: &mut Vec<u8>) {
        out.push(b'f');
        out.extend_from_slice(&self.contents.len().encode_var_vec());
        self.head.write_internal(out);
        for v in self.contents.iter() {
            v.write_internal(out)
        }
    }
}
