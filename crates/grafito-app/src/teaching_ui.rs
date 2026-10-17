//! UI de enseñanza — integra TeachingSession + Whiteboard + ManimOrchestrator.
//!
//! Muestra burbujas morph desde el avatar, pizarra para cada paso y
//! controles para avanzar. Usa `grafito-whiteboard` para dibujo y
//! `manim_orchestrator` para animaciones 3b1b (con fallback nativo).

use crate::anim_ui::{RetentionQueue, TEXTURE_GRACE_FRAMES};
use crate::manim_orchestrator::{ManimOrchestrator, OrchestratorState};

// La gracia cubre el submit GPU en vuelo (N≥2); verificado a compile-time
// (clippy `assertions_on_constants` prohíbe re-chequear la const en runtime).
const _: () = assert!(TEXTURE_GRACE_FRAMES >= 2);
use crate::whiteboard_ui::WhiteboardSession;
use egui::{Color32, Stroke};
use grafito_pedagogy::{
    Curriculum, Exercise, ExerciseGenerator, LearningObjective, PedagogicalLevel, TeachingSession,
};
use grafito_ui::assistant::{
    draw_math_with_locale, humanize_prose_text, AssistantBlocksCache, AssistantMessageBlock,
};
use grafito_ui::i18n::teach_msg;
use grafito_ui::icons::{action_icon_button, Icon};
use grafito_whiteboard::{WhiteboardDoc, WhiteboardElement};
use std::time::{Duration, Instant};

/// Ancho ideal de la ventana de enseñanza (SPACE_XXL * 16 = 640, tokenizado).
const VENTANA_ANCHO_IDEAL: f32 = grafito_ui::tokens::SPACE_XXL * 16.0;
/// Ancho mínimo de la ventana (SPACE_XXL * 12 = 480).
const VENTANA_ANCHO_MIN: f32 = grafito_ui::tokens::SPACE_XXL * 12.0;
/// Ancho máximo de la ventana (SPACE_XXL * 20 = 800).
const VENTANA_ANCHO_MAX: f32 = grafito_ui::tokens::SPACE_XXL * 20.0;
/// Alto ideal de la ventana (SPACE_XXL * 13 = 520).
const VENTANA_ALTO_IDEAL: f32 = grafito_ui::tokens::SPACE_XXL * 13.0;

/// Estado de la UI de enseñanza.
pub struct TeachingUiState {
    pub session: Option<TeachingSession>,
    pub whiteboard: WhiteboardSession,
    pub orchestrator: ManimOrchestrator,
    pub show_manim: bool,
    /// Instant en que se abrió la overlay — para morph avatar→burbuja (ANIM_MICRO 180ms ease-out).
    pub opened_at: Option<Instant>,
    /// WCAG 2.2.2: pausa manual del loop de la animación (botón Icon::Pause).
    pub paused: bool,
    /// Elapsed (ms) congelado al pausar: quedan cue/idx estables en pausa.
    pub frozen_ms: Option<u64>,
    /// Frames nativos generados al completar el orchestrator (fallback sin Manim).
    pub anim_frames: Option<Vec<egui::ColorImage>>,
    /// Texturas cacheadas de `anim_frames` (creadas lazily con `ctx.load_texture`).
    pub anim_textures: Vec<egui::TextureHandle>,
    /// Sets viejos retirados con gracia diferida (fix `Queue::submit`
    /// `egui_texid_Managed(N) has been destroyed`).
    ///
    /// `ensure_textures` / `clear_*` mueven acá el set anterior en vez de
    /// dropearlo: el submit GPU en vuelo puede referenciarlo. Se libera
    /// tras `TEXTURE_GRACE_FRAMES` frames dibujados (ver
    /// `crate::anim_ui::RetentionQueue`). Estado estable: 1 set viejo.
    retired_anim_textures: RetentionQueue<egui::TextureHandle>,
    /// Panel de ejercicio B7 (pedir→generar→responder→próximo). Vacío por defecto.
    pub ejercicio: PanelEjercicio,
    cached_hash: u64,
    cached_len: usize,
    /// Parser del transcript (misma gramática que el asistente): evita
    /// re-parsear el paso en cada frame. Fuente única de markdown.
    bloques: AssistantBlocksCache,
    /// Inicio de la sesión para el binding temporal por cue
    /// (`TeachingSession::revealed_*`): la pizarra hidrata por cue en
    /// `poll_cues`, en vez de todo el hint de golpe.
    session_started_at: Option<Instant>,
    /// Última cantidad revelada (evita re-hidratar sin cambio de cue).
    last_revealed: usize,
    /// El usuario navegó manualmente (advance/retroceder): los cues ya no
    /// pisan la pizarra (manda la navegación explícita).
    navigated: bool,
}

#[allow(clippy::derivable_impls)]
impl Default for TeachingUiState {
    fn default() -> Self {
        Self {
            session: None,
            whiteboard: WhiteboardSession::default(),
            orchestrator: ManimOrchestrator::default(),
            show_manim: false,
            opened_at: None,
            paused: false,
            frozen_ms: None,
            anim_frames: None,
            anim_textures: Vec::new(),
            retired_anim_textures: RetentionQueue::new(),
            ejercicio: PanelEjercicio::default(),
            cached_hash: 0,
            cached_len: 0,
            bloques: AssistantBlocksCache::default(),
            session_started_at: None,
            last_revealed: 0,
            navigated: false,
        }
    }
}

/// Clasificación pura del hint de pizarra en 14 temas más casos borde.
///
/// Es `pub` y totalmente testeable (sin egui, sin I/O): dado el texto del
/// paso pedagógico decide qué dibujar. Los chequeos de [`hint_for_topic`]
/// están ordenados por especificidad —los más específicos primero— para que
/// un hint como "triángulo rectángulo" caiga en Pitágoras y no en el
/// genérico Área (que también matchea "rectángulo").
///
/// NOTA DEUDA: extraer a `grafito-pedagogy` (p. ej. `TeachingTopic::hint_kind`)
/// cuando el DAG lo permita; la Piel sólo debería consumir el enum, no el
/// string-matching. Ver [`whiteboard_elements_for_hint`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhiteboardHint {
    /// Hint vacío o sólo espacios: no dibuja nada.
    Vacio,
    Secante,
    Fraccion,
    Vector,
    Matriz,
    Probabilidad,
    Serie,
    Trigonometria,
    Conica,
    Ecuacion,
    Limite,
    Funcion,
    Area,
    Pitagoras,
    /// «pizarra libre»: dibuja el usuario, no se hidrata nada.
    Libre,
    /// Hint desconocido: texto centrado como fallback.
    General,
}

/// Clasifica el hint de un paso de enseñanza (puro, sin I/O).
///
/// Ordenado por especificidad: Pitágoras precede a Área porque
/// "triángulo rectángulo" contiene "rectángulo".
pub fn hint_for_topic(topic: &str) -> WhiteboardHint {
    let lower = topic.to_lowercase();
    if topic.trim().is_empty() {
        return WhiteboardHint::Vacio;
    }
    if lower.contains("pitágoras")
        || lower.contains("pitagoras")
        || lower.contains("triángulo rectángulo")
        || lower.contains("triangulo rectangulo")
        || lower.contains("cuadrados en cada lado")
    {
        return WhiteboardHint::Pitagoras;
    }
    if lower.contains("secante") || lower.contains("tangente") || lower.contains("pendiente") {
        return WhiteboardHint::Secante;
    }
    if lower.contains("fracc")
        || lower.contains("dividido en partes")
        || lower.contains("común denominador")
        || lower.contains("comun denominador")
        || lower.contains("fracciones equivalentes")
        || lower.contains("rectángulo dividido")
        || lower.contains("rectangulo dividido")
    {
        return WhiteboardHint::Fraccion;
    }
    if lower.contains("vector")
        || lower.contains("flecha en ejes")
        || lower.contains("dos flechas")
        || lower.contains("pizarra vectorial")
        || lower.contains("r²")
        || lower.contains("r2")
    {
        return WhiteboardHint::Vector;
    }
    if lower.contains("matriz")
        || lower.contains("matrices")
        || lower.contains("grilla")
        || lower.contains("2x2")
        || lower.contains("determinante")
        || lower.contains("gauss")
        || lower.contains("pivote")
        || lower.contains("aumentada")
    {
        return WhiteboardHint::Matriz;
    }
    if lower.contains("probab")
        || lower.contains("histograma")
        || lower.contains("árbol")
        || lower.contains("arbol")
        || lower.contains("contingencia")
        || lower.contains("normal")
        || lower.contains("bayes")
        || lower.contains("muestreo")
        || lower.contains("binom")
        || lower.contains("distrib")
        || lower.contains("curva normal")
    {
        return WhiteboardHint::Probabilidad;
    }
    if lower.contains("serie")
        || lower.contains("taylor")
        || lower.contains("fourier")
        || lower.contains("sucesi")
        || lower.contains("geométrica")
        || lower.contains("geometrica")
        || lower.contains("suma parcial")
        || lower.contains("aproxima")
    {
        return WhiteboardHint::Serie;
    }
    if lower.contains("trigon")
        || lower.contains("seno")
        || lower.contains("coseno")
        || lower.contains("círculo unitario")
        || lower.contains("circulo unitario")
        || lower.contains("onda seno")
        || lower.contains("sin(")
        || lower.contains("cos(")
    {
        return WhiteboardHint::Trigonometria;
    }
    if lower.contains("cónica")
        || lower.contains("conica")
        || lower.contains("cono cortado")
        || lower.contains("elipse con focos")
        || lower.contains("hipérbola")
        || lower.contains("hiperbola")
        || (lower.contains("elipse") && lower.contains("focos"))
    {
        return WhiteboardHint::Conica;
    }
    if lower.contains("ecuac")
        || lower.contains("cuadrática")
        || lower.contains("cuadratica")
        || lower.contains("parábola y raíces")
        || lower.contains("parabola y raices")
        || lower.contains("discriminante")
        || lower.contains("recta y corte")
        || lower.contains("dos rectas")
        || lower.contains("sistema")
    {
        return WhiteboardHint::Ecuacion;
    }
    if lower.contains("límite")
        || lower.contains("limite")
        || lower.contains("hueco en a")
        || lower.contains("flechas hacia a")
        || lower.contains("cálculo paso a paso")
        || lower.contains("calculo paso a paso")
    {
        return WhiteboardHint::Limite;
    }
    if lower.contains("función")
        || lower.contains("funcion")
        || lower.contains("ejes con puntos")
        || lower.contains("tabla de valores")
        || lower.contains("curva con ejes")
        || lower.contains("ejemplo con gráfica")
        || lower.contains("gráfica interactiva")
    {
        return WhiteboardHint::Funcion;
    }
    if lower.contains("rectángulo")
        || lower.contains("rectangulo")
        || lower.contains("riemann")
        || lower.contains("área")
        || lower.contains("area")
    {
        return WhiteboardHint::Area;
    }
    if lower.contains("pizarra libre") || lower.contains("libre") {
        return WhiteboardHint::Libre;
    }
    WhiteboardHint::General
}

/// Constructor puro de elementos para secante (ver [`hint_for_topic`]).
fn push_secante_hint(elems: &mut Vec<grafito_whiteboard::WhiteboardElement>) {
    use grafito_whiteboard::WhiteboardElement;
    // Curva x² como trazo suave + secante + tangente
    elems.push(WhiteboardElement::Stroke {
        points: vec![
            (-3.0, 9.0),
            (-2.0, 4.0),
            (-1.0, 1.0),
            (0.0, 0.0),
            (1.0, 1.0),
            (2.0, 4.0),
        ],
        color: (55, 55, 55),
        width: 2.0,
    });
    elems.push(WhiteboardElement::Arrow {
        from: (-1.0, 1.0),
        to: (1.0, 1.0),
    });
    elems.push(WhiteboardElement::Arrow {
        from: (0.5, 0.25),
        to: (1.5, 2.25),
    });
}

/// Constructor puro de elementos para fraccion (ver [`hint_for_topic`]).
fn push_fraccion_hint(elems: &mut Vec<grafito_whiteboard::WhiteboardElement>) {
    use grafito_whiteboard::WhiteboardElement;
    // Fracción — rectángulo dividido en 3 + sombreado 2/3 + texto
    elems.push(WhiteboardElement::Rectangle {
        min: (0.0, 0.0),
        max: (2.4, 1.0),
        fill: None,
    });
    elems.push(WhiteboardElement::Stroke {
        points: vec![(0.8, 0.0), (0.8, 1.0)],
        color: (55, 55, 55),
        width: 1.4,
    });
    elems.push(WhiteboardElement::Stroke {
        points: vec![(1.6, 0.0), (1.6, 1.0)],
        color: (55, 55, 55),
        width: 1.4,
    });
    // Sombrear dos tercios
    elems.push(WhiteboardElement::Rectangle {
        min: (0.05, 0.05),
        max: (0.75, 0.95),
        fill: Some((135, 180, 255)),
    });
    elems.push(WhiteboardElement::Rectangle {
        min: (0.85, 0.05),
        max: (1.55, 0.95),
        fill: Some((135, 180, 255)),
    });
    elems.push(WhiteboardElement::Text {
        at: (0.15, -0.35),
        text: "2/3 del rectángulo".into(),
        size: 10.0,
    });
    elems.push(WhiteboardElement::Text {
        at: (0.2, 1.2),
        text: "1/2 + 1/3 = 5/6".into(),
        size: 9.0,
    });
}

/// Constructor puro de elementos para vector (ver [`hint_for_topic`]).
fn push_vector_hint(elems: &mut Vec<grafito_whiteboard::WhiteboardElement>) {
    use grafito_whiteboard::WhiteboardElement;
    // Vector — ejes + dos flechas (u, v) y suma tip-to-tail
    elems.push(WhiteboardElement::Stroke {
        points: vec![(-1.5, 0.0), (1.8, 0.0)],
        color: (55, 55, 55),
        width: 1.2,
    });
    elems.push(WhiteboardElement::Stroke {
        points: vec![(0.0, -1.2), (0.0, 1.6)],
        color: (55, 55, 55),
        width: 1.2,
    });
    elems.push(WhiteboardElement::Arrow {
        from: (0.0, 0.0),
        to: (1.2, 0.8),
    });
    elems.push(WhiteboardElement::Arrow {
        from: (0.0, 0.0),
        to: (0.6, 1.3),
    });
    // Suma visual tip-to-tail
    elems.push(WhiteboardElement::Arrow {
        from: (1.2, 0.8),
        to: (1.8, 2.1),
    });
    elems.push(WhiteboardElement::Text {
        at: (0.35, 0.55),
        text: "v=(2,3)".into(),
        size: 10.0,
    });
    elems.push(WhiteboardElement::Text {
        at: (0.15, 1.35),
        text: "u".into(),
        size: 10.0,
    });
    elems.push(WhiteboardElement::Text {
        at: (1.25, 1.95),
        text: "u+v".into(),
        size: 9.0,
    });
}

/// Constructor puro de elementos para matriz (ver [`hint_for_topic`]).
fn push_matriz_hint(elems: &mut Vec<grafito_whiteboard::WhiteboardElement>) {
    use grafito_whiteboard::WhiteboardElement;
    // Matriz — grilla 2×2 + etiquetas + determinante
    for row in 0..2 {
        for col in 0..2 {
            let x0 = col as f64 * 0.95;
            let y0 = row as f64 * 0.70;
            elems.push(WhiteboardElement::Rectangle {
                min: (x0, y0),
                max: (x0 + 0.85, y0 + 0.60),
                fill: None,
            });
        }
    }
    // diagonales para pivotes
    elems.push(WhiteboardElement::Stroke {
        points: vec![(0.05, 0.05), (0.80, 0.55)],
        color: (66, 133, 244),
        width: 1.5,
    });
    elems.push(WhiteboardElement::Stroke {
        points: vec![(1.00, 0.75), (1.75, 1.25)],
        color: (66, 133, 244),
        width: 1.5,
    });
    elems.push(WhiteboardElement::Text {
        at: (0.22, 0.22),
        text: "1".into(),
        size: 10.0,
    });
    elems.push(WhiteboardElement::Text {
        at: (1.17, 0.22),
        text: "2".into(),
        size: 10.0,
    });
    elems.push(WhiteboardElement::Text {
        at: (0.22, 0.92),
        text: "3".into(),
        size: 10.0,
    });
    elems.push(WhiteboardElement::Text {
        at: (1.17, 0.92),
        text: "4".into(),
        size: 10.0,
    });
    elems.push(WhiteboardElement::Text {
        at: (0.15, 1.55),
        text: "det≠0 → invertible".into(),
        size: 8.0,
    });
}

/// Constructor puro de elementos para probabilidad (ver [`hint_for_topic`]).
fn push_probabilidad_hint(elems: &mut Vec<grafito_whiteboard::WhiteboardElement>) {
    use grafito_whiteboard::WhiteboardElement;
    // Probabilidad — histograma de 4 barras + eje + fórmula
    let heights = [0.45, 0.95, 0.65, 0.35];
    for (i, h) in heights.iter().enumerate() {
        let x = i as f64 * 0.60;
        elems.push(WhiteboardElement::Rectangle {
            min: (x, 0.0),
            max: (x + 0.45, *h),
            fill: Some((126, 214, 160)),
        });
    }
    // eje base
    elems.push(WhiteboardElement::Stroke {
        points: vec![(0.0, 0.0), (2.45, 0.0)],
        color: (55, 55, 55),
        width: 1.2,
    });
    elems.push(WhiteboardElement::Text {
        at: (0.15, 1.15),
        text: "P(A)=|A|/|Ω|".into(),
        size: 10.0,
    });
    elems.push(WhiteboardElement::Text {
        at: (0.15, -0.35),
        text: "Ω  →  evento A".into(),
        size: 9.0,
    });
}

/// Constructor puro de elementos para serie (ver [`hint_for_topic`]).
fn push_serie_hint(elems: &mut Vec<grafito_whiteboard::WhiteboardElement>) {
    use grafito_whiteboard::WhiteboardElement;
    // Serie — seno + polinomios de Taylor que se acercan
    elems.push(WhiteboardElement::Stroke {
        points: vec![
            (-1.8, 0.0),
            (-1.0, 0.85),
            (0.0, 0.0),
            (1.0, -0.85),
            (1.8, 0.0),
        ],
        color: (55, 55, 55),
        width: 1.6,
    });
    // polinomio aproximante (más grueso, punteado visual con color)
    elems.push(WhiteboardElement::Stroke {
        points: vec![(-1.2, -0.25), (-0.6, 0.45), (0.6, -0.45), (1.2, 0.25)],
        color: (66, 133, 244),
        width: 1.3,
    });
    elems.push(WhiteboardElement::Text {
        at: (-1.6, 1.15),
        text: "f(x)≈ Σ f⁽ⁿ⁾/n!·(x-a)ⁿ".into(),
        size: 8.0,
    });
    elems.push(WhiteboardElement::Text {
        at: (-1.5, -0.55),
        text: "Sₙ → L".into(),
        size: 10.0,
    });
}

/// Constructor puro de elementos para trigonometria (ver [`hint_for_topic`]).
fn push_trigonometria_hint(elems: &mut Vec<grafito_whiteboard::WhiteboardElement>) {
    use grafito_whiteboard::WhiteboardElement;
    // Trigonometría — círculo unitario + ángulo + triángulo
    elems.push(WhiteboardElement::Ellipse {
        center: (0.0, 0.0),
        rx: 1.0,
        ry: 1.0,
    });
    // radio en 45°
    elems.push(WhiteboardElement::Arrow {
        from: (0.0, 0.0),
        to: (0.71, 0.71),
    });
    // proyecciones
    elems.push(WhiteboardElement::Stroke {
        points: vec![(0.71, 0.71), (0.71, 0.0)],
        color: (66, 133, 244),
        width: 1.2,
    });
    elems.push(WhiteboardElement::Stroke {
        points: vec![(0.0, 0.0), (0.71, 0.0)],
        color: (235, 120, 80),
        width: 1.2,
    });
    elems.push(WhiteboardElement::Text {
        at: (-0.95, 1.25),
        text: "sin²+cos²=1".into(),
        size: 9.0,
    });
    elems.push(WhiteboardElement::Text {
        at: (0.35, -0.25),
        text: "cos".into(),
        size: 8.0,
    });
    elems.push(WhiteboardElement::Text {
        at: (0.78, 0.35),
        text: "sin".into(),
        size: 8.0,
    });
}

/// Constructor puro de elementos para conica (ver [`hint_for_topic`]).
fn push_conica_hint(elems: &mut Vec<grafito_whiteboard::WhiteboardElement>) {
    use grafito_whiteboard::WhiteboardElement;
    // Cónica — elipse con focos + ejes
    elems.push(WhiteboardElement::Ellipse {
        center: (0.0, 0.2),
        rx: 1.35,
        ry: 0.85,
    });
    // focos
    elems.push(WhiteboardElement::Ellipse {
        center: (-0.75, 0.2),
        rx: 0.07,
        ry: 0.07,
    });
    elems.push(WhiteboardElement::Ellipse {
        center: (0.75, 0.2),
        rx: 0.07,
        ry: 0.07,
    });
    // ejes punteados sutiles
    elems.push(WhiteboardElement::Stroke {
        points: vec![(-1.35, 0.2), (1.35, 0.2)],
        color: (55, 55, 55),
        width: 1.0,
    });
    elems.push(WhiteboardElement::Stroke {
        points: vec![(0.0, -0.65), (0.0, 1.05)],
        color: (55, 55, 55),
        width: 1.0,
    });
    elems.push(WhiteboardElement::Text {
        at: (-1.2, 1.20),
        text: "x²/a²+y²/b²=1".into(),
        size: 8.0,
    });
    elems.push(WhiteboardElement::Text {
        at: (-0.90, -0.45),
        text: "foco".into(),
        size: 7.0,
    });
}

