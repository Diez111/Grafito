//! Special mathematical functions.
//!
//! This module provides implementations of special functions commonly used
//! in mathematics, physics, and engineering.

/// Compute the Gamma function Γ(x) using Lanczos approximation.
///
/// The Gamma function is a generalization of the factorial function:
/// Γ(n) = (n-1)! for positive integers
/// Γ(x) = ∫₀^∞ t^(x-1) e^(-t) dt for complex numbers
///
/// # Arguments
/// * `x` - Input value
///
/// # Returns
/// Γ(x)
pub fn gamma(x: f64) -> f64 {
    if x <= 0.0 && x.fract() == 0.0 {
        return f64::INFINITY; // Poles at non-positive integers
    }

    // Use reflection formula for negative values
    if x < 0.5 {
        return std::f64::consts::PI / ((std::f64::consts::PI * x).sin() * gamma(1.0 - x));
    }

    // Lanczos approximation
    let g = 7.0;
    #[allow(clippy::inconsistent_digit_grouping)]
    let c = [
        0.999_999_999_999_809_9,
        676.520_368_121_885_1,
        -1259.139_216_722_402_8,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];

    let x = x - 1.0;
    let mut sum = c[0];
    for (i, &ci) in c.iter().enumerate().skip(1) {
        sum += ci / (x + i as f64);
    }

    let t = x + g + 0.5;
    (2.0 * std::f64::consts::PI).sqrt() * t.powf(x + 0.5) * (-t).exp() * sum
}

/// Compute the natural logarithm of the Gamma function.
///
/// This is more numerically stable than computing ln(Γ(x)) directly.
///
/// # Arguments
/// * `x` - Input value
///
/// # Returns
/// ln(Γ(x))
pub fn ln_gamma(x: f64) -> f64 {
    if x <= 0.0 {
        return f64::NAN;
    }
    // Use the Lanczos approximation directly on ln(Γ(x)).
    if x < 0.5 {
        // Reflection formula.
        return (std::f64::consts::PI / (std::f64::consts::PI * x).sin()).ln() - ln_gamma(1.0 - x);
    }

    let g = 7.0;
    #[allow(clippy::inconsistent_digit_grouping)]
    let c = [
        0.999_999_999_999_809_9,
        676.520_368_121_885_1,
        -1259.139_216_722_402_8,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];

    let x = x - 1.0;
    let mut sum = c[0];
    for (i, &ci) in c.iter().enumerate().skip(1) {
        sum += ci / (x + i as f64);
    }

    let t = x + g + 0.5;
    0.5 * (2.0 * std::f64::consts::PI).ln() + (x + 0.5) * t.ln() - t + sum.ln()
}

/// Compute the Beta function B(a, b).
///
/// The Beta function is related to the Gamma function:
/// B(a, b) = Γ(a)Γ(b) / Γ(a+b)
///
/// # Arguments
/// * `a` - First parameter
/// * `b` - Second parameter
///
/// # Returns
/// B(a, b)
pub fn beta(a: f64, b: f64) -> f64 {
    if a <= 0.0 || b <= 0.0 {
        return f64::NAN;
    }
    (ln_gamma(a) + ln_gamma(b) - ln_gamma(a + b)).exp()
}

fn bessel_asymptotic_y(n: f64, x: f64) -> f64 {
    let chi = x - n * std::f64::consts::PI / 2.0 - std::f64::consts::PI / 4.0;
    let mu = 4.0 * n * n;
    let p = 1.0 - (mu - 1.0) * (mu - 9.0) / (2.0 * (8.0 * x).powi(2))
        + (mu - 1.0) * (mu - 9.0) * (mu - 25.0) * (mu - 49.0) / (24.0 * (8.0 * x).powi(4));
    let q =
        (mu - 1.0) / (8.0 * x) - (mu - 1.0) * (mu - 9.0) * (mu - 25.0) / (6.0 * (8.0 * x).powi(3));
    let amp = (2.0 / (std::f64::consts::PI * x)).sqrt();
    amp * (p * chi.sin() + q * chi.cos())
}

fn nonnegative_bessel_order(n: i32) -> Option<(i32, f64)> {
    if n >= 0 {
        return Some((n, 1.0));
    }

    let order = i32::try_from(n.unsigned_abs()).ok()?;
    let sign = if order % 2 == 0 { 1.0 } else { -1.0 };
    Some((order, sign))
}

/// Maximum absolute integer Bessel order accepted by all evaluators.
pub const MAX_BESSEL_ORDER: i32 = 1_000;

/// Convierte un orden de Bessel expresado como `f64` en un entero evaluable.
///
/// Los órdenes no finitos, no enteros o fuera del presupuesto son errores de
/// dominio. Los evaluadores de expresiones los representan con `NaN` en vez de
/// degradarlos silenciosamente a orden cero.
pub fn parse_bessel_order(order: f64) -> Option<i32> {
    if order.is_finite()
        && order.fract() == 0.0
        && (-(MAX_BESSEL_ORDER as f64)..=MAX_BESSEL_ORDER as f64).contains(&order)
    {
        Some(order as i32)
    } else {
        None
    }
}

/// Maximum absolute Bessel Y order evaluated by the forward recurrence.
pub const MAX_BESSEL_Y_ORDER: i32 = MAX_BESSEL_ORDER;

/// Maximum absolute Bessel J order evaluated by the bounded recurrence.
pub const MAX_BESSEL_J_ORDER: i32 = MAX_BESSEL_Y_ORDER;

/// Maximum absolute modified Bessel I order evaluated by the bounded series.
pub const MAX_BESSEL_I_ORDER: i32 = MAX_BESSEL_Y_ORDER;

/// Whether an integer order can be evaluated by [`bessel_j`].
pub fn bessel_j_order_is_supported(n: i32) -> bool {
    nonnegative_bessel_order(n).is_some_and(|(order, _)| order <= MAX_BESSEL_J_ORDER)
}

/// Whether an integer order can be evaluated by [`bessel_y`].
pub fn bessel_y_order_is_supported(n: i32) -> bool {
    nonnegative_bessel_order(n).is_some_and(|(order, _)| order <= MAX_BESSEL_Y_ORDER)
}

/// Whether an integer order can be evaluated by [`bessel_i`].
pub fn bessel_i_order_is_supported(n: i32) -> bool {
    nonnegative_bessel_order(n).is_some_and(|(order, _)| order <= MAX_BESSEL_I_ORDER)
}

