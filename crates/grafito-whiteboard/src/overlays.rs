//! Cortina y lupa estilo OpenBoard para la pizarra.
//!
//! Modelo puro y headless (sin egui): la sesión (`grafito-app`) los dibuja y
//! les da entrada, pero toda la geometría vive aquí y se testea sin UI.
//!
//! Decisión honesta (mismo patrón que `instruments.rs`): cortina y lupa NO
//! son `WhiteboardElement`. Son overlays de sesión manipulables y jamás
//! entran al documento: sin `Serialize`, sin SVG ni export. La toolbar les
//! da un slot único (un overlay visible a la vez, como los instrumentos).
//!
//! - Cortina (`WhiteboardCurtain`): rect que cubre la hoja; se arrastra su
//!   borde libre para revelar progresivo, con dirección configurable
//!   arriba/abajo. Geometría en fracciones de la vista (0..=1 desde arriba)
//!   para no depender del pan/zoom de sesión.
//! - Lupa: factor 2x honesto como zoom-to-point sobre el zoom de sesión
//!   (re-dibujar la escena ampliada con el painter actual no sale limpio:
//!   segundo pass con otro transform para trazos, imágenes cacheadas y
//!   texto divergería del canvas/SVG; el zoom centrado reutiliza `zoom_at`,
//!   ya clampado y testeado). [`MAGNIFIER_FACTOR`] y [`magnifier_zoom`]
//!   pinean el factor y el clamp para la sesión y los tests.

/// Factor de la lupa (2x, estilo OpenBoard).
pub const MAGNIFIER_FACTOR: f64 = 2.0;
/// Tolerancia de agarre del borde libre de la cortina (px de pantalla).
pub const CURTAIN_EDGE_TOL_PX: f64 = 10.0;

/// Dirección de la cortina: desde qué lado cuelga (el borde opuesto es el
/// libre y se arrastra para revelar).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CurtainDirection {
    /// Cuelga de arriba: cubre `[0, edge]`, se revela de abajo hacia arriba.
    #[default]
    Top,
    /// Sube de abajo: cubre `[edge, 1]`, se revela de arriba hacia abajo.
    Bottom,
}

/// Cortina sobre la hoja (overlay, no figura).
///
/// - `edge`: posición del borde libre en fracción de la vista (0 = arriba
///   del todo, 1 = abajo del todo), siempre 0..=1 tras `covered`/`set_edge`
///   /`drag_by` (construcciones literales fuera de rango se defensan en
///   lectura vía `is_usable` + clamp en `visible_fraction`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WhiteboardCurtain {
    pub direction: CurtainDirection,
    pub edge: f64,
}

impl WhiteboardCurtain {
    /// Cortina recién mostrada: cubre toda la hoja.
    pub fn covered(direction: CurtainDirection) -> Self {
        Self {
            direction,
            edge: match direction {
                CurtainDirection::Top => 1.0,
                CurtainDirection::Bottom => 0.0,
            },
        }
    }

    /// ¿Borde finito y en rango? (degenerados → sin agarre ni dibujo).
    pub fn is_usable(&self) -> bool {
        self.edge.is_finite() && (0.0..=1.0).contains(&self.edge)
    }

    /// Fracción visible de la hoja (área revelada pura, 0..=1): con `Top`,
    /// `1 - edge`; con `Bottom`, `edge`. No finitos → 0.0 (cubierta).
    pub fn visible_fraction(&self) -> f64 {
        if !self.edge.is_finite() {
            return 0.0;
        }
        let edge = self.edge.clamp(0.0, 1.0);
        match self.direction {
            CurtainDirection::Top => 1.0 - edge,
            CurtainDirection::Bottom => edge,
        }
    }

    /// Fracción cubierta (`1 - visible`), para chequear el espejo al
    /// cambiar de dirección.
    pub fn covered_fraction(&self) -> f64 {
        1.0 - self.visible_fraction()
    }

    /// Fija el borde en fracción de vista: no finitos se ignoran, el resto
    /// se clampa a 0..=1. Si el estado era inválido, parte de `covered`.
    pub fn set_edge(&mut self, edge: f64) {
        if !edge.is_finite() {
            return;
        }
        self.edge = edge.clamp(0.0, 1.0);
    }

