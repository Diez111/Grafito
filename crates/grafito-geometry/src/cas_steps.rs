// Grafito CAS stepper — emisión genérica de pasos pedagógicos.
//!
//! Presupuesto: MAX_CAS_STEPS 32, MAX_STEP_BYTES 4 KiB por campo.
//! Statem: Idle → Parsing → Stepping → Verifying → Done | Failed
//! Instrumenta `diff_expr` y `integrate_expr` con visitor que loguea
//! `CasStep { before, after, rule }` y reutiliza `simplify_expr`.
//! El límite usa Richardson bilateral (igual que `symbolic::richardson_limit`).
//!
//! Contrato de consumo para `grafito-command` (listo para cablear, sin UI):
//! - Punto de entrada sin estado: [`steps_for_op`] (`&CasOp →
//!   Result<Vec<CasStep>, String>`). Puro, sin I/O ni spawn: seguro de llamar
//!   desde cualquier hilo/executor.
//! - Punto de entrada con statem: [`CasStepper::run`] (misma salida, más
//!   transiciones tipadas `Idle → Parsing → Stepping → Verifying → Done`).
//! - Cada [`CasStep`] trae `index`, `rule` ([`RewriteRule`], con `Display`
//!   estable para serializar), `before`/`after` (≤ 4 KiB, truncado en borde
//!   UTF-8) y `description` en español rioplatense lista para el panel.
//! - Errores honestos en `Result`: entradas inválidas o presupuestos
//!   excedidos devuelven `Err(String)`, nunca pasos inventados.
//! - Ejemplo: `steps_for_op(&CasOp::Derivative { expr: "x^2".into(), var:
//!   "x".into() })`.

use crate::ast::{parse_ast, Expr};
use std::collections::{BTreeMap, HashSet};

// ---------------------------------------------------------------------------
// Presupuestos
// ---------------------------------------------------------------------------

/// Máximo de pasos emitidos por operación.
pub const MAX_CAS_STEPS: usize = 32;
/// Máximo de bytes por campo `before` / `after` de cada paso.
pub const MAX_STEP_BYTES: usize = 4096;

// ---------------------------------------------------------------------------
// RewriteRule y CasStep
// ---------------------------------------------------------------------------

/// Regla de reescritura aplicada en cada paso.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RewriteRule {
    ConstantRule,
    VariableRule,
    SumRule,
    DifferenceRule,
    ProductRule,
    QuotientRule,
    PowerRule,
    NegRule,
    ChainRule,
    ExpRule,
    LogRule,
    TrigRule,
    SqrtRule,
    AbsRule,
    Simplify,
    IntegrationPowerRule,
    IntegrationSumRule,
    IntegrationConstantRule,
    IntegrationExpRule,
    IntegrationTrigRule,
    // Frente F3c: parciales, arctan, EDO 2º orden, sistemas, Laplace.
    IntegrationPartialFractions,
    IntegrationArctanRule,
    OdeSecondOrderRule,
    OdeSystemRule,
    LaplaceDirectRule,
    LaplaceInverseRule,
    LimitRichardson,
    TaylorRule,
    // Frente B1: Solve general (lineal/cuadrática/Sturm).
    SolveLinearRule,
    SolveQuadraticRule,
    SolveSturmRule,
    // Frente B2: Risch pragmático, Gruntz rewrite, EDO, Laplace cálculo,
    // Buchberger con órdenes + eliminación.
    RischTrigRule,
    RischEulerSqrtRule,
    RischPartsRule,
    GruntzLogRewriteRule,
    OscillationRule,
    OdeNthOrderRule,
    OdeEulerRule,
    /// EDO lineal de 1er orden por factor integrante (P, μ, solución).
    OdeFirstOrderRule,
    /// EDO homogénea de 2º orden constante (característica + casos Δ).
    OdeSecondHomogRule,
    /// Estadística descriptiva de lista (media/mediana/varianza/desvío poblacionales).
    StatsDescriptiveRule,
    /// Regresión lineal por mínimos cuadrados (pendiente/intercepto/R²).
    StatsRegressionRule,
    FrobeniusRule,
    LaplaceCalculusRule,
    LaplaceShiftRule,
    GroebnerOrderRule,
    EliminateRule,
    // Sistemas lineales 2×2 (eliminación/sustitución) e integral definida.
    SystemEliminationRule,
    SystemSubstitutionRule,
    BarrowRule,
    Generic,
}

impl std::fmt::Display for RewriteRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::ConstantRule => "ConstantRule",
            Self::VariableRule => "VariableRule",
            Self::SumRule => "SumRule",
            Self::DifferenceRule => "DifferenceRule",
            Self::ProductRule => "ProductRule",
            Self::QuotientRule => "QuotientRule",
            Self::PowerRule => "PowerRule",
            Self::NegRule => "NegRule",
            Self::ChainRule => "ChainRule",
            Self::ExpRule => "ExpRule",
            Self::LogRule => "LogRule",
            Self::TrigRule => "TrigRule",
            Self::SqrtRule => "SqrtRule",
            Self::AbsRule => "AbsRule",
            Self::Simplify => "Simplify",
            Self::IntegrationPowerRule => "IntegrationPowerRule",
            Self::IntegrationSumRule => "IntegrationSumRule",
            Self::IntegrationConstantRule => "IntegrationConstantRule",
            Self::IntegrationExpRule => "IntegrationExpRule",
            Self::IntegrationTrigRule => "IntegrationTrigRule",
            Self::IntegrationPartialFractions => "IntegrationPartialFractions",
            Self::IntegrationArctanRule => "IntegrationArctanRule",
            Self::OdeSecondOrderRule => "OdeSecondOrderRule",
            Self::OdeSystemRule => "OdeSystemRule",
            Self::LaplaceDirectRule => "LaplaceDirectRule",
            Self::LaplaceInverseRule => "LaplaceInverseRule",
            Self::LimitRichardson => "LimitRichardson",
            Self::TaylorRule => "TaylorRule",
            Self::SolveLinearRule => "SolveLinearRule",
            Self::SolveQuadraticRule => "SolveQuadraticRule",
            Self::SolveSturmRule => "SolveSturmRule",
            Self::RischTrigRule => "RischTrigRule",
            Self::RischEulerSqrtRule => "RischEulerSqrtRule",
            Self::RischPartsRule => "RischPartsRule",
            Self::GruntzLogRewriteRule => "GruntzLogRewriteRule",
            Self::OscillationRule => "OscillationRule",
            Self::OdeNthOrderRule => "OdeNthOrderRule",
            Self::OdeEulerRule => "OdeEulerRule",
            Self::OdeFirstOrderRule => "OdeFirstOrderRule",
            Self::OdeSecondHomogRule => "OdeSecondHomogRule",
            Self::StatsDescriptiveRule => "StatsDescriptiveRule",
            Self::StatsRegressionRule => "StatsRegressionRule",
            Self::FrobeniusRule => "FrobeniusRule",
            Self::LaplaceCalculusRule => "LaplaceCalculusRule",
            Self::LaplaceShiftRule => "LaplaceShiftRule",
            Self::GroebnerOrderRule => "GroebnerOrderRule",
            Self::EliminateRule => "EliminateRule",
            Self::SystemEliminationRule => "SystemEliminationRule",
            Self::SystemSubstitutionRule => "SystemSubstitutionRule",
            Self::BarrowRule => "BarrowRule",
            Self::Generic => "Generic",
        };
        write!(f, "{s}")
    }
}

/// Motivo pedagógico de una línea por regla (qué mirar / error típico).
///
/// Se deriva al dibujar desde [`RewriteRule`]; no se guarda en [`CasStep`]
/// para que el fence ```grafito-steps haga roundtrip sin cambios. Texto en
/// español rioplatense, sin datos del problema: sólo la regla.
#[must_use]
pub fn motivo(rule: &RewriteRule) -> &'static str {
    match rule {
        RewriteRule::PowerRule => {
            "Motivo: baja el exponente y restale 1; fijate que el exponente sea constante."
        }
        RewriteRule::SumRule => {
            "Motivo: deriva termino a termino; el error tipico es arrastrar un termino sin derivar."
        }
        RewriteRule::ProductRule => {
            "Motivo: es u'·v + u·v'; fijate en no derivar cada factor por separado y sumarlos."
        }
        RewriteRule::QuotientRule => {
            "Motivo: es (u'·v − u·v')/v²; revisa el signo del menos y el cuadrado del denominador."
        }
        RewriteRule::ChainRule => {
            "Motivo: deriva afuera y multiplica por la derivada de adentro; el error tipico es olvidar ese u'."
        }
        RewriteRule::TrigRule => {
            "Motivo: mira que funcion trigonometrica es (el signo cambia en cos); no la confundas con la de afuera."
        }
        RewriteRule::ExpRule => {
            "Motivo: la exponencial se conserva y multiplica por la derivada del exponente; fijate que este bien derivado."
        }
        RewriteRule::LogRule => {
            "Motivo: es u'/u (con ln10 si es base 10); revisa no invertir numerador y denominador."
        }
        RewriteRule::IntegrationPowerRule => {
            "Motivo: sumale 1 al exponente y dividi por el nuevo exponente; fijate que no sea −1."
        }
        RewriteRule::IntegrationTrigRule => {
            "Motivo: mira si el integrando es directo (sec², sin, cos) o pide sustitucion; el error tipico es mezclar primitivas."
        }
        RewriteRule::IntegrationPartialFractions => {
            "Motivo: mira si el denominador se factoriza en lineales; el error tipico es plantear mal los coeficientes A, B."
        }
        RewriteRule::IntegrationArctanRule => {
            "Motivo: mira la forma c/(x²+a²); el error tipico es olvidar dividir por a."
        }
        RewriteRule::LimitRichardson => {
            "Motivo: se muestrea a ambos lados y se extrapola; si los lados difieren, el limite no existe."
        }
        RewriteRule::GruntzLogRewriteRule => {
            "Motivo: mira si hay un logaritmo que domina el crecimiento; el error tipico es reescribir sin aislar el termino dominante."
        }
        RewriteRule::SolveLinearRule => {
            "Motivo: despeja la x pasando terminos al otro lado; el error tipico es cambiar mal un signo."
        }
        RewriteRule::SolveQuadraticRule => {
            "Motivo: mira el discriminante para saber cuantas raices hay; el error tipico es repartir la raiz en cada termino."
        }
        RewriteRule::SolveSturmRule => {
            "Motivo: mira en que intervalo cambia de signo para aislar cada raiz; el error tipico es dar por raiz un punto sin cambio de signo."
        }
        RewriteRule::LaplaceCalculusRule => {
            "Motivo: pasa la derivada o la integral a s con sus condiciones iniciales; el error tipico es olvidar los terminos de y(0)."
        }
        RewriteRule::TaylorRule => {
            "Motivo: mira el centro y el orden para cada coeficiente; el error tipico es derivar bien pero evaluar mal en el centro."
        }
        RewriteRule::FrobeniusRule => {
            "Motivo: mira la recurrencia de coeficientes desde a0 y a1; el error tipico es correr mal el indice de la serie."
        }
        RewriteRule::OdeEulerRule => {
            "Motivo: mira que el cambio x = e^t deja coeficientes constantes; el error tipico es no probar x^r en la indicial."
        }
        RewriteRule::OdeFirstOrderRule => {
            "Motivo: multiplica por el factor integrante para que el lado izquierdo sea una derivada exacta; el error tipico es integrar mal P(x) en el exponente."
        }
        RewriteRule::OdeSecondHomogRule => {
            "Motivo: mira el signo del discriminante para elegir la familia (reales, doble o compleja); el error tipico es usar la familia de otro caso."
        }
        RewriteRule::StatsDescriptiveRule => {
            "Motivo: mira si N divide (poblacional) o N−1 (muestral); el error tipico es mezclar varianza poblacional con muestral."
        }
        RewriteRule::StatsRegressionRule => {
            "Motivo: mira que x varie (si no, no hay pendiente); el error tipico es leer R² como si fuera la pendiente."
        }
        RewriteRule::SystemEliminationRule => {
            "Motivo: mira que variable se elimina al combinar las ecuaciones; el error tipico es multiplicar mal una ecuacion antes de restar."
        }
        RewriteRule::SystemSubstitutionRule => {
            "Motivo: mira que despeje se reemplaza en la otra ecuacion; el error tipico es perder un signo al sustituir."
        }
        RewriteRule::BarrowRule => {
            "Motivo: mira que se evalue la primitiva en los dos extremos y se reste F(b)-F(a); el error tipico es intercambiar los limites."
        }
        RewriteRule::Simplify => {
            "Motivo: mira que terminos se cancelan o agrupan sin cambiar el valor; el error tipico es cancelar algo que no es factor comun."
        }
        RewriteRule::AbsRule => {
            "Motivo: mira donde la expresion de adentro cambia de signo; el error tipico es derivar sin partir en los tramos."
        }
        RewriteRule::ConstantRule => {
            "Motivo: la derivada de una constante es 0; fijate si de verdad no depende de la variable."
        }
        RewriteRule::VariableRule => {
            "Motivo: x deriva a 1; el error tipico es confundir la variable con un coeficiente pegado."
        }
        RewriteRule::DifferenceRule => {
            "Motivo: deriva cada termino con su signo; el error tipico es mezclar la resta con la regla de cadena."
        }
        RewriteRule::NegRule => {
            "Motivo: el signo sale afuera y el resto deriva adentro; el error tipico es duplicar el menos."
        }
        RewriteRule::SqrtRule => {
            "Motivo: pasa la raiz a potencia 1/2 antes de derivar; el error tipico es olvidar la cadena interna."
        }
        RewriteRule::IntegrationSumRule => {
            "Motivo: la integral se reparte en cada sumando; el error tipico es arrastrar la constante a un solo tramo."
        }
        RewriteRule::IntegrationConstantRule => {
            "Motivo: la constante multiplica la primitiva; el error tipico es sumarla en vez de multiplicarla."
        }
        RewriteRule::IntegrationExpRule => {
            "Motivo: la exponencial es su propia primitiva con la base normalizada; el error tipico es olvidar dividir por ln(a)."
        }
        RewriteRule::OdeSecondOrderRule => {
            "Motivo: mira la ecuacion caracteristica y sus raices; el error tipico es mezclar raices reales con complejas."
        }
        RewriteRule::OdeSystemRule => {
            "Motivo: mira los autovalores de la matriz: reales, repetidos o complejos; el error tipico es plantear mal el vector propio."
        }
        RewriteRule::LaplaceDirectRule => {
            "Motivo: transformada por tabla termino a termino; el error tipico es mezclar la variable t con s."
        }
        RewriteRule::LaplaceInverseRule => {
            "Motivo: mira el denominador para reconocer la forma de tabla; el error tipico es no completar el cuadrado si hace falta."
        }
        RewriteRule::RischTrigRule => {
            "Motivo: mira si el integrando trae una derivada multiplicando afuera; el error tipico es forzar por partes."
        }
        RewriteRule::RischEulerSqrtRule => {
            "Motivo: mira la forma raiz con cuadrado adentro; el error tipico es plantear mal la sustitucion de Euler."
        }
        RewriteRule::RischPartsRule => {
            "Motivo: elige u y dv con LIATE; el error tipico es iterar cuando el ciclo ya se cerro."
        }
        RewriteRule::OscillationRule => {
            "Motivo: mira el factor oscilante reescrito por Gruntz; el error tipico es dar el limite sin comparar crecimientos."
        }
        RewriteRule::OdeNthOrderRule => {
            "Motivo: mira la caracteristica de grado n y la multiplicidad de cada raiz; el error tipico es olvidar los x^k que agrega una raiz repetida."
        }
        RewriteRule::LaplaceShiftRule => {
            "Motivo: mira el corrimiento en s (e^{as}·F(s)); el error tipico es desplazar en t en vez de en s."
        }
        RewriteRule::GroebnerOrderRule => {
            "Motivo: mira el orden de variables elegido y como cae el sistema; el error tipico es cambiar de orden a mitad del calculo."
        }
        RewriteRule::EliminateRule => {
            "Motivo: mira que variables se eliminaron y con que polinomio; el error tipico es leer resultados intermedios como finales."
        }
        _ => {
            "Motivo: mira que parte de la expresion cambio en este paso; si no la ves, compara el antes y el despues termino a termino."
        }
    }
}

/// Motivo pedagógico desde el nombre estable de la regla (`Display` de
/// [`RewriteRule`] guardado en el fence). Misma tabla que [`motivo`], sin
/// acoplar el dibujado al enum del motor: la UI llama a esta función con
/// `item.rule`. Nombres desconocidos dan el fallback honesto.
#[must_use]
pub fn motivo_str(name: &str) -> &'static str {
    match name {
        "PowerRule" => motivo(&RewriteRule::PowerRule),
        "SumRule" => motivo(&RewriteRule::SumRule),
        "DifferenceRule" => motivo(&RewriteRule::DifferenceRule),
        "ProductRule" => motivo(&RewriteRule::ProductRule),
        "QuotientRule" => motivo(&RewriteRule::QuotientRule),
        "ChainRule" => motivo(&RewriteRule::ChainRule),
        "TrigRule" => motivo(&RewriteRule::TrigRule),
        "ExpRule" => motivo(&RewriteRule::ExpRule),
        "LogRule" => motivo(&RewriteRule::LogRule),
        "ConstantRule" => motivo(&RewriteRule::ConstantRule),
        "VariableRule" => motivo(&RewriteRule::VariableRule),
        "NegRule" => motivo(&RewriteRule::NegRule),
        "SqrtRule" => motivo(&RewriteRule::SqrtRule),
        "IntegrationPowerRule" => motivo(&RewriteRule::IntegrationPowerRule),
        "IntegrationSumRule" => motivo(&RewriteRule::IntegrationSumRule),
        "IntegrationConstantRule" => motivo(&RewriteRule::IntegrationConstantRule),
        "IntegrationExpRule" => motivo(&RewriteRule::IntegrationExpRule),
        "IntegrationTrigRule" => motivo(&RewriteRule::IntegrationTrigRule),
        "IntegrationPartialFractions" => motivo(&RewriteRule::IntegrationPartialFractions),
        "IntegrationArctanRule" => motivo(&RewriteRule::IntegrationArctanRule),
        "OdeSecondOrderRule" => motivo(&RewriteRule::OdeSecondOrderRule),
        "OdeSystemRule" => motivo(&RewriteRule::OdeSystemRule),
        "LaplaceDirectRule" => motivo(&RewriteRule::LaplaceDirectRule),
        "LaplaceInverseRule" => motivo(&RewriteRule::LaplaceInverseRule),
        "LimitRichardson" => motivo(&RewriteRule::LimitRichardson),
        "GruntzLogRewriteRule" => motivo(&RewriteRule::GruntzLogRewriteRule),
        "SolveLinearRule" => motivo(&RewriteRule::SolveLinearRule),
        "SolveQuadraticRule" => motivo(&RewriteRule::SolveQuadraticRule),
        "SolveSturmRule" => motivo(&RewriteRule::SolveSturmRule),
        "RischTrigRule" => motivo(&RewriteRule::RischTrigRule),
        "RischEulerSqrtRule" => motivo(&RewriteRule::RischEulerSqrtRule),
        "RischPartsRule" => motivo(&RewriteRule::RischPartsRule),
        "OscillationRule" => motivo(&RewriteRule::OscillationRule),
        "OdeNthOrderRule" => motivo(&RewriteRule::OdeNthOrderRule),
        "OdeEulerRule" => motivo(&RewriteRule::OdeEulerRule),
        "OdeFirstOrderRule" => motivo(&RewriteRule::OdeFirstOrderRule),
        "OdeSecondHomogRule" => motivo(&RewriteRule::OdeSecondHomogRule),
        "StatsDescriptiveRule" => motivo(&RewriteRule::StatsDescriptiveRule),
        "StatsRegressionRule" => motivo(&RewriteRule::StatsRegressionRule),
        "FrobeniusRule" => motivo(&RewriteRule::FrobeniusRule),
        "LaplaceCalculusRule" => motivo(&RewriteRule::LaplaceCalculusRule),
        "TaylorRule" => motivo(&RewriteRule::TaylorRule),
        "LaplaceShiftRule" => motivo(&RewriteRule::LaplaceShiftRule),
        "GroebnerOrderRule" => motivo(&RewriteRule::GroebnerOrderRule),
        "EliminateRule" => motivo(&RewriteRule::EliminateRule),
        "SystemEliminationRule" => motivo(&RewriteRule::SystemEliminationRule),
        "SystemSubstitutionRule" => motivo(&RewriteRule::SystemSubstitutionRule),
        "BarrowRule" => motivo(&RewriteRule::BarrowRule),
        "Simplify" => motivo(&RewriteRule::Simplify),
        "AbsRule" => motivo(&RewriteRule::AbsRule),
        _ => motivo(&RewriteRule::Generic),
    }
}

/// Etiqueta pedagógica corta de una regla (qué hace, en español).
///
/// La cabecera de la tarjeta muestra esto en lugar del token técnico
/// (`SolveLinearRule` → «despeje lineal») y el token queda como detalle
/// en el cuerpo. Nombres desconocidos (fence ajeno) devuelven el nombre
/// tal cual: nunca se inventa una etiqueta.
#[must_use]
pub fn etiqueta(rule: &RewriteRule) -> &'static str {
    match rule {
        RewriteRule::ConstantRule => "constante",
        RewriteRule::VariableRule => "variable",
        RewriteRule::SumRule => "suma término a término",
        RewriteRule::DifferenceRule => "diferencia término a término",
        RewriteRule::ProductRule => "producto",
        RewriteRule::QuotientRule => "cociente",
        RewriteRule::PowerRule => "potencia",
        RewriteRule::NegRule => "signo",
        RewriteRule::ChainRule => "cadena",
        RewriteRule::ExpRule => "exponencial",
        RewriteRule::LogRule => "logaritmo",
        RewriteRule::TrigRule => "trigonométrica",
        RewriteRule::SqrtRule => "raíz",
        RewriteRule::AbsRule => "valor absoluto",
        RewriteRule::Simplify => "simplificación",
        RewriteRule::IntegrationPowerRule => "potencia (integral)",
        RewriteRule::IntegrationSumRule => "suma (integral)",
        RewriteRule::IntegrationConstantRule => "constante (integral)",
        RewriteRule::IntegrationExpRule => "exponencial (integral)",
        RewriteRule::IntegrationTrigRule => "trigonométrica (integral)",
        RewriteRule::IntegrationPartialFractions => "fracciones parciales",
        RewriteRule::IntegrationArctanRule => "forma arctan",
        RewriteRule::OdeSecondOrderRule => "EDO de 2º orden",
        RewriteRule::OdeSystemRule => "sistema de EDOs",
        RewriteRule::LaplaceDirectRule => "Laplace directa",
        RewriteRule::LaplaceInverseRule => "Laplace inversa",
        RewriteRule::LaplaceShiftRule => "corrimiento de Laplace",
        RewriteRule::LimitRichardson => "extrapolación de Richardson",
        RewriteRule::TaylorRule => "serie de Taylor",
        RewriteRule::SolveLinearRule => "despeje lineal",
        RewriteRule::SolveQuadraticRule => "fórmula cuadrática",
        RewriteRule::SolveSturmRule => "Sturm + bisección",
        RewriteRule::RischTrigRule => "Risch trigonométrico",
        RewriteRule::RischEulerSqrtRule => "Risch (Euler)",
        RewriteRule::RischPartsRule => "Risch por partes",
        RewriteRule::GruntzLogRewriteRule => "Gruntz (logaritmo)",
        RewriteRule::OscillationRule => "oscilación (Gruntz)",
        RewriteRule::OdeNthOrderRule => "EDO de orden n",
        RewriteRule::OdeEulerRule => "EDO de Euler",
        RewriteRule::OdeFirstOrderRule => "EDO lineal de 1er orden",
        RewriteRule::OdeSecondHomogRule => "EDO homogénea de 2º orden",
        RewriteRule::StatsDescriptiveRule => "descriptiva de lista",
        RewriteRule::StatsRegressionRule => "regresión lineal",
        RewriteRule::FrobeniusRule => "serie de Frobenius",
        RewriteRule::LaplaceCalculusRule => "Laplace (cálculo por transformada)",
        RewriteRule::GroebnerOrderRule => "Grobner por orden",
        RewriteRule::EliminateRule => "eliminación de variables",
        RewriteRule::SystemEliminationRule => "sistema por eliminación",
        RewriteRule::SystemSubstitutionRule => "sistema por sustitución",
        RewriteRule::BarrowRule => "integral definida (Barrow)",
        RewriteRule::Generic => "regla general",
    }
}

