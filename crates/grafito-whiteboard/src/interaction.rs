//! Máquina de estado del arrastre sobre la pizarra y selección por marco.

use crate::{WhiteboardDoc, WhiteboardElement};

/// Herramienta de pizarra activa.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhiteboardTool {
    Select,
    Pencil,
    /// Resaltador translúcido (estilo OpenBoard): mismo gesto que el
    /// lápiz pero crea `WhiteboardElement::Highlighter` (alfa fijo 35 %,
    /// ancho generoso) insertado AL FONDO, debajo de la tinta existente.
    /// Icono reutilizado `Edit` (sin icono nuevo en `grafito-ui`).
    Highlighter,
    Rectangle,
    Ellipse,
    Arrow,
    Text,
    Eraser,
}

/// Cota del trazo de borrado (anti-DoS): `update` ignora los puntos más allá.
/// Igual que el cap de puntos por trazo de la UI (`whiteboard_ui.rs:34`,
/// `MAX_WHITEBOARD_POINTS_PER_STROKE = 4096`); un arrastre eterno no crece
/// sin cota en memoria. `take_erase_path` sigue exigiendo ≥2 puntos.
pub const MAX_ERASE_PATH_POINTS: usize = 4096;

/// Estado del arrastre del puntero.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum WhiteboardInteraction {
    #[default]
    Idle,
    /// Creando una forma desde `from` hasta `current`.
    Creating {
        from: (f64, f64),
        current: (f64, f64),
        tool: WhiteboardTool,
    },
    /// Borrando objetos bajo un trazo.
    Erasing { path: Vec<(f64, f64)> },
}

impl WhiteboardInteraction {
    pub fn begin(from: (f64, f64), tool: WhiteboardTool) -> Self {
        match tool {
            WhiteboardTool::Eraser => Self::Erasing { path: vec![from] },
            other => Self::Creating {
                from,
                current: from,
                tool: other,
            },
        }
    }

    pub fn update(&mut self, point: (f64, f64)) {
        match self {
            Self::Creating { current, .. } => *current = point,
            Self::Erasing { path } => {
                if path.len() < MAX_ERASE_PATH_POINTS {
                    path.push(point);
                }
            }
            Self::Idle => {}
        }
    }

    pub fn preview(&self) -> Option<WhiteboardElement> {
        let Self::Creating {
            from,
            current,
            tool,
        } = self
        else {
            return None;
        };
        make_element(*tool, *from, *current)
    }

    pub fn end(&self) -> Option<WhiteboardElement> {
        self.preview()
    }

    /// Devuelve el trazo de borrado y lo consume.
    pub fn take_erase_path(&mut self) -> Option<Vec<(f64, f64)>> {
        match self {
            Self::Erasing { path } if path.len() >= 2 => {
                let path = std::mem::take(path);
                *self = Self::Erasing { path: Vec::new() };
                Some(path)
            }
            _ => None,
        }
    }
}

/// Construye el elemento a partir de dos esquinas del arrastre.
pub fn make_element(
    tool: WhiteboardTool,
    from: (f64, f64),
    to: (f64, f64),
) -> Option<WhiteboardElement> {
    // Guard contra coordenadas no finitas (evita crash al tocar “Pizarra” con drag NaN)
    if !from.0.is_finite() || !from.1.is_finite() || !to.0.is_finite() || !to.1.is_finite() {
        return None;
    }
    let min_drag = 1e-6;
    if (from.0 - to.0).abs() < min_drag && (from.1 - to.1).abs() < min_drag {
        return None;
    }
    match tool {
        WhiteboardTool::Rectangle => {
            let min = (from.0.min(to.0), from.1.min(to.1));
            let max = (from.0.max(to.0), from.1.max(to.1));
            Some(WhiteboardElement::Rectangle {
                min,
                max,
                fill: None,
            })
        }
        WhiteboardTool::Ellipse => {
            let width = (from.0 - to.0).abs();
            let height = (from.1 - to.1).abs();
            Some(WhiteboardElement::Ellipse {
                center: ((from.0 + to.0) / 2.0, (from.1 + to.1) / 2.0),
                rx: width / 2.0,
                ry: height / 2.0,
            })
        }
        WhiteboardTool::Arrow => Some(WhiteboardElement::Arrow { from, to }),
        WhiteboardTool::Text => Some(WhiteboardElement::Text {
            at: to,
            text: String::new(),
            size: 14.0,
        }),
        WhiteboardTool::Pencil
        | WhiteboardTool::Highlighter
        | WhiteboardTool::Select
        | WhiteboardTool::Eraser => None,
    }
}