    /// Arrastra el borde por delta en fracción de vista (positivo = hacia
    /// abajo): no finitos se ignoran, el resto se clampa a 0..=1.
    pub fn drag_by(&mut self, delta: f64) {
        if !delta.is_finite() {
            return;
        }
        let base = if self.edge.is_finite() {
            self.edge
        } else {
            Self::covered(self.direction).edge
        };
        self.edge = (base + delta).clamp(0.0, 1.0);
    }

    /// Cambia arriba/abajo espejando el borde (`edge = 1 - edge`) para
    /// conservar el área cubierta. Estado inválido → `covered` del nuevo lado.
    pub fn flip_direction(&mut self) {
        let next = match self.direction {
            CurtainDirection::Top => CurtainDirection::Bottom,
            CurtainDirection::Bottom => CurtainDirection::Top,
        };
        self.direction = next;
        self.edge = if self.edge.is_finite() {
            (1.0 - self.edge).clamp(0.0, 1.0)
        } else {
            Self::covered(next).edge
        };
    }

    /// ¿`screen_y` agarra el borde libre? (px de pantalla, tolerancia
    /// [`CURTAIN_EDGE_TOL_PX`] del llamador). Sin panic: vista degenerada o
    /// valores no finitos dan `false`.
    pub fn edge_hit(&self, screen_y: f64, view_top: f64, view_height: f64, tol_px: f64) -> bool {
        if !self.is_usable()
            || !screen_y.is_finite()
            || !view_top.is_finite()
            || !view_height.is_finite()
            || view_height <= 0.0
            || !tol_px.is_finite()
            || tol_px < 0.0
        {
            return false;
        }
        let edge_y = view_top + self.edge * view_height;
        (screen_y - edge_y).abs() <= tol_px
    }

    /// ¿`screen_y` cae en la zona cubierta? (para el clic que cambia la
    /// dirección). Mismas guardas que [`edge_hit`](Self::edge_hit).
    pub fn covers_y(&self, screen_y: f64, view_top: f64, view_height: f64) -> bool {
        if !self.is_usable()
            || !screen_y.is_finite()
            || !view_top.is_finite()
            || !view_height.is_finite()
            || view_height <= 0.0
        {
            return false;
        }
        let edge_y = view_top + self.edge * view_height;
        match self.direction {
            CurtainDirection::Top => screen_y <= edge_y,
            CurtainDirection::Bottom => screen_y >= edge_y,
        }
    }
}

/// Slot único cortina (un overlay visible a la vez, como `toggle_slot` de
/// instrumentos): `None` la muestra, misma dirección la oculta, otra
/// dirección la reemplaza cubierta.
pub fn toggle_curtain_slot(
    current: &Option<WhiteboardCurtain>,
    direction: CurtainDirection,
) -> Option<WhiteboardCurtain> {
    match current {
        Some(active) if active.direction == direction => None,
        _ => Some(WhiteboardCurtain::covered(direction)),
    }
}

