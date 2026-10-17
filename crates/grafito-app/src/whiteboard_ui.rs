//! Pizarra nativa en overlay (estilo macOS): lienzo a pantalla completa con
//! toolbar flotante redondeada (pill) y herramientas de dibujo libres sobre
//! `grafito_whiteboard`, sincronizada en ambas direcciones con
//! `Document.whiteboard` (editar marca el documento sucio con snapshot undo;
//! abrir/nuevo/undo/redo recarga la sesión).
//!
//! Libro de 32 hojas: `WhiteboardBook` vive en memoria y persiste completo en
//! `Document.whiteboard_pages` (+ índice `whiteboard_current`) vía
//! `push_whiteboard_book_and_store`; `Document.whiteboard` queda como espejo
//! de la hoja actual para el export SVG existente. JSON viejo sin libro se
//! migra en memoria a una hoja ("Hoja 1") sin pérdida.

use egui::{pos2, vec2, Color32, Pos2, Rect, Sense, Stroke};
use grafito_core::{ChangeSet, Document};
use grafito_ui::icons::{action_icon_button_sized, Icon};
use grafito_ui::theme::current_theme;
use grafito_ui::tokens::{
    touch_hit, ALPHA_OVERLAY, RADIUS_MD, RADIUS_SM, SHADOW_ALPHA, SHADOW_POPUP_BLUR,
    SHADOW_POPUP_OFFSET_Y, SPACE_SM, SPACE_XS, TYPE_XS, ZOOM_ICON_HIT, ZOOM_PCT_MIN_W,
    ZOOM_PILL_GAP, ZOOM_PILL_PAD_X, ZOOM_PILL_PAD_Y, ZOOM_PILL_RADIUS, ZOOM_WB_DEFAULT,
    ZOOM_WB_MAX, ZOOM_WB_MIN,
};
use grafito_whiteboard::{
    arrow_tip, axis_lines, compass_circle, recognize_pencil_shape, smooth_stroke, stencil_elements,
    toggle_curtain_slot, toggle_slot, CurtainDirection, FigureStencil, InstrumentDrag,
    InstrumentKind, WhiteboardBackground, WhiteboardCurtain, WhiteboardDoc, WhiteboardElement,
    WhiteboardInstrument, WhiteboardInteraction, WhiteboardTool, CURTAIN_EDGE_TOL_PX,
    DOTTED_STEP_PX, HIGHLIGHTER_ALPHA, HIGHLIGHTER_MIN_WIDTH, INSTRUMENT_SNAP_TOLERANCE,
    MAGNIFIER_FACTOR, PEN_WIDTH_DEFAULT, PEN_WIDTH_MAX, PEN_WIDTH_MIN, RULED_STEP_PX,
};
use std::collections::VecDeque;

/// Máximo de elementos por hoja de pizarra (alineado con `validation.rs:497`).
pub const MAX_WHITEBOARD_ELEMENTS_PER_PAGE: usize = 500;
/// Máximo de hojas del libro de pizarra.
pub const MAX_WHITEBOARD_PAGES: usize = 32;
/// Máximo de puntos por trazo libre (alineado con el cap de `smooth_stroke`).
pub const MAX_WHITEBOARD_POINTS_PER_STROKE: usize = 4096;
/// Máximo de caracteres por texto de pizarra.
pub const MAX_WHITEBOARD_TEXT_CHARS: usize = 2000;

fn finite_pair(point: (f64, f64)) -> bool {
    point.0.is_finite() && point.1.is_finite()
}

/// Valida un elemento antes de insertarlo: cotas por hoja/trazo/texto con
/// mensajes en español. Nunca hace panic; devuelve `Err` con el motivo.
pub fn validate_whiteboard_element(
    element: &WhiteboardElement,
    current_len: usize,
) -> Result<(), String> {
    if current_len >= MAX_WHITEBOARD_ELEMENTS_PER_PAGE {
        return Err(format!(
            "La pizarra alcanzó el máximo de {MAX_WHITEBOARD_ELEMENTS_PER_PAGE} elementos por hoja; borrá algo o creá otra hoja"
        ));
    }
    match element {
        WhiteboardElement::Stroke { points, width, .. }
        | WhiteboardElement::Highlighter { points, width, .. } => {
            if points.len() > MAX_WHITEBOARD_POINTS_PER_STROKE {
                return Err(format!(
                    "El trazo supera el máximo de {MAX_WHITEBOARD_POINTS_PER_STROKE} puntos"
                ));
            }
            if !width.is_finite() || *width <= 0.0 {
                return Err("El grosor del trazo debe ser finito y positivo".to_string());
            }
            if points.iter().any(|point| !finite_pair(*point)) {
                return Err("El trazo contiene coordenadas no finitas".to_string());
            }
        }
        WhiteboardElement::Rectangle { min, max, .. } => {
            if !finite_pair(*min) || !finite_pair(*max) {
                return Err("El rectángulo contiene coordenadas no finitas".to_string());
            }
        }
        WhiteboardElement::Ellipse { center, rx, ry } => {
            if !finite_pair(*center) || !rx.is_finite() || !ry.is_finite() {
                return Err("La elipse contiene valores no finitos".to_string());
            }
            if *rx <= 0.0 || *ry <= 0.0 {
                return Err("La elipse necesita radios positivos".to_string());
            }
        }
        WhiteboardElement::Arrow { from, to } => {
            if !finite_pair(*from) || !finite_pair(*to) {
                return Err("La flecha contiene coordenadas no finitas".to_string());
            }
        }
        WhiteboardElement::Text { at, text, size } => {
            if !finite_pair(*at) || !size.is_finite() || *size <= 0.0 {
                return Err("El texto tiene posición o tamaño no válido".to_string());
            }
            if text.chars().count() > MAX_WHITEBOARD_TEXT_CHARS {
                return Err(format!(
                    "El texto supera el máximo de {MAX_WHITEBOARD_TEXT_CHARS} caracteres"
                ));
            }
        }
        WhiteboardElement::Image {
            bytes_png_jpeg,
            rect,
        } => {
            // Límites pineados del tipo (lado ≤2048, bytes ≤4 MiB): la única
            // fuente es `validate_image_bytes`, sin duplicar números.
            if let Err(causa) = grafito_whiteboard::validate_image_bytes(bytes_png_jpeg) {
                return Err(format!("La imagen no entra: {causa}"));
            }
            if !finite_pair(rect.0) || !finite_pair(rect.1) {
                return Err("La imagen tiene posición no válida".to_string());
            }
        }
    }
    Ok(())
}

/// Compara pizarras por elementos (`WhiteboardDoc` no implementa `PartialEq`).
pub fn whiteboard_docs_equal(first: &WhiteboardDoc, second: &WhiteboardDoc) -> bool {
    first.elements() == second.elements()
}

/// Umbral de aviso suave: la hoja llega al 90% (450 de 500 elementos).
pub const WHITEBOARD_FILL_WARN_ELEMENTS: usize = 450;
/// Umbral de aviso fuerte: la hoja llega al 96% (480 de 500 elementos).
pub const WHITEBOARD_FILL_ALMOST_FULL_ELEMENTS: usize = 480;

/// Aviso de llenado en español según elementos de la hoja (`None` con cupo).
/// 450..480 suave, 480..500 casi llena; el tope 500 lo rechaza `try_add`.
pub fn whiteboard_fill_warning(element_count: usize) -> Option<String> {
    if element_count >= MAX_WHITEBOARD_ELEMENTS_PER_PAGE {
        Some(format!(
            "La hoja está llena ({element_count} de {MAX_WHITEBOARD_ELEMENTS_PER_PAGE}); creá otra hoja o borrá algo"
        ))
    } else if element_count >= WHITEBOARD_FILL_ALMOST_FULL_ELEMENTS {
        Some(format!(
            "La hoja está casi llena ({element_count} de {MAX_WHITEBOARD_ELEMENTS_PER_PAGE}); conviene crear otra hoja"
        ))
    } else if element_count >= WHITEBOARD_FILL_WARN_ELEMENTS {
        Some(format!(
            "La hoja tiene {element_count} de {MAX_WHITEBOARD_ELEMENTS_PER_PAGE} elementos"
        ))
    } else {
        None
    }
}

/// Aviso solo al cruzar un umbral (450/480) para no spamear toasts:
/// `None` si el tramo `[before, after)` no cruza ningún umbral.
fn crossed_fill_warning(before: usize, after: usize) -> Option<String> {
    for threshold in [
        WHITEBOARD_FILL_WARN_ELEMENTS,
        WHITEBOARD_FILL_ALMOST_FULL_ELEMENTS,
    ] {
        if before < threshold && after >= threshold {
            return whiteboard_fill_warning(after);
        }
    }
    None
}

/// Página individual de la pizarra (una “hoja” tipo Notepad).
/// `background` es preferencia de vista en memoria (como pan/zoom): NO se
/// persiste — `to_persisted`/`from_persisted` lo excluyen a propósito (la
/// persistencia del documento es intocable en este slice) y al reabrir la
/// hoja vuelve al cuadriculado default. Conducta documentada, no bug.
#[derive(Clone)]
pub struct WhiteboardPage {
    pub title: String,
    pub doc: WhiteboardDoc,
    pub pan: (f64, f64),
    pub zoom: f64,
    pub background: WhiteboardBackground,
}

impl WhiteboardPage {
    fn new(id: usize) -> Self {
        Self {
            title: format!("Hoja {}", id),
            doc: WhiteboardDoc::new(),
            pan: (0.0, 0.0),
            zoom: ZOOM_WB_DEFAULT,
            background: WhiteboardBackground::default(),
        }
    }
}

/// Libro de pizarras — colección de hojas con índice actual.
#[derive(Clone)]
pub struct WhiteboardBook {
    pub pages: Vec<WhiteboardPage>,
    pub current: usize,
    next_id: usize,
    /// Contador monotónico de mutaciones del libro (hoja agregada/quitada,
    /// cambio de índice, título o contenido sincronizado desde la sesión).
    epoch: u64,
    epoch_synced: u64,
}

impl Default for WhiteboardBook {
    fn default() -> Self {
        Self {
            pages: vec![WhiteboardPage::new(1)],
            current: 0,
            next_id: 2,
            epoch: 0,
            epoch_synced: 0,
        }
    }
}

impl WhiteboardBook {
    /// Cambios desde el último `mark_synced` (fast-path del commit idle).
    pub fn is_synced(&self) -> bool {
        self.epoch == self.epoch_synced
    }

    pub fn mark_synced(&mut self) {
        self.epoch_synced = self.epoch;
    }

    fn bump_epoch(&mut self) {
        self.epoch = self.epoch.wrapping_add(1);
    }

    pub fn len(&self) -> usize {
        self.pages.len()
    }

    pub fn current(&self) -> Option<&WhiteboardPage> {
        self.pages.get(self.current)
    }

    pub fn current_mut(&mut self) -> Option<&mut WhiteboardPage> {
        self.pages.get_mut(self.current)
    }

    pub fn create_page(&mut self) -> usize {
        // Acotado sin panic: al llegar al tope se reutiliza la hoja actual.
        if self.pages.len() >= MAX_WHITEBOARD_PAGES {
            return self.current;
        }
        let page = WhiteboardPage::new(self.next_id);
        self.next_id += 1;
        self.pages.push(page);
        self.bump_epoch();
        self.pages.len() - 1
    }

    /// Crea una hoja o rechaza en español al superar `MAX_WHITEBOARD_PAGES`.
    pub fn try_create_page(&mut self) -> Result<usize, String> {
        if self.pages.len() >= MAX_WHITEBOARD_PAGES {
            return Err(format!(
                "La pizarra alcanzó el máximo de {MAX_WHITEBOARD_PAGES} hojas"
            ));
        }
        Ok(self.create_page())
    }

    pub fn switch_to(&mut self, index: usize) -> bool {
        if index < self.pages.len() {
            self.current = index;
            self.bump_epoch();
            true
        } else {
            false
        }
    }

    pub fn remove(&mut self, index: usize) -> bool {
        if self.pages.len() <= 1 || index >= self.pages.len() {
            return false;
        }
        self.pages.remove(index);
        if self.current >= self.pages.len() {
            self.current = self.pages.len() - 1;
        } else if index < self.current {
            self.current -= 1;
        }
        self.bump_epoch();
        true
    }

    /// Vuelca la sesión a la hoja actual solo si algo cambió (revisión del
    /// doc, vista o fondo): en idle NO clona los elementos por frame.
    /// El fondo viaja sesión↔hoja en memoria pero NUNCA al documento
    /// (ver `to_persisted`): un cambio solo-de-fondo no escribe a disco.
    pub fn save_current_from_session(&mut self, session: &WhiteboardSession) {
        let mut changed = false;
        if let Some(page) = self.current_mut() {
            if page.doc.revision() != session.doc.revision()
                || page.pan != session.pan
                || page.zoom != session.zoom
                || page.background != session.background
            {
                page.doc = session.doc.clone();
                page.pan = session.pan;
                page.zoom = session.zoom;
                page.background = session.background;
                changed = true;
            }
        }
        if changed {
            self.bump_epoch();
        }
    }

    pub fn load_to_session(&self, session: &mut WhiteboardSession) {
        if let Some(page) = self.current() {
            session.replace_doc(page.doc.clone());
            session.pan = page.pan;
            session.zoom = page.zoom;
            session.background = page.background;
        }
    }
}

/// Paridad de cotas con `grafito-core` (rompe compilación si divergen).
const _WHITEBOARD_BOOK_PAGES_PARITY: () =
    assert!(MAX_WHITEBOARD_PAGES == grafito_core::MAX_WHITEBOARD_PAGES);
const _WHITEBOARD_BOOK_ELEMENTS_PARITY: () =
    assert!(MAX_WHITEBOARD_ELEMENTS_PER_PAGE == grafito_core::MAX_WHITEBOARD_ELEMENTS_PER_PAGE);

/// Siguiente id de hoja tras los títulos "Hoja N" (sin panic).
fn next_id_after_titles(pages: &[WhiteboardPage]) -> usize {
    let mut max_seen = 0usize;
    for page in pages {
        if let Some(rest) = page.title.strip_prefix("Hoja ") {
            if let Ok(number) = rest.trim().parse::<usize>() {
                if number > max_seen {
                    max_seen = number;
                }
            }
        }
    }
    max_seen
        .saturating_add(1)
        .max(pages.len().saturating_add(1))
        .max(1)
}

impl WhiteboardBook {
    /// Foto persistible del libro para `Document::set_whiteboard_book`.
    /// Pura (sin I/O): clona títulos, vistas y contenido de cada hoja.
    /// El fondo NO viaja (preferencia de vista en memoria, como documenta
    /// `WhiteboardPage`): la persistencia del documento queda intacta y al
    /// reabrir cada hoja defaulta a cuadriculado.
    pub fn to_persisted(&self) -> Vec<grafito_core::WhiteboardPageData> {
        self.pages
            .iter()
            .map(|page| grafito_core::WhiteboardPageData {
                title: page.title.clone(),
                doc: page.doc.clone(),
                pan: page.pan,
                zoom: page.zoom,
            })
            .collect()
    }

    /// Reconstruye el libro desde páginas persistidas (vacío → 1 hoja).
    /// Acota a 32 hojas y recorta el índice al rango válido, sin panic.
    /// El fondo no viene persistido: cada hoja nace cuadriculada (default
    /// honesto documentado en `WhiteboardPage`).
    pub fn from_persisted(pages: &[grafito_core::WhiteboardPageData], current: usize) -> Self {
        if pages.is_empty() {
            return Self::default();
        }
        let mut book = Self {
            pages: Vec::new(),
            current: 0,
            next_id: 1,
            epoch: 0,
            epoch_synced: 0,
        };
        for page in pages.iter().take(MAX_WHITEBOARD_PAGES) {
            book.pages.push(WhiteboardPage {
                title: page.title.clone(),
                doc: page.doc.clone(),
                pan: page.pan,
                zoom: page.zoom,
                background: WhiteboardBackground::default(),
            });
        }
        if book.pages.is_empty() {
            return Self::default();
        }
        book.next_id = next_id_after_titles(&book.pages);
        book.current = current.min(book.pages.len() - 1);
        book
    }
}

/// Arrastre de imagen con Mover: cuerpo (traslación) o manija SE (escala).
/// Solo imágenes (alcance cerrado del slice); el resto sigue por marquee.
#[derive(Debug, Clone, Copy, PartialEq)]
enum ImageDrag {
    /// Mover el cuerpo: `grab`/`last` en mundo (delta incremental).
    Move {
        index: usize,
        grab: (f64, f64),
        last: (f64, f64),
    },
    /// Escalar con la manija: `fixed` = esquina opuesta fija.
    Scale { index: usize, fixed: (f64, f64) },
}

/// Entrada de la caché de texturas de imágenes (índice → handle + huella).
#[derive(Clone)]
struct WhiteboardImageTexture {
    /// Longitud de bytes (cambio barato de detectar).
    len: usize,
    /// FNV-1a 64 de los bytes (el rect cambia sin tocar los bytes: la
    /// textura se reutiliza al mover/escalar sin re-decodificar).
    hash: u64,
    handle: egui::TextureHandle,
}

/// FNV-1a 64 propio (sin deps): huella barata de bytes de imagen.
fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// Tolerancia de agarre de la manija en px de pantalla (mundo = px/zoom).
const IMAGE_HANDLE_GRAB_PX: f64 = 12.0;

/// Sesión de pizarra activa (overlay) — con color y grosor del lápiz.
#[derive(Clone)]
pub struct WhiteboardSession {
    pub doc: WhiteboardDoc,
    pub tool: WhiteboardTool,
    interaction: WhiteboardInteraction,
    pencil_points: Vec<(f64, f64)>,
    pub pan: (f64, f64),
    pub zoom: f64,
    /// Fondo de página (vista, no contenido): default cuadriculado, no
    /// persiste (ver `WhiteboardPage`). Selector "Fondo" de la toolbar.
    pub background: WhiteboardBackground,
    active_text: Option<usize>,
    marquee: Option<((f64, f64), (f64, f64))>,
    pub pen_color: Color32,
    pub pen_width: f32,
    /// Auto-formas del lápiz (estilo OpenBoard, default apagado para no
    /// mutar el gesto por sorpresa): con `true`, el trazo cerrado ≈
    /// círculo se guarda como `Ellipse` y el recto como `Arrow`; si no
    /// reconoce nada queda el `Stroke` tal cual. El resaltador nunca se
    /// convierte en forma.
    pub auto_shapes: bool,
    pub show_palette: bool,
    /// Instrumento visible (overlay estilo OpenBoard: regla/escuadra/
    /// transportador). Uno a la vez; no es figura: no se selecciona, no
    /// entra al documento ni al SVG. El lápiz/resaltador se pega a su borde.
    pub instrument: Option<WhiteboardInstrument>,
    /// Arrastre en curso sobre el instrumento (cuerpo/manija).
    instrument_drag: Option<InstrumentDrag>,
    /// Compás activo (dos clics → círculo exacto). Comparte el slot único:
    /// al activarse guarda cualquier regla/escuadra/transportador visible.
    pub compass_active: bool,
    /// Centro pendiente del compás (primer clic ya dado, falta el radio).
    pub compass_center: Option<(f64, f64)>,
    /// Cortina estilo OpenBoard (overlay de sesión: cubre la hoja, su borde
    /// libre se arrastra para revelar progresivo). Slot único con el resto:
    /// mostrarla guarda instrumento/compás/lupa. Jamás persiste ni va al SVG.
    pub curtain: Option<WhiteboardCurtain>,
    /// Dirección recordada de la cortina (el botón la muestra con esta
    /// dirección; el clic en la zona cubierta la cambia arriba/abajo).
    pub curtain_direction: CurtainDirection,
    /// Arrastre del borde libre de la cortina en curso (espacio de pantalla).
    curtain_drag: bool,
    /// Lupa 2x estilo OpenBoard (overlay de sesión). Implementación honesta
    /// como zoom-to-point sobre el zoom de sesión (ver `MAGNIFIER_FACTOR`):
    /// el clic acerca centrado en el punto con el clamp existente; el anillo
    /// solo marca el modo. Slot único: activarla guarda el resto.
    pub magnifier_active: bool,
    /// Último rechazo de cota en español (se muestra como toast y se limpia).
    pub last_error: Option<String>,
    /// Último aviso de llenado 450/480 en español (toast informativo, edge-triggered).
    pub last_warning: Option<String>,
    /// Arrastre de imagen en curso (Mover: cuerpo o manija SE).
    img_drag: Option<ImageDrag>,
    /// Caché de texturas de imágenes por índice (se invalida si cambia el
    /// largo del doc; el contenido se valida por huella FNV por frame).
    img_textures: std::collections::HashMap<usize, WhiteboardImageTexture>,
    /// Largo del doc con el que se pobló `img_textures`.
    img_textures_len: usize,
}

impl Default for WhiteboardSession {
    fn default() -> Self {
        Self {
            doc: WhiteboardDoc::new(),
            tool: WhiteboardTool::Pencil,
            interaction: WhiteboardInteraction::Idle,
            pencil_points: Vec::new(),
            pan: (0.0, 0.0),
            zoom: ZOOM_WB_DEFAULT,
            background: WhiteboardBackground::default(),
            active_text: None,
            marquee: None,
            // Color por defecto con contraste en ambos temas; paleta permite cambiar.
            // Se ajusta automáticamente en draw() si el contraste con canvas_bg es bajo.
            // LIGHT.text_primary (#1A1A1A) — oscuro sobre canvas claro; en dark se adapta vía effective_pen_color().
            pen_color: grafito_ui::theme::LIGHT.text_primary,
            pen_width: PEN_WIDTH_DEFAULT as f32,
            auto_shapes: false,
            show_palette: false,
            instrument: None,
            instrument_drag: None,
            compass_active: false,
            compass_center: None,
            curtain: None,
            curtain_direction: CurtainDirection::Top,
            curtain_drag: false,
            magnifier_active: false,
            last_error: None,
            last_warning: None,
            img_drag: None,
            img_textures: std::collections::HashMap::new(),
            img_textures_len: 0,
        }
    }
}

impl WhiteboardSession {
    pub fn world_from_screen(&self, screen: Pos2, rect: Rect) -> (f64, f64) {
        if rect.width() <= 0.0 || rect.height() <= 0.0 || !self.zoom.is_finite() || self.zoom == 0.0
        {
            return (0.0, 0.0);
        }
        if !screen.x.is_finite() || !screen.y.is_finite() {
            return (0.0, 0.0);
        }
        let x = (screen.x - rect.center().x - self.pan.0 as f32) / self.zoom as f32;
        let y = (screen.y - rect.center().y - self.pan.1 as f32) / self.zoom as f32;
        if !x.is_finite() || !y.is_finite() {
            return (0.0, 0.0);
        }
        (x as f64, y as f64)
    }

    fn screen_from_world(&self, world: (f64, f64), rect: Rect) -> Pos2 {
        if !world.0.is_finite() || !world.1.is_finite() || !rect.center().x.is_finite() {
            return rect.center();
        }
        pos2(
            rect.center().x + self.pan.0 as f32 + world.0 as f32 * self.zoom as f32,
            rect.center().y + self.pan.1 as f32 + world.1 as f32 * self.zoom as f32,
        )
    }

    pub fn set_tool(&mut self, tool: WhiteboardTool) {
        self.tool = tool;
        self.interaction = WhiteboardInteraction::Idle;
        self.pencil_points.clear();
        self.marquee = None;
        self.img_drag = None;
        self.active_text = None;
        // El overlay del instrumento sobrevive al cambio de herramienta
        // (como en OpenBoard: se dibuja con el lápiz sobre la regla); solo
        // se suelta un arrastre en curso para no dejarlo enganchado.
        // Cortina y lupa sobreviven igual (mismo slot único).
        self.instrument_drag = None;
        self.curtain_drag = false;
        // Palette solo para herramientas con color; la mantenemos si ya estaba abierta
        if !matches!(
            tool,
            WhiteboardTool::Pencil
                | WhiteboardTool::Highlighter
                | WhiteboardTool::Rectangle
                | WhiteboardTool::Ellipse
                | WhiteboardTool::Arrow
                | WhiteboardTool::Text
        ) {
            self.show_palette = false;
        }
    }

