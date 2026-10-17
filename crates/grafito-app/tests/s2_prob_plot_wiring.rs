//! S2 ui-polish: wiring del prob plot verificado desde fuera del prod.
//!
//! Lee `src/panels.rs` / `src/sheet_layout.rs` por `include_str!` con ruta:
//! `contains()` solo pasa si el PROD trae el pin (este archivo no se
//! auto-matchea, a diferencia del viejo test que vivía en `panels.rs` y se
//! grepeaba a sí mismo). Cómo rompe cada test si se borra el prod:
//! - `s2_helpers_definidos_en_sheet_layout`: falla si se borra alguna const
//!   `PROB_PLOT_*` o helper puro del fundido en `sheet_layout.rs`.
//! - `s2_draw_cablea_helpers_y_a11y`: falla si el draw deja de llamar a los
//!   helpers (p. ej. se re-inlinea la fórmula), si el `animate_bool` deja de
//!   ser por-`dist`, o si las barras pierden hover/`widget_info`.
//! - `s2_sin_formulas_inlineadas`: falla si alguien reintroduce la fórmula
//!   o el gate in-`if` en el draw saltándose los helpers (los tests de
//!   comportamiento de `panels.rs` quedarían midiendo código muerto).

const PANELS: &str = include_str!("../src/panels.rs");
const SHEET: &str = include_str!("../src/sheet_layout.rs");

#[test]
fn s2_helpers_definidos_en_sheet_layout() {
    // Dueños únicos de geometría + fundido (nombres exactos del contrato).
    for pin in [
        "pub(crate) const PROB_PLOT_H: f32 = 84.0;",
        "pub(crate) const PROB_PLOT_AXIS_RESERVE: f32 = 16.0;",
        "pub(crate) const PROB_PLOT_INSET: f32 = 4.0;",
        "pub(crate) const PROB_PLOT_BASELINE_W: f32 = 1.0;",
        "pub(crate) const PROB_PLOT_BAR_INSET_X: f32 = 1.0;",
        "pub(crate) const PROB_PLOT_BAR_GAP: f32 = 4.0;",
        "pub(crate) const PROB_PLOT_BAR_MIN: f32 = 1.0;",
        "pub(crate) const PROB_PLOT_BAR_ROUNDING: f32 = 2.0;",
        "pub(crate) const PROB_PLOT_LABEL_DY: f32 = 4.0;",
        "pub(crate) const PROB_PLOT_GROW_MIN: f32 = 0.25;",
        "pub(crate) fn prob_plot_grow(fade: f32) -> f32",
        "pub(crate) fn prob_plot_band_alpha(alpha: f32, fade: f32) -> f32",
        "pub(crate) fn prob_plot_bar_h(pmf: f64, ymax: f64, plot_h: f32, fade: f32) -> f32",
        "pub(crate) fn prob_plot_bar_text_for(",
    ] {
        assert!(SHEET.contains(pin), "sheet_layout.rs debe traer {pin}");
    }
}

#[test]
fn s2_draw_cablea_helpers_y_a11y() {
    // El draw/barras llaman a los helpers (no fórmulas inlineadas) y
    // exponen hover + lector de pantalla por barra.
    for pin in [
        "crate::sheet_layout::prob_plot_grow(fade)",
        "crate::sheet_layout::prob_plot_bar_h(*pmf, ymax, plot_h, fade)",
        "crate::sheet_layout::prob_plot_band_alpha(alpha, fade)",
        "(\"prob_plot_dist_fade\", state.dist)",
        "animate_bool",
        "paint_discrete_bars(",
        "prob_plot_bar_text_for(*k, *pmf, plot_locale)",
        "(\"prob_plot_bar\", dist, *k)",
        "on_hover_text(bar_text",
        "widget_info",
    ] {
        assert!(PANELS.contains(pin), "panels.rs debe cablear {pin}");
    }
}

#[test]
fn s2_sin_formulas_inlineadas() {
    // El prod no puede saltarse los helpers: si vuelve la fórmula inline
    // o el `if` inline, los tests de comportamiento medirían código muerto.
    for stale in [
        "PROB_PLOT_GROW_MIN + (1.0 - PROB_PLOT_GROW_MIN)",
        "if fade < 1.0",
        "(*pmf / ymax) as f32 * plot_h * grow",
        "gamma_multiply(alpha * fade)",
    ] {
        assert!(
            !PANELS.contains(stale),
            "panels.rs re-inlinea el fundido (usar helper): {stale}"
        );
    }
}
