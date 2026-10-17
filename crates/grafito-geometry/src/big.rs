//! Enteros grandes y racionales arbitrarios para la ola O5 (números y optimización).
//!
//! Slice atómico 5/5 de `Tasks.md` O5 (1/5 Gauss-Kronrod QK15 en `integral.rs`,
//! 2/5 FFT en `fft.rs`, 3/5 redondeo dirigido en `interval.rs`, 4/5 BFGS en
//! `optimize.rs`): aritmética exacta de precisión arbitraria sobre
//! [`num_bigint::BigInt`] y [`num_rational::BigRational`]. El cableado a
//! comandos queda como cierre de O5 (motores puros acá, sin call site todavía).
//!
//! API de strings (pensada para el futuro comando: entra texto, sale texto o
//! error honesto). Todo borde devuelve `Err`: vacío, dígito inválido, entrada
//! de más de [`BIG_MAX_DIGITS`] dígitos, división por cero, `0^0`, factorial
//! de `n > `[`BIG_FACTORIAL_MAX_N`], resultado de más de [`BIG_MAX_DIGITS`]
//! dígitos. La división entera trunca hacia cero (como `/` en Rust).
//!
//! Presupuestos: [`BIG_MAX_DIGITS`] = 10 000 dígitos por operando de entrada
//! y por resultado; [`BIG_FACTORIAL_MAX_N`] = 2000 (`2000!` tiene 5736
//! dígitos); [`BIG_DECIMAL_MAX_PLACES`] = 2000 decimales; exponente de
//! potencia acotado por la estimación `dígitos(base) × exp ≤ tope` ANTES de
//! computar (sin reservar de más).

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, Signed, Zero};
use std::fmt;

/// Dígitos decimales máximos por operando de entrada y por resultado.
pub const BIG_MAX_DIGITS: usize = 10_000;
/// `n` máximo de `bigint_factorial` (`2000!` tiene 5736 dígitos, entra holgado).
pub const BIG_FACTORIAL_MAX_N: u32 = 2000;
/// Decimales máximos de `bigrational_to_decimal` (truncado, no redondeado).
pub const BIG_DECIMAL_MAX_PLACES: usize = 2000;

/// Error honesto de la aritmética grande (sin pánicos: todo borde es `Err`).
#[derive(Debug, Clone, PartialEq)]
pub enum BigError {
    /// Entrada vacía (solo espacios/signo).
    Empty,
    /// Caracter no dígito en la posición `index` (índice sobre el texto
    /// recortado, contando desde 0).
    InvalidCharacter { index: usize, ch: char },
    /// El operando de entrada tiene `digits` dígitos (tope `max`).
    InputTooLong { digits: usize, max: usize },
    /// Denominador cero (`n/0`) o divisor racional cero.
    DivisionByZero,
    /// `0^0` (indeterminado: ni 1 ni error silencioso).
    UndefinedZeroToZero,
    /// Parámetro entero fuera de tope (`value` pedido, `max` permitido).
    TooLarge { value: u64, max: u64 },
    /// El resultado exacto tiene `digits` dígitos (tope `max`).
    ResultTooLarge { digits: usize, max: usize },
}

impl fmt::Display for BigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "entero vacío: se requiere al menos un dígito"),
            Self::InvalidCharacter { index, ch } => {
                write!(f, "caracter inválido '{ch}' en la posición {index}")
            }
            Self::InputTooLong { digits, max } => {
                write!(
                    f,
                    "el operando tiene {digits} dígitos y excede el máximo {max}"
                )
            }
            Self::DivisionByZero => write!(f, "división por cero"),
            Self::UndefinedZeroToZero => write!(f, "0^0 no está definido"),
            Self::TooLarge { value, max } => {
                write!(f, "parámetro {value} excede el máximo {max}")
            }
            Self::ResultTooLarge { digits, max } => {
                write!(
                    f,
                    "el resultado tiene {digits} dígitos y excede el máximo {max}"
                )
            }
        }
    }
}

impl std::error::Error for BigError {}

/// Dígitos decimales de `value` (sin contar el signo).
fn digit_len(value: &BigInt) -> usize {
    let text = value.to_string();
    text.len() - usize::from(text.starts_with('-'))
}

/// Parsea un entero decimal con signo opcional y `_` separadores.
pub fn parse_bigint(text: &str) -> Result<BigInt, BigError> {
    let body = text.trim();
    let digits = body
        .strip_prefix(['+', '-'])
        .unwrap_or(body)
        .replace('_', "");
    if digits.is_empty() {
        return Err(BigError::Empty);
    }
    if let Some((index, ch)) = digits.char_indices().find(|(_, ch)| !ch.is_ascii_digit()) {
        return Err(BigError::InvalidCharacter { index, ch });
    }
    if digits.len() > BIG_MAX_DIGITS {
        return Err(BigError::InputTooLong {
            digits: digits.len(),
            max: BIG_MAX_DIGITS,
        });
    }
    let mut clean = String::with_capacity(body.len());
    if body.starts_with('-') {
        clean.push('-');
    }
    clean.push_str(&digits);
    clean.parse::<BigInt>().map_err(|_| BigError::Empty)
}

