//! The browser build, for the sandbox and the docs' run buttons. `pnpm wasm` builds it; `docs/sandbox/worker.js` calls it.

use crate::ansi::{DIM, RESET, strip_ansi, tint, to_html};
use crate::ast::ExprKind;
use crate::interp::Interp;
use crate::session::{self, Shown};
use crate::value::Value;
use crate::{error, help, parser};
use std::cell::RefCell;
use wasm_bindgen::prelude::wasm_bindgen;

thread_local! {
    /// What the program printed, since the last run took it.
    static OUT: RefCell<String> = const { RefCell::new(String::new()) };
    /// The sandbox's REPL: one interpreter for the whole session, and how many results it has numbered.
    static SESSION: RefCell<(Interp, usize)> = RefCell::new((Interp::new(), 0));
}

pub fn print(s: &str) {
    OUT.with_borrow_mut(|o| {
        o.push_str(s);
        o.push('\n');
    })
}

/// A value as the REPL shows it, in color; nothing for nil and functions.
fn shown(v: &Value) -> String {
    match v {
        Value::Nil | Value::Fn(_) => String::new(),
        Value::Table(t) => crate::modules::data::tables::grid(t, true),
        Value::Str(s) if s.contains(['\n', '\x1b']) => s.to_string(),
        v => format!("{}{v:?}{RESET}", tint(v)),
    }
}

/// Runs a script in a fresh interpreter: `[printed, result, error]`, each HTML and possibly empty.
#[wasm_bindgen]
pub fn run(src: &str) -> Vec<String> {
    OUT.take();
    let (result, err) = match crate::run(&mut Interp::new(), src) {
        Ok(v) => (shown(&v), String::new()),
        Err(e) => (String::new(), error::render(&e, "sandbox", src, true)),
    };
    [OUT.take(), result, err].iter().map(|s| to_html(s.trim_end())).collect()
}

/// Runs a docs code block one top-level statement at a time in one interpreter, like typing it into the REPL.
/// Returns JSON `[[line, text], ...]`: plain text to show after each 0-based `line` that ends a statement with output.
/// Assignments and loops stay quiet, so a recipe shows only what it prints and its last value.
#[wasm_bindgen]
pub fn notebook(src: &str) -> String {
    let last_line = |end: usize| src[..end].trim_end().matches('\n').count();
    let rows: Vec<(usize, String)> = match parser::parse(src) {
        Err(e) => vec![(last_line(src.len()), format!("error: {}", e.msg))],
        Ok(prog) => {
            let mut interp = Interp::new();
            OUT.take();
            prog.iter()
                .enumerate()
                .filter_map(|(i, stmt)| {
                    let r = interp.run(&prog[i..=i]);
                    let line = last_line(stmt.span.end);
                    // `x = 5; x += 2; x` shows one result, like a REPL line.
                    let quiet = matches!(stmt.kind, ExprKind::Assign(..) | ExprKind::Unpack(..) | ExprKind::For(..) | ExprKind::While(..))
                        || prog.get(i + 1).is_some_and(|next| last_line(next.span.end) == line);
                    let printed = OUT.take();
                    let value = match r {
                        // `"abc".find("z")` → nil is an answer; after `print` it's just noise.
                        Ok(Value::Nil) if !quiet && printed.is_empty() => "nil".into(),
                        Ok(v) if !quiet => strip_ansi(&shown(&v)),
                        Ok(_) => String::new(),
                        Err(e) => format!("error: {}", e.msg),
                    };
                    let text = (printed + &value).trim_end().to_string();
                    (!text.is_empty()).then_some((line, text))
                })
                .collect()
        }
    };
    serde_json::to_string(&rows).unwrap()
}

/// The sandbox's sidebar: `help("examples")`'s sections as JSON `[[top-level module, title, [[label, code], ...]], ...]`, highlights first.
#[wasm_bindgen]
pub fn examples() -> String {
    let highlights = crate::guide::HIGHLIGHTS.iter().map(|s| ("start", s));
    let sections = highlights.chain(crate::modules::modules().flat_map(|m| m.examples.iter().map(|s| (crate::modules::path(m).split('.').next().unwrap(), s))));
    let rows: Vec<_> = sections.map(|(module, (title, rows))| (module, title, rows.iter().filter(|(_, code)| !code.is_empty()).collect::<Vec<_>>())).collect();
    serde_json::to_string(&rows).unwrap()
}

/// Starts a fresh REPL session, returning its greeting as HTML.
#[wasm_bindgen]
pub fn repl_reset() -> String {
    SESSION.set((Interp::new(), 0));
    to_html(&format!(
        "zil {} {DIM}· help for docs{RESET}
{}",
        env!("CARGO_PKG_VERSION"),
        help::tip(true)
    ))
}

/// Whether Enter should continue `src` on a new line instead of running it.
#[wasm_bindgen]
pub fn repl_open(src: &str) -> bool {
    session::unclosed(src)
}

/// Runs one REPL input in the session, as the terminal REPL would: `[printed, shown, error]` as HTML, like `run`.
#[wasm_bindgen]
pub fn repl(src: &str) -> Vec<String> {
    OUT.take();
    let (shown, err) = match session::help_shorthand(src) {
        Some(topic) => (help::help(topic, true).unwrap_or_else(|e| e), String::new()),
        // Like the terminal REPL, a bare `help(...)` or `tip()` comes back colored; the log scrolls, so nothing wraps.
        None => SESSION.with_borrow_mut(|(interp, n)| match help::live(session::help_call(src).then_some((true, usize::MAX)), || crate::run(interp, src)) {
            Ok(v) => match session::record(interp, n, &v) {
                Shown::Hidden => (String::new(), String::new()),
                Shown::Bare => (v.to_string(), String::new()),
                Shown::Numbered(k) => (session::show(k, &v, true), String::new()),
            },
            Err(e) => (String::new(), error::render(&e, "<repl>", src, true)),
        }),
    };
    [OUT.take(), shown, err].iter().map(|s| to_html(s.trim_end())).collect()
}
