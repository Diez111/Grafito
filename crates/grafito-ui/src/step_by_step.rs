//! Visor de desarrollo por pasos — tarjeta del transcript del asistente.
//!
//! Superficie de UI para el motor `CasStepper` (`grafito-geometry::cas_steps`):
//! el comando `StepByStep[op]` ya devuelve la traza como texto y esta tarjeta
//! la muestra con revelado progresivo (`revealed` pasos visibles de `total`,
//! acotado a `MAX_CAS_STEPS` 32).
//!
//! - Piel pura: `fn render(&Estado) -> Frame`. Cero I/O, cero spawn en el
//!   dibujado. Los botones emiten [`AssistantUiAction`] y la app muta el
//!   contador fuera del draw.
//! - Fuente de los pasos: bloques ```grafito-steps del transcript (mismo
//!   patrón que `AssistantExerciseCard::to_code_block` / `from_code_block`).
//! - Tokens de [`crate::tokens`] como única fuente de verdad; i18n vía
//!   [`crate::i18n::t`]; animación de aparición gobernada por
//!   [`MotionConfig`] (`reduced_motion` salta el fade).

use crate::assistant::{draw_math_with_locale_and_zoom, AssistantUiAction};
use crate::i18n::{t, Locale};
use crate::projector::MotionConfig;
use crate::tokens::{
    hit_target_size, HIT_TARGET_MIN, RADIUS_MD, SPACE_SM, SPACE_XS, TYPE_SM, TYPE_XS,
};
use grafito_geometry::cas_steps::{
    etiqueta_str, motivo_str, CasStep, MAX_CAS_STEPS, MAX_STEP_BYTES,
};

/// Lenguaje del fence que el transcript reconoce como visor de pasos.
pub const STEPS_FENCE_LANGUAGE: &str = "grafito-steps";

/// Apertura exacta del fence (primera línea).
const STEPS_FENCE_LANGUAGE_FENCE: &str = "```grafito-steps";

/// Ancho bajo el cual los botones de revelado se apilan en vertical a
/// ancho completo: con panel angosto (~320) el `horizontal` fijo recortaba
/// «Ver todo».
pub const STEPS_NARROW_STACK_WIDTH: f32 = 260.0;

/// Estado visible del botón «Leer»/«Detener» de la tarjeta (voz espeak-ng).
///
/// Puro y `Copy`: la app lo resuelve por frame (`disponible` por caché de
/// presencia del binario, `hablando` si ESTE turno suena) y el draw solo
/// pinta + emite [`crate::assistant::AssistantUiAction::ToggleSpeakSteps`]
/// al pulsar. Jamás habla solo: sin clic no hay acción.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepsTtsUi {
    /// Hay `espeak-ng` ejecutable (la app cachea el `PATH` por sesión y lo
    /// re-chequea en vivo al pulsar).
    pub available: bool,
    /// ESTE turno se está leyendo ahora (el botón muestra «Detener»).
    pub speaking: bool,
}

impl StepsTtsUi {
    /// Clave i18n de la etiqueta («Leer» idle, «Detener» sonando). Pura.
    pub const fn label_key(self) -> &'static str {
        if self.speaking {
            "assistant.steps.stop"
        } else {
            "assistant.steps.read"
        }
    }

    /// Clave i18n del hover honesto: sin binario trae el comando
    /// instalable; sonando avisa que detiene; si no, el contrato
    /// (solo al pulsar, 2ª pulsación detiene). Pura.
    pub const fn hint_key(self) -> &'static str {
        if !self.available && !self.speaking {
            "assistant.steps.read_unavailable_hint"
        } else if self.speaking {
            "assistant.steps.stop_hint"
        } else {
            "assistant.steps.read_hint"
        }
    }

    /// ¿El botón responde? Apagado solo sin binario Y sin habla en curso:
    /// si suena (aunque el binario haya desaparecido en el medio) se puede
    /// detener. Pura y pineada en tests.
    pub const fn enabled(self) -> bool {
        self.available || self.speaking
    }
}

/// Niveles discretos de zoom de la tarjeta (porcentaje pineado).
///
/// Cada nivel re-rasteriza cada fórmula a su escala por el camino tex
/// (`draw_math_with_locale_and_zoom`); jamás es un upscale borroso: si el
/// raster a escala N falla o se sale del rango del motor, el nivel se
/// declara no-disponible (botón apagado + motivo) y el dibujado cae al
/// subset honesto. Default [`STEPS_ZOOM_DEFAULT`] (sin re-escalado).
pub const STEPS_ZOOM_LEVELS: [u16; 4] = [100, 125, 150, 200];

/// Zoom por defecto de la tarjeta (100%: dibujado intacto, sin re-escalado).
pub const STEPS_ZOOM_DEFAULT: u16 = 100;

/// Factor multiplicador del nivel (`100 → 1.0`, `200 → 2.0`). Puro.
///
/// Niveles ajenos a [`STEPS_ZOOM_LEVELS`] se normalizan a 1.0: el zoom
/// vive en sesión y un valor rancio jamás deforma la tarjeta.
pub fn steps_zoom_factor(pct: u16) -> f32 {
    if STEPS_ZOOM_LEVELS.contains(&pct) {
        f32::from(pct) / 100.0
    } else {
        1.0
    }
}

/// Índice del nivel en [`STEPS_ZOOM_LEVELS`] (desconocido → 0 = 100%).
/// Puro.
pub fn steps_zoom_index(pct: u16) -> usize {
    STEPS_ZOOM_LEVELS
        .iter()
        .position(|nivel| *nivel == pct)
        .unwrap_or(0)
}

/// Siguiente nivel discreto en la dirección dada (`+1` acerca, `−1`
/// aleja), acotado a los extremos pineados. Puro e idempotente en topes.
pub fn steps_zoom_step(pct: u16, dir: i8) -> u16 {
    let pos = steps_zoom_index(pct);
    if dir > 0 {
        STEPS_ZOOM_LEVELS[pos.saturating_add(1).min(STEPS_ZOOM_LEVELS.len() - 1)]
    } else if dir < 0 {
        STEPS_ZOOM_LEVELS[pos.saturating_sub(1)]
    } else {
        STEPS_ZOOM_LEVELS[pos]
    }
}

/// ¿El nivel re-rasteriza limpio a este DPR cuantizado (`dpr_q = ppp*4`)?
///
/// Puro y sin raster: el motor tex acepta `font_px` en
/// `TEX_MIN_FONT_PX..=TEX_MAX_FONT_PX` (8..=96); el zoom pide
/// `TEX_DEFAULT_FONT_PX * factor * dpr`. Fuera de rango el raster
/// recortaría por clamp silencioso o reventaría el presupuesto: el nivel
/// se declara no-disponible y el llamador lo apaga con motivo, jamás
/// upscale borroso.
pub fn steps_zoom_available(pct: u16, dpr_q: u32) -> bool {
    #[allow(clippy::cast_precision_loss)]
    let dpr = (dpr_q as f32) / 4.0;
    let pedido = grafito_tex::TEX_DEFAULT_FONT_PX * steps_zoom_factor(pct) * dpr;
    (grafito_tex::TEX_MIN_FONT_PX..=grafito_tex::TEX_MAX_FONT_PX).contains(&pedido)
}

