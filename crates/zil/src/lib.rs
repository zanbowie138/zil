//! zil: a terse calculator and scripting language for units, dates and much more. The CLI is the `zil-cli` crate, the browser build `zil-wasm`.

// Internals the CLI and the browser build reach into, until the Session API (roadmap Phase 3a) replaces them.
#[doc(hidden)]
pub mod ansi;
#[doc(hidden)]
pub mod ast;
mod data;
#[doc(hidden)]
pub mod error;
#[doc(hidden)]
pub mod guide;
#[doc(hidden)]
pub mod help;
pub mod host;
#[doc(hidden)]
pub mod interp;
#[doc(hidden)]
pub mod lexer;
#[doc(hidden)]
pub mod modules;
mod ops;
#[cfg(not(target_arch = "wasm32"))]
#[doc(hidden)]
pub mod pager;
#[doc(hidden)]
pub mod parser;
#[doc(hidden)]
pub mod repl;
mod session;
mod signature;
#[doc(hidden)]
pub mod state;
#[doc(hidden)]
pub mod value;

use interp::Interp;
use std::path::PathBuf;

pub use data::Data;
pub use error::{Error, Limit};
pub use interp::Limits;
pub use session::Session;
pub use value::Value;

/// A line of program output, from `print` and friends.
fn print(s: &str) {
    host::get().print(s)
}

/// Per-user cache for currency rates and REPL history. A `zil-data` folder beside the executable takes its place:
/// that is the portable build, which leaves nothing behind outside its own folder.
pub fn cache_dir() -> Option<PathBuf> {
    if let Some(dir) = std::env::current_exe().ok().and_then(|exe| Some(exe.parent()?.join("zil-data"))).filter(|d| d.is_dir()) {
        return Some(dir);
    }
    let env = |k| std::env::var_os(k).map(PathBuf::from);
    let base = env("XDG_CACHE_HOME").or_else(|| env("LOCALAPPDATA")).or_else(|| Some(env("HOME")?.join(".cache")))?;
    Some(base.join("zil"))
}

#[doc(hidden)]
pub fn run(interp: &mut Interp, src: &str) -> Result<Value, Error> {
    interp.run(&parser::parse(src)?)
}

/// Interpreter thread stack: deep enough for `interp::MAX_DEPTH` nested calls in a debug build.
/// Only reserved, not committed, so the size costs nothing until used. The browser build sets its own in `.cargo/config.toml`.
#[cfg(not(target_arch = "wasm32"))]
pub const STACK: usize = 1 << 30;
