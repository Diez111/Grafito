//! Design tokens — Scandinavian (Grafito).
//!
//! Estética: calm, restraint, luz natural. Tipografía Inter, paleta
//! cálida neutra (canvas #FAFAF9), acento sage #6B7A6F. Sin sombras
//! duras ni translucidez. Todo opaco (`from_rgb`).
//!
//! F5 quiet 2026-08-21: ink secondary 64% y tertiary 44%, border 10%,
//! hover 5% — definidos en `theme.rs` vía gamma_multiply (ver `Theme`);
//! tokens aquí fijan spacing/radios, no colores.
//!
//! Estos tokens son la **única fuente de verdad** para tamaños y
//! espacios. Ningún `.size(N)` o `vec2(N,M)` hardcodeado fuera de aquí.
//! — Sistema base 4 para spacing, ratio tipográfico 1.25.

// ═══════════════════════════════════════════════════════════
// Familias tipográficas — Scandinavian
// ═══════════════════════════════════════════════════════════
//
// La app registra Inter en Proportional **y** en Monospace
// (`grafito-app/src/app.rs`, `FontDefinitions`), así que no hay una segunda
// familia que nombrar acá: `TextStyle::Monospace` ya resuelve a Inter. Antes
// este bloque declaraba `FONT_SF_DISPLAY`, `FONT_SF_MONO` ("SF Mono", que la
// app nunca cargaba) y `FONT_FALLBACK_SANS` sin un solo uso: tres constantes
// que describían una tipografía que no existe.

// ═══════════════════════════════════════════════════════════
// Type scale — Scandinavian (Inter 12 / 15 / 19)
// ratio 1.25 (Major Third) — calm progression
// ═══════════════════════════════════════════════════════════

/// Texto doble-extra-pequeño: micro hints, eye-tracking status.
/// Piso mínimo 11.0 (legibilidad: 9.0 viola el mínimo accesible).
// Piso mínimo ya aplicado (11.0).
pub const TYPE_2XS: f32 = 11.0;
/// Texto extra-pequeño: notas, metadatos, hints.
pub const TYPE_XS: f32 = 11.0;
/// Texto pequeño: labels secundarios, captions — Inter 12.
pub const TYPE_SM: f32 = 12.0;
/// Texto base: cuerpo, inputs — Inter 15 (Scandinavian body 15).
pub const TYPE_BASE: f32 = 15.0;
/// Texto mediano: labels destacados, sub-headers — Inter 16.
pub const TYPE_MD: f32 = 16.0;
/// Texto grande: headers de panel — Inter 19.
pub const TYPE_LG: f32 = 19.0;
/// Texto extra-grande: titles — Scandinavian 24.
pub const TYPE_XL: f32 = 24.0;
/// Texto doble-extra-grande: splash, branding.
pub const TYPE_XXL: f32 = 28.0;
/// Texto de cuerpo del drawer móvil: 17 px legible en táctil. Excepción
/// documentada a la escala 12/15/16/19 (ratio 17/15≈1.13): en pantallas
/// angostas el cuerpo 15 px queda chico y el header 19 px muy grande;
/// 17 px es el punto legible sin romper el aire de filas de 52 px.
pub const MOBILE_MENU_TEXT: f32 = 17.0;

// ═══════════════════════════════════════════════════════════
// Spacing scale — Scandinavian (16 / 24 / 40)
// base 4 — todo múltiplo de 4
// ═══════════════════════════════════════════════════════════

/// Espacio extra-pequeño: entre items muy cercanos.
pub const SPACE_XS: f32 = 4.0;
/// Espacio pequeño: entre items de un grupo.
pub const SPACE_SM: f32 = 8.0;
/// Espacio mediano: padding interno de chips.
pub const SPACE_MD: f32 = 12.0;
/// Espacio grande: padding interno de paneles — Scandinavian 16.
pub const SPACE_LG: f32 = 16.0;
/// Espacio extra-grande: separación entre secciones — Scandinavian 24.
pub const SPACE_XL: f32 = 24.0;
/// Espacio doble-extra-grande: separación entre paneles — Scandinavian 40.
pub const SPACE_XXL: f32 = 40.0;

/// Ritmo Scandinavian horizontal (item_spacing.x).
pub const SPACING_MINIMAL_X: f32 = 16.0;
/// Ritmo Scandinavian vertical (item_spacing.y).
pub const SPACING_MINIMAL_Y: f32 = 16.0;

/// Button padding Scandinavian — 16 x 8.
pub const SPACING_BUTTON_X: f32 = 16.0;
pub const SPACING_BUTTON_Y: f32 = 8.0;

// ═══════════════════════════════════════════════════════════
// Radii — Scandinavian (4 / 8 / 12 / 16 / 20 / 24)
// ═══════════════════════════════════════════════════════════

/// Radio extra-pequeño: teclas del teclado en pantalla — 4 px.
pub const RADIUS_XS: f32 = 4.0;
/// Radio pequeño: campos, chips, botones compactos — 8 px.
pub const RADIUS_SM: f32 = 8.0;
/// Radio mediano: tarjetas y contenedores — 12 px.
pub const RADIUS_MD: f32 = 12.0;
/// Radio grande: superficies amplias — 16 px.
pub const RADIUS_LG: f32 = 16.0;
/// Radio extra-grande: ventanas y diálogos — 20 px.
pub const RADIUS_XL: f32 = 20.0;
/// Radio cápsula: sólo toggles y controles segmentados.
///
/// Los tres tokens previos eran todos 16.0: ventana, menú y cada widget de
/// egui dibujaban el mismo radio, así que la escala no distinguía nada y el
/// test la pineaba como si fuera una decisión. El radio grande queda en
/// ventanas/popups — superficies grandes — y los widgets usan `RADIUS_SM`.
pub const RADIUS_2XL: f32 = 24.0;
/// Píldora: reservado a toggles y controles segmentados. Nunca para campos,
/// paneles ni botones compactos (ver `BOTTOM_INPUT_RADIUS`).
pub const RADIUS_PILL: f32 = 999.0;

// ═══════════════════════════════════════════════════════════
// Tamaños de íconos
// ═══════════════════════════════════════════════════════════

pub const ICON_SM: f32 = 16.0;
pub const ICON_MD: f32 = 20.0;
pub const ICON_LG: f32 = 24.0;
pub const ICON_XL: f32 = 32.0;

// ═══════════════════════════════════════════════════════════
// Sombras — Scandinavian (0,2,8 alpha 8) calm, sin drama
// ═══════════════════════════════════════════════════════════

/// Offset Y sombra ventana — 2 px.
pub const SHADOW_WINDOW_OFFSET_Y: f32 = 2.0;
/// Blur sombra ventana — 8 px.
pub const SHADOW_WINDOW_BLUR: f32 = 8.0;
/// Offset Y sombra popup — 2 px.
pub const SHADOW_POPUP_OFFSET_Y: f32 = 2.0;
/// Blur sombra popup — 8 px.
pub const SHADOW_POPUP_BLUR: f32 = 8.0;
/// Alpha sombra — 8 (~3 %) sutil, Scandinavian restraint.
pub const SHADOW_ALPHA: u8 = 8;

// ═══════════════════════════════════════════════════════════
// Strokes — Scandinavian (hairline 1 px, énfasis 1.5 px)
// ═══════════════════════════════════════════════════════════

/// Hairline de bordes y divisores — 1 px, siempre con
/// `Theme::hairline_stroke()` (separator al 10 %). Sin magia suelta.
pub const STROKE_HAIRLINE: f32 = 1.0;
/// Trazo de énfasis (selección, foco, highlight) — 1.5 px.
/// Reemplaza los `Stroke::new(1.5, …)` sin nombre.
pub const STROKE_EMPHASIS: f32 = 1.5;

// ═══════════════════════════════════════════════════════════
// Alphas — Scandinavian quiet overlays
// ═══════════════════════════════════════════════════════════

/// Alpha sombra — canonical 8 (~3 %) — alias de SHADOW_ALPHA.
/// Usar `Color32::from_black_alpha(ALPHA_SHADOW)` para sombras sutiles.
pub const ALPHA_SHADOW: u8 = 8;
/// Alpha overlay — 25 (~10 %) para hover sutil / scrim ligero.
pub const ALPHA_OVERLAY: u8 = 25;

// ═══════════════════════════════════════════════════════════
// Layout — breakpoints (Scandinavian responsive)
// ═══════════════════════════════════════════════════════════

/// Breakpoint compact — 1360 px.
/// Reemplaza 3× hardcodeados 1360:
/// - `COMPACT_TOOLBAR_MAX_WIDTH` (toolbar.rs)
/// - `COMPACT_TOP_CHROME_MAX_WIDTH` (ui.rs)
/// - `ShellLayout::CANVAS_FOCUS_MAX_WIDTH` (lib.rs)
///
/// Por debajo → overflow compact (solo Move + grupo activo + "Más");
/// por encima → toolbar completa y drawers simultáneos.
pub const BREAKPOINT_COMPACT: f32 = 1360.0;

// ─────────────────────────────────────────────────────────────────────────
// Layout — tier tablet honesto (768..=1024) + landscape corto (h<500)
//
// No hay layout tablet propio: la tablet cae en Compact o en táctil según
// el componente (cliff 740→1360). Este tier solo suaviza los dos puntos
// donde el salto se nota: default del asistente y targets táctiles.
// Desktop ≥1360, Nivel 0 <480 y `ShellLayout` intactos por construcción.
// ─────────────────────────────────────────────────────────────────────────