/// Retardo entre la aparición de un paso y el siguiente (revelado
/// escalonado suave).
pub const STEPS_STAGGER_MS: f32 = 60.0;

/// Desplazamiento máximo del fade en el reloj normalizado 0–1: ningún
/// paso arranca más tarde que esto, así todo paso conserva una fracción
/// de reloj para fundirse (sin pops).
const STEPS_STAGGER_MAX_OFFSET: f32 = 0.8;

/// Un paso pedagógico serializable (espejo de `CasStep` sin acoplar el
/// dibujado al tipo del motor: `rule` guarda el `Display` estable de
/// `RewriteRule`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepByStepItem {
    /// Nombre estable de la regla (`PowerRule`, `Simplify`, …).
    pub rule: String,
    /// Expresión antes de aplicar la regla (≤ `MAX_STEP_BYTES`).
    pub before: String,
    /// Expresión después de aplicar la regla (≤ `MAX_STEP_BYTES`).
    pub after: String,
    /// Descripción rioplatense lista para mostrar (≤ `MAX_STEP_BYTES`).
    pub description: String,
}

impl From<&CasStep> for StepByStepItem {
    fn from(step: &CasStep) -> Self {
        Self {
            rule: step.rule.to_string(),
            before: truncate_str(&step.before, MAX_STEP_BYTES),
            after: truncate_str(&step.after, MAX_STEP_BYTES),
            description: truncate_str(&step.description, MAX_STEP_BYTES),
        }
    }
}

/// Estado del visor: pasos + cuántos están revelados.
///
/// `revealed` siempre ≤ `steps.len()` ≤ `MAX_CAS_STEPS`. El panel
/// (`AssistantPanelState::steps_revealed`) conserva el contador por turno;
/// esta struct es el valor efímero que se dibuja cada frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepByStepCardState {
    /// Etiqueta de la operación (`Derivative[x^2, x]`). Puede estar vacía.
    pub title: String,
    /// Pasos acotados a `MAX_CAS_STEPS`.
    pub steps: Vec<StepByStepItem>,
    /// Pasos visibles (revelado progresivo).
    pub revealed: usize,
}

impl StepByStepCardState {
    /// Construye desde la salida del motor. Con pasos no vacíos arranca con
    /// el primero revelado; sin pasos queda vacía (`revealed` 0).
    pub fn from_cas_steps(title: impl Into<String>, steps: &[CasStep]) -> Self {
        let items: Vec<StepByStepItem> = steps
            .iter()
            .take(MAX_CAS_STEPS)
            .map(StepByStepItem::from)
            .collect();
        let revealed = if items.is_empty() { 0 } else { 1 };
        Self {
            title: title.into(),
            steps: items,
            revealed,
        }
    }

    /// Pasos visibles ahora (nunca más que los disponibles).
    pub fn revealed_steps(&self) -> &[StepByStepItem] {
        let end = self.revealed.min(self.steps.len());
        &self.steps[..end]
    }

    /// Avanza un paso, acotado al total. Idempotente en el tope.
    pub fn reveal_next(&mut self) {
        self.revealed = next_revealed(self.revealed, self.steps.len());
    }

    /// Revela todo, acotado a `MAX_CAS_STEPS` vía `steps.len()`.
    pub fn reveal_all(&mut self) {
        self.revealed = self.steps.len();
    }

    /// Contador `revelados/total` (numérico, sin i18n: los dígitos no se
    /// traducen).
    pub fn counter_text(&self) -> String {
        format!(
            "{}/{}",
            self.revealed.min(self.steps.len()),
            self.steps.len()
        )
    }

    /// Fracción revelada 0–1 para la barra de progreso (vacía → 0).
    /// Pura.
    pub fn progress(&self) -> f32 {
        if self.steps.is_empty() {
            return 0.0;
        }
        #[allow(clippy::cast_precision_loss)]
        let total = self.steps.len() as f32;
        #[allow(clippy::cast_precision_loss)]
        let shown = self.revealed.min(self.steps.len()) as f32;
        (shown / total).clamp(0.0, 1.0)
    }

    /// Texto plano para portapapeles: título (si hay) + una línea por
    /// paso (`nº · regla — desc / antes → después`, sin el tramo `— desc`
    /// cuando está vacía), numerados desde `start_no` para que el turno
    /// numere corrido entre fences. Puro.
    pub fn to_copy_text(&self, start_no: usize) -> String {
        let mut lines = Vec::with_capacity(self.steps.len() + 1);
        if !self.title.trim().is_empty() {
            lines.push(self.title.trim().to_string());
        }
        for (pos, item) in self.steps.iter().enumerate() {
            let mut line = format!("{} · {}", start_no.saturating_add(pos), item.rule.trim());
            if !item.description.trim().is_empty() {
                line.push_str(" — ");
                line.push_str(item.description.trim());
            }
            line.push_str(" / ");
            line.push_str(item.before.trim());
            line.push_str(" → ");
            line.push_str(item.after.trim());
            lines.push(line);
        }
        lines.join("\n")
    }

    /// Serializa a bloque ```grafito-steps (compatible con el renderer de
    /// bloques de código del transcript).
    pub fn to_code_block(&self) -> String {
        let mut out = String::from("```grafito-steps\n");
        if !self.title.trim().is_empty() {
            out.push_str("# ");
            out.push_str(&sanitize_line(&self.title));
            out.push('\n');
        }
        for item in self.steps.iter().take(MAX_CAS_STEPS) {
            out.push_str("- rule: ");
            out.push_str(&sanitize_line(&item.rule));
            out.push('\n');
            out.push_str("  desc: ");
            out.push_str(&sanitize_line(&item.description));
            out.push('\n');
            out.push_str("  before: ");
            out.push_str(&sanitize_line(&item.before));
            out.push('\n');
            out.push_str("  after: ");
            out.push_str(&sanitize_line(&item.after));
            out.push('\n');
        }
        out.push_str("```");
        out
    }

    /// Parsea un bloque ```grafito-steps. Acepta el fence completo o el
    /// cuerpo pelado (el parser del transcript entrega `text` sin las
    /// líneas ```). Trunca a `MAX_CAS_STEPS` y suelta el paso trailing
    /// incompleto (sin `rule`/`before`/`after`). Nunca falla con pánico:
    /// el fence malformado da `None`.
    pub fn from_code_block(text: &str) -> Option<Self> {
        let body = strip_fence(text);
        let mut title = String::new();
        let mut steps = Vec::new();
        let mut current: Option<StepByStepItem> = None;
        let mut invalid = false;
        for raw_line in body.lines() {
            let line = raw_line.trim();
            if line.is_empty() {
                continue;
            }
            if let Some(rule) = line.strip_prefix("- rule:") {
                if steps.len() >= MAX_CAS_STEPS {
                    current = None;
                    break;
                }
                if let Some(pending) = current.take() {
                    push_complete(&mut steps, pending);
                }
                current = Some(StepByStepItem {
                    rule: truncate_str(rule.trim(), MAX_STEP_BYTES),
                    before: String::new(),
                    after: String::new(),
                    description: String::new(),
                });
                continue;
            }
            if title.is_empty() {
                if let Some(head) = line.strip_prefix('#') {
                    title = truncate_str(head.trim(), MAX_STEP_BYTES);
                    continue;
                }
            }
            let Some(slot) = current.as_mut() else {
                // Línea fuera de paso: fence ajeno, se aborta a vacía.
                invalid = true;
                break;
            };
            if let Some(value) = line.strip_prefix("desc:") {
                slot.description = truncate_str(value.trim(), MAX_STEP_BYTES);
            } else if let Some(value) = line.strip_prefix("before:") {
                slot.before = truncate_str(value.trim(), MAX_STEP_BYTES);
            } else if let Some(value) = line.strip_prefix("after:") {
                slot.after = truncate_str(value.trim(), MAX_STEP_BYTES);
            } else {
                invalid = true;
                break;
            }
        }
        if invalid {
            return None;
        }
        if let Some(pending) = current.take() {
            push_complete(&mut steps, pending);
        }
        if title.is_empty() && steps.is_empty() {
            return None;
        }
        let revealed = if steps.is_empty() { 0 } else { 1 };
        Some(Self {
            title,
            steps,
            revealed,
        })
    }
}

