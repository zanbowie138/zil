//! The REPL: line editing with completion, hints and syntax colors, multi-line input, `help` shorthand and numbered results.

use crate::ansi::{DIM, RESET, color_on, highlight};
use crate::interp::{self, Interp};
use crate::lexer::{self, Tok};
use crate::session::{self, Shown, help_shorthand, unclosed};
use crate::value::Value;
use crate::{ast, cache_dir, help, modules, pager, parser, report, run};
use rustyline::error::ReadlineError;
use std::borrow::Cow;

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

/// Help shaped for stdout, `(color, width)`, if `src` is help printed straight to it (see `session::help_call`).
pub fn help_term(src: &str) -> Option<(bool, usize)> {
    let (terminal_size::Width(w), _) = terminal_size::terminal_size_of(std::io::stdout())?;
    session::help_call(src).then_some((color_on(), w as usize))
}

impl rustyline::validate::Validator for Names {}
impl rustyline::Helper for Names {}

/// Alt+Q: drop the line being typed but keep it in history, to come back to with Up.
struct Stash(std::sync::Arc<std::sync::Mutex<Option<String>>>);

impl rustyline::ConditionalEventHandler for Stash {
    fn handle(&self, _: &rustyline::Event, _: rustyline::RepeatCount, _: bool, ctx: &rustyline::EventContext) -> Option<rustyline::Cmd> {
        *self.0.lock().unwrap() = Some(ctx.line().to_string());
        Some(rustyline::Cmd::Interrupt)
    }
}

pub fn repl(interp: &mut Interp) {
    let config = rustyline::Config::builder().completion_type(rustyline::CompletionType::List).build();
    let mut rl = rustyline::Editor::with_config(config).expect("terminal");
    rl.set_helper(Some(Names::new(interp.globals())));
    let stashed = std::sync::Arc::default();
    rl.bind_sequence(
        rustyline::KeyEvent(rustyline::KeyCode::Char('q'), rustyline::Modifiers::ALT),
        rustyline::EventHandler::Conditional(Box::new(Stash(std::sync::Arc::clone(&stashed)))),
    );
    let history = cache_dir().map(|d| d.join("history.txt"));
    if let Some(h) = &history {
        let _ = rl.load_history(h);
    }
    let (dim, reset) = if color_on() { (DIM, RESET) } else { ("", "") };
    println!("zil {} {dim}· help for docs, Alt-Q to set a line aside, exit or Ctrl-D to quit{reset}", env!("CARGO_PKG_VERSION"));
    println!("{}\n", help::tip(color_on()));
    let mut buf = String::new();
    let mut n = 0;
    loop {
        let line = match rl.readline(if buf.is_empty() { "> " } else { ". " }) {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) => {
                if let Some(line) = stashed.lock().unwrap().take() {
                    buf.push_str(&line);
                    if !buf.trim().is_empty() {
                        let _ = rl.add_history_entry(buf.trim_end());
                    }
                }
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
            Ok(v) => match session::record(interp, &mut n, &v) {
                Shown::Hidden => {}
                Shown::Bare if help_term(&src).is_some() => pager::page(&v.to_string()),
                Shown::Bare => println!("{v}"),
                Shown::Numbered(k) => println!("{}", session::show(k, &v, color_on())),
            },
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
