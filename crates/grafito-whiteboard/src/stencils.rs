//! Figuras de geometría (biblioteca de recursos, slice propio).
//!
//! El botón "Figuras" de la toolbar inserta el stencil como elementos
//! NORMALES ([`crate::WhiteboardElement`]: trazos, elipse, flecha, texto):
//! quedan editables y borrables como cualquier trazo — NO existe variante
//! nueva de elemento. Todo vectorial generado en código con coordenadas en
//! múltiplos de la grilla (40 px): cero binarios, cero imágenes, cero
//! dependencias nuevas.
//!
//! Cada stencil es puro (`origin` = esquina de referencia en mundo) y sus
//! coordenadas van pineadas en los tests: si cambian los números, cambian
//! los tests a la par.

use crate::WhiteboardElement;

/// Ancho de la base de los triángulos (4 celdas de grilla).
pub const TRI_BASE: f64 = 160.0;
/// Alto de los triángulos (3 celdas de grilla).
pub const TRI_HEIGHT: f64 = 120.0;
/// Lado del cuadradito de ángulo recto.
pub const RIGHT_MARK: f64 = 16.0;
/// Radio del círculo con centro+radio marcado (y del arco del ángulo).
pub const CIRCLE_RADIUS: f64 = 60.0;
/// Semieje de la cruz del centro.
pub const CENTER_TICK: f64 = 6.0;
/// Largo total de la recta numérica (de -160 a +160).
pub const NUMBER_LINE_HALF: f64 = 160.0;
/// Paso entre marcas de la recta (−120..=120 cada 40).
pub const NUMBER_LINE_STEP: f64 = 40.0;
/// Alto de cada marca de la recta.
pub const NUMBER_TICK_HALF: f64 = 6.0;
/// Tamaño de texto de las etiquetas de los stencils.
pub const STENCIL_TEXT_SIZE: f64 = 14.0;
/// Semialto del eje Y del plano cartesiano (el semiancho X reutiliza
/// [`NUMBER_LINE_HALF`]: 160).
pub const CART_Y_HALF: f64 = 120.0;
/// Radio del transportador (semicírculo superior).
pub const PROT_RADIUS: f64 = 80.0;
/// Radio interior de los ticks chicos del transportador (80→70).
pub const PROT_TICK_INNER: f64 = 70.0;
/// Extremo interior de la marca larga de 90° (80→62).
pub const PROT_MARK_INNER: f64 = 62.0;
/// Radio del hexágono regular (vértices sobre el círculo).
pub const HEX_RADIUS: f64 = 80.0;
/// Lado del cubo 3D (frente de 120×120).
pub const CUBE_SIDE: f64 = 120.0;
/// Desplazamiento del fondo del cubo (arista de profundidad 40×40).
pub const CUBE_DEPTH: f64 = 40.0;
/// Grosor honesto de las aristas ocultas (fino: `Stroke` no soporta
/// punteado, así que 1.0 frente al 1.8 normal).
pub const CUBE_HIDDEN_WIDTH: f64 = 1.0;
/// Largo del minutero del reloj (9 minutos).
pub const CLOCK_MIN_LEN: f64 = 48.0;
/// Largo de la aguja horaria del reloj (10:09).
pub const CLOCK_HOUR_LEN: f64 = 30.0;
/// Radio interior de los ticks horarios (60→52).
pub const CLOCK_TICK_INNER: f64 = 52.0;
/// Ancho de columna de la tabla (3×80).
pub const TABLE_COL: f64 = 80.0;
/// Alto de fila de la tabla (2×40).
pub const TABLE_ROW: f64 = 40.0;
/// Línea interior del encabezado (banda a 32 px: sombreado con líneas).
pub const TABLE_HEADER_LINE: f64 = 32.0;

/// Stencil disponible en el botón "Figuras", en orden del menú.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FigureStencil {
    /// Triángulo rectángulo (catetos 160×120 + marca de 90°).
    RightTriangle,
    /// Triángulo isósceles (base 160, alto 120, altura + ticks iguales).
    IsoscelesTriangle,
    /// Círculo con centro (cruz) y radio marcado.
    CircleWithRadius,
    /// Ángulo marcado (dos rayos + arco + etiqueta).
    MarkedAngle,
    /// Recta numérica (−3..=3 con flecha).
    NumberLine,
    /// Plano cartesiano (ejes x/y con flechas + ticks + x,y,O).
    CartesianPlane,
    /// Transportador (semicírculo + ticks cada 10° + marca 90°).
    Protractor,
    /// Círculo en cuartos (diámetros + etiqueta 1/4).
    FractionCircle,
    /// Hexágono regular (6 lados + centro).
    RegularHexagon,
    /// Cubo 3D en alambre (12 aristas, 3 ocultas finas).
    Cube3D,
    /// Reloj (esfera + 12 ticks + manecillas 10:09 + centro).
    Clock,
    /// Tabla 2×3 (rejilla 3 cols × 2 filas + banda de encabezado).
    Table2x3,
}

impl FigureStencil {
    /// Los 12 del menú, en orden (las 5 viejas primero, luego las 7 nuevas).
    pub const ALL: [Self; 12] = [
        Self::RightTriangle,
        Self::IsoscelesTriangle,
        Self::CircleWithRadius,
        Self::MarkedAngle,
        Self::NumberLine,
        Self::CartesianPlane,
        Self::Protractor,
        Self::FractionCircle,
        Self::RegularHexagon,
        Self::Cube3D,
        Self::Clock,
        Self::Table2x3,
    ];

