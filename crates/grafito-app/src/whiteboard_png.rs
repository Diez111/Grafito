//! Raster PNG headless de la pizarra (Compartir → PNG).
//!
//! Rasterizador software propio y chico: dibuja sobre un búfer RGBA8 con
//! primitivas implementadas aquí (Bresenham + sello circular para grosor,
//! elipse por muestreo paramétrico denso, triángulo por edge-functions,
//! `src-over` para alfa, blit por vecino más cercano). El crate `image`
//! —ya declarado en `grafito-app`, sin altas nuevas— se usa SOLO como
//! codec: decodifica los `Image` (PNG/JPEG) y codifica el PNG final. Nada
//! de `resvg`/`tiny-skia`/`ab_glyph` para dibujar.
//!
//! Qué INCLUYE el PNG: fondo (`canvas_bg` del tema actual vía
//! [`PngOptions`], oscuro DARK por defecto), grid sutil de 40u cuando el
//! paso en píxeles ≥ 6 (misma condición de corte que `draw_grid`), trazos
//! (`Stroke`), resaltado (`Highlighter` con el [`HIGHLIGHTER_ALPHA`] del
//! modelo), rectángulos (relleno + borde), elipses, flechas (fuste +
//! triángulo con [`arrow_tip`]) e imágenes bliteadas con su alfa.
//!
//! Qué EXCLUYE (honesto, no silencioso): el TEXTO. Sin raster de glifos
//! sin dependencias (`ab_glyph`/`tiny-skia` vetados por contrato: solo
//! `image`), no hay forma fiel de pintar letras; los `Text` se omiten y
//! [`png_skips_text`] lo avisa para el diálogo (el SVG sigue siendo la
//! vía completa, ese sí trae texto). Las imágenes que `image` no
//! decodifica se omiten como en el canvas (sin romper la hoja).
//!
//! Colores espejo del canvas (`WhiteboardSession::effective_pen_color`):
//! la tinta guardada se respeta salvo bajo contraste (claro-sobre-claro
//! u oscuro-sobre-oscuro → `text_primary` del tema que toque); las formas
//! usan `text_primary` directo, igual que `draw_element`.
//!
//! Cotas fail-closed (nunca OOM ni downscale silencioso): lado de salida
//! ≤ [`MAX_WHITEBOARD_PNG_SIDE`] (2048, espejo de `MAX_IMAGE_SIDE_PX`),
//! búfer ≤ [`MAX_WHITEBOARD_PNG_BYTES`] (12 MiB) con chequeo
//! `ancho×alto×4` en `u64` ANTES de allocar, escala finita > 0 acotada a
//! [`WHITEBOARD_PNG_SCALE_MIN`]..=[`WHITEBOARD_PNG_SCALE_MAX`]. Excedido
//! → `Err` en español con la causa, como el `to_svg` de 100 KiB.
//!
//! [`HIGHLIGHTER_ALPHA`]: grafito_whiteboard::HIGHLIGHTER_ALPHA
//! [`arrow_tip`]: grafito_whiteboard::arrow_tip

use grafito_whiteboard::{
    arrow_tip, WhiteboardDoc, WhiteboardElement, HIGHLIGHTER_ALPHA, MAX_IMAGE_BYTES,
    MAX_IMAGE_SIDE_PX,
};

/// Lado máximo del PNG de salida en píxeles (espejo de `MAX_IMAGE_SIDE_PX`:
/// lo que entra como imagen también sale como página).
pub const MAX_WHITEBOARD_PNG_SIDE: u32 = 2048;
/// Techo del búfer RGBA (`ancho×alto×4`) antes de allocar: 12 MiB. Cabe un
/// 1600×1200 (hoja vacía a escala 2: 7.7 MiB) con margen; más → `Err`.
pub const MAX_WHITEBOARD_PNG_BYTES: usize = 12 * 1024 * 1024;
/// Escala mínima y máxima aceptadas por [`render_png_rgba`] (el botón usa
/// [`WHITEBOARD_PNG_EXPORT_SCALE`]; fuera de rango se acota, documentado).
pub const WHITEBOARD_PNG_SCALE_MIN: f64 = 0.5;
pub const WHITEBOARD_PNG_SCALE_MAX: f64 = 4.0;
/// Escala del botón Compartir → PNG: 2× para nitidez sin reventar la cota
/// (hoja vacía 800×600 → 1600×1200 = 7.7 MiB < 12 MiB).
pub const WHITEBOARD_PNG_EXPORT_SCALE: f64 = 2.0;
/// Margen del encuadre alrededor del contenido (paridad con `to_svg`: 20u).
pub const WHITEBOARD_PNG_MARGIN: f64 = 20.0;
/// Encuadre de hoja vacía en mundo (paridad con `to_svg`: 800×600).
pub const WHITEBOARD_PNG_EMPTY_WIDTH: f64 = 800.0;
pub const WHITEBOARD_PNG_EMPTY_HEIGHT: f64 = 600.0;
/// Paso del grid en mundo (paridad con `draw_grid`: 40u) y paso mínimo en
/// píxeles para pintarlo (misma condición de corte que `draw_grid`: 6px;
/// "grid si cabe limpio").
pub const WHITEBOARD_PNG_GRID_STEP: f64 = 40.0;
pub const WHITEBOARD_PNG_GRID_MIN_STEP_PX: f64 = 6.0;
/// Fondo por defecto: canvas DARK `#131518` (SLATE_N0). El botón pasa el
/// `canvas_bg` del tema actual vía [`PngOptions`]; el puro usa este.
pub const WHITEBOARD_PNG_DEFAULT_BG: (u8, u8, u8) = (0x13, 0x15, 0x18);
/// Tinta de formas sobre fondo oscuro: `DARK.text_primary` `#FAFAF9`.
pub const WHITEBOARD_PNG_DARK_INK: (u8, u8, u8) = (250, 250, 249);
/// Tinta de formas sobre fondo claro: `LIGHT.text_primary` `#1A1A1A`
/// (igual que `WHITEBOARD_SVG_INK` del `to_svg`).
pub const WHITEBOARD_PNG_LIGHT_INK: (u8, u8, u8) = (0x1A, 0x1A, 0x1A);
/// Ancho cuando el guardado trae uno roto (paridad con `to_svg`: 2.0).
pub const WHITEBOARD_PNG_FALLBACK_WIDTH: f64 = 2.0;
/// Ancho de formas sin ancho propio (paridad con `to_svg`: 1.8).
pub const WHITEBOARD_PNG_SHAPE_WIDTH: f64 = 1.8;

