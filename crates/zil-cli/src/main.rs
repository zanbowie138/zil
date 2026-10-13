//! The `zil` command line: REPL, `-e`, scripts and `--docs`.

mod docs;
mod repl;

use std::process::exit;
use zil::interp::Interp;
use zil::value::Value;
use zil::{Error, STACK, ansi, help, modules};

/// Run `src` and exit with status 1 on error.
fn run_or_exit(interp: &mut Interp, name: &str, src: &str) -> Value {
    zil::run(interp, src).unwrap_or_else(|e: Error| {
        report(name, src, e);
        exit(1)
    })
}

const USAGE: &str = "usage: zil                        start the REPL
       zil -e <code> [args...]   evaluate code and print the result
       zil <file.zil> [args...]  run a script; args() lists the extra arguments
       zil --version             print the version";

fn main() {
    // The legacy console (e.g. a Start menu shortcut) shows ANSI colors as raw codes until asked not to.
    #[cfg(windows)]
    crossterm::ansi_support::supports_ansi();
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
        [flag] if flag == "-V" || flag == "--version" => println!("zil {}", env!("CARGO_PKG_VERSION")),
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

/// Prints `e` to stderr, colored when stderr is a terminal.
fn report(name: &str, src: &str, e: Error) {
    use std::io::IsTerminal;
    let color = std::io::stderr().is_terminal() && std::env::var_os("NO_COLOR").is_none();
    eprint!("{}", zil::error::render(&e, name, src, color));
}