/// Siguiente valor de revelado, acotado a `total` (puro y testeable; lo usan
/// el modelo y el mapa del panel para no duplicar la cota).
pub fn next_revealed(current: usize, total: usize) -> usize {
    current.saturating_add(1).min(total)
}

/// Separa la etiqueta de método de la expresión: `"0 (lineal exacta)"` →
/// `("0", Some("lineal exacta"))`; sin un `(...)` final con contenido tipo
/// palabra queda `(trimmed, None)`. La etiqueta se dibuja como caption
/// aparte: dentro de la fuente LaTeX un espacio entre letras se traga y
/// salía `0(linealeexacta)`. Puro, sin pánico (sólo `find`/`split_at` en
/// índices de borde ASCII).
fn split_method_tag(source: &str) -> (&str, Option<&str>) {
    let trimmed = source.trim();
    let Some(open) = trimmed.find(" (") else {
        return (trimmed, None);
    };
    let (expr, rest) = trimmed.split_at(open);
    if !rest.ends_with(')') {
        return (trimmed, None);
    }
    let tag = rest[2..rest.len() - 1].trim();
    let es_tag = !tag.is_empty()
        && tag
            .chars()
            .all(|c| c.is_alphanumeric() || c == ' ' || c == '-' || c == '/');
    if es_tag {
        (expr.trim(), Some(tag))
    } else {
        // Parens interiores que no son tag de método: la expresión queda
        // COMPLETA (jamás se parte una fórmula por parens ajenos).
        (trimmed, None)
    }
}

/// Cortes ```grafito-steps de `content` (texto con fences incluidos, para
/// parsear de a uno). Comparte el recorrido con [`count_steps_in_content`]
/// y [`copy_text_for_content`].
fn steps_fence_texts(content: &str) -> Vec<&str> {
    let mut fences = Vec::new();
    let mut rest = content;
    while let Some(start) = rest.find("```grafito-steps") {
        let fence = &rest[start..];
        let body_end = fence
            .find("\n```")
            .map(|pos| pos + "\n```".len())
            .unwrap_or(fence.len());
        let fence_text = &fence[..body_end.min(fence.len())];
        fences.push(fence_text);
        let advance = (start + body_end).min(rest.len());
        if advance == 0 {
            break;
        }
        rest = &rest[advance..];
        if rest.is_empty() {
            break;
        }
    }
    fences
}

/// Cuenta los pasos de todos los fences ```grafito-steps en `content`
/// (0 si no hay ninguno o son inválidos). Lo usa la app para acotar el
/// avance sin re-parsear el transcript a mano.
pub fn count_steps_in_content(content: &str) -> usize {
    let mut total: usize = 0;
    for fence_text in steps_fence_texts(content) {
        if let Some(card) = StepByStepCardState::from_code_block(fence_text) {
            total = total.saturating_add(card.steps.len());
        }
    }
    total.min(MAX_CAS_STEPS)
}

/// Texto plano de todos los fences ```grafito-steps de `content` para el
/// portapapeles (numeración corrida entre fences, bloques separados con
/// línea en blanco). Vacío si no hay pasos válidos. Puro; la app lo copia
/// fuera del draw vía `AssistantUiAction::CopySteps`.
pub fn copy_text_for_content(content: &str) -> String {
    let mut blocks = Vec::new();
    let mut next_no = 1usize;
    for fence_text in steps_fence_texts(content) {
        let Some(card) = StepByStepCardState::from_code_block(fence_text) else {
            continue;
        };
        if card.steps.is_empty() {
            continue;
        }
        blocks.push(card.to_copy_text(next_no));
        next_no = next_no.saturating_add(card.steps.len());
    }
    blocks.join("\n\n")
}

/// Desplazamiento normalizado (0–1) del fade del paso `pos` dentro del lote
/// que sigue fundiéndose.
///
/// El primer paso del lote arranca en 0; cada siguiente suma el retardo
/// nominal `STEPS_STAGGER_MS` sobre `anim_time_s` (0.18 con el tema), con
/// compresión uniforme sólo si el lote no entra (card gigante): el tope
/// `STEPS_STAGGER_MAX_OFFSET` deja a todo paso una fracción de reloj para
/// fundirse (sin pops). Puro y testeable; el draw lo combina con un
/// `animate_bool` propio por paso (sin replays: lo ya fundido queda en 1).
pub fn stagger_offset(pos: usize, first_running: usize, running: usize, anim_time_s: f32) -> f32 {
    if running <= 1 || pos <= first_running {
        return 0.0;
    }
    let anim_time_s = if anim_time_s.is_finite() && anim_time_s > 0.0 {
        anim_time_s
    } else {
        0.18
    };
    let nominal = STEPS_STAGGER_MS / 1000.0 / anim_time_s;
    #[allow(clippy::cast_precision_loss)]
    let span = running.saturating_sub(1).max(1) as f32;
    let step = nominal.min(STEPS_STAGGER_MAX_OFFSET / span);
    #[allow(clippy::cast_precision_loss)]
    let distance = pos.saturating_sub(first_running) as f32;
    (distance * step).min(STEPS_STAGGER_MAX_OFFSET)
}

/// Trunca a `max_bytes` respetando borde UTF-8 (igual que el motor).
fn truncate_str(value: &str, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value.to_string();
    }
    let mut end = max_bytes.min(value.len());
    while end > 0 && !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].to_string()
}

/// Una línea del fence no puede tener saltos: se colapsan a espacio para
/// que el bloque haga roundtrip.
fn sanitize_line(value: &str) -> String {
    let colapsado = value.split_whitespace().collect::<Vec<_>>().join(" ");
    truncate_str(colapsado.trim(), MAX_STEP_BYTES)
}

/// Quita el fence ```grafito-steps si está (primera línea exacta y cierre
/// final con ```) y devuelve el cuerpo. Sin fences, devuelve el texto tal
/// cual: el parser del transcript ya pela las cercas del bloque `Code`.
fn strip_fence(text: &str) -> String {
    let mut lines: Vec<&str> = text.lines().collect();
    if lines
        .first()
        .is_some_and(|first| first.trim() == STEPS_FENCE_LANGUAGE_FENCE)
    {
        lines.remove(0);
    }
    if lines
        .last()
        .is_some_and(|last| last.trim_start().starts_with("```"))
    {
        lines.pop();
    }
    lines.join("\n")
}