/// Opciones del raster: fondo RGB y grid sí/no (el paso mínimo lo decide
/// el corte de 6px aunque `grid` sea `true`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PngOptions {
    /// Fondo del PNG (el botón pasa `theme.canvas_bg`).
    pub background: (u8, u8, u8),
    /// Pide grid de 40u; solo se pinta si el paso en píxeles ≥ 6.
    pub grid: bool,
}

impl Default for PngOptions {
    fn default() -> Self {
        Self {
            background: WHITEBOARD_PNG_DEFAULT_BG,
            grid: true,
        }
    }
}

/// `true` si el documento trae al menos un `Text`: el PNG lo omitirá
/// (ver encabezado del módulo) y el diálogo debe avisarlo. Conservador a
/// propósito: cualquier `Text` dispara el aviso aunque esté vacío.
pub fn png_skips_text(doc: &WhiteboardDoc) -> bool {
    doc.elements()
        .iter()
        .any(|element| matches!(element, WhiteboardElement::Text { .. }))
}

/// Rasteriza la hoja a PNG con fondo DARK y grid (ver [`PngOptions`]).
/// Puro: sin I/O, sin egui, sin clipboard. `Err` honesto en español.
///
/// (El par `render_png_bytes`/`_with` se retiró: copiar y guardar ya
/// pasan por [`render_png_rgba`] + [`encode_png_rgba`], y enlazar la
/// codificación dos veces no aportaba.)
pub(crate) fn render_png_rgba(
    doc: &WhiteboardDoc,
    escala: f64,
    options: &PngOptions,
) -> Result<(u32, u32, Vec<u8>), String> {
    if !escala.is_finite() || escala <= 0.0 {
        return Err("PNG no se generó; la escala debe ser finita y mayor que cero".to_string());
    }
    // Acote documentado (el botón pasa 2.0, dentro del rango).
    let escala = escala.clamp(WHITEBOARD_PNG_SCALE_MIN, WHITEBOARD_PNG_SCALE_MAX);
    let (view_x, view_y, view_w, view_h) = png_view_box(doc);
    let width_f = view_w * escala;
    let height_f = view_h * escala;
    if !width_f.is_finite() || !height_f.is_finite() {
        return Err("PNG no se generó; el encuadre de la hoja no es finito".to_string());
    }
    if width_f > f64::from(MAX_WHITEBOARD_PNG_SIDE) || height_f > f64::from(MAX_WHITEBOARD_PNG_SIDE)
    {
        return Err(format!(
            "PNG no se generó; la hoja a escala {escala} mide {width_f:.0}×{height_f:.0}px \
            y excede el lado máximo de {}px (achicá el contenido o la escala)",
            MAX_WHITEBOARD_PNG_SIDE
        ));
    }
    // `max(1.0)`: contenido diminuto (p. ej. un punto) da al menos 1px.
    let (width, height) = (
        width_f.round().max(1.0) as u32,
        height_f.round().max(1.0) as u32,
    );
    // Chequeo previo en u64 (sin overflow): excedido → Err, jamás OOM.
    let byte_len = u64::from(width)
        .checked_mul(u64::from(height))
        .and_then(|pixels| pixels.checked_mul(4));
    match byte_len {
        Some(len) if len <= MAX_WHITEBOARD_PNG_BYTES as u64 => {}
        _ => {
            return Err(format!(
                "PNG no se generó; {width}×{height}px necesitan más de {} MiB \
                (achicá el contenido o la escala)",
                MAX_WHITEBOARD_PNG_BYTES / (1024 * 1024)
            ));
        }
    }
    let mut canvas = Canvas::new(width, height, options.background);
    if options.grid {
        draw_grid(&mut canvas, view_x, view_y, escala, options.background);
    }
    for element in doc.elements() {
        draw_element(&mut canvas, element, view_x, view_y, escala, options);
    }
    Ok((width, height, canvas.into_raw()))
}

/// Codifica un RGBA8 fila-mayor a PNG con el crate `image` (codec, no
/// dibujo: el raster es propio, ver encabezado).
pub(crate) fn encode_png_rgba(width: u32, height: u32, rgba: &[u8]) -> Result<Vec<u8>, String> {
    if rgba.len() != width as usize * height as usize * 4 {
        return Err("PNG no se generó; el búfer RGBA no coincide con las dimensiones".to_string());
    }
    let buffer =
        image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(width, height, rgba.to_vec())
            .ok_or_else(|| {
                "PNG no se generó; el búfer RGBA no coincide con las dimensiones".to_string()
            })?;
    let mut out = Vec::new();
    image::DynamicImage::ImageRgba8(buffer)
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .map_err(|error| format!("PNG no se generó; el codificador falló: {error}"))?;
    if out.is_empty() {
        return Err("PNG no se generó; el codificador devolvió cero bytes".to_string());
    }
    Ok(out)
}

/// Encuadre como `to_svg`: envolvente de `bounds()` finitos + margen de
/// 20u; hoja vacía o sin nada finito → (0, 0, 800, 600).
fn png_view_box(doc: &WhiteboardDoc) -> (f64, f64, f64, f64) {
    let mut min = (f64::INFINITY, f64::INFINITY);
    let mut max = (f64::NEG_INFINITY, f64::NEG_INFINITY);
    let mut found = false;
    for element in doc.elements() {
        let Some((a, b)) = element.bounds() else {
            continue;
        };
        if !a.0.is_finite() || !a.1.is_finite() || !b.0.is_finite() || !b.1.is_finite() {
            continue;
        }
        min.0 = min.0.min(a.0.min(b.0));
        min.1 = min.1.min(a.1.min(b.1));
        max.0 = max.0.max(a.0.max(b.0));
        max.1 = max.1.max(a.1.max(b.1));
        found = true;
    }
    if found {
        (
            min.0 - WHITEBOARD_PNG_MARGIN,
            min.1 - WHITEBOARD_PNG_MARGIN,
            (max.0 - min.0 + 2.0 * WHITEBOARD_PNG_MARGIN).max(1.0),
            (max.1 - min.1 + 2.0 * WHITEBOARD_PNG_MARGIN).max(1.0),
        )
    } else {
        (
            0.0,
            0.0,
            WHITEBOARD_PNG_EMPTY_WIDTH,
            WHITEBOARD_PNG_EMPTY_HEIGHT,
        )
    }
}

