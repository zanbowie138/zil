//! Everything a session does to the world outside it goes through its `Host`: output, input, the clock, randomness,
//! the environment, files, processes, the network, the cache and exiting. A method the host doesn't implement is
//! denied, so `NullHost` is a session that touches nothing. Fresh interpreters take the running session's host.

use crate::state;
use jiff::tz::TimeZone;
use jiff::{Timestamp, Zoned};
use std::io;
use std::rc::Rc;

/// What a file or directory looks like, for `ls`, `file_size` and friends.
pub struct Meta {
    pub is_dir: bool,
    pub len: u64,
    pub modified: Timestamp,
}

/// A finished shell command: its exit code (-1 if killed) and output.
pub struct Ran {
    pub code: i32,
    pub out: String,
    pub err: String,
}

fn denied<T>(what: &str) -> io::Result<T> {
    Err(io::Error::new(io::ErrorKind::PermissionDenied, format!("{what} is disabled in this session")))
}

pub trait Host {
    /// A line of program output, from `print` and friends.
    fn print(&self, _text: &str) {}
    /// Long output from `page`, which a terminal can scroll through.
    fn page(&self, text: &str) {
        self.print(text)
    }
    /// A side note for the user that isn't program output, like a stale cache warning.
    fn note(&self, _text: &str) {}
    /// Columns and rows when output goes to a terminal that understands ANSI escapes.
    fn terminal(&self) -> Option<(usize, usize)> {
        None
    }
    /// Pauses between `animate` frames.
    fn sleep(&self, _secs: f64) {}
    /// All of stdin, for `input`.
    fn stdin(&self) -> io::Result<String> {
        denied("reading input")
    }
    fn now(&self) -> Timestamp {
        Timestamp::UNIX_EPOCH
    }
    fn time_zone(&self) -> TimeZone {
        TimeZone::UTC
    }
    /// Seeds the session's random numbers once, when it starts.
    fn seed(&self) -> u64 {
        0
    }
    fn env(&self, _name: &str) -> Option<String> {
        None
    }
    fn env_all(&self) -> Vec<(String, String)> {
        vec![]
    }
    fn read(&self, _path: &str) -> io::Result<Vec<u8>> {
        denied("file access")
    }
    fn write(&self, _path: &str, _bytes: &[u8], _append: bool) -> io::Result<()> {
        denied("file access")
    }
    /// The names in a directory, in any order.
    fn list(&self, _dir: &str) -> io::Result<Vec<String>> {
        denied("file access")
    }
    /// Follows symlinks, but describes a broken one itself.
    fn metadata(&self, _path: &str) -> io::Result<Meta> {
        denied("file access")
    }
    /// A file or an empty directory.
    fn remove(&self, _path: &str) -> io::Result<()> {
        denied("file access")
    }
    fn rename(&self, _from: &str, _to: &str) -> io::Result<()> {
        denied("file access")
    }
    fn copy(&self, _from: &str, _to: &str) -> io::Result<()> {
        denied("file access")
    }
    /// A directory and any missing parents.
    fn mkdir(&self, _path: &str) -> io::Result<()> {
        denied("file access")
    }
    fn cwd(&self) -> io::Result<String> {
        denied("file access")
    }
    fn cd(&self, _dir: &str) -> io::Result<()> {
        denied("file access")
    }
    /// Runs `cmd` in the system shell.
    fn run(&self, _cmd: &str) -> io::Result<Ran> {
        denied("running commands")
    }
    /// HTTP GET `url` with `query` parameters, returning the body.
    fn http_get(&self, _url: &str, _query: &[(&str, &str)]) -> Result<String, String> {
        Err("network access is disabled in this session".into())
    }
    /// A cache entry and when it was put there.
    fn cache_get(&self, _key: &str) -> Option<(String, Timestamp)> {
        None
    }
    fn cache_put(&self, _key: &str, _value: &str) {}
    /// Ends the process; returns only if it can't, with why.
    fn exit(&self, _code: i32) -> String {
        "exit is disabled in this session".into()
    }
}

/// Denies everything: no output, a clock stuck at 1970 UTC, and the same random numbers every time.
pub struct NullHost;
impl Host for NullHost {}

/// The host of a thread's first session: the real machine natively, nothing in the browser until `zil-wasm` installs its own.
pub fn default() -> Rc<dyn Host> {
    #[cfg(not(target_arch = "wasm32"))]
    return Rc::new(NativeHost);
    #[cfg(target_arch = "wasm32")]
    return Rc::new(NullHost);
}

pub use crate::state::set_default_host as set_default;

/// The running session's host.
pub fn get() -> Rc<dyn Host> {
    state::with(|s| s.host.clone())
}

/// The time now, in the host's time zone.
pub fn now() -> Zoned {
    let h = get();
    h.now().to_zoned(h.time_zone())
}

pub fn timestamp() -> Timestamp {
    get().now()
}

pub fn tz() -> TimeZone {
    get().time_zone()
}

/// The running session's random numbers, seeded by its host.
pub fn rng<T>(f: impl FnOnce(&mut fastrand::Rng) -> T) -> T {
    state::with(|s| f(&mut s.rng.borrow_mut()))
}

/// The machine zil runs on, with nothing held back: what the CLI and the tests use.
#[cfg(not(target_arch = "wasm32"))]
pub struct NativeHost;