/// Ancho mínimo del tier tablet — 768 px (retrato iPad clásica).
/// Por debajo manda el régimen phone (<740 táctil / <480 Nivel 0).
/// Múltiplo de base 4.
pub const TABLET_MIN: f32 = 768.0;
/// Ancho máximo del tier tablet — 1024 px (apaisado iPad clásica).
/// Por encima manda Compact/desktop. Múltiplo de base 4.
pub const TABLET_MAX: f32 = 1024.0;
/// Alto bajo el cual un viewport se considera landscape corto: el
/// bottom-sheet a altura completa tapa el canvas (a 430 px Mili ocupaba
/// ~60 %). Solo el onboarding contemplaba altura corta (h<560); el
/// asistente no. Múltiplo de base 4.
pub const LANDSCAPE_SHORT_MAX_H: f32 = 500.0;
/// Fracción del viewport que el sheet del asistente puede ocupar en
/// landscape corto — deja ~45 % al canvas en vez de taparlo.
pub const ASSISTANT_SHEET_LANDSCAPE_FRACTION: f32 = 0.55;
/// Fracción del viewport para el default del asistente en tablet: a 900
/// da 315 (dentro del 300..520 pineado) en vez del fijo 400, y el canvas
/// conserva aire con drawer. El rango y el presupuesto de canvas no cambian.
pub const ASSISTANT_TABLET_DEFAULT_FRACTION: f32 = 0.35;

/// Tier honesto del viewport según su ancho: phone / tablet / desktop.
/// Los bordes son inclusivos en tablet (768 y 1024 ya son tablet).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewportTier {
    Phone,
    Tablet,
    Desktop,
}

/// Clasifica el viewport por ancho (puro y testeable headless).
/// No toca ningún breakpoint existente: <768 sigue phone (740 táctil y
/// 480 Nivel 0 mandan igual) y >1024 sigue Compact/desktop.
#[inline]
pub fn viewport_tier(viewport_width: f32) -> ViewportTier {
    if viewport_width < TABLET_MIN {
        ViewportTier::Phone
    } else if viewport_width <= TABLET_MAX {
        ViewportTier::Tablet
    } else {
        ViewportTier::Desktop
    }
}

/// ¿El ancho cae en el tier tablet (768..=1024)? Puro y pineado.
#[inline]
pub fn is_tablet_viewport(viewport_width: f32) -> bool {
    viewport_tier(viewport_width) == ViewportTier::Tablet
}

/// ¿El alto es landscape corto (h<500)? Puro y pineado: solo altura,
/// sin adivinar orientación (el llamador ya sabe si hay sheet/lateral).
#[inline]
pub fn is_short_landscape_height(viewport_height: f32) -> bool {
    viewport_height < LANDSCAPE_SHORT_MAX_H
}

// ═══════════════════════════════════════════════════════════
// Layout — paneles laterales y drawers
// ═══════════════════════════════════════════════════════════

/// Ancho por defecto panel izquierdo (Álgebra/CAS/Vista) — 260 px.
pub const PANEL_LEFT_DEFAULT: f32 = 300.0;
/// Ancho mínimo panel izquierdo — 240 px.
pub const PANEL_LEFT_MIN: f32 = 240.0;
/// Fracción máxima del viewport para panel izquierdo — 0.45 (45 %).
/// Uso: `max_width = (available_width * PANEL_LEFT_MAX_FRACTION).max(200.0)`
pub const PANEL_LEFT_MAX_FRACTION: f32 = 0.45;

/// Drawer derecho (Inspector/Utilidad Geometry 3D) — ancho por defecto 344 px.
pub const DRAWER_RIGHT_DEFAULT: f32 = 344.0;
/// Drawer derecho — ancho mínimo 292 px.
pub const DRAWER_RIGHT_MIN: f32 = 292.0;
/// Drawer derecho — ancho máximo 440 px.
pub const DRAWER_RIGHT_MAX: f32 = 440.0;

/// Ancho rail lateral izquierdo (icon bar 68 px) — Scandinavian single rail.
/// 68 = múltiplo de base 4; deja 4 px de respiro por lado para que el borde
/// de la burbuja activa nunca quede clipado por el panel.
pub const RAIL_WIDTH: f32 = 68.0;
/// Alto de cada tab del rail — 60 px (icono 20 + etiqueta 11 + aire).
pub const RAIL_ITEM_HEIGHT: f32 = 60.0;
/// Respiro horizontal del item dentro del rail — 4 px por lado.
pub const RAIL_ITEM_PAD_X: f32 = SPACE_XS;

// ═══════════════════════════════════════════════════════════
// Cards — Scandinavian quiet surfaces
// ═══════════════════════════════════════════════════════════

/// Espacio entre cards — 12.0 = SPACE_MD (base 4).
pub const CARD_SPACING: f32 = SPACE_MD;
/// Radio card de objeto — 8.0 = RADIUS_SM.
pub const OBJECT_CARD_RADIUS: f32 = RADIUS_SM;
/// Radio card inspector/sección — 12.0 = RADIUS_MD.
pub const INSPECTOR_CARD_RADIUS: f32 = RADIUS_MD;

// ═══════════════════════════════════════════════════════════
// Splash
// ═══════════════════════════════════════════════════════════

/// Tamaño logo splash — 128 px cuadrado.
pub const SPLASH_LOGO_SIZE: f32 = 128.0;

// ═══════════════════════════════════════════════════════════
// Ola 2 — literales de draws centralizados (sin magia en Piel)
// ═══════════════════════════════════════════════════════════

/// Ancho del breakpoint angosto del preview de adjuntos del asistente.
/// `assistant.rs`: preview arriba si `available_width < 720`.
pub const ASSISTANT_PREVIEW_NARROW_BREAKPOINT: f32 = 720.0;

/// Botón del selector de idioma del toolbar — 36×24 mínimo táctil.
pub const TOOLBAR_LOCALE_MIN_W: f32 = 36.0;
/// Alto mínimo del botón de idioma — 24 px (piso táctil reducido, texto 11).
pub const TOOLBAR_LOCALE_MIN_H: f32 = 24.0;
/// Padding horizontal interno del frame del toolbar — 4 px = SPACE_XS.
pub const TOOLBAR_INNER_PAD_X: f32 = SPACE_XS;

/// Alto de tecla del teclado en pantalla — 32 px (piso táctil).
pub const KEYBOARD_KEY_H: f32 = 32.0;
/// Ancho máximo de chip del teclado — 40 px (clamp superior).
pub const KEYBOARD_CHIP_W_MAX: f32 = 40.0;

/// Ancho de la ventana de onboarding — 420 px Scandinavian.
pub const ONBOARDING_WINDOW_WIDTH: f32 = 420.0;
/// Botón de onboarding — 120×32.
pub const ONBOARDING_BUTTON_W: f32 = 120.0;
pub const ONBOARDING_BUTTON_H: f32 = KEYBOARD_KEY_H;
/// Separación entre botones de onboarding — 8 px = SPACE_SM.
pub const ONBOARDING_BUTTON_GAP: f32 = SPACE_SM;
/// Padding horizontal interno de la ventana de onboarding — 20 px.
pub const ONBOARDING_INNER_PAD_X: f32 = 20.0;

// ═══════════════════════════════════════════════════════════
// Bienvenida fullscreen —composition, no modal centrado
//
// El espacio muerto venía de anclar el bloque de copy en el centro
// matemático del viewport y dejar el resto vacío. La composición nueva
// es una columna de lectura con medida acotada, anclada al tercio
// superior, sobre una banda de progreso propia: el ojo tiene un
// punto de partida y el resto del alto es deliberado, no residuo.
// ═══════════════════════════════════════════════════════════

/// Medida de la columna de lectura de la bienvenida — 640 px.
///
/// Porte de texto, no ancho de ventana: el headline nunca pasa de ~32
/// caracteres por línea y el subtítulo se mantiene en 55–70 ch.
pub const WELCOME_COLUMN_WIDTH: f32 = 640.0;

/// Margen lateral del contenido de la bienvenida — 24 px = SPACE_XL.
/// Une el margen físico con la escala de spacing (regla 1-4-9).
pub const WELCOME_MARGIN_X: f32 = SPACE_XL;

/// Descanso del bloque de copy respecto del borde superior.
///
/// Proporción del alto del viewport. El bloque completo (marca, titular,
/// subtítulo y navegación) entra en el tercio superior: 0.14 deja el bloque
/// completo visible sin que el ojo tenga que recorrer media pantalla vacía
/// antes de llegar a la acción.
pub const WELCOME_BLOCK_TOP_RATIO: f32 = 0.14;

/// Medida de la columna secundaria (subtítulo) — 520 px.
pub const WELCOME_SUB_COLUMN_WIDTH: f32 = 520.0;

/// Alto de la banda de progreso de la bienvenida — 4 px.
///
/// Es un indicador, no un control: 4 px es legible a distancia y no
/// compite con la marca. Reemplaza los dots de 6 px alineados a mano.
pub const WELCOME_PROGRESS_BAR_H: f32 = 4.0;

/// Ancho máximo de la banda de progreso — 240 px.
pub const WELCOME_PROGRESS_MAX_W: f32 = 240.0;

