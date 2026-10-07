//! General: printing, type conversions, number bases, parsing, files, help, `input`.

use super::{Claim, Doc, Module, bad_args, strings::hex};
use crate::Error;
use crate::ast::{Expr, ExprKind, Radix, UnOp};
use crate::help::help;
use crate::interp::{Interp, Value, fits, num};
use crate::lexer::Span;

pub const MODULE: Module = Module {
    name: "general",
    about: "values, printing, conversions, number bases, parsing, files",
    example: "type(5 km)",
    #[rustfmt::skip]
    guide: &[
        ("types", &[("nil", "nil"), ("bool", "true"), ("int", "0xff"), ("float", "1.5e3"), ("fn", r"\x -> x * 2")]),
        ("names", &[("input  (stdin as a string)", "")]),
        ("conversions", &[
            ("to str int float bool list", r#""42" to int"#),
            ("to hex bin oct dec", "255 to bin"),
            ("to hex(bits), to base(b)", "-1 to hex(16)"),
        ]),
    ],
    fns: FNS,
    #[rustfmt::skip]
    groups: &[
        ("values", &["type", "len", "parse"]),
        ("convert", &["str", "int", "float", "bool", "list"]),
        ("bases", &["hex", "bin", "oct", "dec", "base"]),
        ("io", &["print", "read_file", "write_file", "help"]),
    ],
    call,
    #[rustfmt::skip]
    targets: &[
        ("str", "str"), ("int", "int"), ("float", "float"), ("bool", "bool"), ("list", "list"),
        ("hex", "hex"), ("bin", "bin"), ("oct", "oct"), ("dec", "dec"), ("base", "base"),
    ],
    ident: Some(ident),
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    ("print", "print(a, b, ...)", "print values separated by spaces", &[], &["str"]),
    ("type", "type(v)", "the type name of a value", &["type(5 km)", r#"type("hi")"#, "type([1])"], &["str", "int", "float"]),
    ("str", "str(v)", "convert to a string (full float precision)", &["str(1/3)", "str(5 km)"], &["int", "float"]),
    ("int", "int(v, base?)", "convert to an integer, parsing strings in an optional base", &["int(3.9)", r#""ff".int(16)"#, r#""0b101".int"#], &["float", "str"]),
    ("float", "float(v)", "convert to a float; drops a quantity's unit", &[r#"float("2.5")"#, "float(5 km)"], &["int", "str"]),
    ("bool", "bool(v)", "truthiness: false only for nil and false", &["bool(0)", "nil to bool"], &["str"]),
    ("hex", "hex(v, bits?)", "same as `v to hex` / `v to hex(bits)`; strings become hex bytes", &["hex(255)", "hex(-1, 16)", r#"hex("hi")"#], &["bin", "base", "int"]),
    ("bin", "bin(v, bits?)", "same as `v to bin` / `v to bin(bits)`", &["bin(10)", "bin(5, 8)"], &["hex", "oct"]),
    ("oct", "oct(v, bits?)", "same as `v to oct`", &["oct(8)"], &["hex", "bin"]),
    ("dec", "dec(v, bits?)", "same as `v to dec`; with bits, reads two's complement as unsigned", &["dec(0xff)", "dec(-1, 8)"], &["hex", "int"]),
    ("base", "base(v, b)", "same as `v to base(b)`, any base 2-36", &["base(35, 36)", "base(10, 3)"], &["hex", "digits"]),
    ("list", "list(v)", "convert to a list: characters, a copy, or [key, value] pairs", &[r#"list("abc")"#, r#""abc" to list"#, "{a: 1, b: 2}.list"], &["chars", "parse"]),
    ("parse", "parse(s)", "read a zil literal (number, string, list, map, quantity); never runs code", &[r#""[1, 2.5, 0xff]".parse"#, r#""5 km".parse to m"#], &["str", "nums"]),
    ("len", "len(v)", "length of a string, list or map", &[r#""héllo".len"#, "[1, 2, 3].len", "{a: 1}.len"], &[]),
    ("read_file", "read_file(path)", "file contents as a string", &[], &["write_file", "lines"]),
    ("write_file", "write_file(path, v)", "write v to a file as text", &[], &["read_file"]),
    ("help", "help(topic?)", "this help; topic is a function, unit or unit kind", &[], &[]),
];

fn call(_: &mut Interp, name: &'static str, args: Vec<Value>, span: &Span) -> Result<Value, Error> {
    let err = |msg: String| Error::new(format!("{name}: {msg}"), span.clone());
    use Value::*;
    Ok(match (name, args.as_slice()) {
        ("print", vs) => {
            let parts: Vec<String> = vs.iter().map(Value::to_string).collect();
            println!("{}", parts.join(" "));
            Nil
        }
        ("help", []) => help(None).map(|_| Nil).map_err(err)?,
        ("help", [Str(s)]) => help(Some(s)).map(|_| Nil).map_err(err)?,
        ("help", [Builtin(_, f)]) => help(Some(f)).map(|_| Nil).map_err(err)?,
        ("help", [Fn(_)]) => return Err(err("user-defined function; no help available".into())),
        ("type", [v]) => Value::str(v.type_name()),
        ("str", [Float(n)]) => Value::str(n.to_string()),
        ("str", [v]) => Value::str(v.to_string()),
        ("int", [Int(n, _)]) => Value::int(*n),
        ("int", [Float(n)]) => Value::int(*n as i64),
        ("int", [Str(s)]) => parse_int(s, None).ok_or_else(|| err(format!("cannot parse {s:?}")))?,
        ("int", [Str(s), Int(b, _)]) if (2..=36).contains(b) => parse_int(s, Some(*b as u32)).ok_or_else(|| err(format!("cannot parse {s:?} in base {b}")))?,
        ("float", [v @ (Int(..) | Float(_))]) => Float(num(v).unwrap()),
        ("float", [Qty(n, _)]) => Float(*n),
        ("float", [Str(s)]) => Float(s.trim().parse().map_err(|_| err(format!("cannot parse {s:?}")))?),
        ("bool", [v]) => Bool(v.truthy()),
        ("hex" | "bin" | "oct" | "dec" | "base", [v, rest @ ..]) if rest.len() <= 1 => {
            let base = match name {
                "hex" => 16,
                "bin" => 2,
                "oct" => 8,
                "dec" => 10,
                _ => match rest {
                    [Int(b, _)] if (2..=36).contains(b) => *b as u32,
                    _ => return Err(err("expected base(v, 2-36)".into())),
                },
            };
            let width = match (name, rest) {
                ("base", _) | (_, []) => 0,
                (_, [Int(w, _)]) if (1..=64).contains(w) => *w as u32,
                _ => return Err(err("width must be 1-64 bits".into())),
            };
            to_radix(v, Radix { base, width }).map_err(err)?
        }
        ("list", [Str(s)]) => Value::list(s.chars().map(|c| Value::str(c.to_string())).collect()),
        ("list", [List(l)]) => Value::list(l.borrow().clone()),
        ("list", [Map(m)]) => Value::list(m.borrow().iter().map(|(k, v)| Value::list(vec![Value::str(k.as_str()), v.clone()])).collect()),
        ("parse", [Str(s)]) => parse_literal(s).map_err(err)?,
        ("len", [Str(s)]) => Value::int(s.chars().count() as i64),
        ("len", [List(l)]) => Value::int(l.borrow().len() as i64),
        ("len", [Map(m)]) => Value::int(m.borrow().len() as i64),
        ("read_file", [Str(path)]) => Value::str(std::fs::read_to_string(&**path).map_err(|e| err(e.to_string()))?),
        ("write_file", [Str(path), v]) => {
            std::fs::write(&**path, v.to_string()).map_err(|e| err(e.to_string()))?;
            Nil
        }
        _ => return Err(bad_args(name, &args, span)),
    })
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

/// An integer displayed in another base or bit width; strings to hex become their bytes.
fn to_radix(v: &Value, r: Radix) -> Result<Value, String> {
    Ok(match v {
        Value::Int(n, _) if r.width == 0 || fits(*n, r.width) => Value::Int(*n, r),
        Value::Int(n, _) => return Err(format!("{n} does not fit in {} bits", r.width)),
        Value::Float(x) if x.fract() == 0.0 && x.abs() < 9.2e18 => to_radix(&Value::int(*x as i64), r)?,
        Value::Str(s) if r == (Radix { base: 16, width: 0 }) => Value::str(hex(s.as_bytes())),
        v => return Err(format!("cannot convert {} like that", v.type_name())),
    })
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
    let n = i64::from_str_radix(digits, base).ok()?;
    Some(Value::Int(if neg { -n } else { n }, Radix { base, width: 0 }))
}

/// A single zil literal (number, string, list, map, quantity); anything that could run code is refused.
fn parse_literal(s: &str) -> Result<Value, String> {
    fn literal(e: &Expr) -> bool {
        match &e.kind {
            ExprKind::Nil | ExprKind::Bool(_) | ExprKind::Int(..) | ExprKind::Float(_) | ExprKind::Str(_) | ExprKind::Regex(_) => true,
            ExprKind::List(items) => items.iter().all(literal),
            ExprKind::Map(entries) => entries.iter().all(|(_, v)| literal(v)),
            ExprKind::Qty(n, _) | ExprKind::Unary(UnOp::Neg, n) => literal(n),
            _ => false,
        }
    }
    let ast = crate::lexer::lex(s).and_then(crate::parser::parse).map_err(|e| e.msg)?;
    match &ast[..] {
        [e] if literal(e) => Interp::new().run(&ast).map_err(|e| e.msg),
        _ => Err(format!("not a literal value: {s:?}")),
    }
}

#[cfg(test)]
mod tests {
    use crate::interp::tests::{eval, try_eval};

    fn show(src: &str) -> String {
        eval(src).to_string()
    }

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
}