/// Espejo headless de `WhiteboardSession::effective_pen_color` (mismos
/// umbrales 200/60; el histórico 240,241,244 cae en `pen_light`): tinta
/// guardada salvo bajo contraste, que va a `text_primary` del fondo.
fn effective_ink(stored: (u8, u8, u8), background: (u8, u8, u8)) -> (u8, u8, u8) {
    let light_bg = background.0 > 200;
    let pen = if light_bg {
        WHITEBOARD_PNG_LIGHT_INK
    } else {
        WHITEBOARD_PNG_DARK_INK
    };
    let pen_light = stored.0 > 200 && stored.1 > 200 && stored.2 > 200;
    let pen_dark = stored.0 < 60 && stored.1 < 60 && stored.2 < 60;
    if (light_bg && pen_light) || (!light_bg && pen_dark) {
        pen
    } else {
        stored
    }
}

/// Tinta de formas: `text_primary` del fondo (igual que `draw_element`, que
/// usa `theme.text_primary` para rectángulos/elipses/flechas).
fn shape_ink(background: (u8, u8, u8)) -> (u8, u8, u8) {
    if background.0 > 200 {
        WHITEBOARD_PNG_LIGHT_INK
    } else {
        WHITEBOARD_PNG_DARK_INK
    }
}

/// Grid de 40u en múltiplos del origen mundo, 1px, sutil: aproximación
/// headless documentada del `separator.gamma_multiply(0.10/0.35)` del
/// canvas (mezcla lineal 10 % hacia blanco en oscuro, 12 % hacia negro en
/// claro; no replica el gamma de egui píxel a píxel). Solo si el paso en
/// píxeles ≥ 6, como `draw_grid`.
fn draw_grid(canvas: &mut Canvas, view_x: f64, view_y: f64, escala: f64, background: (u8, u8, u8)) {
    if WHITEBOARD_PNG_GRID_STEP * escala < WHITEBOARD_PNG_GRID_MIN_STEP_PX {
        return;
    }
    let target = if background.0 > 200 {
        (0, 0, 0)
    } else {
        (255, 255, 255)
    };
    let amount = if background.0 > 200 { 0.12 } else { 0.10 };
    let color = mix_rgb(background, target, amount);
    // A diferencia del canvas (origen en centro+pan), el headless no tiene
    // vista: líneas en múltiplos de 40u dentro del encuadre.
    let view_w = f64::from(canvas.width) / escala;
    let view_h = f64::from(canvas.height) / escala;
    let mut k = (view_x / WHITEBOARD_PNG_GRID_STEP).ceil() as i64;
    loop {
        let world = k as f64 * WHITEBOARD_PNG_GRID_STEP;
        if world > view_x + view_w {
            break;
        }
        let px = ((world - view_x) * escala).round() as i64;
        canvas.vline(px, color);
        k += 1;
        if k > 1_000_000 {
            break; // Fail-closed: mundo hostil no cuelga el export.
        }
    }
    let mut k = (view_y / WHITEBOARD_PNG_GRID_STEP).ceil() as i64;
    loop {
        let world = k as f64 * WHITEBOARD_PNG_GRID_STEP;
        if world > view_y + view_h {
            break;
        }
        let px = ((world - view_y) * escala).round() as i64;
        canvas.hline(px, color);
        k += 1;
        if k > 1_000_000 {
            break;
        }
    }
}

/// Ancho sano en mundo (paridad `to_svg`: 2.0 si roto) pasado a píxeles.
fn width_px(width: f64, escala: f64) -> f64 {
    let safe = if width.is_finite() && width > 0.0 {
        width
    } else {
        WHITEBOARD_PNG_FALLBACK_WIDTH
    };
    (safe * escala).max(1.0)
}

