#![no_main]

libfuzzer_sys::fuzz_target!(|data: &[u8]| prototype_fuzz::run("packet", data));