/// Parsea `n`, `n/d` o decimal `3.25` (signo líder, `_` permitido).
pub fn parse_bigrational(text: &str) -> Result<BigRational, BigError> {
    let body = text.trim();
    if body.is_empty() {
        return Err(BigError::Empty);
    }
    if let Some((num_text, den_text)) = body.split_once('/') {
        if den_text.contains('/') {
            let index = body.find('/').unwrap_or(0) + 1 + den_text.find('/').unwrap_or(0);
            return Err(BigError::InvalidCharacter { index, ch: '/' });
        }
        let num = parse_bigint(num_text)?;
        let den = parse_bigint(den_text)?;
        if den.is_zero() {
            return Err(BigError::DivisionByZero);
        }
        return Ok(BigRational::new(num, den));
    }
    if let Some((int_text, frac_text)) = body.split_once('.') {
        if frac_text.contains('.') {
            let index = body.find('.').unwrap_or(0) + 1 + frac_text.find('.').unwrap_or(0);
            return Err(BigError::InvalidCharacter { index, ch: '.' });
        }
        let negative = int_text.trim().starts_with('-');
        let int_part = parse_bigint(if int_text.trim().is_empty() {
            "0"
        } else {
            int_text
        })?;
        let frac_clean = frac_text.replace('_', "");
        if frac_clean.is_empty() {
            return Ok(BigRational::from_integer(int_part));
        }
        if let Some((index, ch)) = frac_clean
            .char_indices()
            .find(|(_, ch)| !ch.is_ascii_digit())
        {
            let base = body.find('.').unwrap_or(0) + 1;
            return Err(BigError::InvalidCharacter {
                index: base + index,
                ch,
            });
        }
        if frac_clean.len() > BIG_MAX_DIGITS {
            return Err(BigError::InputTooLong {
                digits: frac_clean.len(),
                max: BIG_MAX_DIGITS,
            });
        }
        let scale = BigInt::from(10).pow(frac_clean.len() as u32);
        let frac_part: BigInt = if frac_clean.trim_start_matches('0').is_empty() {
            BigInt::zero()
        } else {
            frac_clean.parse::<BigInt>().map_err(|_| BigError::Empty)?
        };
        let scaled = int_part.abs() * &scale + frac_part;
        let num = if negative { -scaled } else { scaled };
        return Ok(BigRational::new(num, scale));
    }
    parse_bigint(body).map(BigRational::from_integer)
}

/// Renderiza un racional exacto: `n` si es entero, `n/d` normalizado si no.
#[must_use]
pub fn render_bigrational(value: &BigRational) -> String {
    if value.denom() == &BigInt::one() {
        value.numer().to_string()
    } else {
        format!("{}/{}", value.numer(), value.denom())
    }
}

/// Chequea el tope de salida sobre el texto ya computado.
fn check_rendered(text: &str) -> Result<String, BigError> {
    let digits = text.chars().filter(char::is_ascii_digit).count();
    if digits > BIG_MAX_DIGITS {
        return Err(BigError::ResultTooLarge {
            digits,
            max: BIG_MAX_DIGITS,
        });
    }
    Ok(text.to_string())
}

/// Suma exacta `a + b`.
pub fn bigint_add(a: &str, b: &str) -> Result<String, BigError> {
    let x = parse_bigint(a)?;
    let y = parse_bigint(b)?;
    if digit_len(&x).max(digit_len(&y)) + 1 > BIG_MAX_DIGITS {
        return Err(BigError::ResultTooLarge {
            digits: digit_len(&x).max(digit_len(&y)) + 1,
            max: BIG_MAX_DIGITS,
        });
    }
    check_rendered(&(x + y).to_string())
}

/// Resta exacta `a - b`.
pub fn bigint_sub(a: &str, b: &str) -> Result<String, BigError> {
    let x = parse_bigint(a)?;
    let y = parse_bigint(b)?;
    if digit_len(&x).max(digit_len(&y)) + 1 > BIG_MAX_DIGITS {
        return Err(BigError::ResultTooLarge {
            digits: digit_len(&x).max(digit_len(&y)) + 1,
            max: BIG_MAX_DIGITS,
        });
    }
    check_rendered(&(x - y).to_string())
}