// ═══════════════════════════════════════════════════════════
// Configuración — composition de dos carriles
//
// El screenshot mostraba ~1200 px de ventana con la navegación
// apretada a la izquierda y ~470 px de contenido, dejando el resto
// como vacío. La composición nueva reparte: carril de navegación con
// ancho propio y columna de lectura que crece con el viewport.
// ═══════════════════════════════════════════════════════════

/// Ancho del carril de navegación de Ajustes — 240 px.
///
/// Antes 220 px fijos o 34 % del ancho con piso 180: en ventanas
/// angostas el carril se comía el contenido. 240 px con piso duro
/// deja la columna de lectura respirar incluso en 500 px.
pub const SETTINGS_NAV_WIDTH: f32 = 240.0;

/// Ancho mínimo de la columna de lectura de Ajustes — 320 px.
///
/// Por debajo de esto la fila `label + control` no cabe y hay que
/// apilar (ver `SETTINGS_CONTENT_NARROW`).
pub const SETTINGS_CONTENT_MIN_WIDTH: f32 = 320.0;

/// Breakpoint interno: debajo de este ancho de contenido, las filas
/// `label + control` se apilan en vez de competir por el ancho.
///
/// 480 px = 320 (mínimo) + 160 (control más ancho). El mismo valor
/// que usa el panel de avatar, para que ambas columnas cambien a la vez.
pub const SETTINGS_CONTENT_NARROW: f32 = 480.0;

/// Ancho del control (combo, switch) dentro de una fila de Ajustes
/// — 200 px. Antes 220 px, que empujaba el label a 120 px en ventana
/// angosta y lo partía en dos líneas.
pub const SETTINGS_CONTROL_WIDTH: f32 = 200.0;

/// Alto del switch de Ajustes — 24 px = HIT_TARGET_MIN (WCAG 2.5.8).
/// El switch dibujado a mano medía 22 px de alto: un punto bajo el
/// mínimo accesible, con el texto pegado encima.
pub const SETTINGS_SWITCH_W: f32 = 44.0;
pub const SETTINGS_SWITCH_H: f32 = HIT_TARGET_MIN;

/// Radio del knob del switch — 8 px = (44 − 2×8) / 2 + 4 de aire.
/// Con H=24 y knob=16 el juego vertical queda en 4 px por lado.
pub const SETTINGS_SWITCH_KNOB_R: f32 = 8.0;

/// Alto de una fila de Ajustes (label + control) — 44 px.
/// Cubre el alvo táctil de aula sin separar label y control.
pub const SETTINGS_ROW_MIN_H: f32 = HIT_TARGET_AULA;

/// Alto de una fila de switch en los drawers — 56 px.
/// Misma fila M3 que `drill_action` en `grafito-app/src/ui.rs`:
/// el switch 44×24 comparte ritmo con las filas de acción de 56 px en vez
/// de quedar enano (24 px) entre ellas. El drawer es táctil: 56 > 44.
pub const DRILL_SWITCH_ROW_H: f32 = 56.0;

// ═══════════════════════════════════════════════════════════
// Ajustes — simetría y ritmo
//
// El buscador flotaba arriba de la composición con un ancho tope de 260 px
// mientras el carril mide 240: no se alineaba con ninguno de los dos
// carriles y dejaba ~876 px muertos a su derecha. Y el `separator()` de
// ancho completo se dibujaba con el `min_rect` del `horizontal` (292 px),
// no con el de la ventana: una línea de 292 px pegada a la izquierda.
// ═══════════════════════════════════════════════════════════

/// Padding horizontal interno de los carriles de Ajustes — 12 px.
///
/// Navigation y lectura usan el mismo inset: antes el nav pintaba su ícono
/// a +8 y su texto a +30, así que el título de sección quedaba 8 px a la
/// izquierda de sus propios ítems.
pub const SETTINGS_CARD_PAD_X: f32 = SPACE_MD;

/// Ícono del ítem de navegación: ancho — 16 px.
pub const SETTINGS_NAV_ICON_W: f32 = ICON_SM;

/// Separación entre el ícono y el texto del ítem de navegación — 8 px.
/// Ícono (16) + separador (8) = 24 px de inset, múltiplo de 4.
pub const SETTINGS_NAV_TEXT_GAP: f32 = SPACE_SM;

/// Alto del ítem de navegación — 32 px.
///
/// Distinto del alto de fila de contenido (44) a propósito: el carril es un
/// índice, la lectura es un formulario. Se documenta para que la diferencia
/// sea una decisión y no un descuido.
pub const SETTINGS_NAV_ITEM_H: f32 = 32.0;

/// Alto del campo de búsqueda — 32 px = BOTTOM_INPUT_H.
pub const SETTINGS_SEARCH_H: f32 = BOTTOM_INPUT_H;

/// Ícono del buscador: ancho y alto — 16 px.
pub const SETTINGS_SEARCH_ICON: f32 = ICON_SM;

/// Respiro entre secciones de Ajustes — 16 px.
///
/// Unifica los cuatro ritmos que coexistían (4, 8 y 12 px) alrededor de
/// `separator()`. Una sola medida para "estoy empezando otra sección".
pub const SETTINGS_SECTION_GAP: f32 = SPACE_LG;

/// Medida máxima de la columna de lectura — 680 px.
///
/// Sin tope, la columna crece con la ventana y las filas `label + control`
/// dejan de leerse como una unidad: el label queda a 900 px del control.
/// 680 px mantiene el bloque legible en pantallas anchas.
pub const SETTINGS_COLUMN_MAX_W: f32 = 680.0;

/// Ancho del campo de búsqueda de Ajustes — 320 px.
///
/// Acotado a la izquierda: a todo el ancho era un bloque oscuro que competía
/// con el contenido. 320 = `SETTINGS_CONTENT_MIN_WIDTH`, base 4.
pub const SETTINGS_SEARCH_W: f32 = 320.0;

// ═══════════════════════════════════════════════════════════
// Scrollbar — overlay flotante estilo macOS
//
// Oculta en reposo, visible al hover/scrollear/drag. Sin espacio reservado
// en layout: el contenido va a todo ancho y la barra flota encima.
// ═══════════════════════════════════════════════════════════

/// Ancho del carril del scrollbar — 8 px, estilo overlay moderno.
///
/// 12 px competía con el contenido; 8 flota discreto y deja más aire.
pub const SCROLLBAR_W: f32 = 8.0;

/// Aire derecho del contenido en scrolls (24 = barra 8 + margen 4 + aire 12).
///
/// La barra flotante necesita su casa sin tocar ni tapar: el contenido se
/// angosta esto a la derecha (medido de la caja externa, estable). Sin esto
/// la barra cae encima de botones y texto al aparecer.
/// Aire mínimo contenido-scrollbar: mínimo para que no esté pegado pero sin franja muerta (4px).
pub const SCROLL_CONTENT_PAD_R: f32 = 4.0;

/// Ancho de contenido con el aire derecho descontado (sin piso: el llamador
/// acota según su mínimo).
#[inline]
pub fn scroll_content_width(available_width: f32) -> f32 {
    available_width - SCROLL_CONTENT_PAD_R
}

/// Largo mínimo del thumb — 24 px = `HIT_TARGET_MIN` (WCAG 2.5.8).
/// Un thumb más corto no se puede arrastrar con precisión.
pub const SCROLL_HANDLE_MIN_H: f32 = 24.0;

/// Campos del editor de rango del deslizador — 64×20: columna alineada,
/// compacta, múltiplos de base 4.
pub const RANGE_FIELD_W: f32 = 64.0;
/// Alto del campo de rango — 24 px (piso WCAG 2.5.8, base 4).
pub const RANGE_FIELD_H: f32 = 24.0;
/// Alto de la acción al pie del popup (Borrar) — 28 px: táctil, base 4.
pub const POPUP_ACTION_H: f32 = 28.0;
/// Ancho mínimo del contenido del popup de rango — 140 px: dos campos de
/// ~66 + gap, múltiplo de base 4 (la tarjeta se ciñe, no flota vacía).
pub const POPUP_MIN_W: f32 = 140.0;

/// Margen de las cards overlay del canvas 2D (círculo unitario, etc.) — 14 px.
pub const OVERLAY_CARD_MARGIN: f32 = 14.0;
/// Radio del marcador de hover con snap — 6 px.
pub const HOVER_MARKER_R_SNAP: f32 = 6.0;
/// Radio del marcador de hover libre — 4 px = SPACE_XS.
pub const HOVER_MARKER_R: f32 = SPACE_XS;
/// Anillo extra del marcador de hover — 1 px.
pub const HOVER_MARKER_RING: f32 = 1.0;

// ═══════════════════════════════════════════════════════════
// Paleta de comandos — Scandinavian quiet overlay
// ═══════════════════════════════════════════════════════════

