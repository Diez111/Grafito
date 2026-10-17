//! FFT discreta radix-2 para la ola O5 (números y optimización).
//!
//! Slice atómico 2/5 de `Tasks.md` O5 (1/5 ya hecho: Gauss-Kronrod QK15 +
//! adaptativo en `integral.rs`, sin cableado a comando todavía; 3/5 ya hecho:
//! operaciones con redondeo dirigido `add`/`sub`/`mul`/`div` en `interval.rs`):
//! DFT directa e inversa iterativa Cooley–Tukey sobre [`num_complex::Complex64`], más
//! convolución lineal vía FFT con cero-relleno a la próxima potencia de dos.
//!
//! Núcleo numérico puro sobre muestras: no introduce ningún tipo de expresión
//! y por tanto respeta la regla fija de UNA sola representación (`Expr`/
//! bytecode para lo simbólico; aquí solo `f64`/`Complex64`).
//!
//! Lo que cubre este módulo: `fft_forward`, `fft_inverse`, `fft_forward_real`,
//! `dft_naive` (referencia O(n²) para tests, tope 1024), `convolution` lineal
//! y `pad_to_power_of_two`. Convención de signo: directa con `exp(−2πi/n)`
//! (la de `numpy.fft.fft`), inversa con `exp(+2πi/n)` y escala `1/n`.
//!
//! Lo que NO cubre (resto de O5, abierto): el cableado a comandos de los
//! motores Gauss-Kronrod, FFT, BFGS y bigint/bigrational (`optimize.rs` ya
//! tiene `bfgs_minimize`, `big.rs` la aritmética exacta; operaciones dirigidas
//! sobre `Interval` ya hechas en `interval.rs`).
//!
//! Presupuestos: `FFT_MAX_LEN` = 2²⁰ puntos (~16 MiB por buffer `Complex64`);
//! todo lo que exceda responde `TooLong` honesto sin reservar. Las entradas no
//! finitas y las salidas desbordadas (`1e308 + 1e308`) responden `NonFinite`
//! en vez de propagar `NaN`/`inf` en silencio.

use num_complex::Complex64;
use std::f64::consts::PI;
use std::fmt;

/// Tope de puntos por transformada (2²⁰ ≈ 1M; un buffer son ~16 MiB).
pub const FFT_MAX_LEN: usize = 1 << 20;
/// Tope de la DFT ingenua O(n²) (referencia de tests, no vía de cómputo).
pub const DFT_NAIVE_MAX_LEN: usize = 1024;

/// Error honesto de la FFT (sin pánicos: todo borde devuelve `Err`).
#[derive(Debug, Clone, PartialEq)]
pub enum FftError {
    /// Entrada vacía (la DFT del vacío no está definida).
    Empty,
    /// Longitud que excede el presupuesto sin reservar nada.
    TooLong { len: usize, max: usize },
    /// La radix-2 exige potencia de dos exacta (ver `pad_to_power_of_two`).
    NotPowerOfTwo { len: usize },
    /// Muestra no finita (`NaN`/`inf`); `index` es la posición orientativa
    /// en la entrada (o en la salida si el cómputo desbordó).
    NonFinite { index: usize },
}

impl fmt::Display for FftError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "FFT vacía: se requiere al menos una muestra"),
            Self::TooLong { len, max } => {
                write!(f, "FFT de {len} puntos excede el máximo {max}")
            }
            Self::NotPowerOfTwo { len } => write!(
                f,
                "FFT radix-2 exige potencia de dos (llegaron {len} puntos; use pad_to_power_of_two)"
            ),
            Self::NonFinite { index } => {
                write!(f, "muestra no finita en la posición {index}")
            }
        }
    }
}

impl std::error::Error for FftError {}

/// `true` sii `n` es potencia de dos (0 no lo es).
#[must_use]
pub fn is_power_of_two(n: usize) -> bool {
    n.is_power_of_two()
}

/// Próxima potencia de dos `>= n` (`0 → 1`); `None` si desborda o supera
/// [`FFT_MAX_LEN`] (honesto, sin reservar ni pánicos en debug).
#[must_use]
pub fn next_power_of_two_len(n: usize) -> Option<usize> {
    // `checked_*`: `next_power_of_two` a secas hace panic por overflow en
    // debug (`usize::MAX`), cazado por el test de errores honestos.
    let next = n.checked_next_power_of_two()?;
    if next > FFT_MAX_LEN {
        return None;
    }
    Some(next)
}

