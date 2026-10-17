use glam::{Mat4, Vec2, Vec3};
use serde::{Deserialize, Serialize};

/// 2D point in world coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point2 {
    pub x: f64,
    pub y: f64,
}

impl Point2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn to_vec2(&self) -> Vec2 {
        Vec2::new(self.x as f32, self.y as f32)
    }

    pub fn from_vec2(v: Vec2) -> Self {
        Self::new(v.x as f64, v.y as f64)
    }

    pub fn distance(&self, other: &Point2) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

/// 2D line (infinite) defined by two points.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Line2 {
    pub a: Point2,
    pub b: Point2,
}

impl Line2 {
    pub fn new(a: Point2, b: Point2) -> Self {
        Self { a, b }
    }

    pub fn direction(&self) -> Vec2 {
        Vec2::new((self.b.x - self.a.x) as f32, (self.b.y - self.a.y) as f32)
    }

    pub fn length(&self) -> f64 {
        self.a.distance(&self.b)
    }
}

/// Circle defined by center and radius.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Circle {
    pub center: Point2,
    pub radius: f64,
}

impl Circle {
    pub fn new(center: Point2, radius: f64) -> Self {
        Self { center, radius }
    }

    pub fn contains(&self, p: &Point2) -> bool {
        self.center.distance(p) <= self.radius
    }
}

/// Axis-aligned bounding box.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AABB {
    pub min: Point2,
    pub max: Point2,
}

impl AABB {
    pub fn new(min: Point2, max: Point2) -> Self {
        Self { min, max }
    }

    pub fn expand(&mut self, p: &Point2) {
        self.min.x = self.min.x.min(p.x);
        self.min.y = self.min.y.min(p.y);
        self.max.x = self.max.x.max(p.x);
        self.max.y = self.max.y.max(p.y);
    }

    pub fn contains(&self, p: &Point2) -> bool {
        p.x >= self.min.x && p.x <= self.max.x && p.y >= self.min.y && p.y <= self.max.y
    }
}

/// 2D camera/view transform.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ViewTransform {
    pub offset: Point2,
    pub scale: f64,
    pub screen_size: Vec2,
    pub x_log: bool,
    pub y_log: bool,
}

impl Default for ViewTransform {
    fn default() -> Self {
        Self {
            offset: Point2::new(0.0, 0.0),
            scale: 50.0,
            screen_size: Vec2::new(800.0, 600.0),
            x_log: false,
            y_log: false,
        }
    }
}

impl ViewTransform {
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        Self {
            offset: Point2::new(0.0, 0.0),
            scale: 50.0,
            screen_size: Vec2::new(screen_width, screen_height),
            x_log: false,
            y_log: false,
        }
    }

    /// Transform world coordinate to screen using current axis modes
    pub fn world_to_screen(&self, world: Point2) -> Vec2 {
        let ox = self.screen_size.x as f64 * 0.5 + self.offset.x;
        let oy = self.screen_size.y as f64 * 0.5 + self.offset.y;
        let sx = if self.x_log {
            if world.x <= 0.0 {
                ox - 1000.0 * self.scale
            } else {
                ox + world.x.log10() * self.scale
            }
        } else {
            ox + world.x * self.scale
        };
        let sy = if self.y_log {
            if world.y <= 0.0 {
                oy + 1000.0 * self.scale
            } else {
                oy - world.y.log10() * self.scale
            }
        } else {
            oy - world.y * self.scale
        };
        Vec2::new(sx as f32, sy as f32)
    }

    /// Transform screen point to world coordinates
    pub fn screen_to_world(&self, screen: Vec2) -> Point2 {
        let ox = self.screen_size.x as f64 * 0.5 + self.offset.x;
        let oy = self.screen_size.y as f64 * 0.5 + self.offset.y;
        let wx = if self.x_log {
            10_f64.powf((screen.x as f64 - ox) / self.scale)
        } else {
            (screen.x as f64 - ox) / self.scale
        };
        let wy = if self.y_log {
            10_f64.powf((oy - screen.y as f64) / self.scale)
        } else {
            (oy - screen.y as f64) / self.scale
        };
        Point2::new(wx, wy)
    }

    /// Toggle log mode for both axes (log-log, semi-log, or linear)
    pub fn toggle_log_x(&mut self) {
        self.x_log = !self.x_log;
    }
    pub fn toggle_log_y(&mut self) {
        self.y_log = !self.y_log;
    }

    pub fn pan(&mut self, delta_screen: Vec2) {
        self.offset.x += delta_screen.x as f64;
        self.offset.y += delta_screen.y as f64;
    }

    pub fn zoom(&mut self, factor: f32, anchor_screen: Vec2) {
        if factor.is_nan() || factor.is_infinite() || factor <= 1e-4 {
            return;
        }
        let anchor_world = self.screen_to_world(anchor_screen);
        if (self.x_log && anchor_world.x <= 0.0) || (self.y_log && anchor_world.y <= 0.0) {
            return;
        }
        // Infinito Geogebra: 60 órdenes (1e-30..1e30) — praktisch sin límite, f32 safe (3e38)
        self.scale = (self.scale * factor as f64).clamp(1e-30, 1e30);
        let ox = self.screen_size.x as f64 * 0.5 + self.offset.x;
        let oy = self.screen_size.y as f64 * 0.5 + self.offset.y;
        // Use log-aware coordinate transform, consistent with world_to_screen()
        let sx = if self.x_log {
            ox + anchor_world.x.log10() * self.scale
        } else {
            ox + anchor_world.x * self.scale
        };
        let sy = if self.y_log {
            oy - anchor_world.y.log10() * self.scale
        } else {
            oy - anchor_world.y * self.scale
        };
        self.offset.x += anchor_screen.x as f64 - sx;
        self.offset.y += anchor_screen.y as f64 - sy;
    }

    pub fn projection_matrix(&self) -> Mat4 {
        let half_size = self.screen_size * 0.5;
        Mat4::orthographic_lh(
            -half_size.x,
            half_size.x,
            -half_size.y,
            half_size.y,
            -1.0,
            1.0,
        )
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::from_translation(Vec3::new(self.offset.x as f32, self.offset.y as f32, 0.0))
            * Mat4::from_scale(Vec3::new(self.scale as f32, self.scale as f32, 1.0))
    }

    pub fn mvp_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }
}

