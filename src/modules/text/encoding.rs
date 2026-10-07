//! Encodings: base64/32/58, URL and hex, code points, UTF-8 bytes.

use crate::interp::Interp;
use crate::lexer::Span;
use crate::modules::{Call, Doc, Fail, Module, doc};
use crate::value::Value;
use base64::Engine;

pub const MODULE: Module = Module {
    name: "encoding",
    about: "base64, base32, base58, URL and hex encodings, code points, UTF-8 bytes",
    #[rustfmt::skip]
    examples: &[
        ("encoding", &[
            ("base64", r#""hi" to base64"#),
            ("URL encoding", r#""a b&c".encode("url")"#),
            ("base58, as in Bitcoin", r#""hello".encode("base58")"#),
            ("UTF-8 bytes", r#""héllo".bytes"#),
            ("code point to char", "chr(9731)"),
            ("bytes vs characters", r#"["😀".byte_len, "😀".len]"#),
        ]),
    ],
    guide: &[("conversions", &[("to base64", r#""hi" to base64"#)])],
    fns: FNS,
    call,
    targets: &[("base64", "base64")],
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("base64", "base64(s: str)", "base64-encode a string; same as `s to base64`", &[r#"base64("hi there")"#, r#""hi" to base64"#], &["encode", "decode"]),
    doc("encode", "encode(s: str, fmt: str)", "encode as \"base64\", \"base32\", \"base58\", \"url\" or \"hex\"", &[r#""hi there".encode("base64")"#, r#""a b&c".encode("url")"#, r#""hi".encode("base32")"#], &["decode"]),
    doc("decode", "decode(s: str, fmt: str)", "decode \"base64\", \"base32\", \"base58\", \"url\" or \"hex\"; bytes that aren't UTF-8 come back as a list", &[r#""aGk=".decode("base64")"#, r#""6869".decode("hex")"#, r#""Cn8eVZg".decode("base58")"#], &["encode"]),
    doc("ord", "ord(c: str)", "Unicode code point of a single character", &[r#""A".ord"#, r#""A".ord to hex"#], &["chr", "bytes"]),
    doc("chr", "chr(n: int)", "character for a Unicode code point", &["97.chr", "(65..70).map(chr).join"], &["ord"]),
    doc("bytes", "bytes(s: str)", "list of the string's UTF-8 bytes", &[r#""hé".bytes"#], &["from_bytes", "ord"]),
    doc("from_bytes", "from_bytes(xs: list)", "string from a list of UTF-8 bytes", &["[104, 105].from_bytes"], &["bytes", "chr"]),
    doc("byte_len", "byte_len(v: str|list)", "length in UTF-8 bytes rather than characters", &[r#""héllo".byte_len"#], &["len", "bytes"]),
];

fn call(_: &mut Interp, name: &'static str, args: &[Value], _: &Span) -> Call {
    use Value::*;
    Ok(match (name, args) {
        ("base64", [Str(s)]) => Value::str(base64::engine::general_purpose::STANDARD.encode(s.as_bytes())),
        ("encode", [Str(s), Str(fmt)]) => Value::str(match &**fmt {
            "base64" => base64::engine::general_purpose::STANDARD.encode(s.as_bytes()),
            "base32" => base32(s.as_bytes()),
            "base58" => base58(s.as_bytes()),
            "url" => url_encode(s),
            "hex" => hex(s.as_bytes()),
            _ => return Err(Fail::Arg(1, unknown_encoding(fmt))),
        }),
        ("decode", [Str(s), Str(fmt)]) => {
            let bytes = match &**fmt {
                "base64" => base64::engine::general_purpose::STANDARD.decode(s.trim()).map_err(|e| e.to_string())?,
                "base32" => unbase32(s.trim()).ok_or("invalid base32")?,
                "base58" => unbase58(s.trim()).ok_or("invalid base58")?,
                "url" => url_decode(s).ok_or("invalid percent-encoding")?,
                "hex" => unhex(s.trim()).ok_or("invalid hex")?,
                _ => return Err(Fail::Arg(1, unknown_encoding(fmt))),
            };
            match String::from_utf8(bytes) {
                Ok(s) => Value::str(s),
                Err(e) => Value::list(e.into_bytes().into_iter().map(|b| Value::int(b as i64)).collect()),
            }
        }
        ("ord", [Str(s)]) => match s.chars().collect::<Vec<_>>()[..] {
            [c] => Value::int(c as i64),
            ref cs => return Err(Fail::Arg(0, format!("expected a single character, got {}", cs.len()))),
        },
        ("chr", [Int(n, _)]) => {
            let c = u32::try_from(*n).ok().and_then(char::from_u32);
            Value::str(
                c.ok_or_else(|| Fail::Arg(0, format!("`{n}` is not a Unicode code point\nnote: code points go from 0 to 0x10ffff, skipping 0xd800-0xdfff")))?
                    .to_string(),
            )
        }
        ("bytes", [Str(s)]) => Value::list(s.bytes().map(|b| Value::int(b as i64)).collect()),
        ("from_bytes", [List(l)]) => {
            let byte = |v: &Value| match v {
                Int(n, _) => u8::try_from(*n).ok(),
                _ => None,
            };
            let bytes: Option<Vec<u8>> = l.borrow().iter().map(byte).collect();
            let bytes = bytes.ok_or("expected a list of integers 0-255")?;
            Value::str(String::from_utf8(bytes).map_err(|_| "bytes are not UTF-8")?)
        }
        ("byte_len", [v]) => Value::int(bytes(v).ok_or(Fail::BadArgs)?.len() as i64),
        _ => return Err(Fail::BadArgs),
    })
}

fn unknown_encoding(fmt: &str) -> String {
    let hint = crate::error::did_you_mean(fmt, ["base64", "base32", "base58", "url", "hex"]);
    if hint.is_empty() {
        format!("unknown encoding {fmt:?}\nnote: encodings are base64, base32, base58, url and hex")
    } else {
        format!("unknown encoding {fmt:?}{hint}")
    }
}

/// A string's UTF-8 bytes, or a list of ints 0-255.
pub fn bytes(v: &Value) -> Option<Vec<u8>> {
    match v {
        Value::Str(s) => Some(s.as_bytes().to_vec()),
        Value::List(l) => l.borrow().iter().map(|x| if let Value::Int(n, _) = x { u8::try_from(*n).ok() } else { None }).collect(),
        _ => None,
    }
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn unhex(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return None;
    }
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(s.get(i..i + 2)?, 16).ok()).collect()
}

const B32: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
const B58: &[u8; 58] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

/// RFC 4648 base32, padded with `=` to a multiple of 8.
fn base32(b: &[u8]) -> String {
    let (mut out, mut acc, mut bits) = (String::new(), 0u32, 0);
    for &x in b {
        (acc, bits) = ((acc << 8 | x as u32) & 0xfff, bits + 8);
        while bits >= 5 {
            bits -= 5;
            out.push(B32[(acc >> bits & 31) as usize] as char);
        }
    }
    if bits > 0 {
        out.push(B32[(acc << (5 - bits) & 31) as usize] as char);
    }
    while !out.len().is_multiple_of(8) {
        out.push('=');
    }
    out
}

/// Case-insensitive; padding is optional.
fn unbase32(s: &str) -> Option<Vec<u8>> {
    let (mut out, mut acc, mut bits) = (Vec::new(), 0u32, 0);
    for c in s.trim_end_matches('=').bytes() {
        let v = B32.iter().position(|&a| a == c.to_ascii_uppercase())? as u32;
        (acc, bits) = ((acc << 5 | v) & 0xfff, bits + 5);
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    Some(out)
}

/// Bitcoin's alphabet; each leading zero byte becomes a `1`.
fn base58(b: &[u8]) -> String {
    let zeros = b.iter().take_while(|&&x| x == 0).count();
    let rest = &b[zeros..];
    let digits = if rest.is_empty() { vec![] } else { num_bigint::BigUint::from_bytes_be(rest).to_radix_be(58) };
    "1".repeat(zeros) + &digits.iter().map(|&d| B58[d as usize] as char).collect::<String>()
}

fn unbase58(s: &str) -> Option<Vec<u8>> {
    let zeros = s.bytes().take_while(|&c| c == b'1').count();
    let digits: Vec<u8> = s[zeros..].bytes().map(|c| B58.iter().position(|&a| a == c).map(|d| d as u8)).collect::<Option<_>>()?;
    let rest = if digits.is_empty() { vec![] } else { num_bigint::BigUint::from_radix_be(&digits, 58)?.to_bytes_be() };
    Some([vec![0; zeros], rest].concat())
}

pub fn url_encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

pub fn url_decode(s: &str) -> Option<Vec<u8>> {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'%' => {
                out.push(u8::from_str_radix(s.get(i + 1..i + 3)?, 16).ok()?);
                i += 3;
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use crate::interp::tests::{show, try_eval};

    #[test]
    fn encodings() {
        assert_eq!(show(r#""hi there".encode("base64")"#), "aGkgdGhlcmU=");
        assert_eq!(show(r#""aGkgdGhlcmU=".decode("base64")"#), "hi there");
        assert_eq!(show(r#""a b&c".encode("url")"#), "a%20b%26c");
        assert_eq!(show(r#""a%20b%26c".decode("url")"#), "a b&c");
        assert_eq!(show(r#""hi".encode("hex").decode("hex")"#), "hi");
        assert_eq!(show(r#""A".ord"#), "65");
        assert_eq!(show("0x1f600.chr"), "😀");
        assert_eq!(show(r#""hé".bytes"#), "[104, 195, 169]");
        assert_eq!(show(r#""hé".bytes.from_bytes"#), "hé");
        assert_eq!(show(r#""héllo".byte_len"#), "6");
        assert!(try_eval(r#""ab".ord"#).is_err());
        assert_eq!(show(r#""foobar".encode("base32")"#), "MZXW6YTBOI======");
        assert_eq!(show(r#""fo".encode("base32")"#), "MZXQ====");
        assert_eq!(show(r#""mzxw6ytboi".decode("base32")"#), "foobar");
        assert_eq!(show(r#""hello world".encode("base58")"#), "StV1DL6CwTryKyV");
        assert_eq!(show(r#""StV1DL6CwTryKyV".decode("base58")"#), "hello world");
        assert_eq!(show(r#""11".decode("base58").bytes"#), "[0, 0]");
        assert_eq!(show(r#""/w==".decode("base64")"#), "[255]");
        assert!(try_eval(r#""0OIl".decode("base58")"#).is_err());
        assert!(try_eval("[256].from_bytes").is_err());
    }
}
