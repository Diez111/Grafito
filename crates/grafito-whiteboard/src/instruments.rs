//! Instrumentos de pizarra estilo OpenBoard: regla, escuadra (triángulo),
//! transportador y compás.
//!
//! Modelo puro y headless (sin egui): la sesión (`grafito-app`) los dibuja y
//! les da entrada, pero toda la geometría —arrastrar, rotar con manija,
//! pegado del lápiz al borde y círculo del compás— vive aquí y se testea sin
//! UI.
//!
//! Decisión honesta: los instrumentos NO son `WhiteboardElement`. Son
//! overlays manipulables (se arrastran, se rotan con manija, no se
//! seleccionan como figura) y jamás entran al documento: no serializan (sin
//! `Serialize`), no van al SVG ni al export. El compás no crea elemento
//! nuevo: dos clics (centro + radio) devuelven un `Ellipse` círculo exacto
//! reutilizando la variante existente.
//!
//! Referencia: OpenBoard (`UBGraphicsRuler`/`UBGraphicsTriangle`/
//! `UBGraphicsProtractor`/`UBGraphicsCompass`: objetos sobre el canvas que
//! se arrastran, se rotan con manija y pegan el trazo al borde).

use crate::WhiteboardElement;

/// Tolerancia de pegado del lápiz/resaltador al borde del instrumento
/// (unidades mundo @zoom 1; la grilla es 40 px): dentro → proyección al
/// segmento más cercano; fuera → trazo libre honesto, sin forzar nada.
pub const INSTRUMENT_SNAP_TOLERANCE: f64 = 12.0;
/// Largo por defecto de la regla (mundo @zoom 1).
pub const RULER_DEFAULT_LEN: f64 = 300.0;
/// Cateto por defecto de la escuadra (triángulo rectángulo isósceles).
pub const TRIANGLE_DEFAULT_LEG: f64 = 220.0;
/// Radio por defecto del transportador (medio disco).
pub const PROTRACTOR_DEFAULT_RADIUS: f64 = 120.0;
/// Separación de la manija de rotación respecto al borde del instrumento.
pub const INSTRUMENT_HANDLE_GAP: f64 = 18.0;
/// Tolerancia de agarre del cuerpo para arrastrar (mundo @zoom 1).
pub const INSTRUMENT_BODY_TOLERANCE: f64 = 10.0;
/// Tolerancia de agarre de la manija para rotar (mundo @zoom 1).
pub const INSTRUMENT_HANDLE_TOLERANCE: f64 = 14.0;
/// Segmentos con los que se aproxima el arco del transportador.
pub const PROTRACTOR_ARC_STEPS: usize = 24;
/// Radio mínimo del círculo del compás (clic doble en el mismo punto → nada).
pub const COMPASS_MIN_RADIUS: f64 = 1.0;
/// Tamaño mínimo de un instrumento (evita degenerados invisibles).
pub const INSTRUMENT_MIN_SIZE: f64 = 1.0;

/// Clase de instrumento (para el slot único de la toolbar: un instrumento
/// visible a la vez; clic de nuevo lo guarda).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstrumentKind {
    Ruler,
    Triangle,
    Protractor,
}

/// Instrumento manipulable sobre el canvas (overlay, no figura).
///
/// - `pos`: centro de la regla / vértice recto de la escuadra / centro del
///   transportador (mundo).
/// - `angle`: giro en radianes.
/// - `len`/`leg`/`radius`: tamaño en unidades mundo.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WhiteboardInstrument {
    Ruler {
        pos: (f64, f64),
        angle: f64,
        len: f64,
    },
    Triangle {
        pos: (f64, f64),
        angle: f64,
        leg: f64,
    },
    Protractor {
        pos: (f64, f64),
        angle: f64,
        radius: f64,
    },
}

impl WhiteboardInstrument {
    /// Regla por defecto centrada en el origen, sin girar.
    pub fn ruler_default() -> Self {
        Self::Ruler {
            pos: (0.0, 0.0),
            angle: 0.0,
            len: RULER_DEFAULT_LEN,
        }
    }

