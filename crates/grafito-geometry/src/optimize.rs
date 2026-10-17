//! Optimización 1D/multivariada y EDO numérica: `Minimize`/`Maximize`,
//! BFGS multivariado y `NSolveODE`.
//!
//! Todo acotado y determinista: la grilla 1D es fija, el refinamiento es
//! sección áurea, el BFGS usa diferencias centrales + backtracking de Armijo
//! con techos de iteraciones/evaluaciones, y el RK45 es Dormand–Prince con
//! paso adaptativo y techo de evaluaciones. El mínimo reportado es local
//! (global no garantizado: se declara en el mensaje, no se inventa).
//!
//! El BFGS evalúa sobre la ÚNICA representación de expresión (`Expr`/
//! bytecode) vía `crate::expr::evaluate_cached` (sin tipos nuevos).

use crate::ast::{parse_ast, Expr};
use std::fmt;

/// Puntos de la grilla gruesa de minimización.
pub const MAX_MINIMIZE_GRID: usize = 4096;
/// Iteraciones de sección áurea por candidato.
pub const MAX_GOLDEN_ITERS: usize = 64;
/// Puntos máximos de una tabla `NSolveODE`.
pub const MAX_ODE_POINTS: usize = 1000;
/// Evaluaciones máximas del integrador RK45.
pub const MAX_ODE_EVALS: usize = 100_000;

/// Mínimo/máximo local refinado.
#[derive(Debug, Clone, PartialEq)]
pub struct Extremum {
    /// Abscisa del extremo.
    pub x: f64,
    /// Valor de la función.
    pub value: f64,
}

/// Evalúa `expr` (una variable) en `at`; `None` si no evalúa finito.
fn eval_1d(expr: &Expr, var: &str, at: f64) -> Option<f64> {
    let value = expr.eval_at(var, at);
    value.is_finite().then_some(value)
}

/// Extremo global-en-grilla + refinamiento áureo de `expr` en `[lo, hi]`.
///
/// `find_max` invierte el sentido. Devuelve el mejor candidato con su
/// valor; si nada evalúa finito, error honesto.
pub fn minimize_1d(
    expr: &str,
    var: &str,
    lo: f64,
    hi: f64,
    find_max: bool,
) -> Result<Extremum, String> {
    if !lo.is_finite() || !hi.is_finite() || lo >= hi {
        return Err("el intervalo debe ser finito con a < b".to_string());
    }
    let ast = parse_ast(&expr.replace(' ', ""))
        .map_err(|e| format!("no se pudo parsear '{expr}': {e}"))?;
    let better = |a: f64, b: f64| {
        if find_max {
            a > b
        } else {
            a < b
        }
    };
    // Grilla gruesa determinista.
    let mut best: Option<(f64, f64)> = None;
    for i in 0..=MAX_MINIMIZE_GRID {
        let x = lo + (hi - lo) * (i as f64) / (MAX_MINIMIZE_GRID as f64);
        if let Some(v) = eval_1d(&ast, var, x) {
            match best {
                Some((_, bv)) if !better(v, bv) => {}
                _ => best = Some((x, v)),
            }
        }
    }
    let (mut bx, mut bv) = best.ok_or_else(|| "f no evalúa finito en el intervalo".to_string())?;
    // Refinamiento áureo alrededor del mejor punto de grilla.
    let step = (hi - lo) / (MAX_MINIMIZE_GRID as f64);
    let (mut a, mut b) = ((bx - step).max(lo), (bx + step).min(hi));
    const INV_PHI: f64 = 0.618_033_988_749_894_9;
    for _ in 0..MAX_GOLDEN_ITERS {
        if (b - a).abs() < 1e-13 * (1.0 + a.abs().max(b.abs())) {
            break;
        }
        let c = b - INV_PHI * (b - a);
        let d = a + INV_PHI * (b - a);
        match (eval_1d(&ast, var, c), eval_1d(&ast, var, d)) {
            (Some(fc), Some(fd)) => {
                let take_left = if find_max { fc > fd } else { fc < fd };
                if take_left {
                    b = d;
                } else {
                    a = c;
                }
            }
            _ => break,
        }
    }
    for x in [a, b, (a + b) * 0.5] {
        if let Some(v) = eval_1d(&ast, var, x) {
            if better(v, bv) {
                bx = x;
                bv = v;
            }
        }
    }
    Ok(Extremum { x: bx, value: bv })
}

