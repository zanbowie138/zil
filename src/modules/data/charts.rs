//! Text charts for the terminal: sparklines, bar charts, histograms, function plots.

use crate::interp::Interp;
use crate::lexer::Span;
use crate::modules::{Call, Doc, Fail, Module, doc};
use crate::value::{Value, fmt_float, num};

pub const MODULE: Module = Module {
    name: "charts",
    about: "sparklines, bar charts, histograms and braille function plots, as text",
    #[rustfmt::skip]
    examples: &[
        ("charts", &[
            ("a trend in one line", "[3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5].sparkline"),
            ("a budget", "bars({rent: $1800, food: $600, fun: $250})"),
            ("letter frequencies", r#""mississippi".chars.count_by(|c| c) |> bars"#),
            ("dice sums", "(1..=2000).map(|_| rand(1, 6) + rand(1, 6)).histogram(11)"),
            ("a sine wave", "plot(sin, 0, 2 * pi)"),
        ]),
    ],
    fns: FNS,
    call,
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("sparkline", "sparkline(xs: list)", "a one-line chart: one block character per number, scaled min to max", &["[1, 5, 2, 8, 3].sparkline", "(0..20).map(|x| sin(x / 3)).sparkline"], &["bars", "plot"]),
    doc("bars", "bars(m: map|list, width?: int)", "a horizontal bar chart of a map (labels → numbers) or a list; width defaults to 40", &["bars({a: 3, b: 7, c: 5})", "[2, 4, 8].bars(16)"], &["histogram", "sparkline"]),
    doc("histogram", "histogram(xs: list, bins?: int)", "count numbers into equal-width bins (default 10) and draw them as bars", &["[1, 2, 2, 3, 3, 3, 4, 4, 5].histogram(5)"], &["bars", "count_by"]),
    doc("plot", "plot(f: fn, a: num, b: num)", "plot f from a to b in braille dots, 60 columns by 16 rows", &["plot(|x| x ** 2, -1, 1)"], &["sparkline"]),
];

const BLOCKS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

/// A number, or a quantity's number in its own unit.
fn f(v: &Value) -> Option<f64> {
    match v {
        Value::Qty(x, _) => Some(*x),
        v => num(v),
    }
}

fn floats(l: &[Value]) -> Result<Vec<f64>, Fail> {
    l.iter().map(f).collect::<Option<_>>().ok_or_else(|| "expected a list of numbers".into())
}

fn call(it: &mut Interp, name: &'static str, args: &[Value], span: &Span) -> Call {
    use Value::*;
    Ok(match (name, args) {
        ("sparkline", [List(l)]) => {
            let xs = floats(&l.borrow())?;
            let (lo, hi) = xs.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), x| (lo.min(*x), hi.max(*x)));
            let level = |x: f64| if hi > lo { ((x - lo) / (hi - lo) * 7.0).round() as usize } else { 3 };
            Value::str(xs.iter().map(|x| BLOCKS[level(*x)]).collect::<String>())
        }
        ("bars", [data, rest @ ..]) if rest.len() <= 1 => {
            let width = match rest {
                [] => 40,
                [Int(w, _)] if (1..=500).contains(w) => *w as usize,
                _ => return Err(Fail::Arg(1, "width must be an int from 1 to 500".into())),
            };
            let rows: Vec<(String, Value)> = match data {
                Map(m) => m.borrow().iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
                List(l) => l.borrow().iter().enumerate().map(|(i, v)| (i.to_string(), v.clone())).collect(),
                _ => return Err(Fail::BadArgs),
            };
            Value::str(bars(&rows, width)?)
        }
        ("histogram", [List(l), rest @ ..]) if rest.len() <= 1 => {
            let bins = match rest {
                [] => 10,
                [Int(b, _)] if (1..=200).contains(b) => *b as usize,
                _ => return Err(Fail::Arg(1, "bins must be an int from 1 to 200".into())),
            };
            let xs = floats(&l.borrow())?;
            if xs.is_empty() {
                return Err("empty list".into());
            }
            let (lo, hi) = xs.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), x| (lo.min(*x), hi.max(*x)));
            let step = if hi > lo { (hi - lo) / bins as f64 } else { 1.0 };
            let mut counts = vec![0i64; bins];
            for x in xs {
                counts[(((x - lo) / step) as usize).min(bins - 1)] += 1;
            }
            let rows: Vec<_> = counts
                .iter()
                .enumerate()
                .map(|(i, c)| (format!("{}–{}", fmt_float(lo + step * i as f64), fmt_float(lo + step * (i + 1) as f64)), Value::int(*c)))
                .collect();
            Value::str(bars(&rows, 40)?)
        }
        ("plot", [func, a, b]) => {
            let (a, b) = (num(a).ok_or(Fail::Arg(1, "expected a number".into()))?, num(b).ok_or(Fail::Arg(2, "expected a number".into()))?);
            const W: usize = 60 * 2;
            const H: usize = 16 * 4;
            let mut ys = Vec::with_capacity(W);
            for i in 0..W {
                let x = a + (b - a) * i as f64 / (W - 1) as f64;
                let y = it.call(func, vec![Float(x)], span)?;
                ys.push(f(&y).ok_or_else(|| format!("f must return a number, got {}", y.type_name()))?);
            }
            let finite = ys.iter().copied().filter(|y| y.is_finite());
            let (lo, hi) = finite.fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), y| (lo.min(y), hi.max(y)));
            if !lo.is_finite() {
                return Err("f has no finite values on that range".into());
            }
            let mut grid = vec![[0u8; W / 2]; H / 4];
            for (i, y) in ys.iter().enumerate().filter(|(_, y)| y.is_finite()) {
                let row = if hi > lo { ((hi - y) / (hi - lo) * (H - 1) as f64).round() as usize } else { H / 2 };
                const BIT: [[u8; 4]; 2] = [[0x01, 0x02, 0x04, 0x40], [0x08, 0x10, 0x20, 0x80]];
                grid[row / 4][i / 2] |= BIT[i % 2][row % 4];
            }
            let (top, bottom) = (fmt_float(hi), fmt_float(lo));
            let pad = top.len().max(bottom.len());
            let mut out: Vec<String> = grid
                .iter()
                .enumerate()
                .map(|(r, cells)| {
                    let label = if r == 0 {
                        &top
                    } else if r == grid.len() - 1 {
                        &bottom
                    } else {
                        ""
                    };
                    format!("{label:>pad$} ┤{}", cells.iter().map(|b| char::from_u32(0x2800 + *b as u32).unwrap()).collect::<String>())
                })
                .collect();
            let (left, right) = (fmt_float(a), fmt_float(b));
            out.push(format!("{:pad$}  {left}{right:>w$}", "", w = W / 2 - left.chars().count()));
            Value::str(out.join("\n"))
        }
        _ => return Err(Fail::BadArgs),
    })
}