/// Producto exacto `a * b` (tope estimado ANTES de multiplicar).
pub fn bigint_mul(a: &str, b: &str) -> Result<String, BigError> {
    let x = parse_bigint(a)?;
    let y = parse_bigint(b)?;
    if x.is_zero() || y.is_zero() {
        return Ok("0".to_string());
    }
    let estimate = digit_len(&x) + digit_len(&y);
    if estimate > BIG_MAX_DIGITS {
        return Err(BigError::ResultTooLarge {
            digits: estimate,
            max: BIG_MAX_DIGITS,
        });
    }
    check_rendered(&(x * y).to_string())
}

/// Cociente y resto truncados hacia cero (`a = q*b + r`, `|r| < |b|`).
pub fn bigint_div_rem(a: &str, b: &str) -> Result<(String, String), BigError> {
    let x = parse_bigint(a)?;
    let y = parse_bigint(b)?;
    if y.is_zero() {
        return Err(BigError::DivisionByZero);
    }
    let quotient = &x / &y;
    let rest = &x % &y;
    Ok((check_rendered(&quotient.to_string())?, rest.to_string()))
}

/// Potencia exacta `base^exp` (tope estimado ANTES de computar).
pub fn bigint_pow(base: &str, exp: u32) -> Result<String, BigError> {
    let x = parse_bigint(base)?;
    if x.is_zero() {
        if exp == 0 {
            return Err(BigError::UndefinedZeroToZero);
        }
        return Ok("0".to_string());
    }
    if exp == 0 {
        return Ok("1".to_string());
    }
    let base_digits = digit_len(&x);
    let estimate = base_digits.saturating_mul(exp as usize);
    if estimate > BIG_MAX_DIGITS {
        return Err(BigError::ResultTooLarge {
            digits: estimate,
            max: BIG_MAX_DIGITS,
        });
    }
    check_rendered(&x.pow(exp).to_string())
}

/// Factorial exacto `n!` con `n ≤ `[`BIG_FACTORIAL_MAX_N`].
pub fn bigint_factorial(n: u32) -> Result<String, BigError> {
    if n > BIG_FACTORIAL_MAX_N {
        return Err(BigError::TooLarge {
            value: u64::from(n),
            max: u64::from(BIG_FACTORIAL_MAX_N),
        });
    }
    let mut acc = BigInt::one();
    for k in 2..=n {
        acc *= BigInt::from(k);
    }
    check_rendered(&acc.to_string())
}

/// MCD no negativo de `a` y `b` (Euclides; `gcd(0,0) = 0`).
pub fn bigint_gcd(a: &str, b: &str) -> Result<String, BigError> {
    let mut x = parse_bigint(a)?.abs();
    let mut y = parse_bigint(b)?.abs();
    while !y.is_zero() {
        let rest = x % &y;
        x = y;
        y = rest;
    }
    check_rendered(&x.to_string())
}

/// Suma exacta de racionales (`n` o `n/d`).
pub fn bigrational_add(a: &str, b: &str) -> Result<String, BigError> {
    let x = parse_bigrational(a)?;
    let y = parse_bigrational(b)?;
    Ok(render_bigrational(&(x + y)))
}

/// Resta exacta de racionales.
pub fn bigrational_sub(a: &str, b: &str) -> Result<String, BigError> {
    let x = parse_bigrational(a)?;
    let y = parse_bigrational(b)?;
    Ok(render_bigrational(&(x - y)))
}

/// Producto exacto de racionales.
pub fn bigrational_mul(a: &str, b: &str) -> Result<String, BigError> {
    let x = parse_bigrational(a)?;
    let y = parse_bigrational(b)?;
    Ok(render_bigrational(&(x * y)))
}

/// Cociente exacto de racionales (divisor cero → error honesto, sin pánico).
pub fn bigrational_div(a: &str, b: &str) -> Result<String, BigError> {
    let x = parse_bigrational(a)?;
    let y = parse_bigrational(b)?;
    if y.is_zero() {
        return Err(BigError::DivisionByZero);
    }
    Ok(render_bigrational(&(x / y)))
}

