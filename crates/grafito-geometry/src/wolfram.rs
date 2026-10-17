//! Motor Wolfram InputForm (subset honesto, bidireccional).
//!
//! Traduce entre la expresión canónica de Grafito (la que acepta
//! [`crate::ast::parse_ast`], p. ej. `sin(x) + x^2`) y el subset útil del
//! InputForm de Wolfram Language (el de `Sin[x]`, `Pi`, `Sum[x, {n, 1, 10}]`).
//! Lo implementan los comandos `ParseWolfram` y `ToWolfram`
//! (`grafito-command`): este módulo es el motor, `commands.rs` solo lo cablea.
//!
//! ## API
//!
//! ```text
//! to_wolfram("x^2 + sin(x)")  ->  "x^2 + Sin[x]"
//! to_wolfram(&ast)            ->  String  (acepta &str, String y &Expr)
//! parse_wolfram("Sin[x]^2")   ->  Ok("sin(x)^(2)")
//! parse_wolfram("Integrate[x]") -> Err("... fuera del subset ...")
//! ```
//!
//! ## Subset soportado
//!
//! - Aritmética: `+ - * / ^` con precedencia habitual y `^` asociativo a
//!   derecha; paréntesis `(…)`; yuxtaposición (`2x`, `2 x`, `(a)(b)`).
//! - Constantes: `Pi`, `E` (también `pi`, `e` minúsculas por leniencia).
//! - Llamadas `Head[args]`: `Sin Cos Tan ArcSin ArcCos ArcTan Exp Sqrt Abs`
//!   `Sinh Cosh Tanh Floor Ceiling Round Sec Csc Cot ArcSinh ArcCosh ArcTanh`
//!   `Sign HeavisideTheta CubeRoot Re Im Arg Conjugate Erf Erfc Gamma`
//!   `LogGamma Min Max Mod Clip Beta BesselJ BesselY BesselI Rational Power`
//!   `Plus Times Divide ArcTan[x, y] Sum Product Piecewise`.
//! - `Log[x]` → `ln(x)` (natural); `Log[10, x]` → `log(x)` (base 10, la única
//!   otra base del motor); cualquier otra base da error honesto.
//! - `ArcTan[x, y]` → `atan2(y, x)` (orden canónico: primero `y`).
//! - `Clip[x, {lo, hi}]` → `clamp(x, lo, hi)`.
//! - `PolyGamma[x]` → `digamma(x)`; `PolyGamma[0, x]` → `digamma(x)`;
//!   `PolyGamma[1, x]` → `trigamma(x)`; otro orden da error honesto.
//! - `Sum[cuerpo, {var, lo, hi}]` → `sum(cuerpo, var, lo, hi)` (igual
//!   `Product`); `Piecewise[{{v1, c1}, …}, d]` → `piecewise(c1, v1, …, d)`
//!   (Wolfram ordena `{valor, condición}`, el canónico `(cond, val)`).
//! - Comparaciones: `== != < > <= >=` (un `=` solo se lee como `==`).
//!
//! Fuera de esto el parser NO adivina: devuelve un error en rioplatense que
//! nombra al culpable (`Integrate`, `I`, `Infinity`, `_`, `{…}` peladas...).
//!
//! ## Decisiones que conviene conocer
//!
//! - `to_wolfram` recibe texto canónico o `&Expr` (trait [`ToWolframInput`]) y
//!   devuelve `String` sin `Result`: si el texto no parsea, devuelve el texto
//!   tal cual (documentado en la función).
//! - `pi`/`tau`/`e` se detectan por igualdad exacta de bits: un `3.14`
//!   aproximado NO se vuelve `Pi`; `tau` se emite como `(2*Pi)` (Wolfram no
//!   tiene `Tau` canónico).
//! - `Min`/`Max` son binarios en el motor: `Min[a, b, c]` da error honesto.
//! - `I` (imaginaria), `Infinity`, `Indeterminate`, `True`/`False` y matrices
//!   están fuera del subset: el motor es real y escalar.
//! - Una llamada con paréntesis (`sin(x)`) se acepta por leniencia solo si el
//!   nombre es una función conocida; si no, la yuxtaposición multiplica
//!   (`x(y+1)` = `x*(y+1)`).

use crate::ast::Expr;

/// Largo máximo aceptado (espeja `crate::expr::MAX_EXPR_LENGTH`).
const MAX_WOLFRAM_INPUT_BYTES: usize = crate::expr::MAX_EXPR_LENGTH;

/// Anidado máximo del descenso recursivo (espeja `MAX_EXPR_NESTING`).
const MAX_WOLFRAM_DEPTH: usize = crate::expr::MAX_EXPR_NESTING;

/// Entrada válida para [`to_wolfram`]: texto canónico o AST ya parseado.
///
/// Existe para que los dos cableados posibles compilen sin cambios:
/// `to_wolfram("x^2")` y `to_wolfram(&ast)` valen lo mismo.
pub trait ToWolframInput {
    /// Renderiza la entrada como Wolfram InputForm del subset soportado.
    fn render_wolfram(&self) -> String;
}

impl ToWolframInput for str {
    fn render_wolfram(&self) -> String {
        wolfram_from_canonical(self)
    }
}

impl ToWolframInput for String {
    fn render_wolfram(&self) -> String {
        wolfram_from_canonical(self.as_str())
    }
}

impl ToWolframInput for &str {
    fn render_wolfram(&self) -> String {
        wolfram_from_canonical(self)
    }
}

impl ToWolframInput for &String {
    fn render_wolfram(&self) -> String {
        wolfram_from_canonical(self.as_str())
    }
}

impl ToWolframInput for Expr {
    fn render_wolfram(&self) -> String {
        to_wolfram_expr(self)
    }
}

impl ToWolframInput for &Expr {
    fn render_wolfram(&self) -> String {
        to_wolfram_expr(self)
    }
}

/// Expresión canónica (o AST) a Wolfram InputForm.
///
/// Acepta `&str`, `String` y `&Expr` vía [`ToWolframInput`]. Si el texto no
/// parsea con [`crate::ast::parse_ast`] (o excede el presupuesto), devuelve el
/// texto recortado tal cual en vez de inventar: el comando que muestra el
/// resultado nunca miente sobre una traducción que no pudo hacer.
pub fn to_wolfram(input: impl ToWolframInput) -> String {
    input.render_wolfram()
}

