//! Modules group builtins with everything else a feature owns: docs, operators, ordering,
//! `to` conversions and magic identifiers. All are always loaded and share one flat namespace.
//!
//! Hooks return `None` for "not mine"; dispatch tries modules in `MODULES` order, so order matters:
//! strings before units (`"a" + 5 km` concatenates), dates before units (`date + 1 d` is calendar math).

pub mod core;
pub mod dates;
pub mod lists;
pub mod math;
pub mod random;
pub mod strings;
pub mod units;

use crate::Error;
use crate::ast::{BinOp, Target};
use crate::interp::{Interp, Value};
use crate::lexer::Span;
use std::cmp::Ordering;

/// Name, signature, summary, examples, see also.
pub type Doc = (&'static str, &'static str, &'static str, &'static [&'static str], &'static [&'static str]);

/// A help page section: heading, then (label, example) rows; examples run live, and an empty one prints the label alone.
pub type Section = (&'static str, &'static [(&'static str, &'static str)]);

/// A hook's answer: `None` means "not mine, ask the next module".
pub type Claim = Option<Result<Value, String>>;

/// Why a builtin call failed; `Fail::error` turns it into an `Error` naming the builtin.
pub enum Fail {
    Msg(String),
    /// Already located, e.g. from a callback: passed through untouched.
    Err(Error),
    BadArgs,
}

impl From<String> for Fail {
    fn from(s: String) -> Fail {
        Fail::Msg(s)
    }
}

impl From<&str> for Fail {
    fn from(s: &str) -> Fail {
        Fail::Msg(s.into())
    }
}

impl From<Error> for Fail {
    fn from(e: Error) -> Fail {
        Fail::Err(e)
    }
}

impl Fail {
    pub fn error(self, name: &str, args: &[Value], span: &Span) -> Error {
        match self {
            Fail::Msg(m) => Error::new(format!("{name}: {m}"), span.clone()),
            Fail::Err(e) => e,
            Fail::BadArgs => {
                let types: Vec<_> = args.iter().map(Value::type_name).collect();
                Error::new(format!("{name}: unsupported arguments ({})", types.join(", ")), span.clone())
            }
        }
    }
}

pub type Call = Result<Value, Fail>;

pub struct Module {
    /// Help category.
    pub name: &'static str,
    /// One line under the module's help page title.
    pub about: &'static str,
    /// Shown with its result in the `help()` overview.
    pub example: &'static str,
    /// Help page sections; the labels of a "types" section also show in the overview.
    pub guide: &'static [Section],
    /// Exported builtins and their help.
    pub fns: &'static [Doc],
    /// Function names by category for the help page; every fn in exactly one. Empty lists them all on one line.
    pub groups: &'static [(&'static str, &'static [&'static str])],
    pub call: fn(&mut Interp, &'static str, &[Value], &Span) -> Call,
    pub consts: &'static [(&'static str, f64)],
    /// `to` keywords and the exported fn each calls: `x to UTC` is `utc(x)`, `x to hex(8)` is `hex(x, 8)`.
    pub targets: &'static [(&'static str, &'static str)],
    /// Names that aren't variables, tried after scope lookup fails (`now`, units).
    pub ident: Option<fn(&mut Interp, &str) -> Claim>,
    pub binary: Option<fn(BinOp, &Value, &Value) -> Claim>,
    pub compare: Option<fn(&Value, &Value) -> Option<Ordering>>,
    /// Conversions to targets that aren't keywords: units (`to km`), strings (`to "Europe/Paris"`).
    pub convert: Option<fn(&Value, &Target) -> Claim>,
    /// Extra help topics; prints and returns true if it knows `topic`.
    pub topic: Option<fn(&str) -> bool>,
}

impl Module {
    pub const EMPTY: Module = Module {
        name: "",
        about: "",
        example: "",
        guide: &[],
        fns: &[],
        groups: &[],
        call: |_, _, _, _| Err(Fail::BadArgs),
        consts: &[],
        targets: &[],
        ident: None,
        binary: None,
        compare: None,
        convert: None,
        topic: None,
    };
}

/// Also the `help()` overview order.
pub const MODULES: &[Module] = &[strings::MODULE, lists::MODULE, math::MODULE, dates::MODULE, random::MODULE, units::MODULE, core::MODULE];

/// The module and fn a `to` keyword calls.
pub fn target(keyword: &str) -> Option<(&'static Module, &'static str)> {
    MODULES.iter().find_map(|m| m.targets.iter().find(|t| t.0 == keyword).map(|t| (m, t.1)))
}

pub fn ident(it: &mut Interp, name: &str) -> Claim {
    MODULES.iter().filter_map(|m| m.ident).find_map(|f| f(it, name))
}

pub fn binary(op: BinOp, a: &Value, b: &Value) -> Claim {
    MODULES.iter().filter_map(|m| m.binary).find_map(|f| f(op, a, b))
}

pub fn compare(a: &Value, b: &Value) -> Option<Ordering> {
    MODULES.iter().filter_map(|m| m.compare).find_map(|f| f(a, b))
}

pub fn convert(v: &Value, t: &Target) -> Claim {
    MODULES.iter().filter_map(|m| m.convert).find_map(|f| f(v, t))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry() {
        let mut seen = std::collections::HashMap::new();
        for m in MODULES {
            for f in m.fns {
                if let Some(other) = seen.insert(f.0, m.name) {
                    panic!("{} exported by both {other} and {}", f.0, m.name);
                }
            }
            for (kw, f) in m.targets {
                assert!(m.fns.iter().any(|d| d.0 == *f), "{}: target {kw} calls {f}, which it doesn't export", m.name);
                let is_unit = units::TABLE.iter().any(|row| row.0.split(' ').any(|n| n == *kw));
                assert!(!is_unit, "target {kw} would shadow the unit `to {kw}`");
            }
            let grouped: Vec<_> = m.groups.iter().flat_map(|g| g.1.iter()).collect();
            for f in m.fns.iter().filter(|_| !m.groups.is_empty()) {
                assert_eq!(grouped.iter().filter(|g| **g == &f.0).count(), 1, "{}: {} must be in exactly one group", m.name, f.0);
            }
            for g in grouped {
                assert!(m.fns.iter().any(|f| f.0 == *g), "{}: group lists unknown fn {g}", m.name);
            }
        }
    }
}
