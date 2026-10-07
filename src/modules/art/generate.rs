//! Generative art: L-system turtle drawings and named fractals, elementary cellular automata, the Ulam spiral, starfields.

use super::Dots;
use crate::interp::Interp;
use crate::lexer::Span;
use crate::modules::{Call, Doc, Fail, Module, doc};
use crate::value::{Value, num};
use std::collections::HashMap;

pub const MODULE: Module = Module {
    name: "generate",
    about: "L-systems and named fractals in braille, cellular automata like rule 30, the Ulam prime spiral, starfields",
    #[rustfmt::skip]
    examples: &[
        ("grow things", &[
            ("a dragon curve", r#"fractal("dragon", 8, 30)"#),
            ("a tree in a box", r#"fractal("tree", 6, 24).boxed("round")"#),
            ("Sierpinski, two ways", r#"rule(90, 32, 16).beside(fractal("sierpinski", 4, 16), 4)"#),
            ("invent a plant", r#"lsystem("X", {X: "F[-X][X]F[-X]+FX", F: "FF"}, 25, 4, 30)"#),
        ]),
    ],
    fns: FNS,
    call,
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("lsystem", "lsystem(axiom: str, rules: map, angle: num, n: int, width?: int)", "rewrite axiom n times with rules (one character → its replacement), then draw it with a turtle in braille, width characters wide (default 60). F and G draw a step, f steps without drawing, + and - turn by angle degrees, | turns around, [ and ] save and restore position; other characters do nothing. Starts facing up", &[r#"lsystem("F", {F: "F+F-F-F+F"}, 90, 3, 30)"#], &["fractal"]),
    doc("fractal", "fractal(name?: str, n?: int, width?: int)", "a named L-system fractal: \"dragon\", \"koch\", \"hilbert\", \"sierpinski\", \"levy\", \"gosper\", \"tree\", \"fern\" or \"bush\"; n is the depth, width as in lsystem. No name lists them", &[r#"fractal("koch", 3, 30)"#, "fractal()"], &["lsystem", "rule"]),
    doc("rule", "rule(n: int, width?: int, steps?: int)", "an elementary cellular automaton, 0-255, grown from one cell for steps rows (default width / 2) on a wrapping row of width cells (default 64); two rows per line", &["rule(30, 32)", "rule(90, 32, 16)"], &["life", "fractal"]),
    doc("spiral", "spiral(size?: int)", "an Ulam spiral: 1, 2, 3… wound out from the center with the primes as dots, which line up on diagonals; size characters wide (default 40)", &["spiral(20)"], &["is_prime"]),
    doc("starfield", "starfield(width: int, height: int, seed?: int)", "a random night sky; the same seed gives the same sky", &["starfield(30, 4, 7)"], &["negative"]),
];

fn call(_: &mut Interp, name: &'static str, args: &[Value], _: &Span) -> Call {
    use Value::*;
    let int = |i: usize, default: i64, lo: i64, hi: i64| match args.get(i) {
        None => Ok(default),
        Some(Int(n, _)) if (lo..=hi).contains(n) => Ok(*n),
        Some(_) => Err(Fail::Arg(i, format!("expected an int from {lo} to {hi}"))),
    };
    Ok(match (name, args) {
        ("lsystem", [Str(axiom), Map(m), angle, Int(..), rest @ ..]) if rest.len() <= 1 => {
            let angle = num(angle).ok_or(Fail::Arg(2, "expected a number of degrees".into()))?;
            let mut rules = HashMap::new();
            for (k, v) in m.borrow().iter() {
                let mut cs = k.chars();
                let (Some(c), None) = (cs.next(), cs.next()) else { return Err(Fail::Arg(1, format!("rule keys are single characters, got {k:?}"))) };
                let Str(v) = v else { return Err(Fail::Arg(1, format!("rule {k} must map to a string"))) };
                rules.insert(c, v.to_string());
            }
            let program = expand(axiom, &rules, int(3, 0, 0, 30)? as usize)?;
            Value::str(turtle(&program, angle, 90.0, int(4, 60, 4, 500)? as usize)?)
        }
        ("fractal", []) => Value::list(PRESETS.iter().map(|p| Value::str(p.0)).collect()),
        ("fractal", [Str(n), rest @ ..]) if rest.len() <= 2 => {
            let &(_, axiom, rules, angle, heading, depth) =
                PRESETS.iter().find(|p| p.0 == &**n).ok_or_else(|| Fail::Arg(0, format!("no fractal {n:?}; fractal() lists them")))?;
            let rules = rules.iter().map(|(c, r)| (*c, r.to_string())).collect();
            let program = expand(axiom, &rules, int(1, depth, 0, 30)? as usize)?;
            Value::str(turtle(&program, angle, heading, int(2, 60, 4, 500)? as usize)?)
        }
        ("rule", [Int(..), rest @ ..]) if rest.len() <= 2 => {
            let n = int(0, 0, 0, 255)? as u8;
            let w = int(1, 64, 3, 1000)? as usize;
            let steps = int(2, w as i64 / 2, 1, 1000)? as usize;
            let mut row = vec![false; w];
            row[w / 2] = true;
            let mut rows = vec![row.clone()];
            for _ in 1..steps {
                row = (0..w).map(|i| n >> ((row[(i + w - 1) % w] as u8) << 2 | (row[i] as u8) << 1 | row[(i + 1) % w] as u8) & 1 == 1).collect();
                rows.push(row.clone());
            }
            let lines = rows.chunks(2).map(|pair| {
                let line = (0..w).map(|i| match (pair[0][i], pair.get(1).is_some_and(|b| b[i])) {
                    (true, true) => '█',
                    (true, false) => '▀',
                    (false, true) => '▄',
                    _ => ' ',
                });
                line.collect::<String>().trim_end().to_string()
            });
            Value::str(lines.collect::<Vec<_>>().join("\n"))
        }
        ("spiral", [] | [Int(..)]) => {
            let d = int(0, 40, 2, 200)? as usize * 2;
            let n = d * d;
            let mut composite = vec![false; n + 1];
            composite[0] = true;
            composite[1] = true;
            for i in 2..=n.isqrt() {
                if !composite[i] {
                    (i * i..=n).step_by(i).for_each(|j| composite[j] = true);
                }
            }
            let mut dots = Dots::new(d, d);
            let (mut x, mut y, mut k) = ((d / 2) as i64, (d / 2) as i64, 1);
            let mut set = |x: i64, y: i64, k: usize| {
                if !composite[k] {
                    dots.set(x, y);
                }
            };
            set(x, y, 1);
            // Runs of 1, 1, 2, 2, 3, 3… going right, up, left, down.
            'walk: for run in 1.. {
                for (dx, dy) in [(1, 0), (0, -1), (-1, 0), (0, 1)].into_iter().skip((run - 1) % 2 * 2).take(2) {
                    for _ in 0..run {
                        (x, y, k) = (x + dx, y + dy, k + 1);
                        if k > n {
                            break 'walk;
                        }
                        set(x, y, k);
                    }
                }
            }
            Value::str(dots.render())
        }
        ("starfield", [Int(..), Int(..), rest @ ..]) if rest.len() <= 1 => {
            let (w, h) = (int(0, 0, 1, 1000)?, int(1, 0, 1, 1000)?);
            let mut rng = match rest {
                [] => fastrand::Rng::new(),
                [Int(s, _)] => fastrand::Rng::with_seed(*s as u64),
                _ => return Err(Fail::BadArgs),
            };
            let star = |r: f64| match r {
                r if r < 0.004 => '✦',
                r if r < 0.012 => '*',
                r if r < 0.02 => '+',
                r if r < 0.045 => '.',
                r if r < 0.07 => '·',
                _ => ' ',
            };
            let lines = (0..h).map(|_| (0..w).map(|_| star(rng.f64())).collect::<String>().trim_end().to_string());
            Value::str(lines.collect::<Vec<_>>().join("\n"))
        }
        _ => return Err(Fail::BadArgs),
    })
}

/// Rewrite every character with its rule n times.
fn expand(axiom: &str, rules: &HashMap<char, String>, n: usize) -> Result<String, Fail> {
    let mut s = axiom.to_string();
    for _ in 0..n {
        s = s.chars().map(|c| rules.get(&c).map_or_else(|| c.to_string(), Clone::clone)).collect();
        if s.len() > 2_000_000 {
            return Err("that grows past 2 million characters; lower n".into());
        }
    }
    Ok(s)
}

/// Walk the turtle program, then fit its path into a braille box `width` characters wide.
fn turtle(program: &str, angle: f64, heading: f64, width: usize) -> Result<String, Fail> {
    let (mut pos, mut dir) = ((0.0f64, 0.0f64), heading);
    let mut saved = vec![];
    let mut segs = vec![];
    for c in program.chars() {
        match c {
            'F' | 'G' | 'f' => {
                let next = (pos.0 + dir.to_radians().cos(), pos.1 + dir.to_radians().sin());
                if c != 'f' {
                    segs.push((pos, next));
                }
                pos = next;
            }
            '+' => dir += angle,
            '-' => dir -= angle,
            '|' => dir += 180.0,
            '[' => saved.push((pos, dir)),
            ']' => (pos, dir) = saved.pop().unwrap_or((pos, dir)),
            _ => {}
        }
    }
    if segs.is_empty() {
        return Err("nothing to draw: the program has no F or G".into());
    }
    let pts = segs.iter().flat_map(|(a, b)| [a, b]);
    let (x0, x1, y0, y1) = pts.fold((f64::MAX, f64::MIN, f64::MAX, f64::MIN), |(x0, x1, y0, y1), p| (x0.min(p.0), x1.max(p.0), y0.min(p.1), y1.max(p.1)));
    // Dots are about square; cap the height at as many dots as the width.
    let w = (width * 2) as f64;
    let scale = ((w - 1.0) / (x1 - x0).max(1e-9)).min((w - 1.0) / (y1 - y0).max(1e-9));
    let mut dots = Dots::new(((x1 - x0) * scale) as usize + 1, ((y1 - y0) * scale) as usize + 1);
    let at = |p: &(f64, f64)| ((p.0 - x0) * scale, (y1 - p.1) * scale);
    for (a, b) in &segs {
        dots.line(at(a), at(b));
    }
    Ok(dots.render())
}

type Preset = (&'static str, &'static str, &'static [(char, &'static str)], f64, f64, i64);

/// Name, axiom, rules, turn angle, starting heading (degrees, 0 = right, 90 = up), default depth.
#[rustfmt::skip]
const PRESETS: &[Preset] = &[
    ("dragon", "FX", &[('X', "X+YF+"), ('Y', "-FX-Y")], 90.0, 0.0, 10),
    ("koch", "F--F--F", &[('F', "F+F--F+F")], 60.0, 0.0, 3),
    ("hilbert", "A", &[('A', "+BF-AFA-FB+"), ('B', "-AF+BFB+FA-")], 90.0, 0.0, 5),
    ("sierpinski", "F-G-G", &[('F', "F-G+F+G-F"), ('G', "GG")], 120.0, 180.0, 5),
    ("levy", "F", &[('F', "+F--F+")], 45.0, 0.0, 10),
    ("gosper", "F", &[('F', "F-G--G+F++FF+G-"), ('G', "+F-FF--G-F+G+G")], 60.0, 0.0, 3),
    ("tree", "X", &[('X', "F[+X][-X]"), ('F', "FF")], 30.0, 90.0, 7),
    ("fern", "X", &[('X', "F+[[X]-X]-F[-FX]+X"), ('F', "FF")], 25.0, 70.0, 5),
    ("bush", "F", &[('F', "FF+[+F-F-F]-[-F+F+F]")], 22.5, 90.0, 4),
];

#[cfg(test)]
mod tests {
    use crate::interp::tests::{show, try_eval};

    #[test]
    fn generate() {
        assert_eq!(show(r#"lsystem("FF+F", {}, 90, 0, 4)"#).lines().count(), 2);
        assert_eq!(show(r#"lsystem("F", {}, 90, 0, 4)"#).lines().count(), 2);
        assert!(try_eval(r#"lsystem("+", {}, 90, 1)"#).is_err());
        assert!(try_eval(r#"lsystem("F", {FF: "F"}, 90, 1)"#).is_err());
        assert!(try_eval(r#"lsystem("F", {F: "FFFFFFFF"}, 90, 30)"#).is_err());
        for name in ["dragon", "koch", "hilbert", "sierpinski", "levy", "gosper", "tree", "fern", "bush"] {
            let art = show(&format!("fractal({name:?}, 3, 20)"));
            assert!(art.lines().all(|l| l.chars().count() <= 20), "{name} too wide:\n{art}");
            assert!(art.chars().any(|c| ('\u{2801}'..='\u{28ff}').contains(&c)), "{name} drew nothing");
        }
        assert!(try_eval(r#"fractal("blob")"#).is_err());
        // Rule 90 from one cell is Pascal's triangle mod 2.
        assert_eq!(show("rule(90, 7, 4)"), "  ▄▀▄\n▄▀▄ ▄▀▄");
        assert_eq!(show("rule(0, 5, 2)"), "  ▀");
        assert!(try_eval("rule(256)").is_err());
        // 1 sits at the center; 2 and 3 (primes) are right and up-right of it.
        let s = show("spiral(2)");
        assert_eq!(s.lines().count(), 1);
        assert_eq!(show("starfield(10, 3, 1)"), show("starfield(10, 3, 1)"));
        assert_eq!(show("starfield(10, 3, 1)").split('\n').count(), 3);
    }
}
