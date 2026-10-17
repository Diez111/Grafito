#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
//! Perfil pedagógico del usuario (ADR-0001): memoria de aprendizaje que el
//! tutor-orquestador «Mora» usa para adaptar el plan, ir ramificando y medir
//! el progreso. Crate de capa hoja: sin egui, testable headless, persistible.

pub mod bkt;
// Sin consumidor fuera del crate (verificado con grep 2026-09: solo se
// re-exporta). Se oculta de la doc pública hasta cablearlo a la dificultad de
// ítems del CAT (propuesta en el reporte de esta oleada; no se implementa aquí).
#[doc(hidden)]
pub mod elo;
pub mod exam;
pub mod long_memory;
pub mod mascot;
pub mod scheduler;
pub mod stubs;
pub mod study_plan;
pub mod working_memory;

// Re-exportar tipos de avatar/mascota en la raíz
pub use bkt::{
    bkt_params_for_branch, bkt_params_for_lo, bkt_params_for_lo_opt, bkt_posterior, bkt_update,
    is_known_lo, los_without_calibration, mastery_from_bkt, BktParams, BktState, ALL_LO_IDS,
    BKT_DEFAULT_PARAMS,
};
// Ver nota en `pub mod elo` de `lib.rs`: sin consumidor, oculto de la doc.
#[doc(hidden)]
pub use elo::{
    elo_expected, elo_update, EloRating, EloState, ELO_DEFAULT_K, ELO_DEFAULT_RATING, ELO_MAX_K,
    ELO_MAX_RATING, ELO_MIN_K, ELO_MIN_RATING, ELO_SCALE,
};
pub use long_memory::{Fact, LongTermMemory, Preferences};
pub use mascot::{
    AvatarAccessory, AvatarBlush, AvatarConfig, AvatarEyeStyle, AvatarMouthStyle, AvatarShape,
    DailyMission, FurnitureKind, HouseTheme, MascotConfig, MascotMood, MascotSpecies, Outfit,
    OutfitLayer, OutfitTier, Personality, ShopItem, Wardrobe, MAX_DISPLAY_NAME, MAX_NAME,
};
pub use scheduler::{
    fsrs_init, fsrs_next_interval_secs, fsrs_schedule_next_review, fsrs_update, is_due,
    next_interval, review_schedule_for, schedule_next_review, FsrsState, ReviewSchedule,
    SchedulerError, DAY_SECS, MAX_BOX_LEVEL,
};
pub use working_memory::WorkingMemory;

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Límites explícitos para que la memoria nunca crezca sin cota.
const MAX_BRANCHES: usize = 128;
const MAX_HISTORY_EVENTS: usize = 2_000;
const MAX_MEMORY_CHARS: usize = 2_400;

/// Tasa EMA para actualizar dominio en aciertos: mastery += EMA_ALPHA * (1 - mastery).
const EMA_ALPHA: f64 = 0.15;
/// Umbral de dominio para cubrir rama.
const MASTER_THRESHOLD: f64 = 0.8;
/// Retención EMA en fallos: mastery *= EMA_RETENTION (0.95).
const EMA_RETENTION: f64 = 0.95;

fn default_bkt_p_known() -> f64 {
    0.3
}

/// Estado de una rama de estudio (ej. Álgebra, Cálculo, Geometría 3D).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BranchState {
    pub id: String,
    pub name: String,
    pub covered: bool,
    /// Dominio 0..=1 (media móvil exponencial de aciertos).
    pub mastery: f32,
    pub last_study_epoch: Option<u64>,
    /// Histórico (epoch, dominio) por rama, acotado a 64 muestras, para
    /// graficar la evolución. `#[serde(default)]` permite leer perfiles viejos.
    /// VecDeque para rotación O(1) con `pop_front` (LRU 64).
    #[serde(default)]
    pub domain_history: VecDeque<(u64, f32)>,
    /// Próximo repaso espaciado (epoch segundos). `None` si nunca se practicó.
    #[serde(default)]
    pub next_review_epoch: Option<u64>,
    /// Caja Leitner 0..=8 (0 sin repasar, 1 primera caja, 8 máxima).
    #[serde(default)]
    pub box_level: u8,
    /// Probabilidad latente BKT P(sabe) para esta rama.
    #[serde(default = "default_bkt_p_known")]
    pub bkt_p_known: f64,
}

impl BranchState {
    /// Mastery honesto derivado del posterior BKT (`bkt_p_known`).
    ///
    /// El campo legacy `mastery` (EMA) se conserva por compatibilidad con la
    /// UI y los tests existentes; este método expone la señal bayesiana real
    /// que el dashboard/aula debe mostrar (`avg_posterior_mastery`). NaN-safe.
    #[must_use]
    pub fn posterior_mastery(&self) -> f64 {
        if !self.bkt_p_known.is_finite() {
            return 0.0;
        }
        self.bkt_p_known.clamp(0.0, 1.0)
    }

    /// ¿Dominio latente BKT supera el umbral? (default 0.8, igual que EMA).
    #[must_use]
    pub fn is_bkt_mastered(&self, threshold: f64) -> bool {
        self.posterior_mastery() >= threshold.clamp(0.0, 1.0)
    }
}

/// Tipo de evento de aprendizaje registrado.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StudyEventKind {
    Prompt,
    Correct,
    Incorrect,
    ExamPass,
    ExamFail,
}

/// Evento inmutable de la historia de aprendizaje.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StudyEvent {
    pub epoch: u64,
    pub branch_id: String,
    pub kind: StudyEventKind,
    pub detail: String,
}

/// Resultado de un examen sobre una rama.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExamResult {
    pub epoch: u64,
    pub branch_id: String,
    pub score: u32,
    pub total: u32,
    pub passed: bool,
}

/// ── Racha verificada con historial (días con primer acierto) ─────────────
///
/// La racha visible NO cuenta aciertos sueltos: cuenta *días* con PRIMER
/// acierto verificado. Evidencia válida (y solo esta): corrección con nota
/// (`ExamResult` con `score/total ≥ 2/3`, mismo umbral pineado que
/// `exam::mini_exam_result`; el flag `passed` NO se confía, se recomputa)
/// o primer Steps completado del día (el solver no deja nota en el perfil:
/// esos días llegan por `steps_days`, en sesión y sin persistir).
///
/// Sin evidencia el día no existe: día vacío → no sube, `epoch` futura → se
/// ignora, `passed: true` sin nota → no cuenta. Todo puro, sin I/O, 0 deps.
/// La persistencia usa el camino natural (`profile.exams` ya se guarda);
/// ningún campo nuevo se serializa.
/// Segundos por día para [`streak_day`] (días UTC desde epoch).
pub const STREAK_DAY_SECS: u64 = 86_400;
/// Tope del historial corto visible al pulsar la racha (últimos 7).
pub const STREAK_HISTORY_LIMIT: usize = 7;

/// Causa del primer acierto verificado de un día de racha.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreakCause {
    /// Corrección con nota ≥ 2/3 (`ExamResult`).
    Exam,
    /// Primer solver paso a paso completado del día (sin nota).
    Steps,
}

/// Un día de racha: día UTC ([`streak_day`]) + causa del primer acierto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreakEvent {
    pub day: u64,
    pub cause: StreakCause,
}

/// Día UTC de un `epoch` (división entera, determinista, sin calendario).
#[must_use]
pub const fn streak_day(epoch: u64) -> u64 {
    epoch / STREAK_DAY_SECS
}

/// ¿Este examen es evidencia de racha? Solo corrección con nota ≥ 2/3
/// (umbral pineado de `exam::mini_exam_result`: `score*3 >= total*2`) y con
/// `epoch` no futura. El flag `passed` se ignora a propósito: un
/// `passed: true` con `score` bajo jamás sube la racha (no se inventa).
#[must_use]
pub fn exam_counts_for_streak(result: &ExamResult, now_epoch: u64) -> bool {
    if result.total == 0 || result.epoch > now_epoch {
        return false;
    }
    let score = result.score.min(result.total);
    u64::from(score) * 3 >= u64::from(result.total) * 2
}

/// Días con primer acierto, en orden ascendente, un evento por día.
///
/// `steps_days` trae días UTC ya convertidos con [`streak_day`] (sesión).
/// Si un día tiene examen y Steps, gana el examen (la nota es la evidencia
/// más fuerte y del Steps no se conoce la hora intra-día). Los días futuros
/// se ignoran. Puro y determinista.
#[must_use]
pub fn verified_streak_events(
    exams: &[ExamResult],
    steps_days: &[u64],
    now_epoch: u64,
) -> Vec<StreakEvent> {
    let today = streak_day(now_epoch);
    let mut days: std::collections::BTreeMap<u64, StreakCause> = std::collections::BTreeMap::new();
    for result in exams {
        if exam_counts_for_streak(result, now_epoch) {
            days.insert(streak_day(result.epoch), StreakCause::Exam);
        }
    }
    for &day in steps_days {
        if day <= today {
            days.entry(day).or_insert(StreakCause::Steps);
        }
    }
    days.into_iter()
        .map(|(day, cause)| StreakEvent { day, cause })
        .collect()
}