    /// Escuadra por defecto centrada en el origen, sin girar.
    pub fn triangle_default() -> Self {
        Self::Triangle {
            pos: (0.0, 0.0),
            angle: 0.0,
            leg: TRIANGLE_DEFAULT_LEG,
        }
    }

    /// Transportador por defecto centrado en el origen, sin girar.
    pub fn protractor_default() -> Self {
        Self::Protractor {
            pos: (0.0, 0.0),
            angle: 0.0,
            radius: PROTRACTOR_DEFAULT_RADIUS,
        }
    }

    /// Clase del instrumento (para el slot único).
    pub fn kind(&self) -> InstrumentKind {
        match self {
            Self::Ruler { .. } => InstrumentKind::Ruler,
            Self::Triangle { .. } => InstrumentKind::Triangle,
            Self::Protractor { .. } => InstrumentKind::Protractor,
        }
    }

    /// Posición de referencia del instrumento (mundo).
    pub fn pos(&self) -> (f64, f64) {
        match *self {
            Self::Ruler { pos, .. } | Self::Triangle { pos, .. } | Self::Protractor { pos, .. } => {
                pos
            }
        }
    }

    /// Giro en radianes.
    pub fn angle(&self) -> f64 {
        match *self {
            Self::Ruler { angle, .. }
            | Self::Triangle { angle, .. }
            | Self::Protractor { angle, .. } => angle,
        }
    }

    /// Tamaño característico (largo / cateto / radio).
    pub fn size(&self) -> f64 {
        match *self {
            Self::Ruler { len, .. } => len,
            Self::Triangle { leg, .. } => leg,
            Self::Protractor { radius, .. } => radius,
        }
    }

    /// ¿Tamaño finito y útil? (degenerados → sin agarre ni bordes).
    pub fn is_usable(&self) -> bool {
        let (x, y) = self.pos();
        x.is_finite()
            && y.is_finite()
            && self.angle().is_finite()
            && self.size().is_finite()
            && self.size() >= INSTRUMENT_MIN_SIZE
    }

    /// Dirección unitaria del eje principal.
    fn dir(&self) -> (f64, f64) {
        (self.angle().cos(), self.angle().sin())
    }

    /// Desplaza el instrumento (arrastre del cuerpo): traslación pura, el
    /// giro no cambia. Sin panic: deltas no finitos se ignoran.
    pub fn move_by(&mut self, delta: (f64, f64)) {
        if !delta.0.is_finite() || !delta.1.is_finite() {
            return;
        }
        let (x, y) = self.pos();
        let pos = (x + delta.0, y + delta.1);
        match self {
            Self::Ruler { pos: slot, .. }
            | Self::Triangle { pos: slot, .. }
            | Self::Protractor { pos: slot, .. } => *slot = pos,
        }
    }

    /// Posición de la manija de rotación (mundo): extremo de la regla +
    /// [`INSTRUMENT_HANDLE_GAP`], punto medio de la hipotenusa desplazado
    /// hacia fuera en la escuadra, coronilla del arco en el transportador.
    /// `None` si el instrumento es degenerado.
    pub fn handle_pos(&self) -> Option<(f64, f64)> {
        if !self.is_usable() {
            return None;
        }
        let (x, y) = self.pos();
        let (dx, dy) = self.dir();
        let (nx, ny) = (-dy, dx);
        match *self {
            Self::Ruler { len, .. } => {
                let half = len / 2.0 + INSTRUMENT_HANDLE_GAP;
                Some((x + dx * half, y + dy * half))
            }
            Self::Triangle { leg, .. } => {
                // Punto medio de la hipotenusa + normal exterior.
                let mid = (x + (dx + nx) * leg / 2.0, y + (dy + ny) * leg / 2.0);
                let (ox, oy) = (mid.0 - x, mid.1 - y);
                let norm = (ox * ox + oy * oy).sqrt().max(1e-9);
                Some((
                    mid.0 + ox / norm * INSTRUMENT_HANDLE_GAP,
                    mid.1 + oy / norm * INSTRUMENT_HANDLE_GAP,
                ))
            }
            Self::Protractor { radius, .. } => Some((
                x + nx * (radius + INSTRUMENT_HANDLE_GAP),
                y + ny * (radius + INSTRUMENT_HANDLE_GAP),
            )),
        }
    }

