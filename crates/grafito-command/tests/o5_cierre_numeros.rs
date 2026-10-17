//! O5-cierre: GaussKronrod + FFT + BfgsMinimize + BigInt + BigRational
//! cablean los 4 motores O5 que estaban sin call site (los slices GK/FFT/
//! intervalos/BFGS/big dejaban "cableado pendiente" en sus docs). Con esto los
//! 5 frentes O5 quedan ejecutables por comando (intervalos dirigidos no
//! llevan comando: son infraestructura de graficado). Igual que
//! `o1_cierre_edp`: happy + errores honestos por comando.

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
fn o5_cierre_numeros_visibles_y_ejecutan() {
    for (canonical, category) in [
        ("GaussKronrod", "CAS"),
        ("FFT", "CAS"),
        ("BfgsMinimize", "Análisis"),
        ("BigInt", "CAS"),
        ("BigRational", "CAS"),
    ] {
        let spec = command_registry::resolve(canonical)
            .unwrap_or_else(|| panic!("{canonical} debe estar registrado"));
        assert!(
            spec.palette_visible,
            "{canonical} debe ser visible en paleta"
        );
        assert_eq!(spec.category, category, "{canonical} categoría");
    }
    // Alias útil: bfgs → BfgsMinimize.
    assert_eq!(
        command_registry::resolve("bfgs").map(|s| s.canonical),
        Some("BfgsMinimize")
    );
    let mut document = Document::new();
    // ∫₀¹ x² = 1/3 por QK15.
    assert_message_contains(&mut document, "GaussKronrod[x^2, x, 0, 1]", "0.333");
    assert_message_contains(
        &mut document,
        "GaussKronrod[sin(x), x, 0, 3.14159265358979]",
        "2",
    );
    // DFT de [1, 0, -1, 0]: X = [0, 2, 0, 2], pico en k=1.
    assert_message_contains(&mut document, "FFT[{1, 0, -1, 0}]", "n=4");
    assert_message_contains(&mut document, "FFT[{1, 0, -1, 0}]", "k=1");
    // Cuadrática 2D: mínimo en (1, -2).
    assert_message_contains(
        &mut document,
        "BfgsMinimize[(x-1)^2+(y+2)^2, {x, y}, {0, 0}]",
        "mínimo local",
    );
    // Enteros grandes exactos (más allá de i64).
    assert_message_contains(
        &mut document,
        "BigInt[add, 123456789123456789123456789, 1]",
        "123456789123456789123456790",
    );
    assert_message_contains(
        &mut document,
        "BigInt[factorial, 25]",
        "15511210043330985984000000",
    );
    // Racionales exactos normalizados.
    assert_message_contains(&mut document, "BigRational[add, 1/3, 1/6]", "= 1/2");
    assert_message_contains(&mut document, "BigRational[decimal, 1/3, 6]", "0.333333");
}

#[test]
fn o5_cierre_errores_honestos() {
    let mut document = Document::new();
    // Límite invertido: lo atrapa el comando antes del motor.
    assert_error_contains(&mut document, "GaussKronrod[x^2, x, 1, 0]", "a <= b");
    // tol no positiva: la atrapa el comando.
    assert_error_contains(
        &mut document,
        "GaussKronrod[x^2, x, 0, 1, 0]",
        "tol debe ser finita y positiva",
    );
    // 3 muestras no es potencia de dos (el motor lo rechaza con guía).
    assert_error_contains(&mut document, "FFT[{1, 2, 3}]", "potencia de dos");
    // Dimensiones cruzadas: 2 vars con 3 valores.
    assert_error_contains(
        &mut document,
        "BfgsMinimize[x^2+y^2, {x, y}, {1, 2, 3}]",
        "2 variables pero 3 valores",
    );
    // Operación desconocida con guía (sin "no reconocido" ni "no implementado").
    assert_error_contains(&mut document, "BigInt[mod, 7, 3]", "no válida");
    assert_error_contains(&mut document, "BigRational[pow, 1/2, 3]", "no válida");
    // División por cero honesta (racional y entera).
    assert_error_contains(&mut document, "BigRational[div, 1/2, 0]", "cero");
    assert_error_contains(&mut document, "BigInt[divrem, 7, 0]", "cero");
    // Aridad: no cae a "no reconocido" (el dispatcher valida por spec).
    assert_error_contains(&mut document, "GaussKronrod[x^2]", "GaussKronrod");
    assert_error_contains(&mut document, "FFT[]", "FFT");
}