/// `label │█████▌ value` rows, the longest bar `width` cells; eighth-blocks for the remainder.
fn bars(rows: &[(String, Value)], width: usize) -> Result<String, Fail> {
    let xs = rows.iter().map(|r| f(&r.1).ok_or_else(|| format!("expected numbers, got {}", r.1.type_name()))).collect::<Result<Vec<_>, _>>()?;
    let max = xs.iter().fold(0f64, |m, x| m.max(x.abs()));
    let label_w = rows.iter().map(|r| r.0.chars().count()).max().unwrap_or(0);
    let lines = rows.iter().zip(&xs).map(|((label, v), x)| {
        let eighths = if max > 0.0 { (x.abs() / max * width as f64 * 8.0).round() as usize } else { 0 };
        let bar = "█".repeat(eighths / 8) + ["", "▏", "▎", "▍", "▌", "▋", "▊", "▉"][eighths % 8];
        format!("{label:>label_w$} │{bar} {v}")
    });
    Ok(lines.collect::<Vec<_>>().join("\n"))
}

#[cfg(test)]
mod tests {
    use crate::interp::tests::{show, try_eval};

    #[test]
    fn charts() {
        assert_eq!(show("[1, 2, 3, 4, 5, 6, 7, 8].sparkline"), "▁▂▃▄▅▆▇█");
        assert_eq!(show("[5, 5].sparkline"), "▄▄");
        assert_eq!(show("bars({a: 2, bb: 4}, 4)"), " a │██ 2\nbb │████ 4");
        assert_eq!(show("[1, 3].bars(8)"), "0 │██▋ 1\n1 │████████ 3");
        assert_eq!(show("[1, 1, 2].histogram(2)"), format!("1–1.5 │{} 2\n1.5–2 │{} 1", "█".repeat(40), "█".repeat(20)));
        let p = show("plot(|x| x, 0, 1)");
        assert_eq!(p.lines().count(), 17);
        assert!(p.starts_with("1 ┤") && p.lines().nth(15).unwrap().starts_with("0 ┤"));
        assert!(try_eval(r#"["a"].sparkline"#).is_err());
        assert!(try_eval(r#"plot(|x| "a", 0, 1)"#).is_err());
    }
}
