//! Terminal styling: the ANSI colors zil prints with, when to use them, and how to take them out again.

use crate::lexer::Tok;
use crate::value::Value;

pub const DIM: &str = "[2m";
pub const CYAN: &str = "[36m";
pub const GREEN: &str = "[32m";
pub const MAGENTA: &str = "[35m";
pub const BOLD: &str = "[1m";
pub const RESET: &str = "[0m";

/// Whether to print colors: stdout is a terminal and `NO_COLOR` is unset.
pub fn color_on() -> bool {
    use std::io::IsTerminal;
    std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none()
}

/// Terminal width, or 80 without one.
pub fn columns() -> usize {
    #[cfg(not(target_arch = "wasm32"))]
    if let Some((w, _)) = terminal_size::terminal_size() {
        return w.0 as usize;
    }
    80
}

/// A value's color by type, as REPL results and help show it.
pub fn tint(v: &Value) -> &'static str {
    match v {
        Value::Int(..) | Value::Big(..) | Value::Frac(..) | Value::Float(_) | Value::Qty(..) | Value::Unc(..) | Value::Cplx(..) => CYAN,
        Value::Str(_) | Value::Regex(_) => GREEN,
        Value::Bool(_) | Value::Nil => MAGENTA,
        _ => "",
    }
}

/// `src` with numbers, strings and keywords colored; unlexable bits stay plain.
pub fn highlight(src: &str) -> String {
    use logos::Logos;
    let mut out = String::with_capacity(src.len() * 2);
    let mut last = 0;
    for (tok, span) in Tok::lexer(src).spanned() {
        let color = match tok {
            Ok(Tok::Int(_) | Tok::Float(_) | Tok::Dec(_) | Tok::Dur(_) | Tok::Clock(_) | Tok::Based(_) | Tok::Currency(_)) => CYAN,
            Ok(Tok::Str(_) | Tok::Regex(_)) => GREEN,
            Ok(
                Tok::Fn
                | Tok::If
                | Tok::Else
                | Tok::While
                | Tok::For
                | Tok::In
                | Tok::To
                | Tok::Of
                | Tok::Return
                | Tok::Break
                | Tok::Continue
                | Tok::True
                | Tok::False
                | Tok::Nil,
            ) => MAGENTA,
            _ => continue,
        };
        out.push_str(&src[last..span.start]);
        out.push_str(color);
        out.push_str(&src[span.clone()]);
        out.push_str(RESET);
        last = span.end;
    }
    out.push_str(&src[last..]);
    out
}

/// Remove terminal color and cursor codes.
pub fn strip_ansi(s: &str) -> String {
    let re = regex::Regex::new(r"\x1b\[[0-9;?]*[A-Za-z]").unwrap();
    re.replace_all(s, "").into_owned()
}

