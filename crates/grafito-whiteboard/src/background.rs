//! Fondos de página de la pizarra (biblioteca de recursos, slice propio).
//!
//! El fondo NO es un [`crate::WhiteboardElement`]: es preferencia de vista
//! de la página, como el zoom — la sesión lo dibuja bajo el contenido y
//! jamás entra al documento, al JSON ni al SVG. La persistencia del
//! documento (`grafito-core::WhiteboardPageData`) no guarda vista de fondo,
//! así que al reabrir la hoja el fondo vuelve al default
//! ([`WhiteboardBackground::Grid`]): conducta documentada, no bug.
//!
//! Todo vectorial generado en código (líneas/puntos calculados): cero
//! binarios, cero imágenes, cero dependencias nuevas (solo `serde`+`log`
//! del crate).

use serde::{Deserialize, Serialize};

/// Paso de la cuadrícula en px de pantalla @zoom 1: el grid histórico de la
/// pizarra (`whiteboard_ui.rs::draw_grid`), siempre visible hasta hoy.
/// Cambiarlo mueve TODA la tinta relativa a la vista: pineado a propósito.
pub const GRID_STEP_PX: f64 = 40.0;
/// Paso del rayado (líneas horizontales, cuaderno): más estrecho que la
/// cuadrícula, múltiplo de 8 para alinear con el snap táctil.
pub const RULED_STEP_PX: f64 = 32.0;
/// Paso de la retícula de puntos (ambos ejes).
pub const DOTTED_STEP_PX: f64 = 24.0;
/// Paso mínimo en px para dibujar (igual que el early-return del grid:
/// con menos de ~6 px las líneas se empastan y solo gastan painter).
pub const MIN_STEP_PX: f64 = 6.0;
/// Cota de líneas por eje (anti-DoS si la vista degenera a gigante).
pub const MAX_LINES_PER_AXIS: usize = 4096;

/// Fondo de página (vista, no contenido). `Default` = el cuadriculado de
/// siempre: las hojas viejas y la sesión nueva se ven igual que antes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WhiteboardBackground {
    /// Cuadriculado 40 px (el grid histórico, default intacto).
    #[default]
    Grid,
    /// Rayado horizontal 32 px (cuaderno).
    Ruled,
    /// Punteado 24 px (retícula de puntos).
    Dotted,
    /// Blanco liso (solo el color del canvas).
    Plain,
}

impl WhiteboardBackground {
    /// Las 4 opciones del selector "Fondo", en orden de la toolbar.
    pub const ALL: [Self; 4] = [Self::Grid, Self::Ruled, Self::Dotted, Self::Plain];

    /// Etiqueta en español para la toolbar (como el resto de la pizarra:
    /// hardcoded, sin claves i18n nuevas, 6 locales intactos).
    pub fn label(self) -> &'static str {
        match self {
            Self::Grid => "Cuadriculado",
            Self::Ruled => "Rayado",
            Self::Dotted => "Punteado",
            Self::Plain => "Blanco",
        }
    }

    /// Paso en px de pantalla @zoom 1 (`None` = sin marcas: blanco liso).
    pub fn step_px(self) -> Option<f64> {
        match self {
            Self::Grid => Some(GRID_STEP_PX),
            Self::Ruled => Some(RULED_STEP_PX),
            Self::Dotted => Some(DOTTED_STEP_PX),
            Self::Plain => None,
        }
    }

    /// ¿Dibuja líneas verticales? (el rayado solo raya horizontal).
    pub fn has_vertical(self) -> bool {
        match self {
            Self::Grid | Self::Dotted => true,
            Self::Ruled | Self::Plain => false,
        }
    }

    /// Siguiente opción (el botón "Fondo" cicla en este orden).
    pub fn next(self) -> Self {
        match self {
            Self::Grid => Self::Ruled,
            Self::Ruled => Self::Dotted,
            Self::Dotted => Self::Plain,
            Self::Plain => Self::Grid,
        }
    }
}

