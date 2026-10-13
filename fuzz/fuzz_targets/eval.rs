#![no_main]
use std::rc::Rc;
use zil::interp::Interp;

libfuzzer_sys::fuzz_target!(|src: &str| {
    let mut it = Interp::with_host(Rc::new(zil::host::NullHost));
    it.limits.steps = 100_000;
    it.limits.depth = 100;
    it.limits.len = 10_000;
    it.limits.bits = 10_000;
    it.limits.output = 10_000;
    let _ = zil::run(&mut it, src);
});