/// Compute the Bessel function of the first kind J_n(x).
///
/// J_n(x) = Σ_{m=0}^∞ (-1)^m / (m! Γ(m+n+1)) * (x/2)^(2m+n)
///
/// Uses forward recurrence when the argument dominates the order, and a
/// continued fraction followed by normalized backward recurrence otherwise.
///
/// # Arguments
/// * `n` - Order (integer)
/// * `x` - Input value
///
/// # Returns
/// J_n(x)
pub fn bessel_j(n: i32, x: f64) -> f64 {
    if !x.is_finite() {
        return f64::NAN;
    }

    let Some((n, order_sign)) = nonnegative_bessel_order(n) else {
        return f64::NAN;
    };
    if n > MAX_BESSEL_J_ORDER {
        return f64::NAN;
    }

    let (x, argument_sign) = if x < 0.0 {
        // J_n(-x) = (-1)^n J_n(x)
        let sign = if n % 2 == 0 { 1.0 } else { -1.0 };
        (-x, sign)
    } else {
        (x, 1.0)
    };

    order_sign * argument_sign * libm::jn(n, x)
}

/// Compute the Bessel function of the second kind Y_n(x) using the relation:
/// Y_n(x) = (J_n(x) cos(nπ) - J_{-n}(x)) / sin(nπ)
///
/// For integer n, use the limit form.
///
/// # Arguments
/// * `n` - Order (integer)
/// * `x` - Input value (must be positive)
///
/// # Returns
/// Y_n(x)
fn bessel_y0(x: f64) -> f64 {
    // Series expansion for Y_0(x) for x > 0.
    let j0 = bessel_j(0, x);
    let gamma_euler = 0.5772156649015329;
    let z = x * x / 4.0;
    let mut sum = 0.0;
    let mut harmonic = 0.0;
    let mut fact2 = 1.0; // (k!)^2
    let mut z_pow = z;
    let mut sign = 1.0; // (-1)^(k-1)
    for k in 1..100 {
        harmonic += 1.0 / k as f64;
        fact2 *= (k * k) as f64;
        let term = sign * harmonic * z_pow / fact2;
        sum += term;
        if term.abs() < 1e-15 {
            break;
        }
        sign = -sign;
        z_pow *= z;
    }
    (2.0 / std::f64::consts::PI) * (j0 * ((x / 2.0).ln() + gamma_euler) + sum)
}

fn bessel_y1_numerical(x: f64) -> f64 {
    // Y_1(x) = -d/dx Y_0(x) computed via central difference.
    let h = 1e-7 * x.max(1e-6);
    let y0_plus = bessel_y0(x + h);
    let y0_minus = bessel_y0(x - h);
    -(y0_plus - y0_minus) / (2.0 * h)
}

pub fn bessel_y(n: i32, x: f64) -> f64 {
    if x <= 0.0 {
        return f64::NAN;
    }

    let Some((n, order_sign)) = nonnegative_bessel_order(n) else {
        return f64::NAN;
    };
    if n > MAX_BESSEL_Y_ORDER {
        return f64::NAN;
    }

    if x > 15.0 {
        return order_sign * bessel_asymptotic_y(n as f64, x);
    }

    let y0 = bessel_y0(x);
    if n == 0 {
        return order_sign * y0;
    }

    let y1 = bessel_y1_numerical(x);
    if n == 1 {
        return order_sign * y1;
    }

    // Forward recurrence: Y_{m+1}(x) = (2m/x) Y_m(x) - Y_{m-1}(x).
    let mut y_m1 = y0;
    let mut y_0 = y1;
    for m in 1..n {
        let y_p1 = (2.0 * m as f64 / x) * y_0 - y_m1;
        y_m1 = y_0;
        y_0 = y_p1;
    }
    order_sign * y_0
}

/// Compute the modified Bessel function of the first kind I_n(x).
///
/// I_n(x) = i^(-n) J_n(ix)
///
/// # Arguments
/// * `n` - Order (integer)
/// * `x` - Input value
///
/// # Returns
/// I_n(x)
pub fn bessel_i(n: i32, x: f64) -> f64 {
    if !x.is_finite() {
        return f64::NAN;
    }

    let Some((n, _)) = nonnegative_bessel_order(n) else {
        return f64::NAN;
    };
    if n > MAX_BESSEL_I_ORDER {
        return f64::NAN;
    }
    let n = n as f64;
    let mut sum = 0.0;
    let mut term = (x / 2.0).powf(n) / gamma(n + 1.0);

    for m in 0..100 {
        sum += term;
        term *= x * x / (4.0 * (m as f64 + 1.0) * (m as f64 + n + 1.0));

        if term.abs() < 1e-15 {
            break;
        }
    }

    sum
}

/// Compute the error function erf(x).
///
/// erf(x) = (2/√π) ∫₀^x e^(-t²) dt
///
/// # Arguments
/// * `x` - Input value
///
/// # Returns
/// erf(x)
pub fn erf(x: f64) -> f64 {
    // Approximation using Horner's method
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();

    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();

    sign * y
}

/// Compute the complementary error function erfc(x) = 1 - erf(x).
///
/// # Arguments
/// * `x` - Input value
///
/// # Returns
/// erfc(x)
pub fn erfc(x: f64) -> f64 {
    1.0 - erf(x)
}

/// Compute the digamma function ψ(x) = d/dx ln(Γ(x)).
///
/// # Arguments
/// * `x` - Input value
///
/// # Returns
/// ψ(x)
pub fn digamma(x: f64) -> f64 {
    if x.is_nan() || x == f64::NEG_INFINITY {
        return f64::NAN;
    }
    if x == f64::INFINITY {
        return f64::INFINITY;
    }
    if x <= 0.0 && x.fract() == 0.0 {
        return f64::NAN;
    }

    if x < 0.0 {
        let sin_pi_x = (std::f64::consts::PI * x).sin();
        return digamma(1.0 - x) - std::f64::consts::PI / sin_pi_x.tan();
    }

    // Use recurrence relation to shift x to large values
    let mut result = 0.0;
    let mut x = x;

    while x < 6.0 {
        result -= 1.0 / x;
        x += 1.0;
    }

    // Asymptotic expansion for large x
    let x2 = 1.0 / (x * x);
    result += x.ln() - 0.5 / x - x2 * (1.0 / 12.0 - x2 * (1.0 / 120.0 - x2 * (1.0 / 252.0)));

    result
}