/// Dimensión máxima del BFGS multivariado (Hessiana inversa n×n ≤ 8×8).
pub const BFGS_MAX_DIM: usize = 8;
/// Iteraciones máximas del BFGS.
pub const BFGS_MAX_ITERS: usize = 200;
/// Evaluaciones máximas de `f` (incluye las 2n de cada gradiente).
pub const BFGS_MAX_EVALS: usize = 20_000;
/// Paso base de diferencias centrales: `h_i = BFGS_GRAD_STEP * (1 + |x_i|)`.
pub const BFGS_GRAD_STEP: f64 = 1e-8;

/// Mínimo local multivariado por BFGS.
#[derive(Debug, Clone, PartialEq)]
pub struct BfgsResult {
    /// Punto de mínimo (misma longitud que `vars`).
    pub x: Vec<f64>,
    /// Valor `f(x)` finito en el mínimo.
    pub value: f64,
    /// Iteraciones BFGS ejecutadas.
    pub iters: usize,
    /// Evaluaciones de `f` consumidas (función + gradientes + búsqueda).
    pub evals: usize,
    /// Norma L2 del gradiente en el mínimo (≤ tolerancia si convergió).
    pub grad_norm: f64,
}

/// Error honesto del BFGS (sin pánicos: todo borde devuelve `Err`).
#[derive(Debug, Clone, PartialEq)]
pub enum BfgsError {
    /// Sin variables que optimizar.
    EmptyVars,
    /// `vars.len() != x0.len()`.
    DimMismatch { vars: usize, x0: usize },
    /// Más variables que el presupuesto sin reservar nada.
    TooManyVars { got: usize, max: usize },
    /// Nombre de variable duplicado.
    DuplicateVar { name: String },
    /// Nombre de variable inválido (`[A-Za-z_][A-Za-z0-9_]*`).
    BadVarName { name: String },
    /// Tolerancia no finita o no positiva.
    BadTolerance { got: f64 },
    /// Punto inicial no finito (`index` orientativo).
    NonFiniteStart { index: usize },
    /// `f(x0)` no evalúa (parse, presupuesto o dominio); `detail` orientativo.
    BadFunction { detail: String },
    /// `f(x0)` evalúa pero no finito (polo en el arranque).
    NonFiniteAtStart,
    /// El gradiente central necesita `f(x±h)` finito en cada eje.
    NonFiniteGradient { index: usize },
    /// La búsqueda lineal agotó el backtracking sin paso de Armijo.
    LineSearchFailed,
    /// Techo de evaluaciones superado sin converger.
    EvalBudgetExceeded { evals: usize, max: usize },
    /// Techo de iteraciones sin alcanzar la tolerancia (p. ej. sin mínimo).
    NotConverged {
        iters: usize,
        grad_norm: f64,
        value: f64,
    },
}

impl fmt::Display for BfgsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyVars => write!(f, "BFGS necesita al menos una variable"),
            Self::DimMismatch { vars, x0 } => {
                write!(f, "BFGS: {vars} variables pero {x0} valores iniciales")
            }
            Self::TooManyVars { got, max } => {
                write!(f, "BFGS de {got} variables excede el máximo {max}")
            }
            Self::DuplicateVar { name } => {
                write!(f, "BFGS: variable duplicada '{name}'")
            }
            Self::BadVarName { name } => {
                write!(f, "BFGS: nombre de variable inválido '{name}'")
            }
            Self::BadTolerance { got } => {
                write!(f, "BFGS: tolerancia no válida ({got})")
            }
            Self::NonFiniteStart { index } => {
                write!(f, "BFGS: x0[{index}] no es finito")
            }
            Self::BadFunction { detail } => {
                write!(f, "BFGS: f(x0) no evalúa ({detail})")
            }
            Self::NonFiniteAtStart => {
                write!(f, "BFGS: f(x0) no es finito (polo en el arranque)")
            }
            Self::NonFiniteGradient { index } => {
                write!(f, "BFGS: f(x±h) no es finito en el eje {index}")
            }
            Self::LineSearchFailed => {
                write!(f, "BFGS: la búsqueda lineal no halló paso de Armijo")
            }
            Self::EvalBudgetExceeded { evals, max } => {
                write!(f, "BFGS: {evals} evaluaciones exceden el máximo {max}")
            }
            Self::NotConverged {
                iters,
                grad_norm,
                value,
            } => {
                write!(
                    f,
                    "BFGS: sin converger en {iters} iteraciones (grad {grad_norm:.3e}, f {value:.6e})"
                )
            }
        }
    }
}

