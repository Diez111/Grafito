//! Descriptiva poblacional y regresión lineal por mínimos cuadrados vía
//! `StepByStep`. Solo esos dos casos tienen pasos (media/mediana/varianza/
//! desvío ÷N con N pineado; pendiente/intercepto por fórmulas cerradas + R²
//! con verificación 0≤R²≤1 recalculada por residuos). Cuantiles y
//! distribuciones quedan fuera: van directo sin pasos (redirect honesto del
//! asistente). La tarjeta ```grafito-steps y el remate socrático intactos.

use grafito_command::commands::{process_input, CommandOutcome};
use grafito_core::Document;

fn run(command: &str) -> CommandOutcome {
    let mut document = Document::new();
    let mut input = command.to_string();
    process_input(&mut document, &mut input)
}

fn ok_text(command: &str) -> String {
    match run(command) {
        CommandOutcome::Message(text) => text,
        CommandOutcome::Ok => String::new(),
        other => panic!("{command} esperaba éxito, llegó {other:?}"),
    }
}

fn err_text(command: &str) -> String {
    match run(command) {
        CommandOutcome::Error(text) => text,
        other => panic!("{command} esperaba error honesto, llegó {other:?}"),
    }
}

#[test]
fn step_by_step_descriptiva_media_123_con_verificacion() {
    // [1,2,3]: N=3, media=2, mediana=2, σ²=2/3, σ=√(2/3).
    let out = ok_text("StepByStep[Mean[{1, 2, 3}]]");
    assert!(out.contains("6 pasos"), "{out}");
    assert!(out.contains("StatsDescriptiveRule"), "{out}");
    assert!(out.contains("N = 3"), "N pineado: {out}");
    assert!(out.contains("media = 2"), "{out}");
    assert!(out.contains("mediana = 2"), "{out}");
    assert!(out.contains("0.666667"), "σ² = 2/3: {out}");
    assert!(out.contains("0.816497"), "σ = √(2/3): {out}");
    assert!(out.contains("residuo"), "verificación recalculada: {out}");
    assert!(out.contains("```grafito-steps"), "{out}");
    assert!(out.contains("Antes de cerrar:"), "remate intacto: {out}");
}

#[test]
fn step_by_step_descriptiva_par_con_decimales_pineados() {
    // [1,2,3,4]: media=2.5, mediana=2.5, σ²=1.25, σ≈1.118034.
    let out = ok_text("StepByStep[Variance[{1, 2, 3, 4}]]");
    assert!(out.contains("2.5"), "{out}");
    assert!(out.contains("1.25"), "{out}");
    assert!(out.contains("```grafito-steps"), "{out}");
    let out = ok_text("StepByStep[SD[{1, 2, 3, 4}]]");
    assert!(out.contains("1.118034"), "{out}");
    assert!(out.contains("StatsDescriptiveRule"), "{out}");
    let out = ok_text("StepByStep[Median[{1, 2, 3, 4}]]");
    assert!(out.contains("mediana = 2.5"), "{out}");
}

#[test]
fn step_by_step_descriptiva_forma_plana() {
    let out = ok_text("StepByStep[Mean, {1, 2, 3}]");
    assert!(out.contains("pasos"), "{out}");
    assert!(out.contains("N = 3"), "{out}");
    assert!(out.contains("```grafito-steps"), "{out}");
}

#[test]
fn step_by_step_descriptiva_errores_honestos() {
    // Vacía: sin datos no hay pasos.
    let e = err_text("StepByStep[Mean[{}]]");
    assert!(e.contains("vacía"), "{e}");
    // Texto no numérico: jamás inventar datos.
    let e = err_text("StepByStep[Median[{a, b}]]");
    assert!(e.contains("inválido") || e.contains("solo números"), "{e}");
    // Falta argumento.
    let e = err_text("StepByStep[Variance[]]");
    assert!(e.contains("falta el argumento"), "{e}");
}

#[test]
fn step_by_step_regresion_recta_exacta_r2_pin() {
    // y = 2x: m=2, b=0, R²=1 pineado.
    let out = ok_text("StepByStep[LinearRegression[{1, 2, 3}, {2, 4, 6}]]");
    assert!(out.contains("5 pasos"), "{out}");
    assert!(out.contains("StatsRegressionRule"), "{out}");
    assert!(out.contains("m = 2"), "{out}");
    assert!(out.contains("b = 0"), "{out}");
    assert!(out.contains("R² = 1"), "R² pin: {out}");
    assert!(out.contains("0 ≤ R² ≤ 1"), "{out}");
    assert!(out.contains("residuos"), "verificación por residuos: {out}");
    assert!(out.contains("```grafito-steps"), "{out}");
    assert!(out.contains("Antes de cerrar:"), "remate intacto: {out}");
}

#[test]
fn step_by_step_regresion_no_trivial_r2_pin() {
    // [0,1,2],[1,3,2]: m=0.5, b=1.5, R²=0.25 pineado.
    let out = ok_text("StepByStep[LinearRegression[{0, 1, 2}, {1, 3, 2}]]");
    assert!(out.contains("m = 0.5"), "{out}");
    assert!(out.contains("b = 1.5"), "{out}");
    assert!(out.contains("R² = 0.25"), "R² pin: {out}");
}

#[test]
fn step_by_step_regresion_errores_honestos() {
    // Largos distintos.
    let e = err_text("StepByStep[LinearRegression[{1, 2}, {1, 2, 3}]]");
    assert!(e.contains("mismo largo"), "{e}");
    // x constante: sin pendiente única.
    let e = err_text("StepByStep[LinearRegression[{2, 2}, {1, 3}]]");
    assert!(e.contains("no varía") || e.contains("varía"), "{e}");
    // Un solo punto.
    let e = err_text("StepByStep[LinearRegression[{1}, {2}]]");
    assert!(e.contains("al menos 2 puntos"), "{e}");
}

#[test]
fn step_by_step_fuera_de_subset_stats_honesto() {
    // Cuantil, muestral y correlación no tienen pasos: error honesto que
    // lista el subset con pasos (cuantil/distribución van directo).
    for op in [
        "StepByStep[Percentile[{1, 2, 3}, 90]]",
        "StepByStep[SampleVariance[{1, 2, 3}]]",
        "StepByStep[Correlation[{1, 2}, {3, 4}]]",
    ] {
        let honest = run(op);
        assert!(
            matches!(honest, CommandOutcome::Error(ref m) if m.contains("no tiene pasos soportados") && m.contains("LinearRegression")),
            "got {honest:?} para {op}"
        );
    }
}
