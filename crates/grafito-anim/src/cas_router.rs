//! Router puro paso CAS → escena (sin UI, sin motor CAS, sin Manim).
//!
//! Capa pura que convierte cada [`CasStepView`] (foto de
//! `grafito-geometry::cas_steps::CasStep`: `before --rule--> after`) en una
//! escena `{template, params, effect, easing}` y ensambla la secuencia en
//! chunks de timeline que respetan los presupuestos del player.
//!
//! - Mapping pineado para las 16 reglas de más tráfico (10 plantillas,
//!   todas en [`crate::protocol::CANONICAL_TEMPLATES`]): cadena, potencia,
//!   integrales (Barrow/Riemann), Taylor con resto, simplificación,
//!   Laplace (directa/inversa/cálculo/shift), sistemas lineales (puente
//!   honesto a `matriz-transformacion`: no existe regla de matrices en el
//!   CAS, ver [`REGLAS_MAPEADAS`]), EDOs (1er orden + homogénea de 2º
//!   orden → `edo-campo-direcciones`) y stats (descriptiva → `universal`
//!   pineado por decisión; regresión → `regresion-lineal`).
//! - Fallback honesto [`TEMPLATE_FALLBACK`] (`universal`: placeholder
//!   grilla+texto que renderiza la Piel, como hoy) para el resto.
//! - Ensamblador [`assemble_timeline`]: frames por ítem 1..=48, total por
//!   chunk ≤96 (paridad [`crate::player::PLAYER_MAX_FRAMES`] y
//!   [`crate::player::PLAYER_MAX_TOTAL_FRAMES`]); si la secuencia excede,
//!   compone por chunks (jamás un `Vec` total más grande que lo necesario).
//! - Guion/captions por paso con la infra existente ([`compilar_paso_cas`]
//!   vía `PasoGuion::try_new`, [`captions_del_chunk`] vía
//!   `voiceover_segments`): si un paso no compila, es `Err` honesto, jamás
//!   escena inventada en silencio.
//!
//! PROHIBIDO acá: UI, actions, i18n, tocar el motor CAS, plantillas fuera
//! de las 69 canónicas, Manim. Los `params` viajan vacíos a propósito: el
//! router no parsea expresiones (eso es el CAS); cada plantilla usa sus
//! defaults honestos.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::captions::{voiceover_segments, CaptionError, CaptionTrack};
use crate::guion::{
    EfectoPaso, GuionError, PasoGuion, PasoTexto, CONCEPTO_MAX_CHARS, VOICEOVER_MAX_PALABRAS,
    WHITEBOARD_MAX_CHARS,
};
use crate::player::{PLAYER_MAX_FRAMES, PLAYER_MAX_TOTAL_FRAMES};
use crate::scene::RateFunc;

// ── Constantes ──────────────────────────────────────────────────────────────

/// Plantilla fallback honesta (placeholder grilla+texto de la Piel).
pub const TEMPLATE_FALLBACK: &str = "universal";
/// Frames por defecto de cada paso (entra en guion 4..=16 y player 1..=48).
pub const STEP_DEFAULT_FRAMES: usize = 12;
/// Corrida por defecto de cada paso en ms (100..=60000 en ambos mundos).
pub const STEP_DEFAULT_RUN_MS: u64 = 2000;
/// Tope de pasos por pedido (paridad `MAX_CAS_STEPS` del CAS).
pub const ROUTER_MAX_STEPS: usize = 32;

/// Reglas con mapping pineado (nombres `Display` de `RewriteRule`).
/// Todo lo demás cae al fallback honesto [`TEMPLATE_FALLBACK`].
pub const REGLAS_MAPEADAS: &[&str] = &[
    "ChainRule",
    "PowerRule",
    "IntegrationPowerRule",
    "IntegrationConstantRule",
    "IntegrationSumRule",
    "TaylorRule",
    "Simplify",
    "LaplaceDirectRule",
    "LaplaceInverseRule",
    "LaplaceCalculusRule",
    "LaplaceShiftRule",
    "SolveLinearRule",
    "OdeFirstOrderRule",
    "OdeSecondHomogRule",
    "StatsDescriptiveRule",
    "StatsRegressionRule",
];

