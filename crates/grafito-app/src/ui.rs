//! Top-level egui chrome: menu bar, toolbar, icon sidebar, input/status bars,
//! and the floating color-picker dialog.

use crate::app::{
    ActiveColorPicker, AutocompleteItem, ColorPickerTarget, DeferredPanelSnapshot, DocumentAction,
    FileCommand, UnsavedDecision,
};
use crate::{GrafitoApp, Perspective, ViewMode, WorkspaceDockTab};
use egui::{Align2, Color32};
use grafito_ui::animation::interpolate_color;
use grafito_ui::icons::{action_icon_button, action_icon_button_sized, draw_icon, Icon};
use grafito_ui::theme::{current_theme, DARK, LIGHT};
use grafito_ui::tokens::{
    bottom_input_h, AUTOCOMPLETE_CHROME_RESERVE, AUTOCOMPLETE_GAP, AUTOCOMPLETE_MIN_LIST_H,
    BOTTOM_BAR_PAD_X, BOTTOM_BAR_PAD_Y, BOTTOM_INPUT_RADIUS, BOTTOM_INPUT_RESERVED_W,
    BREAKPOINT_COMPACT, DRAWER_RIGHT_DEFAULT, DRAWER_RIGHT_MAX, DRAWER_RIGHT_MIN, HIT_TARGET_AULA,
    ICON_MD, MOBILE_LEVEL0_MAX_W, MOBILE_MENU_TEXT, PANEL_LEFT_MIN, RADIUS_LG, RADIUS_PILL,
    RADIUS_SM, RAIL_ITEM_HEIGHT, RAIL_ITEM_PAD_X, RAIL_WIDTH, SHADOW_ALPHA, SHADOW_POPUP_BLUR,
    SHADOW_POPUP_OFFSET_Y, SPACE_LG, SPACE_MD, SPACE_SM, SPACE_XS, SPACING_BUTTON_X,
    SPACING_BUTTON_Y, SPACING_MINIMAL_X, SPACING_MINIMAL_Y, TYPE_2XS, TYPE_BASE, TYPE_SM, TYPE_XS,
};
use grafito_ui::Tool;
use std::collections::VecDeque;

pub(crate) struct CommandInputResponse {
    pub submitted: bool,
    pub changed: bool,
}

/// The native minimum is 960 logical points; this also covers HiDPI windows
/// whose physical captures look much narrower than their egui width.
pub(crate) const COMPACT_TOP_CHROME_MAX_WIDTH: f32 = BREAKPOINT_COMPACT;

/// Ancho mínimo del rail para pintar sus etiquetas cortas. Debajo de este
/// ancho el rail colapsa a iconos puros: pintar texto en una franja de 1-2px
/// deja slivers (fragmentos en zigzag) en el borde del panel. Puro y
/// testeable headless.
pub(crate) const RAIL_LABEL_MIN_WIDTH: f32 = 40.0;

/// Geometría del toggle de tema en la barra.
///
/// El alto es `HIT_TARGET_MIN` (24 px, WCAG 2.5.8): el de 20 px que usaba
/// antes quedaba cuatro puntos por debajo del mínimo y rompía el bloque de
/// pulsación real, no solo el visual. 24 + 2×12 de aire sigue entrando en la
/// banda de 48 px de la barra, así que el arreglo no cuesta layout.
const THEME_TOGGLE_W: f32 = 36.0;
const THEME_TOGGLE_H: f32 = grafito_ui::tokens::HIT_TARGET_MIN;
/// Lado del icono sol/luna — 12 px.
const THEME_TOGGLE_ICON: f32 = 12.0;
/// Inset del icono y de la perilla respecto del borde del track — 8 px.
///
/// Icono y perilla comparten el valor: antes iban a 6.5 y 8.5, con 2 px de
/// desfase en el mismo control.
const THEME_TOGGLE_ICON_INSET: f32 = 8.0;
/// Radio de la perilla — 6 px, el mayor disco que entra en el track de 24 px
/// con aire a ambos lados (8 + 6 + 6 = 20 de los 24 disponibles, dejando 4 px
/// de fondo para el dedo). El valor viejo (6 px) ya era el correcto para este
/// alto; lo que estaba mal calibrado era el alto de 20 px, no el radio.
const THEME_TOGGLE_KNOB_R: f32 = 6.0;
/// Gamma del icono inactivo — pasa por `dimmed_text`, así que el piso real es
/// `TEXT_GAMMA_FLOOR`. Antes se usaba `gamma_multiply(0.60)` directo.
const THEME_TOGGLE_INACTIVE_GAMMA: f32 = 0.85;

/// Alto de botones de acción dentro de menús (28 px = mismo lenguaje que
/// el inspector; ≥ `HIT_TARGET_MIN` 24, WCAG 2.5.8).
const MENU_ACTION_H: f32 = grafito_ui::tokens::POPUP_ACTION_H;

/// Ancho mínimo del panel de menús con lista (Perspectivas).
///
/// 252 px era un literal suelto: era el único `set_min_width` de toda la barra
/// y dejaba ese popup más ancho que sus vecinos. Ahora el token deja la
/// decisión documentada y el ancho sale de un solo lugar.
const MENU_PANEL_MIN_W: f32 = 240.0;

pub(crate) fn rail_labels_visible(rect_width: f32) -> bool {
    rect_width >= RAIL_LABEL_MIN_WIDTH
}

/// Ancho mínimo del drawer izquierdo para pintar su contenido textual.
///
/// egui colapsa el `SidePanel` a `available` cuando el mínimo no entra
/// (`panel.rs`: `clamp_to_range(width, width_range).at_most(available_rect)`):
/// con 0-60px disponibles el contenido igual exige `min_width` y el clip
/// deja slivers (fragmentos en zigzag) en el borde izquierdo. Debajo de
/// `PANEL_LEFT_MIN` el layout no pinta el drawer; el rail sigue ofreciendo
/// reapertura. Precedente: `RAIL_LABEL_MIN_WIDTH` para el rail. Puro y
/// testeable headless.
pub(crate) fn left_drawer_content_visible(available_width: f32) -> bool {
    available_width >= PANEL_LEFT_MIN
}

/// Dead-click del rail (BB3-P1-5): el click en el rail prende los flags
/// pero el drawer no se pinta porque el ancho disponible no llega a
/// `PANEL_LEFT_MIN`. Solo fuera de Nivel 0 (bajo 480 el Nivel 0 ya avisa
/// con su propio toast): la franja típica es 480-520 con rail + panel
/// mínimo comiéndose el canvas. Puro y pineado.
pub(crate) fn rail_drawer_blocked(
    viewport_width: f32,
    available_width: f32,
    drawer_requested: bool,
) -> bool {
    drawer_requested
        && !mobile_level0(viewport_width)
        && !left_drawer_content_visible(available_width)
}

/// Altura default del dock compacto inferior (BB3-P1-7): el `default_height`
/// fijo de 360 px clipaba cuando el viewport angosto dejaba `max_height`
/// por debajo (egui lo clampaba en silencio y el contenido nacía recortado).
/// Se clampéa a `min(360, max)`. Puro y pineado.
pub(crate) fn compact_dock_default_height(available_height: f32) -> f32 {
    let max = (available_height * 0.65).max(180.0);
    360.0_f32.min(max)
}

/// Altura del chip de herramienta armada en Nivel 0 (BB3-P1-6): 44 px
/// táctiles en vez de 32 (el target más chico de la zona más usada a 390).
/// Puro y pineado.
pub(crate) const fn tool_chip_height() -> f32 {
    HIT_TARGET_AULA
}

pub(crate) fn top_chrome_uses_overflow(viewport_width: f32) -> bool {
    viewport_width <= COMPACT_TOP_CHROME_MAX_WIDTH
}

/// Nivel 0 móvil: bajo 480 px el rail (68) + drawers (300) se comen el
/// canvas; se ocultan y quedan pill de herramienta + FAB lupa + entrada.
/// Puro y pineado. Desktop ≥480 intacto por construcción.
pub(crate) fn mobile_level0(viewport_width: f32) -> bool {
    viewport_width < MOBILE_LEVEL0_MAX_W
}

/// ¿Se suprime el tooltip en táctil? En Nivel 0 (<480, sin cursor) el
/// `on_hover_text` que repite el texto visible es eco: el dedo tapa la
/// pantalla y el tooltip no aporta nada (en el drawer angosto hasta puede
/// empujar el layout). Se conserva si el texto dice algo distinto al
/// visible (hint, conteo, atajo) y siempre en botones solo-ícono
/// (`visible` vacío: el hover ES el nombre accesible). Puro y pineado.
pub(crate) fn hover_echo_suppressed(visible: &str, hover: &str, narrow_touch: bool) -> bool {
    narrow_touch && !visible.is_empty() && visible == hover
}

/// Burbuja del asistente (M3 FAB): bajo 740 px el panel inferior fijo
/// se come la pantalla (Mili ocupaba ~60% a 430 px). Con burbuja hay
/// FAB 56 cuando está oculto y hoja a pantalla completa cuando abre:
/// mismo contenido, otro contenedor (precedente S8 Aula). Pura y
/// pineada. Desktop ≥740 intacto por construcción.
pub(crate) fn mobile_assistant_bubble(viewport_width: f32) -> bool {
    viewport_width < grafito_ui::tokens::PALETTE_BOTTOM_SHEET_BREAKPOINT
}

/// Ancho mínimo del contenido de un diálogo modal. En desktop 320 px;
/// en viewports angostos deja 16 px de margen por lado para no pegar el
/// texto al borde (los `set_min_width(320)` fijos daban margen cero en
/// 320 px). Puro y pineado.
pub(crate) fn dialog_content_min_width(viewport_width: f32) -> f32 {
    (viewport_width - 32.0).clamp(200.0, 320.0)
}

/// ¿Se apilan los botones del diálogo en vertical? Si la fila completa no
/// cabe en el ancho disponible, apilar a ancho completo (etiquetas intactas
/// y targets grandes) en vez de recortar a la derecha. Puro y pineado.
pub(crate) fn dialog_buttons_stack(available_width: f32, row_width: f32) -> bool {
    available_width < row_width
}

pub(crate) const fn assistant_reopen_control_visible(assistant_visible: bool) -> bool {
    !assistant_visible
}

pub(crate) fn restore_assistant_visibility(
    assistant_visible: &mut bool,
    reopen_requested: bool,
) -> bool {
    if !reopen_requested || !assistant_reopen_control_visible(*assistant_visible) {
        return false;
    }
    *assistant_visible = true;
    true
}

pub(crate) fn draw_assistant_reopen_control(
    ui: &mut egui::Ui,
    assistant_visible: &mut bool,
    accent: Color32,
    locale: grafito_ui::i18n::Locale,
) -> Option<egui::Response> {
    if !assistant_reopen_control_visible(*assistant_visible) {
        return None;
    }
    // B-A186: etiqueta + hover vía `shell.*` (antes ES duros).
    let reopen_label = grafito_ui::i18n::t("shell.assistant_reopen", locale);
    let reopen_hint = grafito_ui::i18n::t("shell.assistant_reopen_hint", locale);
    let response = ui.button(egui::RichText::new(reopen_label).color(accent));
    // Sin eco táctil: el hint aporta algo distinto ("Mostrar…"), así que
    // queda también en Nivel 0; si algún día repitiera la etiqueta, la
    // guarda lo suprime solo ahí (botones solo-ícono siempre lo conservan).
    let narrow_touch = mobile_level0(ui.ctx().screen_rect().width());
    let response = if hover_echo_suppressed(reopen_label, reopen_hint, narrow_touch) {
        response
    } else {
        response.on_hover_text(reopen_hint)
    };
    let _ = restore_assistant_visibility(assistant_visible, response.clicked());
    Some(response)
}

fn draw_theme_toggle_switch(
    ui: &mut egui::Ui,
    is_dark: bool,
    locale: grafito_ui::i18n::Locale,
) -> bool {
    use grafito_ui::i18n::t;
    let theme = current_theme(ui.ctx());
    // B-A186: hover + nombre accesible vía `shell.*` (antes ES duros).
    let label = if is_dark {
        t("shell.theme_to_light", locale)
    } else {
        t("shell.theme_to_dark", locale)
    };
    let desired_size = egui::vec2(THEME_TOGGLE_W, THEME_TOGGLE_H);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    let response = response
        .on_hover_text(label)
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    // Nombre, rol y estado para lectores de pantalla (WCAG 4.1.2). El estado
    // del switch viajaba solo en el icono sol/luna, que es un detalle gráfico:
    // sin esto, un lector de pantalla anunciaba un control sin nombre y sin
    // indicación de si el tema estaba claro u oscuro.
    response.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::Button,
            true,
            if is_dark {
                t("shell.theme_dark_on", locale)
            } else {
                t("shell.theme_light_on", locale)
            },
        )
    });
    if ui.is_rect_visible(rect) {
        // Fondo del track: `input_bg` es una superficie real del sistema, no un
        // `gamma_multiply` crudo sobre el separator (que daba 1.16:1 en oscuro
        // y arrastraba el alfa del canal).
        let bg = if response.hovered() {
            theme.button_hover
        } else {
            theme.input_bg
        };
        let radius = THEME_TOGGLE_H / 2.0;
        ui.painter().rect_filled(rect, radius, bg);
        ui.painter()
            .rect_stroke(rect, radius, theme.hairline_stroke());

        // Iconos sol/luna: el activo usa el acento, el inactivo pasa por
        // `dimmed_text`, que respeta el piso `TEXT_GAMMA_FLOOR`. Antes se
        // usaba `gamma_multiply(0.60)` directo: daba 2.30:1 en oscuro (el piso
        // de WCAG 1.4.11 para no-texto es 3:1) y de paso bajaba el alfa del
        // canal, así que el icono apagado era casi invisible.
        let icon_side = THEME_TOGGLE_ICON;
        let inactive =
            grafito_ui::theme::Theme::dimmed_text(theme.text_tertiary, THEME_TOGGLE_INACTIVE_GAMMA);
        let inset = THEME_TOGGLE_ICON_INSET;
        draw_icon(
            ui.painter(),
            egui::Rect::from_center_size(
                egui::pos2(rect.min.x + inset, rect.center().y),
                egui::vec2(icon_side, icon_side),
            ),
            Icon::Sun,
            if !is_dark { theme.accent } else { inactive },
        );
        draw_icon(
            ui.painter(),
            egui::Rect::from_center_size(
                egui::pos2(rect.max.x - inset, rect.center().y),
                egui::vec2(icon_side, icon_side),
            ),
            Icon::Moon,
            if is_dark { theme.accent } else { inactive },
        );

        // Perilla: se posiciona con los mismos inset que los iconos para que
        // activo e icono caigan en el mismo eje. Antes iban a 8.5 y los iconos
        // a 6.5 — 2 px de desfasaje en el mismo control.
        let knob_r = THEME_TOGGLE_KNOB_R;
        let knob_x = if is_dark {
            rect.max.x - inset
        } else {
            rect.min.x + inset
        };
        let knob_center = egui::pos2(knob_x, rect.center().y);
        // Sombra con el alfa canónico: `from_black_alpha(20)` era 2.5× el
        // `SHADOW_ALPHA` del sistema.
        ui.painter().circle_filled(
            knob_center + egui::vec2(0.0, 1.0),
            knob_r,
            Color32::from_black_alpha(SHADOW_ALPHA),
        );
        ui.painter()
            .circle_filled(knob_center, knob_r, Color32::WHITE);
        ui.painter()
            .circle_stroke(knob_center, knob_r, theme.hairline_stroke());
        // Anillo de foco (WCAG 2.4.7 / 2.4.13). El control se dibuja a mano
        // con `allocate_exact_size`, así que —igual que el rail y el botón de
        // Configuración— no hereda ningún indicador de egui: sin esto, un
        // usuario de teclado no tenía forma de saber dónde estaba.
        if response.has_focus() {
            theme.paint_focus_ring(ui.painter(), rect);
        }
    }
    // `Sense::click` solo no responde al teclado: el switch se activaba
    // únicamente con el puntero (WCAG 2.1.1).
    response.clicked()
        || (response.has_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space)))
}

/// Ritmo escandinavo de menú: ítems compactos y padding de botón consistente.
/// El `item_spacing` global (16 px) es para el lienzo; dentro de un menú separa
/// de más y rompe la lectura vertical.
fn menu_rhythm(ui: &mut egui::Ui) {
    ui.spacing_mut().item_spacing.y = SPACE_XS;
    ui.spacing_mut().item_spacing.x = SPACE_SM;
    ui.spacing_mut().button_padding = egui::vec2(SPACE_SM, SPACE_XS);
}

// Drill-in móvil: None = popup desktop; Some(profundidad) = página
// fullscreen (0 contenido directo, ≥1 acordeón inline, sin popups).
thread_local! {
    static MOBILE_DRILL: std::cell::Cell<Option<usize>> =
        const { std::cell::Cell::new(None) };
}
fn mobile_drill_depth() -> Option<usize> {
    MOBILE_DRILL.with(|d| d.get())
}
/// `ui.menu_button` con el ritmo de menú ya aplicado: todos los menús de la
/// barra comparten altura de ítem, padding y separación.
fn menu_button(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    if let Some(depth) = mobile_drill_depth() {
        MOBILE_DRILL.with(|d| d.set(Some(depth + 1)));
        if depth == 0 {
            // Página de sección: contenido directo (el título ya está en
            // el header con ← volver).
            add_contents(ui);
        } else {
            // Anidado (Exportar): acordeón inline COLAPSADO por defecto.
            // Pulido 2026-10-17 (directiva usuario: las filas quedaban
            // corridas a la derecha y desbordaban el borde en angosto):
            // sin `ui.indent` — las filas van flush con los mismos márgenes
            // simétricos del frame que las filas raíz; la jerarquía la da
            // el header `mobile_menu_row` (52 px + AT), no sangría.
            let theme = current_theme(ui.ctx());
            ui.add_space(SPACE_XS);
            if mobile_menu_row(ui, theme, title) {
                MOBILE_SECTION_OPEN.with(|c| c.set(!c.get()));
            }
            if MOBILE_SECTION_OPEN.with(|c| c.get()) {
                ui.add_space(SPACE_XS);
                add_contents(ui);
            }
        }
        MOBILE_DRILL.with(|d| d.set(Some(depth)));
        return;
    }
    ui.menu_button(title, |ui| {
        menu_rhythm(ui);
        add_contents(ui);
    });
}
/// Cierra el drawer móvil (drill): pone los dos flags a su estado raíz.
/// En desktop es harmless (el `Area` solo existe bajo <740 px y el flag ya
/// se resetea en `draw_top_bar`). Toggles y Undo/Redo NO lo llaman
/// (multi-toggle y multi-deshacer con un dedo quedan abiertos a propósito);
/// acciones de navegación SÍ (si no, el drawer fullscreen tapa el
/// resultado). X, Esc y hamburguesa pasan por acá (cierre unificado, sin
/// caminos paralelos que dejen el acordeón colgado). Llamar ANTES de abrir
/// diálogos nativos para que el modal quede solo en pantalla.
fn close_mobile_drawer(app: &mut GrafitoApp) {
    app.mobile_menu_open = false;
    app.mobile_menu_page = MobileMenuPage::Root;
    MOBILE_SECTION_OPEN.with(|c| c.set(false));
}
/// Páginas del drawer móvil: raíz con 7 filas + una por sección.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum MobileMenuPage {
    #[default]
    Root,
    Archivo,
    Editar,
    Vista,
    Perspectivas,
    Herramientas,
    Paneles,
    Ayuda,
}

