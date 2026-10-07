//! Statistics over lists: sums, means, spread, extremes.

use crate::ast::BinOp;
use crate::interp::{Interp, binary as op};
use crate::lexer::Span;
use crate::modules::units::Unit;
use crate::modules::{Call, Doc, Fail, Module, doc};
use crate::value::{Value, compare, num};
use std::collections::HashMap;

pub const MODULE: Module = Module {
    name: "stats",
    about: "sums, averages, spread and extremes of lists; units welcome",
    #[rustfmt::skip]
    examples: &[
        ("stats", &[
            ("standard deviation", "[2, 4, 4, 4, 5, 5, 7, 9].stdev"),
            ("percentile", "(1..=100).percentile(90)"),
        ]),
    ],
    fns: FNS,
    #[rustfmt::skip]
    groups: &[
        ("aggregate", &["sum", "product", "avg", "min", "max"]),
        ("spread", &["median", "mode", "percentile", "variance", "stdev"]),
    ],
    call,
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("sum", "sum(xs: list)", "add up a list; works with units", &["[1, 2, 3].sum", "[1 m, 50 cm].sum"], &["avg", "reduce"]),
    doc("avg", "avg(xs: list)", "mean of a list", &["[1, 2, 4].avg", "[2 h, 30 min].avg"], &["sum", "median"]),
    doc("product", "product(xs: list)", "multiply a list together", &["[2, 3, 4].product", "[2 m, 3 m].product"], &["sum", "factorial"]),
    doc("median", "median(xs: list)", "middle value; mean of the middle two for an even count", &["[3, 1, 2].median", "[1 m, 3 m, 50 cm, 2 m].median"], &["avg", "percentile"]),
    doc("mode", "mode(xs: list)", "most common item; the first one on ties", &["[1, 2, 2, 3].mode", r#""hello".chars.mode"#], &["median", "count"]),
    doc("percentile", "percentile(xs: list, p: num)", "the p-th percentile (0-100), interpolating between items", &["[1, 2, 3, 4, 5].percentile(90)", "(1..=100).percentile(25)"], &["median"]),
    doc("variance", "variance(xs: list)", "sample variance (n - 1)", &["[2, 4, 4, 4, 5, 5, 7, 9].variance"], &["stdev"]),
    doc("stdev", "stdev(xs: list)", "sample standard deviation (n - 1); works with units", &["[2, 4, 4, 4, 5, 5, 7, 9].stdev", "[1 m, 2 m, 3 m].stdev"], &["variance", "avg"]),
    doc("min", "min(xs: list) / min(a: any, b: any, ...)", "smallest value", &["min(3, 9, 4)", "[2 km, 1 mi].min"], &["max", "sort"]),
    doc("max", "max(xs: list) / max(a: any, b: any, ...)", "largest value", &["max(3, 9, 4)", r#"["b", "a"].max"#], &["min", "sort"]),
];

fn call(_: &mut Interp, name: &'static str, args: &[Value], _: &Span) -> Call {
    use Value::*;
    Ok(match (name, args) {
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
                ("percentile", [p]) => return Err(Fail::Arg(1, format!("p must be from 0 to 100, got {p:?}"))),
                ("variance" | "stdev", []) if xs.len() < 2 => {
                    return Err(format!("needs at least 2 items, got {}", xs.len()).into());
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
        _ => return Err(Fail::BadArgs),
    })
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
    Ok((xs.collect::<Option<_>>().ok_or("expected a list of numbers, or of quantities of one kind")?, unit))
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
                let ord = compare(v, b).ok_or_else(|| format!("cannot compare {} and {}", crate::modules::short(v), crate::modules::short(b)))?;
                if (name == "min" && ord.is_lt()) || (name == "max" && ord.is_gt()) { v } else { b }
            }
        });
    }
    best.cloned().ok_or_else(|| "empty list".into())
}

#[cfg(test)]
mod tests {
    use crate::interp::tests::{show, try_eval};

    #[test]
    fn stats() {
        assert_eq!(show("[1, 2, 3, 4].avg"), "2.5");
        assert_eq!(show("[1 m, 50 cm].sum"), "1.5 m");
        assert_eq!(show("max(3, 9, 4)"), "9");
        assert_eq!(show("[1 km, 900 m].min"), "900 m");
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