    /// Gira el instrumento para que la manija apunte al cursor (rotación con
    /// manija estilo OpenBoard). Traslación nula: solo cambia el giro. Sin
    /// panic: cursor no finito o sobre el centro se ignora.
    pub fn rotate_to(&mut self, cursor: (f64, f64)) {
        if !cursor.0.is_finite() || !cursor.1.is_finite() || !self.is_usable() {
            return;
        }
        let (x, y) = self.pos();
        let (vx, vy) = (cursor.0 - x, cursor.1 - y);
        if vx * vx + vy * vy < 1e-12 {
            return;
        }
        // La manija nace desplazada del eje según la pieza: regla 0,
        // escuadra 45° (hipotenusa), transportador 90° (coronilla).
        let offset = match self {
            Self::Ruler { .. } => 0.0,
            Self::Triangle { .. } => std::f64::consts::FRAC_PI_4,
            Self::Protractor { .. } => std::f64::consts::FRAC_PI_2,
        };
        let angle = vy.atan2(vx) - offset;
        if !angle.is_finite() {
            return;
        }
        match self {
            Self::Ruler { angle: slot, .. }
            | Self::Triangle { angle: slot, .. }
            | Self::Protractor { angle: slot, .. } => *slot = angle,
        }
    }

    /// Bordes donde el lápiz se pega (segmentos en mundo): un filo en la
    /// regla; los tres lados en la escuadra (mismo camino que la regla);
    /// base + arco (24 tramos) en el transportador. Vacío si degenerado.
    pub fn edge_segments(&self) -> Vec<((f64, f64), (f64, f64))> {
        if !self.is_usable() {
            return Vec::new();
        }
        let (x, y) = self.pos();
        let (dx, dy) = self.dir();
        let (nx, ny) = (-dy, dx);
        match *self {
            Self::Ruler { len, .. } => {
                let half = len / 2.0;
                vec![(
                    (x - dx * half, y - dy * half),
                    (x + dx * half, y + dy * half),
                )]
            }
            Self::Triangle { leg, .. } => {
                let corner = (x, y);
                let end_a = (x + dx * leg, y + dy * leg);
                let end_b = (x + nx * leg, y + ny * leg);
                vec![(corner, end_a), (corner, end_b), (end_a, end_b)]
            }
            Self::Protractor { radius, .. } => {
                let mut out = Vec::with_capacity(PROTRACTOR_ARC_STEPS + 1);
                out.push((
                    (x - dx * radius, y - dy * radius),
                    (x + dx * radius, y + dy * radius),
                ));
                let mut prev = (x + dx * radius, y + dy * radius);
                for step in 1..=PROTRACTOR_ARC_STEPS {
                    let theta = std::f64::consts::PI * step as f64 / PROTRACTOR_ARC_STEPS as f64;
                    let (c, s) = (theta.cos(), theta.sin());
                    let point = (
                        x + dx * radius * c + nx * radius * s,
                        y + dy * radius * c + ny * radius * s,
                    );
                    out.push((prev, point));
                    prev = point;
                }
                out
            }
        }
    }

    /// ¿El punto agarra el cuerpo (para arrastrar)?
    pub fn body_hit(&self, point: (f64, f64)) -> bool {
        if !point.0.is_finite() || !point.1.is_finite() || !self.is_usable() {
            return false;
        }
        self.edge_segments()
            .iter()
            .any(|(a, b)| project_to_segment(*a, *b, point).1 <= INSTRUMENT_BODY_TOLERANCE)
    }