/// Ancho máximo de la paleta — 640 px (lectura cómoda, no tapa el lienzo).
pub const PALETTE_MAX_WIDTH: f32 = 640.0;
/// Margen lateral que la paleta deja al viewport — 16 px = SPACE_LG.
pub const PALETTE_VIEWPORT_MARGIN: f32 = SPACE_LG;
/// Ancho mínimo de ventana (evita colapso en viewports angostos) — 1 px.
pub const PALETTE_MIN_WIDTH: f32 = 1.0;
/// Posición por defecto de la paleta: 8 px desde la izquierda…
pub const PALETTE_POS_X: f32 = SPACE_SM;
/// …y 40 px desde arriba (= TOP_BAR_HEIGHT, respira bajo la barra).
pub const PALETTE_POS_Y: f32 = TOP_BAR_HEIGHT;
/// Caja del icono de búsqueda — 20 px = ICON_MD (piso táctil visual).
pub const PALETTE_SEARCH_ICON: f32 = ICON_MD;
/// Altura reservada a cromo (búsqueda + estado + pie): la lista usa el
/// resto del viewport. 152 px = 38 × base 4 (medido: search 44 + chips
/// 44 + conteo 44 con Limpiar a 44 + gaps 8+8+4; antes 168 con Limpiar
/// de 19 px, sobraba y robaba ~1/3 de fila en viewports cortos).
pub const PALETTE_LIST_RESERVED: f32 = 152.0;
/// Altura mínima de la lista — 120 px (muestra ~4 filas + aire).
pub const PALETTE_LIST_MIN_HEIGHT: f32 = 120.0;
/// Sangría del detalle de la fila seleccionada — 16 px = SPACE_LG.
pub const PALETTE_DETAIL_INDENT: f32 = SPACE_LG;
/// Paso de página en la paleta (PageUp/PageDown) — 10 filas.
pub const PALETTE_PAGE_STEP: usize = 10;
/// Breakpoint paleta-sheet: bajo 740 px la paleta deja de flotar arriba
/// y se ancla abajo (alcance del pulgar). Reusa el 740 del asistente.
pub const PALETTE_BOTTOM_SHEET_BREAKPOINT: f32 = 740.0;
/// Fracción del alto que la sheet ocupa como máximo — 62 % (media hoja:
/// búsqueda + ~8 filas sin tapar todo el lienzo).
pub const PALETTE_SHEET_MAX_FRACTION: f32 = 0.62;
/// Separación de la sheet al borde inferior — 8 px = SPACE_SM.
pub const PALETTE_SHEET_BOTTOM_GAP: f32 = SPACE_SM;
/// Cromo total de la sheet (búsqueda + chips + conteo + MRU + gaps):
/// la lista se acota a `viewport − esto` para que el ancla abajo nunca
/// recorte la búsqueda en viewports cortos. 212 px = 53 × base 4
/// (medido: 44+8+44+8+44+4+44+16 padding; antes 240, sobraba ~28 px).
pub const PALETTE_SHEET_CHROME_RESERVE: f32 = 212.0;
/// Insets del sistema Android (barra de estado + navegación por gestos):
/// estimados de `WindowInsets` típicos (24/32dp); en desktop son 0.
/// eframe 0.29 no expone safe-area, así que el cromo móvil los reserva.
pub const ANDROID_STATUS_INSET: f32 = 24.0;
pub const ANDROID_NAV_INSET: f32 = 32.0;
/// Inset superior del sistema: 0 en desktop, estado en Android.
#[inline]
pub fn safe_inset_top() -> f32 {
    if cfg!(target_os = "android") {
        ANDROID_STATUS_INSET
    } else {
        0.0
    }
}
/// Inset inferior del sistema: 0 en desktop, navegación en Android.
#[inline]
pub fn safe_inset_bottom() -> f32 {
    if cfg!(target_os = "android") {
        ANDROID_NAV_INSET
    } else {
        0.0
    }
}
/// Ancho máximo del Nivel 0 móvil: bajo 480 px se ocultan rail y drawers
/// (68 + 300 px no caben en 360) y el canvas manda con pill + FAB.
/// Desktop ≥480 intacto por construcción.
pub const MOBILE_LEVEL0_MAX_W: f32 = 480.0;
/// Diámetro del FAB móvil (lupa) — 56 px, above-thumb reach.
pub const MOBILE_FAB_D: f32 = 56.0;
/// Diámetro de la burbuja Mili, proporcional al ancho: 11 % del
/// viewport, entre 40 (piso táctil chico) y 56 (techo M3). En un
/// teléfono de 420 px da ~46: presencia sin tapar el lienzo. Pura.
#[inline]
pub fn mili_bubble_d(viewport_width: f32) -> f32 {
    // Piso 44 (antes 40): a 320 el glifo quedaba en ~16 px, justo;
    // 44 sostiene el mínimo táctil y el glifo respira.
    (viewport_width * 0.11).clamp(44.0, MOBILE_FAB_D)
}

/// Alto de filas de ingreso (40px): campo y botón al mismo centro.
pub const INPUT_ROW_H: f32 = 40.0;

// ═══════════════════════════════════════════════════════════
// Bottom chrome — barra de entrada + estado (Scandinavian quiet)
// Una sola superficie `panel_bg`; el campo es tarjeta inset `input_bg`
// con hairline 10% y radio SM (8, rectangular), nunca pill.
// ═══════════════════════════════════════════════════════════

/// Alto del campo de entrada inferior — 32 px = KEYBOARD_KEY_H, base 4,
/// ≥ HIT_TARGET_MIN 24 (WCAG 2.5.8).
pub const BOTTOM_INPUT_H: f32 = 32.0;
/// Alto de la fila de estado (coordenadas · hint · conteo) — 24 px, base 4.
pub const BOTTOM_STATUS_H: f32 = 24.0;
/// Radio del campo de entrada — 8 px = RADIUS_SM (rectangular suave).
/// El pill (999) queda reservado a toggles/controles segmentados.
pub const BOTTOM_INPUT_RADIUS: f32 = RADIUS_SM;
/// Ancho reservado a la derecha del campo (botón ejecutar 44 + aire 8) — 52 px, base 4.
pub const BOTTOM_INPUT_RESERVED_W: f32 = 52.0;
/// Padding horizontal del chrome inferior — 16 px = SPACE_LG.
pub const BOTTOM_BAR_PAD_X: f32 = SPACE_LG;
/// Padding vertical del chrome inferior — 8 px = SPACE_SM.
pub const BOTTOM_BAR_PAD_Y: f32 = SPACE_SM;

// ═══════════════════════════════════════════════════════════
// Autocompletado — popup integrado al campo (Scandinavian quiet)
// Abajo (drawer/álgebra, pegado) o arriba (barra inferior sin aire
// debajo, card flotante con hairline + sombra popup). Nunca tapa el campo:
// el lado se elige por aire disponible y la lista scrollea acotada.
// ═══════════════════════════════════════════════════════════

/// Separación campo ↔ popup flotante de arriba — 4 px = SPACE_XS.
pub const AUTOCOMPLETE_GAP: f32 = SPACE_XS;
/// Reserva de cromo del popup (footer + padding interno) — 32 px, base 4.
pub const AUTOCOMPLETE_CHROME_RESERVE: f32 = 32.0;
/// Lista mínima útil (una fila) — 28 px, base 4.
pub const AUTOCOMPLETE_MIN_LIST_H: f32 = 28.0;

// ═══════════════════════════════════════════════════════════
// Helpers — layout functions (Scandinavian, sin hardcodes)
// ═══════════════════════════════════════════════════════════

/// Indica si el viewport exige layout compacto (≤ BREAKPOINT_COMPACT).
#[inline]
pub fn is_compact_viewport(width: f32) -> bool {
    width <= BREAKPOINT_COMPACT
}

/// Ancho máximo permitido para panel izquierdo dado el ancho disponible.
/// Clamp inferior 200 px evita drawer inutilizable en viewports estrechos.
#[inline]
pub fn panel_left_max_width(available_width: f32) -> f32 {
    (available_width * PANEL_LEFT_MAX_FRACTION).max(200.0)
}

/// Ancho clamped para panel izquierdo (min .. max dinámico).
#[inline]
pub fn clamp_panel_left_width(requested: f32, available_width: f32) -> f32 {
    requested.clamp(PANEL_LEFT_MIN, panel_left_max_width(available_width))
}

/// Ancho clamped para drawer derecho (292 .. 440).
#[inline]
pub fn clamp_drawer_right_width(requested: f32) -> f32 {
    requested.clamp(DRAWER_RIGHT_MIN, DRAWER_RIGHT_MAX)
}

// ═══════════════════════════════════════════════════════════
// Top bar — Scandinavian single bar
// ═══════════════════════════════════════════════════════════

/// Altura única top bar Scandinavian — 40 px single bar (F18.1 unificada:
///
/// breadcrumb F16.2 + disparador paleta Ctrl+K en una sola barra).
pub const TOP_BAR_HEIGHT: f32 = 40.0;

// ═══════════════════════════════════════════════════════════
// Tira de pestañas — Obsidian (S7)
//
// Franja fina a todo el ancho: pestañas RECTANGULARES pegadas (spacing 0,
// radio solo arriba), la activa en superficie elevada con línea de acento
// superior de 2 px (`FOCUS_RING_WIDTH`) y las inactivas atenuadas sobre el
// fondo de la barra. El "+" es una celda más de la tira, mismo alto.
// ═══════════════════════════════════════════════════════════

