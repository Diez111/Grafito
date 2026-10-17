//! Sistemas lineales 2×2 con pasos + integral definida por Barrow vía `StepByStep`.
//! La tarjeta ```grafito-steps y el remate socrático quedan intactos.

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
fn step_by_step_sistema_2x2_con_fence_y_remate() {
    let out = ok_text("StepByStep[SolveNlSystem[x+y-3, x-y-1, x, y]]");
    assert!(out.contains("StepByStep"), "{out}");
    assert!(out.contains("pasos"), "{out}");
    assert!(out.contains('→'), "{out}");
    assert!(out.contains("```grafito-steps"), "{out}");
    assert!(out.contains("Antes de cerrar:"), "remate intacto: {out}");
    assert!(out.contains("x = 2"), "{out}");
    assert!(out.contains("y = 1"), "{out}");
}

#[test]
fn step_by_step_sistema_forma_plana() {
    let out = ok_text("StepByStep[SolveNlSystem, x+y-3, x-y-1, x, y]");
    assert!(out.contains("pasos"), "{out}");
    assert!(out.contains("```grafito-steps"), "{out}");
}

#[test]
fn step_by_step_sistema_errores_honestos() {
    // No lineal: sin pasos inventados.
    let e = err_text("StepByStep[SolveNlSystem[x^2+y, x-y, x, y]]");
    assert!(e.contains("no lineal"), "{e}");
    // Singular: sin solución única.
    let e = err_text("StepByStep[SolveNlSystem[x+y, 2*x+2*y, x, y]]");
    assert!(e.contains("singular") || e.contains("det"), "{e}");
}

#[test]
fn step_by_step_integral_definida_barrow() {
    let out = ok_text("StepByStep[Integral[x^2, x, 0, 1]]");
    assert!(out.contains("pasos"), "{out}");
    assert!(out.contains("0.333333"), "{out}");
    assert!(out.contains("```grafito-steps"), "{out}");
    assert!(out.contains("Barrow"), "{out}");
    assert!(out.contains("Antes de cerrar:"), "remate intacto: {out}");
}

#[test]
fn step_by_step_integral_indefinida_intacta() {
    let out = ok_text("StepByStep[Integral[x^2, x]]");
    // "paso"/"pasos": el plural ya no miente en casos de 1 paso.
    assert!(out.contains("paso"), "{out}");
    assert!(!out.contains("Barrow"), "{out}");
}

#[test]
fn step_by_step_integral_sin_primitiva_honesta() {
    let e = err_text("StepByStep[Integral[exp(x^2), x, 0, 1]]");
    assert!(e.contains("no hay primitiva"), "{e}");
}

#[test]
fn step_by_step_fuera_de_subset_lista_nuevas_ops() {
    let honest = run("StepByStep[NoExiste[x]]");
    assert!(
        matches!(honest, CommandOutcome::Error(ref m) if m.contains("no tiene pasos soportados") && m.contains("SolveNlSystem")),
        "got {honest:?}"
    );
}

#[test]
fn step_by_step_singular_es_un_paso() {
    // Solve lineal exacto de un paso: singular honesto, sin "1 pasos".
    let out = ok_text("StepByStep[Solve[x-3, x]]");
    assert!(
        out.contains("1 paso\n") || out.contains(": 1 paso "),
        "{out}"
    );
    assert!(!out.contains("1 pasos"), "no se dice «1 pasos»: {out}");
    assert!(out.contains("```grafito-steps"), "{out}");
}