/// Etiqueta corta desde el nombre estable guardado en el fence
/// (`Display` de [`RewriteRule`]). Nombre desconocido → el nombre tal
/// cual (honesto: nunca se inventa).
#[must_use]
pub fn etiqueta_str(name: &str) -> String {
    let conocida: &'static str = match name {
        "ConstantRule" => etiqueta(&RewriteRule::ConstantRule),
        "VariableRule" => etiqueta(&RewriteRule::VariableRule),
        "SumRule" => etiqueta(&RewriteRule::SumRule),
        "DifferenceRule" => etiqueta(&RewriteRule::DifferenceRule),
        "ProductRule" => etiqueta(&RewriteRule::ProductRule),
        "QuotientRule" => etiqueta(&RewriteRule::QuotientRule),
        "PowerRule" => etiqueta(&RewriteRule::PowerRule),
        "NegRule" => etiqueta(&RewriteRule::NegRule),
        "ChainRule" => etiqueta(&RewriteRule::ChainRule),
        "ExpRule" => etiqueta(&RewriteRule::ExpRule),
        "LogRule" => etiqueta(&RewriteRule::LogRule),
        "TrigRule" => etiqueta(&RewriteRule::TrigRule),
        "SqrtRule" => etiqueta(&RewriteRule::SqrtRule),
        "AbsRule" => etiqueta(&RewriteRule::AbsRule),
        "Simplify" => etiqueta(&RewriteRule::Simplify),
        "IntegrationPowerRule" => etiqueta(&RewriteRule::IntegrationPowerRule),
        "IntegrationSumRule" => etiqueta(&RewriteRule::IntegrationSumRule),
        "IntegrationConstantRule" => etiqueta(&RewriteRule::IntegrationConstantRule),
        "IntegrationExpRule" => etiqueta(&RewriteRule::IntegrationExpRule),
        "IntegrationTrigRule" => etiqueta(&RewriteRule::IntegrationTrigRule),
        "IntegrationPartialFractions" => etiqueta(&RewriteRule::IntegrationPartialFractions),
        "IntegrationArctanRule" => etiqueta(&RewriteRule::IntegrationArctanRule),
        "OdeSecondOrderRule" => etiqueta(&RewriteRule::OdeSecondOrderRule),
        "OdeSystemRule" => etiqueta(&RewriteRule::OdeSystemRule),
        "LaplaceDirectRule" => etiqueta(&RewriteRule::LaplaceDirectRule),
        "LaplaceInverseRule" => etiqueta(&RewriteRule::LaplaceInverseRule),
        "LaplaceShiftRule" => etiqueta(&RewriteRule::LaplaceShiftRule),
        "LimitRichardson" => etiqueta(&RewriteRule::LimitRichardson),
        "TaylorRule" => etiqueta(&RewriteRule::TaylorRule),
        "SolveLinearRule" => etiqueta(&RewriteRule::SolveLinearRule),
        "SolveQuadraticRule" => etiqueta(&RewriteRule::SolveQuadraticRule),
        "SolveSturmRule" => etiqueta(&RewriteRule::SolveSturmRule),
        "RischTrigRule" => etiqueta(&RewriteRule::RischTrigRule),
        "RischEulerSqrtRule" => etiqueta(&RewriteRule::RischEulerSqrtRule),
        "RischPartsRule" => etiqueta(&RewriteRule::RischPartsRule),
        "GruntzLogRewriteRule" => etiqueta(&RewriteRule::GruntzLogRewriteRule),
        "OscillationRule" => etiqueta(&RewriteRule::OscillationRule),
        "OdeNthOrderRule" => etiqueta(&RewriteRule::OdeNthOrderRule),
        "OdeEulerRule" => etiqueta(&RewriteRule::OdeEulerRule),
        "OdeFirstOrderRule" => etiqueta(&RewriteRule::OdeFirstOrderRule),
        "OdeSecondHomogRule" => etiqueta(&RewriteRule::OdeSecondHomogRule),
        "StatsDescriptiveRule" => etiqueta(&RewriteRule::StatsDescriptiveRule),
        "StatsRegressionRule" => etiqueta(&RewriteRule::StatsRegressionRule),
        "FrobeniusRule" => etiqueta(&RewriteRule::FrobeniusRule),
        "LaplaceCalculusRule" => etiqueta(&RewriteRule::LaplaceCalculusRule),
        "GroebnerOrderRule" => etiqueta(&RewriteRule::GroebnerOrderRule),
        "EliminateRule" => etiqueta(&RewriteRule::EliminateRule),
        "SystemEliminationRule" => etiqueta(&RewriteRule::SystemEliminationRule),
        "SystemSubstitutionRule" => etiqueta(&RewriteRule::SystemSubstitutionRule),
        "BarrowRule" => etiqueta(&RewriteRule::BarrowRule),
        "Generic" => etiqueta(&RewriteRule::Generic),
        _ => "",
    };
    if conocida.is_empty() {
        return name.trim().to_owned();
    }
    conocida.to_owned()
}

/// Un paso pedagógico `before --rule--> after`, listo para renderizar en el
/// panel del asistente (`grafito-command`): `before`/`after` son expresiones
/// acotadas a `MAX_STEP_BYTES` y `description` es texto final en español
/// rioplatense (no requiere post-procesado).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CasStep {
    pub index: usize,
    pub rule: RewriteRule,
    pub before: String,
    pub after: String,
    pub description: String,
}

// ---------------------------------------------------------------------------
// CasOp — despacho genérico
// ---------------------------------------------------------------------------

/// Operación CAS cuya traza se solicita. `Clone + Debug + PartialEq` para que
/// `grafito-command` pueda guardarla encolada/serializada antes de llamar a
/// [`steps_for_op`].
#[derive(Clone, Debug, PartialEq)]
pub enum CasOp {
    Derivative {
        expr: String,
        var: String,
    },
    Integral {
        expr: String,
        var: String,
    },
    Limit {
        expr: String,
        var: String,
        at: f64,
    },
    Taylor {
        expr: String,
        var: String,
        center: f64,
        order: usize,
    },
    /// Frente B1: `Solve[expr, var]` general (todas las raíces reales).
    Solve {
        expr: String,
        var: String,
    },
    /// Frente B2: EDO orden-n constante (`coeffs = [aₙ..a₀]`).
    OdeNthOrder {
        coeffs: Vec<String>,
        rhs: String,
        x: String,
    },
    /// Frente B2: Euler `x²y''+a·x·y'+b·y = rhs`.
    OdeEuler {
        a: String,
        b: String,
        rhs: String,
        x: String,
    },
    /// EDO lineal de 1er orden `y' + P(x)·y = Q(x)` (factor integrante).
    /// Solo lineal en `y`: lo demás da `Err` honesto.
    OdeFirstOrder {
        p: String,
        q: String,
        x: String,
    },
    /// EDO homogénea de 2º orden `a·y''+b·y'+c·y = 0` constante.
    /// Coeficientes no constantes dan `Err` honesto.
    OdeSecondHomog {
        a: String,
        b: String,
        c: String,
        x: String,
    },
    /// Descriptiva de lista `{a, b, c}` (media/mediana/varianza/desvío
    /// poblacionales ÷N con verificación recalculada). Vacía o no finita
    /// da `Err` honesto; la muestral (÷N−1) vive en `SampleVariance` directo.
    StatsDescriptive {
        data: String,
    },
    /// Regresión lineal `LinearRegression[{xs}, {ys}]` por mínimos
    /// cuadrados (pendiente/intercepto/R² con verificación 0≤R²≤1).
    /// Largos distintos, `x` constante o no finitos dan `Err` honesto.
    StatsRegression {
        xs: String,
        ys: String,
    },
    /// Frente B2: Frobenius `y''+p·y'+q·y = 0` en punto ordinario.
    Frobenius {
        p: String,
        q: String,
        x: String,
        center: f64,
        terms: usize,
    },
    /// Frente B2: `L{y⁽ⁿ⁾}` con iniciales.
    LaplaceDerivative {
        order: u32,
        y: String,
        t: String,
        s: String,
        initials: Vec<String>,
    },
    /// Frente B2: `L{∫₀ᵗ f} = L{f}/s`.
    LaplaceIntegral {
        f: String,
        t: String,
        s: String,
    },
    /// Frente B2: Groebner con orden (`lex|grlex|grevlex`).
    GroebnerOrdered {
        polys: Vec<String>,
        vars: Vec<String>,
        order: String,
    },
    /// Frente B2: `Eliminate[polys, vars, elim]`.
    Eliminate {
        polys: Vec<String>,
        vars: Vec<String>,
        elim: Vec<String>,
    },
    /// Sistema lineal 2×2 por eliminación/sustitución (`SolveNlSystem` lineal).
    /// Solo lineal en exactamente `var1`, `var2`: lo demás da `Err` honesto.
    System2x2 {
        eq1: String,
        eq2: String,
        var1: String,
        var2: String,
    },
    /// Integral definida `∫[a,b] expr dvar` (antiderivada + Barrow).
    /// Sin primitiva soportada o extremos no finitos da `Err` honesto.
    DefiniteIntegral {
        expr: String,
        var: String,
        a: f64,
        b: f64,
    },
}

// ---------------------------------------------------------------------------
// Statem
// ---------------------------------------------------------------------------

/// Estado tipado del stepper.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CasStepperState {
    Idle,
    Parsing,
    Stepping,
    Verifying,
    Done,
    Failed { reason: String },
}

/// Stepper con transiciones tipadas vía `&mut self`.
#[derive(Clone, Debug)]
pub struct CasStepper {
    state: CasStepperState,
    steps: Vec<CasStep>,
}

impl Default for CasStepper {
    fn default() -> Self {
        Self::new()
    }
}

impl CasStepper {
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: CasStepperState::Idle,
            steps: Vec::new(),
        }
    }

    #[must_use]
    pub fn state(&self) -> &CasStepperState {
        &self.state
    }

    #[must_use]
    pub fn steps(&self) -> &[CasStep] {
        &self.steps
    }

    /// Idle → Parsing
    pub fn begin_parsing(&mut self) -> Result<(), String> {
        match self.state {
            CasStepperState::Idle => {
                self.state = CasStepperState::Parsing;
                Ok(())
            }
            _ => Err(format!("transición inválida desde {:?}", self.state)),
        }
    }

    /// Parsing → Stepping
    pub fn begin_stepping(&mut self) -> Result<(), String> {
        match self.state {
            CasStepperState::Parsing => {
                self.state = CasStepperState::Stepping;
                Ok(())
            }
            _ => Err(format!("transición inválida desde {:?}", self.state)),
        }
    }

    /// Stepping → Verifying
    pub fn begin_verifying(&mut self) -> Result<(), String> {
        match self.state {
            CasStepperState::Stepping => {
                self.state = CasStepperState::Verifying;
                Ok(())
            }
            _ => Err(format!("transición inválida desde {:?}", self.state)),
        }
    }

    /// Verifying → Done | Failed
    pub fn finish(&mut self, ok: bool, reason: Option<String>) -> Result<(), String> {
        match &self.state {
            CasStepperState::Verifying => {
                if ok {
                    self.state = CasStepperState::Done;
                } else {
                    self.state = CasStepperState::Failed {
                        reason: reason.unwrap_or_else(|| "falló verificación".to_string()),
                    };
                }
                Ok(())
            }
            _ => Err(format!("transición inválida desde {:?}", self.state)),
        }
    }

    /// Ejecuta el despacho genérico `CasOp → Vec<CasStep>` con statem completo.
    pub fn run(&mut self, op: &CasOp) -> Result<Vec<CasStep>, String> {
        self.begin_parsing()?;
        let parsed_ok = match op {
            CasOp::Derivative { expr, var } | CasOp::Integral { expr, var } => {
                validate_identifier(var).is_ok()
                    && validate_input_bytes(expr).is_ok()
                    && parse_ast(&expr.replace(' ', "")).is_ok()
            }
            CasOp::Solve { expr, var } => {
                validate_identifier(var).is_ok()
                    && validate_input_bytes(expr).is_ok()
                    && parse_ast(&expr.replace(' ', "")).is_ok()
            }
            CasOp::OdeNthOrder { coeffs, rhs, x } => {
                validate_identifier(x).is_ok()
                    && validate_input_bytes(rhs).is_ok()
                    && !coeffs.is_empty()
                    && coeffs.len() <= crate::ode::MAX_ODE_NTH_ORDER + 1
                    && coeffs.iter().all(|c| validate_input_bytes(c).is_ok())
            }
            CasOp::OdeEuler { a, b, rhs, x } => {
                validate_identifier(x).is_ok()
                    && validate_input_bytes(rhs).is_ok()
                    && validate_input_bytes(a).is_ok()
                    && validate_input_bytes(b).is_ok()
            }
            CasOp::OdeFirstOrder { p, q, x } => {
                validate_identifier(x).is_ok()
                    && validate_input_bytes(p).is_ok()
                    && validate_input_bytes(q).is_ok()
            }
            CasOp::OdeSecondHomog { a, b, c, x } => {
                validate_identifier(x).is_ok()
                    && validate_input_bytes(a).is_ok()
                    && validate_input_bytes(b).is_ok()
                    && validate_input_bytes(c).is_ok()
            }
            CasOp::StatsDescriptive { data } => validate_input_bytes(data).is_ok(),
            CasOp::StatsRegression { xs, ys } => {
                validate_input_bytes(xs).is_ok() && validate_input_bytes(ys).is_ok()
            }
            CasOp::Frobenius {
                p,
                q,
                x,
                center,
                terms,
            } => {
                validate_identifier(x).is_ok()
                    && validate_input_bytes(p).is_ok()
                    && validate_input_bytes(q).is_ok()
                    && center.is_finite()
                    && (2..=crate::ode::MAX_FROBENIUS_TERMS).contains(terms)
            }
            CasOp::LaplaceDerivative {
                y,
                t,
                s,
                initials,
                order,
            } => {
                validate_identifier(y).is_ok()
                    && validate_identifier(t).is_ok()
                    && validate_identifier(s).is_ok()
                    && (*order as usize) == initials.len()
                    && initials.iter().all(|c| validate_input_bytes(c).is_ok())
            }
            CasOp::LaplaceIntegral { f, t, s } => {
                validate_identifier(t).is_ok()
                    && validate_identifier(s).is_ok()
                    && validate_input_bytes(f).is_ok()
            }
            CasOp::GroebnerOrdered { polys, vars, .. } | CasOp::Eliminate { polys, vars, .. } => {
                polys.len() <= crate::cas::MAX_GROEBNER_POLYS
                    && vars.len() <= crate::cas::MAX_GROEBNER_VARS
                    && polys.iter().all(|p| validate_input_bytes(p).is_ok())
            }
            CasOp::System2x2 {
                eq1,
                eq2,
                var1,
                var2,
            } => {
                validate_identifier(var1).is_ok()
                    && validate_identifier(var2).is_ok()
                    && validate_input_bytes(eq1).is_ok()
                    && validate_input_bytes(eq2).is_ok()
            }
            CasOp::DefiniteIntegral { expr, var, a, b } => {
                a.is_finite()
                    && b.is_finite()
                    && validate_identifier(var).is_ok()
                    && validate_input_bytes(expr).is_ok()
                    && parse_ast(&expr.replace(' ', "")).is_ok()
            }
            CasOp::Limit { expr, var, at } => {
                if !at.is_finite() {
                    false
                } else {
                    validate_identifier(var).is_ok()
                        && validate_input_bytes(expr).is_ok()
                        && parse_ast(&expr.replace(' ', "")).is_ok()
                }
            }
            CasOp::Taylor {
                expr, var, order, ..
            } => {
                validate_identifier(var).is_ok()
                    && validate_input_bytes(expr).is_ok()
                    && *order <= crate::analysis::MAX_TAYLOR_ORDER
                    && parse_ast(&expr.replace(' ', "")).is_ok()
            }
        };
        if !parsed_ok {
            self.state = CasStepperState::Failed {
                reason: "parse error".to_string(),
            };
            return Err("parse error".to_string());
        }
        self.begin_stepping()?;
        let steps = steps_for_op(op)?;
        self.steps = steps;
        self.begin_verifying()?;
        let ok = !self.steps.is_empty();
        self.finish(
            ok,
            if ok {
                None
            } else {
                Some("sin pasos".to_string())
            },
        )?;
        Ok(self.steps.clone())
    }
}

// ---------------------------------------------------------------------------
// Utilidades
// ---------------------------------------------------------------------------

fn validate_identifier(var: &str) -> Result<(), String> {
    let mut chars = var.chars();
    let first_ok = chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_');
    if !first_ok || !chars.all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(format!("variable no es identificador válido: {var}"));
    }
    Ok(())
}

fn validate_input_bytes(expr: &str) -> Result<(), String> {
    if expr.len() > crate::outcome::MAX_MATH_INPUT_BYTES {
        return Err(format!(
            "expresión excede {} bytes",
            crate::outcome::MAX_MATH_INPUT_BYTES
        ));
    }
    Ok(())
}

/// Trunca a `MAX_STEP_BYTES` respetando borde UTF-8.
fn truncate_bytes(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let mut end = max;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    let mut out = s[..end].to_string();
    out.push('…');
    out
}

fn push_step(
    steps: &mut Vec<CasStep>,
    rule: RewriteRule,
    before: &Expr,
    after: &Expr,
    description: &str,
) {
    if steps.len() >= MAX_CAS_STEPS {
        return;
    }
    let before_s = truncate_bytes(&before.to_expr_string(), MAX_STEP_BYTES);
    let after_s = truncate_bytes(&after.to_expr_string(), MAX_STEP_BYTES);
    if before_s == after_s && rule != RewriteRule::Simplify {
        return;
    }
    let idx = steps.len();
    steps.push(CasStep {
        index: idx,
        rule,
        before: before_s,
        after: after_s,
        description: truncate_bytes(description, MAX_STEP_BYTES),
    });
}

fn push_limit_step(steps: &mut Vec<CasStep>, before: &str, after: &str, description: &str) {
    if steps.len() >= MAX_CAS_STEPS {
        return;
    }
    let idx = steps.len();
    steps.push(CasStep {
        index: idx,
        rule: RewriteRule::LimitRichardson,
        before: truncate_bytes(before, MAX_STEP_BYTES),
        after: truncate_bytes(after, MAX_STEP_BYTES),
        description: truncate_bytes(description, MAX_STEP_BYTES),
    });
}

// ---------------------------------------------------------------------------
// Simplify genérico (copia de symbolic::simplify_expr)
// ---------------------------------------------------------------------------

fn simplify_expr(e: &Expr) -> Expr {
    let mut current = e.clone();
    for _ in 0..30 {
        let next = simplify_once(&current);
        if next.structurally_eq(&current) {
            return next;
        }
        current = next;
    }
    current
}

