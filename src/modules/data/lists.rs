//! Lists: ranges and higher-order fns.

use crate::ast::BinOp;
use crate::interp::Interp;
use crate::lexer::Span;
use crate::modules::{Call, Claim, Doc, Fail, Module, doc};
use crate::value::Value;

pub const MODULE: Module = Module {
    name: "lists",
    about: "ranges, map/filter/reduce, building lists",
    #[rustfmt::skip]
    examples: &[
        ("lists", &[
            ("stepped range", "(1..=10).step(3)"),
            ("primes under 100", "(1..100).filter(is_prime).len"),
            ("pipe into a function", r#"(1..=10).map(|x| x ** 2) |> sum"#),
            ("fold", r#"[1, 2, 3].reduce(10, |acc, x| acc + x)"#),
        ]),
    ],
    #[rustfmt::skip]
    guide: &[
        ("operators", &[("a..b", "1..4"), ("a..=b", "1..=4"), ("list + list", "[1] + [2, 3]")]),
    ],
    fns: FNS,
    #[rustfmt::skip]
    groups: &[
        ("build", &["range", "push", "step"]),
        ("transform", &["map", "filter", "reduce"]),
    ],
    call,
    binary: Some(binary),
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("range", "range(n: int) / range(a: int, b: int)", "integers in [0, n) or [a, b); same as a..b", &["range(4)", "range(2, 5)"], &["map"]),
    doc("push", "push(xs: list, v: any)", "append v in place and return the list", &["[1, 2].push(3)"], &[]),
    doc("map", "map(xs: list, f: fn)", "apply f to every item", &[r"[1, 2, 3].map(|x| x * 10)"], &["filter", "reduce"]),
    doc("filter", "filter(xs: list, f: fn)", "keep items where f is truthy", &[r"(1..10).filter(|x| x % 3 == 0)"], &["map", "reduce"]),
    doc("reduce", "reduce(xs: list, init: any, f: fn)", "fold with f(acc, item)", &[r"[1, 2, 3].reduce(10, |acc, x| acc + x)"], &["sum", "map"]),
    doc("step", "step(xs: list, n: int)", "every nth item, starting with the first", &["(0..=20).step(5)", "(1..10).step(2)"], &["range"]),
];

fn call(it: &mut Interp, name: &'static str, args: &[Value], span: &Span) -> Call {
    use Value::*;
    Ok(match (name, args) {
        ("range", [Int(n, _)]) => range(0, *n)?,
        ("range", [Int(a, _), Int(b, _)]) => range(*a, *b)?,
        ("push", [List(l), v]) => {
            l.borrow_mut().push(v.clone());
            List(l.clone())
        }
        ("map", [List(l), f]) => {
            let items = l.borrow().clone();
            let mut out = Vec::with_capacity(items.len());
            for v in items {
                out.push(it.call(f, vec![v], span)?);
            }
            Value::list(out)
        }
        ("filter", [List(l), f]) => {
            let items = l.borrow().clone();
            let mut out = Vec::new();
            for v in items {
                if it.call(f, vec![v.clone()], span)?.truthy() {
                    out.push(v);
                }
            }
            Value::list(out)
        }
        ("reduce", [List(l), init, f]) => {
            let items = l.borrow().clone();
            let mut acc = init.clone();
            for v in items {
                acc = it.call(f, vec![acc, v], span)?;
            }
            acc
        }
        ("step", [List(l), Int(n, _)]) if *n > 0 => Value::list(l.borrow().iter().step_by(*n as usize).cloned().collect()),
        ("step", [List(_), Int(n, _)]) => return Err(Fail::Arg(1, format!("step must be at least 1, got {n}"))),
        _ => return Err(Fail::BadArgs),
    })
}

/// `a..b` ranges and list concatenation.
fn binary(op: BinOp, a: &Value, b: &Value) -> Claim {
    use Value::*;
    Some(match (op, a, b) {
        (BinOp::Range, Int(x, _), Int(y, _)) => range(*x, *y),
        (BinOp::RangeIncl, Int(x, _), Int(y, _)) => y.checked_add(1).ok_or_else(|| "range too large".into()).and_then(|y| range(*x, y)),
        (BinOp::Add, List(x), List(y)) => Ok(Value::list(x.borrow().iter().chain(y.borrow().iter()).cloned().collect())),
        _ => return None,
    })
}

fn range(a: i64, b: i64) -> Result<Value, String> {
    if b.saturating_sub(a) > 10_000_000 {
        return Err(format!("range too large\nnote: ranges hold at most 10,000,000 items; this one has {}", b.saturating_sub(a)));
    }
    Ok(Value::list((a..b).map(Value::int).collect()))
}

#[cfg(test)]
mod tests {
    use crate::interp::tests::show;

    #[test]
    fn lists() {
        assert_eq!(show("range(5).reduce(0, |acc, x| acc + x)"), "10");
        assert_eq!(show("(1..10).filter(|x| x % 3 == 0).map(|x| x * 2)"), "[6, 12, 18]");
        assert_eq!(show("[[1] + [2], (0..=6).step(3)]"), "[[1, 2], [0, 3, 6]]");
    }
}