/// ── Reconocimiento geométrico del lápiz (estilo OpenBoard) ─────────────
///
/// Con la opción «Formas» del panel, el trazo del lápiz se intenta cerrar
/// a forma antes de guardarlo como tinta libre:
///
/// - trazo cerrado ≈ círculo → `Ellipse` (centro = centro del bbox,
///   radios = semiejes del bbox);
/// - trazo recto ≈ línea → `Arrow` (de primer a último punto; la pizarra
///   no tiene variante «recta suelta» y la flecha es su línea honesta);
/// - si no cierra con nada → `None` y el llamador guarda el `Stroke` tal
///   cual (honesto: el garabato queda garabato).
///
/// Tolerancias documentadas (unidades mundo @zoom 1; la grilla es 40 px):
///
/// - [`SHAPE_MIN_POINTS`]: con menos puntos no se intenta nada.
/// - cierre: `dist(first, last) ≤ max(SHAPE_CLOSED_ABS_GAP,
///   SHAPE_CLOSED_REL_GAP · diagonal_bbox)`.
/// - círculo: aspecto del bbox en
///   [`SHAPE_CIRCLE_ASPECT_MIN`]..=[`SHAPE_CIRCLE_ASPECT_MAX`], radio medio
///   ≥ [`SHAPE_CIRCLE_MIN_RADIUS`] y CV radial (std/media de distancias al
///   centro) ≤ [`SHAPE_CIRCLE_MAX_RADIAL_CV`].
/// - recta: largo ≥ [`SHAPE_LINE_MIN_LEN`] y desvío máximo de los puntos
///   al segmento ≤ `max(SHAPE_LINE_MAX_ABS_DEV, SHAPE_LINE_MAX_REL_DEV ·
///   largo)`.
///
/// Todo es O(n) local, sin dependencias ni I/O: funciona offline en clase.
/// Los puntos no finitos devuelven `None` (el llamador valida y rechaza).
///
/// Polígonos (rectángulo/triángulo a mano alzada): EXCLUIDOS a propósito —
/// con una sola pasada geométrica no salen limpios (esquinas ambiguas,
/// falsos positivos en garabatos) y un casi-rectángulo convertido a
/// `Rectangle` perfecto mentiría sobre el gesto. Quedan como `Stroke`.
///
/// Handwriting recognition: EXCLUIDO a propósito (evaluación honesta).
/// `tesseract` (ver `grafito-app/src/ocr_cli.rs`) consume rásters PNG/JPEG
/// vía binario externo con timeout; los trazos son vectores y no existe un
/// rasterizador headless de la pizarra sola (pendiente PNG documentado en
/// `whiteboard_ui.rs::draw_share_buttons`), así que habría que rasterizar
/// solo para el OCR — frágil y lento. Y el OCR lee *texto*, no geometría:
/// no distingue un círculo de una «o». El geométrico local de arriba cubre
/// el caso útil del aula sin dependencias nuevas.
///
/// Nota: `Ellipse`/`Arrow` no llevan color ni grosor propios (se dibujan
/// con la tinta de formas `text_primary`, igual que las formas de la
/// toolbar): el color/grosor del lápiz no se traslada a la forma
/// reconocida. Solo el fallback `Stroke` conserva la tinta elegida.
///
/// Puntos mínimos para intentar reconocer (menos → `None` directo).
pub const SHAPE_MIN_POINTS: usize = 8;
/// Cierre: gap absoluto en unidades mundo.
pub const SHAPE_CLOSED_ABS_GAP: f64 = 16.0;
/// Cierre: gap relativo a la diagonal del bbox.
pub const SHAPE_CLOSED_REL_GAP: f64 = 0.20;
/// Círculo: aspecto mínimo del bbox (ancho/alto normalizado).
pub const SHAPE_CIRCLE_ASPECT_MIN: f64 = 0.65;
/// Círculo: aspecto máximo del bbox.
pub const SHAPE_CIRCLE_ASPECT_MAX: f64 = 1.45;
/// Círculo: radio medio mínimo (un bucle diminuto no es un círculo).
pub const SHAPE_CIRCLE_MIN_RADIUS: f64 = 8.0;
/// Círculo: CV radial máximo (std/media de distancias al centro).
pub const SHAPE_CIRCLE_MAX_RADIAL_CV: f64 = 0.25;
/// Recta: largo mínimo del segmento primer→último punto.
pub const SHAPE_LINE_MIN_LEN: f64 = 20.0;
/// Recta: desvío absoluto máximo al segmento.
pub const SHAPE_LINE_MAX_ABS_DEV: f64 = 6.0;
/// Recta: desvío relativo máximo (fracción del largo).
pub const SHAPE_LINE_MAX_REL_DEV: f64 = 0.06;

