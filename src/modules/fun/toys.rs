//! Terminal toys: the Mandelbrot set, Conway's Game of Life, random mazes.

use crate::interp::Interp;
use crate::lexer::Span;
use crate::modules::{Call, Doc, Fail, Module, doc};
use crate::value::Value;
use std::collections::HashSet;

pub const MODULE: Module = Module {
    name: "toys",
    about: "ASCII Mandelbrot set, Conway's Game of Life, random mazes",
    #[rustfmt::skip]
    examples: &[
        ("toys", &[
            ("the Mandelbrot set", "mandelbrot()"),
            ("a glider, 8 steps on", r#"life("glider", 8)"#),
            ("draw your own", r#"life(".#.\n.#.\n.#.", 1)"#),
            ("a maze", "maze(12, 6)"),
        ]),
    ],
    fns: FNS,
    call,
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("mandelbrot", "mandelbrot(width?: int, height?: int)", "the Mandelbrot set in ASCII, default 72×24", &["mandelbrot(40, 12)"], &["life"]),
    doc("life", "life(start: str, steps?: int)", "run Conway's Game of Life for steps (default 1) and draw the live cells; start is rows of # and . or one of \"glider\", \"blinker\", \"pulsar\", \"r_pentomino\"", &[r#"life("blinker")"#, r#"life("glider", 4)"#], &["maze"]),
    doc("maze", "maze(width?: int, height?: int)", "a random perfect maze (one path between any two cells), default 20×10 cells", &["maze(6, 3)"], &["life"]),
];

fn call(_: &mut Interp, name: &'static str, args: &[Value], _: &Span) -> Call {
    use Value::*;
    let size = |i: usize, default: i64, max: i64| match args.get(i) {
        None => Ok(default as usize),
        Some(Int(n, _)) if (1..=max).contains(n) => Ok(*n as usize),
        Some(_) => Err(Fail::Arg(i, format!("expected an int from 1 to {max}"))),
    };
    Ok(match (name, args) {
        ("mandelbrot", [] | [Int(..)] | [Int(..), Int(..)]) => {
            let (w, h) = (size(0, 72, 500)?, size(1, 24, 500)?);
            const SHADES: &[u8] = b" .:-=+*#%@";
            let rows: Vec<String> = (0..h)
                .map(|y| {
                    (0..w)
                        .map(|x| {
                            let c = (-2.2 + 3.0 * x as f64 / w as f64, -1.2 + 2.4 * y as f64 / h as f64);
                            let (mut zr, mut zi, mut i) = (0.0f64, 0.0f64, 0);
                            while i < 50 && zr * zr + zi * zi <= 4.0 {
                                (zr, zi) = (zr * zr - zi * zi + c.0, 2.0 * zr * zi + c.1);
                                i += 1;
                            }
                            if i == 50 { '@' } else { SHADES[i * (SHADES.len() - 1) / 50] as char }
                        })
                        .collect::<String>()
                        .trim_end()
                        .to_string()
                })
                .collect();
            Value::str(rows.join("\n"))
        }
        ("life", [Str(start), rest @ ..]) if rest.len() <= 1 => {
            let steps = match rest {
                [] => 1,
                [Int(n, _)] if (0..=10_000).contains(n) => *n,
                _ => return Err(Fail::Arg(1, "steps must be an int from 0 to 10000".into())),
            };
            let pattern = PATTERNS.iter().find(|p| p.0 == &**start).map_or(&**start, |p| p.1);
            let mut live: HashSet<(i64, i64)> = HashSet::new();
            for (y, line) in pattern.lines().enumerate() {
                for (x, c) in line.chars().enumerate() {
                    match c {
                        '#' | '█' | 'O' | 'o' | '*' => _ = live.insert((x as i64, y as i64)),
                        '.' | ' ' | '_' => {}
                        _ => return Err(Fail::Arg(0, format!("unknown cell {c:?}: use # for alive and . for dead, or a pattern name"))),
                    }
                }
            }
            for _ in 0..steps {
                live = step(&live);
                if live.len() > 100_000 {
                    return Err("too many live cells".into());
                }
            }
            Value::str(draw(&live))
        }
        ("maze", [] | [Int(..)] | [Int(..), Int(..)]) => {
            let (w, h) = (size(0, 20, 200)?, size(1, 10, 200)?);
            // Grid of walls; cell (x, y) sits at (2x+1, 2y+1). Carve with an iterative depth-first search.
            let (gw, gh) = (2 * w + 1, 2 * h + 1);
            let mut open = vec![vec![false; gw]; gh];
            let mut stack = vec![(0usize, 0usize)];
            open[1][1] = true;
            while let Some(&(x, y)) = stack.last() {
                let mut next: Vec<(usize, usize)> = [(0, -1), (1, 0), (0, 1), (-1, 0)]
                    .iter()
                    .map(|(dx, dy)| (x as i64 + dx, y as i64 + dy))
                    .filter(|(nx, ny)| (0..w as i64).contains(nx) && (0..h as i64).contains(ny))
                    .map(|(nx, ny)| (nx as usize, ny as usize))
                    .filter(|(nx, ny)| !open[2 * ny + 1][2 * nx + 1])
                    .collect();
                if next.is_empty() {
                    stack.pop();
                    continue;
                }
                fastrand::shuffle(&mut next);
                let (nx, ny) = next[0];
                open[y + ny + 1][x + nx + 1] = true;
                open[2 * ny + 1][2 * nx + 1] = true;
                stack.push((nx, ny));
            }
            // Entrance top-left, exit bottom-right.
            open[0][1] = true;
            open[gh - 1][gw - 2] = true;
            Value::str(open.iter().map(|row| row.iter().map(|o| if *o { "  " } else { "██" }).collect::<String>()).collect::<Vec<_>>().join("\n"))
        }
        _ => return Err(Fail::BadArgs),
    })
}

fn step(live: &HashSet<(i64, i64)>) -> HashSet<(i64, i64)> {
    let mut counts = std::collections::HashMap::new();
    for (x, y) in live {
        for dx in -1..=1 {
            for dy in -1..=1 {
                if (dx, dy) != (0, 0) {
                    *counts.entry((x + dx, y + dy)).or_insert(0) += 1;
                }
            }
        }
    }
    counts.into_iter().filter(|(c, n)| *n == 3 || (*n == 2 && live.contains(c))).map(|(c, _)| c).collect()
}

/// The live cells' bounding box as rows of # and .; "(empty)" when everything died.
fn draw(live: &HashSet<(i64, i64)>) -> String {
    if live.is_empty() {
        return "(empty)".into();
    }
    let (x0, x1) = (live.iter().map(|c| c.0).min().unwrap(), live.iter().map(|c| c.0).max().unwrap());
    let (y0, y1) = (live.iter().map(|c| c.1).min().unwrap(), live.iter().map(|c| c.1).max().unwrap());
    (y0..=y1).map(|y| (x0..=x1).map(|x| if live.contains(&(x, y)) { '#' } else { '.' }).collect::<String>()).collect::<Vec<_>>().join("\n")
}

const PATTERNS: &[(&str, &str)] = &[
    ("glider", ".#.\n..#\n###"),
    ("blinker", "###"),
    ("r_pentomino", ".##\n##.\n.#."),
    (
        "pulsar",
        "..###...###..\n.............\n#....#.#....#\n#....#.#....#\n#....#.#....#\n..###...###..\n.............\n..###...###..\n#....#.#....#\n#....#.#....#\n#....#.#....#\n.............\n..###...###..",
    ),
];

#[cfg(test)]
mod tests {
    use crate::interp::tests::{show, try_eval};

    #[test]
    fn toys() {
        let m = show("mandelbrot(40, 12)");
        assert_eq!(m.lines().count(), 12);
        assert!(m.contains('@'));
        assert_eq!(show(r#"life("blinker")"#), "#\n#\n#");
        assert_eq!(show(r#"life("blinker", 2)"#), "###");
        // A glider is itself again after 4 steps, one cell down and right.
        assert_eq!(show(r#"life("glider", 4)"#), show(r#"life("glider", 0)"#));
        assert_eq!(show(r#"life("pulsar", 3)"#), show(r#"life("pulsar", 0)"#));
        assert_eq!(show(r##"life("#", 1)"##), "(empty)");
        let mz = show("maze(5, 3)");
        assert_eq!(mz.lines().count(), 7);
        assert!(mz.lines().all(|l| l.chars().count() == 22));
        // A perfect maze on 15 cells has 14 passages: open cells = 15 + 14 + 2 doors.
        let open: usize = mz.lines().map(|l| l.chars().collect::<Vec<_>>().chunks(2).filter(|c| c[0] == ' ').count()).sum();
        assert_eq!(open, 15 + 14 + 2);
        assert!(try_eval(r#"life("x")"#).is_err());
        assert!(try_eval("maze(0)").is_err());
    }
}
