//! Data drawn as pictures: trees, heatmaps, progress bars, QR codes, chess boards, playing cards, dice, seven-segment digits.

use crate::interp::Interp;
use crate::lexer::Span;
use crate::modules::{Call, Doc, Fail, Module, doc};
use crate::value::{Value, num};

pub const MODULE: Module = Module {
    name: "renderers",
    about: "nested data as a tree, heatmaps, progress bars, QR codes, chess boards from FEN, playing cards, dice faces, seven-segment digits",
    #[rustfmt::skip]
    examples: &[
        ("draw data", &[
            ("a config at a glance", r#"tree_view({server: {host: "localhost", ports: [80, 443]}, debug: false})"#),
            ("a multiplication table", "(1..=6).map(|r| (1..=10).map(|c| r * c)).heatmap"),
            ("a link for your phone", r#"qr("https://github.com/zanbowie138/zil")"#),
            ("what you rolled", r#"dice(roll("3d6").rolls)"#),
            ("a royal flush", r#"cards(["10♠", "J♠", "Q♠", "K♠", "A♠"])"#),
            ("a big clock", r#"segments(now.format("%H:%M"))"#),
        ]),
    ],
    fns: FNS,
    call,
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("tree_view", "tree_view(v: map|list)", "nested maps and lists drawn like the `tree` command", &[r#"tree_view({src: {main: "rs", lib: "rs"}, docs: ["book"]})"#], &["keys", "from_json"]),
    doc("heatmap", "heatmap(rows: list)", "a list of rows of numbers as shades from ░ (low) to █ (high)", &["[[1, 2, 3], [4, 5, 6], [7, 8, 9]].heatmap"], &["bars", "sparkline"]),
    doc("progress", "progress(frac: num, width?: int)", "a progress bar for frac from 0 to 1, width cells wide (default 30)", &["progress(0.42)", "progress(2 / 3, 12)"], &["bars"]),
    doc("qr", "qr(text: str, ec?: str)", "a scannable QR code in half blocks, light on dark; ec is the error correction level \"L\", \"M\" (default), \"Q\" or \"H\"", &[r#"qr("zil")"#], &["url_build"]),
    doc("chess", "chess(fen?: str)", "a chess board from the piece part of a FEN string; the starting position by default", &["chess()", r#"chess("8/8/8/4k3/8/8/4P3/4K3 w - - 0 1")"#], &["card"]),
    doc("card", "card(c: str)", "a playing card like \"A♠\", \"10h\" or \"qd\" (suits ♠♥♦♣ or s h d c); \"?\" is the back", &[r#"card("Q♥")"#], &["cards", "dice"]),
    doc("cards", "cards(hand: list)", "playing cards side by side", &[r#"cards(["As", "Kh", "?"])"#], &["card"]),
    doc("dice", "dice(faces: int|list)", "die faces with pips, 1-6, side by side for a list", &["dice(5)", "dice([1, 2, 3])"], &["roll", "card"]),
    doc("segments", "segments(s: str|int)", "big seven-segment digits for 0-9, A-F, - . : and spaces", &[r#"segments("12:45")"#, r#"segments("C0FFEE")"#], &["banner", "clock"]),
];

fn call(_: &mut Interp, name: &'static str, args: &[Value], _: &Span) -> Call {
    use Value::*;
    Ok(match (name, args) {
        ("tree_view", [v @ (Map(_) | List(_))]) => {
            let mut out = vec![];
            tree(v, "", &mut out);
            Value::str(out.join("\n"))
        }
        ("heatmap", [List(rows)]) => {
            let grid = rows
                .borrow()
                .iter()
                .map(|r| match r {
                    List(r) => r.borrow().iter().map(num).collect::<Option<Vec<f64>>>(),
                    _ => None,
                })
                .collect::<Option<Vec<_>>>()
                .ok_or(Fail::Arg(0, "expected a list of lists of numbers".into()))?;
            let all = grid.iter().flatten().copied();
            let (lo, hi) = all.fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), x| (lo.min(x), hi.max(x)));
            const SHADES: [&str; 5] = ["  ", "░░", "▒▒", "▓▓", "██"];
            let shade = |x: f64| if hi > lo { SHADES[((x - lo) / (hi - lo) * 4.0).round() as usize] } else { SHADES[2] };
            Value::str(grid.iter().map(|r| r.iter().map(|x| shade(*x)).collect::<String>().trim_end().to_string()).collect::<Vec<_>>().join("\n"))
        }
        ("progress", [f, rest @ ..]) if rest.len() <= 1 => {
            let f = num(f).ok_or(Fail::Arg(0, "expected a number from 0 to 1".into()))?.clamp(0.0, 1.0);
            let w = match rest {
                [] => 30,
                [Int(w, _)] if (1..=500).contains(w) => *w as usize,
                _ => return Err(Fail::Arg(1, "width must be an int from 1 to 500".into())),
            };
            let eighths = (f * w as f64 * 8.0).round() as usize;
            let bar = "█".repeat(eighths / 8) + ["", "▏", "▎", "▍", "▌", "▋", "▊", "▉"][eighths % 8];
            let pad = w - bar.chars().count();
            Value::str(format!("▕{bar}{}▏ {}%", " ".repeat(pad), (f * 100.0).round()))
        }
        ("qr", [Str(s), rest @ ..]) if rest.len() <= 1 => {
            use qrcode::{EcLevel, QrCode, render::unicode::Dense1x2};
            let ec = match rest {
                [] => EcLevel::M,
                [Str(e)] => match e.to_uppercase().as_str() {
                    "L" => EcLevel::L,
                    "M" => EcLevel::M,
                    "Q" => EcLevel::Q,
                    "H" => EcLevel::H,
                    _ => return Err(Fail::Arg(1, "ec is \"L\", \"M\", \"Q\" or \"H\"".into())),
                },
                _ => return Err(Fail::BadArgs),
            };
            let code = QrCode::with_error_correction_level(s.as_bytes(), ec).map_err(|e| Fail::Arg(0, format!("can't encode: {e}")))?;
            // Swapped colors: dark modules print as background, so the code reads right on a dark terminal.
            Value::str(code.render::<Dense1x2>().dark_color(Dense1x2::Light).light_color(Dense1x2::Dark).build())
        }
        ("chess", []) => Value::str(chess("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR")?),
        ("chess", [Str(fen)]) => Value::str(chess(fen.split_whitespace().next().unwrap_or("")).map_err(|e| Fail::Arg(0, e))?),
        ("card", [Str(c)]) => Value::str(card(c).map_err(|e| Fail::Arg(0, e))?.join("\n")),
        ("cards", [List(l)]) => {
            let faces = l
                .borrow()
                .iter()
                .map(|c| match c {
                    Str(c) => card(c),
                    v => Err(format!("cards are strings like \"A♠\", got {}", v.type_name())),
                })
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| Fail::Arg(0, e))?;
            Value::str(side_by_side(&faces))
        }
        ("dice", [Int(..)]) => Value::str(side_by_side(&[die(&args[0])?])),
        ("dice", [List(l)]) => Value::str(side_by_side(&l.borrow().iter().map(die).collect::<Result<Vec<_>, _>>()?)),
        ("segments", [v @ (Str(_) | Int(..))]) => {
            let mut rows = [String::new(), String::new(), String::new()];
            for c in v.to_string().chars() {
                let glyph: [String; 3] = match c {
                    ':' => [" ".into(), "•".into(), "•".into()],
                    '.' => [" ".into(), " ".into(), "▄".into()],
                    _ => {
                        let m = SEGMENTS.iter().find(|s| s.0 == c.to_ascii_uppercase()).ok_or_else(|| Fail::Arg(0, format!("no segment digit for {c:?}")))?.1;
                        let on = |bit: u8, s: &'static str, off: &'static str| if m & bit != 0 { s } else { off };
                        // Bits: a top, b top right, c bottom right, d bottom, e bottom left, f top left, g middle.
                        [
                            format!(" {} ", on(1, "▄▄", "  ")),
                            format!("{}{}{}", on(32, "█", " "), on(64, "▄▄", "  "), on(2, "█", " ")),
                            format!("{}{}{}", on(16, "█", " "), on(8, "▄▄", "  "), on(4, "█", " ")),
                        ]
                    }
                };
                for (row, g) in rows.iter_mut().zip(glyph) {
                    *row += &g;
                    row.push(' ');
                }
            }
            Value::str(rows.iter().map(|r| r.trim_end()).collect::<Vec<_>>().join("\n"))
        }
        _ => return Err(Fail::BadArgs),
    })
}

fn tree(v: &Value, prefix: &str, out: &mut Vec<String>) {
    let items: Vec<(String, Value)> = match v {
        Value::Map(m) => m.borrow().iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        Value::List(l) => l.borrow().iter().enumerate().map(|(i, v)| (format!("[{i}]"), v.clone())).collect(),
        _ => return,
    };
    let is_list = matches!(v, Value::List(_));
    for (i, (k, v)) in items.iter().enumerate() {
        let last = i == items.len() - 1;
        let nested = match v {
            Value::Map(m) => !m.borrow().is_empty(),
            Value::List(l) => !l.borrow().is_empty(),
            _ => false,
        };
        let label = match (nested, is_list) {
            (true, _) => k.clone(),
            (false, true) => v.to_string(),
            (false, false) => format!("{k}: {v}"),
        };
        out.push(format!("{prefix}{}{label}", if last { "└── " } else { "├── " }));
        if nested {
            tree(v, &format!("{prefix}{}", if last { "    " } else { "│   " }), out);
        }
    }
}

fn chess(board: &str) -> Result<String, String> {
    let ranks: Vec<&str> = board.split('/').collect();
    if ranks.len() != 8 {
        return Err(format!("a board has 8 ranks separated by /, got {}", ranks.len()));
    }
    let mut out = vec![];
    for (r, rank) in ranks.iter().enumerate() {
        let mut row = vec![];
        for c in rank.chars() {
            match c {
                '1'..='8' => row.extend(std::iter::repeat_n(None, c as usize - '0' as usize)),
                _ => row.push(Some(PIECES.iter().find(|p| p.0 == c).ok_or(format!("unknown piece {c:?}"))?.1)),
            }
        }
        if row.len() != 8 {
            return Err(format!("rank {} has {} squares, not 8", 8 - r, row.len()));
        }
        // a1 is dark: dark squares are where file + rank (both from 0 at a1) is even.
        let squares = row.iter().enumerate().map(|(f, p)| p.unwrap_or(if (f + 7 - r) % 2 == 0 { '·' } else { ' ' }).to_string());
        out.push(format!("{} {}", 8 - r, squares.collect::<Vec<_>>().join(" ")));
    }
    out.push("  a b c d e f g h".into());
    Ok(out.join("\n"))
}

fn card(c: &str) -> Result<Vec<String>, String> {
    if c.trim() == "?" {
        return Ok(["┌─────┐", "│░░░░░│", "│░░░░░│", "│░░░░░│", "└─────┘"].map(String::from).to_vec());
    }
    let c = c.trim();
    let suit = c.chars().last().ok_or("empty card")?;
    let rank = c[..c.len() - suit.len_utf8()].to_uppercase();
    let suit = match suit.to_ascii_lowercase() {
        's' | '♠' | '♤' => '♠',
        'h' | '♥' | '♡' => '♥',
        'd' | '♦' | '♢' => '♦',
        'c' | '♣' | '♧' => '♣',
        _ => return Err(format!("unknown suit {suit:?}; use ♠♥♦♣ or s h d c")),
    };
    let rank = match rank.as_str() {
        "T" => "10".to_string(),
        "A" | "J" | "Q" | "K" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "10" => rank,
        _ => return Err(format!("unknown rank {rank:?}; use A, 2-10, J, Q or K")),
    };
    Ok(vec!["┌─────┐".into(), format!("│{rank:<5}│"), format!("│  {suit}  │"), format!("│{rank:>5}│"), "└─────┘".into()])
}

fn die(v: &Value) -> Result<Vec<String>, Fail> {
    let n = match v {
        Value::Int(n @ 1..=6, _) => *n as usize,
        _ => return Err(Fail::Arg(0, format!("die faces are 1 to 6, got {v}"))),
    };
    // Pip slots in reading order: top left, top right, middle left, center, middle right, bottom left, bottom right.
    let pips: &[usize] = [&[3][..], &[1, 5], &[0, 3, 6], &[0, 1, 5, 6], &[0, 1, 3, 5, 6], &[0, 1, 2, 4, 5, 6]][n - 1];
    let p = |i: usize| if pips.contains(&i) { '●' } else { ' ' };
    Ok(vec![
        "┌───────┐".into(),
        format!("│ {}   {} │", p(0), p(1)),
        format!("│ {} {} {} │", p(2), p(3), p(4)),
        format!("│ {}   {} │", p(5), p(6)),
        "└───────┘".into(),
    ])
}

/// Equal-height blocks of lines joined with one space between.
fn side_by_side(blocks: &[Vec<String>]) -> String {
    let h = blocks.first().map_or(0, Vec::len);
    (0..h).map(|r| blocks.iter().map(|b| b[r].as_str()).collect::<Vec<_>>().join(" ")).collect::<Vec<_>>().join("\n")
}

const PIECES: &[(char, char)] =
    &[('K', '♔'), ('Q', '♕'), ('R', '♖'), ('B', '♗'), ('N', '♘'), ('P', '♙'), ('k', '♚'), ('q', '♛'), ('r', '♜'), ('b', '♝'), ('n', '♞'), ('p', '♟')];

#[rustfmt::skip]
const SEGMENTS: &[(char, u8)] = &[
    ('0', 63), ('1', 6), ('2', 91), ('3', 79), ('4', 102), ('5', 109), ('6', 125), ('7', 7), ('8', 127), ('9', 111),
    ('A', 119), ('B', 124), ('C', 57), ('D', 94), ('E', 121), ('F', 113), ('-', 64), (' ', 0),
];

#[cfg(test)]
mod tests {
    use crate::interp::tests::{show, try_eval};

    #[test]
    fn renderers() {
        assert_eq!(show(r#"tree_view({a: {b: 1, c: [2, 3]}, d: "x"})"#), "├── a\n│   ├── b: 1\n│   └── c\n│       ├── 2\n│       └── 3\n└── d: x");
        assert_eq!(show("[[0, 1], [2, 4]].heatmap"), "  ░░\n▒▒██");
        assert!(try_eval("[1, 2].heatmap").is_err());
        assert_eq!(show("progress(0.5, 4)"), "▕██  ▏ 50%");
        assert_eq!(show("progress(2, 2)"), "▕██▏ 100%");
        let q = show(r#"qr("hi")"#);
        // Version 1 is 21 modules plus a 4-module quiet zone each side: 29 wide, ceil(29 / 2) lines.
        assert_eq!((q.lines().count(), q.lines().next().unwrap().chars().count()), (15, 29));
        assert!(try_eval(r#"qr("x", "Z")"#).is_err());
        let board = show("chess()");
        assert!(board.starts_with("8 ♜ ♞ ♝ ♛ ♚ ♝ ♞ ♜\n7 ♟"));
        assert!(board.contains("\n1 ♖ ♘ ♗ ♕ ♔ ♗ ♘ ♖\n  a b"));
        assert_eq!(show(r#"chess("8/8/8/8/8/8/8/K7")"#).lines().nth(7), Some("1 ♔   ·   ·   ·  "));
        assert!(try_eval(r#"chess("8/8")"#).is_err());
        assert!(try_eval(r#"chess("9/8/8/8/8/8/8/8")"#).is_err());
        assert_eq!(show(r#"card("10h")"#), "┌─────┐\n│10   │\n│  ♥  │\n│   10│\n└─────┘");
        assert!(try_eval(r#"card("1x")"#).is_err());
        assert_eq!(show(r#"cards(["As", "?"])"#).lines().nth(1), Some("│A    │ │░░░░░│"));
        assert_eq!(show("dice(1)").lines().nth(2), Some("│   ●   │"));
        assert_eq!(show("dice([6, 2])").lines().nth(2), Some("│ ●   ● │ │       │"));
        assert!(try_eval("dice(7)").is_err());
        assert_eq!(show(r#"segments("1")"#), "\n   █\n   █");
        assert_eq!(show(r#"segments("8.")"#), " ▄▄\n█▄▄█\n█▄▄█ ▄");
        assert!(try_eval(r#"segments("x")"#).is_err());
    }
}
