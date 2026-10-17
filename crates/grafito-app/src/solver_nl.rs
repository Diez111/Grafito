//! Solucionador paso a paso tipo Photomath (local-primero).
//!
//! Entrada en lenguaje natural → comando `StepByStep[Op[…]]` verificado por
//! el motor (`grafito-geometry::cas_steps`), nunca inventado por IA. La IA
//! solo narra encima del cálculo local. Puro y testeable: sin `Document`,
//! sin red, sin egui.

/// Pedido normalizado al solver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolverRequest {
    /// Comando listo para `execute_command_and_record`, p. ej.
    /// `StepByStep[Derivative[x^2, x]]`.
    pub command: String,
    /// Etiqueta corta para el toast (`Derivada`, `Integral`, …).
    pub label: String,
    /// `true` si el comando trae pasos verificados (`StepByStep`); `false`
    /// en directa honesta (`Simplify`/`Factor`/`Expand`: sin pasos).
    pub steps: bool,
    /// Expresión para graficar en el Canvas si aplica (`Some("x^2")`).
    pub graph_expr: Option<String>,
}

/// Normaliza: minúsculas + sin tildes + espacios colapsados.
/// (Mismo contrato que el fix del bug #5 en `assistant.rs`.)
fn norm(prompt: &str) -> String {
    let plegado = grafito_ui::command_palette::fold_spanish(&prompt.trim().to_lowercase());
    // Superíndices → caret: el teclado móvil (y la prosa) escriben `x²`
    // y el CAS espera `x^2` — sin esto el pedido llegaba crudo y el motor
    // devolvía error de sintaxis (parecía "falla de la API").
    let transcrito = plegado
        .replace('²', "^2")
        .replace('³', "^3")
        .replace('⁴', "^4");
    transcrito.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Quita muletillas de pedido para quedarse con la matemática cruda.
/// Por tokens (no subcadenas): evita romper `derivada` al sacar `de`.
fn strip_ask(p: &str) -> String {
    // Frases multi-palabra primero (subcadena acá sí es segura).
    let mut s = format!(" {p} ");
    for cut in [
        "paso a paso",
        "paso por paso",
        "por favor",
        "porfa",
        "tiende a ",
        "tiende al ",
        "respecto a ",
        "respecto de ",
    ] {
        s = s.replace(cut, " ");
    }
    const STOP: &[&str] = &[
        "muestrame",
        "mostrame",
        "mostrarme",
        "explicame",
        "explica",
        "resolveme",
        "resolve",
        "resuelve",
        "resolver",
        "resolv",
        "calcula",
        "calculame",
        "calcular",
        "halla",
        "hallar",
        "encuentra",
        "saca",
        "dame",
        "graficame",
        "grafica",
        "graficar",
        "grafic",
        "derivada",
        "derivar",
        "deriva",
        "deriv",
        "integral",
        "integra",
        "integrar",
        "integr",
        "limite",
        "limit",
        "taylor",
        "laplace",
        "simplifica",
        "simplificar",
        "simplific",
        "factoriza",
        "factorizar",
        "factoriz",
        "factorea",
        "expand",
        "expande",
        "expandir",
        "desarrolla",
        "desarrollar",
        "desarroll",
        "ecuacion",
        "despeja",
        "despejar",
        "despej",
        "de",
        "la",
        "el",
        "del",
        "en",
        "cuando",
        "y",
    ];
    s.split_whitespace()
        .filter(|w| {
            let w = w.trim_matches(|c: char| {
                !c.is_alphanumeric()
                    && !matches!(
                        c,
                        '^' | '=' | '(' | ')' | '+' | '-' | '*' | '/' | '.' | '_' | '→'
                    )
            });
            !w.is_empty() && !STOP.contains(&w)
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Limpia una parte separada por comas hasta la ecuación cruda: quita
/// muletillas + keyword `sistema` y pega los tokens matemáticos.
/// `None` si no queda nada con `=`.
fn clean_system_part(part: &str) -> Option<String> {
    let mut s = format!(" {part} ");
    for cut in [
        "paso a paso",
        "paso por paso",
        "por favor",
        "porfa",
        "sistema de ecuaciones",
        "sistema de",
        "sistemas",
        "sistema",
        "ecuaciones",
        "ecuacion",
    ] {
        s = s.replace(cut, " ");
    }
    const STOP: &[&str] = &[
        "muestrame",
        "mostrame",
        "mostrarme",
        "explicame",
        "explica",
        "resolveme",
        "resolve",
        "resuelve",
        "resolver",
        "resolv",
        "calcula",
        "calculame",
        "calcular",
        "halla",
        "hallar",
        "encuentra",
        "saca",
        "dame",
        "graficame",
        "grafica",
        "graficar",
        "grafic",
        "de",
        "la",
        "el",
        "del",
        "en",
        "cuando",
        "y",
    ];
    let kept: Vec<String> = s
        .split_whitespace()
        .map(|w| {
            w.trim_matches(|c: char| {
                !c.is_alphanumeric()
                    && !matches!(
                        c,
                        '^' | '=' | '(' | ')' | '+' | '-' | '*' | '/' | '.' | '_' | '→'
                    )
            })
            .to_string()
        })
        .filter(|w| !w.is_empty() && !STOP.contains(&w.as_str()))
        .collect();
    if kept.is_empty() {
        return None;
    }
    let joined = kept.join("");
    (joined.contains('=') && joined.len() >= 3 && joined.len() <= 200).then_some(joined)
}

/// Extrae las ecuaciones de un sistema del pedido normalizado: partes con
/// `=` separadas por `,`/`;`/`|`, o tokens con `=` si no hay comas.
/// Vacío o > 2 también se devuelve tal cual: el llamador responde honesto.
fn extract_system_equations(p: &str) -> Vec<String> {
    let normalized = p.replace([';', '|'], ",");
    if normalized.contains(',') {
        normalized
            .split(',')
            .filter_map(clean_system_part)
            .collect()
    } else {
        p.split_whitespace()
            .filter_map(|tok| {
                let t = tok
                    .trim_matches(|c: char| {
                        matches!(
                            c,
                            '"' | '\'' | '¿' | '?' | '¡' | '!' | '.' | ':' | ';' | ','
                        )
                    })
                    .trim();
                // Prosa pegada con `:` (`sistema:x+y=3`): quédate con el
                // fragmento que trae el `=`.
                let frag = t.split(':').find(|f| f.contains('='))?;
                let frag = frag.trim();
                (frag.len() >= 3 && frag.len() <= 200).then_some(frag.to_string())
            })
            .collect()
    }
}

/// Dos incógnitas de un sistema 2×2 por orden de aparición (sin contar las
/// letras de `sin/cos/tan/log/exp/sqrt/abs/atan`). `None` si no hay
/// exactamente 2: no se adivina, el llamador responde honesto.
fn detect_system_vars(eq1: &str, eq2: &str) -> Option<(String, String)> {
    let mut joined = format!("{eq1} {eq2}");
    for func in [
        "atan", "asin", "acos", "sqrt", "sin", "cos", "tan", "log", "exp", "abs",
    ] {
        joined = joined.replace(func, " ");
    }
    let mut vars = Vec::new();
    for ch in joined.chars() {
        if ch.is_ascii_alphabetic() && !vars.contains(&ch) {
            vars.push(ch);
        }
    }
    if vars.len() == 2 {
        Some((vars[0].to_string(), vars[1].to_string()))
    } else {
        None
    }
}

/// Número compacto (`3` en vez de `3.000000`) para el despeje a graficar.
fn fmt_graph_num(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        let s = format!("{v:.6}");
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

/// Despeja `y(x)` de una recta `a·x + b·y + c = 0` para graficarla como
/// `Function[y(x)]`. `None` si no es recta graficable (vertical o no lineal).
fn despeja_y_para_grafico(eq: &str, x: &str, y: &str) -> Option<String> {
    let (a, b, c) = grafito_geometry::cas_steps::linear_eq_coeffs_2x2(eq, x, y)?;
    if !a.is_finite() || !b.is_finite() || !c.is_finite() || b.abs() < 1e-12 {
        return None;
    }
    let (m, k) = (-a / b, -c / b);
    if !m.is_finite() || !k.is_finite() {
        return None;
    }
    let expr = if m.abs() < 1e-12 {
        fmt_graph_num(k)
    } else if k.abs() < 1e-12 {
        if (m - 1.0).abs() < 1e-12 {
            x.to_string()
        } else if (m + 1.0).abs() < 1e-12 {
            format!("-{x}")
        } else {
            format!("{}*{x}", fmt_graph_num(m))
        }
    } else if (m - 1.0).abs() < 1e-12 {
        format!("{x}+{}", fmt_graph_num(k))
    } else if (m + 1.0).abs() < 1e-12 {
        format!("{}-{x}", fmt_graph_num(k))
    } else {
        format!("{}*{x}+{}", fmt_graph_num(m), fmt_graph_num(k))
    };
    (is_plottable(&expr) && expr.len() <= 120).then_some(expr)
}

/// Intento de sistema 2×2: `Some` con el pedido listo, `None` honesto si no
/// son exactamente 2 ecuaciones con 2 incógnitas (el redirect de
/// `assistant.rs` pide las 2 ecuaciones; acá nunca se inventa).
fn try_parse_system(p: &str) -> Option<SolverRequest> {
    let eqs = extract_system_equations(p);
    if eqs.len() != 2 {
        return None;
    }
    let (v1, v2) = detect_system_vars(&eqs[0], &eqs[1])?;
    // El gráfico del Canvas es `y = f(x)`: solo con incógnitas x e y; con
    // otras incógnitas igual hay pasos, pero sin gráfico inventado.
    let graph_expr =
        if [v1.as_str(), v2.as_str()].contains(&"x") && [v1.as_str(), v2.as_str()].contains(&"y") {
            despeja_y_para_grafico(&eqs[0], "x", "y")
                .or_else(|| despeja_y_para_grafico(&eqs[1], "x", "y"))
                .filter(|g| is_plottable(g))
        } else {
            None
        };
    Some(SolverRequest {
        command: format!(
            "StepByStep[SolveNlSystem[{}, {}, {v1}, {v2}]]",
            eqs[0], eqs[1]
        ),
        label: "Sistema".to_string(),
        steps: true,
        graph_expr,
    })
}

/// Ecuación con `y'` del pedido: tokens matemáticos desde el que trae
/// `y'` (con `=` exactamente una vez). `None` si no hay forma `a = b`.
fn extract_ode_equation(p: &str) -> Option<String> {
    let mut s = format!(" {p} ");
    for cut in ["paso a paso", "paso por paso", "por favor", "porfa"] {
        s = s.replace(cut, " ");
    }
    let s = s.replace('′', "'");
    let toks: Vec<&str> = s.split_whitespace().collect();
    let inicio = toks.iter().position(|t| t.contains("y'"))?;
    let mut eq = String::new();
    for tok in toks.iter().skip(inicio) {
        let matematico = !tok.is_empty()
            && tok.len() <= 60
            && tok.chars().all(|c| {
                c.is_alphanumeric()
                    || matches!(
                        c,
                        '=' | '+' | '-' | '*' | '/' | '^' | '(' | ')' | '.' | '_' | '\'' | '→'
                    )
            });
        if !matematico {
            break;
        }
        eq.push_str(tok);
    }
    (eq.matches('=').count() == 1 && (4..=200).contains(&eq.len())).then_some(eq)
}

/// Intento de EDO lineal de 1er orden `y' + P·y = Q`: `Some` con
/// `StepByStep[SolveODE1[P, Q, x]]` directo (sin redirect), `None` honesto
/// si es ambigua, de 2º orden o no lineal (el redirect de `assistant.rs`
/// pide los coeficientes; acá nunca se inventa).
fn try_parse_ode_first_order(p: &str) -> Option<SolverRequest> {
    let eq = extract_ode_equation(p)?;
    if eq.contains("''") || eq.contains('″') {
        return None;
    }
    let x = detect_var(p);
    let (pp, qq) = grafito_geometry::ode::edo_first_order_parts(&eq, &x, "y")?;
    Some(SolverRequest {
        command: format!("StepByStep[SolveODE1[{pp}, {qq}, {x}]]"),
        label: "EDO lineal".to_string(),
        steps: true,
        graph_expr: None,
    })
}
/// Listas numéricas `[a, b]` o `{a, b}` del pedido (en orden de aparición).
/// Solo dígitos y separadores: texto adentro invalida la lista (`None`).
fn extract_stats_lists(p: &str) -> Vec<Vec<String>> {
    let mut listas = Vec::new();
    let bytes = p.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let (apertura, cierre) = match bytes[i] as char {
            '[' => ('[', ']'),
            '{' => ('{', '}'),
            _ => {
                i += 1;
                continue;
            }
        };
        let _ = apertura;
        if let Some(fin) = p[i + 1..].find(cierre) {
            let cuerpo = &p[i + 1..i + 1 + fin];
            let nums: Vec<String> = cuerpo
                .split([',', ';'])
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect();
            if !nums.is_empty()
                && nums.len() <= 10_000
                && nums
                    .iter()
                    .all(|t| t.parse::<f64>().is_ok_and(|v| v.is_finite()))
            {
                listas.push(nums);
            }
            i += fin + 2;
        } else {
            break;
        }
    }
    listas
}

/// Token limpio (sin puntuación de borde) para keywords de stats.
fn stats_token(tok: &str) -> &str {
    tok.trim_matches(|c: char| !c.is_alphanumeric())
}

/// Intento de estadística con lista explícita (`media de [1,2,3]`): `Some`
/// con `StepByStep[Mean|Median|Variance|SD|LinearRegression]` directo (sin
/// redirect), `None` honesto si no hay lista numérica (el redirect de
/// `assistant.rs` deriva cuantil/distribución a plantilla; acá nunca se
/// inventan datos).
fn try_parse_stats(p: &str) -> Option<SolverRequest> {
    let toks: Vec<&str> = p.split_whitespace().map(stats_token).collect();
    let tiene = |kw: &[&str]| toks.iter().any(|t| kw.contains(t));
    let es_regresion = tiene(&["regresion", "ajuste"]);
    let listas = extract_stats_lists(p);
    if es_regresion {
        let (xs, ys) = (listas.first(), listas.get(1));
        let (xs, ys) = (xs?, ys?);
        if xs.len() != ys.len() {
            return None;
        }
        return Some(SolverRequest {
            command: format!(
                "StepByStep[LinearRegression[{{{}}}, {{{}}}]]",
                xs.join(", "),
                ys.join(", ")
            ),
            label: "Regresión".to_string(),
            steps: true,
            graph_expr: None,
        });
    }
    let datos = listas.first()?;
    let op = if tiene(&["mediana"]) {
        "Median"
    } else if tiene(&["media", "promedio"]) {
        "Mean"
    } else if tiene(&["varianza", "variancia"]) {
        "Variance"
    } else if tiene(&["desvio", "desviacion"]) {
        "SD"
    } else {
        return None;
    };
    let etiqueta = match op {
        "Median" => "Mediana",
        "Variance" => "Varianza",
        "SD" => "Desvío",
        _ => "Media",
    };
    Some(SolverRequest {
        command: format!("StepByStep[{op}[{{{}}}]]", datos.join(", ")),
        label: etiqueta.to_string(),
        steps: true,
        graph_expr: None,
    })
}
fn extract_numbers(s: &str) -> Vec<String> {
    s.split_whitespace()
        .map(|tok| {
            tok.trim_matches(|c: char| !c.is_ascii_digit() && c != '-' && c != '.' && c != '+')
        })
        .filter(|tok| {
            !tok.is_empty()
                && tok.chars().any(|c| c.is_ascii_digit())
                && tok.parse::<f64>().is_ok_and(|v| v.is_finite())
        })
        .map(str::to_string)
        .collect()
}

/// Límites `entre A y B` / `de A a B` / `desde A hasta B` del pedido
/// normalizado. `None` si no hay dos números finitos: no se adivina, el
/// llamador integra indefinido.
fn parse_definite_bounds(p: &str) -> Option<(String, String)> {
    if let Some(rest) = p.split("entre").nth(1) {
        let nums = extract_numbers(rest);
        if nums.len() >= 2 {
            return Some((nums[0].clone(), nums[1].clone()));
        }
        return None;
    }
    if let Some(rest) = p.split("desde").nth(1) {
        let (lo, hi) = match rest.split_once("hasta") {
            Some((a, b)) => (a, b),
            None => return None,
        };
        let (a, b) = (extract_numbers(lo), extract_numbers(hi));
        if let (Some(a), Some(b)) = (a.into_iter().next(), b.into_iter().next()) {
            return Some((a, b));
        }
        return None;
    }
    if let Some(rest) = p.split(" de ").nth(1) {
        // `de A a B`: corta en la última ` a ` para no comerse la `a` de `para`.
        let (lo, hi) = match rest.rsplit_once(" a ") {
            Some((a, b)) => (a, b),
            None => return None,
        };
        let (a, b) = (extract_numbers(lo), extract_numbers(hi));
        if let (Some(a), Some(b)) = (a.into_iter().next(), b.into_iter().next()) {
            return Some((a, b));
        }
    }
    None
}

/// Variable: `respecto a t` / `en x` / `variable y`, default `x`.
fn detect_var(p: &str) -> String {
    for key in [
        "respecto a ",
        "respecto de ",
        "variable ",
        "en la variable ",
    ] {
        if let Some(rest) = p.split(key).nth(1) {
            if let Some(ch) = rest.trim().chars().next() {
                if ch.is_ascii_alphabetic() {
                    return ch.to_string();
                }
            }
        }
    }
    "x".to_string()
}

/// ¿Pide gráfico explícito?
fn wants_graph(p: &str) -> bool {
    p.contains("grafic") || p.contains("dibuja") || p.contains("plot")
}

/// Extrae la expresión matemática: el token con más densidad matemática
/// (robusto a prosa y a restos como `x 0` del límite). `None` si no hay nada.
fn extract_expr(raw: &str) -> Option<String> {
    let cleaned = raw
        .trim()
        .trim_matches(|c| {
            matches!(
                c,
                '"' | '\'' | '¿' | '?' | '¡' | '!' | '.' | ':' | ';' | ','
            )
        })
        .to_string();
    if cleaned.is_empty() {
        return None;
    }
    let mut best: Option<(i32, String)> = None;
    for tok in cleaned.split_whitespace() {
        let t = tok.trim_matches(|c: char| {
            matches!(
                c,
                '"' | '\'' | '¿' | '?' | '¡' | '!' | '.' | ':' | ';' | ','
            )
        });
        if t.len() < 2 || t.len() > 200 {
            // Tokens de 1 letra (`x`, `t`, `0`) son var o `at`, no la expr.
            continue;
        }
        let mut score = 0i32;
        for c in t.chars() {
            match c {
                '^' | '=' => score += 3,
                '(' | ')' | '/' | '*' => score += 2,
                c if c.is_ascii_digit() => score += 2,
                '+' | '-' | '_' => score += 1,
                _ => {}
            }
        }
        if t.contains("sin")
            || t.contains("cos")
            || t.contains("tan")
            || t.contains("log")
            || t.contains("sqrt")
        {
            score += 3;
        }
        if score >= 3 {
            let better = match &best {
                Some((bs, bt)) => score > *bs || (score == *bs && t.len() > bt.len()),
                None => true,
            };
            if better {
                best = Some((score, t.to_string()));
            }
        }
    }
    best.map(|(_, t)| t)
}

/// Parsea un pedido en lenguaje natural al solver. `None` = no es pedido
/// de solver (sigue el flujo normal del asistente).
///
/// Dispara solo con intención explícita (`paso a paso`, `resolv…`,
/// `deriv…`, `integr…`, `límite/limit`, `taylor`, `ecuación/despej…`,
/// `grafica…`) + expresión con matemática. Sin keyword no hay solver
/// (evita secuestrar charla normal).
// GroebnerOrdered/Eliminate no se mapean desde NL: el orden monomial y la
// variable a eliminar son decisiones; adivinarlas inventaría el resultado.
#[must_use]
pub fn parse_solver_nl(prompt: &str) -> Option<SolverRequest> {
    let p = norm(prompt);
    if p.len() > 500 {
        return None;
    }
    let has_steps = p.contains("paso a paso") || p.contains("paso por paso");
    let wants_deriv = p.contains("deriv");
    let wants_integ = p.contains("integr");
    let wants_limit = p.contains("limit") || p.contains("limite");
    let wants_taylor = p.contains("taylor");
    let wants_solve = p.contains("resolv")
        || p.contains("resuelv")
        || p.contains("ecuacion")
        || p.contains("despej")
        || p.contains("solve");
    // LaplaceInt directo; `laplace + deriv` (LaplaceDeriv con iniciales) no
    // se adivina: se deja al redirect con plantilla exacta (assistant.rs).
    let wants_laplace = p.contains("laplace") && !wants_deriv;
    let wants_simplify = p.contains("simplific")
        || p.contains("factoriz")
        || p.contains("factorea")
        || p.contains("expand")
        || p.contains("desarroll");
    let wants_system = p.contains("sistema");
    let wants_graph_only = wants_graph(&p);
    // Stats con lista explícita (`media de [1,2,3]`): StepByStep directo.
    // Cuantil/distribución no se parsea acá: `None` → redirect honesto.
    let wants_stats = p.split_whitespace().map(stats_token).any(|t| {
        matches!(
            t,
            "media"
                | "promedio"
                | "mediana"
                | "varianza"
                | "variancia"
                | "desvio"
                | "desviacion"
                | "regresion"
                | "ajuste"
        )
    });
    if !(has_steps
        || wants_deriv
        || wants_integ
        || wants_limit
        || wants_taylor
        || wants_solve
        || wants_laplace
        || wants_simplify
        || wants_system
        || wants_stats
        || wants_graph_only)
    {
        return None;
    }
    // Sistemas 2×2 con pasos: si hay keyword `sistema` no se adivina nada
    // más (ni Solve de una sola ecuación): 2 ecuaciones o `None` honesto.
    if wants_system {
        return try_parse_system(&p);
    }
    // EDO lineal de 1er orden con ecuación explícita (`y' + 2y = 0`):
    // StepByStep directo sin redirect; ambigua o no lineal → None.
    if (p.contains("y'") || p.contains('′')) && p.contains('=') {
        return try_parse_ode_first_order(&p);
    }
    // Estadística con lista explícita: StepByStep directo sin redirect;
    // sin lista numérica → None (jamás inventar datos).
    if wants_stats {
        return try_parse_stats(&p);
    }
    // Quita el verbo gráfico para no contaminar la expresión, pero recuerda graficar.
    let graph = wants_graph_only;
    let core = strip_ask(&p);
    let expr = extract_expr(&core)?;
    if expr.len() < 2 || expr.len() > 200 {
        return None;
    }
    let var = detect_var(&p);
    // Clasifica la operación (orden: laplace > taylor > limit > integral >
    // derivada > simplificación directa > solve).
    let (op, label, steps): (String, String, bool) = if wants_laplace {
        (
            format!("LaplaceInt[{expr}, t, s]"),
            "Laplace".to_string(),
            true,
        )
    } else if wants_taylor {
        (
            format!("Taylor[{expr}, {var}, 0, 5]"),
            "Taylor".to_string(),
            true,
        )
    } else if wants_limit {
        // `límite de X cuando y→N`: extrae N si hay `→`/`tiende a`/`cuando`.
        let at = parse_limit_at(&p).unwrap_or("0".to_string());
        (
            format!("Limit[{expr}, {var}, {at}]"),
            "Límite".to_string(),
            true,
        )
    } else if wants_integ {
        match parse_definite_bounds(&p) {
            Some((a, b)) => (
                format!("Integral[{expr}, {var}, {a}, {b}]"),
                "Integral definida".to_string(),
                true,
            ),
            None => (
                format!("Integral[{expr}, {var}]"),
                "Integral".to_string(),
                true,
            ),
        }
    } else if wants_deriv {
        (
            format!("Derivative[{expr}, {var}]"),
            "Derivada".to_string(),
            true,
        )
    } else if expr.contains('=') || wants_solve {
        (
            format!("Solve[{expr}, {var}]"),
            "Ecuación".to_string(),
            true,
        )
    } else if wants_simplify {
        // Directa honesta: el motor no tiene CasOp de simplificación con
        // pasos; se ejecuta `Simplify`/`Factor`/`Expand` sin inventar pasos.
        let (cmd, lab) = if p.contains("factoriz") || p.contains("factorea") {
            ("Factor", "Factorización")
        } else if p.contains("expand") || p.contains("desarroll") {
            ("Expand", "Desarrollo")
        } else {
            ("Simplify", "Simplificación")
        };
        (format!("{cmd}[{expr}]"), lab.to_string(), false)
    } else if has_steps || graph {
        // `paso a paso` sin verbo: ecuación si hay `=`, si no derivada.
        if expr.contains('=') {
            (
                format!("Solve[{expr}, {var}]"),
                "Ecuación".to_string(),
                true,
            )
        } else {
            (
                format!("Derivative[{expr}, {var}]"),
                "Derivada".to_string(),
                true,
            )
        }
    } else if graph {
        (
            format!("Solve[{expr}, {var}]"),
            "Ecuación".to_string(),
            true,
        )
    } else {
        return None;
    };
    // Gráfico: funciones explícitas (sin `=`) o `y = f(x)`.
    let graph_expr = if graph || (!expr.contains('=') && is_plottable(&expr)) {
        Some(graph_target(&expr))
    } else if expr.contains('=') {
        solve_rhs_for_graph(&expr)
    } else {
        None
    };
    Some(SolverRequest {
        command: if steps {
            format!("StepByStep[{op}]")
        } else {
            op
        },
        label,
        steps,
        graph_expr,
    })
}

/// `cuando x tiende a 2` / `x→2` / `en 2` → `2`.
fn parse_limit_at(p: &str) -> Option<String> {
    for key in ["tiende a ", "tiende al ", "cuando ", "en "] {
        if let Some(rest) = p.split(key).nth(1) {
            let tok = rest
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_matches(|c: char| !c.is_ascii_digit() && c != '-' && c != '.' && c != '+');
            if !tok.is_empty() && tok.chars().any(|c| c.is_ascii_digit()) {
                return Some(tok.to_string());
            }
        }
    }
    // Flecha unicode `→2`.
    if let Some(i) = p.find('→') {
        let tok: String = p[i + '→'.len_utf8()..]
            .chars()
            .take_while(|c| c.is_ascii_digit() || matches!(c, '-' | '.' | '+'))
            .collect();
        if !tok.is_empty() {
            return Some(tok);
        }
    }
    None
}

/// ¿Se puede graficar? Letras + dígitos + operadores, sin `;` ni llaves.
fn is_plottable(expr: &str) -> bool {
    !expr.is_empty()
        && expr.len() <= 120
        && !expr.contains([';', '{', '}', '\n'])
        && expr.chars().any(|c| c == 'x' || c.is_ascii_digit())
}

/// `y = x^2` → `x^2`; `x^2-4=0` → `x^2-4` (LHS para ver raíces);
/// `x^2` → `x^2`.
fn graph_target(expr: &str) -> String {
    match expr.split_once('=') {
        Some((lhs, rhs)) => {
            let (l, r) = (lhs.trim(), rhs.trim());
            if l == "y" || l == "f(x)" || l == "f" {
                r.to_string()
            } else if r == "y" || r == "f(x)" || r == "f" {
                l.to_string()
            } else if r == "0" && is_plottable(l) {
                // Ecuación `f(x)=0`: se grafica `f` para ver las raíces.
                l.to_string()
            } else {
                expr.to_string()
            }
        }
        None => expr.to_string(),
    }
}

/// `x^2+3x=0` → intenta graficar el LHS (`x^2+3x`).
fn solve_rhs_for_graph(expr: &str) -> Option<String> {
    let (lhs, _) = expr.split_once('=')?;
    let lhs = lhs.trim();
    if is_plottable(lhs) {
        Some(lhs.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derivada_con_tilde_y_doble_espacio() {
        let r = parse_solver_nl("derivada   de x^2  paso a paso").expect("solver");
        assert_eq!(r.command, "StepByStep[Derivative[x^2, x]]");
        assert_eq!(r.label, "Derivada");
        assert_eq!(r.graph_expr.as_deref(), Some("x^2"));
    }

    #[test]
    fn resuelve_ecuacion_con_acentos() {
        let r = parse_solver_nl("resolvé x^2+3x=0 paso a paso").expect("solver");
        assert_eq!(r.command, "StepByStep[Solve[x^2+3x=0, x]]");
        assert_eq!(r.label, "Ecuación");
    }

    #[test]
    fn superindices_del_teclado_movil_se_transcriben() {
        // El teclado móvil escribe x² (U+00B2): el CAS espera x^2. Sin el
        // transcode el pedido crudo llegaba al asistente y fallaba como
        // "error de conexión".
        let r = parse_solver_nl("resolvé x²+3x=0 paso a paso").expect("solver");
        assert_eq!(r.command, "StepByStep[Solve[x^2+3x=0, x]]");
        let g = parse_solver_nl("grafica y=x²").expect("solver");
        assert_eq!(g.graph_expr.as_deref(), Some("x^2"));
    }

    #[test]
    fn integrar_sin_pasos_explicitos() {
        let r = parse_solver_nl("integrar sin(x)").expect("solver");
        assert_eq!(r.command, "StepByStep[Integral[sin(x), x]]");
    }

    #[test]
    fn limite_con_tiende_a() {
        let r =
            parse_solver_nl("límite de sin(x)/x cuando x tiende a 0 paso a paso").expect("solver");
        assert_eq!(r.command, "StepByStep[Limit[sin(x)/x, x, 0]]");
        assert_eq!(r.label, "Límite");
    }

    #[test]
    fn charla_normal_no_secuestra() {
        assert!(parse_solver_nl("hola, ¿cómo estás?").is_none());
        assert!(parse_solver_nl("qué es una derivada").is_none());
        assert!(parse_solver_nl("gracias").is_none());
    }

    #[test]
    fn variable_respecto_a_t() {
        let r = parse_solver_nl("derivada de t^3 respecto a t paso a paso").expect("solver");
        assert_eq!(r.command, "StepByStep[Derivative[t^3, t]]");
    }

    #[test]
    fn grafica_pide_plot() {
        let r = parse_solver_nl("resolvé x^2-4=0 y grafica").expect("solver");
        assert_eq!(r.graph_expr.as_deref(), Some("x^2-4"));
    }

    #[test]
    fn laplace_de_seno() {
        let r = parse_solver_nl("calculá la laplace de sin(t) paso a paso").expect("solver");
        assert_eq!(r.command, "StepByStep[LaplaceInt[sin(t), t, s]]");
        assert_eq!(r.label, "Laplace");
        assert!(r.steps);
    }

    #[test]
    fn laplace_mas_derivada_no_adivina() {
        // LaplaceDeriv pide orden + iniciales: no se inventa, da None y
        // el redirect del asistente da la plantilla exacta.
        assert!(parse_solver_nl("laplace de la derivada de y").is_none());
    }

    #[test]
    fn simplifica_directo_sin_pasos() {
        let r = parse_solver_nl("simplificá (x+1)^2").expect("solver");
        assert_eq!(r.command, "Simplify[(x+1)^2]");
        assert_eq!(r.label, "Simplificación");
        assert!(!r.steps);
    }

    #[test]
    fn factoriza_y_expande() {
        let r = parse_solver_nl("factorizá x^2-4").expect("solver");
        assert_eq!(r.command, "Factor[x^2-4]");
        assert!(!r.steps);
        let r = parse_solver_nl("desarrollá (x+1)^2").expect("solver");
        assert_eq!(r.command, "Expand[(x+1)^2]");
        assert_eq!(r.label, "Desarrollo");
    }

    #[test]
    fn factorial_no_secuestra_como_factor() {
        assert!(parse_solver_nl("cuánto es el factorial de 5").is_none());
    }

    #[test]
    fn euler_metodo_numerico_no_da_plantilla_eulerode() {
        // El método numérico de Euler lo cubre el redirect honesto de
        // `assistant.rs` (no soportado); el NL nunca inventa EulerODE.
        for prompt in [
            "explica el metodo de euler paso a paso",
            "metodo de euler con x^2 paso a paso",
        ] {
            match parse_solver_nl(prompt) {
                None => {}
                Some(r) => {
                    assert!(!r.command.contains("EulerODE"), "{r:?}");
                    assert!(!r.command.contains("FrobeniusSeries"), "{r:?}");
                }
            }
        }
    }

    #[test]
    fn euler_equidimensional_lo_cubre_redirect_no_solver() {
        // La forma x²y″+a·x·y′+b·y=rhs pide coeficientes explícitos: el NL
        // no adivina EulerODE, lo da el redirect con plantilla exacta.
        for prompt in [
            "ecuacion equidimensional de euler paso a paso",
            "euler cauchy x^2 y'' paso a paso",
        ] {
            match parse_solver_nl(prompt) {
                None => {}
                Some(r) => assert!(!r.command.contains("EulerODE"), "{r:?}"),
            }
        }
    }

    #[test]
    fn frobenius_lo_cubre_redirect_no_solver() {
        // p y q son explícitos en el motor; el NL no inventa FrobeniusSeries.
        for prompt in [
            "serie de frobenius paso a paso",
            "frobenius con p=-2*x paso a paso",
        ] {
            match parse_solver_nl(prompt) {
                None => {}
                Some(r) => assert!(!r.command.contains("FrobeniusSeries"), "{r:?}"),
            }
        }
    }

    #[test]
    fn pedidos_normales_no_caen_en_euler_frobenius() {
        let r = parse_solver_nl("derivada de x^2 paso a paso").expect("solver");
        assert_eq!(r.command, "StepByStep[Derivative[x^2, x]]");
        let r = parse_solver_nl("factorizá x^2-4").expect("solver");
        assert_eq!(r.command, "Factor[x^2-4]");
    }

    #[test]
    fn sistema_2x2_con_pasos_y_grafico() {
        let r = parse_solver_nl("resolvé el sistema x+y=3, x-y=1 paso a paso").expect("solver");
        assert_eq!(r.command, "StepByStep[SolveNlSystem[x+y=3, x-y=1, x, y]]");
        assert_eq!(r.label, "Sistema");
        assert!(r.steps);
        assert_eq!(r.graph_expr.as_deref(), Some("3-x"));
    }

    #[test]
    fn sistema_sin_coma_y_con_coeficientes() {
        let r = parse_solver_nl("sistema 2*x+3*y=7, x-y=1 paso a paso").expect("solver");
        assert_eq!(
            r.command,
            "StepByStep[SolveNlSystem[2*x+3*y=7, x-y=1, x, y]]"
        );
        assert_eq!(r.label, "Sistema");
        assert!(r.steps);
        assert!(r.graph_expr.as_deref().is_some());
    }

    #[test]
    fn sistema_que_no_es_2x2_no_adivina() {
        // 3 ecuaciones: `None` honesto (el redirect pide las 2 ecuaciones).
        assert!(parse_solver_nl("resolvé el sistema x+y=1, x-y=0, 2*x+y=3").is_none());
        // 1 sola ecuación con keyword: tampoco se adivina la otra.
        assert!(parse_solver_nl("resolvé el sistema x+y=3 paso a paso").is_none());
        // Sin matemática no hay solver.
        assert!(parse_solver_nl("qué es un sistema de ecuaciones").is_none());
    }

    #[test]
    fn edo_1er_orden_lineal_directa_a_step_by_step() {
        // `y' + 2y = 0` va directo a pasos, sin redirect.
        let r = parse_solver_nl("resuelve y' + 2y = 0").expect("solver EDO");
        assert_eq!(r.command, "StepByStep[SolveODE1[2, 0, x]]");
        assert_eq!(r.label, "EDO lineal");
        assert!(r.steps);
        assert!(r.graph_expr.is_none());
        // Ecuación pegada y forma `y' = …` también leen P y Q.
        let r = parse_solver_nl("resolvé y'=2*y paso a paso").expect("solver EDO");
        assert_eq!(r.command, "StepByStep[SolveODE1[-2, 0, x]]");
        let r = parse_solver_nl("resolvé y' - 2*y = sin(x) paso a paso").expect("Q(x)");
        assert_eq!(r.command, "StepByStep[SolveODE1[-2, sin(x), x]]");
    }

    #[test]
    fn edo_no_lineal_o_2do_orden_no_adivina() {
        // No lineal en y: `None` (deriva al redirect honesto).
        assert!(parse_solver_nl("resuelve y*y' = x paso a paso").is_none());
        assert!(parse_solver_nl("resuelve y' + y^2 = 0").is_none());
        // 2º orden: `None` (el redirect pide coeficientes / Laplace).
        assert!(parse_solver_nl("resolvé la ecuación diferencial y''+y=0").is_none());
        // Sin `=`: no hay ecuación que leer.
        assert!(parse_solver_nl("resuelve y' + 2y").is_none());
    }

    #[test]
    fn integral_definida_entre_a_y_b() {
        let r = parse_solver_nl("integra x^2 entre 0 y 1 paso a paso").expect("solver");
        assert_eq!(r.command, "StepByStep[Integral[x^2, x, 0, 1]]");
        assert_eq!(r.label, "Integral definida");
        assert!(r.steps);
        assert_eq!(r.graph_expr.as_deref(), Some("x^2"));
    }

    #[test]
    fn integral_definida_de_a_hasta_b() {
        let r = parse_solver_nl("integrá sin(x) de 0 a 3.14").expect("solver");
        assert_eq!(r.command, "StepByStep[Integral[sin(x), x, 0, 3.14]]");
        assert_eq!(r.label, "Integral definida");
    }

    #[test]
    fn integral_sin_cotas_sigue_indefinida() {
        // Sin `entre/de`: indefinida como hoy, sin límites inventados.
        let r = parse_solver_nl("integrar x^2 paso a paso").expect("solver");
        assert_eq!(r.command, "StepByStep[Integral[x^2, x]]");
        assert_eq!(r.label, "Integral");
    }

    #[test]
    fn stats_descriptiva_con_lista_directa_a_step_by_step() {
        // `'media de [1,2,3]'` → StepByStep directo (patrón EDO).
        let r = parse_solver_nl("media de [1,2,3]").expect("solver stats");
        assert_eq!(r.command, "StepByStep[Mean[{1, 2, 3}]]");
        assert_eq!(r.label, "Media");
        assert!(r.steps);
        assert!(r.graph_expr.is_none());
        // Llaves, tildes y sinónimos también leen la lista.
        let r = parse_solver_nl("mediana de {2, 1, 3} paso a paso").expect("solver");
        assert_eq!(r.command, "StepByStep[Median[{2, 1, 3}]]");
        let r = parse_solver_nl("varianza de [1,2,3,4]").expect("solver");
        assert_eq!(r.command, "StepByStep[Variance[{1, 2, 3, 4}]]");
        let r = parse_solver_nl("desvío estándar de [1,2,3,4]").expect("solver");
        assert_eq!(r.command, "StepByStep[SD[{1, 2, 3, 4}]]");
    }

    #[test]
    fn stats_regresion_con_dos_listas_directa_a_step_by_step() {
        let r = parse_solver_nl("regresión de [1,2,3] [2,4,6]").expect("solver");
        assert_eq!(
            r.command,
            "StepByStep[LinearRegression[{1, 2, 3}, {2, 4, 6}]]"
        );
        assert_eq!(r.label, "Regresión");
        assert!(r.steps);
    }

    #[test]
    fn stats_sin_lista_o_fuera_de_subset_no_adivina() {
        // Sin lista numérica: `None` (jamás inventar datos).
        assert!(parse_solver_nl("calculá la media paso a paso").is_none());
        // Largos distintos: `None` (el error honesto vive en el motor).
        assert!(parse_solver_nl("regresion de [1,2] [1,2,3]").is_none());
        // Cuantil/distribución: `None` (redirect honesto del asistente).
        assert!(parse_solver_nl("cuantil 90 de [1,2,3]").is_none());
        assert!(parse_solver_nl("distribución normal paso a paso").is_none());
    }
}