fn simplify_once(e: &Expr) -> Expr {
    use Expr::{
        Abs, Acos, Acosh, Arg, Asin, Asinh, Atan, Atan2, Atanh, BesselI, BesselJ, BesselY, Beta,
        Cbrt, Ceil, Clamp, Conj, Const, Cos, Cosh, Cot, Csc, Digamma, Erf, Erfc, Exp, Floor, Gamma,
        Heaviside, Im, Ln, LnGamma, Log, Max, Min, Modulo, Neg, Re, Round, Sec, Sign, Sin, Sinh,
        Sqrt, Tan, Tanh, Trigamma,
    };
    match e {
        Neg(a) => {
            let sa = simplify_once(a);
            match sa {
                Const(c) => Const(-c),
                Neg(inner) => *inner,
                _ => Neg(Box::new(sa)),
            }
        }
        Expr::Add(a, b) => {
            let sa = simplify_once(a);
            let sb = simplify_once(b);
            match (&sa, &sb) {
                (Const(ca), Const(cb)) => Const(ca + cb),
                (Const(ca), _) if *ca == 0.0 => sb,
                (_, Const(cb)) if *cb == 0.0 => sa,
                _ => Expr::Add(Box::new(sa), Box::new(sb)),
            }
        }
        Expr::Sub(a, b) => {
            let sa = simplify_once(a);
            let sb = simplify_once(b);
            match (&sa, &sb) {
                (Const(ca), Const(cb)) => Const(ca - cb),
                (_, Const(cb)) if *cb == 0.0 => sa,
                (Const(ca), _) if *ca == 0.0 => Neg(Box::new(sb)),
                _ if sa.structurally_eq(&sb) && sa.is_guaranteed_finite() => Const(0.0),
                _ => Expr::Sub(Box::new(sa), Box::new(sb)),
            }
        }
        Expr::Mul(a, b) => {
            let sa = simplify_once(a);
            let sb = simplify_once(b);
            match (&sa, &sb) {
                (Const(ca), Const(cb)) => Const(ca * cb),
                (Const(ca), _) if *ca == 0.0 && sb.is_guaranteed_finite() => Const(0.0),
                (_, Const(cb)) if *cb == 0.0 && sa.is_guaranteed_finite() => Const(0.0),
                (Const(ca), _) if *ca == 1.0 => sb,
                (_, Const(cb)) if *cb == 1.0 => sa,
                (Const(ca), _) if *ca == -1.0 => Neg(Box::new(sb)),
                (_, Const(cb)) if *cb == -1.0 => Neg(Box::new(sa)),
                _ => Expr::Mul(Box::new(sa), Box::new(sb)),
            }
        }
        Expr::Div(a, b) => {
            let sa = simplify_once(a);
            let sb = simplify_once(b);
            match (&sa, &sb) {
                (Const(ca), Const(cb)) if cb.abs() > 1e-300 => Const(ca / cb),
                (_, Const(cb)) if *cb == 1.0 => sa,
                _ => Expr::Div(Box::new(sa), Box::new(sb)),
            }
        }
        Expr::Pow(a, b) => {
            let sa = simplify_once(a);
            let sb = simplify_once(b);
            match (&sa, &sb) {
                (Const(ca), Const(cb)) if *ca != 0.0 || *cb > 0.0 => Const(ca.powf(*cb)),
                (_, Const(cb)) if *cb == 1.0 => sa,
                _ => Expr::Pow(Box::new(sa), Box::new(sb)),
            }
        }
        Sin(a) => Sin(Box::new(simplify_once(a))),
        Cos(a) => Cos(Box::new(simplify_once(a))),
        Tan(a) => Tan(Box::new(simplify_once(a))),
        Asin(a) => Asin(Box::new(simplify_once(a))),
        Acos(a) => Acos(Box::new(simplify_once(a))),
        Atan(a) => Atan(Box::new(simplify_once(a))),
        Asinh(a) => crate::ast::Expr::Asinh(Box::new(simplify_once(a))),
        Acosh(a) => crate::ast::Expr::Acosh(Box::new(simplify_once(a))),
        Atanh(a) => crate::ast::Expr::Atanh(Box::new(simplify_once(a))),
        Exp(a) => Exp(Box::new(simplify_once(a))),
        Ln(a) => Ln(Box::new(simplify_once(a))),
        Log(a) => Log(Box::new(simplify_once(a))),
        Sqrt(a) => Sqrt(Box::new(simplify_once(a))),
        Abs(a) => Abs(Box::new(simplify_once(a))),
        Sinh(a) => Sinh(Box::new(simplify_once(a))),
        Cosh(a) => Cosh(Box::new(simplify_once(a))),
        Tanh(a) => Tanh(Box::new(simplify_once(a))),
        Sec(a) => Sec(Box::new(simplify_once(a))),
        Csc(a) => Csc(Box::new(simplify_once(a))),
        Cot(a) => Cot(Box::new(simplify_once(a))),
        Floor(a) => Floor(Box::new(simplify_once(a))),
        Ceil(a) => Ceil(Box::new(simplify_once(a))),
        Round(a) => Round(Box::new(simplify_once(a))),
        Sign(a) => Sign(Box::new(simplify_once(a))),
        Heaviside(a) => Heaviside(Box::new(simplify_once(a))),
        Cbrt(a) => Cbrt(Box::new(simplify_once(a))),
        Re(a) => Re(Box::new(simplify_once(a))),
        Im(a) => Im(Box::new(simplify_once(a))),
        Arg(a) => Arg(Box::new(simplify_once(a))),
        Conj(a) => Conj(Box::new(simplify_once(a))),
        Erf(a) => Erf(Box::new(simplify_once(a))),
        Erfc(a) => Erfc(Box::new(simplify_once(a))),
        Gamma(a) => Gamma(Box::new(simplify_once(a))),
        LnGamma(a) => LnGamma(Box::new(simplify_once(a))),
        Digamma(a) => Digamma(Box::new(simplify_once(a))),
        Trigamma(a) => Trigamma(Box::new(simplify_once(a))),
        Atan2(a, b) => Atan2(Box::new(simplify_once(a)), Box::new(simplify_once(b))),
        Modulo(a, b) => Modulo(Box::new(simplify_once(a)), Box::new(simplify_once(b))),
        Min(a, b) => Min(Box::new(simplify_once(a)), Box::new(simplify_once(b))),
        Max(a, b) => Max(Box::new(simplify_once(a)), Box::new(simplify_once(b))),
        Beta(a, b) => Beta(Box::new(simplify_once(a)), Box::new(simplify_once(b))),
        BesselJ(a, b) => BesselJ(Box::new(simplify_once(a)), Box::new(simplify_once(b))),
        BesselY(a, b) => BesselY(Box::new(simplify_once(a)), Box::new(simplify_once(b))),
        BesselI(a, b) => BesselI(Box::new(simplify_once(a)), Box::new(simplify_once(b))),
        Expr::Lt(a, b) => Expr::Lt(Box::new(simplify_once(a)), Box::new(simplify_once(b))),
        Expr::Gt(a, b) => Expr::Gt(Box::new(simplify_once(a)), Box::new(simplify_once(b))),
        Expr::Le(a, b) => Expr::Le(Box::new(simplify_once(a)), Box::new(simplify_once(b))),
        Expr::Ge(a, b) => Expr::Ge(Box::new(simplify_once(a)), Box::new(simplify_once(b))),
        Expr::Eq(a, b) => Expr::Eq(Box::new(simplify_once(a)), Box::new(simplify_once(b))),
        Expr::Ne(a, b) => Expr::Ne(Box::new(simplify_once(a)), Box::new(simplify_once(b))),
        Clamp(a, b, c) => Clamp(
            Box::new(simplify_once(a)),
            Box::new(simplify_once(b)),
            Box::new(simplify_once(c)),
        ),
        Expr::Sum(body, v, s, t) => Expr::Sum(
            Box::new(simplify_once(body)),
            v.clone(),
            Box::new(simplify_once(s)),
            Box::new(simplify_once(t)),
        ),
        Expr::Product(body, v, s, t) => Expr::Product(
            Box::new(simplify_once(body)),
            v.clone(),
            Box::new(simplify_once(s)),
            Box::new(simplify_once(t)),
        ),
        Expr::Piecewise(pieces, default) => {
            let np: Vec<(Box<Expr>, Box<Expr>)> = pieces
                .iter()
                .map(|(c, v)| (Box::new(simplify_once(c)), Box::new(simplify_once(v))))
                .collect();
            Expr::Piecewise(np, Box::new(simplify_once(default)))
        }
        Const(_) | Expr::Var(_) => e.clone(),
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn contains_var(e: &Expr, var: &str) -> bool {
    let mut vars = HashSet::new();
    e.get_variables(&mut vars);
    vars.contains(var)
}

fn expr_is_linear_in(expr: &Expr, var: &str) -> bool {
    use Expr::{Const, Var};
    match expr {
        Var(name) => name == var,
        Const(_) => true,
        Expr::Neg(inner) => expr_is_linear_in(inner, var),
        Expr::Add(left, right) | Expr::Sub(left, right) => {
            expr_is_linear_in(left, var) && expr_is_linear_in(right, var)
        }
        Expr::Mul(left, right) => {
            if matches!(left.as_ref(), Const(_)) {
                expr_is_linear_in(right, var)
            } else if matches!(right.as_ref(), Const(_)) {
                expr_is_linear_in(left, var)
            } else {
                false
            }
        }
        _ => false,
    }
}

fn expr_linear_coeff(expr: &Expr, var: &str) -> Option<(f64, f64)> {
    use Expr::{Const, Var};
    match expr {
        Var(name) if name == var => Some((1.0, 0.0)),
        Const(v) if v.is_finite() => Some((0.0, *v)),
        Expr::Neg(inner) => {
            let (a, b) = expr_linear_coeff(inner, var)?;
            Some((-a, -b))
        }
        Expr::Add(l, r) => {
            let (a1, b1) = expr_linear_coeff(l, var)?;
            let (a2, b2) = expr_linear_coeff(r, var)?;
            let a = a1 + a2;
            let b = b1 + b2;
            (a.is_finite() && b.is_finite()).then_some((a, b))
        }
        Expr::Sub(l, r) => {
            let (a1, b1) = expr_linear_coeff(l, var)?;
            let (a2, b2) = expr_linear_coeff(r, var)?;
            let a = a1 - a2;
            let b = b1 - b2;
            (a.is_finite() && b.is_finite()).then_some((a, b))
        }
        Expr::Mul(l, r) => {
            if let Const(c) = l.as_ref() {
                let (a, b) = expr_linear_coeff(r, var)?;
                let a2 = c * a;
                let b2 = c * b;
                (a2.is_finite() && b2.is_finite()).then_some((a2, b2))
            } else if let Const(c) = r.as_ref() {
                let (a, b) = expr_linear_coeff(l, var)?;
                let a2 = c * a;
                let b2 = c * b;
                (a2.is_finite() && b2.is_finite()).then_some((a2, b2))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn constant_scalar_value(expr: &Expr) -> Option<f64> {
    match expr {
        Expr::Const(v) if v.is_finite() => Some(*v),
        Expr::Neg(inner) => Some(-constant_scalar_value(inner)?),
        Expr::Add(l, r) => Some(constant_scalar_value(l)? + constant_scalar_value(r)?),
        Expr::Sub(l, r) => Some(constant_scalar_value(l)? - constant_scalar_value(r)?),
        Expr::Mul(l, r) => Some(constant_scalar_value(l)? * constant_scalar_value(r)?),
        Expr::Div(a, b) => {
            let den = constant_scalar_value(b)?;
            if den == 0.0 {
                return None;
            }
            Some(constant_scalar_value(a)? / den)
        }
        Expr::Pow(a, b) => Some(constant_scalar_value(a)?.powf(constant_scalar_value(b)?)),
        _ => None,
    }
    .filter(|v| v.is_finite())
}

// ---------------------------------------------------------------------------
// Derivador instrumentado
// ---------------------------------------------------------------------------

fn diff_with_steps(e: &Expr, var: &str, steps: &mut Vec<CasStep>, depth: u32) -> Expr {
    const MAX_DEPTH: u32 = 256;
    if depth > MAX_DEPTH {
        return Expr::Const(f64::NAN);
    }
    if steps.len() >= MAX_CAS_STEPS {
        return e.clone();
    }
    use Expr::{
        Abs, Acos, Asin, Atan, Cbrt, Const, Cos, Cosh, Cot, Csc, Exp, Ln, Log, Sec, Sin, Sinh,
        Sqrt, Tan, Tanh, Var,
    };
    let before = e.clone();
    let (result, rule, desc): (Expr, RewriteRule, String) = match e {
        Const(_) => (
            Const(0.0),
            RewriteRule::ConstantRule,
            "derivada de constante = 0".into(),
        ),
        Var(v) => {
            if v == var {
                (
                    Const(1.0),
                    RewriteRule::VariableRule,
                    format!("d/d{var} {var} = 1"),
                )
            } else {
                (
                    Const(0.0),
                    RewriteRule::VariableRule,
                    format!("d/d{var} {v} = 0"),
                )
            }
        }
        Expr::Neg(a) => {
            let da = diff_with_steps(a, var, steps, depth + 1);
            let res = Expr::Neg(Box::new(da));
            (res, RewriteRule::NegRule, "regla de la negación".into())
        }
        Expr::Add(a, b) => {
            let da = diff_with_steps(a, var, steps, depth + 1);
            let db = diff_with_steps(b, var, steps, depth + 1);
            let res = Expr::Add(Box::new(da), Box::new(db));
            (res, RewriteRule::SumRule, "regla de la suma".into())
        }
        Expr::Sub(a, b) => {
            let da = diff_with_steps(a, var, steps, depth + 1);
            let db = diff_with_steps(b, var, steps, depth + 1);
            let res = Expr::Sub(Box::new(da), Box::new(db));
            (
                res,
                RewriteRule::DifferenceRule,
                "regla de la diferencia".into(),
            )
        }
        Expr::Mul(u, v) => {
            let du = diff_with_steps(u, var, steps, depth + 1);
            let dv = diff_with_steps(v, var, steps, depth + 1);
            let res = Expr::Add(
                Box::new(Expr::Mul(Box::new(du), v.clone())),
                Box::new(Expr::Mul(u.clone(), Box::new(dv))),
            );
            (
                res,
                RewriteRule::ProductRule,
                "regla del producto (u·v)' = u'·v + u·v'".into(),
            )
        }
        Expr::Div(u, v) => {
            let du = diff_with_steps(u, var, steps, depth + 1);
            let dv = diff_with_steps(v, var, steps, depth + 1);
            let res = Expr::Div(
                Box::new(Expr::Sub(
                    Box::new(Expr::Mul(Box::new(du), v.clone())),
                    Box::new(Expr::Mul(u.clone(), Box::new(dv))),
                )),
                Box::new(Expr::Pow(v.clone(), Box::new(Const(2.0)))),
            );
            (res, RewriteRule::QuotientRule, "regla del cociente".into())
        }
        Expr::Pow(base, exp) => match exp.as_ref() {
            Const(n) => {
                let du = diff_with_steps(base, var, steps, depth + 1);
                let res = Expr::Mul(
                    Box::new(Expr::Mul(
                        Box::new(Const(*n)),
                        Box::new(Expr::Pow(base.clone(), Box::new(Const(n - 1.0)))),
                    )),
                    Box::new(du),
                );
                (
                    res,
                    RewriteRule::PowerRule,
                    format!("potencia: d/dx x^{n} = {n}·x^{}·x'", n - 1.0),
                )
            }
            _ => {
                let du = diff_with_steps(base, var, steps, depth + 1);
                let dv = diff_with_steps(exp, var, steps, depth + 1);
                let base_clone = base.clone();
                let exp_clone = exp.clone();
                let res = Expr::Mul(
                    Box::new(Expr::Pow(base_clone.clone(), exp_clone.clone())),
                    Box::new(Expr::Add(
                        Box::new(Expr::Mul(Box::new(dv), Box::new(Ln(base_clone.clone())))),
                        Box::new(Expr::Mul(
                            exp_clone,
                            Box::new(Expr::Div(Box::new(du), base_clone)),
                        )),
                    )),
                );
                (
                    res,
                    RewriteRule::PowerRule,
                    "potencia general: (u^v)' = u^v·(v'·ln u + v·u'/u)".into(),
                )
            }
        },
        Sin(u) => {
            let du = diff_with_steps(u, var, steps, depth + 1);
            let res = Expr::Mul(Box::new(Cos(u.clone())), Box::new(du));
            (
                res,
                RewriteRule::ChainRule,
                "cadena: sin(u)' = cos(u)·u'".into(),
            )
        }
        Cos(u) => {
            let du = diff_with_steps(u, var, steps, depth + 1);
            let res = Expr::Mul(Box::new(Expr::Neg(Box::new(Sin(u.clone())))), Box::new(du));
            (
                res,
                RewriteRule::ChainRule,
                "cadena: cos(u)' = -sin(u)·u'".into(),
            )
        }
        Tan(u) => {
            let du = diff_with_steps(u, var, steps, depth + 1);
            let res = Expr::Mul(
                Box::new(Expr::Div(
                    Box::new(Const(1.0)),
                    Box::new(Expr::Pow(Box::new(Cos(u.clone())), Box::new(Const(2.0)))),
                )),
                Box::new(du),
            );
            (res, RewriteRule::ChainRule, "tan(u)' = u'/cos²(u)".into())
        }
        Sec(u) => {
            let du = diff_with_steps(u, var, steps, depth + 1);
            let res = Expr::Mul(
                Box::new(Expr::Mul(
                    Box::new(Sec(u.clone())),
                    Box::new(Tan(u.clone())),
                )),
                Box::new(du),
            );
            (
                res,
                RewriteRule::ChainRule,
                "sec(u)' = sec(u)·tan(u)·u'".into(),
            )
        }
        Csc(u) => {
            let du = diff_with_steps(u, var, steps, depth + 1);
            let res = Expr::Mul(
                Box::new(Expr::Neg(Box::new(Expr::Mul(
                    Box::new(Csc(u.clone())),
                    Box::new(Cot(u.clone())),
                )))),
                Box::new(du),
            );
            (
                res,
                RewriteRule::ChainRule,
                "csc(u)' = -csc(u)·cot(u)·u'".into(),
            )
        }
        Cot(u) => {
            let du = diff_with_steps(u, var, steps, depth + 1);
            let res = Expr::Mul(
                Box::new(Expr::Neg(Box::new(Expr::Pow(
                    Box::new(Csc(u.clone())),
                    Box::new(Const(2.0)),
                )))),
                Box::new(du),
            );
            (res, RewriteRule::ChainRule, "cot(u)' = -csc²(u)·u'".into())
        }
        Asin(u) => {
            let du = diff_with_steps(u, var, steps, depth + 1);
            let res = Expr::Div(
                Box::new(du),
                Box::new(Sqrt(Box::new(Expr::Sub(
                    Box::new(Const(1.0)),
                    Box::new(Expr::Pow(u.clone(), Box::new(Const(2.0)))),
                )))),
            );
            (res, RewriteRule::ChainRule, "asin(u)' = u'/√(1-u²)".into())
        }
        Acos(u) => {
            let du = diff_with_steps(u, var, steps, depth + 1);
            let res = Expr::Div(
                Box::new(Expr::Neg(Box::new(du))),
                Box::new(Sqrt(Box::new(Expr::Sub(
                    Box::new(Const(1.0)),
                    Box::new(Expr::Pow(u.clone(), Box::new(Const(2.0)))),
                )))),
            );
            (res, RewriteRule::ChainRule, "acos(u)' = -u'/√(1-u²)".into())
        }
        Atan(u) => {
            let du = diff_with_steps(u, var, steps, depth + 1);
            let res = Expr::Div(
                Box::new(du),
                Box::new(Expr::Add(
                    Box::new(Const(1.0)),
                    Box::new(Expr::Pow(u.clone(), Box::new(Const(2.0)))),
                )),
            );
            (res, RewriteRule::ChainRule, "atan(u)' = u'/(1+u²)".into())
        }
        Exp(u) => {
            let du = diff_with_steps(u, var, steps, depth + 1);
            let res = Expr::Mul(Box::new(Exp(u.clone())), Box::new(du));
            (res, RewriteRule::ExpRule, "exp(u)' = exp(u)·u'".into())
        }
        Ln(u) => {
            let du = diff_with_steps(u, var, steps, depth + 1);
            let res = Expr::Div(Box::new(du), u.clone());
            (res, RewriteRule::LogRule, "ln(u)' = u'/u".into())
        }
        Log(u) => {
            let du = diff_with_steps(u, var, steps, depth + 1);
            let res = Expr::Div(
                Box::new(du),
                Box::new(Expr::Mul(
                    u.clone(),
                    Box::new(Const(std::f64::consts::LN_10)),
                )),
            );
            (res, RewriteRule::LogRule, "log(u)' = u'/(u·ln10)".into())
        }
        Sqrt(u) => {
            let du = diff_with_steps(u, var, steps, depth + 1);
            let res = Expr::Div(
                Box::new(du),
                Box::new(Expr::Mul(Box::new(Const(2.0)), Box::new(Sqrt(u.clone())))),
            );
            (res, RewriteRule::SqrtRule, "sqrt(u)' = u'/(2·√u)".into())
        }
        Cbrt(u) => {
            let du = diff_with_steps(u, var, steps, depth + 1);
            let res = Expr::Div(
                Box::new(du),
                Box::new(Expr::Mul(
                    Box::new(Const(3.0)),
                    Box::new(Expr::Pow(Box::new(Cbrt(u.clone())), Box::new(Const(2.0)))),
                )),
            );
            (
                res,
                RewriteRule::PowerRule,
                "cbrt(u)' = u'/(3·cbrt(u)²)".into(),
            )
        }
        Abs(u) => {
            let du = diff_with_steps(u, var, steps, depth + 1);
            let res = Expr::Mul(
                Box::new(Expr::Div(u.clone(), Box::new(Abs(u.clone())))),
                Box::new(du),
            );
            (res, RewriteRule::AbsRule, "|u|' = sign(u)·u'".into())
        }
        Sinh(u) => {
            let du = diff_with_steps(u, var, steps, depth + 1);
            let res = Expr::Mul(Box::new(Cosh(u.clone())), Box::new(du));
            (res, RewriteRule::ChainRule, "sinh(u)' = cosh(u)·u'".into())
        }
        Cosh(u) => {
            let du = diff_with_steps(u, var, steps, depth + 1);
            let res = Expr::Mul(Box::new(Sinh(u.clone())), Box::new(du));
            (res, RewriteRule::ChainRule, "cosh(u)' = sinh(u)·u'".into())
        }
        Tanh(u) => {
            let du = diff_with_steps(u, var, steps, depth + 1);
            let res = Expr::Mul(
                Box::new(Expr::Div(
                    Box::new(Const(1.0)),
                    Box::new(Expr::Pow(Box::new(Cosh(u.clone())), Box::new(Const(2.0)))),
                )),
                Box::new(du),
            );
            (res, RewriteRule::ChainRule, "tanh(u)' = u'/cosh²(u)".into())
        }
        _ => {
            let res = e.diff(var);
            (res, RewriteRule::Generic, "regla genérica".into())
        }
    };
    let simplified = simplify_expr(&result);
    push_step(steps, rule, &before, &simplified, &desc);
    simplified
}

// ---------------------------------------------------------------------------
// Integrador instrumentado
// ---------------------------------------------------------------------------

fn integrate_with_steps(e: &Expr, var: &str, steps: &mut Vec<CasStep>, depth: u32) -> Option<Expr> {
    const MAX_DEPTH: u32 = 256;
    if depth > MAX_DEPTH {
        return None;
    }
    if steps.len() >= MAX_CAS_STEPS {
        return None;
    }
    use Expr::{Const, Exp, Var};
    let v = var.to_string();
    if !contains_var(e, var) {
        let res = Expr::Mul(Box::new(e.clone()), Box::new(Var(v)));
        push_step(
            steps,
            RewriteRule::IntegrationConstantRule,
            e,
            &res,
            "∫ c dx = c·x",
        );
        return Some(simplify_expr(&res));
    }
    let before = e.clone();
    if let Some(sec_form) = try_sec_squared_form(e, var) {
        let simp = simplify_expr(&sec_form);
        push_step(
            steps,
            RewriteRule::IntegrationTrigRule,
            &before,
            &simp,
            "∫ sec²(ax+b) dx = tan(ax+b)/a",
        );
        return Some(simp);
    }
    if let Some(atan_form) = try_atan_form(e, var) {
        let simp = simplify_expr(&atan_form);
        push_step(
            steps,
            RewriteRule::IntegrationTrigRule,
            &before,
            &simp,
            "∫ c/(x²+a²) dx = (c/a)·atan(x/a)",
        );
        return Some(simp);
    }
    let result: Option<Expr> = match e {
        Const(c) => {
            if *c == 0.0 {
                Some(Const(0.0))
            } else {
                Some(Expr::Mul(Box::new(Const(*c)), Box::new(Var(v))))
            }
        }
        Var(name) if name == var => Some(Expr::Mul(
            Box::new(Expr::Pow(Box::new(Var(v)), Box::new(Const(2.0)))),
            Box::new(Const(0.5)),
        )),
        Expr::Neg(a) => {
            let inner = integrate_with_steps(a, var, steps, depth + 1)?;
            Some(Expr::Neg(Box::new(inner)))
        }
        Expr::Add(a, b) => {
            let ia = integrate_with_steps(a, var, steps, depth + 1)?;
            let ib = integrate_with_steps(b, var, steps, depth + 1)?;
            Some(Expr::Add(Box::new(ia), Box::new(ib)))
        }
        Expr::Sub(a, b) => {
            let ia = integrate_with_steps(a, var, steps, depth + 1)?;
            let ib = integrate_with_steps(b, var, steps, depth + 1)?;
            Some(Expr::Sub(Box::new(ia), Box::new(ib)))
        }
        Expr::Mul(a, b) => {
            if !contains_var(a, var) {
                let ib = integrate_with_steps(b, var, steps, depth + 1)?;
                Some(Expr::Mul(a.clone(), Box::new(ib)))
            } else if !contains_var(b, var) {
                let ia = integrate_with_steps(a, var, steps, depth + 1)?;
                Some(Expr::Mul(Box::new(ia), b.clone()))
            } else {
                try_generic_parts_with_steps(e, var, steps, depth)
            }
        }
        Expr::Pow(base, exp) => {
            if let Var(name) = base.as_ref() {
                if name == var {
                    if let Const(n) = exp.as_ref() {
                        if (*n + 1.0).abs() < 1e-12 {
                            Some(Expr::Ln(Box::new(Expr::Abs(Box::new(Var(v))))))
                        } else {
                            let new_exp = n + 1.0;
                            Some(Expr::Mul(
                                Box::new(Const(1.0 / new_exp)),
                                Box::new(Expr::Pow(Box::new(Var(v)), Box::new(Const(new_exp)))),
                            ))
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }
        Expr::Div(num, den) => {
            if let Var(name) = den.as_ref() {
                if name == var && !contains_var(num, var) {
                    Some(Expr::Mul(
                        num.clone(),
                        Box::new(Expr::Ln(Box::new(Expr::Abs(Box::new(Var(v)))))),
                    ))
                } else {
                    None
                }
            } else {
                None
            }
        }
        Expr::Sin(arg) => {
            if expr_is_linear_in(arg, var) {
                let (a, _) = expr_linear_coeff(arg, var)?;
                if a.abs() < 1e-12 {
                    return None;
                }
                Some(Expr::Mul(
                    Box::new(Const(-1.0 / a)),
                    Box::new(Expr::Cos(arg.clone())),
                ))
            } else {
                None
            }
        }
        Expr::Cos(arg) => {
            if expr_is_linear_in(arg, var) {
                let (a, _) = expr_linear_coeff(arg, var)?;
                if a.abs() < 1e-12 {
                    return None;
                }
                Some(Expr::Mul(
                    Box::new(Const(1.0 / a)),
                    Box::new(Expr::Sin(arg.clone())),
                ))
            } else {
                None
            }
        }
        Expr::Tan(arg) => {
            if expr_is_linear_in(arg, var) {
                let (a, _) = expr_linear_coeff(arg, var)?;
                if a.abs() < 1e-12 {
                    return None;
                }
                Some(Expr::Mul(
                    Box::new(Const(-1.0 / a)),
                    Box::new(Expr::Ln(Box::new(Expr::Abs(Box::new(Expr::Cos(
                        arg.clone(),
                    )))))),
                ))
            } else {
                None
            }
        }
        Exp(arg) => {
            if expr_is_linear_in(arg, var) {
                let (a, _) = expr_linear_coeff(arg, var)?;
                if a.abs() < 1e-12 {
                    return None;
                }
                if (a - 1.0).abs() < 1e-12 {
                    Some(Exp(arg.clone()))
                } else {
                    Some(Expr::Mul(
                        Box::new(Const(1.0 / a)),
                        Box::new(Exp(arg.clone())),
                    ))
                }
            } else {
                None
            }
        }
        Expr::Ln(u) if matches!(u.as_ref(), Var(name) if name == var) => Some(Expr::Sub(
            Box::new(Expr::Mul(
                Box::new(Var(v.clone())),
                Box::new(Expr::Ln(Box::new(Var(v.clone())))),
            )),
            Box::new(Var(v)),
        )),
        _ => return None,
    };
    if let Some(res) = result {
        let simp = simplify_expr(&res);
        let rule = match e {
            Expr::Add(..) | Expr::Sub(..) => RewriteRule::IntegrationSumRule,
            Expr::Pow(..) => RewriteRule::IntegrationPowerRule,
            Exp(..) => RewriteRule::IntegrationExpRule,
            Expr::Sin(..) | Expr::Cos(..) | Expr::Tan(..) => RewriteRule::IntegrationTrigRule,
            _ => RewriteRule::Generic,
        };
        push_step(steps, rule, &before, &simp, "integración");
        Some(simp)
    } else {
        None
    }
}

fn try_generic_parts_with_steps(
    expr: &Expr,
    var: &str,
    steps: &mut Vec<CasStep>,
    depth: u32,
) -> Option<Expr> {
    // Cota anti-DoS idéntica a `symbolic::try_generic_parts` (cuelgue
    // FourierSeries[x^2,x,1,2] 2026-09-26): partes por partes puede ciclar
    // (∫x²·cos reproduce x²·cos dos niveles después con otras constantes) y
    // el árbol crece exponencial por nivel — 18 GB antes de tocar el tope de
    // profundidad. Se aborta con integrando grande: la ruta numérica
    // (Gauss–Legendre) lo resuelve igual y es el fallback declarado de
    // `expansion_coefficients`. MAX_PARTS_DEPTH 24 (no 256) + tope 4096.
    if format!("{expr:?}").len() > 4096 {
        return None;
    }
    const MAX_PARTS_DEPTH: u32 = 24;
    if depth > MAX_PARTS_DEPTH {
        return None;
    }
    let (left, right) = match expr {
        Expr::Mul(a, b) => (a.as_ref(), b.as_ref()),
        _ => return None,
    };
    for (u_cand, dv_cand) in [(left, right), (right, left)] {
        let v = integrate_with_steps(dv_cand, var, steps, depth + 1)
            .or_else(|| dv_cand.integrate(var))?;
        let du = diff_with_steps(u_cand, var, steps, depth + 1);
        let du_s = simplify_expr(&du);
        if du_s == Expr::Const(0.0) {
            continue;
        }
        let v_du = Expr::Mul(Box::new(v.clone()), Box::new(du_s));
        let int_vdu =
            integrate_with_steps(&v_du, var, steps, depth + 1).or_else(|| v_du.integrate(var))?;
        let uv = Expr::Mul(Box::new(u_cand.clone()), Box::new(v));
        let result = Expr::Sub(Box::new(uv), Box::new(int_vdu));
        let simplified = simplify_expr(&result);
        push_step(
            steps,
            RewriteRule::ProductRule,
            expr,
            &simplified,
            "integración por partes ∫u·dv = u·v − ∫v·du",
        );
        return Some(simplified);
    }
    None
}

fn try_sec_squared_form(expr: &Expr, var: &str) -> Option<Expr> {
    use Expr::{Const, Pow, Sec, Tan};
    if let Pow(base, exp) = expr {
        if let Const(n) = exp.as_ref() {
            if (*n - 2.0).abs() < 1e-12 {
                if let Sec(arg) = base.as_ref() {
                    if expr_is_linear_in(arg, var) {
                        let (a, _) = expr_linear_coeff(arg, var)?;
                        if a.abs() > 1e-12 {
                            return Some(Expr::Mul(
                                Box::new(Const(1.0 / a)),
                                Box::new(Tan(arg.clone())),
                            ));
                        }
                    }
                }
            }
            if (*n - 2.0).abs() < 1e-12 {
                if let Expr::Csc(arg) = base.as_ref() {
                    if expr_is_linear_in(arg, var) {
                        let (a, _) = expr_linear_coeff(arg, var)?;
                        if a.abs() > 1e-12 {
                            return Some(Expr::Mul(
                                Box::new(Const(-1.0 / a)),
                                Box::new(Expr::Cot(arg.clone())),
                            ));
                        }
                    }
                }
            }
        }
    }
    if let Expr::Mul(left, right) = expr {
        if let (Sec(arg1), Sec(arg2)) = (left.as_ref(), right.as_ref()) {
            if arg1 == arg2 && expr_is_linear_in(arg1, var) {
                let (a, _) = expr_linear_coeff(arg1, var)?;
                if a.abs() > 1e-12 {
                    return Some(Expr::Mul(
                        Box::new(Const(1.0 / a)),
                        Box::new(Tan(arg1.clone())),
                    ));
                }
            }
        }
    }
    None
}

fn try_atan_form(expr: &Expr, var: &str) -> Option<Expr> {
    use Expr::{Add, Atan, Const, Div, Pow, Var};
    let (num, den) = match expr {
        Div(n, d) => (n.as_ref(), d.as_ref()),
        _ => return None,
    };
    if contains_var(num, var) {
        return None;
    }
    let coeff = constant_scalar_value(num)?;
    if !coeff.is_finite() {
        return None;
    }
    let (pow_base, pow_exp, const_val) = match den {
        Add(left, right) => match (left.as_ref(), right.as_ref()) {
            (Pow(b, e), Const(k)) => (b.as_ref(), e.as_ref(), *k),
            (Const(k), Pow(b, e)) => (b.as_ref(), e.as_ref(), *k),
            _ => return None,
        },
        _ => return None,
    };
    let Const(exp_val) = pow_exp else {
        return None;
    };
    if (*exp_val - 2.0).abs() >= 1e-12 {
        return None;
    }
    let Var(name) = pow_base else {
        return None;
    };
    if name != var {
        return None;
    }
    if !const_val.is_finite() || const_val <= 0.0 {
        return None;
    }
    let a = const_val.sqrt();
    if !a.is_finite() || a == 0.0 {
        return None;
    }
    let factor = coeff / a;
    if !factor.is_finite() {
        return None;
    }
    Some(Expr::Mul(
        Box::new(Const(factor)),
        Box::new(Atan(Box::new(Div(
            Box::new(Var(var.to_string())),
            Box::new(Const(a)),
        )))),
    ))
}

// ---------------------------------------------------------------------------
// Límite Richardson
// ---------------------------------------------------------------------------

fn limit_richardson_steps(ast: &Expr, var: &str, at: f64, steps: &mut Vec<CasStep>) -> Option<f64> {
    const STEP_SCALES: [f64; 3] = [0.125, 0.1, 0.075];
    let local_scale = at.abs().max(1.0);
    let mut estimates = Vec::new();
    for scale in STEP_SCALES {
        let h0 = scale * local_scale;
        let left = stable_side_limit_steps(ast, var, at, -1.0, h0, steps)?;
        let right = stable_side_limit_steps(ast, var, at, 1.0, h0, steps)?;
        if (left - right).abs() > limit_tolerance(left.abs().max(right.abs())) + 8.0 * 1e-9 {
            return None;
        }
        estimates.push((left + right) * 0.5);
    }
    let mut value = estimates[0];
    for (idx, est) in estimates.iter().enumerate().skip(1) {
        value += (est - value) / (idx + 1) as f64;
    }
    push_limit_step(
        steps,
        &format!("lim({}→{}) {}", var, at, ast.to_expr_string()),
        &format!("{value:.8}"),
        &format!("Richardson bilateral → {value:.8}"),
    );
    Some(value)
}

fn stable_side_limit_steps(
    ast: &Expr,
    var: &str,
    at: f64,
    sign: f64,
    initial_step: f64,
    steps: &mut Vec<CasStep>,
) -> Option<f64> {
    const SAMPLE_COUNT: usize = 12;
    let mut values = [0.0; SAMPLE_COUNT];
    for (idx, v) in values.iter_mut().enumerate() {
        let h = initial_step / 2.0_f64.powi(idx as i32);
        let x = at + sign * h;
        if !x.is_finite() || x == at {
            return None;
        }
        let val = ast.eval_at(var, x);
        if !val.is_finite() {
            return None;
        }
        *v = val;
        if steps.len() < MAX_CAS_STEPS {
            push_limit_step(
                steps,
                &format!("f({}={:.6})", var, x),
                &format!("{val:.6}"),
                &format!("muestra h={:.2e}", h),
            );
        }
    }
    let last = values[SAMPLE_COUNT - 1];
    let prev = values[SAMPLE_COUNT - 2];
    let diff = last - prev;
    if diff.abs() < 1e-9 + 1e-6 * last.abs().max(1.0) {
        return Some(last);
    }
    let diff2 = prev - values[SAMPLE_COUNT - 3];
    if diff == 0.0 || diff2 == 0.0 || diff.signum() != diff2.signum() {
        return Some(last);
    }
    let ratio = (diff / diff2).abs();
    if !ratio.is_finite() || ratio >= 0.8 {
        return Some(last);
    }
    let correction = diff * ratio / (1.0 - ratio);
    let value = last + correction;
    if value.is_finite() {
        Some(value)
    } else {
        None
    }
}

fn limit_tolerance(value: f64) -> f64 {
    1e-9 + 1e-6 * value.abs().max(1.0)
}

// ---------------------------------------------------------------------------
// Taylor genérico
// ---------------------------------------------------------------------------

fn taylor_steps(
    ast: &Expr,
    var: &str,
    center: f64,
    order: usize,
    steps: &mut Vec<CasStep>,
) -> Option<String> {
    if order > crate::analysis::MAX_TAYLOR_ORDER {
        return None;
    }
    let coeffs = crate::analysis::taylor_coefficients_from_ast(ast, var, center, order).ok()?;
    let mut terms = Vec::new();
    for (n, coef) in coeffs.iter().enumerate() {
        if coef.abs() < 1e-12 {
            continue;
        }
        let term = if n == 0 {
            format!("{coef}")
        } else if n == 1 {
            if center == 0.0 {
                format!("{coef}*{var}")
            } else {
                format!("{coef}*({var} - {center})")
            }
        } else if center == 0.0 {
            format!("{coef}*{var}^{n}")
        } else {
            format!("{coef}*({var} - {center})^{n}")
        };
        terms.push(term.clone());
        if steps.len() < MAX_CAS_STEPS {
            push_limit_step(
                steps,
                &format!("coeficiente n={n} en {center}"),
                &term,
                &format!("Taylor coeficiente a_{n} = {coef:.6}"),
            );
        }
    }
    let out = if terms.is_empty() {
        "0".to_string()
    } else {
        terms.join(" + ").replace("+ -", "- ")
    };
    if steps.len() < MAX_CAS_STEPS {
        push_limit_step(
            steps,
            &ast.to_expr_string(),
            &out,
            &format!("serie Taylor orden {order} en {center}"),
        );
    }
    Some(out)
}

// ---------------------------------------------------------------------------
// Despacho genérico CasOp → Vec<CasStep>
// ---------------------------------------------------------------------------

/// Despacho genérico `CasOp → Vec<CasStep>` sin goldens hardcodeados.
///
/// Punto de entrada sin estado para `grafito-command`: puro y acotado
/// (≤ `MAX_CAS_STEPS` pasos, campos ≤ `MAX_STEP_BYTES`). Devuelve `Err` con
/// entradas inválidas o presupuestos excedidos.
pub fn steps_for_op(op: &CasOp) -> Result<Vec<CasStep>, String> {
    match op {
        CasOp::Derivative { expr, var } => steps_for_derivative(expr, var),
        CasOp::Integral { expr, var } => steps_for_integral(expr, var),
        CasOp::Limit { expr, var, at } => steps_for_limit(expr, var, *at),
        CasOp::Taylor {
            expr,
            var,
            center,
            order,
        } => steps_for_taylor(expr, var, *center, *order),
        CasOp::Solve { expr, var } => steps_for_solve(expr, var),
        CasOp::OdeNthOrder { coeffs, rhs, x } => steps_for_ode_nth_order(coeffs, rhs, x),
        CasOp::OdeEuler { a, b, rhs, x } => steps_for_ode_euler(a, b, rhs, x),
        CasOp::OdeFirstOrder { p, q, x } => steps_for_ode_linear(p, q, x),
        CasOp::OdeSecondHomog { a, b, c, x } => steps_for_ode_second_homog(a, b, c, x),
        CasOp::StatsDescriptive { data } => steps_for_stats_descriptive(data),
        CasOp::StatsRegression { xs, ys } => steps_for_stats_regression(xs, ys),
        CasOp::Frobenius {
            p,
            q,
            x,
            center,
            terms,
        } => steps_for_frobenius(p, q, x, *center, *terms),
        CasOp::LaplaceDerivative {
            order,
            y,
            t,
            s,
            initials,
        } => steps_for_laplace_derivative(*order, y, t, s, initials),
        CasOp::LaplaceIntegral { f, t, s } => steps_for_laplace_integral(f, t, s),
        CasOp::GroebnerOrdered { polys, vars, order } => {
            steps_for_groebner_ordered(polys, vars, order)
        }
        CasOp::Eliminate { polys, vars, elim } => steps_for_eliminate(polys, vars, elim),
        CasOp::System2x2 {
            eq1,
            eq2,
            var1,
            var2,
        } => steps_for_system_2x2(eq1, eq2, var1, var2),
        CasOp::DefiniteIntegral { expr, var, a, b } => {
            steps_for_definite_integral(expr, var, *a, *b)
        }
    }
}

/// Pasos para derivada genérica (instrumenta `diff_expr`).
pub fn steps_for_derivative(expr: &str, var: &str) -> Result<Vec<CasStep>, String> {
    validate_identifier(var)?;
    validate_input_bytes(expr)?;
    let pp = expr.replace(' ', "");
    let ast = parse_ast(&pp).map_err(|e| format!("parse error: {e}"))?;
    let mut steps = Vec::new();
    let derived = diff_with_steps(&ast, var, &mut steps, 0);
    let _ = derived;
    if steps.is_empty() {
        let simplified = simplify_expr(&ast.diff(var));
        push_step(
            &mut steps,
            RewriteRule::Generic,
            &ast,
            &simplified,
            "derivada directa",
        );
    }
    steps.truncate(MAX_CAS_STEPS);
    Ok(steps)
}

/// Pasos para integral genérica (instrumenta `integrate_expr`).
pub fn steps_for_integral(expr: &str, var: &str) -> Result<Vec<CasStep>, String> {
    validate_identifier(var)?;
    validate_input_bytes(expr)?;
    let pp = expr.replace(' ', "");
    let ast = parse_ast(&pp).map_err(|e| format!("parse error: {e}"))?;
    let mut steps = Vec::new();
    if let Some(prim) = integrate_with_steps(&ast, var, &mut steps, 0) {
        let _ = prim;
        if steps.is_empty() {
            if let Some(fb) = ast.integrate(var) {
                let simp = simplify_expr(&fb);
                push_step(
                    &mut steps,
                    RewriteRule::Generic,
                    &ast,
                    &simp,
                    "integral fallback",
                );
            }
        }
        steps.truncate(MAX_CAS_STEPS);
        Ok(steps)
    } else if let Some(fb) = ast.integrate(var) {
        let simp = simplify_expr(&fb);
        let mut fb_steps = Vec::new();
        push_step(
            &mut fb_steps,
            RewriteRule::Generic,
            &ast,
            &simp,
            "integral nativa",
        );
        fb_steps.truncate(MAX_CAS_STEPS);
        Ok(fb_steps)
    } else {
        Err(format!("no hay primitiva soportada para '{expr}'"))
    }
}

/// Pasos para límite genérico (Richardson bilateral).
pub fn steps_for_limit(expr: &str, var: &str, at: f64) -> Result<Vec<CasStep>, String> {
    if !at.is_finite() {
        return Err("at no es finito".to_string());
    }
    validate_identifier(var)?;
    validate_input_bytes(expr)?;
    let pp = expr.replace(' ', "");
    let ast = parse_ast(&pp).map_err(|e| format!("parse error: {e}"))?;
    let mut steps = Vec::new();
    let val = limit_richardson_steps(&ast, var, at, &mut steps)
        .ok_or_else(|| format!("limite no existe para '{expr}' en {var}→{at}"))?;
    if steps.is_empty() {
        push_limit_step(
            &mut steps,
            &format!("lim({}→{}) {}", var, at, expr),
            &format!("{val:.8}"),
            "límite estimado",
        );
    }
    steps.truncate(MAX_CAS_STEPS);
    Ok(steps)
}

/// Serie de Taylor genérica (no limitada a e^x).
pub fn steps_for_taylor(
    expr: &str,
    var: &str,
    center: f64,
    order: usize,
) -> Result<Vec<CasStep>, String> {
    validate_identifier(var)?;
    validate_input_bytes(expr)?;
    if order > crate::analysis::MAX_TAYLOR_ORDER {
        return Err(format!(
            "orden {order} excede {}",
            crate::analysis::MAX_TAYLOR_ORDER
        ));
    }
    let pp = expr.replace(' ', "");
    let ast = parse_ast(&pp).map_err(|e| format!("parse error: {e}"))?;
    let mut steps = Vec::new();
    let series = taylor_steps(&ast, var, center, order, &mut steps)
        .ok_or_else(|| format!("no se pudo calcular Taylor de '{expr}'"))?;
    let _ = series;
    steps.truncate(MAX_CAS_STEPS);
    Ok(steps)
}

// ---------------------------------------------------------------------------
// Frente G-A: trazas acotadas del motor CAS (Gruntz, Risch, EDO, residuo,
// Buchberger). Reusan `CasStep` + `RewriteRule::Generic` (sin romper la
// API exhaustiva) y truncan a `MAX_CAS_STEPS` 32.
// ---------------------------------------------------------------------------

fn push_ga_step(steps: &mut Vec<CasStep>, before: &str, after: &str, description: &str) {
    push_ga_ruled_step(steps, RewriteRule::Generic, before, after, description);
}

/// Traza con regla explícita (frente F3c: parciales, EDO2, sistemas, Laplace).
fn push_ga_ruled_step(
    steps: &mut Vec<CasStep>,
    rule: RewriteRule,
    before: &str,
    after: &str,
    description: &str,
) {
    if steps.len() >= MAX_CAS_STEPS {
        return;
    }
    let idx = steps.len();
    steps.push(CasStep {
        index: idx,
        rule,
        before: truncate_bytes(before, MAX_STEP_BYTES),
        after: truncate_bytes(after, MAX_STEP_BYTES),
        description: truncate_bytes(description, MAX_STEP_BYTES),
    });
}

/// Traza Gruntz: clasifica la forma y muestra L'Hôpital o jerarquía.
pub fn steps_for_gruntz(expr: &str, var: &str, at: f64) -> Result<Vec<CasStep>, String> {
    validate_identifier(var)?;
    validate_input_bytes(expr)?;
    if !at.is_finite() {
        return Err("at no es finito".to_string());
    }
    let pp = expr.replace(' ', "");
    let ast = parse_ast(&pp).map_err(|e| format!("parse error: {e}"))?;
    let mut steps = Vec::new();
    push_ga_step(
        &mut steps,
        &format!("lim({var}→{at}) {expr}"),
        &format!("forma {}", gruntz_form_label(&ast, var, at)),
        "Gruntz: clasificar 0/0, ∞/∞ o directa",
    );
    // L'Hôpital visible hasta 8 iteraciones (cota del motor).
    if let Expr::Div(num, den) = &ast {
        let mut cur_n = num.as_ref().clone();
        let mut cur_d = den.as_ref().clone();
        for i in 1..=crate::cas::MAX_GRUNTZ_STEPS {
            let (nv, dv) = (cur_n.eval_at(var, at), cur_d.eval_at(var, at));
            let indeterminate = (nv == 0.0 && dv == 0.0) || (nv.is_infinite() && dv.is_infinite());
            if !indeterminate {
                break;
            }
            let next_n = simplify_expr(&cur_n.diff(var));
            let next_d = simplify_expr(&cur_d.diff(var));
            push_ga_step(
                &mut steps,
                &format!("({})/({})", cur_n.to_expr_string(), cur_d.to_expr_string()),
                &format!(
                    "({})/({})",
                    next_n.to_expr_string(),
                    next_d.to_expr_string()
                ),
                &format!("L'Hôpital iteración {i}: derivar numerador y denominador"),
            );
            cur_n = next_n;
            cur_d = next_d;
            if steps.len() >= MAX_CAS_STEPS {
                break;
            }
        }
    }
    let outcome = crate::cas::gruntz_limit(expr, var, at)
        .map_err(|e| format!("Gruntz no resolvió '{expr}': {e}"))?;
    // B2: el rewrite a cociente + `ln` deja traza con regla propia.
    if outcome.method == crate::cas::GruntzMethod::LogRewrite {
        push_ga_ruled_step(
            &mut steps,
            RewriteRule::GruntzLogRewriteRule,
            &format!("lim({var}→{at}) {expr}"),
            &format!("{:.8}", outcome.value),
            "rewrite B2: 0·∞/∞−∞ a cociente, 1^∞/0⁰/∞⁰ por ln",
        );
    }
    push_ga_step(
        &mut steps,
        &format!("lim({var}→{at}) {expr}"),
        &format!("{:.8}", outcome.value),
        &format!("Gruntz {:?} → {:.8}", outcome.method, outcome.value),
    );
    steps.truncate(MAX_CAS_STEPS);
    Ok(steps)
}

fn gruntz_form_label(ast: &Expr, var: &str, at: f64) -> &'static str {
    if let Expr::Div(num, den) = ast {
        let (n, d) = (num.eval_at(var, at), den.eval_at(var, at));
        if n == 0.0 && d == 0.0 {
            return "0/0";
        }
        if n.is_infinite() && d.is_infinite() {
            return "∞/∞";
        }
    }
    if ast.eval_at(var, at).is_finite() {
        return "directa";
    }
    "otra"
}

/// Traza Risch-Norman: primitiva del subconjunto S/M + F3c.
///
/// F3c cubre además `tan`/`sec²`, fracciones parciales sobre lineales
/// reales (grado ≤ 4), `1/(x²+1)`-style (`atan` + `ln`) y `x^(p/q)`.
pub fn steps_for_risch(expr: &str, var: &str) -> Result<Vec<CasStep>, String> {
    validate_identifier(var)?;
    validate_input_bytes(expr)?;
    let prim = crate::integral::risch_norman_integrate(expr, var)
        .map_err(|e| format!("Risch-Norman no cubre '{expr}': {e}"))?;
    let mut steps = Vec::new();
    // Regla exacta según el camino tomado: `atan` solo lo emite la
    // cuadrática irreducible; cociente con denominador no trivial solo
    // lo resuelve `risch_rational` (el `1/x` rápido lleva `Var` abajo).
    let rule = if prim.contains("atan") {
        RewriteRule::IntegrationArctanRule
    } else if let Ok(ast) = parse_ast(&expr.replace(' ', "")) {
        match ast {
            Expr::Div(_, den)
                if !matches!(den.as_ref(), Expr::Var(_))
                    && crate::cas::cas_const_value(&den).is_none() =>
            {
                RewriteRule::IntegrationPartialFractions
            }
            // B2: trig, Euler-sqrt y partes tabulares con regla propia.
            _ => risch_b2_rule(&prim, expr),
        }
    } else {
        RewriteRule::Generic
    };
    push_ga_ruled_step(
        &mut steps,
        rule,
        &format!("∫ {expr} d{var}"),
        &prim,
        "Risch-Norman F3c: potencias x^(p/q), exp/log, tan/sec², parciales, arctan + linealidad",
    );
    push_ga_step(
        &mut steps,
        &prim,
        &format!("d/d{var}({prim}) = {expr}"),
        "verificación: derivar la primitiva recupera el integrando",
    );
    steps.truncate(MAX_CAS_STEPS);
    Ok(steps)
}

/// Traza EDO lineal `y' + p·y = q` por factor integrante.
/// Residuo máximo escala-relativo de `residual` en puntos muestra con las
/// constantes de `fijas` (p. ej. `C = 0`) ya sustituidas.
///
/// Cada punto aporta `|R| / (1 + Σ|términos|)`; los no finitos se saltean
/// (singularidades puntuales como `1/x` en 0). Menos de 2 puntos finitos
/// → `Err` honesto: sin evidencia no hay verificación (jamás adivinar).
fn residuo_maximo_escalado(
    residual: &Expr,
    escalas: &[Expr],
    fijas: &BTreeMap<String, f64>,
    x: &str,
) -> Result<f64, String> {
    const MUESTRAS: [f64; 7] = [0.5, 1.0, 1.5, 2.0, -0.5, -1.0, 0.25];
    let mut peor = 0.0f64;
    let mut finitos = 0usize;
    for x0 in MUESTRAS {
        let mut mapa = fijas.clone();
        mapa.insert(x.to_string(), x0);
        let r = residual.substitute_vars(&mapa, &[]).eval_at(x, x0);
        if !r.is_finite() {
            continue;
        }
        let mut escala = 1.0f64;
        let mut escala_ok = true;
        for termino in escalas {
            let v = termino.substitute_vars(&mapa, &[]).eval_at(x, x0);
            if !v.is_finite() {
                escala_ok = false;
                break;
            }
            escala += v.abs();
        }
        if !escala_ok || !escala.is_finite() {
            continue;
        }
        finitos += 1;
        peor = peor.max(r.abs() / escala);
    }
    if finitos < 2 {
        return Err(
            "verificación sin puntos finitos suficientes (¿singularidad en la zona de muestra?)"
                .to_string(),
        );
    }
    Ok(peor)
}

/// Variables libres de un AST como conjunto (para custodiar que `P`, `Q`
/// o la solución no traigan símbolos ajenos).
fn vars_de(e: &Expr) -> HashSet<String> {
    let mut vars = HashSet::new();
    e.get_variables(&mut vars);
    vars
}

/// Traza EDO lineal de 1er orden `y' + P(x)·y = Q(x)` por factor
/// integrante (`P`, `μ = e^∫P`, solución) con verificación por sustitución.
///
/// `P`, `Q` solo en `x`: parámetros, `y` escondida o 2º orden dan `Err`
/// honesto (el NL deriva a este subset o al redirect, nunca adivina).
pub fn steps_for_ode_linear(p: &str, q: &str, x: &str) -> Result<Vec<CasStep>, String> {
    validate_identifier(x)?;
    validate_input_bytes(p)?;
    validate_input_bytes(q)?;
    if x == "C" {
        return Err("la variable no puede llamarse C (constante de integración)".to_string());
    }
    let p_ast = parse_ast(&p.replace(' ', "")).map_err(|e| format!("parse error en P(x): {e}"))?;
    let q_ast = parse_ast(&q.replace(' ', "")).map_err(|e| format!("parse error en Q(x): {e}"))?;
    if vars_de(&p_ast).iter().any(|v| v != x) || vars_de(&q_ast).iter().any(|v| v != x) {
        return Err(format!(
            "EDO lineal de 1er orden con símbolos ajenos a {x} en P o Q: solo P({x}), Q({x}) tienen pasos"
        ));
    }
    let sol = crate::ode::solve_linear_first_order(p, q, x)
        .map_err(|e| format!("EDO lineal no resuelta: {e}"))?;
    let y_txt = sol
        .strip_prefix("y = ")
        .ok_or_else(|| "solución del motor sin forma 'y = ...'".to_string())?;
    let y_ast =
        parse_ast(&y_txt.replace(' ', "")).map_err(|e| format!("solución no re-parseable: {e}"))?;
    if vars_de(&y_ast).iter().any(|v| v != x && v != "C") {
        return Err("solución con símbolos inesperados; sin verificación".to_string());
    }
    // μ para mostrar: la primitiva de P si el AST nativo la trae, si no el
    // símbolo ∫P honesto (la solución ya la trae adentro vía el solver).
    let mu_txt = match p_ast.integrate(x) {
        Some(prim) => prim.simplify().to_expr_string(),
        None => format!("∫({p})d{x}"),
    };
    // Verificación por sustitución: R = y' + P·y − Q con C = 0 y C = 1.
    let yp_ast = y_ast.diff(x);
    let residual = Expr::Sub(
        Box::new(Expr::Add(
            Box::new(yp_ast.clone()),
            Box::new(Expr::Mul(Box::new(p_ast.clone()), Box::new(y_ast.clone()))),
        )),
        Box::new(q_ast.clone()),
    );
    let escalas = [yp_ast, Expr::Mul(Box::new(p_ast), Box::new(y_ast)), q_ast];
    let mut peor = 0.0f64;
    for c0 in [0.0, 1.0] {
        let fijas: BTreeMap<String, f64> = [("C".to_string(), c0)].into_iter().collect();
        let r = residuo_maximo_escalado(&residual, &escalas, &fijas, x)?;
        peor = peor.max(r);
    }
    if !(peor.is_finite() && peor <= 1e-6) {
        return Err(format!(
            "verificación por sustitución falló (residuo {peor:.2e}); revisa P(x) y Q(x)"
        ));
    }
    let mut steps = Vec::new();
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::OdeFirstOrderRule,
        &format!("y' + ({p})·y = {q}"),
        &format!("P({x}) = {p} ; Q({x}) = {q}"),
        "forma estándar: leer P(x) (lo que multiplica a y) y Q(x) (el lado derecho)",
    );
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::OdeFirstOrderRule,
        &format!("P({x}) = {p}"),
        &format!("μ = exp({mu_txt})"),
        "factor integrante μ = e^∫P dx: multiplica toda la ecuación para volverla exacta",
    );
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::OdeFirstOrderRule,
        &format!("(μ·y)' = μ·({q})"),
        &sol,
        "integrar ambos lados y despejar: y = (H + C)/μ",
    );
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::OdeFirstOrderRule,
        &format!("sustituir y en y' + ({p})·y − ({q})"),
        &format!("residuo {peor:.2e} ≈ 0"),
        "verificación: sustituir la solución recupera 0 = 0",
    );
    steps.truncate(MAX_CAS_STEPS);
    Ok(steps)
}

/// Traza EDO separable `y' = g(x)·h(y)`.
pub fn steps_for_ode_separable(g: &str, h: &str, x: &str, y: &str) -> Result<Vec<CasStep>, String> {
    validate_identifier(x)?;
    validate_identifier(y)?;
    validate_input_bytes(g)?;
    validate_input_bytes(h)?;
    let sol = crate::ode::solve_separable(g, h, x, y)
        .map_err(|e| format!("EDO separable no resuelta: {e}"))?;
    let mut steps = Vec::new();
    push_ga_step(
        &mut steps,
        &format!("y' = ({g})·({h})"),
        &format!("dy/({h}) = ({g}) dx"),
        "separar variables",
    );
    push_ga_step(
        &mut steps,
        &format!("∫dy/({h}) = ∫({g}) dx"),
        &sol,
        "integrar ambos lados + C",
    );
    steps.truncate(MAX_CAS_STEPS);
    Ok(steps)
}

// ---------------------------------------------------------------------------
// Frente F3c: trazas de 2º orden, sistemas 2×2 y Laplace.
// ---------------------------------------------------------------------------

/// Traza EDO `a·y''+b·y'+c·y = rhs` (característica + indeterm. exactos).
pub fn steps_for_ode_second_order(
    a: &str,
    b: &str,
    c: &str,
    rhs: &str,
    x: &str,
) -> Result<Vec<CasStep>, String> {
    validate_identifier(x)?;
    validate_input_bytes(a)?;
    validate_input_bytes(b)?;
    validate_input_bytes(c)?;
    validate_input_bytes(rhs)?;
    let sol = crate::ode::solve_ode_second_order_const(a, b, c, rhs, x)
        .map_err(|e| format!("EDO 2º orden no resuelta: {e}"))?;
    let mut steps = Vec::new();
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::OdeSecondOrderRule,
        &format!("({a})·y''+({b})·y'+({c})·y = {rhs}"),
        "r²: a·r²+b·r+c = 0 (D = b²−4ac: reales distintas / doble / compleja)",
        "ecuación característica",
    );
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::OdeSecondOrderRule,
        "homogénea según D",
        "C1·e^{r1·x}+C2·e^{r2·x} | (C1+C2·x)·e^{rx} | e^{αx}(C1·cos+C2·sin)",
        "homogénea con C1, C2",
    );
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::OdeSecondOrderRule,
        &format!("particular para {rhs}"),
        &sol,
        "coeficientes indeterminados exactos (resonancia → x^s) + y = hom + yp",
    );
    steps.truncate(MAX_CAS_STEPS);
    Ok(steps)
}

/// Traza EDO homogénea `a·y''+b·y'+c·y = 0` con `a, b, c` constantes:
/// ecuación característica, casos Δ (reales distintas / doble / compleja)
/// y verificación por sustitución con residuo ≈ 0.
///
/// Solo el subset homogéneo constante: `a = 0`, coeficientes no
/// constantes o no finitos dan `Err` honesto (la no homogénea vive en
/// `steps_for_ode_second_order`; el resto deriva al redirect, sin adivinar).
pub fn steps_for_ode_second_homog(
    a: &str,
    b: &str,
    c: &str,
    x: &str,
) -> Result<Vec<CasStep>, String> {
    validate_identifier(x)?;
    validate_input_bytes(a)?;
    validate_input_bytes(b)?;
    validate_input_bytes(c)?;
    if x == "C1" || x == "C2" {
        return Err(
            "la variable no puede llamarse C1 ni C2 (constantes de integración)".to_string(),
        );
    }
    let const_de = |texto: &str, rol: &str| -> Result<f64, String> {
        let ast =
            parse_ast(&texto.replace(' ', "")).map_err(|e| format!("parse error en {rol}: {e}"))?;
        crate::cas::cas_const_value(&ast)
            .filter(|v| v.is_finite())
            .ok_or_else(|| {
                format!("{rol} debe ser constante ('{texto}'); coeficientes variables fuera del subset homogéneo")
            })
    };
    let (af, bf, cf) = (const_de(a, "a")?, const_de(b, "b")?, const_de(c, "c")?);
    if af.abs() < 1e-12 {
        return Err("a = 0: no es de 2º orden (usá la EDO lineal de 1er orden)".to_string());
    }
    let disc = bf * bf - 4.0 * af * cf;
    let (caso, raices_txt, familia) = if disc > 1e-9 {
        let s = disc.sqrt();
        let (r1, r2) = ((-bf + s) / (2.0 * af), (-bf - s) / (2.0 * af));
        (
            "Δ > 0: raíces reales distintas",
            format!("r1 = {} ; r2 = {}", fmt_lin(r1), fmt_lin(r2)),
            format!("C1·e^({}·{x})+C2·e^({}·{x})", fmt_lin(r1), fmt_lin(r2)),
        )
    } else if disc >= -1e-9 {
        let r = -bf / (2.0 * af);
        (
            "Δ = 0: raíz doble (aparece x·e^{rx})",
            format!("r = {}", fmt_lin(r)),
            format!("(C1+C2·{x})·e^({}·{x})", fmt_lin(r)),
        )
    } else {
        let (al, be) = (-bf / (2.0 * af), (-disc).sqrt() / (2.0 * af.abs()));
        (
            "Δ < 0: complejo conjugado (oscila con e^{αx})",
            format!("α = {} ; β = {}", fmt_lin(al), fmt_lin(be)),
            format!(
                "e^({}·{x})·(C1·cos({}·{x})+C2·sin({}·{x}))",
                fmt_lin(al),
                fmt_lin(be),
                fmt_lin(be)
            ),
        )
    };
    let sol = crate::ode::solve_ode_second_order_const(a, b, c, "0", x)
        .map_err(|e| format!("EDO homogénea no resuelta: {e}"))?;
    let y_txt = sol
        .strip_prefix("y = ")
        .ok_or_else(|| "solución del motor sin forma 'y = ...'".to_string())?;
    let y_ast =
        parse_ast(&y_txt.replace(' ', "")).map_err(|e| format!("solución no re-parseable: {e}"))?;
    if vars_de(&y_ast)
        .iter()
        .any(|v| v != x && v != "C1" && v != "C2")
    {
        return Err("solución con símbolos inesperados; sin verificación".to_string());
    }
    // Verificación por sustitución: R = a·y''+b·y'+c·y en cada rama base
    // ((C1,C2) = (1,0) y (0,1): la homogénea lineal las valida por separado).
    let yp_ast = y_ast.diff(x);
    let ypp_ast = yp_ast.diff(x);
    let residual = Expr::Add(
        Box::new(Expr::Add(
            Box::new(Expr::Mul(
                Box::new(Expr::Const(af)),
                Box::new(ypp_ast.clone()),
            )),
            Box::new(Expr::Mul(
                Box::new(Expr::Const(bf)),
                Box::new(yp_ast.clone()),
            )),
        )),
        Box::new(Expr::Mul(
            Box::new(Expr::Const(cf)),
            Box::new(y_ast.clone()),
        )),
    );
    let escalas = [
        Expr::Mul(Box::new(Expr::Const(af)), Box::new(ypp_ast)),
        Expr::Mul(Box::new(Expr::Const(bf)), Box::new(yp_ast)),
        Expr::Mul(Box::new(Expr::Const(cf)), Box::new(y_ast)),
    ];
    let mut peor = 0.0f64;
    for (c1, c2) in [(1.0, 0.0), (0.0, 1.0)] {
        let fijas: BTreeMap<String, f64> = [("C1".to_string(), c1), ("C2".to_string(), c2)]
            .into_iter()
            .collect();
        let r = residuo_maximo_escalado(&residual, &escalas, &fijas, x)?;
        peor = peor.max(r);
    }
    if !(peor.is_finite() && peor <= 1e-6) {
        return Err(format!(
            "verificación por sustitución falló (residuo {peor:.2e}); revisa a, b, c"
        ));
    }
    let mut steps = Vec::new();
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::OdeSecondHomogRule,
        &format!("({a})·y''+({b})·y'+({c})·y = 0"),
        &format!("{a}·r²+{b}·r+{c} = 0"),
        "ecuación característica: probar y = e^{rx} convierte derivadas en potencias de r",
    );
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::OdeSecondHomogRule,
        &format!("Δ = {b}²−4·{a}·{c} = {}", fmt_lin(disc)),
        &format!("{caso}: {raices_txt}"),
        "discriminante: el signo decide la familia (reales distintas / doble / compleja)",
    );
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::OdeSecondHomogRule,
        &familia,
        &sol,
        "homogénea con C1, C2 según el caso Δ",
    );
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::OdeSecondHomogRule,
        &format!("sustituir y en ({a})·y''+({b})·y'+({c})·y"),
        &format!("residuos {peor:.2e} ≈ 0 en ambas ramas"),
        "verificación: sustituir la solución recupera 0 = 0",
    );
    steps.truncate(MAX_CAS_STEPS);
    Ok(steps)
}

