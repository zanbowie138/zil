//! Text art: a clip-art gallery and color for any of it; frames, generators, renderers, fonts,
//! images, animation and toys below. Art is just a multi-line string, so every function composes.

pub mod figlet;
pub mod frames;
pub mod generate;
pub mod images;
pub mod motion;
pub mod renderers;
pub mod toys;

use crate::interp::Interp;
use crate::lexer::Span;
use crate::modules::dev::colors::{self, Rgb};
use crate::modules::{Call, Doc, Fail, Module, doc};
use crate::value::Value;

pub const MODULE: Module = Module {
    name: "art",
    about: "clip art, rainbows and gradients; boxes, cowsay and composition, fractals, QR codes, chess boards, FIGlet banners, images and animation below",
    #[rustfmt::skip]
    examples: &[
        ("gallery", &[
            ("what's in it", "clipart()"),
            ("a cat in a box", r#"clipart("cat").boxed("round")"#),
            ("pride", r#"banner("zil", "slant").rainbow"#),
            ("measure colored art", r#"rainbow("hi").strip_ansi.len"#),
        ]),
    ],
    fns: FNS,
    call,
    children: &[frames::MODULE, generate::MODULE, renderers::MODULE, figlet::MODULE, images::MODULE, motion::MODULE, toys::MODULE],
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("clipart", "clipart(name?: str)", "a piece from the gallery, or the list of names", &[r#"clipart("owl")"#, "clipart().len"], &["cowsay", "boxed"]),
    doc("rainbow", "rainbow(s: str)", "color each character along a diagonal rainbow, for truecolor terminals", &[r#""rainbow".rainbow"#], &["gradient", "strip_ansi"]),
    doc("gradient", "gradient(s: str, from: str|list, to: str|list)", "fade text left to right between two colors, for truecolor terminals", &[r#"gradient("sunset", "gold", "crimson")"#], &["rainbow", "color"]),
    doc("strip_ansi", "strip_ansi(s: str)", "remove terminal color and cursor codes", &[r#"gradient("ab", "red", "blue").strip_ansi"#], &["rainbow"]),
];

fn call(_: &mut Interp, name: &'static str, args: &[Value], _: &Span) -> Call {
    use Value::*;
    let color = |i: usize| colors::parse(&args[i]).ok_or_else(|| Fail::Arg(i, "expected a color name, \"#rrggbb\" or [r, g, b]".into()));
    Ok(match (name, args) {
        ("clipart", []) => Value::list(GALLERY.iter().map(|g| Value::str(g.0)).collect()),
        ("clipart", [Str(n)]) => {
            let piece = GALLERY
                .iter()
                .find(|g| g.0 == n.trim().to_lowercase())
                .ok_or_else(|| Fail::Arg(0, format!("no {n:?} in the gallery; clipart() lists them")))?;
            Value::str(piece.1.trim_start_matches('\n'))
        }
        ("rainbow", [Str(s)]) => Value::str(paint(s, |col, row, _| colors::from_hsl((col * 12 + row * 6) as f64, 1.0, 0.6))),
        ("gradient", [Str(s), _, _]) => {
            let (a, b) = (color(1)?, color(2)?);
            Value::str(paint(s, |col, _, w| colors::mix(a, b, if w > 1 { col as f64 / (w - 1) as f64 } else { 0.0 })))
        }
        ("strip_ansi", [Str(s)]) => Value::str(strip_ansi(s)),
        _ => return Err(Fail::BadArgs),
    })
}

/// Color every visible character by its column, row and the art's width; whitespace stays plain.
fn paint(s: &str, rgb: impl Fn(usize, usize, usize) -> Rgb) -> String {
    let s = strip_ansi(s);
    let w = s.lines().map(|l| l.chars().count()).max().unwrap_or(0);
    let lines = s.lines().enumerate().map(|(row, line)| {
        let mut out = String::new();
        for (col, c) in line.chars().enumerate() {
            if c.is_whitespace() {
                out.push(c);
            } else {
                out += &format!("{}{c}", fg(rgb(col, row, w)));
            }
        }
        if out.len() > line.len() { out + RESET } else { out }
    });
    lines.collect::<Vec<_>>().join("\n")
}

pub const RESET: &str = "\x1b[0m";

/// The truecolor escape for a foreground color.
pub fn fg(c: Rgb) -> String {
    let [r, g, b] = c.map(|x| x.round().clamp(0.0, 255.0) as u8);
    format!("\x1b[38;2;{r};{g};{b}m")
}

pub fn strip_ansi(s: &str) -> String {
    let re = regex::Regex::new(r"\x1b\[[0-9;?]*[A-Za-z]").unwrap();
    re.replace_all(s, "").into_owned()
}

/// Art as a rectangle of chars, short lines padded with spaces.
// ponytail: one char = one column; wide chars (emoji, CJK) and color codes skew alignment, add unicode-width if that bites.
pub fn grid(s: &str) -> Vec<Vec<char>> {
    let mut g: Vec<Vec<char>> = s.lines().map(|l| l.chars().collect()).collect();
    let w = g.iter().map(Vec::len).max().unwrap_or(0);
    for row in &mut g {
        row.resize(w, ' ');
    }
    g
}

/// A grid back to text, trailing spaces dropped.
pub fn ungrid(g: &[Vec<char>]) -> String {
    g.iter().map(|row| row.iter().collect::<String>().trim_end().to_string()).collect::<Vec<_>>().join("\n")
}

/// A braille canvas: each character cell holds 2×4 dots, which come out about square in a terminal.
pub struct Dots {
    pub w: usize,
    pub h: usize,
    cells: Vec<u8>,
}

impl Dots {
    pub fn new(w: usize, h: usize) -> Dots {
        Dots { w, h, cells: vec![0; w.div_ceil(2) * h.div_ceil(4)] }
    }

    pub fn set(&mut self, x: i64, y: i64) {
        if (0..self.w as i64).contains(&x) && (0..self.h as i64).contains(&y) {
            const BIT: [[u8; 4]; 2] = [[0x01, 0x02, 0x04, 0x40], [0x08, 0x10, 0x20, 0x80]];
            let (x, y) = (x as usize, y as usize);
            self.cells[y / 4 * self.w.div_ceil(2) + x / 2] |= BIT[x % 2][y % 4];
        }
    }

    pub fn line(&mut self, (x0, y0): (f64, f64), (x1, y1): (f64, f64)) {
        let n = (x1 - x0).abs().max((y1 - y0).abs()).ceil().max(1.0) as usize;
        for i in 0..=n {
            let t = i as f64 / n as f64;
            self.set((x0 + (x1 - x0) * t).round() as i64, (y0 + (y1 - y0) * t).round() as i64);
        }
    }

    /// Empty cells come out as spaces so lines trim cleanly.
    pub fn render(&self) -> String {
        let rows = self.cells.chunks(self.w.div_ceil(2).max(1));
        let lines =
            rows.map(|r| r.iter().map(|b| if *b == 0 { ' ' } else { char::from_u32(0x2800 + *b as u32).unwrap() }).collect::<String>().trim_end().to_string());
        lines.collect::<Vec<_>>().join("\n")
    }
}

/// Original pieces and old folk art from Usenet days.
const GALLERY: &[(&str, &str)] = &[
    (
        "cat",
        r"
 /\_/\
( o.o )
 > ^ <",
    ),
    (
        "dog",
        r"
  __      _
o'')}____//
 `_/      )
 (_(_/-(_/",
    ),
    (
        "owl",
        r#"
 ,_,
(O,O)
(   )
-"-"-"#,
    ),
    (
        "bunny",
        r"
(\_/)
(•ᴗ•)
/ > ♥",
    ),
    (
        "fish",
        r"
   ><(((º>",
    ),
    (
        "duck",
        r"
   __
 <(o )___
  ( ._> /
   `---'",
    ),
    (
        "bat",
        r#"
 /\                 /\
/ \'._   (\_/)   _.'/ \
|.''._'--(o.o)--'_.''.|
 \_ / `;=/ " \=;` \ _/
   `\__| \___/ |__/`
        \(_|_)/
         " ` ""#,
    ),
    (
        "snail",
        r#"
    .----.   @   @
   / .-"-.`.  \v/
   | | '\ \ \_/ )
 ,-\ `-.' /.'  /
'---`----'----'"#,
    ),
    (
        "crab",
        r"
   _~^~^~_
\) /  o o  \ (/
  '_   -   _'
  / '-----' \",
    ),
    (
        "ghost",
        r"
  .-.
 (o o)
 | O \
  \   \
   `~~~'",
    ),
    (
        "skull",
        r"
  _____
 /     \
| () () |
 \  ^  /
  |||||
  |||||",
    ),
    (
        "coffee",
        r"
   ( (
    ) )
  ........
  |      |]
  \      /
   `----'",
    ),
    (
        "rocket",
        r"
    ^
   / \
  /___\
  |   |
  | z |
  |   |
 /|   |\
/_|___|_\
   /_\",
    ),
    (
        "house",
        r"
    /\
   /  \
  /____\
  | [] |
  |_||_|",
    ),
    (
        "tree",
        r"
    *
   /o\
  /o  \
 /  o  \
/_o___o_\
   |_|",
    ),
    (
        "cactus",
        r"
     _
    | |
 _  | |  _
| | | | | |
| |_| |_| |
 \___   _/
     | |
     | |",
    ),
    (
        "heart",
        r"
 ,d88b.d88b,
 88888888888
 `Y8888888Y'
   `Y888Y'
     `Y'",
    ),
];

#[cfg(test)]
mod tests {
    use crate::interp::tests::{show, try_eval};

    #[test]
    fn gallery() {
        assert_eq!(show(r#"clipart("CAT")"#), " /\\_/\\\n( o.o )\n > ^ <");
        assert!(try_eval(r#"clipart("unicorn")"#).is_err());
        assert_eq!(show(r#"rainbow("a b").strip_ansi"#), "a b");
        assert_eq!(show(r#"gradient("ab", "red", "blue")"#), "\x1b[38;2;255;0;0ma\x1b[38;2;0;0;255mb\x1b[0m");
        assert_eq!(show(r#"rainbow(" ")"#), " ");
    }
}