fn draw_file_menu(ui: &mut egui::Ui, app: &mut GrafitoApp) {
    use grafito_ui::i18n::t;
    let locale = app.config_locale();
    menu_button(ui, t("menu.file.title", locale), |ui| {
        if drill_action(ui, t("menu.file.new", locale), "Ctrl+N", false, true, None) {
            close_mobile_drawer(app);
            app.handle_file_command(FileCommand::New);
            ui.close_menu();
        }
        if drill_action(ui, t("menu.file.open", locale), "Ctrl+O", false, true, None) {
            close_mobile_drawer(app);
            app.handle_file_command(FileCommand::Open);
            ui.close_menu();
        }
        drill_separator(ui);
        if drill_action(ui, t("menu.file.save", locale), "Ctrl+S", false, true, None) {
            close_mobile_drawer(app);
            app.handle_file_command(FileCommand::Save);
            ui.close_menu();
        }
        if drill_action(
            ui,
            t("menu.file.save_as", locale),
            "Ctrl+Shift+S",
            false,
            true,
            None,
        ) {
            close_mobile_drawer(app);
            app.handle_file_command(FileCommand::SaveAs);
            ui.close_menu();
        }
        drill_separator(ui);
        if drill_action(ui, t("menu.file.import_ggb", locale), "", false, true, None) {
            close_mobile_drawer(app);
            app.choose_and_import_ggb(ui.ctx());
            ui.close_menu();
        }
        menu_button(ui, t("menu.file.export", locale), |ui| {
            if drill_action(ui, t("menu.file.export_ggb", locale), "", false, true, None) {
                close_mobile_drawer(app);
                app.choose_and_export_ggb(ui.ctx());
                ui.close_menu();
            }
            for (label, format) in [
                (
                    t("menu.file.export_svg", locale),
                    crate::export::ExportFormat::Svg,
                ),
                (
                    t("menu.file.export_png", locale),
                    crate::export::ExportFormat::Png,
                ),
                (
                    t("menu.file.export_tikz", locale),
                    crate::export::ExportFormat::Tikz,
                ),
            ] {
                // W3 — "…" = abre diálogo (elegís carpeta y nombre ahí).
                if drill_action(
                    ui,
                    label,
                    "",
                    false,
                    true,
                    Some(t("menu.file.export_pick_dir", locale)),
                ) {
                    close_mobile_drawer(app);
                    app.export_with_dialog(format, Some(ui.ctx()));
                    ui.close_menu();
                }
            }
            // PDF multipágina: una página por hoja de pizarra con contenido.
            if drill_action(
                ui,
                t("menu.file.export_pdf", locale),
                "",
                false,
                true,
                Some(t("menu.file.export_pdf_hint", locale)),
            ) {
                close_mobile_drawer(app);
                app.choose_and_export_pdf(ui.ctx());
                ui.close_menu();
            }
            ui.add_space(SPACE_XS);
            // Lección Markdown: recetas re-ejecutables para el vault
            // Obsidian (```grafito``` + tabla de sliders, KB de texto).
            if drill_action(
                ui,
                t("menu.file.export_lesson", locale),
                "",
                false,
                true,
                Some(t("menu.file.export_lesson_hint", locale)),
            ) {
                close_mobile_drawer(app);
                app.choose_and_export_lesson_md(ui.ctx());
                ui.close_menu();
            }
            // Lección → vault: 1 clic al vault Obsidian (timestamp UTC).
            if drill_action(
                ui,
                t("menu.file.lesson_vault", locale),
                "",
                false,
                true,
                Some(t("menu.file.lesson_vault_hint", locale)),
            ) {
                close_mobile_drawer(app);
                app.export_lesson_to_vault(ui.ctx());
                ui.close_menu();
            }
            drill_separator(ui);
            // W3 — export con salida: revela la carpeta de la última exportación.
            let reveal_hover = if app.last_export_dir.is_some() {
                t("menu.file.reveal_hint", locale).to_string()
            } else {
                t("menu.file.reveal_empty", locale).to_string()
            };
            if drill_action(
                ui,
                t("menu.file.reveal", locale),
                "",
                false,
                app.last_export_dir.is_some(),
                Some(&reveal_hover),
            ) {
                close_mobile_drawer(app);
                app.reveal_last_export();
                ui.close_menu();
            }
        });
        drill_separator(ui);
        if drill_action(ui, t("menu.file.exit", locale), "", false, true, None) {
            close_mobile_drawer(app);
            app.handle_file_command(FileCommand::Exit);
            ui.close_menu();
        }
    });
}

fn draw_edit_menu(ui: &mut egui::Ui, app: &mut GrafitoApp) {
    use grafito_ui::i18n::t;
    let locale = app.config_locale();
    menu_button(ui, t("menu.edit.title", locale), |ui| {
        let undo_len = app.undo_stack.len();
        let redo_len = app.redo_stack.len();
        // En drill el hover no existe: el paso va inline (`Deshacer · 3/50`).
        // En desktop el ítem queda limpio y el paso vive en el hover.
        // Undo/Redo NO cierran el drawer (multi-deshacer con un dedo).
        let drill_inline = mobile_drill_depth().is_some();
        let max_undo = crate::app::MAX_UNDO;
        let undo_label = if drill_inline {
            format!("{} · {undo_len}/{max_undo}", t("menu.edit.undo", locale))
        } else {
            t("menu.edit.undo", locale).to_string()
        };
        let redo_label = if drill_inline {
            format!("{} · {redo_len}/{max_undo}", t("menu.edit.redo", locale))
        } else {
            t("menu.edit.redo", locale).to_string()
        };
        let undo_hover = t("menu.edit.steps", locale)
            .replace("{n}", &undo_len.to_string())
            .replace("{max}", &crate::app::MAX_UNDO.to_string());
        if drill_action(
            ui,
            &undo_label,
            "Ctrl+Z",
            false,
            !app.undo_stack.is_empty(),
            Some(&undo_hover),
        ) {
            app.undo();
            ui.close_menu();
        }
        let redo_hover = t("menu.edit.steps", locale)
            .replace("{n}", &redo_len.to_string())
            .replace("{max}", &crate::app::MAX_UNDO.to_string());
        if drill_action(
            ui,
            &redo_label,
            "Ctrl+Y",
            false,
            !app.redo_stack.is_empty(),
            Some(&redo_hover),
        ) {
            app.redo();
            ui.close_menu();
        }
        drill_separator(ui);
        // Eliminar gateado: sin selección el botón se deshabilita (antes
        // era un tap muerto que cerraba el menú sin hacer nada).
        if drill_action(
            ui,
            t("menu.edit.delete", locale),
            "Supr",
            false,
            app.selected_object.is_some(),
            None,
        ) {
            app.delete_selected();
            close_mobile_drawer(app);
            ui.close_menu();
        }
    });
}

fn draw_view_menu(ui: &mut egui::Ui, app: &mut GrafitoApp) {
    use grafito_ui::i18n::t;
    let locale = app.config_locale();
    menu_button(ui, t("menu.view.title", locale), |ui| {
        grafito_ui::assistant::drill_switch(
            ui,
            t("menu.view.grid", locale),
            "",
            &mut app.show_grid,
            true,
        );
        // Ctrl+T como `shortcut_text` real (antes texto literal en la etiqueta).
        // El toggle muta `dark_mode` igual que `shortcuts.rs::Ctrl+T`.
        // En drill la fila nativa marca con barra + check (patrón perspectiva).
        if drill_action(
            ui,
            t("menu.view.dark_mode", locale),
            "Ctrl+T",
            app.dark_mode,
            true,
            None,
        ) {
            app.dark_mode = !app.dark_mode;
            if app.dark_mode {
                DARK.apply(ui.ctx());
            } else {
                LIGHT.apply(ui.ctx());
            }
            app.save_app_config();
            ui.close_menu();
        }
        // G snap como `shortcut_text` real (handler en `shortcuts.rs:146`).
        if drill_action(
            ui,
            t("menu.view.snap", locale),
            "G",
            app.snap_to_grid,
            true,
            None,
        ) {
            app.snap_to_grid = !app.snap_to_grid;
            app.snap_config.snap_to_grid = app.snap_to_grid;
            ui.close_menu();
        }
        drill_separator(ui);
        // D2: salida de examen con confirmación (nunca directo).
        if mobile_drill_depth().is_some() {
            ui.add_space(SPACE_SM);
        }
        app.exam_mode_checkbox(ui);
        if mobile_drill_depth().is_some() {
            ui.add_space(SPACE_SM);
        }
        // Shift+L/K/J como `shortcut_text` real (handlers `shortcuts.rs:133-144`).
        let (x_log, y_log) = {
            let v = app.document.view();
            (v.x_log, v.y_log)
        };
        if drill_action(
            ui,
            t("menu.view.x_log", locale),
            "Shift+L",
            x_log,
            true,
            None,
        ) {
            app.document.view_mut().x_log = !x_log;
            ui.close_menu();
        }
        if drill_action(
            ui,
            t("menu.view.y_log", locale),
            "Shift+K",
            y_log,
            true,
            None,
        ) {
            app.document.view_mut().y_log = !y_log;
            ui.close_menu();
        }
        // Toggle real: marcado solo si AMBOS ejes están en log (antes
        // fantasma, sin estado visible aunque mutaba los dos).
        if drill_action(
            ui,
            t("menu.view.both_log", locale),
            "Shift+J",
            x_log && y_log,
            true,
            None,
        ) {
            let both = !x_log || !y_log;
            let v = app.document.view_mut();
            v.x_log = both;
            v.y_log = both;
            ui.close_menu();
        }
        drill_separator(ui);
        grafito_ui::assistant::drill_switch(
            ui,
            t("menu.view.gpu", locale),
            "",
            &mut app.use_gpu,
            true,
        );
    });
}

/// Fila nativa de perspectiva en drill: 52 px full-width, título 17 px,
/// atajo a la derecha en terciario 11 px, barra de acento + check si activa.
/// Sin `Button` de egui: el popup desktop conserva sus botones intactos.
fn perspective_mobile_row(
    ui: &mut egui::Ui,
    theme: &grafito_ui::theme::Theme,
    title: &str,
    shortcut: &str,
    active: bool,
) -> bool {
    let w = ui.available_width().max(40.0);
    // M3 lista una línea: 56 px (antes 52: quedaba 4 px bajo el estándar
    // nativo y el ritmo se veía comprimido en Android).
    let h = 56.0;
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, h), egui::Sense::click());
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    resp.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, true, title.to_owned())
    });
    if ui.is_rect_visible(rect) {
        if active {
            ui.painter()
                .rect_filled(rect, RADIUS_SM, theme.button_hover);
            // Barra de acento + check de 2 trazos a la izquierda.
            ui.painter().rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(rect.min.x, rect.min.y + 8.0),
                    egui::vec2(3.0, h - 16.0),
                ),
                RADIUS_SM,
                theme.accent,
            );
        } else if resp.hovered() || resp.has_focus() {
            ui.painter()
                .rect_filled(rect, RADIUS_SM, theme.button_hover);
        }
        if resp.has_focus() {
            theme.paint_focus_ring(ui.painter(), rect);
        }
        let cy = rect.center().y;
        let text_x = rect.min.x + SPACE_LG + if active { 24.0 } else { 0.0 };
        if active {
            let cx0 = rect.min.x + SPACE_LG;
            for (a, b) in [
                (egui::pos2(cx0, cy - 1.0), egui::pos2(cx0 + 5.0, cy + 4.0)),
                (
                    egui::pos2(cx0 + 5.0, cy + 4.0),
                    egui::pos2(cx0 + 13.0, cy - 6.0),
                ),
            ] {
                ui.painter()
                    .line_segment([a, b], egui::Stroke::new(2.0, theme.accent));
            }
        }
        // Elide: el título largo pisaba el atajo y el borde derecho.
        // (En drill el atajo ya no se pasa: ver llamada con "".)
        // Reserva 96 solo si hay atajo: en drill con "" el elide era
        // prematuro (~164 px a 320 en vez de ~260).
        let trail_reserve = if shortcut.is_empty() { 0.0 } else { 96.0 };
        let shown = drill_fit_label(
            ui.ctx(),
            title,
            w - (text_x - rect.min.x) - SPACE_LG - trail_reserve,
        );
        ui.painter().text(
            egui::pos2(text_x, cy),
            egui::Align2::LEFT_CENTER,
            shown,
            egui::FontId::proportional(MOBILE_MENU_TEXT),
            theme.text_primary,
        );
        if !shortcut.is_empty() {
            ui.painter().text(
                egui::pos2(rect.max.x - SPACE_LG, cy),
                egui::Align2::RIGHT_CENTER,
                shortcut,
                egui::FontId::proportional(TYPE_XS),
                theme.text_tertiary,
            );
        }
    }
    resp.clicked()
        || (resp.has_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space)))
}
fn draw_perspectives_menu(ui: &mut egui::Ui, app: &mut GrafitoApp) {
    use grafito_ui::i18n::t;
    let locale = app.config_locale();
    menu_button(ui, t("menu.persp.title", locale), |ui| {
        let theme = current_theme(ui.ctx());
        // Ancho derivado del contenido, no un literal de 252 px: era el único
        // `set_min_width` de toda la barra y dejaba el popup más ancho que sus
        // vecinos (y bastante más ancho cuando colgaba del menú "Más").
        // En desktop el popup necesita ancho mínimo; en drill el contenido
        // va a ancho completo y 240 px fijos constreñirían el drawer a 320.
        if mobile_drill_depth().is_none() {
            ui.set_min_width(MENU_PANEL_MIN_W);
        }
        // Encabezado de sección: misma jerarquía que el inspector
        // (etiqueta quiet + lista), en vez de un muro de radios sueltos.
        ui.add_space(SPACE_XS);
        ui.label(
            egui::RichText::new(t("menu.persp.heading", locale))
                .color(theme.text_tertiary)
                .size(TYPE_XS)
                .strong(),
        );
        ui.add_space(SPACE_XS);
        // P1a-1 VIBLE: en examen el selector se deshabilita (lockdown sin bypass).
        ui.add_enabled_ui(!app.exam_mode, |ui| {
            // El ítem activo se marca con `.selected()` en vez de mutar un
            // `radio_value`: así el estado vive en el botón y el atajo puede
            // ir por `shortcut_text`, alineado con el resto de los menús.
            // `selected` arranca en la actual y el click la mueve: antes
            // la Response se descartaba y el `if` de abajo jamás disparaba
            // (click muerto en todas las perspectivas).
            let mut selected = app.perspective;
            // Drill: filas nativas 52 px (título + atajo + check). Desktop:
            // botones con popup (intacto). El gate de examen envuelve ambos.
            if mobile_drill_depth().is_some() {
                for perspective in Perspective::ALL {
                    // El renglón que separa matemática de física.
                    if perspective == crate::Perspective::Fisica {
                        ui.add_space(SPACE_XS);
                        ui.separator();
                        ui.add_space(SPACE_XS);
                    }
                    // Drill sin atajo: en táctil no hay teclado físico y
                    // ~90 px de `Ctrl+Shift+…` comprimían el título (en
                    // desktop el atajo sigue por `shortcut_text`).
                    if perspective_mobile_row(
                        ui,
                        theme,
                        perspective.title(),
                        "",
                        selected == perspective,
                    ) {
                        selected = perspective;
                    }
                }
            } else {
                for perspective in Perspective::ALL {
                    // El renglón que separa matemática de física: Física va
                    // después del bloque matemático, con separador propio.
                    if perspective == crate::Perspective::Fisica {
                        ui.add_space(SPACE_XS);
                        ui.separator();
                        ui.add_space(SPACE_XS);
                    }
                    // El atajo va por `shortcut_text`, no embebido en el label con
                    // doble espacio: así la columna de atajos queda alineada como
                    // en el resto de los menús.
                    let mut item = ui.add(
                        egui::Button::new(perspective.title().to_string())
                            .selected(selected == perspective)
                            .shortcut_text(format!("Ctrl+Shift+{}", perspective.shortcut_label())),
                    );
                    // Sin hover eco: el título ya se lee en el botón. Solo en
                    // examen el hover avisa del bloqueo (texto distinto).
                    if app.exam_mode {
                        item = item.on_hover_text(t("menu.persp.locked", locale));
                    }
                    if item.clicked() {
                        selected = perspective;
                    }
                }
            }
            if selected != app.perspective {
                // `set_perspective` también gatea (defensa en profundidad).
                let _ = app.try_set_perspective(selected);
                // Cierra el popup desktop Y el drawer móvil (antes el menú
                // quedaba abierto sobre la perspectiva nueva).
                ui.close_menu();
                close_mobile_drawer(app);
            }
        });
        if app.exam_mode {
            // Aviso del lockdown como tarjeta quiet (antes: párrafo suelto
            // de tres líneas sin jerarquía dentro del menú).
            ui.add_space(SPACE_SM);
            egui::Frame::none()
                .fill(theme.input_bg)
                .stroke(theme.hairline_stroke())
                .rounding(egui::Rounding::same(RADIUS_SM))
                .inner_margin(egui::Margin::same(SPACE_SM))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.label(
                        egui::RichText::new(t("menu.persp.exam_mode", locale))
                            .color(theme.text_primary)
                            .size(TYPE_XS)
                            .strong(),
                    );
                    ui.add_space(SPACE_XS);
                    ui.label(
                        egui::RichText::new(t("menu.persp.exam_hint", locale))
                            .color(theme.text_tertiary)
                            .size(TYPE_XS),
                    );
                });
        }
        ui.add_space(SPACE_SM);
        ui.separator();
        ui.add_space(SPACE_SM);
        if ui
            .add_sized(
                [
                    ui.available_width(),
                    // En drill a 44 (28 < piso táctil); desktop intacto.
                    if mobile_drill_depth().is_some() {
                        MENU_ACTION_H.max(HIT_TARGET_AULA)
                    } else {
                        MENU_ACTION_H
                    },
                ],
                // Sin `rounding` propio: el radio del tema se aplica a todos los
                // ítems por igual. Este único ítem con `RADIUS_SM` hardcodeado
                // se veía como un escalón dentro del propio menú.
                egui::Button::new(
                    egui::RichText::new(t("menu.persp.load_example", locale)).size(TYPE_SM),
                ),
            )
            .on_hover_text(t("menu.persp.load_example_hint", locale))
            .clicked()
        {
            // Siempre agrega encima: el staged es append seguro + snapshot
            // (deshacer lo revierte). La guarda `object_count()==0` dejaba
            // al alumno con 1 punto en un canvas casi vacío sin laboratorio.
            match app.load_perspective_examples(app.perspective) {
                Ok(()) => app.notify(
                    t("shell.example_loaded", locale),
                    grafito_ui::toast::ToastKind::Success,
                ),
                Err(error) => app.handle_command_outcome(
                    grafito_command::commands::CommandOutcome::Error(error),
                    ui.ctx().input(|input| input.time),
                    // B-A188: reutiliza `menu.persp.load_example`
                    // (misma acción que el botón, 0 keys nuevas).
                    t("menu.persp.load_example", locale),
                ),
            }
            close_mobile_drawer(app);
            ui.close_menu();
        }
    });
}