/// Compute the trigamma function ψ₁(x) = d/dx ψ(x).
///
/// The recurrence shifts finite inputs to a range where the asymptotic series
/// converges rapidly. Non-positive integers are poles and return `NaN`.
pub fn trigamma(x: f64) -> f64 {
    if x.is_nan() || x == f64::NEG_INFINITY || (x <= 0.0 && x.fract() == 0.0) {
        return f64::NAN;
    }
    if x == f64::INFINITY {
        return 0.0;
    }
    if x < 0.0 {
        let sin_pi_x = (std::f64::consts::PI * x).sin();
        return std::f64::consts::PI.powi(2) / (sin_pi_x * sin_pi_x) - trigamma(1.0 - x);
    }

    let mut result = 0.0;
    let mut x = x;
    while x < 10.0 {
        result += 1.0 / (x * x);
        x += 1.0;
    }

    let inv = 1.0 / x;
    let inv2 = inv * inv;
    result
        + inv
        + inv2
            * (0.5
                + inv
                    * (1.0 / 6.0
                        + inv2
                            * (-1.0 / 30.0
                                + inv2
                                    * (1.0 / 42.0 + inv2 * (-1.0 / 30.0 + inv2 * (5.0 / 66.0))))))
}

// ── O6 Especiales (polygamma >1, ₂F₁, elípticas K/E, Airy, Bessel K, armónicos) ──
// Motores puros f64 sobre UNA representación (sin tipos nuevos): los comandos
// los llaman con args ya evaluados (mismo criterio que O5: intervalos sin
// comando por ser infraestructura; acá todo lleva comando en grafito-command).

/// Orden máximo de polygamma evaluable (m! en f64 exacto hasta 22; 10 de sobra).
pub const MAX_POLYGAMMA_ORDER: i32 = 10;

/// Orden máximo del armónico esférico (recurrencia estable + norma vía lnΓ).
pub const MAX_SPHERICAL_DEGREE: i32 = 32;

/// Orden máximo de Bessel K por recurrencia hacia adelante (crece como (n-1)!).
pub const MAX_BESSEL_K_ORDER: i32 = MAX_BESSEL_ORDER;

/// ¿Puede evaluarlo [`polygamma`]?
pub fn polygamma_order_is_supported(m: i32) -> bool {
    (0..=MAX_POLYGAMMA_ORDER).contains(&m)
}

fn fact_f64(n: u32) -> f64 {
    let mut acc = 1.0;
    let mut k = 2u32;
    while k <= n {
        acc *= k as f64;
        k += 1;
    }
    acc
}

/// Polygamma ψ_m(x) = d^{m+1}/dx^{m+1} ln Γ(x).
///
/// m=0 delega a [`digamma`], m=1 a [`trigamma`]; m≥2 usa recurrencia hasta
/// y≥10 + serie asintótica con B₂..B₁₀. Polos (x entero ≤0) y x≤0 no entero
/// en m≥2 (reflexión no implementada) devuelven `NaN` honesto.
pub fn polygamma(m: i32, x: f64) -> f64 {
    if !(0..=MAX_POLYGAMMA_ORDER).contains(&m) {
        return f64::NAN;
    }
    if x.is_nan() {
        return f64::NAN;
    }
    if m == 0 {
        return digamma(x);
    }
    if m == 1 {
        return trigamma(x);
    }
    if x == f64::INFINITY {
        return 0.0;
    }
    if x == f64::NEG_INFINITY || x <= 0.0 && x.fract() == 0.0 {
        return f64::NAN;
    }
    if x <= 0.0 {
        return f64::NAN;
    }
    let m_u = m as u32;
    let fact_m = fact_f64(m_u);
    // (-1)^m
    let s: f64 = if m % 2 == 0 { 1.0 } else { -1.0 };
    let mut corr = 0.0;
    let mut y = x;
    while y < 10.0 {
        corr -= s * fact_m / y.powf(f64::from(m_u) + 1.0);
        y += 1.0;
    }
    // Asintótica: (-1)^{m-1} [ (m-1)!/y^m + m!/(2y^{m+1})
    //   + Σ B_{2k} (m+2k-1)!/((2k)! y^{m+2k}) ], k=1..5.
    const B: [f64; 5] = [1.0 / 6.0, -1.0 / 30.0, 1.0 / 42.0, -1.0 / 30.0, 5.0 / 66.0];
    let sign: f64 = if (m - 1) % 2 == 0 { 1.0 } else { -1.0 };
    let mut asym =
        fact_f64(m_u - 1) / y.powf(f64::from(m_u)) + fact_m / (2.0 * y.powf(f64::from(m_u) + 1.0));
    let mut k = 1u32;
    while k <= 5 {
        let two_k = 2 * k;
        let num = fact_f64(m_u + two_k - 1);
        let den = fact_f64(two_k);
        asym += B[(k - 1) as usize] * num / den / y.powf(f64::from(m_u + two_k));
        k += 1;
    }
    corr + sign * asym
}

/// Hipergeométrica de Gauss ₂F₁(a,b;c;z) por serie de potencias.
///
/// Subconjunto honesto: |z|<1 por serie (2000 términos, tol 1e-15) + z=1 por
/// suma de Gauss si c-a-b>0. |z|>1, z=1 divergente y c entero ≤0 → `NaN`.
pub fn hypergeometric_2f1(a: f64, b: f64, c: f64, z: f64) -> f64 {
    if a.is_nan() || b.is_nan() || c.is_nan() || z.is_nan() {
        return f64::NAN;
    }
    if !a.is_finite() || !b.is_finite() || !c.is_finite() || !z.is_finite() {
        return f64::NAN;
    }
    if c <= 0.0 && c.fract() == 0.0 {
        return f64::NAN;
    }
    if z == 0.0 {
        return 1.0;
    }
    if z == 1.0 {
        let s = c - a - b;
        if s <= 0.0 {
            return f64::NAN;
        }
        let ca = c - a;
        let cb = c - b;
        if (ca <= 0.0 && ca.fract() == 0.0) || (cb <= 0.0 && cb.fract() == 0.0) {
            return f64::NAN;
        }
        return gamma(c) * gamma(s) / (gamma(ca) * gamma(cb));
    }
    if z.abs() >= 1.0 {
        return f64::NAN;
    }
    let mut sum = 1.0;
    let mut term = 1.0;
    let mut n = 0u32;
    while n < 2000 {
        let nf = f64::from(n);
        term *= (a + nf) * (b + nf) / ((c + nf) * (nf + 1.0)) * z;
        sum += term;
        if !term.is_finite() || !sum.is_finite() {
            return f64::NAN;
        }
        if term.abs() <= 1e-15 * sum.abs().max(1.0) {
            return sum;
        }
        // Serie terminada (a o b entero negativo): el término se anula exacto.
        if term == 0.0 {
            return sum;
        }
        n += 1;
    }
    f64::NAN
}