/// Alto de la tira en desktop — 36 px, fina estilo Obsidian.
pub const TAB_STRIP_H_DESKTOP: f32 = 36.0;
/// Alto de la tira en táctil — 44 px = `HIT_TARGET_AULA` (WCAG 2.5.5).
pub const TAB_STRIP_H_TOUCH: f32 = 44.0;
/// Radio superior de cada pestaña — 4 px = `RADIUS_XS`; abajo 0 para que
/// la activa "conecte" con el contenido sin junta redondeada.
pub const TAB_RADIUS_TOP: f32 = 4.0;
/// Separación entre pestañas — 0: van pegadas, sin pills flotantes.
pub const TAB_STRIP_SPACING: f32 = 0.0;

/// Alto de la tira según viewport: táctil (44) bajo el Nivel 0 móvil
/// (`MOBILE_LEVEL0_MAX_W`), fina desktop (36) en el resto. Misma llave
/// que `mobile_level0` en `grafito-app/src/ui.rs`: altura y predicado no
/// pueden divergir (ver tests de ambos lados).
#[inline]
pub fn tab_strip_h(viewport_width: f32) -> f32 {
    if viewport_width < MOBILE_LEVEL0_MAX_W {
        TAB_STRIP_H_TOUCH
    } else {
        TAB_STRIP_H_DESKTOP
    }
}

// ═══════════════════════════════════════════════════════════
// Zoom — rangos y pill (Scandinavian, opaco, sin translucidez)
// Geogebra-like infinito: 12 órdenes (1e-6..1e6) pizarra, 15 órdenes (1e-6..1e9) 3D
// ═══════════════════════════════════════════════════════════

pub const ZOOM_WB_MIN: f64 = 1e-6;
pub const ZOOM_WB_MAX: f64 = 1e6;
pub const ZOOM_WB_DEFAULT: f64 = 1.0;
pub const ZOOM_3D_MIN: f32 = 1e-6;
pub const ZOOM_3D_MAX: f32 = 1e9;
pub const ZOOM_3D_DEFAULT: f32 = 10.0;
pub const ZOOM_PILL_RADIUS: f32 = RADIUS_SM;
pub const ZOOM_PILL_PAD_X: f32 = SPACE_XS;
pub const ZOOM_PILL_PAD_Y: f32 = SPACE_XS;
pub const ZOOM_PILL_GAP: f32 = SPACE_XS;
pub const ZOOM_PCT_MIN_W: f32 = 52.0;
pub const ZOOM_ICON_HIT: f32 = 32.0;

// ═══════════════════════════════════════════════════════════
// Animation timings
// ═══════════════════════════════════════════════════════════

/// Duración de respuestas visuales a hover y selección (ms).
///
/// Único valor de la escala de motion que la Piel consume. `ANIM_FAST` (100)
/// y `ANIM_NORMAL` (200) se pineaban en un test pero no los usaba ningún
/// call-site: eran constantes que sólo existían para que el test pasara.
pub const ANIM_MICRO: f32 = 180.0;

// ═══════════════════════════════════════════════════════════
// Accesibilidad aula (A11Y) — WCAG 2.2
// Única fuente para foco visible, hit-targets, tipo aula y toasts.
// ═══════════════════════════════════════════════════════════

/// Anillo de foco visible — ancho 2 px (WCAG 2.4.13: perímetro ≥ 2 px).
/// Ver `crate::theme::Theme::focus_ring_stroke`.
pub const FOCUS_RING_WIDTH: f32 = 2.0;
/// Anillo de foco — desplazamiento respecto al borde del widget, 2 px.
pub const FOCUS_RING_OFFSET: f32 = 2.0;
/// Anillo de foco — contraste mínimo contra el fondo adyacente, 3:1 (WCAG 1.4.11).
pub const FOCUS_RING_MIN_CONTRAST: f32 = 3.0;

/// Piso de atenuación gamma para texto — 0.85.
/// Ningún texto se atenúa por debajo del 85 %: `Theme::dimmed_text` hace
/// clamp de todo `gamma_multiply` a `[TEXT_GAMMA_FLOOR, 1.0]` (WCAG 1.4.3 AA).
pub const TEXT_GAMMA_FLOOR: f32 = 0.85;

/// Hit-target mínimo general — 24 px (WCAG 2.5.8 Target Size Minimum).
pub const HIT_TARGET_MIN: f32 = 24.0;
/// Hit-target modo aula/proyector y táctil — 44 px (WCAG 2.5.5 Enhanced).
pub const HIT_TARGET_AULA: f32 = 44.0;

/// Hit-target según viewport: táctil (44) bajo el breakpoint móvil,
/// mínimo desktop (24) en el resto. Puro y pineado: ningún gesto,
/// solo tamaño. Desktop ≥740 intacto por construcción.
pub fn touch_hit(viewport_width: f32) -> f32 {
    if viewport_width < PALETTE_BOTTOM_SHEET_BREAKPOINT {
        HIT_TARGET_AULA
    } else {
        HIT_TARGET_MIN
    }
}

/// Alto del campo de entrada inferior: 44 táctil / 32 desktop.
/// Misma regla que [`touch_hit`], mismo breakpoint.
pub fn bottom_input_h(viewport_width: f32) -> f32 {
    if viewport_width < PALETTE_BOTTOM_SHEET_BREAKPOINT {
        HIT_TARGET_AULA
    } else {
        BOTTOM_INPUT_H
    }
}

/// Tamaño tipográfico mínimo en modo aula — 12 px (legible a distancia).
/// Ver `aula_font_size`.
pub const TYPE_MIN_AULA: f32 = 12.0;
/// Escala tipográfica modo aula — 1.25 (ratio Major Third del sistema).
pub const AULA_FONT_SCALE: f32 = 1.25;

/// Duración por defecto de toasts — 7 s (WCAG 2.2.1: tiempo de lectura).
pub const TOAST_DURATION_DEFAULT: f64 = 7.0;
/// Duración de toasts de error — persistente hasta dismiss con clic (sin auto-dismiss).
pub const TOAST_DURATION_ERROR: f64 = f64::INFINITY;
/// Fade-in de toasts — 0.2 s.
pub const TOAST_FADE_IN: f32 = 0.2;
/// Fade-out de toasts — 0.5 s.
pub const TOAST_FADE_OUT: f32 = 0.5;
/// Altura mínima de toast — 30 px (≥ HIT_TARGET_MIN: clicable para dismiss).
pub const TOAST_MIN_HEIGHT: f32 = 30.0;
/// Offset superior de la pila de toasts — 48 px = TOP_BAR_HEIGHT (40) + SPACE_SM (8).
pub const TOAST_TOP_OFFSET: f32 = TOP_BAR_HEIGHT + SPACE_SM;
/// Fracción máxima de pantalla para la pila — 0.25 (deja 3/4 libres al composer).
pub const TOAST_MAX_SCREEN_FRACTION: f32 = 0.25;

/// Clamp de tamaño interactivo al mínimo WCAG 2.5.8 (24 px).
#[inline]
pub fn hit_target_size(requested: f32) -> f32 {
    requested.max(HIT_TARGET_MIN)
}

/// Clamp de tamaño interactivo al mínimo aula/táctil (44 px).
#[inline]
pub fn aula_hit_target_size(requested: f32) -> f32 {
    requested.max(HIT_TARGET_AULA)
}

/// Tamaño tipográfico modo aula: escala 1.25 con piso 12 px.
#[inline]
pub fn aula_font_size(base: f32) -> f32 {
    (base * AULA_FONT_SCALE).max(TYPE_MIN_AULA)
}

// ═══════════════════════════════════════════════════════════
// Nordic Slate — superficies oscuras (F15.1)
// Rampa azul-grisada, sin negros planos: el canvas #0A0A0A y los
// grises neutros #14/#1A/#2E se reemplazan por N0..N3 + borde
// #2F3642. Separación por borde 1 px (40-60%), no por saltos de
// fill: panel/sidebar/toolbar comparten N1 a propósito.
// ═══════════════════════════════════════════════════════════

/// Canvas oscuro — `#131518` (19, 21, 24). Reemplaza `#0A0A0A`.
pub const SLATE_N0: [u8; 3] = [0x13, 0x15, 0x18];
/// Chrome oscuro (panel/sidebar/toolbar/status) — `#1B1F26`.
pub const SLATE_N1: [u8; 3] = [0x1B, 0x1F, 0x26];
/// Superficie elevada / grilla mayor — `#232935`.
pub const SLATE_N2: [u8; 3] = [0x23, 0x29, 0x35];
/// Hover / base de borde — `#2B323E`.
pub const SLATE_N3: [u8; 3] = [0x2B, 0x32, 0x3E];
/// Borde slate 1 px — `#2F3642` (47, 54, 66). Se usa al 40-60%
/// (`Theme::slate_border_stroke`, punto medio 0.50): quieto pero
/// visible sobre N0/N1. Decorativo (no frontera significativa WCAG
/// 1.4.11: los límites que importan usan el anillo de foco 3:1).
pub const SLATE_BORDER: [u8; 3] = [0x2F, 0x36, 0x42];
/// Rango de alfa del borde slate (40-60%, punto medio 0.50).
pub const SLATE_BORDER_GAMMA_MIN: f32 = 0.4;
pub const SLATE_BORDER_GAMMA_MID: f32 = 0.5;
pub const SLATE_BORDER_GAMMA_MAX: f32 = 0.6;

#[cfg(test)]
// Onda 2: un solo allow a nivel módulo en vez de 11 por test. Estos tests
// pinnean relaciones entre constantes a propósito (p. ej. `TYPE_XS < TYPE_SM`),
// así que `clippy::assertions_on_constants` es un falso positivo aquí.
#[allow(clippy::assertions_on_constants)]
mod tests {
    use super::*;

