//! Profesionalismo (Tasks.md:78): skeleton+%, preview LaTeX vivo, sonificación/Braille.
//!
//! Piel pura (`fn render(&Estado) -> Frame`): sin I/O, sin spawn. Cierra los
//! 3 gaps restantes del epic; los otros 6 subitems ya existen:
//! - sliders Play+prompt auto → `grafito-app/src/panels.rs:6317,6784,6874`
//! - trace hover readout → `trace.rs:22` (`trace_from_hover` + formato + chip)
//! - tooltips ricos + MRU paleta → `command_palette.rs:576,512,551` (MRU cap 8)
//! - keypad matemático → `grafito-app/src/keyboard.rs` (teclado en pantalla
//!   cableado: dock inferior con tabs 123/f(x)/ABC/3D, `insert_keyboard_text`
//!   con foco composer→barra, `draw_math_keypad` de ui eliminado en B-A116
//!   por duplicado sin callers)
//! - onboarding tour → `grafito-app/src/app.rs:9211` (`draw_onboarding_window`,
//!   gating `!onboarding_completed`, reabrible desde Ayuda)
//! - projectorMode → `projector.rs:14,87` (toggle + hit-targets aula)
//! - preview LaTeX base → `assistant.rs:13812` (`draw_math` raster tex) +
//!   `grafito-tex` 8 KiB; acá solo el gate puro del vivo (sin raster por frame)
//! - progreso honesto base → `assistant.rs:787` (`Exporting` indeterminado,
//!   jamás inventa %); acá el texto + decisión de skeleton puros.
//!
//! La sonificación/Braille (L) queda en stubs honestos pineados (precedente
//! línea 76 IA): `Err` visible con motivo, jamás silencio ni fake.

/// Texto honesto de progreso para la Piel. `None`/NaN → "…" (indeterminado,
/// jamás inventa %); `Some(v)` clampeado a 0..=1 en "N %".
pub fn format_progress_percent(progress: Option<f32>) -> String {
    let Some(value) = progress else {
        return "…".to_string();
    };
    if !value.is_finite() {
        return "…".to_string();
    }
    let pct = (value.clamp(0.0, 1.0) * 100.0).round() as u32;
    format!("{pct} %")
}

/// ¿Mostrar skeleton placeholder? Solo con job activo y sin completar.
/// `active` lo setea la app fuera del draw; `progress` es el mismo
/// `Option<f32>` del diálogo (`None` = indeterminado con spinner).
/// Completo (`Some(1.0)`) o inactivo → `false` (nada que esqueletar).
pub fn skeleton_visible(active: bool, progress: Option<f32>) -> bool {
    if !active {
        return false;
    }
    match progress {
        None => true,
        Some(value) => {
            if !value.is_finite() {
                return true;
            }
            value.clamp(0.0, 1.0) < 1.0
        }
    }
}

/// Gate puro de la vista previa LaTeX en vivo (mientras se escribe).
///
/// No rasteriza (el raster `draw_math`/`latex_to_rgba` corre a lo sumo al
/// confirmar o con throttle en el llamador): solo decide si hay algo que
/// previsualizar y si entra en presupuesto.
/// - Vacío/solo espacios → `Ok(None)` (nada que mostrar, sin error).
/// - NUL → `Err` honesto (el pipeline tex lo rechazaría igual).
/// - Más de `TEX_INPUT_MAX_BYTES` (8 KiB) → `Err` honesto con el tope.
/// - Resto → `Ok(Some(bytes))` para que el llamador mida el costo.
pub fn latex_live_preview_check(source: &str) -> Result<Option<usize>, String> {
    if source.contains('\0') {
        return Err("la fórmula contiene NUL".to_string());
    }
    let bytes = source.len();
    if bytes > grafito_tex::TEX_INPUT_MAX_BYTES {
        return Err(format!(
            "vista previa de {bytes} bytes, máximo {}",
            grafito_tex::TEX_INPUT_MAX_BYTES
        ));
    }
    if source.trim().is_empty() {
        return Ok(None);
    }
    Ok(Some(bytes))
}

/// Sonificación no disponible: stub honesto (L). Siempre `Err` visible con
/// motivo y alternativa (subtítulos/toast visual); jamás silencio.
pub fn sonify_unavailable() -> Result<(), &'static str> {
    Err("sonificación no disponible: usá el readout visual de la traza")
}

/// Braille no disponible: stub honesto (L). Siempre `Err` visible con
/// motivo; jamás salida fake.
pub fn braille_unavailable() -> Result<(), &'static str> {
    Err("salida Braille no disponible en esta versión")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progreso_honesto_nunca_inventa_porcentaje() {
        assert_eq!(format_progress_percent(None), "…");
        assert_eq!(format_progress_percent(Some(f32::NAN)), "…");
        assert_eq!(format_progress_percent(Some(f32::INFINITY)), "…");
        assert_eq!(format_progress_percent(Some(0.0)), "0 %");
        assert_eq!(format_progress_percent(Some(0.123)), "12 %");
        assert_eq!(format_progress_percent(Some(1.0)), "100 %");
        assert_eq!(format_progress_percent(Some(2.0)), "100 %");
        assert_eq!(format_progress_percent(Some(-0.5)), "0 %");
    }

    #[test]
    fn skeleton_solo_con_job_activo_sin_completar() {
        assert!(!skeleton_visible(false, None));
        assert!(!skeleton_visible(false, Some(0.5)));
        assert!(skeleton_visible(true, None));
        assert!(skeleton_visible(true, Some(0.0)));
        assert!(skeleton_visible(true, Some(0.5)));
        assert!(skeleton_visible(true, Some(f32::NAN)));
        assert!(!skeleton_visible(true, Some(1.0)));
        assert!(!skeleton_visible(true, Some(2.0)));
    }

    #[test]
    fn preview_vivo_vacio_sin_error_y_tope_8kib() {
        assert_eq!(latex_live_preview_check(""), Ok(None));
        assert_eq!(latex_live_preview_check("   \n "), Ok(None));
        assert_eq!(latex_live_preview_check("x^2"), Ok(Some(3)));
        assert!(latex_live_preview_check("x\0y").is_err());
        let largo = "x".repeat(grafito_tex::TEX_INPUT_MAX_BYTES + 1);
        let err = latex_live_preview_check(&largo).expect_err("sobre 8 KiB");
        assert!(err.contains("máximo"), "{err}");
        let justo = "x".repeat(grafito_tex::TEX_INPUT_MAX_BYTES);
        assert!(latex_live_preview_check(&justo).is_ok());
    }

    #[test]
    fn stubs_l_siempre_error_honesto() {
        assert!(sonify_unavailable().is_err());
        assert!(braille_unavailable().is_err());
    }
}