/// Renderiza un AST como Wolfram InputForm del subset. Núcleo de [`to_wolfram`].
pub fn to_wolfram_expr(expr: &Expr) -> String {
    match expr {
        Expr::Const(c) => render_const(*c),
        Expr::Var(v) => v.clone(),
        Expr::Add(a, b) => format!("{} + {}", grouped(a, 1), grouped(b, 1)),
        Expr::Sub(a, b) => format!("{} - {}", grouped(a, 1), grouped(b, 2)),
        Expr::Mul(a, b) => format!("{}*{}", grouped(a, 2), grouped(b, 2)),
        Expr::Div(a, b) => format!("{}/{}", grouped(a, 2), grouped(b, 3)),
        Expr::Pow(base, exp) => {
            format!("{}^{}", grouped(base, 4), grouped(exp, 3))
        }
        Expr::Neg(u) => {
            let inner = to_wolfram_expr(u);
            if prec(u) < 2 {
                format!("-({inner})")
            } else {
                format!("-{inner}")
            }
        }
        Expr::Lt(a, b) => format!("{} < {}", grouped(a, 1), grouped(b, 1)),
        Expr::Gt(a, b) => format!("{} > {}", grouped(a, 1), grouped(b, 1)),
        Expr::Le(a, b) => format!("{} <= {}", grouped(a, 1), grouped(b, 1)),
        Expr::Ge(a, b) => format!("{} >= {}", grouped(a, 1), grouped(b, 1)),
        Expr::Eq(a, b) => format!("{} == {}", grouped(a, 1), grouped(b, 1)),
        Expr::Ne(a, b) => format!("{} != {}", grouped(a, 1), grouped(b, 1)),
        Expr::Sin(u) => call1("Sin", u),
        Expr::Cos(u) => call1("Cos", u),
        Expr::Tan(u) => call1("Tan", u),
        Expr::Asin(u) => call1("ArcSin", u),
        Expr::Acos(u) => call1("ArcCos", u),
        Expr::Atan(u) => call1("ArcTan", u),
        Expr::Exp(u) => call1("Exp", u),
        Expr::Ln(u) => call1("Log", u),
        Expr::Log(u) => format!("Log[10, {}]", to_wolfram_expr(u)),
        Expr::Sqrt(u) => call1("Sqrt", u),
        Expr::Abs(u) => call1("Abs", u),
        Expr::Sinh(u) => call1("Sinh", u),
        Expr::Cosh(u) => call1("Cosh", u),
        Expr::Tanh(u) => call1("Tanh", u),
        Expr::Floor(u) => call1("Floor", u),
        Expr::Ceil(u) => call1("Ceiling", u),
        Expr::Sec(u) => call1("Sec", u),
        Expr::Csc(u) => call1("Csc", u),
        Expr::Cot(u) => call1("Cot", u),
        Expr::Cbrt(u) => call1("CubeRoot", u),
        Expr::Min(a, b) => call2("Min", a, b),
        Expr::Max(a, b) => call2("Max", a, b),
        Expr::Asinh(u) => call1("ArcSinh", u),
        Expr::Acosh(u) => call1("ArcCosh", u),
        Expr::Atanh(u) => call1("ArcTanh", u),
        Expr::Sign(u) => call1("Sign", u),
        Expr::Heaviside(u) => call1("HeavisideTheta", u),
        Expr::Round(u) => call1("Round", u),
        Expr::Re(u) => call1("Re", u),
        Expr::Im(u) => call1("Im", u),
        Expr::Arg(u) => call1("Arg", u),
        Expr::Conj(u) => call1("Conjugate", u),
        Expr::Erf(u) => call1("Erf", u),
        Expr::Erfc(u) => call1("Erfc", u),
        Expr::Gamma(u) => call1("Gamma", u),
        Expr::LnGamma(u) => call1("LogGamma", u),
        Expr::Digamma(u) => call1("PolyGamma", u),
        Expr::Trigamma(u) => format!("PolyGamma[1, {}]", to_wolfram_expr(u)),
        // Wolfram `ArcTan[x, y]` = atan2(y, x): el canónico ordena (y, x).
        Expr::Atan2(y, x) => format!("ArcTan[{}, {}]", to_wolfram_expr(x), to_wolfram_expr(y)),
        Expr::Modulo(a, b) => call2("Mod", a, b),
        Expr::Beta(a, b) => call2("Beta", a, b),
        Expr::BesselJ(n, u) => call2("BesselJ", n, u),
        Expr::BesselY(n, u) => call2("BesselY", n, u),
        Expr::BesselI(n, u) => call2("BesselI", n, u),
        Expr::Clamp(x, lo, hi) => format!(
            "Clip[{}, {{{}, {}}}]",
            to_wolfram_expr(x),
            to_wolfram_expr(lo),
            to_wolfram_expr(hi)
        ),
        Expr::Sum(body, var, start, end) => format!(
            "Sum[{}, {{{var}, {}, {}}}]",
            to_wolfram_expr(body),
            to_wolfram_expr(start),
            to_wolfram_expr(end)
        ),
        Expr::Product(body, var, start, end) => format!(
            "Product[{}, {{{var}, {}, {}}}]",
            to_wolfram_expr(body),
            to_wolfram_expr(start),
            to_wolfram_expr(end)
        ),
        Expr::Piecewise(pieces, default) => {
            // Wolfram ordena {valor, condición}; el canónico (cond, val).
            let mut pairs = Vec::with_capacity(pieces.len());
            for (cond, val) in pieces {
                pairs.push(format!(
                    "{{{}, {}}}",
                    to_wolfram_expr(val),
                    to_wolfram_expr(cond)
                ));
            }
            format!(
                "Piecewise[{{{}}}, {}]",
                pairs.join(", "),
                to_wolfram_expr(default)
            )
        }
    }
}

/// Wolfram InputForm (subset soportado) a expresión canónica.
///
/// Devuelve el texto canónico listo para [`crate::ast::parse_ast`] (p. ej.
/// `Sin[x]^2` → `sin(x)^(2)`). El resultado se valida contra el parser
/// canónico antes de salir: si algo no cierra, el error lo dice en
/// rioplatense y nombra al culpable en vez de adivinar.
pub fn parse_wolfram(input: &str) -> Result<String, String> {
    let text = input.trim().to_string();
    if text.is_empty() {
        return Err("ParseWolfram: entrada vacía, pasame algo como `Sin[x]`".into());
    }
    if text.len() > MAX_WOLFRAM_INPUT_BYTES {
        return Err(format!(
            "ParseWolfram: entrada de {} bytes excede el máximo {MAX_WOLFRAM_INPUT_BYTES}",
            text.len()
        ));
    }
    let mut parser = Parser {
        chars: text.chars().collect(),
        pos: 0,
    };
    let out = parser.parse_cmp(0)?;
    parser.skip_spaces();
    if let Some(rest) = parser.peek() {
        return Err(parser.err(format!(
            "«{rest}» sobra al final: fijate que los paréntesis y corchetes cierren bien"
        )));
    }
    match crate::ast::parse_ast(&out) {
        Ok(_) => Ok(out),
        Err(detail) => Err(format!(
            "ParseWolfram: la traducción («{out}») no pasó el parser canónico ({detail}): avisá que hay un bug en el motor Wolfram"
        )),
    }
}

