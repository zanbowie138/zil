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
