//! Sección spreadsheet (hoja vinculada editable + serie + resumen + tablas).
//!
//! Split F14.4.4 de `panels.rs`: movida verbatim, sin cambio de
//! comportamiento. Lo puro (textos, anchos, métricas, parseos) ya vivía en
//! `sheet_layout.rs`; acá queda solo el dibujo egui + commits con undo.
//! Los helpers de `panels.rs` que usa este módulo son `pub(crate)` a
//! propósito (`panel_theme_local`, `draw_inspector_section`,
//! `panel_primary_button`, `open_data_tables_in_assistant`,
//! `import_local_xy_table`); `panels.rs` re-exporta
//! `draw_spreadsheet_section` para los call sites existentes.

use crate::panels::{
    draw_inspector_section, import_local_xy_table, open_data_tables_in_assistant,
    panel_primary_button, panel_theme_local,
};
use crate::sheet_layout::{
    build_series_edits, collect_sheet_tables, example_sheet_table,
    example_sheet_table_error_text_for, example_sheet_table_loaded_text_for,
    format_series_preview_text_for, format_series_success_for, format_statistic, parse_sheet_goto,
    series_preview, sheet_cell_display, sheet_cell_width_for, sheet_chip_hint_text_for,
    sheet_chip_text, sheet_col_label, sheet_hidden_tables_overflow_for, sheet_nav_metrics,
    sheet_series_field_w, sheet_summary_field_w, sheet_summary_tiles_for, sheet_table_rows,
    sheet_table_rows_overflow_for, sheet_table_shown, sheet_table_subtitle_for,
    sheet_table_title_for, stat_tile_cols, stat_tile_width_for, summarize_range_for,
    truncate_table_name, visible_sheet_cols, SheetCellKind, SheetEditState, SheetSeriesState,
    SheetSummaryState, SheetTable, SheetViewState, MAX_SPREADSHEET_TABLES, PANEL_BUTTON_H,
    SHEET_CELL_H, SHEET_CONTROL_H, SHEET_SUMMARY_BTN_W, SHEET_VIEW_ROWS,
};
use crate::GrafitoApp;
use grafito_ui::theme::current_theme;
use grafito_ui::tokens::{
    HIT_TARGET_AULA, RADIUS_MD, RADIUS_SM, SPACE_LG, SPACE_MD, SPACE_SM, SPACE_XS, TYPE_SM, TYPE_XS,
};