fn draw_tools_menu(ui: &mut egui::Ui, app: &mut GrafitoApp) {
    use grafito_ui::i18n::t;
    let locale = app.config_locale();
    menu_button(ui, t("menu.tools.title", locale), |ui| {
        // Atajos verificados (`shortcuts.rs`): cada entrada muestra su
        // `shortcut_text` real y dispara la misma mutación que el handler.
        // F7 está libre a propósito (hueco entre F6 Función y F8 Esfera 3D):
        // no hay herramienta asignada, se reserva para no romper la serie F1-F6/F8-F9.
        // Ctrl+Y es dual: sin Shift es Rehacer (`CtrlYShortcut::Redo`),
        // con Shift es herramienta YIntercept (`shortcuts.rs:28-37`);
        // el menú Editar muestra "Ctrl+Y" para Rehacer (ver `ui.rs:295`).
        if drill_action(
            ui,
            t("menu.tools.analyze", locale),
            "Ctrl+A",
            false,
            true,
            None,
        ) {
            close_mobile_drawer(app);
            app.current_tool = Tool::Analyze;
            app.tool_ghost = None;
            app.reset_tool_input();
            ui.close_menu();
        }
        if drill_action(
            ui,
            t("menu.tools.palette", locale),
            "Ctrl+K",
            false,
            true,
            None,
        ) {
            close_mobile_drawer(app);
            app.command_palette.open_palette();
            ui.close_menu();
        }
        drill_separator(ui);
        if drill_action(
            ui,
            t("menu.tools.pencil", locale),
            "Ctrl+P",
            false,
            true,
            None,
        ) {
            close_mobile_drawer(app);
            app.current_tool = Tool::Pencil;
            app.tool_ghost = None;
            app.reset_tool_input();
            ui.close_menu();
        }
        if drill_action(
            ui,
            t("menu.tools.eraser", locale),
            "Ctrl+E",
            false,
            true,
            None,
        ) {
            close_mobile_drawer(app);
            app.current_tool = Tool::Eraser;
            app.tool_ghost = None;
            app.reset_tool_input();
            ui.close_menu();
        }
        drill_separator(ui);
        // Switches unificados a la derecha (`drill_switch`, pill 44×24):
        // la caja del checkbox a la izquierda quedaba enana entre filas
        // de 56 px (S6b: también Vista, inspector, Aula y Complex).
        if grafito_ui::assistant::drill_switch(
            ui,
            t("menu.tools.keyboard", locale),
            "",
            &mut app.keyboard_visible,
            true,
        ) {
            // Elección explícita: las perspectivas dejan de imponer su default.
            app.keyboard_visible_explicit = true;
            if !app.keyboard_visible {
                app.keyboard_expanded = false;
            }
        }
        if grafito_ui::assistant::drill_switch(
            ui,
            t("menu.tools.assistant", locale),
            "",
            &mut app.assistant_visible,
            true,
        ) && app.assistant_visible
        {
            app.open_assistant_workspace();
        }
        drill_separator(ui);
        let supported = crate::app::trig_animation_supported(app.current_view);
        let mut trig_visible = app.show_trig_animation;
        let hint = if supported {
            t("menu.tools.trig_hint", locale)
        } else {
            t("menu.tools.trig_unavailable", locale)
        };
        // El hint y el enabled viajan dentro del switch (antes los ataba
        // `add_enabled_ui` + `on_hover_text` alrededor del checkbox).
        let trig_changed = grafito_ui::assistant::drill_switch(
            ui,
            t("menu.tools.trig", locale),
            hint,
            &mut trig_visible,
            supported,
        );
        if trig_changed {
            app.set_trig_animation_visible(trig_visible);
            ui.close_menu();
        }
        drill_separator(ui);
        if drill_action(
            ui,
            t("menu.tools.save_custom", locale),
            "",
            false,
            true,
            None,
        ) {
            close_mobile_drawer(app);
            app.show_custom_tool_dialog = true;
            app.custom_tool_name.clear();
            ui.close_menu();
        }
        if drill_action(
            ui,
            t("menu.tools.load_custom", locale),
            "",
            false,
            true,
            None,
        ) {
            close_mobile_drawer(app);
            app.load_custom_tool_from_dialog();
            ui.close_menu();
        }
        drill_separator(ui);
        if drill_action(ui, t("menu.tools.colab", locale), "", false, true, None) {
            close_mobile_drawer(app);
            app.show_colab_window = true;
            ui.close_menu();
        }
    });
}

fn draw_panels_menu(ui: &mut egui::Ui, app: &mut GrafitoApp) {
    use grafito_ui::i18n::t;
    let locale = app.config_locale();
    menu_button(ui, t("menu.panels.title", locale), |ui| {
        // 5 tabs de primera clase (regla visible ⇒ botón): cada entrada abre
        // su tab. Datos y Prob. tienen casa propia; las perspectivas
        // Estadística/Probabilidad/Análisis de datos solo eligen el tab
        // inicial (ver `LeftPanelContent::default_sidebar_tab`).
        // Vault ELIMINADO 2026-10-06 (directiva usuario): fuera del menú paneles.
        for (tab, label) in [
            (0, t("menu.panels.algebra", locale)),
            (1, t("menu.panels.tools", locale)),
            (2, t("menu.panels.data", locale)),
            (3, t("menu.panels.prob", locale)),
            (4, t("menu.panels.view", locale)),
        ] {
            // `selected` honesto: si el Nivel 0 (<480) o el dock lo ocultan,
            // no se pinta como abierto. En drill la fila nativa marca con
            // barra + check (patrón perspectiva).
            let drawer_visible =
                app.compact_drawer_open && !mobile_level0(ui.ctx().screen_rect().width());
            let selected = drawer_visible && app.sidebar_tab == tab;
            if drill_action(ui, label, "", selected, true, None) {
                close_mobile_drawer(app);
                app.sidebar_tab = tab;
                app.compact_drawer_open = true;
                app.left_drawer_open = true;
                ui.close_menu();
            }
        }
        // Aula: estaba invisible hasta saber dónde está el toggle.
        // Esta fila lo activa (opt-in) y abre su tab: mismo panel,
        // otro acceso. Reusa `rail.tab_aula` (0 keys nuevas).
        drill_separator(ui);
        {
            let drawer_visible =
                app.compact_drawer_open && !mobile_level0(ui.ctx().screen_rect().width());
            let selected = drawer_visible && app.sidebar_tab == GrafitoApp::AULA_TAB_INDEX;
            if drill_action(ui, t("rail.tab_aula", locale), "", selected, true, None) {
                close_mobile_drawer(app);
                app.classroom.set_opt_in(true);
                app.sidebar_tab = GrafitoApp::AULA_TAB_INDEX;
                app.compact_drawer_open = true;
                app.left_drawer_open = true;
                ui.close_menu();
            }
        }
        if app.perspective == Perspective::Geometry3D {
            drill_separator(ui);
            let inspector_selected = app.compact_geometry_utility_open
                && app.workspace_dock_tab == WorkspaceDockTab::Inspector;
            if drill_action(
                ui,
                t("menu.panels.inspector_3d", locale),
                "",
                inspector_selected,
                true,
                None,
            ) {
                close_mobile_drawer(app);
                app.workspace_dock_tab = WorkspaceDockTab::Inspector;
                app.compact_geometry_utility_open = true;
                ui.close_menu();
            }
            let assistant_selected = app.compact_geometry_utility_open
                && app.workspace_dock_tab == WorkspaceDockTab::Assistant;
            if drill_action(
                ui,
                t("menu.panels.assistant_3d", locale),
                "",
                assistant_selected,
                true,
                None,
            ) {
                close_mobile_drawer(app);
                app.open_assistant_workspace();
                ui.close_menu();
            }
            if app.compact_geometry_utility_open {
                drill_separator(ui);
                if drill_action(ui, t("menu.panels.hide_3d", locale), "", false, true, None) {
                    close_mobile_drawer(app);
                    app.compact_geometry_utility_open = false;
                    ui.close_menu();
                }
            }
        }
        if app.compact_drawer_open {
            drill_separator(ui);
            if drill_action(ui, t("menu.panels.hide", locale), "", false, true, None) {
                close_mobile_drawer(app);
                // Ocultar cierra AMBOS flags (antes `left_drawer_open`
                // quedaba colgado y el drawer volvía solo).
                app.compact_drawer_open = false;
                app.left_drawer_open = false;
                ui.close_menu();
            }
        }
    });
}

fn draw_help_menu(ui: &mut egui::Ui, app: &mut GrafitoApp) {
    use grafito_ui::i18n::t;
    let locale = app.config_locale();
    menu_button(ui, t("menu.help.title", locale), |ui| {
        let version = env!("CARGO_PKG_VERSION");
        if drill_action(
            ui,
            &t("menu.help.about", locale).replace("{version}", version),
            "",
            false,
            true,
            None,
        ) {
            close_mobile_drawer(app);
            app.show_about = true;
            ui.close_menu();
        }
        // Ola 0.6: hoja de atajos (claves `cheat.*`, antes sin caller).
        if drill_action(ui, t("menu.help.shortcuts", locale), "", false, true, None) {
            close_mobile_drawer(app);
            app.show_cheat_sheet = true;
            ui.close_menu();
        }
        // F1: re-entry de la bienvenida first-run (ver Ayuda → Bienvenida).
        if drill_action(ui, t("menu.help.welcome", locale), "", false, true, None) {
            close_mobile_drawer(app);
            app.show_onboarding = true;
            app.welcome_page = 0;
            app.welcome_dont_show_again = false;
            ui.close_menu();
        }
        drill_separator(ui);
        // O2 i18n: selector segmentado con persistencia (Piel pura: el
        // selector solo setea el flag local; `set_locale` escribe afuera).
        ui.label(
            egui::RichText::new(t("menu.help.language", locale))
                .color(current_theme(ui.ctx()).text_tertiary)
                .size(TYPE_XS)
                .strong(),
        );
        ui.add_space(SPACE_XS);
        drill_locale_row(ui, app);
    });
}

/// Top bar plegable móvil <740 (M3 top app bar compacta).
///
/// Una fila: hamburguesa (todo lo de la barra desktop en un popup) +
/// marca + buscar (paleta). Sin menús visibles ni iconos sueltos: a
/// 430 px la barra completa tapaba el canvas. El popup reusa los mismos
/// `draw_*_menu` + toggles del desktop: nada se pierde, solo se pliega.
/// Desktop ≥740 intacto (otra rama).
/// Fila rápida del drawer móvil (pizarra / tema / ajustes): 44 px táctil,
/// centrada. Sin strings nuevos (hovers heredados del desktop).
fn draw_mobile_quick_bar(
    ui: &mut egui::Ui,
    app: &mut GrafitoApp,
    accent: egui::Color32,
    theme: &grafito_ui::theme::Theme,
    locale: grafito_ui::i18n::Locale,
) {
    ui.vertical_centered(|ui| {
        // main_justify: el horizontal ocupa todo el ancho pero su
        // contenido va centrado (sin esto quedaba a la izquierda).
        ui.with_layout(
            egui::Layout::left_to_right(egui::Align::Center).with_main_justify(true),
            |ui| {
                // Tema como botón 44 uniforme (el switch quedaba invisible en
                // claro y la fila perdía 2 iconos): luna en claro, sol en oscuro.
                {
                    let (theme_icon, theme_hint) = if app.dark_mode {
                        (
                            Icon::Sun,
                            grafito_ui::i18n::t("shell.theme_to_light", locale),
                        )
                    } else {
                        (
                            Icon::Moon,
                            grafito_ui::i18n::t("shell.theme_to_dark", locale),
                        )
                    };
                    // D8 acento = estado: el tema no tiene activo/inactivo,
                    // va en primario (antes siempre acento, jerarquía invertida).
                    if action_icon_button_sized(
                        ui,
                        theme_icon,
                        theme.text_primary,
                        theme_hint,
                        egui::vec2(44.0, 44.0),
                    )
                    .clicked()
                    {
                        app.dark_mode = !app.dark_mode;
                        if app.dark_mode {
                            DARK.apply(ui.ctx());
                        } else {
                            LIGHT.apply(ui.ctx());
                        }
                        app.save_app_config();
                    }
                }
                if action_icon_button_sized(
                    ui,
                    Icon::Whiteboard,
                    if app.whiteboard_open {
                        accent
                    } else {
                        theme.text_secondary
                    },
                    grafito_ui::i18n::t("center.menu_whiteboard", locale),
                    egui::vec2(44.0, 44.0),
                )
                .clicked()
                {
                    app.whiteboard_open = !app.whiteboard_open;
                }
                {
                    use grafito_ui::i18n::t as shell_t;
                    let is_open = app.assistant.settings_open;
                    if action_icon_button_sized(
                        ui,
                        Icon::Settings,
                        if is_open {
                            accent
                        } else {
                            theme.text_secondary
                        },
                        shell_t("shell.config_btn", locale),
                        egui::vec2(44.0, 44.0),
                    )
                    .clicked()
                    {
                        let will_open = !is_open;
                        app.assistant.settings_open = will_open;
                        app.assistant.config_tab = 0;
                        app.assistant.settings_search.clear();
                        app.assistant.settings_open_show_welcome =
                            !crate::utils::load_config().onboarding_completed;
                        app.show_mascot_config = false;
                    }
                }
            },
        );
    });
}

/// Fila raíz del drawer móvil: 52 px táctil, texto a la izquierda, chevron
/// dibujado a la derecha (sin glifo exótico). Chevron simétrico al texto
/// (pad 16 en ambos lados). Devuelve el clic (puntero o teclado).
/// Elide si el rotulo excede el presupuesto: pintar texto sin clip
/// desbordaba a la derecha en angosto (el painter no recorta).
/// `budget` = ancho util en px desde `text_x`.
fn drill_fit_label(ctx: &egui::Context, label: &str, budget: f32) -> String {
    let font = egui::FontId::proportional(MOBILE_MENU_TEXT);
    // epaint 0.29 pide color aunque solo se mida el ancho: se pasa
    // WHITE (el ancho no depende del color).
    let width_of = |s: &str| {
        ctx.fonts(|f| {
            f.layout_no_wrap(s.to_owned(), font.clone(), egui::Color32::WHITE)
                .size()
                .x
        })
    };
    if budget <= 24.0 {
        return "\u{2026}".to_owned();
    }
    if width_of(label) <= budget {
        return label.to_owned();
    }
    let mut short: String = label.chars().take(12).collect();
    while !short.is_empty() && width_of(&format!("{short}\u{2026}")) > budget {
        short.pop();
    }
    format!("{short}\u{2026}")
}