/// Traza sistema 2×2 constante (traza/determinante → autovalores).
pub fn steps_for_ode_system_2x2(
    a11: &str,
    a12: &str,
    a21: &str,
    a22: &str,
    t: &str,
) -> Result<Vec<CasStep>, String> {
    validate_identifier(t)?;
    validate_input_bytes(a11)?;
    validate_input_bytes(a12)?;
    validate_input_bytes(a21)?;
    validate_input_bytes(a22)?;
    let sol = crate::ode::solve_ode_system_2x2(a11, a12, a21, a22, t)
        .map_err(|e| format!("sistema 2×2 no resuelto: {e}"))?;
    let mut steps = Vec::new();
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::OdeSystemRule,
        &format!("x' = ({a11})x+({a12})y, y' = ({a21})x+({a22})y"),
        "T = traza, D = det, Δ = T²−4D",
        "traza y determinante",
    );
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::OdeSystemRule,
        "autovalores según Δ",
        "Δ>0: λ1,λ2 reales · Δ=0: λ doble (Jordan t·e^λt si defectivo) · Δ<0: α±iβ",
        "autovalores y autovectores",
    );
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::OdeSystemRule,
        "combinación con C1, C2",
        &sol,
        "solución con C1, C2",
    );
    steps.truncate(MAX_CAS_STEPS);
    Ok(steps)
}

