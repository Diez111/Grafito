//! Grafito Toast Notifications — Non-intrusive feedback messages.
//!
//! Los colores se resuelven dinámicamente desde el [`crate::theme::Theme`] activo usando
//! [`current_theme`]. Esto garantiza que los toasts respeten el modo
//! claro/oscuro sin hardcodear valores.
//!
//! Accesibilidad (WCAG 2.2.1 Timing Adjustable):
//! - Duraciones desde tokens (`TOAST_DURATION_DEFAULT` 7 s, error persistente).
//! - Hover pausa el temporizador; clic descarta el toast.

use egui::{Color32, FontId, Stroke, Vec2};

use crate::i18n::{t, Locale};
use crate::theme::current_theme;
use crate::tokens::{
    RADIUS_SM, SPACE_MD, SPACE_SM, TOAST_DURATION_DEFAULT, TOAST_DURATION_ERROR, TOAST_FADE_IN,
    TOAST_FADE_OUT, TOAST_MAX_SCREEN_FRACTION, TOAST_MIN_HEIGHT, TOAST_TOP_OFFSET, TYPE_SM,
};

#[derive(Clone, Debug, PartialEq)]
pub enum ToastAction {
    /// Copia el texto (un comando instalable) al portapapeles.
    CopyText(String),
}

#[derive(Clone)]
pub struct Toast {
    pub message: String,
    pub kind: ToastKind,
    pub created: f64,
    pub duration: f64,
    /// Acción opcional del aviso (`None` = solo descartar, como siempre).
    pub action: Option<ToastAction>,
}

