//! The world outside the script: arguments, environment, shell commands, HTTP.

use crate::interp::Interp;
use crate::lexer::Span;
use crate::modules::{Call, Doc, Fail, Module, doc};
use crate::value::Value;
use indexmap::IndexMap;

/// Whether features that go online on their own (currency rates, translation, zip codes) may do so.
pub fn network_allowed() -> bool {
    crate::state::with(|s| s.network.get())
}

pub const MODULE: Module = Module {
    name: "sys",
    about: "script arguments, environment variables, shell commands, HTTP GET",
    #[rustfmt::skip]
    examples: &[
        ("sys", &[
            ("arguments after the script", "args()"),
            ("is a variable set?", r#"env("ZIL_SURELY_UNSET") == nil"#),
            ("how many env vars", "env().len > 0"),
        ]),
    ],
    #[rustfmt::skip]
    guide: &[
        ("scripts", &[
            ("zil script.zil a b  →  args() is [\"a\", \"b\"]", ""),
            ("sh fails on a non-zero exit; run never does", ""),
            ("currency rates, translation and zip codes stay offline until allow_network_access()", ""),
        ]),
    ],
    fns: FNS,
    call,
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("args", "args()", "command-line arguments after the script (or after `-e code`), as strings", &["args()"], &["env"]),
    doc("env", "env() / env(name: str)", "an environment variable, or nil; with no name, all of them as a map", &[r#"env("ZIL_SURELY_UNSET")"#], &["args"]),
    doc("exit", "exit(code?: int)", "stop the program with an exit code (default 0)", &[], &["args"]).shown(&["exit(1)"]),
    doc("sh", "sh(cmd: str)", "run a shell command (sh -c, or cmd /C on Windows) and return its output, trailing newline trimmed; errors on a non-zero exit", &[], &["run"])
        .shown(&[r#"sh("git rev-parse --short HEAD")"#, r#"sh("ls").lines.len"#]),
    doc("run", "run(cmd: str)", "run a shell command and return {code, out, err}; never fails on the exit code", &[], &["sh"]).shown(&[r#"run("git status").code"#]),
    doc("fetch", "fetch(url: str)", "HTTP GET a URL and return the body as a string; errors on a non-2xx status", &[], &["from_json"])
        .shown(&[r#"fetch("https://api.github.com/repos/rust-lang/rust").from_json.stargazers_count"#]),
    doc("allow_network_access", "allow_network_access()", "let currency rates, translation and zip codes go online for the rest of the session; until then they only use cached data", &[], &["fetch"])
        .shown(&["allow_network_access()"]),
];

fn call(it: &mut Interp, name: &'static str, args: &[Value], _: &Span) -> Call {
    use Value::*;
    Ok(match (name, args) {
        ("args", []) => Value::list(it.args.iter().map(|a| Value::str(a.as_str())).collect()),
        // The browser has no environment, and std panics asking for one.
        ("env", []) if cfg!(target_arch = "wasm32") => Value::map(IndexMap::new()),
        ("env", []) => Value::map(std::env::vars().map(|(k, v)| (k, Value::str(v))).collect()),
        ("env", [Str(k)]) => std::env::var(&**k).map_or(Nil, Value::str),
        ("exit", _) if cfg!(target_arch = "wasm32") => return Err("there's no process to exit in the browser".into()),
        ("exit", []) => std::process::exit(0),
        ("exit", [Int(n, _)]) => std::process::exit(*n as i32),
        ("sh" | "run", [Str(cmd)]) => {
            let out = shell(cmd).output().map_err(|e| format!("cannot run {cmd:?}: {e}"))?;
            let text = |b: &[u8]| String::from_utf8_lossy(b).trim_end_matches(['\n', '\r']).to_string();
            let (out_s, err_s) = (text(&out.stdout), text(&out.stderr));
            let code = out.status.code().unwrap_or(-1);
            if name == "run" {
                let m: IndexMap<_, _> = [("code", Value::int(code.into())), ("out", Value::str(out_s)), ("err", Value::str(err_s))]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v))
                    .collect();
                return Ok(Value::map(m));
            }
            if !out.status.success() {
                return Err(format!("{cmd:?} exited with {code}{}", if err_s.is_empty() { String::new() } else { format!("\n{err_s}") }).into());
            }
            Value::str(out_s)
        }
        ("fetch", [Str(url)]) => Value::str(get(url, &[]).map_err(|e| format!("{url}: {e}"))?),
        ("allow_network_access", []) => {
            crate::state::with(|s| s.network.set(true));
            crate::modules::units::retry_rates();
            Nil
        }
        _ => return Err(Fail::BadArgs),
    })
}

/// `key`'s entry in the disk cache `file`, else `fetch`ed and cached for next time; `fetch` checks `network_allowed` itself.
// ponytail: rereads and rewrites the whole cache file per call; keep it in memory if it grows large.
pub fn cached(file: &str, key: &str, fetch: impl FnOnce() -> Result<String, String>) -> Result<String, String> {
    let path = crate::cache_dir().map(|d| d.join(file));
    let mut cache: serde_json::Map<String, serde_json::Value> =
        path.as_ref().and_then(|p| std::fs::read_to_string(p).ok()).and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
    if let Some(hit) = cache.get(key).and_then(|v| v.as_str()) {
        return Ok(hit.into());
    }
    let out = fetch()?;
    if let Some(p) = &path {
        cache.insert(key.into(), out.clone().into());
        let _ = std::fs::create_dir_all(p.parent().unwrap());
        let _ = std::fs::write(p, serde_json::to_string(&cache).unwrap());
    }
    Ok(out)
}

/// HTTP GET `url` with `query` parameters, returning the body.
pub fn get(url: &str, query: &[(&str, &str)]) -> Result<String, String> {
    #[cfg(not(target_arch = "wasm32"))]
    return ureq::get(url).query_pairs(query.iter().copied()).call().and_then(|mut r| r.body_mut().read_to_string()).map_err(|e| e.to_string());
    #[cfg(target_arch = "wasm32")]
    return zil_get(url, &serde_json::to_string(query).unwrap()).map_err(|e| e.as_string().unwrap_or_else(|| "network error".into()));
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    /// A blocking GET, defined by `docs/sandbox/worker.js`: `query` is JSON `[[key, value], ...]`, and failures throw a message.
    #[wasm_bindgen(catch, js_name = zilGet)]
    fn zil_get(url: &str, query: &str) -> Result<String, wasm_bindgen::JsValue>;
}

fn shell(cmd: &str) -> std::process::Command {
    let (sh, flag) = if cfg!(windows) { ("cmd", "/C") } else { ("sh", "-c") };
    let mut c = std::process::Command::new(sh);
    c.args([flag, cmd]);
    c
}

#[cfg(test)]
mod tests {
    use crate::interp::tests::{show, try_eval};

    #[test]
    fn sys() {
        assert_eq!(show("args()"), "[]");
        assert_eq!(show(r#"env("ZIL_SURELY_UNSET")"#), "nil");
        assert_eq!(show(r#"sh("echo hi")"#), "hi");
        assert_eq!(show(r#"run("exit 3").code"#), "3");
        assert_eq!(show("allow_network_access()"), "nil");
        assert!(try_eval(r#"sh("exit 3")"#).is_err());
    }
}