    /// Etiqueta en español para el menú (hardcoded como el resto de la
    /// toolbar: sin claves i18n nuevas, 6 locales intactos).
    pub fn label(self) -> &'static str {
        match self {
            Self::RightTriangle => "Triángulo rectángulo",
            Self::IsoscelesTriangle => "Triángulo isósceles",
            Self::CircleWithRadius => "Círculo con radio",
            Self::MarkedAngle => "Ángulo marcado",
            Self::NumberLine => "Recta numérica",
            Self::CartesianPlane => "Plano cartesiano",
            Self::Protractor => "Transportador",
            Self::FractionCircle => "Círculo en cuartos",
            Self::RegularHexagon => "Hexágono regular",
            Self::Cube3D => "Cubo 3D",
            Self::Clock => "Reloj",
            Self::Table2x3 => "Tabla 2×3",
        }
    }

    /// ¿Cuántos elementos inserta? (pineado: el test lo exige exacto).
    pub fn element_count(self) -> usize {
        stencil_elements(self, (0.0, 0.0)).len()
    }
}

fn stroke(a: (f64, f64), b: (f64, f64)) -> WhiteboardElement {
    WhiteboardElement::Stroke {
        points: vec![a, b],
        color: (26, 26, 26),
        width: 1.8,
    }
}

fn label(at: (f64, f64), text: &str) -> WhiteboardElement {
    WhiteboardElement::Text {
        at,
        text: text.to_string(),
        size: STENCIL_TEXT_SIZE,
    }
}

/// Arista oculta del cubo: `Stroke` no tiene estilo punteado, así que el
/// "oculto" es fino honesto ([`CUBE_HIDDEN_WIDTH`]) del mismo color.
fn hidden(a: (f64, f64), b: (f64, f64)) -> WhiteboardElement {
    WhiteboardElement::Stroke {
        points: vec![a, b],
        color: (26, 26, 26),
        width: CUBE_HIDDEN_WIDTH,
    }
}