// ── Vista de paso ───────────────────────────────────────────────────────────

/// Foto owned de un paso CAS (`before --rule--> after`).
///
/// `rule` es el nombre estable (`Display` de `RewriteRule`); reglas
/// desconocidas son válidas y caen al fallback (jamás `Err` en el ruteo).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CasStepView {
    /// Índice del paso en la secuencia.
    pub index: usize,
    /// Nombre estable de la regla (p. ej. `"ChainRule"`).
    pub rule: String,
    /// Expresión antes del paso.
    pub before: String,
    /// Expresión después del paso.
    pub after: String,
    /// Descripción pedagógica en rioplatense.
    pub description: String,
}

impl CasStepView {
    /// Constructor directo (para reglas sin `CasStep`, tests incluidos).
    pub fn new(index: usize, rule: &str, before: &str, after: &str, description: &str) -> Self {
        Self {
            index,
            rule: rule.to_string(),
            before: before.to_string(),
            after: after.to_string(),
            description: description.to_string(),
        }
    }
}

impl From<&grafito_geometry::cas_steps::CasStep> for CasStepView {
    fn from(paso: &grafito_geometry::cas_steps::CasStep) -> Self {
        Self {
            index: paso.index,
            rule: paso.rule.to_string(),
            before: paso.before.clone(),
            after: paso.after.clone(),
            description: paso.description.clone(),
        }
    }
}

// ── Escena ruteada ──────────────────────────────────────────────────────────

/// Escena para un paso: `{template, params, effect, easing}` + presupuestos.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepScene {
    /// Regla que la originó (para traza honesta).
    pub rule: String,
    /// Plantilla canónica (siempre una de las 69).
    pub template: String,
    /// Parámetros numéricos (vacíos: defaults honestos de la plantilla).
    pub params: BTreeMap<String, f64>,
    /// Efecto de revelado.
    pub effect: EfectoPaso,
    /// Easing del progreso.
    pub easing: RateFunc,
    /// Frames del ítem (1..=48).
    pub frames: usize,
    /// Corrida en ms (100..=60000).
    pub run_ms: u64,
}

impl StepScene {
    /// ¿Es el fallback honesto?
    pub fn es_fallback(&self) -> bool {
        self.template == TEMPLATE_FALLBACK
    }
}

