//! Lists and maps: ranges, higher-order fns, aggregates, sorting.

use super::units::Unit;
use super::{Call, Claim, Doc, Fail, Module, doc};
use crate::ast::BinOp;
use crate::interp::{Interp, binary as op};
use crate::lexer::Span;
use crate::value::{Value, compare, num};
use std::cmp::Ordering;
use std::collections::HashMap;

pub const MODULE: Module = Module {
    name: "lists",
    about: "ranges, higher-order fns, aggregates, sorting",
    #[rustfmt::skip]
    examples: &[
        ("lists", &[
            ("stepped range", "(1..=10).step(3)"),
            ("primes under 100", "(1..100).filter(is_prime).len"),
            ("pipe into a function", r#"(1..=10).map(\x -> x ** 2) |> sum"#),
            ("fold", r#"[1, 2, 3].reduce(10, \acc, x -> acc + x)"#),
            ("standard deviation", "[2, 4, 4, 4, 5, 5, 7, 9].stdev"),
            ("percentile", "(1..=100).percentile(90)"),
            ("unique, order kept", "[3, 1, 3, 2, 1].unique"),
            ("sort quantities", "[1 km, 900 m, 1 mi].sort"),
            ("sort dates", r#"[date("2026-12-25"), date("2026-01-01")].sort"#),
            ("sum a map", "{a: 1, b: 2}.values.sum"),
        ]),
    ],
    #[rustfmt::skip]
    guide: &[
        ("types", &[("list", "[1, 2, 3]"), ("map", "{a: 1, b: 2}")]),
        ("operators", &[("a..b", "1..4"), ("a..=b", "1..=4"), ("x in list", "2 in [1, 2]"), ("list + list", "[1] + [2, 3]")]),
    ],
    fns: FNS,
    #[rustfmt::skip]
    groups: &[
        ("build", &["range", "push"]),
        ("transform", &["map", "filter", "reduce", "sort", "unique"]),
        ("aggregate", &["sum", "product", "avg", "min", "max"]),
        ("stats", &["median", "mode", "percentile", "variance", "stdev"]),
        ("pick", &["first", "last", "step"]),
        ("maps", &["keys", "values"]),
    ],
    call,
    binary: Some(binary),
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("range", "range(n) / range(a, b)", "integers in [0, n) or [a, b); same as a..b", &["range(4)", "range(2, 5)"], &["map"]),
    doc("push", "push(list, v)", "append v in place and return the list", &["[1, 2].push(3)"], &[]),
    doc("map", "map(list, f)", "apply f to every item", &[r"[1, 2, 3].map(\x -> x * 10)"], &["filter", "reduce"]),
    doc("filter", "filter(list, f)", "keep items where f is truthy", &[r"(1..10).filter(\x -> x % 3 == 0)"], &["map", "reduce"]),
    doc("reduce", "reduce(list, init, f)", "fold with f(acc, item)", &[r"[1, 2, 3].reduce(10, \acc, x -> acc + x)"], &["sum", "map"]),
    doc("sum", "sum(list)", "add up a list; works with units", &["[1, 2, 3].sum", "[1 m, 50 cm].sum"], &["avg", "reduce"]),
    doc("avg", "avg(list)", "mean of a list", &["[1, 2, 4].avg", "[2 h, 30 min].avg"], &["sum", "median"]),
    doc("product", "product(list)", "multiply a list together", &["[2, 3, 4].product", "[2 m, 3 m].product"], &["sum", "factorial"]),
    doc("median", "median(list)", "middle value; mean of the middle two for an even count", &["[3, 1, 2].median", "[1 m, 3 m, 50 cm, 2 m].median"], &["avg", "percentile"]),
    doc("mode", "mode(list)", "most common item; the first one on ties", &["[1, 2, 2, 3].mode", r#""hello".chars.mode"#], &["median", "count"]),
    doc("percentile", "percentile(list, p)", "the p-th percentile (0-100), interpolating between items", &["[1, 2, 3, 4, 5].percentile(90)", "(1..=100).percentile(25)"], &["median"]),
    doc("variance", "variance(list)", "sample variance (n - 1)", &["[2, 4, 4, 4, 5, 5, 7, 9].variance"], &["stdev"]),
    doc("stdev", "stdev(list)", "sample standard deviation (n - 1); works with units", &["[2, 4, 4, 4, 5, 5, 7, 9].stdev", "[1 m, 2 m, 3 m].stdev"], &["variance", "avg"]),
    doc("min", "min(list) / min(a, b, ...)", "smallest value", &["min(3, 9, 4)", "[2 km, 1 mi].min"], &["max", "sort"]),
    doc("max", "max(list) / max(a, b, ...)", "largest value", &["max(3, 9, 4)", r#"["b", "a"].max"#], &["min", "sort"]),
    doc("sort", "sort(list, key?)", "sorted copy, optionally by key function", &["[3, 1, 2].sort", r#"["ccc", "a", "bb"].sort(\w -> w.len)"#], &["reverse", "unique"]),
    doc("unique", "unique(list)", "drop duplicates, keeping first occurrences", &["[1, 2, 1, 3].unique"], &["sort", "count"]),
    doc("first", "first(list)", "first item, or nil", &["[7, 8].first"], &["last"]),
    doc("last", "last(list)", "last item, or nil", &["[7, 8].last"], &["first"]),
    doc("step", "step(list, n)", "every nth item, starting with the first", &["(0..=20).step(5)", "(1..10).step(2)"], &["range"]),
    doc("keys", "keys(map)", "list of map keys", &["{a: 1, b: 2}.keys"], &["values"]),
    doc("values", "values(map)", "list of map values", &["{a: 1, b: 2}.values"], &["keys"]),
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
        ("product", [List(l)]) => {
            let mut acc = Value::int(1);
            for v in l.borrow().iter() {
                acc = op(BinOp::Mul, &acc, v)?;
            }
            acc
        }
        ("median" | "percentile" | "variance" | "stdev", [List(l), rest @ ..]) => {
            let (mut xs, u) = floats(&l.borrow())?;
            let qty = |x: f64, u: Option<Unit>| u.map_or(Float(x), |u| Value::qty(x, u));
            match (name, rest) {
                ("median", []) => qty(percentile(&mut xs, 50.0), u),
                ("percentile", [p]) if num(p).is_some_and(|p| (0.0..=100.0).contains(&p)) => qty(percentile(&mut xs, num(p).unwrap()), u),
                ("percentile", [_]) => return Err("p must be 0-100".into()),
                ("variance" | "stdev", []) if xs.len() < 2 => {
                    return Err("needs at least 2 items".into());
                }
                ("variance", []) => qty(variance(&xs), u.map(|u| u.pow(2))),
                ("stdev", []) => qty(variance(&xs).sqrt(), u),
                _ => return Err(Fail::BadArgs),
            }
        }
        ("mode", [List(l)]) => {
            // Keyed by display form, so 1 and 1.0 count together (1 m and 100 cm don't).
            let l = l.borrow();
            let mut counts: HashMap<String, usize> = HashMap::new();
            for v in l.iter() {
                *counts.entry(format!("{v:?}")).or_default() += 1;
            }
            let best = counts.values().max().ok_or("empty list")?;
            l.iter().find(|v| counts[&format!("{v:?}")] == *best).unwrap().clone()
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
        ("step", [List(l), Int(n, _)]) if *n > 0 => Value::list(l.borrow().iter().step_by(*n as usize).cloned().collect()),
        ("step", [List(_), Int(..)]) => return Err("step must be positive".into()),
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
        (BinOp::RangeIncl, Int(x, _), Int(y, _)) => y.checked_add(1).ok_or_else(|| "range too large".into()).and_then(|y| range(*x, y)),
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

/// Numbers, or quantities of one kind in the first item's unit, as f64s.
fn floats(l: &[Value]) -> Result<(Vec<f64>, Option<Unit>), String> {
    let unit = match l.first().ok_or("empty list")? {
        Value::Qty(_, u) => Some(u.clone()),
        _ => None,
    };
    let xs = l.iter().map(|v| match (v, &unit) {
        (Value::Qty(x, w), Some(u)) if w.dim() == u.dim() => Some(u.value_from_si(w.to_si(*x))),
        (_, None) => num(v),
        _ => None,
    });
    Ok((xs.collect::<Option<_>>().ok_or("expected numbers, or quantities of one kind")?, unit))
}

/// Linear interpolation between the closest ranks, like numpy's default.
fn percentile(xs: &mut [f64], p: f64) -> f64 {
    xs.sort_by(f64::total_cmp);
    let r = p / 100.0 * (xs.len() - 1) as f64;
    let (lo, hi) = (r.floor() as usize, r.ceil() as usize);
    xs[lo] + (xs[hi] - xs[lo]) * (r - lo as f64)
}

fn variance(xs: &[f64]) -> f64 {
    let mean = xs.iter().sum::<f64>() / xs.len() as f64;
    xs.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (xs.len() - 1) as f64
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
    use crate::interp::tests::{show, try_eval};

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

    #[test]
    fn stats() {
        assert_eq!(show("[[3, 1, 2].median, [4, 1, 3, 2].median, [1 m, 3 m].median, [1 m, 3 m, 50 cm, 2 m].median]"), "[2, 2.5, 2 m, 1.5 m]");
        assert_eq!(show("[[1, 2, 2, 3].mode, [1, 1.0, 2, 2].mode, \"hello\".chars.mode]"), "[2, 1, \"l\"]");
        assert_eq!(show("[[1, 2, 3, 4, 5].percentile(90), [5].percentile(50), (1..=100).percentile(0)]"), "[4.6, 5, 1]");
        assert_eq!(show("[[2, 4, 4, 4, 5, 5, 7, 9].variance, [1, 2, 3, 4].stdev.round(4)]"), "[4.57143, 1.291]");
        assert_eq!(show("[[1 m, 2 m, 3 m].stdev, [1 m, 200 cm, 3 m].variance]"), "[1 m, 1 m^2]");
        assert_eq!(show("[[2, 3, 4].product, [].product, [2 m, 3 m].product, (1..=25).product]"), "[24, 1, 6 m^2, 15511210043330985984000000]");
        for bad in ["[].median", "[1].stdev", "[1 m, 2].median", "[1, 2].percentile(101)", "[].mode"] {
            assert!(try_eval(bad).is_err(), "{bad}");
        }
    }
}