// ── Render helpers ───────────────────────────────────────────────────────────

/// Constantes exactas `pi`/`tau`/`e` por bits (el parser canónico las pliega a
/// número, así se recuperan); un `3.14` aproximado NO se vuelve `Pi`.
/// `tau` se emite como `(2*Pi)` (Wolfram no tiene `Tau` canónico).
fn render_const(value: f64) -> String {
    if value.to_bits() == std::f64::consts::PI.to_bits() {
        return "Pi".to_string();
    }
    if value.to_bits() == std::f64::consts::TAU.to_bits() {
        return "(2*Pi)".to_string();
    }
    if value.to_bits() == std::f64::consts::E.to_bits() {
        return "E".to_string();
    }
    if !value.is_finite() {
        // NaN/inf no tienen forma canónica finita: se emite un marcador que
        // `parse_wolfram` rechaza con error honesto al reingresar.
        return "Indeterminate".to_string();
    }
    if value.fract() == 0.0 && value.abs() < 1e15 {
        format!("{value:.0}")
    } else {
        format!("{value:?}")
    }
}

/// Precedencia espejo del parser canónico (ver `to_expr_string_paren`).
fn prec(expr: &Expr) -> u8 {
    match expr {
        Expr::Lt(..) | Expr::Gt(..) | Expr::Le(..) | Expr::Ge(..) | Expr::Eq(..) | Expr::Ne(..) => {
            0
        }
        Expr::Add(..) | Expr::Sub(..) => 1,
        Expr::Mul(..) | Expr::Div(..) => 2,
        Expr::Neg(..) => 3,
        Expr::Const(c) if c.is_sign_negative() => 3,
        Expr::Pow(..) => 4,
        _ => 10,
    }
}

/// Hijo con paréntesis si su precedencia no alcanza.
fn grouped(expr: &Expr, min_prec: u8) -> String {
    // La negación y las constantes negativas siempre se parenthesizan como
    // hijo: `-x*y` se releería como `-(x*y)` y rompería el roundtrip.
    if matches!(expr, Expr::Neg(..)) {
        return format!("({})", to_wolfram_expr(expr));
    }
    if let Expr::Const(c) = expr {
        if c.is_sign_negative() {
            return format!("({})", to_wolfram_expr(expr));
        }
    }
    let rendered = to_wolfram_expr(expr);
    if prec(expr) < min_prec {
        format!("({rendered})")
    } else {
        rendered
    }
}

fn call1(head: &str, arg: &Expr) -> String {
    format!("{head}[{}]", to_wolfram_expr(arg))
}

fn call2(head: &str, a: &Expr, b: &Expr) -> String {
    format!("{head}[{}, {}]", to_wolfram_expr(a), to_wolfram_expr(b))
}

/// Texto canónico → Wolfram (falla devolviendo el texto tal cual).
fn wolfram_from_canonical(expr: &str) -> String {
    let trimmed = expr.trim();
    if trimmed.is_empty() || trimmed.len() > MAX_WOLFRAM_INPUT_BYTES {
        return trimmed.to_string();
    }
    match crate::ast::parse_ast(trimmed) {
        Ok(ast) => to_wolfram_expr(&ast),
        Err(_) => trimmed.to_string(),
    }
}

// ── Parser ───────────────────────────────────────────────────────────────────

struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn skip_spaces(&mut self) {
        while self.peek().is_some_and(|c| c.is_whitespace()) {
            self.pos += 1;
        }
    }

    fn err(&self, msg: String) -> String {
        format!("ParseWolfram: {msg} (posición {})", self.pos)
    }

    /// ¿El input en `pos` empieza con `s`? (por chars, sin partir UTF-8).
    fn starts_with(&self, s: &str) -> bool {
        s.chars()
            .enumerate()
            .all(|(offset, c)| self.chars.get(self.pos + offset).copied() == Some(c))
    }

    fn consume_str(&mut self, s: &str) -> bool {
        if self.starts_with(s) {
            self.pos += s.chars().count();
            true
        } else {
            false
        }
    }

    /// Lee un identificador ASCII (`[A-Za-z_][A-Za-z0-9_]*`).
    fn read_ident(&mut self) -> String {
        let mut name = String::new();
        let mut first = true;
        while let Some(c) = self.peek() {
            if first {
                if c.is_ascii_alphabetic() || c == '_' {
                    name.push(c);
                    self.pos += 1;
                    first = false;
                } else {
                    break;
                }
            } else if c.is_ascii_alphanumeric() || c == '_' {
                name.push(c);
                self.pos += 1;
            } else {
                break;
            }
        }
        name
    }

    // — niveles de precedencia —

    fn parse_cmp(&mut self, depth: usize) -> Result<String, String> {
        if depth > MAX_WOLFRAM_DEPTH {
            return Err(self.err("demasiado anidado, simplificá la expresión".into()));
        }
        let mut out = self.parse_add_sub(depth)?;
        loop {
            self.skip_spaces();
            let op: Option<&str> = if self.consume_str("==") {
                Some("==")
            } else if self.consume_str("!=") {
                Some("!=")
            } else if self.consume_str("<=") {
                Some("<=")
            } else if self.consume_str(">=") {
                Some(">=")
            } else if self.consume_str("<") {
                Some("<")
            } else if self.consume_str(">") {
                Some(">")
            } else if self.consume_str("=") {
                // `=` solo se lee como `==` (comparación, no asignación).
                Some("==")
            } else {
                None
            };
            let Some(op) = op else { break };
            let rhs = self.parse_add_sub(depth)?;
            out = format!("{out} {op} {rhs}");
        }
        Ok(out)
    }

    fn parse_add_sub(&mut self, depth: usize) -> Result<String, String> {
        let mut out = self.parse_unary(depth)?;
        loop {
            self.skip_spaces();
            match self.peek() {
                Some('+') => {
                    self.pos += 1;
                    let rhs = self.parse_unary(depth)?;
                    out = format!("{out} + {rhs}");
                }
                Some('-') => {
                    self.pos += 1;
                    let rhs = self.parse_unary(depth)?;
                    out = format!("{out} - {rhs}");
                }
                _ => break,
            }
        }
        Ok(out)
    }

    fn parse_unary(&mut self, depth: usize) -> Result<String, String> {
        let mut neg = false;
        loop {
            self.skip_spaces();
            match self.peek() {
                Some('+') => self.pos += 1,
                Some('-') => {
                    self.pos += 1;
                    neg = !neg;
                }
                _ => break,
            }
        }
        let out = self.parse_mul_div(depth)?;
        if neg {
            Ok(format!("-({out})"))
        } else {
            Ok(out)
        }
    }

    fn parse_mul_div(&mut self, depth: usize) -> Result<String, String> {
        let mut out = self.parse_pow(depth)?;
        loop {
            self.skip_spaces();
            let op: Option<char> = if self.consume_str("*") {
                Some('*')
            } else if self.consume_str("/") {
                Some('/')
            } else if self.operand_starts() {
                // Yuxtaposición: `2x`, `2 x`, `(a)(b)`, `2Pi`.
                None
            } else {
                break;
            };
            match op {
                Some('*') | None => {
                    let rhs = self.parse_pow(depth)?;
                    out = format!("{out}*{rhs}");
                }
                Some('/') => {
                    let rhs = self.parse_pow(depth)?;
                    out = format!("{out}/{rhs}");
                }
                Some(_) => break,
            }
        }
        Ok(out)
    }

    /// `^` asociativo a derecha; el exponente acepta signo (`x^-2` vale).
    fn parse_pow(&mut self, depth: usize) -> Result<String, String> {
        let base = self.parse_atom(depth)?;
        self.skip_spaces();
        if self.peek() != Some('^') {
            return Ok(base);
        }
        self.pos += 1;
        let exp = self.parse_unary(depth)?;
        Ok(format!("{base}^({exp})"))
    }

    /// ¿Empieza un operando acá? (para la multiplicación implícita).
    fn operand_starts(&self) -> bool {
        match self.peek() {
            Some(c) if c.is_ascii_digit() => true,
            Some(c) if c.is_ascii_alphabetic() || c == '_' => true,
            Some('(') => true,
            Some('.') => self
                .chars
                .get(self.pos + 1)
                .copied()
                .is_some_and(|c| c.is_ascii_digit()),
            _ => false,
        }
    }

    // — átomos —

    fn parse_atom(&mut self, depth: usize) -> Result<String, String> {
        self.skip_spaces();
        match self.peek() {
            None => Err(self.err("expresión incompleta: falta un operando".into())),
            Some('(') => {
                self.pos += 1;
                let inner = self.parse_cmp(depth + 1)?;
                self.skip_spaces();
                if !self.consume_str(")") {
                    return Err(self.err("falta cerrar el paréntesis".into()));
                }
                Ok(format!("({inner})"))
            }
            Some('{') => Err(self.err(
                "listas `{…}` peladas están fuera del subset: solo valen dentro de `Sum[…, {var, lo, hi}]`, `Clip[…, {lo, hi}]` y `Piecewise`"
                    .into(),
            )),
            Some('[') => Err(self.err(
                "corchete `[` suelto: las llamadas son `Nombre[args]`, con el nombre adelante"
                    .into(),
            )),
            Some(']') => Err(self.err("sobra un `]` de cierre".into())),
            Some('}') => Err(self.err("sobra un `}` de cierre".into())),
            Some(',') => {
                Err(self.err("la coma solo vale dentro de una llamada, p. ej. `Min[x, y]`".into()))
            }
            Some('*') | Some('/') | Some('^') => {
                Err(self.err("falta el operando izquierdo de ese operador".into()))
            }
            Some(c) if c.is_ascii_digit() => self.scan_number(),
            Some('.') => {
                if self
                    .chars
                    .get(self.pos + 1)
                    .copied()
                    .is_some_and(|c| c.is_ascii_digit())
                {
                    self.scan_number()
                } else {
                    Err(self.err("punto suelto, fuera del subset".into()))
                }
            }
            Some(c) if c.is_ascii_alphabetic() || c == '_' => self.parse_ident_or_call(depth),
            Some('$') => Err(self.err(
                "`$` está fuera del subset: usá variables simples sin `$`".into(),
            )),
            _ => Err(self.err("ese caracter está fuera del subset soportado".into())),
        }
    }

    /// Número canónico (entero, decimal o científica `1e-3`).
    fn scan_number(&mut self) -> Result<String, String> {
        let start = self.pos;
        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.pos += 1;
        }
        if self.peek() == Some('.') {
            self.pos += 1;
            while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                self.pos += 1;
            }
        }
        if matches!(self.peek(), Some('e') | Some('E')) {
            let save = self.pos;
            self.pos += 1;
            if matches!(self.peek(), Some('+') | Some('-')) {
                self.pos += 1;
            }
            if self.peek().is_some_and(|c| c.is_ascii_digit()) {
                while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                    self.pos += 1;
                }
            } else {
                self.pos = save; // la `e` era una variable (`2e` → `2*e`)
            }
        }
        let literal: String = self.chars[start..self.pos].iter().collect();
        match literal.parse::<f64>() {
            Ok(v) if v.is_finite() => Ok(literal),
            _ => Err(self.err(format!("número inválido «{literal}»"))),
        }
    }

    /// Variable, constante (`Pi`/`E`) o llamada `Head[…]`.
    fn parse_ident_or_call(&mut self, depth: usize) -> Result<String, String> {
        let name = self.read_ident();
        if name.ends_with('_') || name.contains("__") {
            return Err(self.err(
                "patrones (`x_`, `__`) están fuera del subset: usá variables simples".into(),
            ));
        }
        if is_nonfinite_name(&name) {
            return Err(self.err(format!(
                "«{name}» no es finito: las constantes no finitas están fuera del subset"
            )));
        }
        if name == "I" {
            return Err(self.err(
                "la unidad imaginaria `I` está fuera del subset: el motor es real, usá `re`/`im`/`conj`".into(),
            ));
        }
        if name == "True" || name == "False" {
            return Err(self.err(
                "booleanos (`True`/`False`) están fuera del subset: usá comparaciones (`x > 0`)"
                    .into(),
            ));
        }
        if let Some(canon) = well_known_constant(&name) {
            return Ok(canon.to_string());
        }
        self.skip_spaces();
        if self.peek() == Some('[') {
            return self.dispatch_head(&name, depth);
        }
        if self.peek() == Some('(') {
            // Leniciencia: `sin(x)` con paréntesis vale si es función conocida.
            if let Some(canon) = canonical_for_paren_call(&name) {
                self.pos += 1;
                let mut args = Vec::new();
                self.skip_spaces();
                if self.peek() == Some(')') {
                    self.pos += 1;
                } else {
                    loop {
                        args.push(self.parse_cmp(depth + 1)?);
                        self.skip_spaces();
                        if self.peek() == Some(',') {
                            self.pos += 1;
                            continue;
                        }
                        if self.peek() == Some(')') {
                            self.pos += 1;
                            break;
                        }
                        return Err(self.err(format!("en `{name}(…)` se esperaba `,` o el cierre")));
                    }
                }
                return finish_simple_call(&name, canon, args, self.pos);
            }
            // Si no, la yuxtaposición multiplica: `x(y+1)` = `x*(y+1)`.
            return Ok(name);
        }
        if canonical_for_paren_call(&name).is_some() {
            return Err(self.err(format!(
                "`{name}` es una función: escribila `…[…]` con el argumento entre corchetes, p. ej. `{suggest}[x]`",
                suggest = suggested_head(&name),
            )));
        }
        if let Some(suggest) = lowercase_head_suggestion(&name) {
            return Err(self.err(format!(
                "«{name}» con minúscula no existe en Wolfram: usá `{suggest}[…]` con mayúscula"
            )));
        }
        Ok(name)
    }

    /// Llamada `Head[args]`: el `[` todavía no se consumió.
    fn dispatch_head(&mut self, head: &str, depth: usize) -> Result<String, String> {
        // Estructurales primero (necesitan listas `{…}` como argumentos).
        if head == "Sum" || head == "Product" {
            return self.parse_sum_product(head, depth);
        }
        if head == "Piecewise" {
            return self.parse_piecewise(depth);
        }
        if head == "Clip" {
            return self.parse_clip(depth);
        }
        if head == "PolyGamma" {
            return self.parse_polygamma(head, depth);
        }
        if head == "Log" {
            return self.parse_log(depth);
        }
        if head == "ArcTan" {
            return self.parse_arctan(depth);
        }
        let args = self.bracket_args(head, depth)?;
        let Some(canon) = canonical_for_head(head) else {
            if let Some(suggest) = lowercase_head_suggestion(head) {
                return Err(self.err(format!(
                    "«{head}» con minúscula no existe en Wolfram: usá `{suggest}[…]` con mayúscula"
                )));
            }
            return Err(unsupported_head(head, self.pos));
        };
        finish_simple_call(head, canon, args, self.pos)
    }

    /// Argumentos `Head[a, b]` ya con el nombre consumido; consume `[…]`.
    fn bracket_args(&mut self, head: &str, depth: usize) -> Result<Vec<String>, String> {
        self.skip_spaces();
        if self.peek() != Some('[') {
            return Err(self.err(format!("a `{head}` le faltan los corchetes: `{head}[…]`")));
        }
        self.pos += 1;
        let mut args = Vec::new();
        self.skip_spaces();
        if self.peek() == Some(']') {
            self.pos += 1;
            return Ok(args);
        }
        loop {
            args.push(self.parse_cmp(depth + 1)?);
            self.skip_spaces();
            match self.peek() {
                Some(',') => {
                    self.pos += 1;
                }
                Some(']') => {
                    self.pos += 1;
                    break;
                }
                _ => {
                    return Err(
                        self.err(format!("en `{head}[…]` se esperaba `,` o el `]` de cierre"))
                    );
                }
            }
        }
        Ok(args)
    }

    /// Parte una lista `{a, b, …}` en textos crudos (sin traducir) hasta el `}`.
    fn split_brace_list_raw(&mut self, owner: &str) -> Result<Vec<String>, String> {
        self.skip_spaces();
        if self.peek() != Some('{') {
            return Err(self.err(format!(
                "`{owner}` necesita su lista entre llaves, p. ej. `{{…}}`"
            )));
        }
        self.pos += 1;
        let start = self.pos;
        let mut nested: usize = 0;
        let mut items: Vec<String> = Vec::new();
        let mut current_start = start;
        let end = loop {
            match self.peek() {
                None => {
                    return Err(self.err(format!("falta cerrar la llave de `{owner}`")));
                }
                Some('(') | Some('[') | Some('{') => {
                    nested += 1;
                    self.pos += 1;
                }
                Some(')') | Some(']') | Some('}') => {
                    if nested == 0 {
                        break self.pos;
                    }
                    nested = nested.saturating_sub(1);
                    self.pos += 1;
                }
                Some(',') if nested == 0 => {
                    items.push(self.chars[current_start..self.pos].iter().collect());
                    self.pos += 1;
                    current_start = self.pos;
                }
                Some(_) => self.pos += 1,
            }
        };
        items.push(self.chars[current_start..end].iter().collect());
        self.pos = end + 1; // consume el `}` que cierra
        Ok(items)
    }

    /// Lee una lista `{a, b, …}` y traduce cada ítem como expresión.
    fn brace_list_raw(&mut self, owner: &str, depth: usize) -> Result<Vec<String>, String> {
        let items = self.split_brace_list_raw(owner)?;
        // Traduce cada ítem con un sub-parser (misma profundidad + 1).
        let mut out = Vec::with_capacity(items.len());
        for raw in items {
            let mut sub = Parser {
                chars: raw.chars().collect(),
                pos: 0,
            };
            let translated = sub.parse_cmp(depth + 1)?;
            sub.skip_spaces();
            if sub.peek().is_some() {
                return Err(self.err(format!("eso dentro de `{owner}` no se entiende")));
            }
            out.push(translated);
        }
        Ok(out)
    }

    fn parse_sum_product(&mut self, head: &str, depth: usize) -> Result<String, String> {
        let canon = if head == "Sum" { "sum" } else { "product" };
        self.skip_spaces();
        if self.peek() != Some('[') {
            return Err(self.err(format!("a `{head}` le faltan los corchetes")));
        }
        self.pos += 1;
        let body = self.parse_cmp(depth + 1)?;
        self.skip_spaces();
        if self.peek() != Some(',') {
            return Err(self.err(format!(
                "`{head}` lleva cuerpo e iterador: `{head}[cuerpo, {{var, lo, hi}}]`"
            )));
        }
        self.pos += 1;
        let iter = self.brace_list_raw(head, depth)?;
        if iter.len() != 3 {
            return Err(self.err(format!(
                "el iterador de `{head}` lleva 3 partes `{{var, lo, hi}}`, le pasaste {}",
                iter.len()
            )));
        }
        if !is_plain_var(&iter[0]) {
            return Err(self.err(format!(
                "el iterador de `{head}` va sobre una variable simple, no «{}»",
                iter[0]
            )));
        }
        self.skip_spaces();
        if self.peek() != Some(']') {
            return Err(self.err(format!("falta cerrar el `]` de `{head}`")));
        }
        self.pos += 1;
        Ok(format!(
            "{canon}({}, {}, {}, {})",
            body, iter[0], iter[1], iter[2]
        ))
    }

    fn parse_clip(&mut self, depth: usize) -> Result<String, String> {
        self.skip_spaces();
        if self.peek() != Some('[') {
            return Err(self.err("a `Clip` le faltan los corchetes".into()));
        }
        self.pos += 1;
        let value = self.parse_cmp(depth + 1)?;
        self.skip_spaces();
        if self.peek() != Some(',') {
            return Err(self.err("`Clip` lleva valor y rango: `Clip[x, {lo, hi}]`".into()));
        }
        self.pos += 1;
        let bounds = self.brace_list_raw("Clip", depth)?;
        if bounds.len() != 2 {
            return Err(self.err(format!(
                "el rango de `Clip` lleva 2 partes `{{lo, hi}}`, le pasaste {}",
                bounds.len()
            )));
        }
        self.skip_spaces();
        if self.peek() != Some(']') {
            return Err(self.err("falta cerrar el `]` de `Clip`".into()));
        }
        self.pos += 1;
        Ok(format!("clamp({}, {}, {})", value, bounds[0], bounds[1]))
    }

    fn parse_polygamma(&mut self, head: &str, depth: usize) -> Result<String, String> {
        let args = self.bracket_args(head, depth)?;
        match args.len() {
            1 => Ok(format!("digamma({})", args[0])),
            2 => {
                let order = args[0].trim();
                if order == "0" {
                    Ok(format!("digamma({})", args[1]))
                } else if order == "1" {
                    Ok(format!("trigamma({})", args[1]))
                } else {
                    Err(self.err(format!(
                        "`PolyGamma` solo cubre digamma (orden 0) y trigamma (orden 1): pediste orden «{}»",
                        args[0]
                    )))
                }
            }
            n => Err(self.err(format!(
                "`PolyGamma` lleva 1 o 2 argumentos, le pasaste {n}"
            ))),
        }
    }

    fn parse_log(&mut self, depth: usize) -> Result<String, String> {
        let args = self.bracket_args("Log", depth)?;
        match args.len() {
            1 => Ok(format!("ln({})", args[0])),
            2 => {
                if args[0].trim() == "10" {
                    Ok(format!("log({})", args[1]))
                } else {
                    Err(self.err(format!(
                        "logaritmo en base «{}», fuera del subset: el motor solo tiene natural (`Log[x]`) y base 10 (`Log[10, x]`)",
                        args[0]
                    )))
                }
            }
            n => Err(self.err(format!("`Log` lleva 1 o 2 argumentos, le pasaste {n}"))),
        }
    }

    fn parse_arctan(&mut self, depth: usize) -> Result<String, String> {
        let args = self.bracket_args("ArcTan", depth)?;
        match args.len() {
            1 => Ok(format!("atan({})", args[0])),
            // Wolfram `ArcTan[x, y]` = atan2(y, x): el canónico ordena (y, x).
            2 => Ok(format!("atan2({}, {})", args[1], args[0])),
            n => Err(self.err(format!("`ArcTan` lleva 1 o 2 argumentos, le pasaste {n}"))),
        }
    }

    fn parse_piecewise(&mut self, depth: usize) -> Result<String, String> {
        self.skip_spaces();
        if self.peek() != Some('[') {
            return Err(self.err("a `Piecewise` le faltan los corchetes".into()));
        }
        self.pos += 1;
        self.skip_spaces();
        // Lista externa de pares `{{v1, c1}, …}`: se extrae cruda hasta su `}`.
        if self.peek() != Some('{') {
            return Err(self.err(
                "`Piecewise` lleva lista de pares: `Piecewise[{{v1, c1}, …}, default]`".into(),
            ));
        }
        // Reusa el partidor crudo sobre la envoltura: lee `{par1, par2, …}`.
        let pairs_raw_nested = self.split_brace_list_raw("Piecewise")?;
        // Cada elemento es `{val, cond}` crudo: se parte de nuevo por dentro.
        let mut canon_parts: Vec<String> = Vec::new();
        for pair in pairs_raw_nested {
            let mut sub = Parser {
                chars: pair.chars().collect(),
                pos: 0,
            };
            sub.skip_spaces();
            if sub.peek() != Some('{') {
                return Err(
                    self.err("cada caso de `Piecewise` es un par `{valor, condición}`".into())
                );
            }
            let both = sub.brace_list_raw("Piecewise", depth)?;
            if both.len() != 2 {
                return Err(self.err(format!(
                    "cada caso de `Piecewise` lleva 2 partes `{{valor, condición}}`, le pasaste {}",
                    both.len()
                )));
            }
            // Canónico: (cond, val).
            canon_parts.push(both[1].clone());
            canon_parts.push(both[0].clone());
        }
        self.skip_spaces();
        if self.peek() != Some(',') {
            return Err(
                self.err("a `Piecewise` le falta el valor por defecto: `…, default]`".into())
            );
        }
        self.pos += 1;
        let default = self.parse_cmp(depth + 1)?;
        self.skip_spaces();
        if self.peek() != Some(']') {
            return Err(self.err("falta cerrar el `]` de `Piecewise`".into()));
        }
        self.pos += 1;
        let mut all = canon_parts;
        all.push(default);
        Ok(format!("piecewise({})", all.join(", ")))
    }
}

