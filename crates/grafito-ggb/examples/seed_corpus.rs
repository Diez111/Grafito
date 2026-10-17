//! Siembra el corpus de fuzz de `ggb_import` con exports reales.
//!
//! Dogfood: los seeds son `.ggb` generados por [`grafito_ggb::export`]
//! (nuestro propio exportador), que es exactamente el shape que el
//! importador debe tolerar. Tres escenas: punto+punto+segmento, círculo
//! unitario y un triángulo con slider. `export_ggb_bytes` produce el zip
//! completo (XML + manifiesto); el roundtrip importa contra
//! `import_ggb_bytes` y corta si algún seed no re-importa.
//!
//! Corrida: `cargo run -p grafito-ggb --example seed_corpus`
//! (escribe en `../../fuzz/corpus/ggb_import/*.ggb`).

use grafito_ggb::export::{export_ggb_bytes, export_omitidos, GgbExportItem};

fn main() {
    let out_dir =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fuzz/corpus/ggb_import");
    std::fs::create_dir_all(&out_dir)
        .unwrap_or_else(|e| panic!("no se pudo crear {}: {e}", out_dir.display()));

    let escenas: &[(&str, Vec<GgbExportItem>)] = &[
        (
            "punto-segmento",
            vec![
                GgbExportItem::Point {
                    label: "A".into(),
                    x: -1.0,
                    y: 0.0,
                },
                GgbExportItem::Point {
                    label: "B".into(),
                    x: 1.0,
                    y: 0.5,
                },
                GgbExportItem::Segment {
                    label: "AB".into(),
                    from: "A".into(),
                    to: "B".into(),
                },
            ],
        ),
        (
            "circulo-unitario",
            vec![
                GgbExportItem::Point {
                    label: "O".into(),
                    x: 0.0,
                    y: 0.0,
                },
                GgbExportItem::Circle {
                    label: "c".into(),
                    cx: 0.0,
                    cy: 0.0,
                    r: 1.0,
                },
            ],
        ),
        (
            "triangulo-slider",
            vec![
                GgbExportItem::Slider {
                    label: "k".into(),
                    value: 2.0,
                    min: -5.0,
                    max: 5.0,
                },
                GgbExportItem::Point {
                    label: "A".into(),
                    x: 0.0,
                    y: 0.0,
                },
                GgbExportItem::Point {
                    label: "B".into(),
                    x: 3.0,
                    y: 0.0,
                },
                GgbExportItem::Point {
                    label: "C".into(),
                    x: 1.5,
                    y: 2.0,
                },
                GgbExportItem::Polygon {
                    label: "t1".into(),
                    vertices: vec!["A".into(), "B".into(), "C".into()],
                },
            ],
        ),
    ];

    let mut escritos = 0_usize;
    for (nombre, items) in escenas {
        let (bytes, report) =
            export_ggb_bytes(items).unwrap_or_else(|e| panic!("export de {nombre} falló: {e:?}"));
        // Dogfood roundtrip: nuestro importador debe tragar nuestro export.
        let _ = grafito_ggb::import_ggb_bytes(&bytes)
            .unwrap_or_else(|e| panic!("roundtrip de {nombre} falló: {e:?}"));
        let destino = out_dir.join(format!("{nombre}.ggb"));
        std::fs::write(&destino, &bytes)
            .unwrap_or_else(|e| panic!("write {}: {e}", destino.display()));
        escritos += 1;
        println!(
            "seed {nombre}: {} bytes, {} ítems, omitidos {}",
            bytes.len(),
            report.escritos,
            export_omitidos_count(&report),
        );
    }
    println!("{escritos} seeds en {}", out_dir.display());
}

fn export_omitidos_count(report: &grafito_ggb::export::ExportReport) -> usize {
    export_omitidos(report).len()
}
