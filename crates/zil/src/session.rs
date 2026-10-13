//! The embedding API: one sandboxed session of zil code.

use crate::host::Host;
use crate::interp::{Interp, Limits, lookup};
use crate::{Data, Error, Value};
use std::rc::Rc;

/// A zil session: globals, user units and rates that persist across `eval`s, with every effect going through `host`.
///
/// ```
/// use std::rc::Rc;
/// let mut s = zil::Session::new(Rc::new(zil::host::NullHost));
/// s.limits().steps = 1_000_000;
/// s.set("n", zil::Value::int(20));
/// assert_eq!(s.eval("n * 2 + 2").unwrap().to_string(), "42");
/// ```
pub struct Session(Interp);

impl Session {
    /// A fresh session on `host`. `host::NullHost` denies every capability, for untrusted code.
    pub fn new(host: Rc<dyn Host>) -> Session {
        Session(Interp::with_host(host))
    }

    /// Runs `src`, returning the last statement's value.
    pub fn eval(&mut self, src: &str) -> Result<Value, Error> {
        crate::run(&mut self.0, src)
    }

    /// Sets a global variable.
    pub fn set(&mut self, name: &str, v: Value) {
        self.0.set_global(name, v)
    }

    /// A global variable or builtin.
    pub fn get(&self, name: &str) -> Option<Value> {
        lookup(&self.0.globals(), name)
    }

    /// The value `d` describes, with units resolved in this session (so user units work too).
    pub fn from_data(&self, d: Data) -> Result<Value, String> {
        crate::state::enter(&self.0.state(), || Value::from_data(d))
    }

    /// Caps on each `eval`; hitting one is an `Error` whose `limit` says which.
    pub fn limits(&mut self) -> &mut Limits {
        &mut self.0.limits
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Limit;
    use crate::host::NullHost;

    #[test]
    fn embeds() {
        let mut s = Session::new(Rc::new(NullHost));
        s.eval("f = |x| x * 2").unwrap();
        s.set("y", Value::int(21));
        assert_eq!(s.eval("f(y)").unwrap().to_string(), "42");
        assert!(s.get("f").is_some() && s.get("nope").is_none());
        s.limits().steps = 1000;
        assert_eq!(s.eval("while true {}").unwrap_err().limit, Some(Limit::Steps));
        assert!(s.eval(r#"read_file("x")"#).is_err());
    }

    #[test]
    fn data_round_trips() {
        let mut s = Session::new(Rc::new(NullHost));
        let src = r#"unit widget; [nil, true, 2^70, 7/2, 1.5, 3 kg*m/s^2, 2 widget/h, 5 ± 0.1 m, 2 - 3i, "hi", r"a+", date("2026-10-06T14:30:00[Europe/Paris]"), {a: set(1, "x")}, table([{n: 1}])]"#;
        let v = s.eval(src).unwrap();
        let d = v.to_data();
        assert_eq!(s.from_data(d.clone()).unwrap().to_data(), d);
        assert_eq!(s.from_data(d).unwrap().to_string(), v.to_string());
        assert!(matches!(s.eval("|x| x").unwrap().to_data(), Data::Fn(_)));
        assert!(s.from_data(Data::Set(vec![Data::List(vec![])])).is_err());
        assert!(s.from_data(Data::Table { cols: vec!["a".into()], rows: vec![vec![]] }).is_err());
        assert!(s.from_data(Data::Qty { value: 1.0, unit: "nope".into() }).is_err());
    }
}
