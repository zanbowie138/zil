//! The `zil` command line: REPL, `-e`, scripts and `--docs`.

use crate::interp::Interp;
use crate::value::Value;
use crate::{Error, STACK, ansi, docs, help, modules, repl, report};
use std::process::exit;

/// Run `src` and exit with status 1 on error.
fn run_or_exit(interp: &mut Interp, name: &str, src: &str) -> Value {
    crate::run(interp, src).unwrap_or_else(|e: Error| {
        report(name, src, e);
        exit(1)
    })
}

const USAGE: &str = "usage: zil                        start the REPL
       zil -e <code> [args...]   evaluate code and print the result
       zil <file.zil> [args...]  run a script; args() lists the extra arguments
       zil --docs <dir>          write the mdBook reference into dir
       zil --version             print the version";

pub fn main() {
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