/// Constructor puro de elementos para ecuacion (ver [`hint_for_topic`]).
fn push_ecuacion_hint(elems: &mut Vec<grafito_whiteboard::WhiteboardElement>, lower: &str) {
    use grafito_whiteboard::WhiteboardElement;
    // Ecuación — parábola con raíces + discriminante + sistema de rectas
    elems.push(WhiteboardElement::Stroke {
        points: vec![
            (-1.4, 1.2),
            (-0.7, 0.15),
            (0.0, -0.25),
            (0.7, 0.15),
            (1.4, 1.2),
        ],
        color: (55, 55, 55),
        width: 1.6,
    });
    // eje x
    elems.push(WhiteboardElement::Stroke {
        points: vec![(-1.6, 0.0), (1.6, 0.0)],
        color: (55, 55, 55),
        width: 1.0,
    });
    // raíces marcadas
    elems.push(WhiteboardElement::Ellipse {
        center: (-0.85, 0.0),
        rx: 0.07,
        ry: 0.07,
    });
    elems.push(WhiteboardElement::Ellipse {
        center: (0.85, 0.0),
        rx: 0.07,
        ry: 0.07,
    });
    // sistema alternativa: dos rectas que se cortan (si hint contiene sistema/dos rectas)
    if lower.contains("sistema") || lower.contains("dos rectas") {
        elems.push(WhiteboardElement::Stroke {
            points: vec![(-1.3, -0.9), (1.3, 0.9)],
            color: (66, 133, 244),
            width: 1.2,
        });
        elems.push(WhiteboardElement::Stroke {
            points: vec![(-1.3, 0.9), (1.3, -0.9)],
            color: (235, 120, 80),
            width: 1.2,
        });
    }
    elems.push(WhiteboardElement::Text {
        at: (-1.45, 1.40),
        text: "Δ=b²-4ac".into(),
        size: 9.0,
    });
    elems.push(WhiteboardElement::Text {
        at: (0.95, -0.35),
        text: "raíz".into(),
        size: 8.0,
    });
}

/// Constructor puro de elementos para limite (ver [`hint_for_topic`]).
fn push_limite_hint(elems: &mut Vec<grafito_whiteboard::WhiteboardElement>) {
    use grafito_whiteboard::WhiteboardElement;
    // Límite — recta con hueco + flechas de acercamiento
    elems.push(WhiteboardElement::Stroke {
        points: vec![(-1.5, -0.9), (1.5, 0.9)],
        color: (55, 55, 55),
        width: 1.5,
    });
    // hueco abierto en a=0
    elems.push(WhiteboardElement::Ellipse {
        center: (0.0, 0.0),
        rx: 0.12,
        ry: 0.12,
    });
    elems.push(WhiteboardElement::Arrow {
        from: (-0.85, -0.45),
        to: (-0.12, -0.06),
    });
    elems.push(WhiteboardElement::Arrow {
        from: (0.85, 0.45),
        to: (0.12, 0.06),
    });
    elems.push(WhiteboardElement::Text {
        at: (-0.25, 0.35),
        text: "a".into(),
        size: 10.0,
    });
    elems.push(WhiteboardElement::Text {
        at: (-1.35, 1.1),
        text: "limₓ→ₐ f(x)=L".into(),
        size: 9.0,
    });
}

/// Constructor puro de elementos para funcion (ver [`hint_for_topic`]).
fn push_funcion_hint(elems: &mut Vec<grafito_whiteboard::WhiteboardElement>) {
    use grafito_whiteboard::WhiteboardElement;
    // Función — ejes + curva + puntos (x, f(x))
    elems.push(WhiteboardElement::Stroke {
        points: vec![(-1.6, 0.0), (1.6, 0.0)],
        color: (55, 55, 55),
        width: 1.1,
    });
    elems.push(WhiteboardElement::Stroke {
        points: vec![(0.0, -1.2), (0.0, 1.4)],
        color: (55, 55, 55),
        width: 1.1,
    });
    // curva f(x)=x² leve
    elems.push(WhiteboardElement::Stroke {
        points: vec![
            (-1.2, 1.1),
            (-0.6, 0.25),
            (0.0, 0.05),
            (0.6, 0.25),
            (1.2, 1.1),
        ],
        color: (66, 133, 244),
        width: 1.6,
    });
    // puntos marcados
    elems.push(WhiteboardElement::Ellipse {
        center: (-0.6, 0.25),
        rx: 0.07,
        ry: 0.07,
    });
    elems.push(WhiteboardElement::Ellipse {
        center: (0.6, 0.25),
        rx: 0.07,
        ry: 0.07,
    });
    elems.push(WhiteboardElement::Text {
        at: (-0.55, 0.45),
        text: "(x, f(x))".into(),
        size: 8.0,
    });
    elems.push(WhiteboardElement::Text {
        at: (-1.45, -0.95),
        text: "f: x → y".into(),
        size: 9.0,
    });
}

/// Constructor puro de elementos para area (ver [`hint_for_topic`]).
fn push_area_hint(elems: &mut Vec<grafito_whiteboard::WhiteboardElement>) {
    use grafito_whiteboard::WhiteboardElement;
    // 4 rectángulos de Riemann bajo x² entre 0 y 2
    for i in 0..4 {
        let x = i as f64 * 0.5;
        let y = x * x * 0.5 + 0.2;
        elems.push(WhiteboardElement::Rectangle {
            min: (x, 0.0),
            max: (x + 0.45, y),
            fill: None,
        });
    }
    elems.push(WhiteboardElement::Stroke {
        points: vec![(0.0, 0.0), (2.0, 2.0)],
        color: (55, 55, 55),
        width: 1.5,
    });
}

/// Constructor puro de elementos para pitagoras (ver [`hint_for_topic`]).
fn push_pitagoras_hint(elems: &mut Vec<grafito_whiteboard::WhiteboardElement>) {
    use grafito_whiteboard::WhiteboardElement;
    elems.push(WhiteboardElement::Rectangle {
        min: (-2.0, -1.0),
        max: (0.0, 1.0),
        fill: None,
    });
    elems.push(WhiteboardElement::Rectangle {
        min: (0.0, -1.0),
        max: (2.0, 1.0),
        fill: None,
    });
    elems.push(WhiteboardElement::Stroke {
        points: vec![(-2.0, -1.0), (2.0, -1.0), (0.0, 1.0), (-2.0, -1.0)],
        color: (55, 55, 55),
        width: 2.0,
    });
}

/// Hidrata la pizarra del paso actual según su hint (dispatch sobre [`hint_for_topic`]).
pub(crate) fn whiteboard_elements_for_hint(
    hint: &str,
) -> Vec<grafito_whiteboard::WhiteboardElement> {
    if hint.trim().is_empty() {
        return Vec::new();
    }
    let mut elems = Vec::new();
    match hint_for_topic(hint) {
        WhiteboardHint::Vacio | WhiteboardHint::Libre => {}
        WhiteboardHint::Secante => push_secante_hint(&mut elems),
        WhiteboardHint::Fraccion => push_fraccion_hint(&mut elems),
        WhiteboardHint::Vector => push_vector_hint(&mut elems),
        WhiteboardHint::Matriz => push_matriz_hint(&mut elems),
        WhiteboardHint::Probabilidad => push_probabilidad_hint(&mut elems),
        WhiteboardHint::Serie => push_serie_hint(&mut elems),
        WhiteboardHint::Trigonometria => push_trigonometria_hint(&mut elems),
        WhiteboardHint::Conica => push_conica_hint(&mut elems),
        WhiteboardHint::Ecuacion => {
            let lower = hint.to_lowercase();
            push_ecuacion_hint(&mut elems, &lower);
        }
        WhiteboardHint::Limite => push_limite_hint(&mut elems),
        WhiteboardHint::Funcion => push_funcion_hint(&mut elems),
        WhiteboardHint::Area => push_area_hint(&mut elems),
        WhiteboardHint::Pitagoras => push_pitagoras_hint(&mut elems),
        WhiteboardHint::General => {
            use grafito_whiteboard::WhiteboardElement;
            // Fallback: texto centrado

            elems.push(WhiteboardElement::Text {
                at: (-1.5, 0.0),

                text: hint.chars().take(40).collect(),

                size: 14.0,
            });
        }
    }
    elems
}

// M3-8: las ex `pista_para_topico` / `elementos_para_topico` (dispatch por
// enum `TeachingTopic`, muerto con `#[allow(dead_code)]` + TODO) se
// borraron: el único núcleo vivo es `whiteboard_elements_for_hint` (dispatch
// por hint textual vía `hint_for_topic`, usado por `hidratar_pizarra` y
// cubierto en `tests.rs::whiteboard_elements_for_hint_covers_all_teaching_topics`).
// Si el enum tipado vuelve a hacer falta, reintroducirlo SOBRE este núcleo
// (sin duplicar los `push_*`), no al lado.
// ── Tarjeta de ejercicio inline (pura + dibujo acotado) ──

/// Tope de caracteres del enunciado para que la tarjeta no desborde.
pub const MAX_ENUNCIADO_CHARS: usize = 280;
/// Tope por opción múltiple.
pub const MAX_OPCION_CHARS: usize = 120;

/// Recorta por borde de carácter (nunca parte UTF-8). Puro, sin I/O.
pub fn acotar_texto(texto: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    if texto.chars().count() <= max {
        return texto.to_string();
    }
    let recortado: String = texto.chars().take(max.saturating_sub(1)).collect();
    format!("{recortado}…")
}

fn numero_simple(texto: &str) -> Option<f64> {
    let t = texto.trim().replace(',', ".");
    if t.is_empty() {
        return None;
    }
    if let Some((a, b)) = t.split_once('/') {
        let a: f64 = a.trim().parse().ok()?;
        let b: f64 = b.trim().parse().ok()?;
        if b.abs() > 1e-12 {
            return Some(a / b);
        }
        return None;
    }
    t.parse::<f64>().ok()
}

fn formatear_numero(valor: f64) -> String {
    if (valor - valor.round()).abs() < 1e-9 && valor.abs() < 1e9 {
        format!("{}", valor.round() as i64)
    } else {
        format!("{valor:.2}")
    }
}

/// Opciones múltiples deterministas (4, con la solución incluida). Puro, sin I/O.
#[allow(dead_code)] // Canónico Es pineado por los tests B7/E1; prod usa `_for`.
pub fn opciones_para_ejercicio(ejercicio: &Exercise) -> Vec<String> {
    opciones_para_ejercicio_for(ejercicio, grafito_ui::i18n::Locale::Es)
}

/// Opciones múltiples en el `locale` dado. Puro, sin I/O.
///
/// Solo los distractores textuales se localizan (keys
/// `teach.exercise_none_of_these` y `teach.exercise_variant` con template
/// `{n}`); la solución y los distractores numéricos/sufijados van tal cual
/// (contenido del ejercicio, no chrome). El wrapper Es preserva el canónico
/// pineado por los tests B7.
pub fn opciones_para_ejercicio_for(
    ejercicio: &Exercise,
    locale: grafito_ui::i18n::Locale,
) -> Vec<String> {
    let solucion = ejercicio.solution.trim().to_string();
    let mut candidatas: Vec<String> = Vec::with_capacity(4);
    candidatas.push(solucion.clone());
    if let Some(n) = numero_simple(&solucion) {
        for d in [1.0, -1.0, 2.0] {
            let v = if d == 2.0 { n * 2.0 } else { n + d };
            let s = formatear_numero(v);
            if !candidatas.iter().any(|c| c == &s) {
                candidatas.push(s);
            }
            if candidatas.len() >= 4 {
                break;
            }
        }
        for extra in ["1", "2", "-1", "10"] {
            if candidatas.len() >= 4 {
                break;
            }
            if !candidatas.iter().any(|c| c == extra) {
                candidatas.push(extra.to_string());
            }
        }
    } else {
        for suf in [" + 1", " − 1", " (otra forma)"] {
            if candidatas.len() >= 4 {
                break;
            }
            let s = format!("{solucion}{suf}");
            if !candidatas.iter().any(|c| c == &s) {
                candidatas.push(s);
            }
        }
        let ninguna = teach_msg("exercise_none_of_these", locale).to_string();
        if !candidatas.iter().any(|c| c == &ninguna) {
            candidatas.push(ninguna);
        }
    }
    candidatas.truncate(4);
    while candidatas.len() < 4 {
        let relleno = teach_msg("exercise_variant", locale)
            .replace("{n}", &format!("{}", candidatas.len() + 1));
        candidatas.push(relleno);
    }
    let rot = (ejercicio.seed.unwrap_or(0) % 4) as usize;
    candidatas.rotate_left(rot);
    candidatas
}

/// Índice de la solución dentro de `opciones`. Puro, sin I/O.
pub fn indice_respuesta_correcta(ejercicio: &Exercise, opciones: &[String]) -> Option<usize> {
    let sol = ejercicio.solution.trim();
    opciones.iter().position(|o| o.trim() == sol)
}

/// Estado local de la tarjeta inline (vive en la Piel, no persiste).
#[derive(Debug, Clone, Default)]
pub struct EstadoTarjetaEjercicio {
    pub respuesta: String,
    pub opcion_elegida: Option<usize>,
    pub devolucion: Option<grafito_pedagogy::Feedback>,
}

/// Encabezado del panel de ejercicio con el tema interpolado. Puro.
///
/// Usa el template `teach.exercise_header` (`Ejercicio · {tema}` en ES):
/// el `{tema}` se sustituye tal cual, sin truncar (el call-site ya acota
/// el tema con `MAX_TEMA_CHARS`). Precedente: `export_summary_text`
/// (B-A62) y `shortcut_empty_text` (B-A57).
pub fn exercise_header_text(tema: &str, locale: grafito_ui::i18n::Locale) -> String {
    teach_msg("exercise_header", locale).replace("{tema}", tema)
}

/// Texto del toast al arrancar un ejercicio, con el tema interpolado. Puro.
///
/// Usa el template `teach.exercise_building_toast`
/// (`Armo un ejercicio de {tema}, che.` en ES): el `{tema}` se sustituye
/// tal cual, sin truncar (el call-site ya acota con `MAX_TEMA_CHARS`).
/// Precedente: `exercise_header_text` (B-A74).
pub fn exercise_building_toast_text(tema: &str, locale: grafito_ui::i18n::Locale) -> String {
    teach_msg("exercise_building_toast", locale).replace("{tema}", tema)
}

/// Hint de la burbuja morph con los índices interpolados. Puro.
///
/// Usa el template `teach.exercise_step_hint`
/// (`Mirá el paso {n} de {m} — vamos de a poco, che.` en ES).
/// Precedente: `exercise_header_text` (B-A74).
pub fn exercise_step_hint_text(n: usize, m: usize, locale: grafito_ui::i18n::Locale) -> String {
    teach_msg("exercise_step_hint", locale)
        .replace("{n}", &n.to_string())
        .replace("{m}", &m.to_string())
}

/// Label del fallback nativo del overlay con template y conteo. Puro.
///
/// Usa el template `teach.overlay_anim_reserve`
/// (`Animación: {template} — {n} cuadros (reserva nativa)` en ES):
/// `{template}` es el id del template (no se traduce) y `{n}` el conteo.
/// Precedente: `exercise_step_hint_text` (B-A78).
pub fn overlay_anim_reserve_text(
    template: &str,
    n: usize,
    locale: grafito_ui::i18n::Locale,
) -> String {
    teach_msg("overlay_anim_reserve", locale)
        .replace("{template}", template)
        .replace("{n}", &n.to_string())
}

/// Paleta de la devolución (ok/error) derivada del tema. Pura, sin I/O.
///
/// Retorna `(borde, fondo)`: el borde lleva el semáforo (`success`/`danger`
/// del tema) y el fondo es `card_bg` elevado — nunca un verde/rosa
/// hardcodeado (el `#E8F5E9`/`#FDEDED` fijo fundía el texto en oscuro).
/// Título y cuerpo usan `text_primary`/`text_secondary` (AA ≥ 4.5 sobre
/// `card_bg` en ambos modos, ver test): el estado lo dan borde + texto, no
/// el tono del título (el `danger` oscuro sobre `card_bg` da 3.27:1).
fn colores_devolucion(
    correcto: bool,
    tema: &grafito_ui::theme::Theme,
) -> (egui::Color32, egui::Color32) {
    let borde = if correcto { tema.success } else { tema.danger };
    (borde, tema.card_bg)
}

/// Color del texto de la devolución, resuelto del mismo tema que el draw.
///
/// Retorna `(cuerpo, sugerencia)`: título + mensaje usan `text_primary` y
/// el paso siguiente `text_secondary`. Existe como función para que el test
/// AA mida EXACTAMENTE lo dibujado (misma resolución, sin duplicar
/// literales): si el draw volviera a `#282828`/`#5A5A5A`, el test cae.
fn colores_texto_devolucion(tema: &grafito_ui::theme::Theme) -> (egui::Color32, egui::Color32) {
    (tema.text_primary, tema.text_secondary)
}

/// `WidgetInfo` de una opción (radio) de la tarjeta. Constructor único que
/// usan el draw y los tests: misma etiqueta/rol en ambos.
fn info_opcion_tarjeta(texto: &str, elegida: bool) -> egui::WidgetInfo {
    egui::WidgetInfo::selected(
        egui::WidgetType::RadioButton,
        true,
        elegida,
        texto.to_owned(),
    )
}

/// `WidgetInfo` del botón Responder. Constructor único (draw + tests).
/// Recibe la etiqueta ya resuelta (i18n la resuelve el draw) para no sumar
/// firmas con `locale` al pin B-A70.
fn info_boton_responder(etiqueta: &str) -> egui::WidgetInfo {
    egui::WidgetInfo::labeled(egui::WidgetType::Button, true, etiqueta.to_owned())
}

#[cfg(test)]
thread_local! {
    /// Hook de test: registra cada `WidgetInfo` que la tarjeta anuncia.
    ///
    /// El registro ocurre DENTRO del closure de `widget_info` (egui solo lo
    /// ejecuta con `show_interactive_widgets` o accesskit): presencia en el
    /// buffer prueba que el draw anunció accesibilidad de verdad. En prod es
    /// no-op (sin retener nada).
    static INFOS_TARJETA: std::cell::RefCell<Vec<(egui::Id, egui::WidgetInfo)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

#[cfg(test)]
fn registrar_info_tarjeta(id: egui::Id, info: &egui::WidgetInfo) {
    INFOS_TARJETA.with(|c| c.borrow_mut().push((id, info.clone())));
}

#[cfg(not(test))]
#[inline]
fn registrar_info_tarjeta(_id: egui::Id, _info: &egui::WidgetInfo) {}

#[cfg(test)]
fn infos_tarjeta() -> Vec<(egui::Id, egui::WidgetInfo)> {
    INFOS_TARJETA.with(|c| c.borrow().clone())
}

#[cfg(test)]
fn limpiar_infos_tarjeta() {
    INFOS_TARJETA.with(|c| c.borrow_mut().clear());
}

/// Tarjeta inline: enunciado + opciones + devolución tras responder.
///
/// Layout acotado (ancho del panel, `wrap` + recorte). Sin I/O ni spawn:
/// sólo evalúa con `FeedbackEngine` (CPU en memoria). Textos en rioplatense.
pub fn draw_tarjeta_ejercicio(
    ui: &mut egui::Ui,
    ejercicio: &Exercise,
    opciones: &[String],
    estado: &mut EstadoTarjetaEjercicio,
    locale: grafito_ui::i18n::Locale,
) {
    let tema = grafito_ui::theme::current_theme(ui.ctx());
    egui::Frame::none()
        .fill(tema.panel_bg)
        .stroke(Stroke::new(1.0, tema.separator.gamma_multiply(0.10)))
        .rounding(grafito_ui::tokens::RADIUS_MD)
        .inner_margin(egui::Margin::same(grafito_ui::tokens::SPACE_MD))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.add(
                egui::Label::new(
                    egui::RichText::new(acotar_texto(&ejercicio.prompt, MAX_ENUNCIADO_CHARS))
                        .strong()
                        .size(grafito_ui::tokens::TYPE_BASE)
                        .color(tema.text_primary),
                )
                .wrap(),
            );
            ui.add_space(grafito_ui::tokens::SPACE_XS);
            for (idx, op) in opciones.iter().enumerate() {
                let texto = acotar_texto(op, MAX_OPCION_CHARS);
                let resp = ui.radio_value(&mut estado.opcion_elegida, Some(idx), texto.clone());
                resp.widget_info(|| {
                    let info = info_opcion_tarjeta(&texto, estado.opcion_elegida == Some(idx));
                    registrar_info_tarjeta(resp.id, &info);
                    info
                });
                if resp.has_focus() {
                    tema.paint_focus_ring(ui.painter(), resp.rect);
                }
                if resp.clicked() {
                    estado.respuesta = op.clone();
                    estado.devolucion = None;
                }
            }
            ui.add_space(grafito_ui::tokens::SPACE_SM);
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new(teach_msg("exercise_answer_label", locale))
                        .size(grafito_ui::tokens::TYPE_SM)
                        .color(tema.text_secondary),
                );
                ui.text_edit_singleline(&mut estado.respuesta);
                let boton = egui::Button::new(
                    egui::RichText::new(teach_msg("exercise_respond", locale))
                        .size(grafito_ui::tokens::TYPE_SM)
                        .strong()
                        .color(tema.keyboard_enter_text),
                )
                .fill(tema.accent)
                .stroke(Stroke::NONE)
                .rounding(grafito_ui::tokens::RADIUS_SM);
                let resp = ui.add(boton);
                resp.widget_info(|| {
                    let info = info_boton_responder(teach_msg("exercise_respond", locale));
                    registrar_info_tarjeta(resp.id, &info);
                    info
                });
                if resp.has_focus() {
                    tema.paint_focus_ring(ui.painter(), resp.rect);
                }
                if resp.clicked() {
                    let fb = grafito_pedagogy::FeedbackEngine.assess(ejercicio, &estado.respuesta);
                    estado.devolucion = Some(fb);
                }
            });
            if let Some(fb) = estado.devolucion.as_ref() {
                ui.add_space(grafito_ui::tokens::SPACE_SM);
                let (borde, fondo) = colores_devolucion(fb.correct, tema);
                let (texto_cuerpo, texto_sugerencia) = colores_texto_devolucion(tema);
                egui::Frame::none()
                    .fill(fondo)
                    .stroke(Stroke::new(grafito_ui::tokens::STROKE_HAIRLINE, borde))
                    .rounding(grafito_ui::tokens::RADIUS_MD)
                    .inner_margin(egui::Margin::same(grafito_ui::tokens::SPACE_SM))
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        let titulo = if fb.correct {
                            teach_msg("exercise_feedback_ok", locale)
                        } else {
                            teach_msg("exercise_feedback_retry", locale)
                        };
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(titulo)
                                    .strong()
                                    .size(grafito_ui::tokens::TYPE_SM)
                                    .color(texto_cuerpo),
                            )
                            .wrap(),
                        );
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(acotar_texto(&fb.message, MAX_ENUNCIADO_CHARS))
                                    .size(grafito_ui::tokens::TYPE_SM)
                                    .color(texto_cuerpo),
                            )
                            .wrap(),
                        );
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(acotar_texto(
                                    &fb.next_step,
                                    MAX_ENUNCIADO_CHARS,
                                ))
                                .size(grafito_ui::tokens::TYPE_XS)
                                .color(texto_sugerencia),
                            )
                            .wrap(),
                        );
                    });
            }
        });
}