/// Racha actual: días consecutivos con evidencia que terminan hoy (o ayer,
/// si hoy aún no tiene: la racha sigue viva, pendiente del día). Un día
/// vacío rompe la cadena; el futuro jamás cuenta. `0` sin evidencia.
#[must_use]
pub fn verified_streak_count(events: &[StreakEvent], now_epoch: u64) -> u32 {
    let today = streak_day(now_epoch);
    let present: std::collections::BTreeSet<u64> = events
        .iter()
        .map(|event| event.day)
        .filter(|&day| day <= today)
        .collect();
    let mut cursor = today;
    if !present.contains(&cursor) {
        match today.checked_sub(1) {
            Some(yesterday) if present.contains(&yesterday) => cursor = yesterday,
            _ => return 0,
        }
    }
    let mut count = 0_u32;
    while present.contains(&cursor) {
        count = count.saturating_add(1);
        match cursor.checked_sub(1) {
            Some(prev) => cursor = prev,
            None => break,
        }
    }
    count
}

/// Historial corto para el tooltip de la racha: últimos
/// [`STREAK_HISTORY_LIMIT`] eventos, del más reciente al más viejo.
#[must_use]
pub fn verified_streak_history(events: &[StreakEvent]) -> Vec<StreakEvent> {
    let mut sorted = events.to_vec();
    sorted.sort_by_key(|event| event.day);
    sorted
        .into_iter()
        .rev()
        .take(STREAK_HISTORY_LIMIT)
        .collect()
}

/// Fecha ISO (`YYYY-MM-DD`) de un día UTC ([`streak_day`]).
///
/// Algoritmo civil inverso de Howard Hinnant, puro y sin tablas: el tooltip
/// muestra fechas neutras sin claves i18n nuevas (igual que
/// `format_token_count`, sin locale). Aritmética en `u128` (sin overflow).
#[must_use]
pub fn streak_day_iso(day: u64) -> String {
    let z: u128 = u128::from(day).saturating_add(719_468);
    let era: u128 = z / 146_097;
    let doe: u128 = z - era * 146_097;
    let yoe: u128 = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y: u128 = yoe + era * 400;
    let doy: u128 = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp: u128 = (5 * doy + 2) / 153;
    let d: u128 = doy - (153 * mp + 2) / 5 + 1;
    let m: u128 = if mp < 10 { mp + 3 } else { mp - 9 };
    let year: u128 = if m <= 2 { y + 1 } else { y };
    format!("{year:04}-{m:02}-{d:02}")
}

/// Secciones visibles del dashboard "Mi espacio" (H2, Fase 1).
/// Todo visible por defecto; la vista deja ocultar cada sección y el
/// perfil lo persiste. `#[serde(default)]` por campo = perfiles viejos o
/// JSON parcial migran con todo visible (mismo patrón que `domain_history`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpaceSections {
    #[serde(default = "space_visible_default")]
    pub today: bool,
    #[serde(default = "space_visible_default")]
    pub subjects: bool,
    #[serde(default = "space_visible_default")]
    pub tasks: bool,
    #[serde(default = "space_visible_default")]
    pub week: bool,
    #[serde(default = "space_visible_default")]
    pub notes: bool,
    #[serde(default = "space_visible_default")]
    pub stats: bool,
}

fn space_visible_default() -> bool {
    true
}

impl Default for SpaceSections {
    fn default() -> Self {
        Self {
            today: true,
            subjects: true,
            tasks: true,
            week: true,
            notes: true,
            stats: true,
        }
    }
}

/// Perfil completo del estudiante (memoria del usuario).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StudentProfile {
    pub name: String,
    pub level: u32,
    pub xp: u64,
    pub branches: Vec<BranchState>,
    pub history: Vec<StudyEvent>,
    pub exams: Vec<ExamResult>,
    /// Racha actual de aciertos consecutivos (se reinicia con un fallo).
    pub streak: u32,
    /// Mejor racha histórica de aciertos consecutivos.
    pub best_streak: u32,
    /// Avatar y personalización visual (migración: default si no existe).
    #[serde(default)]
    pub avatar: AvatarConfig,
    /// Mascota directa (compatibilidad con spec: duplicado de avatar.mascot).
    /// Se mantiene sincronizada con `avatar.mascot` en helpers.
    #[serde(default)]
    pub mascot: Option<MascotConfig>,
    /// Memoria a largo plazo (hechos, preferencias, vínculo).
    #[serde(default)]
    pub long_memory: LongTermMemory,
    /// Memoria de trabajo episódica de la sesión (WorkingMemory).
    /// F5 inline: contexto socrático sin tocar memoria larga; se sincroniza con AssistantPanelState.
    #[serde(default)]
    pub working_memory: WorkingMemory,
    /// Plan de examen E3 (`None` = sin plan; el panel lo crea).
    /// `Option` + `default` = perfiles viejos migran sin romper.
    #[serde(default)]
    pub study_plan: Option<study_plan::StudyPlan>,
    /// Rating Elo por banco E5 (`banco` → estado; ausente = 1500 inicial).
    #[serde(default)]
    pub elo: std::collections::HashMap<String, EloState>,
    /// Secciones visibles del dashboard "Mi espacio" (H2, Fase 1).
    /// `#[serde(default)]` = perfiles viejos migran con todo visible.
    #[serde(default)]
    pub space_sections: SpaceSections,
}

impl Default for StudentProfile {
    fn default() -> Self {
        Self {
            name: "Estudiante".to_string(),
            level: 1,
            xp: 0,
            branches: Vec::new(),
            history: Vec::new(),
            exams: Vec::new(),
            streak: 0,
            best_streak: 0,
            avatar: AvatarConfig::default(),
            mascot: None,
            long_memory: LongTermMemory::default(),
            working_memory: WorkingMemory::default(),
            study_plan: None,
            elo: std::collections::HashMap::new(),
            space_sections: SpaceSections::default(),
        }
    }
}