/// Tabla pineada regla → (plantilla, efecto, easing).
///
/// - `ChainRule` → `chain-rule` (la sonda barre adentro/afuera: `Tracker`).
/// - `PowerRule` → `derivative-slope` (resalta la tangente: `Indicate`).
/// - `IntegrationPowerRule`/`IntegrationConstantRule` → `integral-area`
///   (Barrow: el área se traza: `Create`).
/// - `IntegrationSumRule` → `riemann-sums` (rectángulos parejos: `Linear`).
/// - `TaylorRule` → `taylor-remainder` (los órdenes se revelan: `Write`).
/// - `Simplify` → `universal` explícito (reescritura de texto: `Write`;
///   pineado igual que el fallback pero por decisión, no por omisión).
/// - Laplace ×4 → `edo-laplace` (cambio de dominio: `Fade`).
/// - `SolveLinearRule` → `matriz-transformacion` (el sistema lineal es la
///   acción de la matriz, `M(t)` de `I` a `A`: `Tracker`). Puente honesto:
///   el CAS no tiene regla de matrices; se documenta y reporta.
/// - EDO ×2 → `edo-campo-direcciones` (el campo se traza: `Create`).
/// - `StatsDescriptiveRule` → `universal` explícito (tabla de números:
///   `Write`; pineado por decisión, no por omisión).
/// - `StatsRegressionRule` → `regresion-lineal` (la recta se traza:
///   `Create`).
/// - Resto → (`universal`, `Write`, `Linear`): placeholder grilla+texto.
fn tabla_rule(rule: &str) -> (&'static str, EfectoPaso, RateFunc) {
    match rule {
        "ChainRule" => ("chain-rule", EfectoPaso::Tracker, RateFunc::Smooth),
        "PowerRule" => ("derivative-slope", EfectoPaso::Indicate, RateFunc::Smooth),
        "IntegrationPowerRule" | "IntegrationConstantRule" => {
            ("integral-area", EfectoPaso::Create, RateFunc::Smooth)
        }
        "IntegrationSumRule" => ("riemann-sums", EfectoPaso::Create, RateFunc::Linear),
        "TaylorRule" => ("taylor-remainder", EfectoPaso::Write, RateFunc::Smooth),
        "Simplify" => (TEMPLATE_FALLBACK, EfectoPaso::Write, RateFunc::Linear),
        "LaplaceDirectRule" | "LaplaceInverseRule" | "LaplaceCalculusRule" | "LaplaceShiftRule" => {
            ("edo-laplace", EfectoPaso::Fade, RateFunc::Smooth)
        }
        "SolveLinearRule" => (
            "matriz-transformacion",
            EfectoPaso::Tracker,
            RateFunc::Smooth,
        ),
        "OdeFirstOrderRule" | "OdeSecondHomogRule" => (
            "edo-campo-direcciones",
            EfectoPaso::Create,
            RateFunc::Smooth,
        ),
        "StatsDescriptiveRule" => (TEMPLATE_FALLBACK, EfectoPaso::Write, RateFunc::Linear),
        "StatsRegressionRule" => ("regresion-lineal", EfectoPaso::Create, RateFunc::Smooth),
        _ => (TEMPLATE_FALLBACK, EfectoPaso::Write, RateFunc::Linear),
    }
}

/// Rutea una regla (total: desconocida → fallback honesto, jamás `Err`).
#[must_use]
pub fn route_rule(rule: &str) -> StepScene {
    let (template, effect, easing) = tabla_rule(rule.trim());
    StepScene {
        rule: rule.trim().to_string(),
        template: template.to_string(),
        params: BTreeMap::new(),
        effect,
        easing,
        frames: STEP_DEFAULT_FRAMES,
        run_ms: STEP_DEFAULT_RUN_MS,
    }
}

/// Rutea un paso CAS a su escena.
#[must_use]
pub fn route_step(paso: &CasStepView) -> StepScene {
    route_rule(&paso.rule)
}

// ── Guion por paso (infra existente) ────────────────────────────────────────

/// Trunca a `n` chars (borde UTF-8 honesto). Pura.
fn trunca_chars(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        s.chars().take(n).collect()
    }
}

/// Efecto tipado → string del wire del guion (parseo total). Pura.
fn efecto_str(effect: EfectoPaso) -> &'static str {
    match effect {
        EfectoPaso::Create => "create",
        EfectoPaso::Write => "write",
        EfectoPaso::Fade => "fade",
        EfectoPaso::Grow => "grow",
        EfectoPaso::Indicate => "indicate",
        EfectoPaso::Tracker => "tracker",
        EfectoPaso::Wait => "wait",
    }
}