/// Valida longitud radix-2 sin reservar (para pinear el presupuesto en tests).
pub fn check_fft_len(len: usize) -> Result<(), FftError> {
    if len == 0 {
        return Err(FftError::Empty);
    }
    if len > FFT_MAX_LEN {
        return Err(FftError::TooLong {
            len,
            max: FFT_MAX_LEN,
        });
    }
    if !len.is_power_of_two() {
        return Err(FftError::NotPowerOfTwo { len });
    }
    Ok(())
}

fn check_finite(data: &[Complex64]) -> Result<(), FftError> {
    for (i, c) in data.iter().enumerate() {
        if !c.re.is_finite() || !c.im.is_finite() {
            return Err(FftError::NonFinite { index: i });
        }
    }
    Ok(())
}

fn check_finite_output(data: &[Complex64]) -> Result<(), FftError> {
    for (i, c) in data.iter().enumerate() {
        if !c.re.is_finite() || !c.im.is_finite() {
            return Err(FftError::NonFinite { index: i });
        }
    }
    Ok(())
}

/// Núcleo radix-2 iterativo in-place (`invert` = signo `+` con escala `1/n`).
fn fft_in_place(data: &mut [Complex64], invert: bool) {
    let n = data.len();
    let mut j = 0_usize;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j ^= bit;
        if i < j {
            data.swap(i, j);
        }
    }
    let mut len = 2_usize;
    while len <= n {
        let half = len / 2;
        let ang = 2.0 * PI / len as f64 * if invert { 1.0 } else { -1.0 };
        let wlen = Complex64::new(ang.cos(), ang.sin());
        let mut i = 0_usize;
        while i < n {
            let mut w = Complex64::new(1.0, 0.0);
            for k in 0..half {
                let u = data[i + k];
                let v = data[i + k + half] * w;
                data[i + k] = u + v;
                data[i + k + half] = u - v;
                w *= wlen;
            }
            i += len;
        }
        len *= 2;
    }
    if invert {
        let inv = 1.0 / n as f64;
        for x in data.iter_mut() {
            *x *= inv;
        }
    }
}

/// DFT directa (`exp(−2πi/n)`): exige potencia de dos no vacía dentro del tope.
pub fn fft_forward(input: &[Complex64]) -> Result<Vec<Complex64>, FftError> {
    check_fft_len(input.len())?;
    check_finite(input)?;
    let mut data = input.to_vec();
    fft_in_place(&mut data, false);
    check_finite_output(&data)?;
    Ok(data)
}

/// DFT inversa (`exp(+2πi/n)` con escala `1/n`): misma validación que la directa.
pub fn fft_inverse(input: &[Complex64]) -> Result<Vec<Complex64>, FftError> {
    check_fft_len(input.len())?;
    check_finite(input)?;
    let mut data = input.to_vec();
    fft_in_place(&mut data, true);
    check_finite_output(&data)?;
    Ok(data)
}

/// DFT directa de muestras reales (parte imaginaria cero).
pub fn fft_forward_real(input: &[f64]) -> Result<Vec<Complex64>, FftError> {
    if input.is_empty() {
        return Err(FftError::Empty);
    }
    if input.len() > FFT_MAX_LEN {
        return Err(FftError::TooLong {
            len: input.len(),
            max: FFT_MAX_LEN,
        });
    }
    if !input.len().is_power_of_two() {
        return Err(FftError::NotPowerOfTwo { len: input.len() });
    }
    let mut data = Vec::with_capacity(input.len());
    for (i, &v) in input.iter().enumerate() {
        if !v.is_finite() {
            return Err(FftError::NonFinite { index: i });
        }
        data.push(Complex64::new(v, 0.0));
    }
    fft_in_place(&mut data, false);
    check_finite_output(&data)?;
    Ok(data)
}