    #[test]
    fn type_scale_is_monotonic() {
        // TYPE_2XS == TYPE_XS == 11.0: piso mínimo, ya no estrictamente menor.
        assert!(TYPE_2XS <= TYPE_XS);
        assert!(TYPE_XS < TYPE_SM);
        assert!(TYPE_SM < TYPE_BASE);
        assert!(TYPE_BASE < TYPE_MD);
        assert!(TYPE_MD < TYPE_LG);
        assert!(TYPE_LG < TYPE_XL);
        assert!(TYPE_XL < TYPE_XXL);
    }

    #[test]
    fn type_scale_uses_scandinavian_sizes() {
        assert_eq!(TYPE_SM, 12.0);
        assert_eq!(TYPE_BASE, 15.0);
        assert_eq!(TYPE_LG, 19.0);
    }

    #[test]
    fn spacing_scale_is_monotonic() {
        assert!(SPACE_XS < SPACE_SM);
        assert!(SPACE_SM < SPACE_MD);
        assert!(SPACE_MD < SPACE_LG);
        assert!(SPACE_LG < SPACE_XL);
        assert!(SPACE_XL < SPACE_XXL);
    }

    #[test]
    fn spacing_uses_scandinavian_scale() {
        assert_eq!(SPACE_LG, 16.0);
        assert_eq!(SPACE_XL, 24.0);
        assert_eq!(SPACE_XXL, 40.0);
        assert_eq!(SPACING_MINIMAL_X, 16.0);
        assert_eq!(SPACING_MINIMAL_Y, 16.0);
        assert_eq!(SPACING_BUTTON_X, 16.0);
        assert_eq!(SPACING_BUTTON_Y, 8.0);
    }

    #[test]
    fn radii_use_scandinavian_scale() {
        assert_eq!(RADIUS_SM, 8.0);
        assert_eq!(RADIUS_MD, 12.0);
        assert_eq!(RADIUS_LG, 16.0);
        assert_eq!(RADIUS_XL, 20.0);
        assert_eq!(RADIUS_2XL, 24.0);
        // Escala estrictamente creciente: `RADIUS_LG`, `RADIUS_XL` y
        // `RADIUS_2XL` eran los tres 16.0, así que ventana, menú y widgets
        // dibujaban el mismo radio y la escala no distinguía superficies.
        assert!(RADIUS_XS < RADIUS_SM);
        assert!(RADIUS_SM < RADIUS_MD);
        assert!(RADIUS_MD < RADIUS_LG);
        assert!(RADIUS_LG < RADIUS_XL);
        assert!(RADIUS_XL < RADIUS_2XL);
        assert!(RADIUS_2XL < RADIUS_PILL);
    }

    #[test]
    fn shadows_use_scandinavian_values() {
        assert_eq!(SHADOW_WINDOW_OFFSET_Y, 2.0);
        assert_eq!(SHADOW_WINDOW_BLUR, 8.0);
        assert_eq!(SHADOW_POPUP_OFFSET_Y, 2.0);
        assert_eq!(SHADOW_POPUP_BLUR, 8.0);
        assert_eq!(SHADOW_ALPHA, 8);
        assert_eq!(TOP_BAR_HEIGHT, 40.0);
    }

    #[test]
    fn tab_strip_tokens_pinean_geometria_obsidian() {
        // S7: tira fina pegada — desktop 36, táctil 44, spacing 0, radio 4 arriba.
        assert_eq!(TAB_STRIP_H_DESKTOP, 36.0);
        assert_eq!(TAB_STRIP_H_TOUCH, 44.0);
        assert_eq!(TAB_STRIP_H_TOUCH, HIT_TARGET_AULA);
        assert_eq!(TAB_RADIUS_TOP, 4.0);
        assert_eq!(TAB_RADIUS_TOP, RADIUS_XS);
        assert_eq!(TAB_STRIP_SPACING, 0.0);
        // El helper usa la misma llave que `mobile_level0` (<480 táctil).
        assert_eq!(tab_strip_h(320.0), TAB_STRIP_H_TOUCH);
        assert_eq!(tab_strip_h(479.0), TAB_STRIP_H_TOUCH);
        assert_eq!(tab_strip_h(480.0), TAB_STRIP_H_DESKTOP);
        assert_eq!(tab_strip_h(1280.0), TAB_STRIP_H_DESKTOP);
    }

    #[test]
    fn micro_interaction_timing_is_a_quiet_response() {
        // Respuesta a hover y selección: por debajo de 300 ms, que es el techo
        // que la guía de motion pone al motion de UI ordinario.
        assert!(ANIM_MICRO > 0.0);
        assert!(ANIM_MICRO <= 300.0);
    }

    // ── New tokens — breakpoint & panel relationships ──

    #[test]
    fn breakpoint_compact_is_canonical_1360() {
        assert_eq!(BREAKPOINT_COMPACT, 1360.0);
        // Helper debe coincidir con el breakpoint
        assert!(is_compact_viewport(1360.0));
        assert!(is_compact_viewport(960.0));
        assert!(!is_compact_viewport(1361.0));
        assert!(!is_compact_viewport(1680.0));
    }

    #[test]
    fn tablet_tier_splits_phone_tablet_desktop() {
        // Constantes pineadas: 768/1024, múltiplos de base 4, min < max.
        assert_eq!(TABLET_MIN, 768.0);
        assert_eq!(TABLET_MAX, 1024.0);
        assert!(TABLET_MIN < TABLET_MAX);
        assert_eq!(TABLET_MIN % 4.0, 0.0);
        assert_eq!(TABLET_MAX % 4.0, 0.0);
        // Bordes inclusivos en tablet; fuera no cambia nada existente.
        assert_eq!(viewport_tier(320.0), ViewportTier::Phone);
        assert_eq!(viewport_tier(739.0), ViewportTier::Phone);
        assert_eq!(viewport_tier(767.0), ViewportTier::Phone);
        assert_eq!(viewport_tier(768.0), ViewportTier::Tablet);
        assert_eq!(viewport_tier(800.0), ViewportTier::Tablet);
        assert_eq!(viewport_tier(1024.0), ViewportTier::Tablet);
        assert_eq!(viewport_tier(1025.0), ViewportTier::Desktop);
        assert_eq!(viewport_tier(1360.0), ViewportTier::Desktop);
        assert_eq!(viewport_tier(1680.0), ViewportTier::Desktop);
        // El helper booleano coincide con el tier.
        assert!(!is_tablet_viewport(767.0));
        assert!(is_tablet_viewport(768.0));
        assert!(is_tablet_viewport(1024.0));
        assert!(!is_tablet_viewport(1025.0));
        assert!(!is_tablet_viewport(1360.0));
    }

    #[test]
    fn short_landscape_height_threshold_and_fractions() {
        // Umbral pineado h<500 (el onboarding usa su propio h<560).
        assert_eq!(LANDSCAPE_SHORT_MAX_H, 500.0);
        assert!(is_short_landscape_height(499.0));
        assert!(!is_short_landscape_height(500.0));
        assert!(!is_short_landscape_height(800.0));
        // Fracciones en (0, 1): el sheet deja canvas, el default cabe.
        assert!(ASSISTANT_SHEET_LANDSCAPE_FRACTION > 0.0);
        assert!(ASSISTANT_SHEET_LANDSCAPE_FRACTION < 1.0);
        assert!(ASSISTANT_TABLET_DEFAULT_FRACTION > 0.0);
        assert!(ASSISTANT_TABLET_DEFAULT_FRACTION < 1.0);
        // A 900 el default tablet (315) cae dentro del 300..520 pineado.
        let tablet_default = 900.0 * ASSISTANT_TABLET_DEFAULT_FRACTION;
        assert!((300.0..=520.0).contains(&tablet_default));
    }

    #[test]
    fn mili_bubble_proporcional_al_ancho() {
        // 420 px → ~46 (más chica que 56); piso 44; techo 56.
        assert!((mili_bubble_d(420.0) - 46.2).abs() < 0.01);
        assert_eq!(mili_bubble_d(320.0), 44.0);
        assert_eq!(mili_bubble_d(360.0), 44.0);
        assert_eq!(mili_bubble_d(800.0), MOBILE_FAB_D);
    }

    #[test]
    fn panel_left_relationships_hold() {
        assert!(PANEL_LEFT_MIN < PANEL_LEFT_DEFAULT);
        assert_eq!(PANEL_LEFT_DEFAULT, 300.0);
        assert_eq!(PANEL_LEFT_MIN, 240.0);
        assert!((PANEL_LEFT_MAX_FRACTION - 0.45).abs() < f32::EPSILON);
        assert!(PANEL_LEFT_MAX_FRACTION > 0.0 && PANEL_LEFT_MAX_FRACTION < 1.0);
        // Default cabe dentro del max para viewport típico 1280
        let max_1280 = panel_left_max_width(1280.0);
        assert!(PANEL_LEFT_DEFAULT <= max_1280);
        assert!(max_1280 >= 200.0);
        // Clamp respeta min/max
        assert_eq!(clamp_panel_left_width(100.0, 1280.0), PANEL_LEFT_MIN);
        assert_eq!(clamp_panel_left_width(1000.0, 1280.0), max_1280);
        assert_eq!(clamp_panel_left_width(260.0, 1280.0), 260.0);
    }