/// Grilla editable sobre `Document.spreadsheet` (F3b) con ventana navegable
/// de 8×14. Cada celda edita su fuente (`=A1+B1`, `=x(A)`, `(A1, B1*2)`); el
/// commit es por celda vía `stage_spreadsheet_cell_edits` (atómico, con undo)
/// y el error queda en esa celda sin voltear la hoja (una fórmula rota
/// muestra `—`). Ventana acotada por rendimiento; los presupuestos reales
/// (400×400/10k) los impone el core.
fn draw_sheet_editable_grid(ui: &mut egui::Ui, app: &mut GrafitoApp) {
    let ctx = ui.ctx().clone();
    let goto_locale = app.config_locale();
    let (_is_dark, accent, _fill, _sep, txt_col, txt_dim, hdr_col) = panel_theme_local(&ctx);
    let id = egui::Id::new("gc_sheet_edit_state");
    let mut edit: SheetEditState = ctx
        .data_mut(|data| data.get_temp::<SheetEditState>(id))
        .unwrap_or_default();
    let view_id = egui::Id::new("gc_sheet_view_state");
    let mut view: SheetViewState = ctx
        .data_mut(|data| data.get_temp::<SheetViewState>(view_id))
        .unwrap_or_default()
        .clamped();
    let mut snapshot = crate::app::DeferredPanelSnapshot::new(app.undo_stack.len());
    let mut dirty: Vec<(usize, usize, String)> = Vec::new();
    // Grilla adaptativa: las columnas visibles dependen del ancho real del
    // contenido (252 px en el drawer compacto → 4 columnas; panel ancho → 6).
    // Nunca hay columnas cortadas fuera del marco.
    let cols = visible_sheet_cols(ui.available_width());

    // Navegación de la ventana: la hoja es 400×400, la grilla muestra
    // `cols`×14. Toolbar única (misma estructura en todo ancho): rango visible
    // a la izquierda, flechas agrupadas y reset a la derecha, sin textos que
    // envuelvan. La fila "Ir a" salta directo a una celda con el parser
    // del cerebro (`parse_cell_reference`).
    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
    // ── Navegación de la hoja ──
    // Dos filas de ancho seguro: rango (o celda en edición) + cuatro flechas
    // iguales; abajo «Ir a» + Ir + A1. Todos los controles comparten
    // `SHEET_CONTROL_H` (misma altura exacta) y `sheet_nav_metrics` reparte el
    // ancho sin desbordar. Cero `Grid`: los botones no se escalonan.
    let metrics = sheet_nav_metrics(ui.available_width());
    let mut goto_requested = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = SPACE_XS;
        ui.spacing_mut().interact_size.y = SHEET_CONTROL_H;
        // Padding local: el theme usa 16×8 para botones "calmos"; acá el set
        // de navegación debe medir EXACTO `SHEET_CONTROL_H`.
        ui.spacing_mut().button_padding = egui::vec2(SPACE_SM, SPACE_XS);
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
        let (chip_rect, chip_resp) = ui.allocate_exact_size(
            egui::vec2(metrics.chip_w, SHEET_CONTROL_H),
            egui::Sense::hover(),
        );
        if ui.is_rect_visible(chip_rect) {
            let theme = current_theme(&ctx);
            let chip_text = sheet_chip_text(edit.editing, &view, cols);
            let chip_color = if edit.editing.is_some() {
                accent
            } else {
                hdr_col
            };
            ui.painter()
                .rect_filled(chip_rect, RADIUS_SM, theme.input_bg);
            ui.painter()
                .rect_stroke(chip_rect, RADIUS_SM, theme.hairline_stroke());
            // Sin texto si no entra: a chip colapsado el "C3:H10" se fugaba.
            if chip_rect.width() >= 40.0 {
                ui.painter().text(
                    chip_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    chip_text,
                    egui::FontId::monospace(TYPE_XS),
                    chip_color,
                );
            }
        }
        chip_resp.on_hover_text(sheet_chip_hint_text_for(
            &view.window_label(cols),
            goto_locale,
        ));
        let arrow = |ui: &mut egui::Ui, glyph: &str, tip: &str| {
            // `add_sized` + padding local reducido: el frame llena el alto
            // exacto del token (chip, flechas y botones quedan a ras).
            let resp = ui
                .add_sized(
                    [metrics.arrow_w, SHEET_CONTROL_H],
                    egui::Button::new(egui::RichText::new(glyph).size(TYPE_XS))
                        .wrap_mode(egui::TextWrapMode::Extend)
                        .rounding(RADIUS_SM),
                )
                .on_hover_text(tip);
            resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tip));
            resp.clicked()
        };
        let step = cols.max(1);
        let nav_back = grafito_ui::i18n::stats_msg("sheet_nav_back", goto_locale);
        let nav_forward = grafito_ui::i18n::stats_msg("sheet_nav_forward", goto_locale);
        let nav_up = grafito_ui::i18n::stats_msg("sheet_nav_up", goto_locale);
        let nav_down = grafito_ui::i18n::stats_msg("sheet_nav_down", goto_locale);
        if arrow(ui, "◀", nav_back) {
            view.origin_col = view.origin_col.saturating_sub(step);
        }
        if arrow(ui, "▶", nav_forward) {
            view.origin_col = view.origin_col.saturating_add(step);
            view = view.clamped();
        }
        if arrow(ui, "▲", nav_up) {
            view.origin_row = view.origin_row.saturating_sub(SHEET_VIEW_ROWS);
        }
        if arrow(ui, "▼", nav_down) {
            view.origin_row = view.origin_row.saturating_add(SHEET_VIEW_ROWS);
            view = view.clamped();
        }
    });
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = SPACE_XS;
        ui.spacing_mut().interact_size.y = SHEET_CONTROL_H;
        ui.spacing_mut().button_padding = egui::vec2(SPACE_SM, SPACE_XS);
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
        let goto_hint = grafito_ui::i18n::stats_msg("sheet_goto_hint", goto_locale);
        let goto_go = grafito_ui::i18n::stats_msg("sheet_goto_go", goto_locale);
        let resp = ui.add_sized(
            [metrics.field_w, SHEET_CONTROL_H],
            egui::TextEdit::singleline(&mut edit.goto)
                .hint_text(goto_hint)
                .vertical_align(egui::Align::Center)
                .font(egui::FontId::monospace(TYPE_XS)),
        );
        if resp.has_focus() {
            current_theme(&ctx).paint_focus_ring(ui.painter(), resp.rect);
        }
        if resp.changed() {
            edit.goto_error = None;
        }
        if resp.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)) {
            goto_requested = true;
        }
        if ui
            .add_sized(
                [metrics.action_w, SHEET_CONTROL_H],
                egui::Button::new(egui::RichText::new(goto_go).size(TYPE_XS)).rounding(RADIUS_SM),
            )
            .clicked()
        {
            goto_requested = true;
        }
        if ui
            .add_sized(
                [metrics.action_w, SHEET_CONTROL_H],
                egui::Button::new(egui::RichText::new("A1").size(TYPE_XS)).rounding(RADIUS_SM),
            )
            .clicked()
        {
            view = SheetViewState::default();
        }
    });
    if goto_requested && !edit.goto.trim().is_empty() {
        match parse_sheet_goto(&edit.goto) {
            Ok((row, col)) => {
                view.focus_cell(row, col, cols);
                edit.goto.clear();
                edit.goto_error = None;
            }
            Err(message) => {
                edit.goto_error = Some(message);
            }
        }
    }
    // Slot reservado: sin él la grilla salta al validar el "Ir a".
    if let Some(error) = &edit.goto_error {
        ui.label(
            egui::RichText::new(error)
                .color(current_theme(&ctx).danger)
                .size(TYPE_XS),
        );
    } else {
        ui.label(egui::RichText::new("\u{a0}").size(TYPE_XS));
    }
    ui.add_space(SPACE_SM);

    // Tabla real de una sola línea por celda (estilo planilla, no muro de
    // pills): sin editar muestra el VALOR; con clic muestra la FÓRMULA.
    // Enter o salir de la celda confirma; el error queda en esa celda.
    // Sin scroll: la grilla entra por construcción y navega por
    // flechas/chip/"Ir a" (la barra negra no aportaba nada).
    let theme = current_theme(&ctx);
    egui::Frame::none()
        .fill(theme.input_bg)
        .stroke(theme.hairline_stroke())
        .rounding(egui::Rounding::same(RADIUS_SM))
        .inner_margin(egui::Margin::same(SPACE_XS))
        .show(ui, |ui| {
            let cell_w = sheet_cell_width_for(ui.available_width(), cols);
            // Sin scroll horizontal: la grilla entra por construcción
            // (`visible_sheet_cols` reparte el ancho real) y la navegación va
            // por flechas/chip/"Ir a". La barra negra no aportaba nada.
            {
                // El `interact_size` global (38×26) imponía filas de 26:
                // la hoja fija el suyo para medir lo que dice la celda.
                ui.spacing_mut().interact_size.y = SHEET_CELL_H;
                egui::Grid::new("gc_sheet_editable")
                    .num_columns(cols + 1)
                    .striped(true)
                    .spacing([SPACE_XS, 0.0])
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("").size(TYPE_XS));
                        for col in 0..cols {
                            ui.label(
                                egui::RichText::new(sheet_col_label(view.origin_col + col))
                                    .color(hdr_col)
                                    .size(TYPE_XS)
                                    .strong(),
                            );
                        }
                        ui.end_row();
                        for view_row in 0..SHEET_VIEW_ROWS {
                            let row = view.origin_row + view_row;
                            ui.label(
                                egui::RichText::new(format!("{}", row + 1))
                                    .color(txt_dim)
                                    .size(TYPE_XS),
                            );
                            for view_col in 0..cols {
                                let col = view.origin_col + view_col;
                                let source = app.document.get_spreadsheet_cell(row, col);
                                let draft = edit
                                    .drafts
                                    .entry((row, col))
                                    .or_insert_with(|| source.clone());
                                if !edit.errors.contains_key(&(row, col)) && *draft != source {
                                    // Fuente cambió por fuera (otra celda/undo): re-sincroniza.
                                    *draft = source.clone();
                                }
                                let computed = app.document.eval_spreadsheet_cell(row, col);
                                if edit.editing == Some((row, col)) {
                                    let mut text = draft.clone();
                                    let resp = ui.add_sized(
                                        [cell_w, SHEET_CELL_H],
                                        egui::TextEdit::singleline(&mut text)
                                            .hint_text("=A1+B1")
                                            .vertical_align(egui::Align::Center)
                                            .font(egui::FontId::monospace(TYPE_XS)),
                                    );
                                    // A11Y foco visible en celda (anillo 2px del tema,
                                    // ref `toolbar.rs:714`).
                                    if resp.has_focus() {
                                        current_theme(ui.ctx())
                                            .paint_focus_ring(ui.painter(), resp.rect);
                                    }
                                    if !resp.has_focus() && !resp.lost_focus() {
                                        resp.request_focus();
                                    }
                                    if resp.changed() {
                                        *draft = text.clone();
                                        edit.errors.remove(&(row, col));
                                    }
                                    let enter = resp.has_focus()
                                        && ui.input(|input| input.key_pressed(egui::Key::Enter));
                                    if enter && *draft != source {
                                        dirty.push((row, col, draft.clone()));
                                        edit.editing = None;
                                    } else if resp.lost_focus() {
                                        if *draft != source {
                                            dirty.push((row, col, draft.clone()));
                                        }
                                        edit.editing = None;
                                    }
                                } else {
                                    // Lectura: valor calculado (o "—" / vacío).
                                    // Puro en `sheet_layout::sheet_cell_display`
                                    // (slice 12 F14.4.4); acá solo el color.
                                    let (display, hover, kind) = sheet_cell_display(
                                        computed,
                                        &source,
                                        edit.errors.get(&(row, col)).map(String::as_str),
                                        format_statistic,
                                    );
                                    let color = if kind == SheetCellKind::Value {
                                        txt_col
                                    } else {
                                        txt_dim
                                    };
                                    if edit.errors.contains_key(&(row, col)) {
                                        let resp = ui.add_sized(
                                            [cell_w, SHEET_CELL_H],
                                            egui::Label::new(
                                                egui::RichText::new("—")
                                                    .color(theme.danger)
                                                    .size(TYPE_XS)
                                                    .monospace(),
                                            )
                                            .sense(egui::Sense::click()),
                                        );
                                        if resp.clicked() {
                                            edit.editing = Some((row, col));
                                        }
                                        resp.on_hover_text(
                                            edit.errors
                                                .get(&(row, col))
                                                .cloned()
                                                .unwrap_or_default(),
                                        );
                                    } else {
                                        let resp = ui.add_sized(
                                            [cell_w, SHEET_CELL_H],
                                            egui::Label::new(
                                                egui::RichText::new(&display)
                                                    .color(color)
                                                    .size(TYPE_XS)
                                                    .monospace(),
                                            )
                                            .sense(egui::Sense::click()),
                                        );
                                        if resp.clicked() {
                                            edit.editing = Some((row, col));
                                        }
                                        if !hover.is_empty() {
                                            resp.on_hover_text(hover);
                                        } else if !display.is_empty() {
                                            resp.on_hover_text(format!(
                                                "{}{} = {display}",
                                                sheet_col_label(col),
                                                row + 1,
                                            ));
                                        }
                                    }
                                }
                            }
                            ui.end_row();
                        }
                    });
            }
        });
    ui.add_space(SPACE_SM);
    // “Aplicar” compromete todas las celdas sucias (ordenadas, una por vez
    // para error por celda); Enter ya encoló la celda actual arriba.
    // B-A151: botón localizado (display, storage intacto).
    let sheet_apply_locale = app.config_locale();
    let sheet_apply_label = grafito_ui::i18n::stats_msg("sheet_apply", sheet_apply_locale);
    let sheet_apply_hint = grafito_ui::i18n::stats_msg("sheet_apply_hint", sheet_apply_locale);
    let mut apply_all = panel_primary_button(ui, sheet_apply_label)
        .on_hover_text(sheet_apply_hint)
        .clicked();
    for ((row, col), draft) in &edit.drafts {
        let source = app.document.get_spreadsheet_cell(*row, *col);
        if *draft != source && !dirty.iter().any(|(r, c, _)| r == row && c == col) {
            apply_all = true;
            break;
        }
    }
    if apply_all {
        let mut batch: Vec<(usize, usize, String)> = edit
            .drafts
            .iter()
            .filter_map(|((row, col), draft)| {
                let source = app.document.get_spreadsheet_cell(*row, *col);
                (*draft != source).then(|| (*row, *col, draft.clone()))
            })
            .collect();
        batch.sort_unstable_by_key(|(row, col, _)| (*row, *col));
        dirty.extend(batch);
        dirty.sort_unstable_by_key(|(row, col, _)| (*row, *col));
        dirty.dedup_by_key(|(row, col, _)| (*row, *col));
    }
    if !dirty.is_empty() {
        snapshot.capture(&app.document);
        let mut ok = 0usize;
        for (row, col, value) in dirty {
            let label = format!("{}{}", sheet_col_label(col), row + 1);
            match app
                .document
                .stage_spreadsheet_cell_edits(&[(row, col, value)])
            {
                Ok(staged) => {
                    app.document = staged;
                    edit.drafts.remove(&(row, col));
                    edit.errors.remove(&(row, col));
                    ok += 1;
                }
                Err(error) => {
                    edit.errors.insert((row, col), format!("{label}: {error}"));
                }
            }
        }
        if ok > 0 {
            app.cas_result = format!("Hoja: {ok} celda(s) actualizada(s)");
        } else if let Some(((row, col), _)) = edit.drafts.iter().next() {
            let _ = (row, col);
        }
        snapshot.save_if_semantically_changed(
            &mut app.document,
            &mut app.undo_stack,
            &mut app.redo_stack,
        );
        ctx.request_repaint();
    }
    ctx.data_mut(|data| data.insert_temp(id, edit));
    ctx.data_mut(|data| data.insert_temp(view_id, view));
}