fn mobile_menu_row(ui: &mut egui::Ui, theme: &grafito_ui::theme::Theme, title: &str) -> bool {
    let w = ui.available_width().max(40.0);
    // M3 lista una línea: 56 px (ver perspective_mobile_row).
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, 56.0), egui::Sense::click());
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    resp.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, true, title.to_owned())
    });
    if ui.is_rect_visible(rect) {
        if resp.hovered() || resp.has_focus() {
            ui.painter()
                .rect_filled(rect, RADIUS_SM, theme.button_hover);
        }
        if resp.has_focus() {
            theme.paint_focus_ring(ui.painter(), rect);
        }
        let cy = rect.center().y;
        // Elide antes de pintar: el painter no recorta y el título largo
        // pisaba el chevron y el borde (desborde derecho en angosto).
        let shown = drill_fit_label(ui.ctx(), title, w - SPACE_LG * 2.0 - 32.0);
        ui.painter().text(
            egui::pos2(rect.min.x + SPACE_LG, cy),
            egui::Align2::LEFT_CENTER,
            shown,
            egui::FontId::proportional(MOBILE_MENU_TEXT),
            theme.text_primary,
        );
        // Chevron › con 2 trazos: punta en max.x-16, simétrico al texto.
        let cx = rect.max.x - SPACE_LG - 7.0;
        for (a, b) in [
            (egui::pos2(cx, cy - 8.0), egui::pos2(cx + 7.0, cy)),
            (egui::pos2(cx + 7.0, cy), egui::pos2(cx, cy + 8.0)),
        ] {
            ui.painter()
                .line_segment([a, b], egui::Stroke::new(2.0, theme.text_secondary));
        }
    }
    resp.clicked()
        || (resp.has_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space)))
}
/// X 44 px dibujada (2 trazos, sin glifo exótico). Devuelve el clic
/// (puntero o teclado). `label` = nombre accesible (sin strings nuevos).
fn mobile_menu_x(ui: &mut egui::Ui, theme: &grafito_ui::theme::Theme, label: &str) -> bool {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(44.0, 44.0), egui::Sense::click());
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    resp.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label.to_owned())
    });
    if ui.is_rect_visible(rect) {
        if resp.has_focus() {
            theme.paint_focus_ring(ui.painter(), rect);
        }
        let pad = 15.0;
        for (a, b) in [
            (
                egui::pos2(rect.min.x + pad, rect.min.y + pad),
                egui::pos2(rect.max.x - pad, rect.max.y - pad),
            ),
            (
                egui::pos2(rect.max.x - pad, rect.min.y + pad),
                egui::pos2(rect.min.x + pad, rect.max.y - pad),
            ),
        ] {
            ui.painter()
                .line_segment([a, b], egui::Stroke::new(2.0, theme.text_primary));
        }
    }
    resp.clicked()
        || (resp.has_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space)))
}
/// Fila de acción del drill (52 px nativa) / botón desktop clásico.
/// En drill: sin frame ni atajo (el `shortcut_text` en móvil es ruido:
/// roba ~90 px y no hay teclado físico), con barra de acento + check si
/// `selected`, AT + anillo de foco + Enter/Espacio. En desktop: `Button`
/// con `shortcut_text` + `selected`, idéntico a hoy. `enabled=false` es
/// deshabilitado visible (texto terciario, sin click). El `hover` solo se
/// aplica en desktop (en táctil no existe hover).
fn drill_action(
    ui: &mut egui::Ui,
    label: &str,
    shortcut: &str,
    selected: bool,
    enabled: bool,
    hover: Option<&str>,
) -> bool {
    if mobile_drill_depth().is_none() {
        let btn = egui::Button::new(label).selected(selected);
        let btn = if shortcut.is_empty() {
            btn
        } else {
            btn.shortcut_text(shortcut)
        };
        let resp = ui.add_enabled(enabled, btn);
        let resp = match hover {
            Some(h) => resp.on_hover_text(h),
            None => resp,
        };
        return resp.clicked();
    }
    let theme = current_theme(ui.ctx());
    let w = ui.available_width().max(40.0);
    // M3 lista una línea: 56 px (ver perspective_mobile_row).
    let h = 56.0;
    let sense = if enabled {
        egui::Sense::click()
    } else {
        egui::Sense::hover()
    };
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, h), sense);
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    resp.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, label.to_owned())
    });
    if ui.is_rect_visible(rect) {
        if selected {
            ui.painter()
                .rect_filled(rect, RADIUS_SM, theme.button_hover);
            ui.painter().rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(rect.min.x, rect.min.y + 8.0),
                    egui::vec2(3.0, h - 16.0),
                ),
                RADIUS_SM,
                theme.accent,
            );
        } else if resp.hovered() || resp.has_focus() {
            ui.painter()
                .rect_filled(rect, RADIUS_SM, theme.button_hover);
        }
        if resp.has_focus() {
            theme.paint_focus_ring(ui.painter(), rect);
        }
        let cy = rect.center().y;
        let text_x = rect.min.x + SPACE_LG + if selected { 24.0 } else { 0.0 };
        if selected {
            let cx0 = rect.min.x + SPACE_LG;
            for (a, b) in [
                (egui::pos2(cx0, cy - 1.0), egui::pos2(cx0 + 5.0, cy + 4.0)),
                (
                    egui::pos2(cx0 + 5.0, cy + 4.0),
                    egui::pos2(cx0 + 13.0, cy - 6.0),
                ),
            ] {
                ui.painter()
                    .line_segment([a, b], egui::Stroke::new(2.0, theme.accent));
            }
        }
        // Elide: sin clip el rótulo largo salía por la derecha.
        let shown = drill_fit_label(ui.ctx(), label, w - (text_x - rect.min.x) - SPACE_LG);
        ui.painter().text(
            egui::pos2(text_x, cy),
            egui::Align2::LEFT_CENTER,
            shown,
            egui::FontId::proportional(MOBILE_MENU_TEXT),
            if enabled {
                theme.text_primary
            } else {
                theme.text_tertiary
            },
        );
    }
    resp.clicked()
        || (enabled
            && resp.has_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space)))
}
/// Separador con aire en drill (el `separator()` desnudo quedaba pegado a
/// los pills); en desktop, separador clásico sin aire.
fn drill_separator(ui: &mut egui::Ui) {
    if mobile_drill_depth().is_some() {
        ui.add_space(SPACE_SM);
        ui.separator();
        ui.add_space(SPACE_SM);
    } else {
        ui.separator();
    }
}
/// Selector de idioma 44 px en drill (el segmentado de 36×24 desborda a
/// 320 px y queda bajo el mínimo táctil): rejilla envuelta con botones
/// 64×44 nativos (AT + teclado de egui incluidos). Códigos de 2 letras
/// (universales, 0 keys nuevas). Desktop intacto.
fn drill_locale_row(ui: &mut egui::Ui, app: &mut GrafitoApp) {
    use grafito_ui::i18n::Locale;
    if mobile_drill_depth().is_none() {
        let mut loc = app.config_locale();
        if grafito_ui::toolbar::locale_selector(ui, &mut loc).changed() {
            app.set_locale(loc);
        }
        return;
    }
    let current = app.config_locale();
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(SPACE_SM, SPACE_SM);
        for (value, short) in [
            (Locale::Es, "ES"),
            (Locale::En, "EN"),
            (Locale::Pt, "PT"),
            (Locale::It, "IT"),
            (Locale::Fr, "FR"),
            (Locale::De, "DE"),
        ] {
            if ui
                .add_sized(
                    egui::vec2(64.0, 44.0),
                    egui::Button::new(egui::RichText::new(short).size(MOBILE_MENU_TEXT).strong())
                        .selected(current == value),
                )
                .clicked()
            {
                app.set_locale(value);
            }
        }
    });
}
// Acordeón Exportar colapsado por defecto en drill (11 filas siempre
// expandidas = la mitad del scroll a 320 px). Desktop intacto.
thread_local! {
    static MOBILE_SECTION_OPEN: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
/// Título de sección del drawer (`None` = raíz). Mismas keys del desktop.
fn mobile_menu_section_title(
    page: MobileMenuPage,
    locale: grafito_ui::i18n::Locale,
) -> Option<&'static str> {
    use grafito_ui::i18n::t as menu_t;
    Some(match page {
        MobileMenuPage::Root => return None,
        MobileMenuPage::Archivo => menu_t("menu.file.title", locale),
        MobileMenuPage::Editar => menu_t("menu.edit.title", locale),
        MobileMenuPage::Vista => menu_t("menu.view.title", locale),
        MobileMenuPage::Perspectivas => menu_t("menu.persp.title", locale),
        MobileMenuPage::Herramientas => menu_t("menu.tools.title", locale),
        MobileMenuPage::Paneles => menu_t("menu.panels.title", locale),
        MobileMenuPage::Ayuda => menu_t("menu.help.title", locale),
    })
}
/// Header de página de sección: ← 44 px + título centrado + X 44 px.
/// Devuelve (volver, cerrar).
fn mobile_menu_page_header(
    ui: &mut egui::Ui,
    theme: &grafito_ui::theme::Theme,
    title: &str,
    locale: grafito_ui::i18n::Locale,
) -> (bool, bool) {
    let mut back = false;
    let mut closed = false;
    ui.horizontal(|ui| {
        let (rect, resp) = ui.allocate_exact_size(egui::vec2(44.0, 44.0), egui::Sense::click());
        let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
        if ui.is_rect_visible(rect) {
            let cy = rect.center().y;
            let cx = rect.center().x;
            for (a, b) in [
                (egui::pos2(cx + 7.0, cy - 8.0), egui::pos2(cx, cy)),
                (egui::pos2(cx, cy), egui::pos2(cx + 7.0, cy + 8.0)),
            ] {
                ui.painter()
                    .line_segment([a, b], egui::Stroke::new(2.0, theme.text_primary));
            }
        }
        resp.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, true, title.to_owned())
        });
        if ui.is_rect_visible(rect) && resp.has_focus() {
            theme.paint_focus_ring(ui.painter(), rect);
        }
        if resp.clicked()
            || (resp.has_focus()
                && ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space)))
        {
            back = true;
        }
        // Título centrado real: ← (44) a la izquierda y X (44) a la
        // derecha, simétrico por construcción (sin add_space extra: ese
        // corría el centro ~22 px a la derecha).
        let title_w = (ui.available_width() - 44.0).max(40.0);
        ui.allocate_ui_with_layout(
            egui::vec2(title_w, 44.0),
            egui::Layout::centered_and_justified(egui::Direction::TopDown),
            |ui| {
                ui.label(
                    egui::RichText::new(title)
                        .size(grafito_ui::tokens::TYPE_LG)
                        .strong()
                        .color(theme.text_primary),
                );
            },
        );
        if mobile_menu_x(ui, theme, grafito_ui::i18n::t("classroom.close", locale)) {
            closed = true;
        }
    });
    (back, closed)
}
pub(crate) fn draw_mobile_top_bar(
    ui: &mut egui::Ui,
    app: &mut GrafitoApp,
    show_compact_panel_menu: bool,
) {
    let theme = current_theme(ui.ctx());
    let accent = theme.accent;
    let locale = app.config_locale();
    // Hamburguesa dibujada (3 hairlines): sin glifo exótico que la
    // fuente pueda no traer.
    // 40×32 (no 44): la barra mide 40 px y la zona útil es 32; 44 px
    // desbordaría hacia abajo como antes el aire "caído". Táctil justo
    // pero deliberado y pineado: en el drawer todo es 44, acá manda la
    // barra. Foco + Enter/Espacio + nombre (paridad con el switch tema).
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(40.0, 32.0), egui::Sense::click());
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    // Sin eco táctil: la hamburguesa es solo-ícono (sin texto visible), así
    // que el nombre va siempre — es su etiqueta accesible en Nivel 0.
    let narrow_touch = mobile_level0(ui.ctx().screen_rect().width());
    let more_label = grafito_ui::i18n::t("menu.more", locale);
    let resp = if hover_echo_suppressed("", more_label, narrow_touch) {
        resp
    } else {
        resp.on_hover_text(more_label)
    };
    if ui.is_rect_visible(rect) {
        if resp.hovered() || app.mobile_menu_open {
            ui.painter()
                .rect_filled(rect, RADIUS_SM, theme.button_hover);
        }
        if resp.has_focus() {
            theme.paint_focus_ring(ui.painter(), rect);
        }
        let cy = rect.center().y;
        for dy in [-7.0, 0.0, 7.0] {
            ui.painter().line_segment(
                [
                    egui::pos2(rect.min.x + 11.0, cy + dy),
                    egui::pos2(rect.max.x - 11.0, cy + dy),
                ],
                egui::Stroke::new(2.0, theme.text_primary),
            );
        }
    }
    resp.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::Button,
            true,
            grafito_ui::i18n::t("menu.more", locale),
        )
    });
    if resp.clicked()
        || (resp.has_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space)))
    {
        // Cierre unificado: al cerrar resetea página + acordeón (el flip
        // suelto dejaba `MOBILE_SECTION_OPEN` colgado); al abrir, raíz.
        if app.mobile_menu_open {
            close_mobile_drawer(app);
        } else {
            app.mobile_menu_open = true;
            app.mobile_menu_page = MobileMenuPage::Root;
        }
    }
    // Marca invariante (precedente Vault/Math): no se localiza.
    ui.label(
        egui::RichText::new("Grafito")
            .color(theme.text_primary)
            .size(TYPE_BASE)
            .strong(),
    );
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        // 32×32: misma altura que la hamburguesa (32 de zona útil),
        // mismo centro vertical. No 44: no entra en la barra de 40.
        // D8 acento = estado: lupa en acento solo con paleta abierta.
        if action_icon_button_sized(
            ui,
            Icon::Search,
            if app.command_palette.open {
                accent
            } else {
                theme.text_primary
            },
            grafito_ui::i18n::t("palette.search_hint", locale),
            egui::vec2(40.0, 32.0),
        )
        .clicked()
        {
            app.command_palette.open_palette();
        }
    });
    if app.mobile_menu_open {
        // Salida por teclado (el drawer no la tenía: foco atrapado).
        // La paleta tiene prioridad (su Esc corre en su draw); acá Esc
        // sube un nivel y en raíz cierra (back-stack mínimo).
        if !app.command_palette.open && ui.ctx().input(|i| i.key_pressed(egui::Key::Escape)) {
            if app.mobile_menu_page == MobileMenuPage::Root {
                close_mobile_drawer(app);
            } else {
                app.mobile_menu_page = MobileMenuPage::Root;
            }
        }
        // Drawer M3 de pantalla completa: cubre todo, con X propia
        // arriba. Mismos menús del desktop en columna con scroll;
        // sin pantallas nuevas, sin strings nuevos.
        let screen = ui.ctx().screen_rect();
        // Safe-area: el header nacía en y=0, bajo el status de Android
        // (24 px). En desktop safe=0, sin cambio.
        let safe_top = grafito_ui::tokens::safe_inset_top();
        let safe_bottom = grafito_ui::tokens::safe_inset_bottom();
        let drawer_pos = screen.min + egui::vec2(0.0, safe_top);
        let drawer_size = egui::vec2(
            screen.width(),
            (screen.height() - safe_top - safe_bottom).max(200.0),
        );
        egui::Area::new(egui::Id::new("mobile_menu_popup"))
            .fixed_pos(drawer_pos)
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                ui.set_min_size(drawer_size);
                ui.set_max_size(drawer_size);
                egui::Frame::none()
                    .fill(theme.panel_bg)
                    .inner_margin(egui::Margin::same(SPACE_MD))
                    .show(ui, |ui| {
                        // Drawer móvil: cuerpo 17 px en todo el menú (token).
                        ui.style_mut().override_font_id =
                            Some(egui::FontId::proportional(MOBILE_MENU_TEXT));
                        // Un solo header: raíz (Más + X) o sección
                        // (← + título + X). Nada de vista doble.
                        if let Some(section) =
                            mobile_menu_section_title(app.mobile_menu_page, locale)
                        {
                            let (back, closed) =
                                mobile_menu_page_header(ui, theme, section, locale);
                            if back {
                                app.mobile_menu_page = MobileMenuPage::Root;
                            }
                            if closed {
                                // Cierre unificado (misma que la X de raíz).
                                close_mobile_drawer(app);
                            }
                        } else {
                            ui.horizontal(|ui| {
                                // Título centrado real: compensa la X (44) a
                                // la izquierda; el resto va centrado.
                                ui.add_space(44.0);
                                let title_w = (ui.available_width() - 44.0).max(40.0);
                                ui.allocate_ui_with_layout(
                                    egui::vec2(title_w, 44.0),
                                    egui::Layout::centered_and_justified(egui::Direction::TopDown),
                                    |ui| {
                                        ui.label(
                                            egui::RichText::new(grafito_ui::i18n::t(
                                                "menu.more",
                                                locale,
                                            ))
                                            .size(grafito_ui::tokens::TYPE_LG)
                                            .strong()
                                            .color(theme.text_primary),
                                        );
                                    },
                                );
                                if mobile_menu_x(
                                    ui,
                                    theme,
                                    grafito_ui::i18n::t("classroom.close", locale),
                                ) {
                                    // Cierre unificado: resetea página + acordeón
                                    // (el inline dejaba `MOBILE_SECTION_OPEN` colgado).
                                    close_mobile_drawer(app);
                                }
                            });
                        }
                        ui.separator();
                        // Barra rápida ANCLADA abajo (panel inferior exacto,
                        // sin aritmética de spacers): el scroll ocupa lo que
                        // queda y ya no puede empujarla fuera de pantalla.
                        egui::TopBottomPanel::bottom("mobile_drawer_quickbar")
                            .exact_height(44.0 + grafito_ui::tokens::safe_inset_bottom())
                            .show_separator_line(true)
                            .frame(egui::Frame::none())
                            .show_inside(ui, |ui| {
                                draw_mobile_quick_bar(ui, app, accent, theme, locale);
                            });
                        egui::ScrollArea::vertical()
                            .auto_shrink([false; 2])
                            .show(ui, |ui| {
                                // Aire entre filas (8 px, base rítmica).
                                ui.spacing_mut().item_spacing.y = SPACE_SM;
                                match app.mobile_menu_page {
                                    MobileMenuPage::Root => {
                                        // 7 filas táctiles de 52 px: entrar
                                        // es página fullscreen, no popup.
                                        use grafito_ui::i18n::t as menu_t;
                                        for (page, title) in [
                                            (
                                                MobileMenuPage::Archivo,
                                                menu_t("menu.file.title", locale),
                                            ),
                                            (
                                                MobileMenuPage::Editar,
                                                menu_t("menu.edit.title", locale),
                                            ),
                                            (
                                                MobileMenuPage::Vista,
                                                menu_t("menu.view.title", locale),
                                            ),
                                            (
                                                MobileMenuPage::Perspectivas,
                                                menu_t("menu.persp.title", locale),
                                            ),
                                            (
                                                MobileMenuPage::Herramientas,
                                                menu_t("menu.tools.title", locale),
                                            ),
                                            (
                                                MobileMenuPage::Paneles,
                                                menu_t("menu.panels.title", locale),
                                            ),
                                            (
                                                MobileMenuPage::Ayuda,
                                                menu_t("menu.help.title", locale),
                                            ),
                                        ] {
                                            // Paneles solo si el modo lo ofrece
                                            // (gate previo del drawer).
                                            if page == MobileMenuPage::Paneles
                                                && !show_compact_panel_menu
                                            {
                                                continue;
                                            }
                                            if mobile_menu_row(ui, theme, title) {
                                                app.mobile_menu_page = page;
                                            }
                                        }
                                    }
                                    page => {
                                        use grafito_ui::i18n::t as menu_t;
                                        // Título en el header fijo; acá solo el
                                        // contenido (_, draw).
                                        let (_, draw): (&str, fn(&mut egui::Ui, &mut GrafitoApp)) =
                                            match page {
                                                MobileMenuPage::Root => (
                                                    menu_t("menu.file.title", locale),
                                                    draw_file_menu
                                                        as fn(&mut egui::Ui, &mut GrafitoApp),
                                                ),
                                                MobileMenuPage::Archivo => (
                                                    menu_t("menu.file.title", locale),
                                                    draw_file_menu
                                                        as fn(&mut egui::Ui, &mut GrafitoApp),
                                                ),
                                                MobileMenuPage::Editar => (
                                                    menu_t("menu.edit.title", locale),
                                                    draw_edit_menu
                                                        as fn(&mut egui::Ui, &mut GrafitoApp),
                                                ),
                                                MobileMenuPage::Vista => (
                                                    menu_t("menu.view.title", locale),
                                                    draw_view_menu
                                                        as fn(&mut egui::Ui, &mut GrafitoApp),
                                                ),
                                                MobileMenuPage::Perspectivas => (
                                                    menu_t("menu.persp.title", locale),
                                                    draw_perspectives_menu
                                                        as fn(&mut egui::Ui, &mut GrafitoApp),
                                                ),
                                                MobileMenuPage::Herramientas => (
                                                    menu_t("menu.tools.title", locale),
                                                    draw_tools_menu
                                                        as fn(&mut egui::Ui, &mut GrafitoApp),
                                                ),
                                                MobileMenuPage::Paneles => (
                                                    menu_t("menu.panels.title", locale),
                                                    draw_panels_menu
                                                        as fn(&mut egui::Ui, &mut GrafitoApp),
                                                ),
                                                MobileMenuPage::Ayuda => (
                                                    menu_t("menu.help.title", locale),
                                                    draw_help_menu
                                                        as fn(&mut egui::Ui, &mut GrafitoApp),
                                                ),
                                            };
                                        // Filas a ancho completo: aire táctil
                                        // como la raíz (el header vive fijo
                                        // arriba, no en el scroll). Padding
                                        // base-4 (8,12 ≈ 44 px de alto) en
                                        // vez del (10,12) fuera de escala.
                                        ui.spacing_mut().button_padding =
                                            egui::vec2(SPACE_SM, SPACE_MD);
                                        // Drill fullscreen: el menu_button
                                        // externo vuelca contenido y los
                                        // anidados van en acordeón.
                                        ui.with_layout(
                                            egui::Layout::top_down_justified(egui::Align::LEFT),
                                            |ui| {
                                                MOBILE_DRILL.with(|d| d.set(Some(0)));
                                                draw(ui, app);
                                                MOBILE_DRILL.with(|d| d.set(None));
                                            },
                                        );
                                    }
                                }
                            });
                        // (Barra rápida: draw_mobile_quick_bar en el panel
                        // inferior anclado de arriba; sin duplicado aquí.)
                    });
            });
    }
}

