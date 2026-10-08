//! zil: an expression calculator and scripting language. CLI entry point.

mod ansi;
mod ast;
mod docs;
mod error;
mod guide;
mod help;
mod interp;
mod lexer;
mod modules;
mod ops;
mod pager;
mod parser;
mod repl;
mod signature;
mod value;

use interp::Interp;
use std::path::PathBuf;
use std::process::exit;
use value::Value;

pub use error::Error;

/// Per-user cache for currency rates and REPL history.
pub fn cache_dir() -> Option<PathBuf> {
    let env = |k| std::env::var_os(k).map(PathBuf::from);
    let base = env("XDG_CACHE_HOME").or_else(|| env("LOCALAPPDATA")).or_else(|| Some(env("HOME")?.join(".cache")))?;
    Some(base.join("zil"))
}

fn run(interp: &mut Interp, src: &str) -> Result<Value, Error> {
    interp.run(&parser::parse(src)?)
}

fn report(name: &str, src: &str, e: Error) {
    use std::io::IsTerminal;
    let color = std::io::stderr().is_terminal() && std::env::var_os("NO_COLOR").is_none();
    eprint!("{}", error::render(&e, name, src, color));
}

/// Run `src` and exit with status 1 on error.
fn run_or_exit(interp: &mut Interp, name: &str, src: &str) -> Value {
    run(interp, src).unwrap_or_else(|e| {
        report(name, src, e);
        exit(1)
    })
}

const USAGE: &str = "usage: zil                        start the REPL
       zil -e <code> [args...]   evaluate code and print the result
       zil <file.zil> [args...]  run a script; args() lists the extra arguments
       zil --docs <dir>          write the mdBook reference into dir";

/// Interpreter thread stack: deep enough for `interp::MAX_DEPTH` nested calls in a debug build.
/// Only reserved, not committed, so the size costs nothing until used.
pub const STACK: usize = 1 << 30;

fn main() {
    let t = std::thread::Builder::new().stack_size(STACK).spawn(real_main).expect("spawn interpreter thread");
    if t.join().is_err() {
        exit(101)
    }
}

fn real_main() {
    let mut interp = Interp::new();
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [] => repl::repl(&mut interp),
        [flag, code, rest @ ..] if flag == "-e" => {
            interp.args = rest.to_vec();
            match help::live(repl::help_term(code), || run_or_exit(&mut interp, "<-e>", code)) {
                Value::Nil => {}
                Value::Table(t) => println!("{}", modules::data::tables::grid(&t, ansi::color_on())),
                v => println!("{v}"),
            }
        }
        [flag, dir] if flag == "--docs" => docs::write(dir.as_ref()).unwrap_or_else(|e| {
            eprintln!("zil: {dir}: {e}");
            exit(1)
        }),
        [flag] if flag == "-h" || flag == "--help" => println!("{USAGE}"),
        [path, rest @ ..] if !path.starts_with('-') => {
            interp.args = rest.to_vec();
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