/// Autorrelleno por serie, rediseño v2 (frente C2): mismo motor que el
/// comando `FillSeries` (lineal `inicio+paso·i` o geométrica `inicio·pasoⁱ`
/// sobre un rango 1D). Commit atómico con undo; el error queda en la fila.
/// Estructura fija de 5 bloques que no depende del ancho —cero píxeles
/// fijos, todo proporcional— así no hay desborde ni en el drawer compacto:
/// tipo, rango, inicio+paso, vista previa en vivo y aplicar.
fn draw_sheet_series_row(ui: &mut egui::Ui, app: &mut GrafitoApp) {
    let ctx = ui.ctx().clone();
    let (_is_dark, accent, _fill, _sep, _txt_col, txt_dim, _hdr_col) = panel_theme_local(&ctx);
    // B-A150: chrome de la fila de serie localizado (display, storage intacto).
    let series_locale = app.config_locale();
    let series_type_label = grafito_ui::i18n::stats_msg("series_type", series_locale);
    let series_geometric_label = grafito_ui::i18n::stats_msg("series_geometric", series_locale);
    let series_geometric_hint = grafito_ui::i18n::stats_msg("series_geometric_hint", series_locale);
    let series_linear_label = grafito_ui::i18n::stats_msg("series_linear", series_locale);
    let series_range_label = grafito_ui::i18n::stats_msg("series_range", series_locale);
    let series_start_label = grafito_ui::i18n::stats_msg("series_start", series_locale);
    let series_step_label = grafito_ui::i18n::stats_msg("series_step", series_locale);
    let series_apply_label = grafito_ui::i18n::stats_msg("series_apply", series_locale);
    let id = egui::Id::new("gc_sheet_series_state");
    let mut series: SheetSeriesState = ctx
        .data_mut(|data| data.get_temp::<SheetSeriesState>(id))
        .unwrap_or_default();
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = SPACE_XS;
        ui.spacing_mut().item_spacing.x = SPACE_SM;
        // 1 · Tipo: etiqueta a la izquierda, opción a la derecha.
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(series_type_label)
                    .color(txt_dim)
                    .size(TYPE_XS)
                    .strong(),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = SPACE_XS;
                ui.radio_value(&mut series.geometric, true, series_geometric_label)
                    .on_hover_text(series_geometric_hint);
                ui.radio_value(&mut series.geometric, false, series_linear_label);
            });
        });
        // 2 · Rango a todo el ancho.
        ui.label(
            egui::RichText::new(series_range_label)
                .color(txt_dim)
                .size(TYPE_XS),
        );
        let rango_resp = ui.add_sized(
            [ui.available_width(), SHEET_CELL_H],
            egui::TextEdit::singleline(&mut series.range)
                .hint_text("A1:A8")
                .vertical_align(egui::Align::Center)
                .font(egui::FontId::proportional(TYPE_XS)),
        );
        if rango_resp.has_focus() {
            current_theme(&ctx).paint_focus_ring(ui.painter(), rango_resp.rect);
        }
        // 3 · Inicio + Paso en una fila (dos números cortos siempre entran).
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = SPACE_SM;
            // Sin clamp que empuje: a panel angosto cede en vez de desbordar.
            let field_w = sheet_series_field_w(ui.available_width());
            for (label, value, hint) in [
                (series_start_label, &mut series.start, "1"),
                (series_step_label, &mut series.step, "1"),
            ] {
                ui.vertical(|ui| {
                    ui.set_min_width(field_w);
                    ui.label(
                        egui::RichText::new(label)
                            .color(txt_dim)
                            .size(TYPE_XS)
                            .strong(),
                    );
                    let num_resp = ui.add_sized(
                        [field_w, SHEET_CELL_H],
                        egui::TextEdit::singleline(value)
                            .hint_text(hint)
                            .vertical_align(egui::Align::Center)
                            .font(egui::FontId::proportional(TYPE_XS)),
                    );
                    if num_resp.has_focus() {
                        current_theme(&ctx).paint_focus_ring(ui.painter(), num_resp.rect);
                    }
                });
            }
        });
        // 4 · Vista previa en vivo con el motor real (o ayuda si no valida).
        if let Some((count, first)) =
            series_preview(&series.range, &series.start, &series.step, series.geometric)
        {
            let text = format_series_preview_text_for(count, &first, series_locale);
            ui.label(
                egui::RichText::new(text)
                    .color(accent)
                    .size(TYPE_XS)
                    .strong(),
            );
        }
        // 5 · Aplicar.
        let apply = panel_primary_button(ui, series_apply_label).clicked();
        if apply {
            series.error = None;
            let outcome: Result<usize, String> = (|| {
                let edits = build_series_edits(
                    &series.range,
                    &series.start,
                    &series.step,
                    series.geometric,
                )?;
                let count = edits.len();
                let mut snapshot = crate::app::DeferredPanelSnapshot::new(app.undo_stack.len());
                snapshot.capture(&app.document);
                let staged = app.document.stage_spreadsheet_cell_edits(&edits)?;
                app.document = staged;
                snapshot.save_if_semantically_changed(
                    &mut app.document,
                    &mut app.undo_stack,
                    &mut app.redo_stack,
                );
                Ok(count)
            })();
            match outcome {
                Ok(count) => {
                    app.cas_result = format_series_success_for(count, &series.range, series_locale);
                }
                Err(error) => {
                    series.error = Some(error);
                }
            }
            ctx.request_repaint();
        }
    });
    // Slot reservado: la vista previa no salta al validar.
    if let Some(error) = &series.error {
        ui.label(
            egui::RichText::new(error)
                .color(current_theme(&ctx).danger)
                .size(TYPE_XS),
        );
    } else {
        ui.label(egui::RichText::new(" ").size(TYPE_XS));
    }
    ctx.data_mut(|data| data.insert_temp(id, series));
}