/// Arma el `PasoTexto` de un paso CAS (saneos honestos y documentados):
/// - `texto`: `description` (≤500 chars); vacía → `"Paso {i} ({rule}):
///   {before} → {after}"` (nunca vacío).
/// - `math_expr`: `after` si pasa la forma CAS, si no `None` (el paso sigue).
/// - `whiteboard_hint`: `"{before} → {after}"` (≤200 chars, puede vacía).
/// - `voiceover`: primeras 40 palabras de `description`; vacía → `None`
///   (hueco silencioso legítimo).
#[must_use]
pub fn paso_texto_para_cas(paso: &CasStepView) -> PasoTexto {
    let escena = route_step(paso);
    let desc = paso
        .description
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let voz: Option<String> = {
        let palabras: Vec<&str> = desc
            .split_whitespace()
            .take(VOICEOVER_MAX_PALABRAS)
            .collect();
        if palabras.is_empty() {
            None
        } else {
            Some(palabras.join(" "))
        }
    };
    let texto = if desc.trim().is_empty() {
        format!(
            "Paso {} ({}): {} → {}",
            paso.index, escena.rule, paso.before, paso.after
        )
    } else {
        desc
    };
    PasoTexto {
        texto: trunca_chars(texto.trim(), CONCEPTO_MAX_CHARS),
        math_expr: crate::guion::sanear_math_expr(Some(paso.after.clone())),
        whiteboard_hint: trunca_chars(
            format!("{} → {}", paso.before, paso.after).trim(),
            WHITEBOARD_MAX_CHARS,
        ),
        template_hint: escena.template,
        params: escena.params,
        efecto: efecto_str(escena.effect).to_string(),
        frames: escena.frames,
        run_ms: escena.run_ms,
        wait_after_ms: 0,
        voiceover: voz,
    }
}

/// Compila un paso CAS con la validación existente (`Err` honesto si algo
/// excede: jamás escena inventada).
pub fn compilar_paso_cas(paso: &CasStepView) -> Result<PasoGuion, GuionError> {
    PasoGuion::try_new(paso_texto_para_cas(paso))
}

// ── Ensamblador por chunks ──────────────────────────────────────────────────

/// Chunk de timeline: pasos compilados cuyo total entra en el player.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineChunk {
    /// Pasos del chunk, en orden.
    pub items: Vec<PasoGuion>,
    /// Suma de frames (1..=96).
    pub total_frames: usize,
}

impl TimelineChunk {
    /// ¿Vacío?
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    /// Cantidad de pasos.
    pub fn len(&self) -> usize {
        self.items.len()
    }
}

/// Ensambla pasos CAS en chunks de timeline.
///
/// Compila cada paso (el que no pasa es `Err` honesto con su índice) y
/// corta chunks cuando el siguiente ítem excedería
/// [`PLAYER_MAX_TOTAL_FRAMES`] (cada ítem ya trae 1..=48 por
/// construcción). Secuencia vacía → `Vec` vacío (nada que componer).
/// Más de [`ROUTER_MAX_STEPS`] pasos es `Err` honesto (paridad CAS).
pub fn assemble_timeline(pasos: &[CasStepView]) -> Result<Vec<TimelineChunk>, GuionError> {
    if pasos.len() > ROUTER_MAX_STEPS {
        return Err(GuionError::Invalido {
            campo: "pasos",
            motivo: format!(
                "{} pasos (válido 0..={ROUTER_MAX_STEPS}): partí el pedido",
                pasos.len()
            ),
        });
    }
    let mut chunks: Vec<TimelineChunk> = Vec::new();
    let mut actual = TimelineChunk {
        items: Vec::new(),
        total_frames: 0,
    };
    for paso in pasos {
        let compilado = compilar_paso_cas(paso).map_err(|e| GuionError::Invalido {
            campo: "pasos",
            motivo: format!("paso {}: {e}", paso.index),
        })?;
        // Defensa honesta aunque el default ya cumple (12 ∈ 1..=48).
        let frames = compilado.frames.clamp(1, PLAYER_MAX_FRAMES);
        if !actual.items.is_empty() && actual.total_frames + frames > PLAYER_MAX_TOTAL_FRAMES {
            chunks.push(actual);
            actual = TimelineChunk {
                items: Vec::new(),
                total_frames: 0,
            };
        }
        actual.total_frames += frames;
        actual.items.push(compilado);
    }
    if !actual.items.is_empty() {
        chunks.push(actual);
    }
    Ok(chunks)
}