/// Dibuja un elemento (no finitos se omiten, como `to_svg` y el canvas).
/// El `Text` se omite a propósito (ver encabezado; avisa
/// [`png_skips_text`]).
#[allow(clippy::too_many_arguments)]
fn draw_element(
    canvas: &mut Canvas,
    element: &WhiteboardElement,
    view_x: f64,
    view_y: f64,
    escala: f64,
    options: &PngOptions,
) {
    let to_px = |point: (f64, f64)| {
        (
            ((point.0 - view_x) * escala).round() as i64,
            ((point.1 - view_y) * escala).round() as i64,
        )
    };
    match element {
        WhiteboardElement::Stroke {
            points,
            color,
            width,
        }
        | WhiteboardElement::Highlighter {
            points,
            color,
            width,
        } => {
            let highlight = matches!(element, WhiteboardElement::Highlighter { .. });
            let finite: Vec<(f64, f64)> = points
                .iter()
                .copied()
                .filter(|point| point.0.is_finite() && point.1.is_finite())
                .collect();
            if finite.len() < 2 {
                return;
            }
            let ink = effective_ink(*color, options.background);
            // Alfa fijo del modelo (0.35): translúcido de verdad, igual que
            // `draw_element` (`gamma_multiply(HIGHLIGHTER_ALPHA)`) y el SVG
            // (`stroke-opacity`). `src-over` por canal, documentado.
            let alpha = if highlight {
                Some(HIGHLIGHTER_ALPHA)
            } else {
                None
            };
            let width = width_px(*width, escala);
            for pair in finite.windows(2) {
                let a = to_px(pair[0]);
                let b = to_px(pair[1]);
                canvas.thick_segment(a, b, width, ink, alpha);
            }
        }
        WhiteboardElement::Rectangle { min, max, fill } => {
            if !min.0.is_finite() || !min.1.is_finite() || !max.0.is_finite() || !max.1.is_finite()
            {
                return;
            }
            let (x0, y0) = to_px((min.0.min(max.0), min.1.min(max.1)));
            let (x1, y1) = to_px((min.0.max(max.0), min.1.max(max.1)));
            if x1 <= x0 || y1 <= y0 {
                return;
            }
            if let Some(rgb) = fill {
                canvas.fill_rect(x0, y0, x1, y1, *rgb);
            }
            let ink = shape_ink(options.background);
            let width = width_px(WHITEBOARD_PNG_SHAPE_WIDTH, escala);
            canvas.thick_segment((x0, y0), (x1, y0), width, ink, None);
            canvas.thick_segment((x1, y0), (x1, y1), width, ink, None);
            canvas.thick_segment((x1, y1), (x0, y1), width, ink, None);
            canvas.thick_segment((x0, y1), (x0, y0), width, ink, None);
        }
        WhiteboardElement::Ellipse { center, rx, ry } => {
            if !center.0.is_finite()
                || !center.1.is_finite()
                || !rx.is_finite()
                || !ry.is_finite()
                || *rx <= 0.0
                || *ry <= 0.0
            {
                return;
            }
            // Muestreo paramétrico denso (~1 punto cada 1px de perímetro de
            // Ramanujan, 16..=2048) con el mismo sello que las polilíneas:
            // simple, sin midpoint entero, grosor uniforme.
            let (cx, cy) = to_px(*center);
            let (a, b) = (rx * escala, ry * escala);
            let perimeter =
                std::f64::consts::PI * (3.0 * (a + b) - ((3.0 * a + b) * (a + 3.0 * b)).sqrt());
            let steps = (perimeter.ceil() as usize).clamp(16, 2048);
            let ink = shape_ink(options.background);
            let width = width_px(WHITEBOARD_PNG_SHAPE_WIDTH, escala);
            let mut prev: Option<(i64, i64)> = None;
            for i in 0..=steps {
                let angle = 2.0 * std::f64::consts::PI * i as f64 / steps as f64;
                let point = (
                    cx + (a * angle.cos()).round() as i64,
                    cy + (b * angle.sin()).round() as i64,
                );
                if let Some(from) = prev {
                    canvas.thick_segment(from, point, width, ink, None);
                }
                prev = Some(point);
            }
        }
        WhiteboardElement::Arrow { from, to } => {
            if !from.0.is_finite() || !from.1.is_finite() || !to.0.is_finite() || !to.1.is_finite()
            {
                return;
            }
            let length = ((to.0 - from.0).powi(2) + (to.1 - from.1).powi(2)).sqrt();
            if !length.is_finite() || length <= 0.0 {
                return;
            }
            let ink = shape_ink(options.background);
            let width = width_px(WHITEBOARD_PNG_SHAPE_WIDTH, escala);
            let a = to_px(*from);
            let b = to_px(*to);
            canvas.thick_segment(a, b, width, ink, None);
            // Punta proporcional con el mismo clamp que el SVG
            // (`(largo*0.12).clamp(4, 24)`), rellena sólida.
            let head = (length * 0.12).clamp(4.0, 24.0);
            let (wing_right, wing_left) = arrow_tip(*from, *to, head);
            let tip = to_px(*to);
            let right = to_px(wing_right);
            let left = to_px(wing_left);
            canvas.fill_triangle(tip, right, left, ink);
        }
        WhiteboardElement::Text { .. } => {
            // Omitido honesto (sin raster de glifos sin deps): avisa
            // `png_skips_text`; el SVG es la vía completa. No se pinta
            // cajita ni marcador: nada inventado.
        }
        WhiteboardElement::Image {
            bytes_png_jpeg,
            rect,
        } => {
            if !rect.0 .0.is_finite()
                || !rect.0 .1.is_finite()
                || !rect.1 .0.is_finite()
                || !rect.1 .1.is_finite()
            {
                return;
            }
            let (x0, y0) = to_px((rect.0 .0.min(rect.1 .0), rect.0 .1.min(rect.1 .1)));
            let (x1, y1) = to_px((rect.0 .0.max(rect.1 .0), rect.0 .1.max(rect.1 .1)));
            if x1 <= x0 || y1 <= y0 {
                return;
            }
            // Decode con `image` (codec, mismo que el canvas vía
            // `decode_whiteboard_image`); lo no-decodificable se omite sin
            // romper la hoja. Capa de defensa en píxeles (el modelo ya
            // acota a 2048 por lado al insertar).
            let Some((src_w, src_h, src)) = decode_image(bytes_png_jpeg) else {
                return;
            };
            canvas.blit_nearest((&src, src_w, src_h), (x0, y0, x1 - x0, y1 - y0));
        }
    }
}

/// Decodifica PNG/JPEG a RGBA8 crudo con `image`; `None` honesto si no
/// decodifica o excede 2048 por lado (paridad con `decode_whiteboard_image`
/// del canvas, que también omite sin romper).
fn decode_image(bytes: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    if bytes.is_empty() || bytes.len() > MAX_IMAGE_BYTES {
        return None;
    }
    let rgba = image::load_from_memory(bytes).ok()?.to_rgba8();
    let (width, height) = (rgba.width(), rgba.height());
    if width == 0 || height == 0 || width > MAX_IMAGE_SIDE_PX || height > MAX_IMAGE_SIDE_PX {
        return None;
    }
    let raw = rgba.into_raw();
    if raw.len() != width as usize * height as usize * 4 {
        return None;
    }
    Some((width, height, raw))
}

/// Mezcla lineal `base←objetivo` en `amount` 0..=1 por canal.
fn mix_rgb(base: (u8, u8, u8), target: (u8, u8, u8), amount: f32) -> (u8, u8, u8) {
    let mix = |b: u8, t: u8| (f32::from(b) + (f32::from(t) - f32::from(b)) * amount).round() as u8;
    (
        mix(base.0, target.0),
        mix(base.1, target.1),
        mix(base.2, target.2),
    )
}