/// Tile escandinavo de dato: rótulo micro arriba, valor mono abajo, marco
/// hairline sobre `input_bg`. El ancho lo reparte [`stat_tile_row`].
pub(crate) fn stat_tile(
    ui: &mut egui::Ui,
    width: f32,
    label: &str,
    value: &str,
    accent: egui::Color32,
) {
    let theme = current_theme(ui.ctx());
    let height = HIT_TARGET_AULA;
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        // Clip al tile: un valor largo (`-1.23e+308`) no pinta afuera.
        let painter = ui.painter().with_clip_rect(rect);
        painter.rect_filled(rect, RADIUS_SM, theme.input_bg);
        painter.rect_stroke(rect, RADIUS_SM, theme.hairline_stroke());
        painter.text(
            egui::pos2(rect.left() + SPACE_SM, rect.top() + SPACE_SM),
            egui::Align2::LEFT_TOP,
            label,
            egui::FontId::proportional(TYPE_XS),
            theme.text_tertiary,
        );
        painter.text(
            egui::pos2(rect.left() + SPACE_SM, rect.bottom() - SPACE_SM),
            egui::Align2::LEFT_BOTTOM,
            value,
            egui::FontId::monospace(TYPE_SM),
            accent,
        );
    }
    let _ = resp.on_hover_text(format!("{label}: {value}"));
}

/// Fila de tiles a ancho completo (≤3 por fila, el resto envuelve). Cero
/// píxeles fijos: el ancho sale del disponible real, nunca desborda.
pub(crate) fn stat_tile_row(ui: &mut egui::Ui, items: &[(String, String)], accent: egui::Color32) {
    // Ancho con gutter de scroll descontado: dentro de un ScrollArea el
    // available incluye la zona bajo la barra flotante y los tiles la
    // invadían (se cortaban al borde del panel).
    let content_w = grafito_ui::tokens::scroll_content_width(ui.available_width()).max(0.0);
    let cols = stat_tile_cols(content_w);
    for chunk in items.chunks(cols) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = SPACE_SM;
            let width = stat_tile_width_for(content_w, chunk.len());
            for (label, value) in chunk {
                stat_tile(ui, width, label, value, accent);
            }
        });
    }
}

/// Card «Resumen de rango»: corre [`summarize_range`] al pedir y muestra los
/// cinco estadísticos en tiles. Sin cálculo por frame: solo al confirmar.
fn draw_sheet_summary_card(ui: &mut egui::Ui, app: &mut GrafitoApp) {
    let ctx = ui.ctx().clone();
    let summary_locale = app.config_locale();
    let summary_btn_label = grafito_ui::i18n::stats_msg("sheet_summary_btn", summary_locale);
    let summary_btn_hint = grafito_ui::i18n::stats_msg("sheet_summary_btn_hint", summary_locale);
    let (_is_dark, accent, _fill, _sep, _txt_col, _txt_dim, _hdr_col) = panel_theme_local(&ctx);
    let id = egui::Id::new("gc_sheet_summary_state");
    let mut state: SheetSummaryState = ctx
        .data_mut(|data| data.get_temp::<SheetSummaryState>(id))
        .unwrap_or_default();
    let mut requested = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = SPACE_SM;
        let btn_w = SHEET_SUMMARY_BTN_W;
        let field_w = sheet_summary_field_w(ui.available_width());
        let resp = ui.add_sized(
            [field_w, SHEET_CONTROL_H],
            egui::TextEdit::singleline(&mut state.range)
                .hint_text("A1:A8")
                .vertical_align(egui::Align::Center)
                .font(egui::FontId::monospace(TYPE_XS)),
        );
        if resp.has_focus() {
            current_theme(&ctx).paint_focus_ring(ui.painter(), resp.rect);
        }
        if resp.changed() {
            state.error = None;
        }
        if resp.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)) {
            requested = true;
        }
        if ui
            .add_sized(
                [btn_w, SHEET_CONTROL_H],
                egui::Button::new(egui::RichText::new(summary_btn_label).size(TYPE_XS))
                    .rounding(RADIUS_SM),
            )
            .on_hover_text(summary_btn_hint)
            .clicked()
        {
            requested = true;
        }
    });
    if requested {
        match summarize_range_for(&app.document, &state.range, summary_locale) {
            Ok(summary) => {
                state.summary = Some(summary);
                state.error = None;
            }
            Err(error) => {
                state.summary = None;
                state.error = Some(error);
            }
        }
        ctx.request_repaint();
    }
    if let Some(summary) = &state.summary {
        ui.add_space(SPACE_SM);
        let tiles: Vec<(String, String)> =
            sheet_summary_tiles_for(summary, format_statistic, summary_locale)
                .into_iter()
                .collect();
        stat_tile_row(ui, &tiles, accent);
    }
    // Una línea siempre (error o nbsp): el ejemplo vive en el hint "A1:A8".
    if let Some(error) = &state.error {
        ui.label(
            egui::RichText::new(error)
                .color(current_theme(&ctx).danger)
                .size(TYPE_XS),
        );
    } else {
        ui.label(egui::RichText::new(" ").size(TYPE_XS));
    }
    ctx.data_mut(|data| data.insert_temp(id, state));
}