    #[test]
    fn drawer_right_relationships_hold() {
        assert!(DRAWER_RIGHT_MIN < DRAWER_RIGHT_DEFAULT);
        assert!(DRAWER_RIGHT_DEFAULT < DRAWER_RIGHT_MAX);
        assert_eq!(DRAWER_RIGHT_DEFAULT, 344.0);
        assert_eq!(DRAWER_RIGHT_MIN, 292.0);
        assert_eq!(DRAWER_RIGHT_MAX, 440.0);
        assert_eq!(clamp_drawer_right_width(200.0), DRAWER_RIGHT_MIN);
        assert_eq!(clamp_drawer_right_width(500.0), DRAWER_RIGHT_MAX);
        assert_eq!(clamp_drawer_right_width(344.0), 344.0);
    }

    #[test]
    fn rail_and_splash_use_scandinavian_tokens() {
        assert_eq!(RAIL_WIDTH, 68.0);
        assert_eq!(RAIL_ITEM_HEIGHT, 60.0);
        assert_eq!(RAIL_ITEM_PAD_X, SPACE_XS);
        // El item respira 4 px por lado: el borde nunca toca el panel.
        assert_eq!(RAIL_WIDTH - 2.0 * RAIL_ITEM_PAD_X, 60.0);
        assert_eq!(SPLASH_LOGO_SIZE, 128.0);
        // Splash es múltiplo de base 4 y cuadrado
        assert_eq!(SPLASH_LOGO_SIZE % 4.0, 0.0);
        assert_eq!(RAIL_WIDTH % 4.0, 0.0);
    }

    #[test]
    fn alphas_match_scandinavian_quiet() {
        assert_eq!(ALPHA_SHADOW, 8);
        assert_eq!(SHADOW_ALPHA, ALPHA_SHADOW);
        assert_eq!(ALPHA_OVERLAY, 25);
        // Sombra < overlay (sutil → visible). El separator canónico no es un
        // alfa: `theme.rs` lo deriva con `gamma_multiply(0.10)` sobre el color
        // del token, así que depende de la superficie y no tiene un valor
        // único que se pueda pinear acá.
        assert!(ALPHA_SHADOW < ALPHA_OVERLAY);
    }

    #[test]
    fn card_tokens_are_canonical_aliases() {
        assert_eq!(CARD_SPACING, SPACE_MD);
        assert_eq!(CARD_SPACING, 12.0);
        assert_eq!(OBJECT_CARD_RADIUS, RADIUS_SM);
        assert_eq!(OBJECT_CARD_RADIUS, 8.0);
        assert_eq!(INSPECTOR_CARD_RADIUS, RADIUS_MD);
        assert_eq!(INSPECTOR_CARD_RADIUS, 12.0);
    }

    #[test]
    fn spacing_uses_base_4() {
        for v in [
            SPACE_XS,
            SPACE_SM,
            SPACE_MD,
            SPACE_LG,
            SPACE_XL,
            SPACE_XXL,
            SPACING_MINIMAL_X,
            SPACING_MINIMAL_Y,
            SPACING_BUTTON_X,
            SPACING_BUTTON_Y,
            CARD_SPACING,
            RAIL_WIDTH,
            SPLASH_LOGO_SIZE,
            RANGE_FIELD_W,
            RANGE_FIELD_H,
            POPUP_ACTION_H,
            POPUP_MIN_W,
            TAB_STRIP_H_DESKTOP,
            TAB_STRIP_H_TOUCH,
            TAB_RADIUS_TOP,
        ] {
            assert_eq!(v % 4.0, 0.0, "spacing value {v} must be multiple of base 4");
        }
    }

    #[test]
    fn type_scale_ratio_stays_near_1_25() {
        // Ratio 1.25 Major Third — verificar que la progresión no se rompa.
        // No exacto por redondeo Scandinavian (12/15/19), pero cercano.
        let ratio_sm_base = TYPE_BASE / TYPE_SM; // 15/12 = 1.25 exact
        let ratio_base_lg = TYPE_LG / TYPE_BASE; // 19/15 ≈ 1.266
        assert!((ratio_sm_base - 1.25).abs() < 0.01);
        assert!((ratio_base_lg - 1.25).abs() < 0.05);
        // Monotonic ya verificado, aquí sólo ratio
        assert!(TYPE_SM < TYPE_BASE && TYPE_BASE < TYPE_LG);
    }

    #[test]
    fn focus_ring_tokens_meet_wcag_minimums() {
        assert_eq!(FOCUS_RING_WIDTH, 2.0);
        assert_eq!(FOCUS_RING_OFFSET, 2.0);
        assert_eq!(FOCUS_RING_MIN_CONTRAST, 3.0);
    }

    #[test]
    fn text_gamma_floor_preserves_contrast() {
        assert_eq!(TEXT_GAMMA_FLOOR, 0.85);
        assert!(TEXT_GAMMA_FLOOR > 0.0 && TEXT_GAMMA_FLOOR <= 1.0);
    }

    #[test]
    fn hit_targets_meet_wcag_sizes() {
        assert_eq!(HIT_TARGET_MIN, 24.0);
        assert_eq!(HIT_TARGET_AULA, 44.0);
        assert!(HIT_TARGET_MIN < HIT_TARGET_AULA);
        assert_eq!(hit_target_size(16.0), HIT_TARGET_MIN);
        assert_eq!(hit_target_size(32.0), 32.0);
        assert_eq!(aula_hit_target_size(24.0), HIT_TARGET_AULA);
        assert_eq!(aula_hit_target_size(48.0), 48.0);
    }

    #[test]
    fn aula_type_has_floor_and_scale() {
        assert_eq!(TYPE_MIN_AULA, 12.0);
        assert_eq!(AULA_FONT_SCALE, 1.25);
        assert_eq!(aula_font_size(TYPE_BASE), TYPE_BASE * AULA_FONT_SCALE);
        // Piso: bases pequeñas no bajan de 12 px.
        assert_eq!(aula_font_size(9.0), TYPE_MIN_AULA);
        assert_eq!(aula_font_size(TYPE_XS), TYPE_XS * AULA_FONT_SCALE);
    }

    #[test]
    fn toast_durations_give_reading_time_and_persistent_errors() {
        assert_eq!(TOAST_DURATION_DEFAULT, 7.0);
        assert!(TOAST_DURATION_ERROR.is_infinite());
        assert!(TOAST_DURATION_DEFAULT < TOAST_DURATION_ERROR);
        assert!(TOAST_MIN_HEIGHT >= HIT_TARGET_MIN);
        assert_eq!(TOAST_TOP_OFFSET, TOP_BAR_HEIGHT + SPACE_SM);
    }

    #[test]
    fn bottom_chrome_tokens_are_quiet_and_rectangular() {
        // Rectangular suave: 8, nunca pill.
        assert_eq!(BOTTOM_INPUT_RADIUS, RADIUS_SM);
        assert_eq!(BOTTOM_INPUT_RADIUS, 8.0);
        assert!(BOTTOM_INPUT_RADIUS < 16.0);
        // Altos táctiles en base 4 y por encima del mínimo WCAG 24.
        assert_eq!(BOTTOM_INPUT_H, 32.0);
        assert_eq!(BOTTOM_STATUS_H, 24.0);
        assert!(BOTTOM_INPUT_H >= HIT_TARGET_MIN);
        assert_eq!(BOTTOM_BAR_PAD_X, SPACE_LG);
        assert_eq!(BOTTOM_BAR_PAD_Y, SPACE_SM);
        for v in [
            BOTTOM_INPUT_H,
            BOTTOM_STATUS_H,
            BOTTOM_INPUT_RESERVED_W,
            BOTTOM_BAR_PAD_X,
            BOTTOM_BAR_PAD_Y,
        ] {
            assert_eq!(v % 4.0, 0.0, "bottom value {v} must be multiple of base 4");
        }
    }

    #[test]
    fn autocomplete_tokens_fit_base_4_and_reserve_chrome() {
        assert_eq!(AUTOCOMPLETE_GAP, SPACE_XS);
        assert_eq!(AUTOCOMPLETE_CHROME_RESERVE, 32.0);
        assert_eq!(AUTOCOMPLETE_MIN_LIST_H, 28.0);
        // La reserva cubre footer + padding: la lista mínima + cromo es
        // lo mínimo que exige el lado de abajo para no flipear.
        let min_below = AUTOCOMPLETE_MIN_LIST_H + AUTOCOMPLETE_CHROME_RESERVE + AUTOCOMPLETE_GAP;
        assert_eq!(min_below, 64.0);
        for v in [
            AUTOCOMPLETE_GAP,
            AUTOCOMPLETE_CHROME_RESERVE,
            AUTOCOMPLETE_MIN_LIST_H,
        ] {
            assert_eq!(v % 4.0, 0.0, "value {v} must be multiple of base 4");
        }
    }

