//! The REPL: line editing with completion, hints and syntax colors, multi-line input, `help` shorthand and numbered results.

use crate::ansi::{DIM, RESET, color_on, highlight, tint};
use crate::interp::{self, Interp};
use crate::lexer::{self, Tok};
use crate::value::Value;
use crate::{ast, cache_dir, help, modules, pager, parser, report, run};
use rustyline::error::ReadlineError;
use std::borrow::Cow;

/// More `(`/`[`/`{` than closers: the REPL keeps reading lines.
fn unclosed(src: &str) -> bool {
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
fn help_shorthand(src: &str) -> Option<Option<&str>> {
    let rest = src.trim().strip_prefix("help")?;
    let topic = rest.trim_start();
    if topic.is_empty() {
        return Some(None);
    }
    let word = topic.chars().all(|c| c.is_alphanumeric() || c == '_');
    (rest.starts_with(char::is_whitespace) && word).then_some(Some(topic))
}

/// REPL tab completion over builtins, constants, `to` targets, units, help topics and
/// variables in scope. After `.`, only functions; after `to`, every unit the left side converts to.
struct Names(Vec<String>, interp::Env);

impl Names {
    fn new(globals: interp::Env) -> Names {
        let mut v: Vec<String> = modules::ALL.iter().map(|(path, _)| path.clone()).collect();
        v.extend(
            modules::modules()
                .flat_map(|m| m.fns.iter().map(|f| f.name).chain(m.consts.iter().map(|c| c.0)).chain(m.targets.iter().map(|t| t.0)).chain([m.name]))
                .map(String::from),
        );
        v.extend(modules::units::TABLE.iter().flat_map(|u| u.0.split_whitespace()).map(String::from));
        v.extend(modules::units::DIMS.iter().map(|d| d.0.to_string()));
        v.extend(["syntax", "examples", "advanced"].map(String::from));
        v.sort();
        v.dedup();
        Names(v, globals)
    }

    /// Units convertible from the expression before a trailing `to`, read from literals and
    /// existing variables only: nothing is evaluated, so Tab has no side effects.
    fn conversions(&self, before: &str) -> Option<Vec<String>> {
        let toks = lexer::lex(before).ok()?;
        let [.., (Tok::To, to), (Tok::Eof, _)] = toks.as_slice() else {
            return None;
        };
        // Longest parseable tail, so `print(5 km to` looks at `5 km`.
        let ast = toks.iter().find_map(|(_, s)| parser::parse(&before[s.start..to.start]).ok())?;
        let mut e = ast.last()?;
        while let ast::ExprKind::Assign(_, rhs) = &e.kind {
            e = rhs;
        }
        let dim = match &e.kind {
            ast::ExprKind::Qty(_, spec) => modules::units::known_dim(spec)?,
            ast::ExprKind::Ident(name) => match interp::lookup(&self.1, name) {
                Some(Value::Qty(_, u)) => u.dim(),
                _ => modules::units::known_dim(&vec![(name.clone(), 1)])?,
            },
            _ => return None,
        };
        Some(modules::units::names_with_dim(dim))
    }
}

impl rustyline::completion::Completer for Names {
    type Candidate = String;
    fn complete(&self, line: &str, pos: usize, _: &rustyline::Context<'_>) -> rustyline::Result<(usize, Vec<String>)> {
        let start = pos - line[..pos].chars().rev().take_while(|c| c.is_alphanumeric() || *c == '_').map(char::len_utf8).sum::<usize>();
        let word = &line[start..pos];
        let mut hits: Vec<String> = if let Some(units) = self.conversions(&line[..start]) {
            units
        } else if line[..start].ends_with('.') {
            interp::methods(&self.1)
        } else if word.is_empty() {
            vec![]
        } else {
            self.0.iter().cloned().chain(interp::names(&self.1)).chain(modules::units::user_units()).collect()
        };
        hits.retain(|n| n.starts_with(word));
        hits.sort();
        hits.dedup();
        Ok((start, hits))
    }
}

/// Fish-style ghost text: the rest of the word when exactly one completion fits.
impl rustyline::hint::Hinter for Names {
    type Hint = String;
    fn hint(&self, line: &str, pos: usize, ctx: &rustyline::Context<'_>) -> Option<String> {
        use rustyline::completion::Completer;
        if pos < line.len() {
            return None;
        }
        let (start, hits) = self.complete(line, pos, ctx).ok()?;
        let [hit] = hits.as_slice() else { return None };
        Some(hit[pos - start..].to_string()).filter(|h| !h.is_empty())
    }
}

/// Syntax colors for the line being typed; unlexable bits stay plain.
impl rustyline::highlight::Highlighter for Names {
    fn highlight<'l>(&self, line: &'l str, _: usize) -> Cow<'l, str> {
        Cow::Owned(highlight(line))
    }
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(&'s self, prompt: &'p str, _: bool) -> Cow<'b, str> {
        Cow::Owned(format!("{DIM}{prompt}{RESET}"))
    }
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned(format!("{DIM}{hint}{RESET}"))
    }
    fn highlight_char(&self, _: &str, _: usize, _: rustyline::highlight::CmdKind) -> bool {
        true
    }
}

