//! O4.3a — comandos complejos nuevos: ArgumentPrinciple +
//! ComplexDerivative + CauchyRiemannCheck sobre el núcleo compartido
//! `circle_integral_of_values` (GL16 + Kahan) en `grafito-complex`.
//! Valores exactos: z²−1 (N = 2/0/−1), f′(1+i) = 2+2i, CR de z² ≈ 0.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use grafito_command::command_registry;
use grafito_command::commands::{process_input, CommandOutcome};
use grafito_core::Document;

fn run_in(document: &mut Document, command: &str) -> CommandOutcome {
    let mut input = command.to_string();
    process_input(document, &mut input)
}

fn ok_in(document: &mut Document, command: &str) -> String {
    match run_in(document, command) {
        CommandOutcome::Message(text) => text,
        CommandOutcome::Ok => String::new(),
        other => panic!("{command} esperaba éxito, llegó {other:?}"),
    }
}

fn err_in(document: &mut Document, command: &str) -> String {
    match run_in(document, command) {
        CommandOutcome::Error(text) => text,
        other => panic!("{command} esperaba error honesto, llegó {other:?}"),
    }
}

#[test]
fn o43a_complejos_registrados_y_visibles() {
    for canonical in [
        "ArgumentPrinciple",
        "ComplexDerivative",
        "CauchyRiemannCheck",
    ] {
        let spec = command_registry::resolve(canonical)
            .unwrap_or_else(|| panic!("{canonical} debe estar registrado"));
        assert!(spec.palette_visible, "{canonical} visible en paleta");
        assert_eq!(spec.category, "Complejos");
    }
}

#[test]
fn o43a_argument_principle_cuenta_ceros_menos_polos() {
    let mut document = Document::new();
    // z²−1: 2 ceros dentro de |z| < 2.
    let out = ok_in(&mut document, "ArgumentPrinciple[z^2-1, 0, 2]");
    assert!(out.contains("= 2"), "N = 2: {out}");
    // Ningún cero dentro de |z| < 0.5.
    let out = ok_in(&mut document, "ArgumentPrinciple[z^2-1, 0, 0.5]");
    assert!(out.contains("= 0"), "N = 0: {out}");
    // 1/z: un polo simple ⇒ N = −1.
    let out = ok_in(&mut document, "ArgumentPrinciple[1/z, 0, 1]");
    assert!(out.contains("= -1"), "N = -1: {out}");
    // Cero sobre el contorno (|z| = 1 pasa por z = 1): error honesto.
    let err = err_in(&mut document, "ArgumentPrinciple[z^2-1, 0, 1]");
    assert!(
        err.contains("contour") || err.contains("contorno"),
        "cero sobre el contorno: {err}"
    );
}

#[test]
fn o43a_complex_derivative_y_cauchy_riemann() {
    let mut document = Document::new();
    // f′(1+i) de z² = 2+2i (formato `format_complex_rounded`).
    let out = ok_in(&mut document, "ComplexDerivative[z^2, 1+i]");
    assert!(out.contains("2.000 + 2.000i"), "f′(1+i) = 2+2i: {out}");
    // z² es holomorfa en (1,1): veredicto positivo.
    let out = ok_in(&mut document, "CauchyRiemannCheck[z^2, 1, 1]");
    assert!(out.contains("holomorfa aquí"), "z² holomorfa: {out}");
    // conj(z) no es holomorfa: veredicto negativo.
    let out = ok_in(&mut document, "CauchyRiemannCheck[conj(z), 1, 1]");
    assert!(out.contains("NO holomorfa"), "conj no holomorfa: {out}");
}
