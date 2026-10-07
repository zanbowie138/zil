mod ast;
mod help;
mod interp;
mod lexer;
mod modules;
mod parser;
mod value;

use ariadne::{Label, Report, ReportKind, Source};
use interp::Interp;
use lexer::{Span, Tok};
use rustyline::error::ReadlineError;
use std::path::PathBuf;
use std::process::exit;
use value::Value;

#[derive(Debug)]
pub struct Error {
    pub msg: String,
    pub span: Span,
}

impl Error {
    pub fn new(msg: impl Into<String>, span: Span) -> Error {
        Error { msg: msg.into(), span }
    }
}

/// Per-user cache for currency rates and REPL history.
pub fn cache_dir() -> Option<PathBuf> {
    let env = |k| std::env::var_os(k).map(PathBuf::from);
    let base = env("XDG_CACHE_HOME").or_else(|| env("LOCALAPPDATA")).or_else(|| Some(env("HOME")?.join(".cache")))?;
    Some(base.join("zil"))
}

fn run(interp: &mut Interp, src: &str) -> Result<Value, Error> {
    let ast = parser::parse(lexer::lex(src)?)?;
    interp.run(&ast)
}

fn report(name: &str, src: &str, e: Error) {
    let _ = Report::build(ReportKind::Error, (name, e.span.clone()))
        .with_message(&e.msg)
        .with_label(Label::new((name, e.span)).with_message(&e.msg))
        .finish()
        .eprint((name, Source::from(src)));
}

/// Run `src` and exit with status 1 on error.
fn run_or_exit(interp: &mut Interp, name: &str, src: &str) -> Value {
    run(interp, src).unwrap_or_else(|e| {
        report(name, src, e);
        exit(1)
    })
}

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

/// REPL `help` / `help topic` rewritten to a call; `None` for ordinary code.
fn help_shorthand(src: &str) -> Option<String> {
    let rest = src.trim().strip_prefix("help")?;
    let topic = rest.trim_start();
    if topic.is_empty() {
        return Some("help()".into());
    }
    let word = topic.chars().all(|c| c.is_alphanumeric() || c == '_');
    (rest.starts_with(char::is_whitespace) && word).then(|| format!("help({topic:?})"))
}

/// REPL tab completion over builtins, constants, `to` targets, units and help topics.
// ponytail: names fixed at startup, so user variables don't complete; pass the interp's globals in if wanted.
struct Names(Vec<String>);

impl Names {
    fn new() -> Names {
        let ms = modules::MODULES;
        let mut v: Vec<String> = ms.iter().flat_map(|m| m.fns.iter().map(|f| f.name).chain(m.consts.iter().map(|c| c.0)).chain(m.targets.iter().map(|t| t.0)).chain([m.name])).map(String::from).collect();
        v.extend(modules::units::TABLE.iter().flat_map(|u| u.0.split_whitespace()).map(String::from));
        v.extend(modules::units::DIMS.iter().map(|d| d.0.to_string()));
        v.sort();
        v.dedup();
        Names(v)
    }
}

impl rustyline::completion::Completer for Names {
    type Candidate = String;
    fn complete(&self, line: &str, pos: usize, _: &rustyline::Context<'_>) -> rustyline::Result<(usize, Vec<String>)> {
        let start = pos - line[..pos].chars().rev().take_while(|c| c.is_alphanumeric() || *c == '_').map(char::len_utf8).sum::<usize>();
        let word = &line[start..pos];
        let hits = match word.is_empty() {
            true => vec![],
            false => self.0.iter().filter(|n| n.starts_with(word)).cloned().collect(),
        };
        Ok((start, hits))
    }
}

impl rustyline::hint::Hinter for Names {
    type Hint = String;
}
impl rustyline::highlight::Highlighter for Names {}
impl rustyline::validate::Validator for Names {}
impl rustyline::Helper for Names {}

fn repl(interp: &mut Interp) {
    let config = rustyline::Config::builder().completion_type(rustyline::CompletionType::List).build();
    let mut rl = rustyline::Editor::with_config(config).expect("terminal");
    rl.set_helper(Some(Names::new()));
    let history = cache_dir().map(|d| d.join("history.txt"));
    if let Some(h) = &history {
        let _ = rl.load_history(h);
    }
    let mut buf = String::new();
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
        let mut src = std::mem::take(&mut buf);
        if src.trim().is_empty() {
            continue;
        }
        let _ = rl.add_history_entry(src.trim_end());
        if let Some(call) = help_shorthand(&src) {
            src = call;
        }
        match run(interp, &src) {
            Ok(Value::Nil | Value::Fn(_)) => {}
            Ok(v) => {
                println!("{v}");
                interp.set_global("_", v);
            }
            Err(e) => report("<repl>", &src, e),
        }
    }
    if let Some(h) = &history {
        let _ = std::fs::create_dir_all(h.parent().unwrap());
        let _ = rl.save_history(h);
    }
}

const USAGE: &str = "usage: zil              start the REPL
       zil -e <code>    evaluate code and print the result
       zil <file.zil>   run a script";

fn main() {
    let mut interp = Interp::new();
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [] => repl(&mut interp),
        [flag, code] if flag == "-e" => match run_or_exit(&mut interp, "<-e>", code) {
            Value::Nil => {}
            v => println!("{v}"),
        },
        [flag] if flag == "-h" || flag == "--help" => println!("{USAGE}"),
        [path] => {
            let src = std::fs::read_to_string(path).unwrap_or_else(|e| {
                eprintln!("zil: {path}: {e}");
                exit(1)
            });
            run_or_exit(&mut interp, path, &src);
        }
        _ => {
            eprintln!("{USAGE}");
            exit(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustyline::completion::Completer;

    #[test]
    fn completes_names_at_cursor() {
        let history = rustyline::history::DefaultHistory::new();
        let ctx = rustyline::Context::new(&history);
        let names = Names::new();
        assert_eq!(names.complete("x.upp", 5, &ctx).unwrap(), (2, vec!["upper".to_string()]));
        assert!(names.complete("5 kilo", 6, &ctx).unwrap().1.contains(&"kilometers".to_string()));
        assert!(names.complete("x ", 2, &ctx).unwrap().1.is_empty());
    }
}
