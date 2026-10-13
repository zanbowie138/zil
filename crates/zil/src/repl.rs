//! What a REPL does with each input, shared by the terminal REPL (`zil-cli`) and the browser's (`zil-wasm`):
//! when a line continues, `help topic` shorthand, and results saved and labeled as `_n`.

use crate::Session;
use crate::ansi::{RESET, Role, tint};
use crate::lexer::{self, Tok};
use crate::value::Value;

/// More `(`/`[`/`{` than closers: the REPL keeps reading lines.
pub fn unclosed(src: &str) -> bool {
    let Ok(toks) = lexer::lex(src) else {
        return false;
    };
    let depth: i32 = toks
        .iter()
        .map(|(t, _)| match t {
            Tok::LParen | Tok::LBracket | Tok::LBrace => 1,
            Tok::RParen | Tok::RBracket | Tok::RBrace => -1,
            _ => 0,
        })
        .sum();
    depth > 0
}

/// REPL `help` / `help topic`: the topic (`None` for the overview); `None` for ordinary code.
pub fn help_shorthand(src: &str) -> Option<Option<&str>> {
    let rest = src.trim().strip_prefix("help")?;
    let topic = rest.trim_start();
    if topic.is_empty() {
        return Some(None);
    }
    let word = topic.chars().all(|c| c.is_alphanumeric() || c == '_');
    (rest.starts_with(char::is_whitespace) && word).then_some(Some(topic))
}

/// Whether `src` is help printed straight to the screen: `help upper`, or a line that is just `help(...)` or `tip()`.
/// Help inside a bigger expression stays plain text for piping.
pub fn help_call(src: &str) -> bool {
    use crate::ast::ExprKind::{Call, Ident};
    let call =
        matches!(crate::parser::parse(src).as_deref(), Ok([e]) if matches!(&e.kind, Call(f, _) if matches!(&f.kind, Ident(n) if n == "help" || n == "tip")));
    call || help_shorthand(src).is_some()
}

/// Units convertible from the expression before a trailing `to`, for completion. Read from literals and `s`'s
/// variables only: nothing is evaluated, so Tab has no side effects. Call it inside `s.enter` so user units count.
pub fn conversions(s: &Session, before: &str) -> Option<Vec<String>> {
    use crate::ast::ExprKind::{Assign, Ident, Qty};
    use crate::modules::units::{known_dim, names_with_dim};
    let toks = lexer::lex(before).ok()?;
    let [.., (Tok::To, to), (Tok::Eof, _)] = toks.as_slice() else {
        return None;
    };
    // Longest parseable tail, so `print(5 km to` looks at `5 km`.
    let ast = toks.iter().find_map(|(_, sp)| crate::parser::parse(&before[sp.start..to.start]).ok())?;
    let mut e = ast.last()?;
    while let Assign(_, rhs) = &e.kind {
        e = rhs;
    }
    let dim = match &e.kind {
        Qty(_, spec) => known_dim(spec)?,
        Ident(name) => match s.get(name) {
            Some(Value::Qty(_, u)) => u.dim(),
            _ => known_dim(&vec![(name.clone(), 1)])?,
        },
        _ => return None,
    };
    Some(names_with_dim(dim))
}

/// How the REPL shows a result.
pub enum Shown {
    /// nil and functions: nothing.
    Hidden,
    /// Multi-line text, like `help(upper)`, reads better bare and unnumbered.
    Bare,
    /// Everything else, labeled `_n =`.
    Numbered(usize),
}

/// Saves a result as `_`, and as the next `_n` (counted in `n`) when it's numbered.
pub fn record(s: &mut Session, n: &mut usize, v: &Value) -> Shown {
    let shown = match v {
        Value::Nil | Value::Fn(_) => return Shown::Hidden,
        Value::Str(s) if s.contains('\n') => Shown::Bare,
        _ => {
            *n += 1;
            s.set(&format!("_{n}"), v.clone());
            Shown::Numbered(*n)
        }
    };
    s.set("_", v.clone());
    shown
}

/// A numbered result labeled with the `_n` it's saved as, colored by type if `color`.
pub fn show(n: usize, v: &Value, color: bool) -> String {
    if let Value::Table(t) = v {
        let label = if color { format!("{} · _{n}{RESET}", Role::ResultLabel.code()) } else { format!(" · _{n}") };
        return crate::modules::data::tables::grid(t, color) + &label;
    }
    if !color {
        return format!("_{n} = {v}");
    }
    format!("{}_{n} ={RESET} {}{v}{RESET}", Role::ResultLabel.code(), tint(v).code())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session() {
        assert!(unclosed("f = fn(x) {") && !unclosed("[1, 2]") && !unclosed(r#""(""#));
        assert_eq!(help_shorthand("help upper"), Some(Some("upper")));
        assert_eq!(help_shorthand("help"), Some(None));
        assert_eq!(help_shorthand("help(upper)"), None);

        let (mut it, mut n) = (Session::new(crate::host::get()), 0);
        let mut put = |src: &str| {
            let v = it.eval(src).unwrap();
            match record(&mut it, &mut n, &v) {
                Shown::Hidden => "hidden".to_string(),
                Shown::Bare => "bare".to_string(),
                Shown::Numbered(k) => show(k, &v, false),
            }
        };
        assert_eq!(put("1 + 1"), "_1 = 2");
        assert_eq!(put("nil"), "hidden");
        assert_eq!(put(r#""a\nb""#), "bare");
        assert_eq!(put("_1 * 10"), "_2 = 20");
        assert_eq!(put("_"), "_3 = 20");
    }
}