/// Guarda el paso sólo si trae `rule`, `before` y `after` (la descripción
/// puede estar vacía).
fn push_complete(steps: &mut Vec<StepByStepItem>, item: StepByStepItem) {
    if !item.rule.is_empty() && !item.before.is_empty() && !item.after.is_empty() {
        steps.push(item);
    }
}

// ── Dibujado ──

/// Dibuja la tarjeta de pasos en el transcript.
///
/// `card.revealed` ya viene resuelto por el llamador (mapa del panel);
/// `zoom_pct` es el nivel de sesión pineado ([`STEPS_ZOOM_LEVELS`],
/// default [`STEPS_ZOOM_DEFAULT`]) y re-rasteriza cada fórmula a su escala
/// (instantáneo: sin stagger ni fade nuevo, nada que `reduced_motion`
/// deba saltar); los botones emiten la acción y la app avanza fuera del
/// draw. Cada paso nuevo aparece con fade escalonado ([`STEPS_STAGGER_MS`] entre pasos, ver
/// [`stagger_offset`]); con `motion.reduced_motion` todo queda en 1.0 (sin
/// transición). En angosto (`available_width` < [`STEPS_NARROW_STACK_WIDTH`])
/// los botones se apilan en vertical a ancho completo en vez del
/// `horizontal` fijo.
///
/// `tts` ([`StepsTtsUi`]) gobierna el botón «Leer»/«Detener» (voz
/// espeak-ng, solo al pulsar; 2ª pulsación detiene): el draw solo pinta el
/// estado que la app resolvió y emite `ToggleSpeakSteps` al clic. Jamás
/// habla solo, jamás spawnea (piel pura).
#[allow(clippy::too_many_arguments)]
pub fn draw_step_by_step_card(
    ui: &mut egui::Ui,
    card: &StepByStepCardState,
    turn_index: usize,
    card_index: usize,
    zoom_pct: u16,
    motion: MotionConfig,
    locale: Locale,
    tts: StepsTtsUi,
) -> Option<AssistantUiAction> {
    let theme = crate::theme::current_theme(ui.ctx());
    let mut action = None;
    egui::Frame::none()
        .fill(theme.panel_bg)
        .stroke(theme.hairline_stroke())
        .rounding(RADIUS_MD)
        .inner_margin(egui::Margin::same(SPACE_SM))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            // En angosto la cabecera no da para Copiar/Leer/Animar: «Leer»
            // y «Ver animación» bajan a filas propias a ancho completo
            // (mismo patrón que Revelar/Ver todo); en ancho viven en la
            // cabecera junto a Copiar.
            let narrow = ui.available_width() < STEPS_NARROW_STACK_WIDTH;
            let animate_label = t("assistant.steps.animate", locale);
            // Cabecera: título + copiar (+ ver animación en ancho) +
            // contador `i/total`.
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(if card.title.trim().is_empty() {
                        t("assistant.steps.title", locale)
                    } else {
                        card.title.trim()
                    })
                    .color(theme.text_primary)
                    .size(TYPE_SM)
                    .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let counter = ui.label(
                        egui::RichText::new(card.counter_text())
                            .color(theme.accent_strong)
                            .size(TYPE_XS)
                            .strong(),
                    );
                    crate::toolbar::tag_live_region(
                        &counter,
                        format!(
                            "{} {}",
                            t("assistant.steps.title", locale),
                            card.counter_text()
                        ),
                    );
                    // Copiar emite la acción; la app hace `copy_text` fuera
                    // del draw (draw puro, como Revelar/Ver todo).
                    if ui
                        .small_button(t("assistant.turn.copy", locale))
                        .on_hover_text(t("assistant.turn.copy_code_hint", locale))
                        .clicked()
                    {
                        action = Some(AssistantUiAction::CopySteps { turn: turn_index });
                    }
                    // «Leer»/«Detener» (voz espeak-ng, solo al pulsar): la
                    // etiqueta y el hover vienen de `tts` (claves
                    // `assistant.steps.read*/stop*` en 6 idiomas); apagado
                    // sin binario con el comando instalable en el hover;
                    // sonando muestra «Detener». El clic emite el toggle y
                    // la app habla/detiene fuera del draw (piel pura).
                    if !narrow && !card.steps.is_empty() {
                        let respuesta_tts = ui
                            .add_enabled(
                                tts.enabled(),
                                egui::Button::new(t(tts.label_key(), locale)).small(),
                            )
                            .on_hover_text(t(tts.hint_key(), locale));
                        let tts_habilitado = tts.enabled();
                        let tts_etiqueta = t(tts.label_key(), locale).to_owned();
                        respuesta_tts.widget_info(|| {
                            egui::WidgetInfo::labeled(
                                egui::WidgetType::Button,
                                tts_habilitado,
                                tts_etiqueta.clone(),
                            )
                        });
                        if respuesta_tts.clicked() {
                            action = Some(AssistantUiAction::ToggleSpeakSteps { turn: turn_index });
                        }
                    }
                    // «Ver animación» emite el arranque; la app arma la
                    // timeline desde la card cacheada y arranca directo el
                    // motor (el clic es la aprobación). Oculto sin pasos:
                    // nada que animar.
                    if !narrow
                        && !card.steps.is_empty()
                        && ui
                            .small_button(animate_label)
                            .on_hover_text(t("assistant.steps.animate_hint", locale))
                            .clicked()
                    {
                        action = Some(AssistantUiAction::AnimateSteps { turn: turn_index });
                    }
                });
            });
            ui.add_space(SPACE_XS);
            // Barra fina de progreso `i/N` (el texto ya estaba; la barra lo
            // vuelve escaneable). Sin porcentaje visible: el contador manda.
            ui.add(
                egui::ProgressBar::new(card.progress())
                    .animate(!motion.reduced_motion)
                    .desired_height(SPACE_XS),
            );
            ui.add_space(SPACE_XS);
            // Zoom de tarjeta por niveles discretos pineados (default
            // 100): cada nivel re-rasteriza a su escala, jamás upscale
            // borroso. Instantáneo por construcción (sin stagger ni fade
            // nuevo: nada que `reduced_motion` deba saltar). Botones con
            // símbolos/número, sin i18n nueva: el nivel actual resetea a
            // 100 al pulsarlo. Nivel no-disponible a este DPR → `+`
            // apagado con el motivo del raster (clave existente
            // `assistant.math.tex_bad_dims`, 6 idiomas intactos).
            {
                let (_, dpr_q) = crate::assistant::dpr_cuantizado(ui.ctx());
                let mas = steps_zoom_step(zoom_pct, 1);
                let menos = steps_zoom_step(zoom_pct, -1);
                let en_tope_mas = mas == zoom_pct;
                let mas_disponible = !en_tope_mas && steps_zoom_available(mas, dpr_q);
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(menos != zoom_pct, egui::Button::new("−").small())
                        .clicked()
                    {
                        action = Some(AssistantUiAction::StepsZoomOut { turn: turn_index });
                    }
                    if ui
                        .add_enabled(
                            zoom_pct != STEPS_ZOOM_DEFAULT,
                            egui::Button::new(format!("{zoom_pct}%")).small(),
                        )
                        .clicked()
                    {
                        action = Some(AssistantUiAction::StepsZoomReset { turn: turn_index });
                    }
                    let respuesta_mas =
                        ui.add_enabled(mas_disponible, egui::Button::new("+").small());
                    let respuesta_mas = if !en_tope_mas && !mas_disponible {
                        respuesta_mas.on_hover_text(t("assistant.math.tex_bad_dims", locale))
                    } else {
                        respuesta_mas
                    };
                    if respuesta_mas.clicked() {
                        action = Some(AssistantUiAction::StepsZoomIn { turn: turn_index });
                    }
                });
                ui.add_space(SPACE_XS);
            }
            let visible = card.revealed_steps();
            // Reloj propio por paso (id estable, sin `revealed`): lo ya
            // fundido queda en 1.0 sin replays; el lote nuevo se escalona
            // con `stagger_offset` sobre el mismo reloj del tema.
            let anim_time_s = ui.style().animation_time;
            let raws: Vec<f32> = (0..visible.len())
                .map(|pos| {
                    ui.ctx().animate_bool(
                        ui.make_persistent_id(("steps_card_reveal", turn_index, card_index, pos)),
                        true,
                    )
                })
                .collect();
            let first_running = raws
                .iter()
                .position(|raw| *raw < 0.999)
                .unwrap_or(visible.len());
            let running = raws.iter().filter(|raw| **raw < 0.999).count();
            for (pos, item) in visible.iter().enumerate() {
                let step_no = pos + 1;
                // Fundido: ya fundido → 1.0 directo; en curso → fade con su
                // desplazamiento; movimiento reducido → 1.0 (sin transición).
                let raw = raws.get(pos).copied().unwrap_or(1.0);
                let alpha = if raw >= 0.999 {
                    1.0
                } else {
                    let offset = stagger_offset(pos, first_running, running, anim_time_s);
                    motion.morph_t(((raw - offset) / (1.0 - offset)).clamp(0.0, 1.0))
                };
                let previous_opacity = ui.opacity();
                let staged = alpha < 0.999;
                if staged {
                    ui.set_opacity(previous_opacity * alpha.max(0.12));
                }
                // Cabecera del paso: número + etiqueta pedagógica de la
                // regla («Paso 1 · despeje lineal»); el token técnico
                // (`SolveLinearRule`) queda honesto cuando no hay etiqueta
                // conocida. El contador `i/total` ya vive en la cabecera de
                // la tarjeta, acá no se repite.
                let step_header = format!(
                    "{} · {}",
                    t("assistant.steps.step_n", locale).replace("{n}", &step_no.to_string()),
                    etiqueta_str(item.rule.trim()),
                );
                egui::CollapsingHeader::new(step_header)
                    .default_open(true)
                    .id_salt(ui.make_persistent_id((
                        "steps_card_step",
                        turn_index,
                        card_index,
                        pos,
                    )))
                    .show(ui, |ui| {
                        if !item.description.trim().is_empty() {
                            ui.label(
                                egui::RichText::new(item.description.trim())
                                    .color(theme.text_secondary)
                                    .size(TYPE_SM),
                            );
                            ui.add_space(SPACE_XS);
                        }
                        ui.label(
                            egui::RichText::new(motivo_str(&item.rule))
                                .color(theme.text_tertiary)
                                .size(TYPE_SM),
                        );
                        let _ = draw_math_with_locale_and_zoom(ui, &item.before, locale, zoom_pct);
                        ui.label(
                            egui::RichText::new("→")
                                .color(theme.text_tertiary)
                                .size(TYPE_SM),
                        );
                        // El método de resolución viaja pegado al resultado
                        // (`"0 (lineal exacta)"`): se separa — la fuente LaTeX
                        // se comería el espacio y maquetaba `0(linealeexacta)`.
                        let (after_expr, method_tag) = split_method_tag(&item.after);
                        let _ = draw_math_with_locale_and_zoom(ui, after_expr, locale, zoom_pct);
                        if let Some(tag) = method_tag {
                            ui.label(
                                egui::RichText::new(format!(
                                    "{}: {tag}",
                                    t("assistant.steps.method", locale)
                                ))
                                .color(theme.text_tertiary)
                                .size(TYPE_XS),
                            );
                        }
                    });
                if staged {
                    ui.set_opacity(previous_opacity);
                }
                ui.add_space(SPACE_XS);
            }
            // Fila de revelado: avanza de a un paso o muestra todo. En
            // angosto se apila en vertical a ancho completo (el `horizontal`
            // fijo recortaba «Ver todo» a 320 px).
            if card.revealed < card.steps.len() {
                let reveal_label = t("assistant.steps.reveal", locale);
                let all_label = t("assistant.steps.reveal_all", locale);
                if ui.available_width() < STEPS_NARROW_STACK_WIDTH {
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = SPACE_XS;
                        if steps_reveal_button(ui, reveal_label, ui.available_width()).clicked() {
                            action = Some(AssistantUiAction::RevealStep { turn: turn_index });
                        }
                        if steps_reveal_button(ui, all_label, ui.available_width()).clicked() {
                            action = Some(AssistantUiAction::RevealAllSteps { turn: turn_index });
                        }
                    });
                } else {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = SPACE_SM;
                        let half = (ui.available_width() - SPACE_SM) / 2.0;
                        if steps_reveal_button(ui, reveal_label, half).clicked() {
                            action = Some(AssistantUiAction::RevealStep { turn: turn_index });
                        }
                        if steps_reveal_button(ui, all_label, half).clicked() {
                            action = Some(AssistantUiAction::RevealAllSteps { turn: turn_index });
                        }
                    });
                }
            }
            // Filas propias a ancho completo en angosto (mismo patrón que
            // el revelado apilado); en ancho ya viven en la cabecera. Puro
            // dibujado: la app habla/anima fuera del draw.
            if narrow && !card.steps.is_empty() {
                ui.add_space(SPACE_XS);
                // «Leer»/«Detener» apilado: apagado sin binario (hover con
                // el comando instalable), detenible si suena. Solo emite
                // al pulsar: jamás auto-play.
                let tts_habilitado = tts.enabled();
                let tts_etiqueta = t(tts.label_key(), locale);
                let tts_pista = t(tts.hint_key(), locale);
                let respuesta_tts = ui
                    .add_enabled_ui(tts_habilitado, |ui| {
                        steps_reveal_button(ui, tts_etiqueta, ui.available_width())
                            .on_hover_text(tts_pista)
                    })
                    .inner;
                respuesta_tts.widget_info(|| {
                    egui::WidgetInfo::labeled(
                        egui::WidgetType::Button,
                        tts_habilitado,
                        tts_etiqueta.to_owned(),
                    )
                });
                if respuesta_tts.clicked() {
                    action = Some(AssistantUiAction::ToggleSpeakSteps { turn: turn_index });
                }
                if steps_reveal_button(ui, animate_label, ui.available_width()).clicked() {
                    action = Some(AssistantUiAction::AnimateSteps { turn: turn_index });
                }
            }
        });
    action
}