/// Búfer RGBA8 fila-mayor con primitivas propias. Todo recorta al lienzo;
/// jamás panic por coordenadas (mundo hostil → píxeles fuera → se ignoran).
struct Canvas {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl Canvas {
    fn new(width: u32, height: u32, background: (u8, u8, u8)) -> Self {
        let mut pixels = vec![0u8; width as usize * height as usize * 4];
        for chunk in pixels.chunks_exact_mut(4) {
            chunk[0] = background.0;
            chunk[1] = background.1;
            chunk[2] = background.2;
            chunk[3] = 255;
        }
        Self {
            width,
            height,
            pixels,
        }
    }

    fn into_raw(self) -> Vec<u8> {
        self.pixels
    }

    fn in_bounds(&self, x: i64, y: i64) -> bool {
        x >= 0 && y >= 0 && x < i64::from(self.width) && y < i64::from(self.height)
    }

    /// Píxel opaco.
    fn plot(&mut self, x: i64, y: i64, rgb: (u8, u8, u8)) {
        if !self.in_bounds(x, y) {
            return;
        }
        let offset = (y as usize * self.width as usize + x as usize) * 4;
        self.pixels[offset] = rgb.0;
        self.pixels[offset + 1] = rgb.1;
        self.pixels[offset + 2] = rgb.2;
        self.pixels[offset + 3] = 255;
    }

    /// `src-over`: `salida = src·a + dst·(1−a)` por canal (alfa del
    /// resaltador y de imágenes con transparencia).
    fn blend(&mut self, x: i64, y: i64, rgb: (u8, u8, u8), alpha: f32) {
        if !self.in_bounds(x, y) {
            return;
        }
        let alpha = alpha.clamp(0.0, 1.0);
        let offset = (y as usize * self.width as usize + x as usize) * 4;
        for (i, channel) in [rgb.0, rgb.1, rgb.2].iter().enumerate() {
            let dst = f32::from(self.pixels[offset + i]);
            self.pixels[offset + i] =
                (f32::from(*channel) * alpha + dst * (1.0 - alpha)).round() as u8;
        }
        self.pixels[offset + 3] = 255;
    }

    /// Sello circular sólido de radio `radius` (disco por distancia
    /// euclídea): con `radius` 0 pinta un píxel. Es lo que da grosor y
    /// remates/redondeos a las polilíneas ("Bresenham grueso"): el grosor
    /// real difiere ±1px del pedido, documentado, no defecto.
    fn stamp(&mut self, cx: i64, cy: i64, radius: i64, rgb: (u8, u8, u8), alpha: Option<f32>) {
        if radius <= 0 {
            match alpha {
                Some(a) => self.blend(cx, cy, rgb, a),
                None => self.plot(cx, cy, rgb),
            }
            return;
        }
        let squared = radius * radius;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy > squared {
                    continue;
                }
                match alpha {
                    Some(a) => self.blend(cx + dx, cy + dy, rgb, a),
                    None => self.plot(cx + dx, cy + dy, rgb),
                }
            }
        }
    }

    /// Segmento con grosor: retícula de Bresenham entera (los 8 octantes)
    /// + sello circular en cada punto (uniones y remates redondos gratis).
    fn thick_segment(
        &mut self,
        from: (i64, i64),
        to: (i64, i64),
        width_px: f64,
        rgb: (u8, u8, u8),
        alpha: Option<f32>,
    ) {
        let radius = (width_px / 2.0).floor().max(0.0) as i64;
        bresenham(from.0, from.1, to.0, to.1, |x, y| {
            self.stamp(x, y, radius, rgb, alpha);
        });
    }

    /// Rectángulo sólido `[x0,x1)×[y0,y1)` recortado al lienzo.
    fn fill_rect(&mut self, x0: i64, y0: i64, x1: i64, y1: i64, rgb: (u8, u8, u8)) {
        let (x0, x1) = (
            x0.max(0).min(i64::from(self.width)),
            x1.max(0).min(i64::from(self.width)),
        );
        let (y0, y1) = (
            y0.max(0).min(i64::from(self.height)),
            y1.max(0).min(i64::from(self.height)),
        );
        for y in y0..y1 {
            for x in x0..x1 {
                self.plot(x, y, rgb);
            }
        }
    }

    /// Línea vertical/horizontal de 1px (grid), recortada.
    fn vline(&mut self, x: i64, rgb: (u8, u8, u8)) {
        if x < 0 || x >= i64::from(self.width) {
            return;
        }
        for y in 0..i64::from(self.height) {
            self.plot(x, y, rgb);
        }
    }

    fn hline(&mut self, y: i64, rgb: (u8, u8, u8)) {
        if y < 0 || y >= i64::from(self.height) {
            return;
        }
        for x in 0..i64::from(self.width) {
            self.plot(x, y, rgb);
        }
    }

    /// Triángulo sólido por edge-functions sobre su bbox entera (para la
    /// punta de flecha): píxel adentro si está del mismo lado de las tres
    /// aristas (orientación consistente).
    fn fill_triangle(&mut self, a: (i64, i64), b: (i64, i64), c: (i64, i64), rgb: (u8, u8, u8)) {
        let edge = |p: (i64, i64), q: (i64, i64), r: (i64, i64)| {
            (r.0 - p.0) * (q.1 - p.1) - (r.1 - p.1) * (q.0 - p.0)
        };
        let area = edge(a, b, c);
        if area == 0 {
            return;
        }
        let (x0, x1) = (a.0.min(b.0).min(c.0), a.0.max(b.0).max(c.0));
        let (y0, y1) = (a.1.min(b.1).min(c.1), a.1.max(b.1).max(c.1));
        for y in y0..=y1 {
            for x in x0..=x1 {
                let inside = if area > 0 {
                    edge(a, b, (x, y)) >= 0 && edge(b, c, (x, y)) >= 0 && edge(c, a, (x, y)) >= 0
                } else {
                    edge(a, b, (x, y)) <= 0 && edge(b, c, (x, y)) <= 0 && edge(c, a, (x, y)) <= 0
                };
                if inside {
                    self.plot(x, y, rgb);
                }
            }
        }
    }

    /// Blit con reescalado por vecino más cercano (documentado: sin filtro;
    /// bordes duros si el destino no es 1:1, a propósito —nada de blur
    /// silencioso—). Respeta el alfa de la fuente vía `src-over`.
    /// `src` es `(píxeles RGBA8, ancho, alto)`; `dest` es `(x, y, w, h)`.
    fn blit_nearest(&mut self, src: (&[u8], u32, u32), dest: (i64, i64, i64, i64)) {
        let (pixels, src_w, src_h) = src;
        let (dx0, dy0, dw, dh) = dest;
        if dw <= 0 || dh <= 0 || src_w == 0 || src_h == 0 {
            return;
        }
        for dy in 0..dh {
            let sy = (dy as u64 * u64::from(src_h) / dh as u64).min(u64::from(src_h) - 1) as usize;
            for dx in 0..dw {
                let sx =
                    (dx as u64 * u64::from(src_w) / dw as u64).min(u64::from(src_w) - 1) as usize;
                let offset = (sy * src_w as usize + sx) * 4;
                let (Some(r), Some(g), Some(b), Some(a)) = (
                    pixels.get(offset),
                    pixels.get(offset + 1),
                    pixels.get(offset + 2),
                    pixels.get(offset + 3),
                ) else {
                    continue;
                };
                self.blend(dx0 + dx, dy0 + dy, (*r, *g, *b), f32::from(*a) / 255.0);
            }
        }
    }
}