/// Help shaped for stdout, `(color, width)`, if `src` is help printed straight to it: `help upper`,
/// or a line that is just `help(...)`. Help inside a bigger expression stays plain text for piping.
pub fn help_term(src: &str) -> Option<(bool, usize)> {
    use ast::ExprKind::{Call, Ident};
    let call = matches!(parser::parse(src).as_deref(), Ok([e]) if matches!(&e.kind, Call(f, _) if matches!(&f.kind, Ident(n) if n == "help")));
    let (terminal_size::Width(w), _) = terminal_size::terminal_size_of(std::io::stdout())?;
    (call || help_shorthand(src).is_some()).then_some((color_on(), w as usize))
}

/// A REPL result labeled with the `_n` it's saved as, colored by type when `color_on`.
fn show(n: usize, v: &Value) -> String {
    if let Value::Table(t) = v {
        let label = if color_on() { format!("{DIM} · _{n}{RESET}") } else { format!(" · _{n}") };
        return modules::data::tables::grid(t, color_on()) + &label;
    }
    if !color_on() {
        return format!("_{n} = {v}");
    }
    format!("{DIM}_{n} ={RESET} {}{v}{RESET}", tint(v))
}
impl rustyline::validate::Validator for Names {}
impl rustyline::Helper for Names {}

/// A random `(label, code)` from what `help("examples")` and `help("advanced")` show, for the startup blurb.
fn tip() -> (&'static str, &'static str) {
    use crate::guide::{ADVANCED, ADVANCED_UNRUN, HIGHLIGHTS};
    let examples = HIGHLIGHTS.iter().chain(modules::modules().flat_map(|m| m.examples)).flat_map(|s| s.1.iter().copied());
    let recipes = ADVANCED.iter().chain([&ADVANCED_UNRUN]).flat_map(|t| t.1).map(|&(what, _, code)| (what, code));
    // The highlights end on a syntax note with no code.
    let tips: Vec<_> = examples.chain(recipes).filter(|(_, code)| !code.is_empty()).collect();
    tips[fastrand::usize(..tips.len())]
}

