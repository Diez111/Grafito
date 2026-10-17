//! EDO lineal de 1er orden (factor integrante) y homogénea de 2º orden
//! constante vía `StepByStep`. Solo esos dos casos tienen pasos; el resto
//! deriva al redirect honesto. La tarjeta ```grafito-steps y el remate
//! socrático quedan intactos.

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
fn step_by_step_ode1_factor_integrante_con_verificacion() {
    // y' + 2y = 0: P, μ, solución y verificación con residuo ≈ 0.
    let out = ok_text("StepByStep[SolveODE1[2, 0, x]]");
    assert!(out.contains("4 pasos"), "{out}");
    assert!(out.contains("OdeFirstOrderRule"), "{out}");
    assert!(out.contains("P(x)"), "{out}");
    assert!(out.contains("exp"), "{out}");
    assert!(out.contains("residuo"), "{out}");
    assert!(out.contains("```grafito-steps"), "{out}");
    assert!(out.contains("Antes de cerrar:"), "remate intacto: {out}");
}

#[test]
fn step_by_step_ode1_forma_plana_y_q_no_nulo() {
    let out = ok_text("StepByStep[SolveODE1, 2, 3, x]");
    assert!(out.contains("pasos"), "{out}");
    assert!(out.contains("residuo"), "{out}");
    assert!(out.contains("```grafito-steps"), "{out}");
}

#[test]
fn step_by_step_ode1_errores_honestos() {
    // P con y escondida: sin pasos inventados.
    let e = err_text("StepByStep[SolveODE1[y, 0, x]]");
    assert!(e.contains("ajenos") || e.contains("no resuelta"), "{e}");
    // Falta argumento.
    let e = err_text("StepByStep[SolveODE1[2]]");
    assert!(e.contains("falta el argumento"), "{e}");
}

#[test]
fn step_by_step_ode2h_tres_casos_delta_con_verificacion() {
    // Δ > 0: y''−3y'+2y = 0.
    let dpos = ok_text("StepByStep[SolveODEHomog[1, -3, 2, x]]");
    assert!(dpos.contains("4 pasos"), "{dpos}");
    assert!(dpos.contains("OdeSecondHomogRule"), "{dpos}");
    assert!(dpos.contains("Δ > 0"), "{dpos}");
    assert!(dpos.contains("residuo"), "{dpos}");
    // Δ = 0: y''+2y'+y = 0.
    let dcero = ok_text("StepByStep[SolveODEHomog[1, 2, 1, x]]");
    assert!(dcero.contains("Δ = 0"), "{dcero}");
    assert!(dcero.contains("residuo"), "{dcero}");
    // Δ < 0: y''+y = 0.
    let dneg = ok_text("StepByStep[SolveODEHomog[1, 0, 1, x]]");
    assert!(dneg.contains("Δ < 0"), "{dneg}");
    assert!(dneg.contains("cos"), "{dneg}");
    for out in [&dpos, &dcero, &dneg] {
        assert!(out.contains("```grafito-steps"), "{out}");
        assert!(out.contains("Antes de cerrar:"), "remate intacto: {out}");
    }
}

#[test]
fn step_by_step_ode2h_errores_honestos() {
    // a = 0 no es 2º orden.
    let e = err_text("StepByStep[SolveODEHomog[0, 1, 1, x]]");
    assert!(e.contains("a = 0"), "{e}");
    // Coeficiente variable: fuera del subset homogéneo constante.
    let e = err_text("StepByStep[SolveODEHomog[x, 1, 1, x]]");
    assert!(e.contains("constante"), "{e}");
}

#[test]
fn step_by_step_lista_nuevas_ops_en_error() {
    let honest = run("StepByStep[NoExiste[x]]");
    assert!(
        matches!(honest, CommandOutcome::Error(ref m) if m.contains("no tiene pasos soportados") && m.contains("SolveODE1") && m.contains("SolveODEHomog")),
        "got {honest:?}"
    );
}
