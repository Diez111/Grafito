//! Cuaterniones de Hamilton para la ola O2 (no-conmutativo).
//!
//! Slice atómico 1/3 de `Tasks.md` O2 (cerrada 2026-10-04 con 2/3
//! `symbolic_matrix.rs` + 3/3 F4/F5 en `cas.rs`): el producto de Hamilton es el ejemplo
//! canónico de álgebra no conmutativa (`i·j = k` pero `j·i = −k`). Núcleo
//! numérico puro sobre `f64` — no introduce un nuevo tipo de expresión y por
//! tanto respeta la regla fija de UNA sola representación (`Expr`/bytecode
//! para lo simbólico; aquí solo rotaciones y álgebra `f64`).
//!
//! Lo que cubre este módulo: construcción, Hamilton `hamilton_mul`, conjugado, norma,
//! inverso honesto, `from_axis_angle`, rotación de vectores, matriz 3×3 y
//! `slerp` con fallback lineal en el régimen casi-paralelo.
//!
//! Lo que NO cubre (cerrado en O2): matrices simbólicas sobre `Expr` en
//! `symbolic_matrix.rs` y F4/F5 (Faugère) en `cas.rs` (`buchberger_f4_basis`,
//! `buchberger_f5_basis` + `f5_rewritten_skip`) — el Buchberger con cotas
//! `MAX_GROEBNER_S_POLY` 384 sigue siendo la vía por defecto.
//!
//! Presupuesto: cero alloc en el núcleo (todo por valor); bordes con
//! `f64::is_finite` devuelven `Err` honesto en vez de propagar NaN/inf.

/// Cuaternión `w + x·i + y·j + z·k` en `f64`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quaternion {
    /// Parte escalar.
    pub w: f64,
    /// Coeficiente de `i`.
    pub x: f64,
    /// Coeficiente de `j`.
    pub y: f64,
    /// Coeficiente de `k`.
    pub z: f64,
}

impl Quaternion {
    /// Construye sin validar (vía rápida para constantes exactas de tests).
    ///
    /// Los bordes públicos (`from_axis_angle`, `normalize`, `inverse`,
    /// `rotate_vector`, `slerp`) sí exigen finitud vía [`Self::is_finite`].
    #[must_use]
    pub const fn new(w: f64, x: f64, y: f64, z: f64) -> Self {
        Self { w, x, y, z }
    }

    /// Identidad `1 + 0·i + 0·j + 0·k`.
    #[must_use]
    pub const fn identity() -> Self {
        Self::new(1.0, 0.0, 0.0, 0.0)
    }

    /// Base `i`.
    #[must_use]
    pub const fn i() -> Self {
        Self::new(0.0, 1.0, 0.0, 0.0)
    }

    /// Base `j`.
    #[must_use]
    pub const fn j() -> Self {
        Self::new(0.0, 0.0, 1.0, 0.0)
    }

    /// Base `k`.
    #[must_use]
    pub const fn k() -> Self {
        Self::new(0.0, 0.0, 0.0, 1.0)
    }

    /// Finitud de las 4 componentes.
    #[must_use]
    pub fn is_finite(self) -> bool {
        self.w.is_finite() && self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }

    fn check_finite(self, ctx: &str) -> Result<Self, String> {
        if self.is_finite() {
            Ok(self)
        } else {
            Err(format!(
                "cuaternión no finito en {ctx}: [{}, {}, {}, {}]",
                self.w, self.x, self.y, self.z
            ))
        }
    }

    /// Conjugado `w − x·i − y·j − z·k`.
    #[must_use]
    pub const fn conjugate(self) -> Self {
        Self::new(self.w, -self.x, -self.y, -self.z)
    }