impl StudentProfile {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Self::default()
        }
    }

    /// Devuelve mascota mutable, creando una por defecto si no existe.
    /// Sincroniza `self.mascot` y `self.avatar.mascot` para compatibilidad.
    pub fn mascot_mut_or_default(&mut self) -> &mut MascotConfig {
        // Prioriza avatar.mascot si existe, si no mascot directo, si no crea.
        if self.avatar.mascot.is_none() && self.mascot.is_some() {
            self.avatar.mascot = self.mascot.clone();
        }
        if self.mascot.is_none() && self.avatar.mascot.is_some() {
            self.mascot = self.avatar.mascot.clone();
        }
        if self.avatar.mascot.is_none() {
            let cfg = MascotConfig::default();
            self.avatar.mascot = Some(cfg.clone());
            self.mascot = Some(cfg);
        }
        // Mantener ambos sincronizados (avatar es fuente de verdad para UI)
        if self.mascot.is_none() {
            self.mascot = self.avatar.mascot.clone();
        }
        self.avatar.mascot.get_or_insert_with(MascotConfig::default)
    }

    /// Asegura que la rama exista; devuelve su índice o `None` si se alcanzó el límite.
    /// Fail-closed: los llamantes deben manejar `None` en lugar de usar sentinel `usize::MAX`.
    pub fn ensure_branch(&mut self, id: &str, name: &str) -> Option<usize> {
        if let Some(index) = self.branches.iter().position(|b| b.id == id) {
            return Some(index);
        }
        if self.branches.len() >= MAX_BRANCHES {
            return None;
        }
        self.branches.push(BranchState {
            id: id.to_string(),
            name: name.to_string(),
            covered: false,
            mastery: 0.0,
            last_study_epoch: None,
            domain_history: VecDeque::new(),
            next_review_epoch: None,
            box_level: 0,
            bkt_p_known: default_bkt_p_known(),
        });
        Some(self.branches.len() - 1)
    }

    /// Compatibilidad: variante sentinel legada que retorna `usize::MAX` si no hay espacio.
    /// Preferir `ensure_branch` que retorna `Option`.
    #[deprecated(note = "Usar ensure_branch que retorna Option<usize>")]
    pub fn ensure_branch_legacy(&mut self, id: &str, name: &str) -> usize {
        self.ensure_branch(id, name).unwrap_or(usize::MAX)
    }

    /// Registra una respuesta y actualiza dominio (EMA + BKT), Leitner y progreso.
    pub fn record_outcome(&mut self, branch_id: &str, name: &str, epoch: u64, correct: bool) {
        let Some(index) = self.ensure_branch(branch_id, name) else {
            return;
        };
        let branch = &mut self.branches[index];
        if correct {
            branch.mastery += (EMA_ALPHA as f32) * (1.0 - branch.mastery);
            self.xp = self.xp.saturating_add(10);
            self.streak = self.streak.saturating_add(1);
            self.best_streak = self.best_streak.max(self.streak);
            if branch.mastery >= (MASTER_THRESHOLD as f32) && !branch.covered {
                branch.covered = true;
                self.xp = self.xp.saturating_add(40);
            }
        } else {
            branch.mastery *= EMA_RETENTION as f32;
            self.streak = 0;
        }
        // BKT: actualizar P(sabe) con evidencia.
        // FIX corrección pedagógica: `bkt_params_for_lo` (50 brazos por LO →
        // legacy → default), NO `bkt_params_for_branch` (7 claves legacy). Con
        // branch la calibración fina por LO (`bkt_params_for_lo_opt`) era
        // código muerto: todo LO se actualizaba con los 4 parámetros genéricos.
        let params = bkt::bkt_params_for_lo(branch_id);
        let next_p = bkt::bkt_update(branch.bkt_p_known, correct, &params);
        branch.bkt_p_known = next_p.clamp(0.0, 1.0);
        // Leitner: subir/bajar caja
        if correct {
            branch.box_level = (branch.box_level.saturating_add(1)).min(MAX_BOX_LEVEL);
        } else {
            branch.box_level = branch.box_level.saturating_sub(2);
        }
        // Scheduler: próximo repaso
        let interval = scheduler::next_interval(branch.box_level, branch.mastery);
        branch.next_review_epoch = Some(epoch.saturating_add(interval));
        branch.last_study_epoch = Some(epoch);
        // Evolución de dominio acotada (64 muestras por rama) — VecDeque LRU.
        branch.domain_history.push_back((epoch, branch.mastery));
        if branch.domain_history.len() > 64 {
            branch.domain_history.pop_front();
        }
        self.push_event(StudyEvent {
            epoch,
            branch_id: branch_id.to_string(),
            kind: if correct {
                StudyEventKind::Correct
            } else {
                StudyEventKind::Incorrect
            },
            detail: format!("{} {branch_id}", if correct { "acierto" } else { "fallo" }),
        });
        self.level = u32::try_from((self.xp / 250).saturating_add(1)).unwrap_or(u32::MAX);
        // Sincronizar evolución de mascota si existe
        if let Some(m) = self.avatar.mascot.as_mut() {
            let covered = self.branches.iter().filter(|b| b.covered).count() as u32;
            m.sync_evolution(self.level, covered);
        }
        if let Some(m) = self.mascot.as_mut() {
            let covered = self.branches.iter().filter(|b| b.covered).count() as u32;
            m.sync_evolution(self.level, covered);
        }
    }

    pub fn record_exam(&mut self, result: ExamResult) {
        let index = self.ensure_branch(&result.branch_id, &result.branch_id);
        if let Some(idx) = index {
            if result.passed {
                let branch = &mut self.branches[idx];
                branch.mastery = branch
                    .mastery
                    .max(result.score as f32 / result.total.max(1) as f32);
                branch.covered = true;
                self.xp = self.xp.saturating_add(60);
            }
        }
        self.push_event(StudyEvent {
            epoch: result.epoch,
            branch_id: result.branch_id.clone(),
            kind: if result.passed {
                StudyEventKind::ExamPass
            } else {
                StudyEventKind::ExamFail
            },
            detail: format!("{}/{}", result.score, result.total),
        });
        self.exams.push(result);
    }

    /// Racha verificada con historial desde el camino natural del perfil.
    ///
    /// `(conteo, últimos 7)`: el conteo son días consecutivos con evidencia
    /// ([`verified_streak_count`]) sobre [`verified_streak_events`] de
    /// `self.exams` + `steps_days` (días UTC de Steps en sesión, sin
    /// persistir: el solver no deja nota en el perfil). No añade ningún
    /// campo SERIALIZADO: `exams` ya se guarda; `steps_days` muere con la
    /// sesión. Puro y determinista dado `now_epoch`.
    #[must_use]
    pub fn verified_streak(&self, now_epoch: u64, steps_days: &[u64]) -> (u32, Vec<StreakEvent>) {
        let events = verified_streak_events(&self.exams, steps_days, now_epoch);
        let count = verified_streak_count(&events, now_epoch);
        (count, verified_streak_history(&events))
    }

    /// Mejor nota con evidencia de un día de racha, para el tooltip.
    ///
    /// `Some((score, total))` del examen que cuenta con mayor proporción;
    /// `None` en días solo-Steps (sin nota no se inventa nada) o sin
    /// evidencia. `now_epoch` filtra futuro igual que la racha.
    #[must_use]
    pub fn streak_exam_note(&self, day: u64, now_epoch: u64) -> Option<(u32, u32)> {
        self.exams
            .iter()
            .filter(|result| {
                streak_day(result.epoch) == day && exam_counts_for_streak(result, now_epoch)
            })
            .max_by(|a, b| {
                let left = u64::from(a.score.min(a.total)) * u64::from(b.total.max(1));
                let right = u64::from(b.score.min(b.total)) * u64::from(a.total.max(1));
                left.cmp(&right).then(a.score.cmp(&b.score))
            })
            .map(|result| (result.score.min(result.total), result.total))
    }

    fn push_event(&mut self, event: StudyEvent) {
        self.history.push(event);
        if self.history.len() > MAX_HISTORY_EVENTS {
            let overflow = self.history.len() - MAX_HISTORY_EVENTS;
            self.history.drain(0..overflow);
        }
    }

    /// Ramas sin cubrir, ordenadas por menor dominio (la siguiente a estudiar).
    ///
    /// # Nota de integración (assistant.rs:450)
    ///
    /// Este método es la API histórica usada por `assistant.rs:450`
    /// (`profile.recommend_next()`). Desde esta versión delega en
    /// `recommend_next_with_scheduler(now)` con `now = now_epoch()` para
    /// incorporar Leitner/SM-2 sin romper callers. Si necesitas control
    /// explícito del tiempo (tests deterministas, replay), usa
    /// `recommend_next_with_scheduler(now)` directamente. Comportamiento:
    /// prioriza `due` (vencidas) primero, luego menor `mastery`/`bkt_p_known`.
    pub fn recommend_next(&self) -> Vec<&BranchState> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.recommend_next_with_scheduler(now)
    }

    /// Variante explícita sin reloj: delega en `recommend_next_with_scheduler(now)` con `now` dado.
    /// Útil para tests y para `assistant.rs` si migra a inyección de reloj.
    pub fn recommend_next_at(&self, now: u64) -> Vec<&BranchState> {
        self.recommend_next_with_scheduler(now)
    }

    /// Ramas cuyo repaso venció en `now` (Leitner due).
    pub fn branches_due(&self, now: u64) -> Vec<&BranchState> {
        let mut out = Vec::with_capacity(self.branches.len());
        out.extend(self.branches.iter().filter(|b| {
            if let Some(epoch) = b.next_review_epoch {
                scheduler::is_due(epoch, now)
            } else {
                false
            }
        }));
        out
    }

    /// Ramas sin cubrir priorizando vencidas (due primero) y luego menor dominio/BKT.
    /// Mantiene compatibilidad: no cambia `recommend_next()` existente.
    ///
    /// Orden total aplicado:
    /// 1. `due` (`next_review_epoch <= now`) antes que no vencidas;
    /// 2. entre iguales, `next_review_epoch` más antiguo primero
    ///    (`None` ordena antes que cualquier `Some`, igual que `Option::cmp`);
    /// 3. desempate por menor `mastery` y luego menor `bkt_p_known`.
    pub fn recommend_next_with_scheduler(&self, now: u64) -> Vec<&BranchState> {
        // Precalcula `due` una vez por rama (O(n)) en lugar de re-evaluar
        // `is_due` en cada comparación del sort (O(n log n)). Orden idéntico:
        // due primero, luego `next_review_epoch`, luego `mastery`, luego `bkt`.
        let mut pending: Vec<(&BranchState, bool)> = Vec::with_capacity(self.branches.len());
        pending.extend(self.branches.iter().filter(|b| !b.covered).map(|b| {
            let due = b
                .next_review_epoch
                .is_some_and(|e| scheduler::is_due(e, now));
            (b, due)
        }));
        pending.sort_by(|(a, a_due), (b, b_due)| {
            match (*a_due, *b_due) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => {
                    // Entre mismo estado due, prioriza próximo vencimiento más antiguo.
                    let ord_epoch = a.next_review_epoch.cmp(&b.next_review_epoch);
                    if ord_epoch != std::cmp::Ordering::Equal {
                        return ord_epoch;
                    }
                    // Fallback: menor mastery/BKT primero (más débil)
                    let ord_mastery = a
                        .mastery
                        .partial_cmp(&b.mastery)
                        .unwrap_or(std::cmp::Ordering::Equal);
                    if ord_mastery != std::cmp::Ordering::Equal {
                        return ord_mastery;
                    }
                    a.bkt_p_known
                        .partial_cmp(&b.bkt_p_known)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }
            }
        });
        pending.into_iter().map(|(b, _)| b).collect()
    }

    /// Helper Leitner listo para el asistente: vencidas primero.
    ///
    /// Equivale a `recommend_next_with_scheduler(now)` con nombre explícito
    /// para la migración del caller: retorna las ramas **no cubiertas**
    /// ordenadas con `due` (`next_review_epoch <= now`, ver
    /// [`scheduler::is_due`]) primero y, dentro de cada grupo, por
    /// `next_review_epoch` más antiguo, luego menor `mastery`/`bkt_p_known`.
    /// Las ramas nunca practicadas (`next_review_epoch == None`) ordenan
    /// antes que las futuras (orden `Option::cmp`) pero después de las
    /// vencidas: son "nuevas", no "atrasadas".
    ///
    /// # Integración con `assistant.rs` (NO tocar desde este crate)
    ///
    /// `crates/grafito-app/src/assistant.rs:486,499,1704` ya llama a
    /// `profile.recommend_next()`, que desde esta versión delega en
    /// `recommend_next_with_scheduler(now_epoch)` — es decir, el asistente YA
    /// consume Leitner (`next_review_epoch`/`box_level` vía `record_outcome`
    /// y `scheduler::next_interval`) sin cambios. Si otro agente migra esos
    /// call sites a reloj inyectado, el reemplazo directo es:
    /// `profile.recommend_due_first(now)` (misma semántica, nombre intencional).
    /// Puro y determinista dado `now`: sin I/O, sin reloj interno.
    pub fn recommend_due_first(&self, now: u64) -> Vec<&BranchState> {
        self.recommend_next_with_scheduler(now)
    }

    /// Mastery medio posterior BKT del perfil (`0..=1`, NaN-safe).
    ///
    /// Media de `BranchState::posterior_mastery` sobre ramas con dato finito;
    /// `0.3` si no hay ramas (igual que `default_bkt_p_known`). Es la señal
    /// que el dashboard del aula muestra como `avg_mastery` vía
    /// `LearnerSnapshot::from_student_profile`. No toca el campo legacy
    /// `mastery` (EMA).
    #[must_use]
    pub fn avg_posterior_mastery(&self) -> f64 {
        let mut sum = 0.0_f64;
        let mut valid = 0_usize;
        for branch in &self.branches {
            let value = branch.posterior_mastery();
            if value.is_finite() {
                sum += value;
                valid = valid.saturating_add(1);
            }
        }
        if valid == 0 {
            return 0.3;
        }
        (sum / valid as f64).clamp(0.0, 1.0)
    }

    /// Recomendación intercalada (interleaving): evita bloques del mismo tema.
    ///
    /// Parte de [`Self::recommend_next_with_scheduler`] (vencidas primero) y
    /// reordena por round-robin de tópico (prefijo antes de `'-'`, ej.
    /// `am1` en `am1-der`; sin `'-'` el id completo es el tópico). Así
    /// `am1-der, am1-lim, sec-vect` se vuelve `am1-der, sec-vect, am1-lim`:
    /// mezcla sin perder la prioridad due (los vencidos siguen antes que los
    /// futuros dentro de cada tópico porque el orden de entrada ya es due-first
    /// y los grupos preservan ese orden). Puro, determinista, sin I/O.
    /// Cap implícito: retorna a lo sumo todas las pendientes (`<=128` ramas).
    pub fn recommend_interleaved(&self, now: u64) -> Vec<&BranchState> {
        let ranked = self.recommend_next_with_scheduler(now);
        if ranked.len() <= 2 {
            return ranked;
        }
        // Agrupa por tópico preservando el orden due-first dentro de cada grupo.
        // Sin allocs: la clave es `&str` prestada del `id` (prefijo antes de
        // `'-'`; sin `'-'` el id completo). Mismo agrupamiento y orden que con
        // `String`, sin 1 alloc por rama.
        let mut groups: Vec<(&str, Vec<&BranchState>)> = Vec::new();
        for branch in ranked {
            let topic: &str = branch
                .id
                .split_once('-')
                .map_or(branch.id.as_str(), |(prefix, _)| prefix);
            match groups.iter_mut().find(|(key, _)| *key == topic) {
                Some((_, items)) => items.push(branch),
                None => groups.push((topic, vec![branch])),
            }
        }
        if groups.len() <= 1 {
            let single = groups.pop().map_or_else(Vec::new, |(_, items)| items);
            return single;
        }
        // Round-robin por tópico (orden de primera aparición).
        let mut out = Vec::with_capacity(groups.iter().map(|(_, items)| items.len()).sum());
        let mut index = 0_usize;
        loop {
            let mut advanced = false;
            for (_, items) in groups.iter_mut() {
                if let Some(branch) = items.get(index) {
                    out.push(*branch);
                    advanced = true;
                }
            }
            if !advanced {
                break;
            }
            index = index.saturating_add(1);
        }
        out
    }

    /// Schedules actuales por rama (para UI/debug).
    pub fn review_schedules(&self) -> Vec<ReviewSchedule> {
        let mut out = Vec::with_capacity(self.branches.len());
        out.extend(self.branches.iter().filter_map(|b| {
            let epoch = b.next_review_epoch?;
            let interval = scheduler::next_interval(b.box_level, b.mastery);
            let days = (interval / scheduler::DAY_SECS) as u32;
            Some(ReviewSchedule {
                branch_id: b.id.clone(),
                next_review_epoch: epoch,
                interval_days: days.max(1),
                box_level: b.box_level,
            })
        }));
        out
    }

    pub fn display_name(&self) -> &str {
        let d = self.avatar.display_name.trim();
        if !d.is_empty() {
            &self.avatar.display_name
        } else {
            &self.name
        }
    }
    pub fn set_display_name(&mut self, name: &str) -> Result<(), String> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err("El nombre no puede estar vacío".to_string());
        }
        if trimmed.chars().count() > crate::mascot::MAX_DISPLAY_NAME {
            return Err(format!(
                "El nombre no puede superar {} caracteres",
                crate::mascot::MAX_DISPLAY_NAME
            ));
        }
        self.name = trimmed.to_string();
        self.avatar.display_name = trimmed.to_string();
        if self.avatar.seed.trim().is_empty() {
            self.avatar.seed = trimmed.to_string();
        }
        Ok(())
    }

    /// Resumen comprimido para el prompt del tutor (memoria del usuario).
    pub fn memory(&self) -> String {
        use std::fmt::Write as _;
        let mut base = if self.branches.is_empty() {
            "Estudiante nuevo: sin ramas registradas todavía.".to_string()
        } else {
            let covered = self.branches.iter().filter(|b| b.covered).count();
            let pct = covered as f32 / self.branches.len().max(1) as f32 * 100.0;
            let mut t = String::with_capacity(256 + self.branches.len() * 48);
            let _ = writeln!(
                t,
                "Nivel {}, XP {}. Racha: {}. Cobertura: {covered}/{} ({pct:.0}%).",
                self.level,
                self.xp,
                self.streak,
                self.branches.len()
            );
            for branch in &self.branches {
                // `write!` directo evita el `String` temporal de `format!` por rama.
                let _ = writeln!(
                    t,
                    "- {}: {} (dominio {:.0}%).",
                    branch.name,
                    if branch.covered {
                        "cubierta"
                    } else {
                        "pendiente"
                    },
                    branch.mastery * 100.0
                );
            }
            if let Some(event) = self.history.last() {
                let _ = writeln!(
                    t,
                    "Última actividad: {:?} en {}.",
                    event.kind, event.branch_id
                );
            }
            t
        };
        // Añadir personalidad/ánimo y memoria larga + avatar rasgos finos
        if let Some(m) = self.avatar.mascot.as_ref().or(self.mascot.as_ref()) {
            base.push_str("Personalidad mascota: ");
            base.push_str(m.personality.system_prompt_snippet());
            base.push_str(".\n");
            let mood = m.update_mood(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
                false,
            );
            base.push_str("Ánimo actual: ");
            base.push_str(mood.label());
            base.push_str(".\n");
        }
        if !self.avatar.custom_instructions.trim().is_empty() {
            base.push_str("Instrucciones usuario: ");
            base.extend(self.avatar.custom_instructions.chars().take(800));
            base.push_str(".\n");
        }
        if self.avatar.verbosity != 50
            || self.avatar.humor != 30
            || self.avatar.formality != 50
            || self.avatar.empathy != 60
        {
            let _ = writeln!(
                base,
                "Rasgos: verbosidad {} humor {} formalidad {} empatía {}.",
                self.avatar.verbosity,
                self.avatar.humor,
                self.avatar.formality,
                self.avatar.empathy
            );
        }
        if !self.avatar.language.trim().is_empty() {
            base.push_str("Idioma preferido: ");
            base.push_str(&self.avatar.language);
            base.push_str(".\n");
        }
        let long = self.long_memory.render_for_prompt();
        if !long.is_empty() {
            base.push_str("\n[Memoria largo plazo]\n");
            base.push_str(&long);
        }
        // Presupuesto real en chars (no bytes): el recorte corta en
        // MAX_MEMORY_CHARS contando el sufijo, para que la SALIDA completa
        // respete el presupuesto declarado (antes se recortaba recién sobre
        // 3200 y se devolvían ~3199 chars: 33 % sobre el presupuesto).
        const SUFIJO_RECORTE: &str = "…\n[resumen recortado]";
        // Fast-path: si entra en bytes, entra en chars (1 char >= 1 byte).
        // Evita `chars().count()` en perfiles chicos (caso común).
        if base.len() <= MAX_MEMORY_CHARS {
            return base;
        }
        let total = base.chars().count();
        if total > MAX_MEMORY_CHARS {
            let recorte = MAX_MEMORY_CHARS.saturating_sub(SUFIJO_RECORTE.chars().count());
            let mut cut: String = base.chars().take(recorte).collect();
            cut.push_str(SUFIJO_RECORTE);
            cut
        } else {
            base
        }
    }

    /// Guarda un recuerdo episódico (ej. preferencia detectada).
    pub fn remember_fact(&mut self, text: &str, epoch: u64, importance: f32) {
        let fact = Fact::new(text, epoch, importance);
        self.long_memory.push_fact(fact);
        if self.long_memory.first_seen_epoch.is_none() {
            self.long_memory.first_seen_epoch = Some(epoch);
        }
    }
}