/// Captions de un chunk con la infra existente (un segmento por paso con
/// voz; pasos mudos se saltean, como `voiceover_segments` manda).
/// Duración por paso = su `run_ms` (≥ palabras: 2000 ms ≫ 40 palabras).
pub fn captions_del_chunk(chunk: &TimelineChunk) -> Result<CaptionTrack, CaptionError> {
    let duraciones: Vec<u32> = chunk
        .items
        .iter()
        .map(|p| p.run_ms.min(u64::from(u32::MAX)) as u32)
        .collect();
    voiceover_segments(&chunk.items, &duraciones)
}

// ── Tests inline (mismo gate `cargo test`, sin cableado extra) ──────────────

#[cfg(test)]
mod cas_router_tests {
    use super::*;
    use crate::protocol::CANONICAL_TEMPLATES;

    fn vista(rule: &str) -> CasStepView {
        CasStepView::new(
            0,
            rule,
            "x^2",
            "2*x",
            "Deriva término a término con la regla de la potencia.",
        )
    }

    #[test]
    fn mappings_pineados_regla_a_template() {
        // Tabla pineada: (regla, template, es_fallback).
        let casos = [
            ("ChainRule", "chain-rule", false),
            ("PowerRule", "derivative-slope", false),
            ("IntegrationPowerRule", "integral-area", false),
            ("IntegrationConstantRule", "integral-area", false),
            ("IntegrationSumRule", "riemann-sums", false),
            ("TaylorRule", "taylor-remainder", false),
            ("Simplify", "universal", true),
            ("LaplaceDirectRule", "edo-laplace", false),
            ("LaplaceInverseRule", "edo-laplace", false),
            ("LaplaceCalculusRule", "edo-laplace", false),
            ("LaplaceShiftRule", "edo-laplace", false),
            ("SolveLinearRule", "matriz-transformacion", false),
            ("OdeFirstOrderRule", "edo-campo-direcciones", false),
            ("OdeSecondHomogRule", "edo-campo-direcciones", false),
            ("StatsDescriptiveRule", "universal", true),
            ("StatsRegressionRule", "regresion-lineal", false),
        ];
        assert_eq!(casos.len(), REGLAS_MAPEADAS.len());
        for (rule, template, fallback) in casos {
            assert!(
                REGLAS_MAPEADAS.contains(&rule),
                "{rule} pineada pero fuera de REGLAS_MAPEADAS"
            );
            let escena = route_rule(rule);
            assert_eq!(escena.template, template, "regla {rule}");
            assert_eq!(escena.es_fallback(), fallback, "regla {rule}");
            assert!(
                CANONICAL_TEMPLATES.contains(&escena.template.as_str()),
                "{template} fuera de las 69 canónicas"
            );
            assert!((1..=PLAYER_MAX_FRAMES).contains(&escena.frames));
            assert!((100..=60_000).contains(&escena.run_ms));
            // El paso rutea igual que la regla sola.
            assert_eq!(route_step(&vista(rule)), escena);
        }
    }

    #[test]
    fn fallback_para_regla_desconocida() {
        for rara in ["ReglaInventada", "", "chainrule", "Chain Rule", "GENERIC"] {
            let escena = route_rule(rara);
            assert_eq!(escena.template, TEMPLATE_FALLBACK, "regla {rara:?}");
            assert!(escena.es_fallback());
            assert_eq!(escena.effect, EfectoPaso::Write);
            // Compila igual con la infra existente (placeholder honesto).
            let paso = compilar_paso_cas(&vista(rara)).unwrap();
            assert_eq!(paso.template, TEMPLATE_FALLBACK);
        }
    }

