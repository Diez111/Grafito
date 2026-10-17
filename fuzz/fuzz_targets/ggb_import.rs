#![no_main]

use libfuzzer_sys::fuzz_target;

// GGB = zip + XML + mapeo a objetos: el importador nunca debe panicar con
// bytes arbitrarios (zip truncado, XML malformado, sliders corruptos).
const MAX_INPUT_BYTES: usize = 65_536;

fuzz_target!(|data: &[u8]| {
    let bytes = &data[..data.len().min(MAX_INPUT_BYTES)];
    let _ = grafito_ggb::import_ggb_bytes(bytes);
});