/// Botón de revelado a ancho dado, con altura táctil y etiqueta accesible.
/// Puro dibujado: devuelve la respuesta para que el llamador emita la
/// acción fuera del draw (mismo patrón en fila horizontal y apilada).
fn steps_reveal_button(ui: &mut egui::Ui, label: &str, width: f32) -> egui::Response {
    let response = ui.add_sized(
        egui::vec2(width, hit_target_size(HIT_TARGET_MIN)),
        egui::Button::new(egui::RichText::new(label).size(TYPE_SM)),
    );
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label.to_owned())
    });
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cas_step(rule: &str, before: &str, after: &str) -> CasStep {
        grafito_geometry::cas_steps::CasStep {
            index: 0,
            rule: rule_for(rule),
            before: before.to_string(),
            after: after.to_string(),
            description: format!("aplica {rule}"),
        }
    }

    fn rule_for(name: &str) -> grafito_geometry::cas_steps::RewriteRule {
        use grafito_geometry::cas_steps::RewriteRule as R;
        match name {
            "Simplify" => R::Simplify,
            "PowerRule" => R::PowerRule,
            _ => R::Generic,
        }
    }

    #[test]
    fn modelo_arranca_con_uno_revelado_y_avanza_acotado() {
        let steps = vec![
            cas_step("PowerRule", "x^2", "2*x"),
            cas_step("Simplify", "2*x", "2*x"),
        ];
        let mut card = StepByStepCardState::from_cas_steps("Derivative[x^2, x]", &steps);
        assert_eq!(card.revealed, 1);
        assert_eq!(card.counter_text(), "1/2");
        assert_eq!(card.revealed_steps().len(), 1);
        card.reveal_next();
        assert_eq!(card.counter_text(), "2/2");
        card.reveal_next();
        assert_eq!(card.revealed, 2, "el avance es idempotente en el tope");
        card.reveal_all();
        assert_eq!(card.revealed, 2);
    }

    #[test]
    fn modelo_vacio_queda_en_cero() {
        let card = StepByStepCardState::from_cas_steps("Integral[1, x]", &[]);
        assert_eq!(card.revealed, 0);
        assert_eq!(card.counter_text(), "0/0");
        assert!(card.revealed_steps().is_empty());
    }

    #[test]
    fn fence_hace_roundtrip() {
        let steps = vec![
            cas_step("PowerRule", "x^2", "2*x"),
            cas_step("Simplify", "2*x", "2*x"),
        ];
        let card = StepByStepCardState::from_cas_steps("Derivative[x^2, x]", &steps);
        let fence = card.to_code_block();
        assert!(fence.starts_with("```grafito-steps\n"));
        let Some(parsed) = StepByStepCardState::from_code_block(&fence) else {
            panic!("parsea su propio fence");
        };
        assert_eq!(parsed.title, "Derivative[x^2, x]");
        assert_eq!(parsed.steps.len(), 2);
        assert_eq!(parsed.steps[0].rule, "PowerRule");
        assert_eq!(parsed.steps[0].before, "x^2");
        assert_eq!(parsed.steps[0].after, "2*x");
    }

    #[test]
    fn parseo_trunca_a_max_cas_steps() {
        let mut fence = String::from("```grafito-steps\n");
        for i in 0..(MAX_CAS_STEPS + 10) {
            fence.push_str(&format!(
                "- rule: Generic\n  desc: d{i}\n  before: b{i}\n  after: a{i}\n"
            ));
        }
        fence.push_str("```");
        let Some(card) = StepByStepCardState::from_code_block(&fence) else {
            panic!("fence largo parsea");
        };
        assert_eq!(card.steps.len(), MAX_CAS_STEPS);
    }

    #[test]
    fn fence_malformado_no_panica_y_da_none() {
        assert!(StepByStepCardState::from_code_block("```grafito\nx\n```").is_none());
        assert!(StepByStepCardState::from_code_block("texto plano").is_none());
        assert!(StepByStepCardState::from_code_block("```grafito-steps\nbasura\n```").is_none());
        // Paso incompleto (sin after) se suelta, no se inventa.
        let fence = "```grafito-steps\n- rule: Generic\n  desc: d\n  before: b\n```";
        assert!(StepByStepCardState::from_code_block(fence).is_none());
    }

    #[test]
    fn next_revealed_acota_al_total() {
        assert_eq!(next_revealed(0, 3), 1);
        assert_eq!(next_revealed(3, 3), 3);
        assert_eq!(next_revealed(9, 2), 2);
    }

    #[test]
    fn split_method_tag_separa_el_metodo_de_la_ecuacion() {
        // Caso del Solve lineal: la tag sale de la ecuación (el LaTeX se
        // comería el espacio y maquetaba `0(linealeexacta)`).
        let (expr, tag) = split_method_tag("0 (lineal exacta)");
        assert_eq!(expr, "0");
        assert_eq!(tag, Some("lineal exacta"));
        // Sin tag real: la expresión queda intacta.
        assert_eq!(split_method_tag("2*x"), ("2*x", None));
        assert_eq!(split_method_tag("x^2 = 0"), ("x^2 = 0", None));
        // Parens interiores que no son tag de método: no se separa nada.
        assert_eq!(
            split_method_tag("sin (algo) y (más)"),
            ("sin (algo) y (más)", None)
        );
        assert_eq!(split_method_tag("(solo)"), ("(solo)", None));
        // Paren sin cerrar: no se separa (defensa).
        assert_eq!(
            split_method_tag("0 (lineal exacta"),
            ("0 (lineal exacta", None)
        );
        // Tag corta e hífen se aceptan (formas compuestas).
        let (expr, tag) = split_method_tag("1+2 (riemann-bis)");
        assert_eq!(expr, "1+2");
        assert_eq!(tag, Some("riemann-bis"));
    }

    #[test]
    fn conteo_en_contenido_suma_fences_y_tolera_basura() {
        let steps = vec![cas_step("PowerRule", "x^2", "2*x")];
        let card = StepByStepCardState::from_cas_steps("op", &steps);
        let content = format!(
            "Hola\n{}\nchau\n{}",
            card.to_code_block(),
            card.to_code_block()
        );
        assert_eq!(count_steps_in_content(&content), 2);
        assert_eq!(count_steps_in_content("sin fences"), 0);
        assert_eq!(count_steps_in_content("```grafito-steps\nbasura\n```"), 0);
    }

    #[test]
    fn movimiento_reducido_salta_el_fade() {
        let reduced = MotionConfig {
            reduced_motion: true,
        };
        assert_eq!(reduced.morph_t(0.2), 1.0);
        assert_eq!(MotionConfig::default().morph_t(0.2), 0.2);
    }

    #[test]
    fn truncado_respeta_borde_utf8() {
        let value = "áéíóú".to_string();
        let cut = truncate_str(&value, 5);
        assert!(cut.len() <= 5);
        assert!(value.starts_with(&cut));
    }

    #[test]
    fn progreso_fraccion_revelada() {
        let steps = vec![
            cas_step("PowerRule", "x^2", "2*x"),
            cas_step("Simplify", "2*x", "2*x"),
        ];
        let mut card = StepByStepCardState::from_cas_steps("op", &steps);
        assert_eq!(card.progress(), 0.5);
        card.reveal_all();
        assert_eq!(card.progress(), 1.0);
        let empty = StepByStepCardState::from_cas_steps("op", &[]);
        assert_eq!(empty.progress(), 0.0);
    }

    #[test]
    fn copy_text_formatea_pasos_numerados() {
        let steps = vec![
            cas_step("PowerRule", "x^2", "2*x"),
            grafito_geometry::cas_steps::CasStep {
                index: 1,
                rule: rule_for("Simplify"),
                before: "2*x".to_string(),
                after: "2*x".to_string(),
                description: String::new(),
            },
        ];
        let card = StepByStepCardState::from_cas_steps("Derivative[x^2, x]", &steps);
        let text = card.to_copy_text(1);
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "Derivative[x^2, x]");
        assert_eq!(lines[1], "1 · PowerRule — aplica PowerRule / x^2 → 2*x");
        // Sin descripción no queda el guion colgando.
        assert_eq!(lines[2], "2 · Simplify / 2*x → 2*x");
    }

    #[test]
    fn copy_text_for_content_numera_corrido_entre_fences() {
        let steps = vec![cas_step("PowerRule", "x^2", "2*x")];
        let card = StepByStepCardState::from_cas_steps("op", &steps);
        let fence = card.to_code_block();
        let content = format!("mirá\n{fence}\ny esto\n{fence}");
        let text = copy_text_for_content(&content);
        assert!(text.contains("1 · PowerRule"), "{text}");
        assert!(text.contains("2 · PowerRule"), "{text}");
        assert_eq!(copy_text_for_content("sin fences"), "");
        assert_eq!(copy_text_for_content("```grafito-steps\nbasura\n```"), "");
    }

    #[test]
    fn dibujado_sin_click_no_emite_y_no_panica_en_ancho_y_angosto() {
        for width in [800.0, 200.0] {
            let steps = vec![
                cas_step("PowerRule", "x^2", "2*x"),
                cas_step("Simplify", "2*x", "2*x"),
            ];
            let mut card = StepByStepCardState::from_cas_steps("Derivative[x^2, x]", &steps);
            card.reveal_all();
            let motion = MotionConfig::default();
            egui::__run_test_ctx(|ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.allocate_ui(egui::vec2(width, 600.0), |ui| {
                        let action = draw_step_by_step_card(
                            ui,
                            &card,
                            0,
                            0,
                            STEPS_ZOOM_DEFAULT,
                            motion,
                            Locale::Es,
                            StepsTtsUi {
                                available: true,
                                speaking: false,
                            },
                        );
                        assert!(action.is_none(), "sin click no hay acción (ancho {width})");
                    });
                });
            });
        }
        // Sin pasos no hay botón (nada que proponer) y tampoco pánico.
        let empty = StepByStepCardState::from_cas_steps("op", &[]);
        egui::__run_test_ctx(|ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let action = draw_step_by_step_card(
                    ui,
                    &empty,
                    0,
                    0,
                    STEPS_ZOOM_DEFAULT,
                    MotionConfig::default(),
                    Locale::Es,
                    StepsTtsUi {
                        available: true,
                        speaking: false,
                    },
                );
                assert!(action.is_none(), "sin pasos no hay acción");
            });
        });
    }

    #[test]
    fn boton_ver_animacion_emite_en_angosto() {
        use egui::{Event, PointerButton};
        let ctx = egui::Context::default();
        ctx.set_fonts(egui::FontDefinitions::empty());
        let motion = MotionConfig::default();
        let screen = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(200.0, 700.0));
        // Barrido de abajo hacia arriba sin tocar la cabecera (un click en
        // el título colapsaría la tarjeta y movería el layout): con todo
        // revelado la fila de revelado está oculta y el botón apilado es el
        // último interactivo; el primer `AnimateSteps` es el botón buscado.
        let mut found = false;
        let mut y = 690.0;
        while y > 150.0 && !found {
            for pressed in [true, false] {
                let mut input = egui::RawInput {
                    screen_rect: Some(screen),
                    ..Default::default()
                };
                input.events.push(Event::PointerButton {
                    pos: egui::pos2(100.0, y),
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::NONE,
                });
                let steps = vec![
                    cas_step("PowerRule", "x^2", "2*x"),
                    cas_step("Simplify", "2*x", "2*x"),
                ];
                let mut card = StepByStepCardState::from_cas_steps("Derivative[x^2, x]", &steps);
                card.reveal_all();
                let mut got = None;
                let _ = ctx.run(input, |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        got = draw_step_by_step_card(
                            ui,
                            &card,
                            3,
                            0,
                            STEPS_ZOOM_DEFAULT,
                            motion,
                            Locale::Es,
                            StepsTtsUi {
                                available: true,
                                speaking: false,
                            },
                        );
                    });
                });
                if matches!(got, Some(AssistantUiAction::AnimateSteps { turn: 3 })) {
                    found = true;
                    break;
                }
            }
            y -= 6.0;
        }
        assert!(
            found,
            "el botón «Ver animación» no emitió AnimateSteps en angosto"
        );
    }

    #[test]
    fn zoom_niveles_pineados_y_default_100() {
        assert_eq!(STEPS_ZOOM_LEVELS, [100, 125, 150, 200]);
        assert_eq!(STEPS_ZOOM_DEFAULT, 100);
        assert_eq!(steps_zoom_factor(100), 1.0);
        assert_eq!(steps_zoom_factor(125), 1.25);
        assert_eq!(steps_zoom_factor(150), 1.5);
        assert_eq!(steps_zoom_factor(200), 2.0);
        assert_eq!(steps_zoom_factor(77), 1.0, "nivel ajeno normaliza a 1.0");
        assert_eq!(steps_zoom_index(100), 0);
        assert_eq!(steps_zoom_index(200), 3);
        assert_eq!(steps_zoom_index(77), 0, "desconocido → 100");
    }

    #[test]
    fn zoom_step_acota_en_topes() {
        assert_eq!(steps_zoom_step(100, 1), 125);
        assert_eq!(steps_zoom_step(125, 1), 150);
        assert_eq!(steps_zoom_step(150, 1), 200);
        assert_eq!(steps_zoom_step(200, 1), 200, "tope idempotente");
        assert_eq!(steps_zoom_step(200, -1), 150);
        assert_eq!(steps_zoom_step(150, -1), 125);
        assert_eq!(steps_zoom_step(125, -1), 100);
        assert_eq!(steps_zoom_step(100, -1), 100, "piso idempotente");
        assert_eq!(steps_zoom_step(150, 0), 150);
        assert_eq!(steps_zoom_step(77, 1), 125, "rancio arranca de 100");
    }

    #[test]
    fn zoom_disponibilidad_por_dpr() {
        // DPR 1 (dpr_q 4): `24*dpr*factor` en 8..=96 para los 4 niveles.
        for nivel in STEPS_ZOOM_LEVELS {
            assert!(
                steps_zoom_available(nivel, 4),
                "{nivel}% disponible a DPR 1"
            );
        }
        // DPR 2 (dpr_q 8): 200% pide 96.0 exactos (tope del motor, limpio).
        for nivel in STEPS_ZOOM_LEVELS {
            assert!(
                steps_zoom_available(nivel, 8),
                "{nivel}% disponible a DPR 2"
            );
        }
        // DPR 4 (dpr_q 16): 100% pide 96.0 (limpio); el resto se sale del
        // rango → no-disponible honesto, jamás clamp silencioso.
        assert!(steps_zoom_available(100, 16));
        for nivel in [125, 150, 200] {
            assert!(!steps_zoom_available(nivel, 16), "{nivel}% fuera a DPR 4");
        }
    }

    #[test]
    fn zoom_reraster_dimensiones_escalan() {
        // El camino tex a escala N da bitmap ~N× (re-raster limpio, no
        // upscale): a 200% las dims son ~2× las de 100% (tolerancia por el
        // `ceil` del raster).
        for fuente in ["x^2", "\\frac{1}{2}"] {
            let (Ok(base), Ok(doble)) = (
                grafito_tex::latex_to_rgba(fuente, grafito_tex::TEX_DEFAULT_FONT_PX),
                grafito_tex::latex_to_rgba(fuente, grafito_tex::TEX_DEFAULT_FONT_PX * 2.0),
            ) else {
                panic!("el camino tex rasteriza {fuente} a 100 y 200");
            };
            for (lado, a, b) in [
                ("ancho", base.width, doble.width),
                ("alto", base.height, doble.height),
            ] {
                #[allow(clippy::cast_precision_loss)]
                let ratio = b as f32 / a as f32;
                assert!(
                    (ratio - 2.0).abs() < 0.1,
                    "{fuente} {lado}: {ratio:.3}×, no 2×"
                );
            }
        }
    }

    #[test]
    fn dibujado_zoom_200_sin_click_no_emite() {
        let steps = vec![cas_step("PowerRule", "x^2", "2*x")];
        let mut card = StepByStepCardState::from_cas_steps("Derivative[x^2, x]", &steps);
        card.reveal_all();
        egui::__run_test_ctx(|ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let action = draw_step_by_step_card(
                    ui,
                    &card,
                    0,
                    0,
                    200,
                    MotionConfig::default(),
                    Locale::Es,
                    StepsTtsUi {
                        available: true,
                        speaking: false,
                    },
                );
                assert!(action.is_none(), "zoom 200 sin click no emite");
            });
        });
    }
    #[test]
    fn stagger_offset_escalona_60ms_y_comprime_lotes_grandes() {
        // Lote de un paso o paso ya pasado: sin desplazamiento (fade ya).
        assert_eq!(stagger_offset(0, 0, 1, 0.18), 0.0);
        assert_eq!(stagger_offset(0, 1, 1, 0.18), 0.0);
        // Retardo nominal 60 ms sobre reloj 0.18 s = 1/3 por paso.
        assert!((stagger_offset(1, 0, 2, 0.18) - 1.0 / 3.0).abs() < 1e-6);
        assert!((stagger_offset(2, 0, 3, 0.18) - 2.0 / 3.0).abs() < 1e-6);
        // Lote gigante: compresión uniforme con tope (sin pops).
        let big = stagger_offset(31, 0, 32, 0.18);
        assert!(big <= 0.8, "{big}");
        assert!(big > 0.0, "{big}");
        // Reloj degenerado: fallback al tema (0.18), mismo nominal.
        assert!((stagger_offset(1, 0, 2, 0.0) - 1.0 / 3.0).abs() < 1e-6);
        assert!((stagger_offset(1, 0, 2, f32::NAN) - 1.0 / 3.0).abs() < 1e-6);
    }

    #[test]
    fn tts_gate_etiqueta_hover_y_habilitado_pineados() {
        // Gate puro del botón Leer/Detener (sin binario ni mocks): idle
        // «Leer» + contrato en el hover; sonando «Detener»; sin binario
        // apagado con el comando instalable; sonando siempre detenible
        // (aunque el binario caiga en el medio).
        let idle = StepsTtsUi {
            available: true,
            speaking: false,
        };
        assert_eq!(idle.label_key(), "assistant.steps.read");
        assert_eq!(idle.hint_key(), "assistant.steps.read_hint");
        assert!(idle.enabled());
        let sonando = StepsTtsUi {
            available: true,
            speaking: true,
        };
        assert_eq!(sonando.label_key(), "assistant.steps.stop");
        assert_eq!(sonando.hint_key(), "assistant.steps.stop_hint");
        assert!(sonando.enabled());
        let sin_binario = StepsTtsUi {
            available: false,
            speaking: false,
        };
        assert_eq!(sin_binario.label_key(), "assistant.steps.read");
        assert_eq!(
            sin_binario.hint_key(),
            "assistant.steps.read_unavailable_hint"
        );
        assert!(!sin_binario.enabled(), "sin voz el botón va apagado");
        let sonando_sin_binario = StepsTtsUi {
            available: false,
            speaking: true,
        };
        assert!(sonando_sin_binario.enabled(), "sonando se puede detener");
        assert_eq!(sonando_sin_binario.label_key(), "assistant.steps.stop");
    }

    /// Barrido de clic en angosto sobre la fila apilada de «Leer»: con voz
    /// emite `ToggleSpeakSteps` (y «Detener» si suena); sin binario ningún
    /// clic de la tarjeta emite el toggle (botón apagado). La tarjeta se
    /// reconstruye por frame (layout estable, sin colapsos a medias).
    fn barre_tarjeta_angosto(tts: StepsTtsUi, de_arriba: bool) -> bool {
        use egui::{Event, PointerButton};
        let ctx = egui::Context::default();
        ctx.set_fonts(egui::FontDefinitions::empty());
        let motion = MotionConfig::default();
        let screen = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(200.0, 700.0));
        let mut encontrado = false;
        let mut y = if de_arriba { 150.0 } else { 690.0 };
        while if de_arriba { y < 690.0 } else { y > 150.0 } {
            for pressed in [true, false] {
                let mut input = egui::RawInput {
                    screen_rect: Some(screen),
                    ..Default::default()
                };
                input.events.push(Event::PointerButton {
                    pos: egui::pos2(100.0, y),
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::NONE,
                });
                let steps = vec![
                    cas_step("PowerRule", "x^2", "2*x"),
                    cas_step("Simplify", "2*x", "2*x"),
                ];
                let mut card = StepByStepCardState::from_cas_steps("Derivative[x^2, x]", &steps);
                card.reveal_all();
                let mut got = None;
                let _ = ctx.run(input, |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        got = draw_step_by_step_card(
                            ui,
                            &card,
                            3,
                            0,
                            STEPS_ZOOM_DEFAULT,
                            motion,
                            Locale::Es,
                            tts,
                        );
                    });
                });
                if matches!(got, Some(AssistantUiAction::ToggleSpeakSteps { turn: 3 })) {
                    encontrado = true;
                    break;
                }
            }
            if encontrado {
                break;
            }
            y += if de_arriba { 6.0 } else { -6.0 };
        }
        encontrado
    }

    #[test]
    fn boton_leer_emite_toggle_en_angosto_y_detener_si_suena() {
        assert!(
            barre_tarjeta_angosto(
                StepsTtsUi {
                    available: true,
                    speaking: false
                },
                true
            ),
            "«Leer» no emitió ToggleSpeakSteps en angosto"
        );
        // Sonando, la misma fila («Detener») emite el mismo toggle.
        assert!(
            barre_tarjeta_angosto(
                StepsTtsUi {
                    available: true,
                    speaking: true
                },
                true
            ),
            "«Detener» no emitió ToggleSpeakSteps en angosto"
        );
    }

    #[test]
    fn boton_leer_sin_binario_apagado_no_emite_toggle() {
        // Sin espeak ningún clic emite el toggle (ni de arriba ni de
        // abajo: el botón va apagado y nada habla sin clic válido).
        for de_arriba in [true, false] {
            assert!(
                !barre_tarjeta_angosto(
                    StepsTtsUi {
                        available: false,
                        speaking: false
                    },
                    de_arriba
                ),
                "sin voz no debe emitirse ToggleSpeakSteps"
            );
        }
    }
}
