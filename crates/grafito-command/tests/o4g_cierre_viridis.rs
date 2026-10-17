//! O6 — paleta viridis en heat maps: `HeatMap[..., paleta]` con
//! 0 = térmica legacy (default, paridad) y 1 = viridis (perceptual +
//! daltónica). GeoGebra no ofrece rampa daltónica en mapas de calor.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use grafito_command::commands::{process_input, CommandOutcome};
use grafito_core::{Document, GeoObject};

fn run(document: &mut Document, command: &str) -> CommandOutcome {
    let mut input = command.to_owned();
    process_input(document, &mut input)
}

fn heat_palette(document: &Document) -> Option<u8> {
    document
        .objects_iter()
        .find_map(|(_, object)| match object {
            GeoObject::ComplexGrid(grid) if grid.render_mode == 2 => Some(grid.palette),
            _ => None,
        })
}

#[test]
fn o6_heatmap_palette_defaults_thermal_and_accepts_viridis() {
    let mut document = Document::new();
    let outcome = run(&mut document, "HeatMap[x+y, -2, 2, -2, 2, 16]");
    assert!(matches!(outcome, CommandOutcome::Message(_)), "{outcome:?}");
    assert_eq!(heat_palette(&document), Some(0));

    let mut document = Document::new();
    let outcome = run(&mut document, "HeatMap[x+y, -2, 2, -2, 2, 16, 1]");
    assert!(matches!(outcome, CommandOutcome::Message(_)), "{outcome:?}");
    assert_eq!(heat_palette(&document), Some(1));
}

#[test]
fn o6_heatmap_rejects_unknown_palette_atomically() {
    for command in [
        "HeatMap[x+y, -2, 2, -2, 2, 16, 2]",
        "HeatMap[x+y, -2, 2, -2, 2, 16, viridis]",
    ] {
        let mut document = Document::new();
        let before = serde_json::to_value(&document).unwrap();
        let outcome = run(&mut document, command);
        assert!(
            matches!(outcome, CommandOutcome::Error(_)),
            "{command}: {outcome:?}"
        );
        assert_eq!(
            serde_json::to_value(&document).unwrap(),
            before,
            "{command}"
        );
    }
}
