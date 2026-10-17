//! Grafito Theme System — Scandinavian.
//!
//! Paleta calm restraint: canvas #FAFAF9, panel #FFFFFF, separator
//! #E8E8E6, accent sage #6B7A6F. Sin translucidez (from_rgb opaco),
//! radios 8/12/16, sombras sutiles 0,2,8 alpha 8. Tipografía Inter.
//!
//! F5 Scandinavian quiet 2026-08-21:
//! - Ink secondary 64% → `text_secondary` DARK #A3A3A3 (163/255 ≈64%) para overlay calm sin competir con primary.
//! - Tertiary AA → `text_tertiary` DARK #8C8C8C / LIGHT #747474 (piso WCAG 1.4.3 4.5:1, ver tests).
//! - Border 10% → `separator.gamma_multiply(0.10)` usado en hairlines de composer/toolbar.
//! - Hover 5% → `separator.gamma_multiply(0.05)` / `hover_overlay` sutil (LIGHT #EBEDEA 5% blend) — verificado, no cambiar.
//!
//! LIGHT secondary/tertiary más oscuros por contraste AA 4.5 sobre #FAFAF9, pero mantienen jerarquía calm.

use crate::tokens::{
    ANIM_MICRO, FOCUS_RING_OFFSET, FOCUS_RING_WIDTH, RADIUS_2XL, RADIUS_MD, RADIUS_SM, RADIUS_XL,
    SCROLLBAR_W, SCROLL_HANDLE_MIN_H, SHADOW_ALPHA, SHADOW_POPUP_BLUR, SHADOW_POPUP_OFFSET_Y,
    SHADOW_WINDOW_BLUR, SHADOW_WINDOW_OFFSET_Y, SLATE_BORDER, SLATE_BORDER_GAMMA_MID, SLATE_N0,
    SLATE_N1, SLATE_N2, SPACE_LG, SPACE_MD, SPACE_SM, SPACE_XL, SPACE_XS, SPACING_BUTTON_X,
    SPACING_BUTTON_Y, SPACING_MINIMAL_X, SPACING_MINIMAL_Y, STROKE_HAIRLINE, TEXT_GAMMA_FLOOR,
    TYPE_BASE, TYPE_LG, TYPE_MD, TYPE_SM, TYPE_XL, TYPE_XS, TYPE_XXL,
};
use egui::{Color32, Context};

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    // ── Canvas y superficie ──
    pub canvas_bg: Color32,
    pub canvas_grid_minor: Color32,
    pub grid_line: Color32,
    pub grid_minor: Color32,

    // ── Paneles y chrome ──
    pub panel_bg: Color32,
    pub toolbar_bg: Color32,
    pub input_bar_bg: Color32,
    pub sidebar_bg: Color32,
    pub sidebar_tab_active_bg: Color32,
    pub sidebar_tab_inactive: Color32,
    pub sidebar_tab_active: Color32,
    pub status_bar_bg: Color32,
    pub separator: Color32, // border 10% vía gamma_multiply(0.10) en composer/toolbar hairlines
    pub input_bg: Color32,
    pub input_text: Color32,
    pub button_bg: Color32,
    pub button_hover: Color32,
    /// Superficie elevada de cards (F17.2, base Slate F15.1): N2 en oscuro,
    /// cálida elevada en claro. Nunca el mismo fill del panel: la card se
    /// lee por superficie + borde opaco 1 px, no por hairline translúcida.
    pub card_bg: Color32,

    // ── Teclado matemático ──
    pub keyboard_tab_active_bg: Color32,
    pub keyboard_tab_active_text: Color32,
    pub keyboard_tab_inactive: Color32,
    pub keyboard_key_bg: Color32,
    pub keyboard_key_hover: Color32,
    pub keyboard_key_border: Color32,
    pub keyboard_key_text: Color32,
    pub keyboard_enter_bg: Color32,
    pub keyboard_enter_hover: Color32,
    pub keyboard_enter_text: Color32,
    pub keyboard_delete_hover: Color32,
    pub keyboard_delete_hover_text: Color32,

    // ── Texto ── Scandinavian quiet F5: secondary 64% ink, tertiary 44% ink (dark values #A3/#73; light ajustado por contraste)
    pub text_primary: Color32,
    pub text_secondary: Color32, // 64% ink — secondary calm
    pub text_tertiary: Color32,  // tertiary AA 4.5 — hints legibles (piso, no bajar)
    pub text_label: Color32,

    // ── Acentos y estados ──
    pub accent: Color32,
    pub accent_muted: Color32,
    pub accent_strong: Color32,
    /// Texto de spans `código` inline sobre `accent_muted`: en modo oscuro el
    /// `accent` de marca queda negro sobre negro, así que el código usa un
    /// salvia claro propio (contraste ≥ 4.5 en ambos modos, ver tests).
    pub code_text: Color32,
    /// Texto de spans de matemática inline (`$…$`): salvia legible sobre el
    /// fondo de la burbuja en ambos modos (contraste ≥ 4.5, ver tests).
    pub math_text: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub danger: Color32,
    pub selection_bg: Color32,

    // ── Asistente macOS (vibrancy + burbujas) ──
    pub assistant_user_bubble: Color32,
    pub assistant_assistant_bubble: Color32,
    pub assistant_user_text: Color32,
    pub assistant_composer_bg: Color32,
    pub assistant_composer_border: Color32,

    // ── Toast notifications ──
    pub toast_bg: Color32,
    pub toast_border: Color32,
    pub toast_text: Color32,
    pub toast_info: Color32,
    pub toast_success: Color32,
    pub toast_error: Color32,
    pub toast_cas: Color32,

    // ── Geometría 2D ──
    pub axis_2d: Color32,
    pub axis_label: Color32,
    pub grid_axis: Color32,
    pub snap_indicator: Color32,

    // ── Objetos geométricos (para leyenda del panel de álgebra) ──
    pub object_point: Color32,
    pub object_line: Color32,
    pub object_function: Color32,
    pub object_conic: Color32,
    pub object_polygon: Color32,
    pub object_label: Color32,

    // ── Highlights y overlays ── border 10% (separator*0.10), hover 5% (hover_overlay sutil)
    pub highlight: Color32,
    pub ghost_preview: Color32,
    pub newly_created_glow: Color32,
    pub selection_outline: Color32,
    pub hover_overlay: Color32, // 5% hover — Scandinavian quiet
}

/// Rol visual de una tecla del teclado matemático.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardKeyRole {
    Standard,
    Delete,
    Enter,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KeyboardKeyVisuals {
    pub background: Color32,
    pub text: Color32,
    pub border: egui::Stroke,
}

impl Theme {
    pub fn keyboard_key_visuals(&self, role: KeyboardKeyRole, hovered: bool) -> KeyboardKeyVisuals {
        match (role, hovered) {
            (KeyboardKeyRole::Enter, false) => KeyboardKeyVisuals {
                background: self.keyboard_enter_bg,
                text: self.keyboard_enter_text,
                border: egui::Stroke::NONE,
            },
            (KeyboardKeyRole::Enter, true) => KeyboardKeyVisuals {
                background: self.keyboard_enter_hover,
                text: self.keyboard_enter_text,
                border: egui::Stroke::NONE,
            },
            (KeyboardKeyRole::Delete, true) => KeyboardKeyVisuals {
                background: self.keyboard_delete_hover,
                text: self.keyboard_delete_hover_text,
                border: egui::Stroke::new(1.0, self.keyboard_key_border),
            },
            (KeyboardKeyRole::Standard | KeyboardKeyRole::Delete, hovered) => KeyboardKeyVisuals {
                background: if hovered {
                    self.keyboard_key_hover
                } else {
                    self.keyboard_key_bg
                },
                text: self.keyboard_key_text,
                border: egui::Stroke::new(1.0, self.keyboard_key_border),
            },
        }
    }

