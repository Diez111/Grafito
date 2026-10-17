//! O4-cierre: NDerivativeSym + Partial + SubstituteInt + ImproperIntegral +
//! LaurentSeries + SumClosed + LambertW + Gruntz(vía Limit) ya estaban
//! cableados (motores + specs + dispatch). Este test pinea los 8 ejecutables
//! por comando con happy + errores honestos, igual que `o1_cierre_edp`.

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
fn o4_cierre_calculo_visibles_y_ejecutan() {
    for canonical in [
        "NDerivativeSym",
        "Partial",
        "SubstituteInt",
        "LambertW",
        "LaurentSeries",
        "SumClosed",
        "ImproperIntegral",
    ] {
        let spec = command_registry::resolve(canonical)
            .unwrap_or_else(|| panic!("{canonical} debe estar registrado"));
        assert!(
            spec.palette_visible,
            "{canonical} debe ser visible en paleta"
        );
    }
    let mut document = Document::new();
    // n-ésima simbólica: d²/dx² x^3 = 6*x (forma sin simplificar del motor).
    assert_message_contains(&mut document, "NDerivativeSym[x^3, x, 2]", "2 * x");
    // Parcial: ∂/∂x (x^2*y) = 2*x*y.
    assert_message_contains(&mut document, "Partial[x^2*y, x]", "2");
    // u-du: ∫2*x*exp(x^2) con u=x^2 → exp(x^2).
    assert_message_contains(&mut document, "SubstituteInt[2*x*exp(x^2), x, x^2]", "exp");
    // Impropia calculada: ∫₀^∞ exp(-x) = 1.
    assert_message_contains(
        &mut document,
        "ImproperIntegral[exp(-x), x, 0, inf]",
        "ImproperIntegral: 1",
    );
    // Laurent en polo: principal exacta x^-1 + nota honesta de regular.
    assert_message_contains(&mut document, "LaurentSeries[1/x, x, 0, 2]", "x^-1");
    assert_message_contains(
        &mut document,
        "LaurentSeries[1/x, x, 0, 2]",
        "parte regular no resoluble",
    );
    // Laurent en punto analítico: Taylor puro.
    assert_message_contains(
        &mut document,
        "LaurentSeries[1/(x+1), x, 0, 2]",
        "LaurentSeries",
    );
    // Suma cerrada: Σ₁¹⁰ k = 55.
    assert_message_contains(&mut document, "SumClosed[k, k, 1, 10]", "55");
    // Lambert W0(1) ≈ 0.567.
    assert_message_contains(&mut document, "LambertW[1]", "0.567");
    // Gruntz real vía Limit: sin(x)/x → 1.
    assert_message_contains(&mut document, "Limit[sin(x)/x, x, 0]", "1.00000000");
}

#[test]
fn o4_cierre_errores_honestos() {
    let mut document = Document::new();
    // n fuera de 1..=16: lo atrapa el comando.
    assert_error_contains(
        &mut document,
        "NDerivativeSym[x^3, x, 0]",
        "fuera de 1..=16",
    );
    // Variable inválida.
    assert_error_contains(&mut document, "Partial[x^2, 9x]", "variable válida");
    // Cambio trivial u == var.
    assert_error_contains(&mut document, "SubstituteInt[x^2, x, x]", "trivial");
    // Lambert fuera de dominio real.
    assert_error_contains(&mut document, "LambertW[-1]", "fuera de dominio");
    // Impropia divergente honesta.
    assert_error_contains(
        &mut document,
        "ImproperIntegral[1/x, x, 0, 1]",
        "divergente",
    );
    // Suma invertida.
    assert_error_contains(&mut document, "SumClosed[k, k, 5, 1]", "hi >= lo");
    // Laurent con n fuera de 0..=16.
    assert_error_contains(
        &mut document,
        "LaurentSeries[1/x, x, 0, 17]",
        "fuera de 0..=16",
    );
    // Aridad: no cae a "no reconocido".
    assert_error_contains(&mut document, "NDerivativeSym[x^3, x]", "NDerivativeSym");
}
