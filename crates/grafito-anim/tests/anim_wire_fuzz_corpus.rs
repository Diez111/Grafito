//! Golden del corpus semilla de fuzz `anim_wire` contra producción.
//!
//! El corpus commiteado (`fuzz/corpus/anim_wire/`, 9 vectores) lo consumen el
//! target libFuzzer (`fuzz/fuzz_targets/anim_wire.rs`, semanal) y este test:
//! cada vector se pineea contra el Rust de producción que el fuzzer espeja
//! (`protocol::try_downcast`, `engine::DEFAULT_LINE_CAP_BYTES`,
//! `guion::math_expr_valida_para_cas`). Si el corpus cambia, este test cae
//! y obliga a actualizar el pin (no hay skips).
//!
//! Capas honestas que el test deja por escrito:
//! - Los 3 `.expr` PASAN el hook Rust de forma (`math_expr_valida_para_cas`):
//!   el rechazo de `z^2` (solo `x`) y de `__import__` (dunder) vive en el
//!   worker (`engines/python/manim_engine/__main__.py`: `SAFE_FUNCS`,
//!   `MAX_NODES`, `MAX_EXPR_LEN`, rechazo `__`) y en el CAS-gate de
//!   pedagogía, no acá.
//! - El vector oversize (66 KiB) lo rechaza el transporte por bytes
//!   (`> DEFAULT_LINE_CAP_BYTES`) y, si llegara al wire, `try_downcast`
//!   lo rechaza por forma (sin `job_id`, requerido).
//!
//! Sin dependencias nuevas (crate + std), sin red, sin spawnear workers.
//! Sin `expect`/`unwrap` (el workspace los niega con `-D warnings`): todo
//! fallo se reporta con `panic!` y contexto.

use grafito_anim::engine::DEFAULT_LINE_CAP_BYTES;
use grafito_anim::guion::math_expr_valida_para_cas;
use grafito_anim::protocol::{try_downcast, WireMessage};
use std::path::PathBuf;

fn corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fuzz/corpus/anim_wire")
}

fn lee(nombre: &str) -> Vec<u8> {
    let ruta = corpus_dir().join(nombre);
    match std::fs::read(&ruta) {
        Ok(bytes) => bytes,
        Err(e) => panic!("corpus ilegible {ruta:?}: {e}"),
    }
}

fn carga_json(nombre: &str) -> serde_json::Value {
    let raw = lee(nombre);
    match serde_json::from_slice(&raw) {
        Ok(valor) => valor,
        Err(e) => panic!("{nombre} no es JSON válido: {e}"),
    }
}

#[test]
fn corpus_tiene_los_9_vectores_pineados() {
    let dir = corpus_dir();
    let entradas = match std::fs::read_dir(&dir) {
        Ok(it) => it,
        Err(e) => panic!("sin corpus {dir:?}: {e}"),
    };
    let mut nombres: Vec<String> = Vec::new();
    for entrada in entradas {
        match entrada {
            Ok(e) => nombres.push(e.file_name().to_string_lossy().into_owned()),
            Err(e) => panic!("entrada de corpus ilegible: {e}"),
        }
    }
    nombres.sort();
    assert_eq!(
        nombres,
        [
            "01_z_squared.expr",
            "02_exp_x.expr",
            "03_dunder_import.expr",
            "04_hello_v1.json",
            "05_progress_60.json",
            "06_render_result.json",
            "07_error.json",
            "08_pong.json",
            "09_oversize_66k.json",
        ],
        "el corpus cambió: actualizar pin + docs de fuzz"
    );
    // Tamaños pineados de los vectores de expresión (forma exacta del seed).
    assert_eq!(lee("01_z_squared.expr"), b"z^2\n");
    assert_eq!(lee("02_exp_x.expr"), b"exp(x)\n");
    assert_eq!(lee("03_dunder_import.expr"), b"__import__('os')\n");
}