    pub fn clear(&mut self) {
        self.doc.clear();
        self.active_text = None;
    }

    /// Inserta un elemento respetando las cotas (500/hoja, 4096 pts, 2000
    /// caracteres). Rechaza con mensaje en español en `Err` y lo guarda en
    /// `last_error`; nunca hace panic.
    pub fn try_add(&mut self, element: WhiteboardElement) -> Result<(), String> {
        self.push_validated(element, false)
    }

    /// Como [`try_add`](Self::try_add) pero inserta AL FONDO (índice 0):
    /// lo usa el resaltador para quedar DEBAJO de la tinta existente con
    /// orden honesto (documento == pintado). Misma validación y avisos.
    pub fn try_add_highlighter(&mut self, element: WhiteboardElement) -> Result<(), String> {
        self.push_validated(element, true)
    }

    /// Inserta un stencil del botón "Figuras" como elementos normales
    /// (trazos/textos editables y borrables, nada de variante nueva).
    /// El stencil nace centrado en la vista actual; la inserción es
    /// atómica: si no hay cupo para los N elementos, no entra ninguno y
    /// vuelve `Err` en español (mismo idioma que `try_add`). Devuelve
    /// cuántos elementos entraron (pineado por stencil en los tests).
    pub fn insert_figure(&mut self, kind: FigureStencil) -> Result<usize, String> {
        let zoom = if self.zoom.is_finite() && self.zoom > 0.0 {
            self.zoom
        } else {
            1.0
        };
        let origin = (-self.pan.0 / zoom, -self.pan.1 / zoom);
        let elements = stencil_elements(kind, origin);
        if elements.is_empty() {
            return Err("La figura no tiene posición válida.".to_string());
        }
        if self.doc.len() + elements.len() > MAX_WHITEBOARD_ELEMENTS_PER_PAGE {
            let message = format!(
                "La pizarra alcanzó el máximo de {MAX_WHITEBOARD_ELEMENTS_PER_PAGE} elementos por hoja; borrá algo o creá otra hoja"
            );
            self.last_error = Some(message.clone());
            return Err(message);
        }
        let before = self.doc.len();
        for element in elements {
            self.try_add(element)?;
        }
        Ok(self.doc.len() - before)
    }

    fn push_validated(&mut self, element: WhiteboardElement, below: bool) -> Result<(), String> {
        match validate_whiteboard_element(&element, self.doc.len()) {
            Ok(()) => {
                let before = self.doc.len();
                if below {
                    self.doc.insert_at_bottom(element);
                } else {
                    self.doc.add(element);
                }
                self.last_error = None;
                let after = self.doc.len();
                // Aviso 450/480 solo al cruzar el umbral (sin spam de toasts).
                self.last_warning = crossed_fill_warning(before, after);
                Ok(())
            }
            Err(message) => {
                self.last_error = Some(message.clone());
                Err(message)
            }
        }
    }

    /// Reemplaza el contenido y resetea el estado transitorio de edición.
    fn replace_doc(&mut self, doc: WhiteboardDoc) {
        self.doc = doc;
        self.active_text = None;
        self.interaction = WhiteboardInteraction::Idle;
        self.pencil_points.clear();
        self.marquee = None;
        self.img_drag = None;
    }

    pub fn zoom_at(&mut self, factor: f64, screen: Pos2, rect: Rect) {
        if !factor.is_finite() || factor <= 0.0 || !screen.x.is_finite() {
            return;
        }
        let before = self.world_from_screen(screen, rect);
        self.zoom = (self.zoom * factor).clamp(ZOOM_WB_MIN, ZOOM_WB_MAX);
        if !self.zoom.is_finite() {
            self.zoom = ZOOM_WB_DEFAULT;
        }
        let after = self.world_from_screen(screen, rect);
        if !before.0.is_finite() || !after.0.is_finite() {
            return;
        }
        // Corregido drift: pan está en píxeles, delta mundo debe escalarse por zoom
        let dx = (before.0 - after.0) * self.zoom;
        let dy = (before.1 - after.1) * self.zoom;
        if dx.is_finite() && dy.is_finite() {
            self.pan = (self.pan.0 - dx, self.pan.1 - dy);
        }
    }

    /// Conmuta un instrumento del slot único (un instrumento visible a la
    /// vez; clic de nuevo lo guarda). Activarlo guarda el compás pendiente.
    /// El overlay sobrevive al cambio de herramienta: se dibuja con el
    /// lápiz/resaltador sobre él y el trazo se pega a su borde.
    pub fn toggle_instrument(&mut self, kind: InstrumentKind) {
        let fresh = match kind {
            InstrumentKind::Ruler => WhiteboardInstrument::ruler_default(),
            InstrumentKind::Triangle => WhiteboardInstrument::triangle_default(),
            InstrumentKind::Protractor => WhiteboardInstrument::protractor_default(),
        };
        self.instrument = toggle_slot(&self.instrument, fresh);
        if self.instrument.is_some() {
            self.compass_active = false;
            self.compass_center = None;
            self.curtain = None;
            self.magnifier_active = false;
        }
        self.instrument_drag = None;
        self.curtain_drag = false;
        self.interaction = WhiteboardInteraction::Idle;
        self.pencil_points.clear();
        self.marquee = None;
    }

    /// Conmuta el compás (dos clics → círculo exacto). Comparte el slot
    /// único: al activarse guarda regla/escuadra/transportador.
    pub fn toggle_compass(&mut self) {
        self.compass_active = !self.compass_active;
        if self.compass_active {
            self.instrument = None;
            self.instrument_drag = None;
            self.curtain = None;
            self.curtain_drag = false;
            self.magnifier_active = false;
        }
        self.compass_center = None;
        self.interaction = WhiteboardInteraction::Idle;
        self.pencil_points.clear();
        self.marquee = None;
    }

    /// Clase del instrumento visible (`None` si está guardado).
    pub fn instrument_kind(&self) -> Option<InstrumentKind> {
        self.instrument.map(|instrument| instrument.kind())
    }

    /// Conmuta la cortina del slot único (clic de nuevo la oculta).
    /// Mostrarla guarda instrumento/compás/lupa; jamás toca el documento.
    pub fn toggle_curtain(&mut self) {
        self.curtain = toggle_curtain_slot(&self.curtain, self.curtain_direction);
        if self.curtain.is_some() {
            self.instrument = None;
            self.instrument_drag = None;
            self.compass_active = false;
            self.compass_center = None;
            self.magnifier_active = false;
        }
        self.curtain_drag = false;
        self.interaction = WhiteboardInteraction::Idle;
        self.pencil_points.clear();
        self.marquee = None;
    }

    /// Cambia la dirección recordada arriba/abajo; con la cortina visible
    /// espeja el borde para conservar el área cubierta.
    pub fn flip_curtain_direction(&mut self) {
        self.curtain_direction = match self.curtain_direction {
            CurtainDirection::Top => CurtainDirection::Bottom,
            CurtainDirection::Bottom => CurtainDirection::Top,
        };
        if let Some(curtain) = self.curtain.as_mut() {
            curtain.flip_direction();
        }
    }

    /// Conmuta la lupa 2x del slot único (clic de nuevo la apaga).
    /// Activarla guarda instrumento/compás/cortina; jamás toca el documento.
    pub fn toggle_magnifier(&mut self) {
        self.magnifier_active = !self.magnifier_active;
        if self.magnifier_active {
            self.instrument = None;
            self.instrument_drag = None;
            self.compass_active = false;
            self.compass_center = None;
            self.curtain = None;
            self.curtain_drag = false;
        }
        self.interaction = WhiteboardInteraction::Idle;
        self.pencil_points.clear();
        self.marquee = None;
    }

    /// ¿`screen_y` agarra el borde libre de la cortina? (px de pantalla,
    /// tolerancia pineada [`CURTAIN_EDGE_TOL_PX`]). Puro y testeable.
    pub fn curtain_edge_hit(&self, screen_y: f64, view_top: f64, view_height: f64) -> bool {
        self.curtain.is_some_and(|curtain| {
            curtain.edge_hit(screen_y, view_top, view_height, CURTAIN_EDGE_TOL_PX)
        })
    }

    /// ¿`screen_y` cae en la zona cubierta por la cortina?
    pub fn curtain_covers(&self, screen_y: f64, view_top: f64, view_height: f64) -> bool {
        self.curtain
            .is_some_and(|curtain| curtain.covers_y(screen_y, view_top, view_height))
    }

    /// Lleva el borde libre de la cortina a `screen_y` (clamp 0..=1).
    /// No-op sin cortina visible o con vista degenerada.
    pub fn apply_curtain_drag(&mut self, screen_y: f64, view_top: f64, view_height: f64) {
        if !screen_y.is_finite()
            || !view_top.is_finite()
            || !view_height.is_finite()
            || view_height <= 0.0
        {
            return;
        }
        if let Some(curtain) = self.curtain.as_mut() {
            curtain.set_edge((screen_y - view_top) / view_height);
        }
    }

    /// Lupa honesta como zoom-to-point 2x: acerca centrado en `screen` con
    /// el clamp de `zoom_at` (`ZOOM_WB_MIN..=ZOOM_WB_MAX` intactos, default
    /// intacto). Equivale a `magnifier_zoom` sobre el zoom actual.
    pub fn apply_magnifier_at(&mut self, screen: Pos2, rect: Rect) {
        self.zoom_at(MAGNIFIER_FACTOR, screen, rect);
    }

    /// ¿El punto agarra el instrumento visible (cuerpo o manija)? Lo usa la
    /// rama de paneo táctil para no robarle el gesto al instrumento.
    pub(crate) fn wants_instrument_drag(&self, world: (f64, f64)) -> bool {
        self.instrument
            .is_some_and(|instrument| instrument.begin_drag(world).is_some())
    }

    /// Pega un punto del lápiz/resaltador al borde del instrumento activo
    /// (proyección al segmento más cercano dentro de la tolerancia pineada;
    /// fuera de tolerancia devuelve el punto tal cual: trazo libre honesto).
    /// La tolerancia vive en mundo pero se compensa por zoom para que el
    /// agarre efectivo sea ~12 px de pantalla a cualquier zoom.
    fn snap_with_instrument(&self, point: (f64, f64)) -> (f64, f64) {
        match self.instrument {
            Some(instrument) => {
                let zoom = if self.zoom.is_finite() && self.zoom > 0.0 {
                    self.zoom
                } else {
                    1.0
                };
                instrument
                    .snap_point(point, INSTRUMENT_SNAP_TOLERANCE / zoom)
                    .unwrap_or(point)
            }
            None => point,
        }
    }

    /// Un clic con el compás activo: el primero fija el centro, el segundo
    /// emite el círculo exacto (reusa `Ellipse`) y deja el compás listo para
    /// el siguiente; un radio diminuto conserva el centro pendiente.
    fn handle_compass_click(&mut self, world: (f64, f64)) {
        match self.compass_center {
            None => self.compass_center = Some(world),
            Some(center) => {
                if let Some(circle) = compass_circle(center, world) {
                    match self.try_add(circle) {
                        Ok(()) => self.compass_center = None,
                        Err(message) => self.last_error = Some(message),
                    }
                }
            }
        }
    }

    /// Intenta agarrar una imagen con Mover: manija SE → escala, cuerpo →
    /// mover (topmost primero). Selecciona el índice y arma el arrastre;
    /// `false` = no hay imagen bajo el punto (el llamador hace marquee).
    fn begin_image_drag(&mut self, world: (f64, f64)) -> bool {
        let zoom = if self.zoom.is_finite() && self.zoom > 0.0 {
            self.zoom
        } else {
            1.0
        };
        let handle_tol = IMAGE_HANDLE_GRAB_PX / zoom;
        let body_tol = f64::from(SPACE_SM) / zoom;
        // Manija primero (topmost), después cuerpo (topmost).
        for index in (0..self.doc.len()).rev() {
            if self.doc.elements()[index].image_hit_handle(world, handle_tol) {
                let Some(((lox, loy), _)) = self.doc.elements()[index].bounds() else {
                    continue;
                };
                self.doc.select_at(world, handle_tol);
                self.img_drag = Some(ImageDrag::Scale {
                    index,
                    fixed: (lox, loy),
                });
                return true;
            }
        }
        for index in (0..self.doc.len()).rev() {
            let is_image = matches!(self.doc.elements()[index], WhiteboardElement::Image { .. });
            if is_image && self.doc.elements()[index].distance_to(world) <= body_tol {
                self.doc.select_at(world, body_tol);
                self.img_drag = Some(ImageDrag::Move {
                    index,
                    grab: world,
                    last: world,
                });
                return true;
            }
        }
        false
    }

    /// Aplica el arrastre de imagen al punto mundo actual (mover por delta
    /// incremental o escalar con la esquina fija). Índice stale (borrado en
    /// otro gesto) → suelta el arrastre sin tocar nada.
    fn apply_image_drag(&mut self, world: (f64, f64)) {
        let Some(drag) = self.img_drag else {
            return;
        };
        match drag {
            ImageDrag::Move { index, last, .. } => {
                let delta = (world.0 - last.0, world.1 - last.1);
                let moved = self
                    .doc
                    .element_mut(index)
                    .is_some_and(|element| element.move_by(delta.0, delta.1));
                if moved {
                    self.img_drag = Some(ImageDrag::Move {
                        index,
                        grab: match self.img_drag {
                            Some(ImageDrag::Move { grab, .. }) => grab,
                            _ => world,
                        },
                        last: world,
                    });
                } else {
                    self.img_drag = None;
                }
            }
            ImageDrag::Scale { index, fixed } => {
                let scaled = self
                    .doc
                    .element_mut(index)
                    .is_some_and(|element| element.scale_image_to(fixed, world));
                if !scaled {
                    self.img_drag = None;
                }
            }
        }
    }

    pub fn handle_pointer(&mut self, world: (f64, f64), pressed: bool, released: bool) {
        if !world.0.is_finite() || !world.1.is_finite() {
            return;
        }
        // Compás: los clics fijan centro/radio, no dibujan trazos.
        if self.compass_active && pressed {
            self.handle_compass_click(world);
            self.interaction = WhiteboardInteraction::Idle;
            self.marquee = None;
            return;
        }
        if pressed {
            // El instrumento flota sobre el canvas (OpenBoard): agarrarlo
            // mueve/rota sin manchar la hoja.
            if let Some(instrument) = self.instrument {
                if let Some(drag) = instrument.begin_drag(world) {
                    self.instrument_drag = Some(drag);
                    self.interaction = WhiteboardInteraction::Idle;
                    self.marquee = None;
                    self.pencil_points.clear();
                    return;
                }
            }
            // Validar herramienta antes de iniciar interacción
            self.interaction = WhiteboardInteraction::begin(world, self.tool);
            self.marquee = None;
            if matches!(
                self.tool,
                WhiteboardTool::Pencil | WhiteboardTool::Highlighter
            ) {
                // El primer punto ya se pega al borde si cae sobre el filo.
                self.pencil_points = vec![self.snap_with_instrument(world)];
                // Capar puntos para evitar DoS
                if self.pencil_points.len() > 4096 {
                    self.pencil_points.truncate(4096);
                }
            }
            if self.tool == WhiteboardTool::Select {
                // Mover solo-imágenes: manija SE (escala) o cuerpo
                // (traslación) con topmost; si no agarra imagen, marquee.
                if !self.begin_image_drag(world) {
                    self.marquee = Some((world, world));
                }
            }
            return;
        }
        // Arrastre en curso sobre el instrumento: no dibuja.
        if let Some(drag) = self.instrument_drag {
            if let Some(instrument) = self.instrument.as_mut() {
                instrument.apply_drag(drag, world);
            }
            if released {
                self.instrument_drag = None;
            }
            return;
        }
        // Arrastre de imagen en curso: aplica mover/escalar puro y listo.
        if self.img_drag.is_some() {
            if released {
                self.img_drag = None;
                self.interaction = WhiteboardInteraction::Idle;
                self.marquee = None;
            } else {
                self.apply_image_drag(world);
            }
            return;
        }
        if matches!(self.interaction, WhiteboardInteraction::Idle) {
            return;
        }
        self.interaction.update(world);
        if matches!(
            self.tool,
            WhiteboardTool::Pencil | WhiteboardTool::Highlighter
        ) && matches!(self.interaction, WhiteboardInteraction::Creating { .. })
            && self.pencil_points.len() < 4096
        {
            self.pencil_points.push(self.snap_with_instrument(world));
        }
        if self.tool == WhiteboardTool::Select {
            if let Some((a, _)) = self.marquee {
                self.marquee = Some((a, world));
            }
        }
        if released {
            self.finalize();
        }
    }

    /// Cierra el trazo del lápiz/resaltador: suaviza, valida finitud y crea
    /// el elemento con el grosor (1..=24) y color elegidos. Ramas honestas:
    /// resaltador → `Highlighter` al fondo (ancho generoso, alfa fijo);
    /// lápiz con auto-formas → `Ellipse`/`Arrow` si reconoce, `Stroke` si
    /// no; lápiz normal → `Stroke` tal cual. Nunca hace panic.
    fn finish_pencil(&mut self) {
        let points = smooth_stroke(&self.pencil_points, 3);
        if points.len() >= 2 && points.iter().all(|(x, y)| x.is_finite() && y.is_finite()) {
            let c = self.pen_color;
            let rgb = (c.r(), c.g(), c.b());
            let width = f64::from(
                self.pen_width
                    .clamp(PEN_WIDTH_MIN as f32, PEN_WIDTH_MAX as f32),
            );
            let element = if self.tool == WhiteboardTool::Highlighter {
                WhiteboardElement::Highlighter {
                    points,
                    color: rgb,
                    width: width.max(HIGHLIGHTER_MIN_WIDTH),
                }
            } else if self.auto_shapes {
                recognize_pencil_shape(&points).unwrap_or(WhiteboardElement::Stroke {
                    points,
                    color: rgb,
                    width,
                })
            } else {
                WhiteboardElement::Stroke {
                    points,
                    color: rgb,
                    width,
                }
            };
            let result = if matches!(element, WhiteboardElement::Highlighter { .. }) {
                self.try_add_highlighter(element)
            } else {
                self.try_add(element)
            };
            if let Err(message) = result {
                self.last_error = Some(message);
            }
        }
        self.pencil_points.clear();
    }

    fn finalize(&mut self) {
        match self.tool {
            WhiteboardTool::Pencil | WhiteboardTool::Highlighter => self.finish_pencil(),
            WhiteboardTool::Select => {
                if let Some((a, b)) = self.marquee {
                    let center = ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
                    let _ = self.doc.select_at(center, SPACE_SM as f64 / self.zoom);
                }
                self.marquee = None;
            }
            WhiteboardTool::Eraser => {
                if let Some(path) = self.interaction.take_erase_path() {
                    self.erase_trace(path);
                }
            }
            other => {
                if let Some(element) = self.interaction.end() {
                    let is_text = other == WhiteboardTool::Text;
                    match self.try_add(element) {
                        Ok(()) => {
                            if is_text {
                                self.active_text = Some(self.doc.len().saturating_sub(1));
                            }
                        }
                        Err(message) => {
                            self.last_error = Some(message);
                        }
                    }
                }
            }
        }
        self.interaction = WhiteboardInteraction::Idle;
    }

    fn erase_trace(&mut self, path: Vec<(f64, f64)>) {
        // Tolerancia ligada a tokens: SPACE_SM/2 ≈ 4 + ajuste para grosor trazo
        let tolerance = (SPACE_SM as f64 - 2.0) / self.zoom;
        for point in path {
            let mut before = self.doc.len();
            while let Some(erased) = self.doc.erase_at(point, tolerance) {
                // Si borramos el texto activo, invalidar índice (evita crash al seguir escribiendo)
                if let Some(active) = self.active_text {
                    if erased == active {
                        self.active_text = None;
                    } else if erased < active {
                        self.active_text = Some(active.saturating_sub(1));
                    }
                    if self.active_text.is_some_and(|i| i >= self.doc.len()) {
                        self.active_text = None;
                    }
                }
                if self.doc.len() == before {
                    break;
                }
                before = self.doc.len();
            }
        }
        self.validate_active_text();
        if self.doc.is_empty() {
            self.active_text = None;
        }
    }

    pub fn type_char(&mut self, character: char) {
        let Some(index) = self.active_text else {
            return;
        };
        if index >= self.doc.len() {
            self.active_text = None;
            return;
        }
        if let Some(WhiteboardElement::Text { text, .. }) = self.doc.element_mut(index) {
            // Guard contra control chars que crashean layout (excepto \n)
            if character.is_control() && character != '\n' {
                return;
            }
            if text.chars().count() >= MAX_WHITEBOARD_TEXT_CHARS {
                self.last_error = Some(format!(
                    "El texto supera el máximo de {MAX_WHITEBOARD_TEXT_CHARS} caracteres"
                ));
                return;
            }
            text.push(character);
        } else {
            self.active_text = None;
        }
    }

    pub fn backspace_text(&mut self) {
        let Some(index) = self.active_text else {
            return;
        };
        if index >= self.doc.len() {
            self.active_text = None;
            return;
        }
        if let Some(WhiteboardElement::Text { text, .. }) = self.doc.element_mut(index) {
            text.pop();
        } else {
            self.active_text = None;
        }
    }

    /// Invalida active_text si apunta fuera de rango (tras borrado/clear).
    fn validate_active_text(&mut self) {
        if self.active_text.is_some_and(|i| i >= self.doc.len()) {
            self.active_text = None;
        }
    }

    fn effective_pen_color(&self, theme: &grafito_ui::theme::Theme) -> Color32 {
        // Si el usuario eligió un color claro sobre canvas claro (o oscuro sobre oscuro),
        // usa text_primary que siempre contrasta. Solo para el default histórico claro.
        let is_light_canvas = theme.canvas_bg.r() > 200;
        let pen_is_light =
            self.pen_color.r() > 200 && self.pen_color.g() > 200 && self.pen_color.b() > 200;
        let pen_is_dark =
            self.pen_color.r() < 60 && self.pen_color.g() < 60 && self.pen_color.b() < 60;
        if is_light_canvas && pen_is_light {
            return theme.text_primary;
        }
        if !is_light_canvas && pen_is_dark {
            // Quiet light on dark: theme.text_primary en dark es FAFAF9 (250,250,249); históricamente 240,241,244
            return theme.text_primary;
        }
        // Compatibilidad: el default histórico era claro (240,241,244 ≈ FAFAF9) que es invisible en modo claro.
        // Se mantiene comparación exacta para docs viejos, pero retorno usa theme (no hardcode).
        const HISTORIC_LIGHT_PEN: Color32 = Color32::from_rgb(240, 241, 244);
        if is_light_canvas && self.pen_color == HISTORIC_LIGHT_PEN {
            return theme.text_primary;
        }
        self.pen_color
    }