/// Zoom resultante de la lupa (factor [`MAGNIFIER_FACTOR`] con clamp
/// honesto a `[min, max]`): lo usa la sesión vía `zoom_at`, que ya clampa
/// a `ZOOM_WB_MIN..=ZOOM_WB_MAX`. Cotas no finitas o invertidas → no-op
/// (devuelve `current` sin tocar nada).
pub fn magnifier_zoom(current: f64, min: f64, max: f64) -> f64 {
    if !current.is_finite() || !min.is_finite() || !max.is_finite() || min > max {
        return current;
    }
    (current * MAGNIFIER_FACTOR).clamp(min, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curtain_pins_factor_and_edge_tolerance() {
        // Si cambian, cambian la sesión (`whiteboard_ui.rs`) a la par.
        assert_eq!(MAGNIFIER_FACTOR, 2.0);
        assert_eq!(CURTAIN_EDGE_TOL_PX, 10.0);
    }

    #[test]
    fn curtain_covers_fully_on_show_and_reveals_pure_area() {
        // Al mostrar cubre todo (visible 0); arrastrar revela progresivo.
        let mut curtain = WhiteboardCurtain::covered(CurtainDirection::Top);
        assert_eq!(curtain.visible_fraction(), 0.0);
        curtain.drag_by(-0.5);
        assert!(
            (curtain.visible_fraction() - 0.5).abs() < 1e-12,
            "mitad revelada, fue: {}",
            curtain.visible_fraction()
        );
        // Más allá del borde se clampa a revelado total, sin pasarse de 1.
        curtain.drag_by(-10.0);
        assert_eq!(curtain.visible_fraction(), 1.0);
        curtain.drag_by(10.0);
        assert_eq!(curtain.visible_fraction(), 0.0);
        // Deltas no finitos se ignoran sin panic.
        curtain.drag_by(f64::NAN);
        assert_eq!(curtain.visible_fraction(), 0.0);
        curtain.set_edge(f64::INFINITY);
        assert_eq!(curtain.visible_fraction(), 0.0);
        // Bottom espeja: mismo borde numérico, área complementaria.
        let bottom = WhiteboardCurtain {
            direction: CurtainDirection::Bottom,
            edge: 0.5,
        };
        assert_eq!(bottom.visible_fraction(), 0.5);
        assert_eq!(
            WhiteboardCurtain::covered(CurtainDirection::Bottom).visible_fraction(),
            0.0
        );
    }

    #[test]
    fn curtain_flip_keeps_covered_area() {
        // Cambiar arriba/abajo conserva el área cubierta (espejo del borde).
        let mut curtain = WhiteboardCurtain::covered(CurtainDirection::Top);
        curtain.drag_by(-0.25);
        assert!((curtain.covered_fraction() - 0.75).abs() < 1e-12);
        curtain.flip_direction();
        assert_eq!(curtain.direction, CurtainDirection::Bottom);
        assert!(
            (curtain.covered_fraction() - 0.75).abs() < 1e-12,
            "cubierta intacta, fue: {}",
            curtain.covered_fraction()
        );
        assert!((curtain.edge - 0.25).abs() < 1e-12);
    }

    #[test]
    fn curtain_edge_hit_uses_screen_pixels() {
        // Borde a mitad de un lienzo de 300 px: ±10 px agarra, lejos no.
        let curtain = WhiteboardCurtain {
            direction: CurtainDirection::Top,
            edge: 0.5,
        };
        assert!(curtain.edge_hit(150.0, 0.0, 300.0, CURTAIN_EDGE_TOL_PX));
        assert!(curtain.edge_hit(141.0, 0.0, 300.0, CURTAIN_EDGE_TOL_PX));
        assert!(!curtain.edge_hit(200.0, 0.0, 300.0, CURTAIN_EDGE_TOL_PX));
        // Guardas sin panic: NaN, alto nulo, tolerancia negativa.
        assert!(!curtain.edge_hit(f64::NAN, 0.0, 300.0, CURTAIN_EDGE_TOL_PX));
        assert!(!curtain.edge_hit(150.0, 0.0, 0.0, CURTAIN_EDGE_TOL_PX));
        assert!(!curtain.edge_hit(150.0, 0.0, 300.0, -1.0));
        // Zona cubierta: Top cubre [0, edge].
        assert!(curtain.covers_y(100.0, 0.0, 300.0));
        assert!(!curtain.covers_y(200.0, 0.0, 300.0));
        let bottom = WhiteboardCurtain {
            direction: CurtainDirection::Bottom,
            edge: 0.5,
        };
        assert!(!bottom.covers_y(100.0, 0.0, 300.0));
        assert!(bottom.covers_y(200.0, 0.0, 300.0));
    }

    #[test]
    fn magnifier_doubles_and_clamps_at_bounds() {
        // 2x exacto en rango; en los bordes se clampa sin pasarse.
        assert_eq!(magnifier_zoom(1.0, 1e-6, 1e6), 2.0);
        assert_eq!(magnifier_zoom(6e5, 1e-6, 1e6), 1e6);
        assert_eq!(magnifier_zoom(5e-7, 1e-6, 1e6), 1e-6);
        // Cotas rotas → no-op honesto, sin panic.
        assert!(magnifier_zoom(f64::NAN, 1e-6, 1e6).is_nan());
        assert_eq!(magnifier_zoom(2.0, 10.0, 1.0), 2.0);
        assert_eq!(magnifier_zoom(2.0, f64::NAN, 1e6), 2.0);
    }

    #[test]
    fn curtain_slot_shows_hides_and_replaces() {
        // Slot único: mostrar, ocultar con segundo clic, reemplazar dirección.
        use CurtainDirection::{Bottom, Top};
        assert_eq!(
            toggle_curtain_slot(&None, Top),
            Some(WhiteboardCurtain::covered(Top))
        );
        assert_eq!(
            toggle_curtain_slot(&Some(WhiteboardCurtain::covered(Top)), Top),
            None
        );
        assert_eq!(
            toggle_curtain_slot(&Some(WhiteboardCurtain::covered(Top)), Bottom),
            Some(WhiteboardCurtain::covered(Bottom))
        );
    }
}
