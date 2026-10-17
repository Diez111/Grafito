//! Confeti contenido — celebración visceral mínima (puro painter, sin estado global).
//!
//! Ráfaga escandinava: sutil, nunca invasiva. ~40 partículas en tonos sobrios
//! del tema activo, 600 ms de vida, una sola vez por hito de sesión. Pura
//! matemática determinista por seed de tiempo + un dibujado efímero sobre un
//! layer no-interactivo (jamás bloquea input, jamás tapa paneles: el origen
//! vive arriba al centro, bajo la top-bar, donde ya viven los toasts).
//!
//! - Piel pura: [`confetti_particles`], [`confetti_offset`], [`confetti_alpha`],
//!   [`confetti_alive`] y [`confetti_visible_count`] no tocan egui ni I/O.
//! - `reduced_motion` (WCAG 2.3.3): [`confetti_visible_count`] y [`draw_burst`]
//!   devuelven CERO partículas / no pintan nada.
//! - Sin persistencia, sin sonido, sin streak: el estado vive en la sesión de
//!   la app (`grafito-app/src/celebration.rs`) y muere con ella.

use crate::theme::Theme;

/// Partículas por ráfaga. Pocas a propósito: se lee como destello, no como fiesta.
pub const CONFETTI_COUNT: usize = 40;
/// Vida total de la ráfaga en milisegundos. Corta: aparece y se va.
pub const CONFETTI_DURATION_MS: f32 = 600.0;
/// Gravedad del vuelo (px/s²). Caída leve, sin drama.
const CONFETTI_GRAVITY_PX_S2: f32 = 420.0;
/// Colores de la paleta contenida (pastel sobrio del tema, sin neones).
const CONFETTI_PALETTE_SIZE: usize = 6;

/// Una partícula de la ráfaga: vuelo balístico + giro. Todo `Copy` y
/// determinista por seed (ver [`confetti_particles`]).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConfettiParticle {
    /// Dirección inicial del vuelo (radianes, 0..`TAU`).
    pub angle_rad: f32,
    /// Velocidad inicial (px/s). Acotada para que la ráfaga no salga del cuarto superior.
    pub speed_px_s: f32,
    /// Lado del papelito (px). Chica: 3..6.
    pub size_px: f32,
    /// Rotación inicial (radianes).
    pub rotation_rad: f32,
    /// Velocidad de giro (rad/s). Suave.
    pub spin_rad_s: f32,
    /// Índice en [`confetti_palette`] (`< CONFETTI_PALETTE_SIZE`).
    pub color_idx: u8,
}

/// Ráfaga efímera: instante de inicio + seed. El origen se resuelve al dibujar
/// (arriba al centro del viewport actual), así el spawn no necesita contexto.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConfettiBurst {
    /// Instante de disparo en ms (mismo reloj que los toasts: `ui_time * 1000`).
    pub start_ms: f64,
    /// Seed determinista del disparo (ver [`confetti_seed`]).
    pub seed: u64,
}

impl ConfettiBurst {
    /// Dispara una ráfaga ahora. Puro, sin I/O.
    pub fn new(start_ms: f64, seed: u64) -> Self {
        Self { start_ms, seed }
    }

    /// Milisegundos desde el disparo (`now_ms` en el mismo reloj que `start_ms`).
    /// Reloj no-finito o anterior al disparo cuenta como 0 (sin fantasma).
    pub fn elapsed_ms(&self, now_ms: f64) -> f32 {
        if !now_ms.is_finite() || !self.start_ms.is_finite() {
            return 0.0;
        }
        ((now_ms - self.start_ms).max(0.0)) as f32
    }

    /// `true` mientras la ráfaga sigue viva (ver [`confetti_alive`]).
    pub fn is_alive(&self, now_ms: f64) -> bool {
        confetti_alive(self.elapsed_ms(now_ms))
    }
}

/// Seed determinista desde el tiempo: el mismo instante siempre da la misma
/// ráfaga (reproducible en tests y dorados). La multiplicación por impar es
/// biyectiva en `u64`: instantes distintos dan ráfagas distintas.
pub fn confetti_seed(now_ms: f64) -> u64 {
    let ticks = if now_ms.is_finite() && now_ms > 0.0 {
        now_ms as u64
    } else {
        0
    };
    ticks.wrapping_mul(0x9E3779B97F4A7C15)
}