    /// ¿El punto agarra la manija (para rotar)? (prioridad sobre el cuerpo).
    pub fn handle_hit(&self, point: (f64, f64)) -> bool {
        if !point.0.is_finite() || !point.1.is_finite() {
            return false;
        }
        match self.handle_pos() {
            Some(handle) => {
                ((handle.0 - point.0).powi(2) + (handle.1 - point.1).powi(2)).sqrt()
                    <= INSTRUMENT_HANDLE_TOLERANCE
            }
            None => false,
        }
    }

    /// Inicia un arrastre: manija primero (rotar), luego cuerpo (mover).
    pub fn begin_drag(&self, point: (f64, f64)) -> Option<InstrumentDrag> {
        if self.handle_hit(point) {
            return Some(InstrumentDrag::Rotate);
        }
        if self.body_hit(point) {
            let (x, y) = self.pos();
            return Some(InstrumentDrag::MoveBody {
                offset: (x - point.0, y - point.1),
            });
        }
        None
    }

    /// Aplica un arrastre en curso al cursor actual (puro salvo `&mut`).
    pub fn apply_drag(&mut self, drag: InstrumentDrag, cursor: (f64, f64)) {
        if !cursor.0.is_finite() || !cursor.1.is_finite() {
            return;
        }
        match drag {
            InstrumentDrag::MoveBody { offset } => {
                if offset.0.is_finite() && offset.1.is_finite() {
                    let (x, y) = (cursor.0 + offset.0, cursor.1 + offset.1);
                    if x.is_finite() && y.is_finite() {
                        match self {
                            Self::Ruler { pos, .. }
                            | Self::Triangle { pos, .. }
                            | Self::Protractor { pos, .. } => *pos = (x, y),
                        }
                    }
                }
            }
            InstrumentDrag::Rotate => self.rotate_to(cursor),
        }
    }

    /// Pega un punto del lápiz/resaltador al borde: proyección al segmento
    /// más cercano si cae dentro de `tolerance`; `None` (trazo libre) si el
    /// punto está lejos o algo es degenerado. Nunca hace panic.
    pub fn snap_point(&self, point: (f64, f64), tolerance: f64) -> Option<(f64, f64)> {
        if !point.0.is_finite() || !point.1.is_finite() || !tolerance.is_finite() || tolerance < 0.0
        {
            return None;
        }
        let mut best: Option<((f64, f64), f64)> = None;
        for (a, b) in self.edge_segments() {
            let (proj, dist) = project_to_segment(a, b, point);
            let closer = best.map(|(_, d)| dist < d).unwrap_or(true);
            if closer {
                best = Some((proj, dist));
            }
        }
        match best {
            Some((proj, dist)) if dist <= tolerance => Some(proj),
            _ => None,
        }
    }
}

/// Arrastre en curso sobre un instrumento: mover el cuerpo o rotar con manija.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InstrumentDrag {
    /// Arrastrar el cuerpo (`offset` = pos − punto de agarre, mundo).
    MoveBody { offset: (f64, f64) },
    /// Rotar con la manija.
    Rotate,
}

/// Proyección de `p` sobre el segmento `a–b` + distancia. Sin panic:
/// segmento degenerado → extremo `a`.
fn project_to_segment(a: (f64, f64), b: (f64, f64), p: (f64, f64)) -> ((f64, f64), f64) {
    let dx = b.0 - a.0;
    let dy = b.1 - a.1;
    let len_sq = dx * dx + dy * dy;
    if !len_sq.is_finite() || len_sq <= 0.0 {
        let dist = ((p.0 - a.0).powi(2) + (p.1 - a.1).powi(2)).sqrt();
        return (a, dist);
    }
    let t = (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / len_sq).clamp(0.0, 1.0);
    let proj = (a.0 + t * dx, a.1 + t * dy);
    let dist = ((p.0 - proj.0).powi(2) + (p.1 - proj.1).powi(2)).sqrt();
    (proj, dist)
}