    /// Rasteriza una imagen del doc con textura cacheada (una decodificación
    /// por contenido, no por frame): valida cabecera, reutiliza el handle si
    /// la huella FNV coincide (mover/escalar no re-decodifica) y dibuja el
    /// marco de selección con manija SE si está seleccionada. Sin panic.
    fn draw_cached_image(
        &mut self,
        ui: &mut egui::Ui,
        index: usize,
        canvas: Rect,
        selected: bool,
        theme: &grafito_ui::theme::Theme,
    ) {
        let Some(WhiteboardElement::Image {
            bytes_png_jpeg,
            rect,
        }) = self.doc.elements().get(index)
        else {
            return;
        };
        if grafito_whiteboard::validate_image_bytes(bytes_png_jpeg).is_err() {
            return;
        }
        let rect = *rect;
        let hash = fnv1a_64(bytes_png_jpeg);
        let fresh = self
            .img_textures
            .get(&index)
            .is_some_and(|cached| cached.len == bytes_png_jpeg.len() && cached.hash == hash);
        if !fresh {
            // Solo en miss: clona acotado (≤4 MiB validado) para decodificar
            // sin pelear con el borrow de la caché.
            let bytes = bytes_png_jpeg.clone();
            let Some(color) = decode_whiteboard_image(&bytes) else {
                return;
            };
            let handle = ui.ctx().load_texture(
                format!("wbimg-{index}-{hash:016x}"),
                color,
                egui::TextureOptions::LINEAR,
            );
            self.img_textures.insert(
                index,
                WhiteboardImageTexture {
                    len: bytes.len(),
                    hash,
                    handle,
                },
            );
        }
        let Some(cached) = self.img_textures.get(&index) else {
            return;
        };
        let texture_id = cached.handle.id();
        let a = self.screen_from_world(rect.0, canvas);
        let b = self.screen_from_world(rect.1, canvas);
        if !a.x.is_finite() || !b.x.is_finite() {
            return;
        }
        let shown = Rect::from_two_pos(a, b);
        ui.painter().image(
            texture_id,
            shown,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );
        if selected {
            ui.painter()
                .rect_stroke(shown, 0.0, Stroke::new(1.5, theme.accent));
            let se = shown.max;
            ui.painter().rect_filled(
                Rect::from_center_size(se, vec2(10.0, 10.0)),
                2.0,
                theme.accent,
            );
        }
    }

    pub fn draw(&mut self, ui: &mut egui::Ui, rect: Rect) {
        if rect.width() <= 0.0 || rect.height() <= 0.0 || !rect.min.x.is_finite() {
            return;
        }
        let theme = current_theme(ui.ctx());
        // `&mut` por la caché de texturas de imágenes (decodifica una vez).
        if self.img_textures_len != self.doc.len() {
            self.img_textures.clear();
            self.img_textures_len = self.doc.len();
        }
        ui.painter().rect_filled(rect, 0.0, theme.canvas_bg);
        // Grid con contraste adaptativo: más opaco en modo claro sobre #FAFAF9
        let is_light = theme.canvas_bg.r() > 200;
        let grid_col = if is_light {
            theme.separator.gamma_multiply(0.35)
        } else {
            theme.separator.gamma_multiply(0.10)
        };
        if grid_col.a() > 0 {
            draw_background(
                ui.painter(),
                rect,
                self.pan,
                self.zoom,
                self.background,
                grid_col,
            );
        }
        for index in 0..self.doc.len() {
            let Some(element) = self.doc.elements().get(index) else {
                continue;
            };
            // Proteger contra elementos con NaN
            if let Some((min, max)) = element.bounds() {
                if !min.0.is_finite() || !max.0.is_finite() {
                    continue;
                }
            }
            // La imagen se rasteriza con textura cacheada (&mut); el resto
            // igual que antes (los borrows compartidos ya terminaron acá).
            let selected = self.doc.selected() == Some(index);
            if matches!(element, WhiteboardElement::Image { .. }) {
                self.draw_cached_image(ui, index, rect, selected, theme);
                continue;
            }
            draw_element(ui.painter(), element, rect, self, theme);
        }
        // Trazo en vivo del lápiz/resaltador: color, grosor (1..=24) y alfa actuales.
        if matches!(
            self.tool,
            WhiteboardTool::Pencil | WhiteboardTool::Highlighter
        ) && !self.pencil_points.is_empty()
        {
            let highlighter = self.tool == WhiteboardTool::Highlighter;
            let live_width = (if highlighter {
                self.pen_width.max(HIGHLIGHTER_MIN_WIDTH as f32)
            } else {
                self.pen_width
            })
            .clamp(PEN_WIDTH_MIN as f32, PEN_WIDTH_MAX as f32);
            let live_color = if highlighter {
                self.effective_pen_color(theme)
                    .gamma_multiply(HIGHLIGHTER_ALPHA)
            } else {
                self.effective_pen_color(theme)
            };
            let stroke = Stroke::new(live_width * self.zoom as f32, live_color);
            for pair in self.pencil_points.windows(2) {
                if !pair[0].0.is_finite() || !pair[1].0.is_finite() {
                    continue;
                }
                let a = self.screen_from_world(pair[0], rect);
                let b = self.screen_from_world(pair[1], rect);
                if a.x.is_finite() && b.x.is_finite() {
                    ui.painter().line_segment([a, b], stroke);
                }
            }
            if self.pencil_points.len() == 1 {
                let p = self.screen_from_world(self.pencil_points[0], rect);
                if p.x.is_finite() {
                    ui.painter().circle_filled(
                        p,
                        (live_width * 0.75).clamp(1.0, 18.0) * self.zoom as f32,
                        live_color,
                    );
                }
            }
        }
        if let Some(preview) = self.interaction.preview() {
            draw_element(ui.painter(), &preview, rect, self, theme);
        }
        if let Some((a, b)) = self.marquee {
            if a.0.is_finite() && b.0.is_finite() {
                let (pa, pb) = (
                    self.screen_from_world(a, rect),
                    self.screen_from_world(b, rect),
                );
                if pa.x.is_finite() && pb.x.is_finite() {
                    ui.painter().rect_stroke(
                        Rect::from_two_pos(pa, pb),
                        2.0,
                        Stroke::new(1.5, theme.accent.gamma_multiply(0.6)),
                    );
                }
            }
        }
        // Instrumentos estilo OpenBoard: flotan SOBRE la tinta (son objetos
        // físicos, no dibujo) con manija de rotación. Nunca entran al
        // documento ni al SVG: solo se dibujan aquí.
        draw_instrument_overlay(ui.painter(), self, rect, theme);
        // Cortina y lupa: overlays de sesión sobre todo lo anterior; jamás
        // tocan el documento ni el SVG (mismo patrón que instrumentos).
        draw_curtain_overlay(ui.painter(), self, rect, theme);
        draw_magnifier_hint(ui, self, rect, theme);
        // Goma visible: círculo cursor + trazo de borrado en vivo (antes invisible).
        // Radio = tolerancia de borrado en px para que lo que ves es lo que borra.
        if self.tool == WhiteboardTool::Eraser {
            let erase_r_px = (SPACE_SM - 2.0).max(6.0);
            // Táctil: durante el arrastre no hay hover; el punto activo
            // muestra el círculo (si no, se borra a ciegas).
            let touch_point = ui.input(|i| i.pointer.hover_pos().or(i.pointer.interact_pos()));
            if let Some(hover) = touch_point {
                if rect.contains(hover) {
                    ui.painter()
                        .circle_stroke(hover, erase_r_px, Stroke::new(1.5, theme.accent));
                    ui.painter()
                        .circle_filled(hover, 2.0, theme.accent.gamma_multiply(0.9));
                }
            }
            if let WhiteboardInteraction::Erasing { path } = &self.interaction {
                let erase_stroke = Stroke::new(2.0, theme.accent.gamma_multiply(0.7));
                for pair in path.windows(2) {
                    let a = self.screen_from_world(pair[0], rect);
                    let b = self.screen_from_world(pair[1], rect);
                    if a.x.is_finite() && b.x.is_finite() {
                        ui.painter().line_segment([a, b], erase_stroke);
                    }
                }
            }
        }
    }

    pub fn handle_canvas_input(
        &mut self,
        rect: Rect,
        ui: &egui::Ui,
        budget: &mut crate::app::RepaintBudget,
    ) {
        // Repintar a 60Hz si hay cualquier pointer/touch activo (tableta, mouse, touch).
        // F17: vía presupuesto coalescido, no `ctx.request_repaint_after` directo.
        if ui
            .ctx()
            .input(|i| i.pointer.any_down() || i.multi_touch().is_some())
        {
            budget.request(std::time::Duration::from_millis(16));
        }
        let response = ui.interact(
            rect,
            egui::Id::new("whiteboard_canvas"),
            Sense::click_and_drag(),
        );
        let zoom_delta = ui.input(|i| i.zoom_delta());
        let pointer = ui.input(|i| i.pointer.clone());
        let current_pos = response
            .interact_pointer_pos()
            .or(response.hover_pos())
            .or(pointer.latest_pos());
        if zoom_delta != 1.0 {
            if let Some(pos) = current_pos {
                self.zoom_at(zoom_delta as f64, pos, rect);
            }
        }
        // Rueda del mouse: zoom centrado en el cursor
        let scroll = ui.input(|i| i.raw_scroll_delta.y);
        if scroll.abs() > f32::EPSILON {
            if let Some(pos) = current_pos {
                if rect.contains(pos) {
                    let factor = (1.0 + scroll * 0.008).clamp(0.85, 1.15) as f64;
                    self.zoom_at(factor, pos, rect);
                    // F17: un frame por evento de rueda vía presupuesto
                    // coalescido (equivale al inmediato, sin romper la
                    // ruta única `apply_repaint_budget`).
                    budget.request(std::time::Duration::ZERO);
                }
            }
        }
        // Atajos de teclado: Ctrl/Cmd + rueda ya cubiertos; +/- y 0 para reset
        if ui.input(|i| i.modifiers.ctrl || i.modifiers.command) {
            if ui.input(|i| i.key_pressed(egui::Key::Equals) || i.key_pressed(egui::Key::Plus)) {
                if let Some(pos) = current_pos.or(Some(rect.center())) {
                    self.zoom_at(1.2, pos, rect);
                }
            }
            if ui.input(|i| i.key_pressed(egui::Key::Minus)) {
                if let Some(pos) = current_pos.or(Some(rect.center())) {
                    self.zoom_at(0.8333, pos, rect);
                }
            }
            if ui.input(|i| i.key_pressed(egui::Key::Num0)) {
                self.zoom = ZOOM_WB_DEFAULT;
                self.pan = (0.0, 0.0);
            }
        } else {
            // Sin Ctrl: +/- también funcionan si el canvas tiene foco
            if ui.input(|i| i.key_pressed(egui::Key::Equals) || i.key_pressed(egui::Key::Plus)) {
                if let Some(pos) = current_pos.or(Some(rect.center())) {
                    self.zoom_at(1.15, pos, rect);
                }
            }
            if ui.input(|i| i.key_pressed(egui::Key::Minus)) {
                if let Some(pos) = current_pos.or(Some(rect.center())) {
                    self.zoom_at(0.87, pos, rect);
                }
            }
        }
        if response.dragged_by(egui::PointerButton::Middle) {
            return; // pan medio reservado para el canvas principal
        }
        // S3b bis: con 2 dedos manda el zoom (su rama); los overlays no
        // consumen el gesto multitáctil.
        let navigating_here = ui.input(|i| i.multi_touch().is_some());
        // Cortina (overlay de sesión, no dibuja): arrastrar el borde libre
        // revela progresivo; clic en la zona cubierta (fuera del borde)
        // cambia arriba/abajo. Consume el gesto para no manchar la hoja.
        if self.curtain.is_some() && !navigating_here {
            match current_pos {
                Some(pos) => {
                    let top = rect.min.y as f64;
                    let height = rect.height() as f64;
                    let y = pos.y as f64;
                    if self.curtain_drag {
                        if response.dragged() || response.drag_started() {
                            self.apply_curtain_drag(y, top, height);
                        }
                        if response.drag_stopped() {
                            self.curtain_drag = false;
                        }
                        return;
                    }
                    if response.drag_started() && self.curtain_edge_hit(y, top, height) {
                        self.curtain_drag = true;
                        self.apply_curtain_drag(y, top, height);
                        return;
                    }
                    if response.clicked()
                        && self.curtain_covers(y, top, height)
                        && !self.curtain_edge_hit(y, top, height)
                    {
                        self.flip_curtain_direction();
                        return;
                    }
                }
                // Puntero perdido durante el arrastre: suelta el borde.
                None => self.curtain_drag = false,
            }
        }
        // Lupa honesta (zoom-to-point 2x, ver `MAGNIFIER_FACTOR`): el clic
        // acerca centrado en el punto con el clamp de `zoom_at`.
        if self.magnifier_active && !navigating_here {
            if let Some(pos) = current_pos {
                if rect.contains(pos) && response.clicked() {
                    self.apply_magnifier_at(pos, rect);
                    return;
                }
            }
        }
        // Compatibilidad tableta: cualquier botón de dibujo + fallback Touch
        let any_draw_button = pointer.button_down(egui::PointerButton::Primary)
            || pointer.button_down(egui::PointerButton::Secondary)
            || pointer.button_down(egui::PointerButton::Middle)
            || pointer.any_down();
        // S3b: 2 dedos navegan (zoom en su rama); con multitouch no nace
        // ningún trazo (misma regla que el canvas matemático S3a).
        let navigating = ui.input(|i| i.multi_touch().is_some());
        if let Some(world) = current_pos.map(|pos| self.world_from_screen(pos, rect)) {
            let is_idle = matches!(self.interaction, WhiteboardInteraction::Idle);
            let stylus_pressed =
                any_draw_button && is_idle && current_pos.is_some_and(|p| rect.contains(p));
            let drag_any = response.dragged();
            // Móvil (<740): con Mover, arrastrar panea la hoja (el tap
            // selecciona, como hoy). Sin esto solo 2 dedos la mueven.
            // El instrumento manda: agarrarlo mueve/rota, no panea.
            let narrow_pan = ui.ctx().screen_rect().width()
                < grafito_ui::tokens::PALETTE_BOTTOM_SHEET_BREAKPOINT
                && self.tool == WhiteboardTool::Select
                && drag_any
                && !navigating
                && !self.wants_instrument_drag(world);
            if narrow_pan {
                let delta = response.drag_delta();
                if delta.x.is_finite() && delta.y.is_finite() {
                    self.pan = (self.pan.0 + delta.x as f64, self.pan.1 + delta.y as f64);
                }
                self.interaction = WhiteboardInteraction::Idle;
                self.marquee = None;
            } else if !navigating
                && (stylus_pressed || response.drag_started() || response.clicked())
            {
                self.handle_pointer(world, true, false);
            } else if response.drag_stopped() || (!any_draw_button && !is_idle) {
                self.handle_pointer(world, false, true);
            } else if !navigating && (drag_any || any_draw_button) {
                // Sin el gate el pinch-zoom (2.º dedo) seguía añadiendo
                // puntos mundo con el pan/zoom ya cambiado → zigzag que
                // corrompía el trazo/goma. Con multitouch manda la rama
                // de zoom; al soltar, un solo segmento une (sin picos).
                self.handle_pointer(world, false, false);
            }
        }
    }
}

/// Fondo de página (vista, no contenido): despacha al dibujo que toque.
/// `Grid` = el `draw_grid` histórico intacto; `Ruled`/`Dotted` usan el
/// mismo origen (centro + pan) con su paso; `Plain` no dibuja nada.
/// Sin panic: pasos no finitos o vista degenerada → nada.
fn draw_background(
    painter: &egui::Painter,
    rect: Rect,
    pan: (f64, f64),
    zoom: f64,
    background: WhiteboardBackground,
    color: Color32,
) {
    match background {
        WhiteboardBackground::Plain => {}
        WhiteboardBackground::Grid => draw_grid(painter, rect, pan, zoom, color),
        WhiteboardBackground::Ruled => {
            if !zoom.is_finite() || zoom <= 0.0 {
                return;
            }
            let step = RULED_STEP_PX as f32 * zoom as f32;
            if step < 6.0 || !step.is_finite() {
                return;
            }
            let origin_y = rect.center().y + pan.1 as f32;
            for y in axis_lines(
                f64::from(rect.min.y),
                f64::from(rect.max.y),
                f64::from(origin_y),
                f64::from(step),
            ) {
                painter.line_segment(
                    [pos2(rect.min.x, y as f32), pos2(rect.max.x, y as f32)],
                    Stroke::new(1.0, color),
                );
            }
        }
        WhiteboardBackground::Dotted => {
            if !zoom.is_finite() || zoom <= 0.0 {
                return;
            }
            let step = DOTTED_STEP_PX as f32 * zoom as f32;
            if step < 6.0 || !step.is_finite() {
                return;
            }
            let origin = (
                f64::from(rect.center().x + pan.0 as f32),
                f64::from(rect.center().y + pan.1 as f32),
            );
            let xs = axis_lines(
                f64::from(rect.min.x),
                f64::from(rect.max.x),
                origin.0,
                f64::from(step),
            );
            let ys = axis_lines(
                f64::from(rect.min.y),
                f64::from(rect.max.y),
                origin.1,
                f64::from(step),
            );
            for x in xs {
                for y in &ys {
                    painter.circle_filled(pos2(x as f32, *y as f32), 1.5, color);
                }
            }
        }
    }
}
fn draw_grid(painter: &egui::Painter, rect: Rect, pan: (f64, f64), zoom: f64, color: Color32) {
    let step = 40.0_f32 * zoom as f32;
    if step < 6.0 {
        return;
    }
    // Origen = el mismo que el contenido (`screen_from_world`: centro +
    // pan). Antes nacía en `rect.min` y la grilla derivaba respecto a los
    // trazos al cambiar el zoom.
    let origin = egui::pos2(
        rect.center().x + pan.0 as f32,
        rect.center().y + pan.1 as f32,
    );
    let mut x = origin.x - ((origin.x - rect.min.x) / step).floor() * step;
    while x < rect.max.x {
        painter.line_segment(
            [pos2(x, rect.min.y), pos2(x, rect.max.y)],
            Stroke::new(1.0, color),
        );
        x += step;
    }
    let mut y = origin.y - ((origin.y - rect.min.y) / step).floor() * step;
    while y < rect.max.y {
        painter.line_segment(
            [pos2(rect.min.x, y), pos2(rect.max.x, y)],
            Stroke::new(1.0, color),
        );
        y += step;
    }
}

/// Dibuja el instrumento visible + la marca del compás pendiente: overlays
/// físicos sobre la tinta (OpenBoard), con manija de rotación. Puro dibujo:
/// jamás toca el documento (PROHIBIDO serializar instrumentos). Sin panic.
fn draw_instrument_overlay(
    painter: &egui::Painter,
    session: &WhiteboardSession,
    rect: Rect,
    theme: &grafito_ui::theme::Theme,
) {
    let edge = Stroke::new(2.0, theme.accent);
    let fill = theme.accent.gamma_multiply(0.12);
    let tick = Stroke::new(1.0, theme.accent.gamma_multiply(0.7));
    if let Some(instrument) = session.instrument {
        match instrument {
            WhiteboardInstrument::Ruler { .. } | WhiteboardInstrument::Triangle { .. } => {
                // Filos como polígono cerrado (regla: tira de 16; escuadra:
                // triángulo rectángulo) con graduaciones cada 20 unidades.
                let segments = instrument.edge_segments();
                let mut outline: Vec<Pos2> = Vec::new();
                if let Some(first) = segments.first() {
                    let (dx, dy) = {
                        let (ax, ay) = (first.1 .0 - first.0 .0, first.1 .1 - first.0 .1);
                        let norm = (ax * ax + ay * ay).sqrt().max(1e-9);
                        (-ay / norm * 8.0, ax / norm * 8.0)
                    };
                    // Regla: tira alrededor del primer filo; escuadra: sus 3 vértices.
                    if matches!(instrument, WhiteboardInstrument::Ruler { .. }) {
                        let (a, b) = segments[0];
                        for world in [
                            (a.0 - dx, a.1 - dy),
                            (b.0 - dx, b.1 - dy),
                            (b.0 + dx, b.1 + dy),
                            (a.0 + dx, a.1 + dy),
                        ] {
                            if world.0.is_finite() && world.1.is_finite() {
                                outline.push(session.screen_from_world(world, rect));
                            }
                        }
                    } else {
                        for (a, _) in &segments {
                            if a.0.is_finite() && a.1.is_finite() {
                                outline.push(session.screen_from_world(*a, rect));
                            }
                        }
                    }
                }
                if outline.iter().all(|p| p.x.is_finite()) && outline.len() >= 3 {
                    painter.add(egui::Shape::convex_polygon(
                        outline.clone(),
                        fill,
                        Stroke::NONE,
                    ));
                    painter.add(egui::Shape::closed_line(outline, edge));
                }
                for (a, b) in segments {
                    draw_edge_ticks(painter, session, rect, a, b, tick);
                }
            }
            WhiteboardInstrument::Protractor { pos, angle, radius } => {
                if pos.0.is_finite() && angle.is_finite() && radius.is_finite() {
                    let (dx, dy) = (angle.cos(), angle.sin());
                    let (nx, ny) = (-dy, dx);
                    let center = session.screen_from_world(pos, rect);
                    // Base (diámetro) + arco de medio disco.
                    let ends = [
                        session.screen_from_world((pos.0 - dx * radius, pos.1 - dy * radius), rect),
                        session.screen_from_world((pos.0 + dx * radius, pos.1 + dy * radius), rect),
                    ];
                    if center.x.is_finite() && ends.iter().all(|p| p.x.is_finite()) {
                        painter.line_segment([center, ends[0]], edge);
                        painter.line_segment([center, ends[1]], edge);
                        let mut arc: Vec<Pos2> = Vec::new();
                        for step in 0..=24 {
                            let theta = std::f64::consts::PI * f64::from(step) / 24.0;
                            let (c, s) = (theta.cos(), theta.sin());
                            let world = (
                                pos.0 + dx * radius * c + nx * radius * s,
                                pos.1 + dy * radius * c + ny * radius * s,
                            );
                            arc.push(session.screen_from_world(world, rect));
                        }
                        if arc.iter().all(|p| p.x.is_finite()) {
                            painter.add(egui::Shape::line(arc, edge));
                        }
                    }
                    for (a, b) in instrument.edge_segments().iter().take(1) {
                        draw_edge_ticks(painter, session, rect, *a, *b, tick);
                    }
                }
            }
        }
        // Manija de rotación: círculo de agarre (prioridad sobre el cuerpo).
        if let Some(handle) = instrument.handle_pos() {
            let screen = session.screen_from_world(handle, rect);
            if screen.x.is_finite() {
                painter.circle_filled(screen, 7.0, theme.panel_bg);
                painter.circle_stroke(screen, 7.0, edge);
                painter.circle_filled(screen, 2.0, theme.accent);
            }
        }
    }
    // Compás activo con centro pendiente: cruz honesta donde cayó el clic.
    if session.compass_active {
        if let Some(center) = session.compass_center {
            let screen = session.screen_from_world(center, rect);
            if screen.x.is_finite() {
                painter.circle_stroke(screen, 8.0, edge);
                painter.circle_filled(screen, 2.0, theme.accent);
                painter.line_segment([screen + vec2(-12.0, 0.0), screen + vec2(12.0, 0.0)], tick);
                painter.line_segment([screen + vec2(0.0, -12.0), screen + vec2(0.0, 12.0)], tick);
            }
        }
    }
}

/// Graduaciones cada 20 unidades mundo sobre un filo (puro dibujo, sin panic).
fn draw_edge_ticks(
    painter: &egui::Painter,
    session: &WhiteboardSession,
    rect: Rect,
    a: (f64, f64),
    b: (f64, f64),
    tick: Stroke,
) {
    let dx = b.0 - a.0;
    let dy = b.1 - a.1;
    let len = (dx * dx + dy * dy).sqrt();
    if !len.is_finite() || len < 20.0 {
        return;
    }
    let (ux, uy) = (dx / len, dy / len);
    let (nx, ny) = (-uy, ux);
    let count = (len / 20.0).floor() as usize;
    for index in 1..=count {
        let base = (
            a.0 + ux * 20.0 * index as f64,
            a.1 + uy * 20.0 * index as f64,
        );
        let tip = (base.0 + nx * 5.0, base.1 + ny * 5.0);
        if !base.0.is_finite() || !tip.0.is_finite() {
            continue;
        }
        let (pa, pb) = (
            session.screen_from_world(base, rect),
            session.screen_from_world(tip, rect),
        );
        if pa.x.is_finite() && pb.x.is_finite() {
            painter.line_segment([pa, pb], tick);
        }
    }
}

