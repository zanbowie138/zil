//! zil: an expression calculator and scripting language. The CLI lives in `cli`, the browser build in `wasm`.
// The browser build has no REPL, so helpers only it uses go unused there.
#![cfg_attr(target_arch = "wasm32", allow(dead_code))]

mod ansi;
mod ast;
#[cfg(not(target_arch = "wasm32"))]
pub mod cli;
#[cfg(not(target_arch = "wasm32"))]
mod docs;
mod error;
mod guide;
mod help;
mod interp;
mod lexer;
mod modules;
mod ops;
#[cfg(not(target_arch = "wasm32"))]
mod pager;
mod parser;
#[cfg(not(target_arch = "wasm32"))]
mod repl;
mod session;
mod signature;
mod value;
#[cfg(target_arch = "wasm32")]
mod wasm;

/// No terminal to page in the browser: `page(...)` just prints.
#[cfg(target_arch = "wasm32")]
mod pager {
    pub fn page(text: &str) {
        crate::print(text)
    }
}

use interp::Interp;
use std::path::PathBuf;
use value::Value;

pub use error::Error;
#[cfg(target_arch = "wasm32")]
use wasm::print;

/// A line of program output, from `print` and friends.
#[cfg(not(target_arch = "wasm32"))]
fn print(s: &str) {
    println!("{s}")
}

/// Per-user cache for currency rates and REPL history.
pub fn cache_dir() -> Option<PathBuf> {
    let env = |k| std::env::var_os(k).map(PathBuf::from);
    let base = env("XDG_CACHE_HOME").or_else(|| env("LOCALAPPDATA")).or_else(|| Some(env("HOME")?.join(".cache")))?;
    Some(base.join("zil"))
}

fn run(interp: &mut Interp, src: &str) -> Result<Value, Error> {
    interp.run(&parser::parse(src)?)
}

#[cfg(not(target_arch = "wasm32"))]
fn report(name: &str, src: &str, e: Error) {
    use std::io::IsTerminal;
    let color = std::io::stderr().is_terminal() && std::env::var_os("NO_COLOR").is_none();
    eprint!("{}", error::render(&e, name, src, color));
}

/// Interpreter thread stack: deep enough for `interp::MAX_DEPTH` nested calls in a debug build.
/// Only reserved, not committed, so the size costs nothing until used. The browser build sets its own in `.cargo/config.toml`.
#[cfg(not(target_arch = "wasm32"))]
pub const STACK: usize = 1 << 30;
