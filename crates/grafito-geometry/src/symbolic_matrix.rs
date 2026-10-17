//! Matrices simbólicas sobre `Expr` para la ola O2 (no-conmutativo).
//!
//! Slice 2/3 de `Tasks.md` O2: el producto matricial es el otro ejemplo
//! canónico de álgebra no conmutativa (`A·B ≠ B·A` en general). A diferencia
//! de `matrices.rs` (`f64` numérico) y `exact_linalg.rs` (`ExactRational`
//! `i128`), las entradas son expresiones en la ÚNICA representación de
//! Grafito (`String` sobre el AST/`Expr` de `ast.rs`, canónica vía
//! `symbolic::expand`; regla fija de las olas 1-6).
//!
//! Alcance honesto: suma/resta/producto/traza/transpuesta generales,
//! determinante por Laplace (`n ≤ 5`, factorial acotado) e inversa por
//! adjunta (`n ≤ 3`). Todo borde devuelve `Err` en vez de pánico o `None`
//! mudo; cada entrada y cada salida respeta `MAX_CAS_EXPR_BYTES` 2000.

use crate::ast::parse_ast;
use crate::cas::MAX_CAS_EXPR_BYTES;
use crate::symbolic::{expand, is_identically_zero};

/// Lado máximo de una matriz simbólica (mismo espíritu que `MAX_EXACT_DIM`:
/// acota el costo cúbico/factorial y el desborde de salidas).
pub const MAX_SYMBOLIC_MATRIX_DIM: usize = 8;
/// Determinante solo hasta 5×5 (120 términos de Laplace como techo honesto).
pub const MAX_SYMBOLIC_DET_DIM: usize = 5;
/// Inversa por adjunta solo hasta 3×3 (9 menores 2×2 como techo honesto).
pub const MAX_SYMBOLIC_INV_DIM: usize = 3;

/// Matriz densa de expresiones (fila mayor, cada entrada un `String` sobre
/// el AST de Grafito).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolicMatrix {
    rows: usize,
    cols: usize,
    data: Vec<String>,
}

fn valida_entrada(expr: &str) -> Result<String, String> {
    let t = expr.trim();
    if t.is_empty() {
        return Err("entrada vacía en matriz simbólica".to_string());
    }
    if t.len() > MAX_CAS_EXPR_BYTES {
        return Err(format!(
            "entrada de {} bytes excede {MAX_CAS_EXPR_BYTES}; usa Eliminate[...]",
            t.len()
        ));
    }
    parse_ast(&t.replace(' ', "")).map_err(|e| format!("entrada no parseable '{t}': {e}"))?;
    Ok(t.to_string())
}

/// Canónica de una expresión combinada: `expand` cuando es polinómica,
/// cruda parentizada cuando no (p. ej. `sin(x)`), siempre ≤ 2000 bytes.
fn canonica(raw: &str) -> Result<String, String> {
    match expand(raw) {
        Ok(out) => {
            if out.len() > MAX_CAS_EXPR_BYTES {
                Err(format!(
                    "salida de {} bytes excede {MAX_CAS_EXPR_BYTES}",
                    out.len()
                ))
            } else {
                Ok(out)
            }
        }
        Err(_) => {
            // Fuera del subconjunto expandible (trig, etc.): conserva la
            // forma cruda si parsea y cabe en el presupuesto.
            parse_ast(&raw.replace(' ', ""))
                .map_err(|e| format!("no se pudo canonalizar '{raw}': {e}"))?;
            if raw.len() > MAX_CAS_EXPR_BYTES {
                return Err(format!(
                    "salida de {} bytes excede {MAX_CAS_EXPR_BYTES}",
                    raw.len()
                ));
            }
            Ok(raw.to_string())
        }
    }
}

fn combina(a: &str, op: char, b: &str) -> Result<String, String> {
    let raw = format!("({a}){op}({b})");
    canonica(&raw)
}

impl SymbolicMatrix {
    /// Construye validando dimensiones y cada entrada (parse + cota 2000).
    pub fn new(rows: usize, cols: usize, data: Vec<String>) -> Result<Self, String> {
        if rows == 0 || cols == 0 {
            return Err("matriz simbólica vacía (dims 0)".to_string());
        }
        if rows > MAX_SYMBOLIC_MATRIX_DIM || cols > MAX_SYMBOLIC_MATRIX_DIM {
            return Err(format!(
                "matriz {rows}x{cols} excede el máximo simbólico {MAX_SYMBOLIC_MATRIX_DIM}x{MAX_SYMBOLIC_MATRIX_DIM}"
            ));
        }
        if data.len() != rows.saturating_mul(cols) {
            return Err(format!("datos {} no calzan con {rows}x{cols}", data.len()));
        }
        let mut clean = Vec::with_capacity(data.len());
        for e in &data {
            clean.push(valida_entrada(e)?);
        }
        Ok(Self {
            rows,
            cols,
            data: clean,
        })
    }

