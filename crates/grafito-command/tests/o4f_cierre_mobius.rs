//! O4.3b — Möbius compleja: puntos fijos + clasificación por traza +
//! círculo imagen. GeoGebra no tiene nada de esto (su `Moebius` es la
//! banda 3D). Referencias: 1/z → fijos {1,−1} elíptica; 2z → {0}
//! hiperbólica; z+1 → ∅ parabólica; 1/z sobre |z|=1 → imagen |w|=1.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use grafito_command::{
    command_registry,
    commands::{process_input, CommandOutcome},
};
use grafito_core::Document;

fn run(document: &mut Document, command: &str) -> CommandOutcome {
    let mut input = command.to_owned();
    process_input(document, &mut input)
}

fn assert_message_contains(document: &mut Document, command: &str, needle: &str) {
    match run(document, command) {
        CommandOutcome::Message(message) => assert!(
            message.contains(needle),
            "{command} → {message} (esperaba '{needle}')"
        ),
        CommandOutcome::Ok => panic!("{command} dio Ok, esperaba Message con '{needle}'"),
        CommandOutcome::Error(message) => {
            panic!("{command} dio Error: {message} (esperaba '{needle}')")
        }
    }
}

fn assert_error_contains(document: &mut Document, command: &str, needle: &str) {
    match run(document, command) {
        CommandOutcome::Error(message) => assert!(
            message.contains(needle),
            "{command} → {message} (esperaba error con '{needle}')"
        ),
        CommandOutcome::Message(message) => {
            panic!("{command} dio Message: {message} (esperaba error '{needle}')")
        }
        CommandOutcome::Ok => panic!("{command} dio Ok, esperaba error '{needle}'"),
    }
}

#[test]
fn o43b_mobius_visibles_y_ejecutan() {
    for canonical in ["MobiusFixedPoints", "MobiusClassify", "MobiusCircleImage"] {
        let spec = command_registry::resolve(canonical)
            .unwrap_or_else(|| panic!("{canonical} debe estar registrado"));
        assert!(spec.palette_visible, "{canonical} visible en paleta");
        assert_eq!(spec.category, "Complejos");
    }
    let mut document = Document::new();
    // 1/z: fijos 1 y −1.
    assert_message_contains(&mut document, "MobiusFixedPoints[0, 1, 1, 0]", "1.000");
    assert_message_contains(&mut document, "MobiusFixedPoints[0, 1, 1, 0]", "-1.000");
    // 1/z elíptica, 2z hiperbólica, z+1 parabólica, (1+i)z loxodrómica.
    assert_message_contains(&mut document, "MobiusClassify[0, 1, 1, 0]", "elíptica");
    assert_message_contains(&mut document, "MobiusClassify[2, 0, 0, 1]", "hiperbólica");
    assert_message_contains(&mut document, "MobiusClassify[1, 1, 0, 1]", "parabólica");
    assert_message_contains(&mut document, "MobiusClassify[1+i, 0, 0, 1]", "loxodrómica");
    // 1/z sobre |z| = 1: imagen |w| = 1 (radio 1.000000).
    assert_message_contains(
        &mut document,
        "MobiusCircleImage[0, 1, 1, 0, 0, 1]",
        "radio 1.000000",
    );
}

#[test]
fn o43b_mobius_errores_honestos() {
    let mut document = Document::new();
    // det = 0: no es transformación.
    assert_error_contains(&mut document, "MobiusFixedPoints[1, 1, 1, 1]", "ad−bc");
    assert_error_contains(&mut document, "MobiusClassify[1, 1, 1, 1]", "ad−bc");
    // Identidad: todo punto es fijo.
    assert_error_contains(&mut document, "MobiusFixedPoints[1, 0, 0, 1]", "identidad");
}