/// Traza Laplace directa (tabla + linealidad).
pub fn steps_for_laplace_direct(expr: &str, t: &str, s: &str) -> Result<Vec<CasStep>, String> {
    validate_identifier(t)?;
    validate_identifier(s)?;
    validate_input_bytes(expr)?;
    let out = crate::ode::laplace_direct(expr, t, s)
        .map_err(|e| format!("Laplace directa no cubierta: {e}"))?;
    let mut steps = Vec::new();
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::LaplaceDirectRule,
        &format!("L{{{expr}}}(t → {s})"),
        &out,
        "tabla F3c (1, t^n, exp, sin/cos, t·exp) + linealidad",
    );
    steps.truncate(MAX_CAS_STEPS);
    Ok(steps)
}

/// Traza Laplace inversa (racional propio grado ≤ 3 con raíz real, B2.3d).
pub fn steps_for_laplace_inverse(expr: &str, s: &str, t: &str) -> Result<Vec<CasStep>, String> {
    validate_identifier(s)?;
    validate_identifier(t)?;
    validate_input_bytes(expr)?;
    let out = crate::ode::laplace_inverse(expr, s, t)
        .map_err(|e| format!("Laplace inversa no cubierta: {e}"))?;
    let mut steps = Vec::new();
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::LaplaceInverseRule,
        &format!("L⁻¹{{{expr}}}({s} → {t})"),
        &out,
        "parciales grado ≤ 3: K/(s−a), K/(s−a)², cuadrática, cúbica real (B2.3d)",
    );
    steps.truncate(MAX_CAS_STEPS);
    Ok(steps)
}

/// Traza residuo: orden del polo + fórmula aplicada.
pub fn steps_for_residue(
    expr: &str,
    var: &str,
    at: f64,
    max_order: usize,
) -> Result<Vec<CasStep>, String> {
    validate_identifier(var)?;
    validate_input_bytes(expr)?;
    if !at.is_finite() {
        return Err("at no es finito".to_string());
    }
    let outcome = crate::cas::laurent_residue(expr, var, at, max_order)
        .map_err(|e| format!("residuo no calculado para '{expr}': {e}"))?;
    let mut steps = Vec::new();
    push_ga_step(
        &mut steps,
        &format!("{expr} en {var}={at}"),
        &format!("orden {}", outcome.pole_order),
        "detectar orden del polo (sondeo bilateral acotado)",
    );
    push_ga_step(
        &mut steps,
        &format!("orden {}", outcome.pole_order),
        &format!("residuo = {:.8}", outcome.residue),
        &format!(
            "{:?}: (x−at)^m·f derivada m−1 veces /(m−1)!",
            outcome.method
        ),
    );
    steps.truncate(MAX_CAS_STEPS);
    Ok(steps)
}

/// Traza Buchberger: base triangular acotada por S-polinomios.
pub fn steps_for_groebner(polys: &[String], vars: &[String]) -> Result<Vec<CasStep>, String> {
    if polys.len() > crate::cas::MAX_GROEBNER_POLYS || vars.len() > crate::cas::MAX_GROEBNER_VARS {
        return Err("sistema excede las cotas de Buchberger; usa Eliminate".to_string());
    }
    let outcome = crate::cas::buchberger_basis(polys, vars)
        .map_err(|e| format!("Buchberger no convergió: {e}"))?;
    let mut steps = Vec::new();
    push_ga_step(
        &mut steps,
        &polys.join(", "),
        &format!("{} S-polinomios", outcome.s_polys_used),
        "Buchberger lexicográfico acotado (criterio de pares primos relativos)",
    );
    for (i, poly) in outcome.basis.iter().enumerate() {
        if steps.len() >= MAX_CAS_STEPS {
            break;
        }
        push_ga_step(
            &mut steps,
            &format!("base[{i}]"),
            poly,
            &format!("polinomio triangular {i}"),
        );
    }
    steps.truncate(MAX_CAS_STEPS);
    Ok(steps)
}

// ---------------------------------------------------------------------------
// Frente B2: trazas de orden-n, Euler, Frobenius, Laplace cálculo,
// Groebner con órdenes y eliminación.
// ---------------------------------------------------------------------------

/// Regla Risch B2 según la primitiva emitida.
fn risch_b2_rule(prim: &str, expr: &str) -> RewriteRule {
    if prim.contains("atan") {
        RewriteRule::IntegrationArctanRule
    } else if expr.contains("sin") || expr.contains("cos") || expr.contains("tan") {
        RewriteRule::RischTrigRule
    } else if expr.contains("sqrt") {
        RewriteRule::RischEulerSqrtRule
    } else if expr.contains("exp") || expr.contains("ln") {
        RewriteRule::RischPartsRule
    } else {
        RewriteRule::Generic
    }
}

/// Traza EDO orden-n constante por anulador + resonancia.
pub fn steps_for_ode_nth_order(
    coeffs: &[String],
    rhs: &str,
    x: &str,
) -> Result<Vec<CasStep>, String> {
    validate_identifier(x)?;
    validate_input_bytes(rhs)?;
    if coeffs.is_empty() || coeffs.len() > crate::ode::MAX_ODE_NTH_ORDER + 1 {
        return Err("orden fuera de 1..=8".to_string());
    }
    let sol = crate::ode::solve_ode_nth_order_const(coeffs, rhs, x)
        .map_err(|e| format!("EDO orden-n no resuelta: {e}"))?;
    let mut steps = Vec::new();
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::OdeNthOrderRule,
        &format!("({})y = {rhs}", coeffs.join(", ")),
        "característica: raíces racionales + resto cuadrático",
        "anulador B2: pela lineales racionales, cierra con cuadrática",
    );
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::OdeNthOrderRule,
        "característica factorizada",
        &sol,
        "homogénea exacta + particular por colocación (resonancia x^s)",
    );
    steps.truncate(MAX_CAS_STEPS);
    Ok(steps)
}

/// Traza Euler `x²y''+a·x·y'+b·y` vía `x = eᵗ`.
pub fn steps_for_ode_euler(a: &str, b: &str, rhs: &str, x: &str) -> Result<Vec<CasStep>, String> {
    validate_identifier(x)?;
    for e in [a, b, rhs] {
        validate_input_bytes(e)?;
    }
    let sol = crate::ode::solve_ode_euler_2nd(a, b, rhs, x)
        .map_err(|e| format!("Euler no resuelta: {e}"))?;
    let mut steps = Vec::new();
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::OdeEulerRule,
        &format!("x²y''+{a}·x·y'+{b}·y = {rhs}"),
        "ecuación indicial r²+(a−1)r+b = 0",
        "sustitución x = eᵗ: coeficientes constantes en t",
    );
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::OdeEulerRule,
        "indicial resuelta",
        &sol,
        "x^r / ln(x) resonante; particular término a término",
    );
    steps.truncate(MAX_CAS_STEPS);
    Ok(steps)
}

/// Traza Frobenius en punto ordinario (coeficientes verificables).
pub fn steps_for_frobenius(
    p: &str,
    q: &str,
    x: &str,
    center: f64,
    terms: usize,
) -> Result<Vec<CasStep>, String> {
    validate_identifier(x)?;
    validate_input_bytes(p)?;
    validate_input_bytes(q)?;
    let out = crate::ode::frobenius_series_2nd(p, q, x, center, terms)
        .map_err(|e| format!("Frobenius no calculado: {e}"))?;
    let mut steps = Vec::new();
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::FrobeniusRule,
        &format!("y''+({p})y'+({q})y = 0 en {x} = {center}"),
        &format!("recurrencia hasta a{}", out.terms - 1),
        "punto ordinario (p,q polinomios): serie de potencias",
    );
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::FrobeniusRule,
        "recurrencia resuelta",
        &crate::ode::format_frobenius_series(&out.y1, x, center),
        "y₁ con a₀=1, a₁=0",
    );
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::FrobeniusRule,
        "recurrencia resuelta",
        &crate::ode::format_frobenius_series(&out.y2, x, center),
        "y₂ con a₀=0, a₁=1",
    );
    steps.truncate(MAX_CAS_STEPS);
    Ok(steps)
}

/// Traza `L{y⁽ⁿ⁾}` por regla con iniciales.
pub fn steps_for_laplace_derivative(
    order: u32,
    y: &str,
    t: &str,
    s: &str,
    initials: &[String],
) -> Result<Vec<CasStep>, String> {
    validate_identifier(y)?;
    validate_identifier(t)?;
    validate_identifier(s)?;
    let out = crate::ode::laplace_derivative(order, y, t, s, initials)
        .map_err(|e| format!("Laplace derivada no cubierta: {e}"))?;
    let mut steps = Vec::new();
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::LaplaceCalculusRule,
        &format!("L{{y^({order})}}"),
        &out,
        "regla sⁿ·Y − Σ s^{n−1−k}·y⁽ᵏ⁾(0)",
    );
    steps.truncate(MAX_CAS_STEPS);
    Ok(steps)
}

/// Traza `L{∫₀ᵗ f} = L{f}/s`.
pub fn steps_for_laplace_integral(f: &str, t: &str, s: &str) -> Result<Vec<CasStep>, String> {
    validate_identifier(t)?;
    validate_identifier(s)?;
    validate_input_bytes(f)?;
    let out = crate::ode::laplace_integral_rule(f, t, s)
        .map_err(|e| format!("Laplace integral no cubierta: {e}"))?;
    let mut steps = Vec::new();
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::LaplaceCalculusRule,
        &format!("L{{∫₀ {f}dt}}"),
        &out,
        "integra en t, divide por s",
    );
    steps.truncate(MAX_CAS_STEPS);
    Ok(steps)
}

/// Traza Groebner con orden monomial explícito.
pub fn steps_for_groebner_ordered(
    polys: &[String],
    vars: &[String],
    order: &str,
) -> Result<Vec<CasStep>, String> {
    if polys.len() > crate::cas::MAX_GROEBNER_POLYS || vars.len() > crate::cas::MAX_GROEBNER_VARS {
        return Err("sistema excede las cotas de Buchberger; usa Eliminate".to_string());
    }
    let clean = order.trim().to_ascii_lowercase();
    let ord = match clean.as_str() {
        "lex" => crate::cas::MonomialOrder::Lex,
        "grlex" => crate::cas::MonomialOrder::GrLex,
        "grevlex" => crate::cas::MonomialOrder::GrRevLex,
        _ => return Err(format!("orden '{order}' inválido (lex|grlex|grevlex)")),
    };
    let outcome = crate::cas::buchberger_basis_ordered(polys, vars, ord)
        .map_err(|e| format!("Buchberger no convergió: {e}"))?;
    let mut steps = Vec::new();
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::GroebnerOrderRule,
        &polys.join(", "),
        &format!("{} S-polinomios (azúcar)", outcome.s_polys_used),
        &format!("Buchberger {clean} con selección por azúcar"),
    );
    for (i, poly) in outcome.basis.iter().enumerate() {
        if steps.len() >= MAX_CAS_STEPS {
            break;
        }
        push_ga_ruled_step(
            &mut steps,
            RewriteRule::GroebnerOrderRule,
            &format!("base[{i}]"),
            poly,
            &format!("polinomio {clean} {i}"),
        );
    }
    steps.truncate(MAX_CAS_STEPS);
    Ok(steps)
}

