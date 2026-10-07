//! Roman numerals, both ways.

use super::{Call, Doc, Fail, Module, doc};
use crate::interp::Interp;
use crate::lexer::Span;
use crate::value::Value;

pub const MODULE: Module = Module {
    name: "numerals",
    about: "Roman numerals, both ways",
    #[rustfmt::skip]
    examples: &[
        ("roman", &[
            ("this year", "2026 to roman"),
            ("Super Bowl math", r#"roman("LX") + 1 to roman"#),
        ]),
    ],
    #[rustfmt::skip]
    guide: &[("conversions", &[("to roman", "1999 to roman")])],
    fns: FNS,
    call,
    targets: &[("roman", "roman")],
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("roman", "roman(v)", "an int 1-3999 as a Roman numeral, or a numeral back to an int; same as `n to roman`", &["roman(2026)", r#""MCMXCIX".roman"#], &[]),
];

const DIGITS: [(i64, &str); 13] =
    [(1000, "M"), (900, "CM"), (500, "D"), (400, "CD"), (100, "C"), (90, "XC"), (50, "L"), (40, "XL"), (10, "X"), (9, "IX"), (5, "V"), (4, "IV"), (1, "I")];

fn call(_: &mut Interp, name: &'static str, args: &[Value], _: &Span) -> Call {
    match (name, args) {
        ("roman", [Value::Int(n, _)]) if (1..4000).contains(n) => Ok(Value::str(to_roman(*n))),
        ("roman", [Value::Int(..)]) => Err("Romans only went from 1 to 3999".into()),
        ("roman", [Value::Str(s)]) => {
            let s = s.trim().to_uppercase();
            // Read greedily, then check by writing it back: rejects IIII, IC, VX and friends.
            let (mut n, mut rest) = (0, s.as_str());
            for (v, d) in DIGITS {
                while let Some(r) = rest.strip_prefix(d) {
                    n += v;
                    rest = r;
                }
            }
            if !rest.is_empty() || n == 0 || to_roman(n) != s {
                return Err(format!("not a Roman numeral: {s:?}").into());
            }
            Ok(Value::int(n))
        }
        _ => Err(Fail::BadArgs),
    }
}

fn to_roman(mut n: i64) -> String {
    let mut s = String::new();
    for (v, d) in DIGITS {
        while n >= v {
            s += d;
            n -= v;
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use crate::interp::tests::{show, try_eval};

    #[test]
    fn roman() {
        assert_eq!(show("2026 to roman"), "MMXXVI");
        assert_eq!(show("3999.roman"), "MMMCMXCIX");
        assert_eq!(show(r#""mcmxcix".roman"#), "1999");
        assert!((1..4000).all(|n| show(&format!("{n}.roman.roman")) == n.to_string()));
        assert!(try_eval(r#""IIII".roman"#).is_err());
        assert!(try_eval(r#""IC".roman"#).is_err());
        assert!(try_eval("0.roman").is_err());
    }
}