fn next_u64(state: &mut u64) -> u64 {
    if *state == 0 {
        *state = 0x243F6A8885A308D3;
    }
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

/// Fracción uniforme en [0, 1) desde el estado (xorshift64 + 53 bits altos).
fn unit(state: &mut u64) -> f32 {
    const DIVISOR: f64 = (1u64 << 53) as f64;
    ((next_u64(state) >> 11) as f64 / DIVISOR) as f32
}

/// Las [`CONFETTI_COUNT`] partículas de una ráfaga. Puro y determinista: el
/// mismo seed siempre devuelve el mismo arreglo (ver test).
pub fn confetti_particles(seed: u64) -> [ConfettiParticle; CONFETTI_COUNT] {
    let mut state = seed;
    std::array::from_fn(|_| {
        let angle_rad = unit(&mut state) * std::f32::consts::TAU;
        let speed_px_s = 60.0 + unit(&mut state) * 160.0;
        let size_px = 3.0 + unit(&mut state) * 3.0;
        let rotation_rad = unit(&mut state) * std::f32::consts::TAU;
        let spin_rad_s = -6.0 + unit(&mut state) * 12.0;
        let color_idx = (next_u64(&mut state) % CONFETTI_PALETTE_SIZE as u64) as u8;
        ConfettiParticle {
            angle_rad,
            speed_px_s,
            size_px,
            rotation_rad,
            spin_rad_s,
            color_idx,
        }
    })
}

/// Desplazamiento (px) desde el origen a los `elapsed_ms` del disparo.
/// Vuelo balístico: radial + caída leve por gravedad. Puro, sin I/O.
pub fn confetti_offset(particle: &ConfettiParticle, elapsed_ms: f32) -> (f32, f32) {
    let tick = (elapsed_ms / 1_000.0).clamp(0.0, CONFETTI_DURATION_MS / 1_000.0);
    let x = particle.angle_rad.cos() * particle.speed_px_s * tick;
    let y = particle.angle_rad.sin() * particle.speed_px_s * tick
        + 0.5 * CONFETTI_GRAVITY_PX_S2 * tick * tick;
    (x, y)
}

/// Alfa global de la ráfaga (1 → 0 lineal en [`CONFETTI_DURATION_MS`]).
/// Fuera de tiempo (negativo o vencido) da 0: sin partículas fantasma.
pub fn confetti_alpha(elapsed_ms: f32) -> f32 {
    if !elapsed_ms.is_finite() || elapsed_ms < 0.0 {
        return 0.0;
    }
    (1.0 - elapsed_ms / CONFETTI_DURATION_MS).clamp(0.0, 1.0)
}

/// `true` mientras la ráfaga vive: `[0, CONFETTI_DURATION_MS)`.
pub fn confetti_alive(elapsed_ms: f32) -> bool {
    elapsed_ms.is_finite() && (0.0..CONFETTI_DURATION_MS).contains(&elapsed_ms)
}

/// Partículas visibles ahora: [`CONFETTI_COUNT`] viva, CERO si `reduced_motion`
/// (WCAG 2.3.3) o fuera de tiempo. Puro, sin I/O.
pub fn confetti_visible_count(elapsed_ms: f32, reduced_motion: bool) -> usize {
    if reduced_motion || !confetti_alive(elapsed_ms) {
        0
    } else {
        CONFETTI_COUNT
    }
}

/// Paleta contenida: 6 acentos sobrios del tema (sin neones, sin hardcodeo).
/// Pura, sin I/O.
pub fn confetti_palette(theme: &Theme) -> [egui::Color32; CONFETTI_PALETTE_SIZE] {
    [
        theme.accent,
        theme.success,
        theme.toast_cas,
        theme.object_point,
        theme.warning,
        theme.accent_strong,
    ]
}

/// Dibuja la ráfaga sobre `painter` desde `origin`. Puro painter: papelitos
/// rotados con alfa global; con `reduced_motion` o fuera de tiempo no pinta
/// nada (CERO formas). No registra interacción: jamás bloquea input.
pub fn draw_burst(
    painter: &egui::Painter,
    origin: egui::Pos2,
    burst: &ConfettiBurst,
    now_ms: f64,
    theme: &Theme,
    reduced_motion: bool,
) {
    if reduced_motion {
        return;
    }
    let elapsed = burst.elapsed_ms(now_ms);
    if !confetti_alive(elapsed) {
        return;
    }
    let alpha = confetti_alpha(elapsed);
    let palette = confetti_palette(theme);
    let tick = (elapsed / 1_000.0).clamp(0.0, CONFETTI_DURATION_MS / 1_000.0);
    for particle in confetti_particles(burst.seed) {
        let (dx, dy) = confetti_offset(&particle, elapsed);
        let center = origin + egui::vec2(dx, dy);
        let rotation = particle.rotation_rad + particle.spin_rad_s * tick;
        let half = particle.size_px * 0.5;
        let (sin, cos) = rotation.sin_cos();
        let corners = [(-half, -half), (half, -half), (half, half), (-half, half)]
            .map(|(x, y)| center + egui::vec2(x * cos - y * sin, x * sin + y * cos));
        let color =
            palette[(particle.color_idx as usize) % CONFETTI_PALETTE_SIZE].linear_multiply(alpha);
        painter.add(egui::Shape::convex_polygon(
            corners.to_vec(),
            color,
            egui::Stroke::NONE,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::{
        confetti_alive, confetti_alpha, confetti_offset, confetti_particles, confetti_seed,
        confetti_visible_count, ConfettiBurst, CONFETTI_COUNT, CONFETTI_DURATION_MS,
    };

    #[test]
    fn conteo_exactamente_cuarenta() {
        assert_eq!(CONFETTI_COUNT, 40);
        assert_eq!(confetti_particles(1).len(), 40);
        assert_eq!(confetti_visible_count(0.0, false), 40);
        assert_eq!(confetti_visible_count(599.0, false), 40);
    }

    #[test]
    fn determinista_por_seed() {
        let a = confetti_particles(12345);
        let b = confetti_particles(12345);
        assert_eq!(a, b);
        let c = confetti_particles(12346);
        assert_ne!(a, c, "seeds distintos dan ráfagas distintas");
    }

    #[test]
    fn seed_determinista_y_biyectivo() {
        assert_eq!(confetti_seed(1000.0), confetti_seed(1000.0));
        assert_ne!(confetti_seed(1000.0), confetti_seed(1001.0));
        assert_eq!(confetti_seed(f64::NAN), confetti_seed(f64::NEG_INFINITY));
    }

    #[test]
    fn particulas_en_rango_sutil() {
        for p in confetti_particles(7) {
            assert!((0.0..=std::f32::consts::TAU).contains(&p.angle_rad));
            assert!((60.0..=220.0).contains(&p.speed_px_s), "{}", p.speed_px_s);
            assert!((3.0..=6.0).contains(&p.size_px), "{}", p.size_px);
            assert!((-6.0..=6.0).contains(&p.spin_rad_s), "{}", p.spin_rad_s);
            assert!(p.color_idx < 6, "{}", p.color_idx);
        }
    }

    #[test]
    fn vida_alfa_decae_lineal() {
        assert!((confetti_alpha(0.0) - 1.0).abs() < 1e-6);
        assert!((confetti_alpha(300.0) - 0.5).abs() < 1e-6);
        assert_eq!(confetti_alpha(CONFETTI_DURATION_MS), 0.0);
        assert_eq!(confetti_alpha(900.0), 0.0);
        assert_eq!(confetti_alpha(-1.0), 0.0);
        assert_eq!(confetti_alpha(f32::NAN), 0.0);
        let a = confetti_alpha(100.0);
        let b = confetti_alpha(400.0);
        assert!(a > b, "el alfa decae con el tiempo");
    }

    #[test]
    fn fuera_de_tiempo_sin_particulas() {
        assert!(confetti_alive(0.0));
        assert!(confetti_alive(599.9));
        assert!(!confetti_alive(CONFETTI_DURATION_MS));
        assert!(!confetti_alive(3_600.0));
        assert!(!confetti_alive(-0.1));
        assert!(!confetti_alive(f32::NAN));
        assert_eq!(confetti_visible_count(CONFETTI_DURATION_MS, false), 0);
        assert_eq!(confetti_visible_count(10_000.0, false), 0);
        assert_eq!(confetti_visible_count(-5.0, false), 0);
    }

    #[test]
    fn reduced_motion_cero_particulas() {
        assert_eq!(confetti_visible_count(0.0, true), 0);
        assert_eq!(confetti_visible_count(300.0, true), 0);
    }

    #[test]
    fn vuelo_acotado_y_finito() {
        for p in confetti_particles(99) {
            for t in [0.0, 100.0, 300.0, 599.0] {
                let (x, y) = confetti_offset(&p, t);
                assert!(x.is_finite() && y.is_finite());
                assert!(
                    x.abs() < 500.0 && y.abs() < 500.0,
                    "ráfaga contenida: {x},{y}"
                );
            }
            assert_eq!(confetti_offset(&p, 0.0), (0.0, 0.0));
        }
    }

    #[test]
    fn burst_mide_tiempo_del_mismo_reloj() {
        let burst = ConfettiBurst::new(1_000.0, 5);
        assert!((burst.elapsed_ms(1_300.0) - 300.0).abs() < 1e-3);
        assert_eq!(burst.elapsed_ms(500.0), 0.0);
        assert_eq!(burst.elapsed_ms(f64::NAN), 0.0);
        assert!(burst.is_alive(1_300.0));
        assert!(!burst.is_alive(1_600.0));
        assert!(!burst.is_alive(1_000.0 + f64::from(CONFETTI_DURATION_MS)));
    }

    #[test]
    fn draw_respeta_reduced_motion_y_vencimiento() {
        let ctx = egui::Context::default();
        let theme = *crate::theme::DARK;
        let origin = egui::pos2(200.0, 72.0);
        let burst = ConfettiBurst::new(0.0, 11);
        let count_shapes = |now_ms: f64, reduced: bool| {
            let output = ctx.run(egui::RawInput::default(), |ctx| {
                let layer =
                    egui::LayerId::new(egui::Order::Foreground, egui::Id::new("test_confetti"));
                let painter = ctx.layer_painter(layer);
                super::draw_burst(&painter, origin, &burst, now_ms, &theme, reduced);
            });
            output.shapes.len()
        };
        let baseline = count_shapes(10_000.0, false);
        assert_eq!(
            count_shapes(300.0, true),
            baseline,
            "reduced_motion: CERO formas"
        );
        assert_eq!(
            count_shapes(10_000.0, false),
            baseline,
            "vencida: CERO formas"
        );
        assert!(
            count_shapes(300.0, false) > baseline,
            "viva sin reduced_motion: pinta partículas"
        );
    }
}