// ── Tablas y validación ──────────────────────────────────────────────────────

/// Constantes Wolfram (exactas) → nombre canónico.
fn well_known_constant(name: &str) -> Option<&'static str> {
    match name {
        "Pi" | "pi" => Some("pi"),
        "E" => Some("e"),
        _ => None,
    }
}

/// Nombres que el parser canónico leería como no-finitos.
fn is_nonfinite_name(name: &str) -> bool {
    matches!(
        name,
        "Infinity" | "ComplexInfinity" | "Indeterminate" | "NaN" | "nan" | "inf" | "infinity"
    )
}

/// Head Wolfram (o alias leniente con paréntesis) → nombre canónico.
/// Devuelve `None` si no es una función del subset.
fn canonical_for_head(head: &str) -> Option<&'static str> {
    // Estricto en mayúsculas (Wolfram lo es): `sin[x]` se rechaza con guía a
    // `Sin[x]`. La leniencia con paréntesis (`sin(x)`) va por
    // `canonical_for_paren_call`.
    match head {
        "Sin" => Some("sin"),
        "Cos" => Some("cos"),
        "Tan" => Some("tan"),
        "ArcSin" => Some("asin"),
        "ArcCos" => Some("acos"),
        "ArcTan" => Some("atan"),
        "Exp" => Some("exp"),
        "Sqrt" => Some("sqrt"),
        "Abs" => Some("abs"),
        "Sinh" => Some("sinh"),
        "Cosh" => Some("cosh"),
        "Tanh" => Some("tanh"),
        "Floor" => Some("floor"),
        "Ceiling" => Some("ceil"),
        "Round" => Some("round"),
        "Sec" => Some("sec"),
        "Csc" => Some("csc"),
        "Cot" => Some("cot"),
        "ArcSinh" => Some("asinh"),
        "ArcCosh" => Some("acosh"),
        "ArcTanh" => Some("atanh"),
        "Sign" => Some("sign"),
        "HeavisideTheta" => Some("heaviside"),
        "CubeRoot" => Some("cbrt"),
        "Min" => Some("min"),
        "Max" => Some("max"),
        "Mod" => Some("mod"),
        "Re" => Some("re"),
        "Im" => Some("im"),
        "Arg" => Some("arg"),
        "Conjugate" => Some("conj"),
        "Erf" => Some("erf"),
        "Erfc" => Some("erfc"),
        "Gamma" => Some("gamma"),
        "LogGamma" => Some("lngamma"),
        "Beta" => Some("beta"),
        "BesselJ" => Some("besselj"),
        "BesselY" => Some("bessely"),
        "BesselI" => Some("besseli"),
        "Rational" => Some("__rational"),
        "Power" => Some("__power"),
        "Plus" => Some("__plus"),
        "Times" => Some("__times"),
        "Divide" => Some("__divide"),
        _ => None,
    }
}