/// Expansión decimal truncada con exactamente `places` decimales.
pub fn bigrational_to_decimal(rational: &str, places: usize) -> Result<String, BigError> {
    if places > BIG_DECIMAL_MAX_PLACES {
        return Err(BigError::TooLarge {
            value: places as u64,
            max: BIG_DECIMAL_MAX_PLACES as u64,
        });
    }
    let value = parse_bigrational(rational)?;
    if places == 0 {
        return Ok((value.numer() / value.denom()).to_string());
    }
    let negative = value.is_negative();
    let abs_num = value.numer().abs();
    let den = value.denom().clone();
    let int_part = &abs_num / &den;
    let mut rest = &abs_num % &den;
    let mut frac = String::with_capacity(places);
    let ten = BigInt::from(10);
    for _ in 0..places {
        rest *= &ten;
        let digit = &rest / &den;
        rest %= &den;
        let byte = b'0' + u8::try_from(&digit).unwrap_or(9);
        frac.push(byte as char);
        if rest.is_zero() {
            while frac.len() < places {
                frac.push('0');
            }
            break;
        }
    }
    let mut out = String::with_capacity(int_part.to_string().len() + places + 2);
    if negative {
        out.push('-');
    }
    out.push_str(&int_part.to_string());
    out.push('.');
    out.push_str(&frac);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enteros_grandes_suman_y_multiplican_exactos() {
        assert_eq!(
            bigint_add("99999999999999999999999999999", "1").unwrap(),
            "100000000000000000000000000000"
        );
        assert_eq!(
            bigint_mul("123456789", "987654321").unwrap(),
            "121932631112635269"
        );
        assert_eq!(bigint_sub("1000", "1001").unwrap(), "-1");
    }

    #[test]
    fn potencia_y_factorial_exactos() {
        assert_eq!(
            bigint_pow("2", 100).unwrap(),
            "1267650600228229401496703205376"
        );
        assert_eq!(bigint_pow("-3", 3).unwrap(), "-27");
        assert_eq!(bigint_pow("5", 0).unwrap(), "1");
        assert_eq!(bigint_factorial(0).unwrap(), "1");
        assert_eq!(bigint_factorial(20).unwrap(), "2432902008176640000");
    }

    #[test]
    fn division_trunca_y_mcd() {
        assert_eq!(
            bigint_div_rem("-7", "2").unwrap(),
            ("-3".to_string(), "-1".to_string())
        );
        assert_eq!(bigint_gcd("48", "18").unwrap(), "6");
        assert_eq!(bigint_gcd("0", "0").unwrap(), "0");
    }

    #[test]
    fn racionales_exactos_normalizados() {
        assert_eq!(bigrational_add("1/3", "1/6").unwrap(), "1/2");
        assert_eq!(bigrational_sub("1/2", "3/4").unwrap(), "-1/4");
        assert_eq!(bigrational_mul("2/3", "9/4").unwrap(), "3/2");
        assert_eq!(bigrational_div("1/2", "3/4").unwrap(), "2/3");
        assert_eq!(
            parse_bigrational("3.25").unwrap(),
            BigRational::new(13.into(), 4.into())
        );
        assert_eq!(bigrational_add("0.1", "0.2").unwrap(), "3/10");
    }

    #[test]
    fn decimal_trunca_sin_redondear() {
        assert_eq!(bigrational_to_decimal("1/3", 10).unwrap(), "0.3333333333");
        assert_eq!(bigrational_to_decimal("-7/2", 2).unwrap(), "-3.50");
        assert_eq!(bigrational_to_decimal("4", 3).unwrap(), "4.000");
    }

    #[test]
    fn errores_honestos_en_todos_los_bordes() {
        assert_eq!(parse_bigint("").unwrap_err(), BigError::Empty);
        assert_eq!(
            parse_bigint("12a4").unwrap_err(),
            BigError::InvalidCharacter { index: 2, ch: 'a' }
        );
        assert_eq!(
            bigint_div_rem("1", "0").unwrap_err(),
            BigError::DivisionByZero
        );
        assert_eq!(
            parse_bigrational("3/0").unwrap_err(),
            BigError::DivisionByZero
        );
        assert_eq!(
            bigint_pow("0", 0).unwrap_err(),
            BigError::UndefinedZeroToZero
        );
        assert_eq!(
            bigint_factorial(BIG_FACTORIAL_MAX_N + 1).unwrap_err(),
            BigError::TooLarge {
                value: u64::from(BIG_FACTORIAL_MAX_N) + 1,
                max: u64::from(BIG_FACTORIAL_MAX_N),
            }
        );
        assert!(matches!(
            parse_bigint(&format!("1{}", "0".repeat(BIG_MAX_DIGITS))).unwrap_err(),
            BigError::InputTooLong { .. }
        ));
        assert!(matches!(
            bigint_pow("10", 100_000).unwrap_err(),
            BigError::ResultTooLarge { .. }
        ));
    }

    #[test]
    fn separadores_y_signos() {
        assert_eq!(parse_bigint("1_000_000").unwrap(), BigInt::from(1_000_000));
        assert_eq!(parse_bigint("+42").unwrap(), BigInt::from(42));
        assert_eq!(parse_bigint("  -7 ").unwrap(), BigInt::from(-7));
        assert_eq!(
            parse_bigrational("-3/2").unwrap(),
            BigRational::new((-3).into(), 2.into())
        );
    }
}