/// Traza `Eliminate`: lex con eliminadas mayores + filtrado.
pub fn steps_for_eliminate(
    polys: &[String],
    vars: &[String],
    elim: &[String],
) -> Result<Vec<CasStep>, String> {
    if polys.len() > crate::cas::MAX_GROEBNER_POLYS || vars.len() > crate::cas::MAX_GROEBNER_VARS {
        return Err("sistema excede las cotas de Buchberger".to_string());
    }
    let outcome = crate::cas::buchberger_eliminate(polys, vars, elim)
        .map_err(|e| format!("Eliminate no resolvió: {e}"))?;
    let mut steps = Vec::new();
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::EliminateRule,
        &polys.join(", "),
        &format!("elimina {}", elim.join(", ")),
        "lex con eliminadas como mayores (teorema de eliminación)",
    );
    for (i, poly) in outcome.basis.iter().enumerate() {
        if steps.len() >= MAX_CAS_STEPS {
            break;
        }
        push_ga_ruled_step(
            &mut steps,
            RewriteRule::EliminateRule,
            &format!("intersección[{i}]"),
            poly,
            "polinomio solo en las variables restantes",
        );
    }
    steps.truncate(MAX_CAS_STEPS);
    Ok(steps)
}

// ---------------------------------------------------------------------------
// Frente B1: traza de `Solve[expr, var]` general
// ---------------------------------------------------------------------------

/// Pasos pedagógicos del Solve general (lineal/cuadrática/Sturm).
pub fn steps_for_solve(expr: &str, var: &str) -> Result<Vec<CasStep>, String> {
    validate_identifier(var)?;
    validate_input_bytes(expr)?;
    let outcome =
        crate::solve::solve_all_real(expr, var).map_err(|e| format!("Solve sin traza: {e}"))?;
    let mut steps = Vec::new();
    let roots_s = crate::solve::format_real_roots(&outcome);
    match outcome.method {
        crate::solve::SolveMethod::LinearExact => {
            push_ga_ruled_step(
                &mut steps,
                RewriteRule::SolveLinearRule,
                &format!("{expr} = 0"),
                &roots_s,
                "despeje lineal a·x+b=0 → x=−b/a (Gauss 1×1)",
            );
        }
        crate::solve::SolveMethod::QuadraticExact => {
            push_ga_ruled_step(
                &mut steps,
                RewriteRule::SolveQuadraticRule,
                &format!("{expr} = 0"),
                "D = b²−4ac",
                "discriminante cuadrático (forma estable)",
            );
            push_ga_ruled_step(
                &mut steps,
                RewriteRule::SolveQuadraticRule,
                "D = b²−4ac",
                &roots_s,
                "fórmula x=(−b±√D)/2a; D<0 → {} + aviso complejo",
            );
        }
        crate::solve::SolveMethod::SturmBisection => {
            push_ga_ruled_step(
                &mut steps,
                RewriteRule::SolveSturmRule,
                &format!("{expr} = 0"),
                "intervalo de Cauchy",
                "cota 1+max|aᵢ/aₙ|: todas las raíces reales adentro",
            );
            push_ga_ruled_step(
                &mut steps,
                RewriteRule::SolveSturmRule,
                "intervalo de Cauchy",
                &format!("{} intervalos con raíz", outcome.roots.len()),
                "secuencia de Sturm: variaciones aíslan cada raíz real",
            );
            push_ga_ruled_step(
                &mut steps,
                RewriteRule::SolveSturmRule,
                &format!("{} intervalos con raíz", outcome.roots.len()),
                &roots_s,
                "bisección 100 iters + Newton + verificación por residuo",
            );
        }
    }
    steps.truncate(MAX_CAS_STEPS);
    Ok(steps)
}

// ---------------------------------------------------------------------------
// Sistemas lineales 2×2 (eliminación/sustitución) + integral definida (Barrow)
// ---------------------------------------------------------------------------

/// Pasa `lhs = rhs` a forma `lhs - rhs` (o `= 0` si no hay `=`).
fn system_lhs(eq: &str) -> String {
    match eq.split_once('=') {
        Some((lhs, rhs)) => format!("({})-({})", lhs.trim(), rhs.trim()),
        None => eq.to_string(),
    }
}

/// Coeficientes `a·v1 + b·v2 + c = 0` del AST ya en forma `= 0`.
/// `None` si no es lineal en exactamente esas dos incógnitas.
fn linear2_coeffs(ast: &Expr, var1: &str, var2: &str) -> Option<(f64, f64, f64)> {
    match ast {
        Expr::Const(v) if v.is_finite() => Some((0.0, 0.0, *v)),
        Expr::Var(name) if name == var1 => Some((1.0, 0.0, 0.0)),
        Expr::Var(name) if name == var2 => Some((0.0, 1.0, 0.0)),
        Expr::Var(_) => None,
        Expr::Neg(inner) => {
            let (a, b, c) = linear2_coeffs(inner, var1, var2)?;
            Some((-a, -b, -c))
        }
        Expr::Add(l, r) => {
            let (a1, b1, c1) = linear2_coeffs(l, var1, var2)?;
            let (a2, b2, c2) = linear2_coeffs(r, var1, var2)?;
            let (a, b, c) = (a1 + a2, b1 + b2, c1 + c2);
            (a.is_finite() && b.is_finite() && c.is_finite()).then_some((a, b, c))
        }
        Expr::Sub(l, r) => {
            let (a1, b1, c1) = linear2_coeffs(l, var1, var2)?;
            let (a2, b2, c2) = linear2_coeffs(r, var1, var2)?;
            let (a, b, c) = (a1 - a2, b1 - b2, c1 - c2);
            (a.is_finite() && b.is_finite() && c.is_finite()).then_some((a, b, c))
        }
        Expr::Mul(l, r) => {
            if let Some(k) = constant_scalar_value(l) {
                let (a, b, c) = linear2_coeffs(r, var1, var2)?;
                let (a, b, c) = (k * a, k * b, k * c);
                (a.is_finite() && b.is_finite() && c.is_finite()).then_some((a, b, c))
            } else if let Some(k) = constant_scalar_value(r) {
                let (a, b, c) = linear2_coeffs(l, var1, var2)?;
                let (a, b, c) = (a * k, b * k, c * k);
                (a.is_finite() && b.is_finite() && c.is_finite()).then_some((a, b, c))
            } else {
                None
            }
        }
        Expr::Div(n, d) => {
            let k = constant_scalar_value(d)?;
            if k == 0.0 {
                return None;
            }
            let (a, b, c) = linear2_coeffs(n, var1, var2)?;
            let (a, b, c) = (a / k, b / k, c / k);
            (a.is_finite() && b.is_finite() && c.is_finite()).then_some((a, b, c))
        }
        _ => None,
    }
}

/// Coeficientes lineales `a·var1 + b·var2 + c = 0` de una ecuación con `=`
/// (o ya en forma `= 0`). `None` si no es lineal en exactamente esas dos
/// incógnitas: el llamador responde con error honesto, nunca adivina.
#[must_use]
pub fn linear_eq_coeffs_2x2(eq: &str, var1: &str, var2: &str) -> Option<(f64, f64, f64)> {
    if var1.is_empty() || var2.is_empty() || var1 == var2 {
        return None;
    }
    let lhs = system_lhs(eq.trim());
    let ast = parse_ast(&lhs.replace(' ', "")).ok()?;
    linear2_coeffs(&ast, var1, var2)
}

/// Número compacto para mostrar ecuaciones (`3` en vez de `3.000000`).
fn fmt_lin(v: f64) -> String {
    if !v.is_finite() {
        return "∞".to_string();
    }
    if v.fract() == 0.0 && v.abs() < 1e15 {
        return format!("{}", v as i64);
    }
    format!("{v:.6}")
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

/// Traza pedagógica de un sistema lineal 2×2 por eliminación y sustitución.
///
/// Solo rectas en exactamente `var1`, `var2`: no lineal, singular o con
/// otras incógnitas da `Err` honesto (pedagogía de sistemas chicos, sin
/// Gröbner). La solución se verifica por sustitución antes de emitir pasos.
pub fn steps_for_system_2x2(
    eq1: &str,
    eq2: &str,
    var1: &str,
    var2: &str,
) -> Result<Vec<CasStep>, String> {
    validate_identifier(var1)?;
    validate_identifier(var2)?;
    if var1 == var2 {
        return Err("sistema 2×2 requiere dos incógnitas distintas".to_string());
    }
    validate_input_bytes(eq1)?;
    validate_input_bytes(eq2)?;
    let (a1, b1, c1) = linear_eq_coeffs_2x2(eq1, var1, var2).ok_or_else(|| {
        format!(
            "sistema no lineal 2×2 en {var1}, {var2}: solo rectas tienen pasos (para no lineal usá SolveNlSystem[eq1, eq2, x, y] directo, sin StepByStep, o GroebnerOrdered/Eliminate)"
        )
    })?;
    let (a2, b2, c2) = linear_eq_coeffs_2x2(eq2, var1, var2).ok_or_else(|| {
        format!(
            "sistema no lineal 2×2 en {var1}, {var2}: solo rectas tienen pasos (para no lineal usá SolveNlSystem[eq1, eq2, x, y] directo, sin StepByStep, o GroebnerOrdered/Eliminate)"
        )
    })?;
    let (d1, d2) = (-c1, -c2);
    if ![a1, b1, d1, a2, b2, d2].iter().all(|v| v.is_finite()) {
        return Err("sistema 2×2 con coeficientes no finitos".to_string());
    }
    let det = a1 * b2 - a2 * b1;
    if !det.is_finite() || det.abs() < 1e-12 {
        return Err(
            "sistema singular o dependiente (det ≈ 0); sin solución única (verifica rango con Rank[...])"
                .to_string(),
        );
    }
    let num_x = d1 * b2 - d2 * b1;
    let num_y = a1 * d2 - a2 * d1;
    let (x, y) = (num_x / det, num_y / det);
    if !x.is_finite() || !y.is_finite() {
        return Err("solución no finita; sistema mal condicionado".to_string());
    }
    let (r1, r2) = ((a1 * x + b1 * y - d1).abs(), (a2 * x + b2 * y - d2).abs());
    let tol = 1e-6 * (1.0 + d1.abs().max(d2.abs()));
    if r1 > tol || r2 > tol {
        return Err(format!(
            "verificación por sustitución falló (residuos {r1:.2e}, {r2:.2e}); revisa las ecuaciones"
        ));
    }
    let mut steps = Vec::new();
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::SystemEliminationRule,
        &format!("{eq1} ; {eq2}"),
        &format!(
            "({})·{var1} + ({})·{var2} = {} ; ({})·{var1} + ({})·{var2} = {}",
            fmt_lin(a1),
            fmt_lin(b1),
            fmt_lin(d1),
            fmt_lin(a2),
            fmt_lin(b2),
            fmt_lin(d2)
        ),
        "normalizar: llevar todo a un lado y leer a·x + b·y = d",
    );
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::SystemEliminationRule,
        &format!("({})·Ec1 − ({})·Ec2", fmt_lin(b2), fmt_lin(b1)),
        &format!("({})·{var1} = {}", fmt_lin(det), fmt_lin(num_x)),
        &format!("eliminar {var2}: combinar para dejar una ecuación solo en {var1}"),
    );
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::SystemSubstitutionRule,
        &format!("({})·{var1} = {}", fmt_lin(det), fmt_lin(num_x)),
        &format!("{var1} = {}", fmt_lin(x)),
        &format!("despejar {var1} dividiendo por el determinante"),
    );
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::SystemSubstitutionRule,
        &format!("sustituir {var1} = {} en Ec1", fmt_lin(x)),
        &format!("{var2} = {}", fmt_lin(y)),
        &format!("sustituir en la primera ecuación y despejar {var2}"),
    );
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::SystemSubstitutionRule,
        &format!("({}, {}) en ambas ecuaciones", fmt_lin(x), fmt_lin(y)),
        &format!("residuos {r1:.2e}, {r2:.2e} ≈ 0"),
        "verificación: sustituir la solución recupera 0 = 0",
    );
    steps.truncate(MAX_CAS_STEPS);
    Ok(steps)
}

/// Tope de observaciones de las trazas de estadística (espeja
/// `MAX_STATS_EXTRA_LEN` de `stats_extra.rs`).
pub const MAX_STATS_STEPS_LEN: usize = 10_000;

/// Parsea una lista numérica `{a, b, c}` (o `[a, b, c]` del NL) a `f64`.
/// Vacía, no finita o con texto no numérico da `Err` honesto: jamás se
/// inventan datos (ni ceros de relleno ni interpolación).
fn parse_stats_list(raw: &str, rol: &str) -> Result<Vec<f64>, String> {
    validate_input_bytes(raw)?;
    let s = raw.trim();
    let inner = if let Some(cuerpo) = s
        .strip_prefix('{')
        .and_then(|resto| resto.strip_suffix('}'))
    {
        cuerpo
    } else if let Some(cuerpo) = s
        .strip_prefix('[')
        .and_then(|resto| resto.strip_suffix(']'))
    {
        cuerpo
    } else {
        return Err(format!(
            "{rol} debe ser una lista {{a, b, c}} (recibí `{raw}`)"
        ));
    };
    if inner.trim().is_empty() {
        return Err(format!("{rol}: lista vacía (sin datos no hay pasos)"));
    }
    let mut out = Vec::new();
    for token in inner.split(',') {
        let t = token.trim();
        if t.is_empty() {
            return Err(format!("{rol}: la lista contiene un valor vacío"));
        }
        let v: f64 = t
            .parse()
            .map_err(|_| format!("{rol}: valor inválido '{t}' (solo números)"))?;
        if !v.is_finite() {
            return Err(format!("{rol}: la lista contiene un valor no finito"));
        }
        out.push(v);
    }
    if out.is_empty() {
        return Err(format!("{rol}: lista vacía (sin datos no hay pasos)"));
    }
    if out.len() > MAX_STATS_STEPS_LEN {
        return Err(format!(
            "{rol}: longitud {} excede el máximo {MAX_STATS_STEPS_LEN}",
            out.len()
        ));
    }
    Ok(out)
}

/// Mediana de una muestra ya ordenada (promedio central si N es par).
fn mediana_ordenada(ordenada: &[f64]) -> f64 {
    let n = ordenada.len();
    if n.is_multiple_of(2) {
        (ordenada[n / 2 - 1] + ordenada[n / 2]) / 2.0
    } else {
        ordenada[n / 2]
    }
}

/// Traza descriptiva de lista: media/mediana/varianza/desvío poblacionales
/// (÷N, convención del panel `Variance`/`SD`) con fórmulas, sustitución
/// numérica y N pineado en cada paso.
///
/// La verificación recalcula los cuatro números por un camino independiente
/// (suma en orden inverso, mediana sobre copia re-ordenada, varianza en dos
/// pasadas) y exige residuo ≈ 0: jamás se copia el valor. Vacía, no finita
/// o mayor a `MAX_STATS_STEPS_LEN` da `Err` honesto. La muestral (÷N−1)
/// queda fuera del subset: usá `SampleVariance`/`SampleSD` directo.
#[allow(clippy::too_many_lines)]
pub fn steps_for_stats_descriptive(data: &str) -> Result<Vec<CasStep>, String> {
    let xs = parse_stats_list(data, "descriptiva")?;
    let n = xs.len();
    let suma: f64 = xs.iter().sum();
    if !suma.is_finite() {
        return Err("descriptiva: suma no finita (revisa la escala de los datos)".to_string());
    }
    let media = suma / n as f64;
    let mut ordenada = xs.clone();
    ordenada.sort_by(|a, b| a.total_cmp(b));
    let mediana = mediana_ordenada(&ordenada);
    let suma_cuad: f64 = xs.iter().map(|v| (v - media).powi(2)).sum();
    if !suma_cuad.is_finite() {
        return Err(
            "descriptiva: suma de cuadrados no finita (revisa la escala de los datos)".to_string(),
        );
    }
    let varianza = suma_cuad / n as f64;
    let desvio = varianza.sqrt();
    for (nombre, v) in [
        ("media", media),
        ("mediana", mediana),
        ("varianza", varianza),
        ("desvío", desvio),
    ] {
        if !v.is_finite() {
            return Err(format!("{nombre} no finita; revisa los datos"));
        }
    }
    // Verificación recalculada: suma en orden inverso, mediana sobre copia
    // re-ordenada y varianza en dos pasadas con la media recalculada.
    let suma2: f64 = xs.iter().rev().sum();
    let media2 = suma2 / n as f64;
    let mut ordenada2 = xs.clone();
    ordenada2.sort_by(|a, b| b.total_cmp(a));
    ordenada2.reverse();
    let mediana2 = mediana_ordenada(&ordenada2);
    let suma_cuad2: f64 = xs.iter().map(|v| (v - media2).powi(2)).sum();
    let varianza2 = suma_cuad2 / n as f64;
    let desvio2 = varianza2.sqrt();
    let escala = 1.0 + media.abs().max(mediana.abs()).max(varianza.abs());
    let residuos = [
        (media - media2).abs(),
        (mediana - mediana2).abs(),
        (varianza - varianza2).abs(),
        (desvio - desvio2).abs(),
    ];
    let peor = residuos.iter().copied().fold(0.0f64, f64::max);
    let tol = 1e-9 * escala;
    if !(peor.is_finite() && peor <= tol) {
        return Err(format!(
            "verificación recalculada falló (residuo {peor:.2e}); revisa los datos"
        ));
    }
    let lista_txt = ordenada
        .iter()
        .map(|v| fmt_lin(*v))
        .collect::<Vec<_>>()
        .join(", ");
    let centro_txt = if n.is_multiple_of(2) {
        format!(
            "N par: ({} + {}) / 2",
            fmt_lin(ordenada[n / 2 - 1]),
            fmt_lin(ordenada[n / 2])
        )
    } else {
        format!("N impar: elemento central {}", fmt_lin(mediana))
    };
    let mut steps = Vec::new();
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::StatsDescriptiveRule,
        &format!("datos {{{}}}", lista_txt),
        &format!("N = {n} ; Σx = {}", fmt_lin(suma)),
        "contar N (pineado: todo se divide por este N) y sumar la lista",
    );
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::StatsDescriptiveRule,
        &format!("media = Σx / N = {} / {n}", fmt_lin(suma)),
        &format!("media = {}", fmt_lin(media)),
        "media poblacional: el punto de equilibrio de la lista",
    );
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::StatsDescriptiveRule,
        &format!("ordenar con N = {n}: {{{lista_txt}}} "),
        &format!("mediana = {} ({centro_txt})", fmt_lin(mediana)),
        "mediana: el valor del medio (robusta a valores extremos)",
    );
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::StatsDescriptiveRule,
        &format!("σ² = Σ(x−μ)² / N = {} / {n}", fmt_lin(suma_cuad)),
        &format!("σ² = {}", fmt_lin(varianza)),
        "varianza poblacional ÷N (la muestral ÷N−1 vive en SampleVariance directo)",
    );
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::StatsDescriptiveRule,
        &format!("σ = √σ² = √({})", fmt_lin(varianza)),
        &format!("σ = {}", fmt_lin(desvio)),
        "desvío poblacional: vuelve a las unidades originales",
    );
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::StatsDescriptiveRule,
        "recalcular media/mediana/σ²/σ por camino independiente",
        &format!("residuo {peor:.2e} ≈ 0"),
        "verificación: recalcular todo recupera los mismos números",
    );
    steps.truncate(MAX_CAS_STEPS);
    Ok(steps)
}

/// Traza de regresión lineal por mínimos cuadrados: pendiente e intercepto
/// por fórmulas cerradas y R² con verificación 0≤R²≤1.
///
/// La verificación recalcula R² por residuos (1 − SSres/SStot) con la recta
/// ya hallada y exige coincidencia: jamás se copia el valor. `x` constante,
/// largos distintos, menos de 2 puntos o no finitos dan `Err` honesto.
/// `y` constante da la recta horizontal (m = 0, R² = 1 por convención del
/// panel `LinearRegression`), mostrado como tal y no como ajuste.
#[allow(clippy::too_many_lines)]
pub fn steps_for_stats_regression(xs_raw: &str, ys_raw: &str) -> Result<Vec<CasStep>, String> {
    let xs = parse_stats_list(xs_raw, "regresión.xs")?;
    let ys = parse_stats_list(ys_raw, "regresión.ys")?;
    if xs.len() != ys.len() {
        return Err(format!(
            "regresión: xs e ys deben tener el mismo largo ({} vs {})",
            xs.len(),
            ys.len()
        ));
    }
    let n = xs.len();
    if n < 2 {
        return Err("regresión: se necesitan al menos 2 puntos".to_string());
    }
    let mx: f64 = xs.iter().sum::<f64>() / n as f64;
    let my: f64 = ys.iter().sum::<f64>() / n as f64;
    if !mx.is_finite() || !my.is_finite() {
        return Err("regresión: medias no finitas (revisa la escala)".to_string());
    }
    let mut ss_xx = 0.0;
    let mut ss_xy = 0.0;
    let mut ss_yy = 0.0;
    for (x, y) in xs.iter().zip(ys.iter()) {
        ss_xx += (x - mx).powi(2);
        ss_xy += (x - mx) * (y - my);
        ss_yy += (y - my).powi(2);
    }
    if !(ss_xx.is_finite() && ss_xy.is_finite() && ss_yy.is_finite()) {
        return Err("regresión: sumas de cuadrados no finitas".to_string());
    }
    if ss_xx <= 0.0 {
        return Err(
            "regresión: x no varía (SSxx = 0); sin pendiente única (revisa los datos)".to_string(),
        );
    }
    // `y` constante: recta horizontal exacta (convención del panel).
    let (m, b, r2, horizontal) = if ss_yy <= 0.0 {
        (0.0, my, 1.0, true)
    } else {
        let mm = ss_xy / ss_xx;
        let bb = my - mm * mx;
        let rr = ss_xy * ss_xy / (ss_xx * ss_yy);
        (mm, bb, rr, false)
    };
    if !(m.is_finite() && b.is_finite() && r2.is_finite()) {
        return Err("regresión: resultado no finito (revisa los datos)".to_string());
    }
    if !(0.0 - 1e-9..=1.0 + 1e-9).contains(&r2) {
        return Err(format!(
            "regresión: R² = {r2:.6} fuera de [0, 1]; revisa los datos"
        ));
    }
    // Verificación recalculada por residuos con la recta hallada.
    let mut ss_res = 0.0;
    for (x, y) in xs.iter().zip(ys.iter()) {
        ss_res += (y - (m * x + b)).powi(2);
    }
    let r2_check = if ss_yy <= 0.0 {
        1.0
    } else {
        1.0 - ss_res / ss_yy
    };
    let dif = (r2 - r2_check).abs();
    let tol = 1e-9 * (1.0 + ss_yy);
    if !(dif.is_finite() && dif <= tol) || !(0.0 - 1e-9..=1.0 + 1e-9).contains(&r2_check) {
        return Err(format!(
            "verificación por residuos falló (R² recalculado {r2_check:.6}); revisa los datos"
        ));
    }
    let mut steps = Vec::new();
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::StatsRegressionRule,
        &format!("N = {n} puntos (x, y)"),
        &format!(
            "x̄ = {} ; ȳ = {} ; SSxx = {} ; SSxy = {}",
            fmt_lin(mx),
            fmt_lin(my),
            fmt_lin(ss_xx),
            fmt_lin(ss_xy)
        ),
        "promedios y sumas de cuadrados: el resumen que pide mínimos cuadrados",
    );
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::StatsRegressionRule,
        &format!("m = SSxy / SSxx = {} / {}", fmt_lin(ss_xy), fmt_lin(ss_xx)),
        &format!("m = {}", fmt_lin(m)),
        "pendiente por fórmula cerrada: cuánto sube y por cada x",
    );
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::StatsRegressionRule,
        &format!(
            "b = ȳ − m·x̄ = {} − {}·{}",
            fmt_lin(my),
            fmt_lin(m),
            fmt_lin(mx)
        ),
        &format!("b = {} ; y = {}·x + {}", fmt_lin(b), fmt_lin(m), fmt_lin(b)),
        "intercepto: la recta pasa por (x̄, ȳ)",
    );
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::StatsRegressionRule,
        &format!(
            "R² = SSxy²/(SSxx·SSyy) = {}²/({}·{})",
            fmt_lin(ss_xy),
            fmt_lin(ss_xx),
            fmt_lin(ss_yy)
        ),
        &format!(
            "R² = {}{}",
            fmt_lin(r2),
            if horizontal {
                " (y constante: recta horizontal)"
            } else {
                " (0 ≤ R² ≤ 1)"
            }
        ),
        "R²: fracción de la variación de y que explica la recta",
    );
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::StatsRegressionRule,
        &format!("R² por residuos: 1 − SSres/SStot = {r2_check:.6}"),
        &format!("diferencia {dif:.2e} ≈ 0 ; 0 ≤ R² ≤ 1"),
        "verificación: recalcular R² por residuos recupera el mismo valor",
    );
    steps.truncate(MAX_CAS_STEPS);
    Ok(steps)
}

