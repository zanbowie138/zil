mod ast;
mod builtins;
mod interp;
mod lexer;
mod parser;

use ariadne::{Label, Report, ReportKind, Source};
use interp::{Interp, Value};
use lexer::Span;
use std::io::{BufRead, Write};

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

fn run(interp: &mut Interp, src: &str) -> Result<Value, Error> {
    let ast = parser::parse(lexer::lex(src)?)?;
    interp.run(&ast)
}

fn report(name: &str, src: &str, e: Error) {
    Report::build(ReportKind::Error, (name, e.span.clone()))
        .with_message(&e.msg)
        .with_label(Label::new((name, e.span)).with_message(&e.msg))
        .finish()
        .eprint((name, Source::from(src)))
        .unwrap();
}

fn main() {
    let mut interp = Interp::new();
    if let Some(path) = std::env::args().nth(1) {
        let src = std::fs::read_to_string(&path).unwrap_or_else(|e| {
            eprintln!("zil: {path}: {e}");
            std::process::exit(1);
        });
        if let Err(e) = run(&mut interp, &src) {
            report(&path, &src, e);
            std::process::exit(1);
        }
        return;
    }

    // ponytail: single-line REPL on plain stdin; add rustyline for history/multi-line input.
    let stdin = std::io::stdin();
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();
        let mut line = String::new();
        if stdin.lock().read_line(&mut line).unwrap_or(0) == 0 {
            break;
        }
        match run(&mut interp, &line) {
            Ok(Value::Nil) => {}
            Ok(v) => println!("{v}"),
            Err(e) => report("<repl>", &line, e),
        }
    }
}