/// DFT ingenua O(n²) como referencia (cualquier longitud, tope 1024).
pub fn dft_naive(input: &[Complex64]) -> Result<Vec<Complex64>, FftError> {
    if input.is_empty() {
        return Err(FftError::Empty);
    }
    if input.len() > DFT_NAIVE_MAX_LEN {
        return Err(FftError::TooLong {
            len: input.len(),
            max: DFT_NAIVE_MAX_LEN,
        });
    }
    check_finite(input)?;
    let n = input.len();
    let mut out = Vec::with_capacity(n);
    for k in 0..n {
        let mut acc = Complex64::new(0.0, 0.0);
        for (j, &x) in input.iter().enumerate() {
            let ang = -2.0 * PI * (k * j) as f64 / n as f64;
            acc += x * Complex64::new(ang.cos(), ang.sin());
        }
        out.push(acc);
    }
    check_finite_output(&out)?;
    Ok(out)
}

/// Cero-relleno hasta la próxima potencia de dos (dentro del tope).
pub fn pad_to_power_of_two(data: &[Complex64]) -> Result<Vec<Complex64>, FftError> {
    if data.is_empty() {
        return Err(FftError::Empty);
    }
    if data.len() > FFT_MAX_LEN {
        return Err(FftError::TooLong {
            len: data.len(),
            max: FFT_MAX_LEN,
        });
    }
    check_finite(data)?;
    let Some(target) = next_power_of_two_len(data.len()) else {
        return Err(FftError::TooLong {
            len: data.len(),
            max: FFT_MAX_LEN,
        });
    };
    let mut out = data.to_vec();
    out.resize(target, Complex64::new(0.0, 0.0));
    Ok(out)
}