    /// Producto escalar de 4 componentes.
    #[must_use]
    pub fn dot(self, other: Self) -> f64 {
        self.w * other.w + self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Norma al cuadrado (sin raíz: útil para comparar sin costo).
    #[must_use]
    pub fn norm_squared(self) -> f64 {
        self.dot(self)
    }

    /// Norma `|q|`.
    #[must_use]
    pub fn norm(self) -> f64 {
        self.norm_squared().sqrt()
    }

    /// Producto de Hamilton (NO conmutativo: `q·p ≠ p·q` en general).
    #[must_use]
    pub fn hamilton_mul(self, other: Self) -> Self {
        Self::new(
            self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z,
            self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y,
            self.w * other.y - self.x * other.z + self.y * other.w + self.z * other.x,
            self.w * other.z + self.x * other.y - self.y * other.x + self.z * other.w,
        )
    }

    /// Suma por componentes.
    #[must_use]
    pub fn plus(self, other: Self) -> Self {
        Self::new(
            self.w + other.w,
            self.x + other.x,
            self.y + other.y,
            self.z + other.z,
        )
    }

    /// Escala por un escalar finito (chequeo en el llamante si hace falta).
    #[must_use]
    pub fn scale(self, s: f64) -> Self {
        Self::new(self.w * s, self.x * s, self.y * s, self.z * s)
    }

    /// Normaliza a norma 1; error honesto ante cero/no-finito.
    pub fn normalize(self) -> Result<Self, String> {
        let q = self.check_finite("normalize")?;
        let n = q.norm();
        if !n.is_finite() || n == 0.0 {
            return Err(format!(
                "cuaternión no normalizable (norma {n}): [{}, {}, {}, {}]",
                q.w, q.x, q.y, q.z
            ));
        }
        Ok(q.scale(1.0 / n))
    }

    /// Inverso `q⁻¹ = conj(q)/|q|²`; error honesto ante cero/no-finito.
    pub fn inverse(self) -> Result<Self, String> {
        let q = self.check_finite("inverse")?;
        let n2 = q.norm_squared();
        if !n2.is_finite() || n2 == 0.0 {
            return Err(format!(
                "cuaternión sin inverso (norma² {n2}): [{}, {}, {}, {}]",
                q.w, q.x, q.y, q.z
            ));
        }
        Ok(q.conjugate().scale(1.0 / n2))
    }

    /// Construye la rotación de `angle_rad` alrededor de `axis`.
    ///
    /// El eje debe ser no-nulo y finito; el ángulo finito. Devuelve el
    /// cuaternión unitario `cos(θ/2) + sin(θ/2)·ê`.
    pub fn from_axis_angle(axis: [f64; 3], angle_rad: f64) -> Result<Self, String> {
        if !angle_rad.is_finite() {
            return Err(format!("ángulo no finito en from_axis_angle: {angle_rad}"));
        }
        let (ax, ay, az) = (axis[0], axis[1], axis[2]);
        if !ax.is_finite() || !ay.is_finite() || !az.is_finite() {
            return Err(format!(
                "eje no finito en from_axis_angle: [{ax}, {ay}, {az}]"
            ));
        }
        let len = (ax * ax + ay * ay + az * az).sqrt();
        if !len.is_finite() || len == 0.0 {
            return Err(format!("eje nulo en from_axis_angle: [{ax}, {ay}, {az}]"));
        }
        let half = 0.5 * angle_rad;
        let (s, c) = half.sin_cos();
        Ok(Self::new(c, s * ax / len, s * ay / len, s * az / len))
    }

    /// Rota `v` con `q·(0,v)·q⁻¹` (q debe ser unitario o normalizable).
    pub fn rotate_vector(self, v: [f64; 3]) -> Result<[f64; 3], String> {
        if !v[0].is_finite() || !v[1].is_finite() || !v[2].is_finite() {
            return Err(format!(
                "vector no finito en rotate_vector: [{}, {}, {}]",
                v[0], v[1], v[2]
            ));
        }
        let q = self.normalize()?;
        let pure = Self::new(0.0, v[0], v[1], v[2]);
        let inv = q.inverse()?;
        let r = q.hamilton_mul(pure).hamilton_mul(inv);
        Ok([r.x, r.y, r.z])
    }

    /// Matriz de rotación 3×3 (filas) del cuaternión normalizado.
    pub fn to_rotation_matrix(self) -> Result<[[f64; 3]; 3], String> {
        let q = self.normalize()?;
        let (w, x, y, z) = (q.w, q.x, q.y, q.z);
        Ok([
            [
                1.0 - 2.0 * (y * y + z * z),
                2.0 * (x * y - z * w),
                2.0 * (x * z + y * w),
            ],
            [
                2.0 * (x * y + z * w),
                1.0 - 2.0 * (x * x + z * z),
                2.0 * (y * z - x * w),
            ],
            [
                2.0 * (x * z - y * w),
                2.0 * (y * z + x * w),
                1.0 - 2.0 * (x * x + y * y),
            ],
        ])
    }

    /// Interpolación esférica `slerp(q0, q1, t)` con `t ∈ [0, 1]`.
    ///
    /// Toma el camino corto (niega `q1` si el dot es negativo) y cae a lerp
    /// normalizado cuando los cuaterniones son casi paralelos (`|dot| > 0.9995`,
    /// umbral clásico anti-`sin(Ω)≈0`).
    pub fn slerp(self, other: Self, t: f64) -> Result<Self, String> {
        if !t.is_finite() || !(0.0..=1.0).contains(&t) {
            return Err(format!("parámetro slerp fuera de [0, 1]: {t}"));
        }
        let q0 = self.normalize()?;
        let mut q1 = other.normalize()?;
        let mut dot = q0.dot(q1);
        if dot < 0.0 {
            q1 = q1.scale(-1.0);
            dot = -dot;
        }
        if dot > 0.9995 {
            // Régimen casi-paralelo: lerp + renormalización (estable).
            let lerped = q0.scale(1.0 - t).plus(q1.scale(t));
            return lerped.normalize();
        }
        let omega = dot.clamp(-1.0, 1.0).acos();
        let sin_omega = omega.sin();
        if !sin_omega.is_finite() || sin_omega == 0.0 {
            return Err(format!(
                "slerp degenerado (sin Ω = {sin_omega}, dot = {dot})"
            ));
        }
        let a = ((1.0 - t) * omega).sin() / sin_omega;
        let b = (t * omega).sin() / sin_omega;
        Ok(q0.scale(a).plus(q1.scale(b)))
    }

    /// Igualdad aproximada por componentes (para tests y call sites numéricos).
    #[must_use]
    pub fn approx_eq(self, other: Self, eps: f64) -> bool {
        (self.w - other.w).abs() <= eps
            && (self.x - other.x).abs() <= eps
            && (self.y - other.y).abs() <= eps
            && (self.z - other.z).abs() <= eps
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::{FRAC_PI_2, PI};

    const EPS: f64 = 1e-12;

    #[test]
    fn hamilton_ij_es_k() {
        assert!(Quaternion::i()
            .hamilton_mul(Quaternion::j())
            .approx_eq(Quaternion::k(), EPS));
    }

    #[test]
    fn hamilton_no_conmuta() {
        let ij = Quaternion::i().hamilton_mul(Quaternion::j());
        let ji = Quaternion::j().hamilton_mul(Quaternion::i());
        // j·i = −k = −(i·j): el signo distingue el orden.
        assert!(ji.approx_eq(Quaternion::k().scale(-1.0), EPS));
        assert!(!ij.approx_eq(ji, 1e-6));
    }

    #[test]
    fn identidad_es_neutro() {
        let q = Quaternion::new(1.0, 2.0, -3.0, 0.5);
        assert!(Quaternion::identity().hamilton_mul(q).approx_eq(q, EPS));
        assert!(q.hamilton_mul(Quaternion::identity()).approx_eq(q, EPS));
    }

    #[test]
    fn inverso_por_norma() {
        let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
        let inv = q.inverse().expect("inverso de cuaternión no-nulo");
        assert!(q.hamilton_mul(inv).approx_eq(Quaternion::identity(), 1e-9));
    }

    #[test]
    fn inverso_de_cero_es_error_honesto() {
        let err = Quaternion::new(0.0, 0.0, 0.0, 0.0)
            .inverse()
            .expect_err("cero no tiene inverso");
        assert!(err.contains("sin inverso"), "inesperado: {err}");
    }

    #[test]
    fn rotacion_90_z_lleva_x_a_y() {
        let q = Quaternion::from_axis_angle([0.0, 0.0, 1.0], FRAC_PI_2).expect("eje z válido");
        let r = q.rotate_vector([1.0, 0.0, 0.0]).expect("rotación válida");
        assert!((r[0]).abs() < 1e-12, "x residual: {r:?}");
        assert!((r[1] - 1.0).abs() < 1e-12, "y esperado 1: {r:?}");
        assert!((r[2]).abs() < 1e-12, "z residual: {r:?}");
    }

    #[test]
    fn eje_nulo_y_no_finito_son_error_honesto() {
        assert!(Quaternion::from_axis_angle([0.0, 0.0, 0.0], 1.0).is_err());
        assert!(Quaternion::from_axis_angle([f64::NAN, 0.0, 0.0], 1.0).is_err());
        assert!(Quaternion::from_axis_angle([0.0, 0.0, 1.0], f64::INFINITY).is_err());
    }

    #[test]
    fn slerp_extremos_y_punto_medio() {
        let q0 = Quaternion::identity();
        let q1 = Quaternion::from_axis_angle([0.0, 0.0, 1.0], PI).expect("giro π válido");
        let a = q0.slerp(q1, 0.0).expect("t=0");
        let b = q0.slerp(q1, 1.0).expect("t=1");
        assert!(a.approx_eq(q0.normalize().expect("norm"), 1e-9));
        // Identidad vs giro π: ortogonales en 4D (dot ≈ 0, no ±1); t=1 llega a q1.
        assert!(
            a.dot(b).abs() < 1e-9,
            "identidad vs giro π: dot={}",
            a.dot(b)
        );
        assert!(
            b.approx_eq(q1, 1e-9) || b.approx_eq(q1.scale(-1.0), 1e-9),
            "t=1 llega a q1 (salvo signo): {b:?} vs {q1:?}"
        );
        // Punto medio de 0→π = giro de π/2: rota x→y como el test de rotación.
        let mid = q0.slerp(q1, 0.5).expect("t=0.5");
        let r = mid.rotate_vector([1.0, 0.0, 0.0]).expect("rotación media");
        assert!(
            (r[0]).abs() < 1e-9 && (r[1] - 1.0).abs() < 1e-9,
            "medio: {r:?}"
        );
        assert!(q0.slerp(q1, 2.0).is_err());
    }

    #[test]
    fn matriz_identidad_del_unitario() {
        let m = Quaternion::identity()
            .to_rotation_matrix()
            .expect("matriz identidad");
        for (r, row) in m.iter().enumerate() {
            for (c, v) in row.iter().enumerate() {
                let want = f64::from(r == c);
                assert!((v - want).abs() < EPS, "m[{r}][{c}] = {v}");
            }
        }
    }
}