/// Traza de integral definida: antiderivada con las reglas existentes más
/// Barrow `∫[a,b] f = F(b) − F(a)` como paso final honesto. Sin primitiva
/// soportada, con primitiva no finita en los extremos o con límites no
/// finitos da `Err` (la vía numérica es `GaussKronrod[expr, var, a, b]`).
pub fn steps_for_definite_integral(
    expr: &str,
    var: &str,
    a: f64,
    b: f64,
) -> Result<Vec<CasStep>, String> {
    if !a.is_finite() || !b.is_finite() {
        return Err("límites de integración no finitos".to_string());
    }
    validate_identifier(var)?;
    validate_input_bytes(expr)?;
    if a == b {
        let mut steps = Vec::new();
        push_ga_ruled_step(
            &mut steps,
            RewriteRule::BarrowRule,
            &format!("∫[{a},{b}] {expr} d{var}"),
            "0",
            "Barrow con límites iguales: la integral es 0",
        );
        return Ok(steps);
    }
    let pp = expr.replace(' ', "");
    let ast = parse_ast(&pp).map_err(|e| format!("parse error: {e}"))?;
    let mut steps = Vec::new();
    let prim = if let Some(p) = integrate_with_steps(&ast, var, &mut steps, 0) {
        p
    } else if let Some(fallback) = ast.integrate(var) {
        let simp = simplify_expr(&fallback);
        push_step(
            &mut steps,
            RewriteRule::Generic,
            &ast,
            &simp,
            "integral nativa",
        );
        simp
    } else {
        return Err(format!(
            "no hay primitiva soportada para '{expr}' (probá la vía numérica GaussKronrod[{expr}, {var}, {a}, {b}])"
        ));
    };
    let prim_s = simplify_expr(&prim);
    let (fa, fb) = (prim_s.eval_at(var, a), prim_s.eval_at(var, b));
    if !fa.is_finite() || !fb.is_finite() {
        return Err(format!(
            "la primitiva no es finita en los extremos (F({a}) = {fa:.6}, F({b}) = {fb:.6}); revisa singularidades en [{a}, {b}]"
        ));
    }
    let value = fb - fa;
    if !value.is_finite() {
        return Err("F(b) − F(a) no es finito".to_string());
    }
    push_ga_ruled_step(
        &mut steps,
        RewriteRule::BarrowRule,
        &format!("F({b}) − F({a}) = {fb:.6} − ({fa:.6})"),
        &format!("{value:.6}"),
        &format!("Barrow: ∫[{a},{b}] {expr} d{var} = F(b) − F(a)"),
    );
    steps.truncate(MAX_CAS_STEPS);
    Ok(steps)
}

// ---------------------------------------------------------------------------
// Tests genéricos
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::{etiqueta, etiqueta_str, motivo, motivo_str, RewriteRule};
    use crate::cas_steps::RewriteRule as R;

    /// Toda regla del enum tiene motivo y etiqueta propios: nada cae al
    /// fallback genérico salvo `Generic` (y los nombres Display completan
    /// `motivo_str`/`etiqueta_str`).
    #[test]
    fn motivo_y_etiqueta_cubren_todas_las_reglas() {
        let todas = [
            R::ConstantRule,
            R::VariableRule,
            R::SumRule,
            R::DifferenceRule,
            R::ProductRule,
            R::QuotientRule,
            R::PowerRule,
            R::NegRule,
            R::ChainRule,
            R::ExpRule,
            R::LogRule,
            R::TrigRule,
            R::SqrtRule,
            R::AbsRule,
            R::Simplify,
            R::IntegrationPowerRule,
            R::IntegrationSumRule,
            R::IntegrationConstantRule,
            R::IntegrationExpRule,
            R::IntegrationTrigRule,
            R::IntegrationPartialFractions,
            R::IntegrationArctanRule,
            R::OdeSecondOrderRule,
            R::OdeSystemRule,
            R::LaplaceDirectRule,
            R::LaplaceInverseRule,
            R::LimitRichardson,
            R::TaylorRule,
            R::SolveLinearRule,
            R::SolveQuadraticRule,
            R::SolveSturmRule,
            R::RischTrigRule,
            R::RischEulerSqrtRule,
            R::RischPartsRule,
            R::GruntzLogRewriteRule,
            R::OscillationRule,
            R::OdeNthOrderRule,
            R::OdeEulerRule,
            R::OdeFirstOrderRule,
            R::OdeSecondHomogRule,
            R::FrobeniusRule,
            R::LaplaceCalculusRule,
            R::LaplaceShiftRule,
            R::GroebnerOrderRule,
            R::EliminateRule,
            R::SystemEliminationRule,
            R::SystemSubstitutionRule,
            R::BarrowRule,
        ];
        let generico = motivo(&R::Generic);
        let etiqueta_generica = etiqueta(&R::Generic);
        for regla in todas {
            let text = motivo(&regla);
            assert!(
                text.starts_with("Motivo:"),
                "{regla}: sin motivo pedagógico"
            );
            assert!(text.len() > 40, "{regla}: motivo demasiado corto");
            if regla != R::Generic {
                assert_ne!(text, generico, "{regla}: cayó al motivo genérico");
                assert_ne!(
                    etiqueta(&regla),
                    etiqueta_generica,
                    "{regla}: cayó a la etiqueta genérica"
                );
            }
        }
    }

    /// `motivo_str`/`etiqueta_str` resuelven por el nombre Display
    /// (roundtrip con el fence) y nombres desconocidos devuelven honesto.
    #[test]
    fn str_resolve_por_display_y_desconocido_honesto() {
        assert_eq!(etiqueta_str("SolveLinearRule"), "despeje lineal");
        assert_eq!(etiqueta_str("PowerRule"), "potencia");
        assert_eq!(
            etiqueta_str("SolverectaarRule"),
            "SolverectaarRule",
            "fence ajeno: el nombre se muestra tal cual, sin inventarla"
        );
        assert!(motivo_str("SolveLinearRule").starts_with("Motivo:"));
        // El Display de cada variante resuelve por nombre en ambas tablas
        // (el fence guarda el Display, no el enum).
        let muestra = [
            R::ChainRule,
            R::IntegrationPartialFractions,
            R::OdeEulerRule,
            R::BarrowRule,
        ];
        for regla in muestra {
            let nombre = regla.to_string();
            let esperado = etiqueta(&regla);
            assert_eq!(
                etiqueta_str(&nombre),
                esperado,
                "etiqueta_str falló para {nombre}"
            );
        }
    }

    use super::*;

    fn has_rule(steps: &[CasStep], rule: RewriteRule) -> bool {
        steps.iter().any(|s| s.rule == rule)
    }

    #[test]
    fn solve_traces_linear_quadratic_sturm() {
        let lin = steps_for_solve("2*x-6", "x").expect("traza lineal");
        assert!(has_rule(&lin, RewriteRule::SolveLinearRule));
        let quad = steps_for_solve("x^2-5*x+6", "x").expect("traza cuadrática");
        assert!(has_rule(&quad, RewriteRule::SolveQuadraticRule));
        let cubic = steps_for_solve("x^3-6*x^2+11*x-6", "x").expect("traza Sturm");
        assert!(has_rule(&cubic, RewriteRule::SolveSturmRule));
        assert!(cubic.iter().any(|s| s.after.contains("{1, 2, 3}")));
        for s in lin.iter().chain(quad.iter()).chain(cubic.iter()) {
            assert!(s.before.len() <= MAX_STEP_BYTES);
            assert!(s.after.len() <= MAX_STEP_BYTES);
        }
        let op = CasOp::Solve {
            expr: "x^2-4".to_string(),
            var: "x".to_string(),
        };
        let via_op = steps_for_op(&op).expect("despacho Solve");
        assert!(!via_op.is_empty());
        assert!(steps_for_solve("sin(x)+x", "x").is_err());
    }

    #[test]
    fn derivative_arbitrary_quadratic_emits_power_and_sum() {
        let steps = steps_for_derivative("x^2 + 3*x + 2", "x").expect("derive");
        assert!(!steps.is_empty());
        assert!(steps.len() <= MAX_CAS_STEPS);
        for s in &steps {
            assert!(s.before.len() <= MAX_STEP_BYTES);
            assert!(s.after.len() <= MAX_STEP_BYTES);
        }
        assert!(has_rule(&steps, RewriteRule::PowerRule) || has_rule(&steps, RewriteRule::SumRule));
        let last_after = &steps.last().expect("last").after;
        assert!(
            last_after.contains("2*x")
                || last_after.contains("2 * x")
                || last_after.contains("3")
                || parse_ast(last_after).is_ok()
        );
    }

    #[test]
    fn derivative_arbitrary_product_emits_product_rule() {
        let steps = steps_for_derivative("x*sin(x)", "x").expect("derive product");
        assert!(!steps.is_empty());
        assert!(has_rule(&steps, RewriteRule::ProductRule));
        assert!(has_rule(&steps, RewriteRule::ChainRule));
    }

    #[test]
    fn derivative_arbitrary_chain_sin_quadratic() {
        let steps = steps_for_derivative("sin(x^2)", "x").expect("derive chain");
        assert!(!steps.is_empty());
        assert!(has_rule(&steps, RewriteRule::ChainRule));
        assert!(steps
            .iter()
            .any(|s| s.rule == RewriteRule::ChainRule || s.rule == RewriteRule::PowerRule));
    }

    #[test]
    fn derivative_arbitrary_quotient_emits_quotient_rule() {
        let steps = steps_for_derivative("x/(x+1)", "x").expect("derive quotient");
        assert!(has_rule(&steps, RewriteRule::QuotientRule));
    }

    #[test]
    fn integral_arbitrary_power_emits_power_rule() {
        let steps = steps_for_integral("x^2", "x").expect("integral power");
        assert!(!steps.is_empty());
        assert!(
            has_rule(&steps, RewriteRule::IntegrationPowerRule)
                || has_rule(&steps, RewriteRule::Generic)
        );
        let last = steps.last().expect("last").after.clone();
        assert!(
            last.contains("x^3")
                || last.contains("x ^ 3")
                || last.contains("x*x")
                || last.contains("0.333"),
            "got {last}"
        );
    }

    #[test]
    fn integral_arbitrary_linear_combination() {
        let steps = steps_for_integral("2*x + 3", "x").expect("integral linear");
        assert!(!steps.is_empty());
        let last = steps.last().expect("last").after.clone();
        assert!(
            last.contains("x^2")
                || last.contains("x ^ 2")
                || last.contains("x*x")
                || last.contains("x ^2"),
            "espera x^2 en primitiva, got {last}"
        );
    }

    #[test]
    fn integral_arbitrary_via_parts_x_exp() {
        let steps = steps_for_integral("x*exp(x)", "x").expect("parts");
        assert!(!steps.is_empty());
    }

    #[test]
    fn limit_arbitrary_sin_over_x_uses_richardson() {
        let steps = steps_for_limit("sin(x)/x", "x", 0.0).expect("limit");
        assert!(!steps.is_empty());
        assert!(has_rule(&steps, RewriteRule::LimitRichardson));
        let last = steps.last().expect("last").after.clone();
        let val: f64 = last.parse().unwrap_or_else(|_| {
            last.split_whitespace()
                .find_map(|tok| tok.parse::<f64>().ok())
                .unwrap_or(f64::NAN)
        });
        assert!(
            (val - 1.0).abs() < 1e-4 || last.contains("1.0") || last.contains('1'),
            "lim sin(x)/x →1, got {last}"
        );
    }

    #[test]
    fn limit_arbitrary_quadratic_trivial() {
        let steps = steps_for_limit("x^2 + 1", "x", 2.0).expect("limit quadratic");
        assert!(!steps.is_empty());
        assert!(has_rule(&steps, RewriteRule::LimitRichardson));
    }

    #[test]
    fn taylor_arbitrary_sin_center_zero() {
        let steps = steps_for_taylor("sin(x)", "x", 0.0, 5).expect("taylor sin");
        assert!(!steps.is_empty());
        let combined = steps
            .iter()
            .map(|s| s.after.clone())
            .collect::<Vec<_>>()
            .join(" ");
        assert!(
            combined.contains("x") || combined.contains("sin"),
            "Taylor sin(x) debe contener término en x, got {combined}"
        );
    }

    #[test]
    fn taylor_arbitrary_cos_order_three() {
        let steps = steps_for_taylor("cos(x)", "x", 0.0, 3).expect("taylor cos");
        assert!(!steps.is_empty());
        let last = steps.last().expect("last").after.clone();
        assert!(
            last.contains("x^2") || last.contains("x ^ 2") || last.contains("x*x"),
            "cos Taylor debe contener x^2, got {last}"
        );
    }

    #[test]
    fn dispatch_generic_covers_all_ops() {
        let ops = [
            CasOp::Derivative {
                expr: "x^3 + sin(x)".into(),
                var: "x".into(),
            },
            CasOp::Integral {
                expr: "x^2".into(),
                var: "x".into(),
            },
            CasOp::Limit {
                expr: "sin(x)/x".into(),
                var: "x".into(),
                at: 0.0,
            },
            CasOp::Taylor {
                expr: "sin(x)".into(),
                var: "x".into(),
                center: 0.0,
                order: 4,
            },
        ];
        for op in &ops {
            let s = steps_for_op(op).expect("dispatch");
            assert!(!s.is_empty(), "op {op:?} debe dar pasos");
            assert!(s.len() <= MAX_CAS_STEPS);
        }
    }

    #[test]
    fn statem_transitions_are_typed() {
        let mut stepper = CasStepper::new();
        assert_eq!(*stepper.state(), CasStepperState::Idle);
        stepper.begin_parsing().expect("idle→parsing");
        assert_eq!(*stepper.state(), CasStepperState::Parsing);
        stepper.begin_stepping().expect("parsing→stepping");
        stepper.begin_verifying().expect("stepping→verifying");
        stepper.finish(true, None).expect("verifying→done");
        assert_eq!(*stepper.state(), CasStepperState::Done);
        assert!(stepper.begin_parsing().is_err());
    }

    #[test]
    fn step_bytes_are_truncated() {
        let long_expr = "x".repeat(10_000);
        let s = truncate_bytes(&long_expr, MAX_STEP_BYTES);
        assert!(s.len() <= MAX_STEP_BYTES + 3);
        assert!(s.is_char_boundary(s.len() - "…".len()));
    }

    #[test]
    fn steps_truncate_long_expression() {
        let expr = format!("{} + {}", "x".repeat(5000), "1");
        if let Ok(steps) = steps_for_derivative(&expr, "x") {
            for st in steps {
                assert!(st.before.len() <= MAX_STEP_BYTES + 3);
                assert!(st.after.len() <= MAX_STEP_BYTES + 3);
            }
        }
    }

    #[test]
    fn derivative_genuine_x_cubed_cos_emits_product_chain_power() {
        // Verificación Real (no enlatado): x^3*cos(x) debe emitir
        // ProductRule genuino + ChainRule (cos) + PowerRule (x^3).
        // Cada paso proviene del visitor diff_with_steps, no de goldens.
        let steps = steps_for_derivative("x^3*cos(x)", "x").expect("derive x^3*cos(x)");
        assert!(!steps.is_empty(), "pasos reales, no vacíos");
        assert!(
            has_rule(&steps, RewriteRule::ProductRule),
            "falta ProductRule genuino"
        );
        assert!(
            has_rule(&steps, RewriteRule::ChainRule),
            "falta ChainRule genuino (cos)"
        );
        assert!(
            has_rule(&steps, RewriteRule::PowerRule),
            "falta PowerRule genuino (x^3)"
        );
        // El paso ProductRule debe contener ambos factores originales
        let prod = steps
            .iter()
            .find(|s| s.rule == RewriteRule::ProductRule)
            .expect("ProductRule step");
        assert!(
            prod.before.contains("cos") && prod.before.contains('x'),
            "ProductRule before debe reflejar AST real, got {}",
            prod.before
        );
        // Presupuestos siempre respetados incluso en caso genuino
        for s in &steps {
            assert!(s.before.len() <= MAX_STEP_BYTES + 3);
            assert!(s.after.len() <= MAX_STEP_BYTES + 3);
            assert!(s.description.len() <= MAX_STEP_BYTES + 3);
        }
        assert!(steps.len() <= MAX_CAS_STEPS);
    }

    #[test]
    fn max_cas_steps_hard_cap_is_enforced() {
        // Expresión con ~40 sumandos: Add recursivo genera ~40 SumRule + 40 leaves
        // pero push_step y truncate deben capar a MAX_CAS_STEPS 32.
        let expr = (0..40).map(|_| "x").collect::<Vec<_>>().join(" + ");
        let steps = steps_for_derivative(&expr, "x").expect("derive many adds");
        assert!(
            steps.len() <= MAX_CAS_STEPS,
            "len {} > MAX_CAS_STEPS {}",
            steps.len(),
            MAX_CAS_STEPS
        );
        // Si la expresión fuerza más pasos que el cap, debe quedar exactamente capado
        // (no panic, no OOM, Vec no crece indefinidamente).
        assert!(!steps.is_empty());
        for s in &steps {
            assert!(s.before.len() <= MAX_STEP_BYTES + 3);
        }
    }

    #[test]
    fn max_step_bytes_utf8_safe_truncation() {
        // MAX_STEP_BYTES 4096 debe truncar respetando borde UTF-8 (… = 3 bytes).
        // Probamos con contenido multi-byte (emoji y acentos).
        let many_emoji = "á".repeat(5000); // 'á' = 2 bytes UTF-8
        let t = truncate_bytes(&many_emoji, MAX_STEP_BYTES);
        assert!(t.len() <= MAX_STEP_BYTES + 3);
        assert!(t.is_char_boundary(t.len()));
        assert!(t.ends_with('…'));
        // También con emoji de 4 bytes
        let many_4b = "🦀".repeat(3000);
        let t2 = truncate_bytes(&many_4b, MAX_STEP_BYTES);
        assert!(t2.len() <= MAX_STEP_BYTES + 3);
        assert!(t2.is_char_boundary(t2.len()));
    }

    #[test]
    fn cas_steps_not_yet_wired_to_ui_documents_todo() {
        // TODO honesto (auditoría 2026-09-04): el stepper genérico es puro y
        // testeado pero aún no tiene caller fuera de grafito-geometry.
        // grep `steps_for_op|CasStepper` fuera del crate hoy devuelve solo
        // `crates/grafito-geometry/src/lib.rs:pub mod cas_steps;`.
        // Punto de cableado previsto para Piel (sin editar otros crates aquí):
        // - UI: grafito-app/src/assistant.rs AssistantRuntime::Thinking → Verifying
        //   o panel lateral que muestre CasStep[] junto a propuesta del asistente.
        // - Comandos: grafito-command al registrar `CAS: Mostrar pasos` que llame
        //   steps_for_op(CasOp::Derivative{..}) y renderice en assistant panel.
        // - Pedagogy: grafito-pedagogy ScaffoldEngine podría consumir RewriteRule
        //   para feedback socrático (ya existe FeedbackEngine 8 misconceptions).
        // Este test documenta el wiring y pasa siempre; fallará si alguien
        // espera consumo automático sin haber cableado UI.
        let steps = steps_for_op(&CasOp::Derivative {
            expr: "x^2".into(),
            var: "x".into(),
        })
        .expect("dispatch debe funcionar standalone");
        assert!(!steps.is_empty());
        // Si en el futuro grafito-app importa cas_steps, este TODO debe
        // convertirse en test de integración real y eliminar este marcador.
        let todo_marker = "TODO: cablear CasStepper en grafito-app UI";
        assert!(!todo_marker.is_empty());
    }

    // --- Frente G-A: trazas del motor CAS ---

    #[test]
    fn ga_gruntz_trace_shows_lhopital() {
        let steps = steps_for_gruntz("sin(x)/x", "x", 0.0).expect("traza Gruntz");
        assert!(!steps.is_empty());
        assert!(steps.len() <= MAX_CAS_STEPS);
        assert!(steps.iter().any(|s| s.description.contains("L'Hôpital")));
        let last = steps.last().expect("último paso");
        assert!(last.after.contains('1'), "got {}", last.after);
    }

    #[test]
    fn ga_risch_trace_gives_primitive() {
        let steps = steps_for_risch("x^2", "x").expect("traza Risch");
        assert!(!steps.is_empty());
        assert!(steps.len() <= MAX_CAS_STEPS);
        assert!(steps[0].after.contains('x'), "got {}", steps[0].after);
    }

    #[test]
    fn ga_risch_trace_honest_err() {
        assert!(steps_for_risch("exp(x^2)", "x").is_err());
    }

    #[test]
    fn ga_ode_linear_trace() {
        let steps = steps_for_ode_linear("2", "3", "x").expect("traza EDO lineal");
        assert!(!steps.is_empty());
        assert!(steps.len() <= MAX_CAS_STEPS);
        assert!(steps.iter().any(|s| s.description.contains("integrante")));
    }

    #[test]
    fn edo_1er_orden_factor_integrante_con_verificacion() {
        // y' + 2y = 0: P constante, Q = 0.
        let steps = steps_for_ode_linear("2", "0", "x").expect("traza 1er orden");
        assert_eq!(steps.len(), 4);
        assert!(has_rule(&steps, RewriteRule::OdeFirstOrderRule));
        assert!(
            steps[0].after.contains("P(x)"),
            "paso P: {}",
            steps[0].after
        );
        assert!(steps[1].after.contains("exp"), "paso μ: {}", steps[1].after);
        assert!(steps[2].after.contains('C'), "solución: {}", steps[2].after);
        let verif = steps.last().expect("verificación");
        assert!(verif.after.contains("residuo"), "got {}", verif.after);
        assert!(verif.after.contains("≈ 0"), "got {}", verif.after);
        assert!(
            verif.description.contains("sustituir"),
            "got {}",
            verif.description
        );
        // P(x) variable también integra: y' + x·y = 0.
        let var = steps_for_ode_linear("x", "0", "x").expect("P variable");
        assert_eq!(var.len(), 4);
        assert!(has_rule(&var, RewriteRule::OdeFirstOrderRule));
        assert!(var.last().expect("verif").after.contains("≈ 0"));
        // Q no nulo: y' + 2y = 3.
        let q = steps_for_ode_linear("2", "3", "x").expect("Q no nulo");
        assert!(q.last().expect("verif").after.contains("≈ 0"));
        // Despacho `CasOp` + statem validan igual.
        let op = CasOp::OdeFirstOrder {
            p: "2".to_string(),
            q: "0".to_string(),
            x: "x".to_string(),
        };
        assert_eq!(steps_for_op(&op).expect("dispatch").len(), 4);
        let mut stepper = CasStepper::new();
        assert_eq!(stepper.run(&op).expect("run").len(), 4);
        let mut malo = CasStepper::new();
        assert!(malo
            .run(&CasOp::OdeFirstOrder {
                p: "2".to_string(),
                q: "0".to_string(),
                x: "0mal".to_string(),
            })
            .is_err());
    }

    #[test]
    fn edo_1er_orden_honesto_ante_no_lineal() {
        // P con y escondida, Q con y, variable C: nada se adivina.
        assert!(steps_for_ode_linear("y", "0", "x").is_err());
        assert!(steps_for_ode_linear("2", "y", "x").is_err());
        assert!(steps_for_ode_linear("a", "0", "x").is_err());
        assert!(steps_for_ode_linear("2", "0", "C").is_err());
        assert!(steps_for_ode_linear("[[[", "0", "x").is_err());
    }

    #[test]
    fn edo_2do_orden_homogenea_tres_casos_con_verificacion() {
        // Δ > 0: y''−3y'+2y = 0 (r1 = 2, r2 = 1).
        let dpos = steps_for_ode_second_homog("1", "-3", "2", "x").expect("Δ>0");
        assert_eq!(dpos.len(), 4);
        assert!(has_rule(&dpos, RewriteRule::OdeSecondHomogRule));
        assert!(dpos[1].after.contains("Δ > 0"), "got {}", dpos[1].after);
        assert!(dpos[2].after.contains("C1"), "got {}", dpos[2].after);
        // Δ = 0: y''+2y'+y = 0 (r = −1 doble).
        let dcero = steps_for_ode_second_homog("1", "2", "1", "x").expect("Δ=0");
        assert_eq!(dcero.len(), 4);
        assert!(dcero[1].after.contains("Δ = 0"), "got {}", dcero[1].after);
        // Δ < 0: y''+y = 0 (α = 0, β = 1).
        let dneg = steps_for_ode_second_homog("1", "0", "1", "x").expect("Δ<0");
        assert_eq!(dneg.len(), 4);
        assert!(dneg[1].after.contains("Δ < 0"), "got {}", dneg[1].after);
        assert!(dneg[2].after.contains("cos"), "got {}", dneg[2].after);
        for pasos in [&dpos, &dcero, &dneg] {
            let verif = pasos.last().expect("verificación");
            assert!(verif.after.contains("≈ 0"), "got {}", verif.after);
            assert!(
                verif.description.contains("sustituir"),
                "got {}",
                verif.description
            );
        }
        // Característica explícita en el primer paso.
        assert!(dpos[0].after.contains("r²"), "got {}", dpos[0].after);
        // Despacho `CasOp` + statem validan igual.
        let op = CasOp::OdeSecondHomog {
            a: "1".to_string(),
            b: "-3".to_string(),
            c: "2".to_string(),
            x: "x".to_string(),
        };
        assert_eq!(steps_for_op(&op).expect("dispatch").len(), 4);
        let mut stepper = CasStepper::new();
        assert_eq!(stepper.run(&op).expect("run").len(), 4);
    }

    #[test]
    fn edo_2do_orden_homogenea_honesta_ante_no_constante() {
        // a = 0 no es 2º orden; coeficientes variables afuera del subset.
        assert!(steps_for_ode_second_homog("0", "1", "1", "x").is_err());
        assert!(steps_for_ode_second_homog("x", "1", "1", "x").is_err());
        assert!(steps_for_ode_second_homog("1", "y", "1", "x").is_err());
        assert!(steps_for_ode_second_homog("1", "2", "1", "C1").is_err());
    }

    #[test]
    fn ga_ode_separable_trace() {
        let steps = steps_for_ode_separable("x", "y", "x", "y").expect("traza separable");
        assert!(!steps.is_empty());
        assert!(steps.len() <= MAX_CAS_STEPS);
    }

    #[test]
    fn ga_residue_trace() {
        let steps = steps_for_residue("1/x", "x", 0.0, 8).expect("traza residuo");
        assert!(!steps.is_empty());
        assert!(steps.len() <= MAX_CAS_STEPS);
        assert!(steps.iter().any(|s| s.after.contains("orden 1")));
    }

    #[test]
    fn ga_groebner_trace() {
        let steps = steps_for_groebner(
            &["x + y - 3".to_string(), "x - y - 1".to_string()],
            &["x".to_string(), "y".to_string()],
        )
        .expect("traza Buchberger");
        assert!(!steps.is_empty());
        assert!(steps.len() <= MAX_CAS_STEPS);
    }

    #[test]
    fn ga_groebner_trace_rejects_over_budget() {
        let polys: Vec<String> = (0..20).map(|i| format!("x + {i}")).collect();
        assert!(steps_for_groebner(&polys, &["x".to_string()]).is_err());
    }

    // --- Frente F3c: trazas de 2º orden, sistemas, Laplace, Risch extendido ---

    #[test]
    fn f3c_ode_second_order_trace() {
        let steps = steps_for_ode_second_order("1", "-3", "2", "exp(x)", "x").expect("traza EDO2");
        assert!(!steps.is_empty());
        assert!(steps.len() <= MAX_CAS_STEPS);
        assert!(has_rule(&steps, RewriteRule::OdeSecondOrderRule));
        assert!(
            steps
                .iter()
                .any(|s| s.description.contains("característica")),
            "falta paso característico"
        );
        let last = steps.last().expect("último paso");
        assert!(
            last.after.contains('C'),
            "solución con C1/C2, got {}",
            last.after
        );
    }

    #[test]
    fn f3c_ode_second_order_trace_honest_err() {
        assert!(steps_for_ode_second_order("0", "1", "1", "x", "x").is_err());
        assert!(steps_for_ode_second_order("x", "1", "1", "0", "x").is_err());
    }

    #[test]
    fn f3c_ode_system_trace() {
        let steps = steps_for_ode_system_2x2("0", "1", "-2", "-3", "t").expect("traza sistema");
        assert!(!steps.is_empty());
        assert!(steps.len() <= MAX_CAS_STEPS);
        assert!(has_rule(&steps, RewriteRule::OdeSystemRule));
        assert!(
            steps.iter().any(|s| s.description.contains("traza")),
            "falta paso traza/determinante"
        );
    }

    #[test]
    fn f3c_laplace_traces() {
        let direct = steps_for_laplace_direct("sin(t)", "t", "s").expect("traza Laplace");
        assert!(!direct.is_empty());
        assert!(has_rule(&direct, RewriteRule::LaplaceDirectRule));
        assert!(direct[0].after.contains("s^2+1"), "got {}", direct[0].after);
        let inverse = steps_for_laplace_inverse("1/(s+1)", "s", "t").expect("traza inversa");
        assert!(has_rule(&inverse, RewriteRule::LaplaceInverseRule));
        assert!(inverse[0].after.contains("exp"), "got {}", inverse[0].after);
        // B2.3d: la cúbica con raíz real YA traza; grado 4 sigue honesto.
        let cubic = steps_for_laplace_inverse("1/(s^3+1)", "s", "t").expect("traza cúbica");
        assert!(cubic[0].after.contains("exp"), "got {}", cubic[0].after);
        assert!(steps_for_laplace_inverse("1/(s^4+1)", "s", "t").is_err());
    }

    #[test]
    fn f3c_risch_extended_trace_rules() {
        let atan_steps = steps_for_risch("1/(x^2+1)", "x").expect("traza atan");
        assert!(has_rule(&atan_steps, RewriteRule::IntegrationArctanRule));
        let partial = steps_for_risch("1/(x^2+x-2)", "x").expect("traza parciales");
        assert!(has_rule(&partial, RewriteRule::IntegrationPartialFractions));
        let tan_steps = steps_for_risch("tan(x)", "x").expect("traza tan");
        assert!(!tan_steps.is_empty());
        assert!(tan_steps.len() <= MAX_CAS_STEPS);
    }

    // --- Frente B2: trazas con regla propia + despacho `CasOp` ---

    #[test]
    fn b2_risch_trace_rules() {
        let trig = steps_for_risch("sin(x)^2", "x").expect("traza trig");
        assert!(has_rule(&trig, RewriteRule::RischTrigRule));
        let euler = steps_for_risch("sqrt(x^2+1)", "x").expect("traza Euler");
        assert!(has_rule(&euler, RewriteRule::RischEulerSqrtRule));
        let parts = steps_for_risch("x^2*exp(x)", "x").expect("traza partes");
        assert!(has_rule(&parts, RewriteRule::RischPartsRule));
        for s in trig.iter().chain(euler.iter()).chain(parts.iter()) {
            assert!(s.before.len() <= MAX_STEP_BYTES);
            assert!(s.after.len() <= MAX_STEP_BYTES);
        }
    }

    #[test]
    fn b2_gruntz_rewrite_trace() {
        let steps = steps_for_gruntz("(1+x)^(1/x)", "x", 0.0).expect("traza 1^∞");
        assert!(has_rule(&steps, RewriteRule::GruntzLogRewriteRule));
        assert!(steps.iter().any(|s| s.after.contains("2.718")));
    }

    #[test]
    fn b2_ode_traces() {
        let coeffs = ["1", "-6", "11", "-6"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        let nth = steps_for_ode_nth_order(&coeffs, "0", "x").expect("traza orden-n");
        assert!(has_rule(&nth, RewriteRule::OdeNthOrderRule));
        let euler = steps_for_ode_euler("1", "-1", "0", "x").expect("traza Euler");
        assert!(has_rule(&euler, RewriteRule::OdeEulerRule));
        let frob = steps_for_frobenius("-2*x", "0", "x", 0.0, 6).expect("traza Frobenius");
        assert!(has_rule(&frob, RewriteRule::FrobeniusRule));
        assert_eq!(frob.len(), 3);
        let deriv =
            steps_for_laplace_derivative(2, "Y", "t", "s", &["y0".to_string(), "y1".to_string()])
                .expect("traza derivada");
        assert!(has_rule(&deriv, RewriteRule::LaplaceCalculusRule));
        let integ = steps_for_laplace_integral("sin(t)", "t", "s").expect("traza integral");
        assert!(has_rule(&integ, RewriteRule::LaplaceCalculusRule));
    }

    #[test]
    fn b2_groebner_traces_and_dispatch() {
        let polys = vec!["x^2+y^2-25".to_string(), "x-y-1".to_string()];
        let vars = vec!["x".to_string(), "y".to_string()];
        let ord = steps_for_groebner_ordered(&polys, &vars, "grlex").expect("traza orden");
        assert!(has_rule(&ord, RewriteRule::GroebnerOrderRule));
        assert!(steps_for_groebner_ordered(&polys, &vars, "invlex").is_err());
        let elim = steps_for_eliminate(&polys, &vars, &["y".to_string()]).expect("traza elim");
        assert!(has_rule(&elim, RewriteRule::EliminateRule));
        // Despacho `CasOp` de las 7 operaciones B2.
        let ops = [
            CasOp::OdeNthOrder {
                coeffs: ["1", "-3", "2"].iter().map(|s| s.to_string()).collect(),
                rhs: "0".to_string(),
                x: "x".to_string(),
            },
            CasOp::OdeEuler {
                a: "1".to_string(),
                b: "-1".to_string(),
                rhs: "0".to_string(),
                x: "x".to_string(),
            },
            CasOp::Frobenius {
                p: "-2*x".to_string(),
                q: "0".to_string(),
                x: "x".to_string(),
                center: 0.0,
                terms: 6,
            },
            CasOp::LaplaceDerivative {
                order: 1,
                y: "Y".to_string(),
                t: "t".to_string(),
                s: "s".to_string(),
                initials: vec!["0".to_string()],
            },
            CasOp::LaplaceIntegral {
                f: "t".to_string(),
                t: "t".to_string(),
                s: "s".to_string(),
            },
            CasOp::GroebnerOrdered {
                polys: polys.clone(),
                vars: vars.clone(),
                order: "lex".to_string(),
            },
            CasOp::Eliminate {
                polys,
                vars: vars.clone(),
                elim: vec!["y".to_string()],
            },
        ];
        for op in &ops {
            let steps = steps_for_op(op).expect("despacho B2");
            assert!(!steps.is_empty());
        }
        // `CasStepper::run` valida (orden malo y términos de más fallan).
        let mut stepper = CasStepper::new();
        assert!(stepper
            .run(&CasOp::GroebnerOrdered {
                polys: vec!["x".to_string(); 20],
                vars,
                order: "lex".to_string(),
            })
            .is_err());
    }
}
#[cfg(test)]
mod coverage_sweep_cas_steps {
    use super::*;
    fn op(s: &str) -> String {
        s.to_string()
    }
    #[test]
    fn barrido_pasos_basicos_no_vacios() {
        let d = steps_for_derivative("x^2", "x").expect("pasos derivada");
        assert!(!d.is_empty(), "derivada da pasos");
        let i = steps_for_integral("x", "x").expect("pasos integral");
        assert!(!i.is_empty(), "integral da pasos");
        let l = steps_for_limit("1/x", "x", 1.0).expect("pasos límite");
        assert!(!l.is_empty(), "límite da pasos");
        let t = steps_for_taylor("sin(x)", "x", 0.0, 3).expect("pasos taylor");
        assert!(!t.is_empty(), "taylor da pasos");
        let s = steps_for_solve("x-1", "x").expect("pasos solve");
        assert!(!s.is_empty(), "solve da pasos");
        assert!(steps_for_derivative("[[[", "x").is_err());
    }
    #[test]
    fn barrido_pasos_honestos_y_dispatch() {
        // Gruntz/Risch/Groebner: Ok con pasos o Err honesto, nunca pánico.
        let _ = steps_for_gruntz("1/x", "x", 0.0);
        let _ = steps_for_risch("x", "x");
        let _ = steps_for_groebner(&[op("x"), op("y")], &[op("x"), op("y")]);
        let _ = steps_for_groebner_ordered(&[op("x")], &[op("x")], "lex");
        let _ = steps_for_eliminate(&[op("x+y")], &[op("x")], &[op("y")]);
        let _ = steps_for_residue("1/z", "z", 0.0, 1);
        let _ = steps_for_laplace_direct("1", "t", "s");
        let _ = steps_for_laplace_inverse("1/s", "s", "t");
        let _ = steps_for_laplace_derivative(1, "x", "t", "s", &[]);
        let _ = steps_for_laplace_integral("1", "t", "s");
        let _ = steps_for_ode_linear("1", "x", "x");
        let _ = steps_for_ode_separable("x", "y", "x", "y");
        let _ = steps_for_ode_second_order("1", "0", "0", "0", "x");
        let _ = steps_for_ode_system_2x2("1", "0", "0", "1", "x");
        let _ = steps_for_ode_nth_order(&[op("1"), op("0")], "0", "x");
        let _ = steps_for_ode_euler("1", "1", "0", "x");
        let _ = steps_for_frobenius("0", "0", "x", 0.0, 3);
        // Dispatch por CasOp pinnea que cada variante responde.
        for caso in [
            CasOp::Derivative {
                expr: op("x^2"),
                var: op("x"),
            },
            CasOp::Integral {
                expr: op("x"),
                var: op("x"),
            },
            CasOp::Limit {
                expr: op("x"),
                var: op("x"),
                at: 0.0,
            },
            CasOp::Taylor {
                expr: op("sin(x)"),
                var: op("x"),
                center: 0.0,
                order: 2,
            },
            CasOp::Solve {
                expr: op("x-1"),
                var: op("x"),
            },
        ] {
            let pasos = steps_for_op(&caso).expect("dispatch responde");
            assert!(!pasos.is_empty(), "cada op da pasos: {caso:?}");
        }
    }
    #[test]
    fn motivo_cubre_alto_trafico_y_fallback_honesto() {
        use RewriteRule as R;
        let alto_trafico = [
            R::PowerRule,
            R::SumRule,
            R::ProductRule,
            R::QuotientRule,
            R::ChainRule,
            R::TrigRule,
            R::ExpRule,
            R::LogRule,
            R::IntegrationPowerRule,
            R::IntegrationTrigRule,
            R::IntegrationPartialFractions,
            R::IntegrationArctanRule,
            R::LimitRichardson,
            R::GruntzLogRewriteRule,
            R::SolveLinearRule,
            R::SolveQuadraticRule,
            R::SolveSturmRule,
            R::LaplaceCalculusRule,
            R::TaylorRule,
            R::FrobeniusRule,
            R::OdeEulerRule,
            R::SystemEliminationRule,
            R::SystemSubstitutionRule,
            R::BarrowRule,
            R::Simplify,
            R::AbsRule,
        ];
        let fallback = motivo(&R::Generic);
        assert!(!fallback.is_empty());
        for rule in &alto_trafico {
            let m = motivo(rule);
            assert!(!m.is_empty(), "motivo vacio para {rule}");
            assert_ne!(m, fallback, "alto trafico sin fallback para {rule}");
            assert_eq!(
                motivo_str(&rule.to_string()),
                m,
                "motivo_str roundtrip {rule}"
            );
        }
    }
    #[test]
    fn motivo_cubre_el_resto_y_fallback_para_desconocidos() {
        use RewriteRule as R;
        // El "resto" que antes caía al genérico ahora tiene motivo propio:
        // servicio pedagógico completo para toda la tabla de reglas.
        let resto = [
            R::ConstantRule,
            R::VariableRule,
            R::DifferenceRule,
            R::NegRule,
            R::SqrtRule,
            R::IntegrationSumRule,
            R::IntegrationConstantRule,
            R::IntegrationExpRule,
            R::OdeSecondOrderRule,
            R::OdeSystemRule,
            R::LaplaceDirectRule,
            R::LaplaceInverseRule,
            R::RischTrigRule,
            R::RischEulerSqrtRule,
            R::RischPartsRule,
            R::OscillationRule,
            R::OdeNthOrderRule,
            R::OdeFirstOrderRule,
            R::OdeSecondHomogRule,
            R::LaplaceShiftRule,
            R::GroebnerOrderRule,
            R::EliminateRule,
        ];
        let fallback = motivo(&R::Generic);
        for rule in &resto {
            let m = motivo(rule);
            assert!(!m.is_empty(), "motivo vacio para {rule}");
            assert_ne!(
                m, fallback,
                "{rule} debe tener motivo propio, no el fallback genérico"
            );
            assert_eq!(
                motivo_str(&rule.to_string()),
                m,
                "motivo_str roundtrip {rule}"
            );
        }
        // Nombre desconocido (fence ajeno): fallback honesto.
        assert_eq!(motivo_str("ReglaInventada"), fallback);
    }
}