#[cfg(not(target_arch = "wasm32"))]
impl Host for NativeHost {
    fn print(&self, text: &str) {
        println!("{text}")
    }
    fn page(&self, text: &str) {
        crate::pager::page(text)
    }
    fn note(&self, text: &str) {
        eprintln!("{text}")
    }
    // Tests never see a terminal, wherever they run.
    fn terminal(&self) -> Option<(usize, usize)> {
        use io::IsTerminal;
        let (w, h) = terminal_size::terminal_size()?;
        (!cfg!(test) && io::stdout().is_terminal()).then_some((w.0 as usize, h.0 as usize))
    }
    fn sleep(&self, secs: f64) {
        std::thread::sleep(std::time::Duration::from_secs_f64(secs))
    }
    fn stdin(&self) -> io::Result<String> {
        io::read_to_string(io::stdin())
    }
    // The tests run at a fixed moment in Chicago, whatever the machine's clock and zone.
    fn now(&self) -> Timestamp {
        if cfg!(test) { "2026-10-06T14:30:00Z".parse().unwrap() } else { Timestamp::now() }
    }
    fn time_zone(&self) -> TimeZone {
        if cfg!(test) { TimeZone::get("America/Chicago").unwrap() } else { TimeZone::system() }
    }
    fn seed(&self) -> u64 {
        fastrand::u64(..)
    }
    fn env(&self, name: &str) -> Option<String> {
        std::env::var(name).ok()
    }
    fn env_all(&self) -> Vec<(String, String)> {
        std::env::vars().collect()
    }
    fn read(&self, path: &str) -> io::Result<Vec<u8>> {
        std::fs::read(path)
    }
    fn write(&self, path: &str, bytes: &[u8], append: bool) -> io::Result<()> {
        use io::Write;
        std::fs::OpenOptions::new().write(true).create(true).append(append).truncate(!append).open(path)?.write_all(bytes)
    }
    fn list(&self, dir: &str) -> io::Result<Vec<String>> {
        std::fs::read_dir(dir)?.map(|e| Ok(e?.file_name().to_string_lossy().into_owned())).collect()
    }
    fn metadata(&self, path: &str) -> io::Result<Meta> {
        let md = std::fs::metadata(path).or_else(|_| std::fs::symlink_metadata(path))?;
        let modified = Timestamp::try_from(md.modified()?).map_err(io::Error::other)?;
        Ok(Meta { is_dir: md.is_dir(), len: md.len(), modified })
    }
    fn remove(&self, path: &str) -> io::Result<()> {
        if std::path::Path::new(path).is_dir() { std::fs::remove_dir(path) } else { std::fs::remove_file(path) }
    }
    fn rename(&self, from: &str, to: &str) -> io::Result<()> {
        std::fs::rename(from, to)
    }
    fn copy(&self, from: &str, to: &str) -> io::Result<()> {
        std::fs::copy(from, to).map(drop)
    }
    fn mkdir(&self, path: &str) -> io::Result<()> {
        std::fs::create_dir_all(path)
    }
    fn cwd(&self) -> io::Result<String> {
        Ok(std::env::current_dir()?.to_string_lossy().into_owned())
    }
    fn cd(&self, dir: &str) -> io::Result<()> {
        std::env::set_current_dir(dir)
    }
    fn run(&self, cmd: &str) -> io::Result<Ran> {
        let (sh, flag) = if cfg!(windows) { ("cmd", "/C") } else { ("sh", "-c") };
        let out = std::process::Command::new(sh).args([flag, cmd]).output()?;
        let text = |b: &[u8]| String::from_utf8_lossy(b).into_owned();
        Ok(Ran { code: out.status.code().unwrap_or(-1), out: text(&out.stdout), err: text(&out.stderr) })
    }
    fn http_get(&self, url: &str, query: &[(&str, &str)]) -> Result<String, String> {
        ureq::get(url).query_pairs(query.iter().copied()).call().and_then(|mut r| r.body_mut().read_to_string()).map_err(|e| e.to_string())
    }
    fn cache_get(&self, key: &str) -> Option<(String, Timestamp)> {
        let path = crate::cache_dir()?.join(key);
        let at = Timestamp::try_from(path.metadata().ok()?.modified().ok()?).ok()?;
        Some((std::fs::read_to_string(path).ok()?, at))
    }
    fn cache_put(&self, key: &str, value: &str) {
        if let Some(dir) = crate::cache_dir() {
            let _ = std::fs::create_dir_all(&dir);
            let _ = std::fs::write(dir.join(key), value);
        }
    }
    fn exit(&self, code: i32) -> String {
        std::process::exit(code)
    }
}

#[cfg(test)]
mod tests {
    use super::NullHost;
    use crate::interp::Interp;
    use crate::run;
    use std::rc::Rc;

    #[test]
    fn null_host_touches_nothing() {
        let mut it = Interp::with_host(Rc::new(NullHost));
        let mut show = |s| run(&mut it, s).map(|v| v.to_string()).unwrap_or_else(|e| e.msg);
        assert_eq!(show("now.year"), "1970");
        assert_eq!(show(r#"read_file("Cargo.toml")"#), r#"read_file: cannot read "Cargo.toml": file access is disabled in this session"#);
        assert_eq!(show(r#"env("PATH")"#), "nil");
        assert!(show(r#"sh("echo hi")"#).contains("running commands is disabled"));
        assert!(show(r#"img("x.png")"#).contains("file access is disabled"));
        let a = show("[rand(), rand()]");
        assert_eq!(run(&mut Interp::with_host(Rc::new(NullHost)), "[rand(), rand()]").unwrap().to_string(), a, "seed 0 repeats");
    }
}