/// Dibuja la cortina visible: rect que cubre la hoja desde el lado que
/// indica la dirección, con el borde libre marcado (línea de acento +
/// pastilla de agarre). Puro dibujo sobre la vista (espacio de pantalla,
/// inmune a pan/zoom): jamás toca el documento (PROHIBIDO persistir
/// overlays). Sin panic: cortina inválida o vista degenerada → nada.
fn draw_curtain_overlay(
    painter: &egui::Painter,
    session: &WhiteboardSession,
    rect: Rect,
    theme: &grafito_ui::theme::Theme,
) {
    let Some(curtain) = session.curtain else {
        return;
    };
    if !curtain.is_usable() {
        return;
    }
    let height = rect.height();
    if height <= 0.0 || !rect.min.y.is_finite() || !rect.min.x.is_finite() {
        return;
    }
    let edge_y = rect.min.y + (curtain.edge as f32) * height;
    if !edge_y.is_finite() {
        return;
    }
    // Zona cubierta según la dirección (vacía al revelar del todo: igual
    // se dibuja el borde para poder volver a cubrir arrastrándolo).
    let cover = match curtain.direction {
        CurtainDirection::Top => Rect::from_min_max(rect.min, pos2(rect.max.x, edge_y)),
        CurtainDirection::Bottom => Rect::from_min_max(pos2(rect.min.x, edge_y), rect.max),
    };
    if cover.height() > 0.0 && cover.width() > 0.0 {
        painter.rect_filled(cover, 0.0, theme.panel_bg);
    }
    let edge = Stroke::new(2.0, theme.accent);
    painter.line_segment([pos2(rect.min.x, edge_y), pos2(rect.max.x, edge_y)], edge);
    let pill = Rect::from_center_size(pos2(rect.center().x, edge_y), vec2(72.0, 12.0));
    painter.rect_filled(pill, 6.0, theme.panel_bg);
    painter.rect_stroke(pill, 6.0, edge);
}

/// Marca del modo lupa: anillo en el cursor con el factor (zoom-to-point
/// honesto, ver `MAGNIFIER_FACTOR`). Mismo patrón que el círculo de la
/// goma; sin panic.
fn draw_magnifier_hint(
    ui: &mut egui::Ui,
    session: &WhiteboardSession,
    rect: Rect,
    theme: &grafito_ui::theme::Theme,
) {
    if !session.magnifier_active {
        return;
    }
    let pos = ui.input(|i| i.pointer.hover_pos().or(i.pointer.interact_pos()));
    let Some(pos) = pos else {
        return;
    };
    if !rect.contains(pos) {
        return;
    }
    ui.painter()
        .circle_stroke(pos, 40.0, Stroke::new(1.5, theme.accent));
    ui.painter()
        .circle_filled(pos, 2.0, theme.accent.gamma_multiply(0.9));
    ui.painter().text(
        pos + vec2(10.0, -30.0),
        egui::Align2::LEFT_TOP,
        "2\u{d7}",
        egui::FontId::monospace(12.0),
        theme.text_secondary,
    );
}

/// Decodifica PNG/JPEG a `ColorImage` egui con el crate `image` ya
/// declarado en grafito-app (sin deps nuevas). `None` honesto si no
/// decodifica: el canvas omite la imagen sin romper la hoja. Capa de
/// defensa extra en píxeles (el modelo ya acota a 2048 por lado).
fn decode_whiteboard_image(bytes: &[u8]) -> Option<egui::ColorImage> {
    let dynamic = image::load_from_memory(bytes).ok()?;
    let rgba = dynamic.to_rgba8();
    let (width, height) = (rgba.width() as usize, rgba.height() as usize);
    if width == 0 || height == 0 || width > 2048 || height > 2048 {
        return None;
    }
    let raw = rgba.into_raw();
    if raw.len() != width.saturating_mul(height).saturating_mul(4) {
        return None;
    }
    Some(egui::ColorImage::from_rgba_unmultiplied(
        [width, height],
        &raw,
    ))
}

fn draw_element(
    painter: &egui::Painter,
    element: &WhiteboardElement,
    rect: Rect,
    session: &WhiteboardSession,
    theme: &grafito_ui::theme::Theme,
) {
    // Stroke para formas: usar text_primary para visibilidad en ambos temas (claro sobre oscuro, oscuro sobre claro)
    let shape_stroke = Stroke::new(
        1.8 * session.zoom as f32,
        theme.text_primary.gamma_multiply(0.92),
    );
    let text_color = theme.text_primary;
    match element {
        WhiteboardElement::Stroke {
            points,
            width,
            color,
        }
        | WhiteboardElement::Highlighter {
            points,
            width,
            color,
        } => {
            let highlight = matches!(element, WhiteboardElement::Highlighter { .. });
            let raw = Color32::from_rgb(color.0, color.1, color.2);
            // Adaptación para contraste: si el trazo almacenado es claro sobre canvas claro (o oscuro sobre oscuro), usa text_primary
            let is_light_canvas = theme.canvas_bg.r() > 200;
            let col_is_light = raw.r() > 200 && raw.g() > 200 && raw.b() > 200;
            let col_is_dark = raw.r() < 60 && raw.g() < 60 && raw.b() < 60;
            let col = if is_light_canvas && col_is_light {
                theme.text_primary
            } else if !is_light_canvas && col_is_dark {
                // theme.text_primary en dark es claro (FAFAF9) — históricamente 240,241,244
                theme.text_primary
            } else {
                raw
            };
            // El resaltado se pinta translúcido (alfa fijo del modelo) y
            // queda debajo por orden de documento (inserción al fondo).
            let col = if highlight {
                col.gamma_multiply(HIGHLIGHTER_ALPHA)
            } else {
                col
            };
            for pair in points.windows(2) {
                if !pair[0].0.is_finite() || !pair[1].0.is_finite() {
                    continue;
                }
                let a = session.screen_from_world(pair[0], rect);
                let b = session.screen_from_world(pair[1], rect);
                if !a.x.is_finite() || !b.x.is_finite() {
                    continue;
                }
                painter.line_segment(
                    [a, b],
                    Stroke::new((*width as f32) * session.zoom as f32, col),
                );
            }
        }
        WhiteboardElement::Rectangle { min, max, .. } => {
            if !min.0.is_finite() || !max.0.is_finite() {
                return;
            }
            let a = session.screen_from_world(*min, rect);
            let b = session.screen_from_world(*max, rect);
            if a.x.is_finite() && b.x.is_finite() {
                painter.rect_stroke(Rect::from_two_pos(a, b), RADIUS_MD, shape_stroke);
            }
        }
        WhiteboardElement::Ellipse { center, rx, .. } => {
            if !center.0.is_finite() || !rx.is_finite() {
                return;
            }
            let radius = (*rx as f32 * session.zoom as f32).max(2.0);
            let c = session.screen_from_world(*center, rect);
            if c.x.is_finite() {
                painter.circle_stroke(c, radius, shape_stroke);
            }
        }
        WhiteboardElement::Arrow { from, to } => {
            if !from.0.is_finite() || !to.0.is_finite() {
                return;
            }
            let a = session.screen_from_world(*from, rect);
            let b = session.screen_from_world(*to, rect);
            if a.x.is_finite() && b.x.is_finite() {
                painter.line_segment([a, b], shape_stroke);
                let wing = arrow_tip(*from, *to, 0.55);
                let wing_a = session.screen_from_world(wing.0, rect);
                let wing_b = session.screen_from_world(wing.1, rect);
                if wing_a.x.is_finite() {
                    painter.line_segment([wing_a, b], shape_stroke);
                    painter.line_segment([wing_b, b], shape_stroke);
                }
            }
        }
        WhiteboardElement::Text { at, text, size } => {
            if !text.is_empty() && at.0.is_finite() && size.is_finite() {
                let font = egui::FontId::monospace((*size as f32) * session.zoom as f32);
                let pos = session.screen_from_world(*at, rect);
                if pos.x.is_finite() {
                    painter.text(pos, egui::Align2::LEFT_TOP, text, font, text_color);
                }
            }
        }
        WhiteboardElement::Image {
            rect: image_rect, ..
        } => {
            // Fallback vectorial (marco): el raster real lo dibuja `draw()`
            // con textura cacheada; este brazo solo cubre el preview.
            if !image_rect.0 .0.is_finite() || !image_rect.1 .0.is_finite() {
                return;
            }
            let a = session.screen_from_world(image_rect.0, rect);
            let b = session.screen_from_world(image_rect.1, rect);
            if a.x.is_finite() && b.x.is_finite() {
                painter.rect_stroke(Rect::from_two_pos(a, b), RADIUS_MD, shape_stroke);
            }
        }
    }
}

/// Contenido interno de la toolbar mega simple (mover, escribir, linea, cerrar).
/// Sin desbordes: solo 4 botones, sin separadores extra ni scroll.
#[allow(clippy::too_many_arguments)]
/// Acción Compartir pedida desde la toolbar (S5): el efecto (portapapeles /
/// archivo) lo aplica el llamador, fuera del `Area`, para no prestar `app`
/// dos veces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WhiteboardShareAction {
    /// Copiar el SVG de la hoja actual al portapapeles OS (`ctx.copy_text`,
    /// con respaldo arboard del integration en desktop; sin deps nuevas).
    CopySvg,
    /// Guardar la hoja actual como archivo `.svg` (diálogo nativo + write
    /// en background vía `spawn_whiteboard_svg_write`).
    SaveSvg,
    /// Guardar la hoja actual como archivo `.png` (raster headless
    /// `whiteboard_png` a `WHITEBOARD_PNG_EXPORT_SCALE` + diálogo nativo +
    /// write binario en background; avisa si omite texto).
    SavePng,
}

/// Botones Compartir de la pizarra (S5): van tras el Cerrar en desktop y
/// táctil (copiar SVG, guardar SVG, guardar PNG). El PNG se rasteriza con
/// el `whiteboard_png` headless a `WHITEBOARD_PNG_EXPORT_SCALE`; omite el
/// texto (aviso honesto en el guardado) y el SVG sigue siendo la vía
/// completa.
fn draw_share_buttons(ui: &mut egui::Ui, tool_btn: egui::Vec2) -> Option<WhiteboardShareAction> {
    let theme = current_theme(ui.ctx());
    ui.separator();
    if action_icon_button_sized(
        ui,
        Icon::Copy,
        theme.text_secondary,
        "Compartir: copiar la hoja como SVG",
        tool_btn,
    )
    .clicked()
    {
        return Some(WhiteboardShareAction::CopySvg);
    }
    if ui
        .button("Guardar")
        .on_hover_text("Guardar la hoja como archivo .svg")
        .clicked()
    {
        return Some(WhiteboardShareAction::SaveSvg);
    }
    if ui
        .button("PNG")
        .on_hover_text("Guardar la hoja como archivo .png (el PNG no incluye texto; el SVG sí)")
        .clicked()
    {
        return Some(WhiteboardShareAction::SavePng);
    }
    None
}

/// Botón Pegar de la pizarra: trae la imagen del portapapeles OS con
/// arboard directo (`get_image` → RGBA → PNG en memoria vía `image`, ya
/// declarado) y la inserta por el MISMO camino que archivo (`new_image` +
/// límites 4 MiB/2048, ver `whiteboard_clipboard`). Vacío o sin imagen →
/// toast honesto con la causa, jamás imagen rota. Botón de texto como
/// Guardar/PNG (sin icono nuevo en `grafito-ui`).
fn draw_paste_button(ui: &mut egui::Ui) -> bool {
    ui.button("Pegar")
        .on_hover_text(
            "Pegar la imagen del portapapeles (misma cota que archivo: 2048px por lado, 4 MiB)",
        )
        .clicked()
}

/// Botón Imagen de la pizarra: inserta PNG/JPEG por archivo con el diálogo
/// nativo existente (`pick_file_native`, filtro png/jpg/jpeg). El pegado
/// desde el portapapeles vive en el botón Pegar vecino
/// (`draw_paste_button` + `whiteboard_clipboard`).
fn draw_image_button(ui: &mut egui::Ui, tool_btn: egui::Vec2) -> bool {
    let theme = current_theme(ui.ctx());
    action_icon_button_sized(
        ui,
        Icon::Image,
        theme.text_secondary,
        "Insertar imagen PNG o JPEG (máx. 2048px por lado, 4 MiB)",
        tool_btn,
    )
    .clicked()
}

/// Botón `Fórmula` de la pizarra: abre el diálogo de texto (la inserción como
/// `Image` solo ocurre si el raster produce PNG; si no, avisa honesto y jamás
/// deja imagen rota — ver `whiteboard_formula.rs`). `Icon::Function` ya
/// existe en `grafito-ui`: sin iconos ni binarios nuevos.
fn draw_formula_button(ui: &mut egui::Ui, tool_btn: egui::Vec2) -> bool {
    let theme = current_theme(ui.ctx());
    action_icon_button_sized(
        ui,
        Icon::Function,
        theme.text_secondary,
        "Fórmula: escribir texto y pegarla como imagen (si no rasteriza, avisa)",
        tool_btn,
    )
    .clicked()
}

/// Botón de instrumento estilo OpenBoard (regla/escuadra/transportador/
/// compás): conmuta el overlay del slot único (un instrumento visible a la
/// vez; clic de nuevo lo guarda). Sin icono nuevo en `grafito-ui` (iconos
/// existentes) y sin claves i18n nuevas (tooltips en español como el resto
/// de la toolbar, 6 locales intactos).
fn draw_instrument_button(
    ui: &mut egui::Ui,
    session: &mut WhiteboardSession,
    icon: Icon,
    tip: &str,
    tool_btn: egui::Vec2,
    active: bool,
    toggle: impl FnOnce(&mut WhiteboardSession),
) {
    let theme = current_theme(ui.ctx());
    if action_icon_button_sized(
        ui,
        icon,
        if active {
            theme.accent
        } else {
            theme.text_secondary
        },
        tip,
        tool_btn,
    )
    .clicked()
    {
        toggle(session);
    }
}

/// Fila de instrumentos (slot único + compás) compartida por desktop y táctil.
fn draw_instrument_buttons(
    ui: &mut egui::Ui,
    session: &mut WhiteboardSession,
    tool_btn: egui::Vec2,
) {
    ui.spacing_mut().item_spacing.x = SPACE_XS;
    let kind = session.instrument_kind();
    draw_instrument_button(
        ui,
        session,
        Icon::Line,
        "Regla (el lápiz se pega al borde; arrastrar mueve, manija rota; clic de nuevo la guarda)",
        tool_btn,
        kind == Some(InstrumentKind::Ruler),
        |session| session.toggle_instrument(InstrumentKind::Ruler),
    );
    draw_instrument_button(
        ui,
        session,
        Icon::RegularPolygon,
        "Escuadra (el lápiz se pega al borde; arrastrar mueve, manija rota; clic de nuevo la guarda)",
        tool_btn,
        kind == Some(InstrumentKind::Triangle),
        |session| session.toggle_instrument(InstrumentKind::Triangle),
    );
    draw_instrument_button(
        ui,
        session,
        Icon::Angle,
        "Transportador (el lápiz se pega al borde; arrastrar mueve, manija rota; clic de nuevo lo guarda)",
        tool_btn,
        kind == Some(InstrumentKind::Protractor),
        |session| session.toggle_instrument(InstrumentKind::Protractor),
    );
    let compass = session.compass_active;
    draw_instrument_button(
        ui,
        session,
        Icon::DrawCompass,
        "Compás (clic centro, clic radio → círculo exacto; clic de nuevo lo guarda)",
        tool_btn,
        compass,
        WhiteboardSession::toggle_compass,
    );
}

/// Selector "Fondo" (4 opciones) + menú "Figuras" (5 stencils) de la
/// pizarra, compartido por desktop y táctil. Sin iconos nuevos en
/// `grafito-ui` (texto) ni claves i18n nuevas (español hardcoded como el
/// resto de la toolbar, 6 locales intactos). "Figuras" es menú de acción:
/// al elegir se inserta el stencil al centro de la vista como elementos
/// normales (ver `WhiteboardSession::insert_figure`).
fn draw_background_and_figures_buttons(ui: &mut egui::Ui, session: &mut WhiteboardSession) {
    egui::ComboBox::from_label("Fondo")
        .selected_text(session.background.label())
        .show_ui(ui, |ui| {
            for option in WhiteboardBackground::ALL {
                ui.selectable_value(&mut session.background, option, option.label());
            }
        });
    let mut picked: Option<FigureStencil> = None;
    egui::ComboBox::from_label("Figuras")
        .selected_text("Figuras")
        .show_ui(ui, |ui| {
            for option in FigureStencil::ALL {
                if ui.selectable_label(false, option.label()).clicked() {
                    picked = Some(option);
                }
            }
        });
    if let Some(kind) = picked {
        if let Err(message) = session.insert_figure(kind) {
            session.last_error = Some(message);
        }
    }
}
/// Cortina + lupa estilo OpenBoard (slot único con instrumentos/compás):
/// 2 botones con iconos reutilizados (`Eye`/`EyeOff`, `Search`), sin icono
/// nuevo en `grafito-ui` ni claves i18n nuevas (tooltips en español como el
/// resto de la toolbar).
fn draw_curtain_lupa_buttons(
    ui: &mut egui::Ui,
    session: &mut WhiteboardSession,
    tool_btn: egui::Vec2,
) {
    ui.spacing_mut().item_spacing.x = SPACE_XS;
    let theme = current_theme(ui.ctx());
    let curtain_on = session.curtain.is_some();
    if action_icon_button_sized(
        ui,
        if curtain_on { Icon::Eye } else { Icon::EyeOff },
        if curtain_on {
            theme.accent
        } else {
            theme.text_secondary
        },
        "Cortina (cubre la hoja; arrastrar el borde revela; clic en la zona cubierta cambia arriba/abajo; clic de nuevo la oculta)",
        tool_btn,
    )
    .clicked()
    {
        session.toggle_curtain();
    }
    let lupa = session.magnifier_active;
    if action_icon_button_sized(
        ui,
        Icon::Search,
        if lupa {
            theme.accent
        } else {
            theme.text_secondary
        },
        "Lupa 2× (clic acerca centrado en el punto; zoom-to-point con el clamp de zoom de la hoja; clic de nuevo la apaga)",
        tool_btn,
    )
    .clicked()
    {
        session.toggle_magnifier();
    }
}
/// SVG de la hoja actual del libro (ya sincronizado sesión→libro este
/// frame). `Err` honesto si no hay hoja o excede el presupuesto S5.
fn current_page_svg(app: &crate::GrafitoApp) -> Result<String, String> {
    let Some(page) = app.whiteboard_book.current() else {
        return Err("No hay hoja activa en la pizarra.".to_string());
    };
    page.doc.to_svg(800, 600)
}

/// Aviso honesto del botón PNG cuando la hoja trae texto (el raster omite
/// glifos; el SVG es la vía completa). Const compartida botón↔test: una
/// sola ortografía, sin claves i18n nuevas.
pub(crate) const WHITEBOARD_PNG_TEXT_WARNING: &str =
    "El PNG no incluye texto; el SVG sí lo trae completo.";

/// Render puro del PNG de un documento a `WHITEBOARD_PNG_EXPORT_SCALE`
/// con el fondo dado (el botón pasa `theme.canvas_bg`): raster + encode,
/// sin I/O, sin egui. Testeable headless. `grid` refleja el fondo de la
/// página (solo `Grid` lo trae; Rayado/Punteado/Blanco salen lisos: el
/// raster solo conoce la cuadrícula de 40u).
pub(crate) fn render_current_page_png(
    doc: &WhiteboardDoc,
    background: (u8, u8, u8),
    grid: bool,
) -> Result<Vec<u8>, String> {
    use crate::whiteboard_png::{
        encode_png_rgba, render_png_rgba, PngOptions, WHITEBOARD_PNG_EXPORT_SCALE,
    };
    let (width, height, rgba) = render_png_rgba(
        doc,
        WHITEBOARD_PNG_EXPORT_SCALE,
        &PngOptions { background, grid },
    )?;
    encode_png_rgba(width, height, &rgba)
}

/// Guarda la hoja actual como `.png`: render a
/// `WHITEBOARD_PNG_EXPORT_SCALE` con el `canvas_bg` del tema, diálogo
/// nativo existente y write binario atómico en worker (mismo patrón que
/// `spawn_whiteboard_svg_write`, cuyo summary publica `PendingExportJob`).
/// Sin hoja → toast de error; fallo de render/encode → toast con la causa;
/// con texto en la hoja → aviso honesto (el PNG lo omite, el SVG no).
fn save_current_page_png(app: &mut crate::GrafitoApp, ctx: &egui::Context) {
    use grafito_ui::toast::ToastKind;
    let Some(doc) = app.whiteboard_book.current().map(|page| page.doc.clone()) else {
        app.notify("No hay hoja activa en la pizarra.", ToastKind::Error);
        return;
    };
    let background = current_theme(ctx).canvas_bg;
    // Solo el cuadriculado viaja al PNG (el raster no conoce rayado ni
    // punteado: salen lisos, igual que el Blanco).
    let grid = app.whiteboard.background == grafito_whiteboard::WhiteboardBackground::Grid;
    let bytes =
        match render_current_page_png(&doc, (background.r(), background.g(), background.b()), grid)
        {
            Ok(bytes) => bytes,
            Err(error) => {
                app.notify(error, ToastKind::Error);
                return;
            }
        };
    let skips_text = crate::whiteboard_png::png_skips_text(&doc);
    let Some(path) = app.save_file_native("PNG", &["png"], Some("pizarra.png")) else {
        return;
    };
    let repaint = egui::Context::clone(ctx);
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    let _ = std::thread::Builder::new()
        .name("whiteboard-png".into())
        .spawn(move || {
            let result = crate::export::write_file_atomic(&path, &bytes)
                .map(|()| {
                    let summary = format!("Pizarra guardada como PNG -> {}", path.display());
                    (path.clone(), summary)
                })
                .map_err(|error| error.to_string());
            let _ = tx.send(result);
            repaint.request_repaint();
        });
    app.pending_export_job = Some(crate::app::PendingExportJob { receiver: rx });
    app.notify("Guardando PNG…", ToastKind::Info);
    if skips_text {
        app.notify(WHITEBOARD_PNG_TEXT_WARNING, ToastKind::Info);
    }
}