impl std::error::Error for BfgsError {}

fn is_valid_bfgs_var(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Evalúa `expr` en el punto `x`; `None` si no evalúa finito.
fn eval_bfgs_point(expr: &str, vars: &[String], x: &[f64]) -> Option<f64> {
    let args: Vec<(String, f64)> = vars.iter().cloned().zip(x.iter().copied()).collect();
    crate::expr::evaluate_cached(expr, &args)
        .ok()
        .filter(|v| v.is_finite())
}

/// Gradiente por diferencias centrales con presupuesto de evaluaciones.
fn bfgs_gradient(
    expr: &str,
    vars: &[String],
    x: &[f64],
    evals: &mut usize,
) -> Result<Vec<f64>, BfgsError> {
    let n = x.len();
    let mut grad = vec![0.0; n];
    for i in 0..n {
        let h = BFGS_GRAD_STEP * (1.0 + x[i].abs());
        let mut xp = x.to_vec();
        let mut xm = x.to_vec();
        xp[i] += h;
        xm[i] -= h;
        if !xp[i].is_finite() || !xm[i].is_finite() {
            return Err(BfgsError::NonFiniteGradient { index: i });
        }
        *evals += 2;
        if *evals > BFGS_MAX_EVALS {
            return Err(BfgsError::EvalBudgetExceeded {
                evals: *evals,
                max: BFGS_MAX_EVALS,
            });
        }
        match (
            eval_bfgs_point(expr, vars, &xp),
            eval_bfgs_point(expr, vars, &xm),
        ) {
            (Some(fp), Some(fm)) => grad[i] = (fp - fm) / (2.0 * h),
            _ => return Err(BfgsError::NonFiniteGradient { index: i }),
        }
        if !grad[i].is_finite() {
            return Err(BfgsError::NonFiniteGradient { index: i });
        }
    }
    Ok(grad)
}

fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(u, v)| u * v).sum()
}