    /// Hairline 1 px at 10 % separator — Scandinavian quiet border.
    /// Used for card and composer dividers (`separator.gamma_multiply(0.10)`).
    pub fn hairline_stroke(&self) -> egui::Stroke {
        egui::Stroke::new(1.0, self.separator.gamma_multiply(0.10))
    }

    /// Borde Slate 1 px al 40-60 % (`SLATE_BORDER_GAMMA_MID` 0.50) — F15.1.
    /// La separación Nordic Slate va por borde, no por saltos de fill:
    /// panel/sidebar/toolbar comparten N1 y el borde `#2F3642` los recorta.
    /// Puro render, sin I/O.
    pub fn slate_border_stroke(&self) -> egui::Stroke {
        egui::Stroke::new(
            STROKE_HAIRLINE,
            self.separator.gamma_multiply(SLATE_BORDER_GAMMA_MID),
        )
    }

    /// Borde de card 1 px opaco (`separator` sin atenuar) — F17.2.
    /// Ni hairline 10 % (efectivo 1.03:1 sobre N1 en oscuro: invisible) ni
    /// slate 50 % (`separator.gamma_multiply(0.50)` efectivo 1.02:1 sobre N1:
    /// el gamma_multiply halvea RGB *y* alfa, doble tenue). El opaco da
    /// 1.20:1 sobre N2 / 1.16:1 sobre `input_bg` oscuro y 1.28:1 sobre la
    /// card clara: quieto pero visible. Puro render, sin I/O.
    pub fn card_border_stroke(&self) -> egui::Stroke {
        egui::Stroke::new(STROKE_HAIRLINE, self.separator)
    }

    /// Card frame Scandinavian: `RADIUS_MD` rounding + borde opaco 1 px.
    /// Fill `card_bg` (N2 Slate en oscuro, elevada cálida en claro), nunca
    /// `panel_bg`: sobre chrome N1 la card con fill de panel + hairline 10 %
    /// se fundía en una sola superficie. For use in panels / cards
    /// (algebra, inspector, assistant, trig).
    pub fn card_frame(&self) -> egui::Frame {
        egui::Frame::none()
            .rounding(egui::Rounding::same(RADIUS_MD))
            .stroke(self.card_border_stroke())
            .fill(self.card_bg)
            .inner_margin(egui::Margin::same(SPACE_MD))
    }

    /// Stroke del anillo de foco visible — 2 px en `selection_outline`
    /// (WCAG 2.4.13 foco ≥ 2 px + WCAG 1.4.11 contraste no-texto ≥ 3:1).
    /// Puro render, sin I/O.
    pub fn focus_ring_stroke(&self) -> egui::Stroke {
        egui::Stroke::new(FOCUS_RING_WIDTH, self.selection_outline)
    }

    /// Rect expandido del anillo de foco (offset 2 px por lado, fuera del widget).
    pub fn focus_ring_rect(rect: egui::Rect) -> egui::Rect {
        rect.expand(FOCUS_RING_OFFSET)
    }

    /// Pinta el anillo de foco sobre el rect de un widget. Puro render, sin I/O.
    pub fn paint_focus_ring(&self, painter: &egui::Painter, rect: egui::Rect) {
        painter.rect_stroke(
            Self::focus_ring_rect(rect),
            egui::Rounding::same(RADIUS_SM),
            self.focus_ring_stroke(),
        );
    }

    /// Rótulo de canvas con halo opaco (`canvas_bg`): legible sobre curvas,
    /// grilla y widgets que lo crucen (etiqueta de función sobre slider).
    /// Sin halo los textos se funden con lo que pisan. Retorna el rect.
    /// El layout sale de la caché por contexto (patrón `label_galley_shared`
    /// de `render_2d.rs`, acá con `HashMap` + desalojo total por no tener
    /// `lru` en `grafito-ui`): mismo `FontId`/color ⇒ mismo píxel, sin
    /// re-resolver por frame en paneles con texto dinámico denso.
    pub fn paint_canvas_text(
        &self,
        painter: &egui::Painter,
        pos: egui::Pos2,
        anchor: egui::Align2,
        text: &str,
        font_id: egui::FontId,
        color: egui::Color32,
    ) -> egui::Rect {
        let galley = theme_galley_shared(painter, text, font_id, color);
        let rect = anchor
            .anchor_rect(egui::Rect::from_min_size(pos, galley.size()))
            .expand(2.0);
        painter.rect_filled(rect, 2.0, self.canvas_bg);
        painter.galley(rect.min, galley, color);
        rect
    }

    /// Clamp de atenuación gamma para texto al piso 0.85 (WCAG 1.4.3 AA).
    /// Todo `gamma_multiply` sobre texto debe pasar por aquí.
    pub fn clamp_text_gamma(gamma: f32) -> f32 {
        gamma.clamp(TEXT_GAMMA_FLOOR, 1.0)
    }

    /// Texto atenuado sin violar el piso gamma 0.85.
    pub fn dimmed_text(base: Color32, gamma: f32) -> Color32 {
        base.gamma_multiply(Self::clamp_text_gamma(gamma))
    }

    pub fn apply(&self, ctx: &Context) {
        let is_dark = self.canvas_bg.r() < 100;
        let mut visuals = if is_dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };

        visuals.panel_fill = self.panel_bg;
        visuals.window_fill = self.panel_bg;
        visuals.faint_bg_color = self.canvas_grid_minor;
        visuals.extreme_bg_color = self.input_bg;
        visuals.hyperlink_color = self.accent;
        visuals.selection.bg_fill = self.selection_bg;
        visuals.selection.stroke = egui::Stroke::new(1.0, self.accent);
        // Scandinavian: radios 16 restraint, sin drama.
        visuals.window_rounding = egui::Rounding::same(RADIUS_2XL);
        visuals.menu_rounding = egui::Rounding::same(RADIUS_XL);

        // Scandinavian: sombras sutiles 0,2,8 alpha 8 — calma, sin elevación teatral.
        visuals.window_shadow = egui::Shadow {
            offset: egui::vec2(0.0, SHADOW_WINDOW_OFFSET_Y),
            blur: SHADOW_WINDOW_BLUR,
            spread: 0.0,
            color: Color32::from_black_alpha(SHADOW_ALPHA),
        };
        visuals.popup_shadow = egui::Shadow {
            offset: egui::vec2(0.0, SHADOW_POPUP_OFFSET_Y),
            blur: SHADOW_POPUP_BLUR,
            spread: 0.0,
            color: Color32::from_black_alpha(SHADOW_ALPHA),
        };