/// Integral elíptica completa de primera especie K(k), k = módulo.
///
/// K(k) = π/(2·AGM(1, √(1-k²))). |k|>1 → `NaN`; k=±1 → `INFINITY` (polo
/// logarítmico; el comando lo convierte en error honesto por no finito).
pub fn elliptic_k(k: f64) -> f64 {
    if !k.is_finite() {
        return f64::NAN;
    }
    let m = k * k;
    if m > 1.0 {
        return f64::NAN;
    }
    if m == 1.0 {
        return f64::INFINITY;
    }
    let mut a = 1.0;
    let mut b = (1.0 - m).sqrt();
    let mut i = 0;
    while i < 50 {
        let an = 0.5 * (a + b);
        let bn = (a * b).sqrt();
        a = an;
        b = bn;
        if (a - b).abs() <= 1e-15 * a {
            break;
        }
        i += 1;
    }
    std::f64::consts::PI / (2.0 * a)
}

/// Integral elíptica completa de segunda especie E(k), k = módulo.
///
/// Vía AGM con suma de c_n². |k|>1 → `NaN`; E(±1)=1 exacto.
pub fn elliptic_e(k: f64) -> f64 {
    if !k.is_finite() {
        return f64::NAN;
    }
    let m = k * k;
    if m > 1.0 {
        return f64::NAN;
    }
    if m == 1.0 {
        return 1.0;
    }
    if m == 0.0 {
        return std::f64::consts::PI / 2.0;
    }
    let c0 = m.sqrt();
    let mut a = 1.0;
    let mut b = (1.0 - m).sqrt();
    let mut sum = 0.5 * c0 * c0;
    let mut pow2 = 1.0;
    let mut i = 0;
    while i < 50 {
        let cn = 0.5 * (a - b);
        let an = 0.5 * (a + b);
        let bn = (a * b).sqrt();
        pow2 *= 2.0;
        sum += 0.5 * pow2 * cn * cn;
        a = an;
        b = bn;
        if cn.abs() <= 1e-16 * a {
            break;
        }
        i += 1;
    }
    let kk = std::f64::consts::PI / (2.0 * a);
    kk * (1.0 - sum)
}

// Constantes de la serie de Airy: c1 = 1/(3^{2/3} Γ(2/3)), c2 = 1/(3^{1/3} Γ(1/3)).
const AIRY_C1: f64 = 0.355_028_053_887_817;
const AIRY_C2: f64 = 0.258_819_403_792_807;

fn airy_series_fg(x: f64) -> (f64, f64) {
    let x3 = x * x * x;
    let mut f = 1.0;
    let mut fterm = 1.0;
    let mut g = 1.0;
    let mut gterm = 1.0;
    let mut n = 0u32;
    while n < 60 {
        let nf = f64::from(n);
        fterm *= x3 / ((3.0 * nf + 3.0) * (3.0 * nf + 2.0));
        f += fterm;
        gterm *= x3 / ((3.0 * nf + 4.0) * (3.0 * nf + 3.0));
        g += gterm;
        if fterm.abs() < 1e-16 * f.abs().max(1.0) && gterm.abs() < 1e-16 * g.abs().max(1.0) {
            break;
        }
        n += 1;
    }
    (f, g)
}

fn airy_u_coeffs(nterms: usize) -> Vec<f64> {
    // DLMF 9.7.5 con ζ = 2x^{3/2}/3 (el mismo ζ del prefactor e^{∓ζ}):
    // u_0 = 1, u_1 = 5/72 (verificado por EDO: c = -(5/16)·(2/9) = -5/72).
    let mut u = Vec::with_capacity(nterms);
    u.push(1.0);
    let mut k = 0usize;
    while k + 1 < nterms {
        let kf = k as f64;
        let next = u[k] * (6.0 * kf + 5.0) * (6.0 * kf + 1.0) / (72.0 * (kf + 1.0));
        u.push(next);
        k += 1;
    }
    u
}

/// Ai(x): serie para |x|≤5, asintótica con u_k para |x|>5.
///
/// No finitos → `NaN`; Ai(±∞)=0.
pub fn airy_ai(x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }
    if x.is_infinite() {
        return 0.0;
    }
    if x.abs() <= 5.0 {
        let (f, g) = airy_series_fg(x);
        return AIRY_C1 * f - AIRY_C2 * g * x;
    }
    let u = airy_u_coeffs(8);
    if x > 0.0 {
        let zeta = 2.0 / 3.0 * x.powf(1.5);
        let mut s = 0.0;
        let mut k = 7i32;
        while k >= 0 {
            let ku = k as usize;
            let term = u[ku] / zeta.powf(k as f64);
            s = if k % 2 == 0 { s + term } else { s - term };
            if ku == 0 {
                break;
            }
            k -= 1;
        }
        (-zeta).exp() / (2.0 * std::f64::consts::PI.sqrt() * x.powf(0.25)) * s
    } else {
        let z = -x;
        let zeta = 2.0 / 3.0 * z.powf(1.5);
        let mut p = 0.0;
        let mut q = 0.0;
        let mut k = 7i32;
        while k >= 0 {
            let ku = k as usize;
            let term = u[ku] / zeta.powf(k as f64);
            if k % 2 == 0 {
                // u_{2j} va a P con (-1)^j
                let j = k / 2;
                if j % 2 == 0 {
                    p += term;
                } else {
                    p -= term;
                }
            } else {
                let j = (k - 1) / 2;
                if j % 2 == 0 {
                    q += term;
                } else {
                    q -= term;
                }
            }
            if ku == 0 {
                break;
            }
            k -= 1;
        }
        let phi = zeta + std::f64::consts::FRAC_PI_4;
        (p * phi.sin() - q * phi.cos()) / (std::f64::consts::PI.sqrt() * z.powf(0.25))
    }
}