#[test]
fn wire_json_del_corpus_contra_try_downcast_de_produccion() {
    match try_downcast(&carga_json("04_hello_v1.json")) {
        Ok(WireMessage::Hello {
            protocol_version,
            capabilities,
        }) => {
            assert_eq!(protocol_version, 1);
            assert_eq!(capabilities, ["derivative-slope", "integral-area"]);
        }
        Ok(otro) => panic!("04 debe ser Hello, fue {otro:?}"),
        Err(e) => panic!("04 hello v1 debe parsear: {e:?}"),
    }

    match try_downcast(&carga_json("05_progress_60.json")) {
        Ok(WireMessage::Progress(p)) => {
            assert_eq!(p.job_id, "job-1");
            assert_eq!(p.percent, 60);
            assert!((p.fraction() - 0.6).abs() < f32::EPSILON);
        }
        Ok(otro) => panic!("05 debe ser Progress, fue {otro:?}"),
        Err(e) => panic!("05 progress debe parsear: {e:?}"),
    }

    match try_downcast(&carga_json("06_render_result.json")) {
        Ok(WireMessage::Result(r)) => {
            assert_eq!(r.job_id, "job-1");
            assert_eq!(r.frames, 1);
            assert_eq!(r.duration_ms, 120);
            assert!(!r.media_path.is_empty());
        }
        Ok(otro) => panic!("06 debe ser Result, fue {otro:?}"),
        Err(e) => panic!("06 result debe parsear: {e:?}"),
    }

    match try_downcast(&carga_json("07_error.json")) {
        Ok(WireMessage::Error { code, message }) => {
            assert_eq!(code, "render_failed");
            assert_eq!(message, "boom");
        }
        Ok(otro) => panic!("07 debe ser Error, fue {otro:?}"),
        Err(e) => panic!("07 error debe parsear: {e:?}"),
    }

    match try_downcast(&carga_json("08_pong.json")) {
        Ok(WireMessage::Pong) => {}
        Ok(otro) => panic!("08 debe ser Pong, fue {otro:?}"),
        Err(e) => panic!("08 pong debe parsear: {e:?}"),
    }
}

#[test]
fn oversize_del_corpus_supera_el_line_cap_de_produccion() {
    let raw = lee("09_oversize_66k.json");
    assert_eq!(DEFAULT_LINE_CAP_BYTES, 64 * 1024);
    assert!(
        raw.len() > DEFAULT_LINE_CAP_BYTES,
        "09 debe seguir superando el cap: {} <= {DEFAULT_LINE_CAP_BYTES}",
        raw.len()
    );
    // Honesto en dos capas: el transporte lo descarta por bytes antes de
    // parsear (oversized-line Error) y, si llegara al wire, producción lo
    // rechaza por forma (le falta `job_id`, requerido en `RenderProgress`).
    let valor = carga_json("09_oversize_66k.json");
    match try_downcast(&valor) {
        Ok(otro) => panic!("09 sin job_id debe fallar a nivel wire, parseó {otro:?}"),
        Err(e) => assert!(
            format!("{e:?}").contains("job_id"),
            "el rechazo wire cita el campo faltante: {e:?}"
        ),
    }
}

#[test]
fn progreso_fuera_de_rango_se_rechaza_tipado() {
    // Contrato que el fuzzer explora: `percent` es u8 pero >100 es error.
    let valor = serde_json::json!({"type": "progress", "job_id": "job-1", "percent": 101});
    match try_downcast(&valor) {
        Ok(otro) => panic!("percent 101 debe fallar, parseó {otro:?}"),
        Err(e) => assert!(
            format!("{e:?}").contains("101"),
            "el error cita el valor: {e:?}"
        ),
    }
}

#[test]
fn shape_hook_de_exprs_del_corpus_documenta_las_capas() {
    // Los 3 pasan la FORMA local (el hook no corre el CAS ni conoce SAFE_FUNCS):
    // el rechazo real de `z` y de `__` vive en el worker + pedagogía.
    for (nombre, esperado_forma) in [
        ("01_z_squared.expr", true),
        ("02_exp_x.expr", true),
        ("03_dunder_import.expr", true),
    ] {
        let raw = lee(nombre);
        let src = match String::from_utf8(raw) {
            Ok(s) => s,
            Err(e) => panic!("{nombre} no es UTF-8: {e}"),
        };
        assert_eq!(
            math_expr_valida_para_cas(src.trim()),
            esperado_forma,
            "{nombre}: cambió el hook de forma"
        );
    }
}