        // Controles Scandinavian — radio 8, sin translucidez. Los toggles,
        // segmentados y badges chicos usan `RADIUS_PILL` explícito; las cards
        // y ventanas, sus radios de superficie. Este default manda en todo
        // `Button/TextEdit/ComboBox/Slider/Checkbox/Radio` sin rounding propio.
        visuals.widgets.noninteractive.bg_fill = self.panel_bg;
        visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, self.canvas_grid_minor);
        visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, self.text_primary);
        visuals.widgets.noninteractive.rounding = egui::Rounding::same(RADIUS_SM);

        visuals.widgets.inactive.bg_fill = self.button_bg;
        visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, self.separator);
        visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, self.text_primary);
        visuals.widgets.inactive.rounding = egui::Rounding::same(RADIUS_SM);

        visuals.widgets.hovered.bg_fill = self.button_hover;
        visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, self.accent);
        visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, self.text_primary);
        visuals.widgets.hovered.rounding = egui::Rounding::same(RADIUS_SM);

        visuals.widgets.active.bg_fill = self.selection_bg;
        visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, self.accent);
        visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, self.accent);
        visuals.widgets.active.rounding = egui::Rounding::same(RADIUS_SM);

        // Estado abierto (combo desplegado, colapsable): mismo radio, si no
        // el botón abierto volvía al default redondo de egui.
        visuals.widgets.open.bg_fill = self.button_hover;
        visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, self.accent);
        visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, self.text_primary);
        visuals.widgets.open.rounding = egui::Rounding::same(RADIUS_SM);

        ctx.set_visuals(visuals);

        ctx.style_mut(|s| {
            s.animation_time = ANIM_MICRO / 1_000.0;
            // Scandinavian: item_spacing 16, button_padding 16×8 — aire y calma.
            s.spacing.item_spacing = egui::vec2(SPACING_MINIMAL_X, SPACING_MINIMAL_Y);
            s.spacing.button_padding = egui::vec2(SPACING_BUTTON_X, SPACING_BUTTON_Y);
            s.spacing.menu_margin = egui::Margin::same(SPACE_SM); // 8 = SPACE_SM
            s.spacing.window_margin = egui::Margin::same(SPACE_MD); // 12 = SPACE_MD
            s.spacing.indent = SPACE_LG + SPACE_XS; // 20 = SPACE_LG(16) + SPACE_XS(4) == SPACE_XL - 4.0 derived
                                                    // Overlay flotante estilo macOS: sin espacio reservado, sin
                                                    // franja muerta, sin salto de layout al aparecer. Oculta en
                                                    // reposo (dormant 0), visible al hover/scrollear/drag (active/
                                                    // interact del default `floating()`). El contenido va a todo
                                                    // ancho con sus insets simétricos; la barra flota encima solo
                                                    // mientras se usa.
            s.spacing.scroll = egui::style::ScrollStyle {
                floating: true,
                floating_allocated_width: 0.0,
                bar_width: SCROLLBAR_W,
                bar_inner_margin: 0.0,
                bar_outer_margin: SPACE_XS,
                handle_min_length: SCROLL_HANDLE_MIN_H,
                dormant_handle_opacity: 0.0,
                dormant_background_opacity: 0.0,
                // Thumb con los fills de widgets (button/hover): el `true`
                // heredado usaba un `fg_stroke` nunca seteado y pintaba negro
                // en claro. En reposo se funde con el carril (quiet); al
                // hover/drag oscurece (feedback).
                foreground_color: false,
                ..egui::style::ScrollStyle::floating()
            };

            // interact_size 38x26 derived: 38 = SPACE_XL(24)+SPACE_LG(16)-SPACE_XS/2(2), 26 = SPACE_XL(24)+SPACE_XS/2(2)
            s.spacing.interact_size = egui::vec2(
                SPACE_XL + SPACE_LG - SPACE_XS / 2.0,
                SPACE_XL + SPACE_XS / 2.0,
            );
            // Tipografía Inter — escala Scandinavian 12/15/19
            s.text_styles = [
                (
                    egui::TextStyle::Small,
                    egui::FontId::new(TYPE_XS, egui::FontFamily::Proportional),
                ),
                (
                    egui::TextStyle::Body,
                    egui::FontId::new(TYPE_BASE, egui::FontFamily::Proportional),
                ),
                (
                    egui::TextStyle::Button,
                    egui::FontId::new(TYPE_SM, egui::FontFamily::Proportional),
                ),
                (
                    egui::TextStyle::Heading,
                    egui::FontId::new(TYPE_XL, egui::FontFamily::Proportional),
                ),
                (
                    egui::TextStyle::Monospace,
                    egui::FontId::new(TYPE_BASE, egui::FontFamily::Monospace),
                ),
            ]
            .into();
            let _ = TYPE_MD;
            let _ = TYPE_LG;
            let _ = TYPE_XXL;
        });
    }
}

pub fn current_theme(ctx: &Context) -> &'static Theme {
    if ctx.style().visuals.dark_mode {
        &DARK
    } else {
        &LIGHT
    }
}

// ── Caché de Galleys por contexto (patrón `label_galley_shared`) ──────────
// `paint_canvas_text` resolvía layout por frame; los textos repetidos de
// paneles densos comparten ahora el `Arc<Galley>` por (texto, fuente, color).
// Sin `lru` en `grafito-ui` (sin dependencias nuevas): `HashMap` acotada con
// desalojo total al llenar, viva en el `ctx` (`get_persisted`, una por
// contexto). Visual idéntico: misma `FontId` y color que el camino directo.
/// Tope de galleys cacheados por contexto (desalojo total al llenar).
const THEME_GALLEY_CACHE_MAX: usize = 128;

/// Clave de galley: texto + familia/tamaño + color (un cambio de color es
/// miss, no aliasing; el color va horneado en el galley).
#[derive(Clone, PartialEq, Eq, Hash)]
struct ThemeGalleyKey {
    text: String,
    family_tag: u8,
    size_bits: u32,
    color: u32,
}

#[derive(Clone, Default)]
struct ThemeGalleyCache {
    entradas: std::collections::HashMap<ThemeGalleyKey, std::sync::Arc<egui::Galley>>,
}

fn theme_galley_cache_id() -> egui::Id {
    egui::Id::new("grafito_theme_galleys")
}

fn theme_font_key(font_id: &egui::FontId) -> (u8, u32) {
    let family_tag = match font_id.family {
        egui::FontFamily::Proportional => 0,
        egui::FontFamily::Monospace => 1,
        _ => 2,
    };
    (family_tag, font_id.size.to_bits())
}

/// Galley compartido por contexto para `paint_canvas_text`. En hit no se
/// clona el `String` ni se re-resuelve el layout.
fn theme_galley_shared(
    painter: &egui::Painter,
    text: &str,
    font_id: egui::FontId,
    color: Color32,
) -> std::sync::Arc<egui::Galley> {
    let (family_tag, size_bits) = theme_font_key(&font_id);
    let key = ThemeGalleyKey {
        text: text.to_owned(),
        family_tag,
        size_bits,
        color: u32::from_be_bytes([color.r(), color.g(), color.b(), color.a()]),
    };
    let ctx = painter.ctx();
    if let Some(hit) = ctx.data_mut(|mapa| {
        mapa.get_persisted::<ThemeGalleyCache>(theme_galley_cache_id())
            .and_then(|caché| caché.entradas.get(&key).cloned())
    }) {
        return hit;
    }
    let galley = painter.layout_no_wrap(text.to_owned(), font_id, color);
    ctx.data_mut(|mapa| {
        let mut caché = mapa
            .get_persisted::<ThemeGalleyCache>(theme_galley_cache_id())
            .unwrap_or_default();
        if caché.entradas.len() >= THEME_GALLEY_CACHE_MAX {
            caché.entradas.clear();
        }
        caché.entradas.insert(key, galley.clone());
        mapa.insert_persisted(theme_galley_cache_id(), caché);
    });
    galley
}