/// Bi(x): serie para |x|≤5, asintótica con u_k para |x|>5.
///
/// No finitos → `NaN`; Bi(+∞)=+∞, Bi(-∞)=0.
pub fn airy_bi(x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }
    if x == f64::INFINITY {
        return f64::INFINITY;
    }
    if x == f64::NEG_INFINITY {
        return 0.0;
    }
    if x.abs() <= 5.0 {
        let (f, g) = airy_series_fg(x);
        return 3.0_f64.sqrt() * (AIRY_C1 * f + AIRY_C2 * g * x);
    }
    let u = airy_u_coeffs(8);
    if x > 0.0 {
        let zeta = 2.0 / 3.0 * x.powf(1.5);
        let mut s = 0.0;
        let mut k = 7i32;
        while k >= 0 {
            s += u[k as usize] / zeta.powf(k as f64);
            if k == 0 {
                break;
            }
            k -= 1;
        }
        zeta.exp() / (std::f64::consts::PI.sqrt() * x.powf(0.25)) * s
    } else {
        let z = -x;
        let zeta = 2.0 / 3.0 * z.powf(1.5);
        let mut p = 0.0;
        let mut q = 0.0;
        let mut k = 7i32;
        while k >= 0 {
            let ku = k as usize;
            let term = u[ku] / zeta.powf(k as f64);
            if k % 2 == 0 {
                let j = k / 2;
                if j % 2 == 0 {
                    p += term;
                } else {
                    p -= term;
                }
            } else {
                let j = (k - 1) / 2;
                if j % 2 == 0 {
                    q += term;
                } else {
                    q -= term;
                }
            }
            if ku == 0 {
                break;
            }
            k -= 1;
        }
        let phi = zeta + std::f64::consts::FRAC_PI_4;
        (p * phi.cos() + q * phi.sin()) / (std::f64::consts::PI.sqrt() * z.powf(0.25))
    }
}

fn bessel_k_integrand(t: f64, n: i32, x: f64) -> f64 {
    (f64::from(n) * t).cosh() * (-x * t.cosh()).exp()
}

fn simpson(f_n: i32, f_x: f64, a: f64, b: f64) -> f64 {
    let m = 0.5 * (a + b);
    (b - a) / 6.0
        * (bessel_k_integrand(a, f_n, f_x)
            + 4.0 * bessel_k_integrand(m, f_n, f_x)
            + bessel_k_integrand(b, f_n, f_x))
}

fn adaptive_simpson_rec(n: i32, x: f64, a: f64, b: f64, eps: f64, whole: f64, depth: u32) -> f64 {
    let m = 0.5 * (a + b);
    let left = simpson(n, x, a, m);
    let right = simpson(n, x, m, b);
    let delta = left + right - whole;
    if depth == 0 || delta.abs() <= 15.0 * eps {
        return left + right + delta / 15.0;
    }
    adaptive_simpson_rec(n, x, a, m, eps * 0.5, left, depth - 1)
        + adaptive_simpson_rec(n, x, m, b, eps * 0.5, right, depth - 1)
}

fn bessel_k_base(n: i32, x: f64) -> f64 {
    // T con cola despreciable: x·cosh(T) ≥ 45 (e^-45 ≈ 3e-20).
    let t_cap = if x >= 45.0 {
        2.0
    } else {
        let y = 45.0 / x;
        (y + (y * y - 1.0).sqrt()).ln() + 1.0
    };
    let whole = simpson(n, x, 0.0, t_cap);
    adaptive_simpson_rec(n, x, 0.0, t_cap, 1e-13, whole, 22)
}

/// ¿Puede evaluarlo [`bessel_k`]?
pub fn bessel_k_order_is_supported(n: i32) -> bool {
    nonnegative_bessel_order(n).is_some_and(|(order, _)| order <= MAX_BESSEL_K_ORDER)
}

/// Bessel modificada de segunda especie K_n(x), n entero, x>0.
///
/// K₀/K₁ por cuadratura adaptativa de ∫₀^∞ e^{-x·cosh t}·cosh(nt) dt +
/// recurrencia hacia adelante. x≤0 o no finito → `NaN`.
pub fn bessel_k(n: i32, x: f64) -> f64 {
    if !x.is_finite() || x <= 0.0 {
        if x == f64::INFINITY {
            return 0.0;
        }
        return f64::NAN;
    }
    let Some((n, order_sign)) = nonnegative_bessel_order(n) else {
        return f64::NAN;
    };
    if n > MAX_BESSEL_K_ORDER {
        return f64::NAN;
    }
    // K_{-n} = K_n: el signo de la orden no cambia el valor.
    let _ = order_sign;
    let k0 = bessel_k_base(0, x);
    if n == 0 {
        return k0;
    }
    let k1 = bessel_k_base(1, x);
    if n == 1 {
        return k1;
    }
    let mut k_m1 = k0;
    let mut k_0 = k1;
    let mut m = 1i32;
    while m < n {
        let k_p1 = k_m1 + 2.0 * f64::from(m) / x * k_0;
        if !k_p1.is_finite() {
            return f64::INFINITY;
        }
        k_m1 = k_0;
        k_0 = k_p1;
        m += 1;
    }
    k_0
}

/// Legendre asociada P_l^m(x) con fase Condon-Shortley, 0≤m≤l≤32, |x|≤1.
///
/// Fuera de dominio → `NaN`.
pub fn assoc_legendre(l: i32, m: i32, x: f64) -> f64 {
    if !(0..=MAX_SPHERICAL_DEGREE).contains(&l) || m < 0 || m > l {
        return f64::NAN;
    }
    if !x.is_finite() || x.abs() > 1.0 {
        return f64::NAN;
    }
    let s = (1.0 - x * x).sqrt();
    // P_m^m = (-1)^m (2m-1)!! s^m
    let mut pmm = 1.0;
    let mut k = 1i32;
    while k <= m {
        pmm *= -f64::from(2 * k - 1) * s;
        k += 1;
    }
    if l == m {
        return pmm;
    }
    // P_{m+1}^m = x(2m+1) P_m^m
    let mut p_prev = pmm;
    let mut p_cur = x * f64::from(2 * m + 1) * pmm;
    let mut ll = m + 2;
    while ll <= l {
        let p_next = (x * f64::from(2 * ll - 1) * p_cur - f64::from(ll + m - 1) * p_prev)
            / f64::from(ll - m);
        p_prev = p_cur;
        p_cur = p_next;
        ll += 1;
    }
    p_cur
}