/// Inserta una imagen PNG/JPEG por archivo con el diálogo nativo existente.
/// Lectura síncrona pero acotada (metadata ≤4 MiB antes del `read`: cero OOM
/// en el UI thread); validación del tipo (lado/bytes/formato) con `Err` en
/// español vía toast. El rect inicial es 1:1 en píxeles al centro de la
/// vista (honesto, sin reescalado silencioso); se ajusta con Mover/manija.
/// La imagen queda seleccionada para moverla al momento.
fn insert_whiteboard_image_from_file(app: &mut crate::GrafitoApp) {
    use grafito_ui::toast::ToastKind;
    let Some(path) = app.pick_file_native("Imagen", &["png", "jpg", "jpeg"]) else {
        return;
    };
    let size = std::fs::metadata(&path)
        .map(|meta| meta.len())
        .unwrap_or(u64::MAX);
    if size > grafito_whiteboard::MAX_IMAGE_BYTES as u64 {
        app.notify(
            format!(
                "La imagen supera el máximo de {} bytes; achicala fuera y reintentá",
                grafito_whiteboard::MAX_IMAGE_BYTES
            ),
            ToastKind::Error,
        );
        return;
    }
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(_) => {
            app.notify("No se pudo leer el archivo de imagen.", ToastKind::Error);
            return;
        }
    };
    let (width, height) = match grafito_whiteboard::validate_image_bytes(&bytes) {
        Ok(dims) => dims,
        Err(causa) => {
            app.notify(format!("La imagen no entra: {causa}"), ToastKind::Error);
            return;
        }
    };
    let zoom = if app.whiteboard.zoom.is_finite() && app.whiteboard.zoom > 0.0 {
        app.whiteboard.zoom
    } else {
        1.0
    };
    let center = (-app.whiteboard.pan.0 / zoom, -app.whiteboard.pan.1 / zoom);
    let rect = (
        (
            center.0 - f64::from(width) / 2.0,
            center.1 - f64::from(height) / 2.0,
        ),
        (
            center.0 + f64::from(width) / 2.0,
            center.1 + f64::from(height) / 2.0,
        ),
    );
    let element = match WhiteboardElement::new_image(bytes, rect) {
        Ok(element) => element,
        Err(causa) => {
            app.notify(format!("La imagen no entra: {causa}"), ToastKind::Error);
            return;
        }
    };
    match app.whiteboard.try_add(element) {
        Ok(()) => {
            app.whiteboard.doc.select_at(center, f64::INFINITY);
            app.notify("Imagen insertada.", ToastKind::Success);
        }
        Err(mensaje) => app.notify(mensaje, ToastKind::Error),
    }
}

/// Burbuja de herramientas de la pizarra (herramientas + cerrar + compartir).
/// Los out-params `_clear`/`_ask_ai`/etc. son placeholders de wiring futuro
/// (la firma ya estaba en 8 antes de S5); el `allow` evita el deny del gate.
#[allow(clippy::too_many_arguments)]
fn draw_toolbar_contents(
    ui: &mut egui::Ui,
    app: &mut crate::GrafitoApp,
    selected_tool: &mut Option<WhiteboardTool>,
    _clear: &mut bool,
    close: &mut bool,
    _ask_ai: &mut bool,
    _toggle_assistant: &mut bool,
    _toggle_palette: &mut bool,
    insert_image: &mut bool,
    paste_image: &mut bool,
) -> Option<WhiteboardShareAction> {
    let theme = current_theme(ui.ctx());
    let tool = app.whiteboard.tool;
    // Táctil (<740): botones de 44px (S2); desktop intacto 26x24.
    let touch_wb =
        ui.ctx().screen_rect().width() < grafito_ui::tokens::PALETTE_BOTTOM_SHEET_BREAKPOINT;
    let tool_btn = if touch_wb {
        egui::vec2(44.0, 44.0)
    } else {
        egui::vec2(26.0, 24.0)
    };
    // 8 herramientas + cerrar — burbuja compacta centrada (pro: formas + texto)
    let tools = [
        (Icon::Move, WhiteboardTool::Select, "Mover"),
        (Icon::Pencil, WhiteboardTool::Pencil, "Escribir"),
        // Sin icono nuevo en `grafito-ui`: `Edit` (marcador) distingue el
        // resaltado del lápiz con el tooltip honesto.
        (
            Icon::Edit,
            WhiteboardTool::Highlighter,
            "Resaltador (translúcido, debajo de la tinta)",
        ),
        (Icon::ArrowRight, WhiteboardTool::Arrow, "Flecha/Línea"),
        (Icon::Shapes, WhiteboardTool::Rectangle, "Rectángulo"),
        (Icon::Ellipse, WhiteboardTool::Ellipse, "Elipse"),
        (Icon::Notebook, WhiteboardTool::Text, "Texto (clic escribe)"),
        (Icon::Eraser, WhiteboardTool::Eraser, "Goma (ver círculo)"),
    ];
    let mut share_out: Option<WhiteboardShareAction> = None;
    let mut draw_tool_btn = |ui: &mut egui::Ui, icon: Icon, t: WhiteboardTool, tip: &str| {
        let selected = tool == t;
        if action_icon_button_sized(
            ui,
            icon,
            if selected {
                theme.accent
            } else {
                theme.text_secondary
            },
            tip,
            tool_btn,
        )
        .clicked()
        {
            *selected_tool = Some(t);
        }
    };
    // Táctil angosto: 2 filas × 4 todo visible (una fila = 8×44+gaps ≈
    // 390 px > 320: el Cerrar caía fuera y el scroll oculto nadie lo
    // descubría). 2.ª fila: 3 herramientas + separador + Cerrar ≈ 244 px.
    // 3.ª fila: instrumentos OpenBoard (regla/escuadra/transportador/
    // compás, slot único). Desktop: una fila directa, intacta.
    if touch_wb {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = SPACE_XS;
            for (ri, chunk) in tools.chunks(4).enumerate() {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = SPACE_XS;
                    for &(icon, t, tip) in chunk {
                        draw_tool_btn(ui, icon, t, tip);
                    }
                    if ri == 1 {
                        ui.separator();
                        if action_icon_button_sized(
                            ui,
                            Icon::Close,
                            theme.text_secondary,
                            "Cerrar (Esc)",
                            tool_btn,
                        )
                        .clicked()
                        {
                            *close = true;
                        }
                        if draw_image_button(ui, tool_btn) {
                            *insert_image = true;
                        }
                        if draw_paste_button(ui) {
                            *paste_image = true;
                        }
                        // Gateado por raster disponible: sin raster el botón no
                        // se muestra (clic→error es peor UX que andamio oculto).
                        if grafito_whiteboard::formula::formula_raster_available()
                            && draw_formula_button(ui, tool_btn)
                        {
                            let mut dialog =
                                crate::whiteboard_formula::formula_dialog_get(ui.ctx());
                            dialog.open = true;
                            crate::whiteboard_formula::formula_dialog_set(ui.ctx(), dialog);
                        }
                        share_out = draw_share_buttons(ui, tool_btn);
                    }
                });
            }
            ui.horizontal(|ui| {
                draw_instrument_buttons(ui, &mut app.whiteboard, tool_btn);
                draw_curtain_lupa_buttons(ui, &mut app.whiteboard, tool_btn);
            });
            ui.horizontal(|ui| {
                draw_background_and_figures_buttons(ui, &mut app.whiteboard);
            });
        });
        return share_out;
    }
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = SPACE_XS;
        for (icon, t, tip) in tools {
            draw_tool_btn(ui, icon, t, tip);
        }
        ui.separator();
        draw_instrument_buttons(ui, &mut app.whiteboard, tool_btn);
        draw_curtain_lupa_buttons(ui, &mut app.whiteboard, tool_btn);
        draw_background_and_figures_buttons(ui, &mut app.whiteboard);
        ui.separator();
        if action_icon_button_sized(
            ui,
            Icon::Close,
            theme.text_secondary,
            "Cerrar (Esc)",
            tool_btn,
        )
        .clicked()
        {
            *close = true;
        }
        if draw_image_button(ui, tool_btn) {
            *insert_image = true;
        }
        if draw_paste_button(ui) {
            *paste_image = true;
        }
        // Gateado por raster disponible (ver rama táctil).
        if grafito_whiteboard::formula::formula_raster_available()
            && draw_formula_button(ui, tool_btn)
        {
            let mut dialog = crate::whiteboard_formula::formula_dialog_get(ui.ctx());
            dialog.open = true;
            crate::whiteboard_formula::formula_dialog_set(ui.ctx(), dialog);
        }
        share_out = draw_share_buttons(ui, tool_btn);
    });
    share_out
}

/// Compara libros persistidos por contenido (título, vista y elementos).
fn whiteboard_pages_equal(
    first: &[grafito_core::WhiteboardPageData],
    second: &[grafito_core::WhiteboardPageData],
) -> bool {
    if first.len() != second.len() {
        return false;
    }
    first.iter().zip(second.iter()).all(|(a, b)| {
        a.title == b.title
            && a.pan == b.pan
            && a.zoom == b.zoom
            && whiteboard_docs_equal(&a.doc, &b.doc)
    })
}

/// El libro en memoria coincide con lo persistido (páginas e índice actual).
fn whiteboard_book_matches_document(book: &WhiteboardBook, document: &Document) -> bool {
    let (persisted, current) = document.whiteboard_export_pages();
    book.current == current && whiteboard_pages_equal(&book.to_persisted(), &persisted)
}

/// Snapshot undo con evicción acotada (`MAX_UNDO` / `MAX_UNDO_BYTES`);
/// helper compartido por la ruta legada y la del libro (un solo snapshot
/// por gesto). Sin panic: las pilas vacías cortan los bucles.
fn snapshot_whiteboard_undo(
    document: &Document,
    undo_stack: &mut VecDeque<Document>,
    redo_stack: &mut VecDeque<ChangeSet>,
    undo_total_bytes: &mut usize,
) {
    let snapshot = document.clone();
    let bytes = snapshot.estimated_bytes();
    undo_stack.push_back(snapshot);
    redo_stack.clear();
    *undo_total_bytes = undo_total_bytes.saturating_add(bytes);
    while undo_stack.len() > crate::app::MAX_UNDO {
        if let Some(front) = undo_stack.pop_front() {
            *undo_total_bytes = undo_total_bytes.saturating_sub(front.estimated_bytes());
        } else {
            break;
        }
    }
    while *undo_total_bytes > crate::app::MAX_UNDO_BYTES && undo_stack.len() > 1 {
        if let Some(front) = undo_stack.pop_front() {
            *undo_total_bytes = undo_total_bytes.saturating_sub(front.estimated_bytes());
        } else {
            break;
        }
    }
}

/// Carga el libro completo en memoria y la hoja actual en la sesión.
/// Si el documento es legado (solo `Document.whiteboard`), migra en memoria
/// a un libro de una hoja sin pérdida; el volcado posterior lo persiste.
pub fn load_book_from_document(
    session: &mut WhiteboardSession,
    book: &mut WhiteboardBook,
    document: &Document,
) {
    let (pages, current) = document.whiteboard_export_pages();
    *book = WhiteboardBook::from_persisted(&pages, current);
    book.load_to_session(session);
}

/// Carga `Document.whiteboard` en la sesión y la hoja actual del libro.
/// Se usa al abrir la pizarra, tras abrir/crear documento y tras undo/redo.
/// (Compat: delega en `load_book_from_document`, que trae el libro completo.)
pub fn load_whiteboard_into_session(
    session: &mut WhiteboardSession,
    book: &mut WhiteboardBook,
    document: &Document,
) {
    load_book_from_document(session, book, document);
}

/// Guarda la sesión en `Document.whiteboard` con snapshot undo previo.
/// Marca el documento sucio vía baseline semántico (la pizarra se serializa
/// en el documento). Devuelve `true` si hubo cambios que volcar.
/// (Compat: la ruta principal `push_whiteboard_book_and_store` la reutiliza
/// para el espejo de la hoja actual con un único snapshot por gesto.)
pub fn push_whiteboard_undo_and_store(
    session_doc: &WhiteboardDoc,
    document: &mut Document,
    undo_stack: &mut VecDeque<Document>,
    redo_stack: &mut VecDeque<ChangeSet>,
    undo_total_bytes: &mut usize,
) -> bool {
    if whiteboard_docs_equal(session_doc, &document.whiteboard) {
        return false;
    }
    snapshot_whiteboard_undo(document, undo_stack, redo_stack, undo_total_bytes);
    document.whiteboard = session_doc.clone();
    document.bump_version();
    document.invalidate_all_caches();
    true
}

/// Guarda el libro completo (hojas 1..=32 + índice actual) en `Document`
/// con un único snapshot undo. Sincroniza la sesión a la hoja actual,
/// compara con lo persistido y escribe vía `set_whiteboard_book` (que
/// actualiza el espejo `whiteboard` para el export SVG existente).
/// Devuelve `true` si hubo cambios que volcar.
pub fn push_whiteboard_book_and_store(
    book: &mut WhiteboardBook,
    session: &WhiteboardSession,
    document: &mut Document,
    undo_stack: &mut VecDeque<Document>,
    redo_stack: &mut VecDeque<ChangeSet>,
    undo_total_bytes: &mut usize,
) -> bool {
    book.save_current_from_session(session);
    let pages = book.to_persisted();
    let (persisted, persisted_current) = document.whiteboard_export_pages();
    if book.current == persisted_current
        && whiteboard_pages_equal(&pages, &persisted)
        && !document.whiteboard_pages.is_empty()
    {
        return false;
    }
    // Un solo snapshot por gesto: la ruta legada lo toma si cambió la hoja
    // actual; si solo cambió el índice (o es migración de legado), se toma aquí.
    let current_doc = book
        .current()
        .map(|page| page.doc.clone())
        .unwrap_or_default();
    if !push_whiteboard_undo_and_store(
        &current_doc,
        document,
        undo_stack,
        redo_stack,
        undo_total_bytes,
    ) {
        snapshot_whiteboard_undo(document, undo_stack, redo_stack, undo_total_bytes);
    }
    // `set_whiteboard_book` solo rechaza libro vacío o >32 hojas, imposible
    // aquí (el libro siempre tiene 1..=32); si fallara, no hay panic.
    document.set_whiteboard_book(pages, book.current).is_ok()
}

/// Recarga la sesión desde el documento si difieren (apertura, undo externo).
pub fn sync_whiteboard_from_document(app: &mut crate::GrafitoApp) {
    if whiteboard_docs_equal(&app.whiteboard.doc, &app.document.whiteboard) {
        return;
    }
    load_whiteboard_into_session(&mut app.whiteboard, &mut app.whiteboard_book, &app.document);
}

/// Vuelca sesión y libro al documento si algo cambió (edición del frame,
/// cambio de hoja o migración de legado pendiente). Hace un único snapshot
/// undo y deja el libro en memoria sincronizado.
///
/// `before_revision` es la revisión del doc de sesión al empezar el frame:
/// comparar contadores evita clonar/comparar todos los elementos por frame.
pub fn commit_whiteboard_to_document(app: &mut crate::GrafitoApp, before_revision: u64) -> bool {
    let session_changed = app.whiteboard.doc.revision() != before_revision;
    // Sincroniza la sesión a la hoja actual solo si cambió (sin clones en idle).
    app.whiteboard_book
        .save_current_from_session(&app.whiteboard);
    // Legado con contenido (o varias hojas) aún sin `Vec`: hay que migrarlo
    // a disco aunque el espejo ya coincida.
    let needs_migration_write = app.document.whiteboard_pages.is_empty()
        && (app.whiteboard_book.len() > 1 || !app.whiteboard.doc.is_empty());
    // Fast-path idle: sin edición de sesión, sin migración y con el libro ya
    // sincronizado, no hace falta comparar páginas (clonaba hasta 32 hojas).
    if !session_changed && !needs_migration_write && app.whiteboard_book.is_synced() {
        return false;
    }
    if !session_changed
        && !needs_migration_write
        && whiteboard_book_matches_document(&app.whiteboard_book, &app.document)
    {
        app.whiteboard_book.mark_synced();
        return false;
    }
    let wrote = push_whiteboard_book_and_store(
        &mut app.whiteboard_book,
        &app.whiteboard,
        &mut app.document,
        &mut app.undo_stack,
        &mut app.redo_stack,
        &mut app.undo_total_bytes,
    );
    // Si no escribió, confirmar con la comparación profunda antes de marcar
    // sincronizado (un fallo del set no debe silenciar el próximo commit).
    if wrote || whiteboard_book_matches_document(&app.whiteboard_book, &app.document) {
        app.whiteboard_book.mark_synced();
    }
    wrote
}

// Export SVG del libro: ver `grafito_core::whiteboard_pages_to_svg` (proyección
// pura del libro persistido, un `.svg` por hoja). El export existente de la
// hoja actual sigue en `export.rs` leyendo el espejo `Document.whiteboard`.

/// Undo/redo con la pizarra abierta: `update()` hace early-return en este modo
/// y los atajos globales no llegan; se restauran sesión y libro desde el
/// documento resultante.
fn handle_whiteboard_history_shortcuts(app: &mut crate::GrafitoApp, ctx: &egui::Context) {
    if ctx.wants_keyboard_input() {
        return;
    }
    let (undo, redo) = ctx.input(|input| {
        let undo =
            input.key_pressed(egui::Key::Z) && input.modifiers.ctrl && !input.modifiers.shift;
        let redo =
            (input.key_pressed(egui::Key::Z) && input.modifiers.ctrl && input.modifiers.shift)
                || (input.key_pressed(egui::Key::Y) && input.modifiers.ctrl);
        (undo, redo)
    });
    if undo {
        app.undo();
        sync_whiteboard_from_document(app);
    }
    if redo {
        app.redo();
        sync_whiteboard_from_document(app);
    }
}

