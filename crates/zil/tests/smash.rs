//! Every builtin called with odd arguments must return a value or an error, never panic.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;
use zil::interp::Interp;

const ARGS: &[&str] = &[
    "0",
    "-1",
    "9223372036854775807",
    "-9223372036854775808",
    "10 ** 30",
    "0.5",
    "-1e308",
    "1/3",
    "\"\"",
    "\"é%{\"",
    "[]",
    "[[1, 2], [3]]",
    "{}",
    "nil",
    "5 m",
    "today",
    "|x| x",
];

/// Calls every builtin with every combination of up to `arity` of `ARGS`.
fn smash(arity: usize) {
    let fresh = || {
        let mut it = Interp::with_host(Rc::new(zil::host::NullHost));
        it.limits.steps = 100_000;
        it.limits.len = 10_000;
        it.limits.bits = 10_000;
        it
    };
    let (mut layer, mut lists) = (vec![String::new()], vec![String::new()]);
    for _ in 0..arity {
        layer = layer.iter().flat_map(|l| ARGS.iter().map(move |a| if l.is_empty() { a.to_string() } else { format!("{l}, {a}") })).collect();
        lists.extend(layer.iter().cloned());
    }
    std::panic::set_hook(Box::new(|_| {}));
    let (mut it, mut panics) = (fresh(), vec![]);
    for name in zil::modules::modules().flat_map(|m| m.fns).map(|f| f.name) {
        for args in &lists {
            let src = format!("{name}({args})");
            if catch_unwind(AssertUnwindSafe(|| zil::run(&mut it, &src))).is_err() {
                panics.push(src);
                it = fresh();
            }
        }
    }
    drop(std::panic::take_hook());
    assert!(panics.is_empty(), "{} panics:\n{}", panics.len(), panics.join("\n"));
}

#[test]
fn builtins_never_panic() {
    smash(2);
}

/// About a minute in debug.
#[test]
#[ignore]
fn builtins_never_panic_3() {
    smash(3);
}