    #[test]
    fn palette_tokens_use_base_4_and_fit_viewport() {
        assert_eq!(PALETTE_MAX_WIDTH, 640.0);
        assert_eq!(PALETTE_VIEWPORT_MARGIN, SPACE_LG);
        assert_eq!(PALETTE_POS_X, SPACE_SM);
        assert_eq!(PALETTE_POS_Y, TOP_BAR_HEIGHT);
        assert_eq!(PALETTE_SEARCH_ICON, ICON_MD);
        assert_eq!(PALETTE_DETAIL_INDENT, SPACE_LG);
        assert_eq!(PALETTE_PAGE_STEP, 10);
        for v in [
            PALETTE_MAX_WIDTH,
            PALETTE_VIEWPORT_MARGIN,
            PALETTE_POS_X,
            PALETTE_POS_Y,
            PALETTE_SEARCH_ICON,
            PALETTE_LIST_RESERVED,
            PALETTE_LIST_MIN_HEIGHT,
            PALETTE_DETAIL_INDENT,
        ] {
            assert_eq!(v % 4.0, 0.0, "palette value {v} must be multiple of base 4");
        }
        // La lista siempre deja aire: reservado > mínimo.
        assert!(PALETTE_LIST_RESERVED > PALETTE_LIST_MIN_HEIGHT);
        // S2 táctil: 44 bajo 740, 24 en desktop; entrada 44/32.
        assert_eq!(touch_hit(360.0), HIT_TARGET_AULA);
        assert_eq!(touch_hit(739.0), HIT_TARGET_AULA);
        assert_eq!(touch_hit(740.0), HIT_TARGET_MIN);
        assert_eq!(touch_hit(1360.0), HIT_TARGET_MIN);
        assert_eq!(bottom_input_h(360.0), 44.0);
        assert_eq!(bottom_input_h(1360.0), BOTTOM_INPUT_H);
    }

    // ── New tokens — welcome / settings composition ──

    #[test]
    fn welcome_tokens_keep_the_reading_column_readable() {
        // La medida del headline tiene que dejar el subtítulo en su rango de
        // lectura (55–70 ch) sin que el titular corra a lo ancho del viewport.
        assert_eq!(WELCOME_COLUMN_WIDTH, 640.0);
        assert!(WELCOME_SUB_COLUMN_WIDTH < WELCOME_COLUMN_WIDTH);
        assert!(WELCOME_SUB_COLUMN_WIDTH >= 480.0);
        // El bloque completo entra en el tercio superior: antes flotaba en el
        // centro matemático y dejaba ~700 px muertos encima y debajo.
        assert!(WELCOME_BLOCK_TOP_RATIO > 0.1 && WELCOME_BLOCK_TOP_RATIO < 0.2);
        assert_eq!(WELCOME_MARGIN_X, SPACE_XL);
        // La banda es un indicador, no un control: no necesita 24 px.
        assert!(WELCOME_PROGRESS_BAR_H < HIT_TARGET_MIN);
        assert!(WELCOME_PROGRESS_BAR_H >= SPACE_XS);
        for v in [
            WELCOME_COLUMN_WIDTH,
            WELCOME_SUB_COLUMN_WIDTH,
            WELCOME_MARGIN_X,
            WELCOME_PROGRESS_BAR_H,
            WELCOME_PROGRESS_MAX_W,
        ] {
            assert_eq!(v % 4.0, 0.0, "welcome value {v} must be multiple of base 4");
        }
    }

    #[test]
    fn settings_tokens_leave_room_for_the_control_column() {
        // El carril no crece con el viewport: con un porcentaje fijo se comía
        // la columna de lectura. 240 px contra un mínimo de 320 de contenido.
        assert_eq!(SETTINGS_NAV_WIDTH, 240.0);
        assert!(SETTINGS_CONTENT_MIN_WIDTH > SETTINGS_NAV_WIDTH);
        // El breakpoint de apilado tiene que ser el mínimo + el control más
        // ancho: por debajo, `label + control` no cabe en una línea.
        assert_eq!(SETTINGS_CONTENT_NARROW, SETTINGS_CONTENT_MIN_WIDTH + 160.0);
        // Switch 44 × 24: el dibujado a mano medía 38 × 22 y quedaba dos
        // puntos por debajo del mínimo accesible en alto.
        assert_eq!(SETTINGS_SWITCH_H, HIT_TARGET_MIN);
        assert!(SETTINGS_SWITCH_W >= HIT_TARGET_AULA);
        assert!(SETTINGS_SWITCH_KNOB_R * 2.0 < SETTINGS_SWITCH_W);
        assert!(SETTINGS_SWITCH_KNOB_R * 2.0 < SETTINGS_SWITCH_H);
        assert_eq!(SETTINGS_ROW_MIN_H, HIT_TARGET_AULA);
        for v in [
            SETTINGS_NAV_WIDTH,
            SETTINGS_CONTENT_MIN_WIDTH,
            SETTINGS_CONTENT_NARROW,
            SETTINGS_CONTROL_WIDTH,
            SETTINGS_SWITCH_W,
            SETTINGS_SWITCH_H,
        ] {
            assert_eq!(
                v % 4.0,
                0.0,
                "settings value {v} must be multiple of base 4"
            );
        }
    }

    #[test]
    fn settings_tokens_keep_both_carriles_symmetric() {
        // El buscador se integra al carril: su ancho es el del carril, no un
        // tope suelto de 260 px que no alineaba con ninguno de los dos.
        assert_eq!(SETTINGS_CARD_PAD_X, SPACE_MD);
        assert_eq!(SETTINGS_NAV_ICON_W, ICON_SM);
        assert_eq!(SETTINGS_NAV_TEXT_GAP, SPACE_SM);
        assert_eq!(SETTINGS_SEARCH_H, BOTTOM_INPUT_H);
        assert_eq!(SETTINGS_SEARCH_ICON, ICON_SM);
        // Ícono + gap + padding suman un múltiplo de 4: el texto del ítem
        // arranca en un offset exacto, no en el +30 hardcodeado de antes.
        let text_inset = SETTINGS_CARD_PAD_X + SETTINGS_NAV_ICON_W + SETTINGS_NAV_TEXT_GAP;
        assert_eq!(text_inset, 36.0);
        assert_eq!(text_inset % 4.0, 0.0);
        // La columna tiene tope: sin él las filas label+control se estiran
        // hasta perder la lectura en bloque.
        assert!(SETTINGS_COLUMN_MAX_W > SETTINGS_CONTENT_MIN_WIDTH);
        assert!(SETTINGS_COLUMN_MAX_W <= 720.0);
        // Ajustes es fullscreen con columna centrada: sin tamaño de ventana.
        // Overlay flotante: sin reserva en layout (la barra flota encima).
        // El buscador acotado deja aire + X: menor que la columna mínima.
        assert_eq!(SETTINGS_SEARCH_W, 320.0);
        assert_eq!(SETTINGS_SEARCH_W, SETTINGS_CONTENT_MIN_WIDTH);
        // Un solo ritmo de sección.
        assert_eq!(SETTINGS_SECTION_GAP, SPACE_LG);
        // Alto de ítem de nav y de fila son decisiones distintas y
        // documentadas: índice contra formulario.
        assert!(SETTINGS_NAV_ITEM_H < SETTINGS_ROW_MIN_H);
        assert!(SETTINGS_NAV_ITEM_H >= 24.0);
        for v in [
            SETTINGS_CARD_PAD_X,
            SETTINGS_NAV_ICON_W,
            SETTINGS_NAV_TEXT_GAP,
            SETTINGS_NAV_ITEM_H,
            SETTINGS_SEARCH_H,
            SETTINGS_SEARCH_ICON,
            SETTINGS_COLUMN_MAX_W,
            SETTINGS_SEARCH_W,
            SETTINGS_SECTION_GAP,
        ] {
            assert_eq!(
                v % 4.0,
                0.0,
                "settings value {v} must be multiple of base 4"
            );
        }
    }

    #[test]
    fn slate_tokens_pinean_hex_y_rango_de_borde() {
        // F15.1: sin negros planos, rampa azul-grisada pineada.
        assert_eq!(SLATE_N0, [0x13, 0x15, 0x18]);
        assert_eq!(SLATE_N1, [0x1B, 0x1F, 0x26]);
        assert_eq!(SLATE_N2, [0x23, 0x29, 0x35]);
        assert_eq!(SLATE_N3, [0x2B, 0x32, 0x3E]);
        assert_eq!(SLATE_BORDER, [0x2F, 0x36, 0x42]);
        // Ningún nivel es negro plano ni gris neutro (#00/#0A/#14/#1A/#2E).
        for nivel in [SLATE_N0, SLATE_N1, SLATE_N2, SLATE_N3, SLATE_BORDER] {
            let [r, g, b] = nivel;
            assert!(r >= 0x13, "slate demasiado oscuro: {nivel:?}");
            assert!(
                b > r,
                "slate debe tener tinte azul, no gris neutro: {nivel:?}"
            );
            assert!(g >= r && g <= b, "rampa slate r<=g<=b (fría): {nivel:?}");
        }
        // Rango de borde 40-60% con punto medio 0.50.
        assert_eq!(SLATE_BORDER_GAMMA_MIN, 0.4);
        assert_eq!(SLATE_BORDER_GAMMA_MID, 0.5);
        assert_eq!(SLATE_BORDER_GAMMA_MAX, 0.6);
        assert!(SLATE_BORDER_GAMMA_MIN < SLATE_BORDER_GAMMA_MID);
        assert!(SLATE_BORDER_GAMMA_MID < SLATE_BORDER_GAMMA_MAX);
    }
}