impl Toast {
    /// Error persistente (sin auto-dismiss): sólo sale con clic.
    pub fn is_persistent(&self) -> bool {
        self.duration.is_infinite()
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ToastKind {
    Info,
    Success,
    Error,
    Cas,
}

impl ToastKind {
    /// Color del borde/acento del toast según el tema activo.
    pub fn color(&self, ctx: &egui::Context) -> Color32 {
        let t = current_theme(ctx);
        match self {
            ToastKind::Info => t.toast_info,
            ToastKind::Success => t.toast_success,
            ToastKind::Error => t.toast_error,
            ToastKind::Cas => t.toast_cas,
        }
    }

    /// Duración por severidad: 7 s por defecto, error persistente hasta dismiss.
    pub fn default_duration(&self) -> f64 {
        match self {
            ToastKind::Error => TOAST_DURATION_ERROR,
            ToastKind::Info | ToastKind::Success | ToastKind::Cas => TOAST_DURATION_DEFAULT,
        }
    }
}

/// Máximo de toasts visibles a la vez (Z3): la pila nunca tapa el panel
/// Álgebra ni el composer; el resto expira en cola sin dibujarse.
pub const TOAST_MAX_VISIBLE: usize = 2;
/// Tope de toasts guardados (Z3): evita crecimiento sin cota si se spamea
/// `push` más rápido que el expiry de 7 s. Solo se descartan no-persistentes
/// (los errores viven hasta clic/Esc).
const TOAST_MAX_STORED: usize = 8;

/// Alfa mínimo del borde del toast.
///
/// El borde es un componente y necesita 3:1 contra el fondo (WCAG 1.4.11).
/// Con el 39 % que salía de un alfa fijo daba 1.55:1. Este piso mantiene el
/// borde legible durante el fade sin hacerlo opaco de golpe.
const TOAST_BORDER_ALPHA_FLOOR: f32 = 0.85;

#[derive(Default)]
pub struct ToastManager {
    toasts: Vec<Toast>,
    /// Último `current_time` visto: permite pausar el temporizador en hover.
    last_time: Option<f64>,
    /// Idioma de la etiqueta del botón (`assistant.turn.copy`, reutilizada:
    /// 0 claves nuevas). La app lo setea por frame (`set_locale`).
    locale: Locale,
    /// Intención emitida por el botón (el `draw` es puro: la app la ejecuta
    /// con `copy_text` fuera del frame vía `take_pending_action`).
    pending_action: Option<ToastAction>,
    /// Rects del botón por índice de toast del último `draw` (solo tests:
    /// permite clickear el botón sin duplicar la geometría en el test).
    action_hits: Vec<(usize, egui::Rect)>,
}

impl ToastManager {
    /// Encola un aviso con coalescing por tópico (Z3): mismo mensaje + misma
    /// severidad = reemplaza (refresca el temporizador), no apila. Así
    /// "Generando animación…" repetido jamás forma pills superpuestas.
    /// Sin acción (los call-sites existentes quedan intactos).
    pub fn push(&mut self, msg: impl Into<String>, kind: ToastKind, time: f64) {
        self.push_with_action(msg, kind, time, None);
    }

    /// Variante con acción opcional (errores con salida copiable: OCR sin
    /// tesseract, MP4 sin ffmpeg). El coalescing sigue por mensaje +
    /// severidad; la acción del reemplazo gana.
    pub fn push_with_action(
        &mut self,
        msg: impl Into<String>,
        kind: ToastKind,
        time: f64,
        action: Option<ToastAction>,
    ) {
        let message = msg.into();
        self.toasts
            .retain(|t| !(t.message == message && t.kind == kind));
        let duration = kind.default_duration();
        self.toasts.push(Toast {
            message,
            kind,
            created: time,
            duration,
            action,
        });
        // Cota de guardado: descarta los no-persistentes más viejos.
        while self.toasts.len() > TOAST_MAX_STORED && self.toasts.iter().any(|t| !t.is_persistent())
        {
            if let Some(pos) = self.toasts.iter().position(|t| !t.is_persistent()) {
                self.toasts.remove(pos);
            }
        }
    }

    /// Último aviso encolado (apoyo de tests: pinna texto + severidad del
    /// call-site sin dibujar; la app lo usa desde sus tests).
    pub fn last_for_test(&self) -> Option<&Toast> {
        self.toasts.last()
    }

    /// Idioma del botón de acción. La app lo setea cada frame antes de
    /// `draw` (la etiqueta sale de `assistant.turn.copy`, ya en 6 locales).
    pub fn set_locale(&mut self, locale: Locale) {
        self.locale = locale;
    }

    /// Consume la intención del botón (la app ejecuta `copy_text` fuera del
    /// `draw`). `None` si ningún botón se clickeó desde el último consumo.
    pub fn take_pending_action(&mut self) -> Option<ToastAction> {
        self.pending_action.take()
    }

    /// Rect del botón del toast `index` en el último `draw` (solo tests).
    pub fn action_rect_for_test(&self, toast_index: usize) -> Option<egui::Rect> {
        self.action_hits
            .iter()
            .find(|(index, _)| *index == toast_index)
            .map(|(_, rect)| *rect)
    }

    /// Descarta el persistente más reciente (errores sin auto-dismiss).
    /// Puro, sin I/O: lo llama `draw` con Esc y los tests directo.
    /// Devuelve true si descartó algo.
    pub fn dismiss_top_persistent(&mut self) -> bool {
        if let Some(pos) = self.toasts.iter().rposition(|t| t.is_persistent()) {
            self.toasts.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn draw(&mut self, ui: &mut egui::Ui, current_time: f64) {
        // A11Y: Esc descarta el persistente más reciente (los errores no
        // salen solos; el lector ya anunció el texto vía live-region).
        if ui.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.dismiss_top_persistent();
        }
        self.toasts
            .retain(|t| current_time - t.created < t.duration);
        self.action_hits.clear();
        if self.toasts.is_empty() {
            self.last_time = Some(current_time);
            return;
        }

        // Pausa en hover (WCAG 2.2.1): congela el temporizador avanzando
        // `created` por el dt transcurrido, sin pasar nunca `current_time`.
        let dt = match self.last_time {
            Some(prev) => (current_time - prev).max(0.0),
            None => 0.0,
        };

        let ctx = ui.ctx();
        let theme = current_theme(ctx);
        let screen_rect = ctx.screen_rect();
        let mut y_offset = TOAST_TOP_OFFSET;
        // Leave the lower three quarters free for compact panels and their composer.
        let max_stack_y =
            (screen_rect.height() * TOAST_MAX_SCREEN_FRACTION).max(TOAST_TOP_OFFSET + SPACE_MD);

        // Sin `Vec`: como mucho `TOAST_MAX_VISIBLE` marcas por frame.
        let mut hovered: [Option<usize>; TOAST_MAX_VISIBLE] = [None; TOAST_MAX_VISIBLE];
        let mut hovered_len = 0usize;
        let mut clicked: [Option<usize>; TOAST_MAX_VISIBLE] = [None; TOAST_MAX_VISIBLE];
        let mut clicked_len = 0usize;
        // Intención del botón de acción (clic en botón ≠ dismiss: el toast
        // con error persistente sigue hasta clic en el texto o Esc).
        let mut fired_action: Option<ToastAction> = None;

        // Z3: como mucho TOAST_MAX_VISIBLE (los más recientes primero). Junto
        // al coalescing de `push`, dos jobs encadenados jamás se enciman.
        for (index, toast) in self.toasts.iter().enumerate().rev().take(TOAST_MAX_VISIBLE) {
            let elapsed = (current_time - toast.created) as f32;
            let fade_in = (elapsed / TOAST_FADE_IN).min(1.0);
            let fade_out = if elapsed > toast.duration as f32 - TOAST_FADE_OUT {
                ((toast.duration as f32 - elapsed) / TOAST_FADE_OUT).max(0.0)
            } else {
                1.0
            };
            let alpha = fade_in * fade_out;
            if alpha <= 0.01 {
                continue;
            }

            let text = &toast.message;
            let font = FontId::proportional(TYPE_SM);
            let painter = ui.painter();
            let galley = painter.layout(
                text.to_string(),
                font.clone(),
                theme.toast_text,
                toast_text_width_limit(screen_rect.width()),
            );
            // Etiqueta del botón reutilizada (`assistant.turn.copy`: Copiar /
            // Copy / Copiar / Copia / Copier / Kopieren, 0 claves nuevas).
            let action_label: Option<&'static str> = toast
                .action
                .as_ref()
                .map(|_| t("assistant.turn.copy", self.locale));
            let max_w = (screen_rect.width() - SPACE_MD * 2.0).max(1.0);
            // Geometría con acción: fila si texto + botón entran, apilado
            // debajo en angosto. Puro (ver `toast_action_geometry`).
            let action_geo = action_label.map(|label| {
                let label_galley = painter.layout(
                    label.to_owned(),
                    font.clone(),
                    theme.toast_text,
                    TOAST_ACTION_LABEL_WRAP,
                );
                let button = Vec2::new(
                    label_galley.size().x + SPACE_SM * 2.0,
                    label_galley.size().y + SPACE_SM,
                );
                (
                    label,
                    label_galley,
                    toast_action_geometry(galley.size(), button, max_w),
                )
            });
            let (w, h) = match &action_geo {
                Some((_, _, geo)) => (geo.size.x, geo.size.y),
                None => (
                    (galley.size().x + SPACE_MD * 2.0).min(max_w),
                    TOAST_MIN_HEIGHT.max(galley.size().y + SPACE_MD),
                ),
            };
            if y_offset + h + SPACE_MD > max_stack_y {
                break;
            }

            let pos = egui::pos2(
                // Z3: centrado bajo la top-bar, jamás sobre el panel Álgebra
                // (izquierda) ni el drawer/asistente (derecha).
                screen_rect.min.x + ((screen_rect.width() - w) * 0.5).max(0.0),
                screen_rect.min.y + y_offset,
            );
            let rect = egui::Rect::from_min_size(pos, Vec2::new(w, h));
            y_offset += h + SPACE_SM;
            // Color ya resuelto del `theme` del frame: evita un
            // `current_theme` (lock + `Arc` del estilo) por toast.
            let kind_color = match toast.kind {
                ToastKind::Info => theme.toast_info,
                ToastKind::Success => theme.toast_success,
                ToastKind::Error => theme.toast_error,
                ToastKind::Cas => theme.toast_cas,
            };
            // `from_rgba_premultiplied` NO premultiplica: toma los canales tal
            // cual y solo adjunta el alfa, así que el fade no existia y el
            // fondo se oscurecia hacia negro al bajar `alpha`. El constructor
            // correcto para atenuar es `linear_multiply`.
            let bg = theme.toast_bg.linear_multiply(alpha);
            // El borde es un control, no decoracion: necesita 3:1 contra el
            // fondo (WCAG 1.4.11). Con alfa al 39 % daba 1.55:1 y era
            // indistinguible del panel. Se atenua en el mismo rango que el
            // fondo pero conservando el piso del sistema.
            let border = kind_color.linear_multiply(TOAST_BORDER_ALPHA_FLOOR.min(alpha));
            painter.rect_filled(rect, egui::Rounding::same(RADIUS_SM), bg);
            painter.rect_stroke(
                rect,
                egui::Rounding::same(RADIUS_SM),
                Stroke::new(1.0, border),
            );
            match action_geo {
                Some((label, label_galley, geo)) => {
                    let button_rect =
                        egui::Rect::from_min_size(rect.min + geo.button_offset, geo.button);
                    // Hit disjuntos: el texto descarta, el botón emite la
                    // intención (pura: la app copia fuera del draw).
                    let text_hit = if geo.stacked {
                        egui::Rect::from_min_max(
                            rect.min,
                            egui::pos2(rect.max.x, button_rect.min.y),
                        )
                    } else {
                        egui::Rect::from_min_max(
                            rect.min,
                            egui::pos2(button_rect.min.x, rect.max.y),
                        )
                    };
                    let dismiss = ui.interact(
                        text_hit,
                        ui.id().with(("toast", index)),
                        egui::Sense::click(),
                    );
                    dismiss.widget_info(|| {
                        egui::WidgetInfo::labeled(egui::WidgetType::Label, true, text)
                    });
                    let action_response = ui.interact(
                        button_rect,
                        ui.id().with(("toast-action", index)),
                        egui::Sense::click(),
                    );
                    action_response.widget_info(|| {
                        egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label)
                    });
                    if dismiss.hovered() || action_response.hovered() {
                        hovered[hovered_len] = Some(index);
                        hovered_len += 1;
                    }
                    if dismiss.clicked() {
                        clicked[clicked_len] = Some(index);
                        clicked_len += 1;
                    }
                    if action_response.clicked() {
                        fired_action = toast.action.clone();
                    }
                    painter.rect_filled(
                        button_rect,
                        egui::Rounding::same(RADIUS_SM),
                        kind_color.linear_multiply(alpha * TOAST_ACTION_FILL),
                    );
                    painter.rect_stroke(
                        button_rect,
                        egui::Rounding::same(RADIUS_SM),
                        Stroke::new(1.0, border),
                    );
                    painter.galley_with_override_text_color(
                        button_rect.min
                            + Vec2::new(
                                (geo.button.x - label_galley.size().x).max(0.0) * 0.5,
                                (geo.button.y - label_galley.size().y).max(0.0) * 0.5,
                            ),
                        label_galley,
                        theme.toast_text.linear_multiply(alpha),
                    );
                    painter.galley_with_override_text_color(
                        rect.min + geo.text_offset,
                        galley,
                        theme.toast_text.linear_multiply(alpha),
                    );
                    self.action_hits.push((index, button_rect));
                }
                None => {
                    let response =
                        ui.interact(rect, ui.id().with(("toast", index)), egui::Sense::click());
                    response.widget_info(|| {
                        egui::WidgetInfo::labeled(egui::WidgetType::Label, true, text)
                    });
                    if response.hovered() {
                        hovered[hovered_len] = Some(index);
                        hovered_len += 1;
                    }
                    if response.clicked() {
                        clicked[clicked_len] = Some(index);
                        clicked_len += 1;
                    }
                    painter.galley_with_override_text_color(
                        rect.min + Vec2::new(SPACE_MD, (h - galley.size().y).max(0.0) * 0.5),
                        galley,
                        theme.toast_text.linear_multiply(alpha),
                    );
                }
            }
        }

        // La intención sobrevive al frame si la app aún no la consumió
        // (la app la ejecuta con `copy_text` fuera del `draw`).
        if fired_action.is_some() {
            self.pending_action = fired_action;
        }
        for index in hovered.into_iter().take(hovered_len).flatten() {
            if let Some(toast) = self.toasts.get_mut(index) {
                toast.created = (toast.created + dt).min(current_time);
            }
        }
        // `clicked` llega en orden descendente (loop `.rev()`): `remove` es seguro.
        for index in clicked.into_iter().take(clicked_len).flatten() {
            if index < self.toasts.len() {
                self.toasts.remove(index);
            }
        }
        // A11Y live-region: cada toast ya expone su texto vía `widget_info`
        // en el `interact`; el lector anuncia el push sin widgets extra
        // (un `Area` flotante rompería el hover — verificado en tests).
        self.last_time = Some(current_time);
    }
}

fn toast_text_width_limit(screen_width: f32) -> f32 {
    (screen_width - (SPACE_MD + SPACE_MD) * 2.0).max(1.0)
}

/// Ancho de wrap de la etiqueta del botón: las etiquetas (`assistant.turn.copy`)
/// son una sola palabra; este tope solo evita partirla en viewports absurdos.
const TOAST_ACTION_LABEL_WRAP: f32 = 1024.0;
/// Relleno del botón con el color de severidad (sutil: el borde ya da el 3:1).
const TOAST_ACTION_FILL: f32 = 0.22;

/// Geometría del botón de acción dentro del toast (pura, sin egui).
struct ToastActionGeometry {
    /// Tamaño total del toast.
    size: Vec2,
    /// Offset del texto desde `rect.min`.
    text_offset: Vec2,
    /// Offset del botón desde `rect.min`.
    button_offset: Vec2,
    /// Tamaño medido del botón.
    button: Vec2,
    /// `true` si el botón va apilado debajo (angosto).
    stacked: bool,
}

/// Fila (texto + botón lado a lado) si entran en `max_width`; apilado debajo
/// en angosto. Sin constantes mágicas: la decisión sale del contenido.
fn toast_action_geometry(text: Vec2, button: Vec2, max_width: f32) -> ToastActionGeometry {
    let gap = SPACE_SM;
    if text.x + gap + button.x <= max_width - SPACE_MD * 2.0 {
        let height = TOAST_MIN_HEIGHT.max(text.y.max(button.y) + SPACE_MD);
        ToastActionGeometry {
            size: Vec2::new(text.x + gap + button.x + SPACE_MD * 2.0, height),
            text_offset: Vec2::new(SPACE_MD, (height - text.y).max(0.0) * 0.5),
            button_offset: Vec2::new(SPACE_MD + text.x + gap, (height - button.y).max(0.0) * 0.5),
            button,
            stacked: false,
        }
    } else {
        ToastActionGeometry {
            size: Vec2::new(
                (text.x + SPACE_MD * 2.0).min(max_width),
                text.y + gap + button.y + SPACE_MD,
            ),
            text_offset: Vec2::new(SPACE_MD, SPACE_MD * 0.5),
            button_offset: Vec2::new(SPACE_MD, SPACE_MD * 0.5 + text.y + gap),
            button,
            stacked: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        toast_text_width_limit, ToastKind, ToastManager, TOAST_DURATION_DEFAULT,
        TOAST_DURATION_ERROR, TOAST_TOP_OFFSET,
    };

    #[test]
    fn toast_text_width_preserves_screen_margins() {
        let screen_width = 100.0;
        let limit = toast_text_width_limit(screen_width);

        assert!(limit > 0.0);
        assert!(limit + 48.0 <= screen_width);
    }

    #[test]
    fn long_toasts_stay_inside_a_narrow_viewport() {
        let context = egui::Context::default();
        let mut toasts = ToastManager::default();
        toasts.push("x".repeat(100), ToastKind::Error, 0.0);
        let output = context.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(100.0, 1_200.0),
                )),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| toasts.draw(ui, 0.1));
            },
        );
        let rectangles = output
            .shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(shape) => Some(shape.rect),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert!(!rectangles.is_empty());
        assert!(rectangles.iter().all(|rect| {
            rect.min.x >= 0.0 && rect.max.x <= 100.0 && rect.min.y >= 0.0 && rect.max.y <= 1_200.0
        }));
        assert!(rectangles
            .iter()
            .filter(|rect| rect.width() < 100.0)
            .all(|rect| rect.min.y >= TOAST_TOP_OFFSET && rect.max.y <= 300.0));
    }

    #[test]
    fn toast_durations_come_from_tokens() {
        assert_eq!(ToastKind::Info.default_duration(), TOAST_DURATION_DEFAULT);
        assert_eq!(
            ToastKind::Success.default_duration(),
            TOAST_DURATION_DEFAULT
        );
        assert_eq!(ToastKind::Cas.default_duration(), TOAST_DURATION_DEFAULT);
        assert_eq!(ToastKind::Error.default_duration(), TOAST_DURATION_ERROR);
        assert!(ToastKind::Error.default_duration().is_infinite());
    }

    #[test]
    fn push_assigns_duration_by_kind() {
        let mut manager = ToastManager::default();
        manager.push("info", ToastKind::Info, 0.0);
        manager.push("boom", ToastKind::Error, 0.0);

        assert_eq!(manager.toasts.len(), 2);
        assert_eq!(manager.toasts[0].duration, TOAST_DURATION_DEFAULT);
        assert!(!manager.toasts[0].is_persistent());
        assert!(manager.toasts[1].is_persistent());
    }

    fn toast_rect_count(toasts: &mut ToastManager, time: f64) -> usize {
        let context = egui::Context::default();
        let output = context.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| toasts.draw(ui, time));
        });
        output
            .shapes
            .iter()
            .filter(|clipped| matches!(clipped.shape, egui::Shape::Rect(_)))
            .count()
    }

    fn empty_rect_count(time: f64) -> usize {
        toast_rect_count(&mut ToastManager::default(), time)
    }

    #[test]
    fn default_toast_expires_after_token_duration() {
        let mut manager = ToastManager::default();
        manager.push("hola", ToastKind::Info, 0.0);

        assert!(toast_rect_count(&mut manager, 6.9) > empty_rect_count(6.9));
        assert_eq!(toast_rect_count(&mut manager, 7.1), empty_rect_count(7.1));
    }

    #[test]
    fn error_toast_persists_until_click() {
        let mut manager = ToastManager::default();
        manager.push("boom", ToastKind::Error, 0.0);

        assert!(toast_rect_count(&mut manager, 3_600.0) > empty_rect_count(3_600.0));
    }

    fn draw_with_pointer(
        ctx: &egui::Context,
        toasts: &mut ToastManager,
        time: f64,
        events: Vec<egui::Event>,
    ) -> usize {
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(400.0, 1_200.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| toasts.draw(ui, time));
            },
        );
        output
            .shapes
            .iter()
            .filter(|clipped| matches!(clipped.shape, egui::Shape::Rect(_)))
            .count()
    }

    #[test]
    fn hover_pauses_toast_expiry() {
        // Toast centrado en viewport 400px: hover en (200, 65) cae sobre la
        // pill; lejos en (350, 1100).
        let ctx = egui::Context::default();
        let hover = egui::pos2(200.0, 65.0);
        let away = egui::pos2(350.0, 1_100.0);
        let moved = |pos| vec![egui::Event::PointerMoved(pos)];
        let mut manager = ToastManager::default();
        manager.push("pausa", ToastKind::Info, 0.0);

        // Priming: egui resuelve hover con los rects del pass previo, así que
        // el primer frame sólo registra el widget (hovered=false).
        draw_with_pointer(&ctx, &mut manager, 0.3, moved(hover));
        // 5.7 s bajo el cursor: el temporizador se congela en created = 5.7.
        draw_with_pointer(&ctx, &mut manager, 6.0, moved(hover));
        // A t=12 sin hover han pasado 6.3 s efectivos < 7 s: sigue visible.
        assert!(draw_with_pointer(&ctx, &mut manager, 12.0, moved(away)) > empty_rect_count(12.0));
        // A t=13.5 sin hover: 7.8 s efectivos ≥ 7 s: expiró.
        assert_eq!(
            draw_with_pointer(&ctx, &mut manager, 13.5, moved(away)),
            empty_rect_count(13.5)
        );
    }

    #[test]
    fn click_dismisses_toast() {
        let ctx = egui::Context::default();
        let at_toast = egui::pos2(200.0, 65.0);
        let mut manager = ToastManager::default();
        let baseline = draw_with_pointer(&ctx, &mut ToastManager::default(), 0.0, Vec::new());
        manager.push("fuera", ToastKind::Error, 0.0);

        // Priming: el primer frame sólo registra el widget (hovered=false).
        draw_with_pointer(
            &ctx,
            &mut manager,
            0.3,
            vec![egui::Event::PointerMoved(at_toast)],
        );
        draw_with_pointer(
            &ctx,
            &mut manager,
            0.4,
            vec![egui::Event::PointerButton {
                pos: at_toast,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::default(),
            }],
        );
        draw_with_pointer(
            &ctx,
            &mut manager,
            0.5,
            vec![egui::Event::PointerButton {
                pos: at_toast,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::default(),
            }],
        );
        // Tras press+release sobre el toast, el error persistente desapareció.
        assert_eq!(
            draw_with_pointer(&ctx, &mut manager, 0.6, Vec::new()),
            baseline
        );
    }

    #[test]
    fn dismiss_top_persistent_drops_only_errors() {
        let mut manager = ToastManager::default();
        assert!(!manager.dismiss_top_persistent());
        manager.push("info", ToastKind::Info, 0.0);
        assert!(!manager.dismiss_top_persistent());
        assert_eq!(manager.toasts.len(), 1);
        manager.push("boom", ToastKind::Error, 0.0);
        manager.push("info2", ToastKind::Info, 0.0);
        assert!(manager.dismiss_top_persistent());
        assert_eq!(manager.toasts.len(), 2);
        assert!(manager.toasts.iter().all(|t| !t.is_persistent()));
    }

    #[test]
    fn escape_dismisses_persistent_toast_in_draw() {
        let ctx = egui::Context::default();
        let mut manager = ToastManager::default();
        manager.push("boom", ToastKind::Error, 0.0);
        let esc = vec![egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::default(),
        }];
        let baseline = draw_with_pointer(&ctx, &mut ToastManager::default(), 0.5, Vec::new());
        assert_eq!(draw_with_pointer(&ctx, &mut manager, 0.5, esc), baseline);
        assert!(manager.toasts.is_empty());
    }

    #[test]
    fn same_topic_replaces_instead_of_stacking() {
        // Z3: "Generando animación…" repetido = UNA pill con timer fresco,
        // no dos encimadas. Distinto mensaje sí convive.
        let mut manager = ToastManager::default();
        manager.push("Generando animación…", ToastKind::Info, 0.0);
        manager.push("Generando animación…", ToastKind::Info, 3.0);
        assert_eq!(manager.toasts.len(), 1);
        assert_eq!(manager.toasts[0].created, 3.0);
        manager.push("Animación lista.", ToastKind::Success, 3.5);
        assert_eq!(manager.toasts.len(), 2);
    }

    #[test]
    fn at_most_two_toasts_drawn() {
        // Z3: con 3 avisos encolados solo se dibujan los 2 más recientes.
        // Cada toast pinta 2 rects (fondo + borde); se resta el baseline.
        let mut manager = ToastManager::default();
        manager.push("uno", ToastKind::Info, 0.0);
        manager.push("dos", ToastKind::Info, 0.0);
        manager.push("tres", ToastKind::Info, 0.0);
        let ctx = egui::Context::default();
        let count = draw_with_pointer(&ctx, &mut manager, 0.5, Vec::new());
        let baseline = draw_with_pointer(&ctx, &mut ToastManager::default(), 0.5, Vec::new());
        assert_eq!(count - baseline, 2 * super::TOAST_MAX_VISIBLE);
    }

    #[test]
    fn push_legacy_sin_accion_deja_none() {
        // Contrato: los call-sites existentes (`push`) siguen sin acción.
        let mut manager = ToastManager::default();
        manager.push("info", ToastKind::Info, 0.0);
        manager.push("boom", ToastKind::Error, 0.0);

        assert!(manager.toasts.iter().all(|t| t.action.is_none()));
        assert!(manager.last_for_test().expect("hay aviso").action.is_none());
        assert!(manager.take_pending_action().is_none());
    }

    #[test]
    fn push_with_action_guarda_copytext() {
        use super::ToastAction;

        let mut manager = ToastManager::default();
        manager.push_with_action(
            "sin tesseract",
            ToastKind::Error,
            0.0,
            Some(ToastAction::CopyText("sudo pacman -S tesseract".into())),
        );

        assert_eq!(
            manager.last_for_test().expect("hay aviso").action,
            Some(ToastAction::CopyText("sudo pacman -S tesseract".into()))
        );
        // Sin clic no hay intención: el draw es puro.
        assert!(manager.take_pending_action().is_none());
    }

    /// Dibuja en un viewport de `width` y devuelve la salida completa (para
    /// pinchar `copied_text`: el draw jamás copia).
    fn draw_full_output(
        ctx: &egui::Context,
        toasts: &mut ToastManager,
        time: f64,
        width: f32,
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(width, 1_200.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| toasts.draw(ui, time));
            },
        )
    }

    fn click_sequence(center: egui::Pos2) -> [Vec<egui::Event>; 3] {
        let moved = vec![egui::Event::PointerMoved(center)];
        let press = vec![egui::Event::PointerButton {
            pos: center,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::default(),
        }];
        let release = vec![egui::Event::PointerButton {
            pos: center,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::default(),
        }];
        [moved, press, release]
    }

    #[test]
    fn boton_emite_intencion_y_no_descarta_ni_copia() {
        use super::ToastAction;

        // Botón emite, handler copia, None intacto en viejos: acá se pincha
        // que el draw solo emite la intención (sin copiar ni descartar).
        let ctx = egui::Context::default();
        let mut manager = ToastManager::default();
        let cmd = "sudo pacman -S ffmpeg";
        manager.push_with_action(
            "corto",
            ToastKind::Error,
            0.0,
            Some(ToastAction::CopyText(cmd.into())),
        );

        // Priming: registra widgets y publica el rect del botón.
        draw_full_output(&ctx, &mut manager, 0.3, 400.0, Vec::new());
        let button = manager
            .action_rect_for_test(0)
            .expect("el toast con acción pinta botón");
        let [moved, press, release] = click_sequence(button.center());
        draw_full_output(&ctx, &mut manager, 0.35, 400.0, moved);
        draw_full_output(&ctx, &mut manager, 0.4, 400.0, press);
        let output = draw_full_output(&ctx, &mut manager, 0.5, 400.0, release);

        assert_eq!(
            manager.take_pending_action(),
            Some(ToastAction::CopyText(cmd.into()))
        );
        // El error persistente sigue (el botón no descarta) y el draw no
        // tocó el portapapeles (lo hace la app fuera del draw).
        assert_eq!(manager.toasts.len(), 1);
        assert!(output.platform_output.copied_text.is_empty());
    }

    #[test]
    fn click_en_texto_descarta_sin_emitir() {
        use super::ToastAction;

        let ctx = egui::Context::default();
        let mut manager = ToastManager::default();
        manager.push_with_action(
            "corto",
            ToastKind::Error,
            0.0,
            Some(ToastAction::CopyText("cmd".into())),
        );

        draw_full_output(&ctx, &mut manager, 0.3, 400.0, Vec::new());
        let button = manager
            .action_rect_for_test(0)
            .expect("el toast con acción pinta botón");
        // Punto del área de texto (fila: a la izquierda del botón).
        let at_text = egui::pos2(button.min.x - 5.0, button.center().y);
        let [moved, press, release] = click_sequence(at_text);
        draw_full_output(&ctx, &mut manager, 0.35, 400.0, moved);
        draw_full_output(&ctx, &mut manager, 0.4, 400.0, press);
        draw_full_output(&ctx, &mut manager, 0.5, 400.0, release);

        assert!(manager.toasts.is_empty(), "clic en texto descarta");
        assert!(manager.take_pending_action().is_none());
    }

    #[test]
    fn geometria_fila_si_entra_apila_si_angosto() {
        use super::{toast_action_geometry, SPACE_MD, SPACE_SM};

        let text = egui::vec2(40.0, 12.0);
        let button = egui::vec2(56.0, 20.0);
        let wide = toast_action_geometry(text, button, 376.0);
        assert!(!wide.stacked);
        // Fila: botón a la derecha del texto, misma banda vertical.
        assert!(wide.button_offset.x > wide.text_offset.x + text.x);
        assert!((wide.button_offset.y - (wide.size.y - button.y) * 0.5).abs() < 0.01);

        let narrow = toast_action_geometry(text, button, 76.0);
        assert!(narrow.stacked);
        // Apilado: botón debajo del texto, alineado al padding.
        assert!((narrow.button_offset.x - SPACE_MD).abs() < 0.01);
        assert!(narrow.button_offset.y >= SPACE_MD * 0.5 + text.y + SPACE_SM - 0.01);
    }

    #[test]
    fn angosto_apila_boton_dentro_del_viewport() {
        use super::ToastAction;

        let ctx = egui::Context::default();
        let mut manager = ToastManager::default();
        manager.push_with_action(
            "x".repeat(100),
            ToastKind::Error,
            0.0,
            Some(ToastAction::CopyText("cmd".into())),
        );
        draw_full_output(&ctx, &mut manager, 0.1, 100.0, Vec::new());
        let button = manager
            .action_rect_for_test(0)
            .expect("en angosto también hay botón");
        assert!(button.min.x >= 0.0 && button.max.x <= 100.0);
        assert!(button.min.y >= TOAST_TOP_OFFSET && button.max.y <= 300.0);
    }

    #[test]
    fn etiqueta_boton_existe_en_seis_locales() {
        // La etiqueta reutiliza `assistant.turn.copy`: 0 claves nuevas,
        // `MSG_COUNT` intacto. Si alguna variante cayera a vacío, el botón
        // quedaría mudo en ese idioma.
        use crate::i18n::{t, Locale};

        for locale in [
            Locale::Es,
            Locale::En,
            Locale::Pt,
            Locale::It,
            Locale::Fr,
            Locale::De,
        ] {
            assert!(!t("assistant.turn.copy", locale).is_empty(), "{locale:?}");
        }
    }
}