/// Armónico esférico REAL Y_l^m(θ,φ) con fase Condon-Shortley.
///
/// m=0: N·P; m>0: √2·N·P·cos(mφ); m<0: √2·N·P·sin(|m|φ),
/// N=√((2l+1)/4π·(l-|m|)!/(l+|m|)!). |m|>l o l fuera de \[0,32\] o ángulos no
/// finitos → `NaN`.
pub fn spherical_harm_real(l: i32, m: i32, theta: f64, phi: f64) -> f64 {
    if !(0..=MAX_SPHERICAL_DEGREE).contains(&l) || m.abs() > l {
        return f64::NAN;
    }
    if !theta.is_finite() || !phi.is_finite() {
        return f64::NAN;
    }
    let am = m.abs();
    let p = assoc_legendre(l, am, theta.cos());
    if !p.is_finite() {
        return f64::NAN;
    }
    let lf = f64::from(l);
    let amf = f64::from(am);
    let norm = ((2.0 * lf + 1.0) / (4.0 * std::f64::consts::PI)
        * (ln_gamma(lf - amf + 1.0) - ln_gamma(lf + amf + 1.0)).exp())
    .sqrt();
    if m == 0 {
        norm * p
    } else if m > 0 {
        std::f64::consts::SQRT_2 * norm * p * (amf * phi).cos()
    } else {
        std::f64::consts::SQRT_2 * norm * p * (amf * phi).sin()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn o6_polygamma_supera_digamma_y_trigamma() {
        // ψ_2(1) = -2ζ(3), ψ_3(1) = π⁴/15, recurrencia ψ_2(2) = ψ_2(1) + 2.
        assert!((polygamma(2, 1.0) - -2.404_113_806_319).abs() < 1e-9);
        assert!((polygamma(3, 1.0) - 6.493_939_402_267).abs() < 1e-9);
        assert!((polygamma(2, 2.0) - -0.404_113_806_319).abs() < 1e-9);
        // m=0/1 delegan exacto.
        assert!((polygamma(0, 1.0) - digamma(1.0)).abs() == 0.0);
        assert!((polygamma(1, 1.0) - trigamma(1.0)).abs() == 0.0);
        assert!(polygamma_order_is_supported(10));
        assert!(!polygamma_order_is_supported(11));
        // Dominio honesto: polos, x≤0 en m≥2, orden fuera de cota.
        assert!(polygamma(2, 0.0).is_nan());
        assert!(polygamma(2, -1.0).is_nan());
        assert!(polygamma(2, -0.5).is_nan());
        assert!(polygamma(11, 1.0).is_nan());
        assert!(polygamma(-1, 1.0).is_nan());
        assert!(polygamma(2, f64::NAN).is_nan());
        assert!((polygamma(2, f64::INFINITY) - 0.0).abs() == 0.0);
    }

    #[test]
    fn o6_hypergeometric_2f1_serie_y_gauss() {
        // ₂F₁(1,1;2;1/2) = -ln(1/2)/(1/2) = 2ln2.
        assert!((hypergeometric_2f1(1.0, 1.0, 2.0, 0.5) - 1.386_294_361_12).abs() < 1e-9);
        // ₂F₁(1/2,1/2;3/2;1/4) = arcsin(1/2)/(1/2) = π/3.
        assert!(
            (hypergeometric_2f1(0.5, 0.5, 1.5, 0.25) - std::f64::consts::PI / 3.0).abs() < 1e-12
        );
        // Serie terminada: a entero negativo (1 - 10/3·0.7 + 5·0.7²/2).
        assert!((hypergeometric_2f1(-2.0, 5.0, 3.0, 0.7) - -0.108_333_333_333).abs() < 1e-9);
        // Gauss en z=1: ₂F₁(1,2;4;1) = Γ(4)Γ(1)/(Γ(3)Γ(2)) = 3.
        assert!((hypergeometric_2f1(1.0, 2.0, 4.0, 1.0) - 3.0).abs() < 1e-9);
        assert!((hypergeometric_2f1(0.0, 5.0, 3.0, 0.9) - 1.0).abs() == 0.0);
        // Dominio honesto.
        assert!(hypergeometric_2f1(1.0, 1.0, 2.0, 1.5).is_nan());
        assert!(hypergeometric_2f1(1.0, 1.0, 0.0, 0.5).is_nan());
        assert!(hypergeometric_2f1(1.0, 1.0, -2.0, 0.5).is_nan());
        assert!(hypergeometric_2f1(2.0, 2.0, 3.0, 1.0).is_nan());
        assert!(hypergeometric_2f1(f64::NAN, 1.0, 2.0, 0.5).is_nan());
    }

    #[test]
    fn o6_elipticas_k_e_agm() {
        assert!((elliptic_k(0.0) - std::f64::consts::FRAC_PI_2).abs() < 1e-14);
        assert!((elliptic_k(0.5) - 1.685_750_354_813).abs() < 1e-9);
        assert!((elliptic_k(std::f64::consts::FRAC_1_SQRT_2) - 1.854_074_677_301).abs() < 1e-9);
        assert!((elliptic_e(0.0) - std::f64::consts::FRAC_PI_2).abs() < 1e-14);
        assert!((elliptic_e(0.5) - 1.467_462_209_339).abs() < 1e-9);
        assert!((elliptic_e(std::f64::consts::FRAC_1_SQRT_2) - 1.350_643_881_048).abs() < 1e-9);
        assert!((elliptic_e(1.0) - 1.0).abs() == 0.0);
        assert!(elliptic_k(1.0).is_infinite());
        assert!(elliptic_k(1.5).is_nan());
        assert!(elliptic_e(1.5).is_nan());
        assert!(elliptic_k(f64::NAN).is_nan());
    }

    #[test]
    fn o6_airy_serie_y_asintotica() {
        assert!((airy_ai(0.0) - 0.355_028_053_888).abs() < 1e-9);
        assert!((airy_bi(0.0) - 0.614_926_627_446).abs() < 1e-9);
        assert!((airy_ai(1.0) - 0.135_292_416_313).abs() < 1e-9);
        assert!((airy_bi(1.0) - 1.207_423_594_953).abs() < 1e-6);
        assert!((airy_ai(-1.0) - 0.535_560_883_292).abs() < 1e-9);
        assert!((airy_bi(-1.0) - 0.103_997_389_497).abs() < 1e-9);
        // Asintótica validada contra RK4 h=2e-5 + wronskiano 1/π.
        // Bi(6): truncación u_8/ζ⁸ ≈ 4e-4 → tol 1e-3; Bi(12): 8e-12 → tol 1e-9.
        assert!((airy_bi(6.0) - 6_536.446_104_9).abs() < 1e-3);
        assert!((airy_bi(12.0) - 329_807_225_830.0).abs() / 329_807_225_830.0 < 1e-9);
        assert!((airy_bi(-6.0) - -0.146_698_376_67).abs() < 1e-6);
        let inv_pi = std::f64::consts::PI.recip();
        for x in [0.0_f64, 1.0, -1.0, 6.0, 8.0, -8.0] {
            let h = 1e-6 * x.abs().max(1.0);
            let aip = (airy_ai(x + h) - airy_ai(x - h)) / (2.0 * h);
            let bip = (airy_bi(x + h) - airy_bi(x - h)) / (2.0 * h);
            let w = airy_ai(x) * bip - aip * airy_bi(x);
            assert!((w - inv_pi).abs() < 1e-6, "W({x}) = {w}");
        }
        assert!(airy_ai(f64::NAN).is_nan());
        assert!((airy_ai(f64::INFINITY) - 0.0).abs() == 0.0);
        assert!(airy_bi(f64::INFINITY).is_infinite());
        assert!((airy_bi(f64::NEG_INFINITY) - 0.0).abs() == 0.0);
    }

    #[test]
    fn o6_bessel_k_cuadratura_y_recurrencia() {
        assert!((bessel_k(0, 1.0) - 0.421_024_438_241).abs() < 1e-9);
        assert!((bessel_k(1, 1.0) - 0.601_907_230_197).abs() < 1e-9);
        assert!((bessel_k(2, 1.0) - 1.624_838_898_635).abs() < 1e-9);
        assert!((bessel_k(0, 2.0) - 0.113_893_872_750).abs() < 1e-9);
        // K_{-n} = K_n.
        assert!((bessel_k(-2, 1.0) - bessel_k(2, 1.0)).abs() == 0.0);
        assert!(bessel_k_order_is_supported(1000));
        assert!(!bessel_k_order_is_supported(1001));
        assert!(bessel_k(0, 0.0).is_nan());
        assert!(bessel_k(0, -1.0).is_nan());
        assert!((bessel_k(0, f64::INFINITY) - 0.0).abs() == 0.0);
        assert!(bessel_k(1001, 1.0).is_nan());
    }

    #[test]
    fn o6_armonicos_esfericos_reales() {
        // Y_0^0 = 1/√(4π); Y_1^0(0) = √(3/4π); Y_1^1(π/2,0) = -√(3/4π) (CS).
        assert!((spherical_harm_real(0, 0, 1.0, 0.0) - 0.282_094_791_774).abs() < 1e-9);
        assert!((spherical_harm_real(1, 0, 0.0, 0.0) - 0.488_602_511_903).abs() < 1e-9);
        assert!(
            (spherical_harm_real(1, 1, std::f64::consts::FRAC_PI_2, 0.0) - -0.488_602_511_903)
                .abs()
                < 1e-9
        );
        // Y_1^{-1}(π/2,π/2) = √2·N·P·sin(π/2) con P=-1: -0.4886.
        assert!(
            (spherical_harm_real(
                1,
                -1,
                std::f64::consts::FRAC_PI_2,
                std::f64::consts::FRAC_PI_2
            ) - -0.488_602_511_903)
                .abs()
                < 1e-9
        );
        // Normalización: ∫|Y_0^0|²dΩ = 1/(4π)·4π = 1 vía el valor constante.
        let y00 = spherical_harm_real(0, 0, 0.7, 2.1);
        assert!((y00 * y00 * 4.0 * std::f64::consts::PI - 1.0).abs() < 1e-12);
        assert!(spherical_harm_real(1, 2, 0.5, 0.5).is_nan());
        assert!(spherical_harm_real(-1, 0, 0.5, 0.5).is_nan());
        assert!(spherical_harm_real(33, 0, 0.5, 0.5).is_nan());
        assert!(spherical_harm_real(1, 0, f64::NAN, 0.0).is_nan());
        assert!((assoc_legendre(2, 0, 0.5) - -0.125).abs() < 1e-12);
    }

    #[test]
    fn test_gamma_integers() {
        // Γ(1) = 0! = 1
        assert!((gamma(1.0) - 1.0).abs() < 0.001);

        // Γ(2) = 1! = 1
        assert!((gamma(2.0) - 1.0).abs() < 0.001);

        // Γ(3) = 2! = 2
        assert!((gamma(3.0) - 2.0).abs() < 0.001);

        // Γ(4) = 3! = 6
        assert!((gamma(4.0) - 6.0).abs() < 0.001);

        // Γ(5) = 4! = 24
        assert!((gamma(5.0) - 24.0).abs() < 0.01);
    }

    #[test]
    fn test_gamma_half() {
        // Γ(1/2) = √π
        let expected = std::f64::consts::PI.sqrt();
        assert!((gamma(0.5) - expected).abs() < 0.001);
    }

    #[test]
    fn test_beta() {
        // B(1, 1) = 1
        assert!((beta(1.0, 1.0) - 1.0).abs() < 0.001);

        // B(2, 2) = 1/6
        assert!((beta(2.0, 2.0) - 1.0 / 6.0).abs() < 0.001);
    }

    #[test]
    fn test_bessel_j0() {
        // J_0(0) = 1
        assert!((bessel_j(0, 0.0) - 1.0).abs() < 0.001);

        // J_0(2.4048) ≈ 0 (first zero)
        assert!(bessel_j(0, 2.4048).abs() < 0.01);

        // Large argument test (asymptotic)
        // J_0(15.0) ≈ -0.01422447
        assert!((bessel_j(0, 15.0) - (-0.01422447)).abs() < 1e-5);
        // J_0(20.0) ≈ 0.16702466
        assert!((bessel_j(0, 20.0) - 0.16702466).abs() < 1e-5);
    }

    #[test]
    fn test_bessel_j1() {
        // J_1(0) = 0
        assert!(bessel_j(1, 0.0).abs() < 0.001);

        // J_1(3.8317) ≈ 0 (first zero)
        assert!(bessel_j(1, 3.8317).abs() < 0.01);

        // Large argument test (asymptotic)
        // J_1(15.0) ≈ 0.205103
        assert!((bessel_j(1, 15.0) - 0.205103).abs() < 1e-5);
    }

    #[test]
    fn bessel_j_high_order_moderate_argument_matches_reference() {
        let expected = 3.961_755_094_336_252e-59;
        let actual = bessel_j(100, 20.0);

        assert!(actual.is_finite());
        assert!(
            ((actual - expected) / expected).abs() < 1e-12,
            "J_100(20) = {actual:e}, expected {expected:e}"
        );
    }

    #[test]
    fn bessel_j_matches_low_order_and_transition_references() {
        let cases = [
            (0, 1.0, 0.765_197_686_557_966_6),
            (1, 1.0, 0.440_050_585_744_933_5),
            (2, 5.0, 0.046_565_116_277_752_216),
            (10, 20.0, 0.186_482_558_023_945_1),
            (100, 100.0, 0.096_366_673_295_861_56),
        ];

        for (order, argument, expected) in cases {
            let actual = bessel_j(order, argument);
            assert!(
                (actual - expected).abs() <= 1e-13 * expected.abs().max(1.0),
                "J_{order}({argument}) = {actual:e}, expected {expected:e}"
            );
        }
    }

    #[test]
    fn bessel_j_preserves_integer_order_and_argument_parity() {
        for order in [3, 4, 100] {
            let positive = bessel_j(order, 7.0);
            let parity = if order % 2 == 0 { 1.0 } else { -1.0 };

            assert_eq!(bessel_j(-order, 7.0), parity * positive);
            assert_eq!(bessel_j(order, -7.0), parity * positive);
            assert_eq!(bessel_j(-order, -7.0), positive);
        }
    }

    #[test]
    fn bessel_j_handles_zero_and_rejects_nonfinite_arguments() {
        assert_eq!(bessel_j(0, 0.0), 1.0);
        assert_eq!(bessel_j(100, 0.0), 0.0);
        assert!(bessel_j(0, f64::NAN).is_nan());
        assert!(bessel_j(0, f64::INFINITY).is_nan());
        assert!(bessel_j(0, f64::NEG_INFINITY).is_nan());
    }

    #[test]
    fn test_bessel_y() {
        // Y_0(15.0) ≈ 0.205464
        assert!((bessel_y(0, 15.0) - 0.205464).abs() < 1e-5);
        // Y_0(20.0) ≈ 0.0626406
        assert!((bessel_y(0, 20.0) - 0.0626406).abs() < 1e-5);
    }

    #[test]
    fn test_erf() {
        // erf(0) = 0
        assert!(erf(0.0).abs() < 0.001);

        // erf(∞) = 1
        assert!((erf(10.0) - 1.0).abs() < 0.001);

        // erf(-∞) = -1
        assert!((erf(-10.0) - (-1.0)).abs() < 0.001);
    }

    #[test]
    fn test_erfc() {
        // erfc(0) = 1
        assert!((erfc(0.0) - 1.0).abs() < 0.001);

        // erfc(∞) = 0
        assert!(erfc(10.0).abs() < 0.001);
    }

    #[test]
    fn test_digamma() {
        // ψ(1) = -γ (Euler-Mascheroni constant)
        let gamma_euler = 0.5772156649015329;
        assert!((digamma(1.0) - (-gamma_euler)).abs() < 0.001);
    }

    #[test]
    fn special_functions_reject_non_terminating_or_unrepresentable_orders() {
        assert!(digamma(f64::NEG_INFINITY).is_nan());
        assert!(bessel_j(i32::MIN, 1.0).is_nan());
        assert!(bessel_y(i32::MIN, 1.0).is_nan());
        assert!(bessel_i(i32::MIN, 1.0).is_nan());
    }

    #[test]
    fn bessel_y_rejects_orders_beyond_the_recurrence_budget() {
        assert!(bessel_y(MAX_BESSEL_Y_ORDER + 1, 1.0).is_nan());
        assert!(!bessel_y_order_is_supported(MAX_BESSEL_Y_ORDER + 1));
    }

    #[test]
    fn bessel_j_rejects_orders_beyond_the_recurrence_budget() {
        assert_eq!(MAX_BESSEL_J_ORDER, MAX_BESSEL_Y_ORDER);
        assert!(bessel_j(MAX_BESSEL_J_ORDER, 1.0).is_finite());
        assert!(bessel_j(MAX_BESSEL_J_ORDER + 1, 1.0).is_nan());
        assert!(bessel_j(-(MAX_BESSEL_J_ORDER + 1), 1.0).is_nan());
        assert!(bessel_j(i32::MAX, 1.0).is_nan());
    }

    #[test]
    fn bessel_i_rejects_orders_beyond_the_series_budget() {
        assert_eq!(MAX_BESSEL_I_ORDER, MAX_BESSEL_Y_ORDER);
        assert!(bessel_i_order_is_supported(MAX_BESSEL_I_ORDER));
        assert!(bessel_i(MAX_BESSEL_I_ORDER + 1, 1.0).is_nan());
        assert!(bessel_i(-(MAX_BESSEL_I_ORDER + 1), 1.0).is_nan());
        assert!(bessel_i(i32::MAX, 1.0).is_nan());
    }

    #[test]
    fn test_trigamma_standard_values_and_poles() {
        let pi_squared_over_six = std::f64::consts::PI.powi(2) / 6.0;
        assert!((trigamma(1.0) - pi_squared_over_six).abs() < 1e-12);
        assert!((trigamma(2.0) - (pi_squared_over_six - 1.0)).abs() < 1e-12);
        assert!(!trigamma(0.0).is_finite());
        assert!(!trigamma(-1.0).is_finite());
        assert!(trigamma(-1_000_000.5).is_finite());
    }
}