pub fn draw_whiteboard_overlay(app: &mut crate::GrafitoApp, ctx: &egui::Context) {
    let theme = current_theme(ctx);
    app.sync_assistant_for_frame(ctx);
    // Sync bidireccional Session<->Document.whiteboard (cierra el dead-store):
    // el documento manda al abrir o tras undo/redo externo; la sesión manda
    // al final del frame vía `commit_whiteboard_to_document`.
    sync_whiteboard_from_document(app);
    handle_whiteboard_history_shortcuts(app, ctx);
    // Foto previa por revisión (sin clonar los elementos) para detectar si
    // este frame editó la pizarra.
    let before_revision = app.whiteboard.doc.revision();
    app.whiteboard_book
        .save_current_from_session(&app.whiteboard);

    // ── Toolbar burbuja centrada — Area flotante pill, compacta ──
    // Centrada como burbuja aparte, no barra larga. Solo 4 herramientas + borrar.
    {
        let mut selected_tool: Option<WhiteboardTool> = None;
        let mut close = false;
        let mut insert_image = false;
        let mut paste_image = false;
        let avail = ctx.available_rect();
        let screen = ctx.screen_rect();
        let center_offset_x = avail.center().x - screen.center().x;
        let share: Option<WhiteboardShareAction> =
            egui::Area::new(egui::Id::new("whiteboard_toolbar"))
                .anchor(egui::Align2::CENTER_TOP, vec2(center_offset_x, 8.0))
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    egui::Frame::none()
                        .fill(theme.panel_bg.gamma_multiply(0.96))
                        .stroke(egui::Stroke::new(1.0, theme.separator.gamma_multiply(0.08)))
                        .rounding(RADIUS_SM)
                        .inner_margin(egui::Margin::symmetric(SPACE_XS, SPACE_SM / 2.0))
                        .shadow(egui::Shadow {
                            offset: vec2(0.0, SHADOW_POPUP_OFFSET_Y),
                            blur: SHADOW_POPUP_BLUR,
                            spread: 0.0,
                            color: Color32::from_black_alpha(SHADOW_ALPHA),
                        })
                        .show(ui, |ui| {
                            let mut dummy_clear = false;
                            let mut dummy_ask = false;
                            let mut dummy_toggle_a = false;
                            let mut dummy_toggle_p = false;
                            draw_toolbar_contents(
                                ui,
                                app,
                                &mut selected_tool,
                                &mut dummy_clear,
                                &mut close,
                                &mut dummy_ask,
                                &mut dummy_toggle_a,
                                &mut dummy_toggle_p,
                                &mut insert_image,
                                &mut paste_image,
                            )
                        })
                        .inner
                })
                .inner;
        if let Some(t) = selected_tool {
            app.whiteboard.set_tool(t);
        }
        if close {
            app.whiteboard_open = false;
        }
        // Imagen por archivo (diálogo nativo existente + lectura acotada).
        // D2 lockdown: en examen no entra nada (igual que Compartir).
        if insert_image && !app.exam_blocks("Insertar imagen") {
            insert_whiteboard_image_from_file(app);
        }
        // Pegar desde el portapapeles OS (arboard directo → PNG en
        // memoria → mismo camino que archivo; vacío → toast honesto).
        // D2 lockdown: en examen no entra nada (igual que archivo).
        if paste_image && !app.exam_blocks("Insertar imagen") {
            crate::whiteboard_clipboard::paste_whiteboard_image_from_clipboard(app);
        }
        // S5 Compartir: copiar SVG al portapapeles, guardar `.svg` o
        // guardar `.png` a archivo (diálogo nativo + write en background,
        // cero I/O en Ui::). D2 lockdown: en examen no sale nada (igual
        // que `export_with_dialog`).
        if let Some(action) = share {
            if !app.exam_blocks("Compartir") {
                match action {
                    WhiteboardShareAction::CopySvg | WhiteboardShareAction::SaveSvg => {
                        match current_page_svg(app) {
                            Err(error) => {
                                app.notify(error, grafito_ui::toast::ToastKind::Error);
                            }
                            Ok(svg) => {
                                // Solo llegan CopySvg|SaveSvg (el PNG va por
                                // su propia rama, sin pasar por el SVG).
                                if action == WhiteboardShareAction::CopySvg {
                                    ctx.copy_text(svg);
                                    app.notify(
                                        "Hoja copiada como SVG al portapapeles.",
                                        grafito_ui::toast::ToastKind::Success,
                                    );
                                } else if let Some(path) =
                                    app.save_file_native("SVG", &["svg"], Some("pizarra.svg"))
                                {
                                    app.pending_export_job = Some(crate::app::PendingExportJob {
                                        receiver: crate::app::spawn_whiteboard_svg_write(
                                            svg, path, ctx,
                                        ),
                                    });
                                    app.notify(
                                        "Guardando SVG…",
                                        grafito_ui::toast::ToastKind::Info,
                                    );
                                }
                            }
                        }
                    }
                    WhiteboardShareAction::SavePng => save_current_page_png(app, ctx),
                }
            }
        }
    }
    // ── Diálogo Fórmula (texto → imagen, fallback honesto) ──
    // Estado en memoria temporal de egui (no persiste, no va al documento).
    // Solo se pega como `Image` si rasteriza; si no, toast honesto y jamás
    // imagen rota (ver `whiteboard_formula.rs`).
    {
        let mut dialog = crate::whiteboard_formula::formula_dialog_get(ctx);
        if dialog.open {
            let mut close = false;
            let mut submit = false;
            egui::Window::new("Fórmula en pizarra")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.label(
                        "Escribí la fórmula (subset LaTeX: x^2, \\frac{1}{2}, \\sqrt{x}). \
                        Solo se pega si rasteriza; si no, avisa.",
                    );
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut dialog.text)
                            .hint_text("x^2 + \\frac{1}{2}")
                            .desired_width(280.0),
                    );
                    if response.lost_focus()
                        && ui.input(|input| input.key_pressed(egui::Key::Enter))
                    {
                        submit = true;
                    }
                    ui.horizontal(|ui| {
                        if ui.button("Pegar").clicked() {
                            submit = true;
                        }
                        if ui.button("Cancelar").clicked() {
                            close = true;
                        }
                    });
                });
            if submit {
                close =
                    crate::whiteboard_formula::insert_whiteboard_formula(app, dialog.text.trim());
            }
            if close {
                dialog.open = false;
                dialog.text.clear();
            }
            crate::whiteboard_formula::formula_dialog_set(ctx, dialog);
        }
    }
    // ── Palette flotante (Area) — panel del lápiz estilo OpenBoard:
    // color (paleta curada de 8, reutilizada) + grosor 1..=24 + Formas.
    // Visible con Lápiz o Resaltador.
    let is_pen = matches!(
        app.whiteboard.tool,
        WhiteboardTool::Pencil | WhiteboardTool::Highlighter
    );
    // Palette compacta y centrada — solo se achica al contenido, no ocupa 560 por defecto
    // Movimiento reducido: la paleta aparece/desaparece sin fundido.
    let palette_anim = if app.reduced_motion {
        if is_pen {
            1.0
        } else {
            0.0
        }
    } else {
        ctx.animate_bool(egui::Id::new("whiteboard_palette"), is_pen)
    };
    if palette_anim > 0.01 {
        let avail = ctx.available_rect();
        let screen = ctx.screen_rect();
        let center_offset_x = avail.center().x - screen.center().x;
        egui::Area::new(egui::Id::new("whiteboard_palette_area"))
            .anchor(egui::Align2::CENTER_BOTTOM, vec2(center_offset_x, -12.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                egui::Frame::none()
                    .fill(theme.panel_bg.gamma_multiply(0.96 * palette_anim + 0.04))
                    .stroke(egui::Stroke::new(
                        1.0,
                        theme.separator.gamma_multiply(0.10 * palette_anim),
                    ))
                    .rounding(RADIUS_MD)
                    .inner_margin(egui::Margin::symmetric(SPACE_SM, SPACE_SM))
                    .shadow(egui::Shadow {
                        offset: vec2(0.0, SHADOW_POPUP_OFFSET_Y),
                        blur: SHADOW_POPUP_BLUR,
                        spread: 0.0,
                        color: Color32::from_black_alpha(
                            (SHADOW_ALPHA as f32 * palette_anim) as u8,
                        ),
                    })
                    .show(ui, |ui| {
                        ui.scope(|ui| {
                            ui.style_mut().visuals.override_text_color =
                                Some(theme.text_primary.gamma_multiply(palette_anim));
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = SPACE_SM;
                                egui::ScrollArea::horizontal()
                                    .auto_shrink([true, false])
                                    .scroll_bar_visibility(
                                        egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded,
                                    )
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            // Paleta curada Scandinavian — valores mapeados a theme (quiet, 8 tonos):
                                            // [250,250,249]=LIGHT.canvas_bg/DARK.text_primary (FAFAF9) — fondo claro
                                            // [212,212,216]=separator claro / text_tertiary
                                            // [107,122,111]=accent sage #6B7A6F — oliva calm
                                            // [100,116,139]=steel blue desaturado
                                            // [168,123,110]=terracotta suave
                                            // [91,125,177]=azul medio
                                            // [196,91,91]=rojo apagado
                                            // [209,183,91]=ocre Scandinavian — todos theme-derived, no theatrical
                                            const PALETTE: &[[u8; 3]] = &[
                                                [250, 250, 249],
                                                [212, 212, 216],
                                                [107, 122, 111],
                                                [100, 116, 139],
                                                [168, 123, 110],
                                                [91, 125, 177],
                                                [196, 91, 91],
                                                [209, 183, 91],
                                            ];
                                            let mut picked: Option<Color32> = None;
                                            for rgb in PALETTE.iter() {
                                                let col = Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
                                                let is_sel = app.whiteboard.pen_color == col;
                                                // S2 táctil: celda 44 bajo 740 px, 24 en desktop.
                                                // El scroll horizontal absorbe el ancho: nunca desborda.
                                                let cell =
                                                    touch_hit(ui.ctx().screen_rect().width());
                                                let (rect, resp) = ui.allocate_exact_size(
                                                    vec2(cell, cell),
                                                    Sense::click(),
                                                );
                                                if ui.is_rect_visible(rect) {
                                                    ui.painter().circle_filled(
                                                        rect.center(),
                                                        cell / 2.0 - 1.0,
                                                        col,
                                                    );
                                                    ui.painter().circle_stroke(
                                                        rect.center(),
                                                        cell / 2.0 - 1.0,
                                                        Stroke::new(
                                                            if is_sel { 2.0 } else { 1.0 },
                                                            if is_sel {
                                                                theme
                                                                    .accent
                                                                    .gamma_multiply(palette_anim)
                                                            } else {
                                                                theme.separator.gamma_multiply(
                                                                    0.20 * palette_anim,
                                                                )
                                                            },
                                                        ),
                                                    );
                                                    if is_sel {
                                                        ui.painter().circle_stroke(
                                                            rect.center(),
                                                            (app.whiteboard.pen_width * 1.1)
                                                                .clamp(2.0, 27.0),
                                                            Stroke::new(
                                                                1.0,
                                                                Color32::from_black_alpha(
                                                                    ALPHA_OVERLAY,
                                                                ),
                                                            ),
                                                        );
                                                    }
                                                }
                                                if resp
                                                    .on_hover_text(format!(
                                                        "#{:02X}{:02X}{:02X}",
                                                        rgb[0], rgb[1], rgb[2]
                                                    ))
                                                    .clicked()
                                                {
                                                    picked = Some(col);
                                                }
                                            }
                                            if let Some(col) = picked {
                                                app.whiteboard.pen_color = col;
                                            }
                                        });
                                    });
                                ui.separator();
                                ui.label(
                                    egui::RichText::new("Grosor")
                                        .size(grafito_ui::tokens::TYPE_XS)
                                        .color(theme.text_secondary.gamma_multiply(palette_anim))
                                        .strong(),
                                );
                                let mut w = app.whiteboard.pen_width;
                                // Angosto: el riel usa el ancho real (mismo
                                // patrón que Prob). Cromo MEDIDO (etiqueta +
                                // preview 20 + valor + gaps vivos), no 64 a
                                // mano: con otra escala de fuente el estimado
                                // apretaba el slider. "+8" cubre el strong.
                                if ui.ctx().screen_rect().width()
                                    < grafito_ui::tokens::PALETTE_BOTTOM_SHEET_BREAKPOINT
                                {
                                    let font_xs =
                                        egui::FontId::proportional(grafito_ui::tokens::TYPE_XS);
                                    let meas = |s: &str| {
                                        ui.fonts(|f| {
                                            f.layout_no_wrap(
                                                s.to_owned(),
                                                font_xs.clone(),
                                                egui::Color32::WHITE,
                                            )
                                            .size()
                                            .x
                                        })
                                    };
                                    let gap = ui.spacing().item_spacing.x;
                                    let chrome = meas("Grosor")
                                        + 20.0
                                        + meas(&format!("{w:.1}"))
                                        + 4.0 * gap
                                        + 8.0;
                                    ui.spacing_mut().slider_width =
                                        (ui.available_width() - chrome).max(60.0);
                                }
                                let resp = ui.add(
                                    egui::Slider::new(
                                        &mut w,
                                        PEN_WIDTH_MIN as f32..=PEN_WIDTH_MAX as f32,
                                    )
                                    .show_value(false)
                                    .trailing_fill(true),
                                );
                                if resp.changed() {
                                    app.whiteboard.pen_width =
                                        w.clamp(PEN_WIDTH_MIN as f32, PEN_WIDTH_MAX as f32);
                                }
                                let (pr, _) =
                                    ui.allocate_exact_size(vec2(20.0, 20.0), Sense::hover());
                                ui.painter().circle_stroke(
                                    pr.center(),
                                    (app.whiteboard.pen_width * 1.1).clamp(2.0, 27.0),
                                    Stroke::new(1.2, app.whiteboard.pen_color),
                                );
                                ui.painter().circle_filled(
                                    pr.center(),
                                    1.5,
                                    theme.separator.gamma_multiply(0.25),
                                );
                                ui.label(
                                    egui::RichText::new(format!("{:.1}", app.whiteboard.pen_width))
                                        .size(grafito_ui::tokens::TYPE_XS)
                                        .color(theme.text_primary.gamma_multiply(palette_anim)),
                                );
                                // Auto-formas del lápiz (default apagado):
                                // círculo→elipse, recta→flecha; si no reconoce
                                // nada queda el trazo tal cual. Etiqueta en
                                // español como el resto del panel, sin claves
                                // i18n nuevas (6 locales intactos).
                                ui.separator();
                                let mut shapes = app.whiteboard.auto_shapes;
                                ui.checkbox(&mut shapes, "Formas").on_hover_text(
                                    "El lápiz cierra círculos→elipse y rectas→flecha; si no reconoce nada queda el trazo tal cual",
                                );
                                app.whiteboard.auto_shapes = shapes;
                            });
                        });
                    });
            });
    }
    // ── Canvas + controles de zoom (Scandinavian: pill sutil abajo-derecha) ──
    let mut zoom_in = false;
    let mut zoom_out = false;
    let mut zoom_reset = false;
    let current_zoom = app.whiteboard.zoom;
    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(theme.canvas_bg))
        .show(ctx, |ui| {
            let available = ui.available_rect_before_wrap();
            if available.width() < 10.0 || available.height() < 10.0 {
                return;
            }
            let canvas_rect = available.shrink(8.0);
            ctx.memory_mut(|mem| {
                mem.data
                    .insert_temp(egui::Id::new("whiteboard_canvas_rect"), canvas_rect)
            });
            ui.painter().rect_stroke(
                canvas_rect,
                RADIUS_MD,
                Stroke::new(1.0, theme.separator.gamma_multiply(0.08)),
            );
            app.whiteboard
                .handle_canvas_input(canvas_rect, ui, &mut app.repaint_budget);
            app.whiteboard.draw(ui, canvas_rect);

            // Pill de zoom abajo-derecha — Scandinavian: infinito Geogebra 1e-6..1e6
            let zoom_percent_f = current_zoom * 100.0;
            let at_min = current_zoom <= ZOOM_WB_MIN * 1.001;
            let at_max = current_zoom >= ZOOM_WB_MAX * 0.999;
            let at_limits_exact = at_min || at_max;
            // Infinito: botones siempre habilitados salvo en los extremos absolutos
            let minus_enabled = !at_min;
            let plus_enabled = !at_max;
            // Táctil (<740): botones de 44px y pill arriba de la paleta
            // (a 320px se pisan); desktop intacto.
            let touch_wb =
                ctx.screen_rect().width() < grafito_ui::tokens::PALETTE_BOTTOM_SHEET_BREAKPOINT;
            let zoom_btn = if touch_wb {
                egui::vec2(44.0, 44.0)
            } else {
                egui::vec2(26.0, 24.0)
            };
            let zoom_dy = if touch_wb
                && matches!(
                    app.whiteboard.tool,
                    WhiteboardTool::Pencil | WhiteboardTool::Highlighter
                ) {
                -140.0
            } else {
                -56.0
            };
            egui::Area::new(egui::Id::new("whiteboard_zoom_controls"))
                .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-16.0, zoom_dy))
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    egui::Frame::none()
                        .fill(theme.panel_bg)
                        .stroke(egui::Stroke::new(1.0, theme.separator))
                        .rounding(ZOOM_PILL_RADIUS)
                        .inner_margin(egui::Margin::symmetric(ZOOM_PILL_PAD_X, ZOOM_PILL_PAD_Y))
                        .shadow(egui::Shadow {
                            offset: egui::vec2(0.0, SHADOW_POPUP_OFFSET_Y),
                            blur: SHADOW_POPUP_BLUR,
                            spread: 0.0,
                            color: Color32::from_black_alpha(SHADOW_ALPHA),
                        })
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = ZOOM_PILL_GAP;
                                ui.add_enabled_ui(minus_enabled, |ui| {
                                    let col = if minus_enabled {
                                        theme.text_primary
                                    } else {
                                        theme.text_tertiary
                                    };
                                    if action_icon_button_sized(
                                        ui,
                                        Icon::Minus,
                                        col,
                                        "Alejar (-)",
                                        zoom_btn,
                                    )
                                    .clicked()
                                    {
                                        zoom_out = true;
                                    }
                                });
                                let pct_text = if zoom_percent_f >= 1e7 {
                                    format!("{:.0e}%", zoom_percent_f)
                                } else if zoom_percent_f >= 1000.0 {
                                    format!("{:.0}%", zoom_percent_f)
                                } else if zoom_percent_f < 0.1 {
                                    format!("{:.3}%", zoom_percent_f)
                                } else if zoom_percent_f < 10.0 {
                                    format!("{:.1}%", zoom_percent_f)
                                } else {
                                    format!("{:.0}%", zoom_percent_f)
                                };
                                let pct_color = if at_limits_exact {
                                    theme.warning
                                } else {
                                    theme.text_secondary
                                };
                                let pct_btn = egui::Button::new(
                                    egui::RichText::new(pct_text)
                                        .size(TYPE_XS)
                                        .color(pct_color)
                                        .monospace(),
                                )
                                .min_size(egui::vec2(
                                    ZOOM_PCT_MIN_W,
                                    if touch_wb { 44.0 } else { ZOOM_ICON_HIT },
                                ))
                                .frame(false);
                                if ui
                                    .add(pct_btn)
                                    .on_hover_text("Clic para 100% — Ctrl+0 / doble clic canvas")
                                    .clicked()
                                {
                                    zoom_reset = true;
                                }
                                ui.add_enabled_ui(plus_enabled, |ui| {
                                    let col = if plus_enabled {
                                        theme.text_primary
                                    } else {
                                        theme.text_tertiary
                                    };
                                    if action_icon_button_sized(
                                        ui,
                                        Icon::Plus,
                                        col,
                                        "Acercar (+)",
                                        zoom_btn,
                                    )
                                    .clicked()
                                    {
                                        zoom_in = true;
                                    }
                                });
                                ui.add_space(SPACE_SM);
                                ui.separator();
                                ui.add_space(SPACE_SM);
                                if action_icon_button_sized(
                                    ui,
                                    Icon::Grid,
                                    theme.text_secondary,
                                    "Ajustar vista (Ctrl+0)",
                                    zoom_btn,
                                )
                                .clicked()
                                {
                                    zoom_reset = true;
                                }
                            });
                        });
                });
        });
    // Aplicar zoom anclado al centro del canvas real (Memory), no screen dummy
    let canvas_center_and_rect = ctx.memory(|mem| {
        mem.data
            .get_temp::<egui::Rect>(egui::Id::new("whiteboard_canvas_rect"))
            .map(|r| (r.center(), r))
    });
    let (zoom_center, zoom_rect) = canvas_center_and_rect.unwrap_or_else(|| {
        let c = ctx.screen_rect().center();
        (c, egui::Rect::from_center_size(c, egui::vec2(400.0, 300.0)))
    });
    if zoom_in {
        app.whiteboard.zoom_at(1.2, zoom_center, zoom_rect);
    }
    if zoom_out {
        app.whiteboard.zoom_at(0.8333, zoom_center, zoom_rect);
    }
    if zoom_reset {
        app.whiteboard.zoom = ZOOM_WB_DEFAULT;
        app.whiteboard.pan = (0.0, 0.0);
    }
    // Doble clic en canvas resetea zoom (gesto rápido) — solo si fue sobre el lienzo
    if ctx.input(|i| {
        i.pointer
            .button_double_clicked(egui::PointerButton::Primary)
    }) {
        if let Some(pos) = ctx.input(|i| i.pointer.latest_pos()) {
            if let Some(canvas) = ctx.memory(|mem| {
                mem.data
                    .get_temp::<egui::Rect>(egui::Id::new("whiteboard_canvas_rect"))
            }) {
                if canvas.contains(pos) {
                    app.whiteboard.zoom = ZOOM_WB_DEFAULT;
                    app.whiteboard.pan = (0.0, 0.0);
                }
            }
        }
    }

    if ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
        if app.whiteboard.show_palette {
            app.whiteboard.show_palette = false;
        } else if app.whiteboard.curtain.is_some() || app.whiteboard.magnifier_active {
            // Esc primero guarda los overlays de cortina/lupa (no cierra).
            app.whiteboard.curtain = None;
            app.whiteboard.curtain_drag = false;
            app.whiteboard.magnifier_active = false;
        } else if app.whiteboard_open {
            app.whiteboard_open = false;
        }
    }
    let mut typed = Vec::new();
    let mut backspace = false;
    ctx.input(|input| {
        for event in &input.events {
            match event {
                egui::Event::Text(text) => typed.extend(text.chars()),
                egui::Event::Key {
                    key: egui::Key::Backspace,
                    pressed: true,
                    ..
                } => backspace = true,
                _ => {}
            }
        }
    });
    if app.whiteboard.tool == WhiteboardTool::Text {
        for character in typed {
            app.whiteboard.type_char(character);
        }
        if backspace {
            app.whiteboard.backspace_text();
        }
    }
    // Volcado Session->Document.whiteboard con snapshot undo (marca sucio).
    commit_whiteboard_to_document(app, before_revision);
    if let Some(message) = app.whiteboard.last_error.take() {
        app.notify(message, grafito_ui::toast::ToastKind::Error);
    }
    if let Some(warning) = app.whiteboard.last_warning.take() {
        app.notify(warning, grafito_ui::toast::ToastKind::Info);
    }
    let busy = ctx.input(|input| input.pointer.any_down());
    // F17: la pizarra hace early-return en `GrafitoApp::update`, así que el
    // scheduler unificado no corre en este modo; este es su scheduler local,
    // coalescido vía presupuesto y aplicado una sola vez al final del overlay.
    // P3/H12: en idle no se pide wake (0 wakes/s vs 10/s a 100ms). La paleta
    // sigue animando porque `ctx.animate_bool` repinta solo mientras anima
    // (verificado en egui 0.29 `context.rs::animate_value_with_time`).
    if let Some(delay) = whiteboard_repaint_delay(busy) {
        app.repaint_budget.request(delay);
    }
    app.repaint_budget.apply(ctx);
}

/// Retraso de repintado del overlay de pizarra (P3/H12, pura y testeable).
///
/// `busy` (pointer abajo) → 16ms (60Hz trazo). Idle → `None`: sin wake
/// periódico; `ctx.animate_bool` de la paleta ya repinta mientras anima e
/// interacciones/undo repintan por evento.
pub fn whiteboard_repaint_delay(busy: bool) -> Option<std::time::Duration> {
    busy.then(|| std::time::Duration::from_millis(16))
}

#[cfg(test)]
mod tests {
    use super::*;
    use grafito_whiteboard::magnifier_zoom;

    #[test]
    fn whiteboard_idle_requests_no_repaint_wake() {
        // P3/H12: idle → 0 wakes/s (antes 10/s a 100ms); busy → 16ms.
        assert_eq!(whiteboard_repaint_delay(false), None);
        assert_eq!(
            whiteboard_repaint_delay(true),
            Some(std::time::Duration::from_millis(16))
        );
    }

    #[test]
    fn whiteboard_creates_a_normalized_rectangle_from_a_drag() {
        let mut session = WhiteboardSession::default();
        session.set_tool(WhiteboardTool::Rectangle);
        session.handle_pointer((6.0, 8.0), true, false);
        session.handle_pointer((2.0, 3.0), false, false);
        session.handle_pointer((2.0, 3.0), false, true);
        assert_eq!(session.doc.len(), 1);
        let (min, max) = session.doc.elements()[0].bounds().unwrap();
        assert_eq!(min, (2.0, 3.0));
        assert_eq!(max, (6.0, 8.0));
    }

    #[test]
    fn whiteboard_pencil_snaps_to_ruler_and_stays_free_when_far() {
        // Regla horizontal en y=0: el punto cercano se pega al filo, el
        // lejano queda libre honesto (OpenBoard: el trazo se pega al borde).
        let mut session = WhiteboardSession::default();
        session.toggle_instrument(InstrumentKind::Ruler);
        // La regla por defecto va de (-150, 0) a (150, 0): a 11 px del filo
        // no la agarra el cuerpo (tol. 10) pero el trazo sí se pega (tol. 12).
        session.set_tool(WhiteboardTool::Pencil);
        session.handle_pointer((50.0, 11.0), true, false);
        session.handle_pointer((60.0, 11.0), false, false);
        session.handle_pointer((60.0, 11.0), false, true);
        assert_eq!(session.doc.len(), 1);
        match &session.doc.elements()[0] {
            WhiteboardElement::Stroke { points, .. } => {
                assert!(!points.is_empty());
                for (x, y) in points {
                    assert!(
                        y.abs() < 1e-9,
                        "trazo pegado al filo y=0, punto: ({x}, {y})"
                    );
                }
            }
            other => panic!("trazo esperado, fue: {other:?}"),
        }
        // Lejos del filo (200 px sobre la regla): trazo libre, sin pegar.
        let mut free = WhiteboardSession::default();
        free.toggle_instrument(InstrumentKind::Ruler);
        free.set_tool(WhiteboardTool::Pencil);
        free.handle_pointer((0.0, 200.0), true, false);
        free.handle_pointer((10.0, 200.0), false, false);
        free.handle_pointer((10.0, 200.0), false, true);
        match &free.doc.elements()[0] {
            WhiteboardElement::Stroke { points, .. } => {
                assert!(
                    points.iter().all(|(_, y)| (*y - 200.0).abs() < 30.0),
                    "lejos del borde el trazo queda libre: {points:?}"
                );
            }
            other => panic!("trazo esperado, fue: {other:?}"),
        }
    }

    #[test]
    fn whiteboard_compass_two_clicks_make_an_exact_circle() {
        // Compás: clic centro + clic radio → Ellipse con rx == ry exactos
        // (reusa Ellipse, sin elemento nuevo).
        let mut session = WhiteboardSession::default();
        session.toggle_compass();
        assert!(session.compass_active);
        session.handle_pointer((100.0, 100.0), true, false);
        assert_eq!(session.doc.len(), 0, "el centro solo no dibuja nada");
        session.handle_pointer((130.0, 140.0), true, false);
        assert_eq!(session.doc.len(), 1);
        match &session.doc.elements()[0] {
            WhiteboardElement::Ellipse { center, rx, ry } => {
                assert_eq!(*center, (100.0, 100.0));
                assert_eq!(rx, ry, "compás exacto: rx == ry");
                assert!((rx - 50.0).abs() < 1e-9, "radio 50, fue: {rx}");
            }
            other => panic!("círculo esperado, fue: {other:?}"),
        }
        // Doble clic en el mismo punto: radio diminuto → nada, sin roto.
        let mut tiny = WhiteboardSession::default();
        tiny.toggle_compass();
        tiny.handle_pointer((5.0, 5.0), true, false);
        tiny.handle_pointer((5.0, 5.0), true, false);
        assert!(tiny.doc.is_empty());
    }

    #[test]
    fn whiteboard_single_instrument_slot_hides_and_replaces() {
        // Un instrumento visible a la vez; clic de nuevo lo guarda; el
        // compás comparte el slot (activarlo guarda la regla).
        let mut session = WhiteboardSession::default();
        assert_eq!(session.instrument_kind(), None);
        session.toggle_instrument(InstrumentKind::Ruler);
        assert_eq!(session.instrument_kind(), Some(InstrumentKind::Ruler));
        session.toggle_instrument(InstrumentKind::Ruler);
        assert_eq!(session.instrument_kind(), None, "clic de nuevo lo guarda");
        session.toggle_instrument(InstrumentKind::Ruler);
        session.toggle_instrument(InstrumentKind::Protractor);
        assert_eq!(
            session.instrument_kind(),
            Some(InstrumentKind::Protractor),
            "el nuevo reemplaza al visible"
        );
        session.toggle_compass();
        assert!(session.compass_active);
        assert_eq!(session.instrument_kind(), None);
        session.toggle_compass();
        assert!(!session.compass_active);
    }

    #[test]
    fn whiteboard_instrument_drags_body_and_rotates_with_handle() {
        // Agarrar el filo arrastra (traslación pura); la manija rota.
        let mut session = WhiteboardSession::default();
        session.toggle_instrument(InstrumentKind::Ruler);
        session.set_tool(WhiteboardTool::Pencil);
        // El filo pasa por el origen: agarrar (20, 0) mueve el cuerpo.
        session.handle_pointer((20.0, 0.0), true, false);
        session.handle_pointer((40.0, 10.0), false, false);
        session.handle_pointer((40.0, 10.0), false, true);
        assert!(session.doc.is_empty(), "mover el instrumento no dibuja");
        let pos = session.instrument.map(|instrument| instrument.pos());
        assert_eq!(pos, Some((20.0, 10.0)));
        assert_eq!(
            session.instrument.map(|instrument| instrument.angle()),
            Some(0.0),
            "arrastrar no rota"
        );
        // Manija en (150+18, 0) relativo al centro movido → rotar a 90°.
        let Some(handle) = session
            .instrument
            .and_then(|instrument| instrument.handle_pos())
        else {
            panic!("manija útil esperada");
        };
        session.handle_pointer(handle, true, false);
        session.handle_pointer((20.0, 110.0), false, false);
        session.handle_pointer((20.0, 110.0), false, true);
        let angle = session.instrument.map(|instrument| instrument.angle());
        match angle {
            Some(a) => assert!(
                (a - std::f64::consts::FRAC_PI_2).abs() < 1e-6,
                "manija arriba → 90°, fue: {a}"
            ),
            None => panic!("instrumento perdido tras rotar"),
        }
    }

