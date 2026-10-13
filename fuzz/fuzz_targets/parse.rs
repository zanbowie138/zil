#![no_main]
libfuzzer_sys::fuzz_target!(|src: &str| {
    let _ = zil::parser::parse(src);
});