/// Alias lenientes solo para la forma con paréntesis (`sin(x)` vale como
/// `Sin[x]`; con corchetes Wolfram exige la mayúscula).
fn canonical_for_paren_call(name: &str) -> Option<&'static str> {
    if let Some(canon) = canonical_for_head(name) {
        return Some(canon);
    }
    match name.to_lowercase().as_str() {
        "sin" | "sen" => Some("sin"),
        "cos" => Some("cos"),
        "tan" => Some("tan"),
        "arcsin" | "asin" => Some("asin"),
        "arccos" | "acos" => Some("acos"),
        "arctan" | "atan" => Some("atan"),
        "exp" => Some("exp"),
        "sqrt" => Some("sqrt"),
        "abs" => Some("abs"),
        "ln" => Some("ln"),
        "log" => Some("log"),
        "min" => Some("min"),
        "max" => Some("max"),
        _ => None,
    }
}
/// Head canónica sugerida para el mensaje de "es una función".
fn suggested_head(name: &str) -> String {
    let lower = name.to_lowercase();
    match lower.as_str() {
        "sin" | "sen" => "Sin",
        "cos" => "Cos",
        "tan" => "Tan",
        "arcsin" | "asin" => "ArcSin",
        "arccos" | "acos" => "ArcCos",
        "arctan" | "atan" => "ArcTan",
        "exp" => "Exp",
        "sqrt" => "Sqrt",
        "abs" => "Abs",
        "floor" => "Floor",
        "ceiling" | "ceil" => "Ceiling",
        "log" => "Log",
        "min" => "Min",
        "max" => "Max",
        _ => "Nombre",
    }
    .to_string()
}

