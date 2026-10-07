//! Numbers: digits, rounding, logs, trigonometry (angles as units).

use super::{Doc, Module, bad_args, units};
use crate::Error;
use crate::ast::Radix;
use crate::interp::{Interp, Value, num};
use crate::lexer::Span;

pub const MODULE: Module = Module {
    name: "math",
    example: "round(sqrt(2), 3)",
    fns: FNS,
    call,
    consts: &[("pi", std::f64::consts::PI), ("e", std::f64::consts::E)],
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    ("digits", "digits(n)", "list of digits in the number's own base", &["1234.digits", "0b1011.digits"], &["from_digits"]),
    ("from_digits", "from_digits(list, base?)", "build a number from digits, kept in that base", &["[1, 2, 3].from_digits", "[1, 0, 1, 1].from_digits(2)"], &["digits"]),
    ("sqrt", "sqrt(x)", "square root", &["sqrt(2)"], &[]),
    ("abs", "abs(x)", "absolute value; keeps units", &["abs(-3)", "abs(-2 km)"], &["round"]),
    ("round", "round(x, digits?)", "round to nearest; keeps units", &["round(pi, 2)", "round(2.5 km)"], &["floor", "ceil"]),
    ("floor", "floor(x)", "round down", &["floor(2.7)"], &["ceil", "round"]),
    ("ceil", "ceil(x)", "round up", &["ceil(2.1)"], &["floor", "round"]),
    ("ln", "ln(x)", "natural log", &["ln(e)"], &["log"]),
    ("log", "log(x, base?)", "log base 10, or another base", &["log(1000)", "log(8, 2)"], &["ln"]),
    ("sin", "sin(x)", "sine of radians or an angle unit", &["sin(30 deg)", "sin(pi / 2)"], &["cos", "tan", "asin"]),
    ("cos", "cos(x)", "cosine of radians or an angle unit", &["cos(60 deg)"], &["sin", "tan", "acos"]),
    ("tan", "tan(x)", "tangent of radians or an angle unit", &["tan(45 deg)"], &["sin", "cos", "atan"]),
    ("asin", "asin(x)", "arcsine, as an angle", &["asin(1) to deg"], &["sin"]),
    ("acos", "acos(x)", "arccosine, as an angle", &["acos(0) to deg"], &["cos"]),
    ("atan", "atan(x)", "arctangent, as an angle", &["atan(1) to deg"], &["tan"]),
];

fn call(_: &mut Interp, name: &'static str, args: Vec<Value>, span: &Span) -> Result<Value, Error> {
    let err = |msg: String| Error::new(format!("{name}: {msg}"), span.clone());
    let bad = || bad_args(name, &args, span);
    use Value::*;
    Ok(match (name, args.as_slice()) {
        ("digits", [Int(n, r)]) => {
            let (mut m, b) = (n.unsigned_abs(), r.base as u64);
            let mut d = vec![Value::int((m % b) as i64)];
            while m >= b {
                m /= b;
                d.push(Value::int((m % b) as i64));
            }
            d.reverse();
            Value::list(d)
        }
        ("from_digits", [List(l)]) => from_digits(&l.borrow(), 10).map_err(err)?,
        ("from_digits", [List(l), Int(b, _)]) if (2..=36).contains(b) => from_digits(&l.borrow(), *b as u32).map_err(err)?,
        ("sqrt", [v]) if num(v).is_some() => Float(num(v).unwrap().sqrt()),
        ("abs" | "round" | "floor" | "ceil", [v]) => round_like(name, v, 0).ok_or_else(bad)?,
        ("round", [v, Int(n, _)]) => round_like(name, v, *n).ok_or_else(bad)?,
        ("ln", [v]) if num(v).is_some() => Float(num(v).unwrap().ln()),
        ("log", [v]) if num(v).is_some() => Float(num(v).unwrap().log10()),
        ("log", [v, b]) if num(v).is_some() && num(b).is_some() => Float(num(v).unwrap().log(num(b).unwrap())),
        ("sin" | "cos" | "tan", [v]) => {
            let rad = match v {
                Qty(x, u) if u.dim() == units::unit("rad").unwrap().dim() => u.to_si(*x),
                _ => num(v).ok_or_else(bad)?,
            };
            Float(match name {
                "sin" => rad.sin(),
                "cos" => rad.cos(),
                _ => rad.tan(),
            })
        }
        ("asin" | "acos" | "atan", [v]) if num(v).is_some() => {
            let x = num(v).unwrap();
            let rad = match name {
                "asin" => x.asin(),
                "acos" => x.acos(),
                _ => x.atan(),
            };
            Qty(rad, units::unit("rad").unwrap())
        }
        _ => return Err(bad()),
    })
}

/// abs/round/floor/ceil on numbers and quantities (keeping the unit).
fn round_like(name: &str, v: &Value, digits: i64) -> Option<Value> {
    let f = |x: f64| -> f64 {
        let p = 10f64.powi(digits as i32);
        match name {
            "abs" => x.abs(),
            "round" => (x * p).round() / p,
            "floor" => x.floor(),
            _ => x.ceil(),
        }
    };
    Some(match v {
        Value::Int(n, b) if name == "abs" => Value::Int(n.checked_abs()?, *b),
        Value::Int(..) if digits >= 0 => v.clone(),
        Value::Float(x) if digits == 0 && name != "abs" && f(*x).abs() < 9.2e18 => Value::int(f(*x) as i64),
        Value::Float(x) => Value::Float(f(*x)),
        Value::Qty(x, u) => Value::Qty(f(*x), u.clone()),
        _ => return None,
    })
}

fn from_digits(l: &[Value], base: u32) -> Result<Value, String> {
    let mut acc: i64 = 0;
    for v in l {
        let d = match v {
            Value::Int(d, _) if (0..base as i64).contains(d) => *d,
            _ => return Err(format!("{v:?} is not a base-{base} digit")),
        };
        acc = acc.checked_mul(base as i64).and_then(|a| a.checked_add(d)).ok_or("integer overflow")?;
    }
    Ok(Value::Int(acc, Radix { base, width: 0 }))
}

#[cfg(test)]
mod tests {
    use crate::interp::tests::{eval, try_eval};

    fn show(src: &str) -> String {
        eval(src).to_string()
    }

    #[test]
    fn math() {
        assert_eq!(show("sqrt(16)"), "4");
        assert_eq!(show("round(pi, 2)"), "3.14");
        assert_eq!(show("round(2.5 km)"), "3 km");
        assert_eq!(show("sin(30 deg)"), "0.5");
        assert_eq!(show("asin(1) to deg"), "90 deg");
        assert_eq!(show("log(1000)"), "3");
        assert_eq!(show("1234.digits"), "[1, 2, 3, 4]");
        assert_eq!(show("0b110.digits"), "[1, 1, 0]");
        assert_eq!(show("0.digits"), "[0]");
        assert_eq!(show("[1, 1, 0].from_digits(2)"), "0b110");
        assert_eq!(show("0xff.digits.from_digits(16)"), "0xff");
        assert!(try_eval("[2].from_digits(2)").is_err());
    }
}