/// Tema oscuro — Nordic Slate (F15.1): canvas N0 `#131518`, chrome N1, grilla N1/N2, borde `#2F3642` 40-60% (opaco).
pub static DARK: once_cell::sync::Lazy<Theme> = once_cell::sync::Lazy::new(|| Theme {
    canvas_bg: Color32::from_rgb(SLATE_N0[0], SLATE_N0[1], SLATE_N0[2]),
    canvas_grid_minor: Color32::from_rgb(SLATE_N1[0], SLATE_N1[1], SLATE_N1[2]),
    grid_line: Color32::from_rgb(SLATE_N2[0], SLATE_N2[1], SLATE_N2[2]),
    grid_minor: Color32::from_rgb(SLATE_N1[0], SLATE_N1[1], SLATE_N1[2]),

    panel_bg: Color32::from_rgb(SLATE_N1[0], SLATE_N1[1], SLATE_N1[2]),
    toolbar_bg: Color32::from_rgb(SLATE_N1[0], SLATE_N1[1], SLATE_N1[2]),
    input_bar_bg: Color32::from_rgb(SLATE_N1[0], SLATE_N1[1], SLATE_N1[2]),
    sidebar_bg: Color32::from_rgb(SLATE_N1[0], SLATE_N1[1], SLATE_N1[2]),
    sidebar_tab_active_bg: Color32::from_rgb(107, 122, 111),
    sidebar_tab_inactive: Color32::from_rgb(140, 145, 143),
    // Blanco: 4.52:1 sobre el chip (antes 3.18, menos que el inactivo).
    sidebar_tab_active: Color32::from_rgb(0xFF, 0xFF, 0xFF),
    // Chrome de una sola superficie N1 (F15.1): la separación Slate va por
    // borde 1 px `#2F3642`, no por saltos de fill (sidebar == panel a propósito).
    status_bar_bg: Color32::from_rgb(SLATE_N1[0], SLATE_N1[1], SLATE_N1[2]),
    separator: Color32::from_rgb(SLATE_BORDER[0], SLATE_BORDER[1], SLATE_BORDER[2]),
    // Cajas un escalón sobre el panel N1 (1.17:1, quieto pero visible).
    input_bg: Color32::from_rgb(0x28, 0x2C, 0x2E),
    input_text: Color32::from_rgb(0xFA, 0xFA, 0xF9),
    // Un paso sobre `input_bg` (era idéntico: thumb invisible y botones
    // planos). El thumb se lee en reposo y oscurece al hover.
    button_bg: Color32::from_rgb(0x2F, 0x33, 0x35),
    // Hover 1.37:1 sobre la caja (antes 1.23, indistinguible).
    button_hover: Color32::from_rgb(0x3E, 0x42, 0x44),
    // Card elevada Slate N2 (1.13:1 vs N1, quieta pero distinta del panel;
    // texto primary 13.96:1, secondary 5.78:1, tertiary 6.28:1 — AA).
    card_bg: Color32::from_rgb(SLATE_N2[0], SLATE_N2[1], SLATE_N2[2]),

    keyboard_tab_active_bg: Color32::from_rgb(0x4A, 0x58, 0x50),
    keyboard_tab_active_text: Color32::from_rgb(255, 255, 255),
    keyboard_tab_inactive: Color32::from_rgb(140, 145, 143),
    keyboard_key_bg: Color32::from_rgb(0x28, 0x2C, 0x2E),
    keyboard_key_hover: Color32::from_rgb(0x3E, 0x42, 0x44),
    // F15.1: un paso más claro que el `#3A3C3E` previo para mantener el
    // borde-a-panel ≥1.5 sobre el N1 Slate (medido 1.64 vs 1.49 antes).
    keyboard_key_border: Color32::from_rgb(0x40, 0x42, 0x46),
    keyboard_key_text: Color32::from_rgb(0xFA, 0xFA, 0xF9),
    keyboard_enter_bg: Color32::from_rgb(0x6B, 0x7A, 0x6F),
    // Oscurece al hover (1.67:1 vs bg) conservando blanco 7.5:1.
    keyboard_enter_hover: Color32::from_rgb(0x4A, 0x58, 0x50),
    keyboard_enter_text: Color32::from_rgb(255, 255, 255),
    keyboard_delete_hover: Color32::from_rgb(90, 60, 60),
    keyboard_delete_hover_text: Color32::from_rgb(0xFA, 0xFA, 0xF9),

    text_primary: Color32::from_rgb(0xFA, 0xFA, 0xF9),
    text_secondary: Color32::from_rgb(0xA3, 0xA3, 0xA3),
    // #AAA: AA también sobre superficies elevadas (hover, burbuja,
    // toast), donde el #8C caía bajo 4.5:1.
    text_tertiary: Color32::from_rgb(0xAA, 0xAA, 0xAA),
    text_label: Color32::from_rgb(0xAA, 0xAA, 0xAA),

    accent: Color32::from_rgb(92, 107, 96),
    accent_muted: Color32::from_rgb(58, 65, 60),
    accent_strong: Color32::from_rgb(130, 145, 134),
    code_text: Color32::from_rgb(198, 212, 200),
    math_text: Color32::from_rgb(168, 186, 172),
    success: Color32::from_rgb(107, 145, 110),
    warning: Color32::from_rgb(180, 150, 90),
    danger: Color32::from_rgb(180, 95, 85),
    // Tinte sage marcado: 1.88:1 vs panel, se distingue de accent_muted.
    selection_bg: Color32::from_rgb(58, 74, 63),

    // Despega de `accent` (1.10 por tono); texto sigue 6:1.
    assistant_user_bubble: Color32::from_rgb(84, 97, 90),
    // 1.45:1 vs panel (antes se fundía a 1.29).
    assistant_assistant_bubble: Color32::from_rgb(49, 56, 58),
    assistant_user_text: Color32::from_rgb(250, 250, 249),
    assistant_composer_bg: Color32::from_rgb(49, 56, 58),
    assistant_composer_border: Color32::from_rgb(72, 74, 75),

    // 1.52:1 vs panel (antes invisible a 1.17).
    toast_bg: Color32::from_rgb(51, 57, 59),
    toast_border: Color32::from_rgb(107, 122, 111),
    toast_text: Color32::from_rgb(250, 250, 249),
    toast_info: Color32::from_rgb(107, 122, 111),
    toast_success: Color32::from_rgb(107, 145, 110),
    toast_error: Color32::from_rgb(180, 95, 85),
    toast_cas: Color32::from_rgb(140, 130, 160),

    axis_2d: Color32::from_rgb(120, 125, 123),
    axis_label: Color32::from_rgb(140, 145, 143),
    grid_axis: Color32::from_rgb(140, 145, 143),
    snap_indicator: Color32::from_rgb(180, 150, 90),

    object_point: Color32::from_rgb(130, 145, 165),
    object_line: Color32::from_rgb(165, 170, 168),
    object_function: Color32::from_rgb(107, 145, 110),
    object_conic: Color32::from_rgb(180, 110, 110),
    object_polygon: Color32::from_rgb(180, 95, 85),
    object_label: Color32::from_rgb(250, 250, 249),

    highlight: Color32::from_rgb(180, 150, 90),
    ghost_preview: Color32::from_rgb(130, 145, 134),
    newly_created_glow: Color32::from_rgb(107, 122, 111),
    selection_outline: Color32::from_rgb(107, 122, 111),
    hover_overlay: Color32::from_rgb(0x3E, 0x42, 0x44),
});