pub(crate) fn draw_top_bar(
    app: &mut GrafitoApp,
    ctx: &egui::Context,
    show_sidebar: bool,
    show_compact_panel_menu: bool,
    show_right_drawer_toggle: bool,
) {
    #[cfg(feature = "profile")]
    puffin::profile_scope!("ui_top_bar");

    let theme = current_theme(ctx);
    let accent = theme.accent;
    let bar_fill = theme.panel_bg;
    let side_fill = theme.sidebar_bg;
    let _sep_col = theme.separator;
    // Al volver a desktop el popup móvil no persiste a medias.
    if !mobile_assistant_bubble(ctx.screen_rect().width()) {
        app.mobile_menu_open = false;
    }

    // ── Scandinavian single bar 40px (F18.1 unificada: breadcrumb + paleta)
    // ── hairline 10% sin barra negra
    // D1 safe-area: en Android edge-to-edge la barra nacía en y=0 bajo el
    // status (24 px). En móvil suma `safe_inset_top` arriba (zona útil
    // intacta en 32 px); en desktop safe=0, sin cambio.
    let narrow_top = mobile_assistant_bubble(ctx.screen_rect().width());
    let safe_top = if narrow_top {
        grafito_ui::tokens::safe_inset_top()
    } else {
        0.0
    };
    let top_bar_response = egui::TopBottomPanel::top("top_bar")
        .exact_height(grafito_ui::tokens::TOP_BAR_HEIGHT + safe_top)
        .frame(
            egui::Frame::none()
                .fill(bar_fill)
                // Zona 32 px exacta: hamburguesa/lupa/label de 32 ya no
                // desbordan hacia abajo (ese era el aire "caído").
                .inner_margin(egui::Margin {
                    left: SPACE_MD,
                    right: SPACE_MD,
                    top: 4.0 + safe_top,
                    bottom: 4.0,
                }),
        )
        .show_separator_line(false)
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(SPACING_MINIMAL_X, SPACING_MINIMAL_Y);
            ui.spacing_mut().button_padding = egui::vec2(SPACING_BUTTON_X, SPACING_BUTTON_Y);
            egui::menu::bar(ui, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(SPACE_SM, SPACE_SM);
                // Compact top-chrome dense menu bar: SPACING_BUTTON (16×8) reduced to 10×4 for single-bar Scandinavian fit
                ui.spacing_mut().button_padding = egui::vec2(10.0, 4.0);
                if mobile_assistant_bubble(ctx.screen_rect().width()) {
                    // Móvil <740 (M3 top bar plegable): ni menús ni iconos;
                    // hamburguesa con todo + buscar. Desktop intacto.
                    draw_mobile_top_bar(ui, app, show_compact_panel_menu);
                    return;
                }
                let compact_top_chrome = top_chrome_uses_overflow(ctx.screen_rect().width());
                if compact_top_chrome {
                    draw_file_menu(ui, app);
                    draw_edit_menu(ui, app);
                    if show_compact_panel_menu {
                        draw_panels_menu(ui, app);
                    }
                    menu_button(
                        ui,
                        grafito_ui::i18n::t("menu.more", app.config_locale()),
                        |ui| {
                            draw_view_menu(ui, app);
                            draw_perspectives_menu(ui, app);
                            draw_tools_menu(ui, app);
                            if !show_compact_panel_menu {
                                draw_panels_menu(ui, app);
                            }
                            draw_help_menu(ui, app);
                        },
                    );
                } else {
                    draw_file_menu(ui, app);
                    draw_edit_menu(ui, app);
                    draw_view_menu(ui, app);
                    draw_perspectives_menu(ui, app);
                    draw_tools_menu(ui, app);
                    if show_compact_panel_menu {
                        draw_panels_menu(ui, app);
                    }
                    draw_help_menu(ui, app);
                }

                // Vault ELIMINADO 2026-10-06: sin breadcrumb.
                // Paleta: sin botón en barra (directiva usuario 2026-10-06).
                // Se abre por Herramientas > Paleta o Ctrl+K global.

                // ── Right controls — avatar + tema ──
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing =
                        egui::vec2(SPACING_MINIMAL_X, SPACING_MINIMAL_Y);
                    ui.spacing_mut().button_padding =
                        egui::vec2(SPACING_BUTTON_X, SPACING_BUTTON_Y);

                    if show_right_drawer_toggle
                        && action_icon_button(
                            ui,
                            Icon::Menu,
                            accent,
                            if app.right_drawer_open {
                                "Ocultar panel contextual"
                            } else {
                                "Mostrar panel contextual"
                            },
                        )
                        .clicked()
                    {
                        app.right_drawer_open = !app.right_drawer_open;
                    }
                    if action_icon_button(
                        ui,
                        Icon::Whiteboard,
                        if app.whiteboard_open {
                            accent
                        } else {
                            grafito_ui::theme::current_theme(ui.ctx()).text_secondary
                        },
                        grafito_ui::i18n::t("center.menu_whiteboard", app.config_locale()),
                    )
                    .clicked()
                    {
                        app.whiteboard_open = !app.whiteboard_open;
                    }
                    // Toggle tema pill compacto — integrado a la izquierda de Configuración
                    {
                        if draw_theme_toggle_switch(ui, app.dark_mode, app.config_locale()) {
                            app.dark_mode = !app.dark_mode;
                            if app.dark_mode {
                                DARK.apply(ui.ctx());
                            } else {
                                LIGHT.apply(ui.ctx());
                            }
                            app.save_app_config();
                        }
                    }
                    // Configuración — ventana única (Asistente + Perfil + preview persistente)
                    {
                        use grafito_ui::i18n::t as shell_t;
                        let shell_cfg = app.config_locale();
                        let is_open = app.assistant.settings_open;
                        let bg = if is_open {
                            theme.accent.gamma_multiply(0.14)
                        } else {
                            egui::Color32::TRANSPARENT
                        };
                        // 32×32: entra en la barra de 40 y acerca el objetivo
                        // táctil al mínimo (26×24 era mitad en 2-en-1/táctil).
                        let (rect, resp) =
                            ui.allocate_exact_size(egui::vec2(32.0, 32.0), egui::Sense::click());
                        let resp = resp.on_hover_text(shell_t("shell.config_btn", shell_cfg));
                        // Nombre accesible + estado (WCAG 4.1.2). El botón se
                        // dibuja a mano: sin esto era un control sin nombre y
                        // su condición de abierto solo vivía en el color del
                        // ícono.
                        // B-A186: nombre + estado vía `shell.*` (antes ES duros).
                        resp.widget_info(|| {
                            egui::WidgetInfo::labeled(
                                egui::WidgetType::Button,
                                true,
                                if is_open {
                                    shell_t("shell.config_open", shell_cfg)
                                } else {
                                    shell_t("shell.config_btn", shell_cfg)
                                },
                            )
                        });
                        let hovered = resp.hovered();
                        let cur_bg = if hovered { theme.button_hover } else { bg };
                        if ui.is_rect_visible(rect) {
                            if hovered || is_open {
                                ui.painter().rect_filled(rect, RADIUS_SM, cur_bg);
                                ui.painter().rect_stroke(
                                    rect,
                                    RADIUS_SM,
                                    egui::Stroke::new(
                                        1.0,
                                        if is_open {
                                            theme.accent
                                        } else {
                                            theme.separator.gamma_multiply(0.10)
                                        },
                                    ),
                                );
                            }
                            draw_icon(
                                ui.painter(),
                                rect.shrink(3.0),
                                Icon::Settings,
                                if is_open {
                                    theme.accent
                                } else {
                                    theme.text_secondary
                                },
                            );
                            // Anillo de foco (WCAG 2.4.7 / 2.4.13).
                            if resp.has_focus() {
                                theme.paint_focus_ring(ui.painter(), rect);
                            }
                        }
                        if resp.clicked()
                            || (resp.has_focus()
                                && ui.input(|i| {
                                    i.key_pressed(egui::Key::Enter)
                                        || i.key_pressed(egui::Key::Space)
                                }))
                        {
                            let will_open = !is_open;
                            app.assistant.settings_open = will_open;
                            app.assistant.config_tab = 0;
                            app.assistant.settings_search.clear();
                            app.assistant.settings_open_show_welcome =
                                !crate::utils::load_config().onboarding_completed;
                            app.show_mascot_config = false;
                        }
                    }
                    // B-A186: `locale` pre-borrow (E0502: `&mut app.assistant_visible`
                    // + `app.config_locale()` en la misma llamada).
                    let reopen_locale = app.config_locale();
                    if let Some(response) = draw_assistant_reopen_control(
                        ui,
                        &mut app.assistant_visible,
                        accent,
                        reopen_locale,
                    ) {
                        if response.clicked() {
                            app.open_assistant_workspace();
                        }
                    }
                });
            });
        });
    let _ = top_bar_response;
    // ── LEFT SIDEBAR (68px icon rail) ──
    // Entradas planas (directiva usuario 2026-10-06: sin plegable Math).
    // El grupo junta Álgebra/Herram./Datos/Prob. (tabs 0-3); Vista intacta.
    // J2: sin Vault. Las perspectivas se cambian únicamente desde la barra
    // superior.
    use grafito_ui::i18n::t as rail_t;
    let rail_locale = app.config_locale();
    // B-A185: labels abreviadas por ancho del rail + tips vía `rail.*`;
    // "Math" es nombre de sección invariante (precedente B-A11).
    let math: [(&'static str, Icon, &'static str); 4] = [
        (
            rail_t("rail.tab_algebra", rail_locale),
            Icon::Function,
            rail_t("rail.tip_algebra", rail_locale),
        ),
        (
            rail_t("rail.tab_tools", rail_locale),
            Icon::Settings,
            rail_t("rail.tip_tools", rail_locale),
        ),
        (
            rail_t("rail.tab_data", rail_locale),
            Icon::Spreadsheet,
            rail_t("rail.tip_data", rail_locale),
        ),
        (
            rail_t("rail.tab_prob", rail_locale),
            Icon::Histogram,
            rail_t("rail.tip_prob", rail_locale),
        ),
    ];
    let vista_entry = (
        rail_t("rail.tab_view", rail_locale),
        Icon::Eye,
        rail_t("rail.tip_view", rail_locale),
    );
    if show_sidebar {
        egui::SidePanel::left("icon_bar")
            .exact_width(RAIL_WIDTH)
            .resizable(false)
            .show_separator_line(false)
            .frame(
                egui::Frame::none()
                    .fill(side_fill)
                    .stroke(egui::Stroke::NONE),
            )
            .show(ctx, |ui| {
                #[cfg(feature = "profile")]
                puffin::profile_scope!("ui_sidebar");
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(SPACE_SM);

                            // ── Tabs del sidebar: entradas planas (sin plegable Math).
                            let mut entries: Vec<(usize, &'static str, Icon, &'static str)> =
                                Vec::new();
                            for (t, (label, icon, tip)) in math.iter().enumerate() {
                                entries.push((t, label, *icon, tip));
                            }
                            entries.push((4, vista_entry.0, vista_entry.1, vista_entry.2));
                            for (entry_tab, label, icon, tip) in entries {
                                let active = app.sidebar_tab == entry_tab;
                                let bg = if active {
                                    theme.sidebar_tab_active_bg
                                } else {
                                    Color32::TRANSPARENT
                                };
                                let ic_color = if active {
                                    theme.sidebar_tab_active
                                } else {
                                    theme.sidebar_tab_inactive
                                };

                                // Item insetado: el borde de la burbuja respira 4 px
                                // por lado y nunca lo clipa el borde del panel.
                                let item_w = RAIL_WIDTH - RAIL_ITEM_PAD_X * 2.0;
                                let (rect, resp) = ui.allocate_exact_size(
                                    egui::vec2(item_w, RAIL_ITEM_HEIGHT),
                                    egui::Sense::click(),
                                );
                                let resp = resp.on_hover_text(tip);
                                // Movimiento reducido: salto al estado final
                                // (el fundido del rail animaba siempre).
                                let progress = if app.reduced_motion {
                                    if active || resp.hovered() {
                                        1.0
                                    } else {
                                        0.0
                                    }
                                } else {
                                    ui.ctx().animate_bool(
                                        ui.id().with(("sidebar_tab_state", entry_tab)),
                                        active || resp.hovered(),
                                    )
                                };
                                let target_fill = if active { bg } else { theme.hover_overlay };
                                let target_border = if active {
                                    theme.accent
                                } else {
                                    theme.separator
                                };
                                if ui.is_rect_visible(rect) {
                                    ui.painter().rect(
                                        rect,
                                        RADIUS_SM,
                                        interpolate_color(
                                            Color32::TRANSPARENT,
                                            target_fill,
                                            progress,
                                        ),
                                        egui::Stroke::new(
                                            1.0,
                                            interpolate_color(
                                                Color32::TRANSPARENT,
                                                target_border,
                                                progress,
                                            ),
                                        ),
                                    );
                                    if active {
                                        let marker = egui::Rect::from_center_size(
                                            egui::pos2(rect.min.x + 4.0, rect.center().y),
                                            egui::vec2(2.0, 22.0),
                                        );
                                        ui.painter().rect_filled(marker, 1.0, theme.accent);
                                    }
                                    let icon_rect = egui::Rect::from_center_size(
                                        rect.center() - egui::vec2(0.0, 9.0),
                                        egui::vec2(ICON_MD, ICON_MD),
                                    );
                                    draw_icon(ui.painter(), icon_rect, icon, ic_color);
                                    // Iconos puros si el rail está colapsado: el texto
                                    // de `painter` no respeta layout y, sin clip,
                                    // pinta slivers de 1-2px en el borde.
                                    if rail_labels_visible(rect.width()) {
                                        ui.painter().with_clip_rect(rect).text(
                                            rect.center() + egui::vec2(0.0, 15.0),
                                            Align2::CENTER_CENTER,
                                            label,
                                            egui::FontId::proportional(TYPE_2XS),
                                            ic_color,
                                        );
                                    }
                                }

                                // Anillo de foco (WCAG 2.4.7 / 2.4.13). El ítem se dibuja a mano, así que
                                // no hereda el indicador de egui: el rail era
                                // inalcanzable por teclado sin feedback visual.
                                if resp.has_focus() {
                                    theme.paint_focus_ring(ui.painter(), rect);
                                }

                                // Nombre accesible = etiqueta corta, no el
                                // tooltip. Antes era `*tip` ("Objetos,
                                // variables y CAS"), que describe pero no
                                // nombra: un lector de pantalla anunciaba seis
                                // controles con frases largas en vez de con los
                                // nombres que el rail muestra en pantalla.
                                // El estado activo viaja en el nombre para que
                                // "seleccionado" no dependa solo del color
                                // (WCAG 1.4.1 / 4.1.2).
                                resp.widget_info(|| {
                                    egui::WidgetInfo::labeled(
                                        egui::WidgetType::Button,
                                        true,
                                        if active {
                                            rail_t("rail.active_suffix", rail_locale)
                                                .replace("{label}", label)
                                        } else {
                                            label.to_string()
                                        },
                                    )
                                });
                                // `Sense::click` solo no responde al teclado.
                                if resp.clicked()
                                    || (resp.has_focus()
                                        && ui.input(|i| {
                                            i.key_pressed(egui::Key::Enter)
                                                || i.key_pressed(egui::Key::Space)
                                        }))
                                {
                                    if active {
                                        app.left_drawer_open = !app.left_drawer_open;
                                        app.compact_drawer_open = !app.compact_drawer_open;
                                    } else {
                                        app.sidebar_tab = entry_tab;
                                        app.left_drawer_open = true;
                                        app.compact_drawer_open = true;
                                    }
                                }
                                // Aire Scandinavian entre tabs: 8 px.
                                ui.add_space(SPACE_SM);
                            }

                            ui.add_space(SPACE_SM);
                        });
                    });
                ui.add_space(SPACE_SM);
            });
    }
}

/// Sin tabs: el Canvas es la única vista (tira eliminada 2026-10-12).
#[allow(dead_code)]
pub(crate) fn draw_tab_bar(_app: &mut GrafitoApp, _ctx: &egui::Context) {}

/// Renders the shared Geometry 3D utility contents and returns a close request.
fn draw_geometry_utility_contents(
    app: &mut GrafitoApp,
    ctx: &egui::Context,
    ui: &mut egui::Ui,
) -> bool {
    let theme = current_theme(ctx);
    let mut close_requested = false;
    // B-A186: chrome del dock vía `shell.*` (antes ES duros); tab
    // "Inspector" reutiliza `inspector.panel_title` existente (0 keys).
    use grafito_ui::i18n::t as shell_t;
    let shell_locale = app.config_locale();

    ui.add_space(SPACE_SM);
    // Wrap honesto en el dock: evita slivers clipados en docks angostos.
    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
    ui.horizontal(|ui| {
        ui.add(
            egui::Label::new(
                egui::RichText::new(shell_t("shell.workspace_title", shell_locale))
                    .color(theme.text_secondary)
                    .size(grafito_ui::tokens::TYPE_SM)
                    .strong(),
            )
            .truncate(),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            close_requested = action_icon_button(
                ui,
                Icon::Close,
                theme.text_secondary,
                shell_t("shell.dock_hide", shell_locale),
            )
            .clicked();
        });
    });
    ui.add_space(SPACE_XS);
    // Tabs centrados: control segmentado de ancho completo (dos mitades
    // iguales = grupo centrado por construcción, con jerarquía). El activo
    // es un pill con contraste real (mismo par fill/texto que los pills de
    // acción); tamaño TYPE_SM; SPACE_SM debajo antes del contenido.
    let tab_width = ((ui.available_width() - SPACE_SM) / 2.0).max(0.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = SPACE_SM;
        let inspector_selected = app.workspace_dock_tab == WorkspaceDockTab::Inspector;
        let inspector_fill = if inspector_selected {
            theme.keyboard_enter_bg
        } else {
            Color32::TRANSPARENT
        };
        let inspector_text = if inspector_selected {
            theme.keyboard_enter_text
        } else {
            theme.text_secondary
        };
        if ui
            .add_sized(
                egui::vec2(tab_width, 32.0),
                egui::Button::new(
                    egui::RichText::new(shell_t("inspector.panel_title", shell_locale))
                        .size(TYPE_SM)
                        .strong()
                        .color(inspector_text),
                )
                .selected(inspector_selected)
                .fill(inspector_fill)
                .stroke(if inspector_selected {
                    egui::Stroke::NONE
                } else {
                    egui::Stroke::new(1.0, theme.separator.gamma_multiply(0.10))
                })
                .rounding(RADIUS_PILL),
            )
            .on_hover_text(shell_t("shell.inspector_hint", shell_locale))
            .clicked()
        {
            app.workspace_dock_tab = WorkspaceDockTab::Inspector;
        }
        let assistant_label = if app.assistant.is_pending {
            shell_t("shell.assistant_pending", shell_locale)
        } else {
            shell_t("shell.assistant_reopen", shell_locale)
        };
        let assistant_selected = app.workspace_dock_tab == WorkspaceDockTab::Assistant;
        let assistant_fill = if assistant_selected {
            theme.keyboard_enter_bg
        } else {
            Color32::TRANSPARENT
        };
        let assistant_text = if assistant_selected {
            theme.keyboard_enter_text
        } else {
            theme.text_secondary
        };
        if ui
            .add_sized(
                egui::vec2(tab_width, 32.0),
                egui::Button::new(
                    egui::RichText::new(assistant_label)
                        .size(TYPE_SM)
                        .strong()
                        .color(assistant_text),
                )
                .selected(assistant_selected)
                .fill(assistant_fill)
                .stroke(if assistant_selected {
                    egui::Stroke::NONE
                } else {
                    egui::Stroke::new(1.0, theme.separator.gamma_multiply(0.10))
                })
                .rounding(RADIUS_PILL),
            )
            .on_hover_text(shell_t("shell.assistant_dock_hint", shell_locale))
            .clicked()
        {
            app.open_assistant_workspace();
        }
    });
    ui.add_space(SPACE_SM);
    ui.separator();
    ui.add_space(SPACE_SM);

    match app.workspace_dock_tab {
        WorkspaceDockTab::Inspector => crate::panels::draw_right_properties_contents(app, ui),
        WorkspaceDockTab::Assistant => app.draw_assistant_contents_in_workspace_dock(ctx, ui),
    }

    close_requested
}

/// Un único dock contextual evita que Inspector y Asistente compitan por el
/// canvas de Geometry 3D. Ambos conservan su estado mientras la otra pestaña
/// está activa.
pub(crate) fn draw_geometry_utility_dock(app: &mut GrafitoApp, ctx: &egui::Context) {
    let theme = current_theme(ctx);
    egui::SidePanel::right("geometry_utility_dock")
        .default_width(DRAWER_RIGHT_DEFAULT)
        .min_width(DRAWER_RIGHT_MIN)
        .max_width(DRAWER_RIGHT_MAX)
        .resizable(true)
        .show_separator_line(false)
        .frame(
            // Fondo opaco: sin esto el canvas 3D se transparenta por el
            // borde y deja la tira de glifos en zigzag.
            egui::Frame::none()
                .fill(theme.panel_bg)
                .stroke(egui::Stroke::NONE),
        )
        .show(ctx, |ui| {
            if draw_geometry_utility_contents(app, ctx, ui) {
                app.right_drawer_open = false;
            }
        });
}

/// Compact Geometry 3D keeps the same Inspector/Assistant utility on demand.
pub(crate) fn draw_compact_geometry_utility_dock(app: &mut GrafitoApp, ctx: &egui::Context) {
    let theme = current_theme(ctx);
    let available_h = ctx.available_rect().height();
    // BB3-P1-7: el default fijo de 360 clipaba cuando `max_height` quedaba
    // debajo (viewport angosto): `min(360, max)` nace sin recorte.
    let maximum_height = (available_h * 0.65).max(180.0);
    egui::TopBottomPanel::bottom("geometry_utility_compact_dock")
        .resizable(true)
        .default_height(compact_dock_default_height(available_h))
        .min_height(180.0)
        .max_height(maximum_height)
        .show_separator_line(false)
        .frame(
            egui::Frame::none()
                .fill(theme.panel_bg)
                .stroke(egui::Stroke::NONE),
        )
        .show(ctx, |ui| {
            if draw_geometry_utility_contents(app, ctx, ui) {
                app.compact_geometry_utility_open = false;
            }
        });
}