/// Mínimo local multivariado por BFGS con búsqueda lineal de Armijo.
///
/// `expr` se evalúa vía `evaluate_cached` (parse-once + LRU 128); el
/// gradiente es por diferencias centrales. Determinista (sin azar):
/// `H` arranca en identidad y el backtracking (`c1 = 1e-4`, `rho = 0.5`)
/// es fijo. Todo borde devuelve `Err` honesto; el mínimo es local
/// (global no garantizado).
pub fn bfgs_minimize(
    expr: &str,
    vars: &[&str],
    x0: &[f64],
    tol: f64,
) -> Result<BfgsResult, BfgsError> {
    if vars.is_empty() {
        return Err(BfgsError::EmptyVars);
    }
    if vars.len() != x0.len() {
        return Err(BfgsError::DimMismatch {
            vars: vars.len(),
            x0: x0.len(),
        });
    }
    if vars.len() > BFGS_MAX_DIM {
        return Err(BfgsError::TooManyVars {
            got: vars.len(),
            max: BFGS_MAX_DIM,
        });
    }
    if !tol.is_finite() || tol <= 0.0 {
        return Err(BfgsError::BadTolerance { got: tol });
    }
    let owned: Vec<String> = vars.iter().map(|v| (*v).to_string()).collect();
    for name in &owned {
        if !is_valid_bfgs_var(name) {
            return Err(BfgsError::BadVarName { name: name.clone() });
        }
    }
    let mut seen = std::collections::BTreeSet::new();
    for name in &owned {
        if !seen.insert(name.clone()) {
            return Err(BfgsError::DuplicateVar { name: name.clone() });
        }
    }
    for (i, v) in x0.iter().enumerate() {
        if !v.is_finite() {
            return Err(BfgsError::NonFiniteStart { index: i });
        }
    }
    let n = owned.len();
    let mut evals = 1usize;
    let f0 = match crate::expr::evaluate_cached(
        expr,
        &owned
            .iter()
            .cloned()
            .zip(x0.iter().copied())
            .collect::<Vec<(String, f64)>>(),
    ) {
        Err(detail) => return Err(BfgsError::BadFunction { detail }),
        Ok(v) if !v.is_finite() => return Err(BfgsError::NonFiniteAtStart),
        Ok(v) => v,
    };
    let mut x = x0.to_vec();
    let mut fx = f0;
    let mut grad = bfgs_gradient(expr, &owned, &x, &mut evals)?;
    // Hessiana inversa aproximada (identidad al arranque).
    let mut inv_h: Vec<Vec<f64>> = (0..n)
        .map(|i| (0..n).map(|j| if i == j { 1.0 } else { 0.0 }).collect())
        .collect();
    for it in 0..BFGS_MAX_ITERS {
        let grad_norm = dot(&grad, &grad).sqrt();
        if !grad_norm.is_finite() {
            return Err(BfgsError::NonFiniteGradient { index: 0 });
        }
        if grad_norm < tol {
            return Ok(BfgsResult {
                x,
                value: fx,
                iters: it,
                evals,
                grad_norm,
            });
        }
        // Dirección de descenso (reinicio a -g si H perdió definición).
        let mut direction: Vec<f64> = (0..n)
            .map(|i| {
                -inv_h[i]
                    .iter()
                    .zip(grad.iter())
                    .map(|(h, g)| h * g)
                    .sum::<f64>()
            })
            .collect();
        if !direction.iter().all(|v| v.is_finite()) || dot(&grad, &direction) >= 0.0 {
            for (i, row) in inv_h.iter_mut().enumerate() {
                for (j, cell) in row.iter_mut().enumerate() {
                    *cell = if i == j { 1.0 } else { 0.0 };
                }
            }
            direction = grad.iter().map(|g| -g).collect();
        }
        // Backtracking de Armijo.
        let directional = dot(&grad, &direction);
        let mut alpha = 1.0;
        let mut accepted: Option<(Vec<f64>, f64)> = None;
        for _ in 0..50 {
            let trial: Vec<f64> = x
                .iter()
                .zip(direction.iter())
                .map(|(xi, pi)| xi + alpha * pi)
                .collect();
            if !trial.iter().all(|v| v.is_finite()) {
                alpha *= 0.5;
                continue;
            }
            evals += 1;
            if evals > BFGS_MAX_EVALS {
                return Err(BfgsError::EvalBudgetExceeded {
                    evals,
                    max: BFGS_MAX_EVALS,
                });
            }
            if let Some(ft) = eval_bfgs_point(expr, &owned, &trial) {
                if ft <= fx + 1e-4 * alpha * directional {
                    accepted = Some((trial, ft));
                    break;
                }
            }
            alpha *= 0.5;
            if alpha < 1e-15 {
                break;
            }
        }
        let (x_new, f_new) = match accepted {
            Some(pair) => pair,
            None => return Err(BfgsError::LineSearchFailed),
        };
        let grad_new = bfgs_gradient(expr, &owned, &x_new, &mut evals)?;
        let s: Vec<f64> = x_new.iter().zip(x.iter()).map(|(a, b)| a - b).collect();
        let y: Vec<f64> = grad_new
            .iter()
            .zip(grad.iter())
            .map(|(a, b)| a - b)
            .collect();
        let ys = dot(&y, &s);
        // Actualización BFGS solo con curvatura sana (preserva definición).
        if ys > 1e-12 {
            let rho = 1.0 / ys;
            // A = I - rho*s*y^T; H' = A*H*A^T + rho*s*s^T (n ≤ 8).
            let mut a_mat = vec![vec![0.0; n]; n];
            for i in 0..n {
                for j in 0..n {
                    let eye = if i == j { 1.0 } else { 0.0 };
                    a_mat[i][j] = eye - rho * s[i] * y[j];
                }
            }
            let mut hah: Vec<Vec<f64>> = vec![vec![0.0; n]; n];
            for i in 0..n {
                for j in 0..n {
                    let mut acc = 0.0;
                    for k in 0..n {
                        for l in 0..n {
                            acc += a_mat[i][k] * inv_h[k][l] * a_mat[j][l];
                        }
                    }
                    hah[i][j] = acc;
                }
            }
            for i in 0..n {
                for j in 0..n {
                    inv_h[i][j] = hah[i][j] + rho * s[i] * s[j];
                }
            }
            if !inv_h.iter().flatten().all(|v| v.is_finite()) {
                for (i, row) in inv_h.iter_mut().enumerate() {
                    for (j, cell) in row.iter_mut().enumerate() {
                        *cell = if i == j { 1.0 } else { 0.0 };
                    }
                }
            }
        }
        x = x_new;
        fx = f_new;
        grad = grad_new;
    }
    let grad_norm = dot(&grad, &grad).sqrt();
    Err(BfgsError::NotConverged {
        iters: BFGS_MAX_ITERS,
        grad_norm,
        value: fx,
    })
}