fn shape_dist(a: (f64, f64), b: (f64, f64)) -> f64 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}

fn shape_seg_dist(a: (f64, f64), b: (f64, f64), p: (f64, f64)) -> f64 {
    let dx = b.0 - a.0;
    let dy = b.1 - a.1;
    let len_sq = dx * dx + dy * dy;
    if !len_sq.is_finite() || len_sq <= 0.0 {
        return shape_dist(a, p);
    }
    let t = (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / len_sq).clamp(0.0, 1.0);
    shape_dist((a.0 + t * dx, a.1 + t * dy), p)
}

/// Intenta cerrar el trazo del lápiz a forma (`Ellipse`/`Arrow`).
/// `None` = no reconoce nada (el llamador guarda el `Stroke` honesto).
/// Nunca hace panic: entradas degeneradas o no finitas dan `None`.
pub fn recognize_pencil_shape(points: &[(f64, f64)]) -> Option<WhiteboardElement> {
    if points.len() < SHAPE_MIN_POINTS {
        return None;
    }
    if points.iter().any(|(x, y)| !x.is_finite() || !y.is_finite()) {
        return None;
    }
    let (mut min_x, mut min_y) = points[0];
    let (mut max_x, mut max_y) = points[0];
    for (x, y) in &points[1..] {
        min_x = min_x.min(*x);
        min_y = min_y.min(*y);
        max_x = max_x.max(*x);
        max_y = max_y.max(*y);
    }
    let width = max_x - min_x;
    let height = max_y - min_y;
    let diagonal = (width * width + height * height).sqrt();
    if !diagonal.is_finite() || diagonal <= 0.0 {
        return None;
    }
    let first = points[0];
    let last = points[points.len() - 1];
    // 1) ¿Trazo cerrado? → ¿círculo? (cerrado pero no circular → `None`:
    //    el polígono queda fuera y el garabato sigue a la rama recta, que
    //    lo rechazará por desvío → `Stroke` honesto).
    let gap = shape_dist(first, last);
    if gap <= SHAPE_CLOSED_ABS_GAP.max(SHAPE_CLOSED_REL_GAP * diagonal) {
        let big = width.max(height);
        if big > 0.0 {
            let aspect = width.min(height) / big;
            if (SHAPE_CIRCLE_ASPECT_MIN..=SHAPE_CIRCLE_ASPECT_MAX).contains(&aspect) {
                let center = ((min_x + max_x) / 2.0, (min_y + max_y) / 2.0);
                let mean_r = (width + height) / 4.0;
                if mean_r >= SHAPE_CIRCLE_MIN_RADIUS {
                    let count = points.len() as f64;
                    let mean = points.iter().map(|p| shape_dist(*p, center)).sum::<f64>() / count;
                    if mean > 0.0 {
                        let variance = points
                            .iter()
                            .map(|p| {
                                let d = shape_dist(*p, center) - mean;
                                d * d
                            })
                            .sum::<f64>()
                            / count;
                        if variance.sqrt() / mean <= SHAPE_CIRCLE_MAX_RADIAL_CV {
                            return Some(WhiteboardElement::Ellipse {
                                center,
                                rx: width / 2.0,
                                ry: height / 2.0,
                            });
                        }
                    }
                }
            }
        }
        return None;
    }
    // 2) ¿Trazo recto? → flecha (la «recta» de la pizarra).
    let len = shape_dist(first, last);
    if len >= SHAPE_LINE_MIN_LEN {
        let tolerance = SHAPE_LINE_MAX_ABS_DEV.max(SHAPE_LINE_MAX_REL_DEV * len);
        if points
            .iter()
            .all(|p| shape_seg_dist(first, last, *p) <= tolerance)
        {
            return Some(WhiteboardElement::Arrow {
                from: first,
                to: last,
            });
        }
    }
    None
}