/// RGBA color.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0, 1.0);
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);
    /// Trazo neutro visible sobre canvas claros y oscuros.
    pub const DEFAULT_STROKE: Self = Self::new(0.45, 0.48, 0.55, 1.0);
    pub const RED: Self = Self::new(0.9, 0.2, 0.2, 1.0);
    pub const GREEN: Self = Self::new(0.2, 0.8, 0.2, 1.0);
    pub const BLUE: Self = Self::new(0.2, 0.4, 0.9, 1.0);
    pub const GRAY: Self = Self::new(0.5, 0.5, 0.5, 1.0);
    pub const LIGHT_GRAY: Self = Self::new(0.85, 0.85, 0.85, 1.0);

    /// Paleta categórica Okabe-Ito (8, orden canónico): distinguible para
    /// los tres dicromatismos comunes (deuteranopía/protanopía/tritanopía)
    /// y en escala de grises. Para series/objetos sucesivos: `categorical(i)`.
    /// Fuente: Okabe & Ito 2008; valores sRGB / 255.
    pub const CATEGORICAL: [Self; 8] = [
        Self::new(0.902, 0.624, 0.0, 1.0),   // naranja #E69F00
        Self::new(0.337, 0.706, 0.914, 1.0), // celeste #56B4E9
        Self::new(0.0, 0.620, 0.451, 1.0),   // verde azulado #009E73
        Self::new(0.941, 0.894, 0.259, 1.0), // amarillo #F0E442
        Self::new(0.0, 0.447, 0.698, 1.0),   // azul #0072B2
        Self::new(0.835, 0.369, 0.0, 1.0),   // bermellón #D55E00
        Self::new(0.800, 0.475, 0.655, 1.0), // púrpura rojizo #CC79A7
        Self::new(0.0, 0.0, 0.0, 1.0),       // negro #000000
    ];

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Color categórico i-ésimo (cíclico sobre [`Color::CATEGORICAL`]).
    /// Puro y `const`-compatible en uso: `Color::categorical(3)`.
    pub const fn categorical(index: usize) -> Self {
        Self::CATEGORICAL[index % Self::CATEGORICAL.len()]
    }

    /// Rampa secuencial viridis (9 stops sRGB de matplotlib, O6):
    /// perceptualmente uniforme y legible con deuteranopía/protanopía
    /// (a diferencia de arcoíris/térmicas). Para mapas continuos: `viridis(t)`.
    pub const VIRIDIS: [[f32; 3]; 9] = [
        [0.267, 0.005, 0.329], // #440154
        [0.282, 0.157, 0.471], // #482878
        [0.243, 0.286, 0.537], // #3E4989
        [0.192, 0.408, 0.557], // #31688E
        [0.149, 0.510, 0.557], // #26828E
        [0.208, 0.718, 0.474], // #35B779
        [0.431, 0.808, 0.346], // #6ECE58
        [0.722, 0.871, 0.161], // #B8DE29
        [0.992, 0.906, 0.145], // #FDE725
    ];

    /// Interpolación lineal sobre [`Color::VIRIDIS`]; `t` fuera de [0, 1]
    /// se satura (mismo contrato que rampas de render).
    pub fn viridis(t: f32) -> Self {
        const LAST: f32 = (Color::VIRIDIS.len() - 1) as f32;
        let clamped = t.clamp(0.0, 1.0);
        let scaled = clamped * LAST;
        let low = (scaled as usize).min(Color::VIRIDIS.len() - 2);
        let frac = scaled - low as f32;
        let [r0, g0, b0] = Color::VIRIDIS[low];
        let [r1, g1, b1] = Color::VIRIDIS[low + 1];
        Self::new(
            r0 + (r1 - r0) * frac,
            g0 + (g1 - g0) * frac,
            b0 + (b1 - b0) * frac,
            1.0,
        )
    }

    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_to_screen_origin_with_no_offset() {
        // Diagnóstico: el render del ImplicitCurve mostraba la curva
        // desplazada del origen. Este test verifica que
        // `world_to_screen((0, 0))` con `offset = (0, 0)` retorna
        // exactamente el centro del screen.
        let v = ViewTransform::new(800.0, 600.0);
        let s = v.world_to_screen(Point2::new(0.0, 0.0));
        assert_eq!(s.x, 400.0, "center x debe ser 400 (sw/2)");
        assert_eq!(s.y, 300.0, "center y debe ser 300 (sh/2)");
    }

    #[test]
    fn test_world_to_screen_origin_with_offset() {
        // Con `offset = (-50, 0)`, el centro del mundo (0, 0) se dibuja
        // 50 píxeles a la izquierda del centro del screen.
        let mut v = ViewTransform::new(800.0, 600.0);
        v.offset = Point2::new(-50.0, 0.0);
        let s = v.world_to_screen(Point2::new(0.0, 0.0));
        assert_eq!(s.x, 350.0);
        assert_eq!(s.y, 300.0);
    }

    #[test]
    fn default_stroke_is_not_black() {
        let default_stroke = Color::DEFAULT_STROKE.to_array();
        assert_ne!(default_stroke, Color::BLACK.to_array());
        assert!(default_stroke[0] > 0.4);
    }

    #[test]
    fn categorical_palette_is_okabe_ito_cyclic_and_opaque() {
        // 8 entradas distintas, opacas, componentes en [0, 1].
        assert_eq!(Color::CATEGORICAL.len(), 8);
        for (i, color) in Color::CATEGORICAL.iter().enumerate() {
            let [r, g, b, a] = color.to_array();
            assert!((0.0..=1.0).contains(&r), "r fuera de rango en {i}");
            assert!((0.0..=1.0).contains(&g), "g fuera de rango en {i}");
            assert!((0.0..=1.0).contains(&b), "b fuera de rango en {i}");
            assert_eq!(a, 1.0, "alfa no opaco en {i}");
            for other in &Color::CATEGORICAL[i + 1..] {
                assert_ne!(color, other, "duplicado en paleta");
            }
        }
        // Pin de paridad sRGB: naranja #E69F00 primero, negro último.
        let [r, g, b, _] = Color::CATEGORICAL[0].to_array();
        assert!((r - 230.0 / 255.0).abs() < 1e-3);
        assert!((g - 159.0 / 255.0).abs() < 1e-3);
        assert_eq!(b, 0.0);
        assert_eq!(Color::CATEGORICAL[7], Color::BLACK);
        // Cíclico: el 8vo vuelve al 0.
        assert_eq!(Color::categorical(0), Color::categorical(8));
        assert_eq!(Color::categorical(3), Color::CATEGORICAL[3]);
    }

    #[test]
    fn o6_viridis_endpoints_and_monotone_luminance() {
        // Pines sRGB: #440154 → #FDE725, opaco, saturación honesta.
        let [r0, g0, b0, a0] = Color::viridis(0.0).to_array();
        assert!((r0 - 0.267).abs() < 1e-3 && g0 < 0.01 && (b0 - 0.329).abs() < 1e-3);
        assert_eq!(a0, 1.0);
        let [r1, g1, b1, _] = Color::viridis(1.0).to_array();
        assert!(
            (r1 - 0.992).abs() < 1e-3 && (g1 - 0.906).abs() < 1e-3 && (b1 - 0.145).abs() < 1e-3
        );
        // Luminancia monótona creciente (la gracia de viridis) y punto
        // medio sobre la rampa (stop #35B779 ≈ t 0.625).
        let luminance = |t: f32| {
            let [r, g, b, _] = Color::viridis(t).to_array();
            0.2126 * r + 0.7152 * g + 0.0722 * b
        };
        let (mut previous, mut steps) = (luminance(0.0), 0);
        for i in 1..=16 {
            #[allow(clippy::cast_precision_loss)]
            let current = luminance(i as f32 / 16.0);
            assert!(current >= previous, "luminancia no monótona en {i}/16");
            previous = current;
            steps += 1;
        }
        assert_eq!(steps, 16);
        let [rm, gm, bm, _] = Color::viridis(0.625).to_array();
        assert!(
            (rm - 0.208).abs() < 0.02 && (gm - 0.718).abs() < 0.02 && (bm - 0.474).abs() < 0.02
        );
    }

    #[test]
    fn test_circle_world_points_map_to_circle_screen_points() {
        // Para una circunferencia unitaria, todos los puntos están
        // a distancia 1 del origen. La distancia en screen debe ser
        // proporcional a `scale` (= 50.0 por default).
        let v = ViewTransform::new(800.0, 600.0);
        let s = v.world_to_screen(Point2::new(1.0, 0.0));
        // (1, 0) en mundo → (400 + 1*50, 300) = (450, 300) en screen
        assert_eq!(s.x, 450.0);
        let s2 = v.world_to_screen(Point2::new(0.0, 1.0));
        // (0, 1) en mundo → (400, 300 - 1*50) = (400, 250) en screen
        assert_eq!(s2.y, 250.0);
    }
}
