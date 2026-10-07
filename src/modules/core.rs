//! Core: printing, type conversions, parsing, files, help, `input`.

use super::{Call, Claim, Doc, Fail, Module, doc};
use crate::ast::{BinOp, Expr, ExprKind, Radix, UnOp};
use crate::help::help;
use crate::interp::Interp;
use crate::lexer::Span;
use crate::value::{Value, exact, num};
use num_bigint::BigInt;
use num_rational::BigRational;

pub const MODULE: Module = Module {
    name: "core",
    about: "values, printing, type conversions, parsing, files, help",
    #[rustfmt::skip]
    examples: &[
        ("core", &[
            ("type of anything", "type(5 km)"),
            ("parse values from text", r#"parse("[1, 2, 5 km]")"#),
        ]),
    ],
    #[rustfmt::skip]
    guide: &[
        ("types", &[("nil", "nil"), ("bool", "true"), ("int", "0xff"), ("float", "1.5e3"), ("frac", "7/2 to frac"), ("fn", r"|x| x * 2")]),
        ("names", &[("input  (stdin as a string): input.lines.len", "")]),
        ("conversions", &[("to str int float frac bool list", r#""42" to int"#)]),
    ],
    fns: FNS,
    #[rustfmt::skip]
    groups: &[
        ("values", &["type", "parse"]),
        ("convert", &["str", "int", "float", "frac", "bool", "list"]),
        ("io", &["print", "read_file", "write_file", "help"]),
    ],
    call,
    #[rustfmt::skip]
    targets: &[("str", "str"), ("int", "int"), ("float", "float"), ("frac", "frac"), ("bool", "bool"), ("list", "list")],
    ident: Some(ident),
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("print", "print(a?: any, ...)", "print values separated by spaces", &[], &["str"]).shown(&[r#"print("total:", 5 km)"#]),
    doc("type", "type(v: any)", "the type name of a value", &["type(5 km)", r#"type("hi")"#, "type([1])"], &["str", "int", "float"]),
    doc("str", "str(v: any)", "convert to a string (full float precision)", &["str(1/3)", "str(5 km)"], &["int", "float"]),
    doc("int", "int(v: num|str) / int(s: str, base: int)", "convert to an integer, parsing strings in an optional base", &["int(3.9)", r#""ff".int(16)"#, r#""0b101".int"#], &["float", "str"]),
    doc("float", "float(v: num|quantity|str)", "convert to a float; drops a quantity's unit", &[r#"float("2.5")"#, "float(5 km)"], &["int", "str"]),
    doc("frac", "frac(v: num)", "show as a fraction; floats become the simplest fraction within 1e-12", &["7/2 to frac", "1/3 + 1/6 to frac", "0.75.frac"], &["float"]),
    doc("bool", "bool(v: any)", "truthiness: false only for nil and false", &["bool(0)", "nil to bool"], &["str"]),
    doc("list", "list(v: str|list|map|set)", "convert to a list: characters, a copy, [key, value] pairs, or a set's items", &[r#"list("abc")"#, r#""abc" to list"#, "{a: 1, b: 2}.list"], &["chars", "parse"]),
    doc("parse", "parse(s: str)", "read a zil literal (number, string, list, map, quantity); never runs code", &[r#""[1, 2.5, 0xff]".parse"#, r#""5 km".parse to m"#], &["str", "nums"]),
    doc("read_file", "read_file(path: str)", "file contents as a string", &[], &["write_file", "lines"]).shown(&[r#"read_file("notes.txt").lines.len"#]),
    doc("write_file", "write_file(path: str, v: any)", "write v to a file as text", &[], &["read_file"]).shown(&[r#"write_file("out.txt", [1, 2, 3])"#]),
    doc("help", "help(topic?: any)", "this help, as text; topic is a function, module (\"trig\" or \"math.trig\"), unit, or any value to list functions for its type", &[], &[]).shown(&["help(upper)", r#"help("math.trig")"#, "help(today)", r#"help("text") |> grep("case")"#]),
];

fn call(_: &mut Interp, name: &'static str, args: &[Value], _: &Span) -> Call {
    use Value::*;
    Ok(match (name, args) {
        ("print", vs) => {
            let parts: Vec<String> = vs.iter().map(Value::to_string).collect();
            println!("{}", parts.join(" "));
            Nil
        }
        // Plain text, so it can be piped to `grep` etc. A miss returns its note: not finding help isn't an error in the user's code.
        ("help", []) => Value::str(help(None, false).unwrap_or_else(|e| e)),
        ("help", [Str(s)]) => Value::str(help(Some(s), false).unwrap_or_else(|e| e)),
        ("help", [Builtin(_, f)]) => Value::str(help(Some(f), false).unwrap_or_else(|e| e)),
        ("help", [Fn(_)]) => return Err(Fail::Arg(0, "no help for user-defined functions".into())),
        ("help", [v]) => {
            let t = v.type_name();
            Value::str(
                crate::help::capture(false, || crate::help::type_page(t)).unwrap_or_else(|_| format!("no functions take a {t} first; help() for an overview")),
            )
        }
        ("type", [v]) => Value::str(v.type_name()),
        ("str", [v @ (Float(_) | Frac(_, false))]) => Value::str(num(v).unwrap().to_string()),
        ("str", [v]) => Value::str(v.to_string()),
        ("int", [Int(n, _)]) => Value::int(*n),
        ("int", [Big(n, _)]) => Big(n.clone(), Radix::DEC),
        ("int", [Frac(r, _)]) => exact(BigRational::from_integer(r.to_integer()), Radix::DEC, false),
        ("int", [Float(n)]) => Value::int(*n as i64),
        ("int", [Str(s)]) => parse_int(s, None).ok_or_else(|| Fail::Arg(0, format!("cannot read {s:?} as an integer")))?,
        ("int", [Str(s), Int(b, _)]) if (2..=36).contains(b) => {
            parse_int(s, Some(*b as u32)).ok_or_else(|| Fail::Arg(0, format!("cannot read {s:?} as a base-{b} integer")))?
        }
        ("int", [Str(_), Int(b, _)]) => return Err(Fail::Arg(1, format!("base {b} is out of range\nnote: bases go from 2 to 36"))),
        ("float", [v]) if num(v).is_some() => Float(num(v).unwrap()),
        ("float", [Qty(n, _)]) => Float(*n),
        ("float", [Str(s)]) => Float(s.trim().parse().map_err(|_| Fail::Arg(0, format!("cannot read {s:?} as a number")))?),
        ("frac", [v @ (Int(..) | Big(..))]) => v.clone(),
        ("frac", [Frac(r, _)]) => Frac(r.clone(), true),
        ("frac", [Float(x)]) => exact(simplest(*x).ok_or_else(|| format!("`{x}` has no fraction"))?, Radix::DEC, true),
        ("bool", [v]) => Bool(v.truthy()),
        ("list", [Str(s)]) => Value::list(s.chars().map(|c| Value::str(c.to_string())).collect()),
        ("list", [List(l)]) => Value::list(l.borrow().clone()),
        ("list", [Set(s)]) => Value::list(s.borrow().iter().cloned().collect()),
        ("list", [Map(m)]) => Value::list(m.borrow().iter().map(|(k, v)| Value::list(vec![Value::str(k.as_str()), v.clone()])).collect()),
        ("parse", [Str(s)]) => parse_literal(s)?,
        ("read_file", [Str(path)]) => {
            Value::str(std::fs::read_to_string(&**path).map_err(|e| Fail::Arg(0, format!("cannot read {path:?}: {}", io_reason(&e))))?)
        }
        ("write_file", [Str(path), v]) => {
            std::fs::write(&**path, v.to_string()).map_err(|e| Fail::Arg(0, format!("cannot write {path:?}: {}", io_reason(&e))))?;
            Nil
        }
        _ => return Err(Fail::BadArgs),
    })
}

/// An io error in lowercase with no OS code: `no such file or directory`.
fn io_reason(e: &std::io::Error) -> String {
    let s = e.to_string();
    let s = s.split(" (os error").next().unwrap_or(&s).trim_end_matches('.');
    let mut c = s.chars();
    c.next().map_or(String::new(), |f| f.to_lowercase().chain(c).collect())
}

/// `input`: all of stdin, read once.
fn ident(it: &mut Interp, name: &str) -> Claim {
    if name != "input" {
        return None;
    }
    if it.input.is_none() {
        let mut s = String::new();
        if let Err(e) = std::io::Read::read_to_string(&mut std::io::stdin(), &mut s) {
            return Some(Err(e.to_string()));
        }
        it.input = Some(s.into());
    }
    Some(Ok(Value::Str(it.input.clone().unwrap())))
}

/// Accepts `0x`/`0b`/`0o`/`36#` prefixes and `_` separators when no base is given.
fn parse_int(s: &str, base: Option<u32>) -> Option<Value> {
    let s = s.trim().replace('_', "");
    let (neg, s) = match s.strip_prefix('-') {
        Some(rest) => (true, rest.to_string()),
        None => (false, s),
    };
    let (base, digits) = match base {
        Some(b) => (b, s.as_str()),
        None => match s.get(..2) {
            Some("0x" | "0X") => (16, &s[2..]),
            Some("0b" | "0B") => (2, &s[2..]),
            Some("0o" | "0O") => (8, &s[2..]),
            _ => match s.split_once('#') {
                Some((b, d)) => (b.parse().ok().filter(|b| (2..=36).contains(b))?, d),
                None => (10, s.as_str()),
            },
        },
    };
    if !digits.bytes().all(|c| c.is_ascii_alphanumeric()) {
        return None;
    }
    let n = BigInt::parse_bytes(digits.as_bytes(), base)?;
    Some(exact(BigRational::from_integer(if neg { -n } else { n }), Radix { base, width: 0 }, false))
}

/// The simplest fraction within 1e-12 of `x` (relative), by continued fractions: 0.1 + 0.2 is 3/10.
fn simplest(x: f64) -> Option<BigRational> {
    if x.fract() == 0.0 || !x.is_finite() {
        return BigRational::from_float(x);
    }
    let (mut h, mut k) = ((1i128, 0i128), (0i128, 1i128)); // (current, previous) convergent numerators / denominators
    let mut f = x;
    loop {
        let a = f.floor();
        let next = (a as i128).checked_mul(h.0).and_then(|n| n.checked_add(h.1)).zip((a as i128).checked_mul(k.0).and_then(|d| d.checked_add(k.1)));
        let Some((n, d)) = next.filter(|_| a.abs() < 1e18) else {
            break;
        };
        (h, k) = ((n, h.0), (d, k.0));
        if (n as f64 / d as f64 - x).abs() <= x.abs() * 1e-12 || f == a {
            break;
        }
        f = 1.0 / (f - a);
    }
    Some(BigRational::new(h.0.into(), k.0.into()))
}

/// A single zil literal (number, string, list, map, quantity); anything that could run code is refused.
fn parse_literal(s: &str) -> Result<Value, String> {
    fn literal(e: &Expr) -> bool {
        match &e.kind {
            ExprKind::Nil
            | ExprKind::Bool(_)
            | ExprKind::Int(..)
            | ExprKind::Big(..)
            | ExprKind::Float(_)
            | ExprKind::Dec(_)
            | ExprKind::Str(_)
            | ExprKind::Regex(_) => true,
            ExprKind::List(items) => items.iter().all(literal),
            ExprKind::Map(entries) => entries.iter().all(|(_, v)| literal(v)),
            // `set(1, 2)`, as sets print.
            ExprKind::Call(f, args) => matches!(&f.kind, ExprKind::Ident(n) if n == "set") && args.iter().all(literal),
            ExprKind::Qty(n, _) | ExprKind::Unary(UnOp::Neg, n) => literal(n),
            // `5 ft 11 in`, as `to ft in` prints it.
            ExprKind::Binary(BinOp::Add, a, b) => matches!(a.kind, ExprKind::Qty(..)) && literal(a) && literal(b),
            _ => false,
        }
    }
    let ast = crate::parser::parse(s).map_err(|e| e.msg)?;
    match &ast[..] {
        [e] if literal(e) => Interp::new().run(&ast).map_err(|e| e.msg),
        _ => Err(format!("not a literal value: {s:?}")),
    }
}

#[cfg(test)]
mod tests {
    use crate::interp::tests::{show, try_eval};

    #[test]
    fn conversions() {
        assert_eq!(show(r#""ff".int(16)"#), "0xff");
        assert_eq!(show(r#""0b101".int"#), "0b101");
        assert_eq!(show(r#""z".int(36)"#), "36#z");
        assert_eq!(show(r#""-36#z".int to dec"#), "-35");
        assert_eq!(show(r#"list("ab")"#), r#"["a", "b"]"#);
        assert_eq!(show("{a: 1}.list"), r#"[["a", 1]]"#);
    }

    #[test]
    fn parse() {
        assert_eq!(show(r#""[1, -2.5, \"a\", 0xff, 5 km, \{k: [nil, true]}]".parse"#), r#"[1, -2.5, "a", 0xff, 5 km, {k: [nil, true]}]"#);
        assert_eq!(
            show(
                r#"x = [1, "b"]
str(x).parse == x"#
            ),
            "true"
        );
        for bad in [r#""read_file(\"x\")".parse"#, r#""[x]".parse"#, r#""1 + 2".parse"#, r#""".parse"#] {
            assert!(try_eval(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn type_conversions() {
        assert_eq!(show(r#""ab" to list"#), r#"["a", "b"]"#);
        assert_eq!(show(r#""0xff" to int"#), "0xff");
        assert_eq!(show(r#""2.5" to float * 2"#), "5");
        assert_eq!(show("[1, 2] to str"), "[1, 2]");
        assert_eq!(show("0 to bool"), "true");
        assert_eq!(show("nil to bool"), "false");
        assert_eq!(show("5 km to float"), "5");
        assert!(try_eval("5 to list").is_err());
        assert_eq!(show(r#""hi there" to base64"#), "aGkgdGhlcmU=");
        assert_eq!(show(r#"base64("hi there")"#), "aGkgdGhlcmU=");
    }
}