// ── Panel de ejercicio con estados (B7: cablea la tarjeta) ──

/// Tope del tema pedido (mensajes y resolución de LO acotados).
pub const MAX_TEMA_CHARS: usize = 60;

/// Estado del panel de ejercicio: vacío / generando / lista / error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EstadoPanelEjercicio {
    #[default]
    Vacio,
    Generando,
    Lista,
    Error,
}

/// Panel de ejercicio: dueña del ciclo pedir→generar→responder→próximo.
///
/// Vive en `TeachingUiState` (sin tocar `app.rs`): el hilo de generación
/// escribe por `mpsc` y `sondear` lo cosecha sin bloquear. La corrección
/// (`Responder`) sigue inline en `draw_tarjeta_ejercicio` (CPU en memoria,
/// precedente aceptado); la app asienta el resultado en el perfil aparte.
#[derive(Default)]
pub struct PanelEjercicio {
    pub estado: EstadoPanelEjercicio,
    pub tema: String,
    pub ejercicio: Option<Exercise>,
    pub opciones: Vec<String>,
    pub tarjeta: EstadoTarjetaEjercicio,
    /// Próximo paso BKT (`recommend_next`) o texto honesto si no hay ramas.
    pub proximo: Option<String>,
    pub error: Option<String>,
    /// Semilla: crece en cada pedido para que «Otro ejercicio» varíe.
    pub semilla: u64,
    /// Respuesta ya asentada en el perfil (evita doble conteo BKT).
    pub registrada: Option<String>,
    receptor: Option<std::sync::mpsc::Receiver<Result<Exercise, String>>>,
}

/// Quita tildes para el reintento fuzzy (`función`→`funcion`). Puro.
fn sin_tildes(texto: &str) -> String {
    texto
        .chars()
        .map(|c| match c {
            'á' | 'Á' => 'a',
            'é' | 'É' => 'e',
            'í' | 'Í' => 'i',
            'ó' | 'Ó' => 'o',
            'ú' | 'Ú' | 'ü' | 'Ü' => 'u',
            'ñ' | 'Ñ' => 'n',
            otro => otro,
        })
        .collect()
}

/// Resuelve texto libre → `LearningObjective` (exacto por id, si no fuzzy
/// por concepto con reintento sin tildes). Puro, sin I/O. Mismo criterio
/// que `generate_exercise` del dispatcher, sin round-trip JSON.
#[allow(dead_code)] // Canónico Es pineado por el test B7; prod usa `_for`.
pub fn resolver_lo(tema: &str) -> Result<LearningObjective, String> {
    resolver_lo_for(tema, grafito_ui::i18n::Locale::Es)
}

/// Resuelve en el `locale` dado. Pura, sin I/O.
///
/// Usa los templates `teach.exercise_err_empty_topic` /
/// `teach.exercise_err_not_found` (`«{recortado}»` se sustituye tal cual).
/// El wrapper Es preserva el canónico pineado por el test B7 (patrón B-A64).
pub fn resolver_lo_for(
    tema: &str,
    locale: grafito_ui::i18n::Locale,
) -> Result<LearningObjective, String> {
    let recortado: String = tema.trim().chars().take(MAX_TEMA_CHARS).collect();
    if recortado.is_empty() {
        return Err(teach_msg("exercise_err_empty_topic", locale).to_string());
    }
    if let Some(lo) = Curriculum::get(recortado.trim()) {
        return Ok(lo);
    }
    let mut candidatos = Curriculum::find_for_concept(&recortado);
    if candidatos.is_empty() {
        candidatos = Curriculum::find_for_concept(&sin_tildes(&recortado));
    }
    candidatos.into_iter().next().ok_or_else(|| {
        teach_msg("exercise_err_not_found", locale).replace("{recortado}", &recortado)
    })
}

/// Arma un ejercicio validado para el tema (núcleo sync del job). Puro.
#[allow(dead_code)] // Canónico Es pineado por el test B7; prod usa `_for`.
pub fn armar_ejercicio(
    tema: &str,
    nivel: PedagogicalLevel,
    semilla: u64,
) -> Result<Exercise, String> {
    armar_ejercicio_for(tema, nivel, semilla, grafito_ui::i18n::Locale::Es)
}

/// Arma en el `locale` dado. Pura, sin I/O.
///
/// Usa el template `teach.exercise_err_invalid` (`{motivo}` se sustituye tal
/// cual); el error de `resolver_lo_for` ya viene localizado. El wrapper Es
/// preserva el canónico pineado por el test B7 (patrón B-A64/B-A75).
pub fn armar_ejercicio_for(
    tema: &str,
    nivel: PedagogicalLevel,
    semilla: u64,
    locale: grafito_ui::i18n::Locale,
) -> Result<Exercise, String> {
    let lo = resolver_lo_for(tema, locale)?;
    let ejercicio = ExerciseGenerator.generate_with_seed(&lo, nivel, semilla);
    ejercicio.validate().map_err(|motivo| {
        teach_msg("exercise_err_invalid", locale).replace("{motivo}", &motivo.to_string())
    })?;
    Ok(ejercicio)
}

impl PanelEjercicio {
    /// Pide un ejercicio: pasa a `Generando` y lo arma en background.
    /// La UI sólo cosecha con `sondear`; cero spawn en el dibujado.
    #[allow(dead_code)] // Canónico Es pineado por el test B7; prod usa `_con`.
    pub fn pedir(&mut self, tema: &str, nivel: PedagogicalLevel) {
        self.pedir_con(tema, nivel, grafito_ui::i18n::Locale::Es)
    }

    /// Pide en el `locale` dado (errores del job localizados). Sin cambio de
    /// comportamiento fuera del idioma: el wrapper Es preserva el canónico
    /// de los tests B7 (patrón B-A64/B-A75). El `locale` (`Copy`) cruza al
    /// hilo del job para que `armar_ejercicio_for` ya devuelva el mensaje
    /// localizado.
    pub fn pedir_con(
        &mut self,
        tema: &str,
        nivel: PedagogicalLevel,
        locale: grafito_ui::i18n::Locale,
    ) {
        let recortado: String = tema.trim().chars().take(MAX_TEMA_CHARS).collect();
        if recortado.is_empty() {
            self.estado = EstadoPanelEjercicio::Error;
            self.error = Some(teach_msg("exercise_err_empty_topic", locale).to_string());
            self.receptor = None;
            return;
        }
        let semilla = self.semilla.wrapping_add(1);
        let para_hilo = recortado.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        match std::thread::Builder::new()
            .name("grafito-ejercicio".to_string())
            .spawn(move || {
                let _ = tx.send(armar_ejercicio_for(&para_hilo, nivel, semilla, locale));
            }) {
            Ok(_) => {
                self.semilla = semilla;
                self.tema = recortado;
                self.receptor = Some(rx);
                self.estado = EstadoPanelEjercicio::Generando;
                self.error = None;
                self.proximo = None;
                self.registrada = None;
            }
            Err(_) => {
                self.receptor = None;
                self.estado = EstadoPanelEjercicio::Error;
                self.error = Some(teach_msg("exercise_err_spawn", locale).to_string());
            }
        }
    }

    /// Cosecha el job sin bloquear (agota la cola: vale el último pedido).
    /// Retorna `true` si hubo cambio (el dibujado repinta).
    pub fn sondear(&mut self) -> bool {
        self.sondear_con(grafito_ui::i18n::Locale::Es)
    }

    /// Cosecha en el `locale` dado (distractores localizados). Sin cambio de
    /// comportamiento fuera del idioma: el wrapper Es preserva el canónico de
    /// los tests B7 (patrón B-A64).
    pub fn sondear_con(&mut self, locale: grafito_ui::i18n::Locale) -> bool {
        if self.receptor.is_none() {
            return false;
        }
        let mut cambio = false;
        loop {
            let recibido = match self.receptor.as_ref() {
                None => break,
                Some(rx) => rx.try_recv(),
            };
            match recibido {
                Ok(Ok(ejercicio)) => {
                    cambio = true;
                    self.opciones = opciones_para_ejercicio_for(&ejercicio, locale);
                    // Invariante de la tarjeta: la solución está entre las opciones.
                    debug_assert!(indice_respuesta_correcta(&ejercicio, &self.opciones).is_some());
                    self.ejercicio = Some(ejercicio);
                    self.tarjeta = EstadoTarjetaEjercicio::default();
                    self.estado = EstadoPanelEjercicio::Lista;
                    self.error = None;
                }
                Ok(Err(mensaje)) => {
                    cambio = true;
                    self.estado = EstadoPanelEjercicio::Error;
                    self.error = Some(mensaje);
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    if self.estado == EstadoPanelEjercicio::Generando {
                        self.estado = EstadoPanelEjercicio::Error;
                        self.error =
                            Some(teach_msg("exercise_err_disconnected", locale).to_string());
                        cambio = true;
                    }
                    self.receptor = None;
                    break;
                }
            }
        }
        if cambio && !matches!(self.estado, EstadoPanelEjercicio::Generando) {
            self.receptor = None;
        }
        cambio
    }

    /// Cierra el panel (suelta el job en curso: su `send` falla en silencio).
    pub fn cerrar(&mut self) {
        *self = Self::default();
    }
}

/// Intención del panel (la app ejecuta; el dibujado no genera ni persiste).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccionEjercicio {
    Otro,
    Cerrar,
}

/// Dibuja el panel según estado. `Vacio` no dibuja nada (los entry points
/// son el botón «Andamiar» en cada respuesta y el CTA del estado vacío).
/// Todos los botones hacen algo: sin botones mudos.
pub fn draw_panel_ejercicio(
    ui: &mut egui::Ui,
    panel: &mut PanelEjercicio,
    locale: grafito_ui::i18n::Locale,
) -> Option<AccionEjercicio> {
    if panel.sondear_con(locale) {
        // Ola 2: inmediato a propósito — cosecha de un job instantáneo de CPU
        // (un solo frame extra); sin handle al `RepaintBudget` en esta fn
        // libre y con latencia cero requerida. Lo periódico va por presupuesto.
        ui.ctx().request_repaint();
    }
    let tema = grafito_ui::theme::current_theme(ui.ctx());
    match panel.estado {
        EstadoPanelEjercicio::Vacio => None,
        EstadoPanelEjercicio::Generando => {
            // El job es instantáneo (CPU pura): este frame cosecha al siguiente.
            // Ola 2: inmediato (transitorio de 1 frame, ver nota en `sondear`).
            ui.ctx().request_repaint();
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(
                    egui::RichText::new(teach_msg("exercise_building", locale))
                        .size(grafito_ui::tokens::TYPE_SM)
                        .color(tema.text_secondary),
                );
            });
            None
        }
        EstadoPanelEjercicio::Error => {
            let mensaje = panel
                .error
                .clone()
                .unwrap_or_else(|| teach_msg("exercise_error_generic", locale).to_string());
            egui::Frame::none()
                .fill(tema.panel_bg)
                .stroke(Stroke::new(1.0, tema.separator.gamma_multiply(0.10)))
                .rounding(grafito_ui::tokens::RADIUS_MD)
                .inner_margin(egui::Margin::same(grafito_ui::tokens::SPACE_MD))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.label(
                        egui::RichText::new(acotar_texto(&mensaje, MAX_ENUNCIADO_CHARS))
                            .size(grafito_ui::tokens::TYPE_SM)
                            .color(tema.text_primary),
                    );
                });
            ui.horizontal(|ui| {
                let mut accion = None;
                if ui.button(teach_msg("exercise_retry", locale)).clicked() {
                    accion = Some(AccionEjercicio::Otro);
                }
                if ui.button(teach_msg("overlay_close", locale)).clicked() {
                    accion = Some(AccionEjercicio::Cerrar);
                }
                accion
            })
            .inner
        }
        EstadoPanelEjercicio::Lista => {
            let Some(ejercicio) = panel.ejercicio.as_ref() else {
                panel.estado = EstadoPanelEjercicio::Error;
                panel.error = Some(teach_msg("exercise_error_lost", locale).to_string());
                return None;
            };
            ui.label(
                egui::RichText::new(exercise_header_text(&panel.tema, locale))
                    .size(grafito_ui::tokens::TYPE_XS)
                    .color(tema.text_secondary),
            );
            draw_tarjeta_ejercicio(ui, ejercicio, &panel.opciones, &mut panel.tarjeta, locale);
            if let Some(proximo) = panel.proximo.as_ref() {
                ui.add_space(grafito_ui::tokens::SPACE_XS);
                ui.label(
                    egui::RichText::new(acotar_texto(proximo, MAX_ENUNCIADO_CHARS))
                        .size(grafito_ui::tokens::TYPE_XS)
                        .color(tema.text_secondary),
                );
            }
            ui.add_space(grafito_ui::tokens::SPACE_XS);
            ui.horizontal(|ui| {
                let mut accion = None;
                if ui
                    .button(teach_msg("exercise_other", locale))
                    .on_hover_text(teach_msg("exercise_other_hint", locale))
                    .clicked()
                {
                    accion = Some(AccionEjercicio::Otro);
                }
                if ui.button(teach_msg("overlay_close", locale)).clicked() {
                    accion = Some(AccionEjercicio::Cerrar);
                }
                accion
            })
            .inner
        }
    }
}

// ── Burbuja morph avatar→burbuja (pura + clamp a viewport) ──

/// Ease-out cúbico del morph (ANIM_MICRO 180ms). Puro, sin I/O.
pub fn progreso_morph_suave(t: f32) -> f32 {
    let x = t.clamp(0.0, 1.0);
    1.0 - (1.0 - x) * (1.0 - x) * (1.0 - x)
}

/// Interpola origen→destino y clampeea al `limite` (normalmente `available_rect`).
///
/// Garantiza que la burbuja nunca se pase del viewport: el resultado siempre
/// está contenido en `limite`. Puro, sin I/O.
pub fn rect_burbuja_morph(
    origen: egui::Rect,
    destino: egui::Rect,
    progreso: f32,
    limite: egui::Rect,
) -> egui::Rect {
    let t = progreso.clamp(0.0, 1.0);
    let min = egui::pos2(
        origen.min.x + (destino.min.x - origen.min.x) * t,
        origen.min.y + (destino.min.y - origen.min.y) * t,
    );
    let max = egui::pos2(
        origen.max.x + (destino.max.x - origen.max.x) * t,
        origen.max.y + (destino.max.y - origen.max.y) * t,
    );
    let mut r = egui::Rect::from_min_max(min, max);
    // Clamp duro al límite: intersección + resguardo si no hay solape.
    if r.width() > limite.width() {
        r.set_width(limite.width());
    }
    if r.height() > limite.height() {
        r.set_height(limite.height());
    }
    r = r.intersect(limite);
    if r.is_negative() {
        r = egui::Rect::from_min_size(limite.min, egui::Vec2::ZERO);
    }
    r
}

// ── Frente E1: overlay de enseñanza sin marcas crudas ni duplicación ──

/// Etiqueta de progreso real (1-based): "Paso N de M". Pura, sin I/O.
#[allow(dead_code)] // Canónico Es pineado por el test E1; prod usa `_for`.
pub fn etiqueta_paso(indice_actual: usize, total_pasos: usize) -> String {
    etiqueta_paso_for(indice_actual, total_pasos, grafito_ui::i18n::Locale::Es)
}

/// Etiqueta de progreso en el `locale` dado. Pura, sin I/O.
///
/// Usa el template `teach.progress_step` (`Paso {n} de {m}` en ES) con `{n}`
/// 1-based y `{m}` = total (mínimo 1). El wrapper Es preserva el canónico
/// pineado por el test E1 (patrón B-A64).
pub fn etiqueta_paso_for(
    indice_actual: usize,
    total_pasos: usize,
    locale: grafito_ui::i18n::Locale,
) -> String {
    teach_msg("progress_step", locale)
        .replace("{n}", &format!("{}", indice_actual.saturating_add(1)))
        .replace("{m}", &format!("{}", total_pasos.max(1)))
}

/// ¿Este `#` es marca markdown y no contenido? Marca = al inicio o tras
/// espacio, y seguido de espacio, fin u otro `#`. Así "## Enfoque" se pela
/// pero "C#" o "F# rigid" quedan intactos. Puro, por chars (UTF-8 seguro).
fn es_marca_gato(anterior: Option<char>, siguiente: Option<char>) -> bool {
    let abre = anterior.is_none_or(|c| c == ' ' || c == '#');
    let cierra = siguiente.is_none_or(|c| c == ' ' || c == '#');
    abre && cierra
}

/// Quita marcas markdown en línea (`**`, `__`, `` ` ``, `##` filtrado).
/// No parsea bloques: eso lo hace el parser del transcript
/// (`AssistantBlocksCache`); acá sólo se limpia texto ya extraído
/// (títulos, etiquetas). Puro, sin I/O.
pub fn texto_plano_inline(texto: &str) -> String {
    let sin_negrita = texto.replace("**", "").replace("__", "");
    let sin_codigo = sin_negrita.replace('`', "");
    let chars: Vec<char> = sin_codigo.chars().collect();
    let mut limpio = String::with_capacity(sin_codigo.len());
    for (i, c) in chars.iter().enumerate() {
        if *c == '#' {
            let anterior = if i == 0 {
                None
            } else {
                chars.get(i - 1).copied()
            };
            let siguiente = chars.get(i + 1).copied();
            if es_marca_gato(anterior, siguiente) {
                continue;
            }
        }
        limpio.push(*c);
    }
    // Colapsa espacios dobles que dejan las marcas (una pasada).
    let mut final_out = String::with_capacity(limpio.len());
    let mut previo_espacio = false;
    for c in limpio.chars() {
        if c == ' ' {
            if previo_espacio {
                continue;
            }
            previo_espacio = true;
        } else {
            previo_espacio = false;
        }
        final_out.push(c);
    }
    final_out.trim().to_string()
}

/// Título de una línea sin marcas crudas ("## Enfoque" → "Enfoque").
/// Si no queda nada, honesto: "Paso". Puro, sin I/O.
#[allow(dead_code)] // Canónico Es pineado por el test E1; prod usa `_for`.
pub fn titulo_limpio(titulo: &str) -> String {
    titulo_limpio_for(titulo, grafito_ui::i18n::Locale::Es)
}

/// Título limpio en el `locale` dado. Puro, sin I/O.
///
/// El fallback de vacío sale por `teach.step_fallback` (`Paso` en ES); el
/// wrapper Es preserva el canónico pineado por el test E1 (patrón B-A64).
pub fn titulo_limpio_for(titulo: &str, locale: grafito_ui::i18n::Locale) -> String {
    let plano = texto_plano_inline(titulo);
    if plano.is_empty() {
        teach_msg("step_fallback", locale).to_string()
    } else {
        plano
    }
}

/// Estado de la sección pizarra: colapsada si vacía, lienzo si hay dibujo
/// útil, placeholder honesto si el hint sólo da texto de fallback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstadoPizarra {
    /// Sin hint y sin trazos: no se dibuja nada (ni recuadro negro vacío).
    Oculta,
    /// Hay vectores reales (trazos, flechas, figuras): lienzo interactivo.
    Lienzo,
    /// El hint sólo produce texto de fallback (tiny centrado ilegible):
    /// placeholder honesto en vez del recuadro negro.
    Placeholder,
}

/// ¿El hint produce dibujo útil (algo más que el texto de fallback)?
/// Puro, sin I/O.
pub fn pizarra_tiene_dibujo(hint: &str) -> bool {
    whiteboard_elements_for_hint(hint)
        .iter()
        .any(|e| !matches!(e, WhiteboardElement::Text { .. }))
}

/// Decide la sección pizarra (una sola fuente, cero duplicación).
/// Puro, sin I/O.
pub fn estado_pizarra(hint: &str, doc_vacia: bool) -> EstadoPizarra {
    if hint.trim().is_empty() && doc_vacia {
        return EstadoPizarra::Oculta;
    }
    if pizarra_tiene_dibujo(hint) || !doc_vacia {
        return EstadoPizarra::Lienzo;
    }
    if hint.trim().is_empty() {
        EstadoPizarra::Oculta
    } else {
        EstadoPizarra::Placeholder
    }
}

/// Dibuja texto con la gramática del transcript (mismo parser, sin duplicar:
/// `AssistantBlocksCache` + `humanize_prose_text`). Código y tabla caen a
/// marco monoespaciado honesto; la matemática (`$..$`) se dibuja con el
/// renderer del asistente (`grafito_ui::assistant::draw_math_with_locale`,
/// con fallback a texto si el parse falla; el aviso sale de la key
/// `assistant.math.tex_simplified` en el `locale` dado). Cero I/O y cero
/// spawn: sólo CPU en memoria.
pub fn draw_bloques_markdown(
    ui: &mut egui::Ui,
    texto: &str,
    cache: &mut AssistantBlocksCache,
    locale: grafito_ui::i18n::Locale,
) {
    let tema = grafito_ui::theme::current_theme(ui.ctx());
    let bloques = cache.blocks(texto);
    for bloque in bloques.iter() {
        match bloque {
            AssistantMessageBlock::Heading { level, text } => {
                let size = match level {
                    1 => grafito_ui::tokens::TYPE_LG,
                    2 => grafito_ui::tokens::TYPE_MD,
                    _ => grafito_ui::tokens::TYPE_SM,
                };
                ui.label(
                    egui::RichText::new(humanize_prose_text(&texto_plano_inline(text)))
                        .strong()
                        .size(size)
                        .color(tema.text_primary),
                );
            }
            AssistantMessageBlock::Bullet(text) => {
                ui.horizontal_top(|ui| {
                    ui.label(egui::RichText::new("·").color(tema.accent));
                    ui.label(
                        egui::RichText::new(humanize_prose_text(&texto_plano_inline(text)))
                            .size(grafito_ui::tokens::TYPE_BASE)
                            .color(tema.text_primary),
                    );
                });
            }
            AssistantMessageBlock::Ordered { number, text } => {
                ui.horizontal_top(|ui| {
                    ui.label(
                        egui::RichText::new(format!("{number}."))
                            .strong()
                            .size(grafito_ui::tokens::TYPE_SM)
                            .color(tema.accent),
                    );
                    ui.label(
                        egui::RichText::new(humanize_prose_text(&texto_plano_inline(text)))
                            .size(grafito_ui::tokens::TYPE_BASE)
                            .color(tema.text_primary),
                    );
                });
            }
            AssistantMessageBlock::Quote(text) => {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = grafito_ui::tokens::SPACE_SM;
                    ui.add_space(grafito_ui::tokens::SPACE_XS);
                    ui.label(
                        egui::RichText::new(humanize_prose_text(&texto_plano_inline(text)))
                            .italics()
                            .size(grafito_ui::tokens::TYPE_BASE)
                            .color(tema.text_secondary),
                    );
                });
            }
            AssistantMessageBlock::Paragraph(text) => {
                ui.label(
                    egui::RichText::new(humanize_prose_text(&texto_plano_inline(text)))
                        .size(grafito_ui::tokens::TYPE_BASE)
                        .color(tema.text_primary),
                );
            }
            AssistantMessageBlock::DisplayMath(math) => {
                draw_math_with_locale(ui, math, locale);
            }
            AssistantMessageBlock::Code { language: _, text } => {
                marco_monoespaciado(ui, teach_msg("code_block_title", locale), text);
            }
            AssistantMessageBlock::Table(rows) => {
                marco_monoespaciado(
                    ui,
                    teach_msg("table_block_title", locale),
                    &tabla_como_texto(rows),
                );
            }
        }
    }
}

