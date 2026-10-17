#![no_main]

use grafito_command::commands::{process_input, CommandOutcome};
use grafito_core::Document;
use libfuzzer_sys::fuzz_target;

const MAX_INPUT_BYTES: usize = 65_536;

fuzz_target!(|data: &[u8]| {
    let mut document = Document::new();
    // Sin `expect`: un OOM o error de serialización en el harness sería un
    // falso crash del fuzzer. Si no se puede tomar la línea base, se salta.
    let Ok(before) = serde_json::to_value(&document) else {
        return;
    };
    let version = document.version;
    let mut input = String::from_utf8_lossy(&data[..data.len().min(MAX_INPUT_BYTES)]).into_owned();

    let outcome = process_input(&mut document, &mut input);
    if matches!(outcome, CommandOutcome::Error(_)) {
        let Ok(after) = serde_json::to_value(&document) else {
            return;
        };
        assert_eq!(after, before);
        assert_eq!(document.version, version);
    }
});