pub fn repl(interp: &mut Interp) {
    let config = rustyline::Config::builder().completion_type(rustyline::CompletionType::List).build();
    let mut rl = rustyline::Editor::with_config(config).expect("terminal");
    rl.set_helper(Some(Names::new(interp.globals())));
    let history = cache_dir().map(|d| d.join("history.txt"));
    if let Some(h) = &history {
        let _ = rl.load_history(h);
    }
    let (what, code) = tip();
    let code = if color_on() { highlight(code) } else { code.to_string() };
    let (dim, reset) = if color_on() { (DIM, RESET) } else { ("", "") };
    println!("zil {} {dim}· help() for docs, exit or Ctrl-D to quit{reset}", env!("CARGO_PKG_VERSION"));
    // A multi-line recipe starts under its label.
    let sep = if code.contains('\n') { "\n" } else { " " };
    println!("{dim}tip, {what}:{reset}{sep}{code}\n");
    let mut buf = String::new();
    let mut n = 0;
    loop {
        let line = match rl.readline(if buf.is_empty() { "> " } else { ". " }) {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) => {
                buf.clear();
                continue;
            }
            Err(_) => break,
        };
        buf.push_str(&line);
        buf.push('\n');
        if unclosed(&buf) {
            continue;
        }
        let src = std::mem::take(&mut buf);
        if src.trim().is_empty() {
            continue;
        }
        if matches!(src.trim(), "exit" | "quit") {
            break;
        }
        if src.trim() == "clear" {
            let _ = rl.clear_screen();
            continue;
        }
        let _ = rl.add_history_entry(src.trim_end());
        // Printed directly, so it keeps its colors.
        if let Some(topic) = help_shorthand(&src) {
            pager::page(&help::live(help_term(&src), || help::help(topic, color_on())).unwrap_or_else(|e| e));
            println!();
            continue;
        }
        match help::live(help_term(&src), || run(interp, &src)) {
            Ok(Value::Nil | Value::Fn(_)) => {}
            // Multi-line text, like `help(upper)`, reads better bare and unnumbered.
            Ok(v) if matches!(&v, Value::Str(s) if s.contains('\n')) => {
                match help_term(&src) {
                    Some(_) => pager::page(&v.to_string()),
                    None => println!("{v}"),
                }
                interp.set_global("_", v);
            }
            Ok(v) => {
                n += 1;
                println!("{}", show(n, &v));
                interp.set_global(&format!("_{n}"), v.clone());
                interp.set_global("_", v);
            }
            Err(e) => report("<repl>", &src, e),
        }
        println!();
    }
    if let Some(h) = &history {
        let _ = std::fs::create_dir_all(h.parent().unwrap());
        let _ = rl.save_history(h);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ansi::{CYAN, GREEN, MAGENTA};
    use rustyline::completion::Completer;

    #[test]
    fn completes_names_at_cursor() {
        let history = rustyline::history::DefaultHistory::new();
        let ctx = rustyline::Context::new(&history);
        let mut interp = Interp::new();
        let names = Names::new(interp.globals());
        assert_eq!(names.complete("x.upp", 5, &ctx).unwrap(), (2, vec!["upper".to_string()]));
        assert!(names.complete("5 kilo", 6, &ctx).unwrap().1.contains(&"kilometers".to_string()));
        assert!(names.complete("x ", 2, &ctx).unwrap().1.is_empty());

        let to = |line: &str| names.complete(line, line.len(), &ctx).unwrap().1;
        let length = to("5 km to ");
        assert!(length.contains(&"mi".into()) && length.contains(&"ly".into()));
        assert!(!length.contains(&"kg".into()) && !length.contains(&"kilometers".into()));
        assert_eq!(to("print(5 km to m"), ["m", "marathon", "mi", "mm"]);
        assert!(to("y = 60 km/h to ").contains(&"mph".into()));
        run(&mut interp, "x = 3 kg").unwrap();
        assert!(to("x to ").contains(&"g".into()));
        assert!(to("z to ").is_empty());

        run(&mut interp, "my_total = 1; my_fn = |x| x").unwrap();
        assert_eq!(to("my_"), ["my_fn", "my_total"]);
        assert_eq!(to("x.my_"), ["my_fn"]);
        assert!(!to("x.").contains(&"km".into()) && to("x.").contains(&"upper".into()));
    }

    #[test]
    fn hints_and_highlights() {
        use rustyline::highlight::Highlighter;
        use rustyline::hint::Hinter;
        let history = rustyline::history::DefaultHistory::new();
        let ctx = rustyline::Context::new(&history);
        let names = Names::new(Interp::new().globals());
        assert_eq!(names.hint("x.upp", 5, &ctx).as_deref(), Some("er"));
        assert_eq!(names.hint("x.upp", 3, &ctx), None);
        let line = names.highlight("if x 5 \"s\"", 0);
        assert_eq!(line, format!("{MAGENTA}if{RESET} x {CYAN}5{RESET} {GREEN}\"s\"{RESET}"));
    }
}