/// Sección Datos: hoja vinculada editable (celdas) + tablas en lectura.
/// Vive en el tab Datos (casa de Análisis de datos). La hoja edita
/// `Document.spreadsheet` con validación por celda; las tablas (`DataTable`)
/// siguen en lectura con botón de ejemplo real (nunca mudo).
pub(crate) fn draw_spreadsheet_section(ui: &mut egui::Ui, app: &mut GrafitoApp) {
    let ctx = ui.ctx().clone();
    let (_is_dark, _accent, _fill, _sep, txt_col, txt_dim, _hdr_col) = panel_theme_local(&ctx);
    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
    ui.spacing_mut().item_spacing.y = SPACE_SM;
    // Entrada de archivos grandes: CSV/TSV de 2 columnas → DataTable +
    // ScatterPlot enlazado, más puente al asistente para ordenar/analizar.
    // Scandinavian: una card de entrada, dos botones quiet en horizontal
    // (apilados en angosto), sin muros de texto.
    // B-A148: título/subtítulo por `regression_chrome_for` (mismo dispatcher
    // que el panel Regresión, sin cambio de firma).
    let spreadsheet_locale = app.config_locale();
    let spreadsheet_import_title =
        crate::sheet_layout::regression_chrome_for("spreadsheet_import_title", spreadsheet_locale);
    let spreadsheet_import_subtitle = crate::sheet_layout::regression_chrome_for(
        "spreadsheet_import_subtitle",
        spreadsheet_locale,
    );
    draw_inspector_section(
        ui,
        spreadsheet_import_title,
        spreadsheet_import_subtitle,
        |ui| {
            // Apilados a todo ancho, siempre 1 línea: en 2 columnas los labels
            // envolvían ("Importar / CSV/TSV…") y el clamp de 80 desbordaba.
            // B-A147: el botón Importar reutiliza `regression_chrome_for`
            // (misma cadena que el panel Regresión, mismo sentido).
            let regression_locale = app.config_locale();
            let regression_import_label = crate::sheet_layout::regression_chrome_for(
                "regression_import_label",
                regression_locale,
            );
            let regression_import_hover = crate::sheet_layout::regression_chrome_for(
                "regression_import_hover",
                regression_locale,
            );
            // B-A148: el botón del asistente reutiliza `stats.ask_btn/hint`
            // (misma acción y sentido que el bloque del panel de
            // estadística, B-A124; 0 keys nuevas).
            let ask_btn_label = grafito_ui::i18n::t("stats.ask_btn", regression_locale);
            let ask_btn_hint_label = grafito_ui::i18n::t("stats.ask_btn_hint", regression_locale);
            if panel_primary_button(ui, regression_import_label)
                .on_hover_text(regression_import_hover)
                .clicked()
            {
                import_local_xy_table(app, &ctx);
            }
            ui.add_space(SPACE_XS);
            if ui
                .add_sized(
                    [ui.available_width(), PANEL_BUTTON_H],
                    egui::Button::new(egui::RichText::new(ask_btn_label).size(TYPE_SM))
                        .wrap_mode(egui::TextWrapMode::Extend)
                        .truncate()
                        .rounding(RADIUS_SM),
                )
                .on_hover_text(ask_btn_hint_label)
                .clicked()
            {
                open_data_tables_in_assistant(app, &ctx);
            }
        },
    );
    ui.add_space(SPACE_SM);
    // B-A151: headers de sección localizados (display, storage intacto).
    let sheet_linked_title = grafito_ui::i18n::stats_msg("sheet_linked_title", spreadsheet_locale);
    let sheet_linked_hint = grafito_ui::i18n::stats_msg("sheet_linked_hint", spreadsheet_locale);
    let sheet_series_title = grafito_ui::i18n::stats_msg("sheet_series_title", spreadsheet_locale);
    let sheet_summary_title =
        grafito_ui::i18n::stats_msg("sheet_summary_title", spreadsheet_locale);
    let sheet_tables_title = grafito_ui::i18n::stats_msg("sheet_tables_title", spreadsheet_locale);
    let sheet_tables_subtitle =
        grafito_ui::i18n::stats_msg("sheet_tables_subtitle", spreadsheet_locale);
    draw_inspector_section(ui, sheet_linked_title, "", |ui| {
        ui.label(
            egui::RichText::new(sheet_linked_hint)
                .color(txt_dim)
                .size(TYPE_XS),
        )
        .on_hover_text(grafito_ui::i18n::stats_msg(
            "sheet_linked_long_hint",
            spreadsheet_locale,
        ));
        draw_sheet_editable_grid(ui, app);
    });
    ui.add_space(SPACE_SM);
    draw_inspector_section(ui, sheet_series_title, "", |ui| {
        draw_sheet_series_row(ui, app);
    });
    ui.add_space(SPACE_SM);
    draw_inspector_section(ui, sheet_summary_title, "", |ui| {
        draw_sheet_summary_card(ui, app);
    });
    ui.add_space(SPACE_XS);

    let tables: Vec<SheetTable> = collect_sheet_tables(&app.document);

    draw_inspector_section(ui, sheet_tables_title, sheet_tables_subtitle, |ui| {
        if tables.is_empty() {
            // B-A152: vacíos + ejemplo localizados (display, storage intacto).
            let tables_empty =
                grafito_ui::i18n::stats_msg("sheet_tables_empty", spreadsheet_locale);
            let tables_empty_hint =
                grafito_ui::i18n::stats_msg("sheet_tables_empty_hint", spreadsheet_locale);
            let tables_example =
                grafito_ui::i18n::stats_msg("sheet_tables_example", spreadsheet_locale);
            let tables_example_hint =
                grafito_ui::i18n::stats_msg("sheet_tables_example_hint", spreadsheet_locale);
            // Misma tarjeta quiet que el vacío de estadística.
            let empty_theme = current_theme(ui.ctx());
            egui::Frame::none()
                .fill(empty_theme.input_bg)
                .stroke(egui::Stroke::NONE)
                .rounding(egui::Rounding::same(RADIUS_MD))
                .inner_margin(egui::Margin::same(SPACE_MD))
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new(tables_empty)
                                .color(empty_theme.text_primary)
                                .size(TYPE_SM)
                                .strong(),
                        );
                        ui.add_space(SPACE_XS);
                        ui.label(
                            egui::RichText::new(tables_empty_hint)
                                .color(txt_dim)
                                .size(TYPE_XS),
                        );
                    });
                });
            ui.add_space(SPACE_SM);
            if panel_primary_button(ui, tables_example)
                .on_hover_text(tables_example_hint)
                .clicked()
            {
                let example = example_sheet_table();
                match app.document.try_add_object(example) {
                    Ok(_) => {
                        app.cas_result =
                            example_sheet_table_loaded_text_for(spreadsheet_locale).to_string();
                        app.notify(
                            app.cas_result.clone(),
                            grafito_ui::toast::ToastKind::Success,
                        );
                    }
                    Err(error) => {
                        app.cas_result =
                            example_sheet_table_error_text_for(&error, spreadsheet_locale);
                        app.notify(app.cas_result.clone(), grafito_ui::toast::ToastKind::Error);
                    }
                }
            }
            return;
        }

        let hidden_overflow = sheet_hidden_tables_overflow_for(tables.len(), spreadsheet_locale);
        for table in tables.into_iter().take(MAX_SPREADSHEET_TABLES) {
            let SheetTable {
                id,
                label,
                x_name,
                y_name,
                xs,
                ys,
            } = table;
            let _ = id;
            let title = sheet_table_title_for(&label, spreadsheet_locale);
            let rows = sheet_table_rows(xs.len(), ys.len());
            ui.label(
                egui::RichText::new(sheet_table_subtitle_for(&title, rows, spreadsheet_locale))
                    .color(txt_col)
                    .size(TYPE_SM)
                    .strong(),
            );
            let shown = sheet_table_shown(rows);
            // Sin scroll: 3 columnas de nombres acotados + números
            // siempre entran (igual que la hoja editable).
            {
                egui::Grid::new(format!("gc_sheet_{title}"))
                    .num_columns(3)
                    .striped(true)
                    .spacing([SPACE_SM, SPACE_XS])
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("#").color(txt_dim).size(TYPE_XS));
                        ui.label(
                            egui::RichText::new(truncate_table_name(&x_name))
                                .color(txt_dim)
                                .size(TYPE_XS),
                        )
                        .on_hover_text(x_name);
                        ui.label(
                            egui::RichText::new(truncate_table_name(&y_name))
                                .color(txt_dim)
                                .size(TYPE_XS),
                        )
                        .on_hover_text(y_name);
                        ui.end_row();
                        for row in 0..shown {
                            ui.label(
                                egui::RichText::new(format!("{}", row + 1))
                                    .color(txt_dim)
                                    .size(TYPE_XS),
                            );
                            ui.label(
                                egui::RichText::new(format_statistic(xs[row]))
                                    .color(txt_col)
                                    .size(TYPE_XS)
                                    .monospace(),
                            );
                            ui.label(
                                egui::RichText::new(format_statistic(ys[row]))
                                    .color(txt_col)
                                    .size(TYPE_XS)
                                    .monospace(),
                            );
                            ui.end_row();
                        }
                    });
            }
            if let Some(overflow) = sheet_table_rows_overflow_for(rows, shown, spreadsheet_locale) {
                ui.label(egui::RichText::new(overflow).color(txt_dim).size(TYPE_XS));
            }
            ui.add_space(SPACE_XS);
        }
        if let Some(overflow) = hidden_overflow {
            ui.label(egui::RichText::new(overflow).color(txt_dim).size(TYPE_XS));
        }
    });
}