    #[test]
    fn whiteboard_instruments_never_enter_document_or_svg() {
        // Dibujar con la regla visible solo guarda tinta: el overlay no es
        // figura (no seleccionable) y no serializa al documento/SVG.
        let mut session = WhiteboardSession::default();
        session.toggle_instrument(InstrumentKind::Triangle);
        session.set_tool(WhiteboardTool::Pencil);
        session.handle_pointer((300.0, 300.0), true, false);
        session.handle_pointer((320.0, 300.0), false, false);
        session.handle_pointer((320.0, 300.0), false, true);
        for element in session.doc.elements() {
            assert!(
                matches!(
                    element,
                    WhiteboardElement::Stroke { .. }
                        | WhiteboardElement::Highlighter { .. }
                        | WhiteboardElement::Rectangle { .. }
                        | WhiteboardElement::Ellipse { .. }
                        | WhiteboardElement::Arrow { .. }
                        | WhiteboardElement::Text { .. }
                ),
                "solo figuras de dibujo en el documento: {element:?}"
            );
        }
        let json = match serde_json::to_string(&session.doc) {
            Ok(json) => json,
            Err(error) => panic!("documento serializa: {error}"),
        };
        for key in ["Ruler", "Triangle", "Protractor", "instrument"] {
            assert!(!json.contains(key), "fuga al documento: {key}");
        }
        let svg = match session.doc.to_svg(800, 600) {
            Ok(svg) => svg,
            Err(error) => panic!("SVG serializa: {error}"),
        };
        for key in ["Ruler", "Triangle", "Protractor", "instrument"] {
            assert!(!svg.contains(key), "fuga al SVG: {key}");
        }
    }

    #[test]
    fn whiteboard_background_defaults_to_historic_grid() {
        // Default intacto: sesión y hoja nacen cuadriculadas (el grid de
        // siempre); 4 opciones con etiquetas en español, sin claves i18n.
        assert_eq!(
            WhiteboardSession::default().background,
            WhiteboardBackground::Grid
        );
        assert_eq!(WhiteboardBackground::default(), WhiteboardBackground::Grid);
        assert_eq!(WhiteboardBackground::ALL.len(), 4);
        let labels: Vec<&str> = WhiteboardBackground::ALL
            .iter()
            .map(|bg| bg.label())
            .collect();
        assert_eq!(labels, vec!["Cuadriculado", "Rayado", "Punteado", "Blanco"]);
    }

    #[test]
    fn whiteboard_background_syncs_session_and_page_but_not_document() {
        // El fondo viaja sesión↔hoja en memoria (como pan/zoom) pero NUNCA
        // al documento: al reabrir (from_persisted) defaulta a cuadriculado.
        let mut book = WhiteboardBook::default();
        let session = WhiteboardSession {
            background: WhiteboardBackground::Dotted,
            ..Default::default()
        };
        book.save_current_from_session(&session);
        assert_eq!(
            book.current().map(|page| page.background),
            Some(WhiteboardBackground::Dotted)
        );
        let mut back = WhiteboardSession::default();
        book.load_to_session(&mut back);
        assert_eq!(back.background, WhiteboardBackground::Dotted);
        // Ida a disco y vuelta: el fondo se pierde honesto (default grid).
        let persisted = book.to_persisted();
        let reopened = WhiteboardBook::from_persisted(&persisted, 0);
        assert_eq!(
            reopened.current().map(|page| page.background),
            Some(WhiteboardBackground::Grid),
            "al reabrir defaulta a cuadriculado (documentado, no bug)"
        );
        // Solo-fondo no ensucia el documento: el commit no escribe.
        assert!(whiteboard_pages_equal(&persisted, &reopened.to_persisted()));
    }

    #[test]
    fn whiteboard_insert_figure_adds_exact_pinned_counts() {
        // Cada stencil inserta N elementos normales al centro de la vista
        // (pan/zoom default → origen (0,0)), con coordenadas pineadas.
        for (kind, expected) in [
            (FigureStencil::RightTriangle, 7),
            (FigureStencil::IsoscelesTriangle, 7),
            (FigureStencil::CircleWithRadius, 5),
            (FigureStencil::MarkedAngle, 4),
            (FigureStencil::NumberLine, 15),
            (FigureStencil::CartesianPlane, 15),
            (FigureStencil::Protractor, 22),
            (FigureStencil::FractionCircle, 4),
            (FigureStencil::RegularHexagon, 8),
            (FigureStencil::Cube3D, 12),
            (FigureStencil::Clock, 17),
            (FigureStencil::Table2x3, 8),
        ] {
            let mut session = WhiteboardSession::default();
            let added = session.insert_figure(kind).expect("cupo disponible");
            assert_eq!(added, expected, "conteo pineado: {:?}", kind);
            assert_eq!(session.doc.len(), expected);
        }
        // El rectángulo nace en el origen de la vista.
        let mut session = WhiteboardSession::default();
        assert_eq!(session.insert_figure(FigureStencil::RightTriangle), Ok(7));
        match &session.doc.elements()[0] {
            WhiteboardElement::Stroke { points, .. } => {
                assert_eq!(*points, vec![(0.0, 0.0), (160.0, 0.0)]);
            }
            other => panic!("lado esperado, fue: {other:?}"),
        }
        // Con la vista movida, el stencil sigue al centro visible.
        let mut moved = WhiteboardSession {
            pan: (-80.0, -40.0),
            ..Default::default()
        };
        assert_eq!(moved.insert_figure(FigureStencil::CircleWithRadius), Ok(5));
        match &moved.doc.elements()[0] {
            WhiteboardElement::Ellipse { center, rx, ry } => {
                assert_eq!(*center, (80.0, 40.0));
                assert_eq!((*rx, *ry), (60.0, 60.0));
            }
            other => panic!("círculo esperado, fue: {other:?}"),
        }
    }

    #[test]
    fn whiteboard_insert_figure_is_atomic_when_page_full() {
        // Sin cupo para los N: no entra ninguno (atómico) y avisa en español.
        let mut session = WhiteboardSession::default();
        for _ in 0..MAX_WHITEBOARD_ELEMENTS_PER_PAGE {
            session.try_add(test_rectangle()).expect("cupo disponible");
        }
        let error = session
            .insert_figure(FigureStencil::NumberLine)
            .expect_err("15 elementos no entran en hoja llena");
        assert!(error.contains("500"), "fue: {error}");
        assert_eq!(session.doc.len(), MAX_WHITEBOARD_ELEMENTS_PER_PAGE);
        assert!(session.last_error.is_some());
        // Con cupo parcial tampoco entra a medias.
        let mut tight = WhiteboardSession::default();
        for _ in 0..(MAX_WHITEBOARD_ELEMENTS_PER_PAGE - 3) {
            tight.try_add(test_rectangle()).expect("cupo disponible");
        }
        assert!(tight.insert_figure(FigureStencil::MarkedAngle).is_err());
        assert_eq!(tight.doc.len(), MAX_WHITEBOARD_ELEMENTS_PER_PAGE - 3);
    }

