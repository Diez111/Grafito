//! Ejercicios — generación y validación, sin red.
//!
//! # Cómo seguir con el resto de las familias (patrón sembrado)
//!
//! `prob-distribuciones` es la primera familia sembrada (el `match
//! lo.id.as_str()` de `ExerciseGenerator::generate_with_seed`); los ~40 LOs
//! restantes hoy caen al genérico `_ =>` ("Si f(x)=a·x+b, evalúa en x=c"),
//! que es pedagógicamente incorrecto para `alg-matrices`, `am2-edo`, etc.
//! Para sumar una familia, repetir el patrón:
//!
//! 1. Un brazo `"lo-id" => { ... }` en el `match`, ANTES del `_ =>`.
//! 2. Parámetros 100 % derivados de la seed vía `hash_con_sal` (`h0`,
//!    `h1`, `h2`, un flujo con sal distinta por parámetro): determinismo
//!    puro, sin `rand`, sin reloj.
//! 3. `solution` derivada SIEMPRE de los parámetros (idealmente exacta:
//!    fracción reducida o entero), jamás hardcodeada desincronizada.
//! 4. `kind` coherente con la respuesta: `Numeric` si la respuesta es un
//!    número, `Symbolic` si es expresión, `Graphical` solo si pide un dibujo.
//! 5. `validator`: `NumericTol(0.02)` para números (acepta fracción o
//!    decimal), `Exact` para texto corto.
//! 6. Tests mínimos por familia: determinismo por semilla (misma seed ⇒ mismo
//!    ejercicio), coherencia de `kind`/`validator`, `validate().is_ok()` y
//!    auto-evaluación `FeedbackEngine::assess(&ex, &ex.solution).correct`.
//!
//! Familias sugeridas por esfuerzo: `alg-matrices` (determinante 2×2 con
//! coeficientes sembrados), `am2-edo` (separable con constante sembrada),
//! `prob-var` (esperanza de una VA discreta), `sec-fracc` (suma de
//! fracciones con denominadores sembrados).

