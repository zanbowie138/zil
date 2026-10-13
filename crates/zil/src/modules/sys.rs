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
        ("env", []) => Value::map(crate::host::get().env_all().into_iter().map(|(k, v)| (k, Value::str(v))).collect()),
        ("env", [Str(k)]) => crate::host::get().env(k).map_or(Nil, Value::str),
        ("exit", []) => return Err(crate::host::get().exit(0).into()),
        ("exit", [Int(n, _)]) => return Err(crate::host::get().exit(*n as i32).into()),
        ("sh" | "run", [Str(cmd)]) => {
            let out = crate::host::get().run(cmd).map_err(|e| format!("cannot run {cmd:?}: {}", super::fs::io_reason(&e)))?;
            let text = |s: &str| s.trim_end_matches(['\n', '\r']).to_string();
            let (out_s, err_s, code) = (text(&out.out), text(&out.err), out.code);
            if name == "run" {
                let m: IndexMap<_, _> = [("code", Value::int(code.into())), ("out", Value::str(out_s)), ("err", Value::str(err_s))]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v))
                    .collect();
                return Ok(Value::map(m));
            }
            if code != 0 {
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

/// `key`'s entry in the host's cache `file`, else `fetch`ed and cached for next time; `fetch` checks `network_allowed` itself.
// ponytail: rereads and rewrites the whole cache file per call; keep it in memory if it grows large.
pub fn cached(file: &str, key: &str, fetch: impl FnOnce() -> Result<String, String>) -> Result<String, String> {
    let host = crate::host::get();
    let mut cache: serde_json::Map<String, serde_json::Value> = host.cache_get(file).and_then(|(s, _)| serde_json::from_str(&s).ok()).unwrap_or_default();
    if let Some(hit) = cache.get(key).and_then(|v| v.as_str()) {
        return Ok(hit.into());
    }
    let out = fetch()?;
    cache.insert(key.into(), out.clone().into());
    host.cache_put(file, &serde_json::to_string(&cache).unwrap());
    Ok(out)
}

/// HTTP GET `url` with `query` parameters, returning the body.
pub fn get(url: &str, query: &[(&str, &str)]) -> Result<String, String> {
    crate::host::get().http_get(url, query)
}

#[cfg(test)]
mod tests {
    use crate::interp::tests::show;

    #[test]
    fn sys() {
        assert_eq!(show("args()"), "[]");
        assert_eq!(show(r#"env("ZIL_SURELY_UNSET")"#), "nil");
        assert_eq!(show("allow_network_access()"), "nil");
    }
}
