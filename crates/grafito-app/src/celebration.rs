//! Celebración contenida — ganchos de sesión (primer correcta + primer paso a paso).
//!
//! Nivel visceral mínimo y escandinavo: una ráfaga de confeti de 600 ms +
//! un toast cálido, una sola vez por hito Y por sesión. Sin persistencia
//! (los flags mueren con la app), sin sonido, sin streak visible, sin
//! emojis, sin bloquear input, sin tocar el motor/CAS (solo se LEE el
//! `CommandOutcome` y el `Feedback.correct` ya calculados).
//!
//! - [`CelebrationState`]: flags de sesión + ráfagas vivas. Vive en
//!   `GrafitoApp` (jamás se serializa: la app no persiste).
//! - [`is_steps_success`]: detector puro del primer solver paso a paso
//!   (entrada `StepByStep[` + `Message` con fence de la tarjeta).
//! - [`first_exercise_toast`] / [`first_steps_toast`]: microcopy por locale
//!   (claves `toast.first_*`, rioplatense cálido, sin emojis).
//! - [`draw_celebration_overlay`]: pinta las ráfagas vivas en un layer
//!   no-interactivo; con `reduced_motion` no pinta nada (CERO).

use grafito_ui::i18n::{toast_msg, Locale};

/// Tope de ráfagas vivas a la vez. En la práctica hay como mucho 2 por
/// sesión (un hito dispara una sola vez); la cota es resguardo, no cola.
const MAX_BURSTS: usize = 2;

/// Apertura del comando del solver paso a paso (canónica del motor,
/// case-sensitive: ver `run_step_by_step_command` en grafito-command).
const STEP_BY_STEP_PREFIX: &str = "StepByStep[";
/// Marca de éxito real del solver: el fence que la tarjeta renderiza con
/// revelado progresivo. Sin fence no hubo pasos (motor vacío o error).
const STEPS_FENCE: &str = "```grafito-steps";

/// Estado de celebración de la sesión. `Default` apagado: ningún hito
/// celebrado al arrancar. No se serializa ni se lee de disco.
#[derive(Debug, Clone, Default)]
pub struct CelebrationState {
    /// Ya se celebró la primera respuesta correcta del panel de ejercicios.
    pub exercise_done: bool,
    /// Ya se celebró el primer solver paso a paso exitoso.
    pub steps_done: bool,
    /// Ráfagas vivas (se podan por tiempo en cada frame).
    bursts: Vec<grafito_ui::confetti::ConfettiBurst>,
}

impl CelebrationState {
    /// ¿Dispara la celebración del primer ejercicio correcto? Solo si el
    /// feedback ya dice `correct` y el hito sigue pendiente. Puro, sin I/O.
    pub fn should_celebrate_exercise(&self, correct: bool) -> bool {
        correct && !self.exercise_done
    }

    /// Marca el hito de ejercicio como celebrado y enciende su ráfaga.
    /// `now_ms` en el reloj de la UI (`ui_time * 1000`, como los toasts).
    /// Devuelve `true` si disparó (para tests y para pedir repaint).
    pub fn celebrate_exercise(&mut self, now_ms: f64) -> bool {
        if self.exercise_done {
            return false;
        }
        self.exercise_done = true;
        self.spawn(now_ms);
        true
    }

    /// ¿Dispara la celebración del primer paso a paso? Solo ante éxito real
    /// del solver (ver [`is_steps_success`]) y con el hito pendiente.
    /// Puro, sin I/O.
    pub fn should_celebrate_steps(&self, steps_success: bool) -> bool {
        steps_success && !self.steps_done
    }

    /// Marca el hito de paso a paso como celebrado y enciende su ráfaga.
    /// Devuelve `true` si disparó.
    pub fn celebrate_steps(&mut self, now_ms: f64) -> bool {
        if self.steps_done {
            return false;
        }
        self.steps_done = true;
        self.spawn(now_ms);
        true
    }