pub(crate) fn draw_bottom_bar(app: &mut GrafitoApp, ctx: &egui::Context, show_input: bool) {
    #[cfg(feature = "profile")]
    puffin::profile_scope!("ui_bottom_bar");

    let theme = current_theme(ctx);
    let quiet_action = theme.text_secondary;

    // ── BOTTOM CHROME — solo la entrada (Scandinavian minimalista).
    // La fila de estado (coords/hint/conteo) se eliminó: el hint de la
    // herramienta vive en el hover de la entrada. Campo inset rectangular
    // (radio 8, 32 px), texto centrado verticalmente.
    let mut should_exec = false;
    egui::TopBottomPanel::bottom("bottom_chrome")
        .show_separator_line(false)
        .frame(
            egui::Frame::none()
                .fill(theme.panel_bg)
                .stroke(egui::Stroke::NONE)
                .inner_margin(egui::Margin::symmetric(BOTTOM_BAR_PAD_X, BOTTOM_BAR_PAD_Y)),
        )
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = SPACE_XS;
                // S4 Nivel 0 móvil: pill de herramienta armada (un tap la
                // desarma a Mover). Solo móvil y solo si hay algo armado:
                // en desktop la toolbar ya dice qué está activo.
                let nivel0 = mobile_level0(ctx.screen_rect().width());
                if nivel0 && show_input && app.current_tool != Tool::Select {
                    ui.vertical_centered(|ui| {
                        let nombre = grafito_ui::i18n::tool_label(
                            grafito_ui::toolbar::tool_slug(app.current_tool),
                            app.config_locale(),
                        );
                        // Chip M3 a 44 px táctiles (BB3-P1-6; antes 32): el
                        // target más chico de la zona más usada a 390. La ×
                        // va en hit separado de 44×44 (antes un solo botón
                        // "{nombre} ×" donde cualquier tap desarmaba sin
                        // distinguir): cuerpo + × desarman, pero cada uno con
                        // su zona de pulsación y su nombre accesible.
                        // NOTA z-stack (honesto): Mili burbuja / chip / Play
                        // siguen en flujo de la bottom bar (chip top-center
                        // sobre la entrada, Play en la fila de entrada); no se
                        // reordena el z-stack global: eso exige tocar la
                        // maquinaria del Area/FAB y queda fuera de este slice.
                        let chip_h = tool_chip_height();
                        let chip_g = ui.fonts(|f| {
                            f.layout_no_wrap(
                                nombre.to_owned(),
                                egui::FontId::proportional(TYPE_BASE),
                                egui::Color32::WHITE,
                            )
                        });
                        let mut disarm = false;
                        ui.horizontal(|ui| {
                            ui.add_space(
                                (ui.available_width()
                                    - (chip_g.size().x + 2.0 * SPACE_LG + 4.0 + SPACE_SM + chip_h))
                                    .max(0.0)
                                    / 2.0,
                            );
                            if ui
                                .add_sized(
                                    egui::vec2(chip_g.size().x + 2.0 * SPACE_LG + 4.0, chip_h),
                                    egui::Button::new(nombre),
                                )
                                .on_hover_text("Herramienta armada (tocar × para desarmar)")
                                .clicked()
                            {
                                disarm = true;
                            }
                            if ui
                                .add_sized(egui::vec2(chip_h, chip_h), egui::Button::new("×"))
                                .on_hover_text("Desarmar herramienta")
                                .clicked()
                            {
                                disarm = true;
                            }
                        });
                        if disarm {
                            app.current_tool = Tool::Select;
                            app.tool_ghost = None;
                            app.reset_tool_input();
                        }
                    });
                }
                // ── Entrada: campo rectangular + ejecutar fantasma
                if show_input {
                    // El popup de sugerencias ocupa toda la barra (integra columna).
                    let span = (ui.next_widget_position().x, ui.available_width());
                    let input_row = ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = SPACE_SM;
                        // Radio global es SM (8): sobre 32 px de alto da
                        // rectángulo suave. El campo de entrada es rectangular suave.
                        {
                            let visuals = ui.visuals_mut();
                            visuals.widgets.inactive.rounding =
                                egui::Rounding::same(BOTTOM_INPUT_RADIUS);
                            visuals.widgets.hovered.rounding =
                                egui::Rounding::same(BOTTOM_INPUT_RADIUS);
                            visuals.widgets.active.rounding =
                                egui::Rounding::same(BOTTOM_INPUT_RADIUS);
                        }
                        let response = draw_command_input(
                            ui,
                            app,
                            "bottom_bar",
                            [
                                command_input_width(ui.available_width(), BOTTOM_INPUT_RESERVED_W),
                                bottom_input_h(ui.ctx().screen_rect().width()),
                            ],
                            "Entrada…",
                            true,
                            Some(span),
                        );
                        if response.submitted && !app.input_text.is_empty() {
                            should_exec = true;
                        }

                        if action_icon_button_sized(
                            ui,
                            Icon::Play,
                            quiet_action,
                            "Ejecutar entrada (Enter)",
                            // 44 px como el campo vecino (el 26x24 default era
                            // el target más chico de la zona más usada).
                            egui::vec2(44.0, 44.0),
                        )
                        .clicked()
                        {
                            should_exec = true;
                        }
                    });
                    // Disclosure progresiva: ejemplos + herramienta actual viven
                    // en el hover, no en cromo permanente (la fila de estado
                    // con coords/hint/conteo se eliminó por minimalista).
                    // B-A188: `input_locale` se computa antes porque el hint
                    // de la herramienta ya sale de `hint.*` con `locale`.
                    let input_locale = app.config_locale();
                    let tool_hint = if let Some(h) = app.pending_action_hint() {
                        h
                    } else {
                        match app.current_view {
                            ViewMode::D2 => {
                                status_hint_for_tool(app.current_tool, input_locale).to_string()
                            }
                            ViewMode::D3 => format!(
                                "{} · {}",
                                app.view3d.name(),
                                status_hint_for_3d_tool(app.current_tool, input_locale)
                            ),
                        }
                    };
                    // B-A186: marco del hover vía `shell.input_hint` con
                    // `{tool_hint}` (el hint ya localizado B-A188).
                    // Sin eco táctil: el hint trae ejemplos + herramienta
                    // (distinto de lo tipeado), así que queda también en
                    // Nivel 0; la guarda solo salta si algún día lo iguala.
                    let input_hover = grafito_ui::i18n::t("shell.input_hint", input_locale)
                        .replace("{tool_hint}", &tool_hint);
                    if !hover_echo_suppressed(
                        &app.input_text,
                        &input_hover,
                        mobile_level0(ctx.screen_rect().width()),
                    ) {
                        input_row.response.on_hover_text(input_hover);
                    }
                }
            });
        });
    if should_exec && !app.input_text.is_empty() {
        let time = ctx.input(|i| i.time);
        app.submit_input_text(time);
    }
    // Sin FAB lupa: la búsqueda ya vive en la barra móvil (<740, que
    // cubre a <480). El verde duplicado tapaba el canvas sin aportar.
}

pub(crate) fn draw_command_input(
    ui: &mut egui::Ui,
    app: &mut GrafitoApp,
    id_salt: &'static str,
    size: [f32; 2],
    hint: &str,
    frame: bool,
    // Ancho de columna `(x, ancho)` para el popup de sugerencias: el menú
    // ocupa toda la columna (uniforme con el panel) en vez del input solo.
    // `None` conserva el ancho del campo.
    popup_span: Option<(f32, f32)>,
) -> CommandInputResponse {
    let theme = current_theme(ui.ctx());
    let response = ui.add_sized(
        [size[0].max(0.0), size[1].max(0.0)],
        egui::TextEdit::singleline(&mut app.input_text)
            .id_salt(id_salt)
            .hint_text(hint)
            // Centro vertical + aire lateral: el default LEFT_TOP dejaba el
            // texto pegado arriba en cajas de 32 px (mismo bug del buscador).
            .vertical_align(egui::Align::Center)
            .margin(egui::vec2(SPACE_MD, 0.0))
            .frame(frame)
            .text_color(theme.text_primary),
    );
    if app.command_input_focus_requested {
        response.request_focus();
        app.command_input_focus_requested = false;
    }

    let changed = response.changed();
    let suggestions = if !app.input_text.is_empty() {
        // Física primero también en la Entrada: los laboratorios
        // (Dinámica) suben en su perspectiva, sin rediseño.
        let fisica_first = app.perspective == crate::Perspective::Fisica;
        compute_autocomplete_suggestions_for_perspective(
            &app.input_text,
            &app.document,
            fisica_first,
        )
    } else {
        Vec::new()
    };

    let mut completed = false;
    // Tab completa solo con relación honesta (o elección explícita); si el
    // popup está abierto igual se consume para no saltar de foco.
    let tab_pressed = if !suggestions.is_empty() && (response.has_focus() || response.lost_focus())
    {
        ui.input(|i| {
            is_autocomplete_completion_key(egui::Key::Tab) && i.key_pressed(egui::Key::Tab)
        })
    } else {
        false
    };
    let completion_key = if tab_pressed {
        ui.input_mut(|input| {
            input.consume_key(egui::Modifiers::NONE, egui::Key::Tab);
        });
        Some(egui::Key::Tab)
    } else {
        None
    };
    let mut show_popup =
        !suggestions.is_empty() && (response.has_focus() || completion_key.is_some());
    // Token actual para la regla de aceptación (prefijo real o nada).
    // Propio (String): vive en el frame sin prestar `input_text`.
    let accept_token = autocomplete_token(&app.input_text)
        .unwrap_or("")
        .to_string();
    if changed {
        // El usuario siguió escribiendo: la selección anterior ya no vale.
        app.autocomplete.navigated = false;
    }
    if show_popup {
        if app.autocomplete.selected >= suggestions.len() {
            app.autocomplete.selected = 0;
        }
        let len = suggestions.len();
        if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
            app.autocomplete.selected = (app.autocomplete.selected + 1) % len;
            app.autocomplete.navigated = true;
        }
        if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
            app.autocomplete.selected = (app.autocomplete.selected + len - 1) % len;
            app.autocomplete.navigated = true;
        }
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            show_popup = false;
            app.autocomplete.open = false;
            app.autocomplete.selected = 0;
            app.autocomplete.navigated = false;
        }
        if show_popup && completion_key.is_some() {
            // Tab: acepta si el usuario eligió o si lo escrito es prefijo
            // real de la sugerencia; si no, se ignora (nunca corrompe).
            let take = app.autocomplete.navigated
                || suggestions
                    .get(app.autocomplete.selected)
                    .is_some_and(|item| autocomplete_accepts(&accept_token, &item.text));
            if take {
                completed = complete_autocomplete_selection(
                    &mut app.input_text,
                    &suggestions,
                    &mut app.autocomplete,
                );
                if completed {
                    show_popup = false;
                    response.request_focus();
                }
            }
        }
    }
    if !show_popup {
        app.autocomplete.navigated = false;
    }
    app.autocomplete.open = show_popup;

    if show_popup
        && draw_autocomplete_popup(ui, app, id_salt, response.rect, popup_span, &suggestions)
    {
        response.request_focus();
    }

    let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
    // Profesional y predecible: Enter acepta la sugerencia si el usuario la
    // eligió o si es continuación honesta de lo escrito; si el texto no
    // cambió con la inserción (p. ej. variable ya exacta), se envía igual.
    if enter && show_popup {
        let take = app.autocomplete.navigated
            || suggestions
                .get(app.autocomplete.selected)
                .is_some_and(|item| autocomplete_accepts(&accept_token, &item.text));
        if take {
            let before = app.input_text.clone();
            let applied = complete_autocomplete_selection(
                &mut app.input_text,
                &suggestions,
                &mut app.autocomplete,
            );
            if applied && app.input_text != before {
                completed = true;
                response.request_focus();
            }
        }
    }
    CommandInputResponse {
        submitted: !completed && enter && (response.has_focus() || response.lost_focus()),
        changed,
    }
}

pub(crate) fn command_input_width(available_width: f32, reserved_width: f32) -> f32 {
    (available_width - reserved_width).max(0.0)
}

pub(crate) fn draw_unsaved_changes_dialog(app: &mut GrafitoApp, ctx: &egui::Context) {
    let Some(action) = app.pending_document_action() else {
        return;
    };
    let save_locale = app.config_locale();
    let current_path = app.current_document_path().map(|p| p.to_path_buf());
    // Pulido 2026-10-16 (directiva usuario: el label suelto de un objeto era
    // ruido feo): se muestra el NOMBRE del archivo si existe y, si el
    // documento todavía no tiene path, el subtítulo honesto de la clave save.untitled
    // (nada de labels de objetos al pedo).
    let doc_caption: Option<String> = current_path
        .as_deref()
        .and_then(|p| p.file_name())
        .map(|s| s.to_string_lossy().into_owned())
        .or_else(|| Some(grafito_ui::i18n::t("save.untitled", save_locale).to_owned()));
    let save_error = app.document_save_error().map(str::to_owned);
    let mut open = true;
    let mut decision = None;
    let theme = current_theme(ctx);
    egui::Window::new(grafito_ui::i18n::t("save.title", save_locale))
        .id(egui::Id::new("unsaved_changes_dialog"))
        .collapsible(false)
        .resizable(false)
        .default_width(340.0)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .order(egui::Order::Foreground)
        .open(&mut open)
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(theme.panel_bg)
                .stroke(egui::Stroke::new(1.0, theme.separator))
                .rounding(egui::Rounding::same(RADIUS_LG)),
        )
        .show(ctx, |ui| {
            ui.set_min_width(300.0);
            ui.set_max_width(340.0);
            dialog_contents(
                ui,
                action,
                doc_caption.clone(),
                save_error.as_ref(),
                save_locale,
                &mut decision,
            );
        });
    if !open && decision.is_none() {
        decision = Some(UnsavedDecision::Cancel);
    }
    if let Some(decision) = decision {
        app.queue_unsaved_decision(decision);
    }
}

fn dialog_contents(
    ui: &mut egui::Ui,
    _action: DocumentAction,
    doc_caption: Option<String>,
    save_error: Option<&String>,
    locale: grafito_ui::i18n::Locale,
    decision: &mut Option<UnsavedDecision>,
) {
    use grafito_ui::i18n::t;
    // A11Y: Esc descarta el modal persistente con la opción segura (Cancelar).
    // El orden Tab lo da egui por orden de creación: Cancelar → Descartar →
    // Guardar (el foco arranca en la opción segura).
    if decision.is_none() && ui.input(|input| input.key_pressed(egui::Key::Escape)) {
        *decision = Some(UnsavedDecision::Cancel);
    }
    let theme = current_theme(ui.ctx());
    // Minimalista 2026-10-06 (directiva usuario): el título de la ventana
    // el título de la ventana ya lo dice; el cuerpo es solo el nombre del
    // documento en secundario, sin pregunta repetida.
    egui::Frame::none()
        .inner_margin(egui::Margin::symmetric(SPACE_MD, SPACE_SM))
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(SPACE_XS);
                if let Some(doc_name) = doc_caption.as_deref() {
                    ui.label(
                        egui::RichText::new(doc_name)
                            .size(TYPE_SM)
                            .color(theme.text_secondary),
                    );
                }
                if let Some(error) = save_error {
                    ui.add_space(SPACE_SM);
                    ui.label(
                        egui::RichText::new(t("save.error", locale).replace("{error}", error))
                            .size(TYPE_XS)
                            .color(theme.danger),
                    );
                }
                ui.add_space(SPACE_LG);
                // Buttons — centered group, primary Guardar with accent, others quiet
                ui.horizontal(|ui| {
                    let total_w = 3.0 * 88.0 + 2.0 * SPACE_SM;
                    let pad = ((ui.available_width() - total_w) / 2.0).max(0.0);
                    ui.add_space(pad);
                    ui.spacing_mut().item_spacing.x = SPACE_SM;
                    // Cancelar — ghost
                    let cancel = egui::Button::new(
                        egui::RichText::new(t("save.cancel", locale))
                            .size(TYPE_SM)
                            .color(theme.text_secondary),
                    )
                    .rounding(egui::Rounding::same(RADIUS_SM))
                    .fill(theme.panel_bg)
                    .stroke(egui::Stroke::new(1.0, theme.separator));
                    if ui.add_sized(egui::vec2(88.0, 32.0), cancel).clicked() {
                        *decision = Some(UnsavedDecision::Cancel);
                    }
                    // Descartar — muted, hover danger via text color only (no extra fill)
                    let discard = egui::Button::new(
                        egui::RichText::new(t("save.discard", locale))
                            .size(TYPE_SM)
                            .color(theme.text_secondary),
                    )
                    .rounding(egui::Rounding::same(RADIUS_SM))
                    .fill(theme.panel_bg)
                    .stroke(egui::Stroke::new(1.0, theme.separator));
                    if ui.add_sized(egui::vec2(88.0, 32.0), discard).clicked() {
                        *decision = Some(UnsavedDecision::Discard);
                    }
                    // Guardar — primary accent
                    let save = egui::Button::new(
                        egui::RichText::new(t("save.save", locale))
                            .size(TYPE_SM)
                            .color(Color32::WHITE)
                            .strong(),
                    )
                    .rounding(egui::Rounding::same(RADIUS_SM))
                    .fill(theme.accent)
                    .stroke(egui::Stroke::NONE);
                    if ui.add_sized(egui::vec2(88.0, 32.0), save).clicked() {
                        *decision = Some(UnsavedDecision::Save);
                    }
                    ui.add_space(pad);
                });
                ui.add_space(SPACE_XS);
            });
        });
}

/// Lado del popup de autocompletado respecto al campo: `true` = arriba.
///
/// Sin aire debajo (barra inferior: ~8 px) el `Area` con `constrain` se
/// corría hacia arriba tapando la entrada. La regla es determinista: si
/// abajo no entra ni una fila + cromo, o si arriba hay más aire, se abre
/// hacia arriba. Puro, testeable headless.
pub(crate) fn autocomplete_places_above(space_above: f32, space_below: f32) -> bool {
    let min_below = AUTOCOMPLETE_MIN_LIST_H + AUTOCOMPLETE_CHROME_RESERVE + AUTOCOMPLETE_GAP;
    space_below < min_below || space_below < space_above
}

/// Alto máximo de la lista con scroll: el aire disponible menos gap y
/// cromo (footer + padding). Nunca negativo: con aire 0 solo queda el
/// footer. Puro, testeable headless.
pub(crate) fn autocomplete_list_max_height(available_space: f32) -> f32 {
    (available_space - AUTOCOMPLETE_GAP - AUTOCOMPLETE_CHROME_RESERVE).max(0.0)
}