/// Bresenham entero clásico (los 8 octantes): visita cada punto de la
/// retícula entre `(ax, ay)` y `(bx, by)`, extremos incluidos.
fn bresenham(ax: i64, ay: i64, bx: i64, by: i64, mut visit: impl FnMut(i64, i64)) {
    let (mut x, mut y) = (ax, ay);
    let dx = (bx - ax).abs();
    let dy = -(by - ay).abs();
    let sx = if ax < bx { 1 } else { -1 };
    let sy = if ay < by { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        visit(x, y);
        if x == bx && y == by {
            break;
        }
        let doubled = 2 * err;
        if doubled >= dy {
            err += dy;
            x += sx;
        }
        if doubled <= dx {
            err += dx;
            y += sy;
        }
    }
}

// ── Portapapeles PNG (arboard en worker, como el lienzo) ─────────────────

/// Copia un RGBA8 fila-mayor al portapapeles OS (`arboard::set_image`).
/// Headless honesto: sin servidor gráfico devuelve `Err` con la causa.
/// Con `desktop-dialogs` usa arboard; sin la feature, el mismo `Err`
/// honesto que el PNG del lienzo (misma UI, degradación explícita).
///
/// Hoy sin llamador prod (el botón PNG solo guarda a archivo; el
/// portapapeles excede el alcance): el `allow` cubre ese resto, no el módulo.
#[cfg(feature = "desktop-dialogs")]
#[allow(dead_code)]
pub(crate) fn copy_whiteboard_png_to_clipboard(
    width: u32,
    height: u32,
    rgba: Vec<u8>,
) -> Result<String, String> {
    if width == 0 || height == 0 {
        return Err("PNG no se copió al portapapeles; dimensiones nulas".to_string());
    }
    if rgba.len() != width as usize * height as usize * 4 {
        return Err("PNG no se copió al portapapeles; el búfer RGBA está incompleto".to_string());
    }
    let mut clipboard = arboard::Clipboard::new().map_err(|error| {
        format!("PNG no se copió al portapapeles; portapapeles OS no disponible: {error}")
    })?;
    let image = arboard::ImageData {
        width: width as usize,
        height: height as usize,
        bytes: std::borrow::Cow::Owned(rgba),
    };
    clipboard.set_image(image).map_err(|error| {
        format!("PNG no se copió al portapapeles; el sistema lo rechazó: {error}")
    })?;
    Ok(format!(
        "PNG de pizarra copiado al portapapeles ({width}x{height})"
    ))
}

/// Sin `desktop-dialogs` (p. ej. Android): `Err` honesto, sin pánico.
#[cfg(not(feature = "desktop-dialogs"))]
#[allow(dead_code)]
pub(crate) fn copy_whiteboard_png_to_clipboard(
    _width: u32,
    _height: u32,
    _rgba: Vec<u8>,
) -> Result<String, String> {
    Err(
        "PNG no se copió al portapapeles; portapapeles OS no disponible en esta compilación"
            .to_string(),
    )
}

/// Spawn copia PNG de pizarra en background — arboard bloquea (Wayland) y
/// jamás corre en Ui::. Canal apto para `PendingClipboardJob`; el poll
/// publica summary/error con toast, igual que el PNG del lienzo.
///
/// Hoy sin llamador prod (ver `copy_whiteboard_png_to_clipboard`).
#[cfg(feature = "desktop-dialogs")]
#[allow(dead_code)]
pub(crate) fn spawn_whiteboard_png_clipboard(
    width: u32,
    height: u32,
    rgba: Vec<u8>,
    ctx: &egui::Context,
) -> std::sync::mpsc::Receiver<Result<String, String>> {
    let ctx = egui::Context::clone(ctx);
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    let _ = std::thread::Builder::new()
        .name("whiteboard-png-clipboard".into())
        .spawn(move || {
            let _ = tx.send(copy_whiteboard_png_to_clipboard(width, height, rgba));
            ctx.request_repaint();
        });
    rx
}

