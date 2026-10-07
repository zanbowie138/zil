//! Raw bytes: hex dumps, entropy, checksums. Strings count as their UTF-8 bytes.

use super::{Call, Doc, Fail, Module, doc};
use crate::interp::Interp;
use crate::lexer::Span;
use crate::value::Value;

pub const MODULE: Module = Module {
    name: "binary",
    about: "hex dumps, entropy, checksums of strings and byte lists",
    #[rustfmt::skip]
    examples: &[
        ("binary", &[
            ("look inside a string", r#"print(hexdump("héllo\tworld\n"))"#),
            ("how random is it?", r#""aaaaaaaa".entropy"#),
            ("checksum", r#""hello".crc32 to hex"#),
            ("bytes vs characters", r#"["😀".byte_len, "😀".len]"#),
        ]),
    ],
    fns: FNS,
    call,
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("hexdump", "hexdump(v)", "xxd-style dump of a string's UTF-8 bytes or a list of bytes", &[r#"hexdump("hi\n")"#, "hexdump([0, 255, 65])"], &["bytes", "entropy"]),
    doc("entropy", "entropy(v)", "Shannon entropy in bits per byte: 0 is constant, 8 is random", &[r#""aaaa".entropy"#, r#""abcd".entropy"#], &["hexdump"]),
    doc("crc32", "crc32(v)", "CRC-32 checksum (the zip/PNG one)", &[r#""hello".crc32"#, r#""hello".crc32 to hex"#], &["sha256", "md5"]),
    doc("byte_len", "byte_len(s)", "length in UTF-8 bytes rather than characters", &[r#""héllo".byte_len"#], &["len", "bytes"]),
];

fn call(_: &mut Interp, name: &'static str, args: &[Value], _: &Span) -> Call {
    let [v] = args else { return Err(Fail::BadArgs) };
    let b = bytes(v).ok_or(Fail::BadArgs)?;
    Ok(match name {
        "hexdump" => Value::str(hexdump(&b)),
        "entropy" => {
            let mut counts = [0usize; 256];
            b.iter().for_each(|&x| counts[x as usize] += 1);
            let n = b.len() as f64;
            // `+ 0.0` turns the -0 from a single repeated byte into 0.
            Value::Float(counts.iter().filter(|&&c| c > 0).map(|&c| -(c as f64 / n) * (c as f64 / n).log2()).sum::<f64>() + 0.0)
        }
        "crc32" => Value::int(crc32(&b) as i64),
        "byte_len" => Value::int(b.len() as i64),
        _ => return Err(Fail::BadArgs),
    })
}

fn bytes(v: &Value) -> Option<Vec<u8>> {
    match v {
        Value::Str(s) => Some(s.as_bytes().to_vec()),
        Value::List(l) => l.borrow().iter().map(|x| if let Value::Int(n, _) = x { u8::try_from(*n).ok() } else { None }).collect(),
        _ => None,
    }
}

/// 16 bytes per line: offset, hex in pairs, printable ASCII.
fn hexdump(b: &[u8]) -> String {
    let mut out = Vec::new();
    for (i, row) in b.chunks(16).enumerate() {
        let hex: Vec<String> = row.chunks(2).map(|p| p.iter().map(|x| format!("{x:02x}")).collect()).collect();
        let ascii: String = row.iter().map(|&x| if x.is_ascii_graphic() || x == b' ' { x as char } else { '.' }).collect();
        out.push(format!("{:08x}: {:<39}  {ascii}", i * 16, hex.join(" ")));
    }
    out.join("\n")
}

fn crc32(b: &[u8]) -> u32 {
    let mut c = !0u32;
    for &x in b {
        c ^= x as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 { (c >> 1) ^ 0xEDB8_8320 } else { c >> 1 };
        }
    }
    !c
}

#[cfg(test)]
mod tests {
    use crate::interp::tests::{show, try_eval};

    #[test]
    fn binary() {
        assert_eq!(show(r#""hi\n".hexdump"#), format!("00000000: {:<39}  hi.", "6869 0a"));
        assert_eq!(show("hexdump([])"), "");
        assert_eq!(show(r#""aaaa".entropy"#), "0");
        assert_eq!(show(r#""abcd".entropy"#), "2");
        assert_eq!(show(r#""123456789".crc32 to hex"#), "0xcbf43926");
        assert_eq!(show(r#""héllo".byte_len"#), "6");
        assert!(try_eval("[256].crc32").is_err());
    }
}