fn draw_autocomplete_popup(
    ui: &mut egui::Ui,
    app: &mut GrafitoApp,
    id_salt: &'static str,
    input_rect: egui::Rect,
    popup_span: Option<(f32, f32)>,
    suggestions: &[AutocompleteItem],
) -> bool {
    // Integrado, no ventana: misma columna que el campo. Abajo (drawer,
    // álgebra) pegado como continuación del panel; arriba (barra inferior
    // sin aire) card flotante quieta con hairline + sombra popup. La lista
    // scrollea acotada al aire real: el popup jamás tapa el campo.
    // Sin `popup_span` cae al ancho del campo.
    let (span_x, span_w) = popup_span.unwrap_or((input_rect.min.x, input_rect.width()));
    let screen = ui.ctx().screen_rect();
    let space_above = input_rect.min.y - screen.min.y;
    let space_below = screen.max.y - input_rect.max.y;
    let above = autocomplete_places_above(space_above, space_below);
    let available = if above { space_above } else { space_below };
    let list_max = autocomplete_list_max_height(available);
    let (popup_pos, pivot) = if above {
        (
            egui::pos2(span_x, input_rect.min.y - AUTOCOMPLETE_GAP),
            egui::Align2::LEFT_BOTTOM,
        )
    } else {
        (egui::pos2(span_x, input_rect.max.y), egui::Align2::LEFT_TOP)
    };
    let selected = app.autocomplete.selected;
    // Honestidad visual: solo hay fila destacada si el usuario la eligió
    // con ↑↓ (navigated). Sin navegar, Enter envía el texto tal cual — pintar
    // la fila 0 como seleccionada mentía y el Enter "rompía".
    let highlight = app.autocomplete.navigated;
    let theme = current_theme(ui.ctx());
    let display: Vec<(String, String)> = suggestions
        .iter()
        .take(MAX_AUTOCOMPLETE_SUGGESTIONS)
        .map(|it| (it.text.clone(), it.detail.clone()))
        .collect();
    let popup_id = ui.id().with(id_salt).with("autocomplete_popup");
    let mut clicked: Option<usize> = None;
    // Card flotante de arriba: superficie panel + hairline 10% + sombra
    // popup (tokens); pegada de abajo: continuación sin borde ni sombra.
    let frame = if above {
        egui::Frame::none()
            .fill(theme.panel_bg)
            .stroke(theme.hairline_stroke())
            .rounding(egui::Rounding::same(RADIUS_SM))
            .shadow(egui::Shadow {
                offset: egui::vec2(0.0, SHADOW_POPUP_OFFSET_Y),
                blur: SHADOW_POPUP_BLUR,
                spread: 0.0,
                color: egui::Color32::from_black_alpha(SHADOW_ALPHA),
            })
            .inner_margin(egui::Margin {
                left: SPACE_SM,
                right: SPACE_SM,
                top: SPACE_XS,
                bottom: SPACE_XS,
            })
    } else {
        egui::Frame::none()
            .fill(theme.input_bg)
            .rounding(egui::Rounding {
                nw: 0.0,
                ne: 0.0,
                sw: RADIUS_SM,
                se: RADIUS_SM,
            })
            .inner_margin(egui::Margin {
                left: SPACE_SM,
                right: SPACE_SM,
                top: SPACE_XS,
                bottom: SPACE_XS,
            })
    };
    egui::Area::new(popup_id)
        .fixed_pos(popup_pos)
        .pivot(pivot)
        .order(egui::Order::Foreground)
        .show(ui.ctx(), |ui| {
            frame.show(ui, |ui| {
                ui.set_min_width((span_w - 2.0 * SPACE_SM).max(0.0));
                ui.set_max_width((span_w - 2.0 * SPACE_SM).max(0.0));
                // Selección serena (tinta sage, texto primario) en vez de
                // píldora de acento: guía sin gritar.
                ui.scope(|ui| {
                    let visuals = ui.visuals_mut();
                    visuals.selection.bg_fill = theme.selection_bg;
                    visuals.selection.stroke = egui::Stroke::new(1.0, theme.text_primary);
                    egui::ScrollArea::vertical()
                        .max_height(list_max)
                        .show(ui, |ui| {
                            ui.with_layout(
                                egui::Layout::top_down_justified(egui::Align::LEFT),
                                |ui| {
                                    for (i, (text, detail)) in display.iter().enumerate() {
                                        let mut job = egui::text::LayoutJob::single_section(
                                            text.clone(),
                                            egui::TextFormat {
                                                font_id: egui::FontId::proportional(TYPE_SM),
                                                color: theme.text_primary,
                                                ..Default::default()
                                            },
                                        );
                                        job.append(
                                            "   ",
                                            0.0,
                                            egui::TextFormat {
                                                font_id: egui::FontId::proportional(TYPE_XS),
                                                color: theme.text_tertiary,
                                                ..Default::default()
                                            },
                                        );
                                        job.append(
                                            detail,
                                            0.0,
                                            egui::TextFormat {
                                                font_id: egui::FontId::proportional(TYPE_XS),
                                                color: theme.text_tertiary,
                                                ..Default::default()
                                            },
                                        );
                                        let row = ui.add(egui::SelectableLabel::new(
                                            highlight && i == selected,
                                            job,
                                        ));
                                        if highlight && i == selected {
                                            row.scroll_to_me(Some(egui::Align::Center));
                                        }
                                        if row.clicked() {
                                            clicked = Some(i);
                                        }
                                    }
                                },
                            );
                        });
                });
                ui.add_space(SPACE_XS);
                ui.label(
                    egui::RichText::new("↑↓ elegir · Enter aceptar · Tab completar · Esc cerrar")
                        .size(TYPE_XS)
                        .color(theme.text_tertiary),
                );
            });
        });
    if let Some(i) = clicked {
        app.autocomplete.selected = i;
        return complete_autocomplete_selection(
            &mut app.input_text,
            suggestions,
            &mut app.autocomplete,
        );
    }
    false
}

pub(crate) fn status_hint_for_3d_tool(
    tool: Tool,
    locale: grafito_ui::i18n::Locale,
) -> &'static str {
    use grafito_ui::i18n::hint_msg;
    match tool {
        // B-A188: tesseract/hypercube comparten key con el hint 2D (textos idénticos).
        Tool::Tesseract4D => hint_msg("tesseract_4d", locale),
        Tool::Hypercube5D => hint_msg("hypercube_5d", locale),
        _ => hint_msg("fallback_3d", locale),
    }
}

fn status_hint_for_tool(tool: Tool, locale: grafito_ui::i18n::Locale) -> &'static str {
    use grafito_ui::i18n::hint_msg;
    match tool {
        Tool::Select => hint_msg("select", locale),
        Tool::Point => hint_msg("point", locale),
        Tool::Point3D => hint_msg("point_3d", locale),
        Tool::Line => hint_msg("line", locale),
        Tool::Segment => hint_msg("segment", locale),
        Tool::Ray => hint_msg("ray", locale),
        Tool::Vector => hint_msg("vector", locale),
        Tool::Circle => hint_msg("circle", locale),
        Tool::Polygon => hint_msg("polygon", locale),
        Tool::RegularPolygon => hint_msg("regular_polygon", locale),
        Tool::Function => hint_msg("function", locale),
        Tool::Distance => hint_msg("distance", locale),
        Tool::DistanceConstraint => hint_msg("distance_constraint", locale),
        Tool::Angle => hint_msg("angle", locale),
        Tool::AngleConstraint => hint_msg("angle_constraint", locale),
        Tool::Area => hint_msg("area", locale),
        Tool::Slope => hint_msg("slope", locale),
        Tool::Slider => hint_msg("slider", locale),
        Tool::Locus => hint_msg("locus", locale),
        Tool::Button | Tool::Image => hint_msg("unavailable", locale),
        Tool::Midpoint => hint_msg("midpoint", locale),
        Tool::Perpendicular => hint_msg("perpendicular", locale),
        Tool::Tangent => hint_msg("tangent", locale),
        Tool::Root => hint_msg("root", locale),
        Tool::Extremum => hint_msg("extremum", locale),
        Tool::Intersect => hint_msg("intersect", locale),
        Tool::Coincident => hint_msg("coincident", locale),
        Tool::Horizontal => hint_msg("horizontal", locale),
        Tool::Vertical => hint_msg("vertical", locale),
        Tool::EqualLength => hint_msg("equal_length", locale),
        Tool::Symmetry => hint_msg("symmetry", locale),
        Tool::EllipseByFoci => hint_msg("ellipse", locale),
        Tool::ParabolaByFocusDirectrix => hint_msg("parabola", locale),
        Tool::HyperbolaByFoci => hint_msg("hyperbola", locale),
        Tool::ConicByFivePoints => hint_msg("conic5", locale),
        Tool::PolygonUnion => hint_msg("polygon_union", locale),
        Tool::PolygonIntersection => hint_msg("polygon_intersection", locale),
        Tool::PolygonDifference => hint_msg("polygon_difference", locale),
        Tool::PolygonXor => hint_msg("polygon_xor", locale),
        Tool::Sphere3D => hint_msg("sphere_3d", locale),
        Tool::Cube3D => hint_msg("cube_3d", locale),
        // F3d: hints de las 11 tools F3a (antes caían al fallback genérico).
        Tool::Translate => hint_msg("translate", locale),
        Tool::Rotate => hint_msg("rotate", locale),
        Tool::Dilate => hint_msg("dilate", locale),
        Tool::Reflect => hint_msg("reflect", locale),
        Tool::Compass => hint_msg("compass", locale),
        Tool::Semicircle => hint_msg("semicircle", locale),
        Tool::Spline => hint_msg("spline", locale),
        Tool::Prism3D => hint_msg("prism_3d", locale),
        Tool::Tetrahedron3D => hint_msg("tetrahedron_3d", locale),
        Tool::Checkbox => hint_msg("checkbox", locale),
        Tool::InputBox => hint_msg("input_box", locale),
        Tool::Tesseract4D => hint_msg("tesseract_4d", locale),
        Tool::Hypercube5D => hint_msg("hypercube_5d", locale),
        Tool::ComplexContour => hint_msg("complex_contour", locale),
        _ => hint_msg("fallback_2d", locale),
    }
}

pub(crate) fn apply_color_picker_object_color_change(
    document: &mut grafito_core::Document,
    object_id: grafito_core::ObjectId,
    color: grafito_geometry::Color,
    undo_stack: &mut VecDeque<grafito_core::Document>,
    redo_stack: &mut VecDeque<grafito_core::ChangeSet>,
) -> Result<bool, String> {
    let Some(object) = document.get_object(object_id) else {
        return Ok(false);
    };
    if grafito_ui::color_picker::colors_match(object.color(), color) {
        return Ok(false);
    }
    let mut candidate = object.clone();
    candidate.set_color(color);
    let Some(before) = document.try_replace_object_with_previous(object_id, candidate)? else {
        return Ok(false);
    };

    let mut snapshot = DeferredPanelSnapshot::new(undo_stack.len());
    snapshot.capture_successful_replacement(before);
    Ok(snapshot.save_if_semantically_changed(document, undo_stack, redo_stack))
}

pub(crate) fn apply_color_picker_regular_polychoron_fill_color_change(
    document: &mut grafito_core::Document,
    object_id: grafito_core::ObjectId,
    color: grafito_geometry::Color,
    undo_stack: &mut VecDeque<grafito_core::Document>,
    redo_stack: &mut VecDeque<grafito_core::ChangeSet>,
) -> Result<bool, String> {
    let Some(grafito_core::GeoObject::RegularPolychoron4D(polychoron)) =
        document.get_object(object_id)
    else {
        return Ok(false);
    };
    let Some(fill_color) = polychoron.fill_color else {
        return Ok(false);
    };
    if grafito_ui::color_picker::colors_match(fill_color, color) {
        return Ok(false);
    }

    let mut candidate = grafito_core::GeoObject::RegularPolychoron4D(polychoron.clone());
    let grafito_core::GeoObject::RegularPolychoron4D(candidate_polychoron) = &mut candidate else {
        return Ok(false);
    };
    candidate_polychoron.fill_color = Some(color);
    let Some(before) = document.try_replace_object_with_previous(object_id, candidate)? else {
        return Ok(false);
    };

    let mut snapshot = DeferredPanelSnapshot::new(undo_stack.len());
    snapshot.capture_successful_replacement(before);
    Ok(snapshot.save_if_semantically_changed(document, undo_stack, redo_stack))
}

const COLOR_PICKER_DIALOG_SIZE: egui::Vec2 = egui::vec2(390.0, 350.0);
const COLOR_PICKER_SAFE_MARGIN: f32 = 16.0;

pub(crate) fn color_picker_safe_viewport(viewport: egui::Rect) -> egui::Rect {
    let horizontal_inset = COLOR_PICKER_SAFE_MARGIN.min((viewport.width() - 1.0).max(0.0) * 0.5);
    let vertical_inset = COLOR_PICKER_SAFE_MARGIN.min((viewport.height() - 1.0).max(0.0) * 0.5);
    egui::Rect::from_min_max(
        viewport.min + egui::vec2(horizontal_inset, vertical_inset),
        viewport.max - egui::vec2(horizontal_inset, vertical_inset),
    )
}

/// Acción terminal del diálogo; los cambios del picker siguen siendo transitorios
/// hasta que se recibe [`Self::Apply`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ColorPickerDialogAction {
    Apply,
    Dismiss,
}

pub(crate) fn apply_color_picker_dialog_action(
    action: ColorPickerDialogAction,
    document: &mut grafito_core::Document,
    target: ColorPickerTarget,
    object_id: grafito_core::ObjectId,
    color: grafito_geometry::Color,
    undo_stack: &mut VecDeque<grafito_core::Document>,
    redo_stack: &mut VecDeque<grafito_core::ChangeSet>,
) -> Result<bool, String> {
    match action {
        ColorPickerDialogAction::Apply => match target {
            ColorPickerTarget::ObjectColor => apply_color_picker_object_color_change(
                document, object_id, color, undo_stack, redo_stack,
            ),
            ColorPickerTarget::RegularPolychoronFill => {
                apply_color_picker_regular_polychoron_fill_color_change(
                    document, object_id, color, undo_stack, redo_stack,
                )
            }
        },
        ColorPickerDialogAction::Dismiss => Ok(false),
    }
}

pub(crate) fn draw_color_picker(app: &mut GrafitoApp, ctx: &egui::Context) {
    #[cfg(feature = "profile")]
    puffin::profile_scope!("ui_color_picker");
    use grafito_ui::i18n::t;
    let locale = app.config_locale();
    let Some(ActiveColorPicker {
        object_id,
        target,
        mut picker,
    }) = app.active_color_picker.clone()
    else {
        return;
    };

    let mut keep_open = true;
    let mut outcome = grafito_ui::color_picker::ColorPickerOutcome::default();
    let theme = current_theme(ctx);

    egui::Window::new(t("picker.dialog_title", locale))
        .collapsible(false)
        .resizable(false)
        .fixed_size(COLOR_PICKER_DIALOG_SIZE)
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .constrain_to(color_picker_safe_viewport(ctx.screen_rect()))
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(theme.panel_bg)
                .stroke(egui::Stroke::new(1.0, theme.separator.gamma_multiply(0.10)))
                .rounding(grafito_ui::tokens::RADIUS_LG)
                .shadow(egui::Shadow {
                    offset: egui::vec2(0.0, 2.0),
                    blur: 8.0,
                    spread: 0.0,
                    color: egui::Color32::from_black_alpha(8),
                }),
        )
        .open(&mut keep_open)
        .show(ctx, |ui| {
            outcome = picker.show(ui, &mut app.color_favorites, locale);
            ui.add_space(SPACE_SM);
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new(t("picker.dialog_hint", locale))
                        .size(TYPE_XS)
                        .color(theme.text_tertiary)
                        .weak()
                        .italics(),
                );
            });
        });

    // Live apply — sin botón Aplicar
    if outcome.color_changed {
        let color = picker.to_color();
        let _ = apply_color_picker_dialog_action(
            ColorPickerDialogAction::Apply,
            &mut app.document,
            target,
            object_id,
            color,
            &mut app.undo_stack,
            &mut app.redo_stack,
        );
    }
    if outcome.any_changed() {
        ctx.request_repaint();
    }

    let dialog_action = (!keep_open).then_some(ColorPickerDialogAction::Dismiss);
    if let Some(action) = dialog_action {
        // Cerrar sin revertir — el color ya está aplicado en vivo
        let _ = apply_color_picker_dialog_action(
            action,
            &mut app.document,
            target,
            object_id,
            picker.to_color(),
            &mut app.undo_stack,
            &mut app.redo_stack,
        );
        app.active_color_picker = None;
    } else {
        app.active_color_picker = Some(ActiveColorPicker {
            object_id,
            target,
            picker,
        });
    }
}

const MAX_AUTOCOMPLETE_TOKEN_CHARS: usize = 256;
const MAX_AUTOCOMPLETE_SUGGESTIONS: usize = 8;
const MAX_DOCUMENT_AUTOCOMPLETE_CANDIDATES: usize = 64;
const MIN_AUTOCOMPLETE_SCORE: f64 = 0.35;
/// Los cierres `)]} ` también cortan el token: en `sin(a)` el token es `""`
/// (nada que completar) en vez de `"a)"` — Tab ya no se come el paréntesis.
const AUTOCOMPLETE_SEPARATORS: &[char] = &[
    '[', '(', ',', ' ', '\t', '=', '+', '-', '*', '/', ')', ']', '}',
];
/// La ayuda solo participa con 3+ caracteres: evita que "a"/"de" igualen todo.
const AUTOCOMPLETE_HELP_MIN_CHARS: usize = 3;
/// Puntaje fijo de coincidencia en ayuda/categoría en español: por debajo de
/// cualquier coincidencia en nombre o alias (≥0.9) pero por encima del ruido
/// difuso típico (<0.7). Así "circulo" prefiere Circumcircle (alias) antes
/// que la docena de ayudas que mencionan círculos.
const AUTOCOMPLETE_ES_MATCH_SCORE: f64 = 0.8;

fn autocomplete_token(input: &str) -> Option<&str> {
    let mut token_chars = 0;
    for (index, character) in input.char_indices().rev() {
        if AUTOCOMPLETE_SEPARATORS.contains(&character) {
            return Some(input[index + character.len_utf8()..].trim());
        }
        token_chars += 1;
        if token_chars > MAX_AUTOCOMPLETE_TOKEN_CHARS {
            return None;
        }
    }
    Some(input.trim())
}

fn autocomplete_text_is_bounded(text: &str) -> bool {
    text.char_indices()
        .nth(MAX_AUTOCOMPLETE_TOKEN_CHARS)
        .is_none()
}

/// Calcula el puntaje de similitud entre el query escrito y un candidato
/// utilizando coincidencia exacta, prefijo, subsecuencia y distancia de Levenshtein (tolerancia a erratas).
pub(crate) fn similarity_score(query: &str, candidate: &str) -> f64 {
    if !autocomplete_text_is_bounded(query) || !autocomplete_text_is_bounded(candidate) {
        return 0.0;
    }

    let q = grafito_ui::command_palette::fold_spanish(&query.to_lowercase());
    let c = grafito_ui::command_palette::fold_spanish(&candidate.to_lowercase());

    if q.is_empty() {
        return 0.0;
    }

    if c == q {
        return 2.0; // Coincidencia perfecta
    }

    if c.starts_with(&q) {
        return 1.5 - (c.len() - q.len()) as f64 * 0.02; // Comienza con el query (más corto = mayor score)
    }

    if c.contains(&q) {
        return 1.2 - (c.find(&q).unwrap_or(0) as f64 * 0.05); // Contiene el query
    }

    // Distancia de Levenshtein para tolerancia a erratas
    let q_chars: Vec<char> = q.chars().collect();
    let c_chars: Vec<char> = c.chars().collect();

    let mut previous: Vec<usize> = (0..=c_chars.len()).collect();
    let mut current = vec![0; c_chars.len() + 1];

    for i in 1..=q_chars.len() {
        current[0] = i;
        for j in 1..=c_chars.len() {
            if q_chars[i - 1] == c_chars[j - 1] {
                current[j] = previous[j - 1];
            } else {
                current[j] = 1 + previous[j - 1].min(previous[j].min(current[j - 1]));
            }
        }
        std::mem::swap(&mut previous, &mut current);
    }

    let distance = previous[c_chars.len()];
    let max_len = q_chars.len().max(c_chars.len()) as f64;
    1.0 - (distance as f64 / max_len)
}

const MATH_FUNCTIONS: &[(&str, &str)] = &[
    ("deriv_z", "derivada complejos df/dz"),
    ("deriv_z_conj", "derivada complejos df/d(conj z)"),
    ("sin", "seno complejo/real"),
    ("sen", "seno (alias español de sin)"),
    ("cos", "coseno complejo/real"),
    ("tan", "tangente complejo/real"),
    ("sinh", "seno hiperbólico"),
    ("cosh", "coseno hiperbólico"),
    ("tanh", "tangente hiperbólica"),
    ("exp", "exponencial e^z"),
    ("ln", "logaritmo natural"),
    ("sqrt", "raíz cuadrada"),
    ("abs", "módulo / valor absoluto"),
    ("conj", "conjugado complejo"),
    ("re", "parte real de z"),
    ("im", "parte imaginaria de z"),
    ("arg", "argumento principal de z"),
    ("gamma", "función Gamma"),
    ("zeta", "función Zeta de Riemann"),
    ("bessel_j", "bessel J_0(z)"),
    ("bessel_y", "bessel Y_0(z)"),
    ("lambert_w", "función W de Lambert"),
    ("erf", "función de error"),
];