use crate::curriculum::LearningObjective;
use crate::level::PedagogicalLevel;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExerciseKind {
    Numeric,
    Symbolic,
    Graphical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExerciseDifficulty {
    Easy,
    Medium,
    Hard,
}

/// Estrategia de validación de la respuesta.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum ValidatorKind {
    /// Comparación exacta tras normalizar.
    #[default]
    Exact,
    /// Tolerancia numérica relativa (ej 0.02 = 2 %).
    NumericTol(f64),
    /// Validación simbólica (requiere CAS).
    Symbolic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exercise {
    pub prompt: String,
    pub solution: String,
    pub kind: ExerciseKind,
    pub difficulty: ExerciseDifficulty,
    pub lo_id: String,
    #[serde(default)]
    pub params: BTreeMap<String, f64>,
    #[serde(default)]
    pub seed: Option<u64>,
    #[serde(default)]
    pub validator: ValidatorKind,
}

impl Exercise {
    pub fn validate(&self) -> Result<(), String> {
        if self.prompt.trim().is_empty() || self.solution.trim().is_empty() {
            return Err("ejercicio incompleto".into());
        }
        if self.prompt.len() > 500 || self.solution.len() > 500 {
            return Err("ejercicio demasiado largo".into());
        }
        match self.validator {
            ValidatorKind::NumericTol(tol) => {
                if !tol.is_finite() || tol <= 0.0 || tol > 1.0 {
                    return Err("tolerancia inválida".into());
                }
            }
            ValidatorKind::Exact | ValidatorKind::Symbolic => {}
        }
        for (k, v) in &self.params {
            if k.trim().is_empty() {
                return Err("clave de parámetro vacía".into());
            }
            if !v.is_finite() {
                return Err(format!("parámetro '{k}' no finito"));
            }
        }
        Ok(())
    }
}

/// Generador determinista — mapea LO + nivel + seed a ejercicio.
///
/// # Dificultad por nivel pedagógico (no solo `level_value` numérico)
///
/// | `PedagogicalLevel` | `ExerciseDifficulty` | `level_value` | Criterio |
/// |---|---|---|---|
/// | `Primary` | `Easy` | 2 | Operaciones con enteros 1..5, sin fracciones complejas |
/// | `Secondary` | `Medium` | 8 | Incluye fracciones, trigonometría discreta (4 variantes sin) |
/// | `University` | `Hard` | 15 | Parámetros simbólicos, tolerancia numérica 2% |
/// | `UTN(AM1)` | `Hard` | 12 | Cálculo: derivadas/integrales con `NumericTol(0.02)` |
/// | `UTN(AM2)` | `Hard` | 14 | Series/Taylor, multivariable |
/// | `UTN(Algebra)` | `Hard` | 13 | Matrices, transformaciones |
/// | `UTN(Probabilidad)` | `Hard` | 15 | Distribuciones, inferencia |
///
/// Nota: `level_value` colisionaba históricamente en 14 (AM2 vs Probabilidad);
/// la dificultad se decide por `match level` (variante enum), no por el
/// número crudo, por eso `Probabilidad` y `AM2` ambas son `Hard` aunque
/// ahora difieran en `level_value` (14 vs 15).
#[derive(Debug, Clone, Default)]
pub struct ExerciseGenerator;

const SAL_DORADA: u64 = 0x9E3779B97F4A7C15;

/// Deriva un `u64` pseudoaleatorio de `(seed, sal)` con avalancha completa.
///
/// Cada parámetro del ejercicio (`h0`, `h1`, `h2`) usa una sal distinta, así
/// los parámetros son independientes entre sí y entre semillas consecutivas.
/// La cadena lineal anterior (`h1 = h0*K+C`, `h2 = h1*K`) dejaba `(a, b)`
/// correlacionados: `am1-der` repetía ejercicio en 35/64 semillas consecutivas
/// con solo 9 combinaciones posibles (lo esperado es ~7/64). Con flujos con
/// sal + finalizador estilo splitmix64, la tasa cae a ~5/64. Determinista
/// puro: misma `(seed, sal)` ⇒ mismo valor, sin `rand`, sin reloj.
fn hash_con_sal(seed: u64, sal: u64) -> u64 {
    let mut z = seed
        .wrapping_add(sal.wrapping_mul(SAL_DORADA))
        .wrapping_add(SAL_DORADA);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

/// Combinatoria `C(n, k)` en `u64` (los generadores la usan con `n ≤ 6`:
/// sin overflow posible). `k` se acota a `min(k, n-k)` para multiplicar menos.
fn combinatoria(n: u64, k: u64) -> u64 {
    let k = k.min(n.saturating_sub(k));
    let mut num = 1u64;
    let mut den = 1u64;
    for i in 0..k {
        num = num.saturating_mul(n.saturating_sub(i));
        den = den.saturating_mul(i.saturating_add(1));
    }
    if den == 0 {
        return 0;
    }
    num / den
}

/// Máximo común divisor (Euclides, saturante).
fn mcd(a: u64, b: u64) -> u64 {
    let (mut a, mut b) = (a, b);
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

/// Reduce la fracción `num/den` a términos mínimos (`den == 0` se devuelve
/// sin tocar: inalcanzable desde los generadores, que usan `den ∈ {2, 4}`).
fn reducir_fraccion(num: u64, den: u64) -> (u64, u64) {
    if den == 0 {
        return (num, den);
    }
    let d = mcd(num, den).max(1);
    (num / d, den / d)
}

impl ExerciseGenerator {
    pub fn generate(&self, lo: &LearningObjective, level: PedagogicalLevel) -> Exercise {
        self.generate_with_seed(lo, level, 0)
    }

    pub fn generate_with_seed(
        &self,
        lo: &LearningObjective,
        level: PedagogicalLevel,
        seed: u64,
    ) -> Exercise {
        let difficulty = match level {
            PedagogicalLevel::Primary => ExerciseDifficulty::Easy,
            PedagogicalLevel::Secondary => ExerciseDifficulty::Medium,
            _ => ExerciseDifficulty::Hard,
        };

        // Helpers para coeficientes deterministas: un flujo con sal por
        // parámetro (independientes entre sí y entre semillas consecutivas).
        let h0 = hash_con_sal(seed, 0);
        let h1 = hash_con_sal(seed, 1);
        let h2 = hash_con_sal(seed, 2);

        let (prompt, solution, kind, validator, params) = match lo.id.as_str() {
            "am1-der" => {
                // a y b en 1..=3 desde flujos independientes (h0, h1).
                let a = 1 + (h0 % 3);
                let b = 1 + (h1 % 3);
                let prompt = format!("Deriva f(x)={}*x^2 + {}*x en x=1", a, b);
                let sol_val = 2 * a + b;
                let solution = sol_val.to_string();
                let mut params = BTreeMap::new();
                params.insert("a".to_string(), a as f64);
                params.insert("b".to_string(), b as f64);
                (
                    prompt,
                    solution,
                    ExerciseKind::Symbolic,
                    ValidatorKind::NumericTol(0.02),
                    params,
                )
            }
            "am1-int" => {
                let a = 1 + (h0 % 3);
                let prompt = format!("Calcula ∫₀¹ {}*x^2 dx", a);
                let val = a as f64 / 3.0;
                // Solución con 10 decimales recortados, tolerancia 2% permite fracciones
                let solution = if a.is_multiple_of(3) {
                    (val as i64).to_string()
                } else {
                    // Para a=1 => 0.3333333333, para a=2 => 0.666...
                    // Dejamos representación decimal completa para parse numérico
                    format!("{val}")
                };
                let mut params = BTreeMap::new();
                params.insert("a".to_string(), a as f64);
                (
                    prompt,
                    solution,
                    ExerciseKind::Symbolic,
                    ValidatorKind::NumericTol(0.02),
                    params,
                )
            }
            "sec-trig" => {
                let k = seed % 4;
                let (prompt, solution) = match k {
                    0 => ("¿Cuánto vale sin(0)?".to_string(), "0".to_string()),
                    1 => ("¿Cuánto vale sin(π/2)?".to_string(), "1".to_string()),
                    2 => ("¿Cuánto vale sin(π)?".to_string(), "0".to_string()),
                    3 => ("¿Cuánto vale sin(3·π/2)?".to_string(), "-1".to_string()),
                    _ => ("¿Cuánto vale sin(0)?".to_string(), "0".to_string()),
                };
                let mut params = BTreeMap::new();
                params.insert("k".to_string(), k as f64);
                // también guardamos ángulo en radianes
                let angle = k as f64 * std::f64::consts::FRAC_PI_2;
                params.insert("angle_rad".to_string(), angle);
                (
                    prompt,
                    solution,
                    ExerciseKind::Numeric,
                    ValidatorKind::NumericTol(0.02),
                    params,
                )
            }
            "alg-subespacios" => {
                // 3 variantes por seed: dimensión del span, coordenada de
                // combinación lineal, independencia (1=sí, 0=no).
                let mut params = BTreeMap::new();
                match seed % 3 {
                    0 => {
                        params.insert("dim".to_string(), 2.0);
                        (
                            "Si u=(1,0) y v=(0,1), ¿cuál es la dimensión del span(u,v)?"
                                .to_string(),
                            "2".to_string(),
                            ExerciseKind::Numeric,
                            ValidatorKind::NumericTol(0.02),
                            params,
                        )
                    }
                    1 => {
                        let a = 1 + (h0 % 3);
                        let b = 1 + (h1 % 3);
                        params.insert("a".to_string(), a as f64);
                        params.insert("b".to_string(), b as f64);
                        let prompt = format!(
                            "Si u=(1,1) y v=(1,-1), calculá la primera coordenada de {a}*u+{b}*v"
                        );
                        let solution = (a + b).to_string();
                        (
                            prompt,
                            solution,
                            ExerciseKind::Numeric,
                            ValidatorKind::NumericTol(0.02),
                            params,
                        )
                    }
                    _ => {
                        params.insert("independientes".to_string(), 0.0);
                        (
                            "¿Son linealmente independientes (1,0) y (2,0)? Respondé 1=sí, 0=no"
                                .to_string(),
                            "0".to_string(),
                            ExerciseKind::Numeric,
                            ValidatorKind::NumericTol(0.02),
                            params,
                        )
                    }
                }
            }
            "sec-fractales" => {
                // 3 variantes por seed: Koch tras 1 y 2 iteraciones,
                // Sierpinski tras 2 iteraciones.
                let mut params = BTreeMap::new();
                match seed % 3 {
                    0 => {
                        params.insert("base".to_string(), 3.0);
                        params.insert("factor".to_string(), 4.0);
                        params.insert("iter".to_string(), 1.0);
                        (
                            "El copo de Koch parte de 3 segmentos y cada iteración multiplica por 4. ¿Cuántos segmentos hay tras 1 iteración?".to_string(),
                            "12".to_string(),
                            ExerciseKind::Numeric,
                            ValidatorKind::NumericTol(0.02),
                            params,
                        )
                    }
                    1 => {
                        params.insert("base".to_string(), 3.0);
                        params.insert("factor".to_string(), 4.0);
                        params.insert("iter".to_string(), 2.0);
                        (
                            "El copo de Koch parte de 3 segmentos y cada iteración multiplica por 4. ¿Cuántos segmentos hay tras 2 iteraciones?".to_string(),
                            "48".to_string(),
                            ExerciseKind::Numeric,
                            ValidatorKind::NumericTol(0.02),
                            params,
                        )
                    }
                    _ => {
                        params.insert("base".to_string(), 1.0);
                        params.insert("factor".to_string(), 3.0);
                        params.insert("iter".to_string(), 2.0);
                        (
                            "El triángulo de Sierpinski parte de 1 triángulo y cada iteración multiplica por 3. ¿Cuántos triángulos hay tras 2 iteraciones?".to_string(),
                            "9".to_string(),
                            ExerciseKind::Numeric,
                            ValidatorKind::NumericTol(0.02),
                            params,
                        )
                    }
                }
            }
            "prob-distribuciones" => {
                // Familia sembrada (patrón para el resto, ver doc del módulo):
                // binomial B(n, p) con parámetros 100 % derivados de la seed.
                // Respuesta EXACTA como fracción reducida (el validador
                // `NumericTol` acepta fracción o decimal equivalente).
                let n = 2 + (h0 % 5); // ensayos 2..=6
                let k = h1 % (n + 1); // éxitos 0..=n
                let (p_num, p_den): (u64, u64) = match h2 % 3 {
                    0 => (1, 4),
                    1 => (1, 2),
                    _ => (3, 4),
                };
                let prompt = format!(
                    "Sea X ~ B({n}, {p_num}/{p_den}). ¿Cuánto vale P(X = {k})? Respondé como fracción o decimal."
                );
                // P(X=k) = C(n,k)·p^k·(1-p)^(n-k) en aritmética exacta sobre
                // p_den^n (n ≤ 6 ⇒ p_den^n ≤ 4^6 = 4096: todo entra en u64).
                let comb = combinatoria(n, k);
                let num = comb
                    .saturating_mul(p_num.pow(k as u32))
                    .saturating_mul((p_den - p_num).pow((n - k) as u32));
                let den = p_den.pow(n as u32);
                let (num, den) = reducir_fraccion(num, den);
                let solution = if den == 1 {
                    num.to_string()
                } else {
                    format!("{num}/{den}")
                };
                let mut params = BTreeMap::new();
                params.insert("n".to_string(), n as f64);
                params.insert("k".to_string(), k as f64);
                params.insert("p".to_string(), p_num as f64 / p_den as f64);
                (
                    prompt,
                    solution,
                    ExerciseKind::Numeric,
                    ValidatorKind::NumericTol(0.02),
                    params,
                )
            }
            "fis1-cinematica" => {
                // MRU: x = v·t con v=2..6 m/s, t=2..9 s sembrados. Exacto entero.
                let v = 2 + (h0 % 5);
                let t = 2 + (h1 % 8);
                let prompt =
                    format!("Un móvil va a MRU a {v} m/s. ¿Qué distancia recorre en {t} s?");
                let solution = (v * t).to_string();
                let mut params = BTreeMap::new();
                params.insert("v".to_string(), v as f64);
                params.insert("t".to_string(), t as f64);
                (
                    prompt,
                    solution,
                    ExerciseKind::Numeric,
                    ValidatorKind::NumericTol(0.02),
                    params,
                )
            }
            "dis-combinatoria" => {
                // C(n,k) con n=4..6, k=2..3 sembrados. Exacto vía combinatoria().
                let n = 4 + (h0 % 3);
                let k = 2 + (h1 % 2).min(n - 2);
                let prompt =
                    format!("¿Cuántos subconjuntos de {k} elementos tiene un conjunto de {n}?");
                let solution = combinatoria(n, k).to_string();
                let mut params = BTreeMap::new();
                params.insert("n".to_string(), n as f64);
                params.insert("k".to_string(), k as f64);
                (
                    prompt,
                    solution,
                    ExerciseKind::Numeric,
                    ValidatorKind::NumericTol(0.02),
                    params,
                )
            }
            "fis-resonancia" => {
                // f₀=1/2π√(LC): L=10..14 mH, C=1..5 µF sembrados.
                // Puente con ResonanciaRLC: el tutor pide f₀ y el
                // laboratorio la dibuja.
                let l_mh = 10 + (h0 % 5);
                let c_uf = 1 + (h1 % 5);
                let l = l_mh as f64 * 1e-3;
                let c = c_uf as f64 * 1e-6;
                let f0 = 1.0 / (2.0 * std::f64::consts::PI * (l * c).sqrt());
                let prompt = format!("Un RLC serie tiene L={l_mh} mH y C={c_uf} µF. ¿f₀ en Hz?");
                let solution = format!("{f0:.2}");
                let mut params = BTreeMap::new();
                params.insert("l_mh".to_string(), l_mh as f64);
                params.insert("c_uf".to_string(), c_uf as f64);
                (
                    prompt,
                    solution,
                    ExerciseKind::Numeric,
                    ValidatorKind::NumericTol(0.02),
                    params,
                )
            }
            "fis-doppler" => {
                // f'=f·c/(c−v): v=10..50 m/s, f=400..440 Hz, c=340.
                // Puente con EfectoDoppler: el tutor pide f' y el
                // laboratorio dibuja los frentes.
                let v = 10 + (h0 % 5) * 10;
                let f = 400 + (h1 % 5) * 10;
                const C: f64 = 340.0;
                let fp = f as f64 * C / (C - v as f64);
                let prompt = format!("Una sirena a {f} Hz se acerca a {v} m/s (c=340). ¿f' en Hz?");
                let solution = format!("{fp:.1}");
                let mut params = BTreeMap::new();
                params.insert("v".to_string(), v as f64);
                params.insert("f".to_string(), f as f64);
                (
                    prompt,
                    solution,
                    ExerciseKind::Numeric,
                    ValidatorKind::NumericTol(0.05),
                    params,
                )
            }
            "fis-pendulo" => {
                // T₀=2π√(L/g): L=1..5 m sembrados, g=9.8.
                // Puente con Pendulo: el tutor pide T₀ y el
                // laboratorio dibuja la lenteja y corrige amplitud.
                let l = 1 + (h0 % 5);
                let t0 = 2.0 * std::f64::consts::PI * (l as f64 / 9.8).sqrt();
                let prompt = format!("Un péndulo simple de L={l} m oscila poco. ¿T₀ en s?");
                let solution = format!("{t0:.2}");
                let mut params = BTreeMap::new();
                params.insert("l".to_string(), l as f64);
                (
                    prompt,
                    solution,
                    ExerciseKind::Numeric,
                    ValidatorKind::NumericTol(0.02),
                    params,
                )
            }
            "fis-energia" => {
                // Em=½mv²+mgh: m=1..3 kg, v=2..5 m/s, h=1..5 m, g=9.8.
                // Puente con EnergiaMecanica: el tutor pide Em y el
                // laboratorio verifica la cuenta del plano.
                let m = 1 + (h0 % 3);
                let v = 2 + (h1 % 4);
                let h = 1 + (h0 % 5 + h1 % 5) % 5;
                let em = 0.5 * m as f64 * (v as f64).powi(2) + m as f64 * 9.8 * h as f64;
                let prompt = format!(
                    "Un cuerpo de {m} kg se mueve a {v} m/s a {h} m de altura. ¿Em en J? (g=9.8)"
                );
                let solution = format!("{em:.1}");
                let mut params = BTreeMap::new();
                params.insert("m".to_string(), m as f64);
                params.insert("v".to_string(), v as f64);
                params.insert("h".to_string(), h as f64);
                (
                    prompt,
                    solution,
                    ExerciseKind::Numeric,
                    ValidatorKind::NumericTol(0.05),
                    params,
                )
            }
            "alg-matrices" => {
                let a = 1 + (h0 % 4);
                let b = 1 + (h1 % 4);
                let c = 1 + (h2 % 4);
                let d = 1 + (hash_con_sal(seed, 3) % 4);
                let prompt = format!("Sea A = [[{a}, {b}], [{c}, {d}]]. ¿Cuánto vale det(A)?");
                let det = (a * d) as i64 - (b * c) as i64;
                let solution = det.to_string();
                let mut params = BTreeMap::new();
                params.insert("a".to_string(), a as f64);
                params.insert("b".to_string(), b as f64);
                params.insert("c".to_string(), c as f64);
                params.insert("d".to_string(), d as f64);
                (
                    prompt,
                    solution,
                    ExerciseKind::Numeric,
                    ValidatorKind::NumericTol(0.02),
                    params,
                )
            }
            "am2-edo" => {
                let k = 1 + (h0 % 2);
                let y0 = 1 + (h1 % 3);
                let prompt =
                    format!("Resolvé y'={k}*y con y(0)={y0}. ¿Cuánto vale y(1)? (usá e≈2.71828)");
                let val = y0 as f64 * (k as f64).exp();
                let solution = format!("{val:.4}");
                let mut params = BTreeMap::new();
                params.insert("k".to_string(), k as f64);
                params.insert("y0".to_string(), y0 as f64);
                (
                    prompt,
                    solution,
                    ExerciseKind::Numeric,
                    ValidatorKind::NumericTol(0.02),
                    params,
                )
            }
            "prob-var" => {
                let off = (h0 % 2) as i64;
                let v1 = 1 + off;
                let v2 = 2 + off;
                let v3 = 3 + off;
                let prompt = format!("Sea X con P(X={v1})=1/2, P(X={v2})=1/4, P(X={v3})=1/4. ¿E[X]? (fracción o decimal)");
                let num = 2 * v1 + v2 + v3;
                let (num_r, den_r) = reducir_fraccion(num as u64, 4);
                let solution = if den_r == 1 {
                    num_r.to_string()
                } else {
                    format!("{num_r}/{den_r}")
                };
                let mut params = BTreeMap::new();
                params.insert("off".to_string(), off as f64);
                (
                    prompt,
                    solution,
                    ExerciseKind::Numeric,
                    ValidatorKind::NumericTol(0.02),
                    params,
                )
            }
            "sec-fracc" => {
                let n1 = 1 + (h0 % 3);
                let d1 = 2 + (h1 % 3);
                let n2 = 1 + (h2 % 3);
                let d2 = 2 + (hash_con_sal(seed, 3) % 3);
                let prompt =
                    format!("Calculá {n1}/{d1} + {n2}/{d2}. Respondé como fracción reducida.");
                let num = n1 * d2 + n2 * d1;
                let den = d1 * d2;
                let (num_r, den_r) = reducir_fraccion(num, den);
                let solution = if den_r == 1 {
                    num_r.to_string()
                } else {
                    format!("{num_r}/{den_r}")
                };
                let mut params = BTreeMap::new();
                params.insert("n1".to_string(), n1 as f64);
                params.insert("d1".to_string(), d1 as f64);
                params.insert("n2".to_string(), n2 as f64);
                params.insert("d2".to_string(), d2 as f64);
                (
                    prompt,
                    solution,
                    ExerciseKind::Numeric,
                    ValidatorKind::NumericTol(0.02),
                    params,
                )
            }
            "sec-ec" => {
                // Cuadrática con raíces enteras (a=1): la secundaria pide
                // factorizar x²; se responde con la raíz mayor (validable
                // numéricamente por la planilla del ejercicio).
                let signo1 = if (h0 % 2) == 1 { -1i64 } else { 1i64 };
                let mut r1 = signo1 * (1 + (h1 % 6)) as i64;
                let signo2 = if (h2 % 2) == 1 { -1i64 } else { 1i64 };
                let mut r2 = signo2 * (1 + (hash_con_sal(seed, 3) % 6)) as i64;
                if r1 > r2 {
                    std::mem::swap(&mut r1, &mut r2);
                }
                let b = -(r1 + r2);
                let c = r1 * r2;
                let bs = if b >= 0 {
                    format!("+ {b}*x")
                } else {
                    format!("- {}*x", -b)
                };
                let cs = if c >= 0 {
                    format!("+ {c}")
                } else {
                    format!("- {}", -c)
                };
                let prompt = format!("Resolvé x^2 {bs} {cs} = 0. Escribí SOLO la raíz mayor.");
                let solution = r2.to_string();
                let mut params = BTreeMap::new();
                params.insert("a".to_string(), 1.0);
                params.insert("b".to_string(), b as f64);
                params.insert("c".to_string(), c as f64);
                (
                    prompt,
                    solution,
                    ExerciseKind::Numeric,
                    ValidatorKind::NumericTol(0.02),
                    params,
                )
            }
            "sec-pitagoras" => {
                // Tríos pitagóricos canónicos: la secundaria practica c² = a² + b².
                const TRIOS: [(u64, u64, u64); 6] = [
                    (3, 4, 5),
                    (6, 8, 10),
                    (5, 12, 13),
                    (9, 12, 15),
                    (8, 15, 17),
                    (12, 16, 20),
                ];
                let (c1, c2, h) = TRIOS[(h0 % 6) as usize];
                let (a, b) = if (h1 % 2) == 1 { (c2, c1) } else { (c1, c2) };
                let prompt = format!(
                    "Triángulo rectángulo de catetos {a} y {b}. ¿Cuánto mide la hipotenusa?"
                );
                let solution = h.to_string();
                let mut params = BTreeMap::new();
                params.insert("cateto1".to_string(), a as f64);
                params.insert("cateto2".to_string(), b as f64);
                params.insert("hipotenusa".to_string(), h as f64);
                (
                    prompt,
                    solution,
                    ExerciseKind::Numeric,
                    ValidatorKind::NumericTol(0.02),
                    params,
                )
            }
            _ => {
                // Genérico paramétrico: Si f(x)=a*x+b, evalúa en x=c
                // a,b,c en 1..5 desde flujos independientes (h0, h1, h2).
                let a = 1 + (h0 % 5);
                let b = 1 + (h1 % 5);
                let c = 1 + (h2 % 5);
                let prompt = format!("Si f(x)={}*x+{}, evalúa en x={}", a, b, c);
                let sol = a * c + b;
                let solution = sol.to_string();
                let mut params = BTreeMap::new();
                params.insert("a".to_string(), a as f64);
                params.insert("b".to_string(), b as f64);
                params.insert("c".to_string(), c as f64);
                // para LOs conocidos, ajustamos prompt levemente para variedad
                let (prompt, kind) = match lo.id.as_str() {
                    "am1-func" => (prompt, ExerciseKind::Numeric),
                    "am1-lim" => {
                        // Variante límite: usa mismo a,b,c pero frasea como límite
                        let p =
                            format!("Si f(x)={}*x+{}, ¿cuánto vale lim_{{x→{}}} f(x)?", a, b, c);
                        (p, ExerciseKind::Symbolic)
                    }
                    "am1-cont" => (prompt, ExerciseKind::Symbolic),
                    "am1-der-aplic" | "am1-int-aplic" | "am1-sucesiones" => {
                        (prompt, ExerciseKind::Numeric)
                    }
                    id if id.starts_with("sec-") => (prompt, ExerciseKind::Numeric),
                    _ => (prompt, ExerciseKind::Graphical),
                };
                // Validator: numérico si solución es numérica
                let validator = ValidatorKind::NumericTol(0.02);
                (prompt, solution, kind, validator, params)
            }
        };

        Exercise {
            prompt,
            solution,
            kind,
            difficulty,
            lo_id: lo.id.clone(),
            params,
            seed: Some(seed),
            validator,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generate_valid() {
        let lo = LearningObjective::new("am1-der", "Derivadas", "...", None);
        let ex = ExerciseGenerator.generate(&lo, PedagogicalLevel::Secondary);
        assert!(ex.validate().is_ok());
    }

    #[test]
    fn generate_with_seed_deterministic() {
        let lo = LearningObjective::new("am1-der", "Derivadas", "...", None);
        let gen = ExerciseGenerator;
        let ex1 = gen.generate_with_seed(&lo, PedagogicalLevel::Secondary, 42);
        let ex2 = gen.generate_with_seed(&lo, PedagogicalLevel::Secondary, 42);
        assert_eq!(ex1.prompt, ex2.prompt);
        assert_eq!(ex1.solution, ex2.solution);
        assert_eq!(ex1.seed, Some(42));
        assert_eq!(ex1.params, ex2.params);
    }

    #[test]
    fn generate_with_seed_varies() {
        let lo = LearningObjective::new("am1-der", "Derivadas", "...", None);
        let gen = ExerciseGenerator;
        let ex0 = gen.generate_with_seed(&lo, PedagogicalLevel::Secondary, 0);
        let ex1 = gen.generate_with_seed(&lo, PedagogicalLevel::Secondary, 1);
        // con flujos con sal deberían diferir en al menos prompt o params
        assert!(ex0.prompt != ex1.prompt || ex0.params != ex1.params);
    }

    #[test]
    fn generate_seed_zero_is_generate() {
        let lo = LearningObjective::new("am1-der", "Derivadas", "...", None);
        let gen = ExerciseGenerator;
        let a = gen.generate(&lo, PedagogicalLevel::Secondary);
        let b = gen.generate_with_seed(&lo, PedagogicalLevel::Secondary, 0);
        assert_eq!(a.prompt, b.prompt);
        assert_eq!(a.solution, b.solution);
    }

    #[test]
    fn am1_der_coefficients_and_solution() {
        let lo = LearningObjective::new("am1-der", "Derivadas", "...", None);
        let gen = ExerciseGenerator;
        for seed in 0..10u64 {
            let ex = gen.generate_with_seed(&lo, PedagogicalLevel::Secondary, seed);
            assert!(ex.prompt.contains("Deriva"));
            assert!(ex.params.contains_key("a"));
            assert!(ex.params.contains_key("b"));
            let a = ex.params["a"];
            let b = ex.params["b"];
            assert!((1.0..=3.0).contains(&a));
            assert!((1.0..=3.0).contains(&b));
            let expected = 2.0 * a + b;
            let sol: f64 = ex.solution.parse().unwrap_or(f64::NAN);
            assert!(
                (sol - expected).abs() < 1e-9,
                "seed {seed} sol {sol} expected {expected}"
            );
            match ex.validator {
                ValidatorKind::NumericTol(t) => assert!((t - 0.02).abs() < 1e-9),
                _ => panic!("validator debe ser NumericTol"),
            }
        }
    }

    #[test]
    fn am1_int_solution() {
        let lo = LearningObjective::new("am1-int", "Integrales", "...", None);
        let gen = ExerciseGenerator;
        let ex = gen.generate_with_seed(&lo, PedagogicalLevel::Secondary, 0);
        assert!(ex.prompt.contains("∫₀¹"));
        let a = ex.params["a"];
        let sol: f64 = ex.solution.parse().unwrap_or_else(|_| {
            // si es fracción a/3 como "1/3", parse manual
            if ex.solution.contains('/') {
                let parts: Vec<&str> = ex.solution.split('/').collect();
                parts[0].parse::<f64>().unwrap() / parts[1].parse::<f64>().unwrap()
            } else {
                f64::NAN
            }
        });
        let expected = a / 3.0;
        assert!((sol - expected).abs() < 1e-9);
        assert!(matches!(ex.validator, ValidatorKind::NumericTol(_)));
    }

    #[test]
    fn sec_ec_family_criteria_y_determinismo() {
        let lo = LearningObjective::new("sec-ec", "Ecuaciones", "...", None);
        let gen = ExerciseGenerator;
        for seed in 0..10u64 {
            let ex = gen.generate_with_seed(&lo, PedagogicalLevel::Secondary, seed);
            assert!(ex.prompt.contains("x^2"), "seed {seed}: {:?}", ex.prompt);
            assert!(ex.prompt.contains("raíz mayor"));
            let b = ex.params["b"];
            let c = ex.params["c"];
            // Vieta: b = -(r1+r2), c = r1*r2 con raíces enteras ⇒ discrim cuadrado.
            let d = b * b - 4.0 * c;
            let raiz = d.sqrt();
            assert!(
                (raiz - raiz.round()).abs() < 1e-9,
                "seed {seed}: discriminante {d} no es cuadrado perfecto"
            );
            let mayor = (-b + raiz) / 2.0;
            let sol: f64 = ex.solution.parse().unwrap();
            assert!(
                (sol - mayor).abs() < 1e-6,
                "seed {seed}: sol {sol} != raíz mayor {mayor} (D={d})"
            );
            assert!(matches!(ex.validator, ValidatorKind::NumericTol(_)));
            assert!(crate::FeedbackEngine.assess(&ex, &ex.solution).correct);
            // Determinismo por semilla.
            let ex2 = gen.generate_with_seed(&lo, PedagogicalLevel::Secondary, seed);
            assert_eq!(ex.prompt, ex2.prompt);
        }
    }

    #[test]
    fn sec_pitagoras_family_trios_exactos() {
        let lo = LearningObjective::new("sec-pitagoras", "Pitágoras", "...", None);
        let gen = ExerciseGenerator;
        let mut visto: Vec<(f64, f64, f64)> = Vec::new();
        for seed in 0..24u64 {
            let ex = gen.generate_with_seed(&lo, PedagogicalLevel::Secondary, seed);
            assert!(ex.prompt.contains("hipotenusa"));
            let h = ex.params["hipotenusa"];
            let a = ex.params["cateto1"];
            let b = ex.params["cateto2"];
            assert!((h * h - a * a - b * b).abs() < 1e-9, "seed {seed}");
            let sol: f64 = ex.solution.parse().unwrap();
            assert_eq!(sol, h, "seed {seed}: solución = hipotenusa");
            assert!(matches!(ex.validator, ValidatorKind::NumericTol(_)));
            assert!(crate::FeedbackEngine.assess(&ex, &ex.solution).correct);
            if !visto.iter().any(|&(x, y, z)| x == a && y == b && z == h) {
                visto.push((a, b, h));
            }
        }
        assert!(visto.len() >= 5, "cobertura de tríos: {visto:?}");
    }

    #[test]
    fn sec_trig_variants() {
        let lo = LearningObjective::new("sec-trig", "Trigonometría", "...", None);
        let gen = ExerciseGenerator;
        let expectations = [0.0, 1.0, 0.0, -1.0];
        for k in 0..4u64 {
            let ex = gen.generate_with_seed(&lo, PedagogicalLevel::Secondary, k);
            let sol: f64 = ex.solution.parse().unwrap();
            assert_eq!(sol, expectations[k as usize]);
            assert!(ex.prompt.contains("sin"));
            assert!(matches!(ex.validator, ValidatorKind::NumericTol(_)));
        }
    }

    #[test]
    fn generic_parametric() {
        let lo = LearningObjective::new("am1-func", "Funciones", "...", None);
        let gen = ExerciseGenerator;
        let ex = gen.generate_with_seed(&lo, PedagogicalLevel::Secondary, 7);
        assert!(ex.prompt.contains("f(x)"));
        assert!(ex.params.contains_key("a"));
        assert!(ex.params.contains_key("b"));
        assert!(ex.params.contains_key("c"));
        let a = ex.params["a"];
        let b = ex.params["b"];
        let c = ex.params["c"];
        let expected = a * c + b;
        let sol: f64 = ex.solution.parse().unwrap();
        assert!((sol - expected).abs() < 1e-9);
        assert_eq!(ex.seed, Some(7));
    }

    #[test]
    fn validate_rejects_bad_validator() {
        let mut ex = ExerciseGenerator.generate(
            &LearningObjective::new("am1-der", "D", "...", None),
            PedagogicalLevel::Secondary,
        );
        ex.validator = ValidatorKind::NumericTol(f64::NAN);
        assert!(ex.validate().is_err());
        ex.validator = ValidatorKind::NumericTol(0.0);
        assert!(ex.validate().is_err());
        ex.validator = ValidatorKind::NumericTol(2.0);
        assert!(ex.validate().is_err());
        ex.validator = ValidatorKind::NumericTol(0.02);
        assert!(ex.validate().is_ok());
    }

    #[test]
    fn validate_rejects_non_finite_param() {
        let mut ex = ExerciseGenerator.generate(
            &LearningObjective::new("am1-der", "D", "...", None),
            PedagogicalLevel::Secondary,
        );
        ex.params.insert("a".to_string(), f64::INFINITY);
        assert!(ex.validate().is_err());
    }

    #[test]
    fn serde_backward_compat() {
        // JSON sin params/seed/validator debe deserializar con defaults
        let json =
            r#"{"prompt":"p","solution":"s","kind":"Numeric","difficulty":"Easy","lo_id":"x"}"#;
        let ex: Exercise = serde_json::from_str(json).expect("deserializa");
        assert!(ex.params.is_empty());
        assert!(ex.seed.is_none());
        assert_eq!(ex.validator, ValidatorKind::Exact);
        assert!(ex.validate().is_ok());
    }

    #[test]
    fn difficulty_maps_level() {
        let lo = LearningObjective::new("am1-der", "D", "...", None);
        let gen = ExerciseGenerator;
        let ex = gen.generate_with_seed(&lo, PedagogicalLevel::Primary, 0);
        assert_eq!(ex.difficulty, ExerciseDifficulty::Easy);
        let ex = gen.generate_with_seed(&lo, PedagogicalLevel::Secondary, 0);
        assert_eq!(ex.difficulty, ExerciseDifficulty::Medium);
        let ex = gen.generate_with_seed(&lo, PedagogicalLevel::University, 0);
        assert_eq!(ex.difficulty, ExerciseDifficulty::Hard);
    }

    #[test]
    fn generate_valid_per_level_parametric() {
        use crate::feedback::FeedbackEngine;
        use crate::level::UTNProgram;
        let gen = ExerciseGenerator;
        // (nivel, LO representativo, dificultad esperada)
        let casos: Vec<(PedagogicalLevel, LearningObjective, ExerciseDifficulty)> = vec![
            (
                PedagogicalLevel::Primary,
                LearningObjective::new("pri-conteo", "Conteo", "Conteo", None),
                ExerciseDifficulty::Easy,
            ),
            (
                PedagogicalLevel::Secondary,
                LearningObjective::new("sec-trig", "Trigonometría", "Seno", None),
                ExerciseDifficulty::Medium,
            ),
            (
                PedagogicalLevel::UTN(UTNProgram::AM1),
                LearningObjective::new("am1-der", "Derivadas", "Derivada", None),
                ExerciseDifficulty::Hard,
            ),
        ];
        let semillas = [0u64, 7, 42];
        for (nivel, lo, dificultad_esperada) in &casos {
            for seed in semillas {
                let ex = gen.generate_with_seed(lo, *nivel, seed);
                // Enunciado no vacío y respuesta no vacía.
                assert!(
                    !ex.prompt.trim().is_empty(),
                    "enunciado vacío nivel {:?} seed {seed}",
                    nivel
                );
                assert!(
                    !ex.solution.trim().is_empty(),
                    "respuesta vacía nivel {:?} seed {seed}",
                    nivel
                );
                // Válido según validador interno.
                assert!(
                    ex.validate().is_ok(),
                    "ejercicio inválido nivel {:?} seed {seed}: {:?}",
                    nivel,
                    ex.validate().err()
                );
                // Dificultad acorde al nivel.
                assert_eq!(
                    ex.difficulty, *dificultad_esperada,
                    "dificultad incorrecta nivel {:?} seed {seed}",
                    nivel
                );
                // Respuesta verificable: auto-evaluación con la solución debe dar correcto.
                let fb = FeedbackEngine.assess(&ex, &ex.solution);
                assert!(
                    fb.correct,
                    "respuesta no verificable nivel {:?} seed {seed} prompt '{}' sol '{}'",
                    nivel, ex.prompt, ex.solution
                );
                assert_eq!(ex.seed, Some(seed));
            }
        }
    }

    #[test]
    fn subespacios_y_fractales_variantes_validas_y_verificables() {
        use crate::feedback::{FeedbackEngine, Misconception};
        let gen = ExerciseGenerator;
        let sub = LearningObjective::new(
            "alg-subespacios",
            "Subespacios",
            "span, base, dimensión",
            None,
        );
        let fra = LearningObjective::new("sec-fractales", "Fractales", "Koch", None);
        // 3 variantes por seed: válidas, con ValidatorKind numérico y
        // auto-evaluación correcta.
        for seed in [0u64, 1, 2] {
            for lo in [&sub, &fra] {
                let ex = gen.generate_with_seed(lo, PedagogicalLevel::Secondary, seed);
                assert!(ex.validate().is_ok(), "LO {} seed {seed}", lo.id);
                assert!(
                    matches!(ex.validator, ValidatorKind::NumericTol(_)),
                    "LO {} seed {seed} debe usar NumericTol",
                    lo.id
                );
                let fb = FeedbackEngine.assess(&ex, &ex.solution);
                assert!(fb.correct, "LO {} seed {seed} no verificable", lo.id);
            }
        }
        // Soluciones esperadas por variante.
        assert_eq!(
            gen.generate_with_seed(&sub, PedagogicalLevel::Secondary, 0)
                .solution,
            "2"
        );
        assert_eq!(
            gen.generate_with_seed(&fra, PedagogicalLevel::Secondary, 0)
                .solution,
            "12"
        );
        assert_eq!(
            gen.generate_with_seed(&fra, PedagogicalLevel::Secondary, 1)
                .solution,
            "48"
        );
        assert_eq!(
            gen.generate_with_seed(&fra, PedagogicalLevel::Secondary, 2)
                .solution,
            "9"
        );
        // Misconceptions del patrón existente: signo e independencia.
        let dim = gen.generate_with_seed(&sub, PedagogicalLevel::Secondary, 0);
        let fb_sign = FeedbackEngine.assess(&dim, "-2");
        assert!(!fb_sign.correct);
        assert_eq!(fb_sign.misconception, Misconception::Sign);
        let koch = gen.generate_with_seed(&fra, PedagogicalLevel::Secondary, 0);
        let fb_mal = FeedbackEngine.assess(&koch, "16");
        assert!(!fb_mal.correct);
        assert_ne!(fb_mal.misconception, Misconception::None);
    }
    #[test]
    fn generate_valid_todos_los_los_con_semilla() {
        use crate::curriculum::Curriculum;
        use crate::feedback::FeedbackEngine;
        let gen = ExerciseGenerator;
        // Cobertura: todo LO del currículum genera ejercicio válido con seed 7.
        for lo in Curriculum::all() {
            let ex = gen.generate_with_seed(&lo, PedagogicalLevel::Secondary, 7);
            assert!(
                !ex.prompt.trim().is_empty(),
                "enunciado vacío para LO {}",
                lo.id
            );
            assert!(
                !ex.solution.trim().is_empty(),
                "respuesta vacía para LO {}",
                lo.id
            );
            assert!(
                ex.validate().is_ok(),
                "ejercicio inválido para LO {}",
                lo.id
            );
            let fb = FeedbackEngine.assess(&ex, &ex.solution);
            assert!(
                fb.correct,
                "respuesta no verificable para LO {} sol '{}'",
                lo.id, ex.solution
            );
        }
    }

    /// Parsea `"a/b"` o número decimal a f64 (para verificar la fracción exacta).
    fn parse_fraccion(s: &str) -> Option<f64> {
        if let Some((a, b)) = s.split_once('/') {
            let a: f64 = a.trim().parse().ok()?;
            let b: f64 = b.trim().parse().ok()?;
            if b.abs() < f64::EPSILON {
                return None;
            }
            Some(a / b)
        } else {
            s.trim().parse().ok()
        }
    }

    #[test]
    fn cuatro_familias_nuevas_deterministas_y_verificables() {
        use crate::feedback::FeedbackEngine;
        let gen = ExerciseGenerator;
        for (id, nivel) in [
            (
                "alg-matrices",
                PedagogicalLevel::UTN(crate::level::UTNProgram::Algebra),
            ),
            (
                "am2-edo",
                PedagogicalLevel::UTN(crate::level::UTNProgram::AM2),
            ),
            (
                "prob-var",
                PedagogicalLevel::UTN(crate::level::UTNProgram::Probabilidad),
            ),
            ("sec-fracc", PedagogicalLevel::Secondary),
            (
                "fis1-cinematica",
                PedagogicalLevel::UTN(crate::level::UTNProgram::Fisica1),
            ),
            (
                "dis-combinatoria",
                PedagogicalLevel::UTN(crate::level::UTNProgram::Discreta),
            ),
            (
                "fis-resonancia",
                PedagogicalLevel::UTN(crate::level::UTNProgram::Fisica2),
            ),
            (
                "fis-doppler",
                PedagogicalLevel::UTN(crate::level::UTNProgram::Fisica1),
            ),
            (
                "fis-pendulo",
                PedagogicalLevel::UTN(crate::level::UTNProgram::Fisica1),
            ),
            (
                "fis-energia",
                PedagogicalLevel::UTN(crate::level::UTNProgram::Fisica1),
            ),
        ] {
            let lo = LearningObjective::new(id, id, id, None);
            let mut vistos = std::collections::HashSet::new();
            for seed in 0..10u64 {
                let ex = gen.generate_with_seed(&lo, nivel, seed);
                let otro = gen.generate_with_seed(&lo, nivel, seed);
                assert_eq!(ex.prompt, otro.prompt, "{id} seed {seed}");
                assert_eq!(ex.solution, otro.solution, "{id} seed {seed}");
                assert!(ex.validate().is_ok(), "{id} seed {seed}");
                assert!(matches!(ex.validator, ValidatorKind::NumericTol(_)), "{id}");
                assert!(
                    FeedbackEngine.assess(&ex, &ex.solution).correct,
                    "{id} seed {seed} '{}' sol '{}'",
                    ex.prompt,
                    ex.solution
                );
                vistos.insert(format!("{}|{}", ex.prompt, ex.solution));
            }
            assert!(vistos.len() > 1, "{id} sin variantes");
        }
        // Coherencia exacta: determinante seed 0 = a*d-b*c con a=1+h0%4 etc.
        let lo = LearningObjective::new("alg-matrices", "m", "m", None);
        let ex = gen.generate_with_seed(&lo, PedagogicalLevel::Secondary, 0);
        let (a, b, c, d) = (
            ex.params["a"],
            ex.params["b"],
            ex.params["c"],
            ex.params["d"],
        );
        let esperado = (a * d - b * c) as i64;
        assert_eq!(ex.solution, esperado.to_string());
        // Coherencia exacta: f₀ seed 0 = 1/2π√(LC) con L,C de params.
        let lo = LearningObjective::new("fis-resonancia", "m", "m", None);
        let ex = gen.generate_with_seed(
            &lo,
            PedagogicalLevel::UTN(crate::level::UTNProgram::Fisica2),
            0,
        );
        let l = ex.params["l_mh"] * 1e-3;
        let c = ex.params["c_uf"] * 1e-6;
        let f0 = 1.0 / (2.0 * std::f64::consts::PI * (l * c).sqrt());
        assert_eq!(ex.solution, format!("{f0:.2}"));
        // Coherencia exacta: f' seed 0 = f·c/(c−v) con v,f de params.
        let lo = LearningObjective::new("fis-doppler", "m", "m", None);
        let ex = gen.generate_with_seed(
            &lo,
            PedagogicalLevel::UTN(crate::level::UTNProgram::Fisica1),
            0,
        );
        let fp = ex.params["f"] * 340.0 / (340.0 - ex.params["v"]);
        assert_eq!(ex.solution, format!("{fp:.1}"));
        // Coherencia exacta: T₀ seed 0 = 2π√(L/g) con L de params.
        let lo = LearningObjective::new("fis-pendulo", "m", "m", None);
        let ex = gen.generate_with_seed(
            &lo,
            PedagogicalLevel::UTN(crate::level::UTNProgram::Fisica1),
            0,
        );
        let t0 = 2.0 * std::f64::consts::PI * (ex.params["l"] / 9.8).sqrt();
        assert_eq!(ex.solution, format!("{t0:.2}"));
        // Coherencia exacta: Em seed 0 = ½mv²+mgh con m,v,h de params.
        let lo = LearningObjective::new("fis-energia", "m", "m", None);
        let ex = gen.generate_with_seed(
            &lo,
            PedagogicalLevel::UTN(crate::level::UTNProgram::Fisica1),
            0,
        );
        let em =
            0.5 * ex.params["m"] * ex.params["v"].powi(2) + ex.params["m"] * 9.8 * ex.params["h"];
        assert_eq!(ex.solution, format!("{em:.1}"));
    }

    #[test]
    fn prob_distribuciones_familia_sembra_determinista_y_coherente() {
        // Familia sembrada (patrón para el resto de los LOs, ver doc del
        // módulo): determinismo por semilla, kind coherente y solución exacta.
        use crate::feedback::FeedbackEngine;
        use crate::level::UTNProgram;
        let lo = LearningObjective::new("prob-distribuciones", "Distribuciones", "...", None);
        let gen = ExerciseGenerator;
        let nivel = PedagogicalLevel::UTN(UTNProgram::Probabilidad);
        let mut variantes = std::collections::HashSet::new();
        for seed in 0..20u64 {
            let ex = gen.generate_with_seed(&lo, nivel, seed);
            let otro = gen.generate_with_seed(&lo, nivel, seed);
            // 1. Determinismo por semilla: mismo ejercicio byte a byte.
            assert_eq!(ex.prompt, otro.prompt, "seed {seed}");
            assert_eq!(ex.solution, otro.solution, "seed {seed}");
            assert_eq!(ex.params, otro.params, "seed {seed}");
            assert_eq!(ex.seed, Some(seed));
            // 2. kind coherente: la respuesta es una probabilidad (número).
            assert_eq!(ex.kind, ExerciseKind::Numeric, "seed {seed}");
            assert!(
                matches!(ex.validator, ValidatorKind::NumericTol(_)),
                "seed {seed}"
            );
            assert!(ex.validate().is_ok(), "seed {seed}: {:?}", ex.validate());
            // 3. La solución es la binomial EXACTA (fracción reducida).
            let n = ex.params["n"] as u64;
            let k = ex.params["k"] as u64;
            let p = ex.params["p"];
            let esperado =
                combinatoria(n, k) as f64 * p.powi(k as i32) * (1.0 - p).powi((n - k) as i32);
            let obtenido = parse_fraccion(&ex.solution).expect("solución numérica");
            assert!(
                (obtenido - esperado).abs() < 1e-12,
                "seed {seed}: {obtenido} vs {esperado}"
            );
            // 4. Auto-evaluación: la solución siempre se corrige correcta.
            assert!(
                FeedbackEngine.assess(&ex, &ex.solution).correct,
                "seed {seed}"
            );
            variantes.insert(format!("{}|{}", ex.prompt, ex.solution));
        }
        // La seed varía los parámetros: no es un único enunciado fijo.
        assert!(
            variantes.len() > 4,
            "solo {} variantes en 20 seeds",
            variantes.len()
        );
    }

    #[test]
    fn semillas_consecutivas_no_repiten_ejercicio() {
        // Regresión: la cadena lineal anterior (`h1 = h0*K+C`) dejaba
        // `(a, b)` correlacionados y `am1-der` repetía ejercicio en 35/64
        // semillas consecutivas (con 9 cajas lo esperado es ~7/64). Con
        // flujos con sal independientes la tasa cae a nivel de azar.
        let lo = LearningObjective::new("am1-der", "Derivadas", "...", None);
        let gen = ExerciseGenerator;
        let mut repetidos = 0usize;
        let mut previo = String::new();
        for seed in 0..64u64 {
            let ex = gen.generate_with_seed(&lo, PedagogicalLevel::Secondary, seed);
            let clave = format!("{}|{}", ex.prompt, ex.solution);
            if clave == previo {
                repetidos += 1;
            }
            previo = clave;
        }
        assert!(
            repetidos <= 12,
            "semillas consecutivas repiten demasiado: {repetidos}/64"
        );
    }

    #[test]
    fn combinatoria_y_fraccion_reducida() {
        assert_eq!(combinatoria(6, 3), 20);
        assert_eq!(combinatoria(5, 0), 1);
        assert_eq!(combinatoria(5, 5), 1);
        assert_eq!(combinatoria(4, 2), 6);
        assert_eq!(reducir_fraccion(2, 4), (1, 2));
        assert_eq!(reducir_fraccion(3, 8), (3, 8));
        assert_eq!(reducir_fraccion(8, 8), (1, 1));
        // den == 0 se devuelve sin tocar (inalcanzable desde los generadores).
        assert_eq!(reducir_fraccion(3, 0), (3, 0));
    }
}