/// Tabla `(x, y)` de `y' = f(x, y)`, `y(x0) = y0` hasta `x1` (RK45).
///
/// `n` puntos de salida (2..=`MAX_ODE_POINTS`). Paso adaptativo con
/// tolerancia 1e-9 y techo `MAX_ODE_EVALS`; más allá, error honesto.
pub fn nsolve_ode_table(
    expr: &str,
    x0: f64,
    y0: f64,
    x1: f64,
    n: usize,
) -> Result<Vec<(f64, f64)>, String> {
    if ![x0, y0, x1].iter().all(|v| v.is_finite()) {
        return Err("condiciones inicial y final deben ser finitas".to_string());
    }
    if x1 == x0 {
        return Err("x1 debe diferir de x0".to_string());
    }
    if !(2..=MAX_ODE_POINTS).contains(&n) {
        return Err(format!("n debe estar entre 2 y {MAX_ODE_POINTS}"));
    }
    let ast = parse_ast(&expr.replace(' ', ""))
        .map_err(|e| format!("no se pudo parsear '{expr}': {e}"))?;
    let field = |x: f64, y: f64| -> Option<f64> {
        // Sustituye x e y por constantes y evalúa.
        let sx = crate::ast::Expr::Const(x);
        let sy = crate::ast::Expr::Const(y);
        let mut with_x = substitute_var(&ast, "x", &sx);
        with_x = substitute_var(&with_x, "y", &sy);
        let value = eval_closed(&with_x)?;
        value.is_finite().then_some(value)
    };
    // Dormand–Prince 5(4) con paso adaptativo.
    let direction = (x1 - x0).signum();
    let mut h = (x1 - x0).abs() / (n as f64 * 4.0);
    h = h.clamp(1e-12, (x1 - x0).abs());
    let (mut x, mut y) = (x0, y0);
    let mut evals = 0usize;
    let mut table = vec![(x0, y0)];
    let targets: Vec<f64> = (1..n)
        .map(|i| x0 + (x1 - x0) * (i as f64) / ((n - 1) as f64))
        .collect();
    for &target in &targets {
        let mut guard = 0usize;
        while (target - x) * direction > 0.0 {
            guard += 1;
            if guard > 10_000 {
                return Err("RK45 no avanza: campo rígido o singular".to_string());
            }
            if evals > MAX_ODE_EVALS {
                return Err(format!(
                    "RK45 excede {MAX_ODE_EVALS} evaluaciones; achica el intervalo"
                ));
            }
            let step = h.min((target - x).abs());
            let (y5, y4) = dp45_step(&field, x, y, step * direction)?;
            evals += 6;
            let err = (y5 - y4).abs();
            let tol = 1e-9 * (1.0 + y.abs().max(y5.abs()));
            if err <= tol {
                x += step * direction;
                y = y5;
            }
            h = adapt_step(h, err, tol, (x1 - x0).abs());
            if !y.is_finite() {
                return Err(format!("la solución diverge cerca de x={x}"));
            }
        }
        table.push((target, y));
    }
    Ok(table)
}

fn adapt_step(h: f64, err: f64, tol: f64, span: f64) -> f64 {
    if err == 0.0 {
        return (h * 2.0).min(span);
    }
    let factor = 0.9 * (tol / err).powf(0.2);
    (h * factor.clamp(0.2, 2.0)).clamp(1e-12, span)
}

