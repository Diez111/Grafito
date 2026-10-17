//! O4.4 — ANOVA2: dos vías sin réplica (modelo aditivo, p por F).
//! Cierra el pendiente O4.2 (ANOVA2 paramétrico; Friedman ya cubría el
//! caso no-paramétrico de medidas repetidas). Referencia 3×3 a mano:
//! F_A = 37, F_B = 127.

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

#[test]
fn o44_anova2_visible_y_ejecuta() {
    let spec = command_registry::resolve("ANOVA2").expect("ANOVA2 registrado");
    assert!(spec.palette_visible, "ANOVA2 visible en paleta");
    assert_eq!(spec.category, "Estadística");
    let mut document = Document::new();
    assert_message_contains(
        &mut document,
        "ANOVA2[{6, 8, 10}, {7, 9, 12}, {5, 7, 9}]",
        "filas F = 37.0000",
    );
    assert_message_contains(
        &mut document,
        "ANOVA2[{6, 8, 10}, {7, 9, 12}, {5, 7, 9}]",
        "columnas F = 127.0000",
    );
}

#[test]
fn o44_anova2_errores_honestos() {
    let mut document = Document::new();
    // Aditividad perfecta ⇒ residuo nulo ⇒ F indefinido: mensaje, no panic.
    assert_message_contains(
        &mut document,
        "ANOVA2[{1, 2, 3}, {2, 3, 4}]",
        "no se pudo calcular",
    );
    // Una sola fila no es dos vías: error de aridad del spec.
    match run(&mut document, "ANOVA2[{1, 2, 3}]") {
        CommandOutcome::Error(message) => assert!(
            message.contains("cantidad de argumentos inválida"),
            "aridad: {message}"
        ),
        other => panic!("esperaba error de aridad, llegó {other:?}"),
    }
}