/// Marco monoespaciado honesto para bloques no-prosa. Puro dibujado, sin I/O.
fn marco_monoespaciado(ui: &mut egui::Ui, titulo: &str, contenido: &str) {
    let tema = grafito_ui::theme::current_theme(ui.ctx());
    egui::Frame::none()
        .fill(tema.input_bg)
        .stroke(Stroke::new(1.0, tema.separator.gamma_multiply(0.10)))
        .rounding(grafito_ui::tokens::RADIUS_MD)
        .inner_margin(egui::Margin::same(grafito_ui::tokens::SPACE_SM))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(
                egui::RichText::new(titulo)
                    .strong()
                    .size(grafito_ui::tokens::TYPE_XS)
                    .color(tema.accent),
            );
            ui.label(
                egui::RichText::new(contenido)
                    .monospace()
                    .size(grafito_ui::tokens::TYPE_SM)
                    .color(tema.text_primary),
            );
        });
}

/// Tabla del transcript como texto plano (fallback honesto). Puro, sin I/O.
fn tabla_como_texto(filas: &[Vec<String>]) -> String {
    filas
        .iter()
        .map(|fila| {
            fila.iter()
                .map(|celda| texto_plano_inline(celda))
                .collect::<Vec<_>>()
                .join(" · ")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

impl TeachingUiState {
    /// Huella O(frames) de `anim_frames` para el cache de texturas.
    ///
    /// Frente hash O(píxeles): antes se hasheaba cada píxel de hasta 6
    /// frames en CADA tick (`ensure_textures` corre por frame dibujado).
    /// Ahora solo `len` + `size` + 4 esquinas + centro por frame: el reemplazo
    /// (nuevo set, distinto largo o tamaño) siempre cambia la huella; un cambio
    /// solo-interior fuera del centro se re-subiría tarde como mucho un set
    /// (los frames nuevos llegan con `clear_*`, que ya invalida el cache).
    /// El centro se muestrea porque el contenido vive ahí (una mutación del
    /// píxel central cambia la huella); el resto del interior sigue fuera por
    /// la cota O(1). Sin `unwrap`: índices con `get`, nunca aritmética que
    /// desborde.
    fn anim_frames_hash(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let Some(frames) = &self.anim_frames else {
            return 0;
        };
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        frames.len().hash(&mut hasher);
        for frame in frames.iter() {
            frame.size.hash(&mut hasher);
            let w = frame.size.first().copied().unwrap_or(0);
            let h = frame.size.get(1).copied().unwrap_or(0);
            if w == 0 || h == 0 {
                continue;
            }
            let ultima_fila = h.saturating_sub(1);
            let ultima_col = w.saturating_sub(1);
            // 4 esquinas + centro (división entera; en marcos chicos puede
            // coincidir con una esquina: hashear dos veces es determinista).
            let centro = (w / 2, h / 2);
            for (x, y) in [
                (0, 0),
                (ultima_col, 0),
                (0, ultima_fila),
                (ultima_col, ultima_fila),
                centro,
            ] {
                let indice = y.saturating_mul(w).saturating_add(x);
                if let Some(pixel) = frame.pixels.get(indice) {
                    pixel.r().hash(&mut hasher);
                    pixel.g().hash(&mut hasher);
                    pixel.b().hash(&mut hasher);
                    pixel.a().hash(&mut hasher);
                }
            }
        }
        hasher.finish()
    }

    fn ensure_textures(&mut self, ctx: &egui::Context) {
        // NOTA: `ctx.forget_image(uri)` NO libera texturas de `load_texture`
        // (solo cachés de loaders por URI); la destrucción real es el drop
        // del último `TextureHandle`. Por eso acá nunca se dropea en el
        // mismo frame: el set viejo se retira con gracia (ver
        // `reap_retired_anim_textures`, un tick por frame dibujado).
        let Some(frames) = &self.anim_frames else {
            if !self.anim_textures.is_empty() {
                let old = std::mem::take(&mut self.anim_textures);
                self.retired_anim_textures.retire_all(old);
                self.cached_hash = 0;
                self.cached_len = 0;
            }
            return;
        };
        if frames.is_empty() {
            if !self.anim_textures.is_empty() {
                let old = std::mem::take(&mut self.anim_textures);
                self.retired_anim_textures.retire_all(old);
                self.cached_hash = 0;
                self.cached_len = 0;
            }
            return;
        }
        let hash = self.anim_frames_hash();
        let len = frames.len();
        if self.anim_textures.len() == len && self.cached_hash == hash && self.cached_len == len {
            return;
        }
        let old = std::mem::take(&mut self.anim_textures);
        self.retired_anim_textures.retire_all(old);
        // P1b: nombre con hash (mismo patrón retención+hash que
        // `render_2d::FillTextureCacheStore`): el set viejo ya quedó retirado
        // con gracia arriba, y el nombre versionado distingue dos sets del
        // mismo largo en los logs.
        self.anim_textures = frames
            .iter()
            .enumerate()
            .map(|(idx, frame)| {
                ctx.load_texture(
                    format!("teaching_anim_{idx}_{hash:016x}"),
                    frame.clone(),
                    egui::TextureOptions::LINEAR,
                )
            })
            .collect();
        self.cached_hash = hash;
        self.cached_len = len;
    }

    /// Avanza un frame de gracia y libera los sets expirados. Un tick = un
    /// frame dibujado; se llama una vez por frame desde
    /// `draw_teaching_overlay` (también sin sesión, para no retener de más).
    fn reap_retired_anim_textures(&mut self) {
        let _ = self.retired_anim_textures.tick();
    }

    pub fn clear(&mut self) {
        let old = std::mem::take(&mut self.anim_textures);
        self.retired_anim_textures.retire_all(old);
        // M3-7: avanzar un tick acá también (el `clear` suele correr sin
        // draw posterior): la gracia no queda congelada si nadie dibuja.
        let _ = self.retired_anim_textures.tick();
        self.cached_hash = 0;
        self.cached_len = 0;
        self.anim_frames = None;
    }

    pub fn clear_with_ctx(&mut self, _ctx: &egui::Context) {
        // `_ctx`: `forget_image` es no-op para texturas managed; la
        // liberación real es el drop diferido vía la cola de retiro.
        let old = std::mem::take(&mut self.anim_textures);
        self.retired_anim_textures.retire_all(old);
        // M3-7: idem `clear` (tick para no congelar la gracia).
        let _ = self.retired_anim_textures.tick();
        self.cached_hash = 0;
        self.cached_len = 0;
        self.anim_frames = None;
    }

    fn clear_anim_textures_only(&mut self, _ctx: Option<&egui::Context>) {
        let old = std::mem::take(&mut self.anim_textures);
        self.retired_anim_textures.retire_all(old);
        // M3-7: idem `clear` (tick para no congelar la gracia).
        let _ = self.retired_anim_textures.tick();
        self.cached_hash = 0;
        self.cached_len = 0;
    }

    /// Hidrata la pizarra con los vectores del hint (fuente única: si el
    /// hint no da dibujo útil, la sección lo resuelve con `estado_pizarra`).
    fn hidratar_pizarra(&mut self, hint: &str) {
        let mut doc = WhiteboardDoc::default();
        for elem in whiteboard_elements_for_hint(hint) {
            doc.add(elem);
        }
        self.whiteboard.doc = doc;
    }

    pub fn start(&mut self, topic: &str) {
        let session = TeachingSession::for_topic(topic);
        // Inicializar pizarra con elementos vectoriales reales según hint
        if let Some(step) = session.current() {
            self.hidratar_pizarra(&step.whiteboard_hint.clone());
            // Iniciar orquestación manim para el primer paso — cancela cualquier job previo
            if let Some(tmpl) = &step.manim_template {
                self.orchestrator.cancel();
                let _ = self.orchestrator.start(topic, tmpl.clone());
            }
        }
        self.session = Some(session);
        self.opened_at = Some(Instant::now());
        // Binding temporal: los cues arrancan acá; el primer tick hidrata
        // por cue (paso 1, cue 0) igual que el hidratar directo de arriba.
        self.session_started_at = self.opened_at;
        self.last_revealed = self
            .session
            .as_ref()
            .map(|s| s.revealed_count(0))
            .unwrap_or(0);
        self.navigated = false;
        self.anim_frames = None;
        // La pausa no sobrevive al cambio de sesión: arranca activo.
        self.paused = false;
        self.frozen_ms = None;
        self.clear_anim_textures_only(None);
    }
    pub fn advance(&mut self) -> bool {
        if let Some(session) = &mut self.session {
            let ok = session.advance();
            if let Some(step) = session.current() {
                let hint = step.whiteboard_hint.clone();
                let tmpl = step.manim_template.clone();
                let titulo = step.title.clone();
                // Hidratar pizarra del nuevo paso (drop del borrow antes de &mut).
                self.hidratar_pizarra(&hint);
                if let Some(tmpl) = &tmpl {
                    // Avanzar implica nuevo concepto → cancelar previo y relanzar
                    self.orchestrator.cancel();
                    let _ = self.orchestrator.start(&titulo, tmpl.clone());
                    self.anim_frames = None;
                    self.clear_anim_textures_only(None);
                }
            }
            // Reinicia morph para el nuevo paso (burbuja entra de nuevo);
            // el step nuevo arranca activo (la pausa del anterior no aplica).
            self.opened_at = Some(Instant::now());
            self.paused = false;
            self.frozen_ms = None;
            // Navegación explícita: los cues ya no pisan la pizarra.
            self.navigated = true;
            ok
        } else {
            false
        }
    }
    /// Vuelve al paso anterior (clampa en cero, nunca baja). Rehidrata la
    /// pizarra del paso y reinicia el morph, igual que `advance`.
    pub fn retroceder(&mut self) -> bool {
        if self.session.is_none() {
            return false;
        }
        if let Some(session) = &mut self.session {
            if session.current == 0 {
                return false;
            }
            session.current -= 1;
            if let Some(paso) = session.current_mut() {
                paso.completed = false;
            }
        }
        if let Some(nuevo_hint) = self
            .session
            .as_ref()
            .and_then(|s| s.current().map(|p| p.whiteboard_hint.clone()))
        {
            self.hidratar_pizarra(&nuevo_hint);
        }
        self.opened_at = Some(Instant::now());
        self.navigated = true;
        self.paused = false;
        self.frozen_ms = None;
        true
    }
    pub fn close(&mut self) {
        self.session = None;
        self.session_started_at = None;
        self.last_revealed = 0;
        self.navigated = false;
        self.orchestrator.cancel();
        self.opened_at = None;
        self.anim_frames = None;
        self.paused = false;
        self.frozen_ms = None;
        self.clear_anim_textures_only(None);
    }

    pub fn close_with_ctx(&mut self, ctx: &egui::Context) {
        self.session = None;
        self.session_started_at = None;
        self.last_revealed = 0;
        self.navigated = false;
        self.orchestrator.cancel();
        self.opened_at = None;
        self.anim_frames = None;
        self.paused = false;
        self.frozen_ms = None;
        self.clear_anim_textures_only(Some(ctx));
    }
    pub fn tick(&mut self, now: Instant) {
        // Binding temporal por cue: hidrata la pizarra con los hints
        // revelados hasta el elapsed (en vez de todo el hint de golpe).
        // No pisa navegación manual (`advance`/`retroceder`).
        self.poll_cues(now);
        if let Some(state) = self.orchestrator.tick(now) {
            // Al completar, generar fallback nativo via anim_native si no hay artefacto real
            if matches!(state, OrchestratorState::Completed { .. }) && self.anim_frames.is_none() {
                let concept = self.orchestrator.concept.clone();
                let template = self.orchestrator.template.clone();
                let frames =
                    crate::anim_native::render_anim_for_concept(&template, &concept, 320, 180);
                self.anim_frames = Some(frames);
                self.clear_anim_textures_only(None);
            }
            let _ = state;
        }
    }
    /// Hidrata por cue: unión de los hints revelados hasta el elapsed de la
    /// sesión. Solo re-hidrata cuando cambia la cantidad revelada; nunca si
    /// el usuario navegó manualmente. Puro estado local, sin I/O.
    fn poll_cues(&mut self, now: Instant) {
        if self.navigated {
            return;
        }
        let (Some(session), Some(started)) = (self.session.as_ref(), self.session_started_at)
        else {
            return;
        };
        let elapsed_ms = now
            .duration_since(started)
            .as_millis()
            .min(u64::MAX as u128) as u64;
        let revealed = session.revealed_count(elapsed_ms);
        if revealed == self.last_revealed {
            return;
        }
        self.last_revealed = revealed;
        // Reconstruye acumulando por hint (cada hint se clasifica por
        // separado: unir los strings rompería `hint_for_topic`).
        let mut doc = WhiteboardDoc::default();
        for step in session.revealed_steps(elapsed_ms) {
            for elem in whiteboard_elements_for_hint(&step.whiteboard_hint) {
                doc.add(elem);
            }
        }
        self.whiteboard.doc = doc;
        // R2: el último paso revelado puede ser de guion (`guion:k/n`:
        // 1 acto = 1 paso con `frame_range` particionado del total). Se
        // extraen datos propios ANTES de mutar (el borrow de `session`
        // muere acá): sin eso el orchestrator no se puede tocar.
        let guion_revelado: Option<(String, String, bool)> =
            session.revealed_steps(elapsed_ms).last().and_then(|paso| {
                let tmpl = paso.manim_template.clone()?;
                if !tmpl.trim().starts_with("guion:") {
                    return None;
                }
                let valido = paso_guion_valido(Some(&tmpl), session.steps.len());
                Some((tmpl, paso.title.clone(), valido))
            });
        let Some((tmpl, titulo, valido)) = guion_revelado else {
            return;
        };
        if !valido {
            // Fallback honesto a paso expositivo: la pizarra ya hidrató
            // arriba; sin animación (cancela el orchestrator y limpia
            // frames, jamás basura en pantalla).
            self.orchestrator.cancel();
            self.anim_frames = None;
            self.clear_anim_textures_only(None);
        } else if self.orchestrator.template != tmpl {
            // Reparto por acto: cada acto relanza el orchestrator con su
            // template; al completar, `tick` renderiza el loop nativo y el
            // draw lo ventana por `frame_range` vía `cue_frame_index` (con
            // módulo al total real, según contrato de `TeachingStep`).
            self.orchestrator.cancel();
            let _ = self.orchestrator.start(titulo, tmpl);
            self.anim_frames = None;
            self.clear_anim_textures_only(None);
        }
    }
    /// Progreso 0..=1 del morph burbuja (ANIM_MICRO 180ms ease-out).
    pub fn morph_progress(&self) -> f32 {
        let Some(opened) = self.opened_at else {
            return 1.0;
        };
        let elapsed_ms = Instant::now().duration_since(opened).as_secs_f32() * 1000.0;
        (elapsed_ms / grafito_ui::tokens::ANIM_MICRO).clamp(0.0, 1.0)
    }
}

// R2 — parsea `manim_template` forma `guion:k/n` → `(k, n)`.
// `None` si no es guion o está mal formado (`k` 1-based, `1 <= k <= n`).
// Puro, sin I/O — testeable headless.
pub(crate) fn parse_guion_template(tmpl: &str) -> Option<(u32, u32)> {
    let resto = tmpl.trim().strip_prefix("guion:")?;
    let (k_crudo, n_crudo) = resto.split_once('/')?;
    let k: u32 = k_crudo.trim().parse().ok()?;
    let n: u32 = n_crudo.trim().parse().ok()?;
    if k == 0 || n == 0 || k > n {
        return None;
    }
    Some((k, n))
}

// R2 — ¿el paso guion es válido contra la sesión? `n` debe igualar la
// cantidad de pasos (el adaptador `GuionASesion` emite `guion:k/n` con
// `n` = actos = pasos). Puro, sin I/O.
pub(crate) fn paso_guion_valido(manim_template: Option<&str>, pasos_sesion: usize) -> bool {
    parse_guion_template(manim_template.unwrap_or_default())
        .is_some_and(|(_, n)| n as usize == pasos_sesion)
}

/// R1-8: ¿el overlay debe cerrarse por entrada? `Esc` cierra siempre;
/// clic en el dim cierra SOLO si fue fuera de la ventana (el dim está
/// detrás: un clic en la ventana no llega al dim por orden de capas).
/// Puro, sin I/O — testeable headless.
pub(crate) fn overlay_debe_cerrar_por_entrada(esc: bool, clic_fuera: bool) -> bool {
    esc || clic_fuera
}

/// Dibuja la enseñanza si hay sesión activa. Retorna true si se cerró.
pub fn draw_teaching_overlay(
    state: &mut TeachingUiState,
    ctx: &egui::Context,
    budget: &mut crate::app::RepaintBudget,
    locale: grafito_ui::i18n::Locale,
    reduced_motion: bool,
) -> bool {
    // Tick de gracia SIEMPRE (haya o no sesión): libera sets viejos tras N
    // frames dibujados, nunca en el frame del reemplazo (submit en vuelo).
    state.reap_retired_anim_textures();
    if state.session.is_none() {
        return false;
    }
    // Unifica patrón cached_hash + ensure_textures como en anim_ui.rs:61 — evita clone masivo y leak.
    state.ensure_textures(ctx);
    // Snapshot inmutable para el closure (evita borrow cruzado &mut state.session + &state.orchestrator)
    let opened_at = state.opened_at;
    let Some(session_snapshot) = state.session.clone() else {
        return false;
    };
    let ledger = state.orchestrator.ledger.clone();
    let template = state.orchestrator.template.clone();
    let is_busy = state.orchestrator.is_busy();
    // Evita clone en draw: sólo se clonan handles si hash cambió (ensure_textures). Aquí se snapshot sin clonar frames.
    let anim_textures: Vec<egui::TextureHandle> = state.anim_textures.clone();
    let progress = session_snapshot.progress();
    let is_last = session_snapshot.is_last();
    let topic_label = titulo_limpio_for(&session_snapshot.topic.label(), locale);
    let step_count = session_snapshot.steps.len();
    let current_idx = session_snapshot.current;
    let current_step = session_snapshot.current().cloned();
    let etiqueta = etiqueta_paso_for(current_idx, step_count, locale);
    // Cache del parser del transcript: se clona para dibujar sin pelear
    // borrows con `state`, y se devuelve al cerrar la ventana.
    let mut bloques_cache = state.bloques.clone();

    let mut should_close = false;
    let mut should_advance = false;
    let mut should_retreat = false;
    let theme = grafito_ui::theme::current_theme(ctx);
    let _ = opened_at;
    // R1-8: `Esc` cierra el overlay (antes no había salida por teclado).
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        should_close = true;
    }
    // Fondo opaco a pantalla completa: tapa canvas y paneles para que nada
    // sangre dentro del overlay ni entren clics al canvas de atrás.
    // R1-8: el dim cierra SOLO si el clic fue fuera de la ventana (el Área
    // está detrás por `Order::Middle`; la ventana va encima y consume sus
    // propios clics, así que `clicked` acá = fuera).
    let dim_clic_fuera = egui::Area::new(egui::Id::new("teaching_dim"))
        .fixed_pos(ctx.screen_rect().min)
        .order(egui::Order::Middle)
        .show(ctx, |ui| {
            let (fondo, resp) =
                ui.allocate_exact_size(ctx.screen_rect().size(), egui::Sense::click());
            ui.painter()
                .rect_filled(fondo, 0.0, theme.panel_bg.gamma_multiply(0.92));
            resp.clicked()
        })
        .inner;
    if overlay_debe_cerrar_por_entrada(false, dim_clic_fuera) {
        should_close = true;
    }
    egui::Window::new(teach_msg("overlay_title", locale))
        .id(egui::Id::new("teaching_overlay"))
        .collapsible(false)
        .resizable(true)
        .default_width(VENTANA_ANCHO_IDEAL)
        .min_width(VENTANA_ANCHO_MIN)
        .max_width(VENTANA_ANCHO_MAX)
        .default_height(VENTANA_ALTO_IDEAL)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(theme.panel_bg)
                .stroke(Stroke::new(1.0, theme.separator.gamma_multiply(0.10)))
                .rounding(grafito_ui::tokens::RADIUS_LG)
                .inner_margin(egui::Margin::same(grafito_ui::tokens::SPACE_MD))
                .shadow(egui::Shadow {
                    offset: egui::vec2(0.0, grafito_ui::tokens::SHADOW_WINDOW_OFFSET_Y),
                    blur: grafito_ui::tokens::SHADOW_WINDOW_BLUR,
                    spread: 0.0,
                    color: Color32::from_black_alpha(grafito_ui::tokens::SHADOW_ALPHA),
                }),
        )
        .show(ctx, |ui| {
            let time = ui.input(|i| i.time);
            let hover_pos = ui.input(|i| i.pointer.hover_pos());
            // Header Scandinavian — left-aligned, avatar pequeño + título, cierre con icono
            ui.horizontal(|ui| {
                let cfg = grafito_profile::AvatarConfig::default();
                let (avatar_rect, _) = ui.allocate_exact_size(
                    egui::vec2(grafito_ui::tokens::TYPE_XXL, grafito_ui::tokens::TYPE_XXL),
                    egui::Sense::hover(),
                );
                if ui.is_rect_visible(avatar_rect) {
                    let painter = ui.painter_at(avatar_rect);
                    grafito_ui::avatar::draw_avatar(&painter, avatar_rect, &cfg, time, hover_pos);
                }
                ui.add_space(grafito_ui::tokens::SPACE_SM);
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new(topic_label.clone())
                            .strong()
                            .size(grafito_ui::tokens::TYPE_BASE)
                            .color(theme.text_primary),
                    );
                    ui.label(
                        egui::RichText::new(etiqueta.clone())
                            .size(grafito_ui::tokens::TYPE_XS)
                            // `dimmed_text` y no `gamma_multiply` crudo: el
                            // gamma directo saltaba el piso `TEXT_GAMMA_FLOOR`
                            // y en oscuro daba 4.00:1 (WCAG 1.4.3 pide 4.5:1).
                            // El clamp lleva el mismo 0.60 a 5.96:1 sin cambiar
                            // la intención de jerarquía en claro (6.58:1).
                            .color(grafito_ui::theme::Theme::dimmed_text(
                                theme.text_secondary,
                                0.60,
                            )),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if action_icon_button(
                        ui,
                        Icon::Close,
                        theme.text_secondary,
                        teach_msg("overlay_close", locale),
                    )
                    .clicked()
                    {
                        should_close = true;
                    }
                });
            });
            // Burbuja morph avatar→burbuja, siempre dentro del viewport.
            // Clamp duro a `available_rect`: nunca se pasa del borde visible.
            {
                let progreso = state.morph_progress();
                let suave = progreso_morph_suave(progreso);
                let limite = ui.available_rect_before_wrap();
                let ancho = limite.width().max(grafito_ui::tokens::SPACE_XXL * 2.0);
                let (espacio, _) = ui.allocate_exact_size(
                    egui::vec2(ancho, grafito_ui::tokens::SPACE_XXL),
                    egui::Sense::hover(),
                );
                let origen = egui::Rect::from_min_size(
                    espacio.min,
                    egui::vec2(
                        (espacio.width() * 0.35).max(grafito_ui::tokens::SPACE_XXL * 2.0),
                        grafito_ui::tokens::SPACE_LG,
                    ),
                );
                let destino = egui::Rect::from_min_size(
                    espacio.min,
                    egui::vec2(
                        espacio.width(),
                        grafito_ui::tokens::SPACE_LG + grafito_ui::tokens::SPACE_SM,
                    ),
                );
                let burbuja = rect_burbuja_morph(origen, destino, suave, limite);
                if ui.is_rect_visible(burbuja) {
                    let alpha = (80.0 + 175.0 * suave) as u8;
                    ui.painter().rect_filled(
                        burbuja,
                        grafito_ui::tokens::RADIUS_MD,
                        theme.accent.gamma_multiply(suave * 0.16),
                    );
                    ui.painter().rect_stroke(
                        burbuja,
                        grafito_ui::tokens::RADIUS_MD,
                        Stroke::new(1.0, theme.separator.gamma_multiply(0.10)),
                    );
                    let texto = exercise_step_hint_text(current_idx + 1, step_count, locale);
                    ui.painter().text(
                        burbuja.center(),
                        egui::Align2::CENTER_CENTER,
                        texto.chars().take(72).collect::<String>(),
                        egui::FontId::proportional(grafito_ui::tokens::TYPE_XS),
                        theme.text_primary.gamma_multiply(alpha as f32 / 255.0),
                    );
                }
                // Mientras el morph (<180ms) pide cuadros cortos; al asentar, nada.
                if progreso < 1.0 {
                    budget.request(std::time::Duration::from_millis(16));
                }
            }
            ui.add_space(grafito_ui::tokens::SPACE_SM);
            // Barra progreso — 8px tokenizada, sin animación extra
            let (r, _) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), grafito_ui::tokens::SPACE_SM),
                egui::Sense::hover(),
            );
            ui.painter()
                .rect_filled(r, 2.0, theme.separator.gamma_multiply(0.10));
            ui.painter().rect_filled(
                egui::Rect::from_min_size(r.min, egui::vec2(r.width() * progress, r.height())),
                2.0,
                theme.accent,
            );
            ui.add_space(grafito_ui::tokens::SPACE_SM);
            // Contenido principal — dentro de ScrollArea con max_height para no crear "altura enorme al pedo"
            // cuando la explicación es larga o la animación es alta. El footer (controles) queda fijo.
            let max_scroll_h = (ctx.screen_rect().height() * 0.55).clamp(220.0, 420.0);
            egui::ScrollArea::vertical()
                .id_salt("teaching_scroll")
                .max_height(max_scroll_h)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if let Some(step) = current_step.clone() {
                        egui::Frame::none()
                            .fill(theme.panel_bg)
                            .stroke(Stroke::new(1.0, theme.separator.gamma_multiply(0.10)))
                            .rounding(grafito_ui::tokens::RADIUS_LG)
                            .inner_margin(egui::Margin::same(grafito_ui::tokens::SPACE_MD))
                            .show(ui, |ui| {
                                ui.set_min_width(ui.available_width());
                                // Título del paso: fuente única y sin marcas
                                // crudas (el tópico puede traer "## Enfoque"
                                // del transcript del asistente).
                                ui.label(
                                    egui::RichText::new(titulo_limpio_for(&step.title, locale))
                                        .strong()
                                        .size(grafito_ui::tokens::TYPE_MD)
                                        .color(theme.accent),
                                );
                                ui.add_space(grafito_ui::tokens::SPACE_XS);
                                // Explicación con la gramática del transcript
                                // (encabezados, listas y citas renderizados,
                                // jamás `##` o `**` crudos).
                                draw_bloques_markdown(
                                    ui,
                                    &step.explanation,
                                    &mut bloques_cache,
                                    locale,
                                );
                                if let Some(expr) = &step.math_expr {
                                    ui.add_space(grafito_ui::tokens::SPACE_SM);
                                    egui::Frame::none()
                                        .fill(theme.input_bg)
                                        .stroke(Stroke::new(
                                            1.0,
                                            theme.separator.gamma_multiply(0.10),
                                        ))
                                        .rounding(grafito_ui::tokens::RADIUS_MD)
                                        .inner_margin(egui::Margin::same(
                                            grafito_ui::tokens::SPACE_SM,
                                        ))
                                        .show(ui, |ui| {
                                            ui.label(
                                                egui::RichText::new(teach_msg(
                                                    "verified_expr",
                                                    locale,
                                                ))
                                                .strong()
                                                .size(grafito_ui::tokens::TYPE_XS)
                                                .color(theme.accent),
                                            );
                                            // MathTex overlay: mismo renderer del asistente
                                            // (`draw_math_with_locale`, fuente `$..$`); con
                                            // fallback a texto si el parse falla. Solo llega
                                            // acá lo verificado por el CAS-gate (`verified`).
                                            draw_math_with_locale(ui, expr, locale);
                                        });
                                }
                            });
                        // Pizarra: una sola fuente para la etiqueta (acá) y el
                        // contenido (el lienzo ya dibuja sus vectores: nada de
                        // texto centrado encima ni "Pizarra:" duplicado).
                        // Vacía → nada (sin recuadro negro); sólo-texto →
                        // placeholder honesto; con dibujo → lienzo legible.
                        match estado_pizarra(&step.whiteboard_hint, state.whiteboard.doc.is_empty())
                        {
                            EstadoPizarra::Oculta => {}
                            EstadoPizarra::Placeholder => {
                                ui.add_space(grafito_ui::tokens::SPACE_SM);
                                ui.label(
                                    egui::RichText::new(teach_msg("board_ready_hint", locale))
                                        .size(grafito_ui::tokens::TYPE_XS)
                                        .color(theme.text_secondary),
                                );
                            }
                            EstadoPizarra::Lienzo => {
                                ui.add_space(grafito_ui::tokens::SPACE_SM);
                                egui::Frame::none()
                                    .fill(theme.input_bg)
                                    .stroke(Stroke::new(1.0, theme.separator.gamma_multiply(0.10)))
                                    .rounding(grafito_ui::tokens::RADIUS_MD)
                                    .inner_margin(egui::Margin::same(grafito_ui::tokens::SPACE_SM))
                                    .show(ui, |ui| {
                                        ui.set_min_width(ui.available_width());
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                egui::RichText::new(teach_msg(
                                                    "board_title",
                                                    locale,
                                                ))
                                                .size(grafito_ui::tokens::TYPE_XS)
                                                .color(theme.text_tertiary)
                                                .strong(),
                                            );
                                            let hint =
                                                titulo_limpio_for(&step.whiteboard_hint, locale);
                                            if !hint.is_empty() {
                                                ui.label(
                                                    egui::RichText::new(format!("· {hint}"))
                                                        .size(grafito_ui::tokens::TYPE_XS)
                                                        .color(theme.text_secondary),
                                                );
                                            }
                                        });
                                        ui.add_space(grafito_ui::tokens::SPACE_XS);
                                        // Altura legible: 120 ideal, clamp a
                                        // 96..160 y a 35% del alto disponible.
                                        let wb_h = (grafito_ui::tokens::SPACE_XXL * 3.0)
                                            .clamp(96.0, 160.0)
                                            .min((ui.available_height() * 0.35).max(96.0));
                                        let (wb_rect, _) = ui.allocate_exact_size(
                                            egui::vec2(ui.available_width(), wb_h),
                                            egui::Sense::click_and_drag(),
                                        );
                                        // Dibujar pizarra vectorial real (trazo, rectángulos, flechas)
                                        state.whiteboard.draw(ui, wb_rect);
                                        // Permitir dibujar encima (pencil) dentro del overlay
                                        state.whiteboard.handle_canvas_input(wb_rect, ui, budget);
                                        if ui.is_rect_visible(wb_rect) {
                                            // Borde sutil por encima del draw para definición
                                            ui.painter().rect_stroke(
                                                wb_rect,
                                                grafito_ui::tokens::RADIUS_MD,
                                                Stroke::new(
                                                    1.0,
                                                    theme.separator.gamma_multiply(0.08),
                                                ),
                                            );
                                        }
                                    });
                            }
                        }
                        // Animación nativa fallback (si completó)
                        if !anim_textures.is_empty() {
                            ui.add_space(grafito_ui::tokens::SPACE_SM);
                            // Playback por cue: el paso actual puede traer
                            // `frame_range` (ventana sobre el loop); sin rango
                            // es el loop completo de 12 fps de siempre.
                            // WCAG 2.2.2 + `reduced_motion`: pausa explícita
                            // congela el cuadro (sin churn de repaint); el
                            // botón de abajo alterna Play/Pausa.
                            let now_elapsed_ms = opened_at
                                .map(|t| {
                                    Instant::now()
                                        .duration_since(t)
                                        .as_millis()
                                        .min(u64::MAX as u128)
                                        as u64
                                })
                                .unwrap_or(0);
                            let en_pausa = state.paused || reduced_motion;
                            if en_pausa && state.frozen_ms.is_none() {
                                state.frozen_ms = Some(now_elapsed_ms);
                            }
                            let elapsed_ms = if en_pausa {
                                state.frozen_ms.unwrap_or(now_elapsed_ms)
                            } else {
                                // Al salir de la pausa/reduced-motion, rebasa
                                // `opened_at` para retomar EXACTO el cuadro
                                // congelado (sin salto hacia adelante).
                                if let Some(frozen) = state.frozen_ms.take() {
                                    state.opened_at = Some(
                                        Instant::now() - std::time::Duration::from_millis(frozen),
                                    );
                                }
                                now_elapsed_ms
                            };
                            let len = anim_textures.len();
                            let idx = current_step
                                .as_ref()
                                .and_then(|st| {
                                    TeachingSession::cue_frame_index(st, elapsed_ms, 12.0, len)
                                })
                                .unwrap_or(0)
                                .min(len.saturating_sub(1));
                            debug_assert!(idx < anim_textures.len());
                            let tex = &anim_textures[idx];
                            let max_w = ui
                                .available_width()
                                .max(grafito_ui::tokens::SPACE_XXL * 2.0);
                            // Clampear altura para no generar "altura enorme al pedo" con texturas retrato
                            let max_h = (grafito_ui::tokens::SPACE_XXL * 5.0).min(
                                ui.available_height()
                                    .max(grafito_ui::tokens::SPACE_XXL * 2.0)
                                    * 0.6,
                            );
                            let size = tex.size_vec2();
                            let scale_w = (max_w / size.x.max(1.0)).clamp(0.25, 1.0);
                            let scale_h = (max_h / size.y.max(1.0)).clamp(0.25, 1.0);
                            let scale = scale_w.min(scale_h);
                            let display = egui::vec2(size.x * scale, size.y * scale).ceil();
                            let (rect, _) = ui.allocate_exact_size(display, egui::Sense::hover());
                            ui.painter().image(
                                tex.id(),
                                rect,
                                egui::Rect::from_min_max(
                                    egui::pos2(0.0, 0.0),
                                    egui::pos2(1.0, 1.0),
                                ),
                                Color32::WHITE,
                            );
                            ui.painter().rect_stroke(
                                rect,
                                grafito_ui::tokens::RADIUS_MD,
                                Stroke::new(1.0, theme.separator.gamma_multiply(0.10)),
                            );
                            // F17: playback 12fps vía presupuesto coalescido.
                            // En pausa (o reduced_motion) NO se pide repaint:
                            // el loop queda congelado, cero churn.
                            if !en_pausa {
                                budget.request(Duration::from_millis(80));
                            }
                            ui.add_space(grafito_ui::tokens::SPACE_XS);
                            ui.label(
                                egui::RichText::new(overlay_anim_reserve_text(
                                    &template,
                                    anim_textures.len(),
                                    locale,
                                ))
                                .size(grafito_ui::tokens::TYPE_XS)
                                .color(theme.text_tertiary)
                                .weak(),
                            );
                            // Botón Play/Pausa (igual player por turno del
                            // panel: "⏸"/"▶" + tooltip localizado). En
                            // `reduced_motion` queda disabled con hint.
                            ui.add_space(grafito_ui::tokens::SPACE_XS);
                            ui.horizontal(|ui| {
                                let (glyph, tip_key) = if en_pausa {
                                    ("▶", "assistant.media_tip_play")
                                } else {
                                    ("⏸", "assistant.media_tip_pause")
                                };
                                let tip = if reduced_motion {
                                    teach_msg("reduced_motion_active", locale).to_owned()
                                } else {
                                    grafito_ui::i18n::t(tip_key, locale).to_owned()
                                };
                                let resp = if reduced_motion {
                                    ui.add_enabled(false, egui::Button::new(glyph))
                                } else {
                                    ui.add_sized(egui::vec2(24.0, 24.0), egui::Button::new(glyph))
                                };
                                let resp = resp.on_hover_text(tip.clone());
                                let tip_para_widget = tip.clone();
                                resp.widget_info(|| {
                                    egui::WidgetInfo::labeled(
                                        egui::WidgetType::Button,
                                        !reduced_motion,
                                        tip_para_widget.clone(),
                                    )
                                });
                                if !reduced_motion
                                    && (resp.clicked()
                                        || (resp.has_focus()
                                            && ui.input(|i| i.key_pressed(egui::Key::Space))))
                                {
                                    if state.paused {
                                        // Reproducir: retomar justo donde quedó.
                                        if let Some(frozen) = state.frozen_ms {
                                            state.opened_at = Some(
                                                Instant::now()
                                                    - std::time::Duration::from_millis(frozen),
                                            );
                                        }
                                        state.paused = false;
                                    } else {
                                        state.paused = true;
                                    }
                                }
                            });
                        } else if is_busy {
                            ui.add_space(grafito_ui::tokens::SPACE_XS);
                            ui.horizontal(|ui| {
                                let t = ui.input(|i| i.time);
                                let pulse = ((t * 3.0).sin() + 1.0) * 0.5;
                                let col = theme.accent.gamma_multiply(0.45 + 0.55 * pulse as f32);
                                let (rect, _) = ui.allocate_exact_size(
                                    egui::vec2(
                                        grafito_ui::tokens::SPACE_SM
                                            + grafito_ui::tokens::SHADOW_WINDOW_OFFSET_Y,
                                        grafito_ui::tokens::SPACE_SM
                                            + grafito_ui::tokens::SHADOW_WINDOW_OFFSET_Y,
                                    ),
                                    egui::Sense::hover(),
                                );
                                ui.painter().circle_filled(
                                    rect.center(),
                                    grafito_ui::tokens::SPACE_XS,
                                    col,
                                );
                                ui.label(
                                    egui::RichText::new(teach_msg(
                                        "overlay_generating_manim",
                                        locale,
                                    ))
                                    .size(grafito_ui::tokens::TYPE_XS)
                                    .color(theme.text_secondary),
                                );
                            });
                            // F17: pulso "Generando animación" vía presupuesto coalescido.
                            budget.request(Duration::from_millis(48));
                        }
                        // Ledger colapsable — no ocupa altura si no se necesita
                        if let Some(ledger) = &ledger {
                            ui.add_space(grafito_ui::tokens::SPACE_XS);
                            egui::CollapsingHeader::new(
                                egui::RichText::new(teach_msg("overlay_gen_detail", locale))
                                    .size(grafito_ui::tokens::TYPE_XS)
                                    .color(theme.text_tertiary),
                            )
                            .id_salt("teaching_ledger")
                            .show(ui, |ui| {
                                egui::Frame::none()
                                    .fill(theme.input_bg)
                                    .stroke(Stroke::new(1.0, theme.separator.gamma_multiply(0.08)))
                                    .rounding(grafito_ui::tokens::RADIUS_MD)
                                    .inner_margin(egui::Margin::same(grafito_ui::tokens::SPACE_SM))
                                    .show(ui, |ui| {
                                        ui.label(
                                            egui::RichText::new(ledger)
                                                .monospace()
                                                .size(grafito_ui::tokens::TYPE_2XS)
                                                .color(theme.text_secondary),
                                        );
                                    });
                            });
                        }
                    }
                });
            ui.add_space(grafito_ui::tokens::SPACE_MD);
            // Controles en UNA fila limpia: [Atrás][Siguiente][Cerrar].
            // Sin iconos decorativos sueltos ni filas partidas.
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = grafito_ui::tokens::SPACE_SM;
                let puede_atras = current_idx > 0;
                let atras = egui::Button::new(
                    egui::RichText::new(teach_msg("overlay_back", locale))
                        .size(grafito_ui::tokens::TYPE_SM),
                )
                .fill(egui::Color32::TRANSPARENT)
                .stroke(Stroke::new(1.0, theme.separator.gamma_multiply(0.12)))
                .rounding(grafito_ui::tokens::RADIUS_SM);
                if ui
                    .add_enabled(puede_atras, atras)
                    .on_hover_text(teach_msg("overlay_back_hint", locale))
                    .clicked()
                {
                    should_retreat = true;
                }
                let primary_label = if is_last {
                    teach_msg("overlay_finish", locale)
                } else {
                    teach_msg("overlay_next", locale)
                };
                // Botón primario: fill accent, 36h, RADIUS_MD
                let btn = egui::Button::new(
                    egui::RichText::new(primary_label)
                        .strong()
                        .size(grafito_ui::tokens::TYPE_SM)
                        .color(Color32::WHITE),
                )
                .fill(theme.accent)
                .stroke(Stroke::NONE)
                .rounding(grafito_ui::tokens::RADIUS_SM);
                // Tamaño igual para primaria, ocupa espacio proporcional
                if ui
                    .add_sized(
                        egui::vec2(
                            grafito_ui::tokens::SPACE_XXL * 3.5,
                            grafito_ui::tokens::SPACE_LG * 2.25,
                        ),
                        btn,
                    )
                    .on_hover_text(if is_last {
                        teach_msg("overlay_finish_hint", locale)
                    } else {
                        teach_msg("overlay_next_hint", locale)
                    })
                    .clicked()
                {
                    if is_last {
                        should_close = true;
                    } else {
                        should_advance = true;
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let ghost = egui::Button::new(
                        egui::RichText::new(teach_msg("overlay_close", locale))
                            .size(grafito_ui::tokens::TYPE_SM),
                    )
                    .fill(egui::Color32::TRANSPARENT)
                    .stroke(Stroke::new(1.0, theme.separator.gamma_multiply(0.12)))
                    .rounding(grafito_ui::tokens::RADIUS_SM);
                    if ui
                        .add_sized(
                            egui::vec2(
                                grafito_ui::tokens::SPACE_XL * 4.0,
                                grafito_ui::tokens::SPACE_LG * 2.25,
                            ),
                            ghost,
                        )
                        .clicked()
                    {
                        should_close = true;
                    }
                });
            });
        });
    // Devuelve el cache del parser (con lo ya parseado) al estado.
    state.bloques = bloques_cache;
    if should_retreat {
        state.retroceder();
    }
    if should_advance {
        // `advance` ya retiró con gracia (sin ctx: el drop real ocurre en
        // los ticks); `forget_image` sería no-op para managed. Sin drop
        // post-draw en este frame: el submit en vuelo sigue intacto.
        let ok = state.advance();
        let _ = ok;
    }
    if should_close {
        state.close_with_ctx(ctx);
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r2_guion_template_parsea_k_n_y_rechaza_malos() {
        assert_eq!(parse_guion_template("guion:1/3"), Some((1, 3)));
        assert_eq!(parse_guion_template("  guion:2/2 "), Some((2, 2)));
        assert_eq!(parse_guion_template("derivative-slope"), None);
        assert_eq!(parse_guion_template("guion:0/3"), None);
        assert_eq!(parse_guion_template("guion:4/3"), None);
        assert_eq!(parse_guion_template("guion:1/0"), None);
        assert_eq!(parse_guion_template("guion:x/3"), None);
        assert_eq!(parse_guion_template("guion:1"), None);
        assert_eq!(parse_guion_template("guion:"), None);
        assert_eq!(parse_guion_template(""), None);
    }

    #[test]
    fn r2_paso_guion_valida_n_contra_sesion() {
        // `n` debe igualar los pasos de la sesión (a_sesion: n = actos).
        assert!(paso_guion_valido(Some("guion:1/3"), 3));
        assert!(paso_guion_valido(Some("guion:3/3"), 3));
        assert!(!paso_guion_valido(Some("guion:1/3"), 2));
        assert!(!paso_guion_valido(Some("guion:4/3"), 3));
        assert!(!paso_guion_valido(Some("derivative-slope"), 3));
        assert!(!paso_guion_valido(None, 3));
    }

    #[test]
    fn r2_poll_cues_guion_valido_relanza_orquestador() {
        use grafito_pedagogy::{TeachingStep, TeachingTopic};
        // Sesión de 1 paso con template de guion válido (`n` = pasos).
        let paso = TeachingStep::new("g1", "apertura", "recta secante")
            .with_manim("guion:1/1")
            .with_cue(0);
        let ahora = Instant::now();
        let mut ui = TeachingUiState {
            session: Some(TeachingSession::new(
                TeachingTopic::from_text("derivada"),
                vec![paso],
            )),
            session_started_at: Some(ahora),
            last_revealed: 0,
            ..Default::default()
        };
        assert_ne!(ui.orchestrator.template, "guion:1/1");
        ui.poll_cues(ahora);
        assert_eq!(ui.last_revealed, 1);
        assert_eq!(ui.orchestrator.template, "guion:1/1");
    }

    #[test]
    fn r2_poll_cues_guion_invalido_cae_a_expositivo() {
        use grafito_pedagogy::{TeachingStep, TeachingTopic};
        // `guion:2/1` no valida (k > n): fallback a paso expositivo —
        // pizarra hidratada, sin animación, sin pánico.
        let paso = TeachingStep::new("g1", "apertura", "recta secante")
            .with_whiteboard("Dibuja la curva x² y una secante entre dos puntos")
            .with_manim("guion:2/1")
            .with_cue(0);
        let ahora = Instant::now();
        let mut ui = TeachingUiState {
            session: Some(TeachingSession::new(
                TeachingTopic::from_text("derivada"),
                vec![paso],
            )),
            session_started_at: Some(ahora),
            last_revealed: 0,
            ..Default::default()
        };
        ui.poll_cues(ahora);
        assert_eq!(ui.last_revealed, 1);
        assert!(!ui.whiteboard.doc.is_empty(), "la pizarra igual hidrata");
        assert!(ui.anim_frames.is_none(), "sin frames ante guion roto");
        assert!(!ui.orchestrator.is_busy(), "orchestrator cancelado");
    }

    #[test]
    fn r1_overlay_esc_y_clic_fuera_cierran() {
        // R1-8 flujo abrir→Esc→fuera (headless, puro sobre la decisión):
        // `Esc` cierra, dim cierra solo si clic fuera, nada cierra solo.
        assert!(overlay_debe_cerrar_por_entrada(true, false), "Esc cierra");
        assert!(
            overlay_debe_cerrar_por_entrada(false, true),
            "clic fuera cierra"
        );
        assert!(overlay_debe_cerrar_por_entrada(true, true), "ambos cierran");
        assert!(
            !overlay_debe_cerrar_por_entrada(false, false),
            "sin entrada no cierra solo"
        );
        // El overlay existe con sesión (abrir): sin sesión no hay qué cerrar.
        let mut con_sesion = TeachingUiState {
            session: Some(TeachingSession::for_topic("derivada")),
            ..Default::default()
        };
        assert!(con_sesion.session.is_some(), "abrir deja sesión");
        let ctx = egui::Context::default();
        con_sesion.close_with_ctx(&ctx);
        assert!(con_sesion.session.is_none(), "cerrar limpia la sesión");
    }

    #[test]
    fn cues_hidratan_progresivo_y_navegacion_manda() {
        // Binding temporal: la pizarra hidrata por cue, no todo de golpe; la
        // navegación manual congela los cues.
        let mut ui = TeachingUiState {
            session: Some(TeachingSession::for_topic("derivada")),
            ..Default::default()
        };
        let t0 = Instant::now();
        ui.session_started_at = Some(t0);
        ui.last_revealed = 0;
        ui.poll_cues(t0);
        assert_eq!(ui.last_revealed, 1, "cue 0 revela el paso 1");
        let n1 = ui.whiteboard.doc.len();
        assert!(n1 > 0, "paso 1 hidrata dibujo útil");
        // Sin cambio de cue no re-hidrata (mismo conteo).
        ui.poll_cues(t0 + Duration::from_millis(100));
        assert_eq!(ui.last_revealed, 1);
        // Cue de 5 s revela el paso 2 y acumula elementos.
        ui.poll_cues(t0 + Duration::from_millis(5_000));
        assert_eq!(ui.last_revealed, 2);
        assert!(
            ui.whiteboard.doc.len() >= n1,
            "cues acumulan, no reemplazan"
        );
        // Navegación manual: los cues ya no pisan.
        ui.advance();
        let tras_avance = ui.whiteboard.doc.len();
        ui.poll_cues(t0 + Duration::from_millis(60_000));
        assert_eq!(
            ui.whiteboard.doc.len(),
            tras_avance,
            "navegar congela los cues"
        );
    }
    #[test]
    fn presupuestos_texto_pedagogico_pinneados() {
        // Onda 2: pinnea los topes user-facing de la tarjeta de ejercicio
        // (enunciado/opción) para que la tarjeta no desborde.
        assert_eq!(MAX_ENUNCIADO_CHARS, 280);
        assert_eq!(MAX_OPCION_CHARS, 120);
    }

    fn luminancia_relativa(color: egui::Color32) -> f64 {
        let canal = |v: u8| {
            let v = f64::from(v) / 255.0;
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * canal(color.r()) + 0.7152 * canal(color.g()) + 0.0722 * canal(color.b())
    }

    fn razon_contraste(frente: egui::Color32, fondo: egui::Color32) -> f64 {
        let (a, b) = (luminancia_relativa(frente), luminancia_relativa(fondo));
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    #[test]
    fn tarjeta_devolucion_legible_en_oscuro_y_claro() {
        // S1: mide AA sobre los colores RESUELTOS por el draw (misma
        // resolución: `colores_devolucion` + `colores_texto_devolucion`),
        // no sobre `text_primary` copiado a mano. Si el draw volviera al
        // hardcode, lo resuelto cambia y este test cae (ver trampa abajo).
        for (nombre_tema, tema) in [
            ("oscuro", &*grafito_ui::theme::DARK),
            ("claro", &*grafito_ui::theme::LIGHT),
        ] {
            let (cuerpo, sugerencia) = colores_texto_devolucion(tema);
            for correcto in [true, false] {
                let (borde, fondo) = colores_devolucion(correcto, tema);
                assert_eq!(fondo, tema.card_bg, "fondo siempre elevado del tema");
                let _ = borde;
                for (nombre, texto) in [("cuerpo", cuerpo), ("sugerencia", sugerencia)] {
                    let razon = razon_contraste(texto, fondo);
                    assert!(
                        razon >= 4.5,
                        "{nombre} {texto:?} sobre devolución {fondo:?} \
                         (tema {nombre_tema}, correcto={correcto}) = {razon:.2}:1 < 4.5:1"
                    );
                }
            }
        }
    }

    #[test]
    fn tarjeta_hardcode_historico_no_pasa_aa_en_oscuro() {
        // Trampa: los valores fijos previos al polish (`#282828` cuerpo,
        // `#5A5A5A` sugerencia, `#E8F5E9`/`#FDEDED` fondos) NO llegan a
        // 4.5:1 en oscuro y difieren de lo resuelto hoy. Si el draw
        // revirtiera al hardcode, el test AA principal fallaría: este test
        // lo demuestra inyectando el hardcode y exigiendo el fallo.
        let cuerpo_viejo = egui::Color32::from_rgb(40, 40, 40);
        let sugerencia_vieja = egui::Color32::from_rgb(90, 90, 90);
        let fondo_ok_viejo = egui::Color32::from_rgb(232, 245, 233);
        let fondo_err_viejo = egui::Color32::from_rgb(253, 237, 237);
        let oscuro = &*grafito_ui::theme::DARK;
        let razon_vieja = razon_contraste(cuerpo_viejo, oscuro.card_bg);
        assert!(
            razon_vieja < 4.5,
            "el hardcode viejo debería fallar en oscuro, dio {razon_vieja:.2}:1"
        );
        for correcto in [true, false] {
            let (_, fondo) = colores_devolucion(correcto, oscuro);
            assert_ne!(fondo, fondo_ok_viejo, "el draw volvió a #E8F5E9");
            assert_ne!(fondo, fondo_err_viejo, "el draw volvió a #FDEDED");
        }
        let (cuerpo, sugerencia) = colores_texto_devolucion(oscuro);
        assert_ne!(cuerpo, cuerpo_viejo, "el draw volvió a #282828");
        assert_ne!(sugerencia, sugerencia_vieja, "el draw volvió a #5A5A5A");
    }

    fn contar_anillos_foco(
        shapes: &[egui::epaint::ClippedShape],
        tema: &grafito_ui::theme::Theme,
    ) -> usize {
        shapes
            .iter()
            .filter(|s| match &s.shape {
                egui::Shape::Rect(rect) => {
                    rect.stroke.color == tema.selection_outline
                        && (rect.stroke.width - grafito_ui::tokens::FOCUS_RING_WIDTH).abs()
                            < f32::EPSILON
                }
                _ => false,
            })
            .count()
    }

    fn dibujar_tarjeta(
        ctx: &egui::Context,
        ejercicio: &Exercise,
        opciones: &[String],
        estado: &mut EstadoTarjetaEjercicio,
    ) -> egui::FullOutput {
        // `focused: true`: sin foco de ventana, `Response::has_focus`
        // siempre da falso y el anillo nunca se pintaría en el test.
        let entrada = egui::RawInput {
            focused: true,
            ..Default::default()
        };
        ctx.run(entrada, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                draw_tarjeta_ejercicio(
                    ui,
                    ejercicio,
                    opciones,
                    estado,
                    grafito_ui::i18n::Locale::Es,
                );
            });
        })
    }

    #[test]
    fn tarjeta_dibuja_ok_y_error_sin_panico_y_con_foco() {
        // S1 smoke headless: la tarjeta se dibuja con devolución ok y error
        // en ambos temas; el hook (dentro del closure de `widget_info`, con
        // `show_interactive_widgets`) prueba que cada radio + Responder
        // anunció etiqueta y rol de verdad, y el foco al botón pinta el
        // anillo (delta de shapes con `selection_outline` > 0).
        for (nombre_tema, oscuro) in [("oscuro", true), ("claro", false)] {
            let tema = if oscuro {
                &*grafito_ui::theme::DARK
            } else {
                &*grafito_ui::theme::LIGHT
            };
            let ctx = egui::Context::default();
            tema.apply(&ctx);
            ctx.style_mut(|s| s.debug.show_interactive_widgets = true);
            for correcta in [true, false] {
                let ejercicio =
                    armar_ejercicio("derivada", grafito_pedagogy::PedagogicalLevel::Secondary, 7)
                        .expect("ejercicio válido");
                let opciones = opciones_para_ejercicio(&ejercicio);
                assert!(!opciones.is_empty(), "la tarjeta necesita opciones");
                let fb = grafito_pedagogy::FeedbackEngine.assess(
                    &ejercicio,
                    if correcta {
                        &ejercicio.solution
                    } else {
                        "respuesta errónea a propósito"
                    },
                );
                let mut estado = EstadoTarjetaEjercicio {
                    respuesta: String::new(),
                    opcion_elegida: Some(0),
                    devolucion: Some(fb),
                };
                limpiar_infos_tarjeta();
                // Sin foco heredado de la iteración anterior: la base debe
                // medir la tarjeta sin anillo.
                ctx.memory_mut(|m| {
                    if let Some(id) = m.focused() {
                        m.surrender_focus(id);
                    }
                });
                let salida = dibujar_tarjeta(&ctx, &ejercicio, &opciones, &mut estado);
                let infos = infos_tarjeta();
                // Cada radio anunció su etiqueta + rol + selección.
                for (idx, op) in opciones.iter().enumerate() {
                    let etiqueta = acotar_texto(op, MAX_OPCION_CHARS);
                    let hallada = infos.iter().find(|(_, i)| {
                        i.typ == egui::WidgetType::RadioButton
                            && i.label.as_deref() == Some(&etiqueta)
                    });
                    assert!(
                        hallada.is_some(),
                        "radio {idx} sin widget_info (tema {nombre_tema}, correcta={correcta})"
                    );
                    let (_, info) = hallada.expect("radio anunciado");
                    assert!(info.enabled, "radio {idx} anunciado deshabilitado");
                    assert_eq!(
                        info.selected,
                        Some(idx == 0),
                        "radio {idx} con selección errónea"
                    );
                }
                // El botón Responder anunció etiqueta + rol botón.
                let etiqueta_boton =
                    teach_msg("exercise_respond", grafito_ui::i18n::Locale::Es).to_string();
                let boton = infos
                    .iter()
                    .find(|(_, i)| {
                        i.typ == egui::WidgetType::Button
                            && i.label.as_deref() == Some(etiqueta_boton.as_str())
                    })
                    .unwrap_or_else(|| {
                        panic!(
                        "botón Responder sin widget_info (tema {nombre_tema}, correcta={correcta})"
                    )
                    });
                assert!(boton.1.enabled, "Responder anunciado deshabilitado");
                // Foco al botón → el anillo aparece en los shapes.
                let base = contar_anillos_foco(&salida.shapes, tema);
                ctx.memory_mut(|m| m.request_focus(boton.0));
                let salida_foco = dibujar_tarjeta(&ctx, &ejercicio, &opciones, &mut estado);
                let con_foco = contar_anillos_foco(&salida_foco.shapes, tema);
                assert!(
                    con_foco > base,
                    "sin anillo de foco en Responder (tema {nombre_tema}): {base} → {con_foco}"
                );
            }
        }
    }

    fn color_efectivo_glifo(ts: &egui::epaint::TextShape) -> Vec<egui::Color32> {
        // Color con el que el glifo llega a pantalla: `override` manda;
        // si no, cada sección del galley (PLACEHOLDER → `fallback`).
        if let Some(forzado) = ts.override_text_color {
            return vec![forzado];
        }
        ts.galley
            .job
            .sections
            .iter()
            .map(|s| {
                if s.format.color == egui::Color32::PLACEHOLDER {
                    ts.fallback_color
                } else {
                    s.format.color
                }
            })
            .collect()
    }

    fn recolectar_textos_pintados(
        shape: &egui::Shape,
        salida: &mut Vec<(String, Vec<egui::Color32>)>,
    ) {
        match shape {
            egui::Shape::Text(ts) => {
                salida.push((ts.galley.job.text.clone(), color_efectivo_glifo(ts)));
            }
            egui::Shape::Vec(formas) => {
                for s in formas {
                    recolectar_textos_pintados(s, salida);
                }
            }
            _ => {}
        }
    }

    #[test]
    fn tarjeta_pinta_cuerpo_con_color_del_tema() {
        // Cierre de circularidad helper/draw: observa LO PINTADO (shapes de
        // salida del draw), no los helpers. Dibuja la tarjeta en ctx de test
        // (2 temas × ok/error), ubica el TextShape del cuerpo del feedback
        // (mensaje) y exige que su color == `text_primary` resuelto del
        // tema y != hardcodes históricos. Si el draw revirtiera a hardcode
        // dejando los helpers intactos, este test cae (ver mutación abajo).
        let hardcodes = [
            egui::Color32::from_rgb(40, 40, 40),
            egui::Color32::from_rgb(90, 90, 90),
            egui::Color32::from_rgb(232, 245, 233),
            egui::Color32::from_rgb(253, 237, 237),
        ];
        for (nombre_tema, tema) in [
            ("oscuro", &*grafito_ui::theme::DARK),
            ("claro", &*grafito_ui::theme::LIGHT),
        ] {
            let esperado = colores_texto_devolucion(tema).0;
            assert_eq!(esperado, tema.text_primary);
            for h in hardcodes {
                assert_ne!(
                    esperado, h,
                    "el tema resolvió a un hardcode ({nombre_tema})"
                );
            }
            for correcta in [true, false] {
                let ctx = egui::Context::default();
                tema.apply(&ctx);
                let ejercicio =
                    armar_ejercicio("derivada", grafito_pedagogy::PedagogicalLevel::Secondary, 7)
                        .expect("ejercicio válido");
                let opciones = opciones_para_ejercicio(&ejercicio);
                assert!(!opciones.is_empty(), "la tarjeta necesita opciones");
                let fb = grafito_pedagogy::FeedbackEngine.assess(
                    &ejercicio,
                    if correcta {
                        &ejercicio.solution
                    } else {
                        "respuesta errónea a propósito"
                    },
                );
                let cuerpo = acotar_texto(&fb.message, MAX_ENUNCIADO_CHARS);
                assert!(!cuerpo.is_empty(), "el feedback necesita mensaje");
                let mut estado = EstadoTarjetaEjercicio {
                    respuesta: String::new(),
                    opcion_elegida: Some(0),
                    devolucion: Some(fb),
                };
                let salida = dibujar_tarjeta(&ctx, &ejercicio, &opciones, &mut estado);
                let mut pintados = Vec::new();
                for recortada in &salida.shapes {
                    recolectar_textos_pintados(&recortada.shape, &mut pintados);
                }
                let hallados: Vec<_> = pintados
                    .iter()
                    .filter(|(texto, _)| texto.contains(cuerpo.as_str()))
                    .collect();
                assert!(
                    !hallados.is_empty(),
                    "cuerpo del feedback no pintado (tema {nombre_tema}, correcta={correcta})"
                );
                for (_, colores) in hallados {
                    assert!(!colores.is_empty(), "TextShape del cuerpo sin color");
                    for color in colores {
                        assert_eq!(
                            *color, esperado,
                            "cuerpo pintado {color:?} != tema {esperado:?} \
                             (tema {nombre_tema}, correcta={correcta})"
                        );
                        for h in hardcodes {
                            assert_ne!(
                                *color, h,
                                "cuerpo volvió al hardcode {h:?} \
                                 (tema {nombre_tema}, correcta={correcta})"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn tarjeta_anillo_foco_visible_y_contrastado() {
        // WCAG 2.4.13 (foco ≥ 2 px) + 1.4.11 (no-texto ≥ 3:1) en ambos modos
        // y geometría expandida fuera del widget.
        assert_eq!(
            grafito_ui::tokens::FOCUS_RING_WIDTH,
            2.0,
            "el anillo debe medir 2 px"
        );
        for (nombre_tema, tema) in [
            ("oscuro", &*grafito_ui::theme::DARK),
            ("claro", &*grafito_ui::theme::LIGHT),
        ] {
            let trazo = tema.focus_ring_stroke();
            assert!(trazo.width >= 2.0, "trazo fino en {nombre_tema}");
            for (nombre_fondo, fondo) in [("panel", tema.panel_bg), ("card", tema.card_bg)] {
                let razon = razon_contraste(trazo.color, fondo);
                assert!(
                    razon >= 3.0,
                    "anillo {trazo:?} sobre {nombre_fondo} ({nombre_tema}) = {razon:.2}:1 < 3:1"
                );
            }
            let rect = egui::Rect::from_min_size(egui::pos2(10.0, 10.0), egui::vec2(80.0, 24.0));
            let expandido = grafito_ui::theme::Theme::focus_ring_rect(rect);
            assert!(
                expandido.contains_rect(rect) && expandido != rect,
                "el anillo debe rodear al widget ({nombre_tema})"
            );
        }
    }

    #[test]
    fn tarjeta_devolucion_cablea_semaforo_del_tema() {
        // Wiring: el borde sale de `success`/`danger` del tema (sin `from_rgb`
        // propio) y el fondo es `card_bg`; el botón Responder usa texto
        // legible sobre `accent` en ambos modos (≥ 4.5:1).
        let oscuro = &*grafito_ui::theme::DARK;
        let claro = &*grafito_ui::theme::LIGHT;
        assert_eq!(colores_devolucion(true, oscuro).0, oscuro.success);
        assert_eq!(colores_devolucion(false, oscuro).0, oscuro.danger);
        assert_eq!(colores_devolucion(true, claro).0, claro.success);
        assert_eq!(colores_devolucion(false, claro).0, claro.danger);
        for tema in [oscuro, claro] {
            // El cuerpo/sugerencia salen del tema (cableado que el AA mide).
            assert_eq!(colores_texto_devolucion(tema).0, tema.text_primary);
            assert_eq!(colores_texto_devolucion(tema).1, tema.text_secondary);
        }
        assert!(razon_contraste(oscuro.keyboard_enter_text, oscuro.accent) >= 4.5);
        assert!(razon_contraste(claro.keyboard_enter_text, claro.accent) >= 4.5);
    }

    fn pista_de_sesion(texto: &str) -> String {
        let sesion = TeachingSession::for_topic(texto);
        sesion
            .steps
            .first()
            .map(|s| s.whiteboard_hint.clone())
            .unwrap_or_default()
    }

    fn frame_solido(color: egui::Color32) -> egui::ColorImage {
        egui::ColorImage::new([4, 4], color)
    }

    #[test]
    fn ensure_reemplazo_retira_con_gracia_submit_en_vuelo() {
        // Regresión `egui_texid_Managed(N) has been destroyed`: instalar la
        // 2ª animación no destruye las texturas de la 1ª en el mismo frame.
        let ctx = egui::Context::default();
        let mut estado = TeachingUiState {
            anim_frames: Some(vec![frame_solido(egui::Color32::RED)]),
            ..Default::default()
        };
        estado.ensure_textures(&ctx);
        assert_eq!(estado.anim_textures.len(), 1);
        assert_eq!(estado.retired_anim_textures.pending(), 0);
        // 2ª animación (derivada tras integral): el set viejo se retira.
        estado.anim_frames = Some(vec![frame_solido(egui::Color32::BLUE)]);
        estado.ensure_textures(&ctx);
        assert_eq!(estado.anim_textures.len(), 1);
        assert_eq!(
            estado.retired_anim_textures.pending(),
            1,
            "set viejo retenido, no destruido con submit en vuelo"
        );
        for _ in 0..TEXTURE_GRACE_FRAMES.saturating_sub(1) {
            estado.reap_retired_anim_textures();
            assert_eq!(
                estado.retired_anim_textures.pending(),
                1,
                "en-vuelo intacto durante la gracia"
            );
        }
        estado.reap_retired_anim_textures();
        assert_eq!(estado.retired_anim_textures.pending(), 0);
        assert_eq!(estado.anim_textures.len(), 1, "el set nuevo intacto");
    }

    #[test]
    fn clear_retira_y_avance_no_dropea_post_draw() {
        // `clear` / `advance` retiran (no dropean): el draw ya emitido en
        // este frame sigue referenciando texturas válidas hasta la gracia.
        let ctx = egui::Context::default();
        let mut estado = TeachingUiState {
            anim_frames: Some(vec![
                frame_solido(egui::Color32::WHITE),
                frame_solido(egui::Color32::WHITE),
            ]),
            ..Default::default()
        };
        estado.ensure_textures(&ctx);
        assert_eq!(estado.anim_textures.len(), 2);
        estado.clear_anim_textures_only(None);
        assert!(estado.anim_textures.is_empty());
        assert_eq!(estado.retired_anim_textures.pending(), 2);
        for _ in 0..TEXTURE_GRACE_FRAMES {
            estado.reap_retired_anim_textures();
        }
        assert_eq!(estado.retired_anim_textures.pending(), 0);
    }

    #[test]
    fn flicker_rellamada_sin_cambios_no_realoca() {
        // FLICKER: un tick = un frame dibujado y `draw_teaching_overlay`
        // llama a `ensure_textures` en cada uno. Sin frames nuevos no debe
        // re-subir nada: mismos handles, nada retirado (re-subir por frame
        // parpadearía y quemaría GPU).
        let ctx = egui::Context::default();
        let mut estado = TeachingUiState {
            anim_frames: Some(vec![
                frame_solido(egui::Color32::RED),
                frame_solido(egui::Color32::GREEN),
            ]),
            ..Default::default()
        };
        estado.ensure_textures(&ctx);
        assert_eq!(estado.anim_textures.len(), 2);
        let ids_antes: Vec<egui::TextureId> = estado.anim_textures.iter().map(|t| t.id()).collect();
        for _ in 0..8 {
            estado.ensure_textures(&ctx);
        }
        let ids_despues: Vec<egui::TextureId> =
            estado.anim_textures.iter().map(|t| t.id()).collect();
        assert_eq!(ids_antes, ids_despues, "ni un handle cambia por frame");
        assert_eq!(
            estado.retired_anim_textures.pending(),
            0,
            "nada retirado sin reemplazo"
        );
    }

    #[test]
    fn ensure_10_reemplazos_sin_dibujar_acota_retired() {
        // M3-7: 10× `ensure` con frames nuevos y SIN `tick` (sin draw):
        // cada reemplazo retira el set viejo pero `pending()` nunca supera
        // el tope y el set visible sigue siendo el último.
        use crate::anim_ui::RETENTION_MAX_PENDING;
        let ctx = egui::Context::default();
        let mut estado = TeachingUiState {
            anim_frames: Some(vec![frame_solido(egui::Color32::RED)]),
            ..Default::default()
        };
        estado.ensure_textures(&ctx);
        assert_eq!(estado.retired_anim_textures.pending(), 0);
        for ronda in 0..10u8 {
            estado.anim_frames = Some(vec![frame_solido(egui::Color32::from_rgb(
                ronda * 25,
                100,
                200,
            ))]);
            estado.ensure_textures(&ctx);
            assert_eq!(estado.anim_textures.len(), 1);
            assert!(
                estado.retired_anim_textures.pending() <= RETENTION_MAX_PENDING,
                "ronda {ronda}: pending sin cota"
            );
        }
        assert_eq!(
            estado.retired_anim_textures.pending(),
            10,
            "10 reemplazos sin tick = 10 retenidos (bajo el tope 96)"
        );
        // Y la gracia los drena sin dibujar de más.
        for _ in 0..TEXTURE_GRACE_FRAMES {
            estado.reap_retired_anim_textures();
        }
        assert_eq!(estado.retired_anim_textures.pending(), 0);
    }

    #[test]
    fn hash_esquinas_y_centro_detecta_reemplazo_con_cota_o1() {
        // Frente hash O(píxeles): la huella solo mira len+size+4 esquinas+
        // centro por frame — el tick por frame dibujado no recorre píxeles.
        // Auditoría: el centro SE muestrea (el contenido vive ahí); el resto
        // del interior sigue fuera por la cota O(1).
        fn huella(frames: Vec<egui::ColorImage>) -> u64 {
            TeachingUiState {
                anim_frames: Some(frames),
                ..Default::default()
            }
            .anim_frames_hash()
        }
        let mut grande_a = egui::ColorImage::new([64, 64], egui::Color32::RED);
        let mut solo_interior = grande_a.clone();
        // Cota: píxeles interiores distintos fuera del centro, mismas
        // esquinas+centro → la huella NO los mira (trabajo acotado, no O(n)).
        for (i, pixel) in solo_interior.pixels.iter_mut().enumerate() {
            let w = 64;
            let (x, y) = (i % w, i / w);
            let es_esquina = (x == 0 || x == w - 1) && (y == 0 || y == w - 1);
            let es_centro = (x, y) == (w / 2, w / 2);
            if !es_esquina && !es_centro {
                *pixel = egui::Color32::GREEN;
            }
        }
        assert_eq!(
            huella(vec![grande_a.clone()]),
            huella(vec![solo_interior.clone()]),
            "el interior no muestreado no entra en la huella (cota O(1) por frame)"
        );
        // Mutar el centro SÍ cambia la huella (contenido real, no esquinas).
        let mut centro_mutado = grande_a.clone();
        centro_mutado.pixels[(64 / 2) * 64 + (64 / 2)] = egui::Color32::BLUE;
        assert_ne!(
            huella(vec![grande_a.clone()]),
            huella(vec![centro_mutado]),
            "mutar el centro debe cambiar la huella"
        );
        // Reemplazo real (esquina distinta) sí cambia la huella.
        grande_a.pixels[0] = egui::Color32::BLUE;
        assert_ne!(
            huella(vec![grande_a]),
            huella(vec![solo_interior]),
            "el reemplazo debe invalidar el cache"
        );
        // Largo distinto también (set nuevo, no re-subida tardía).
        assert_ne!(
            huella(vec![frame_solido(egui::Color32::RED)]),
            huella(vec![
                frame_solido(egui::Color32::RED),
                frame_solido(egui::Color32::RED)
            ]),
            "distinto largo, distinta huella"
        );
    }

    // M3-8: los tests del dispatch tipado muerto se borraron con las fns;
    // el núcleo vivo se cubre en `hint_legacy_*` + `tests.rs::whiteboard_*`.
    #[test]
    fn hint_legacy_derivada_da_secante_e_integral_da_area() {
        assert_eq!(
            hint_for_topic(&pista_de_sesion("derivada de x^2")),
            WhiteboardHint::Secante
        );
        assert_eq!(
            hint_for_topic(&pista_de_sesion("integral de x^2")),
            WhiteboardHint::Area
        );
    }

    #[test]
    fn hint_legacy_limite_funcion_pitagoras() {
        assert_eq!(
            hint_for_topic("Recta con hueco en a"),
            WhiteboardHint::Limite
        );
        assert_eq!(
            hint_for_topic("Ejes con puntos (x, f(x))"),
            WhiteboardHint::Funcion
        );
        assert_eq!(
            hint_for_topic("Triángulo rectángulo con cuadrados en cada lado"),
            WhiteboardHint::Pitagoras
        );
    }

    #[test]
    fn hint_legacy_fraccion_vector_matriz() {
        assert_eq!(
            hint_for_topic("Rectángulo dividido en partes"),
            WhiteboardHint::Fraccion
        );
        assert_eq!(hint_for_topic("Flecha en ejes R²"), WhiteboardHint::Vector);
        assert_eq!(hint_for_topic("Grilla 2x2"), WhiteboardHint::Matriz);
    }

    #[test]
    fn hint_legacy_probabilidad_serie_trigonometria() {
        assert_eq!(
            hint_for_topic("Diagrama de árbol"),
            WhiteboardHint::Probabilidad
        );
        assert_eq!(
            hint_for_topic("Suma parcial que se aproxima"),
            WhiteboardHint::Serie
        );
        assert_eq!(
            hint_for_topic("Círculo unitario con ángulo"),
            WhiteboardHint::Trigonometria
        );
    }

    #[test]
    fn hint_legacy_conica_ecuacion_libre_general() {
        assert_eq!(hint_for_topic("Elipse con focos"), WhiteboardHint::Conica);
        assert_eq!(
            hint_for_topic("Parábola y raíces"),
            WhiteboardHint::Ecuacion
        );
        assert_eq!(
            hint_for_topic("Pizarra libre para trazar"),
            WhiteboardHint::Libre
        );
        assert!(hint_for_topic("").is_vacio_o_libre());
        assert_eq!(hint_for_topic("origami de papel"), WhiteboardHint::General);
    }

    #[test]
    fn hint_pitagoras_tiene_prioridad_sobre_area() {
        // "triángulo rectángulo" contiene "rectángulo": debe ganar Pitágoras.
        assert_eq!(
            hint_for_topic("triángulo rectángulo con cuadrados"),
            WhiteboardHint::Pitagoras
        );
    }

    #[test]
    fn elementos_para_hint_vacio_y_libre_no_dibujan() {
        assert!(whiteboard_elements_for_hint("").is_empty());
        assert!(whiteboard_elements_for_hint("   ").is_empty());
        assert!(whiteboard_elements_for_hint("Pizarra libre").is_empty());
    }

    #[test]
    fn morph_suave_arranca_lento_y_termina_en_uno() {
        assert_eq!(progreso_morph_suave(0.0), 0.0);
        assert_eq!(progreso_morph_suave(1.0), 1.0);
        let medio = progreso_morph_suave(0.5);
        assert!(medio > 0.5 && medio < 1.0);
    }

    #[test]
    fn burbuja_nunca_se_pasa_del_limite() {
        let limite = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(300.0, 200.0));
        let origen = egui::Rect::from_min_max(egui::pos2(10.0, 10.0), egui::pos2(60.0, 30.0));
        let destino = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(900.0, 800.0));
        for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let r = rect_burbuja_morph(origen, destino, t, limite);
            assert!(limite.contains_rect(r), "se pasó en t={t}: {r:?}");
        }
    }

    #[test]
    fn acotar_no_rompe_utf8_y_opciones_son_cuatro() {
        assert!(acotar_texto("áéíóú", 3).chars().count() <= 3);
        let ejercicio = Exercise {
            prompt: "Si f(x)=2*x+1, evalúa en x=3".into(),
            solution: "7".into(),
            kind: grafito_pedagogy::ExerciseKind::Numeric,
            difficulty: grafito_pedagogy::ExerciseDifficulty::Medium,
            lo_id: "x".into(),
            params: std::collections::BTreeMap::new(),
            seed: Some(1),
            validator: grafito_pedagogy::ValidatorKind::NumericTol(0.02),
        };
        let ops = opciones_para_ejercicio(&ejercicio);
        assert_eq!(ops.len(), 4);
        assert!(ops.iter().any(|o| o.trim() == "7"));
        assert!(indice_respuesta_correcta(&ejercicio, &ops).is_some());
    }

    // ── B7: panel de ejercicio con estados ──

    #[test]
    fn b7_resolver_lo_exactoy_fuzzy() {
        let por_id = resolver_lo("am1-der").expect("am1-der existe");
        assert_eq!(por_id.id, "am1-der");
        let fuzzy = resolver_lo("derivada").expect("derivada resuelve");
        assert_eq!(fuzzy.id, "am1-der");
        // Con tilde también (reintento sin tildes).
        let tilde = resolver_lo("función").expect("función resuelve");
        assert!(!tilde.id.is_empty());
    }

    #[test]
    fn b7_resolver_lo_vacio_y_desconocido_dan_error_honesto() {
        assert!(resolver_lo("").is_err());
        assert!(resolver_lo("   ").is_err());
        let err = resolver_lo("origami de papel xyz").expect_err("sin LO");
        assert!(err.contains("origami de papel xyz"));
    }

    #[test]
    fn b7_armar_ejercicio_valido_y_acotado() {
        let ejercicio =
            armar_ejercicio("derivada", grafito_pedagogy::PedagogicalLevel::Secondary, 7)
                .expect("ejercicio válido");
        assert!(ejercicio.validate().is_ok());
        assert!(!ejercicio.prompt.trim().is_empty());
        assert!(!ejercicio.solution.trim().is_empty());
        let ops = opciones_para_ejercicio(&ejercicio);
        assert_eq!(ops.len(), 4);
        assert!(indice_respuesta_correcta(&ejercicio, &ops).is_some());
    }

    #[test]
    fn b7_pedir_genera_en_background_y_lista() {
        let mut panel = PanelEjercicio::default();
        assert_eq!(panel.estado, EstadoPanelEjercicio::Vacio);
        panel.pedir("derivada", grafito_pedagogy::PedagogicalLevel::Secondary);
        assert_eq!(panel.estado, EstadoPanelEjercicio::Generando);
        // El job es CPU pura: converge en pocos ms (cota 2s anti-flake).
        let mut lista = false;
        for _ in 0..400 {
            if panel.sondear() && panel.estado == EstadoPanelEjercicio::Lista {
                lista = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert!(lista, "el job debió listar, estado={:?}", panel.estado);
        assert_eq!(panel.opciones.len(), 4);
        assert!(panel.ejercicio.is_some());
    }

    #[test]
    fn b7_pedir_tema_vacio_da_error_y_cerrar_resetea() {
        let mut panel = PanelEjercicio::default();
        panel.pedir("   ", grafito_pedagogy::PedagogicalLevel::Secondary);
        assert_eq!(panel.estado, EstadoPanelEjercicio::Error);
        assert!(panel.error.is_some());
        panel.cerrar();
        assert_eq!(panel.estado, EstadoPanelEjercicio::Vacio);
        assert!(panel.error.is_none());
        assert!(panel.ejercicio.is_none());
    }

    #[test]
    fn b7_otro_ejercicio_cambia_semilla() {
        let mut panel = PanelEjercicio::default();
        panel.pedir("derivada", grafito_pedagogy::PedagogicalLevel::Secondary);
        let primera = panel.semilla;
        panel.pedir("derivada", grafito_pedagogy::PedagogicalLevel::Secondary);
        assert_ne!(panel.semilla, primera);
        panel.cerrar();
    }

    trait VacioOlibre {
        fn is_vacio_o_libre(&self) -> bool;
    }
    impl VacioOlibre for WhiteboardHint {
        fn is_vacio_o_libre(&self) -> bool {
            matches!(self, WhiteboardHint::Vacio | WhiteboardHint::Libre)
        }
    }

    // ── Frente E1: overlay sin marcas crudas ni duplicación ──

    #[test]
    fn e1_etiqueta_paso_es_uno_based_y_real() {
        assert_eq!(etiqueta_paso(0, 3), "Paso 1 de 3");
        assert_eq!(etiqueta_paso(2, 3), "Paso 3 de 3");
        assert_eq!(etiqueta_paso(0, 0), "Paso 1 de 1");
    }

    #[test]
    fn e1_titulo_limpio_pela_heading_y_negrita() {
        assert_eq!(titulo_limpio("## Enfoque"), "Enfoque");
        assert_eq!(titulo_limpio("# Concepto"), "Concepto");
        assert_eq!(titulo_limpio("**Concepto**"), "Concepto");
        assert_eq!(
            titulo_limpio("Vamos a desglosar: ## Enfoque"),
            "Vamos a desglosar: Enfoque"
        );
        // Contenido legítimo intacto: `#` sin forma de marca no se toca.
        assert_eq!(titulo_limpio("C# básico"), "C# básico");
        assert_eq!(titulo_limpio("Función f(x)=x²"), "Función f(x)=x²");
        // Vacío honesto: nunca título en blanco.
        assert_eq!(titulo_limpio("##"), "Paso");
        assert_eq!(titulo_limpio("   "), "Paso");
    }

    #[test]
    fn e1_texto_plano_inline_no_rompe_utf8() {
        let limpio = texto_plano_inline("**áéíóú** `código` ## título");
        assert!(!limpio.contains("**"));
        assert!(!limpio.contains('`'));
        assert!(!limpio.contains('#'));
        assert!(limpio.contains("áéíóú"));
    }

    #[test]
    fn e1_estado_pizarra_oculta_placeholder_lienzo() {
        // Sin hint y sin trazos: nada (ni recuadro negro vacío).
        assert_eq!(estado_pizarra("", true), EstadoPizarra::Oculta);
        assert_eq!(estado_pizarra("   ", true), EstadoPizarra::Oculta);
        // Hint con dibujo útil: lienzo (secante trae trazos y flechas).
        assert_eq!(
            estado_pizarra("Secante que colapsa a tangente", true),
            EstadoPizarra::Lienzo
        );
        // Hint que sólo da texto de fallback: placeholder honesto.
        assert_eq!(
            estado_pizarra("Pizarra para explorar", true),
            EstadoPizarra::Placeholder
        );
        // Trazo del usuario siempre muestra lienzo aunque el hint sea pobre.
        assert_eq!(estado_pizarra("", false), EstadoPizarra::Lienzo);
        assert_eq!(
            estado_pizarra("Pizarra para explorar", false),
            EstadoPizarra::Lienzo
        );
    }

    #[test]
    fn e1_pizarra_tiene_dibujo_distigue_fallback() {
        assert!(pizarra_tiene_dibujo("Secante que colapsa a tangente"));
        assert!(pizarra_tiene_dibujo("Recta con hueco en a"));
        assert!(!pizarra_tiene_dibujo("Pizarra para explorar"));
        assert!(!pizarra_tiene_dibujo(""));
    }

    #[test]
    fn e1_bloques_parsean_heading_sin_marcas_crudas() {
        // Misma gramática del transcript: "## Enfoque" es Heading, no texto.
        let mut cache = AssistantBlocksCache::default();
        let bloques = cache.blocks("## Enfoque\n\nTexto plano");
        assert_eq!(bloques.len(), 2);
        assert!(matches!(
            bloques.first(),
            Some(AssistantMessageBlock::Heading { level: 2, text })
            if text == "Enfoque"
        ));
        assert!(matches!(
            bloques.get(1),
            Some(AssistantMessageBlock::Paragraph(_))
        ));
    }

    #[test]
    fn e1_retroceder_clampea_en_cero_y_vuelve() {
        let mut estado = TeachingUiState::default();
        estado.start("derivada de x²");
        assert!(!estado.retroceder(), "en el primer paso no hay atrás");
        assert_eq!(estado.session.as_ref().map(|s| s.current), Some(0));
        assert!(estado.advance());
        assert_eq!(estado.session.as_ref().map(|s| s.current), Some(1));
        assert!(estado.retroceder());
        assert_eq!(estado.session.as_ref().map(|s| s.current), Some(0));
        assert!(!estado.retroceder(), "otra vez en cero: clampa");
    }

    #[test]
    fn e1_draw_bloques_no_paniquea_headless() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let mut cache = AssistantBlocksCache::default();
                draw_bloques_markdown(
                    ui,
                    "## Enfoque\n\nVamos a desglosar: **derivada**\n\n- paso uno\n- paso dos\n\n> cita honesta\n\n1. primero\n2. segundo",
                    &mut cache,
                    grafito_ui::i18n::Locale::Es,
                );
                draw_bloques_markdown(ui, "", &mut cache, grafito_ui::i18n::Locale::Es);
            });
        });
    }

    #[test]
    fn b_a70_draw_bloques_math_respeta_locale_sin_panico() {
        // B-A70: el DisplayMath del transcript sale por `draw_math_with_locale`
        // (aviso `assistant.math.tex_simplified` en el locale dado); Es y En
        // dibujan sin pánico headless.
        for locale in [grafito_ui::i18n::Locale::Es, grafito_ui::i18n::Locale::En] {
            let ctx = egui::Context::default();
            let _ = ctx.run(egui::RawInput::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let mut cache = AssistantBlocksCache::default();
                    draw_bloques_markdown(ui, "$x^2$", &mut cache, locale);
                });
            });
        }
        // Pin de wiring: ambas fns exponen `locale` y el DisplayMath sale
        // por el renderer localizado. (El conteo incluye este propio pin,
        // que contiene el literal una vez: prod 2 + pin 1 = 3; B-A73 suma
        // `draw_tarjeta_ejercicio` + `draw_panel_ejercicio` = 5; B-A74 suma
        // el helper puro `exercise_header_text` = 6; B-A75 suma
        // `etiqueta_paso_for` + `titulo_limpio_for` +
        // `opciones_para_ejercicio_for` + `sondear_con` = 10; B-A76 suma
        // `resolver_lo_for` + `armar_ejercicio_for` + `pedir_con` = 13;
        // B-A78 suma `exercise_building_toast_text` +
        // `exercise_step_hint_text` = 15; B-A79 suma
        // `overlay_anim_reserve_text` = 16.)
        let fuente = include_str!("teaching_ui.rs");
        assert_eq!(
            fuente
                .match_indices("locale: grafito_ui::i18n::Locale")
                .count(),
            16,
            "draw_bloques_markdown + draw_teaching_overlay + tarjeta/panel + job exponen locale (15 prod + 1 este pin)"
        );
        assert!(
            fuente.contains("draw_math_with_locale(ui, math, locale)"),
            "DisplayMath del transcript cableado al renderer localizado"
        );
        assert!(
            fuente.contains("draw_math_with_locale(ui, expr, locale)"),
            "paso verificado del overlay cableado al renderer localizado"
        );
    }

    #[test]
    fn b_a71_teaching_marcos_y_pizarra_localizan_sin_panico() {
        // B-A71: los 5 literales ES de marcos/pizarra salen por `teach_msg`
        // en el `locale` dado; Es y En dibujan sin pánico headless.
        for locale in [grafito_ui::i18n::Locale::Es, grafito_ui::i18n::Locale::En] {
            let ctx = egui::Context::default();
            let _ = ctx.run(egui::RawInput::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let mut cache = AssistantBlocksCache::default();
                    draw_bloques_markdown(
                        ui,
                        "```python\nx = 1\n```\n\n| a | b |\n|---|---|\n| 1 | 2 |\n",
                        &mut cache,
                        locale,
                    );
                });
            });
        }
        // ES pineado al copy previo (vía `teach_msg`, sin literales duros).
        assert_eq!(
            teach_msg("code_block_title", grafito_ui::i18n::Locale::Es),
            "C\u{f3}digo"
        );
        assert_eq!(
            teach_msg("table_block_title", grafito_ui::i18n::Locale::Es),
            "Tabla"
        );
        assert_eq!(
            teach_msg("verified_expr", grafito_ui::i18n::Locale::Es),
            "Expresi\u{f3}n verificada"
        );
        // Pin de wiring: los 5 sitios usan `teach_msg` con `locale`.
        // (Búsqueda por ventana de 80 chars para tolerar el fmt
        // multilínea; agujas partidas para no auto-matchear este pin
        // vía `include_str!`, precedente B-A70 E0502/pins autorreferentes.)
        let fuente = include_str!("teaching_ui.rs");
        for clave in [
            "code_block_title",
            "table_block_title",
            "verified_expr",
            "board_ready_hint",
            "board_title",
        ] {
            let mut cableado = false;
            let mut inicio = 0;
            while let Some(rel) = fuente[inicio..].find("teach_msg(") {
                let abs = inicio + rel;
                let fin = (abs + 200).min(fuente.len());
                if fuente[abs..fin].contains(clave) && fuente[abs..fin].contains("locale") {
                    cableado = true;
                    break;
                }
                inicio = abs + 10;
            }
            assert!(cableado, "sitio teaching sin cablear a teach_msg: {clave}");
        }
        // Blindaje negativo: ningún call-site prod conserva el literal ES
        // duro (agujas partidas para no auto-matchear este propio test).
        let cod = ["C", "\u{f3}digo"].concat();
        let verif = ["Expresi", "\u{f3}n verificada"].concat();
        for aguja in [
            ["marco_monoespaciado(ui, \"", &cod, "\")"].concat(),
            ["RichText::new(\"", &verif, "\")"].concat(),
        ] {
            assert!(
                !fuente.contains(&aguja),
                "literal ES duro aún en teaching_ui: {aguja}"
            );
        }
    }

    #[test]
    fn b_a73_teaching_ejercicio_overlay_localiza_sin_panico() {
        // B-A73: los 7 literales ES de ejercicio + overlay nav salen por
        // `teach_msg` en el `locale` dado; Es y En dibujan sin pánico headless.
        for locale in [grafito_ui::i18n::Locale::Es, grafito_ui::i18n::Locale::En] {
            let ctx = egui::Context::default();
            let _ = ctx.run(egui::RawInput::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    // Panel vacío no dibuja nada; Generando ejercita
                    // `exercise_building` + spinner sin Exercise.
                    let mut vacio = PanelEjercicio::default();
                    let _ = draw_panel_ejercicio(ui, &mut vacio, locale);
                    let mut generando = PanelEjercicio {
                        estado: EstadoPanelEjercicio::Generando,
                        ..Default::default()
                    };
                    let _ = draw_panel_ejercicio(ui, &mut generando, locale);
                });
            });
        }
        // ES pineado al copy previo (vía `teach_msg`, sin literales duros).
        assert_eq!(
            teach_msg("exercise_answer_label", grafito_ui::i18n::Locale::Es),
            "Tu respuesta:"
        );
        assert_eq!(
            teach_msg("exercise_respond", grafito_ui::i18n::Locale::Es),
            "Responder"
        );
        assert_eq!(
            teach_msg("exercise_building", grafito_ui::i18n::Locale::Es),
            "Armando tu ejercicio…"
        );
        assert_eq!(
            teach_msg("overlay_close", grafito_ui::i18n::Locale::Es),
            "Cerrar"
        );
        // Pin de wiring: los 7 sitios usan `teach_msg` con `locale`.
        let fuente = include_str!("teaching_ui.rs");
        for clave in [
            "exercise_answer_label",
            "exercise_respond",
            "exercise_building",
            "overlay_generating_manim",
            "overlay_gen_detail",
            "overlay_back",
            "overlay_close",
        ] {
            let mut cableado = false;
            let mut inicio = 0;
            while let Some(rel) = fuente[inicio..].find("teach_msg(") {
                let abs = inicio + rel;
                let fin = (abs + 200).min(fuente.len());
                if fuente[abs..fin].contains(clave) && fuente[abs..fin].contains("locale") {
                    cableado = true;
                    break;
                }
                inicio = abs + 10;
            }
            assert!(cableado, "sitio teaching sin cablear a teach_msg: {clave}");
        }
        // Blindaje negativo: los 7 literales ES ya no están en call-sites prod
        // (agujas partidas para no auto-matchear este propio test).
        let resp = ["Tu resp", "uesta:"].concat();
        let arm = ["Armando tu ejer", "cicio…"].concat();
        let manim = ["Generando animaci", "ón con Manim…"].concat();
        let detalle = ["Detalle de generaci", "ón"].concat();
        for aguja in [
            ["RichText::new(\"", &resp, "\")"].concat(),
            ["RichText::new(\"", &arm, "\")"].concat(),
            ["RichText::new(\"", &manim, "\")"].concat(),
            ["RichText::new(\"", &detalle, "\")"].concat(),
        ] {
            assert!(
                !fuente.contains(&aguja),
                "literal ES duro aún en teaching_ui: {aguja}"
            );
        }
    }

    #[test]
    fn b_a74_teaching_feedback_ejercicio_overlay_localiza_sin_panico() {
        // B-A74: los 14 literales ES de feedback + ejercicio resto + overlay
        // nav/hovers salen por `teach_msg` en el `locale` dado; Es y En
        // dibujan sin pánico headless (tarjeta con devolución Ok/Err +
        // panel en Error/Lista + header con tema).
        for locale in [grafito_ui::i18n::Locale::Es, grafito_ui::i18n::Locale::En] {
            let ctx = egui::Context::default();
            let _ = ctx.run(egui::RawInput::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    // Header puro con tema interpolado en ambos idiomas.
                    let header = exercise_header_text("Álgebra", locale);
                    assert!(header.contains("Álgebra"), "header sin tema en {locale:?}");
                    // Panel en Error ejercita `exercise_error_generic` +
                    // `exercise_retry` + `overlay_close`.
                    let mut error = PanelEjercicio {
                        estado: EstadoPanelEjercicio::Error,
                        error: None,
                        ..Default::default()
                    };
                    let _ = draw_panel_ejercicio(ui, &mut error, locale);
                });
            });
        }
        // ES pineado al copy previo (vía `teach_msg`, sin literales duros).
        assert_eq!(
            teach_msg("exercise_feedback_ok", grafito_ui::i18n::Locale::Es),
            "¡Bien ahí, che! Respuesta correcta."
        );
        assert_eq!(
            teach_msg("exercise_error_generic", grafito_ui::i18n::Locale::Es),
            "Algo falló, probá de nuevo, che."
        );
        assert_eq!(
            teach_msg("overlay_title", grafito_ui::i18n::Locale::Es),
            "Enseñanza — Paso a paso"
        );
        assert_eq!(
            exercise_header_text("Álgebra", grafito_ui::i18n::Locale::Es),
            "Ejercicio · Álgebra"
        );
        assert_eq!(
            exercise_header_text("Álgebra", grafito_ui::i18n::Locale::En),
            "Exercise · Álgebra"
        );
        // Pin de wiring: los 14 sitios usan `teach_msg` con `locale`
        // (el header va por el helper `exercise_header_text` + `teach_msg`
        // con `locale` dentro; el close-icon reusa `overlay_close`).
        let fuente = include_str!("teaching_ui.rs");
        for clave in [
            "exercise_feedback_ok",
            "exercise_feedback_retry",
            "exercise_error_generic",
            "exercise_error_lost",
            "exercise_retry",
            "exercise_other",
            "exercise_other_hint",
            "exercise_header",
            "overlay_title",
            "overlay_next",
            "overlay_finish",
            "overlay_next_hint",
            "overlay_finish_hint",
            "overlay_back_hint",
        ] {
            let mut cableado = false;
            let mut inicio = 0;
            while let Some(rel) = fuente[inicio..].find("teach_msg(") {
                let abs = inicio + rel;
                let fin = (abs + 200).min(fuente.len());
                if fuente[abs..fin].contains(clave) && fuente[abs..fin].contains("locale") {
                    cableado = true;
                    break;
                }
                inicio = abs + 10;
            }
            // `exercise_header` solo aparece en el helper + su test; el
            // helper lleva `locale` en la misma ventana de 200 chars.
            if !cableado && clave == "exercise_header" {
                cableado = fuente.contains("teach_msg(\"exercise_header\", locale)");
            }
            assert!(cableado, "sitio teaching sin cablear a teach_msg: {clave}");
        }
        // Blindaje negativo: ningún call-site prod conserva el ES duro.
        // Agujas con forma de call-site (prefijo `button(` / `Window::new(`
        // / `on_hover_text(` / `format!(` + literal partido) para no
        // auto-matchear los pines ES de arriba (precedente B-A73). Los 4
        // literales del `let titulo` + errores van por conteo con la aguja
        // partida (el `match_indices` directo se contaría a sí mismo):
        // pin ES de arriba 1 + prod 0 = 1, resto 0 (precedente B-A70).
        let bien = ["¡Bien ahí, che! Resp", "uesta correcta."].concat();
        let casi = ["Casi… fijate", " de nuevo."].concat();
        let fallo = ["Algo falló, probá de", " nuevo, che."].concat();
        let perdido = ["Se perdió el ejer", "cicio, pedí otro, che."].concat();
        for (agujita, esperadas) in [
            (bien.as_str(), 1),
            (casi.as_str(), 0),
            (fallo.as_str(), 1),
            (perdido.as_str(), 0),
        ] {
            assert_eq!(
                fuente.match_indices(agujita).count(),
                esperadas,
                "literal ES con ocurrencias inesperadas: {agujita}"
            );
        }
        let titulo = ["Enseñanza — Paso", " a paso"].concat();
        for aguja in [
            ["Window::new(\"", &titulo, "\")"].concat(),
            ["button(\"Otro ejer", "cicio\")"].concat(),
            ["button(\"Rein", "tentar\")"].concat(),
            ["format!(\"Ejercicio · ", "{}\", panel.tema)"].concat(),
            ["on_hover_text(\"Genero uno nuevo", " del mismo tema\")"].concat(),
            ["on_hover_text(\"Vuelve al paso", " anterior\")"].concat(),
            ["on_hover_text(\"Cierra la ense", "ñanza\")"].concat(),
            ["on_hover_text(\"Avanza al siguiente", " paso\")"].concat(),
        ] {
            assert!(
                !fuente.contains(&aguja),
                "literal ES duro aún en teaching_ui: {aguja}"
            );
        }
    }

    #[test]
    fn b_a75_teaching_progreso_distractores_localiza_sin_panico() {
        // B-A75: progreso + distractores salen por `teach_msg` en el `locale`
        // dado; Es y En dibujan sin pánico headless (panel Lista con
        // distractores + helpers puros con templates).
        for locale in [grafito_ui::i18n::Locale::Es, grafito_ui::i18n::Locale::En] {
            let ctx = egui::Context::default();
            let _ = ctx.run(egui::RawInput::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    // Helpers puros con templates en ambos idiomas.
                    let etiqueta = etiqueta_paso_for(0, 3, locale);
                    assert!(!etiqueta.is_empty(), "etiqueta vacía en {locale:?}");
                    let fallback = titulo_limpio_for("##", locale);
                    assert!(!fallback.is_empty(), "fallback vacío en {locale:?}");
                    // Panel Lista ejercita `sondear_con` + distractores.
                    let ejercicio = Exercise {
                        prompt: "Derivada de x^2".into(),
                        solution: "2*x".into(),
                        kind: grafito_pedagogy::ExerciseKind::Symbolic,
                        difficulty: grafito_pedagogy::ExerciseDifficulty::Medium,
                        lo_id: "x".into(),
                        params: std::collections::BTreeMap::new(),
                        seed: Some(2),
                        validator: grafito_pedagogy::ValidatorKind::Exact,
                    };
                    let ops = opciones_para_ejercicio_for(&ejercicio, locale);
                    assert_eq!(ops.len(), 4, "opciones != 4 en {locale:?}");
                    let mut lista = PanelEjercicio {
                        estado: EstadoPanelEjercicio::Lista,
                        ejercicio: Some(ejercicio),
                        opciones: ops,
                        ..Default::default()
                    };
                    let _ = draw_panel_ejercicio(ui, &mut lista, locale);
                });
            });
        }
        // ES pineado al copy previo (canónicos intactos vía wrappers Es).
        assert_eq!(etiqueta_paso(0, 3), "Paso 1 de 3");
        assert_eq!(
            etiqueta_paso_for(0, 3, grafito_ui::i18n::Locale::Es),
            "Paso 1 de 3"
        );
        assert_eq!(
            etiqueta_paso_for(0, 3, grafito_ui::i18n::Locale::En),
            "Step 1 of 3"
        );
        assert_eq!(titulo_limpio("##"), "Paso");
        assert_eq!(
            titulo_limpio_for("##", grafito_ui::i18n::Locale::En),
            "Step"
        );
        // Pin de wiring: los 4 sitios usan `teach_msg` con `locale`
        // (helpers `_for` + `sondear_con`).
        let fuente = include_str!("teaching_ui.rs");
        for clave in [
            "progress_step",
            "step_fallback",
            "exercise_none_of_these",
            "exercise_variant",
        ] {
            let mut cableado = false;
            let mut inicio = 0;
            while let Some(rel) = fuente[inicio..].find("teach_msg(") {
                let abs = inicio + rel;
                let fin = (abs + 200).min(fuente.len());
                if fuente[abs..fin].contains(clave) && fuente[abs..fin].contains("locale") {
                    cableado = true;
                    break;
                }
                inicio = abs + 10;
            }
            assert!(cableado, "sitio teaching sin cablear a teach_msg: {clave}");
        }
        // Blindaje negativo: los literales ES ya no están en call-sites prod
        // (agujas partidas para no auto-matchear este propio test vía
        // `include_str!`, precedente B-A70/B-A73).
        let ninguna = ["candidatas.push(\"Ninguna de", " estas\".to_string())"].concat();
        let variante = ["format!(\"Variante {}\", candidatas.len()", " + 1)"].concat();
        let paso_fb = ["\"Pa", "so\".to_string()"].concat();
        for aguja in [&ninguna, &variante, &paso_fb] {
            assert!(
                !fuente.contains(aguja.as_str()),
                "literal ES duro aún en teaching_ui: {aguja}"
            );
        }
    }

    #[test]
    fn b_a76_teaching_errores_job_localizan_sin_panico() {
        // B-A76: los 5 errores del job salen por `teach_msg` en el `locale`
        // dado; Es y En dibujan sin pánico headless (panel Error con los 5
        // mensajes + helpers puros con templates).
        for locale in [grafito_ui::i18n::Locale::Es, grafito_ui::i18n::Locale::En] {
            // Helpers puros con templates en ambos idiomas.
            let vacio = resolver_lo_for("", locale);
            assert!(vacio.is_err(), "tema vacío sin error en {locale:?}");
            let perdido = resolver_lo_for("origami de papel xyz", locale);
            assert!(perdido.is_err(), "tema sin LO sin error en {locale:?}");
            // Panel Error ejercita el display del mensaje ya localizado.
            let ctx = egui::Context::default();
            let _ = ctx.run(egui::RawInput::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let mut error = PanelEjercicio {
                        estado: EstadoPanelEjercicio::Error,
                        error: Some(
                            grafito_ui::i18n::teach_msg("exercise_err_disconnected", locale)
                                .to_string(),
                        ),
                        ..Default::default()
                    };
                    let _ = draw_panel_ejercicio(ui, &mut error, locale);
                });
            });
        }
        // ES pineado al copy previo (canónicos intactos vía wrappers Es).
        assert_eq!(
            resolver_lo("").unwrap_err(),
            "Decime un tema primero, che (ej. «derivada»)."
        );
        assert_eq!(
            resolver_lo_for("", grafito_ui::i18n::Locale::Es).unwrap_err(),
            "Decime un tema primero, che (ej. «derivada»)."
        );
        assert_eq!(
            resolver_lo_for("", grafito_ui::i18n::Locale::En).unwrap_err(),
            "Tell me a topic first (e.g. “derivative”)."
        );
        let sin_lo =
            resolver_lo_for("origami de papel xyz", grafito_ui::i18n::Locale::Es).unwrap_err();
        assert!(
            sin_lo.contains("«origami de papel xyz»"),
            "not_found sin tema interpolado: {sin_lo}"
        );
        // `pedir_con` vacío no spawnea (mismo camino que `pedir` Es).
        let mut panel = PanelEjercicio::default();
        panel.pedir_con(
            "   ",
            grafito_pedagogy::PedagogicalLevel::Secondary,
            grafito_ui::i18n::Locale::En,
        );
        assert_eq!(panel.estado, EstadoPanelEjercicio::Error);
        assert_eq!(
            panel.error.as_deref(),
            Some("Tell me a topic first (e.g. “derivative”).")
        );
        // Pin de wiring: los 5 sitios usan `teach_msg` con `locale`
        // (helpers `_for` + `pedir_con` + `sondear_con`).
        let fuente = include_str!("teaching_ui.rs");
        for clave in [
            "exercise_err_empty_topic",
            "exercise_err_not_found",
            "exercise_err_invalid",
            "exercise_err_spawn",
            "exercise_err_disconnected",
        ] {
            let mut cableado = false;
            let mut inicio = 0;
            while let Some(rel) = fuente[inicio..].find("teach_msg(") {
                let abs = inicio + rel;
                let fin = (abs + 200).min(fuente.len());
                if fuente[abs..fin].contains(clave) && fuente[abs..fin].contains("locale") {
                    cableado = true;
                    break;
                }
                inicio = abs + 10;
            }
            assert!(cableado, "sitio teaching sin cablear a teach_msg: {clave}");
        }
        // Blindaje negativo: los literales ES ya no están en call-sites prod
        // (agujas partidas para no auto-matchear este propio test vía
        // `include_str!`, precedente B-A70/B-A73/B-A75).
        let vacio_es = [
            "return Err(\"Decime un tema primero",
            ", che (ej. «derivada»).\".to_string());",
        ]
        .concat();
        let perdido_es = [
            "format!(\"No encontré ejercicios de",
            " «{recortado}», probá con otro tema, che.\")",
        ]
        .concat();
        let fallado_es = [
            "format!(\"El ejercicio salió fallado",
            " ({motivo}), probá de nuevo, che.\")",
        ]
        .concat();
        let spawn_es = [
            "Some(\"No pude arrancar la generación",
            ", probá de nuevo, che.\".to_string())",
        ]
        .concat();
        let corte_es = [
            "Some(\"Se cortó la generación",
            ", probá de nuevo, che.\".to_string())",
        ]
        .concat();
        for aguja in [&vacio_es, &perdido_es, &fallado_es, &spawn_es, &corte_es] {
            assert!(
                !fuente.contains(aguja.as_str()),
                "literal ES duro aún en teaching_ui: {aguja}"
            );
        }
    }

    #[test]
    fn b_a78_teaching_toast_hint_localizan_sin_panico() {
        // B-A78: el toast de arranque y el hint morph salen por `teach_msg`
        // en el `locale` dado; Es y En sin pánico headless (helpers puros con
        // templates + overlay que ejercita el hint).
        for locale in [grafito_ui::i18n::Locale::Es, grafito_ui::i18n::Locale::En] {
            let toast = exercise_building_toast_text("derivada", locale);
            assert!(toast.contains("derivada"), "toast sin tema en {locale:?}");
            let hint = exercise_step_hint_text(2, 5, locale);
            assert!(hint.contains('2'), "hint sin n en {locale:?}");
            assert!(hint.contains('5'), "hint sin m en {locale:?}");
            // El overlay ejercita el hint morph en ambos idiomas.
            let ctx = egui::Context::default();
            let _ = ctx.run(egui::RawInput::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let _ = exercise_step_hint_text(1, 3, locale);
                    let _ = ui.label("hint");
                });
            });
        }
        // ES pineado a los literales previos (`assistant.rs:6627`,
        // `teaching_ui.rs:2243-2247`).
        assert_eq!(
            exercise_building_toast_text("derivada", grafito_ui::i18n::Locale::Es),
            "Armo un ejercicio de derivada, che."
        );
        assert_eq!(
            exercise_building_toast_text("derivada", grafito_ui::i18n::Locale::En),
            "Building an exercise on derivada."
        );
        assert_eq!(
            exercise_step_hint_text(2, 5, grafito_ui::i18n::Locale::Es),
            "Mirá el paso 2 de 5 — vamos de a poco, che."
        );
        assert_eq!(
            exercise_step_hint_text(2, 5, grafito_ui::i18n::Locale::En),
            "Look at step 2 of 5 — one at a time."
        );
        // Pin de wiring: los 2 sitios usan `teach_msg` con `locale`.
        let fuente = include_str!("teaching_ui.rs");
        for clave in ["exercise_building_toast", "exercise_step_hint"] {
            let mut cableado = false;
            let mut inicio = 0;
            while let Some(rel) = fuente[inicio..].find("teach_msg(") {
                let abs = inicio + rel;
                let fin = (abs + 200).min(fuente.len());
                if fuente[abs..fin].contains(clave) && fuente[abs..fin].contains("locale") {
                    cableado = true;
                    break;
                }
                inicio = abs + 10;
            }
            assert!(cableado, "sitio teaching sin cablear a teach_msg: {clave}");
        }
        // Blindaje negativo: los literales ES ya no están en call-sites prod
        // (agujas partidas para no auto-matchear este propio test vía
        // `include_str!`, precedente B-A70/B-A73/B-A75/B-A76).
        // El hint vivía en este archivo; el toast en `assistant.rs:6627`.
        let hint_es = [
            "format!(\"Mirá el paso {} de",
            " {} — vamos de a poco, che.\")",
        ]
        .concat();
        assert!(
            !fuente.contains(hint_es.as_str()),
            "literal ES duro aún en teaching_ui: {hint_es}"
        );
        let app_fuente = include_str!("assistant.rs");
        assert!(
            app_fuente.contains("exercise_building_toast_text(&tema, locale)"),
            "toast de ejercicio sin cablear al helper localizado"
        );
        let toast_es = ["format!(\"Armo un ejercicio de", " {tema}, che.\")"].concat();
        assert!(
            !app_fuente.contains(toast_es.as_str()),
            "literal ES duro aún en assistant: {toast_es}"
        );
    }

    #[test]
    fn b_a79_teaching_overlay_reserve_localiza_sin_panico() {
        // B-A79: el label de reserva nativa sale por `teach_msg` en el
        // `locale` dado; Es y En sin pánico headless (helper puro).
        for locale in [grafito_ui::i18n::Locale::Es, grafito_ui::i18n::Locale::En] {
            let texto = overlay_anim_reserve_text("guion:1/1", 12, locale);
            assert!(texto.contains("guion:1/1"), "sin template en {locale:?}");
            assert!(texto.contains("12"), "sin n en {locale:?}");
            let ctx = egui::Context::default();
            let _ = ctx.run(egui::RawInput::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let _ = ui.label(overlay_anim_reserve_text("guion:1/1", 12, locale));
                });
            });
        }
        // ES pineado al literal previo (`teaching_ui.rs:2489`).
        assert_eq!(
            overlay_anim_reserve_text("guion:1/1", 12, grafito_ui::i18n::Locale::Es),
            "Animación: guion:1/1 — 12 cuadros (reserva nativa)"
        );
        assert_eq!(
            overlay_anim_reserve_text("guion:1/1", 12, grafito_ui::i18n::Locale::En),
            "Animation: guion:1/1 — 12 frames (native fallback)"
        );
        // Pin de wiring: el sitio usa `teach_msg` con `locale` vía el helper.
        let fuente = include_str!("teaching_ui.rs");
        assert!(
            fuente.contains("overlay_anim_reserve_text(&template,"),
            "reserva nativa sin cablear al helper localizado"
        );
        // Blindaje negativo: el literal ES ya no está en call-sites prod
        // (aguja partida para no auto-matchear este propio test vía
        // `include_str!`, precedente B-A70/B-A73/B-A75/B-A76/B-A78).
        let reserva_es = ["Animación: {} — {} cuadros (reserva", " nativa)"].concat();
        assert!(
            !fuente.contains(reserva_es.as_str()),
            "literal ES duro aún en teaching_ui: {reserva_es}"
        );
    }
}