/// Posiciones (px, en el mismo espacio que `min`/`max`/`origin`) de las
/// líneas de un eje, con una línea pasando por `origin` (el origen del
/// contenido: centro + pan, como `draw_grid`). Pura y sin panic: entradas
/// no finitas, rango degenerado o paso bajo [`MIN_STEP_PX`] dan `vec![]`.
/// Acotada a [`MAX_LINES_PER_AXIS`] (fail-closed, sin truncar a ciegas:
/// si la vista pide más, no se dibuja el eje antes que colgar el painter).
pub fn axis_lines(min: f64, max: f64, origin: f64, step: f64) -> Vec<f64> {
    if !min.is_finite() || !max.is_finite() || !origin.is_finite() || !step.is_finite() {
        return Vec::new();
    }
    // (Post-finitud: `>=` es total y no dispara `neg_cmp_op_on_partial_ord`.)
    if min >= max || step < MIN_STEP_PX {
        return Vec::new();
    }
    let span = max - min;
    if !span.is_finite() || span / step > MAX_LINES_PER_AXIS as f64 {
        return Vec::new();
    }
    let mut first = origin - ((origin - min) / step).floor() * step;
    // `floor` con negativos puede dejar `first` apenas bajo `min` por
    // redondeo: se normaliza al rango en vez de emitir fuera de vista.
    if first < min {
        first += step;
    }
    let mut out = Vec::new();
    let mut pos = first;
    while pos < max {
        out.push(pos);
        pos += step;
        if out.len() >= MAX_LINES_PER_AXIS {
            break;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn background_default_is_the_historic_grid() {
        // Default intacto: hojas viejas y sesión nueva se ven igual.
        assert_eq!(WhiteboardBackground::default(), WhiteboardBackground::Grid);
        assert_eq!(WhiteboardBackground::ALL.len(), 4);
        assert_eq!(GRID_STEP_PX, 40.0);
        assert_eq!(RULED_STEP_PX, 32.0);
        assert_eq!(DOTTED_STEP_PX, 24.0);
        assert_eq!(WhiteboardBackground::Grid.label(), "Cuadriculado");
        assert_eq!(WhiteboardBackground::Ruled.label(), "Rayado");
        assert_eq!(WhiteboardBackground::Dotted.label(), "Punteado");
        assert_eq!(WhiteboardBackground::Plain.label(), "Blanco");
        assert_eq!(WhiteboardBackground::Grid.step_px(), Some(40.0));
        assert_eq!(WhiteboardBackground::Plain.step_px(), None);
        assert!(WhiteboardBackground::Grid.has_vertical());
        assert!(!WhiteboardBackground::Ruled.has_vertical());
        assert!(!WhiteboardBackground::Plain.has_vertical());
        // El botón cicla las 4 y vuelve.
        let mut bg = WhiteboardBackground::Grid;
        for _ in 0..4 {
            bg = bg.next();
        }
        assert_eq!(bg, WhiteboardBackground::Grid);
    }

    #[test]
    fn axis_lines_are_pinned_and_never_panic() {
        // Caso pineado: origen 0, paso 40 → líneas exactas.
        assert_eq!(axis_lines(0.0, 100.0, 0.0, 40.0), vec![0.0, 40.0, 80.0]);
        // Origen desplazado 50: la grilla lo sigue (50-40=10).
        assert_eq!(axis_lines(0.0, 100.0, 50.0, 40.0), vec![10.0, 50.0, 90.0]);
        // Origen negativo: floor(-(-150)/40)=floor(3.75)=3 → -150+120=-30…
        // (-150-0)/40 = -3.75, floor = -4 → first = -150+160 = 10.
        assert_eq!(axis_lines(0.0, 100.0, -150.0, 40.0), vec![10.0, 50.0, 90.0]);
        // Rango que no llega al paso: vacío honesto, no línea suelta.
        assert!(axis_lines(0.0, 10.0, 50.0, 40.0).is_empty());
        // Degenerados: vacíos sin panic (fondos dibujan sin panic).
        for (min, max, origin, step) in [
            (f64::NAN, 100.0, 0.0, 40.0),
            (0.0, f64::INFINITY, 0.0, 40.0),
            (0.0, 100.0, f64::NAN, 40.0),
            (0.0, 100.0, 0.0, f64::INFINITY),
            (100.0, 0.0, 0.0, 40.0),
            (0.0, 0.0, 0.0, 40.0),
            (0.0, 100.0, 0.0, 5.9),
            (0.0, 100.0, 0.0, 0.0),
            (0.0, 100.0, 0.0, -40.0),
            (0.0, 1e12, 0.0, 40.0),
        ] {
            assert!(
                axis_lines(min, max, origin, step).is_empty(),
                "degenerado sin panic: ({min}, {max}, {origin}, {step})"
            );
        }
        // Vista gigante pero finita: acotada, sin OOM.
        assert!(axis_lines(0.0, 1e9, 0.0, 40.0).is_empty());
        // Todas las posiciones caen dentro del rango pedido.
        let lines = axis_lines(7.5, 333.0, 200.0, 32.0);
        assert!(!lines.is_empty());
        assert!(lines.iter().all(|pos| *pos >= 7.5 && *pos < 333.0));
        // Paso exacto entre consecutivas.
        for pair in lines.windows(2) {
            assert!((pair[1] - pair[0] - 32.0).abs() < 1e-9);
        }
    }

    #[test]
    fn dotted_grid_positions_come_from_both_axes() {
        // El punteado es el cartesiano de ambos ejes: 3×3 en 100×100 @24.
        let xs = axis_lines(0.0, 100.0, 0.0, DOTTED_STEP_PX);
        let ys = axis_lines(0.0, 100.0, 0.0, DOTTED_STEP_PX);
        assert_eq!(xs, vec![0.0, 24.0, 48.0, 72.0, 96.0]);
        assert_eq!(ys, xs);
        assert_eq!(xs.len() * ys.len(), 25);
    }
}