    /// Construye desde filas (cada fila mismo largo, no vacías).
    pub fn from_rows(rows: Vec<Vec<String>>) -> Result<Self, String> {
        if rows.is_empty() || rows[0].is_empty() {
            return Err("matriz simbólica vacía (filas)".to_string());
        }
        let cols = rows[0].len();
        if rows.iter().any(|r| r.len() != cols) {
            return Err("filas ragged en matriz simbólica".to_string());
        }
        let data = rows.into_iter().flatten().collect::<Vec<_>>();
        Self::new(data.len() / cols, cols, data)
    }

    /// Dimensión `(filas, columnas)`.
    #[must_use]
    pub const fn dims(&self) -> (usize, usize) {
        (self.rows, self.cols)
    }

    /// Entrada `(r, c)` como texto canónico de construcción.
    #[must_use]
    pub fn get(&self, r: usize, c: usize) -> &str {
        &self.data[r * self.cols + c]
    }

    /// Transpuesta.
    pub fn transpose(&self) -> Result<Self, String> {
        let mut data = Vec::with_capacity(self.data.len());
        for c in 0..self.cols {
            for r in 0..self.rows {
                data.push(self.get(r, c).to_string());
            }
        }
        Self::new(self.cols, self.rows, data)
    }

    /// Suma entrada a entrada (misma dimensión).
    pub fn add(&self, other: &Self) -> Result<Self, String> {
        if self.dims() != other.dims() {
            return Err(format!(
                "suma {:?} + {:?}: dimensiones distintas",
                self.dims(),
                other.dims()
            ));
        }
        let mut data = Vec::with_capacity(self.data.len());
        for (a, b) in self.data.iter().zip(other.data.iter()) {
            data.push(combina(a, '+', b)?);
        }
        Self::new(self.rows, self.cols, data)
    }

    /// Resta entrada a entrada (misma dimensión).
    pub fn sub(&self, other: &Self) -> Result<Self, String> {
        if self.dims() != other.dims() {
            return Err(format!(
                "resta {:?} - {:?}: dimensiones distintas",
                self.dims(),
                other.dims()
            ));
        }
        let mut data = Vec::with_capacity(self.data.len());
        for (a, b) in self.data.iter().zip(other.data.iter()) {
            data.push(combina(a, '-', b)?);
        }
        Self::new(self.rows, self.cols, data)
    }

    /// Producto `self · other` (NO conmutativo): `self.cols == other.rows`.
    pub fn mul(&self, other: &Self) -> Result<Self, String> {
        if self.cols != other.rows {
            return Err(format!(
                "producto {:?} · {:?}: inner distinto",
                self.dims(),
                other.dims()
            ));
        }
        let mut data = Vec::with_capacity(self.rows * other.cols);
        for i in 0..self.rows {
            for j in 0..other.cols {
                let mut acc = "0".to_string();
                for k in 0..self.cols {
                    let term_raw = format!("({})*({})", self.get(i, k), other.get(k, j));
                    let term = canonica(&term_raw)?;
                    let next_raw = format!("({acc})+({term})");
                    acc = canonica(&next_raw)?;
                }
                if acc.len() > MAX_CAS_EXPR_BYTES {
                    return Err(format!(
                        "salida de {} bytes excede {MAX_CAS_EXPR_BYTES}",
                        acc.len()
                    ));
                }
                data.push(acc);
            }
        }
        Self::new(self.rows, other.cols, data)
    }

    /// Traza (solo cuadradas): suma de la diagonal.
    pub fn trace(&self) -> Result<String, String> {
        if self.rows != self.cols {
            return Err(format!("traza exige cuadrada ({:?})", self.dims()));
        }
        let mut acc = "0".to_string();
        for i in 0..self.rows {
            let next_raw = format!("({acc})+({})", self.get(i, i));
            acc = canonica(&next_raw)?;
        }
        Ok(acc)
    }

    fn minor(&self, skip_r: usize, skip_c: usize) -> Result<Self, String> {
        let mut data = Vec::with_capacity((self.rows - 1) * (self.cols - 1));
        for r in 0..self.rows {
            if r == skip_r {
                continue;
            }
            for c in 0..self.cols {
                if c == skip_c {
                    continue;
                }
                data.push(self.get(r, c).to_string());
            }
        }
        Self::new(self.rows - 1, self.cols - 1, data)
    }

