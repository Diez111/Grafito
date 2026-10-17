//! O6-cierre: Polygamma + Hypergeometric2F1 + EllipticK/E + AiryAi/Bi +
//! BesselK + SphericalHarm cablean los 6 frentes O6 (motores nuevos en
//! `special_functions.rs`, sin call site previo). Igual que `o5_cierre_numeros`:
//! happy + errores honestos por comando. Con esto Olas 1–6 quedan cerradas.

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
fn o6_cierre_especiales_visibles_y_ejecutan() {
    for canonical in [
        "Polygamma",
        "Hypergeometric2F1",
        "EllipticK",
        "EllipticE",
        "AiryAi",
        "AiryBi",
        "BesselK",
        "SphericalHarm",
    ] {
        let spec = command_registry::resolve(canonical)
            .unwrap_or_else(|| panic!("{canonical} debe estar registrado"));
        assert!(
            spec.palette_visible,
            "{canonical} debe ser visible en paleta"
        );
        assert_eq!(spec.category, "CAS", "{canonical} categoría");
    }
    let mut document = Document::new();
    // ψ₂(1) = -2ζ(3), ψ₃(1) = π⁴/15.
    assert_message_contains(&mut document, "Polygamma[2, 1]", "-2.404114");
    assert_message_contains(&mut document, "Polygamma[3, 1]", "6.493939");
    assert_message_contains(&mut document, "Polygamma[0, 1]", "-0.577216");
    // ₂F₁(1,1;2;1/2) = 2ln2; Gauss ₂F₁(1,2;4;1) = 3.
    assert_message_contains(&mut document, "Hypergeometric2F1[1, 1, 2, 0.5]", "1.386294");
    assert_message_contains(&mut document, "Hypergeometric2F1[1, 2, 4, 1]", "= 3.000000");
    // K(0) = E(0) = π/2; K(1/2), E(1/2) AGM.
    assert_message_contains(&mut document, "EllipticK[0]", "1.570796");
    assert_message_contains(&mut document, "EllipticK[0.5]", "1.685750");
    assert_message_contains(&mut document, "EllipticE[0]", "1.570796");
    assert_message_contains(&mut document, "EllipticE[0.5]", "1.467462");
    // Airy en 0 y 1.
    assert_message_contains(&mut document, "AiryAi[0]", "0.355028");
    assert_message_contains(&mut document, "AiryBi[0]", "0.614927");
    assert_message_contains(&mut document, "AiryAi[1]", "0.135292");
    assert_message_contains(&mut document, "AiryBi[1]", "1.207424");
    // K₀(1), K₁(1), K₂(1) = K₀ + 2K₁.
    assert_message_contains(&mut document, "BesselK[0, 1]", "0.421024");
    assert_message_contains(&mut document, "BesselK[1, 1]", "0.601907");
    assert_message_contains(&mut document, "BesselK[2, 1]", "1.624839");
    // Y₀⁰ = 1/√(4π); Y₁⁰(0) = √(3/4π).
    assert_message_contains(&mut document, "SphericalHarm[0, 0, 1, 0]", "0.282095");
    assert_message_contains(&mut document, "SphericalHarm[1, 0, 0, 0]", "0.488603");
}

#[test]
fn o6_cierre_errores_honestos() {
    let mut document = Document::new();
    // Orden fuera de cota / polo / x≤0 en m≥2.
    assert_error_contains(&mut document, "Polygamma[11, 1]", "entre 0 y 10");
    assert_error_contains(&mut document, "Polygamma[2, 0]", "no es finito");
    assert_error_contains(&mut document, "Polygamma[2, -1]", "no es finito");
    // Fuera del disco, c entero ≤0, Gauss divergente.
    assert_error_contains(
        &mut document,
        "Hypergeometric2F1[1, 1, 2, 1.5]",
        "no es finito",
    );
    assert_error_contains(
        &mut document,
        "Hypergeometric2F1[1, 1, 0, 0.5]",
        "no es finito",
    );
    assert_error_contains(
        &mut document,
        "Hypergeometric2F1[2, 2, 3, 1]",
        "no es finito",
    );
    // |k|>1 con guía; k=±1 diverge (no finito).
    assert_error_contains(&mut document, "EllipticK[1.5]", "|k|");
    assert_error_contains(&mut document, "EllipticE[1.5]", "|k|");
    assert_error_contains(&mut document, "EllipticK[1]", "no es finito");
    // Orden K fuera de cota / x≤0 con guía.
    assert_error_contains(&mut document, "BesselK[1001, 1]", "entre -1000 y 1000");
    assert_error_contains(&mut document, "BesselK[0, 0]", "x > 0");
    assert_error_contains(&mut document, "BesselK[0, -1]", "x > 0");
    // |m|>l con guía; l fuera de rango.
    assert_error_contains(&mut document, "SphericalHarm[1, 2, 0.5, 0.5]", "|m|");
    assert_error_contains(
        &mut document,
        "SphericalHarm[33, 0, 0.5, 0.5]",
        "entre 0 y 32",
    );
    // Aridad: no cae a "no reconocido".
    assert_error_contains(&mut document, "Polygamma[2]", "Polygamma");
    assert_error_contains(&mut document, "EllipticK[]", "EllipticK");
    assert_error_contains(&mut document, "SphericalHarm[1, 0, 0]", "SphericalHarm");
}
