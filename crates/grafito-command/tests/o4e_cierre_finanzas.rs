//! O4.5 — NPV + IRR: valor actual neto y tasa interna de retorno.
//! GeoGebra solo cubre TVM (Rate/Nper/Pmt/PV/FV); VAN/TIR por bisección
//! con error honesto no existen ahí. Referencia cerrada: [-100, 60, 60]
//! ⇒ VAN(10%) = 4.132231, TIR ≈ 0.13066239.

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
fn o45_npv_irr_visibles_y_ejecutan() {
    for canonical in ["NPV", "IRR"] {
        let spec = command_registry::resolve(canonical)
            .unwrap_or_else(|| panic!("{canonical} debe estar registrado"));
        assert!(spec.palette_visible, "{canonical} visible en paleta");
        assert_eq!(spec.category, "Financiera");
    }
    let mut document = Document::new();
    assert_message_contains(&mut document, "NPV[0.10, {-100, 60, 60}]", "NPV = 4.132231");
    assert_message_contains(&mut document, "IRR[{-100, 60, 60}]", "IRR = 0.1306623863");
}

#[test]
fn o45_npv_irr_errores_honestos() {
    let mut document = Document::new();
    // Flujos del mismo signo: no hay TIR.
    assert_error_contains(&mut document, "IRR[{100, 60, 60}]", "sin cambio de signo");
    // Tasa que anula el descuento: error honesto.
    assert_error_contains(&mut document, "NPV[-1, {-100, 60}]", "tasa inválida");
}