    /// Enciende una ráfaga determinista por seed de tiempo (ver
    /// `grafito_ui::confetti::confetti_seed`). Acotada a [`MAX_BURSTS`].
    fn spawn(&mut self, now_ms: f64) {
        self.prune(now_ms);
        if self.bursts.len() >= MAX_BURSTS {
            return;
        }
        let seed = grafito_ui::confetti::confetti_seed(now_ms);
        self.bursts
            .push(grafito_ui::confetti::ConfettiBurst::new(now_ms, seed));
    }

    /// Suelta las ráfagas vencidas. Puro, sin I/O.
    pub fn prune(&mut self, now_ms: f64) {
        self.bursts.retain(|b| b.is_alive(now_ms));
    }

    /// Ráfagas vivas ahora (apoyo de tests + overlay).
    pub fn live_bursts(&self, now_ms: f64) -> Vec<grafito_ui::confetti::ConfettiBurst> {
        self.bursts
            .iter()
            .copied()
            .filter(|b| b.is_alive(now_ms))
            .collect()
    }
}

/// Detecta el éxito real del solver paso a paso SIN tocar el motor: la
/// entrada debe abrir `StepByStep[` y el mensaje traer el fence de la
/// tarjeta. Un error del solver llega como `Error` (nunca `Message`), y el
/// motor vacío responde sin fence: ninguno celebra. Puro, sin I/O.
pub fn is_steps_success(input: &str, message: &str) -> bool {
    input.trim_start().starts_with(STEP_BY_STEP_PREFIX) && message.contains(STEPS_FENCE)
}

/// Microcopy del toast de primera correcta, en el locale pedido.
/// Rioplatense cálido, sin emojis (clave `toast.first_correct`).
pub fn first_exercise_toast(locale: Locale) -> &'static str {
    toast_msg("first_correct", locale)
}

/// Microcopy del toast de primer paso a paso, en el locale pedido.
/// Rioplatense cálido, sin emojis (clave `toast.first_steps`).
pub fn first_steps_toast(locale: Locale) -> &'static str {
    toast_msg("first_steps", locale)
}

/// Pinta las ráfagas vivas sobre un layer no-interactivo (nunca bloquea
/// input) con origen arriba al centro, bajo la top-bar, donde ya viven los
/// toasts. Con `reduced_motion` no pinta nada (CERO) ni pide repaint.
/// Retorna `true` mientras quede algo vivo (el llamador ya pidió repaint).
pub fn draw_celebration_overlay(
    ctx: &egui::Context,
    state: &mut CelebrationState,
    now_ms: f64,
    reduced_motion: bool,
) -> bool {
    state.prune(now_ms);
    let live = state.live_bursts(now_ms);
    if reduced_motion || live.is_empty() {
        return false;
    }
    let theme = grafito_ui::theme::current_theme(ctx);
    let screen = ctx.screen_rect();
    let origin = egui::pos2(screen.center().x, screen.min.y + 72.0);
    let layer = egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("grafito_celebration_confetti"),
    );
    let painter = ctx.layer_painter(layer);
    for burst in &live {
        grafito_ui::confetti::draw_burst(&painter, origin, burst, now_ms, theme, reduced_motion);
    }
    ctx.request_repaint();
    true
}

#[cfg(test)]
mod tests {
    use super::{first_exercise_toast, first_steps_toast, is_steps_success, CelebrationState};
    use grafito_ui::i18n::{toast_msg, Locale};

    const LOCALES: [Locale; 6] = [
        Locale::Es,
        Locale::En,
        Locale::Pt,
        Locale::It,
        Locale::Fr,
        Locale::De,
    ];