/// Si el nombre en minúsculas sería un Head válido, sugiere la mayúscula.
fn lowercase_head_suggestion(name: &str) -> Option<String> {
    let mut chars = name.chars();
    let first = chars.next()?;
    if !first.is_ascii_lowercase() {
        return None;
    }
    let upper: String = first.to_ascii_uppercase().to_string() + chars.as_str();
    if canonical_for_head(&upper).is_some() || matches!(upper.as_str(), "Log" | "Pi") {
        return Some(upper);
    }
    None
}

/// Aridad esperada (`None` = `Plus`/`Times`/`Piecewise`, que aceptan 1 o más).
fn expected_arity(canon: &str) -> Option<usize> {
    match canon {
        "__plus" | "__times" => None,
        "min" | "max" | "mod" | "beta" | "besselj" | "bessely" | "besseli" => Some(2),
        "__rational" | "__power" | "__divide" => Some(2),
        _ => Some(1),
    }
}

/// Ensambla la llamada simple ya traducida (incluye las formas estructurales
/// `Rational`/`Power`/`Plus`/`Times`/`Divide` y la aridad honesta).
fn finish_simple_call(
    display: &str,
    canon: &str,
    args: Vec<String>,
    pos: usize,
) -> Result<String, String> {
    match expected_arity(canon) {
        Some(want) if args.len() != want => {
            return Err(format!(
                "ParseWolfram: `{display}` lleva {want} argumento(s), le pasaste {} (posición {pos})",
                args.len()
            ));
        }
        None if args.is_empty() => {
            return Err(format!(
                "ParseWolfram: `{display}` necesita al menos 1 argumento (posición {pos})"
            ));
        }
        _ => {}
    }
    match canon {
        "__rational" => Ok(format!("({})/({})", args[0], args[1])),
        "__power" => Ok(format!("{}^({})", args[0], args[1])),
        "__divide" => Ok(format!("({})/({})", args[0], args[1])),
        "__plus" => Ok(args.join(" + ")),
        "__times" => Ok(args.join("*")),
        _ => Ok(format!("{canon}({})", args.join(", "))),
    }
}

/// Error honesto para Heads fuera del subset (nombra al culpable).
fn unsupported_head(head: &str, pos: usize) -> String {
    match head {
        "Integrate" | "D" | "DSolve" | "Limit" | "Sum" | "Product" | "Table" => {
            format!(
                "ParseWolfram: `{head}` (cálculo) está fuera del subset: este motor traduce expresiones, no las evalúa (posición {pos})"
            )
        }
        "Solve" | "NSolve" | "Reduce" | "FindRoot" => {
            format!(
                "ParseWolfram: `{head}` (solvers) está fuera del subset: este motor traduce expresiones (posición {pos})"
            )
        }
        "MatrixForm" | "Matrix" | "TableForm" | "Grid" | "List" | "SparseArray" => {
            format!(
                "ParseWolfram: `{head}` (listas/matrices) está fuera del subset de expresiones escalares (posición {pos})"
            )
        }
        "Plot" | "Plot3D" | "Graphics" | "Show" => {
            format!(
                "ParseWolfram: `{head}` (gráficos) está fuera del subset: este motor traduce expresiones (posición {pos})"
            )
        }
        "N" | "Simplify" | "Expand" | "Factor" | "Together" | "Cancel" => {
            format!(
                "ParseWolfram: `{head}` (evaluación simbólica) está fuera del subset: este motor traduce expresiones (posición {pos})"
            )
        }
        "EulerGamma" | "Catalan" | "GoldenRatio" | "Degree" | "Glaisher" | "Khinchin" => {
            format!(
                "ParseWolfram: la constante `{head}` no tiene equivalente canónico, fuera del subset (posición {pos})"
            )
        }
        _ => {
            format!(
                "ParseWolfram: `{head}` no está soportado (posición {pos}). Subset: aritmética `+ - * / ^`, `Pi`, `E` y llamadas `Sin[x]`, `Cos[x]`, `Sqrt[x]`, `Log[x]`, `Min[x, y]`, `Sum[x, {{n, 1, 10}}]`…"
            )
        }
    }
}