    #[test]
    fn presupuestos_chunks_hasta_96() {
        // 20 pasos × 12 frames = 240 → 3 chunks de 8 (96 exactos).
        let pasos: Vec<CasStepView> = (0..20)
            .map(|i| {
                CasStepView::new(
                    i,
                    "ChainRule",
                    "sin(x^2)",
                    "2*x*cos(x^2)",
                    "Deriva afuera y multiplica por la de adentro.",
                )
            })
            .collect();
        let chunks = assemble_timeline(&pasos).unwrap();
        assert_eq!(chunks.len(), 3);
        for ch in &chunks {
            assert!(ch.total_frames <= PLAYER_MAX_TOTAL_FRAMES);
            assert_eq!(
                ch.total_frames,
                ch.items.iter().map(|p| p.frames).sum::<usize>()
            );
            for p in &ch.items {
                assert!(
                    (1..=PLAYER_MAX_FRAMES).contains(&p.frames),
                    "frames {}",
                    p.frames
                );
                assert!((4..=16).contains(&p.frames), "guion 4..=16, {}", p.frames);
            }
        }
        // 8 pasos × 12 = 96: un solo chunk exacto.
        let ocho: Vec<CasStepView> = (0..8)
            .map(|i| {
                CasStepView::new(
                    i,
                    "PowerRule",
                    "x^3",
                    "3*x^2",
                    "Baja el exponente y restale uno.",
                )
            })
            .collect();
        let un = assemble_timeline(&ocho).unwrap();
        assert_eq!(un.len(), 1);
        assert_eq!(un[0].total_frames, 96);
        // Vacío → vacío; exceso → Err honesto.
        assert!(assemble_timeline(&[]).unwrap().is_empty());
        let muchos: Vec<CasStepView> = (0..33).map(|_| vista("Simplify")).collect();
        assert!(assemble_timeline(&muchos).is_err());
    }

    #[test]
    fn roundtrip_de_la_timeline() {
        let pasos = [
            vista("ChainRule"),
            vista("TaylorRule"),
            vista("OdeSecondOrderRule"),
        ];
        let chunks = assemble_timeline(&pasos).unwrap();
        assert_eq!(chunks.len(), 1);
        let json = serde_json::to_string(&chunks).unwrap();
        let de_vuelta: Vec<TimelineChunk> = serde_json::from_str(&json).unwrap();
        assert_eq!(
            serde_json::to_string(&de_vuelta).unwrap(),
            json,
            "roundtrip JSON estable"
        );
        // Escena sola también hace roundtrip.
        let escena = route_rule("LaplaceDirectRule");
        let e2: StepScene = serde_json::from_str(&serde_json::to_string(&escena).unwrap()).unwrap();
        assert_eq!(e2, escena);
    }

    #[test]
    fn guion_y_captions_por_paso() {
        let pasos = [vista("PowerRule"), vista("ReglaInventada")];
        let chunks = assemble_timeline(&pasos).unwrap();
        assert_eq!(chunks.len(), 1);
        let pista = captions_del_chunk(&chunks[0]).unwrap();
        // Ambos pasos tienen voz (descripción no vacía) → 2 segmentos.
        assert_eq!(pista.len(), 2);
        assert!(pista.segments[0].start_ms < pista.segments[0].end_ms);
        assert!(
            pista.segments[0].end_ms <= pista.segments[1].start_ms
                || pista.segments[1].start_ms == pista.segments[0].end_ms
        );
        // Paso mudo (sin descripción): guion sin voz, caption salteado.
        let mudo = CasStepView::new(7, "Simplify", "x+x", "2*x", "   ");
        let pm = compilar_paso_cas(&mudo).unwrap();
        assert!(pm.voiceover.is_none());
        assert!(!pm.texto.is_empty());
        let ch_mudo = assemble_timeline(&[mudo]).unwrap();
        assert!(captions_del_chunk(&ch_mudo[0]).unwrap().is_empty());
    }

    #[test]
    fn casstep_real_entra_por_from() {
        let real = grafito_geometry::cas_steps::CasStep {
            index: 3,
            rule: grafito_geometry::cas_steps::RewriteRule::ChainRule,
            before: "sin(x^2)".to_string(),
            after: "2*x*cos(x^2)".to_string(),
            description: "Deriva afuera y multiplica por la de adentro.".to_string(),
        };
        let v = CasStepView::from(&real);
        assert_eq!(v.rule, "ChainRule");
        assert_eq!(route_step(&v).template, "chain-rule");
        assert!(compilar_paso_cas(&v).is_ok());
    }
}