/// Terminal text as HTML for the browser build: colors and styles become inline-styled spans, the 16 basic colors
/// `var(--ansi-N)` so the page can theme them; cursor codes vanish.
#[cfg(any(target_arch = "wasm32", test))]
pub fn to_html(s: &str) -> String {
    let re = regex::Regex::new(r"\x1b\[([0-9;?]*)([A-Za-z])").unwrap();
    let basic = |n: u16| format!("var(--ansi-{n})");
    let (mut fg, mut bg, mut flags): (Option<String>, Option<String>, [bool; 5]) = (None, None, [false; 5]);
    // `css` is the style codes asked for, `open` the span written so far: spans change only where text follows,
    // so ariadne's char-by-char colors and reset-then-set pairs collapse into one span.
    let (mut out, mut last, mut open, mut css) = (String::new(), 0, String::new(), String::new());
    let text = |out: &mut String, open: &mut String, css: &str, t: &str| {
        if t.is_empty() {
            return;
        }
        if css != open {
            if !open.is_empty() {
                out.push_str("</span>");
            }
            if !css.is_empty() {
                *out += &format!("<span style=\"{css}\">");
            }
            *open = css.to_string();
        }
        // Glyphs past Latin-1 often come from a fallback font of another width, skewing boxes and columns; each run of
        // them is pinned to the cells a terminal gives it.
        let far = |c: char| c as u32 > 0xFF;
        let mut wide = false;
        for (i, c) in t.char_indices() {
            if far(c) != wide {
                wide = !wide;
                if wide {
                    let n: usize = t[i..].chars().take_while(|&c| far(c)).filter_map(unicode_width::UnicodeWidthChar::width).sum();
                    *out += &format!("<span style=\"display:inline-block;width:{n}ch\">");
                } else {
                    out.push_str("</span>");
                }
            }
            match c {
                '&' => out.push_str("&amp;"),
                '<' => out.push_str("&lt;"),
                '>' => out.push_str("&gt;"),
                c => out.push(c),
            }
        }
        if wide {
            out.push_str("</span>");
        }
    };
    for c in re.captures_iter(s) {
        let m = c.get(0).unwrap();
        text(&mut out, &mut open, &css, &s[last..m.start()]);
        last = m.end();
        if &c[2] != "m" {
            continue;
        }
        let ps: Vec<u16> = c[1].split(';').map(|p| p.parse().unwrap_or(0)).collect();
        let mut it = ps.iter().copied();
        while let Some(p) = it.next() {
            match p {
                0 => (fg, bg, flags) = (None, None, [false; 5]),
                // bold, dim, italic, underline, reverse
                1..=4 => flags[p as usize - 1] = true,
                7 => flags[4] = true,
                22 => (flags[0], flags[1]) = (false, false),
                23 | 24 => flags[p as usize - 21] = false,
                27 => flags[4] = false,
                30..=37 => fg = Some(basic(p - 30)),
                90..=97 => fg = Some(basic(p - 82)),
                40..=47 => bg = Some(basic(p - 40)),
                100..=107 => bg = Some(basic(p - 92)),
                39 => fg = None,
                49 => bg = None,
                38 | 48 => {
                    let color = match it.next() {
                        Some(5) => it.next().map(|n| match n {
                            0..=15 => basic(n),
                            16..=231 => {
                                let k = |x: u16| if x == 0 { 0 } else { 55 + 40 * x };
                                format!("rgb({},{},{})", k((n - 16) / 36), k((n - 16) / 6 % 6), k((n - 16) % 6))
                            }
                            _ => format!("rgb({0},{0},{0})", 8 + 10 * (n.min(255) - 232)),
                        }),
                        Some(2) => Some(format!("rgb({},{},{})", it.next().unwrap_or(0), it.next().unwrap_or(0), it.next().unwrap_or(0))),
                        _ => None,
                    };
                    *if p == 38 { &mut fg } else { &mut bg } = color;
                }
                _ => {}
            }
        }
        let (f, b) =
            if flags[4] { (bg.clone().or(Some("var(--ansi-bg)".into())), fg.clone().or(Some("var(--ansi-fg)".into()))) } else { (fg.clone(), bg.clone()) };
        css.clear();
        for (on, rule) in flags[..4].iter().zip(["font-weight:600", "opacity:.6", "font-style:italic", "text-decoration:underline"]) {
            if *on {
                css += rule;
                css.push(';');
            }
        }
        css += &f.map_or(String::new(), |c| format!("color:{c};"));
        css += &b.map_or(String::new(), |c| format!("background:{c};"));
    }
    text(&mut out, &mut open, &css, &s[last..]);
    if !open.is_empty() {
        out.push_str("</span>");
    }
    out
}

/// Terminal columns `s` takes: color codes take none, wide glyphs two.
pub fn width(s: &str) -> usize {
    unicode_width::UnicodeWidthStr::width(strip_ansi(s).as_str())
}

/// `body` in a rounded box with `label` on the top edge; `body` may carry colors and wide glyphs.
pub fn boxed(label: &str, body: &str, dim: &str, reset: &str) -> String {
    let inner = body.lines().map(width).max().unwrap_or(0).max(width(label) + 2);
    let mut out = format!("{dim}╭─{reset} {label} {dim}{}╮{reset}\n", "─".repeat(inner - width(label) - 1));
    for line in body.lines() {
        out += &format!("{dim}│{reset} {line}{} {dim}│{reset}\n", " ".repeat(inner - width(line)));
    }
    out + &format!("{dim}╰{}╯{reset}", "─".repeat(inner + 2))
}

#[cfg(test)]
mod tests {
    #[test]
    fn to_html() {
        assert_eq!(super::to_html("a < b"), "a &lt; b");
        assert_eq!(super::to_html("[31ma[0m[31mb[0m"), r#"<span style="color:var(--ansi-1);">ab</span>"#);
        assert_eq!(super::to_html("\x1b[1;36mx\x1b[0m y"), r#"<span style="font-weight:600;color:var(--ansi-6);">x</span> y"#);
        assert_eq!(
            super::to_html("\x1b[38;2;1;2;3m\x1b[48;5;196m▀\x1b[0m"),
            r#"<span style="color:rgb(1,2,3);background:rgb(255,0,0);"><span style="display:inline-block;width:1ch">▀</span></span>"#
        );
        assert_eq!(super::to_html("é ｗ𝐛→x"), r#"é <span style="display:inline-block;width:4ch">ｗ𝐛→</span>x"#);
        assert_eq!(super::to_html("\x1b[7mhit\x1b[27m\x1b[2K"), r#"<span style="color:var(--ansi-bg);background:var(--ansi-fg);">hit</span>"#);
    }
}