    #[test]
    fn ejercicio_celebra_solo_la_primera_correcta() {
        let mut state = CelebrationState::default();
        assert!(!state.should_celebrate_exercise(false));
        assert!(state.should_celebrate_exercise(true));
        assert!(state.celebrate_exercise(1_000.0));
        assert!(state.exercise_done);
        assert!(!state.should_celebrate_exercise(true));
        assert!(!state.celebrate_exercise(2_000.0));
        assert_eq!(state.live_bursts(1_100.0).len(), 1);
        assert!(state.live_bursts(2_000.0).is_empty());
    }

    #[test]
    fn pasos_celebra_solo_el_primer_exito() {
        let mut state = CelebrationState::default();
        assert!(!state.should_celebrate_steps(false));
        assert!(state.should_celebrate_steps(true));
        assert!(state.celebrate_steps(1_000.0));
        assert!(state.steps_done);
        assert!(!state.should_celebrate_steps(true));
        assert!(!state.celebrate_steps(2_000.0));
    }

    #[test]
    fn hitos_independientes_entre_si() {
        let mut state = CelebrationState::default();
        assert!(state.celebrate_exercise(1_000.0));
        assert!(state.should_celebrate_steps(true));
        assert!(state.celebrate_steps(1_100.0));
        assert_eq!(state.live_bursts(1_200.0).len(), 2);
    }

    #[test]
    fn detector_paso_a_paso_exige_prefijo_y_fence() {
        let ok = "StepByStep[Derivative]: 3 pasos\n\n```grafito-steps\n# Derivative[x^2, x]\n```";
        assert!(is_steps_success("StepByStep[Derivative[x^2, x]]", ok));
        assert!(is_steps_success("  StepByStep[Solve[x^2+3x=0, x]]", ok));
        assert!(!is_steps_success(
            "StepByStep[Derivative[x^2, x]]",
            "StepByStep[Derivative]: el motor no emitió pasos para esta operación"
        ));
        assert!(!is_steps_success("Derivative[x^2, x]", ok));
        assert!(!is_steps_success("stepbystep[Derivative[x^2, x]]", ok));
        assert!(!is_steps_success("StepByStep", ok));
        assert!(!is_steps_success("", ""));
    }

    #[test]
    fn microcopy_resuelve_en_seis_idiomas() {
        for locale in LOCALES {
            for text in [first_exercise_toast(locale), first_steps_toast(locale)] {
                assert!(!text.is_empty(), "vacío en {locale:?}");
            }
            assert_eq!(
                first_exercise_toast(locale),
                toast_msg("first_correct", locale)
            );
            assert_eq!(first_steps_toast(locale), toast_msg("first_steps", locale));
        }
        assert_eq!(
            first_exercise_toast(Locale::Es),
            "Bien ahí, che. Primera correcta de la sesión, seguí así."
        );
        assert_eq!(
            first_steps_toast(Locale::Es),
            "Paso a paso se entiende mejor, viste. Seguí explorando."
        );
        assert_eq!(
            first_exercise_toast(Locale::En),
            "Nicely done. First correct answer this session, keep it up."
        );
        assert_eq!(
            first_steps_toast(Locale::En),
            "Step by step it sinks in better. Keep exploring."
        );
    }

    /// Sin emojis en los dos microcopys (rangos emoji + símbolos
    /// misceláneos/dingbats que el catálogo ya usa en otras claves viejas).
    fn has_emoji(text: &str) -> bool {
        text.chars().any(|c| {
            matches!(c,
                '\u{2600}'..='\u{27BF}' | '\u{2B00}'..='\u{2BFF}' | '\u{1F300}'..='\u{1FAFF}' | '\u{FE0F}')
        })
    }

    #[test]
    fn microcopy_sin_emojis_rioplatense_calido() {
        for locale in LOCALES {
            for text in [first_exercise_toast(locale), first_steps_toast(locale)] {
                assert!(!has_emoji(text), "emoji en {locale:?}: {text}");
            }
        }
        assert!(first_exercise_toast(Locale::Es).contains("che"));
        assert!(first_steps_toast(Locale::Es).contains("viste"));
    }
}
