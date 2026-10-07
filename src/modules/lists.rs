//! Lists and maps: ranges, higher-order fns, aggregates, sorting.

use super::{Call, Claim, Doc, Fail, Module};
use crate::ast::BinOp;
use crate::interp::{Interp, Value, binary as op, compare};
use crate::lexer::Span;
use std::cmp::Ordering;

pub const MODULE: Module = Module {
    name: "lists",
    about: "ranges, higher-order fns, aggregates, sorting",
    example: r"[3, 1, 2].sort.map(\x -> x * 2)",
    #[rustfmt::skip]
    guide: &[
        ("types", &[("list", "[1, 2, 3]"), ("map", "{a: 1, b: 2}")]),
        ("operators", &[("a..b", "1..4"), ("list + list", "[1] + [2, 3]")]),
    ],
    fns: FNS,
    #[rustfmt::skip]
    groups: &[
        ("build", &["range", "push"]),
        ("transform", &["map", "filter", "reduce", "sort", "unique"]),
        ("aggregate", &["sum", "avg", "min", "max"]),
        ("pick", &["first", "last"]),
        ("maps", &["keys", "values"]),
    ],
    call,
    binary: Some(binary),
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    ("range", "range(n) / range(a, b)", "integers in [0, n) or [a, b); same as a..b", &["range(4)", "range(2, 5)"], &["map"]),
    ("push", "push(list, v)", "append v in place and return the list", &["[1, 2].push(3)"], &[]),
    ("map", "map(list, f)", "apply f to every item", &[r"[1, 2, 3].map(\x -> x * 10)"], &["filter", "reduce"]),
    ("filter", "filter(list, f)", "keep items where f is truthy", &[r"(1..10).filter(\x -> x % 3 == 0)"], &["map", "reduce"]),
    ("reduce", "reduce(list, init, f)", "fold with f(acc, item)", &[r"[1, 2, 3].reduce(10, \acc, x -> acc + x)"], &["sum", "map"]),
    ("sum", "sum(list)", "add up a list; works with units", &["[1, 2, 3].sum", "[1 m, 50 cm].sum"], &["avg", "reduce"]),
    ("avg", "avg(list)", "mean of a list", &["[1, 2, 4].avg", "[2 h, 30 min].avg"], &["sum"]),
    ("min", "min(list) / min(a, b, ...)", "smallest value", &["min(3, 9, 4)", "[2 km, 1 mi].min"], &["max", "sort"]),
    ("max", "max(list) / max(a, b, ...)", "largest value", &["max(3, 9, 4)", r#"["b", "a"].max"#], &["min", "sort"]),
    ("sort", "sort(list, key?)", "sorted copy, optionally by key function", &["[3, 1, 2].sort", r#"["ccc", "a", "bb"].sort(\w -> w.len)"#], &["reverse", "unique"]),
    ("unique", "unique(list)", "drop duplicates, keeping first occurrences", &["[1, 2, 1, 3].unique"], &["sort", "count"]),
    ("first", "first(list)", "first item, or nil", &["[7, 8].first"], &["last"]),
    ("last", "last(list)", "last item, or nil", &["[7, 8].last"], &["first"]),
    ("keys", "keys(map)", "list of map keys", &["{a: 1, b: 2}.keys"], &["values"]),
    ("values", "values(map)", "list of map values", &["{a: 1, b: 2}.values"], &["keys"]),
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
        ("sum", [List(l)]) => sum(&l.borrow())?,
        ("avg", [List(l)]) => {
            let l = l.borrow();
            if l.is_empty() {
                return Err("empty list".into());
            }
            op(BinOp::Div, &sum(&l)?, &Float(l.len() as f64))?
        }
        ("min" | "max", [List(l)]) => extreme(name, &l.borrow())?,
        ("min" | "max", vs) if vs.len() >= 2 => extreme(name, vs)?,
        ("sort", [List(l)]) => {
            let mut keyed: Vec<_> = l.borrow().iter().map(|v| (v.clone(), v.clone())).collect();
            sort_keyed(&mut keyed)?
        }
        ("sort", [List(l), f]) => {
            let mut keyed = Vec::new();
            for v in l.borrow().clone() {
                keyed.push((it.call(f, vec![v.clone()], span)?, v));
            }
            sort_keyed(&mut keyed)?
        }
        ("unique", [List(l)]) => {
            let mut out: Vec<Value> = Vec::new();
            for v in l.borrow().iter() {
                if !out.contains(v) {
                    out.push(v.clone());
                }
            }
            Value::list(out)
        }
        ("first", [List(l)]) => l.borrow().first().cloned().unwrap_or(Nil),
        ("last", [List(l)]) => l.borrow().last().cloned().unwrap_or(Nil),
        ("keys", [Map(m)]) => Value::list(m.borrow().keys().map(|k| Value::str(k.as_str())).collect()),
        ("values", [Map(m)]) => Value::list(m.borrow().values().cloned().collect()),
        _ => return Err(Fail::BadArgs),
    })
}

/// `a..b` ranges and list concatenation.
fn binary(op: BinOp, a: &Value, b: &Value) -> Claim {
    use Value::*;
    Some(match (op, a, b) {
        (BinOp::Range, Int(x, _), Int(y, _)) => range(*x, *y),
        (BinOp::Add, List(x), List(y)) => Ok(Value::list(x.borrow().iter().chain(y.borrow().iter()).cloned().collect())),
        _ => return None,
    })
}

fn range(a: i64, b: i64) -> Result<Value, String> {
    if b.saturating_sub(a) > 10_000_000 {
        return Err("range too large".into());
    }
    Ok(Value::list((a..b).map(Value::int).collect()))
}

fn sum(l: &[Value]) -> Result<Value, String> {
    let Some((first, rest)) = l.split_first() else {
        return Ok(Value::int(0));
    };
    let mut acc = first.clone();
    for v in rest {
        acc = op(BinOp::Add, &acc, v)?;
    }
    Ok(acc)
}

fn extreme(name: &str, vs: &[Value]) -> Result<Value, String> {
    let mut best: Option<&Value> = None;
    for v in vs {
        best = Some(match best {
            None => v,
            Some(b) => {
                let ord = compare(v, b).ok_or_else(|| format!("cannot compare {v:?} and {b:?}"))?;
                if (name == "min" && ord.is_lt()) || (name == "max" && ord.is_gt()) { v } else { b }
            }
        });
    }
    best.cloned().ok_or_else(|| "empty list".into())
}

/// Stable sort of (key, value) pairs by key; errors on incomparable keys.
fn sort_keyed(keyed: &mut [(Value, Value)]) -> Result<Value, String> {
    let mut bad = None;
    keyed.sort_by(|(a, _), (b, _)| {
        compare(a, b).unwrap_or_else(|| {
            bad = Some(format!("cannot compare {a:?} and {b:?}"));
            Ordering::Equal
        })
    });
    match bad {
        Some(e) => Err(e),
        None => Ok(Value::list(keyed.iter().map(|(_, v)| v.clone()).collect())),
    }
}

#[cfg(test)]
mod tests {
    use crate::interp::tests::eval;

    fn show(src: &str) -> String {
        eval(src).to_string()
    }

    #[test]
    fn lists() {
        assert_eq!(show("range(5).reduce(0, \\acc, x -> acc + x)"), "10");
        assert_eq!(show("[3, 1, 2].sort"), "[1, 2, 3]");
        assert_eq!(show(r#"["bb", "a", "ccc"].sort(\s -> s.len)"#), r#"["a", "bb", "ccc"]"#);
        assert_eq!(show("[1, 2, 2, 3].unique"), "[1, 2, 3]");
        assert_eq!(show("[1, 2, 3, 4].avg"), "2.5");
        assert_eq!(show("[1 m, 50 cm].sum"), "1.5 m");
        assert_eq!(show("max(3, 9, 4)"), "9");
        assert_eq!(show("[1 km, 900 m].min"), "900 m");
    }
}
