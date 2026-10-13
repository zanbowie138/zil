#![no_main]
use std::rc::Rc;
use zil::Session;

libfuzzer_sys::fuzz_target!(|src: &str| {
    let mut s = Session::new(Rc::new(zil::host::NullHost));
    let l = s.limits();
    l.steps = 100_000;
    l.depth = 100;
    l.len = 10_000;
    l.bits = 10_000;
    l.output = 10_000;
    let _ = s.eval(src);
});