/// Compás estilo OpenBoard en dos clics: `center` + punto del radio → círculo
/// exacto reutilizando `Ellipse` (`rx == ry == distancia`, sin elemento
/// nuevo). `None` honesto si los puntos no son finitos o el radio es
/// diminuto (doble clic en el mismo punto).
pub fn compass_circle(center: (f64, f64), rim: (f64, f64)) -> Option<WhiteboardElement> {
    if !center.0.is_finite() || !center.1.is_finite() || !rim.0.is_finite() || !rim.1.is_finite() {
        return None;
    }
    let radius = ((rim.0 - center.0).powi(2) + (rim.1 - center.1).powi(2)).sqrt();
    if !radius.is_finite() || radius < COMPASS_MIN_RADIUS {
        return None;
    }
    Some(WhiteboardElement::Ellipse {
        center,
        rx: radius,
        ry: radius,
    })
}

/// Slot único de la toolbar: el instrumento nuevo reemplaza al visible; si es
/// de la misma clase, se guarda (`None`). Un instrumento visible a la vez.
pub fn toggle_slot(
    current: &Option<WhiteboardInstrument>,
    fresh: WhiteboardInstrument,
) -> Option<WhiteboardInstrument> {
    match current {
        Some(active) if active.kind() == fresh.kind() => None,
        _ => Some(fresh),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WhiteboardDoc;

    #[test]
    fn instrument_tolerances_and_defaults_are_pinned() {
        // Si cambian, cambian la sesión (`whiteboard_ui.rs`) a la par.
        assert_eq!(INSTRUMENT_SNAP_TOLERANCE, 12.0);
        assert_eq!(RULER_DEFAULT_LEN, 300.0);
        assert_eq!(TRIANGLE_DEFAULT_LEG, 220.0);
        assert_eq!(PROTRACTOR_DEFAULT_RADIUS, 120.0);
        assert_eq!(INSTRUMENT_HANDLE_GAP, 18.0);
        assert_eq!(INSTRUMENT_BODY_TOLERANCE, 10.0);
        assert_eq!(INSTRUMENT_HANDLE_TOLERANCE, 14.0);
        assert_eq!(PROTRACTOR_ARC_STEPS, 24);
        assert_eq!(COMPASS_MIN_RADIUS, 1.0);
    }

    #[test]
    fn ruler_drag_is_pure_translation() {
        // Arrastrar el cuerpo mueve sin girar (OpenBoard: el filo conserva el ángulo).
        let mut ruler = WhiteboardInstrument::Ruler {
            pos: (10.0, 20.0),
            angle: 0.4,
            len: 200.0,
        };
        let before = ruler.angle();
        ruler.move_by((15.0, -7.0));
        assert_eq!(ruler.pos(), (25.0, 13.0));
        assert_eq!(ruler.angle(), before, "arrastrar no rota");
        // Deltas no finitos se ignoran sin panic.
        ruler.move_by((f64::NAN, 0.0));
        assert_eq!(ruler.pos(), (25.0, 13.0));
    }

    #[test]
    fn ruler_rotate_tracks_handle_cursor() {
        // La manija sigue al cursor: cursor a 0.6 rad del centro → giro 0.6.
        let mut ruler = WhiteboardInstrument::Ruler {
            pos: (10.0, 20.0),
            angle: 0.0,
            len: 200.0,
        };
        let cursor = (10.0 + 50.0 * 0.6_f64.cos(), 20.0 + 50.0 * 0.6_f64.sin());
        ruler.rotate_to(cursor);
        assert!(
            (ruler.angle() - 0.6).abs() < 1e-9,
            "giro esperado 0.6, fue: {}",
            ruler.angle()
        );
        // Cursor sobre el centro: se ignora, no hay giro loco.
        ruler.rotate_to((10.0, 20.0));
        assert!((ruler.angle() - 0.6).abs() < 1e-9);
    }

    #[test]
    fn triangle_uses_the_same_drag_rotate_path() {
        // La escuadra sale con el mismo camino: arrastrar traslada sin girar,
        // la manija (hipotenusa, +45°) rota con el desplazamiento honesto.
        let mut triangle = WhiteboardInstrument::triangle_default();
        let before = triangle.angle();
        triangle.move_by((5.0, 5.0));
        assert_eq!(triangle.pos(), (5.0, 5.0));
        assert_eq!(triangle.angle(), before);
        // Tres lados como bordes de pegado (dos catetos + hipotenusa).
        assert_eq!(triangle.edge_segments().len(), 3);
        // Cursor a 45° del centro → giro 0 (desplazamiento de la manija).
        triangle = WhiteboardInstrument::Triangle {
            pos: (0.0, 0.0),
            angle: 0.0,
            leg: 200.0,
        };
        let quarter = std::f64::consts::FRAC_PI_4;
        triangle.rotate_to((100.0 * quarter.cos(), 100.0 * quarter.sin()));
        assert!(
            triangle.angle().abs() < 1e-9,
            "giro esperado 0, fue: {}",
            triangle.angle()
        );
        // Pegado en un cateto: punto sobre el eje X se queda en el filo.
        let snapped = triangle.snap_point((80.0, 4.0), INSTRUMENT_SNAP_TOLERANCE);
        match snapped {
            Some((x, y)) => {
                assert!((x - 80.0).abs() < 1e-9, "x pegada al filo: {x}");
                assert!(y.abs() < 1e-9, "y sobre el filo: {y}");
            }
            None => panic!("pegado esperado en el cateto"),
        }
    }

    #[test]
    fn protractor_edges_cover_base_and_arc() {
        // Base (diámetro) + 24 tramos de arco = 25 segmentos de pegado.
        let protractor = WhiteboardInstrument::protractor_default();
        assert_eq!(protractor.edge_segments().len(), PROTRACTOR_ARC_STEPS + 1);
        // La manija nace en la coronilla (+radio+gap sobre el centro).
        assert_eq!(
            protractor.handle_pos(),
            Some((0.0, PROTRACTOR_DEFAULT_RADIUS + INSTRUMENT_HANDLE_GAP))
        );
        // Cursor arriba del centro → giro 0 (desplazamiento de la manija).
        let mut rotated = protractor;
        rotated.rotate_to((0.0, 200.0));
        assert!(
            rotated.angle().abs() < 1e-9,
            "giro esperado 0, fue: {}",
            rotated.angle()
        );
        // Pegado en la base: punto bajo el centro sube al diámetro.
        match protractor.snap_point((40.0, -3.0), INSTRUMENT_SNAP_TOLERANCE) {
            Some((x, y)) => {
                assert!((x - 40.0).abs() < 1e-9, "x pegada a la base: {x}");
                assert!(y.abs() < 1e-9, "y sobre la base: {y}");
            }
            None => panic!("pegado esperado en la base"),
        }
    }

    #[test]
    fn snap_projects_point_onto_ruler_edge() {
        // Regla horizontal de 200 en el origen: (30, 5) → (30, 0).
        let ruler = WhiteboardInstrument::Ruler {
            pos: (0.0, 0.0),
            angle: 0.0,
            len: 200.0,
        };
        match ruler.snap_point((30.0, 5.0), INSTRUMENT_SNAP_TOLERANCE) {
            Some((x, y)) => {
                assert!((x - 30.0).abs() < 1e-9, "x pegada al filo: {x}");
                assert!(y.abs() < 1e-9, "y sobre el filo: {y}");
            }
            None => panic!("proyección ortogonal al filo esperada"),
        }
        // Más allá del extremo se pinza al filo (no se inventa recta).
        assert_eq!(
            ruler.snap_point((140.0, 0.0), INSTRUMENT_SNAP_TOLERANCE),
            None,
            "fuera del filo a 40 px: trazo libre"
        );
    }

    #[test]
    fn snap_far_point_stays_free() {
        // Fuera de tolerancia → None para que el llamador guarde el punto tal cual.
        let ruler = WhiteboardInstrument::ruler_default();
        assert_eq!(
            ruler.snap_point((0.0, 100.0), INSTRUMENT_SNAP_TOLERANCE),
            None,
            "lejos del borde: trazo libre honesto"
        );
        assert_eq!(
            ruler.snap_point((f64::NAN, 0.0), INSTRUMENT_SNAP_TOLERANCE),
            None,
            "punto no finito: sin panic"
        );
    }

    #[test]
    fn compass_two_clicks_yield_exact_circle() {
        // Centro (100, 100) + radio (130, 140): r = 50 exacto en ambos ejes.
        match compass_circle((100.0, 100.0), (130.0, 140.0)) {
            Some(WhiteboardElement::Ellipse { center, rx, ry }) => {
                assert_eq!(center, (100.0, 100.0));
                assert_eq!(rx, ry, "compás exacto: rx == ry");
                assert!((rx - 50.0).abs() < 1e-9, "radio 50, fue: {rx}");
            }
            other => panic!("círculo esperado, fue: {other:?}"),
        }
    }

    #[test]
    fn compass_degenerate_clicks_yield_nothing() {
        // Doble clic en el mismo punto (o NaN) → None, sin elemento roto.
        assert_eq!(compass_circle((5.0, 5.0), (5.0, 5.0)), None);
        assert_eq!(compass_circle((0.0, 0.0), (0.5, 0.0)), None);
        assert_eq!(compass_circle((f64::NAN, 0.0), (10.0, 0.0)), None);
    }

    #[test]
    fn single_slot_hides_on_second_click_and_replaces_across_kinds() {
        // Un instrumento visible a la vez; clic de nuevo lo guarda.
        let ruler = WhiteboardInstrument::ruler_default();
        assert_eq!(toggle_slot(&Some(ruler), ruler), None);
        let protractor = WhiteboardInstrument::protractor_default();
        assert_eq!(toggle_slot(&Some(ruler), protractor), Some(protractor));
        assert_eq!(toggle_slot(&None, protractor), Some(protractor));
    }

    #[test]
    fn instrument_drag_prefers_handle_then_body() {
        // Sobre la manija → rotar; sobre el filo → mover; fuera → nada.
        let ruler = WhiteboardInstrument::Ruler {
            pos: (0.0, 0.0),
            angle: 0.0,
            len: 200.0,
        };
        let Some(handle) = ruler.handle_pos() else {
            panic!("manija útil esperada");
        };
        assert_eq!(ruler.begin_drag(handle), Some(InstrumentDrag::Rotate));
        assert!(matches!(
            ruler.begin_drag((20.0, 0.0)),
            Some(InstrumentDrag::MoveBody { .. })
        ));
        assert_eq!(ruler.begin_drag((200.0, 200.0)), None);
        // Arrastrar el cuerpo conserva el desplazamiento de agarre.
        let mut moved = ruler;
        moved.apply_drag(
            InstrumentDrag::MoveBody {
                offset: (5.0, -3.0),
            },
            (100.0, 100.0),
        );
        assert_eq!(moved.pos(), (105.0, 97.0));
        assert_eq!(moved.angle(), 0.0);
    }

    #[test]
    fn instruments_never_serialize_to_document_or_svg() {
        // Los instrumentos son overlays: el documento serializado y el SVG
        // no pueden nombrarlos (PROHIBIDO serializar instrumentos).
        let mut doc = WhiteboardDoc::new();
        doc.add(WhiteboardElement::Stroke {
            points: vec![(0.0, 0.0), (10.0, 10.0)],
            color: (0, 0, 0),
            width: 2.0,
        });
        let json = match serde_json::to_string(&doc) {
            Ok(json) => json,
            Err(error) => panic!("documento serializa: {error}"),
        };
        for key in ["Ruler", "Triangle", "Protractor", "instrument"] {
            assert!(!json.contains(key), "fuga al documento: {key} en {json}");
        }
        let svg = match doc.to_svg(800, 600) {
            Ok(svg) => svg,
            Err(error) => panic!("SVG serializa: {error}"),
        };
        for key in ["Ruler", "Triangle", "Protractor", "instrument"] {
            assert!(!svg.contains(key), "fuga al SVG: {key}");
        }
    }
}
