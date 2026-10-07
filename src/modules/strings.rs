//! Text: case, splitting, search and regex, encodings, hashes, code points.

use super::{Call, Claim, Doc, Fail, Module, doc};
use crate::ast::BinOp;
use crate::interp::Interp;
use crate::lexer::Span;
use crate::value::Value;
use base64::Engine;
use sha2::Digest;

pub const MODULE: Module = Module {
    name: "strings",
    about: "case, splitting, search and regex, encodings, hashes",
    #[rustfmt::skip]
    examples: &[
        ("strings", &[
            ("numbers out of text", r#""a1b22c333".nums.sum"#),
            ("regex captures", r#""2026-10-06".match(r"(\d+)-(\d+)-(\d+)")"#),
            ("regex replace with $1", r#""CamelCaseName".replace(r"(\B[A-Z])", " $1")"#),
            ("grep lines", r#""ok\nERROR disk full\nok".grep(r"ERROR")"#),
            ("initials", r#""Ada Lovelace".split.map(|w| w[0]).join"#),
            ("hashes", r#""hello".md5"#),
            ("base64", r#""hi" to base64"#),
            ("URL encoding", r#""a b&c".encode("url")"#),
            ("UTF-8 bytes", r#""héllo".bytes"#),
            ("code point to char", "chr(9731)"),
            ("printf", r#"format("%-6s|%5.2f", "pi", pi)"#),
        ]),
    ],
    #[rustfmt::skip]
    guide: &[
        ("types", &[("str", r#""héllo\tworld""#), ("regex", r#"r"\d+""#)]),
        ("operators", &[("str + v", r#""v" + 2"#), ("str * int", r#""ab" * 3"#)]),
        ("conversions", &[("to base64", r#""hi" to base64"#)]),
    ],
    fns: FNS,
    #[rustfmt::skip]
    groups: &[
        ("shape", &["upper", "lower", "capitalize", "trim", "reverse", "repeat"]),
        ("split", &["split", "lines", "chars", "join"]),
        ("search", &["contains", "starts_with", "ends_with", "find", "count", "match", "find_all", "grep", "replace", "nums"]),
        ("encode", &["base64", "encode", "decode", "sha256", "md5"]),
        ("chars", &["ord", "chr", "bytes", "from_bytes"]),
    ],
    call,
    targets: &[("base64", "base64")],
    binary: Some(binary),
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("upper", "upper(s)", "uppercase a string", &[r#""hello".upper"#, r#"upper("zil")"#], &["lower", "capitalize"]),
    doc("lower", "lower(s)", "lowercase a string", &[r#""HeLLo".lower"#], &["upper", "capitalize"]),
    doc("trim", "trim(s)", "strip leading and trailing whitespace", &[r#""  hi  ".trim"#], &["split"]),
    doc("capitalize", "capitalize(s)", "uppercase the first character", &[r#""hello world".capitalize"#], &["upper"]),
    doc("reverse", "reverse(v)", "reverse a string or list", &[r#""abc".reverse"#, "[1, 2, 3].reverse"], &["sort"]),
    doc("split", "split(s, sep?)", "split on sep (string or regex); whitespace if omitted", &[r#""a b  c".split"#, r#""x,y;z".split(r"[,;]")"#], &["join", "lines", "chars"]),
    doc("lines", "lines(s)", "split into lines", &[r#""a\nb".lines"#], &["split"]),
    doc("chars", "chars(s)", "list of characters", &[r#""abc".chars"#], &["split"]),
    doc("join", "join(list, sep?)", "join items into a string", &[r#"["a", "b"].join("-")"#, "[1, 2].join"], &["split"]),
    doc("replace", "replace(s, pat, with)", "replace all matches; regex replacements can use $1", &[r#""a.b".replace(".", "-")"#, r#""a  b   c".replace(r"\s+", " ")"#], &["find_all", "match"]),
    doc("contains", "contains(v, x)", "substring/regex in a string, item in a list, key in a map", &[r#""price: $12".contains(r"\$\d+")"#, "[1, 2].contains(2)"], &["find", "starts_with"]),
    doc("starts_with", "starts_with(s, prefix)", "whether s starts with prefix", &[r#""abc".starts_with("a")"#], &["ends_with", "contains"]),
    doc("ends_with", "ends_with(s, suffix)", "whether s ends with suffix", &[r#""file.zil".ends_with(".zil")"#], &["starts_with"]),
    doc("find", "find(v, x)", "index of the first match, or nil", &[r#""hello".find("l")"#, "[5, 6].find(6)", r#""abc".find("z")"#], &["contains", "count"]),
    doc("count", "count(v, x)", "number of matches in a string or list", &[r#""banana".count("a")"#, "[1, 2, 1].count(1)"], &["find"]),
    doc("match", "match(s, regex)", "first match, its capture groups, or nil", &[r#""2026-10-06".match(r"(\d+)-(\d+)")"#, r#""id 42".match(r"\d+")"#], &["find_all", "replace"]),
    doc("find_all", "find_all(s, pat)", "list of all matches", &[r#""a1b22c333".find_all(r"\d+")"#], &["match", "count"]),
    doc("grep", "grep(v, pat)", "lines of a string (or items of a list) containing pat", &[r#""ok\nERROR 1\nERROR 2".grep("ERROR")"#, r#"["a1", "b", "c22"].grep(r"\d")"#], &["lines", "filter", "contains"]),
    doc("repeat", "repeat(s, n)", "repeat a string n times", &[r#""ab".repeat(3)"#], &[]),
    doc("base64", "base64(s)", "base64-encode a string; same as `s to base64`", &[r#"base64("hi there")"#, r#""hi" to base64"#], &["encode", "decode"]),
    doc("encode", "encode(s, fmt)", "encode as \"base64\", \"url\" or \"hex\"", &[r#""hi there".encode("base64")"#, r#""a b&c".encode("url")"#], &["decode"]),
    doc("decode", "decode(s, fmt)", "decode \"base64\", \"url\" or \"hex\"", &[r#""aGk=".decode("base64")"#, r#""6869".decode("hex")"#], &["encode"]),
    doc("sha256", "sha256(s)", "hex SHA-256 hash", &[r#""hello".sha256[..16]"#], &["md5"]),
    doc("md5", "md5(s)", "hex MD5 hash", &[r#""hello".md5"#], &["sha256"]),
    doc("ord", "ord(c)", "Unicode code point of a single character", &[r#""A".ord"#, r#""A".ord to hex"#], &["chr", "bytes"]),
    doc("chr", "chr(n)", "character for a Unicode code point", &["97.chr", "(65..70).map(chr).join"], &["ord"]),
    doc("nums", "nums(s)", "every number in a string", &[r#""x=3, y=-2.5; 1e3".nums"#], &["find_all", "parse"]),
    doc("bytes", "bytes(s)", "list of the string's UTF-8 bytes", &[r#""hé".bytes"#], &["from_bytes", "ord"]),
    doc("from_bytes", "from_bytes(list)", "string from a list of UTF-8 bytes", &["[104, 105].from_bytes"], &["bytes", "chr"]),
];

fn call(_: &mut Interp, name: &'static str, args: &[Value], _: &Span) -> Call {
    use Value::*;
    Ok(match (name, args) {
        ("upper", [Str(s)]) => Value::str(s.to_uppercase()),
        ("lower", [Str(s)]) => Value::str(s.to_lowercase()),
        ("trim", [Str(s)]) => Value::str(s.trim()),
        ("capitalize", [Str(s)]) => {
            let mut c = s.chars();
            Value::str(c.next().map(|f| f.to_uppercase().chain(c).collect::<String>()).unwrap_or_default())
        }
        ("reverse", [Str(s)]) => Value::str(s.chars().rev().collect::<String>()),
        ("reverse", [List(l)]) => Value::list(l.borrow().iter().rev().cloned().collect()),
        ("split", [Str(s)]) => strs(s.split_whitespace()),
        ("split", [Str(s), Str(sep)]) => strs(s.split(&**sep)),
        ("split", [Str(s), Regex(r)]) => strs(r.split(s)),
        ("lines", [Str(s)]) => strs(s.lines()),
        ("chars", [Str(s)]) => Value::list(s.chars().map(|c| Value::str(c.to_string())).collect()),
        ("join", [List(l)]) => Value::str(l.borrow().iter().map(Value::to_string).collect::<String>()),
        ("join", [List(l), Str(sep)]) => Value::str(l.borrow().iter().map(Value::to_string).collect::<Vec<_>>().join(sep)),
        ("replace", [Str(s), Str(from), Str(to)]) => Value::str(s.replace(&**from, to)),
        ("replace", [Str(s), Regex(r), Str(to)]) => Value::str(r.replace_all(s, &**to)),
        ("contains", [Str(s), Str(sub)]) => Bool(s.contains(&**sub)),
        ("contains", [Str(s), Regex(r)]) => Bool(r.is_match(s)),
        ("contains", [List(l), v]) => Bool(l.borrow().contains(v)),
        ("contains", [Map(m), Str(k)]) => Bool(m.borrow().contains_key(&**k)),
        ("starts_with", [Str(s), Str(p)]) => Bool(s.starts_with(&**p)),
        ("ends_with", [Str(s), Str(p)]) => Bool(s.ends_with(&**p)),
        ("find", [Str(s), Str(sub)]) => s.find(&**sub).map_or(Nil, |i| char_index(s, i)),
        ("find", [Str(s), Regex(r)]) => r.find(s).map_or(Nil, |m| char_index(s, m.start())),
        ("find", [List(l), v]) => l.borrow().iter().position(|x| x == v).map_or(Nil, |i| Value::int(i as i64)),
        ("count", [Str(s), Str(sub)]) if !sub.is_empty() => Value::int(s.matches(&**sub).count() as i64),
        ("count", [Str(s), Regex(r)]) => Value::int(r.find_iter(s).count() as i64),
        ("count", [List(l), v]) => Value::int(l.borrow().iter().filter(|x| *x == v).count() as i64),
        // Whole match, or the list of groups when the regex has capture groups.
        ("match", [Str(s), Regex(r)]) => match r.captures(s) {
            None => Nil,
            Some(c) if c.len() == 1 => Value::str(&c[0]),
            Some(c) => Value::list(c.iter().skip(1).map(|g| g.map_or(Nil, |g| Value::str(g.as_str()))).collect()),
        },
        ("find_all", [Str(s), Regex(r)]) => strs(r.find_iter(s).map(|m| m.as_str())),
        ("find_all", [Str(s), Str(sub)]) if !sub.is_empty() => strs(s.matches(&**sub)),
        ("grep", [Str(s), Str(p)]) => strs(s.lines().filter(|l| l.contains(&**p))),
        ("grep", [Str(s), Regex(r)]) => strs(s.lines().filter(|l| r.is_match(l))),
        ("grep", [List(l), p @ (Str(_) | Regex(_))]) => {
            let hit = |v: &Value| match (v.to_string(), p) {
                (t, Str(p)) => t.contains(&**p),
                (t, Regex(r)) => r.is_match(&t),
                _ => unreachable!(),
            };
            Value::list(l.borrow().iter().filter(|v| hit(v)).cloned().collect())
        }
        ("repeat", [Str(s), Int(n, _)]) => Value::str(s.repeat((*n).max(0) as usize)),
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
        ("sha256", [Str(s)]) => Value::str(hex(&sha2::Sha256::digest(s.as_bytes()))),
        ("md5", [Str(s)]) => Value::str(hex(&md5::Md5::digest(s.as_bytes()))),
        ("ord", [Str(s)]) => match s.chars().collect::<Vec<_>>()[..] {
            [c] => Value::int(c as i64),
            _ => return Err("expected a single character".into()),
        },
        ("chr", [Int(n, _)]) => {
            let c = u32::try_from(*n).ok().and_then(char::from_u32);
            Value::str(c.ok_or_else(|| format!("{n} is not a valid code point"))?.to_string())
        }
        // Every number in the text; a `-` glued to a word or number (2026-10-06) is a separator, not a sign.
        ("nums", [Str(s)]) => {
            let re = regex::Regex::new(r"-?\d+(\.\d+)?([eE][+-]?\d+)?").unwrap();
            let num = |m: regex::Match| {
                let glued = s[..m.start()].ends_with(|c: char| c.is_alphanumeric());
                let t = if glued { m.as_str().trim_start_matches('-') } else { m.as_str() };
                t.parse().map_or_else(|_| Float(t.parse().unwrap_or(f64::NAN)), Value::int)
            };
            Value::list(re.find_iter(s).map(num).collect())
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
        _ => return Err(Fail::BadArgs),
    })
}

/// `+` with a string on either side concatenates; `str * n` repeats.
fn binary(op: BinOp, a: &Value, b: &Value) -> Claim {
    use Value::*;
    Some(Ok(match (op, a, b) {
        (BinOp::Add, Str(_), _) | (BinOp::Add, _, Str(_)) => Value::str(format!("{a}{b}")),
        (BinOp::Mul, Str(s), Int(n, _)) | (BinOp::Mul, Int(n, _), Str(s)) => Value::str(s.repeat((*n).max(0) as usize)),
        _ => return None,
    }))
}

fn strs<'a>(it: impl Iterator<Item = &'a str>) -> Value {
    Value::list(it.map(Value::str).collect())
}

fn char_index(s: &str, byte: usize) -> Value {
    Value::int(s[..byte].chars().count() as i64)
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
    fn strings() {
        assert_eq!(show(r##""a1b22c333".replace(r"\d+", "#")"##), "a#b#c#");
        assert_eq!(show(r#""a.b.c".replace(".", "-")"#), "a-b-c");
        assert_eq!(show(r#""2026-10-06".match(r"(\d+)-(\d+)-(\d+)")"#), r#"["2026", "10", "06"]"#);
        assert_eq!(show(r#""x1 y22 z333".find_all(r"\d+")"#), r#"["1", "22", "333"]"#);
        assert_eq!(show(r#""  a  b ".split"#), r#"["a", "b"]"#);
        assert_eq!(show(r#""hello world".capitalize"#), "Hello world");
        assert_eq!(show(r#""ok\nERROR 1\nfine\nERROR 2".grep("ERROR")"#), r#"["ERROR 1", "ERROR 2"]"#);
        assert_eq!(show(r#"["a1", "b", "c22"].grep(r"\d")"#), r#"["a1", "c22"]"#);
    }

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
        assert!(try_eval(r#""ab".ord"#).is_err());
        assert!(try_eval("[256].from_bytes").is_err());
        assert_eq!(show(r#""x=3, y=-2.5 on 2026-10-06, 1e3".nums"#), "[3, -2.5, 2026, 10, 6, 1000]");
        assert_eq!(show(r#""abc".sha256"#), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        assert_eq!(show(r#""abc".md5"#), "900150983cd24fb0d6963f7d28e17f72");
    }
}
