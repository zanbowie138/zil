mod ast;
mod help;
mod interp;
mod lexer;
mod modules;
mod parser;

use ariadne::{Label, Report, ReportKind, Source};
use interp::{Interp, Value};
use lexer::{Span, Tok};
use rustyline::error::ReadlineError;
use std::path::PathBuf;
use std::process::exit;

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

fn repl(interp: &mut Interp) {
    let mut rl = rustyline::DefaultEditor::new().expect("terminal");
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