/// Un paso Dormand–Prince: devuelve `(orden5, orden4)`.
fn dp45_step(
    field: &impl Fn(f64, f64) -> Option<f64>,
    x: f64,
    y: f64,
    h: f64,
) -> Result<(f64, f64), String> {
    let f = |x: f64, y: f64| field(x, y).ok_or_else(|| "campo no evaluable".to_string());
    let k1 = f(x, y)?;
    let k2 = f(x + h / 5.0, y + h * (k1 / 5.0))?;
    let k3 = f(
        x + h * 3.0 / 10.0,
        y + h * (3.0 * k1 / 40.0 + 9.0 * k2 / 40.0),
    )?;
    let k4 = f(
        x + h * 4.0 / 5.0,
        y + h * (44.0 * k1 / 45.0 - 56.0 * k2 / 15.0 + 32.0 * k3 / 9.0),
    )?;
    let k5 = f(
        x + h * 8.0 / 9.0,
        y + h
            * (19372.0 * k1 / 6561.0 - 25360.0 * k2 / 2187.0 + 64448.0 * k3 / 6561.0
                - 212.0 * k4 / 729.0),
    )?;
    let k6 = f(
        x + h,
        y + h
            * (9017.0 * k1 / 3168.0 - 355.0 * k2 / 33.0
                + 46732.0 * k3 / 5247.0
                + 49.0 * k4 / 176.0
                - 5103.0 * k5 / 18656.0),
    )?;
    let y5 = y + h
        * (35.0 * k1 / 384.0 + 500.0 * k3 / 1113.0 + 125.0 * k4 / 192.0 - 2187.0 * k5 / 6784.0
            + 11.0 * k6 / 84.0);
    let k7 = f(x + h, y5)?;
    let y4 = y + h
        * (5179.0 * k1 / 57600.0 + 7571.0 * k3 / 16695.0 + 393.0 * k4 / 640.0
            - 92097.0 * k5 / 339200.0
            + 187.0 * k6 / 2100.0
            + k7 / 40.0);
    Ok((y5, y4))
}

/// Sustituye `var` por `replacement` (clonando; acotado por el AST dado).
fn substitute_var(expr: &Expr, var: &str, replacement: &Expr) -> Expr {
    use Expr::*;
    match expr {
        Const(_) => expr.clone(),
        Var(v) if v == var => replacement.clone(),
        Var(_) => expr.clone(),
        Neg(u) => Neg(Box::new(substitute_var(u, var, replacement))),
        Add(a, b) => Add(
            Box::new(substitute_var(a, var, replacement)),
            Box::new(substitute_var(b, var, replacement)),
        ),
        Sub(a, b) => Sub(
            Box::new(substitute_var(a, var, replacement)),
            Box::new(substitute_var(b, var, replacement)),
        ),
        Mul(a, b) => Mul(
            Box::new(substitute_var(a, var, replacement)),
            Box::new(substitute_var(b, var, replacement)),
        ),
        Div(a, b) => Div(
            Box::new(substitute_var(a, var, replacement)),
            Box::new(substitute_var(b, var, replacement)),
        ),
        Pow(a, b) => Pow(
            Box::new(substitute_var(a, var, replacement)),
            Box::new(substitute_var(b, var, replacement)),
        ),
        Sin(u) => Sin(sub1(u, var, replacement)),
        Cos(u) => Cos(sub1(u, var, replacement)),
        Tan(u) => Tan(sub1(u, var, replacement)),
        Asin(u) => Asin(sub1(u, var, replacement)),
        Acos(u) => Acos(sub1(u, var, replacement)),
        Atan(u) => Atan(sub1(u, var, replacement)),
        Exp(u) => Exp(sub1(u, var, replacement)),
        Ln(u) => Ln(sub1(u, var, replacement)),
        Log(u) => Log(sub1(u, var, replacement)),
        Sqrt(u) => Sqrt(sub1(u, var, replacement)),
        Abs(u) => Abs(sub1(u, var, replacement)),
        Sinh(u) => Sinh(sub1(u, var, replacement)),
        Cosh(u) => Cosh(sub1(u, var, replacement)),
        Tanh(u) => Tanh(sub1(u, var, replacement)),
        Floor(u) => Floor(sub1(u, var, replacement)),
        Ceil(u) => Ceil(sub1(u, var, replacement)),
        Round(u) => Round(sub1(u, var, replacement)),
        Sec(u) => Sec(sub1(u, var, replacement)),
        Csc(u) => Csc(sub1(u, var, replacement)),
        Cot(u) => Cot(sub1(u, var, replacement)),
        Asinh(u) => Asinh(sub1(u, var, replacement)),
        Acosh(u) => Acosh(sub1(u, var, replacement)),
        Atanh(u) => Atanh(sub1(u, var, replacement)),
        Sign(u) => Sign(sub1(u, var, replacement)),
        Heaviside(u) => Heaviside(sub1(u, var, replacement)),
        Cbrt(u) => Cbrt(sub1(u, var, replacement)),
        Re(u) => Re(sub1(u, var, replacement)),
        Im(u) => Im(sub1(u, var, replacement)),
        Arg(u) => Arg(sub1(u, var, replacement)),
        Conj(u) => Conj(sub1(u, var, replacement)),
        Erf(u) => Erf(sub1(u, var, replacement)),
        Erfc(u) => Erfc(sub1(u, var, replacement)),
        Gamma(u) => Gamma(sub1(u, var, replacement)),
        LnGamma(u) => LnGamma(sub1(u, var, replacement)),
        Digamma(u) => Digamma(sub1(u, var, replacement)),
        Trigamma(u) => Trigamma(sub1(u, var, replacement)),
        _ => expr.clone(),
    }
}