    #[test]
    fn whiteboard_inserted_figures_are_plain_erasable_elements() {
        // Lo insertado se selecciona y borra como cualquier trazo: nada de
        // tipo nuevo, nada que requiera herramienta especial.
        let mut session = WhiteboardSession::default();
        session
            .insert_figure(FigureStencil::RightTriangle)
            .expect("inserta");
        // El hit-test de trazos mide a puntos y gana el topmost: el vértice
        // (160,0) lo comparten cateto (0) e hipotenusa (2); gana la última.
        assert_eq!(session.doc.select_at((160.0, 0.0), 2.0), Some(2));
        assert_eq!(session.doc.erase_at((160.0, 0.0), 2.0), Some(2));
        assert_eq!(session.doc.len(), 6);
        // La recta numérica trae etiquetas de texto reales.
        let mut line = WhiteboardSession::default();
        line.insert_figure(FigureStencil::NumberLine)
            .expect("inserta");
        let texts: Vec<&str> = line
            .doc
            .elements()
            .iter()
            .filter_map(|element| match element {
                WhiteboardElement::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(texts, vec!["-3", "-2", "-1", "0", "1", "2", "3"]);
    }

    #[test]
    fn whiteboard_pencil_creates_a_smoothed_stroke() {
        let mut session = WhiteboardSession::default();
        session.set_tool(WhiteboardTool::Pencil);
        session.handle_pointer((0.0, 0.0), true, false);
        session.handle_pointer((0.5, 0.5), false, false);
        session.handle_pointer((1.0, 1.0), false, true);
        assert_eq!(session.doc.len(), 1);
        match &session.doc.elements()[0] {
            WhiteboardElement::Stroke { points, .. } => assert!(points.len() >= 2),
            other => panic!("expected stroke, got {other:?}"),
        }
    }

    #[test]
    fn whiteboard_erases_along_the_trace() {
        let mut session = WhiteboardSession::default();
        session.set_tool(WhiteboardTool::Rectangle);
        session.handle_pointer((0.0, 0.0), true, false);
        session.handle_pointer((5.0, 5.0), false, true);
        assert_eq!(session.doc.len(), 1);
        session.set_tool(WhiteboardTool::Eraser);
        session.handle_pointer((2.0, 2.0), true, false);
        session.handle_pointer((2.2, 2.2), false, true);
        assert!(session.doc.is_empty());
    }

    #[test]
    fn world_screen_roundtrip_is_stable_without_pan() {
        let session = WhiteboardSession::default();
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(400.0, 300.0));
        let world = session.world_from_screen(pos2(200.0, 150.0), rect);
        assert!(world.0.abs() < 1e-6 && world.1.abs() < 1e-6);
    }

    fn test_rectangle() -> WhiteboardElement {
        WhiteboardElement::Rectangle {
            min: (0.0, 0.0),
            max: (1.0, 1.0),
            fill: None,
        }
    }

    #[test]
    fn whiteboard_try_add_rejects_501st_element_in_spanish() {
        let mut session = WhiteboardSession::default();
        for _ in 0..MAX_WHITEBOARD_ELEMENTS_PER_PAGE {
            session.try_add(test_rectangle()).expect("cupo disponible");
        }
        assert_eq!(session.doc.len(), MAX_WHITEBOARD_ELEMENTS_PER_PAGE);
        let error = session
            .try_add(test_rectangle())
            .expect_err("el elemento 501 debe rechazarse");
        assert!(
            error.contains("500"),
            "el mensaje debe citar la cota en español, fue: {error}"
        );
        assert_eq!(session.doc.len(), MAX_WHITEBOARD_ELEMENTS_PER_PAGE);
        assert!(session.last_error.is_some());
    }

    #[test]
    fn whiteboard_bounds_reject_oversized_stroke_text_and_pages() {
        let mut session = WhiteboardSession::default();
        let oversized = WhiteboardElement::Stroke {
            points: vec![(0.0, 0.0); MAX_WHITEBOARD_POINTS_PER_STROKE + 1],
            color: (0, 0, 0),
            width: 2.0,
        };
        let error = session
            .try_add(oversized)
            .expect_err("trazo de 4097 puntos debe rechazarse");
        assert!(error.contains("4096"), "fue: {error}");
        assert!(session.doc.is_empty());

        let long_text = WhiteboardElement::Text {
            at: (0.0, 0.0),
            text: "a".repeat(MAX_WHITEBOARD_TEXT_CHARS + 1),
            size: 14.0,
        };
        let error = session
            .try_add(long_text)
            .expect_err("texto de 2001 caracteres debe rechazarse");
        assert!(error.contains("2000"), "fue: {error}");

        let mut book = WhiteboardBook::default();
        for _ in 1..MAX_WHITEBOARD_PAGES {
            book.try_create_page().expect("cupo de hojas disponible");
        }
        assert_eq!(book.len(), MAX_WHITEBOARD_PAGES);
        let error = book
            .try_create_page()
            .expect_err("la hoja 33 debe rechazarse");
        assert!(error.contains("32"), "fue: {error}");
    }

    #[test]
    fn whiteboard_commit_stores_snapshot_and_reload_restores_session() {
        use std::collections::VecDeque;

        let mut document = grafito_core::Document::new();
        let mut undo: VecDeque<grafito_core::Document> = VecDeque::new();
        let mut redo: VecDeque<grafito_core::ChangeSet> = VecDeque::new();
        let mut undo_bytes = 0usize;

        let mut session = WhiteboardSession::default();
        session.try_add(test_rectangle()).expect("try_add ok");
        assert!(push_whiteboard_undo_and_store(
            &session.doc,
            &mut document,
            &mut undo,
            &mut redo,
            &mut undo_bytes
        ));
        assert_eq!(document.whiteboard.len(), 1);
        assert_eq!(undo.len(), 1);
        // Sin cambios no hay nuevo snapshot.
        assert!(!push_whiteboard_undo_and_store(
            &session.doc,
            &mut document,
            &mut undo,
            &mut redo,
            &mut undo_bytes
        ));
        assert_eq!(undo.len(), 1);

        // Simula undo: restaura el snapshot y recarga la sesión.
        document = undo.pop_back().expect("snapshot disponible");
        assert!(!whiteboard_docs_equal(&session.doc, &document.whiteboard));
        let mut book = WhiteboardBook::default();
        load_whiteboard_into_session(&mut session, &mut book, &document);
        assert!(session.doc.is_empty());
        assert!(whiteboard_docs_equal(&session.doc, &document.whiteboard));
    }

    #[test]
    fn whiteboard_fill_warning_fires_at_450_and_480_in_spanish() {
        assert!(whiteboard_fill_warning(0).is_none());
        assert!(whiteboard_fill_warning(449).is_none());
        let soft = whiteboard_fill_warning(450).expect("aviso a 450");
        assert!(
            soft.contains("450") && soft.contains("500"),
            "el aviso cita el conteo en español, fue: {soft}"
        );
        let strong = whiteboard_fill_warning(480).expect("aviso a 480");
        assert!(strong.contains("casi llena"), "fue: {strong}");
        let full = whiteboard_fill_warning(500).expect("aviso a 500");
        assert!(full.contains("llena"), "fue: {full}");
    }

    #[test]
    fn whiteboard_try_add_warns_only_on_threshold_crossing() {
        let mut session = WhiteboardSession::default();
        for _ in 0..449 {
            session.try_add(test_rectangle()).expect("cupo disponible");
            assert!(session.last_warning.is_none());
        }
        session.try_add(test_rectangle()).expect("elemento 450");
        assert_eq!(session.doc.len(), 450);
        let warning = session.last_warning.clone().expect("aviso al cruzar 450");
        assert!(warning.contains("450"), "fue: {warning}");
        // Añadir sin cruzar umbral no reavisa (evita spam de toasts).
        session.try_add(test_rectangle()).expect("elemento 451");
        assert!(session.last_warning.is_none());
        // Al cruzar 480 vuelve a avisar en español.
        for _ in 451..480 {
            session.try_add(test_rectangle()).expect("cupo disponible");
        }
        assert_eq!(session.doc.len(), 480);
        let warning = session.last_warning.clone().expect("aviso al cruzar 480");
        assert!(warning.contains("casi llena"), "fue: {warning}");
    }

    #[test]
    fn whiteboard_book_epoch_skips_unchanged_session_and_flags_edits() {
        let mut book = WhiteboardBook::default();
        let mut session = WhiteboardSession::default();
        book.load_to_session(&mut session);
        book.mark_synced();
        assert!(book.is_synced());
        // Sesión sin cambios: no clona ni ensucia el libro.
        book.save_current_from_session(&session);
        assert!(book.is_synced(), "idle no debe ensuciar el libro");
        // Editar la sesión (add sube la revisión) sí marca pendiente.
        session.doc.add(test_rectangle());
        book.save_current_from_session(&session);
        assert!(!book.is_synced(), "una edición debe requerir commit");
        assert_eq!(
            book.current().map(|page| page.doc.len()),
            Some(1),
            "el volcado debe copiar el contenido nuevo"
        );
        book.mark_synced();
        assert!(book.is_synced());
    }

    #[test]
    fn whiteboard_book_roundtrips_32_pages_through_document() {
        use std::collections::VecDeque;

        let mut book = WhiteboardBook::default();
        book.current_mut()
            .expect("hoja 1 disponible")
            .doc
            .add(test_rectangle());
        for index in 2..=MAX_WHITEBOARD_PAGES {
            let page = book.try_create_page().expect("cupo de hojas");
            book.switch_to(page);
            book.current_mut()
                .expect("hoja disponible")
                .doc
                .add(WhiteboardElement::Text {
                    at: (index as f64, 0.0),
                    text: format!("hoja {index}"),
                    size: 14.0,
                });
        }
        assert_eq!(book.len(), MAX_WHITEBOARD_PAGES);
        assert_eq!(book.current, MAX_WHITEBOARD_PAGES - 1);

        let session = WhiteboardSession {
            doc: book.current().expect("actual").doc.clone(),
            ..WhiteboardSession::default()
        };
        let mut document = grafito_core::Document::new();
        let mut undo: VecDeque<grafito_core::Document> = VecDeque::new();
        let mut redo: VecDeque<grafito_core::ChangeSet> = VecDeque::new();
        let mut undo_bytes = 0usize;
        assert!(push_whiteboard_book_and_store(
            &mut book,
            &session,
            &mut document,
            &mut undo,
            &mut redo,
            &mut undo_bytes
        ));
        assert_eq!(document.whiteboard_pages.len(), MAX_WHITEBOARD_PAGES);
        assert_eq!(document.whiteboard_current, MAX_WHITEBOARD_PAGES - 1);
        // El espejo sigue a la hoja actual (export/validación intactos).
        assert_eq!(document.whiteboard.len(), 1);
        // Sin cambios no hay nuevo snapshot.
        assert!(!push_whiteboard_book_and_store(
            &mut book,
            &session,
            &mut document,
            &mut undo,
            &mut redo,
            &mut undo_bytes
        ));
        assert_eq!(undo.len(), 1);
        // Roundtrip JSON: las 32 hojas sobreviven a guardar/cargar.
        let json = serde_json::to_string(&document).expect("serializa libro");
        let loaded: grafito_core::Document =
            serde_json::from_str(&json).expect("deserializa libro");
        assert_eq!(loaded.whiteboard_pages.len(), MAX_WHITEBOARD_PAGES);
        assert_eq!(loaded.whiteboard_current, MAX_WHITEBOARD_PAGES - 1);
        let mut book2 = WhiteboardBook::default();
        let mut session2 = WhiteboardSession::default();
        load_book_from_document(&mut session2, &mut book2, &loaded);
        assert_eq!(book2.len(), MAX_WHITEBOARD_PAGES);
        assert_eq!(book2.current, MAX_WHITEBOARD_PAGES - 1);
        assert_eq!(session2.doc.len(), 1);
        match &session2.doc.elements()[0] {
            WhiteboardElement::Text { text, .. } => assert_eq!(text, "hoja 32"),
            other => panic!("hoja 32 inesperada: {other:?}"),
        }
        // Hoja 1 intacta con su rectángulo y su título.
        assert_eq!(book2.pages[0].title, "Hoja 1");
        assert_eq!(book2.pages[0].doc.len(), 1);
    }

    #[test]
    fn whiteboard_book_migrates_legacy_document_without_loss() {
        use std::collections::VecDeque;

        let mut document = grafito_core::Document::new();
        document.whiteboard.add(test_rectangle());
        assert!(document.whiteboard_pages.is_empty());
        // Abrir: migra en memoria a 1 hoja sin perder el contenido.
        let mut book = WhiteboardBook::default();
        let mut session = WhiteboardSession::default();
        load_book_from_document(&mut session, &mut book, &document);
        assert_eq!(book.len(), 1);
        assert_eq!(session.doc.len(), 1);
        assert!(whiteboard_docs_equal(&session.doc, &document.whiteboard));
        // Guardar: persiste el Vec de 1 hoja (ya no se pierde nada).
        let mut undo: VecDeque<grafito_core::Document> = VecDeque::new();
        let mut redo: VecDeque<grafito_core::ChangeSet> = VecDeque::new();
        let mut undo_bytes = 0usize;
        assert!(push_whiteboard_book_and_store(
            &mut book,
            &session,
            &mut document,
            &mut undo,
            &mut redo,
            &mut undo_bytes
        ));
        assert_eq!(document.whiteboard_pages.len(), 1);
        assert_eq!(document.whiteboard.len(), 1);
    }

    #[test]
    fn whiteboard_book_survives_real_save_path_without_losing_pages_2_to_32() {
        use std::collections::VecDeque;

        // Libro completo de 32 hojas con contenido, título y vista distintos.
        let mut book = WhiteboardBook::default();
        book.current_mut()
            .expect("hoja 1 disponible")
            .doc
            .add(test_rectangle());
        for index in 2..=MAX_WHITEBOARD_PAGES {
            let page = book.try_create_page().expect("cupo de hojas");
            book.switch_to(page);
            let current = book.current_mut().expect("hoja disponible");
            current.doc.add(WhiteboardElement::Text {
                at: (index as f64, 0.0),
                text: format!("hoja {index}"),
                size: 14.0,
            });
            current.pan = (index as f64, -(index as f64));
            current.zoom = 1.0 + index as f64 * 0.01;
        }
        assert_eq!(book.len(), MAX_WHITEBOARD_PAGES);

        let session = WhiteboardSession {
            doc: book.current().expect("actual").doc.clone(),
            ..WhiteboardSession::default()
        };
        let mut document = grafito_core::Document::new();
        let mut undo: VecDeque<grafito_core::Document> = VecDeque::new();
        let mut redo: VecDeque<grafito_core::ChangeSet> = VecDeque::new();
        let mut undo_bytes = 0usize;
        assert!(push_whiteboard_book_and_store(
            &mut book,
            &session,
            &mut document,
            &mut undo,
            &mut redo,
            &mut undo_bytes
        ));

        // Ruta real de guardado (envelope + ValidatedDocument, igual que
        // `write_document_atomic` en lifecycle.rs:271): el bug original
        // perdía las hojas 2..=32 porque solo persistía el espejo.
        let json =
            grafito_core::serialize_document(&document).expect("el libro de 32 hojas debe guardar");
        assert!(json.contains("whiteboard_pages"));
        let loaded =
            grafito_core::deserialize_document(&json).expect("el libro de 32 hojas debe cargar");
        assert_eq!(loaded.whiteboard_pages.len(), MAX_WHITEBOARD_PAGES);
        assert_eq!(loaded.whiteboard_current, MAX_WHITEBOARD_PAGES - 1);
        match &loaded.whiteboard_pages[1].doc.elements()[0] {
            WhiteboardElement::Text { text, .. } => assert_eq!(text, "hoja 2"),
            other => panic!("hoja 2 inesperada: {other:?}"),
        }
        match &loaded.whiteboard_pages[MAX_WHITEBOARD_PAGES - 1]
            .doc
            .elements()[0]
        {
            WhiteboardElement::Text { text, .. } => assert_eq!(text, "hoja 32"),
            other => panic!("hoja 32 inesperada: {other:?}"),
        }
        // Vista de una hoja intermedia intacta (hoja 6 → índice 5).
        assert!((loaded.whiteboard_pages[5].pan.0 - 6.0).abs() < 1e-12);
        assert!((loaded.whiteboard_pages[5].zoom - 1.06).abs() < 1e-12);
        // El espejo sigue a la hoja actual (export/validación intactos).
        assert_eq!(loaded.whiteboard.len(), 1);
        // Recarga en memoria: el libro vuelve completo a la sesión.
        let mut book2 = WhiteboardBook::default();
        let mut session2 = WhiteboardSession::default();
        load_book_from_document(&mut session2, &mut book2, &loaded);
        assert_eq!(book2.len(), MAX_WHITEBOARD_PAGES);
        assert_eq!(book2.current, MAX_WHITEBOARD_PAGES - 1);
        assert_eq!(book2.pages[0].title, "Hoja 1");
        assert_eq!(book2.pages[0].doc.len(), 1);
    }

    #[test]
    fn whiteboard_legacy_json_migrates_through_real_load_path() {
        use std::collections::VecDeque;

        // JSON viejo: `whiteboard` con contenido pero sin libro ni índice
        // (como guardaban las versiones de una sola hoja).
        let mut legacy = grafito_core::Document::new();
        legacy.whiteboard.add(test_rectangle());
        let mut value = serde_json::to_value(&legacy).expect("a valor JSON");
        let object = value.as_object_mut().expect("objeto JSON");
        object.remove("whiteboard_pages");
        object.remove("whiteboard_current");
        let raw = serde_json::to_string(&value).expect("json legado");
        let loaded = grafito_core::deserialize_document(&raw).expect("carga legado");
        assert!(loaded.whiteboard_pages.is_empty());
        assert_eq!(loaded.whiteboard.len(), 1);
        // La UI migra en memoria sin pérdida y al guardar persiste 1 hoja.
        let mut book = WhiteboardBook::default();
        let mut session = WhiteboardSession::default();
        load_book_from_document(&mut session, &mut book, &loaded);
        assert_eq!(book.len(), 1);
        assert_eq!(session.doc.len(), 1);
        let mut document = loaded;
        let mut undo: VecDeque<grafito_core::Document> = VecDeque::new();
        let mut redo: VecDeque<grafito_core::ChangeSet> = VecDeque::new();
        let mut undo_bytes = 0usize;
        assert!(push_whiteboard_book_and_store(
            &mut book,
            &session,
            &mut document,
            &mut undo,
            &mut redo,
            &mut undo_bytes
        ));
        let json = grafito_core::serialize_document(&document).expect("el migrado debe guardar");
        let reloaded = grafito_core::deserialize_document(&json).expect("carga migrado");
        assert_eq!(reloaded.whiteboard_pages.len(), 1);
        assert_eq!(reloaded.whiteboard_pages[0].title, "Hoja 1");
        assert_eq!(reloaded.whiteboard.len(), 1);
    }

    #[test]
    fn whiteboard_load_truncates_text_over_2000_chars_without_splitting_unicode() {
        // Cota 2000 caracteres en la ruta de carga: texto multibyte largo
        // (bypassa `try_add` vía asignación directa, como un JSON externo)
        // se trunca por `char` sin partir Unicode.
        let mut document = grafito_core::Document::new();
        let mut page = grafito_core::WhiteboardPageData::blank("Hoja 1");
        page.doc.add(WhiteboardElement::Text {
            at: (0.0, 0.0),
            text: "ñ".repeat(MAX_WHITEBOARD_TEXT_CHARS + 500),
            size: 14.0,
        });
        document.whiteboard_pages = vec![page];
        let value = serde_json::to_value(&document).expect("a valor JSON");
        let loaded: grafito_core::Document = serde_json::from_value(value).expect("carga acotada");
        match &loaded.whiteboard_pages[0].doc.elements()[0] {
            WhiteboardElement::Text { text, .. } => {
                assert_eq!(text.chars().count(), MAX_WHITEBOARD_TEXT_CHARS);
            }
            other => panic!("texto esperado, fue: {other:?}"),
        }
    }

    #[test]
    fn whiteboard_book_svg_exports_one_file_per_page() {
        let mut book = WhiteboardBook::default();
        book.current_mut()
            .expect("hoja disponible")
            .doc
            .add(test_rectangle());
        let pages = grafito_core::whiteboard_pages_to_svg(&book.to_persisted(), 800, 600);
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].0, "Hoja 1");
        assert!(
            pages[0].1.contains("<svg") && pages[0].1.contains("<rect"),
            "svg inesperado: {}",
            pages[0].1
        );
        // Texto con XML se escapa y los lados se acotan a 64..=4096.
        book.current_mut()
            .expect("hoja disponible")
            .doc
            .add(WhiteboardElement::Text {
                at: (0.0, 0.0),
                text: "a<b>&c".to_string(),
                size: 14.0,
            });
        let pages = grafito_core::whiteboard_pages_to_svg(&book.to_persisted(), 10_000, 10);
        assert!(pages[0].1.contains("a&lt;b&gt;&amp;c"));
        assert!(pages[0].1.contains("width=\"4096\""));
        assert!(pages[0].1.contains("height=\"64\""));
    }

    // --- Lápiz potenciado (OpenBoard): grosor 1..=24 + color, resaltador
    //     al fondo, auto-formas del lápiz. ---

    fn pencil_drag(session: &mut WhiteboardSession, points: &[(f64, f64)]) {
        let first = points[0];
        session.handle_pointer(first, true, false);
        for point in &points[1..] {
            session.handle_pointer(*point, false, false);
        }
        session.handle_pointer(points[points.len() - 1], false, true);
    }

    #[test]
    fn pencil_keeps_chosen_width_up_to_24_and_color() {
        // Grosor máximo 24 y color de la paleta llegan intactos al
        // elemento (la paleta de 8 se reutiliza, sin colores nuevos).
        let mut session = WhiteboardSession::default();
        assert_eq!(session.pen_width, 2.0, "default histórico intacto");
        session.set_tool(WhiteboardTool::Pencil);
        session.pen_width = 24.0;
        session.pen_color = egui::Color32::from_rgb(196, 91, 91);
        pencil_drag(&mut session, &[(0.0, 0.0), (50.0, 10.0), (100.0, 0.0)]);
        assert_eq!(session.doc.len(), 1);
        match &session.doc.elements()[0] {
            WhiteboardElement::Stroke { color, width, .. } => {
                assert_eq!(*color, (196, 91, 91));
                assert_eq!(*width, 24.0);
            }
            other => panic!("trazo esperado, fue: {other:?}"),
        }
    }

    #[test]
    fn highlighter_lands_below_ink_with_generous_width() {
        // Tinta primero, resaltado después: el resaltado queda en el
        // índice 0 (debajo) con ancho generoso aunque el slider esté bajo.
        let mut session = WhiteboardSession::default();
        session.set_tool(WhiteboardTool::Pencil);
        pencil_drag(&mut session, &[(0.0, 0.0), (50.0, 0.0), (100.0, 0.0)]);
        assert_eq!(session.doc.len(), 1);
        session.set_tool(WhiteboardTool::Highlighter);
        session.pen_width = 2.0;
        pencil_drag(&mut session, &[(-5.0, -5.0), (50.0, 5.0), (105.0, -5.0)]);
        assert_eq!(session.doc.len(), 2);
        match &session.doc.elements()[0] {
            WhiteboardElement::Highlighter { width, .. } => {
                assert!(
                    *width >= grafito_whiteboard::HIGHLIGHTER_MIN_WIDTH,
                    "ancho generoso aunque el slider pida 2: {width}"
                );
            }
            other => panic!("resaltado al fondo esperado, fue: {other:?}"),
        }
        assert!(
            matches!(session.doc.elements()[1], WhiteboardElement::Stroke { .. }),
            "la tinta queda encima del resaltado"
        );
    }

    #[test]
    fn auto_shapes_turns_circle_and_line_into_shapes_but_keeps_scribbles() {
        // Apagado por defecto: sin sorpresas en el gesto.
        assert!(!WhiteboardSession::default().auto_shapes);
        let mut session = WhiteboardSession::default();
        session.set_tool(WhiteboardTool::Pencil);
        session.auto_shapes = true;
        // Círculo de radio 50 → Ellipse.
        let circle: Vec<(f64, f64)> = (0..33)
            .map(|i| {
                let angle = 2.0 * std::f64::consts::PI * f64::from(i) / 32.0;
                (100.0 + 50.0 * angle.cos(), 100.0 + 50.0 * angle.sin())
            })
            .collect();
        pencil_drag(&mut session, &circle);
        assert_eq!(session.doc.len(), 1);
        assert!(
            matches!(session.doc.elements()[0], WhiteboardElement::Ellipse { .. }),
            "el bucle se cierra a elipse, fue: {:?}",
            session.doc.elements()[0]
        );
        // Recta de 100 px → Arrow (la «recta» de la pizarra).
        let line: Vec<(f64, f64)> = (0..16)
            .map(|i| (f64::from(i) * (100.0 / 15.0), 200.0))
            .collect();
        pencil_drag(&mut session, &line);
        assert_eq!(session.doc.len(), 2);
        assert!(
            matches!(session.doc.elements()[1], WhiteboardElement::Arrow { .. }),
            "la recta se cierra a flecha, fue: {:?}",
            session.doc.elements()[1]
        );
        // Garabato → Stroke honesto (no se inventa forma).
        let scribble: Vec<(f64, f64)> = (0..20)
            .map(|i| {
                (
                    f64::from(i) * 10.0,
                    300.0 + if i % 2 == 0 { 0.0 } else { 40.0 },
                )
            })
            .collect();
        pencil_drag(&mut session, &scribble);
        assert_eq!(session.doc.len(), 3);
        assert!(
            matches!(session.doc.elements()[2], WhiteboardElement::Stroke { .. }),
            "el garabato queda trazo, fue: {:?}",
            session.doc.elements()[2]
        );
    }

    /// PNG mínimo válido con IHDR de `w`×`h` (cabecera: el modelo sniffea
    /// tamaño + límites sin decodificar píxeles).
    fn test_png(w: u32, h: u32) -> Vec<u8> {
        let mut bytes = vec![137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13];
        bytes.extend_from_slice(b"IHDR");
        bytes.extend_from_slice(&w.to_be_bytes());
        bytes.extend_from_slice(&h.to_be_bytes());
        bytes.extend_from_slice(&[8, 2, 0, 0, 0]);
        bytes
    }

    fn session_with_image() -> WhiteboardSession {
        let mut session = WhiteboardSession::default();
        session.set_tool(WhiteboardTool::Select);
        session
            .try_add(
                WhiteboardElement::new_image(test_png(100, 100), ((0.0, 0.0), (100.0, 50.0)))
                    .expect("imagen de test válida"),
            )
            .expect("try_add imagen");
        session
    }

    /// La validación UI rechaza bytes basura y sobre-límites con `Err` en
    /// español; la hoja llena también rechaza la imagen.
    #[test]
    fn whiteboard_image_validate_rejects_oversize() {
        let ok = WhiteboardElement::new_image(test_png(100, 100), ((0.0, 0.0), (10.0, 10.0)))
            .expect("válida");
        assert!(validate_whiteboard_element(&ok, 0).is_ok());
        let mut heavy = test_png(100, 100);
        heavy.resize(grafito_whiteboard::MAX_IMAGE_BYTES + 1, 0);
        let direct = WhiteboardElement::Image {
            bytes_png_jpeg: heavy,
            rect: ((0.0, 0.0), (10.0, 10.0)),
        };
        let err = validate_whiteboard_element(&direct, 0).expect_err("4MiB+1 fuera");
        assert!(err.contains("excede"), "causa honesta, fue: {err}");
        let garbage = WhiteboardElement::Image {
            bytes_png_jpeg: vec![1, 2, 3, 4],
            rect: ((0.0, 0.0), (10.0, 10.0)),
        };
        assert!(validate_whiteboard_element(&garbage, 0).is_err());
        assert!(validate_whiteboard_element(&ok, MAX_WHITEBOARD_ELEMENTS_PER_PAGE).is_err());
    }

    /// Mover con la herramienta Mover: press en el cuerpo + arrastre
    /// traslada el rect (zoom 1.0: mundo = pantalla).
    #[test]
    fn whiteboard_select_moves_image_body() {
        let mut session = session_with_image();
        session.handle_pointer((50.0, 25.0), true, false);
        assert_eq!(session.doc.selected(), Some(0));
        session.handle_pointer((60.0, 30.0), false, false);
        session.handle_pointer((60.0, 30.0), false, true);
        assert_eq!(
            session.doc.elements()[0].bounds(),
            Some(((10.0, 5.0), (110.0, 55.0)))
        );
        assert_eq!(session.doc.selected(), Some(0));
    }

    /// Escala con la manija SE: press en la esquina + arrastre lleva el SE
    /// al cursor fijando el NW.
    #[test]
    fn whiteboard_select_scales_image_with_handle() {
        let mut session = session_with_image();
        session.handle_pointer((100.0, 50.0), true, false);
        assert_eq!(session.doc.selected(), Some(0));
        session.handle_pointer((200.0, 100.0), false, false);
        session.handle_pointer((200.0, 100.0), false, true);
        assert_eq!(
            session.doc.elements()[0].bounds(),
            Some(((0.0, 0.0), (200.0, 100.0)))
        );
    }

    /// Sin imágenes bajo el punto, Mover sigue haciendo marquee (regresión).
    #[test]
    fn whiteboard_select_marquee_without_images() {
        let mut session = WhiteboardSession::default();
        session.set_tool(WhiteboardTool::Select);
        session
            .try_add(WhiteboardElement::Rectangle {
                min: (0.0, 0.0),
                max: (4.0, 4.0),
                fill: None,
            })
            .expect("try_add rectángulo");
        session.handle_pointer((1.0, 1.0), true, false);
        session.handle_pointer((3.0, 3.0), false, false);
        session.handle_pointer((3.0, 3.0), false, true);
        assert_eq!(session.doc.selected(), Some(0));
        assert_eq!(session.doc.len(), 1);
    }

    /// Decodificación real PNG vía `image` (ya declarado): 4×3 entra,
    /// basura y dimensiones hostiles se omiten con `None` honesto.
    #[test]
    fn whiteboard_image_decode_roundtrip() {
        let dynamic = image::DynamicImage::new_rgba8(4, 3);
        let mut buf = Vec::new();
        dynamic
            .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .expect("encode png de test");
        let color = decode_whiteboard_image(&buf).expect("decodifica");
        assert_eq!((color.width(), color.height()), (4, 3));
        assert!(decode_whiteboard_image(&[1, 2, 3]).is_none());
        assert!(decode_whiteboard_image(&test_png(100, 100)).is_none());
    }

    // --- Cortina + lupa (OpenBoard): overlay de sesión, slot único, sin
    //     persistencia; lupa honesta como zoom-to-point 2x. ---

    #[test]
    fn whiteboard_curtain_reveals_pure_visible_area() {
        // Al mostrar cubre todo (visible 0); el borde se arrastra en px de
        // pantalla y revela progresivo con clamp, sin tocar el documento.
        let mut session = WhiteboardSession::default();
        assert!(session.curtain.is_none());
        session.toggle_curtain();
        assert_eq!(session.curtain_direction, CurtainDirection::Top);
        assert_eq!(
            session.curtain.expect("cortina visible").visible_fraction(),
            0.0,
            "al mostrar cubre toda la hoja"
        );
        assert!(session.doc.is_empty());
        // Borde a mitad de un lienzo de 300 px: revela la mitad exacta.
        session.apply_curtain_drag(150.0, 0.0, 300.0);
        assert!(
            (session.curtain.expect("sigue").visible_fraction() - 0.5).abs() < 1e-12,
            "mitad revelada"
        );
        // Agarre del borde en px (±10): sobre el borde sí, lejos no.
        assert!(session.curtain_edge_hit(150.0, 0.0, 300.0));
        assert!(!session.curtain_edge_hit(250.0, 0.0, 300.0));
        // Fuera del lienzo se clampa a revelado total, sin pasarse.
        session.apply_curtain_drag(-50.0, 0.0, 300.0);
        assert_eq!(session.curtain.expect("sigue").visible_fraction(), 1.0);
        // Cambiar arriba/abajo conserva el área cubierta (Top 0.25 → Bottom 0.75).
        session.apply_curtain_drag(75.0, 0.0, 300.0);
        session.flip_curtain_direction();
        assert_eq!(session.curtain_direction, CurtainDirection::Bottom);
        let flipped = session.curtain.expect("sigue visible");
        assert!((flipped.covered_fraction() - 0.25).abs() < 1e-12);
        // Clic de nuevo la oculta (slot único).
        session.toggle_curtain();
        assert!(session.curtain.is_none());
        assert!(session.doc.is_empty(), "la cortina jamás dibuja");
    }

    #[test]
    fn whiteboard_magnifier_is_clamped_zoom_to_point() {
        // La lupa es zoom-to-point 2x honesto (no re-dibujo): duplica el
        // zoom anclando el punto y respeta el clamp existente.
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(400.0, 300.0));
        let mut session = WhiteboardSession::default();
        assert_eq!(session.zoom, ZOOM_WB_DEFAULT, "default intacto");
        session.toggle_magnifier();
        assert!(session.magnifier_active);
        let before = session.world_from_screen(rect.center(), rect);
        session.apply_magnifier_at(rect.center(), rect);
        assert_eq!(session.zoom, ZOOM_WB_DEFAULT * MAGNIFIER_FACTOR);
        let after = session.world_from_screen(rect.center(), rect);
        assert!(
            (before.0 - after.0).abs() < 1e-9 && (before.1 - after.1).abs() < 1e-9,
            "el punto clicado queda fijo: {before:?} vs {after:?}"
        );
        // En el tope no se pasa (clamp): equivale al puro `magnifier_zoom`.
        session.zoom = ZOOM_WB_MAX;
        session.apply_magnifier_at(rect.center(), rect);
        assert_eq!(session.zoom, ZOOM_WB_MAX);
        assert_eq!(
            session.zoom,
            magnifier_zoom(ZOOM_WB_MAX, ZOOM_WB_MIN, ZOOM_WB_MAX)
        );
        session.zoom = ZOOM_WB_MIN;
        session.apply_magnifier_at(rect.center(), rect);
        assert!(session.zoom >= ZOOM_WB_MIN);
        // Clic de nuevo la apaga; el zoom resultante persiste como vista.
        session.toggle_magnifier();
        assert!(!session.magnifier_active);
        assert!(session.doc.is_empty(), "la lupa jamás dibuja");
    }

    #[test]
    fn whiteboard_single_overlay_slot_covers_curtain_and_magnifier() {
        // Un overlay a la vez: cortina/lupa/instrumentos/compás se pisan.
        let mut session = WhiteboardSession::default();
        session.toggle_instrument(InstrumentKind::Ruler);
        assert!(session.instrument.is_some());
        session.toggle_curtain();
        assert!(session.curtain.is_some(), "la cortina entra al slot");
        assert!(session.instrument.is_none(), "guarda la regla");
        assert!(!session.magnifier_active);
        assert!(!session.compass_active);
        session.toggle_magnifier();
        assert!(session.magnifier_active, "la lupa entra al slot");
        assert!(session.curtain.is_none(), "guarda la cortina");
        session.toggle_compass();
        assert!(session.compass_active);
        assert!(!session.magnifier_active, "el compás guarda la lupa");
        session.toggle_magnifier();
        session.toggle_instrument(InstrumentKind::Triangle);
        assert_eq!(session.instrument_kind(), Some(InstrumentKind::Triangle));
        assert!(!session.magnifier_active, "el instrumento guarda la lupa");
        session.toggle_curtain();
        session.toggle_instrument(InstrumentKind::Protractor);
        assert_eq!(session.instrument_kind(), Some(InstrumentKind::Protractor));
        assert!(
            session.curtain.is_none(),
            "el instrumento guarda la cortina"
        );
    }

    #[test]
    fn whiteboard_curtain_and_magnifier_never_persist_or_reach_svg() {
        // Dibujar con cortina/lupa visibles solo guarda tinta: ni el JSON
        // del documento ni el SVG pueden nombrarlos (PROHIBIDO persistir).
        let mut session = WhiteboardSession::default();
        session.toggle_curtain();
        session.set_tool(WhiteboardTool::Pencil);
        pencil_drag(&mut session, &[(0.0, 0.0), (50.0, 0.0), (100.0, 0.0)]);
        session.toggle_magnifier();
        pencil_drag(&mut session, &[(0.0, 20.0), (50.0, 20.0), (100.0, 20.0)]);
        assert_eq!(session.doc.len(), 2);
        let json = match serde_json::to_string(&session.doc) {
            Ok(json) => json,
            Err(error) => panic!("documento serializa: {error}"),
        };
        for key in [
            "Curtain",
            "curtain",
            "Magnifier",
            "magnifier",
            "MAGNIFIER",
            "CURTAIN",
        ] {
            assert!(!json.contains(key), "fuga al documento: {key}");
        }
        let svg = match session.doc.to_svg(800, 600) {
            Ok(svg) => svg,
            Err(error) => panic!("SVG serializa: {error}"),
        };
        for key in ["Curtain", "curtain", "Magnifier", "magnifier"] {
            assert!(!svg.contains(key), "fuga al SVG: {key}");
        }
        // El libro no se ensucia por mostrar overlays (solo doc/pan/zoom).
        let mut book = WhiteboardBook::default();
        let mut clean = WhiteboardSession::default();
        book.load_to_session(&mut clean);
        book.mark_synced();
        clean.toggle_curtain();
        clean.toggle_magnifier();
        clean.flip_curtain_direction();
        book.save_current_from_session(&clean);
        assert!(book.is_synced(), "los overlays no ensucian el libro");
    }

    #[test]
    fn whiteboard_share_png_renders_valid_png_with_signature_and_dims() {
        // El botón PNG existe (acción cableada) y su render a
        // `WHITEBOARD_PNG_EXPORT_SCALE` da un PNG válido: firma de 8 bytes
        // + `IHDR` con las DIMs esperadas (parseo manual big-endian).
        let _ = WhiteboardShareAction::SavePng;
        let mut doc = WhiteboardDoc::new();
        doc.add(WhiteboardElement::Rectangle {
            min: (0.0, 0.0),
            max: (10.0, 10.0),
            fill: None,
        });
        // Encuadre 50×50 mundo (10u + margen 20) a escala 2 → 100×100px.
        let bytes = render_current_page_png(&doc, (0x13, 0x15, 0x18), true)
            .expect("rectángulo chico renderiza");
        assert_eq!(&bytes[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
        assert_eq!(&bytes[12..16], b"IHDR");
        let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
        assert_eq!((width, height), (100, 100));
    }

    #[test]
    fn whiteboard_share_png_warns_when_page_has_text() {
        // Aviso honesto: nombra la omisión del PNG y la vía completa (SVG),
        // una sola ortografía compartida botón↔test; el render con texto
        // sigue saliendo (la letra se omite, no falla).
        assert!(
            WHITEBOARD_PNG_TEXT_WARNING.contains("PNG no incluye texto"),
            "el aviso nombra la omisión: {WHITEBOARD_PNG_TEXT_WARNING}"
        );
        assert!(
            WHITEBOARD_PNG_TEXT_WARNING.contains("SVG"),
            "el aviso nombra la vía completa: {WHITEBOARD_PNG_TEXT_WARNING}"
        );
        assert!(!crate::whiteboard_png::png_skips_text(&WhiteboardDoc::new()));
        let mut doc = WhiteboardDoc::new();
        doc.add(WhiteboardElement::Text {
            at: (13.0, 17.0),
            text: "hola".to_string(),
            size: 12.0,
        });
        assert!(crate::whiteboard_png::png_skips_text(&doc));
        assert!(render_current_page_png(&doc, (0x13, 0x15, 0x18), true).is_ok());
        // Sin grid (fondo Blanco): mismo PNG válido, fondo liso.
        assert!(render_current_page_png(&doc, (0x13, 0x15, 0x18), false).is_ok());
    }
}