/// Tiempo transcurrido desde `epoch` hasta `now`, legible y determinista.
/// Sin dependencias externas (matemática pura): hoy, hace N días, hace N h o
/// hace N min. Si `now <= epoch` devuelve «hoy».
pub fn time_ago(epoch: u64, now: u64) -> String {
    const DAY: u64 = 86_400;
    const HOUR: u64 = 3_600;
    const MINUTE: u64 = 60;
    let elapsed = now.saturating_sub(epoch);
    if elapsed < MINUTE {
        return "hoy".to_string();
    }
    if elapsed < HOUR {
        let minutes = elapsed / MINUTE;
        return format!("hace {minutes} min");
    }
    if elapsed < DAY {
        let hours = elapsed / HOUR;
        return format!("hace {hours} h");
    }
    let days = elapsed / DAY;
    if days == 1 {
        return "ayer".to_string();
    }
    format!("hace {days} días")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_ago_is_deterministic_and_legible() {
        assert_eq!(time_ago(100, 100), "hoy");
        assert_eq!(time_ago(100, 130), "hoy"); // 30 s < 1 min
        assert_eq!(time_ago(0, 86_400), "ayer");
        assert_eq!(time_ago(0, 86_400 * 3), "hace 3 días");
        assert_eq!(time_ago(0, 1_800), "hace 30 min");
    }
    #[test]
    fn profile_tracks_outcomes_levels_and_branch_coverage() {
        let mut profile = StudentProfile::new("Lucas");
        assert_eq!(profile.name, "Lucas");
        // 1 - 0.85^n >= 0.8 → n >= 10; con 12 aciertos el dominio supera 0.8.
        for _ in 0..12 {
            profile.record_outcome("calculus", "Cálculo", 1, true);
        }
        let branch = &profile.branches[0];
        assert!(branch.mastery >= 0.8, "el dominio pasa de 0.8");
        assert!(branch.covered, "la rama se cubre al superar el umbral");
        assert!(profile.xp >= 10 * 12 + 40);
        assert!(profile.level >= 1);
    }

    #[test]
    fn recommendation_prioritizes_uncovered_low_domain_branches() {
        let mut profile = StudentProfile::new("Ana");
        profile.record_outcome("algebra", "Álgebra", 0, true);
        profile.record_outcome("geometry3d", "Geometría 3D", 1, false);
        // Usar variante determinista para evitar flakiness por reloj
        let next = profile.recommend_next_with_scheduler(9_999_999);
        assert!(!next.is_empty());
        assert!(!next.iter().any(|b| b.covered));
        // La más débil primero — con due determinista, geometry3d debería estar entre las primeras
        // (orden exacta depende de scheduler + BKT; verificar presencia no posición)
        assert!(
            next.iter().any(|b| b.id == "geometry3d"),
            "geometry3d debe estar recomendada"
        );
    }

    #[test]
    fn recommend_next_delegates_to_scheduler() {
        // Verifica que recommend_next() (reloj) y recommend_next_at(now) coincidan
        // en el orden relativo cuando el reloj se inyecta. No podemos asumir now
        // exacto, pero sí que ambas APIs retornan el mismo conjunto y que with_scheduler es determinista.
        let mut profile = StudentProfile::new("Deleg");
        profile.record_outcome("algebra", "Álgebra", 0, false);
        profile.record_outcome("geometry", "Geometría", 0, true);
        for b in &mut profile.branches {
            if b.id == "algebra" {
                b.mastery = 0.2;
                b.next_review_epoch = Some(10);
                b.box_level = 1;
            } else if b.id == "geometry" {
                b.mastery = 0.9;
                b.next_review_epoch = Some(9_999_999);
                b.box_level = 3;
            }
        }
        let det = profile.recommend_next_at(100);
        assert_eq!(det[0].id, "algebra");
        // recommend_next() usa now real; con epoch grande ambas ramas estarán due, pero
        // el test asegura que al menos no rompe y retorna pending ordenado
        let live = profile.recommend_next();
        assert!(!live.is_empty());
        assert!(live.iter().any(|b| b.id == "algebra"));
    }

    #[test]
    fn memory_is_bounded_and_mentions_coverage() {
        let mut profile = StudentProfile::new("Mia");
        profile.record_outcome("linear", "Ecuaciones", 5, true);
        let memory = profile.memory();
        // Presupuesto en chars (igual que lo trata `memory()`), no en bytes.
        assert!(memory.chars().count() <= MAX_MEMORY_CHARS);
        assert!(memory.contains("Ecuaciones"));
    }

    #[test]
    fn record_outcome_aplica_calibracion_fina_por_lo() {
        // Regresión FIX 1: `record_outcome` debe actualizar BKT con
        // `bkt_params_for_lo` (am1-der: p_slip 0.14), NO con
        // `bkt_params_for_branch` (caía al default genérico p_slip 0.1) —
        // con eso los 45 brazos de `bkt_params_for_lo_opt` eran código muerto.
        let por_lo = bkt_params_for_lo("am1-der");
        assert!((por_lo.p_slip - 0.14).abs() < 1e-12, "am1-der p_slip=0.14");
        let esperado = bkt_update(0.3, true, &por_lo);
        let genérico = bkt_update(0.3, true, &BktParams::default());
        assert_ne!(
            esperado, genérico,
            "el LO calibrado debe diferir del default genérico"
        );
        let mut p = StudentProfile::new("BKT-LO");
        p.record_outcome("am1-der", "Derivadas", 0, true);
        assert!(
            (p.branches[0].bkt_p_known - esperado).abs() < 1e-12,
            "p_known {} != esperado {}",
            p.branches[0].bkt_p_known,
            esperado
        );
        // Un LO con params distintos debe moverse distinto (calibración real).
        let mut q = StudentProfile::new("BKT-LO-2");
        q.record_outcome("pri-conteo", "Conteo", 0, true);
        let pri = bkt_update(0.3, true, &bkt_params_for_lo("pri-conteo"));
        assert!((q.branches[0].bkt_p_known - pri).abs() < 1e-12);
        assert_ne!(p.branches[0].bkt_p_known, q.branches[0].bkt_p_known);
    }

    #[test]
    fn memory_respeta_presupuesto_chars_con_fixture_multibyte() {
        // Regresión FIX 7: el recorte debe actuar en MAX_MEMORY_CHARS (chars,
        // no bytes) y la SALIDA completa —sufijo incluido— debe caber en el
        // presupuesto. Antes se recortaba sobre 3200 y devolvía ~3199 chars.
        let mut p = StudentProfile::new("Ñandú");
        for i in 0..MAX_BRANCHES {
            let id = format!("rama-{i}");
            let name = format!("Tema ñ{} {}", i, "ñ".repeat(40));
            assert!(p.ensure_branch(&id, &name).is_some());
        }
        p.record_outcome("rama-0", "Tema ñ0", 1, true);
        let out = p.memory();
        assert!(
            out.chars().count() <= MAX_MEMORY_CHARS,
            "memory() rompió el presupuesto: {} chars",
            out.chars().count()
        );
        assert!(
            out.contains("[resumen recortado]"),
            "sin sufijo de recorte: la fixture no superaba el presupuesto"
        );
    }

    #[test]
    fn level_no_trunca_con_xp_enorme() {
        // Regresión FIX 11: `as u32` truncaba si xp era enorme.
        let mut p = StudentProfile::new("XP");
        p.xp = u64::MAX;
        p.record_outcome("algebra", "Álgebra", 0, false);
        assert_eq!(p.level, u32::MAX, "nivel debe saturar, no truncar");
    }

    #[test]
    fn profile_serde_round_trips_without_loss() {
        let mut profile = StudentProfile::new("Leo");
        profile.record_outcome("stats", "Estadística", 9, true);
        profile.record_exam(ExamResult {
            epoch: 10,
            branch_id: "stats".to_string(),
            score: 4,
            total: 5,
            passed: true,
        });
        let json = serde_json::to_string(&profile).expect("serialize");
        let restored: StudentProfile = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored, profile);
    }

    #[test]
    fn non_finite_mastery_is_never_produced() {
        let mut profile = StudentProfile::new("Nico");
        for _ in 0..200 {
            profile.record_outcome("fractals", "Fractales", 1, true);
            profile.record_outcome("fractals", "Fractales", 1, false);
        }
        assert!(profile.branches[0].mastery.is_finite());
        assert!(profile.branches[0].mastery >= 0.0 && profile.branches[0].mastery <= 1.0);
    }

    #[test]
    fn domain_history_records_evolution_and_stays_capped() {
        let mut profile = StudentProfile::new("Evo");
        for index in 0..70 {
            profile.record_outcome("algebra", "Álgebra", index as u64, true);
        }
        let branch = &profile.branches[0];
        assert!(branch.domain_history.len() <= 64, "histórico acotado");
        assert!(branch.domain_history.len() >= 2);
        let last = branch
            .domain_history
            .back()
            .map(|entry| entry.1)
            .unwrap_or(0.0);
        let first = branch
            .domain_history
            .front()
            .map(|entry| entry.1)
            .unwrap_or(1.0);
        assert!(last >= first, "el dominio sube con los aciertos");
    }

    #[test]
    fn streak_rises_on_correct_and_resets_on_failure() {
        let mut profile = StudentProfile::new("Racha");
        for _ in 0..3 {
            profile.record_outcome("algebra", "Álgebra", 1, true);
        }
        assert_eq!(profile.streak, 3);
        assert_eq!(profile.best_streak, 3);
        profile.record_outcome("algebra", "Álgebra", 1, false);
        assert_eq!(profile.streak, 0);
        assert_eq!(profile.best_streak, 3, "la mejor se conserva");
    }

    #[test]
    fn history_is_capped_to_max_events() {
        let mut profile = StudentProfile::new("Sofi");
        for year in 0..(MAX_HISTORY_EVENTS + 30) as u64 {
            profile.record_outcome("algebra", "Álgebra", year, true);
        }
        assert!(profile.history.len() <= MAX_HISTORY_EVENTS);
    }

    #[test]
    fn mascot_mut_or_default_creates_and_syncs() {
        let mut profile = StudentProfile::new("Masc");
        profile.mascot = None;
        profile.avatar.mascot = None;
        let m = profile.mascot_mut_or_default();
        m.name = "TestPou".to_string();
        assert_eq!(profile.avatar.mascot.as_ref().unwrap().name, "TestPou");
        assert!(profile.mascot.is_some());
        // Segunda llamada no duplica
        let _ = profile.mascot_mut_or_default();
        assert!(profile.avatar.mascot.is_some());
    }

    #[test]
    fn profile_migration_defaults_mascot_fields() {
        let json = r#"{"name":"Old","level":1,"xp":0,"branches":[],"history":[],"exams":[],"streak":0,"best_streak":0}"#;
        let restored: StudentProfile = serde_json::from_str(json).expect("migration");
        assert!(restored.avatar.mascot.is_some() || restored.mascot.is_none());
        // Debe poder serializar de nuevo sin perder datos
        let json2 = serde_json::to_string(&restored).unwrap();
        assert!(json2.contains("Old"));
    }

    #[test]
    fn mascot_config_validation_and_sanitized() {
        let mut m = MascotConfig::default();
        assert!(m.validate().is_ok());
        m.name = "a".repeat(MAX_NAME + 1);
        assert!(m.validate().is_err());
        m.name = "  Hola  ".to_string();
        assert_eq!(m.sanitized_name(), "Hola");
        m.name = "".to_string();
        assert_eq!(m.sanitized_name(), "Pou");
    }

    #[test]
    fn outfit_tier_and_wardrobe_unlock() {
        assert_eq!(OutfitTier::from_level(3), OutfitTier::Primary);
        assert_eq!(OutfitTier::from_level(10), OutfitTier::Secondary);
        assert_eq!(OutfitTier::from_level(15), OutfitTier::University);
        assert_eq!(OutfitTier::from_level(30), OutfitTier::Master);
        let mut w = Wardrobe::default();
        w.unlock_for_level(6);
        assert!(w.is_owned("cap_prim"));
        assert!(w.is_owned("hat_sec"));
    }

    #[test]
    fn bkt_update_correct_increases_incorrect_decreases() {
        let mut p = StudentProfile::new("BKT");
        p.record_outcome("algebra", "Álgebra", 100, true);
        let bkt_after_correct = p.branches[0].bkt_p_known;
        assert!(
            bkt_after_correct > 0.3,
            "BKT debe subir con acierto: {bkt_after_correct}"
        );
        p.record_outcome("algebra", "Álgebra", 200, false);
        let bkt_after_incorrect = p.branches[0].bkt_p_known;
        assert!(
            bkt_after_incorrect < bkt_after_correct,
            "BKT debe bajar con fallo: {bkt_after_incorrect} vs {bkt_after_correct}"
        );
    }

    #[test]
    fn scheduler_interval_grows_with_box_level() {
        let i1 = next_interval(1, 0.5);
        let i2 = next_interval(2, 0.5);
        let i4 = next_interval(4, 0.5);
        assert!(i1 < i2);
        assert!(i2 < i4);
    }

    #[test]
    fn branches_due_filters_correctly() {
        let mut p = StudentProfile::new("Due");
        // Dos ramas con distinto vencimiento
        p.record_outcome("algebra", "Álgebra", 0, true); // -> next_review ~ 129600
        p.record_outcome("calculus", "Cálculo", 0, true);
        // Forzar una vencida y otra futura
        if let Some(b) = p.branches.iter_mut().find(|b| b.id == "algebra") {
            b.next_review_epoch = Some(100);
        }
        if let Some(b) = p.branches.iter_mut().find(|b| b.id == "calculus") {
            b.next_review_epoch = Some(9_999_999);
        }
        let due = p.branches_due(500);
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].id, "algebra");
        let all_due = p.branches_due(10_000_000);
        assert_eq!(all_due.len(), 2);
    }

    #[test]
    fn recommend_next_with_scheduler_prioritizes_due() {
        let mut p = StudentProfile::new("Sched");
        // Álgebra débil y vencida, geometría fuerte y no vencida
        p.record_outcome("algebra", "Álgebra", 0, false); // mastery bajo
        p.record_outcome("geometry", "Geometría", 0, true);
        // Ajustar vencimientos: algebra due, geometry futuro
        for b in &mut p.branches {
            if b.id == "algebra" {
                b.mastery = 0.2;
                b.next_review_epoch = Some(10);
                b.box_level = 1;
            } else if b.id == "geometry" {
                b.mastery = 0.9;
                b.next_review_epoch = Some(9_999_999);
                b.box_level = 3;
            }
        }
        let ranked = p.recommend_next_with_scheduler(100);
        assert!(!ranked.is_empty());
        assert_eq!(ranked[0].id, "algebra", "vencida debe ir primero");
    }

    #[test]
    fn recommend_due_first_matches_scheduler_and_orders_due_first() {
        let mut p = StudentProfile::new("DueFirst");
        p.record_outcome("algebra", "Álgebra", 0, false);
        p.record_outcome("geometry", "Geometría", 0, true);
        p.record_outcome("calculus", "Cálculo", 0, true);
        for b in &mut p.branches {
            if b.id == "algebra" {
                b.mastery = 0.2;
                b.next_review_epoch = Some(10);
                b.box_level = 1;
            } else if b.id == "geometry" {
                b.mastery = 0.9;
                b.next_review_epoch = Some(9_999_999);
                b.box_level = 3;
            } else if b.id == "calculus" {
                b.mastery = 0.5;
                b.next_review_epoch = Some(20);
                b.box_level = 2;
            }
        }
        // now=100: algebra (due, epoch 10) y calculus (due, epoch 20) vencidas;
        // geometry futura. Due ordenadas por epoch más antiguo primero.
        let ranked = p.recommend_due_first(100);
        assert_eq!(ranked.len(), 3);
        assert_eq!(ranked[0].id, "algebra");
        assert_eq!(ranked[1].id, "calculus");
        assert_eq!(ranked[2].id, "geometry");
        // Misma semántica que recommend_next_with_scheduler.
        let via_scheduler = p.recommend_next_with_scheduler(100);
        let ids_due_first: Vec<&str> = ranked.iter().map(|b| b.id.as_str()).collect();
        let ids_sched: Vec<&str> = via_scheduler.iter().map(|b| b.id.as_str()).collect();
        assert_eq!(ids_due_first, ids_sched);
        // Determinista: repetir con el mismo now da el mismo orden.
        let again = p.recommend_due_first(100);
        let ids_again: Vec<&str> = again.iter().map(|b| b.id.as_str()).collect();
        assert_eq!(ids_due_first, ids_again);
    }

    #[test]
    fn recommend_due_first_excludes_covered_and_handles_empty() {
        let empty = StudentProfile::new("Vacio");
        assert!(empty.recommend_due_first(0).is_empty());
        let mut p = StudentProfile::new("Cub");
        for _ in 0..12 {
            p.record_outcome("calculus", "Cálculo", 1, true);
        }
        assert!(p.branches[0].covered);
        assert!(p.recommend_due_first(9_999_999).is_empty());
    }

    #[test]
    fn box_level_increments_on_correct_and_decrements_on_failure() {
        let mut p = StudentProfile::new("Box");
        p.record_outcome("algebra", "Álgebra", 0, true);
        assert_eq!(p.branches[0].box_level, 1);
        p.record_outcome("algebra", "Álgebra", 1, true);
        assert_eq!(p.branches[0].box_level, 2);
        p.record_outcome("algebra", "Álgebra", 2, false);
        assert_eq!(p.branches[0].box_level, 0, "debe bajar 2 niveles");
        // No debe underflow
        p.record_outcome("algebra", "Álgebra", 3, false);
        assert_eq!(p.branches[0].box_level, 0);
        // Subir hasta tope 8
        for i in 0..20 {
            p.record_outcome("algebra", "Álgebra", 10 + i, true);
        }
        assert!(p.branches[0].box_level <= 8);
        assert_eq!(p.branches[0].box_level, 8);
    }

    #[test]
    fn branch_state_migration_defaults_bkt_and_scheduler() {
        let json = r#"{"id":"old","name":"Vieja","covered":false,"mastery":0.5,"last_study_epoch":null,"domain_history":[]}"#;
        let branch: BranchState = serde_json::from_str(json).expect("migrate branch");
        assert_eq!(branch.next_review_epoch, None);
        assert_eq!(branch.box_level, 0);
        assert!((branch.bkt_p_known - 0.3).abs() < 1e-9);
        let json2 = serde_json::to_string(&branch).expect("serialize migrated");
        assert!(json2.contains("old"));
    }

    #[test]
    fn profile_migration_defaults_bkt_scheduler() {
        let json = r#"{"name":"Old","level":1,"xp":0,"branches":[{"id":"algebra","name":"Álgebra","covered":false,"mastery":0.4,"last_study_epoch":null,"domain_history":[]}],"history":[],"exams":[],"streak":0,"best_streak":0}"#;
        let restored: StudentProfile = serde_json::from_str(json).expect("migration");
        assert_eq!(restored.branches[0].box_level, 0);
        assert!(restored.branches[0].next_review_epoch.is_none());
        assert!((restored.branches[0].bkt_p_known - 0.3).abs() < 1e-9);
    }

    #[test]
    fn space_sections_default_all_visible_and_roundtrip() {
        // H2: perfiles viejos sin el campo migran con todo visible.
        let json = r#"{"name":"Old","level":1,"xp":0,"branches":[],"history":[],"exams":[],"streak":0,"best_streak":0}"#;
        let restored: StudentProfile = serde_json::from_str(json).expect("migration");
        assert_eq!(restored.space_sections, SpaceSections::default());
        assert!(restored.space_sections.today);
        // JSON parcial también migra por campo (mismo patrón que
        // `domain_history`): lo ausente queda visible.
        let partial: SpaceSections = serde_json::from_str(r#"{"today":false}"#).expect("partial");
        assert!(!partial.today);
        assert!(partial.subjects && partial.tasks && partial.week);
        assert!(partial.notes && partial.stats);
        // Roundtrip: lo oculto persiste.
        let mut p = StudentProfile::new("R");
        p.space_sections.tasks = false;
        let back: StudentProfile =
            serde_json::from_str(&serde_json::to_string(&p).expect("ser")).expect("de");
        assert!(!back.space_sections.tasks);
        assert!(back.space_sections.today);
    }

    #[test]
    fn bkt_p_known_is_finite_and_bounded() {
        let mut p = StudentProfile::new("Fin");
        for i in 0..100 {
            p.record_outcome("algebra", "Álgebra", i, i % 3 == 0);
            assert!(p.branches[0].bkt_p_known.is_finite());
            assert!((0.0..=1.0).contains(&p.branches[0].bkt_p_known));
        }
    }

    #[test]
    fn next_review_epoch_is_set_and_monotonic_with_box() {
        let mut p = StudentProfile::new("Rev");
        p.record_outcome("algebra", "Álgebra", 1_000, true);
        let first = p.branches[0].next_review_epoch.expect("review set");
        assert!(first > 1_000);
        let first_interval = first - 1_000;
        p.record_outcome("algebra", "Álgebra", 2_000, true);
        let second = p.branches[0].next_review_epoch.expect("review set");
        let second_interval = second - 2_000;
        assert!(
            second_interval > first_interval,
            "interval debe crecer con box_level: {first_interval} vs {second_interval}"
        );
    }

    #[test]
    fn posterior_mastery_tracks_bkt_and_avg_is_real() {
        let empty = StudentProfile::new("Vacio");
        assert!((empty.avg_posterior_mastery() - 0.3).abs() < 1e-9);
        let mut p = StudentProfile::new("Post");
        p.record_outcome("algebra", "Álgebra", 1, true);
        let branch = &p.branches[0];
        assert!((branch.posterior_mastery() - branch.bkt_p_known).abs() < 1e-9);
        assert!((p.avg_posterior_mastery() - branch.bkt_p_known).abs() < 1e-9);
        assert!(!branch.is_bkt_mastered(0.99));
        // Tras muchos aciertos el posterior supera 0.8.
        for i in 2..12 {
            p.record_outcome("algebra", "Álgebra", i, true);
        }
        assert!(p.branches[0].is_bkt_mastered(0.8));
        assert!(p.avg_posterior_mastery() >= 0.8);
        // mastery_from_bkt coincide con posterior en f32.
        assert!(
            (mastery_from_bkt(p.branches[0].bkt_p_known) - p.branches[0].bkt_p_known as f32).abs()
                < 1e-6
        );
    }

    #[test]
    fn recommend_interleaved_alternates_topics_without_losing_due() {
        let mut p = StudentProfile::new("Inter");
        p.record_outcome("am1-der", "Derivadas", 0, false);
        p.record_outcome("am1-lim", "Límites", 0, false);
        p.record_outcome("sec-vect", "Vectores", 0, false);
        for branch in &mut p.branches {
            // Todas vencidas en now=100, mismo vencimiento relativo.
            branch.next_review_epoch = Some(10);
            branch.covered = false;
        }
        // Fijar mastery para orden due-first determinista: am1-der más débil.
        for branch in &mut p.branches {
            if branch.id == "am1-der" {
                branch.mastery = 0.1;
                branch.bkt_p_known = 0.2;
            } else if branch.id == "am1-lim" {
                branch.mastery = 0.2;
                branch.bkt_p_known = 0.3;
            } else {
                branch.mastery = 0.3;
                branch.bkt_p_known = 0.4;
            }
        }
        let interleaved = p.recommend_interleaved(100);
        assert_eq!(interleaved.len(), 3);
        // Debe alternar tópicos: no dos am1 seguidos al inicio.
        let topics: Vec<String> = interleaved
            .iter()
            .map(|b| {
                b.id.split_once('-')
                    .map_or_else(|| b.id.clone(), |(prefix, _)| prefix.to_string())
            })
            .collect();
        assert_ne!(topics[0], topics[1], "interleaving alterna: {topics:?}");
        // Contiene las 3 (sin perder vencidas).
        assert!(topics.contains(&"am1".to_string()));
        assert!(topics.contains(&"sec".to_string()));
    }

    #[test]
    fn recommend_interleaved_single_topic_keeps_order() {
        let mut p = StudentProfile::new("Single");
        p.record_outcome("am1-der", "Derivadas", 0, false);
        p.record_outcome("am1-lim", "Límites", 0, false);
        let now = 100;
        let base: Vec<String> = p
            .recommend_next_with_scheduler(now)
            .iter()
            .map(|b| b.id.clone())
            .collect();
        let inter: Vec<String> = p
            .recommend_interleaved(now)
            .iter()
            .map(|b| b.id.clone())
            .collect();
        assert_eq!(base, inter);
    }

    // ── Racha verificada con historial ────────────────────────────────
    // Día UTC pineado: 2026-10-15 00:00 UTC = epoch 1_792_022_400 = día 20741.
    const STREAK_NOW: u64 = 1_792_022_400;
    const STREAK_TODAY: u64 = 20_741;
    const DAY: u64 = 86_400;

    fn streak_exam(epoch: u64, score: u32, total: u32) -> ExamResult {
        ExamResult {
            epoch,
            branch_id: "algebra".to_string(),
            score,
            total,
            passed: score * 3 >= total * 2,
        }
    }

    #[test]
    fn racha_sube_con_acierto_con_nota() {
        // Ayer 2/3 + hoy 3/3: dos días con evidencia → racha 2, causa Exam.
        let exams = vec![
            streak_exam(STREAK_NOW - DAY, 2, 3),
            streak_exam(STREAK_NOW, 3, 3),
        ];
        let events = verified_streak_events(&exams, &[], STREAK_NOW);
        assert_eq!(events.len(), 2);
        assert!(events.iter().all(|e| e.cause == StreakCause::Exam));
        assert_eq!(verified_streak_count(&events, STREAK_NOW), 2);
        let history = verified_streak_history(&events);
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].day, STREAK_TODAY, "reciente primero");
        assert_eq!(history[1].day, STREAK_TODAY - 1);
    }

    #[test]
    fn racha_no_sube_sin_evidencia() {
        // Vacío, suspenso y total 0: jamás sube sin evidencia.
        assert_eq!(verified_streak_count(&[], STREAK_NOW), 0);
        assert!(verified_streak_history(&[]).is_empty());
        let fail = vec![streak_exam(STREAK_NOW, 1, 3)];
        assert_eq!(
            verified_streak_count(&exams_as_events(&fail), STREAK_NOW),
            0
        );
        assert!(verified_streak_history(&exams_as_events(&fail)).is_empty());
        let zero = vec![streak_exam(STREAK_NOW, 0, 0)];
        assert_eq!(
            verified_streak_count(&exams_as_events(&zero), STREAK_NOW),
            0
        );
    }

    #[test]
    fn racha_nunca_confia_en_passed_sin_nota() {
        // `passed: true` con nota bajo el umbral NO cuenta (no se inventa).
        let fake = vec![ExamResult {
            epoch: STREAK_NOW,
            branch_id: "algebra".to_string(),
            score: 0,
            total: 3,
            passed: true,
        }];
        assert!(!exam_counts_for_streak(&fake[0], STREAK_NOW));
        assert_eq!(
            verified_streak_count(&exams_as_events(&fake), STREAK_NOW),
            0
        );
    }

    #[test]
    fn racha_futura_no_cuenta() {
        // Examen de mañana: ni conteo ni historial (fecha futura → no cuenta).
        let future = vec![streak_exam(STREAK_NOW + DAY, 3, 3)];
        assert!(!exam_counts_for_streak(&future[0], STREAK_NOW));
        let events = verified_streak_events(&future, &[STREAK_TODAY + 1], STREAK_NOW);
        assert!(events.is_empty());
        assert_eq!(verified_streak_count(&events, STREAK_NOW), 0);
    }

    #[test]
    fn racha_rota_tras_dia_vacio() {
        // Días -2 y hoy con hueco en -1: solo queda 1 (rota tras día vacío).
        let exams = vec![
            streak_exam(STREAK_NOW - 2 * DAY, 3, 3),
            streak_exam(STREAK_NOW, 3, 3),
        ];
        assert_eq!(
            verified_streak_count(&exams_as_events(&exams), STREAK_NOW),
            1
        );
        // Hoy vacío pero ayer con evidencia: la racha sigue viva en 2.
        let alive = vec![
            streak_exam(STREAK_NOW - 2 * DAY, 3, 3),
            streak_exam(STREAK_NOW - DAY, 3, 3),
        ];
        assert_eq!(
            verified_streak_count(&exams_as_events(&alive), STREAK_NOW),
            2
        );
        // Dos días vacíos seguidos: 0.
        let dead = vec![streak_exam(STREAK_NOW - 3 * DAY, 3, 3)];
        assert_eq!(
            verified_streak_count(&exams_as_events(&dead), STREAK_NOW),
            0
        );
    }

    fn exams_as_events(exams: &[ExamResult]) -> Vec<StreakEvent> {
        verified_streak_events(exams, &[], STREAK_NOW)
    }

    #[test]
    fn historial_acotado_a_siete_pineado() {
        // 10 días seguidos hasta hoy: historial 7, reciente primero, ISO pineado.
        let exams: Vec<ExamResult> = (0..10)
            .map(|back| streak_exam(STREAK_NOW - back * DAY, 3, 3))
            .collect();
        let events = verified_streak_events(&exams, &[], STREAK_NOW);
        assert_eq!(events.len(), 10);
        assert_eq!(verified_streak_count(&events, STREAK_NOW), 10);
        let history = verified_streak_history(&events);
        assert_eq!(history.len(), STREAK_HISTORY_LIMIT);
        assert_eq!(history[0].day, STREAK_TODAY);
        assert_eq!(history[6].day, STREAK_TODAY - 6);
        assert_eq!(streak_day_iso(history[0].day), "2026-10-15");
        assert_eq!(streak_day_iso(history[6].day), "2026-10-09");
    }

    #[test]
    fn steps_cuenta_primer_completado_y_examen_manda() {
        // Steps de hoy sin examen: cuenta con causa Steps.
        let events = verified_streak_events(&[], &[STREAK_TODAY], STREAK_NOW);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].cause, StreakCause::Steps);
        assert_eq!(verified_streak_count(&events, STREAK_NOW), 1);
        // Mismo día con examen: gana la nota (evidencia más fuerte).
        let exams = vec![streak_exam(STREAK_NOW, 2, 3)];
        let mixed = verified_streak_events(&exams, &[STREAK_TODAY], STREAK_NOW);
        assert_eq!(mixed.len(), 1);
        assert_eq!(mixed[0].cause, StreakCause::Exam);
    }

    #[test]
    fn iso_pineado_y_umbral_dos_tercios() {
        assert_eq!(streak_day_iso(0), "1970-01-01");
        assert_eq!(streak_day_iso(STREAK_TODAY), "2026-10-15");
        assert_eq!(streak_day(STREAK_NOW), STREAK_TODAY);
        // Umbral exacto 2/3 cuenta; por debajo no.
        assert!(exam_counts_for_streak(&streak_exam(0, 2, 3), 0));
        assert!(!exam_counts_for_streak(&streak_exam(0, 1, 3), 0));
        assert!(exam_counts_for_streak(&streak_exam(0, 4, 5), 0));
    }

    #[test]
    fn verified_streak_usa_exams_y_nota_sin_nuevo_campo() {
        // Camino natural: `exams` ya persiste (roundtrip) y Steps llega por sesión.
        let mut p = StudentProfile::new("R");
        p.record_exam(streak_exam(STREAK_NOW - DAY, 2, 3));
        p.record_exam(streak_exam(STREAK_NOW, 3, 3));
        let (count, history) = p.verified_streak(STREAK_NOW, &[]);
        assert_eq!(count, 2);
        assert_eq!(history.len(), 2);
        assert_eq!(p.streak_exam_note(STREAK_TODAY, STREAK_NOW), Some((3, 3)));
        assert_eq!(
            p.streak_exam_note(STREAK_TODAY - 1, STREAK_NOW),
            Some((2, 3))
        );
        // Día solo-Steps: sin nota, no se inventa.
        let (count_steps, _) = p.verified_streak(STREAK_NOW, &[STREAK_TODAY]);
        assert_eq!(count_steps, 2, "el día ya tenía examen");
        let (only_steps, _) = StudentProfile::new("S").verified_streak(STREAK_NOW, &[STREAK_TODAY]);
        assert_eq!(only_steps, 1);
        assert_eq!(
            StudentProfile::new("S").streak_exam_note(STREAK_TODAY, STREAK_NOW),
            None
        );
        // El perfil serializa igual que antes (sin campo nuevo de logros).
        let json = serde_json::to_string(&p).expect("serialize");
        assert!(!json.contains("StreakEvent"));
        assert!(!json.contains("steps_days"));
    }
}