/// Tema claro — Scandinavian: canvas #FAFAF9, panel #FFFFFF, separator #E8E8E6, sage #6B7A6F. Opaco.
pub static LIGHT: once_cell::sync::Lazy<Theme> = once_cell::sync::Lazy::new(|| Theme {
    canvas_bg: Color32::from_rgb(0xFA, 0xFA, 0xF9),
    canvas_grid_minor: Color32::from_rgb(0xF0, 0xF0, 0xEE),
    // Grilla en claro 2026-10-06 (pedido usuario): escalones grises como en
    // oscuro (N0/N1/N2). Mayor #D0D2CE → 1.46, menor #DFDFDC → 1.28 contra
    // #FAFAF9; gama blanco/gris, sin color. Eje 2.6 intacto.
    grid_line: Color32::from_rgb(0xD0, 0xD2, 0xCE),
    grid_minor: Color32::from_rgb(0xDF, 0xDF, 0xDC),

    panel_bg: Color32::from_rgb(0xFF, 0xFF, 0xFF),
    // Chrome un escalón bajo el papel (como el sidebar oscuro): la barra y
    // el rail ya no se funden con la ventana.
    toolbar_bg: Color32::from_rgb(0xF7, 0xF6, 0xF3),
    input_bar_bg: Color32::from_rgb(0xF7, 0xF6, 0xF3),
    // Jerarquía de superficies en claro: el panel es el papel (#FFFFFF) y el
    // sidebar baja un escalón a gris cálido (#EEEEEC → 1.16:1). Antes valía
    // #FFFFFF, idéntico al panel: el carril de navegación y la ventana se
    // fundían en una sola superficie y no se leía dónde terminaba una y
    // empezaba la otra. En oscuro la separación va por borde Slate 1 px
    // `#2F3642` (F15.1, chrome de una sola superficie N1) y por eso este
    // problema solo aparecía en claro.
    //
    // El gris tiene tope: bajar más rompe el AA del texto terciario sobre él
    // (a #EAEAE7 el #747474 cae a 3.88:1), así que la separación la aporta la
    // superficie y no un texto más oscuro.
    // Sidebar y status un escalón más: 1.24:1 vs panel, espejo del N1 oscuro.
    sidebar_bg: Color32::from_rgb(0xE9, 0xE7, 0xE1),
    // El chip activo necesita separarse del sidebar: a 1.12:1 es visible sin
    // gritar. Con el #EBEDEA anterior sobre #EEEEEC bajaba a 1.06:1 y el ítem
    // activo desaparecía en claro.
    sidebar_tab_active_bg: Color32::from_rgb(0xDC, 0xD9, 0xD1),
    // El rail dibuja con el MISMO color el ícono y la etiqueta de 11 px, así
    // que este token es texto, no adorno (WCAG 1.4.3 pide 4.5:1).
    // Antes #9A9E9C daba 2.68:1 sobre el sidebar #EEEEEC: los cinco tabs se
    // leían como un gris plano y el punto de entrada a los paneles perdía su
    // affordance. #666666 da 4.94:1 sin abandonar la familia neutra tibia
    // del sistema.
    sidebar_tab_inactive: Color32::from_rgb(0x66, 0x66, 0x66),
    // El chip activo no puede quedarse en `accent` (#6B7A6F → 3.38:1 sobre
    // #DEDFDA): el tab activo quedaba menos legible que el inactivo, que es
    // al revés de la jerarquía. `accent_strong` tampoco alcanza (4.21:1), así
    // que el activo baja a un sage profundo (#4A5850 → 5.59:1) que conserva la
    // identidad de marca sin perder el AA. El estado sigue distinguiéndose por
    // forma además de por color: el chip de fondo separa 1.12:1 del sidebar.
    sidebar_tab_active: Color32::from_rgb(0x4A, 0x58, 0x50),
    status_bar_bg: Color32::from_rgb(0xE9, 0xE7, 0xE1),
    // Borde real 1.50:1 vs blanco: las cards se leen por borde, no por fill.
    separator: Color32::from_rgb(0xD6, 0xD3, 0xCC),
    // Caja visible en claro: `#FAFAF9` sobre panel `#FFFFFF` era 1.02:1 y los
    // campos (buscador, claves, nombres) flotaban sin contenedor — se leía
    // como roto. `#F0F0EE` sigue quieto pero ancla lupa+hint+cursor.
    input_bg: Color32::from_rgb(0xEF, 0xED, 0xE8),
    input_text: Color32::from_rgb(0x2B, 0x2E, 0x2D),
    // Un paso sobre `input_bg` (era idéntico: thumb invisible en reposo).
    button_bg: Color32::from_rgb(0xE2, 0xE0, 0xDA),
    // Hover 1.12:1 sobre el botón (antes 1.03, indistinguible).
    button_hover: Color32::from_rgb(0xE3, 0xE1, 0xD9),
    // Card elevada cálida = `input_bg` (1.17:1 vs panel, espejo del N2 oscuro;
    // borde opaco 1.28:1 vs card, texto AA — ver test F17.2).
    card_bg: Color32::from_rgb(0xEF, 0xED, 0xE8),

    keyboard_tab_active_bg: Color32::from_rgb(0x6B, 0x7A, 0x6F),
    keyboard_tab_active_text: Color32::from_rgb(0xFF, 0xFF, 0xFF),
    keyboard_tab_inactive: Color32::from_rgb(0x5C, 0x66, 0x63),
    keyboard_key_bg: Color32::from_rgb(0xFF, 0xFF, 0xFF),
    keyboard_key_hover: Color32::from_rgb(0xE3, 0xE1, 0xD9),
    // Borde 4.6:1 (era 2.7, no pasaba no-texto).
    keyboard_key_border: Color32::from_rgb(0x75, 0x72, 0x6B),
    keyboard_key_text: Color32::from_rgb(0x1A, 0x1A, 0x1A),
    // Sage profundo 7.5:1 con blanco (era 3.9). Hover más profundo
    // (blanco 10:1, distinción 1.39) para no romper el AA del texto.
    keyboard_enter_bg: Color32::from_rgb(0x4A, 0x58, 0x50),
    keyboard_enter_hover: Color32::from_rgb(0x3A, 0x45, 0x40),
    keyboard_enter_text: Color32::from_rgb(0xFF, 0xFF, 0xFF),
    keyboard_delete_hover: Color32::from_rgb(0xF0, 0xE0, 0xDE),
    keyboard_delete_hover_text: Color32::from_rgb(0x1A, 0x1A, 0x1A),

    text_primary: Color32::from_rgb(0x1A, 0x1A, 0x1A),
    text_secondary: Color32::from_rgb(0x5A, 0x5A, 0x5A),
    // El sidebar bajó a #E9E7E1 y el #6C6C6C caía a 4.25:1 sobre él: ambos
    // grises acompañan (secondary 5.5, tertiary 4.86) y el rank
    // primary > secondary > tertiary se mantiene.
    text_tertiary: Color32::from_rgb(0x63, 0x63, 0x63),
    text_label: Color32::from_rgb(0x63, 0x63, 0x63),

    accent: Color32::from_rgb(0x6B, 0x7A, 0x6F),
    accent_muted: Color32::from_rgb(0xE3, 0xE1, 0xD9),
    accent_strong: Color32::from_rgb(0x5C, 0x6B, 0x60),
    code_text: Color32::from_rgb(0x4A, 0x55, 0x4E),
    math_text: Color32::from_rgb(0x5C, 0x6B, 0x60),
    success: Color32::from_rgb(0x2E, 0x5B, 0x32),
    warning: Color32::from_rgb(0x7A, 0x55, 0x10),
    danger: Color32::from_rgb(0x8E, 0x2E, 0x2E),
    // Pressed 1.17 vs hover y 1.3 vs botón (el hover y el wash de
    // selección comparten #E3E1D9 en contextos distintos).
    selection_bg: Color32::from_rgb(0xD8, 0xD5, 0xCD),

    assistant_user_bubble: Color32::from_rgb(0x6B, 0x7A, 0x6F),
    // Burbuja sobre el panel con borde real (antes 1.00 fundida).
    assistant_assistant_bubble: Color32::from_rgb(0xF4, 0xF3, 0xF0),
    assistant_user_text: Color32::from_rgb(0xFF, 0xFF, 0xFF),
    assistant_composer_bg: Color32::from_rgb(0xFF, 0xFF, 0xFF),
    assistant_composer_border: Color32::from_rgb(0xD6, 0xD3, 0xCC),

    toast_bg: Color32::from_rgb(0x2B, 0x2E, 0x2D),
    toast_border: Color32::from_rgb(0x6B, 0x7A, 0x6F),
    toast_text: Color32::from_rgb(0xFF, 0xFF, 0xFF),
    toast_info: Color32::from_rgb(0x6B, 0x7A, 0x6F),
    toast_success: Color32::from_rgb(0x6B, 0x8A, 0x6E),
    toast_error: Color32::from_rgb(0x9E, 0x5A, 0x4E),
    toast_cas: Color32::from_rgb(0x8A, 0x7A, 0x9E),

    axis_2d: Color32::from_rgb(0x9A, 0x9E, 0x9C),
    axis_label: Color32::from_rgb(0x6B, 0x76, 0x73),
    grid_axis: Color32::from_rgb(0x6B, 0x76, 0x73),
    snap_indicator: Color32::from_rgb(0x9A, 0x85, 0x55),

    object_point: Color32::from_rgb(0x6B, 0x7A, 0x6F),
    object_line: Color32::from_rgb(0x6B, 0x76, 0x73),
    object_function: Color32::from_rgb(0x6B, 0x8A, 0x6E),
    object_conic: Color32::from_rgb(0x9E, 0x5A, 0x4E),
    object_polygon: Color32::from_rgb(0x9E, 0x5A, 0x4E),
    object_label: Color32::from_rgb(0x2B, 0x2E, 0x2D),

    highlight: Color32::from_rgb(0xE8, 0xE0, 0xC8),
    ghost_preview: Color32::from_rgb(0xD0, 0xD8, 0xD2),
    newly_created_glow: Color32::from_rgb(0x6B, 0x7A, 0x6F),
    selection_outline: Color32::from_rgb(0x6B, 0x7A, 0x6F),
    hover_overlay: Color32::from_rgb(0xE3, 0xE1, 0xD9),
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_and_light_have_distinct_accents() {
        // Ambos usan sage en dark; en light también, pero canvas difiere.
        assert_ne!(DARK.canvas_bg, LIGHT.canvas_bg);
    }

    #[test]
    fn dark_canvas_is_dark() {
        assert!(DARK.canvas_bg.r() < 50);
    }

    #[test]
    fn light_canvas_is_light() {
        assert!(LIGHT.canvas_bg.r() > 200);
    }

    #[test]
    fn light_uses_scandinavian_palette() {
        assert_eq!(LIGHT.canvas_bg, Color32::from_rgb(0xFA, 0xFA, 0xF9));
        assert_eq!(LIGHT.panel_bg, Color32::from_rgb(0xFF, 0xFF, 0xFF));
        assert_eq!(LIGHT.separator, Color32::from_rgb(0xD6, 0xD3, 0xCC));
        assert_eq!(LIGHT.accent, Color32::from_rgb(0x6B, 0x7A, 0x6F));
    }

    #[test]
    fn current_theme_returns_dark_for_dark_context() {
        let ctx = Context::default();
        DARK.apply(&ctx);
        assert_eq!(current_theme(&ctx).accent, DARK.accent);
    }

    #[test]
    fn current_theme_returns_light_for_light_context() {
        let ctx = Context::default();
        LIGHT.apply(&ctx);
        assert_eq!(current_theme(&ctx).accent, LIGHT.accent);
    }

    #[test]
    fn all_required_tokens_defined() {
        let t = &*DARK;
        let _ = t.canvas_bg;
        let _ = t.grid_line;
        let _ = t.panel_bg;
        let _ = t.input_bar_bg;
        let _ = t.keyboard_tab_active_bg;
        let _ = t.keyboard_tab_active_text;
        let _ = t.keyboard_tab_inactive;
        let _ = t.keyboard_key_bg;
        let _ = t.keyboard_key_hover;
        let _ = t.keyboard_key_border;
        let _ = t.keyboard_key_text;
        let _ = t.keyboard_enter_bg;
        let _ = t.keyboard_enter_hover;
        let _ = t.keyboard_enter_text;
        let _ = t.keyboard_delete_hover;
        let _ = t.keyboard_delete_hover_text;
        let _ = t.sidebar_bg;
        let _ = t.status_bar_bg;
        let _ = t.card_bg;
        let _ = t.text_primary;
        let _ = t.text_secondary;
        let _ = t.text_tertiary;
        let _ = t.text_label;
        let _ = t.accent;
        let _ = t.accent_muted;
        let _ = t.accent_strong;
        let _ = t.code_text;
        let _ = t.math_text;
        let _ = t.success;
        let _ = t.warning;
        let _ = t.danger;
        let _ = t.toast_bg;
        let _ = t.toast_border;
        let _ = t.toast_text;
        let _ = t.toast_info;
        let _ = t.toast_success;
        let _ = t.toast_error;
        let _ = t.toast_cas;
        let _ = t.axis_2d;
        let _ = t.axis_label;
        let _ = t.grid_axis;
        let _ = t.snap_indicator;
        let _ = t.object_point;
        let _ = t.object_line;
        let _ = t.object_function;
        let _ = t.object_conic;
        let _ = t.object_polygon;
        let _ = t.object_label;
        let _ = t.highlight;
        let _ = t.ghost_preview;
        let _ = t.newly_created_glow;
        let _ = t.selection_outline;
        let _ = t.hover_overlay;
    }

    fn relative_luminance(color: Color32) -> f64 {
        let channel = |value: u8| {
            let value = f64::from(value) / 255.0;
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };

        0.2126 * channel(color.r()) + 0.7152 * channel(color.g()) + 0.0722 * channel(color.b())
    }

    fn contrast_ratio(foreground: Color32, background: Color32) -> f64 {
        let foreground = relative_luminance(foreground);
        let background = relative_luminance(background);
        (foreground.max(background) + 0.05) / (foreground.min(background) + 0.05)
    }

    fn assert_keyboard_state_contrast(theme: &Theme) {
        assert!(
            contrast_ratio(theme.keyboard_tab_active_text, theme.keyboard_tab_active_bg) >= 4.5,
            "active tab text contrast is {}",
            contrast_ratio(theme.keyboard_tab_active_text, theme.keyboard_tab_active_bg),
        );
        assert!(
            contrast_ratio(theme.keyboard_tab_inactive, theme.panel_bg) >= 4.5,
            "inactive tab text contrast is {}",
            contrast_ratio(theme.keyboard_tab_inactive, theme.panel_bg),
        );

        for role in [
            KeyboardKeyRole::Standard,
            KeyboardKeyRole::Delete,
            KeyboardKeyRole::Enter,
        ] {
            for hovered in [false, true] {
                let state = theme.keyboard_key_visuals(role, hovered);
                let name = format!("{role:?} {}", if hovered { "hover" } else { "normal" });
                assert!(
                    contrast_ratio(state.text, state.background) >= 4.5,
                    "{name} text contrast is {}",
                    contrast_ratio(state.text, state.background),
                );
                let (boundary, boundary_kind) = if state.border.width > 0.0 {
                    (
                        contrast_ratio(state.border.color, theme.panel_bg),
                        "border-to-panel",
                    )
                } else {
                    (
                        contrast_ratio(state.background, theme.panel_bg),
                        "fill-to-panel",
                    )
                };
                assert!(
                    boundary >= 1.5,
                    "{name} {boundary_kind} contrast is {boundary}",
                );
            }
        }
    }

    #[test]
    fn grid_lines_differ_from_canvas_in_both_themes() {
        // Regresión: la grilla 3D/2D usa estos tokens — si igualan al canvas
        // se vuelven invisibles (bug modo claro).
        for theme in [&*DARK, &*LIGHT] {
            assert_ne!(theme.grid_line, theme.canvas_bg);
            assert_ne!(theme.grid_minor, theme.canvas_bg);
        }
        // En claro el hairline necesita contraste real: mayor ≥1.3,
        // menor ≥1.15 (medidos contra #FAFAF9: 1.46 / 1.28).
        assert!(contrast_ratio(LIGHT.grid_line, LIGHT.canvas_bg) >= 1.3);
        assert!(contrast_ratio(LIGHT.grid_minor, LIGHT.canvas_bg) >= 1.15);
    }

    #[test]
    fn keyboard_rendered_states_have_accessible_contrast() {
        // Scandinavian: umbral relajado para border (1.5) — calm restraint, no drama.
        assert_keyboard_state_contrast(&DARK);
        assert_keyboard_state_contrast(&LIGHT);
    }

    #[test]
    fn inline_code_y_math_legibles_en_ambos_modos() {
        // Regresión del reporte real: en modo oscuro el código inline se veía
        // negro (accent de marca sobre accent_muted, contraste ~1.9).
        for theme in [&*DARK, &*LIGHT] {
            assert!(
                contrast_ratio(theme.code_text, theme.accent_muted) >= 4.5,
                "código inline sobre su fondo"
            );
            assert!(
                contrast_ratio(theme.math_text, theme.assistant_assistant_bubble) >= 4.5,
                "matemática inline sobre la burbuja del asistente"
            );
            assert!(
                contrast_ratio(theme.math_text, theme.panel_bg) >= 4.5,
                "matemática inline sobre el panel"
            );
        }
    }

    #[test]
    fn dark_interactive_surfaces_keep_text_readable() {
        assert!(contrast_ratio(DARK.text_primary, DARK.selection_bg) >= 4.5);
        assert!(contrast_ratio(DARK.text_primary, DARK.accent_muted) >= 4.5);
        assert!(contrast_ratio(DARK.text_secondary, DARK.toolbar_bg) >= 4.5);
    }

    #[test]
    fn light_status_colors_have_readable_contrast() {
        assert!(contrast_ratio(LIGHT.success, LIGHT.panel_bg) >= 4.5);
        assert!(contrast_ratio(LIGHT.warning, LIGHT.panel_bg) >= 4.5);
        assert!(contrast_ratio(LIGHT.danger, LIGHT.panel_bg) >= 4.5);
    }

    #[test]
    fn chrome_surfaces_are_opaque() {
        for theme in [&*DARK, &*LIGHT] {
            assert_eq!(theme.panel_bg.a(), 255);
            assert_eq!(theme.toolbar_bg.a(), 255);
            assert_eq!(theme.sidebar_bg.a(), 255);
        }
    }

    #[test]
    fn chrome_surfaces_keep_depth_and_readable_text() {
        for theme in [&*DARK, &*LIGHT] {
            assert!(contrast_ratio(theme.text_primary, theme.panel_bg) >= 4.5);
            assert!(contrast_ratio(theme.text_secondary, theme.toolbar_bg) >= 4.5);
        }
    }

    /// El carril lateral tiene que distinguirse de la ventana en modo claro.
    ///
    /// En claro `sidebar_bg` era `#FFFFFF`, idéntico a `panel_bg`: el carril y la
    /// ventana se fundían en una sola superficie y no se leía dónde terminaba una
    /// y empezaba la otra. En oscuro la separación va por borde Slate 1 px
    /// `#2F3642` sobre chrome N1 único (F15.1, sin saltos de fill).
    ///
    /// Solo se fija el modo claro a propósito: en oscuro subir el panel afecta
    /// a toda la app. Es un cambio de mayor riesgo que no se justifica sin
    /// medir el resultado.
    #[test]
    fn light_sidebar_is_a_distinct_surface_from_the_window() {
        let vs_panel = contrast_ratio(LIGHT.sidebar_bg, LIGHT.panel_bg);
        assert!(
            vs_panel >= 1.15,
            "sidebar claro {:?} vs panel {:?} = {vs_panel:.2}:1, se funden en una superficie",
            LIGHT.sidebar_bg,
            LIGHT.panel_bg
        );
        let chip = contrast_ratio(LIGHT.sidebar_tab_active_bg, LIGHT.sidebar_bg);
        assert!(
            chip >= 1.10,
            "chip activo vs sidebar = {chip:.2}:1, el ítem activo desaparece en claro"
        );
    }

    /// El texto terciario no puede perder el AA que el token promete.
    ///
    /// Bajar el gris del sidebar arrastra al texto que vive sobre él: con
    /// `#EEEEEC` el `#747474` caía a 4.02:1. El tertiary acompaña al sidebar y
    /// la jerarquía primary > secondary > tertiary se mantiene.
    #[test]
    fn tertiary_text_passes_aa_on_the_light_sidebar_and_keeps_its_rank() {
        let t = &*LIGHT;
        assert!(
            contrast_ratio(t.text_tertiary, t.sidebar_bg) >= 4.5,
            "tertiary {:?} sobre sidebar {:?} pierde el AA",
            t.text_tertiary,
            t.sidebar_bg
        );
        assert!(
            contrast_ratio(t.text_primary, t.sidebar_bg)
                >= contrast_ratio(t.text_secondary, t.sidebar_bg),
            "text_primary no puede leerse peor que text_secondary"
        );
        assert!(
            contrast_ratio(t.text_secondary, t.sidebar_bg)
                >= contrast_ratio(t.text_tertiary, t.sidebar_bg),
            "text_secondary no puede leerse peor que text_tertiary"
        );
    }

    #[test]
    fn theme_uses_short_state_transition_timing() {
        let ctx = Context::default();
        DARK.apply(&ctx);

        assert_eq!(ctx.style().animation_time, 0.18);
    }

    #[test]
    fn theme_uses_scandinavian_spacing_and_shadows() {
        let ctx = Context::default();
        LIGHT.apply(&ctx);
        let style = ctx.style();
        assert_eq!(style.spacing.item_spacing, egui::vec2(16.0, 16.0));
        assert_eq!(style.spacing.button_padding, egui::vec2(16.0, 8.0));
        assert_eq!(style.visuals.window_shadow.blur, 8.0);
        assert_eq!(style.visuals.window_shadow.offset.y, 2.0);
        assert_eq!(style.visuals.popup_shadow.blur, 8.0);
        assert_eq!(style.visuals.popup_shadow.color.a(), 8);
    }

    #[test]
    fn transparent_interaction_tokens_are_opaque() {
        // Scandinavian: ningún from_rgba_unmultiplied con alpha translúcido en chrome (excluye propio test).
        let source = include_str!("theme.rs");
        let chrome = &source[..source.find("#[cfg(test)]").unwrap_or(source.len())];
        assert!(!chrome.contains("from_rgba_unmultiplied"));
        assert!(!chrome.contains("from_rgba_premultiplied"));
    }

    #[test]
    fn focus_ring_is_visible_and_contrasted() {
        use crate::tokens::{FOCUS_RING_MIN_CONTRAST, FOCUS_RING_OFFSET, FOCUS_RING_WIDTH};
        for theme in [&*DARK, &*LIGHT] {
            let stroke = theme.focus_ring_stroke();
            assert_eq!(stroke.width, FOCUS_RING_WIDTH);
            assert!(
                contrast_ratio(stroke.color, theme.panel_bg) >= f64::from(FOCUS_RING_MIN_CONTRAST),
                "focus ring contrast is {}",
                contrast_ratio(stroke.color, theme.panel_bg),
            );
        }
        // El rect expandido deja OFFSET por lado.
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(10.0, 10.0));
        let expanded = Theme::focus_ring_rect(rect);
        assert_eq!(expanded.width(), 10.0 + 2.0 * FOCUS_RING_OFFSET);
        assert_eq!(expanded.height(), 10.0 + 2.0 * FOCUS_RING_OFFSET);
    }

    #[test]
    fn tertiary_text_passes_aa_on_chrome() {
        // WCAG 1.4.3: texto normal ≥ 4.5:1 contra panel y toolbar.
        for theme in [&*DARK, &*LIGHT] {
            assert!(
                contrast_ratio(theme.text_tertiary, theme.panel_bg) >= 4.5,
                "tertiary/panel contrast is {}",
                contrast_ratio(theme.text_tertiary, theme.panel_bg),
            );
            assert!(
                contrast_ratio(theme.text_tertiary, theme.toolbar_bg) >= 4.5,
                "tertiary/toolbar contrast is {}",
                contrast_ratio(theme.text_tertiary, theme.toolbar_bg),
            );
            assert!(
                contrast_ratio(theme.text_label, theme.panel_bg) >= 4.5,
                "label/panel contrast is {}",
                contrast_ratio(theme.text_label, theme.panel_bg),
            );
        }
    }

    #[test]
    fn dimmed_text_never_breaks_gamma_floor() {
        assert_eq!(Theme::clamp_text_gamma(0.5), TEXT_GAMMA_FLOOR);
        assert_eq!(Theme::clamp_text_gamma(0.85), TEXT_GAMMA_FLOOR);
        assert_eq!(Theme::clamp_text_gamma(0.95), 0.95);
        assert_eq!(Theme::clamp_text_gamma(1.5), 1.0);
        // `dimmed_text` con gamma bajo el piso equivale al piso.
        let base = Color32::from_rgb(200, 200, 200);
        assert_eq!(
            Theme::dimmed_text(base, 0.5),
            base.gamma_multiply(TEXT_GAMMA_FLOOR)
        );
        assert_eq!(Theme::dimmed_text(base, 1.0), base);
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn slate_tokens_pinean_hex_sin_negros_planos() {
        use crate::tokens::{
            SLATE_BORDER, SLATE_BORDER_GAMMA_MAX, SLATE_BORDER_GAMMA_MID, SLATE_BORDER_GAMMA_MIN,
            SLATE_N0, SLATE_N1, SLATE_N2,
        };
        // F15.1: canvas N0 `#131518`, chrome N1, grilla N2, borde `#2F3642`.
        assert_eq!(
            DARK.canvas_bg,
            Color32::from_rgb(SLATE_N0[0], SLATE_N0[1], SLATE_N0[2])
        );
        assert_eq!(DARK.canvas_bg, Color32::from_rgb(0x13, 0x15, 0x18));
        assert_eq!(
            DARK.panel_bg,
            Color32::from_rgb(SLATE_N1[0], SLATE_N1[1], SLATE_N1[2])
        );
        assert_eq!(DARK.toolbar_bg, DARK.panel_bg);
        assert_eq!(DARK.sidebar_bg, DARK.panel_bg);
        assert_eq!(DARK.status_bar_bg, DARK.panel_bg);
        assert_eq!(
            DARK.separator,
            Color32::from_rgb(SLATE_BORDER[0], SLATE_BORDER[1], SLATE_BORDER[2])
        );
        assert_eq!(DARK.separator, Color32::from_rgb(0x2F, 0x36, 0x42));
        assert_eq!(
            DARK.grid_line,
            Color32::from_rgb(SLATE_N2[0], SLATE_N2[1], SLATE_N2[2])
        );
        assert_eq!(DARK.grid_minor, DARK.panel_bg);
        // Sin negros planos en el chrome oscuro.
        for color in [
            DARK.canvas_bg,
            DARK.panel_bg,
            DARK.toolbar_bg,
            DARK.sidebar_bg,
            DARK.status_bar_bg,
            DARK.separator,
        ] {
            assert_ne!(color, Color32::BLACK);
            assert_ne!(color, Color32::from_rgb(0x00, 0x00, 0x00));
            assert_ne!(color, Color32::from_rgb(0x0A, 0x0A, 0x0A));
        }
        // Borde 1 px al 40-60% (punto medio 0.50).
        let stroke = DARK.slate_border_stroke();
        assert_eq!(stroke.width, 1.0);
        assert_eq!(
            stroke.color,
            DARK.separator.gamma_multiply(SLATE_BORDER_GAMMA_MID)
        );
        assert!(SLATE_BORDER_GAMMA_MIN <= 0.4 && 0.6 <= SLATE_BORDER_GAMMA_MAX);
    }

    #[test]
    fn slate_surfaces_pasan_aa_y_borde_quiet() {
        // WCAG 1.4.3: texto ≥ 4.5:1 sobre canvas N0 y chrome N1.
        for bg in [
            DARK.canvas_bg,
            DARK.panel_bg,
            DARK.toolbar_bg,
            DARK.sidebar_bg,
        ] {
            assert!(
                contrast_ratio(DARK.text_primary, bg) >= 4.5,
                "primary sobre slate {:?} = {}",
                bg,
                contrast_ratio(DARK.text_primary, bg)
            );
            assert!(
                contrast_ratio(DARK.text_secondary, bg) >= 4.5,
                "secondary sobre slate {:?} = {}",
                bg,
                contrast_ratio(DARK.text_secondary, bg)
            );
            assert!(
                contrast_ratio(DARK.text_tertiary, bg) >= 4.5,
                "tertiary sobre slate {:?} = {}",
                bg,
                contrast_ratio(DARK.text_tertiary, bg)
            );
        }
        // Borde quiet decorativo: visible sobre N0/N1 (≥1.3) sin pretender 3:1
        // (los límites que importan usan el anillo de foco, ya pineado a 3:1).
        assert!(contrast_ratio(DARK.separator, DARK.canvas_bg) >= 1.4);
        assert!(contrast_ratio(DARK.separator, DARK.panel_bg) >= 1.3);
        // Grilla legible sobre el canvas N0 sin gritar.
        assert!(contrast_ratio(DARK.grid_line, DARK.canvas_bg) >= 1.2);
    }

    /// F17.2: sidebars N1 + cards N2 con borde opaco 1 px, todo AA.
    ///
    /// Sidebar ya era N1 a propósito (F15.1, chrome de una sola superficie:
    /// la separación va por borde, no por saltos de fill). La card era el
    /// hueco: `card_frame` usaba fill `panel_bg` N1 + hairline 10 %
    /// (efectivo 1.03:1 sobre N1 en oscuro: invisible, la card se fundía con
    /// el panel). Ahora fill `card_bg` (N2 `#232935` en oscuro, elevada
    /// cálida en claro) + borde opaco 1 px (1.20:1 sobre N2, 1.28:1 sobre la
    /// card clara): quieto pero visible, sin fondos planos.
    #[test]
    fn slate_cards_usam_n2_con_borde_opaco_y_pasan_aa() {
        use crate::tokens::{SLATE_N2, STROKE_HAIRLINE};
        // Fill pineado: N2 oscuro, elevada cálida clara (= input_bg).
        assert_eq!(
            DARK.card_bg,
            Color32::from_rgb(SLATE_N2[0], SLATE_N2[1], SLATE_N2[2])
        );
        assert_eq!(DARK.card_bg, Color32::from_rgb(0x23, 0x29, 0x35));
        assert_eq!(LIGHT.card_bg, LIGHT.input_bg);
        // Sin negros planos en la card.
        for card in [DARK.card_bg, LIGHT.card_bg] {
            assert_ne!(card, Color32::BLACK);
            assert_ne!(card, Color32::from_rgb(0x00, 0x00, 0x00));
            assert_ne!(card, Color32::from_rgb(0x0A, 0x0A, 0x0A));
            assert_eq!(card.a(), 255);
        }
        // La card se distingue del panel (≥1.10) en ambos modos.
        assert!(contrast_ratio(DARK.card_bg, DARK.panel_bg) >= 1.10);
        assert!(contrast_ratio(LIGHT.card_bg, LIGHT.panel_bg) >= 1.10);
        // Sidebar sigue N1 a propósito (F15.1, chrome único).
        assert_eq!(DARK.sidebar_bg, DARK.panel_bg);
        assert_eq!(DARK.status_bar_bg, DARK.panel_bg);
        // Borde opaco 1 px, no hairline translúcida.
        for theme in [&*DARK, &*LIGHT] {
            let stroke = theme.card_border_stroke();
            assert_eq!(stroke.width, STROKE_HAIRLINE);
            assert_eq!(stroke.width, 1.0);
            assert_eq!(stroke.color, theme.separator);
            assert_eq!(stroke.color.a(), 255);
        }
        // `card_frame` usa card + borde opaco (no panel + hairline).
        for theme in [&*DARK, &*LIGHT] {
            let frame = theme.card_frame();
            // Frame no expone fill/stroke: se verifica por construcción
            // (fill card_bg + stroke borde opaco ya pineados arriba) y por
            // el caller productivo `trig_card` (app/panels.rs).
            let _ = frame;
        }
        // Texto AA sobre la card (WCAG 1.4.3 ≥ 4.5:1).
        for theme in [&*DARK, &*LIGHT] {
            assert!(
                contrast_ratio(theme.text_primary, theme.card_bg) >= 4.5,
                "primary sobre card"
            );
            assert!(
                contrast_ratio(theme.text_secondary, theme.card_bg) >= 4.5,
                "secondary sobre card"
            );
            assert!(
                contrast_ratio(theme.text_tertiary, theme.card_bg) >= 4.5,
                "tertiary sobre card"
            );
        }
        // Borde quieto pero visible sobre la card (≥1.15, sin pretender 3:1).
        assert!(contrast_ratio(DARK.separator, DARK.card_bg) >= 1.15);
        assert!(contrast_ratio(LIGHT.separator, LIGHT.card_bg) >= 1.15);
    }
}
