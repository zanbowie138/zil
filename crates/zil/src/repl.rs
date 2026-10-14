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
fn conversions(s: &Session, before: &str) -> Option<Vec<String>> {
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

/// Tab completions for `word`, given the text `before` it: after `to`, every unit the left side converts to; after `.`, only
/// functions; otherwise builtins, constants, `to` targets, modules, units, help topics and the session's variables and units.
pub fn complete(s: &Session, before: &str, word: &str) -> Vec<String> {
    use crate::modules::{self, units};
    static WORDS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| {
        let mut v: Vec<String> = modules::ALL.iter().map(|(path, _)| path.clone()).collect();
        v.extend(
            modules::modules()
                .flat_map(|m| m.fns.iter().map(|f| f.name).chain(m.consts.iter().map(|c| c.0)).chain(m.targets.iter().map(|t| t.0)).chain([m.name]))
                .map(String::from),
        );
        v.extend(units::TABLE.iter().flat_map(|u| u.0.split_whitespace()).map(String::from));
        v.extend(units::DIMS.iter().map(|d| d.0.to_string()));
        v.extend(["syntax", "examples", "advanced"].map(String::from));
        v
    });
    let mut hits: Vec<String> = s.enter(|| {
        if let Some(units) = conversions(s, before) {
            units
        } else if before.ends_with('.') {
            s.names().into_iter().filter(|n| matches!(s.get(n), Some(Value::Fn(_) | Value::Builtin(..)))).collect()
        } else if word.is_empty() {
            vec![]
        } else {
            WORDS.iter().cloned().chain(s.names()).chain(units::user_units()).collect()
        }
    });
    hits.retain(|n| n.starts_with(word));
    hits.sort();
    hits.dedup();
    hits
}

/// A result as `-e` prints it: tables as a grid, everything else as its text.
pub fn display(v: &Value, color: bool) -> String {
    match v {
        Value::Table(t) => crate::modules::data::tables::grid(t, color),
        v => v.to_string(),
    }
}

/// How the REPL shows a result.
pub enum Shown {
    /// nil and functions: nothing.
    Hidden,
    /// Multi-line text, like `help(upper)`, reads better bare and unnumbered.
    Bare,
    /// Everything else, labeled `_n =`.
    Numbered(usize),
    /// An assignment, labeled with what it assigned and the `_n` it's saved as: `a = 5  · _1`.
    Named(usize, Vec<(String, Value)>),
    /// A function assigned to a name: its signature, `f(x)`.
    Defined(String),
}

/// Saves `src`'s result `v` as `_`, and as the next `_n` (counted in `n`) when it's numbered.
pub fn record(s: &mut Session, n: &mut usize, src: &str, v: &Value) -> Shown {
    let named = assigned(src).map(|names| match &names[..] {
        [one] => vec![(one.clone(), v.clone())],
        _ => names.into_iter().filter_map(|k| Some((s.get(&k)?, k))).map(|(v, k)| (k, v)).collect(),
    });
    let shown = match (v, named) {
        (Value::Fn(f), Some(pairs)) if pairs.len() == 1 => return Shown::Defined(format!("{}({})", pairs[0].0, f.params())),
        (Value::Nil | Value::Fn(_), _) => return Shown::Hidden,
        (Value::Str(s), None) if s.contains('\n') => Shown::Bare,
        (_, named) => {
            *n += 1;
            s.set(&format!("_{n}"), v.clone());
            named.map_or(Shown::Numbered(*n), |pairs| Shown::Named(*n, pairs))
        }
    };
    s.set("_", v.clone());
    shown
}

/// What the last statement of `src` assigns to, as written: `a`, `m.k`, `xs[0]`, or each name in `[a, [b, _]] = ...`.
fn assigned(src: &str) -> Option<Vec<String>> {
    use crate::ast::{ExprKind, Pat};
    fn names(p: &Pat, out: &mut Vec<String>) {
        match p {
            Pat::Name(n) => out.push(n.clone()),
            Pat::Skip => {}
            Pat::List(ps, _) => ps.iter().for_each(|p| names(p, out)),
        }
    }
    match crate::parser::parse(src).ok()?.pop()?.kind {
        ExprKind::Assign(lhs, _) => Some(vec![src.get(lhs.span)?.to_string()]),
        ExprKind::Unpack(p, _) => {
            let mut out = vec![];
            names(&p, &mut out);
            Some(out)
        }
        _ => None,
    }
}

/// An assignment's result: `a = 5  · _1`, or a table's grid labeled `· a · _1`.
pub fn show_named(n: usize, pairs: &[(String, Value)], color: bool) -> String {
    let (dim, reset) = if color { (Role::ResultLabel.code(), RESET) } else { ("", "") };
    if let [(name, Value::Table(t))] = pairs {
        return format!("{}{dim} · {name} · _{n}{reset}", crate::modules::data::tables::grid(t, color));
    }
    let tinted = |v: &Value| if color { format!("{}{v}{RESET}", tint(v).code()) } else { v.to_string() };
    let body: Vec<String> = pairs.iter().map(|(k, v)| format!("{k} = {}", tinted(v))).collect();
    format!("{}  {dim}· _{n}{reset}", body.join(", "))
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
        assert_eq!(complete(&it, "x.", "upp"), ["upper"]);
        assert_eq!(complete(&it, "print(5 km to ", "ma"), ["marathon"]);
        let mut put = |src: &str| {
            let v = it.eval(src).unwrap();
            match record(&mut it, &mut n, src, &v) {
                Shown::Hidden => "hidden".to_string(),
                Shown::Bare => "bare".to_string(),
                Shown::Numbered(k) => show(k, &v, false),
                Shown::Named(k, pairs) => show_named(k, &pairs, false),
                Shown::Defined(sig) => sig,
            }
        };
        assert_eq!(put("1 + 1"), "_1 = 2");
        assert_eq!(put("nil"), "hidden");
        assert_eq!(put(r#""a\nb""#), "bare");
        assert_eq!(put("_1 * 10"), "_2 = 20");
        assert_eq!(put("_"), "_3 = 20");
        assert_eq!(put("a = [1, 2]"), "a = [1, 2]  · _4");
        assert_eq!(put("a += [3]"), "a = [1, 2, 3]  · _5");
        assert_eq!(put("a[0] = 9"), "a[0] = 9  · _6");
        assert_eq!(put("[x, [_, y]] = [1, [2, 3]]"), "x = 1, y = 3  · _7");
        assert_eq!(put("f = |x, [a, _]| x"), "f(x, [a, _])");
        assert_eq!(put("|x| x"), "hidden");
        assert_eq!(put("b = 1; 2"), "_8 = 2");
        assert_eq!(
            put(r#"s = "a
b""#),
            "s = a
b  · _9"
        );
    }
}
