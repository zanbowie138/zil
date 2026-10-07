//! Hashes and checksums of strings and byte lists.

use super::encoding::{bytes, hex};
use crate::interp::Interp;
use crate::lexer::Span;
use crate::modules::{Call, Doc, Fail, Module, doc};
use crate::value::Value;
use sha2::Digest;

pub const MODULE: Module = Module {
    name: "hash",
    about: "SHA-256, MD5 and CRC-32 of strings",
    #[rustfmt::skip]
    examples: &[
        ("hash", &[
            ("hashes", r#""hello".md5"#),
            ("checksum", r#""hello".crc32 to hex"#),
        ]),
    ],
    fns: FNS,
    call,
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("sha256", "sha256(s)", "hex SHA-256 hash", &[r#""hello".sha256[..16]"#], &["md5"]),
    doc("md5", "md5(s)", "hex MD5 hash", &[r#""hello".md5"#], &["sha256"]),
    doc("crc32", "crc32(v)", "CRC-32 checksum (the zip/PNG one)", &[r#""hello".crc32"#, r#""hello".crc32 to hex"#], &["sha256", "md5"]),
];

fn call(_: &mut Interp, name: &'static str, args: &[Value], _: &Span) -> Call {
    Ok(match (name, args) {
        ("sha256", [Value::Str(s)]) => Value::str(hex(&sha2::Sha256::digest(s.as_bytes()))),
        ("md5", [Value::Str(s)]) => Value::str(hex(&md5::Md5::digest(s.as_bytes()))),
        ("crc32", [v]) => Value::int(crc32(&bytes(v).ok_or(Fail::BadArgs)?) as i64),
        _ => return Err(Fail::BadArgs),
    })
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
    fn hashes() {
        assert_eq!(show(r#""abc".sha256"#), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        assert_eq!(show(r#""abc".md5"#), "900150983cd24fb0d6963f7d28e17f72");
        assert_eq!(show(r#""123456789".crc32 to hex"#), "0xcbf43926");
        assert!(try_eval("[256].crc32").is_err());
    }
}
