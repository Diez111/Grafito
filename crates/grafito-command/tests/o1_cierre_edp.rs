//! O1-cierre: Transport1D + HeatSeparation cablean los dos motores PDE que
//! estaban sin call site ("futuro call site" en `pde.rs`). Con esto los 6
//! temas O1 quedan ejecutables por comando (calor/onda/Laplace/Fourier ya
//! estaban en HeatEquation/WaveEquation/Laplace2D/FourierSeries).

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
fn o1_cierre_transport_y_separation_visibles_y_ejecutan() {
    for canonical in ["Transport1D", "HeatSeparation"] {
        let spec = command_registry::resolve(canonical)
            .unwrap_or_else(|| panic!("{canonical} debe estar registrado"));
        assert!(
            spec.palette_visible,
            "{canonical} debe ser visible en paleta"
        );
        assert_eq!(spec.category, "AM2");
    }
    let mut document = Document::new();
    assert_message_contains(
        &mut document,
        "Transport1D[sin(pi*x), x, 1, 0, 0.1]",
        "advección 1D",
    );
    assert_message_contains(
        &mut document,
        "HeatSeparation[sin(pi*x), x, 0, 0.1, 8]",
        "separación de variables",
    );
}

#[test]
fn o1_cierre_errores_honestos() {
    let mut document = Document::new();
    // c = 0: estacionario, no advección (el motor lo rechaza con guía).
    assert_error_contains(
        &mut document,
        "Transport1D[sin(pi*x), x, 0, 0, 0.1]",
        "nula",
    );
    // Horizonte invertido: lo atrapa el comando antes del motor.
    assert_error_contains(
        &mut document,
        "Transport1D[sin(pi*x), x, 1, 1, 0.5]",
        "t_end debe ser >= t0",
    );
    assert_error_contains(
        &mut document,
        "HeatSeparation[sin(pi*x), x, 1, 0.5, 8]",
        "t_end debe ser >= t0",
    );
    // terms fuera de 1..=128: lo atrapa el comando.
    assert_error_contains(
        &mut document,
        "HeatSeparation[sin(pi*x), x, 0, 0.1, 0]",
        "fuera de 1..=128",
    );
    // Aridad: no cae a "no reconocido" (el dispatcher valida por spec).
    assert_error_contains(&mut document, "Transport1D[sin(pi*x), x, 1]", "Transport1D");
}