/// Sin `desktop-dialogs`: canal con el `Err` honesto ya enviado (el
/// llamador no cambia; el poll muestra el toast de error igual).
#[cfg(not(feature = "desktop-dialogs"))]
#[allow(dead_code)]
pub(crate) fn spawn_whiteboard_png_clipboard(
    width: u32,
    height: u32,
    rgba: Vec<u8>,
    _ctx: &egui::Context,
) -> std::sync::mpsc::Receiver<Result<String, String>> {
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    let _ = tx.send(copy_whiteboard_png_to_clipboard(width, height, rgba));
    rx
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pipeline completo en tests (el llamado anterior
    /// `render_png_bytes`): raster + encode, mismas cotas y `Err`.
    fn render_png_bytes(doc: &WhiteboardDoc, escala: f64) -> Result<Vec<u8>, String> {
        let (width, height, rgba) = render_png_rgba(doc, escala, &PngOptions::default())?;
        encode_png_rgba(width, height, &rgba)
    }

    /// Lee píxel RGBA del PNG decodificado con `image` (codec de test).
    fn decode_test_png(bytes: &[u8]) -> (u32, u32, Vec<u8>) {
        let decoded = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
            .expect("el render devuelve PNG válido");
        let rgba = decoded.to_rgba8();
        let (width, height) = (rgba.width(), rgba.height());
        (width, height, rgba.into_raw())
    }

    fn pixel_at(raw: &[u8], width: u32, x: u32, y: u32) -> (u8, u8, u8, u8) {
        let offset = (y as usize * width as usize + x as usize) * 4;
        (
            raw[offset],
            raw[offset + 1],
            raw[offset + 2],
            raw[offset + 3],
        )
    }

    /// Firma PNG de 8 bytes + `IHDR` con las dimensiones esperadas (parseo
    /// manual: ancho en bytes 16..20, alto en 20..24, big-endian).
    #[test]
    fn png_bytes_have_signature_and_dimensions() {
        let mut doc = WhiteboardDoc::new();
        doc.add(WhiteboardElement::Rectangle {
            min: (0.0, 0.0),
            max: (10.0, 10.0),
            fill: None,
        });
        // Encuadre: (0..10)+margen 20 → 50×50 mundo; a escala 1 → 50×50px.
        let bytes = render_png_bytes(&doc, 1.0).expect("rectángulo chico renderiza");
        assert_eq!(&bytes[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
        assert_eq!(&bytes[12..16], b"IHDR");
        let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
        assert_eq!((width, height), (50, 50));
    }

    /// Trazo fino de 1px cae en el píxel esperado con el color guardado.
    #[test]
    fn thin_stroke_hits_expected_pixel() {
        let mut doc = WhiteboardDoc::new();
        doc.add(WhiteboardElement::Stroke {
            points: vec![(10.0, 10.0), (20.0, 10.0)],
            // Rojo medio: ni claro (>200) ni oscuro (<60), la adaptación
            // de contraste lo respeta literal (un (1,2,3) iría a tinta
            // clara como en el canvas).
            color: (200, 30, 30),
            width: 1.0,
        });
        // Encuadre x: 10..20+margen → vx=-10, vw=50; y: 10..10+margen → 40.
        // Mundo (10,10) → píxel (20,20) a escala 1.
        let bytes = render_png_bytes(&doc, 1.0).expect("trazo renderiza");
        let (width, height, raw) = decode_test_png(&bytes);
        assert_eq!((width, height), (50, 40));
        assert_eq!(pixel_at(&raw, width, 20, 20), (200, 30, 30, 255));
        assert_eq!(pixel_at(&raw, width, 30, 20), (200, 30, 30, 255));
        // Fondo DARK intacto lejos del trazo (y=20 es múltiplo de 40 en
        // mundo… no: mundo y=-10+20=10; el grid vive en múltiplos de 40
        // (0, 40): (20,20)→mundo (10,10) no es grid; (0,0)→mundo (-10,-10)
        // tampoco. Fondo puro esperado.
        assert_eq!(
            pixel_at(&raw, width, 0, 0),
            (
                WHITEBOARD_PNG_DEFAULT_BG.0,
                WHITEBOARD_PNG_DEFAULT_BG.1,
                WHITEBOARD_PNG_DEFAULT_BG.2,
                255
            )
        );
    }

    /// Resaltado: `src-over` con `HIGHLIGHTER_ALPHA` (0.35) sobre el fondo.
    /// Rojo (255,0,0) sobre (19,21,24): r=255·.35+19·.65=101.6→102,
    /// g=21·.65=13.65→14, b=24·.65=15.6→16.
    #[test]
    fn highlighter_blends_over_background() {
        let mut doc = WhiteboardDoc::new();
        doc.add(WhiteboardElement::Highlighter {
            points: vec![(10.0, 10.0), (30.0, 10.0)],
            color: (255, 0, 0),
            width: 1.0,
        });
        let bytes = render_png_bytes(&doc, 1.0).expect("resaltado renderiza");
        let (width, _, raw) = decode_test_png(&bytes);
        // Mundo (10,10)→(20,20) igual que el test del trazo.
        assert_eq!(pixel_at(&raw, width, 20, 20), (102, 14, 16, 255));
    }

    /// Lado excedido → `Err` con causa (rectángulo de 3000u a escala 1 →
    /// 3040px > 2048), jamás alloc gigante.
    #[test]
    fn oversize_side_errors() {
        let mut doc = WhiteboardDoc::new();
        doc.add(WhiteboardElement::Rectangle {
            min: (0.0, 0.0),
            max: (3000.0, 100.0),
            fill: None,
        });
        let error = render_png_bytes(&doc, 1.0).expect_err("lado excedido falla");
        assert!(
            error.contains("lado máximo"),
            "causa honesta sobre el lado, fue: {error}"
        );
    }

    /// Memoria excedida → `Err` con causa aunque el lado entre: 2000×1600px
    /// (lados ≤ 2048) necesitan 12.8 MiB > 12 MiB.
    #[test]
    fn memory_cap_errors_before_alloc() {
        let mut doc = WhiteboardDoc::new();
        doc.add(WhiteboardElement::Rectangle {
            min: (0.0, 0.0),
            max: (1960.0, 1560.0),
            fill: None,
        });
        // Encuadre: 1960+40=2000 × 1560+40=1600 a escala 1.
        let error = render_png_bytes(&doc, 1.0).expect_err("memoria excedida falla");
        assert!(
            error.contains("MiB"),
            "causa honesta sobre memoria, fue: {error}"
        );
    }

    /// Escala inválida (0, negativa, NaN, infinita) → `Err`, sin pánico.
    #[test]
    fn invalid_scale_errors() {
        let doc = WhiteboardDoc::new();
        for escala in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(
                render_png_bytes(&doc, escala).is_err(),
                "escala {escala} debe fallar"
            );
        }
    }

    /// Texto: se omite del raster (píxel de fondo donde iría la letra),
    /// `png_skips_text` lo avisa y el SVG sigue trayéndolo (vía completa).
    #[test]
    fn text_is_skipped_but_flagged_with_svg_fallback() {
        assert!(!png_skips_text(&WhiteboardDoc::new()));
        let mut doc = WhiteboardDoc::new();
        doc.add(WhiteboardElement::Text {
            at: (13.0, 17.0),
            text: "hola".to_string(),
            size: 12.0,
        });
        assert!(png_skips_text(&doc));
        let bytes = render_png_bytes(&doc, 1.0).expect("texto omitido no falla");
        let (width, _, raw) = decode_test_png(&bytes);
        // Mundo (13,17) → píxel (33,37): ni trazo ni grid (13 y 17 no son
        // múltiplos de 40) → fondo puro: la letra no se pintó.
        assert_eq!(
            pixel_at(&raw, width, 33, 37),
            (
                WHITEBOARD_PNG_DEFAULT_BG.0,
                WHITEBOARD_PNG_DEFAULT_BG.1,
                WHITEBOARD_PNG_DEFAULT_BG.2,
                255
            )
        );
        let svg = doc.to_svg(800, 600).expect("SVG serializa");
        assert!(svg.contains("hola"), "el SVG es la vía completa del texto");
    }

    /// Imagen 2×1 (rojo, azul) bliteada 1:1 y 2× con vecino más cercano.
    #[test]
    fn image_blits_with_nearest_neighbor() {
        let source = image::RgbaImage::from_fn(2, 1, |x, _| {
            if x == 0 {
                image::Rgba([255, 0, 0, 255])
            } else {
                image::Rgba([0, 0, 255, 255])
            }
        });
        let mut buf = Vec::new();
        image::DynamicImage::ImageRgba8(source)
            .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .expect("PNG de test codifica");
        let mut doc = WhiteboardDoc::new();
        doc.add(WhiteboardElement::Image {
            bytes_png_jpeg: buf,
            // Destino 4×2 mundo @1: cada píxel fuente ocupa 2×2 destino.
            rect: ((0.0, 0.0), (4.0, 2.0)),
        });
        let bytes = render_png_bytes(&doc, 1.0).expect("imagen blitea");
        let (width, _, raw) = decode_test_png(&bytes);
        // Encuadre con margen 20: mundo (0,0) → píxel (20,20).
        assert_eq!(pixel_at(&raw, width, 20, 20), (255, 0, 0, 255));
        assert_eq!(pixel_at(&raw, width, 21, 21), (255, 0, 0, 255));
        assert_eq!(pixel_at(&raw, width, 22, 20), (0, 0, 255, 255));
        assert_eq!(pixel_at(&raw, width, 23, 21), (0, 0, 255, 255));
    }

    /// Stretch no-uniforme: la fila vertical usa el alto fuente (regresión:
    /// `sy` usaba `src_w` y aplastaba la imagen a la primera fila).
    #[test]
    fn image_blit_non_uniform_stretch_uses_source_height() {
        let source = image::RgbaImage::from_fn(1, 2, |_, y| {
            if y == 0 {
                image::Rgba([255, 0, 0, 255])
            } else {
                image::Rgba([0, 0, 255, 255])
            }
        });
        let mut buf = Vec::new();
        image::DynamicImage::ImageRgba8(source)
            .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .expect("PNG de test codifica");
        let mut doc = WhiteboardDoc::new();
        doc.add(WhiteboardElement::Image {
            bytes_png_jpeg: buf,
            // Destino 2×4 mundo @1: cada píxel fuente ocupa 2×2 destino.
            rect: ((0.0, 0.0), (2.0, 4.0)),
        });
        let bytes = render_png_bytes(&doc, 1.0).expect("imagen blitea");
        let (width, _, raw) = decode_test_png(&bytes);
        assert_eq!(pixel_at(&raw, width, 20, 20), (255, 0, 0, 255));
        assert_eq!(pixel_at(&raw, width, 21, 21), (255, 0, 0, 255));
        assert_eq!(pixel_at(&raw, width, 20, 22), (0, 0, 255, 255));
        assert_eq!(pixel_at(&raw, width, 21, 23), (0, 0, 255, 255));
    }

    /// Flecha y elipse no fallan y pintan tinta de forma sobre el fondo.
    #[test]
    fn arrow_and_ellipse_paint_shape_ink() {
        let mut doc = WhiteboardDoc::new();
        doc.add(WhiteboardElement::Arrow {
            from: (0.0, 0.0),
            to: (60.0, 0.0),
        });
        doc.add(WhiteboardElement::Ellipse {
            center: (0.0, 60.0),
            rx: 20.0,
            ry: 10.0,
        });
        let bytes = render_png_bytes(&doc, 1.0).expect("flecha+elipse renderizan");
        let (width, height, raw) = decode_test_png(&bytes);
        assert!(width > 0 && height > 0);
        // El fuste de la flecha (mundo y=0 → fila 20) lleva tinta clara
        // DARK en algún píxel del tramo medio (mundo x=30 → col 50).
        let ink = WHITEBOARD_PNG_DARK_INK;
        assert_eq!(
            pixel_at(&raw, width, 50, 20),
            (ink.0, ink.1, ink.2, 255),
            "el fuste pinta tinta de forma"
        );
    }

    /// Tinta casi negra sobre fondo oscuro se adapta a clara (espejo del
    /// canvas: `effective_pen_color` manda oscuro-sobre-oscuro a
    /// `text_primary`; el PNG no pinta trazos invisibles).
    #[test]
    fn dark_pen_adapts_to_light_ink_on_dark_bg() {
        let mut doc = WhiteboardDoc::new();
        doc.add(WhiteboardElement::Stroke {
            points: vec![(10.0, 10.0), (20.0, 10.0)],
            color: (1, 2, 3),
            width: 1.0,
        });
        let bytes = render_png_bytes(&doc, 1.0).expect("trazo renderiza");
        let (width, _, raw) = decode_test_png(&bytes);
        let ink = WHITEBOARD_PNG_DARK_INK;
        assert_eq!(pixel_at(&raw, width, 20, 20), (ink.0, ink.1, ink.2, 255));
    }

    /// Hoja vacía: 800×600 mundo a escala 2 → PNG 1600×1200 válido.
    #[test]
    fn empty_doc_renders_default_frame() {
        let bytes = render_png_bytes(&WhiteboardDoc::new(), 2.0).expect("hoja vacía renderiza");
        let (width, height, _) = decode_test_png(&bytes);
        assert_eq!((width, height), (1600, 1200));
    }
}
