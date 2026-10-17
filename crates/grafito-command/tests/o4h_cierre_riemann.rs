//! O7 — esfera de Riemann: `RiemannSphere[]` crea la esfera unidad 3D,
//! `RiemannProject[z]` proyecta y crea el Point3D, `RiemannUnproject[x,y,z]`
//! invierte con errores honestos (polo norte = ∞, fuera de esfera).
//! GeoGebra no tiene comandos de proyección estereográfica.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use grafito_command::{
    command_registry,
    commands::{process_input, CommandOutcome},
};
use grafito_core::{Document, GeoObject};

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
fn o7_riemann_visibles_y_ejecutan() {
    for canonical in ["RiemannSphere", "RiemannProject", "RiemannUnproject"] {
        let spec = command_registry::resolve(canonical)
            .unwrap_or_else(|| panic!("{canonical} debe estar registrado"));
        assert!(spec.palette_visible, "{canonical} visible en paleta");
        assert_eq!(spec.category, "Complejos");
    }
    let mut document = Document::new();
    // Esfera unidad creada.
    let outcome = run(&mut document, "RiemannSphere[]");
    assert!(matches!(outcome, CommandOutcome::Message(_)), "{outcome:?}");
    assert!(document.objects_iter().any(|(_, object)| {
        matches!(object, GeoObject::Sphere3D(sphere)
            if sphere.radius == 1.0
                && sphere.center.x == 0.0
                && sphere.center.y == 0.0
                && sphere.center.z == 0.0)
    }));
    // 1 ↦ (1, 0, 0) ecuador; 0 ↦ polo sur.
    assert_message_contains(
        &mut document,
        "RiemannProject[1]",
        "(1.000000, 0.000000, 0.000000)",
    );
    assert_message_contains(
        &mut document,
        "RiemannProject[0]",
        "(0.000000, 0.000000, -1.000000)",
    );
    // Inversa: ecuador ↦ 1, polo sur ↦ 0.
    assert_message_contains(&mut document, "RiemannUnproject[1, 0, 0]", "1.000");
    assert_message_contains(&mut document, "RiemannUnproject[0, 0, -1]", "0.000");
}

#[test]
fn o7_riemann_errores_honestos() {
    let mut document = Document::new();
    // Polo norte = ∞ en ambas direcciones; fuera de esfera.
    assert_error_contains(&mut document, "RiemannUnproject[0, 0, 1]", "polo norte");
    assert_error_contains(&mut document, "RiemannUnproject[0, 0, 0]", "esfera unidad");
    assert_error_contains(
        &mut document,
        "RiemannSphere[x]",
        "cantidad de argumentos inválida",
    );
}