/// Variable simple (`x`, `theta`, sin `_` raro).
fn is_plain_var(text: &str) -> bool {
    let mut chars = text.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    text.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::{parse_wolfram, to_wolfram};

    fn must_parse(input: &str) -> String {
        match parse_wolfram(input) {
            Ok(out) => out,
            Err(e) => panic!("parse_wolfram({input:?}) debía dar Ok, dio Err: {e}"),
        }
    }

    fn must_fail(input: &str) -> String {
        match parse_wolfram(input) {
            Ok(out) => panic!("parse_wolfram({input:?}) debía fallar, dio Ok: {out}"),
            Err(e) => e,
        }
    }

    #[test]
    fn to_wolfram_basico() {
        assert_eq!(to_wolfram("x^2 + sin(x)"), "x^2 + Sin[x]");
        assert_eq!(to_wolfram("x*y + 2"), "x*y + 2");
        assert_eq!(to_wolfram("sqrt(x) + abs(y)"), "Sqrt[x] + Abs[y]");
    }

    #[test]
    fn to_wolfram_constantes_y_log() {
        assert_eq!(to_wolfram("pi"), "Pi");
        assert_eq!(to_wolfram("e"), "E");
        assert_eq!(to_wolfram("tau"), "(2*Pi)");
        assert_eq!(to_wolfram("ln(x)"), "Log[x]");
        assert_eq!(to_wolfram("log(x)"), "Log[10, x]");
        assert_eq!(to_wolfram("atan2(y, x)"), "ArcTan[x, y]");
        assert_eq!(to_wolfram("clamp(x, 0, 1)"), "Clip[x, {0, 1}]");
        assert_eq!(to_wolfram("min(a, b)"), "Min[a, b]");
        assert_eq!(to_wolfram("sum(x, n, 1, 10)"), "Sum[x, {n, 1, 10}]");
        assert_eq!(
            to_wolfram("piecewise(x > 0, 1, 0)"),
            "Piecewise[{{1, x > 0}}, 0]"
        );
        assert_eq!(to_wolfram("digamma(x)"), "PolyGamma[x]");
        assert_eq!(to_wolfram("trigamma(x)"), "PolyGamma[1, x]");
    }

    #[test]
    fn to_wolfram_acepta_ast() {
        let ast = match crate::ast::parse_ast("x^2 + sin(x)") {
            Ok(ast) => ast,
            Err(e) => panic!("parse_ast debía dar Ok, dio Err: {e}"),
        };
        assert_eq!(to_wolfram(&ast), to_wolfram("x^2 + sin(x)"));
    }

    #[test]
    fn parse_wolfram_basico() {
        assert_eq!(must_parse("Sin[x] + x^2"), "sin(x) + x^(2)");
        assert_eq!(must_parse("Pi"), "pi");
        assert_eq!(must_parse("E"), "e");
        assert_eq!(must_parse("Sqrt[x]"), "sqrt(x)");
        assert_eq!(must_parse("2x + 3"), "2*x + 3");
        assert_eq!(must_parse("Min[x, y]"), "min(x, y)");
        assert_eq!(must_parse("x^2^3"), "x^(2^(3))");
    }

    #[test]
    fn parse_wolfram_log_una_y_dos() {
        assert_eq!(must_parse("Log[x]"), "ln(x)");
        assert_eq!(must_parse("Log[10, x]"), "log(x)");
        let e = must_fail("Log[2, x]");
        assert!(e.contains("base"), "base honesta: {e}");
    }

    #[test]
    fn parse_wolfram_arctan_y_clip() {
        assert_eq!(must_parse("ArcTan[x, y]"), "atan2(y, x)");
        assert_eq!(must_parse("ArcTan[x]"), "atan(x)");
        assert_eq!(must_parse("Clip[x, {0, 1}]"), "clamp(x, 0, 1)");
    }

    #[test]
    fn parse_wolfram_sum_product_piecewise() {
        assert_eq!(must_parse("Sum[x, {n, 1, 10}]"), "sum(x, n, 1, 10)");
        assert_eq!(must_parse("Product[x, {n, 1, 10}]"), "product(x, n, 1, 10)");
        assert_eq!(
            must_parse("Piecewise[{{1, x > 0}}, 0]"),
            "piecewise(x > 0, 1, 0)"
        );
    }

    #[test]
    fn parse_wolfram_polygamma() {
        assert_eq!(must_parse("PolyGamma[x]"), "digamma(x)");
        assert_eq!(must_parse("PolyGamma[0, x]"), "digamma(x)");
        assert_eq!(must_parse("PolyGamma[1, x]"), "trigamma(x)");
        let e = must_fail("PolyGamma[2, x]");
        assert!(e.contains("orden"), "orden honesto: {e}");
    }

    #[test]
    fn parse_wolfram_errores_honestos() {
        for (input, pista) in [
            ("Integrate[x]", "Integrate"),
            ("I", "imaginaria"),
            ("Infinity", "finito"),
            ("Sin[x, y]", "1 argumento"),
            ("Min[x, y, z]", "2 argumento"),
            ("Foo[x]", "Foo"),
            ("{1, 2}", "listas"),
            ("x_", "patrones"),
        ] {
            let e = must_fail(input);
            assert!(e.contains(pista), "{input:?} debía nombrar «{pista}»: {e}");
        }
        let minuscula = must_fail("sin[x]");
        assert!(minuscula.contains("Sin"), "sugiere mayúscula: {minuscula}");
        let vacio = must_fail("   ");
        assert!(vacio.contains("vacía"), "vacío honesto: {vacio}");
        let larga = "x+".repeat(1500);
        let e = must_fail(&larga);
        assert!(e.contains("excede"), "tope honesto: {e}");
    }

    #[test]
    fn wolfram_roundtrip_estructural() {
        let casos = [
            "x^2 + sin(x)",
            "sin(x)^(2) + (x + 1)/(x - 1)",
            "ln(x) + log(y)",
            "sqrt(x) + cbrt(y)",
            "min(a, b) + max(c, d)",
            "atan2(y, x) + clamp(x, 0, 1)",
            "sum(x, n, 1, 10) + product(x, n, 1, 10)",
            "piecewise(x > 0, 1, 0)",
            "digamma(x) + trigamma(y)",
            "beta(a, b) + besselj(n, x)",
            "x == y",
            "-(x + 1)*2",
        ];
        for canon in casos {
            let wl = to_wolfram(canon);
            let de_vuelta = must_parse(&wl);
            let antes = match crate::ast::parse_ast(canon) {
                Ok(ast) => ast,
                Err(e) => panic!("parse_ast({canon:?}) debía dar Ok: {e}"),
            };
            let despues = match crate::ast::parse_ast(&de_vuelta) {
                Ok(ast) => ast,
                Err(e) => panic!("parse_ast({de_vuelta:?}) debía dar Ok: {e}"),
            };
            assert!(
                antes.structurally_eq(&despues),
                "roundtrip «{canon}» → «{wl}» → «{de_vuelta}» cambió la estructura"
            );
        }
    }
}