fn sub1(u: &Expr, var: &str, replacement: &Expr) -> Box<Expr> {
    Box::new(substitute_var(u, var, replacement))
}

/// Evalúa un AST cerrado (sin variables libres conocidas).
fn eval_closed(expr: &Expr) -> Option<f64> {
    use Expr::*;
    let num = |u: &Expr| eval_closed(u);
    match expr {
        Const(v) => Some(*v),
        Var(_) => None,
        Neg(u) => num(u).map(|v| -v),
        Add(a, b) => Some(num(a)? + num(b)?),
        Sub(a, b) => Some(num(a)? - num(b)?),
        Mul(a, b) => Some(num(a)? * num(b)?),
        Div(a, b) => {
            let d = num(b)?;
            if d == 0.0 {
                None
            } else {
                Some(num(a)? / d)
            }
        }
        Pow(a, b) => Some(num(a)?.powf(num(b)?)),
        Sin(u) => num(u).map(f64::sin),
        Cos(u) => num(u).map(f64::cos),
        Tan(u) => num(u).map(f64::tan),
        Exp(u) => num(u).map(f64::exp),
        Ln(u) => num(u).filter(|v| *v > 0.0).map(f64::ln),
        Sqrt(u) => num(u).filter(|v| *v >= 0.0).map(f64::sqrt),
        Abs(u) => num(u).map(f64::abs),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimize_parabola() {
        let e = minimize_1d("x^2-4*x+1", "x", -10.0, 10.0, false).expect("mínimo");
        assert!((e.x - 2.0).abs() < 1e-6, "got {e:?}");
        assert!((e.value + 3.0).abs() < 1e-6, "got {e:?}");
    }

    #[test]
    fn maximize_sine() {
        let e = minimize_1d("sin(x)", "x", 0.0, std::f64::consts::TAU, true).expect("máximo");
        assert!((e.value - 1.0).abs() < 1e-4, "got {e:?}");
    }

    #[test]
    fn minimize_needs_finite_interval() {
        assert!(minimize_1d("x", "x", 5.0, 5.0, false).is_err());
    }

    #[test]
    fn nsolve_ode_exponential() {
        // y' = y, y(0) = 1 → e^1 ≈ 2.718 en x=1.
        let table = nsolve_ode_table("y", 0.0, 1.0, 1.0, 11).expect("exponencial");
        assert_eq!(table.len(), 11);
        let last = table.last().unwrap();
        assert!((last.1 - std::f64::consts::E).abs() < 1e-6, "got {last:?}");
    }

    #[test]
    fn nsolve_ode_harmonic_half_period() {
        // Sistema decaído a 1er orden vía campo: y' = -y → e^-1.
        let table = nsolve_ode_table("-y", 0.0, 1.0, 1.0, 11).expect("decaimiento");
        let last = table.last().unwrap();
        assert!(
            (last.1 - 1.0 / std::f64::consts::E).abs() < 1e-6,
            "got {last:?}"
        );
    }

    #[test]
    fn nsolve_ode_rejects_bad_input() {
        assert!(nsolve_ode_table("y", 0.0, 1.0, 0.0, 11).is_err());
        assert!(nsolve_ode_table("y", 0.0, 1.0, 1.0, 1).is_err());
    }

    #[test]
    fn bfgs_resuelve_cuadratica_2d() {
        let r = bfgs_minimize("(x-1)^2+(y+2)^2", &["x", "y"], &[0.0, 0.0], 1e-8)
            .expect("cuadrática válida");
        assert!((r.x[0] - 1.0).abs() < 1e-6, "got {r:?}");
        assert!((r.x[1] + 2.0).abs() < 1e-6, "got {r:?}");
        assert!(r.value.abs() < 1e-12, "got {r:?}");
        assert!(r.grad_norm < 1e-8, "got {r:?}");
    }

    #[test]
    fn bfgs_rosenbrock_converge_cerca_de_1_1() {
        let r = bfgs_minimize("(1-x)^2+100*(y-x^2)^2", &["x", "y"], &[-1.2, 1.0], 1e-5)
            .expect("rosenbrock válido");
        assert!((r.x[0] - 1.0).abs() < 1e-3, "got {r:?}");
        assert!((r.x[1] - 1.0).abs() < 1e-3, "got {r:?}");
        assert!(r.value < 1e-6, "got {r:?}");
    }

    #[test]
    fn bfgs_unidimensional_da_paridad_con_minimize_1d() {
        let r = bfgs_minimize("x^2-4*x+1", &["x"], &[5.0], 1e-9).expect("1D válido");
        assert!((r.x[0] - 2.0).abs() < 1e-6, "got {r:?}");
        assert!((r.value + 3.0).abs() < 1e-6, "got {r:?}");
    }

    #[test]
    fn bfgs_rechaza_entradas_malformadas() {
        assert_eq!(
            bfgs_minimize("x", &[], &[], 1e-6).unwrap_err(),
            BfgsError::EmptyVars
        );
        assert_eq!(
            bfgs_minimize("x+y", &["x", "y"], &[0.0], 1e-6).unwrap_err(),
            BfgsError::DimMismatch { vars: 2, x0: 1 }
        );
        let many = ["a", "b", "c", "d", "e", "f", "g", "h", "i"];
        assert_eq!(
            bfgs_minimize("a", &many, &[0.0; 9], 1e-6).unwrap_err(),
            BfgsError::TooManyVars {
                got: 9,
                max: BFGS_MAX_DIM
            }
        );
        assert_eq!(
            bfgs_minimize("x+x", &["x", "x"], &[0.0, 0.0], 1e-6).unwrap_err(),
            BfgsError::DuplicateVar {
                name: "x".to_string()
            }
        );
        assert_eq!(
            bfgs_minimize("x", &["1x"], &[0.0], 1e-6).unwrap_err(),
            BfgsError::BadVarName {
                name: "1x".to_string()
            }
        );
        assert!(matches!(
            bfgs_minimize("x", &["x"], &[0.0], f64::NAN).unwrap_err(),
            BfgsError::BadTolerance { .. }
        ));
        assert_eq!(
            bfgs_minimize("x", &["x"], &[0.0], 0.0).unwrap_err(),
            BfgsError::BadTolerance { got: 0.0 }
        );
    }

    #[test]
    fn bfgs_error_honesto_en_arranque_no_finito_o_parse() {
        assert_eq!(
            bfgs_minimize("x", &["x"], &[f64::INFINITY], 1e-6).unwrap_err(),
            BfgsError::NonFiniteStart { index: 0 }
        );
        // Polo en el arranque: 1/(x*y) en (0,0) no evalúa finito (el motor
        // responde Err honesto, no un valor; ambas ramas son error honesto).
        assert!(matches!(
            bfgs_minimize("1/(x*y)", &["x", "y"], &[0.0, 0.0], 1e-6).unwrap_err(),
            BfgsError::BadFunction { .. } | BfgsError::NonFiniteAtStart
        ));
        // Parse imposible: el motor no inventa semántica para `x+*`.
        assert!(matches!(
            bfgs_minimize("x+*", &["x"], &[0.0], 1e-6).unwrap_err(),
            BfgsError::BadFunction { .. }
        ));
        // Nota: una variable libre ajena a `vars` (p. ej. `y+1` con
        // `vars=["x"]`) NO da error hoy — el evaluador la resuelve a 0
        // (conducta de `expr::evaluate`, dueño ajeno; no se pinea acá).
    }

    #[test]
    fn bfgs_gradiente_no_finito_en_borde_de_dominio() {
        // f(0) = 0 finito, pero f(-h) sale del dominio de sqrt.
        assert_eq!(
            bfgs_minimize("sqrt(x)", &["x"], &[0.0], 1e-6).unwrap_err(),
            BfgsError::NonFiniteGradient { index: 0 }
        );
    }

    #[test]
    fn bfgs_no_converge_sin_minimo_lineal() {
        // f(x) = x baja sin cota: el gradiente jamás se anula y el
        // backtracking acepta siempre (cada iter da un paso de Armijo).
        // (Nota: exp(x)+exp(y) NO sirve para este pin: en x≈-23.8 el
        // gradiente numérico ya cae bajo 1e-10 y el BFGS converge de
        // mentira por planitud asintótica en f64 — verificado en rojo.)
        let err = bfgs_minimize("x", &["x"], &[0.0], 1e-10).unwrap_err();
        assert!(
            matches!(err, BfgsError::NotConverged { iters: 200, .. }),
            "got {err:?}"
        );
    }
}
