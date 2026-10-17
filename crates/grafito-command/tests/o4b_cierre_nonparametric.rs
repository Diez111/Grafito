//! O4.2-cierre: MannWhitney + WilcoxonSigned + KruskalWallis + Friedman
//! cablean los 4 frentes no-paramétricos (motores nuevos en
//! `statistics.rs`: rangos promedio con corrección de empates, p por
//! aproximación normal / χ²). Igual que `o6_cierre_especiales`:
//! registro visible + ejecución real + errores honestos.

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
fn o4b_cierre_nonparametricos_visibles_y_ejecutan() {
    for canonical in ["MannWhitney", "WilcoxonSigned", "KruskalWallis", "Friedman"] {
        let spec = command_registry::resolve(canonical)
            .unwrap_or_else(|| panic!("{canonical} debe estar registrado"));
        assert!(
            spec.palette_visible,
            "{canonical} debe ser visible en paleta"
        );
    }
    let mut document = Document::new();
    // Separación total: U = 0, p ≈ 0.0808 (bilateral, n1 = n2 = 3).
    assert_message_contains(
        &mut document,
        "MannWhitney[{1, 2, 3}, {4, 5, 6}]",
        "U = 0.0000",
    );
    // Diferencias todas negativas: W = 0, p ≈ 0.036.
    assert_message_contains(
        &mut document,
        "WilcoxonSigned[{1, 2, 3, 4, 5, 6}, {2, 4, 6, 8, 10, 12}]",
        "W = 0.0000",
    );
    // Grupos separados: H = 7.2, p < 0.05.
    assert_message_contains(
        &mut document,
        "KruskalWallis[{1, 2, 3}, {4, 5, 6}, {7, 8, 9}]",
        "H = 7.2000",
    );
    // Rangos perfectos por bloque: Q = 8.
    assert_message_contains(
        &mut document,
        "Friedman[{1, 2, 3, 4}, {2, 3, 4, 5}, {3, 4, 5, 6}]",
        "Q = 8.0000",
    );
}

#[test]
fn o4b_cierre_nonparametricos_errores_honestos() {
    let mut document = Document::new();
    // Sin diferencias no-nulas no hay test: mensaje honesto, no panic.
    assert_message_contains(
        &mut document,
        "WilcoxonSigned[{1, 2}, {1, 2}]",
        "no se pudo calcular",
    );
    // Un solo grupo no es Kruskal-Wallis: error de aridad del spec.
    assert_error_contains(
        &mut document,
        "KruskalWallis[{1, 2, 3}]",
        "cantidad de argumentos inválida",
    );
    // Tratamientos con distinto n no son Friedman.
    assert_message_contains(
        &mut document,
        "Friedman[{1, 2, 3}, {1, 2}]",
        "no se pudo calcular",
    );
}
