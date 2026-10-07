//! Encodings: base64, URL and hex, code points, UTF-8 bytes.

use crate::interp::Interp;
use crate::lexer::Span;
use crate::modules::{Call, Doc, Fail, Module, doc};
use crate::value::Value;
use base64::Engine;

pub const MODULE: Module = Module {
    name: "encoding",
    about: "base64, URL and hex encodings, code points, UTF-8 bytes",
    #[rustfmt::skip]
    examples: &[
        ("encoding", &[
            ("base64", r#""hi" to base64"#),
            ("URL encoding", r#""a b&c".encode("url")"#),
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
    doc("base64", "base64(s)", "base64-encode a string; same as `s to base64`", &[r#"base64("hi there")"#, r#""hi" to base64"#], &["encode", "decode"]),
    doc("encode", "encode(s, fmt)", "encode as \"base64\", \"url\" or \"hex\"", &[r#""hi there".encode("base64")"#, r#""a b&c".encode("url")"#], &["decode"]),
    doc("decode", "decode(s, fmt)", "decode \"base64\", \"url\" or \"hex\"", &[r#""aGk=".decode("base64")"#, r#""6869".decode("hex")"#], &["encode"]),
    doc("ord", "ord(c)", "Unicode code point of a single character", &[r#""A".ord"#, r#""A".ord to hex"#], &["chr", "bytes"]),
    doc("chr", "chr(n)", "character for a Unicode code point", &["97.chr", "(65..70).map(chr).join"], &["ord"]),
    doc("bytes", "bytes(s)", "list of the string's UTF-8 bytes", &[r#""hé".bytes"#], &["from_bytes", "ord"]),
    doc("from_bytes", "from_bytes(list)", "string from a list of UTF-8 bytes", &["[104, 105].from_bytes"], &["bytes", "chr"]),
    doc("byte_len", "byte_len(s)", "length in UTF-8 bytes rather than characters", &[r#""héllo".byte_len"#], &["len", "bytes"]),
];

fn call(_: &mut Interp, name: &'static str, args: &[Value], _: &Span) -> Call {
    use Value::*;
    Ok(match (name, args) {
        ("base64", [Str(s)]) => Value::str(base64::engine::general_purpose::STANDARD.encode(s.as_bytes())),
        ("encode", [Str(s), Str(fmt)]) => Value::str(match &**fmt {
            "base64" => base64::engine::general_purpose::STANDARD.encode(s.as_bytes()),
            "url" => url_encode(s),
            "hex" => hex(s.as_bytes()),
            _ => return Err(format!("unknown encoding {fmt:?} (base64, url, hex)").into()),
        }),
        ("decode", [Str(s), Str(fmt)]) => {
            let bytes = match &**fmt {
                "base64" => base64::engine::general_purpose::STANDARD.decode(s.trim()).map_err(|e| e.to_string())?,
                "url" => url_decode(s).ok_or("invalid percent-encoding")?,
                "hex" => unhex(s.trim()).ok_or("invalid hex")?,
                _ => return Err(format!("unknown encoding {fmt:?} (base64, url, hex)").into()),
            };
            Value::str(String::from_utf8(bytes).map_err(|_| "decoded bytes are not UTF-8")?)
        }
        ("ord", [Str(s)]) => match s.chars().collect::<Vec<_>>()[..] {
            [c] => Value::int(c as i64),
            _ => return Err("expected a single character".into()),
        },
        ("chr", [Int(n, _)]) => {
            let c = u32::try_from(*n).ok().and_then(char::from_u32);
            Value::str(c.ok_or_else(|| format!("{n} is not a valid code point"))?.to_string())
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

fn url_encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

fn url_decode(s: &str) -> Option<Vec<u8>> {
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
        assert!(try_eval("[256].from_bytes").is_err());
    }
}