/// Elementos del stencil con `origin` como referencia en mundo. Solo
/// variantes existentes (`Stroke`/`Ellipse`/`Arrow`/`Text`): nada nuevo.
pub fn stencil_elements(kind: FigureStencil, origin: (f64, f64)) -> Vec<WhiteboardElement> {
    if !origin.0.is_finite() || !origin.1.is_finite() {
        return Vec::new();
    }
    let (ox, oy) = origin;
    match kind {
        FigureStencil::RightTriangle => {
            // Vértices: O=(ox,oy) ángulo recto, A=(ox+160,oy), B=(ox,oy+120).
            let o = (ox, oy);
            let a = (ox + TRI_BASE, oy);
            let b = (ox, oy + TRI_HEIGHT);
            vec![
                stroke(o, a),
                stroke(o, b),
                stroke(a, b),
                // Cuadradito de 90° en O (polilínea de 3 puntos).
                WhiteboardElement::Stroke {
                    points: vec![
                        (ox + RIGHT_MARK, oy),
                        (ox + RIGHT_MARK, oy + RIGHT_MARK),
                        (ox, oy + RIGHT_MARK),
                    ],
                    color: (26, 26, 26),
                    width: 1.8,
                },
                label((ox + TRI_BASE / 2.0 - 4.0, oy - 20.0), "a"),
                label((ox - 20.0, oy + TRI_HEIGHT / 2.0 - 6.0), "b"),
                label(
                    (ox + TRI_BASE / 2.0 + 8.0, oy + TRI_HEIGHT / 2.0 - 6.0),
                    "c",
                ),
            ]
        }
        FigureStencil::IsoscelesTriangle => {
            // Base de (ox,oy+120) a (ox+160,oy+120), ápice en (ox+80,oy).
            let bl = (ox, oy + TRI_HEIGHT);
            let br = (ox + TRI_BASE, oy + TRI_HEIGHT);
            let apex = (ox + TRI_BASE / 2.0, oy);
            let foot = (ox + TRI_BASE / 2.0, oy + TRI_HEIGHT);
            // Ticks de "lados iguales": crucecitas en el punto medio de
            // cada lado igual (segmento de 12 px perpendicular aprox.).
            // Lado apex→bl: dir = (-80,120)/144.2; normal ≈ (-0.832,-0.555).
            let mid_left = ((apex.0 + bl.0) / 2.0, (apex.1 + bl.1) / 2.0);
            let mid_right = ((apex.0 + br.0) / 2.0, (apex.1 + br.1) / 2.0);
            vec![
                stroke(bl, br),
                stroke(apex, bl),
                stroke(apex, br),
                // Altura desde el ápice a la base.
                stroke(apex, foot),
                // Ticks de igualdad (12 px centrados en el punto medio).
                stroke(
                    (mid_left.0 - 5.0, mid_left.1 - 3.0),
                    (mid_left.0 + 5.0, mid_left.1 + 3.0),
                ),
                stroke(
                    (mid_right.0 - 5.0, mid_right.1 + 3.0),
                    (mid_right.0 + 5.0, mid_right.1 - 3.0),
                ),
                label(
                    (ox + TRI_BASE / 2.0 + 8.0, oy + TRI_HEIGHT / 2.0 - 6.0),
                    "h",
                ),
            ]
        }
        FigureStencil::CircleWithRadius => {
            vec![
                WhiteboardElement::Ellipse {
                    center: (ox, oy),
                    rx: CIRCLE_RADIUS,
                    ry: CIRCLE_RADIUS,
                },
                // Cruz del centro (2 trazos).
                stroke((ox - CENTER_TICK, oy), (ox + CENTER_TICK, oy)),
                stroke((ox, oy - CENTER_TICK), (ox, oy + CENTER_TICK)),
                // Radio marcado hasta (ox+60,oy).
                stroke((ox, oy), (ox + CIRCLE_RADIUS, oy)),
                label((ox + CIRCLE_RADIUS / 2.0 - 4.0, oy - 18.0), "r"),
            ]
        }
        FigureStencil::MarkedAngle => {
            // Rayos: horizontal 120 px y diagonal a (100,60); arco r=32
            // del ángulo atan2(60,100) con 9 puntos pineados.
            let end_angle = 60.0_f64.atan2(100.0);
            let mut arc = Vec::with_capacity(9);
            for i in 0..9 {
                let theta = end_angle * i as f64 / 8.0;
                arc.push((ox + 32.0 * theta.cos(), oy + 32.0 * theta.sin()));
            }
            vec![
                stroke((ox, oy), (ox + 120.0, oy)),
                stroke((ox, oy), (ox + 100.0, oy + 60.0)),
                WhiteboardElement::Stroke {
                    points: arc,
                    color: (26, 26, 26),
                    width: 1.8,
                },
                label((ox + 44.0, oy + 8.0), "α"),
            ]
        }
        FigureStencil::NumberLine => {
            // Eje con flecha de (ox-160,oy) a (ox+160,oy), marcas cada 40
            // en -120..=120 con etiquetas -3..=3 debajo.
            let mut out = vec![WhiteboardElement::Arrow {
                from: (ox - NUMBER_LINE_HALF, oy),
                to: (ox + NUMBER_LINE_HALF, oy),
            }];
            for (i, n) in (-3..=3).enumerate() {
                let x = ox + f64::from(n) * NUMBER_LINE_STEP;
                out.push(stroke(
                    (x, oy - NUMBER_TICK_HALF),
                    (x, oy + NUMBER_TICK_HALF),
                ));
                let _ = i;
                out.push(label((x - 5.0, oy + 10.0), &n.to_string()));
            }
            out
        }
        FigureStencil::CartesianPlane => {
            // Ejes con flecha: X de (ox−160,oy) a (ox+160,oy), Y de
            // (ox,oy+120) a (ox,oy−120) (arriba = −y). Ticks cada 40 en
            // X (−120..=120 salvo 0) y en Y (−80..=80 salvo 0).
            let mut out = vec![
                WhiteboardElement::Arrow {
                    from: (ox - NUMBER_LINE_HALF, oy),
                    to: (ox + NUMBER_LINE_HALF, oy),
                },
                WhiteboardElement::Arrow {
                    from: (ox, oy + CART_Y_HALF),
                    to: (ox, oy - CART_Y_HALF),
                },
            ];
            for dx in [-120.0, -80.0, -40.0, 40.0, 80.0, 120.0] {
                out.push(stroke(
                    (ox + dx, oy - NUMBER_TICK_HALF),
                    (ox + dx, oy + NUMBER_TICK_HALF),
                ));
            }
            for dy in [-80.0, -40.0, 40.0, 80.0] {
                out.push(stroke(
                    (ox - NUMBER_TICK_HALF, oy + dy),
                    (ox + NUMBER_TICK_HALF, oy + dy),
                ));
            }
            out.push(label((ox + NUMBER_LINE_HALF - 16.0, oy + 10.0), "x"));
            out.push(label((ox + 8.0, oy - CART_Y_HALF + 8.0), "y"));
            out.push(label((ox - 18.0, oy + 10.0), "O"));
            out
        }
        FigureStencil::Protractor => {
            // Semicírculo superior r=80: 19 puntos cada 10° de 180° a 360°
            // + base + 18 ticks (r 80→70, salvo 90°) + marca larga de 90°
            // (r 80→62) + etiqueta.
            let pt = |radius: f64, i: u32| {
                let theta = std::f64::consts::PI + f64::from(i) * std::f64::consts::PI / 18.0;
                (ox + radius * theta.cos(), oy + radius * theta.sin())
            };
            let mut arc = Vec::with_capacity(19);
            for i in 0..=18_u32 {
                arc.push(pt(PROT_RADIUS, i));
            }
            let mut out = vec![
                WhiteboardElement::Stroke {
                    points: arc,
                    color: (26, 26, 26),
                    width: 1.8,
                },
                stroke((ox - PROT_RADIUS, oy), (ox + PROT_RADIUS, oy)),
            ];
            for i in 0..=18_u32 {
                if i == 9 {
                    continue;
                }
                out.push(stroke(pt(PROT_RADIUS, i), pt(PROT_TICK_INNER, i)));
            }
            out.push(stroke((ox, oy - PROT_RADIUS), (ox, oy - PROT_MARK_INNER)));
            out.push(label((ox + 8.0, oy - PROT_RADIUS + 24.0), "90°"));
            out
        }
        FigureStencil::FractionCircle => {
            // Círculo r=60 con diámetros (4 cuartos) + etiqueta 1/4 en el
            // cuarto superior derecho (y hacia abajo: y < oy).
            vec![
                WhiteboardElement::Ellipse {
                    center: (ox, oy),
                    rx: CIRCLE_RADIUS,
                    ry: CIRCLE_RADIUS,
                },
                stroke((ox, oy - CIRCLE_RADIUS), (ox, oy + CIRCLE_RADIUS)),
                stroke((ox - CIRCLE_RADIUS, oy), (ox + CIRCLE_RADIUS, oy)),
                label((ox + 20.0, oy - 20.0), "1/4"),
            ]
        }
        FigureStencil::RegularHexagon => {
            // Vértices sobre r=80 cada 60° (plano: vértice en 0°) + cruz.
            let v = |k: u32| {
                let a = f64::from(k) * std::f64::consts::PI / 3.0;
                (ox + HEX_RADIUS * a.cos(), oy + HEX_RADIUS * a.sin())
            };
            vec![
                stroke(v(0), v(1)),
                stroke(v(1), v(2)),
                stroke(v(2), v(3)),
                stroke(v(3), v(4)),
                stroke(v(4), v(5)),
                stroke(v(5), v(0)),
                stroke((ox - CENTER_TICK, oy), (ox + CENTER_TICK, oy)),
                stroke((ox, oy - CENTER_TICK), (ox, oy + CENTER_TICK)),
            ]
        }
        FigureStencil::Cube3D => {
            // Alambre: frente de 120×120 en (ox,oy), fondo desplazado
            // (+40,−40). 12 aristas: 9 visibles + 3 ocultas finas (la
            // unión A−A′ y el fondo izq/inf que el frente tapa).
            let a = (ox, oy);
            let b = (ox + CUBE_SIDE, oy);
            let c = (ox + CUBE_SIDE, oy + CUBE_SIDE);
            let d = (ox, oy + CUBE_SIDE);
            let ap = (ox + CUBE_DEPTH, oy - CUBE_DEPTH);
            let bp = (ox + CUBE_SIDE + CUBE_DEPTH, oy - CUBE_DEPTH);
            let cp = (ox + CUBE_SIDE + CUBE_DEPTH, oy + CUBE_SIDE - CUBE_DEPTH);
            let dp = (ox + CUBE_DEPTH, oy + CUBE_SIDE - CUBE_DEPTH);
            vec![
                stroke(a, b),
                stroke(b, c),
                stroke(c, d),
                stroke(d, a),
                stroke(ap, bp),
                stroke(bp, cp),
                stroke(b, bp),
                stroke(c, cp),
                stroke(d, dp),
                hidden(ap, a),
                hidden(ap, dp),
                hidden(dp, cp),
            ]
        }
        FigureStencil::Clock => {
            // Esfera r=60 + 12 ticks horarios (r 60→52, las 12 arriba) +
            // manecillas 10:09 (minutero a 9 min, horario a 10 + 9/60 h)
            // + cruz del centro.
            let mut out = vec![WhiteboardElement::Ellipse {
                center: (ox, oy),
                rx: CIRCLE_RADIUS,
                ry: CIRCLE_RADIUS,
            }];
            for k in 0..12_u32 {
                let a = f64::from(k) * std::f64::consts::PI / 6.0;
                let (dx, dy) = (a.sin(), -a.cos());
                out.push(stroke(
                    (ox + CIRCLE_RADIUS * dx, oy + CIRCLE_RADIUS * dy),
                    (ox + CLOCK_TICK_INNER * dx, oy + CLOCK_TICK_INNER * dy),
                ));
            }
            let min_a = 2.0 * std::f64::consts::PI * 9.0 / 60.0;
            out.push(stroke(
                (ox, oy),
                (
                    ox + CLOCK_MIN_LEN * min_a.sin(),
                    oy - CLOCK_MIN_LEN * min_a.cos(),
                ),
            ));
            let hour_a = 2.0 * std::f64::consts::PI * (10.0 + 9.0 / 60.0) / 12.0;
            out.push(stroke(
                (ox, oy),
                (
                    ox + CLOCK_HOUR_LEN * hour_a.sin(),
                    oy - CLOCK_HOUR_LEN * hour_a.cos(),
                ),
            ));
            out.push(stroke((ox - CENTER_TICK, oy), (ox + CENTER_TICK, oy)));
            out.push(stroke((ox, oy - CENTER_TICK), (ox, oy + CENTER_TICK)));
            out
        }
        FigureStencil::Table2x3 => {
            // Rejilla de 3×80 por 2×40 + doble línea del encabezado (banda
            // a 32 px): sombreado solo con líneas, sin rellenos.
            let mut out = Vec::with_capacity(8);
            for k in 0..=3_u32 {
                let x = ox + f64::from(k) * TABLE_COL;
                out.push(stroke((x, oy), (x, oy + 2.0 * TABLE_ROW)));
            }
            for k in 0..=2_u32 {
                let y = oy + f64::from(k) * TABLE_ROW;
                out.push(stroke((ox, y), (ox + 3.0 * TABLE_COL, y)));
            }
            out.push(stroke(
                (ox, oy + TABLE_HEADER_LINE),
                (ox + 3.0 * TABLE_COL, oy + TABLE_HEADER_LINE),
            ));
            out
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stencil_menu_has_twelve_labeled_options() {
        assert_eq!(FigureStencil::ALL.len(), 12);
        assert_eq!(FigureStencil::RightTriangle.label(), "Triángulo rectángulo");
        assert_eq!(
            FigureStencil::IsoscelesTriangle.label(),
            "Triángulo isósceles"
        );
        assert_eq!(FigureStencil::CircleWithRadius.label(), "Círculo con radio");
        assert_eq!(FigureStencil::MarkedAngle.label(), "Ángulo marcado");
        assert_eq!(FigureStencil::NumberLine.label(), "Recta numérica");
        assert_eq!(FigureStencil::CartesianPlane.label(), "Plano cartesiano");
        assert_eq!(FigureStencil::Protractor.label(), "Transportador");
        assert_eq!(FigureStencil::FractionCircle.label(), "Círculo en cuartos");
        assert_eq!(FigureStencil::RegularHexagon.label(), "Hexágono regular");
        assert_eq!(FigureStencil::Cube3D.label(), "Cubo 3D");
        assert_eq!(FigureStencil::Clock.label(), "Reloj");
        assert_eq!(FigureStencil::Table2x3.label(), "Tabla 2×3");
        // Las 5 viejas primero, luego las 7 nuevas (orden del menú).
        assert_eq!(
            FigureStencil::ALL,
            [
                FigureStencil::RightTriangle,
                FigureStencil::IsoscelesTriangle,
                FigureStencil::CircleWithRadius,
                FigureStencil::MarkedAngle,
                FigureStencil::NumberLine,
                FigureStencil::CartesianPlane,
                FigureStencil::Protractor,
                FigureStencil::FractionCircle,
                FigureStencil::RegularHexagon,
                FigureStencil::Cube3D,
                FigureStencil::Clock,
                FigureStencil::Table2x3,
            ]
        );
        // Constantes pineadas (múltiplos de la grilla 40).
        assert_eq!(TRI_BASE, 160.0);
        assert_eq!(TRI_HEIGHT, 120.0);
        assert_eq!(RIGHT_MARK, 16.0);
        assert_eq!(CIRCLE_RADIUS, 60.0);
        assert_eq!(NUMBER_LINE_HALF, 160.0);
        assert_eq!(NUMBER_LINE_STEP, 40.0);
        assert_eq!(CART_Y_HALF, 120.0);
        assert_eq!(PROT_RADIUS, 80.0);
        assert_eq!(PROT_TICK_INNER, 70.0);
        assert_eq!(PROT_MARK_INNER, 62.0);
        assert_eq!(HEX_RADIUS, 80.0);
        assert_eq!(CUBE_SIDE, 120.0);
        assert_eq!(CUBE_DEPTH, 40.0);
        assert_eq!(CUBE_HIDDEN_WIDTH, 1.0);
        assert_eq!(CLOCK_MIN_LEN, 48.0);
        assert_eq!(CLOCK_HOUR_LEN, 30.0);
        assert_eq!(CLOCK_TICK_INNER, 52.0);
        assert_eq!(TABLE_COL, 80.0);
        assert_eq!(TABLE_ROW, 40.0);
        assert_eq!(TABLE_HEADER_LINE, 32.0);
    }

    #[test]
    fn right_triangle_pins_exact_coordinates() {
        let els = stencil_elements(FigureStencil::RightTriangle, (10.0, 20.0));
        assert_eq!(els.len(), 7, "3 lados + marca 90° + 3 etiquetas");
        assert_eq!(
            els[0],
            stroke((10.0, 20.0), (170.0, 20.0)),
            "cateto horizontal"
        );
        assert_eq!(
            els[1],
            stroke((10.0, 20.0), (10.0, 140.0)),
            "cateto vertical"
        );
        assert_eq!(els[2], stroke((170.0, 20.0), (10.0, 140.0)), "hipotenusa");
        match &els[3] {
            WhiteboardElement::Stroke { points, .. } => assert_eq!(
                *points,
                vec![(26.0, 20.0), (26.0, 36.0), (10.0, 36.0)],
                "cuadradito de 90° en O"
            ),
            other => panic!("marca esperada, fue: {other:?}"),
        }
        assert_eq!(els[4], label((86.0, 0.0), "a"));
        assert_eq!(els[5], label((-10.0, 74.0), "b"));
        assert_eq!(els[6], label((98.0, 74.0), "c"));
    }

    #[test]
    fn isosceles_triangle_pins_exact_coordinates() {
        let els = stencil_elements(FigureStencil::IsoscelesTriangle, (0.0, 0.0));
        assert_eq!(els.len(), 7, "3 lados + altura + 2 ticks + 1 etiqueta");
        assert_eq!(els[0], stroke((0.0, 120.0), (160.0, 120.0)), "base");
        assert_eq!(els[1], stroke((80.0, 0.0), (0.0, 120.0)), "lado izq");
        assert_eq!(els[2], stroke((80.0, 0.0), (160.0, 120.0)), "lado der");
        assert_eq!(els[3], stroke((80.0, 0.0), (80.0, 120.0)), "altura");
        // Puntos medios: izq (40,60), der (120,60); ticks de 10×6.
        assert_eq!(els[4], stroke((35.0, 57.0), (45.0, 63.0)), "tick izq");
        assert_eq!(els[5], stroke((115.0, 63.0), (125.0, 57.0)), "tick der");
        assert_eq!(els[6], label((88.0, 54.0), "h"));
    }

    #[test]
    fn circle_pins_center_radius_and_label() {
        let els = stencil_elements(FigureStencil::CircleWithRadius, (50.0, 50.0));
        assert_eq!(els.len(), 5, "elipse + cruz(2) + radio + etiqueta");
        assert_eq!(
            els[0],
            WhiteboardElement::Ellipse {
                center: (50.0, 50.0),
                rx: 60.0,
                ry: 60.0,
            }
        );
        assert_eq!(els[1], stroke((44.0, 50.0), (56.0, 50.0)), "cruz H");
        assert_eq!(els[2], stroke((50.0, 44.0), (50.0, 56.0)), "cruz V");
        assert_eq!(els[3], stroke((50.0, 50.0), (110.0, 50.0)), "radio");
        assert_eq!(els[4], label((76.0, 32.0), "r"));
    }

    #[test]
    fn marked_angle_pins_rays_arc_and_label() {
        let els = stencil_elements(FigureStencil::MarkedAngle, (0.0, 0.0));
        assert_eq!(els.len(), 4, "2 rayos + arco + etiqueta");
        assert_eq!(els[0], stroke((0.0, 0.0), (120.0, 0.0)), "rayo H");
        assert_eq!(els[1], stroke((0.0, 0.0), (100.0, 60.0)), "rayo diag");
        match &els[2] {
            WhiteboardElement::Stroke { points, .. } => {
                assert_eq!(points.len(), 9, "arco de 9 puntos");
                assert_eq!(points[0], (32.0, 0.0), "arco nace en el rayo H");
                let end = 60.0_f64.atan2(100.0);
                let expected = (32.0 * end.cos(), 32.0 * end.sin());
                assert!(
                    (points[8].0 - expected.0).abs() < 1e-9
                        && (points[8].1 - expected.1).abs() < 1e-9,
                    "arco muere en el rayo diagonal: {:?}",
                    points[8]
                );
                // Radio constante en todo el arco.
                for p in points {
                    let r = (p.0 * p.0 + p.1 * p.1).sqrt();
                    assert!((r - 32.0).abs() < 1e-9, "radio 32: {p:?}");
                }
            }
            other => panic!("arco esperado, fue: {other:?}"),
        }
        assert_eq!(els[3], label((44.0, 8.0), "α"));
    }

    #[test]
    fn number_line_pins_axis_ticks_and_labels() {
        let els = stencil_elements(FigureStencil::NumberLine, (0.0, 0.0));
        assert_eq!(els.len(), 15, "eje + 7 marcas + 7 etiquetas");
        assert_eq!(
            els[0],
            WhiteboardElement::Arrow {
                from: (-160.0, 0.0),
                to: (160.0, 0.0),
            }
        );
        // Marcas en -120..=120 cada 40 con etiquetas -3..=3.
        for (k, n) in (-3..=3).enumerate() {
            let x = f64::from(n) * 40.0;
            assert_eq!(els[1 + k * 2], stroke((x, -6.0), (x, 6.0)), "marca del {n}");
            assert_eq!(
                els[2 + k * 2],
                label((x - 5.0, 10.0), &n.to_string()),
                "etiqueta del {n}"
            );
        }
    }

    #[test]
    fn cartesian_plane_pins_axes_ticks_and_labels() {
        let els = stencil_elements(FigureStencil::CartesianPlane, (0.0, 0.0));
        assert_eq!(els.len(), 15, "2 ejes + 10 ticks + 3 etiquetas");
        assert_eq!(
            els[0],
            WhiteboardElement::Arrow {
                from: (-160.0, 0.0),
                to: (160.0, 0.0),
            },
            "eje X"
        );
        assert_eq!(
            els[1],
            WhiteboardElement::Arrow {
                from: (0.0, 120.0),
                to: (0.0, -120.0),
            },
            "eje Y (arriba = -y)"
        );
        for (k, dx) in [-120.0, -80.0, -40.0, 40.0, 80.0, 120.0].iter().enumerate() {
            assert_eq!(els[2 + k], stroke((*dx, -6.0), (*dx, 6.0)), "tick X {dx}");
        }
        for (k, dy) in [-80.0, -40.0, 40.0, 80.0].iter().enumerate() {
            assert_eq!(els[8 + k], stroke((-6.0, *dy), (6.0, *dy)), "tick Y {dy}");
        }
        assert_eq!(els[12], label((144.0, 10.0), "x"));
        assert_eq!(els[13], label((8.0, -112.0), "y"));
        assert_eq!(els[14], label((-18.0, 10.0), "O"));
    }

    #[test]
    fn protractor_pins_arc_ticks_and_90_mark() {
        let els = stencil_elements(FigureStencil::Protractor, (0.0, 0.0));
        assert_eq!(
            els.len(),
            22,
            "arco + base + 18 ticks + marca 90° + etiqueta"
        );
        match &els[0] {
            WhiteboardElement::Stroke { points, .. } => {
                assert_eq!(points.len(), 19, "un punto cada 10°");
                for p in points {
                    let r = (p.0 * p.0 + p.1 * p.1).sqrt();
                    assert!((r - 80.0).abs() < 1e-9, "radio 80: {p:?}");
                    assert!(p.1 <= 1e-9, "semicírculo superior: {p:?}");
                }
                assert!(
                    (points[0].0 + 80.0).abs() < 1e-9 && points[0].1.abs() < 1e-9,
                    "nace en 180°: {:?}",
                    points[0]
                );
                assert!(
                    (points[18].0 - 80.0).abs() < 1e-9 && points[18].1.abs() < 1e-9,
                    "muere en 360°: {:?}",
                    points[18]
                );
                assert!(
                    points[9].0.abs() < 1e-9 && (points[9].1 + 80.0).abs() < 1e-9,
                    "la cima es 90°: {:?}",
                    points[9]
                );
            }
            other => panic!("arco esperado, fue: {other:?}"),
        }
        assert_eq!(els[1], stroke((-80.0, 0.0), (80.0, 0.0)), "base");
        // 18 ticks de 10 px (r 80→70); el de 90° se salta (marca larga).
        for tick in &els[2..20] {
            match tick {
                WhiteboardElement::Stroke { points, .. } => {
                    assert_eq!(points.len(), 2);
                    let outer = (points[0].0.hypot(points[0].1) - 80.0).abs();
                    let inner = (points[1].0.hypot(points[1].1) - 70.0).abs();
                    assert!(outer < 1e-9 && inner < 1e-9, "tick 80→70: {tick:?}");
                    assert!(
                        points[0].0.abs() > 1.0,
                        "el de 90° no va aquí (tiene marca propia): {tick:?}"
                    );
                }
                other => panic!("tick esperado, fue: {other:?}"),
            }
        }
        assert_eq!(els[20], stroke((0.0, -80.0), (0.0, -62.0)), "marca 90°");
        assert_eq!(els[21], label((8.0, -56.0), "90°"));
    }

    #[test]
    fn fraction_circle_pins_quarters_and_label() {
        let els = stencil_elements(FigureStencil::FractionCircle, (0.0, 0.0));
        assert_eq!(els.len(), 4, "círculo + 2 diámetros + etiqueta");
        assert_eq!(
            els[0],
            WhiteboardElement::Ellipse {
                center: (0.0, 0.0),
                rx: 60.0,
                ry: 60.0,
            }
        );
        assert_eq!(els[1], stroke((0.0, -60.0), (0.0, 60.0)), "vertical");
        assert_eq!(els[2], stroke((-60.0, 0.0), (60.0, 0.0)), "horizontal");
        assert_eq!(els[3], label((20.0, -20.0), "1/4"));
    }

    #[test]
    fn hexagon_pins_six_sides_and_center() {
        let els = stencil_elements(FigureStencil::RegularHexagon, (0.0, 0.0));
        assert_eq!(els.len(), 8, "6 lados + cruz del centro");
        // Lados encadenados sobre el círculo r=80; el primero nace en 0°.
        let mut ends = Vec::with_capacity(12);
        for (k, side) in els[..6].iter().enumerate() {
            match side {
                WhiteboardElement::Stroke { points, .. } => {
                    assert_eq!(points.len(), 2, "lado {k}");
                    for p in points {
                        let r = p.0.hypot(p.1);
                        assert!((r - 80.0).abs() < 1e-9, "vértice r=80: {p:?}");
                    }
                    ends.push(points[0]);
                    ends.push(points[1]);
                }
                other => panic!("lado esperado, fue: {other:?}"),
            }
        }
        assert_eq!(ends[0], (80.0, 0.0), "vértice en 0°");
        for k in 0..6 {
            assert_eq!(
                ends[2 * k + 1],
                ends[(2 * k + 2) % 12],
                "lados encadenados en el vértice {k}"
            );
        }
        assert_eq!(els[6], stroke((-6.0, 0.0), (6.0, 0.0)), "cruz H");
        assert_eq!(els[7], stroke((0.0, -6.0), (0.0, 6.0)), "cruz V");
    }

    #[test]
    fn cube_pins_twelve_edges_three_hidden_thin() {
        let els = stencil_elements(FigureStencil::Cube3D, (0.0, 0.0));
        assert_eq!(els.len(), 12, "9 visibles + 3 ocultas");
        assert_eq!(els[0], stroke((0.0, 0.0), (120.0, 0.0)), "frente arriba");
        assert_eq!(els[1], stroke((120.0, 0.0), (120.0, 120.0)), "frente der");
        assert_eq!(els[2], stroke((120.0, 120.0), (0.0, 120.0)), "frente abajo");
        assert_eq!(els[3], stroke((0.0, 120.0), (0.0, 0.0)), "frente izq");
        assert_eq!(
            els[4],
            stroke((40.0, -40.0), (160.0, -40.0)),
            "fondo arriba"
        );
        assert_eq!(els[5], stroke((160.0, -40.0), (160.0, 80.0)), "fondo der");
        assert_eq!(els[6], stroke((120.0, 0.0), (160.0, -40.0)), "unión B");
        assert_eq!(els[7], stroke((120.0, 120.0), (160.0, 80.0)), "unión C");
        assert_eq!(els[8], stroke((0.0, 120.0), (40.0, 80.0)), "unión D");
        // Ocultas finas honestas (Stroke no tiene punteado: width 1.0).
        for (k, hidden_edge) in els[9..].iter().enumerate() {
            match hidden_edge {
                WhiteboardElement::Stroke {
                    points,
                    color,
                    width,
                } => {
                    assert_eq!(points.len(), 2, "oculta {k}");
                    assert_eq!(*color, (26, 26, 26));
                    assert_eq!(*width, 1.0, "oculta {k} fina");
                }
                other => panic!("oculta esperada, fue: {other:?}"),
            }
        }
        assert_eq!(els[9], hidden((40.0, -40.0), (0.0, 0.0)), "unión A oculta");
        assert_eq!(
            els[10],
            hidden((40.0, -40.0), (40.0, 80.0)),
            "fondo izq oculto"
        );
        assert_eq!(
            els[11],
            hidden((40.0, 80.0), (160.0, 80.0)),
            "fondo abajo oculto"
        );
    }

    #[test]
    fn clock_pins_face_ticks_and_1009_hands() {
        let els = stencil_elements(FigureStencil::Clock, (0.0, 0.0));
        assert_eq!(els.len(), 17, "esfera + 12 ticks + 2 manecillas + cruz");
        assert_eq!(
            els[0],
            WhiteboardElement::Ellipse {
                center: (0.0, 0.0),
                rx: 60.0,
                ry: 60.0,
            }
        );
        assert_eq!(els[1], stroke((0.0, -60.0), (0.0, -52.0)), "las 12");
        // 12 ticks de 8 px sobre la esfera (r 60→52).
        for (k, tick) in els[1..13].iter().enumerate() {
            match tick {
                WhiteboardElement::Stroke { points, .. } => {
                    assert_eq!(points.len(), 2, "tick {k}");
                    let outer = points[0].0.hypot(points[0].1);
                    let inner = points[1].0.hypot(points[1].1);
                    assert!((outer - 60.0).abs() < 1e-9, "tick {k} nace en r=60");
                    assert!((inner - 52.0).abs() < 1e-9, "tick {k} muere en r=52");
                }
                other => panic!("tick esperado, fue: {other:?}"),
            }
        }
        // Manecillas 10:09 (minutero largo 48 a 9 min, horario corto 30
        // pasado las 10): ambas arriba, horario a la izq y minutero a la der.
        for (tick, len) in [(&els[13], 48.0), (&els[14], 30.0)] {
            match tick {
                WhiteboardElement::Stroke { points, .. } => {
                    assert_eq!(points[0], (0.0, 0.0), "nace en el centro");
                    let hand = points[1].0.hypot(points[1].1);
                    assert!((hand - len).abs() < 1e-9, "largo {len}: {tick:?}");
                    assert!(points[1].1 < 0.0, "arriba: {tick:?}");
                }
                other => panic!("manecilla esperada, fue: {other:?}"),
            }
        }
        match (&els[13], &els[14]) {
            (
                WhiteboardElement::Stroke { points: min, .. },
                WhiteboardElement::Stroke { points: hour, .. },
            ) => {
                assert!(min[1].0 > 0.0, "minutero a 9 min (der): {min:?}");
                assert!(hour[1].0 < 0.0, "horario pasadas las 10 (izq): {hour:?}");
            }
            _ => unreachable!(),
        }
        assert_eq!(els[15], stroke((-6.0, 0.0), (6.0, 0.0)), "cruz H");
        assert_eq!(els[16], stroke((0.0, -6.0), (0.0, 6.0)), "cruz V");
    }

    #[test]
    fn table_pins_grid_and_header_band() {
        let els = stencil_elements(FigureStencil::Table2x3, (0.0, 0.0));
        assert_eq!(els.len(), 8, "4 verticales + 3 horizontales + banda");
        for (k, x) in [0.0, 80.0, 160.0, 240.0].iter().enumerate() {
            assert_eq!(els[k], stroke((*x, 0.0), (*x, 80.0)), "vertical {x}");
        }
        for (k, y) in [0.0, 40.0, 80.0].iter().enumerate() {
            assert_eq!(els[4 + k], stroke((0.0, *y), (240.0, *y)), "horizontal {y}");
        }
        assert_eq!(
            els[7],
            stroke((0.0, 32.0), (240.0, 32.0)),
            "banda del encabezado"
        );
    }

    #[test]
    fn stencils_use_only_plain_elements_and_survive_roundtrip() {
        // Nada de variante nueva: solo Stroke/Ellipse/Arrow/Text.
        for kind in FigureStencil::ALL {
            for el in stencil_elements(kind, (5.0, 5.0)) {
                assert!(
                    matches!(
                        el,
                        WhiteboardElement::Stroke { .. }
                            | WhiteboardElement::Ellipse { .. }
                            | WhiteboardElement::Arrow { .. }
                            | WhiteboardElement::Text { .. }
                    ),
                    "solo elementos normales: {el:?}"
                );
                assert!(el.bounds().is_some(), "bounds finitos: {el:?}");
                assert!(el.distance_to((5.0, 5.0)).is_finite());
            }
            // `element_count` honesto = lo que se inserta.
            assert_eq!(
                kind.element_count(),
                stencil_elements(kind, (0.0, 0.0)).len()
            );
        }
        assert_eq!(FigureStencil::RightTriangle.element_count(), 7);
        assert_eq!(FigureStencil::IsoscelesTriangle.element_count(), 7);
        assert_eq!(FigureStencil::CircleWithRadius.element_count(), 5);
        assert_eq!(FigureStencil::MarkedAngle.element_count(), 4);
        assert_eq!(FigureStencil::NumberLine.element_count(), 15);
        assert_eq!(FigureStencil::CartesianPlane.element_count(), 15);
        assert_eq!(FigureStencil::Protractor.element_count(), 22);
        assert_eq!(FigureStencil::FractionCircle.element_count(), 4);
        assert_eq!(FigureStencil::RegularHexagon.element_count(), 8);
        assert_eq!(FigureStencil::Cube3D.element_count(), 12);
        assert_eq!(FigureStencil::Clock.element_count(), 17);
        assert_eq!(FigureStencil::Table2x3.element_count(), 8);
        // Cupo de 500 elementos/hoja: el stencil más grande (22) entra
        // siempre que haya sitio para sus N (la atomicidad la pinean los
        // tests de `insert_figure` en `grafito-app`).
        for kind in FigureStencil::ALL {
            assert!(
                kind.element_count() <= 500,
                "cupo 500: {kind:?} trae {}",
                kind.element_count()
            );
        }
        // Origen no finito → vacío honesto, sin panic.
        for kind in FigureStencil::ALL {
            assert!(stencil_elements(kind, (f64::NAN, 0.0)).is_empty());
            assert!(stencil_elements(kind, (0.0, f64::INFINITY)).is_empty());
        }
        // Viajan por el JSON viejo sin variante nueva (compat intacta).
        let els = stencil_elements(FigureStencil::NumberLine, (0.0, 0.0));
        let json = serde_json::to_string(&els).expect("serializa");
        assert!(!json.contains("Figure"), "fuga de tipo nuevo: {json}");
        let back: Vec<WhiteboardElement> = serde_json::from_str(&json).expect("deserializa");
        assert_eq!(back, els);
    }
}