/// Tab Datos: hoja de cálculo vinculada + tablas del documento.
/// Casa de la perspectiva Análisis de datos; accesible global como tab 2.
/// La hoja navegable y la serie viven en `draw_spreadsheet_section`.
pub(crate) fn draw_data_panel(app: &mut GrafitoApp, ctx: &egui::Context) {
    let (_is_dark, _accent, alg_fill, _sep_col, _txt_col, _txt_dim, _hdr_col) =
        panel_theme_local(ctx);

    egui::SidePanel::left("data_panel")
        .show_separator_line(false)
        .exact_width(crate::panels::panel_left_width(ctx))
        .resizable(false)
        .frame(
            egui::Frame::none()
                .fill(alg_fill)
                .stroke(egui::Stroke::NONE),
        )
        .show(ctx, |ui| {
            // Sin header: el rail dice "Datos" y el tab activo cierra.
            ui.add_space(SPACE_SM);
            // Aire derecho 24 para la flotante (ver tokens).
            egui::ScrollArea::vertical()
                .id_salt("data_panel_scroll")
                .auto_shrink([false, false])
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                .show(ui, |ui| {
                    egui::Frame::none()
                        .inner_margin(egui::Margin {
                            left: SPACE_MD,
                            right: SPACE_XS,
                            top: SPACE_SM,
                            bottom: SPACE_LG,
                        })
                        .show(ui, |ui| {
                            draw_spreadsheet_section(ui, app);
                        });
                });
        });
}