/// Returns a bounded insertion slot for a matching candidate, if it belongs
/// among the suggestions currently retained.
pub(crate) fn autocomplete_candidate_slot(
    candidates: &[(AutocompleteItem, f64)],
    text: &str,
    score: f64,
) -> Option<usize> {
    if score < MIN_AUTOCOMPLETE_SCORE {
        return None;
    }
    if candidates.len() < MAX_AUTOCOMPLETE_SUGGESTIONS {
        return Some(candidates.len());
    }

    let (worst_index, worst) = candidates
        .iter()
        .enumerate()
        .min_by(|(_, left), (_, right)| {
            left.1
                .partial_cmp(&right.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| right.0.text.len().cmp(&left.0.text.len()))
        })?;
    let score_order = score
        .partial_cmp(&worst.1)
        .unwrap_or(std::cmp::Ordering::Equal);
    if score_order.is_gt() || (score_order.is_eq() && text.len() < worst.0.text.len()) {
        Some(worst_index)
    } else {
        None
    }
}

fn add_autocomplete_candidate(
    candidates: &mut Vec<(AutocompleteItem, f64)>,
    current_token: &str,
    text: &str,
    detail: &str,
    bracket: bool,
) {
    let score = similarity_score(current_token, text);
    let Some(slot) = autocomplete_candidate_slot(candidates, text, score) else {
        return;
    };

    let item = AutocompleteItem {
        text: text.to_string(),
        detail: detail.to_string(),
        bracket,
    };
    if slot == candidates.len() {
        candidates.push((item, score));
    } else {
        candidates[slot] = (item, score);
    }
}

/// Calcula hasta 8 sugerencias de autocompletado para el último token del texto de entrada,
/// combinando comandos, objetos, variables y funciones matemáticas.
pub(crate) fn compute_autocomplete_suggestions(
    input: &str,
    document: &grafito_core::Document,
) -> Vec<AutocompleteItem> {
    let mut scored_items: Vec<(AutocompleteItem, f64)> = Vec::new();

    let Some(current_token) = autocomplete_token(input) else {
        return Vec::new();
    };
    if current_token.is_empty() {
        return Vec::new();
    }

    // 1. Agregar comandos de la paleta (búsqueda en español incluida:
    // nombre canónico + aliases ES/EN + categoría + ayuda). Lo que se
    // inserta sigue siendo el nombre canónico (despacho seguro).
    for cmd in grafito_ui::command_palette::all_commands() {
        if !cmd.syntax_hint.contains('[') {
            continue;
        }
        let mut best = similarity_score(current_token, cmd.name);
        if let Some(spec) = cmd.registered_spec() {
            for alias in spec.aliases {
                let score = similarity_score(current_token, alias);
                if score > best {
                    best = score;
                }
            }
        }
        let category_score = similarity_score(current_token, cmd.category);
        if category_score > best {
            best = category_score;
        }
        if current_token.chars().count() >= AUTOCOMPLETE_HELP_MIN_CHARS {
            let help = grafito_ui::command_palette::fold_spanish(&cmd.help.to_lowercase());
            let folded_token =
                grafito_ui::command_palette::fold_spanish(&current_token.to_lowercase());
            if help.contains(&folded_token) && AUTOCOMPLETE_ES_MATCH_SCORE > best {
                best = AUTOCOMPLETE_ES_MATCH_SCORE;
            }
        }
        let Some(slot) = autocomplete_candidate_slot(&scored_items, cmd.name, best) else {
            continue;
        };
        let item = AutocompleteItem {
            text: cmd.name.to_string(),
            detail: cmd.category.to_string(),
            bracket: true,
        };
        if slot == scored_items.len() {
            scored_items.push((item, best));
        } else {
            scored_items[slot] = (item, best);
        }
    }
    // 2. Agregar funciones matemáticas (nombre + descripción en español).
    for (name, desc) in MATH_FUNCTIONS {
        let mut best = similarity_score(current_token, name);
        if current_token.chars().count() >= AUTOCOMPLETE_HELP_MIN_CHARS {
            let folded_desc = grafito_ui::command_palette::fold_spanish(&desc.to_lowercase());
            let folded_token =
                grafito_ui::command_palette::fold_spanish(&current_token.to_lowercase());
            if folded_desc.contains(&folded_token) && AUTOCOMPLETE_ES_MATCH_SCORE > best {
                best = AUTOCOMPLETE_ES_MATCH_SCORE;
            }
        }
        let Some(slot) = autocomplete_candidate_slot(&scored_items, name, best) else {
            continue;
        };
        let item = AutocompleteItem {
            text: name.to_string(),
            detail: desc.to_string(),
            bracket: false,
        };
        if slot == scored_items.len() {
            scored_items.push((item, best));
        } else {
            scored_items[slot] = (item, best);
        }
    }

    // 3. Agregar objetos del documento
    // Bound fuzzy-scoring allocations before large documents can dominate an input frame.
    for (_, obj) in document
        .objects_iter()
        .take(MAX_DOCUMENT_AUTOCOMPLETE_CANDIDATES)
    {
        let label = obj.label();
        if !label.is_empty() {
            add_autocomplete_candidate(&mut scored_items, current_token, label, obj.name(), false);
        }
    }

    // 4. Agregar variables del documento
    for k in document
        .variables()
        .keys()
        .take(MAX_DOCUMENT_AUTOCOMPLETE_CANDIDATES)
    {
        add_autocomplete_candidate(&mut scored_items, current_token, k, "variable", false);
    }

    // Ordenar por puntaje descendente (mejor coincidencia) y tie-breaker por longitud ascendente
    scored_items.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.text.len().cmp(&b.0.text.len()))
    });

    scored_items.into_iter().map(|(item, _)| item).collect()
}

/// Autocompletado con orden por perspectiva: si `fisica_first`, los
/// laboratorios (categoría Dinámica) suben primeros con partición estable
/// sobre el orden clásico por puntaje. Con `false` es idéntico al clásico.
pub(crate) fn compute_autocomplete_suggestions_for_perspective(
    input: &str,
    document: &grafito_core::Document,
    fisica_first: bool,
) -> Vec<AutocompleteItem> {
    let items = compute_autocomplete_suggestions(input, document);
    if !fisica_first {
        return items;
    }
    let mut fisica = Vec::new();
    let mut resto = Vec::new();
    for item in items {
        if item.detail == "Dinámica" {
            fisica.push(item);
        } else {
            resto.push(item);
        }
    }
    fisica.extend(resto);
    fisica
}

/// Reemplaza el token actual (el fragmento que se está escribiendo tras el
/// último separador) por el item seleccionado. Para comandos bracket, añade
/// `[` al final para que el usuario complete los argumentos.
pub(crate) fn apply_autocomplete_item(input: &mut String, item: &AutocompleteItem) {
    let token_start = input
        .rfind(|c: char| AUTOCOMPLETE_SEPARATORS.contains(&c))
        .map(|i| i + 1)
        .unwrap_or(0);
    let prefix = &input[..token_start];

    if item.bracket {
        let command = match item.text.as_str() {
            "Thomas (Butterfly)" => "Thomas",
            other => other,
        };
        *input = format!("{}{}[", prefix, command);
    } else if MATH_FUNCTIONS.iter().any(|(name, _)| name == &item.text) {
        *input = format!("{}{}(", prefix, item.text);
    } else {
        *input = format!("{}{}", prefix, item.text);
    }
}

pub(crate) fn is_autocomplete_completion_key(key: egui::Key) -> bool {
    matches!(key, egui::Key::Tab)
}

/// Regla de aceptación honesta: Tab/Enter solo toman la sugerencia si el
/// usuario la eligió (↑↓/clic) o si lo escrito es prefijo de la sugerencia
/// (o al revés, corrección por recorte). Sin esto, Enter con el popup
/// abierto "generaba otro parámetro" al aceptar ruido difuso.
pub(crate) fn autocomplete_accepts(token: &str, selected_text: &str) -> bool {
    if token.is_empty() || selected_text.is_empty() {
        return false;
    }
    let folded_token = grafito_ui::command_palette::fold_spanish(&token.to_lowercase());
    let folded_selected = grafito_ui::command_palette::fold_spanish(&selected_text.to_lowercase());
    folded_selected.starts_with(&folded_token) || folded_token.starts_with(&folded_selected)
}

pub(crate) fn complete_autocomplete_selection(
    input: &mut String,
    suggestions: &[AutocompleteItem],
    autocomplete: &mut crate::app::InputAutocomplete,
) -> bool {
    let Some(item) = suggestions.get(autocomplete.selected) else {
        return false;
    };

    apply_autocomplete_item(input, item);
    autocomplete.open = false;
    autocomplete.selected = 0;
    autocomplete.navigated = false;
    true
}

#[cfg(test)]
mod status_hint_tests {
    use super::status_hint_for_tool;
    use grafito_ui::i18n::Locale;
    use grafito_ui::Tool;

    #[test]
    fn f3a_tools_have_specific_hints() {
        // F3d: las 11 tools F3a no caen al fallback genérico.
        // B-A188: firma con `locale`; Es pineado (resto por `hint.*`).
        // El fallback sale de la key (blindaje B-A188: 0 literales en prod).
        let fallback = grafito_ui::i18n::hint_msg("fallback_2d", Locale::Es);
        for (tool, word) in [
            (Tool::Translate, "Traslada"),
            (Tool::Rotate, "Rota"),
            (Tool::Dilate, "Homotecia"),
            (Tool::Reflect, "Refleja"),
            (Tool::Compass, "Compás"),
            (Tool::Semicircle, "Semicírculo"),
            (Tool::Spline, "Spline"),
            (Tool::Prism3D, "Prisma"),
            (Tool::Tetrahedron3D, "Tetraedro"),
            (Tool::Checkbox, "Casilla"),
            (Tool::InputBox, "Caja de entrada"),
        ] {
            let hint = status_hint_for_tool(tool, Locale::Es);
            assert_ne!(hint, fallback, "{tool:?} sin hint propio");
            assert!(
                hint.contains(word),
                "{tool:?} debe mencionar '{word}': {hint}"
            );
        }
    }
}

#[cfg(test)]
mod autocomplete_budget_tests {
    use super::{
        MAX_AUTOCOMPLETE_SUGGESTIONS, MAX_AUTOCOMPLETE_TOKEN_CHARS,
        MAX_DOCUMENT_AUTOCOMPLETE_CANDIDATES,
    };

    #[test]
    fn autocomplete_budgets_pin_user_facing_caps() {
        // Onda 2: pinnea los topes user-facing del autocompletado de la paleta.
        assert_eq!(MAX_AUTOCOMPLETE_TOKEN_CHARS, 256);
        assert_eq!(MAX_AUTOCOMPLETE_SUGGESTIONS, 8);
        assert_eq!(MAX_DOCUMENT_AUTOCOMPLETE_CANDIDATES, 64);
    }
}

#[cfg(test)]
mod autocomplete_ux_tests {
    use super::{
        apply_autocomplete_item, autocomplete_token, compute_autocomplete_suggestions,
        similarity_score,
    };

    #[test]
    fn closers_end_token_so_tab_never_eats_parens() {
        // `sin(a)` completo: token vacío → sin popup, Tab inerte.
        assert_eq!(autocomplete_token("sin(a)"), Some(""));
        assert_eq!(autocomplete_token("Circle[A, B]"), Some(""));
        // A medio escribir el token sigue vivo.
        assert_eq!(autocomplete_token("sin(a"), Some("a"));
        assert_eq!(autocomplete_token("comp"), Some("comp"));
    }

    #[test]
    fn apply_keeps_completed_expression_intact() {
        // Con token `a)` (cliente viejo) el reemplazo cubría el cierre;
        // con el tokenizador nuevo el caso no llega, pero el reemplazo
        // parcial nunca debe romper lo ya escrito.
        let mut input = "sin(".to_string();
        let item = crate::app::AutocompleteItem {
            text: "a".to_string(),
            detail: "variable".to_string(),
            bracket: false,
        };
        apply_autocomplete_item(&mut input, &item);
        assert_eq!(input, "sin(a");
    }

    #[test]
    fn similarity_ignores_accents() {
        assert_eq!(similarity_score("circulo", "círculo"), 2.0);
        assert_eq!(similarity_score("lapiz", "Lápiz"), 2.0);
        assert_eq!(similarity_score("seno", "seno"), 2.0);
    }

    #[test]
    fn spanish_queries_find_english_canonical_commands() {
        let doc = grafito_core::Document::new();
        // Ojo: Point/Circle/Polygon/Function no están en paleta
        // (`palette_visible=false`, se crean con herramientas); el español
        // debe encontrar sus equivalentes visibles.
        for (query, expected) in [
            ("circulo", "Circumcircle"),
            ("tangente", "Tangent"),
            ("punto", "Midpoint"),
            ("recta", "Line"),
            ("segmento", "Segment"),
            ("compas", "Compasses"),
            ("sen", "sin"),
            ("seno", "sin"),
            ("construir", "Ray"),
        ] {
            let texts: Vec<String> = compute_autocomplete_suggestions(query, &doc)
                .into_iter()
                .map(|item| item.text)
                .collect();
            assert!(
                texts.contains(&expected.to_string()),
                "{query:?} debería sugerir {expected:?}, encontró {texts:?}"
            );
        }
    }

    #[test]
    fn sin_tabs_sin_restos() {
        // Sin tabs (2026-10-12): ni tira ni notas ni vault ni estudio.
        // Se recorta el propio módulo de tests para no auto-matchear.
        let source = include_str!("ui.rs");
        let prod = source
            .split("mod autocomplete_ux_tests")
            .next()
            .unwrap_or(source);
        for resto in [
            "draw_tab_central",
            "draw_tab_cell",
            "draw_tab_add_cell",
            "tab_strip_rounding",
            "tab_close_width",
            "TabKind",
            "create_untitled_note",
            "untitled_note_name",
            "pending_guide_open",
            "vault_panel",
            "VaultMutation",
            "open_study_plan",
        ] {
            assert!(!prod.contains(resto), "resto de tabs/notas/vault: {resto}");
        }
    }
    #[test]
    fn navigated_defaults_to_submit_mode() {
        // Sin ↑↓ previo, Enter envía: el flag arranca apagado.
        assert!(!crate::app::InputAutocomplete::default().navigated);
    }

    #[test]
    fn accept_rule_requires_honest_prefix_or_choice() {
        use super::autocomplete_accepts;
        // Continuación honesta: Tab/Enter pueden tomarla.
        assert!(autocomplete_accepts("sin", "sin"));
        assert!(autocomplete_accepts("cir", "Circle"));
        assert!(autocomplete_accepts("a", "a"));
        assert!(autocomplete_accepts("circlex", "Circle"));
        // Sin relación: aceptar sería "generar otro parámetro".
        assert!(!autocomplete_accepts("recta", "Line"));
        assert!(!autocomplete_accepts("circulo", "Circumcircle"));
        assert!(!autocomplete_accepts("2", "Sec"));
        assert!(!autocomplete_accepts("", "Circle"));
        assert!(!autocomplete_accepts("a", ""));
    }
}

#[cfg(test)]
mod coverage_sweep_chrome_pure {
    use super::*;
    use grafito_ui::tokens::BOTTOM_INPUT_H;
    use grafito_ui::Tool;
    #[test]
    fn sin_tira_mobile_level0_intacto() {
        // Sin tira: solo queda el breakpoint táctil para el resto del chrome.
        assert!(mobile_level0(320.0));
        assert!(mobile_level0(479.0));
        assert!(!mobile_level0(480.0));
        assert!(!mobile_level0(740.0));
        assert!(!mobile_level0(1360.0));
    }
    #[test]
    fn barrido_umbrales_de_cromo() {
        assert!(rail_labels_visible(200.0));
        assert!(!rail_labels_visible(10.0));
        assert!(left_drawer_content_visible(300.0));
        assert!(!left_drawer_content_visible(10.0));
        assert!(top_chrome_uses_overflow(300.0));
        assert!(!top_chrome_uses_overflow(3000.0));
        assert!(assistant_reopen_control_visible(false));
        assert!(!assistant_reopen_control_visible(true));
        assert_eq!(command_input_width(800.0, 200.0), 600.0);
        assert_eq!(command_input_width(100.0, 200.0), 0.0);
        // S4 Nivel 0 móvil: bajo 480 sin rail/drawers; en 480+ intacto.
        assert!(mobile_level0(320.0));
        assert!(mobile_level0(479.0));
        assert!(!mobile_level0(480.0));
        assert!(!mobile_level0(740.0));
        assert!(!mobile_level0(1360.0));
        // #88 Modales responsive: 320 desktop, viewport-32 en angosto.
        assert_eq!(dialog_content_min_width(1360.0), 320.0);
        assert_eq!(dialog_content_min_width(420.0), 320.0);
        assert_eq!(dialog_content_min_width(320.0), 288.0);
        assert_eq!(dialog_content_min_width(100.0), 200.0);
        // #88: la fila de 388 px se apila bajo 320 de contenido.
        assert!(dialog_buttons_stack(288.0, 388.0));
        assert!(!dialog_buttons_stack(420.0, 388.0));
        assert!(!dialog_buttons_stack(256.0, 256.0));
        // S7 responsive: nada desborda en 320/360/720. Sin FAB lupa
        // (la búsqueda vive en la barra móvil): la entrada manda.
        for w in [320.0, 360.0, 720.0] {
            // La entrada cabe con su reserva y nunca es negativa.
            let iw = command_input_width(w - 16.0, BOTTOM_INPUT_RESERVED_W);
            assert!(iw >= 0.0, "entrada negativa en {w}");
            assert!(iw <= w, "entrada desborda en {w}");
        }
    }

    #[test]
    fn hover_eco_se_suprime_solo_en_tactil_angosto() {
        // S2: en Nivel 0 el tooltip que repite el visible es eco y se va;
        // si aporta algo distinto queda, y en desktop queda todo.
        // (`concat` a propósito: el blindaje B-A186 usa `include_str!` y el
        // literal completo se auto-matchearía como ES duro.)
        let mostrar = ["Mostrar asis", "tente"].concat();
        assert!(hover_echo_suppressed("Asistente", "Asistente", true));
        assert!(!hover_echo_suppressed("Asistente", &mostrar, true));
        // Solo-ícono (`visible` vacío): el hover ES el nombre, queda siempre.
        assert!(!hover_echo_suppressed("", "Más", true));
        assert!(!hover_echo_suppressed("Asistente", "Asistente", false));
        assert!(!hover_echo_suppressed("Asistente", &mostrar, false));
    }

    #[test]
    fn bottom_chrome_es_minimo_y_rectangular() {
        // Rectangular suave (8), nunca pill; alto táctil base 4.
        assert_eq!(BOTTOM_INPUT_RADIUS, RADIUS_SM);
        assert_eq!(BOTTOM_INPUT_H, 32.0);
        // Hint por defecto corto, con separador ·, sin prefijo verbo-so.
        // B-A188: firma con `locale`; Es pineado (resto por `hint.*`).
        let hint = status_hint_for_tool(Tool::Select, grafito_ui::i18n::Locale::Es);
        assert!(hint.contains('·'), "hint Select con ·: {hint}");
        assert!(
            hint.chars().count() <= 40,
            "hint Select corto (≤40): {hint}"
        );
        assert!(!hint.contains("Seleccionar:"));
    }

    #[test]
    fn autocomplete_popup_nunca_tapa_el_campo() {
        // Barra inferior: ~8 px debajo → abre arriba.
        assert!(autocomplete_places_above(600.0, 8.0));
        // Drawer/álgebra con aire debajo → abajo (comportamiento actual).
        assert!(!autocomplete_places_above(100.0, 500.0));
        // Ventana corta: gana el lado con más aire.
        assert!(autocomplete_places_above(400.0, 100.0));
        assert!(!autocomplete_places_above(100.0, 400.0));
        // La lista se acota al aire real menos gap y cromo, nunca negativa.
        assert_eq!(autocomplete_list_max_height(600.0), 600.0 - 4.0 - 32.0);
        assert_eq!(autocomplete_list_max_height(20.0), 0.0);
    }
}
