//! O3-cierre: ParseWolfram + ToWolfram cablean el motor Wolfram InputForm
//! bidireccional (`grafito-geometry/src/wolfram.rs`). La mitad LaTeX ya estaba
//! (ParseLatex/ToLatex del frente G1); con esto O3 queda ejecutable por
//! comando en ambas direcciones.

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
fn o3_cierre_wolfram_visibles_y_ejecutan() {
    for canonical in ["ParseWolfram", "ToWolfram"] {
        let spec = command_registry::resolve(canonical)
            .unwrap_or_else(|| panic!("{canonical} debe estar registrado"));
        assert!(
            spec.palette_visible,
            "{canonical} debe ser visible en paleta"
        );
        assert_eq!(spec.category, "Texto");
    }
    let mut document = Document::new();
    assert_message_contains(&mut document, "ParseWolfram[Sin[x]^2]", "ParseWolfram: ");
    assert_message_contains(&mut document, "ParseWolfram[Sin[x]^2]", "sin(x)^(2)");
    assert_message_contains(&mut document, "ToWolfram[sin(x)^2]", "ToWolfram: ");
    assert_message_contains(&mut document, "ToWolfram[sin(x)^2]", "Sin[x]^2");
}

#[test]
fn o3_cierre_wolfram_errores_honestos() {
    let mut document = Document::new();
    // Head de cálculo: el motor traduce, no evalúa.
    assert_error_contains(&mut document, "ParseWolfram[Integrate[x]]", "Integrate");
    // Imaginaria: el motor es real.
    assert_error_contains(&mut document, "ParseWolfram[I]", "imaginaria");
    // Minúscula: Wolfram exige mayúscula.
    assert_error_contains(&mut document, "ParseWolfram[sin[x]]", "Sin");
    // Aridad: no cae a "no reconocido" (el dispatcher valida por spec).
    assert_error_contains(&mut document, "ParseWolfram[]", "ParseWolfram");
    assert_error_contains(&mut document, "ToWolfram[]", "ToWolfram");
}