#[cfg(test)]
mod tests {
    #[test]
    fn stats_b_a147_spreadsheet_import_reutiliza_regression_msg() {
        // B-A147: el botón Importar de `draw_spreadsheet_section` reutiliza
        // `stats.regression_import_label/hover` (misma cadena que el panel
        // Regresión). Sin egui: pines + blindaje por `include_str!`.
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(
            t("stats.regression_import_label", Locale::Es),
            ["Importar CSV/", "TSV…"].concat()
        );
        assert_eq!(
            t("stats.regression_import_hover", Locale::Es),
            ["Dos columnas x,y · la ruta nunca se ", "guarda"].concat()
        );
        assert_eq!(
            t("stats.regression_import_label", Locale::En),
            "Import CSV/TSV…"
        );
        let source = include_str!("spreadsheet.rs");
        for pin in [
            "let regression_import_label = crate::sheet_layout::regression_chrome_for(",
            "let regression_import_hover = crate::sheet_layout::regression_chrome_for(",
            "\"regression_import_label\",",
            "\"regression_import_hover\",",
            "panel_primary_button(ui, regression_import_label)",
            ".on_hover_text(regression_import_hover)",
        ] {
            assert!(source.contains(pin), "spreadsheet B-A147 cablea {pin}");
        }
        // Blindaje: las consts ya no están en el draw (imports podados).
        // Splits para que `include_str!` no se auto-matchee (precedente B-A124).
        for needle in [
            ["REGRESSION_IMPORT_L", "ABEL"].concat(),
            ["REGRESSION_IMPORT_H", "OVER"].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "spreadsheet sin ES duro en prod (B-A147): {needle}"
            );
        }
    }

    #[test]
    fn stats_b_a148_spreadsheet_header_sale_de_regression_msg() {
        // B-A148: título/subtítulo de la sección de import salen de
        // `stats.spreadsheet_import_title/subtitle` vía
        // `regression_chrome_for` (mismo dispatcher que el panel
        // Regresión, sin cambio de firma). Sin egui: pines + blindaje
        // por `include_str!`.
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(
            t("stats.spreadsheet_import_title", Locale::Es),
            ["Importar y ", "ordenar"].concat()
        );
        assert_eq!(
            t("stats.spreadsheet_import_subtitle", Locale::Es),
            ["Máx. 2 ", "MB."].concat()
        );
        assert_eq!(
            t("stats.spreadsheet_import_title", Locale::En),
            "Import and sort"
        );
        assert_eq!(
            t("stats.spreadsheet_import_subtitle", Locale::Fr),
            "Max 2 Mo."
        );
        let source = include_str!("spreadsheet.rs");
        for pin in [
            "let spreadsheet_import_title = crate::sheet_layout::regression_chrome_for(",
            "let spreadsheet_import_subtitle = crate::sheet_layout::regression_chrome_for(",
            "\"spreadsheet_import_title\",",
            "\"spreadsheet_import_subtitle\",",
            "draw_inspector_section(ui, spreadsheet_import_title, spreadsheet_import_subtitle,",
            "let ask_btn_label = grafito_ui::i18n::t(\"stats.ask_btn\", regression_locale);",
            "let ask_btn_hint_label = grafito_ui::i18n::t(\"stats.ask_btn_hint\", regression_locale);",
            "egui::RichText::new(ask_btn_label)",
            ".on_hover_text(ask_btn_hint_label)",
        ] {
            assert!(source.contains(pin), "spreadsheet B-A148 cablea {pin}");
        }
        // Blindaje: los literales ES ya no están en el draw (consts
        // canónicas intactas en sheet_layout.rs, pineadas por su test;
        // `ask_btn/hint` reutilizan B-A124, pineadas por su test).
        // Splits para que `include_str!` no se auto-matchee.
        for needle in [
            ["Importar y ", "ordenar"].concat(),
            ["Máx. 2 ", "MB."].concat(),
            ["Ordenar con asi", "stente"].concat(),
            ["Abre el asistente con tus ta", "blas"].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "spreadsheet sin ES duro en prod (B-A148): {needle}"
            );
        }
    }

    #[test]
    fn stats_b_a150_serie_sale_de_stats_msg() {
        // B-A150: chrome de la fila de serie sale de `stats.series_*` vía
        // `stats_msg` + helpers `_for` (Es delega al canónico). Sin egui:
        // pines + blindaje por `include_str!` con `concat!` anti-auto-match
        // (precedente B-A89/B-A105).
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(t("stats.series_type", Locale::Es), ["Ti", "po"].concat());
        assert_eq!(
            t("stats.series_apply", Locale::Es),
            ["Aplicar ", "serie"].concat()
        );
        assert_eq!(t("stats.series_linear", Locale::En), "Linear");
        let source = include_str!("spreadsheet.rs");
        for pin in [
            "let series_locale = app.config_locale();",
            "\"series_type\", series_locale",
            "\"series_geometric\", series_locale",
            "\"series_geometric_hint\", series_locale",
            "\"series_linear\", series_locale",
            "\"series_range\", series_locale",
            "\"series_start\", series_locale",
            "\"series_step\", series_locale",
            "\"series_apply\", series_locale",
            "format_series_preview_text_for(count, &first, series_locale)",
            "format_series_success_for(count, &series.range, series_locale)",
            "panel_primary_button(ui, series_apply_label)",
        ] {
            assert!(source.contains(pin), "spreadsheet B-A150 cablea {pin}");
        }
        // Blindaje: los 8 literales ES ya no están en el draw (canónicos
        // intactos en sheet_layout.rs, pineados por su test).
        for needle in [
            ["RichText::new(\"Ti", "po\")"].concat(),
            ["radio_value(&mut series.geometric, true, \"Ge", "om.\")"].concat(),
            ["Geométrica: inicio·pa", "soⁱ"].concat(),
            ["radio_value(&mut series.geometric, false, \"Lin", "eal\")"].concat(),
            ["RichText::new(\"Ran", "go\")"].concat(),
            ["(\"Ini", "cio\", &mut series.start"].concat(),
            ["(\"Pa", "so\", &mut series.step"].concat(),
            ["panel_primary_button(ui, \"Aplicar ", "serie\")"].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "spreadsheet sin ES duro en prod (B-A150): {needle}"
            );
        }
    }

    #[test]
    fn stats_b_a151_hoja_chrome_sale_de_stats_msg() {
        // B-A151: headers de sección + botón Aplicar salen de
        // `stats.sheet_*` vía `stats_msg` (display, storage intacto).
        // Sin egui: pines + blindaje por `include_str!` con `concat!`
        // anti-auto-match (precedente B-A89/B-A105/B-A150).
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(
            t("stats.sheet_linked_title", Locale::Es),
            ["Hoja vinc", "ulada"].concat()
        );
        assert_eq!(
            t("stats.sheet_apply", Locale::Es),
            ["Aplicar ", "hoja"].concat()
        );
        assert_eq!(t("stats.sheet_series_title", Locale::En), "Series");
        assert_eq!(
            t("stats.sheet_summary_title", Locale::Fr),
            "Résumé de la plage"
        );
        let source = include_str!("spreadsheet.rs");
        for pin in [
            "let sheet_apply_locale = app.config_locale();",
            "\"sheet_apply\", sheet_apply_locale",
            "\"sheet_apply_hint\", sheet_apply_locale",
            "panel_primary_button(ui, sheet_apply_label)",
            ".on_hover_text(sheet_apply_hint)",
            "\"sheet_linked_title\", spreadsheet_locale",
            "\"sheet_linked_hint\", spreadsheet_locale",
            "\"sheet_series_title\", spreadsheet_locale",
            "\"sheet_summary_title\", spreadsheet_locale",
            "\"sheet_tables_title\", spreadsheet_locale",
            "\"sheet_tables_subtitle\", spreadsheet_locale",
            "draw_inspector_section(ui, sheet_linked_title,",
            "draw_inspector_section(ui, sheet_series_title,",
            "draw_inspector_section(ui, sheet_summary_title,",
            "sheet_tables_title, sheet_tables_subtitle,",
        ] {
            assert!(source.contains(pin), "spreadsheet B-A151 cablea {pin}");
        }
        // Blindaje: los 8 literales ES ya no están en el draw.
        for needle in [
            ["panel_primary_button(ui, \"Aplicar ", "hoja\")"].concat(),
            ["Compromete las celdas editadas (una por vez, con un", "do)"].concat(),
            ["draw_inspector_section(ui, \"Hoja vinc", "ulada\""].concat(),
            ["Clic edita · Enter confirma · `—` = sin res", "olver"].concat(),
            ["draw_inspector_section(ui, \"Se", "rie\""].concat(),
            ["draw_inspector_section(ui, \"Resumen de ra", "ngo\""].concat(),
            ["\"Tab", "las\","].concat(),
            [
                "Solo lectura; las celdas se editan en la hoja de ar",
                "riba.\"",
            ]
            .concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "spreadsheet sin ES duro en prod (B-A151): {needle}"
            );
        }
    }

    #[test]
    fn stats_b_a152_hoja_tablas_salen_de_stats_msg() {
        // B-A152: vacíos + ejemplo + título/subtítulo + overflows + loaded/
        // error salen de `stats.sheet_*` vía `stats_msg`/`*_for` (display,
        // storage intacto). Sin egui: pines + blindaje por `include_str!`
        // con `concat!` anti-auto-match (precedente B-A89/B-A151).
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(
            t("stats.sheet_tables_empty", Locale::Es),
            ["Sin tablas tod", "avía"].concat()
        );
        assert_eq!(t("stats.sheet_tables_example", Locale::En), "Load example");
        assert_eq!(
            t("stats.sheet_table_untitled", Locale::It),
            "<senza etichetta>"
        );
        let source = include_str!("spreadsheet.rs");
        for pin in [
            "\"sheet_tables_empty\", spreadsheet_locale",
            "\"sheet_tables_empty_hint\", spreadsheet_locale",
            "\"sheet_tables_example\", spreadsheet_locale",
            "\"sheet_tables_example_hint\", spreadsheet_locale",
            "sheet_table_title_for(&label, spreadsheet_locale)",
            "sheet_table_subtitle_for(&title, rows, spreadsheet_locale)",
            "sheet_table_rows_overflow_for(rows, shown, spreadsheet_locale)",
            "sheet_hidden_tables_overflow_for(tables.len(), spreadsheet_locale)",
            "example_sheet_table_loaded_text_for(spreadsheet_locale)",
            "example_sheet_table_error_text_for(&error, spreadsheet_locale)",
        ] {
            assert!(source.contains(pin), "spreadsheet B-A152 cablea {pin}");
        }
        // Blindaje: los 4 literales ES del vacío/ejemplo ya no están en el draw.
        for needle in [
            ["RichText::new(\"Sin tablas tod", "avía\")"].concat(),
            ["Creá una desde la entrada con DataTable", "[…].\")"].concat(),
            ["panel_primary_button(ui, \"Cargar ej", "emplo\")"].concat(),
            ["Inserta una tabla de ejemplo (x: 0..4, y: x", "²)\")"].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "spreadsheet sin ES duro en prod (B-A152): {needle}"
            );
        }
    }

    #[test]
    fn stats_b_a154_resumen_card_sale_de_stats_msg() {
        // B-A154: botón de resumen + hover + hover largo + errores + tiles
        // salen de `stats.sheet_*` vía `stats_msg`/`*_for` (display,
        // storage intacto). Sin egui: pines + blindaje por
        // `include_str!` con `concat!` anti-auto-match (precedente
        // B-A89/B-A151/B-A152).
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(
            t("stats.sheet_summary_btn", Locale::Es),
            ["Resu", "mir"].concat()
        );
        assert_eq!(
            t("stats.sheet_summary_tile_sd", Locale::Es),
            ["Desvío ", "s"].concat()
        );
        assert_eq!(t("stats.sheet_summary_btn", Locale::En), "Summarize");
        assert_eq!(t("stats.sheet_summary_tile_mean", Locale::De), "Mittelwert");
        let source = include_str!("spreadsheet.rs");
        for pin in [
            "let summary_locale = app.config_locale();",
            "\"sheet_summary_btn\", summary_locale",
            "\"sheet_summary_btn_hint\", summary_locale",
            "summarize_range_for(&app.document, &state.range, summary_locale)",
            "sheet_summary_tiles_for(summary, format_statistic, summary_locale)",
            "\"sheet_linked_long_hint\",",
            "spreadsheet_locale",
        ] {
            assert!(source.contains(pin), "spreadsheet B-A154 cablea {pin}");
        }
        // Blindaje: los literales ES ya no están en el draw (canónicos
        // intactos en sheet_layout.rs, pineados por su test).
        for needle in [
            ["Resu", "mir"].concat(),
            ["máximo del rango ", "1D"].concat(),
            ["se recomputa solo: puntos, ", "funciones con"].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "spreadsheet sin ES duro en prod (B-A154): {needle}"
            );
        }
    }

    #[test]
    fn stats_b_a164_goto_sale_de_stats_msg() {
        // B-A164: hint + botón del goto salen de `stats.sheet_goto_*` vía
        // `stats_msg` (display, storage intacto; "A1" es referencia de
        // celda invariante, precedente B-A98/B-A114). Sin egui: pines +
        // blindaje por `include_str!` con `concat!` anti-auto-match
        // (precedente B-A89/B-A151/B-A152).
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(
            t("stats.sheet_goto_hint", Locale::Es),
            ["Ir a celda ", "(C3)"].concat()
        );
        assert_eq!(t("stats.sheet_goto_go", Locale::Es), "Ir");
        assert_eq!(t("stats.sheet_goto_hint", Locale::En), "Go to cell (C3)");
        assert_eq!(t("stats.sheet_goto_go", Locale::De), "Los");
        let source = include_str!("spreadsheet.rs");
        for pin in [
            "let goto_locale = app.config_locale();",
            "\"sheet_goto_hint\", goto_locale",
            "\"sheet_goto_go\", goto_locale",
            ".hint_text(goto_hint)",
            "RichText::new(goto_go)",
        ] {
            assert!(source.contains(pin), "spreadsheet B-A164 cablea {pin}");
        }
        // Blindaje: los 2 literales ES ya no están en el draw ("A1" es
        // notación invariante y queda cruda a propósito).
        for needle in [
            ["hint_text(\"Ir a celda ", "(C3)\")"].concat(),
            ["RichText::new(\"I", "r\")"].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "spreadsheet sin ES duro en prod (B-A164): {needle}"
            );
        }
    }

    #[test]
    fn stats_b_a166_nav_sale_de_stats_msg() {
        // B-A166: los 4 hovers de las flechas de navegación salen de
        // `stats.sheet_nav_*` vía `stats_msg` (display, storage intacto;
        // "◀/▶/▲/▼" son glifos invariantes, precedente B-A98/B-A114). Sin
        // egui: pines + blindaje por `include_str!` con `concat!`
        // anti-auto-match (precedente B-A89/B-A164).
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(
            t("stats.sheet_nav_back", Locale::Es),
            ["Retroceder ", "columnas"].concat()
        );
        assert_eq!(
            t("stats.sheet_nav_forward", Locale::En),
            "Move columns forward"
        );
        assert_eq!(t("stats.sheet_nav_up", Locale::De), "8 Zeilen nach oben");
        let source = include_str!("spreadsheet.rs");
        for pin in [
            "let nav_back = grafito_ui::i18n::stats_msg(\"sheet_nav_back\", goto_locale);",
            "let nav_forward = grafito_ui::i18n::stats_msg(\"sheet_nav_forward\", goto_locale);",
            "let nav_up = grafito_ui::i18n::stats_msg(\"sheet_nav_up\", goto_locale);",
            "let nav_down = grafito_ui::i18n::stats_msg(\"sheet_nav_down\", goto_locale);",
            "if arrow(ui, \"◀\", nav_back)",
            "if arrow(ui, \"▶\", nav_forward)",
            "if arrow(ui, \"▲\", nav_up)",
            "if arrow(ui, \"▼\", nav_down)",
        ] {
            assert!(source.contains(pin), "spreadsheet B-A166 cablea {pin}");
        }
        // Blindaje: los 4 literales ES ya no están en el draw (los glifos
        // quedan crudos a propósito).
        for needle in [
            ["\"Retroceder ", "columnas\""].concat(),
            ["\"Avanzar ", "columnas\""].concat(),
            ["\"Subir 8 ", "filas\""].concat(),
            ["\"Bajar 8 ", "filas\""].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "spreadsheet sin ES duro en prod (B-A166): {needle}"
            );
        }
    }

    #[test]
    fn stats_b_a167_chip_hint_sale_de_helper_for() {
        // B-A167: el hover del chip sale de `sheet_chip_hint_text_for` con
        // `goto_locale` (display, storage intacto; notación `400×400`/
        // `10 000`/`(accent)` intacta, precedente B-A98/B-A114). Sin egui:
        // pines + blindaje por `include_str!` con `concat!` anti-auto-match
        // (precedente B-A89/B-A166).
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(
            t("stats.sheet_chip_hint", Locale::Es),
            [
                "Celda en edición (accent) o esquina visible de la hoja 400×400. ",
                "Ventana {ventana} · 10 000 celdas vivas."
            ]
            .concat()
        );
        assert_eq!(
            t("stats.sheet_chip_hint", Locale::En),
            [
                "Cell being edited (accent) or visible corner of the 400×400 sheet. ",
                "Window {ventana} · 10,000 live cells."
            ]
            .concat()
        );
        let source = include_str!("spreadsheet.rs");
        for pin in [
            "sheet_chip_hint_text_for(",
            "&view.window_label(cols),",
            "goto_locale,",
        ] {
            assert!(source.contains(pin), "spreadsheet B-A167 cablea {pin}");
        }
        // Blindaje: el literal ES ya no está en el draw (vive en el helper
        // canónico de `sheet_layout.rs` + la key `stats.sheet_chip_hint`).
        for needle in [
            [
                "\"Celda en edición (accent) o esquina visible ",
                "de la hoja 400×400. Ventana {}",
            ]
            .concat(),
            ["\"Ventana {} · 10 000 ", "celdas vivas.\""].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "spreadsheet sin ES duro en prod (B-A167): {needle}"
            );
        }
    }
}