/// Selecciona por marco (intersección de cajas).
pub fn select_in_marquee(
    doc: &WhiteboardDoc,
    rect_min: (f64, f64),
    rect_max: (f64, f64),
) -> Vec<usize> {
    let (rmin, rmax) = (
        (rect_min.0.min(rect_max.0), rect_min.1.min(rect_max.1)),
        (rect_min.0.max(rect_max.0), rect_min.1.max(rect_max.1)),
    );
    doc.elements()
        .iter()
        .enumerate()
        .filter_map(|(index, element)| {
            let (min, max) = element.bounds()?;
            let overlaps = min.0 <= rmax.0 && max.0 >= rmin.0 && min.1 <= rmax.1 && max.1 >= rmin.1;
            overlaps.then_some(index)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rectangle_is_normalized_even_when_dragged_reversed() {
        let rectangle = make_element(WhiteboardTool::Rectangle, (8.0, 8.0), (2.0, 2.0));
        assert_eq!(
            rectangle.as_ref().and_then(|element| element.bounds()),
            Some(((2.0, 2.0), (8.0, 8.0))),
            "rectángulo invertido se normaliza sin unwrap"
        );
    }

    #[test]
    fn interaction_creates_previews_and_ending_discards_tiny_drags() {
        let mut interaction = WhiteboardInteraction::begin((0.0, 0.0), WhiteboardTool::Ellipse);
        interaction.update((4.0, 2.0));
        assert!(interaction.preview().is_some());
        let tiny = WhiteboardInteraction::begin((1.0, 1.0), WhiteboardTool::Arrow);
        assert!(tiny.end().is_none());
    }

    #[test]
    fn eraser_interaction_accumulates_a_path() {
        let mut eraser = WhiteboardInteraction::begin((0.0, 0.0), WhiteboardTool::Eraser);
        eraser.update((1.0, 0.0));
        assert!(eraser.take_erase_path().is_some());
    }

    #[test]
    fn eraser_path_is_bounded() {
        let mut eraser = WhiteboardInteraction::begin((0.0, 0.0), WhiteboardTool::Eraser);
        for index in 0..(MAX_ERASE_PATH_POINTS + 500) {
            eraser.update((index as f64, 0.0));
        }
        let path = eraser.take_erase_path().expect("sigue útil con la cota");
        assert_eq!(path.len(), MAX_ERASE_PATH_POINTS);
    }

    #[test]
    fn marquee_selects_only_overlapping_elements() {
        let mut doc = WhiteboardDoc::new();
        doc.add(WhiteboardElement::Rectangle {
            min: (0.0, 0.0),
            max: (2.0, 2.0),
            fill: None,
        });
        doc.add(WhiteboardElement::Rectangle {
            min: (10.0, 10.0),
            max: (12.0, 12.0),
            fill: None,
        });
        let selected = select_in_marquee(&doc, (-1.0, -1.0), (3.0, 3.0));
        assert_eq!(selected, vec![0]);
    }

    // --- Reconocimiento geométrico del lápiz (círculo→Ellipse,
    //     recta→Arrow, garabato→None/Stroke) con tolerancias pineadas. ---

    #[test]
    fn shape_tolerances_are_pinned() {
        // Si cambian, cambia el doc de `recognize_pencil_shape` a la par.
        assert_eq!(SHAPE_MIN_POINTS, 8);
        assert_eq!(SHAPE_CLOSED_ABS_GAP, 16.0);
        assert_eq!(SHAPE_CLOSED_REL_GAP, 0.20);
        assert_eq!(SHAPE_CIRCLE_ASPECT_MIN, 0.65);
        assert_eq!(SHAPE_CIRCLE_ASPECT_MAX, 1.45);
        assert_eq!(SHAPE_CIRCLE_MIN_RADIUS, 8.0);
        assert_eq!(SHAPE_CIRCLE_MAX_RADIAL_CV, 0.25);
        assert_eq!(SHAPE_LINE_MIN_LEN, 20.0);
        assert_eq!(SHAPE_LINE_MAX_ABS_DEV, 6.0);
        assert_eq!(SHAPE_LINE_MAX_REL_DEV, 0.06);
    }

    fn circle_points(center: (f64, f64), radius: f64, count: usize) -> Vec<(f64, f64)> {
        (0..count)
            .map(|i| {
                let angle = 2.0 * std::f64::consts::PI * i as f64 / count as f64;
                (
                    center.0 + radius * angle.cos(),
                    center.1 + radius * angle.sin(),
                )
            })
            .collect()
    }

    #[test]
    fn closed_loop_becomes_ellipse() {
        // Bucle de 32 puntos, radio 50: gap ≈ 9.8 ≤ max(16, 0.2·141) y
        // CV radial ≈ 0 → Ellipse con el bbox como radios.
        let points = circle_points((100.0, 100.0), 50.0, 32);
        match recognize_pencil_shape(&points) {
            Some(WhiteboardElement::Ellipse { center, rx, ry }) => {
                assert!((center.0 - 100.0).abs() < 1.0, "centro x: {center:?}");
                assert!((center.1 - 100.0).abs() < 1.0, "centro y: {center:?}");
                assert!((rx - 50.0).abs() < 1.0, "rx: {rx}");
                assert!((ry - 50.0).abs() < 1.0, "ry: {ry}");
            }
            other => panic!("elipse esperada, fue: {other:?}"),
        }
    }

    #[test]
    fn straight_drag_becomes_arrow() {
        // Recta de 100 px con jitter ±2 (dentro de max(6, 0.06·100)) →
        // Arrow de primer a último punto.
        let points: Vec<(f64, f64)> = (0..16)
            .map(|i| {
                let x = i as f64 * (100.0 / 15.0);
                (x, if i % 2 == 0 { 2.0 } else { -2.0 })
            })
            .collect();
        match recognize_pencil_shape(&points) {
            Some(WhiteboardElement::Arrow { from, to }) => {
                assert_eq!(from, points[0]);
                assert_eq!(to, points[points.len() - 1]);
            }
            other => panic!("flecha esperada, fue: {other:?}"),
        }
    }

    #[test]
    fn scribble_stays_honest_stroke() {
        // Zigzag abierto que no es recta (desvío ≫ tolerancia) y no
        // cierra: `None` para que el llamador guarde el Stroke tal cual.
        let points: Vec<(f64, f64)> = (0..20)
            .map(|i| (i as f64 * 10.0, if i % 2 == 0 { 0.0 } else { 40.0 }))
            .collect();
        assert_eq!(
            recognize_pencil_shape(&points),
            None,
            "el garabato no se cierra a nada"
        );
        // Cerrado pero no circular (lóbulo alargado 100×10, aspecto 0.1):
        // tampoco — el polígono queda fuera y no se inventa elipse.
        let mut lobe: Vec<(f64, f64)> = (0..16)
            .map(|i| (i as f64 * (100.0 / 15.0), (i as f64 * 0.8).sin() * 5.0))
            .collect();
        lobe.push(lobe[0]);
        assert_eq!(
            recognize_pencil_shape(&lobe),
            None,
            "cerrado no circular no es elipse"
        );
    }

    #[test]
    fn short_tiny_and_degenerate_strokes_never_become_shapes() {
        // Menos de 8 puntos: ni se intenta.
        assert_eq!(recognize_pencil_shape(&[(0.0, 0.0), (100.0, 0.0)]), None);
        // Bucle diminuto (radio 2 < 8): no es un círculo útil.
        assert_eq!(
            recognize_pencil_shape(&circle_points((0.0, 0.0), 2.0, 16)),
            None
        );
        // Recta corta (< 20): no es línea.
        let short: Vec<(f64, f64)> = (0..10).map(|i| (i as f64, 0.0)).collect();
        assert_eq!(recognize_pencil_shape(&short), None);
        // Puntos no finitos o idénticos: `None` sin panic.
        assert_eq!(recognize_pencil_shape(&vec![(f64::NAN, 0.0); 16]), None);
        assert_eq!(recognize_pencil_shape(&vec![(5.0, 5.0); 16]), None);
    }
}
