//! Raw bytes: hex dumps and entropy. Strings count as their UTF-8 bytes.

use crate::interp::Interp;
use crate::lexer::Span;
use crate::modules::text::encoding::bytes;
use crate::modules::{Call, Doc, Fail, Module, doc};
use crate::value::Value;

pub const MODULE: Module = Module {
    name: "binary",
    about: "hex dumps and entropy of strings and byte lists",
    #[rustfmt::skip]
    examples: &[
        ("binary", &[
            ("look inside a string", r#"hexdump("héllo\tworld\n")"#),
            ("how random is it?", r#""aaaaaaaa".entropy"#),
        ]),
    ],
    fns: FNS,
    call,
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("hexdump", "hexdump(v: str|list)", "xxd-style dump of a string's UTF-8 bytes or a list of bytes", &[r#"hexdump("hi\n")"#, "hexdump([0, 255, 65])"], &["bytes", "entropy"]),
    doc("entropy", "entropy(v: str|list)", "Shannon entropy in bits per byte: 0 is constant, 8 is random", &[r#""aaaa".entropy"#, r#""abcd".entropy"#], &["hexdump"]),
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
        _ => return Err(Fail::BadArgs),
    })
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

#[cfg(test)]
mod tests {
    use crate::interp::tests::show;

    #[test]
    fn binary() {
        assert_eq!(show(r#""hi\n".hexdump"#), format!("00000000: {:<39}  hi.", "6869 0a"));
        assert_eq!(show("hexdump([])"), "");
        assert_eq!(show(r#""aaaa".entropy"#), "0");
        assert_eq!(show(r#""abcd".entropy"#), "2");
    }
}