/// Convolución lineal `c[k] = Σ a[i]·b[k−i]` vía FFT (cero-relleno a la
/// próxima potencia de dos de `a.len() + b.len() − 1`).
///
/// `NonFinite.index` va sobre la concatenada `a ++ b` para no ambiguar de qué
/// operando vino la muestra.
pub fn convolution(a: &[f64], b: &[f64]) -> Result<Vec<f64>, FftError> {
    if a.is_empty() || b.is_empty() {
        return Err(FftError::Empty);
    }
    let Some(out_len) = a.len().checked_add(b.len()).and_then(|s| s.checked_sub(1)) else {
        return Err(FftError::TooLong {
            len: usize::MAX,
            max: FFT_MAX_LEN,
        });
    };
    if out_len > FFT_MAX_LEN {
        return Err(FftError::TooLong {
            len: out_len,
            max: FFT_MAX_LEN,
        });
    }
    let Some(n) = next_power_of_two_len(out_len) else {
        return Err(FftError::TooLong {
            len: out_len,
            max: FFT_MAX_LEN,
        });
    };
    let mut fa = vec![Complex64::new(0.0, 0.0); n];
    let mut fb = vec![Complex64::new(0.0, 0.0); n];
    for (i, &v) in a.iter().enumerate() {
        if !v.is_finite() {
            return Err(FftError::NonFinite { index: i });
        }
        fa[i] = Complex64::new(v, 0.0);
    }
    for (j, &v) in b.iter().enumerate() {
        if !v.is_finite() {
            return Err(FftError::NonFinite { index: a.len() + j });
        }
        fb[j] = Complex64::new(v, 0.0);
    }
    fft_in_place(&mut fa, false);
    fft_in_place(&mut fb, false);
    for (x, y) in fa.iter_mut().zip(fb.iter()) {
        *x *= *y;
    }
    fft_in_place(&mut fa, true);
    let mut out = Vec::with_capacity(out_len);
    for (i, c) in fa.iter().take(out_len).enumerate() {
        if !c.re.is_finite() || !c.im.is_finite() {
            return Err(FftError::NonFinite { index: i });
        }
        out.push(c.re);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: Complex64, b: Complex64, tol: f64) -> bool {
        (a - b).norm() <= tol
    }

    #[test]
    fn impulso_da_espectro_plano() {
        let x = [
            Complex64::new(1.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0),
        ];
        let out = fft_forward(&x).expect("impulso válido");
        for c in &out {
            assert!(approx(*c, Complex64::new(1.0, 0.0), 1e-12), "plano: {c}");
        }
    }

    #[test]
    fn constante_da_pico_en_cero() {
        let x = [Complex64::new(1.0, 0.0); 4];
        let out = fft_forward(&x).expect("constante válida");
        assert!(approx(out[0], Complex64::new(4.0, 0.0), 1e-12));
        for c in out.iter().skip(1) {
            assert!(approx(*c, Complex64::new(0.0, 0.0), 1e-12), "cero: {c}");
        }
    }

    #[test]
    fn ida_y_vuelta_recupera_la_muestra() {
        let x = [1.0, 2.0, 3.0, 4.0, 0.5, -1.0, 2.5, 3.25].map(|v| Complex64::new(v, 0.0));
        let fwd = fft_forward(&x).expect("directa válida");
        let back = fft_inverse(&fwd).expect("inversa válida");
        for (got, want) in back.iter().zip(x.iter()) {
            assert!(approx(*got, *want, 1e-9), "roundtrip: {got} vs {want}");
        }
    }

    #[test]
    fn coincide_con_la_dft_ingenua() {
        let x = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0].map(|v| Complex64::new(v, 0.0));
        let rapida = fft_forward(&x).expect("rápida válida");
        let ingenua = dft_naive(&x).expect("ingenua válida");
        for (a, b) in rapida.iter().zip(ingenua.iter()) {
            assert!(approx(*a, *b, 1e-9), "paridad: {a} vs {b}");
        }
    }

    #[test]
    fn convolucion_lineal_da_valores_exactos() {
        let c = convolution(&[1.0, 2.0, 3.0], &[0.0, 1.0, 0.5]).expect("válida");
        let esperada = [0.0, 1.0, 2.5, 4.0, 1.5];
        assert_eq!(c.len(), esperada.len());
        for (got, want) in c.iter().zip(esperada.iter()) {
            assert!((got - want).abs() <= 1e-9, "conv: {got} vs {want}");
        }
    }

    #[test]
    fn errores_son_honestos_sin_panico() {
        assert_eq!(fft_forward(&[]), Err(FftError::Empty));
        assert_eq!(
            fft_forward(&[Complex64::new(1.0, 0.0); 3]),
            Err(FftError::NotPowerOfTwo { len: 3 })
        );
        let mut mala = [Complex64::new(1.0, 0.0); 4];
        mala[1] = Complex64::new(f64::NAN, 0.0);
        assert_eq!(fft_forward(&mala), Err(FftError::NonFinite { index: 1 }));
        assert_eq!(
            check_fft_len(FFT_MAX_LEN + 1),
            Err(FftError::TooLong {
                len: FFT_MAX_LEN + 1,
                max: FFT_MAX_LEN
            })
        );
        assert_eq!(convolution(&[], &[1.0]), Err(FftError::Empty));
        assert_eq!(
            convolution(&[1.0], &[f64::INFINITY]),
            Err(FftError::NonFinite { index: 1 })
        );
        // Desborde honesto: 1e308+1e308 no es finito ni en la salida.
        let grande = [Complex64::new(1e308, 0.0); 2];
        assert!(matches!(
            fft_forward(&grande),
            Err(FftError::NonFinite { .. })
        ));
        // La ingenua capa en 1024 sin colgar la máquina.
        let larga = vec![Complex64::new(1.0, 0.0); DFT_NAIVE_MAX_LEN + 1];
        assert_eq!(
            dft_naive(&larga),
            Err(FftError::TooLong {
                len: DFT_NAIVE_MAX_LEN + 1,
                max: DFT_NAIVE_MAX_LEN
            })
        );
        assert_eq!(next_power_of_two_len(0), Some(1));
        assert_eq!(next_power_of_two_len(3), Some(4));
        assert_eq!(next_power_of_two_len(usize::MAX), None);
        let padded = pad_to_power_of_two(&[Complex64::new(1.0, 0.0); 3]).expect("relleno válido");
        assert_eq!(padded.len(), 4);
    }

    #[test]
    fn real_rechaza_no_finitos() {
        assert_eq!(
            fft_forward_real(&[1.0, f64::NAN, 0.0, 0.0]),
            Err(FftError::NonFinite { index: 1 })
        );
        let out = fft_forward_real(&[1.0, 1.0, 1.0, 1.0]).expect("válida");
        assert!(approx(out[0], Complex64::new(4.0, 0.0), 1e-12));
    }
}
