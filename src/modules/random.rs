//! Randomness: numbers, picks, shuffles, UUIDs.

use super::{Call, Doc, Fail, Module, doc, strings::hex};
use crate::interp::{Interp, Value, num};
use crate::lexer::Span;

pub const MODULE: Module = Module { name: "random", about: "numbers, picks, shuffles, UUIDs", example: "rand(1, 6)", fns: FNS, call, ..Module::EMPTY };

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("rand", "rand() / rand(a, b)", "float in [0, 1), or a number from a to b inclusive", &["rand()", "rand(1, 6)"], &["choice", "shuffle"]),
    doc("choice", "choice(list)", "random item", &[r#"["rock", "paper", "scissors"].choice"#], &["rand", "shuffle"]),
    doc("shuffle", "shuffle(list)", "shuffled copy", &["(1..6).shuffle"], &["choice"]),
    doc("uuid", "uuid()", "random v4 UUID", &["uuid()"], &["rand"]),
];

fn call(_: &mut Interp, name: &'static str, args: &[Value], _: &Span) -> Call {
    use Value::*;
    Ok(match (name, args) {
        ("rand", []) => Float(fastrand::f64()),
        ("rand", [Int(a, _), Int(b, _)]) if a <= b => Value::int(fastrand::i64(*a..=*b)),
        ("rand", [a, b]) if num(a).is_some() && num(b).is_some() => {
            let (a, b) = (num(a).unwrap(), num(b).unwrap());
            Float(a + fastrand::f64() * (b - a))
        }
        ("choice", [List(l)]) => {
            let l = l.borrow();
            if l.is_empty() {
                return Err("empty list".into());
            }
            l[fastrand::usize(..l.len())].clone()
        }
        ("shuffle", [List(l)]) => {
            let mut v = l.borrow().clone();
            fastrand::shuffle(&mut v);
            Value::list(v)
        }
        ("uuid", []) => {
            let mut b: [u8; 16] = std::array::from_fn(|_| fastrand::u8(..));
            b[6] = (b[6] & 0x0f) | 0x40; // version 4
            b[8] = (b[8] & 0x3f) | 0x80; // RFC 4122 variant
            let h = hex(&b);
            Value::str(format!("{}-{}-{}-{}-{}", &h[..8], &h[8..12], &h[12..16], &h[16..20], &h[20..]))
        }
        _ => return Err(Fail::BadArgs),
    })
}

#[cfg(test)]
mod tests {
    use crate::interp::tests::{eval, try_eval};

    #[test]
    fn random() {
        assert!(matches!(eval("rand(1, 6)"), crate::interp::Value::Int(1..=6, _)));
        assert_eq!(eval("uuid().len").to_string(), "36");
        assert_eq!(eval("[1, 2, 3].shuffle.sort").to_string(), "[1, 2, 3]");
        assert!(try_eval("choice([])").is_err());
    }
}