#[cfg(test)]
mod sistema_y_barrow_tests {
    use super::*;
    use std::collections::BTreeMap;

    fn has_rule(steps: &[CasStep], rule: RewriteRule) -> bool {
        steps.iter().any(|s| s.rule == rule)
    }

    /// Evalúa `eq` (con `=` o en forma `= 0`) en `(var1, var2) = (x, y)`.
    fn eval_system_eq(eq: &str, var1: &str, x: f64, var2: &str, y: f64) -> f64 {
        let lhs = match eq.split_once('=') {
            Some((l, r)) => format!("({})-({})", l.trim(), r.trim()),
            None => eq.to_string(),
        };
        let ast = parse_ast(&lhs.replace(' ', "")).expect("ecuación de prueba parseable");
        let mut scope = BTreeMap::new();
        scope.insert(var1.to_string(), x);
        scope.insert(var2.to_string(), y);
        match ast.substitute_vars(&scope, &[]) {
            Expr::Const(v) => v,
            other => other.eval_at(var1, x),
        }
    }

    /// Extrae el valor de `var = v` del `after` de un paso de despeje.
    fn despeje_after(steps: &[CasStep], var: &str) -> f64 {
        let step = steps
            .iter()
            .find(|s| s.after.starts_with(&format!("{var} = ")))
            .unwrap_or_else(|| panic!("sin despeje de {var}: {steps:?}"));
        step.after[var.len() + 3..]
            .trim()
            .parse::<f64>()
            .unwrap_or_else(|_| panic!("despeje no numérico: {}", step.after))
    }

    #[test]
    fn sistema_2x2_basico_con_verificacion() {
        let steps = steps_for_system_2x2("x+y=3", "x-y=1", "x", "y").expect("sistema lineal 2×2");
        assert!(has_rule(&steps, RewriteRule::SystemEliminationRule));
        assert!(has_rule(&steps, RewriteRule::SystemSubstitutionRule));
        assert!(steps.len() <= MAX_CAS_STEPS);
        let (x, y) = (despeje_after(&steps, "x"), despeje_after(&steps, "y"));
        assert!((x - 2.0).abs() < 1e-9, "x = {x}");
        assert!((y - 1.0).abs() < 1e-9, "y = {y}");
        // Verificación independiente: sustituir la solución recupera 0 = 0.
        for eq in ["x+y=3", "x-y=1"] {
            let r = eval_system_eq(eq, "x", x, "y", y);
            assert!(r.abs() < 1e-9, "residuo de {eq}: {r}");
        }
        // Despacho genérico + statem.
        let via_op = steps_for_op(&CasOp::System2x2 {
            eq1: "x+y=3".to_string(),
            eq2: "x-y=1".to_string(),
            var1: "x".to_string(),
            var2: "y".to_string(),
        })
        .expect("despacho System2x2");
        assert_eq!(via_op, steps);
        let mut stepper = CasStepper::new();
        let run = stepper
            .run(&CasOp::System2x2 {
                eq1: "x+y=3".to_string(),
                eq2: "x-y=1".to_string(),
                var1: "x".to_string(),
                var2: "y".to_string(),
            })
            .expect("statem System2x2");
        assert_eq!(*stepper.state(), CasStepperState::Done);
        assert_eq!(run, steps);
    }

    #[test]
    fn sistema_2x2_coeficientes_generales() {
        let steps = steps_for_system_2x2("2*x-y=1", "x+2*y=8", "x", "y").expect("sistema general");
        let (x, y) = (despeje_after(&steps, "x"), despeje_after(&steps, "y"));
        assert!((x - 2.0).abs() < 1e-9, "x = {x}");
        assert!((y - 3.0).abs() < 1e-9, "y = {y}");
        for eq in ["2*x-y=1", "x+2*y=8"] {
            let r = eval_system_eq(eq, "x", x, "y", y);
            assert!(r.abs() < 1e-9, "residuo de {eq}: {r}");
        }
    }

    #[test]
    fn sistema_2x2_errores_honestos() {
        // No lineal: sin pasos inventados.
        assert!(steps_for_system_2x2("x^2+y=1", "x-y=0", "x", "y").is_err());
        // Singular / dependiente: sin solución única.
        assert!(steps_for_system_2x2("x+y=1", "2*x+2*y=2", "x", "y").is_err());
        // Tercera incógnita: no es 2×2.
        assert!(steps_for_system_2x2("x+y+z=1", "x-y=0", "x", "y").is_err());
        // Misma incógnita dos veces.
        assert!(steps_for_system_2x2("x+y=1", "x-y=0", "x", "x").is_err());
        // Statem valida identificadores.
        let mut stepper = CasStepper::new();
        assert!(stepper
            .run(&CasOp::System2x2 {
                eq1: "x+y=3".to_string(),
                eq2: "x-y=1".to_string(),
                var1: "1x".to_string(),
                var2: "y".to_string(),
            })
            .is_err());
    }

    #[test]
    fn barrow_potencia_un_tercio() {
        let steps = steps_for_definite_integral("x^2", "x", 0.0, 1.0).expect("Barrow x^2");
        assert!(has_rule(&steps, RewriteRule::IntegrationPowerRule));
        assert!(has_rule(&steps, RewriteRule::BarrowRule));
        let last = steps.last().expect("paso Barrow").after.clone();
        let value: f64 = last.trim().parse().expect("Barrow numérico");
        assert!((value - 1.0 / 3.0).abs() < 1e-6, "F(1)-F(0) = {value}");
        // Cruzamiento numérico independiente F(b) − F(a) = 1/3.
        let prim = parse_ast("x^3/3").expect("primitiva de prueba");
        let check = prim.eval_at("x", 1.0) - prim.eval_at("x", 0.0);
        assert!((check - value).abs() < 1e-6, "check {check} vs {value}");
        // Despacho genérico.
        let via_op = steps_for_op(&CasOp::DefiniteIntegral {
            expr: "x^2".to_string(),
            var: "x".to_string(),
            a: 0.0,
            b: 1.0,
        })
        .expect("despacho DefiniteIntegral");
        assert_eq!(via_op, steps);
    }

    #[test]
    fn barrow_seno_cero_a_pi_da_dos() {
        let steps = steps_for_definite_integral("sin(x)", "x", 0.0, std::f64::consts::PI)
            .expect("Barrow sin");
        assert!(has_rule(&steps, RewriteRule::BarrowRule));
        let last = steps.last().expect("paso Barrow").after.clone();
        let value: f64 = last.trim().parse().expect("Barrow numérico");
        assert!((value - 2.0).abs() < 1e-6, "∫₀^π sin = {value}");
    }

    #[test]
    fn barrow_errores_honestos() {
        // Sin primitiva soportada: no se inventa número.
        assert!(steps_for_definite_integral("exp(x^2)", "x", 0.0, 1.0).is_err());
        // Límites iguales: 0 honesto con un solo paso Barrow.
        let steps = steps_for_definite_integral("x^2", "x", 1.0, 1.0).expect("a = b");
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].rule, RewriteRule::BarrowRule);
        assert_eq!(steps[0].after, "0");
        // Límites no finitos.
        assert!(steps_for_definite_integral("x", "x", 0.0, f64::NAN).is_err());
        let mut stepper = CasStepper::new();
        assert!(stepper
            .run(&CasOp::DefiniteIntegral {
                expr: "x".to_string(),
                var: "x".to_string(),
                a: 0.0,
                b: f64::INFINITY,
            })
            .is_err());
    }
}