    /// Determinante por Laplace (`n ≤ 5`); `Err` honesto más allá.
    pub fn determinant(&self) -> Result<String, String> {
        if self.rows != self.cols {
            return Err(format!("determinante exige cuadrada ({:?})", self.dims()));
        }
        let n = self.rows;
        if n > MAX_SYMBOLIC_DET_DIM {
            return Err(format!(
                "determinante {n}x{n} excede {MAX_SYMBOLIC_DET_DIM}x{MAX_SYMBOLIC_DET_DIM} (Laplace factorial); usa Eliminate[...]"
            ));
        }
        if n == 1 {
            return Ok(self.get(0, 0).to_string());
        }
        if n == 2 {
            let raw = format!(
                "({})*({})-({})*({})",
                self.get(0, 0),
                self.get(1, 1),
                self.get(0, 1),
                self.get(1, 0)
            );
            return canonica(&raw);
        }
        let mut acc = "0".to_string();
        for j in 0..n {
            let cofactor = self.minor(0, j)?.determinant()?;
            let term_raw = format!("({})*({})", self.get(0, j), cofactor);
            let term = canonica(&term_raw)?;
            let signed = if j % 2 == 0 {
                format!("({acc})+({term})")
            } else {
                format!("({acc})-({term})")
            };
            acc = canonica(&signed)?;
        }
        Ok(acc)
    }

    fn es_cero(expr: &str) -> bool {
        let t = expr.trim().replace(' ', "");
        if t == "0" || t == "0.0" {
            return true;
        }
        // Cero polinómico no trivial (`x*x-x*x`): el `expand` lo deja
        // sin colectar, pero `is_identically_zero` (coeficientes) lo ve.
        parse_ast(&t).is_ok_and(|ast| is_identically_zero(&ast))
    }

    /// Inversa por adjunta (`n ≤ 3`); `Err` honesto si singular o mayor.
    pub fn inverse(&self) -> Result<Self, String> {
        if self.rows != self.cols {
            return Err(format!("inversa exige cuadrada ({:?})", self.dims()));
        }
        let n = self.rows;
        if n > MAX_SYMBOLIC_INV_DIM {
            return Err(format!(
                "inversa {n}x{n} excede {MAX_SYMBOLIC_INV_DIM}x{MAX_SYMBOLIC_INV_DIM} (adjunta); usa Eliminate[...]"
            ));
        }
        let det = self.determinant()?;
        if Self::es_cero(&det) {
            return Err("matriz simbólica singular (det = 0)".to_string());
        }
        if n == 1 {
            if Self::es_cero(self.get(0, 0)) {
                return Err("matriz simbólica singular (det = 0)".to_string());
            }
            return Self::new(1, 1, vec![canonica(&format!("1/({})", self.get(0, 0)))?]);
        }
        // Cofactores `C[i][j] = (-1)^{i+j} det(minor)`, adjunta = transpuesta.
        let mut cof = vec!["0".to_string(); n * n];
        for i in 0..n {
            for j in 0..n {
                let m = self.minor(i, j)?.determinant()?;
                let raw = if (i + j) % 2 == 0 {
                    format!("({m})")
                } else {
                    format!("-( {m} )").replace(' ', "")
                };
                cof[i * n + j] = canonica(&raw)?;
            }
        }
        let mut data = Vec::with_capacity(n * n);
        for i in 0..n {
            for j in 0..n {
                // Adjunta[i][j] = Cof[j][i] dividido por det.
                let raw = format!("({})/({det})", cof[j * n + i]);
                data.push(canonica(&raw)?);
            }
        }
        Self::new(n, n, data)
    }

    /// Render multilínea estilo matriz `{{a, b}, {c, d}}`.
    #[must_use]
    pub fn to_display_string(&self) -> String {
        let mut rows = Vec::with_capacity(self.rows);
        for r in 0..self.rows {
            let mut entries = Vec::with_capacity(self.cols);
            for c in 0..self.cols {
                entries.push(self.get(r, c).to_string());
            }
            rows.push(format!("{{{}}}", entries.join(", ")));
        }
        format!("{{{}}}", rows.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::parse_ast as parse;

    fn m(rows: &[&[&str]]) -> SymbolicMatrix {
        let owned = rows
            .iter()
            .map(|r| r.iter().map(|s| (*s).to_string()).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        SymbolicMatrix::from_rows(owned).expect("matriz válida")
    }

    /// Equivalencia polinómica (el `expand` no colecta: `x+x` queda crudo).
    fn equivale(got: &str, want: &str) -> bool {
        let diff = format!("({got})-({want})");
        parse(&diff.replace(' ', "")).is_ok_and(|ast| is_identically_zero(&ast))
    }

    fn eval_en(expr: &str, x: f64) -> f64 {
        crate::expr::evaluate(expr, &[("x".to_string(), x)]).expect("eval")
    }

    #[test]
    fn suma_y_resta_canonizan() {
        let a = m(&[&["x+1", "0"], &["0", "x+1"]]);
        let b = m(&[&["x", "1"], &["1", "x"]]);
        let s = a.add(&b).expect("suma");
        assert!(equivale(s.get(0, 0), "2*x+1"), "suma: {}", s.get(0, 0));
        assert!(equivale(s.get(0, 1), "1"), "suma01: {}", s.get(0, 1));
        let d = a.sub(&b).expect("resta");
        assert!(equivale(d.get(0, 0), "1"), "resta: {}", d.get(0, 0));
    }

    #[test]
    fn producto_no_conmuta() {
        let a = m(&[&["x", "1"], &["0", "x"]]);
        let b = m(&[&["x", "0"], &["1", "x"]]);
        let ab = a.mul(&b).expect("A·B");
        let ba = b.mul(&a).expect("B·A");
        // A·B[0][0] = x*x+1, B·A[0][0] = x*x: difieren en el +1.
        assert!(equivale(ab.get(0, 0), "x*x+1"), "A·B: {}", ab.get(0, 0));
        assert!(equivale(ba.get(0, 0), "x*x"), "B·A: {}", ba.get(0, 0));
        assert_ne!(ab.get(0, 0), ba.get(0, 0));
        // Traza de A = 2*x.
        assert!(equivale(&a.trace().expect("traza"), "2*x"));
    }

    #[test]
    fn determinante_2x2_simbolico() {
        let a = m(&[&["x", "1"], &["0", "x"]]);
        assert!(equivale(&a.determinant().expect("det"), "x*x"));
    }

    #[test]
    fn determinante_3x3_diagonal() {
        let a = m(&[&["x", "0", "0"], &["0", "x", "0"], &["0", "0", "x"]]);
        assert!(equivale(&a.determinant().expect("det3"), "x*x*x"));
    }

    #[test]
    fn inversa_2x2_verifica_identidad() {
        let a = m(&[&["x", "1"], &["0", "x"]]);
        let inv = a.inverse().expect("inversa");
        let prod = a.mul(&inv).expect("producto");
        // A·A⁻¹ = I en x=2 y x=3 (las entradas con `/x` no tienen
        // dominio total en 0 y `is_identically_zero` no aplica).
        for x in [2.0, 3.0] {
            for (r, c, want) in [(0, 0, 1.0), (0, 1, 0.0), (1, 0, 0.0), (1, 1, 1.0)] {
                let v = eval_en(prod.get(r, c), x);
                assert!(
                    (v - want).abs() < 1e-9,
                    "prod[{r}][{c}] en x={x}: {v} (expr {})",
                    prod.get(r, c)
                );
            }
        }
    }

    #[test]
    fn inversa_singular_es_error_honesto() {
        let a = m(&[&["x", "x"], &["x", "x"]]);
        let err = a.inverse().expect_err("singular");
        assert!(err.contains("singular"), "inesperado: {err}");
    }

    #[test]
    fn cotas_y_entradas_honestas() {
        assert!(SymbolicMatrix::new(0, 2, vec![]).is_err());
        assert!(SymbolicMatrix::new(9, 1, vec!["x".to_string(); 9]).is_err());
        assert!(SymbolicMatrix::from_rows(vec![vec!["x".to_string()], vec![]]).is_err());
        assert!(SymbolicMatrix::new(1, 1, vec!["".to_string()]).is_err());
        assert!(SymbolicMatrix::new(1, 1, vec!["x+*".to_string()]).is_err());
        // 1x2 · 1x1: inner 2 ≠ 1 → error honesto (el caso 1x1·1x2 sí vale).
        assert!(m(&[&["y", "z"]]).mul(&m(&[&["x"]])).is_err());
        let wide = SymbolicMatrix::new(1, 6, vec!["x".to_string(); 6]).expect("1x6");
        assert!(wide.determinant().is_err());
        let m6 = SymbolicMatrix::new(6, 6, vec!["x".to_string(); 36]).expect("6x6");
        assert!(m6.determinant().is_err());
        let m4 = SymbolicMatrix::new(4, 4, vec!["x".to_string(); 16]).expect("4x4");
        assert!(m4.inverse().is_err());
    }

    #[test]
    fn transpuesta_y_display() {
        let a = m(&[&["x", "1"], &["0", "x"]]);
        let t = a.transpose().expect("transpuesta");
        assert_eq!(t.get(0, 1), "0");
        assert_eq!(t.get(1, 0), "1");
        assert_eq!(a.to_display_string(), "{{x, 1}, {0, x}}");
    }
}
