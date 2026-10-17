//! Paneles laterales removibles e inspectores (CAS, vista, estadística, propiedades).

use crate::algebra::fit_kind_label_for;
use crate::export::{
    datatable_csv_text, sanitize_export_stem, spawn_csv_export, spawn_pdf_export,
    spawn_png_clipboard,
};
use crate::sheet_layout::{
    attractor_beta_text, attractor_dt_text, attractor_rho_text, attractor_sigma_text,
    attractor_steps_text, binomial_plot_left_label, binomial_plot_masses,
    binomial_plot_right_label, binomial_plot_window, binomial_quantile_honest, cas_cell_input_line,
    cas_cell_number, cas_chrome_for, cas_quick_insert_tooltip_for, cas_quick_label_for,
    cas_sheet_title_for, chi_squared_pdf, chi_squared_plot_hi, chi_squared_quantile_honest,
    clamp_binomial_k, clamp_binomial_n, complex_contour_armed_text,
    complex_contour_last_value_text, complex_quick_hover_text, construction_log_to_latex,
    continuous_density_samples, data_tables_assistant_prompt, discrete_k_sel,
    discrete_plot_left_label, discrete_plot_right_label, discrete_ymax, distribution_moments,
    domain_coloring_mode_label, dynamics_chrome_for, exponential_quantile_honest,
    export_clipboard_copied_text_for, export_grid_cell_w, export_grid_columns,
    export_svg_copied_text_for, export_svg_error_text_for, f_distribution_pdf, f_plot_hi,
    f_quantile_honest, format_statistic, geometric_plot_hi, geometric_plot_masses,
    geometric_quantile_honest, implicit_3d_box_text_for, implicit_3d_expr_text,
    implicit_3d_preview_text_for, implicit_3d_state_text_for, implicit_cancel_label_for,
    implicit_quick_preview_notify, implicit_quick_preview_notify_for, inspector_3d_tag_label_for,
    inspector_chrome_for, inspector_cube_size_range, inspector_equation_text,
    inspector_example_tooltip_for, inspector_identity_tooltip_for, inspector_label_text_for,
    inspector_measure_text_for, inspector_point_size_range, inspector_poly_scale_range,
    inspector_poly_width_range, inspector_sphere_radius_range, inspector_type_caption,
    inspector_visibility_label_for, layer_assign_error_for, layer_assign_result_for,
    layer_row_label_for, layer_selection_text_for, layer_toggle_label_for, layer_toggle_result_for,
    line_style_label_for, live_param_error_text_for, live_param_hover_text_for,
    live_param_readout_text_for, local_xy_table_objects, nd_rotation_plane_label,
    normal_band_fractions, normal_pdf, normal_plot_window, normal_quantile_honest,
    params_error_text_for, parse_local_xy_table, parse_statistics_input,
    plot_df_or_fuera_de_cota_for, plot_message_text_for, point_style_label_for, poisson_plot_hi,
    poisson_plot_masses, poisson_quantile_honest, polychoron_kind_label_for,
    polytope_nd_family_label_for, polytope_nd_projection_body_for, prob_case_command,
    prob_case_copy_text, prob_case_detail_tooltip, prob_case_result, prob_dist_name_for,
    prob_dist_tip_for, prob_dist_tooltip_for, prob_moment_text, prob_plot_bar_text_for,
    prob_point_label_for, prob_quantile_command, prob_tail_text, prob_tail_values,
    protocol_advance, protocol_counter_text, protocol_export_hover, protocol_play_glyph,
    protocol_step_badge, protocol_step_io_text, protocol_steps_title,
    regression_fit_diagnostics_line_for, regression_fit_headline, regression_model_commands_for,
    regression_model_ready_text_for, regression_residual_line, regression_residuals_title_for,
    regression_residuals_truncated_for, regression_section_title_for, regression_source_label_for,
    regression_template_ready_text_for, regression_templates_for, set_complex_mapping_animation,
    set_domain_coloring_mode, statistics_assistant_prompt, statistics_boxplot_command,
    statistics_boxplot_ready_for, statistics_histogram_command, statistics_histogram_counts,
    statistics_histogram_ready_for, statistics_inline_list, statistics_inline_too_many_for,
    statistics_mean_command, statistics_mean_ready_for, statistics_no_inline_hint_for,
    statistics_no_inline_result_for, statistics_quick_chips_for, statistics_quick_ready_text_for,
    statistics_summary, statistics_summary_rows, student_t_pdf, student_t_quantile_honest,
    trig_angle_text, trig_complex_text, trig_function_chip_label, trig_function_columns,
    trig_grid_cell_w, trig_point_text_for, trig_prompt_apply_label, trig_prompt_apply_tooltip,
    trig_prompt_auto_label, trig_prompt_field_tooltip, trig_prompt_field_w, trig_prompt_hint_text,
    trig_speed_text, trig_value_text_for, uniform_pdf, uniform_quantile_honest,
    view_indicate_error_for, wc_exact_integral_command_text, wc_quadrature_key,
    wc_quadrature_method, wc_riemann_command_text, wc_study_command_text, wc_taylor_command_text,
    wc_taylor_remainder_line, LayerPanelState, LocalXYTable, ProbabilityPanelState,
    ProtocolNavState, TrigPromptState, WcVisualState, ATTRACTOR_DEFAULT_COMMAND, CAS_ENTRY_HINT,
    CAS_ENTRY_KEYS_HINT, CAS_QUICK_ACTIONS, COMPLEX_BASE_SYMBOL_HINT, COMPLEX_BASE_SYMBOL_TITLE,
    COMPLEX_CONTOUR_CIRCLE_LABEL, COMPLEX_CONTOUR_CIRCLE_TOOLTIP, COMPLEX_CONTOUR_CLOSE_LABEL,
    COMPLEX_CONTOUR_DRAW_LABEL, COMPLEX_CONTOUR_DRAW_TOOLTIP, COMPLEX_CONTOUR_EXPR_HINT,
    COMPLEX_CONTOUR_PREFILL, COMPLEX_CONTOUR_PREFILL_TOOLTIP, COMPLEX_CONTOUR_RESIDUES_LABEL,
    COMPLEX_CONTOUR_SMOOTH_LABEL, COMPLEX_CONTOUR_SNAP_LABEL, COMPLEX_CONTOUR_STABILIZER_LABEL,
    COMPLEX_CONTOUR_TITLE, COMPLEX_OBJECTS_EMPTY, COMPLEX_OBJECTS_TITLE, COMPLEX_PANEL_INPUT_HINT,
    COMPLEX_PANEL_TITLE, COMPLEX_QUICK_HINTS, COMPLEX_QUICK_TITLE, DOMAIN_COLORING_ACTIVE_TITLE,
    DOMAIN_COLORING_COLORS_BODY, DOMAIN_COLORING_COLORS_TITLE, DOMAIN_COLORING_DERIV_BODY,
    DOMAIN_COLORING_DERIV_TITLE, DOMAIN_COLORING_EMPTY_GUIDE, DOMAIN_COLORING_GUIDE_TITLE,
    DOMAIN_COLORING_HUE_LINE, DOMAIN_COLORING_MODE_LABEL, DOMAIN_COLORING_SUBTITLE,
    DOMAIN_COLORING_SYMBOL_HINT, DOMAIN_COLORING_SYMBOL_LABEL, DOMAIN_COLORING_TITLE,
    EMPTY_PANEL_GUIDE, EMPTY_PANEL_TITLE, EXPORT_CSV_LABEL, EXPORT_FILE_BUTTONS,
    EXPORT_PDF_DEFAULT_FILENAME, EXPORT_TEXT_BUTTONS, IMPLICIT_PREVIEW_START_FAILED,
    INSPECTOR_AXIS_PREFIX_X, INSPECTOR_AXIS_PREFIX_Y, INSPECTOR_AXIS_PREFIX_Z,
    INSPECTOR_CENTER_DRAG_SPEED, MAX_BINOMIAL_N, PANEL_BUTTON_H, POLYCHORON_ROTATION_PLANES,
    PROB_PLOT_AXIS_RESERVE, PROB_PLOT_BAR_GAP, PROB_PLOT_BAR_INSET_X, PROB_PLOT_BAR_MIN,
    PROB_PLOT_BAR_ROUNDING, PROB_PLOT_BASELINE_W, PROB_PLOT_H, PROB_PLOT_INSET, PROB_PLOT_LABEL_DY,
    PROB_QUANTILE_ALPHAS, PROTOCOL_CLEAR_HOVER, PROTOCOL_CLEAR_LABEL, PROTOCOL_DISABLED_LABEL,
    PROTOCOL_EMPTY_GUIDE, PROTOCOL_EMPTY_TITLE, PROTOCOL_EXPORTING, PROTOCOL_EXPORT_LABEL,
    PROTOCOL_LAST_GLYPH, PROTOCOL_LAST_TIP, PROTOCOL_PANEL_SUBTITLE, PROTOCOL_PANEL_TITLE,
    PROTOCOL_PLAY_INTERVAL_SECS, PROTOCOL_PLAY_TIP, PROTOCOL_PREV_GLYPH, PROTOCOL_PREV_TIP,
    PROTOCOL_STEPS_HINT, REGRESSION_MAX_RESIDUALS_SHOWN, SHEET_CONTROL_H, STATISTICS_SUMMARY_BINS,
    TRIG_APPLY_W, TRIG_CANVAS_MIN_W, TRIG_CONTROL_H, TRIG_PANEL_DEFAULT_W, TRIG_PANEL_MAX_W,
    TRIG_PANEL_MIN_W, T_PLOT_HI, T_PLOT_LO, WC_QUADRATURE_METHODS, WC_RIEMANN_SLIDER_MAX,
};
use crate::GrafitoApp;
use egui::Color32;
use grafito_core::symbolic::{clipboard_svg, MAX_LAYERS};
use grafito_core::{
    CasWorksheetStatus, ChangeSet, Document, GeoObject, LineStyle, ObjectId, PointStyle,
    RegularPolytopeNDObj,
};
use grafito_geometry::{Color, RegularPolychoron, RegularPolytopeFamily};
use grafito_ui::icons::{action_icon_button, Icon};
use grafito_ui::theme::{current_theme, DARK, LIGHT};
use grafito_ui::tokens::{
    DRAWER_RIGHT_MAX, DRAWER_RIGHT_MIN, HIT_TARGET_MIN, PANEL_LEFT_DEFAULT, RADIUS_LG, RADIUS_MD,
    RADIUS_SM, RANGE_FIELD_W, SCROLL_CONTENT_PAD_R, SPACE_LG, SPACE_MD, SPACE_SM, SPACE_XS,
    SPACE_XXL, TYPE_BASE, TYPE_LG, TYPE_MD, TYPE_SM, TYPE_XS, ZOOM_ICON_HIT,
};
use std::collections::VecDeque;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::time::Duration;

const MAX_LOCAL_DATA_IMPORT_BYTES: usize = 2_000_000;

// (movido a `sheet_layout.rs`: `LayerPanelState`, slice 23 F14.4.4; el
// dibujo `draw_view_panel` + estado efímero `ctx.data` quedan acá)

// ── Helpers escandinavos de panel (una sola fuente de header/botón) ──────
// Todos los paneles izquierdos comparten: título TYPE_LG acento + subtítulo
// terciario con wrap + hairline + aire SPACE_MD. Los botones primarios son
// pill de ancho completo (alto 28 ≥ HIT_TARGET_MIN 24) para que nunca se
// compriman hasta partir el texto ("Asig/nar" en Vista).

// Ancho máximo real de Datos/Prob: ver `sheet_layout.rs` (slice 34 F14.4.4).
// (movido a `sheet_layout.rs`: `PANEL_DATA_MAX_WIDTH`, slice 34 F14.4.4)
// (movido a `sheet_layout.rs`: `PANEL_BUTTON_H`, slice 40 F14.4.4)
// (movidos a `sheet_layout.rs`: `SHEET_CELL_H` + `SHEET_CONTROL_H`, slice 12 F14.4.4)

/// Botón primario pintado a mano: `Button::fill` congela el fill en todos los
/// estados y el primario quedaba estático. Acá hover/pressed/foco/mano.
fn panel_primary_painted(ui: &mut egui::Ui, label: &str, size: egui::FontId) -> egui::Response {
    let theme = current_theme(ui.ctx());
    let (rect, resp) = ui.allocate_exact_size(
        egui::vec2(ui.available_width().max(HIT_TARGET_MIN), PANEL_BUTTON_H),
        egui::Sense::click(),
    );
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    if ui.is_rect_visible(rect) {
        // Reposo sage, hover/pressed profundo.
        let fill = if resp.is_pointer_button_down_on() || resp.hovered() {
            theme.keyboard_enter_hover
        } else {
            theme.keyboard_enter_bg
        };
        ui.painter().rect_filled(rect, RADIUS_SM, fill);
        ui.painter().with_clip_rect(rect).text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            size,
            theme.keyboard_enter_text,
        );
    }
    resp.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label.to_string())
    });
    if resp.has_focus() {
        theme.paint_focus_ring(ui.painter(), resp.rect);
    }
    resp
}

/// Botón primario pill de ancho completo (nunca parte el texto).
pub(crate) fn panel_primary_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    panel_primary_painted(ui, label, egui::FontId::proportional(TYPE_SM))
}

/// Contenido de scroll con aire derecho para la barra flotante (24px):
/// captura el ancho externo, scroll, y contenido acotado a caja menos aire.
/// La barra nunca tapa botones ni texto; el ancho no salta al aparecer.
/// Misma regla en todos los paneles.
fn panel_scroll_content<R>(
    ui: &mut egui::Ui,
    id_salt: &str,
    max_height: Option<f32>,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let mut scroll = egui::ScrollArea::vertical()
        .id_salt(id_salt)
        .auto_shrink([false, false]);
    if let Some(h) = max_height {
        scroll = scroll.max_height(h);
    }
    scroll
        .show(ui, |ui| {
            egui::Frame::none()
                .inner_margin(egui::Margin {
                    left: 0.0,
                    right: grafito_ui::tokens::SCROLL_CONTENT_PAD_R,
                    top: 0.0,
                    bottom: 0.0,
                })
                .show(ui, add_contents)
                .inner
        })
        .inner
}

/// Grilla escandinava de botones: celdas iguales, rectangulares con bordes
/// apenas redondeados (`RADIUS_SM`), una sola línea de texto (nunca
/// envuelve ni desborda la fila) y detalle en el tooltip.
///
/// Devuelve el índice 0-based del botón clickeado, o `None`.
fn panel_button_grid(
    ui: &mut egui::Ui,
    id_salt: &str,
    cols: usize,
    items: &[(&str, &str)],
) -> Option<usize> {
    // Responsive: a menos de 200 px una sola columna (el clamp de 64
    // desbordaba el panel mínimo). Sin mínimo que empuje + última fila
    // siempre cerrada.
    let cols = if ui.available_width() < 200.0 {
        1
    } else {
        cols.max(1)
    };
    let cell_w = ((ui.available_width() - (cols - 1) as f32 * SPACE_SM) / cols as f32).max(0.0);
    let mut clicked = None;
    egui::Grid::new(id_salt)
        .num_columns(cols)
        .spacing([SPACE_SM, SPACE_XS])
        .show(ui, |ui| {
            for (index, (label, tip)) in items.iter().enumerate() {
                let resp = ui
                    .add_sized(
                        [cell_w, PANEL_BUTTON_H],
                        egui::Button::new(egui::RichText::new(*label).size(TYPE_XS))
                            .wrap_mode(egui::TextWrapMode::Extend)
                            .rounding(RADIUS_SM),
                    )
                    .on_hover_text(*tip);
                if resp.clicked() {
                    clicked = Some(index);
                }
                if (index + 1) % cols == 0 || index + 1 == items.len() {
                    ui.end_row();
                }
            }
        });
    clicked
}

/// Botón primario de ancho completo, rectangular con bordes apenas
/// redondeados (mismo lenguaje que la grilla escandinava).
fn panel_wide_action_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    panel_primary_painted(ui, label, egui::FontId::proportional(TYPE_XS))
}

// (movido a `sheet_layout.rs`: `LocalXYTable`, slice 9 F14.4.4)

/// Lleva los datos 1D de Prob. al asistente: arma el prompt puro y abre el
/// workspace. Cero I/O en el panel salvo abrir el drawer.
fn open_statistics_in_assistant(app: &mut GrafitoApp, ctx: &egui::Context) {
    app.assistant.problem = statistics_assistant_prompt(&app.statistics_data);
    app.assistant_visible = true;
    app.open_assistant_workspace();
    app.cas_result = "Datos enviados al asistente para ordenar y analizar.".to_string();
    ctx.request_repaint();
}

/// Lleva las tablas 2D de Datos al asistente: resume hasta 4 tablas
/// (etiqueta + N + preview x/y) en un prompt accionable. Si no hay tablas,
/// el prompt explica cómo importar.
pub(crate) fn open_data_tables_in_assistant(app: &mut GrafitoApp, ctx: &egui::Context) {
    app.assistant.problem = data_tables_assistant_prompt(&app.document);
    app.assistant_visible = true;
    app.open_assistant_workspace();
    app.cas_result = "Tablas enviadas al asistente para ordenar y analizar.".to_string();
    ctx.request_repaint();
}

// (movido a `sheet_layout.rs`: `parse_local_xy_table` + `parse_delimited_row` +
// `parse_local_xy_values`, slice 9 F14.4.4; acá queda solo la I/O acotada)

pub(crate) fn load_local_xy_table(path: &Path) -> Result<LocalXYTable, String> {
    // La lectura vive en worker (`spawn_csv_import`); esta función es pura I/O
    // acotada: symlink_metadata + O_NOFOLLOW, take() 2MB, sin unwrap/panic.
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|error| format!("No se pudo inspeccionar el archivo: {error}"))?;
    if !metadata.file_type().is_file() {
        return Err("La fuente seleccionada debe ser un archivo regular".to_string());
    }

    let mut bytes = Vec::new();
    let file = open_local_data_file(path)?;
    if !file
        .metadata()
        .map_err(|error| format!("No se pudo verificar el archivo abierto: {error}"))?
        .file_type()
        .is_file()
    {
        return Err("La fuente seleccionada debe ser un archivo regular".to_string());
    }
    // I/O limitado a 2MB para no bloquear UI; budgeting verificado via AttachmentLimits
    file.take(MAX_LOCAL_DATA_IMPORT_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("No se pudo leer el archivo: {error}"))?;
    if bytes.len() > MAX_LOCAL_DATA_IMPORT_BYTES {
        return Err(format!(
            "El archivo supera el máximo de {MAX_LOCAL_DATA_IMPORT_BYTES} bytes"
        ));
    }
    let text = String::from_utf8(bytes)
        .map_err(|_| "El archivo debe estar codificado como UTF-8".to_string())?;
    let is_tsv = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("tsv"));
    parse_local_xy_table(&text, if is_tsv { b'\t' } else { b',' })
}

fn open_local_data_file(path: &Path) -> Result<File, String> {
    #[cfg(unix)]
    {
        use std::fs::OpenOptions;
        use std::os::unix::fs::OpenOptionsExt;

        // Do not follow a path swapped to a symlink, and never block the UI
        // thread on a FIFO substituted after the native file selection.
        OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(path)
            .map_err(|error| format!("No se pudo abrir el archivo: {error}"))
    }
    #[cfg(not(unix))]
    {
        File::open(path).map_err(|error| format!("No se pudo abrir el archivo: {error}"))
    }
}

pub(crate) fn commit_local_xy_table(
    document: &mut Document,
    undo_stack: &mut VecDeque<Document>,
    redo_stack: &mut VecDeque<ChangeSet>,
    table: LocalXYTable,
) -> Result<ObjectId, String> {
    // Armado puro en `sheet_layout.rs` (slice 24 F14.4.4, verbatim); acá
    // queda solo el commit con undo.
    let (data_obj, scatter_obj, data_id) = local_xy_table_objects(table);
    crate::app::commit_object_insertions(
        document,
        undo_stack,
        redo_stack,
        vec![data_obj, scatter_obj],
    )?;
    Ok(data_id)
}

pub(crate) fn import_local_xy_table(app: &mut GrafitoApp, ctx: &egui::Context) {
    // FileController: el diálogo rfd queda en UI thread (modal nativo);
    // la lectura ≤2MB va a worker y el commit se aplica en `poll_background_jobs`.
    // S2: sin `desktop-dialogs` devuelve `None` con toast honesto.
    let Some(path) = app.pick_file_native("Datos CSV o TSV", &["csv", "tsv", "txt"]) else {
        return;
    };
    app.pending_import_job = Some(crate::app::PendingImportJob {
        receiver: crate::app::spawn_csv_import(path, ctx),
    });
    app.cas_result = "Importando tabla local…".to_string();
}

#[cfg(test)]
mod local_data_import_tests {
    use super::{commit_local_xy_table, load_local_xy_table, MAX_LOCAL_DATA_IMPORT_BYTES};
    use crate::sheet_layout::{parse_local_xy_table, LocalXYTable};
    use grafito_core::{Document, GeoObject};
    use std::collections::VecDeque;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_IMPORT_TEST_FILE: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn csv_and_tsv_imports_accept_optional_headers_without_retaining_a_path() {
        let csv = parse_local_xy_table("\"time\",\"distance\"\n0,1\n1,3\n2,5\n", b',')
            .expect("CSV data with a header should parse");
        assert_eq!(csv.x_name, "time");
        assert_eq!(csv.y_name, "distance");
        assert_eq!(csv.xs, vec![0.0, 1.0, 2.0]);
        assert_eq!(csv.ys, vec![1.0, 3.0, 5.0]);

        let tsv = parse_local_xy_table("0\t1\n1\t3\n", b'\t')
            .expect("TSV data without a header should parse");
        assert_eq!(tsv.x_name, "x");
        assert_eq!(tsv.y_name, "y");
        assert_eq!(tsv.xs, vec![0.0, 1.0]);
        assert_eq!(tsv.ys, vec![1.0, 3.0]);
    }

    #[test]
    fn local_data_import_rejects_malformed_or_non_finite_rows() {
        for input in ["x,y\n0,1\n1\n", "x,y\n0,NaN\n", "x,y\n0,1\n"] {
            let error = parse_local_xy_table(input, b',')
                .expect_err("invalid local input must be rejected before mutation");
            assert!(!error.is_empty());
        }
    }

    #[test]
    fn local_data_import_cap_pins_2mb() {
        // Onda 2: pinnea el presupuesto user-facing de importación CSV/TSV.
        assert_eq!(MAX_LOCAL_DATA_IMPORT_BYTES, 2_000_000);
    }

    #[test]
    fn local_data_import_enforces_bounds_and_commits_table_and_scatter_once() {
        let path = std::env::temp_dir().join(format!(
            "grafito-local-data-{}-{}.csv",
            std::process::id(),
            NEXT_IMPORT_TEST_FILE.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, vec![b'x'; MAX_LOCAL_DATA_IMPORT_BYTES + 1])
            .expect("oversized fixture writes");
        let error = load_local_xy_table(&path).expect_err("oversized file must be rejected");
        let _ = std::fs::remove_file(&path);
        assert!(error.contains("máximo"));

        let mut too_many_rows = String::from("x,y\n");
        for index in 0..=grafito_core::validation::MAX_DATA_TABLE_ROWS {
            too_many_rows.push_str(&format!("{index},{}\n", index + 1));
        }
        assert!(parse_local_xy_table(&too_many_rows, b',').is_err());

        let mut document = Document::new();
        let mut undo_stack = VecDeque::new();
        let mut redo_stack = VecDeque::new();
        let data_id = commit_local_xy_table(
            &mut document,
            &mut undo_stack,
            &mut redo_stack,
            LocalXYTable {
                x_name: "time".to_string(),
                y_name: "distance".to_string(),
                xs: vec![0.0, 1.0, 2.0],
                ys: vec![1.0, 3.0, 5.0],
            },
        )
        .expect("valid local data commits");

        assert_eq!(undo_stack.len(), 1);
        assert!(redo_stack.is_empty());
        assert!(matches!(
            document.get_object(data_id),
            Some(GeoObject::DataTable(_))
        ));
        assert!(document.objects_iter().any(|(_, object)| {
            matches!(object, GeoObject::ScatterPlot(scatter) if scatter.source_data == Some(data_id))
        }));
    }
}

#[cfg(test)]
pub(crate) use crate::sheet_layout::apply_object_panel_edit;
pub(crate) use crate::sheet_layout::apply_object_panel_edit_with_previous;

fn color_picker_swatch(ui: &mut egui::Ui, color: Color, label: &str) -> egui::Response {
    let theme = current_theme(ui.ctx());
    let color = Color32::from_rgba_unmultiplied(
        (color.r * 255.0).clamp(0.0, 255.0) as u8,
        (color.g * 255.0).clamp(0.0, 255.0) as u8,
        (color.b * 255.0).clamp(0.0, 255.0) as u8,
        (color.a * 255.0).clamp(0.0, 255.0) as u8,
    );
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(
            grafito_ui::tokens::ICON_XL - 4.0,
            grafito_ui::tokens::ICON_LG,
        ),
        egui::Sense::click(),
    );
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label));
    let border = if response.hovered() {
        theme.accent
    } else {
        theme.separator
    };
    ui.painter().rect_filled(rect.shrink(3.0), RADIUS_SM, color);
    ui.painter()
        .rect_stroke(rect.shrink(3.0), RADIUS_SM, egui::Stroke::new(1.0, border));
    response.on_hover_text(label)
}

/// Helper de retrocompatibilidad. Devuelve la tupla histórica
/// `(is_dark, accent, alg_fill, _sep_col, txt_col, txt_dim, hdr_col)`
/// usando el Theme activo.
#[allow(clippy::type_complexity)]
pub(crate) fn panel_theme_local(
    ctx: &egui::Context,
) -> (bool, Color32, Color32, Color32, Color32, Color32, Color32) {
    let t = current_theme(ctx);
    let is_dark = t.canvas_bg.r() < 100;
    (
        is_dark,
        t.accent,
        t.panel_bg,
        t.separator,
        t.text_primary,
        t.text_tertiary,
        t.text_secondary,
    )
}

/// Header del drawer derecho: título LG + subtítulo XS + hairline.
fn draw_right_drawer_header(
    ui: &mut egui::Ui,
    app: &mut GrafitoApp,
    title: &str,
    subtitle: &str,
    accent: Color32,
) {
    let theme = current_theme(ui.ctx());
    // B-A159: cierre del drawer por dispatcher (antes ES duro).
    let close_label = inspector_chrome_for("close_context", app.config_locale());
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(title)
                .color(accent)
                .size(TYPE_LG)
                .strong(),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if action_icon_button(ui, Icon::Close, theme.text_secondary, close_label).clicked() {
                app.right_drawer_open = false;
            }
        });
    });
    if !subtitle.is_empty() {
        ui.add_space(SPACE_XS);
        ui.label(
            egui::RichText::new(subtitle)
                .color(theme.text_tertiary)
                .size(TYPE_XS),
        );
    }
    ui.add_space(SPACE_SM);
    ui.painter().line_segment(
        [
            ui.cursor().min,
            ui.cursor().min + egui::vec2(ui.available_width(), 0.0),
        ],
        theme.hairline_stroke(),
    );
    ui.add_space(SPACE_SM);
}

// Textos puros del Inspector movidos a `sheet_layout.rs` (slice 17 F14.4.4).

fn draw_inspector_identity(ui: &mut egui::Ui, obj: &GeoObject, locale: grafito_ui::i18n::Locale) {
    let theme = current_theme(ui.ctx());
    let visible = obj.is_visible();
    let label = obj.label().to_string();
    let caption = inspector_type_caption(obj);
    // Titular: ecuación en grande; sin ecuación, placeholder honesto.
    // El nombre del tipo vive solo como caption terciario.
    let headline = inspector_equation_text(obj).unwrap_or_else(|| caption.to_string());
    let equation_shown = inspector_equation_text(obj).is_some();
    egui::Frame::none()
        .fill(theme.input_bg)
        .stroke(egui::Stroke::NONE)
        .rounding(egui::Rounding::same(RADIUS_LG))
        .inner_margin(egui::Margin::same(SPACE_SM))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
            // Título + badge en la misma fila: el badge va a la derecha
            // del título, nunca flotando.
            ui.horizontal(|ui| {
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(inspector_chrome_for("identity_title", locale))
                            .color(theme.text_tertiary)
                            .size(TYPE_SM)
                            .strong(),
                    )
                    .truncate(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(inspector_visibility_label_for(visible, locale))
                                .color(if visible {
                                    theme.success
                                } else {
                                    theme.text_tertiary
                                })
                                .size(TYPE_SM),
                        )
                        .truncate(),
                    )
                    .on_hover_text(inspector_identity_tooltip_for(
                        &headline, &label, visible, locale,
                    ));
                });
            });
            ui.add_space(SPACE_XS);
            let tooltip = inspector_identity_tooltip_for(&headline, &label, visible, locale);
            if equation_shown {
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(&headline)
                            .color(theme.text_primary)
                            .size(TYPE_MD)
                            .monospace(),
                    )
                    .truncate(),
                )
                .on_hover_text(tooltip.as_str());
            } else {
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(inspector_chrome_for("no_equation", locale))
                            .color(theme.text_secondary)
                            .size(TYPE_SM),
                    )
                    .truncate(),
                )
                .on_hover_text(tooltip.as_str());
            }
            ui.add_space(SPACE_XS);
            if let Some(labeled) = inspector_label_text_for(&label, locale) {
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(labeled)
                            .color(theme.text_secondary)
                            .size(TYPE_SM),
                    )
                    .truncate(),
                )
                .on_hover_text(tooltip.as_str());
            }
            ui.add(
                egui::Label::new(
                    egui::RichText::new(caption)
                        .color(theme.text_tertiary)
                        .size(TYPE_XS),
                )
                .truncate(),
            )
            .on_hover_text(tooltip.as_str());
        });
}

// Etiquetas ES de estilo movidas a `sheet_layout.rs` (slice 17 F14.4.4).

pub(crate) fn draw_inspector_section(
    ui: &mut egui::Ui,
    title: &str,
    description: &str,
    contents: impl FnOnce(&mut egui::Ui),
) {
    let theme = current_theme(ui.ctx());
    egui::Frame::none()
        .fill(theme.panel_bg)
        .stroke(theme.hairline_stroke())
        .rounding(egui::Rounding::same(RADIUS_SM))
        .inner_margin(egui::Margin {
            left: SPACE_MD,
            right: SPACE_MD,
            top: SPACE_SM,
            bottom: SPACE_MD,
        })
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
            ui.label(
                egui::RichText::new(title)
                    .color(theme.text_primary)
                    .size(TYPE_SM)
                    .strong(),
            );
            if !description.is_empty() {
                ui.add_space(SPACE_XS);
                ui.label(
                    egui::RichText::new(description)
                        .color(theme.text_tertiary)
                        .size(TYPE_XS),
                );
            }
            ui.add_space(SPACE_SM);
            ui.spacing_mut().item_spacing.y = SPACE_SM;
            ui.spacing_mut().interact_size.y = ZOOM_ICON_HIT;
            contents(ui);
        });
}

fn draw_inspector_empty_state(ui: &mut egui::Ui, locale: grafito_ui::i18n::Locale) {
    let theme = current_theme(ui.ctx());
    let height = ui.available_height().max(140.0);
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), height),
        egui::Layout::top_down(egui::Align::Center),
        |ui| {
            ui.add_space(SPACE_XXL);
            egui::Frame::none()
                .fill(theme.input_bg)
                .stroke(egui::Stroke::NONE)
                .rounding(egui::Rounding::same(RADIUS_LG))
                .inner_margin(egui::Margin::same(SPACE_MD))
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new(inspector_chrome_for("empty_title", locale))
                                .color(theme.text_primary)
                                .size(TYPE_BASE)
                                .strong(),
                        );
                        ui.add_space(SPACE_XS);
                        ui.label(
                            egui::RichText::new(inspector_chrome_for("empty_hint", locale))
                                .color(theme.text_secondary)
                                .size(TYPE_SM),
                        );
                    });
                });
        },
    );
}

fn draw_multidimensional_motion_card(
    ui: &mut egui::Ui,
    app: &mut GrafitoApp,
    title: &str,
    description: &str,
    can_animate: bool,
) {
    let theme = current_theme(ui.ctx());
    // B-A106: chrome motion en el idioma activo (antes ES duro).
    use grafito_ui::i18n::t as motion_t;
    let motion_locale = app.config_locale();
    // B-A106: etiquetas una vez (línea corta, estable a `fmt`).
    let pause_label = motion_t("motion.pause", motion_locale);
    let start_label = motion_t("motion.start", motion_locale);
    let toast_started_label = motion_t("motion.toast_started", motion_locale);
    let toast_paused_label = motion_t("motion.toast_paused", motion_locale);
    let playing_label = motion_t("motion.status_playing", motion_locale);
    let enabled_label = motion_t("motion.status_enabled", motion_locale);
    let paused_label = motion_t("motion.status_paused", motion_locale);
    let unavailable_label = motion_t("motion.status_unavailable", motion_locale);
    let speed_label = motion_t("motion.speed", motion_locale);
    let speed_text_label = motion_t("motion.speed_text", motion_locale);
    let reset_label = motion_t("motion.reset", motion_locale);
    let reset_hint_label = motion_t("motion.reset_hint", motion_locale);
    let need_3d_label = motion_t("motion.need_3d", motion_locale);
    // El interruptor global siempre está disponible ("reproducir siempre"):
    // `is_moving` indica reproducción EFECTIVA acá (objeto visible en vista
    // 3D); el estado del flag global se muestra honesto abajo.
    let global_on = app.multidimensional_motion_enabled;
    let mut is_moving = can_animate && global_on;
    let card_fill = if is_moving {
        theme.accent_muted
    } else {
        theme.input_bg
    };
    let card_stroke = if is_moving {
        theme.accent
    } else {
        theme.separator
    };

    egui::Frame::none()
        .fill(card_fill)
        .stroke(egui::Stroke::new(1.0, card_stroke))
        .rounding(egui::Rounding::same(RADIUS_LG))
        .inner_margin(egui::Margin::same(SPACE_MD))
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new(title)
                        .color(theme.text_primary)
                        .size(TYPE_BASE)
                        .strong(),
                );
                ui.label(
                    egui::RichText::new(description)
                        .color(theme.text_secondary)
                        .size(TYPE_SM),
                );
            });
            ui.add_space(SPACE_SM);

            let action_label = if global_on { pause_label } else { start_label };
            let action_button = egui::Button::new(
                egui::RichText::new(action_label)
                    .color(if is_moving {
                        theme.text_primary
                    } else {
                        theme.keyboard_enter_text
                    })
                    .strong(),
            )
            .fill(if is_moving {
                theme.button_bg
            } else {
                theme.keyboard_enter_bg
            })
            .stroke(egui::Stroke::new(1.0, card_stroke));
            let response = ui.add_sized([ui.available_width(), PANEL_BUTTON_H], action_button);
            if response.clicked() {
                is_moving = crate::app::toggle_default_multidimensional_motion(
                    &mut app.multidimensional_motion_enabled,
                );
                app.notify(
                    if is_moving {
                        toast_started_label
                    } else {
                        toast_paused_label
                    },
                    grafito_ui::toast::ToastKind::Info,
                );
                // Ola 2: one-shot vía presupuesto (ZERO = inmediato) en vez de
                // `request_repaint()` directo en el draw.
                app.request_repaint_budget(Duration::ZERO);
            }

            ui.add_space(SPACE_SM);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(if is_moving {
                        playing_label
                    } else if global_on {
                        enabled_label
                    } else if can_animate {
                        paused_label
                    } else {
                        unavailable_label
                    })
                    .color(if is_moving {
                        theme.success
                    } else {
                        theme.text_secondary
                    })
                    .size(TYPE_SM)
                    .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!("{:.2}x", app.multidimensional_motion_speed))
                            .color(theme.text_primary)
                            .size(TYPE_SM)
                            .monospace(),
                    );
                    ui.label(
                        egui::RichText::new(speed_label)
                            .color(theme.text_secondary)
                            .size(TYPE_SM),
                    );
                });
            });

            let mut speed = app.multidimensional_motion_speed;
            // Riel a ancho real (egui default 100px): si no, el slider sale
            // angosto e impreciso al tacto en panel angosto.
            ui.spacing_mut().slider_width = ui.available_width().max(0.0);
            if ui
                .add(
                    egui::Slider::new(
                        &mut speed,
                        crate::app::MIN_MULTIDIMENSIONAL_MOTION_SPEED
                            ..=crate::app::MAX_MULTIDIMENSIONAL_MOTION_SPEED,
                    )
                    .text(speed_text_label)
                    .show_value(false)
                    .step_by(0.25)
                    .trailing_fill(true),
                )
                .changed()
            {
                app.set_multidimensional_motion_speed(speed);
                // Ola 2: one-shot vía presupuesto (ZERO = inmediato).
                app.request_repaint_budget(Duration::ZERO);
            }
            if ui
                .small_button(reset_label)
                .on_hover_text(reset_hint_label)
                .clicked()
            {
                app.set_multidimensional_motion_speed(
                    crate::app::DEFAULT_MULTIDIMENSIONAL_MOTION_SPEED,
                );
                // Ola 2: one-shot vía presupuesto (ZERO = inmediato).
                app.request_repaint_budget(Duration::ZERO);
            }

            if !can_animate {
                ui.add_space(SPACE_SM);
                ui.label(
                    egui::RichText::new(need_3d_label)
                        .color(theme.text_tertiary)
                        .size(TYPE_SM),
                );
            }
        });
}

fn draw_object_cards_where(
    ui: &mut egui::Ui,
    app: &mut GrafitoApp,
    title: &str,
    empty_text: &str,
    predicate: impl Fn(&GeoObject) -> bool,
) {
    let theme = current_theme(ui.ctx());
    ui.add_space(SPACE_SM);
    ui.label(
        egui::RichText::new(title)
            .color(theme.text_primary)
            .size(TYPE_SM)
            .strong(),
    );
    ui.add_space(SPACE_XS);

    let ids: Vec<ObjectId> = app
        .document
        .objects_iter()
        .filter_map(|(id, obj)| predicate(obj).then_some(*id))
        .collect();

    if ids.is_empty() {
        ui.label(
            egui::RichText::new(empty_text)
                .color(theme.text_tertiary)
                .size(TYPE_XS),
        );
        return;
    }

    for id in ids {
        crate::algebra::draw_object_card(ui, app, id);
    }
}

// ── Frente W-C: paso-a-paso visual (moat donde GeoGebra es débil) ─────────
// Tres secciones del panel CAS que cablean UI a comandos reales existentes:
// FunctionStudy (recorrido + tabla de signos), RiemannSum (slider n +
// exacta para comparar) y Taylor (slider orden 1..=10 + resto observado).
// El estado vive en la memoria temporal de egui como `LayerPanelState`:
// sin campos nuevos en `GrafitoApp`, sin I/O. Cero fantasma: cada botón
// arma `app.input_text` y llama a `submit_cas_worksheet_cell`, igual que
// el botón Ejecutar del panel.

// Métodos de cuadratura del selector W-C: (comando, etiqueta visible).
// (movidos a `sheet_layout.rs`: `WC_QUADRATURE_METHODS` +
// `WC_RIEMANN_SLIDER_MAX` + `WcVisualState` + `wc_quadrature_method/key`,
// slice 22 F14.4.4; el estado efímero `ctx.data` + dibujo quedan acá)

// (movido a `sheet_layout.rs`: `wc_study_command_text` +
// `wc_riemann_command_text` + `wc_exact_integral_command_text` +
// `wc_taylor_command_text` + `wc_taylor_remainder_line`, slice 10 F14.4.4;
// el doc previo "Texto del comando T1…" vive ahora con la función)

/// Ejecuta un comando W-C como si se escribiera en la Entrada: arma
/// `input_text` y dispara la celda (cero fantasma, comando real).
fn wc_submit(app: &mut GrafitoApp, ui: &egui::Ui, command: String) {
    app.input_text = command;
    app.submit_cas_worksheet_cell(ui.ctx().input(|i| i.time));
}

fn draw_wc_study_contents(app: &mut GrafitoApp, ui: &mut egui::Ui, wc: &mut WcVisualState) {
    use grafito_ui::i18n::t as wc_t;
    let wc_locale = app.config_locale();
    let function_label = wc_t("wc.study_function", wc_locale);
    let study_label = wc_t("wc.study_button", wc_locale);
    let hint_label = wc_t("wc.study_hint", wc_locale);
    ui.horizontal(|ui| {
        ui.label(function_label);
        ui.text_edit_singleline(&mut wc.estudio_label);
        if ui.small_button(study_label).clicked() {
            if let Some(cmd) = wc_study_command_text(&wc.estudio_label) {
                wc_submit(app, ui, cmd);
            }
        }
    });
    ui.label(
        egui::RichText::new(hint_label)
            .color(current_theme(ui.ctx()).text_tertiary)
            .size(TYPE_XS),
    );
}

fn draw_wc_integral_contents(app: &mut GrafitoApp, ui: &mut egui::Ui, wc: &mut WcVisualState) {
    use grafito_ui::i18n::t as wc_t;
    let wc_locale = app.config_locale();
    let hover_label = wc_t("wc.integral_hover", wc_locale);
    let simpson_hint_label = wc_t("wc.integral_simpson_hint", wc_locale);
    let approx_label = wc_t("wc.integral_approx", wc_locale);
    let exact_label = wc_t("wc.integral_exact", wc_locale);
    let sheet_hint_label = wc_t("wc.integral_sheet_hint", wc_locale);
    ui.text_edit_singleline(&mut wc.integral_expr)
        .on_hover_text(hover_label);
    ui.horizontal(|ui| {
        ui.label("a:");
        ui.text_edit_singleline(&mut wc.integral_a);
        ui.label("b:");
        ui.text_edit_singleline(&mut wc.integral_b);
    });
    ui.spacing_mut().slider_width = ui.available_width().max(0.0);
    ui.add(egui::Slider::new(&mut wc.integral_n, 1.0..=WC_RIEMANN_SLIDER_MAX).text("n"));
    let metodo_idx = wc.integral_metodo.min(WC_QUADRATURE_METHODS.len() - 1);
    wc.integral_metodo = metodo_idx;
    egui::ComboBox::from_id_salt("wc_metodo")
        .selected_text(wc_quadrature_method(metodo_idx).1)
        .show_ui(ui, |ui| {
            for (i, (_, nombre)) in WC_QUADRATURE_METHODS.iter().enumerate() {
                ui.selectable_value(&mut wc.integral_metodo, i, *nombre);
            }
        });
    let n = wc.integral_n.round() as usize;
    if wc_quadrature_key(wc.integral_metodo) == "simpson" && n % 2 == 1 {
        ui.label(
            egui::RichText::new(simpson_hint_label)
                .color(current_theme(ui.ctx()).text_tertiary)
                .size(TYPE_XS),
        );
    }
    ui.horizontal(|ui| {
        if ui.small_button(approx_label).clicked() {
            if let Some(cmd) = wc_riemann_command_text(
                &wc.integral_expr,
                &wc.integral_a,
                &wc.integral_b,
                wc.integral_n,
                wc_quadrature_key(wc.integral_metodo),
            ) {
                wc_submit(app, ui, cmd);
            }
        }
        if ui.small_button(exact_label).clicked() {
            if let Some(cmd) =
                wc_exact_integral_command_text(&wc.integral_expr, &wc.integral_a, &wc.integral_b)
            {
                wc_submit(app, ui, cmd);
            }
        }
    });
    ui.label(
        egui::RichText::new(sheet_hint_label)
            .color(current_theme(ui.ctx()).text_tertiary)
            .size(TYPE_XS),
    );
}

fn draw_wc_taylor_contents(app: &mut GrafitoApp, ui: &mut egui::Ui, wc: &mut WcVisualState) {
    use grafito_ui::i18n::t as wc_t;
    let wc_locale = app.config_locale();
    let hover_label = wc_t("wc.taylor_hover", wc_locale);
    let center_label = wc_t("wc.taylor_center", wc_locale);
    let order_label = wc_t("wc.taylor_order", wc_locale);
    let poly_label = wc_t("wc.taylor_poly", wc_locale);
    let no_remainder_label = wc_t("wc.taylor_no_remainder", wc_locale);
    ui.text_edit_singleline(&mut wc.taylor_expr)
        .on_hover_text(hover_label);
    ui.horizontal(|ui| {
        ui.label(center_label);
        ui.text_edit_singleline(&mut wc.taylor_centro);
        ui.label("x:");
        ui.text_edit_singleline(&mut wc.taylor_x);
    });
    ui.spacing_mut().slider_width = ui.available_width().max(0.0);
    ui.add(egui::Slider::new(&mut wc.taylor_orden, 1..=10).text(order_label));
    if ui.small_button(poly_label).clicked() {
        if let Some(cmd) = wc_taylor_command_text(
            &wc.taylor_expr,
            &wc.taylor_centro,
            wc.taylor_orden,
            &wc.taylor_x,
        ) {
            wc_submit(app, ui, cmd);
        }
    }
    match wc_taylor_remainder_line(
        &wc.taylor_expr,
        &wc.taylor_centro,
        wc.taylor_orden,
        &wc.taylor_x,
        &app.document.variables,
    ) {
        Some(line) => {
            ui.label(egui::RichText::new(line).monospace().size(TYPE_XS));
        }
        None => {
            ui.label(
                egui::RichText::new(no_remainder_label)
                    .color(current_theme(ui.ctx()).text_tertiary)
                    .size(TYPE_XS),
            );
        }
    }
}

/// Las 3 secciones W-C dentro del panel CAS (tras la Entrada, antes de la
/// Hoja). Estado en temp egui; cada botón ejecuta un comando real.
fn draw_wc_visual_sections(app: &mut GrafitoApp, ui: &mut egui::Ui) {
    // B-A132: headers de sección localizados (`wc.section_*`); contenidos
    // ya localizados B-A107. `app.config_locale()` sin cambio de firma.
    use grafito_ui::i18n::t as wc_t;
    let wc_section_locale = app.config_locale();
    let study_title = wc_t("wc.section_study_title", wc_section_locale);
    let study_subtitle = wc_t("wc.section_study_subtitle", wc_section_locale);
    let integral_title = wc_t("wc.section_integral_title", wc_section_locale);
    let integral_subtitle = wc_t("wc.section_integral_subtitle", wc_section_locale);
    let taylor_title = wc_t("wc.section_taylor_title", wc_section_locale);
    let taylor_subtitle = wc_t("wc.section_taylor_subtitle", wc_section_locale);
    let state_id = ui.id().with("wc_visual");
    let mut wc = ui.ctx().memory_mut(|m| {
        m.data
            .get_temp_mut_or_default::<WcVisualState>(state_id)
            .clone()
    });
    draw_inspector_section(ui, study_title, study_subtitle, |ui| {
        draw_wc_study_contents(app, ui, &mut wc)
    });
    ui.add_space(SPACE_MD);
    draw_inspector_section(ui, integral_title, integral_subtitle, |ui| {
        draw_wc_integral_contents(app, ui, &mut wc)
    });
    ui.add_space(SPACE_MD);
    draw_inspector_section(ui, taylor_title, taylor_subtitle, |ui| {
        draw_wc_taylor_contents(app, ui, &mut wc)
    });
    ui.ctx().memory_mut(|m| m.data.insert_temp(state_id, wc));
}

/// Ancho del drawer izquierdo: en móvil (<740) ocupa todo el viewport
/// (sheet fullscreen: el panel ES la pantalla, con Entrada abajo); en
/// desktop el ancho fijo de siempre. Pura.
#[inline]
pub(crate) fn panel_left_width(ctx: &egui::Context) -> f32 {
    let viewport = ctx.screen_rect().width();
    if viewport < grafito_ui::tokens::PALETTE_BOTTOM_SHEET_BREAKPOINT {
        viewport
    } else {
        PANEL_LEFT_DEFAULT
    }
}

#[allow(dead_code)]
pub(crate) fn draw_cas_panel(app: &mut GrafitoApp, ctx: &egui::Context) {
    let theme = current_theme(ctx);
    let accent = theme.accent;
    let panel_bg = theme.panel_bg;
    let txt_col = theme.text_primary;
    let txt_dim = theme.text_tertiary;
    // B-A177: chrome del panel CAS por dispatcher (Es→consts canónicas,
    // resto→`cas_msg`); notación `x^2, x`/`↵  •  Tab`/snippets intacta.
    let cas_locale = app.config_locale();
    let cas_panel_title = cas_chrome_for("panel_title", cas_locale);
    let cas_clear_label = cas_chrome_for("clear_label", cas_locale);
    let cas_clear_tooltip = cas_chrome_for("clear_tooltip", cas_locale);
    let cas_quick_section_title = cas_chrome_for("quick_section_title", cas_locale);
    let cas_quick_section_subtitle = cas_chrome_for("quick_section_subtitle", cas_locale);
    let cas_entry_label = cas_chrome_for("entry_label", cas_locale);
    let cas_execute_tooltip = cas_chrome_for("execute_tooltip", cas_locale);
    let cas_empty_title = cas_chrome_for("empty_title", cas_locale);
    let cas_empty_hint = cas_chrome_for("empty_hint", cas_locale);
    let cas_empty_example = cas_chrome_for("empty_example", cas_locale);

    // Panel CAS — Scandinavian quiet: secciones calm con disclosure progresivo
    egui::SidePanel::left("cas_panel")
        .show_separator_line(false)
        .exact_width(panel_left_width(ctx))
        .resizable(false)
        .frame(
            egui::Frame::none()
                .fill(panel_bg)
                .stroke(egui::Stroke::new(1.0, theme.separator)),
        )
        .show(ctx, |ui| {
            ui.add_space(SPACE_LG);
            ui.horizontal(|ui| {
                ui.add_space(SPACE_SM);
                ui.label(
                    egui::RichText::new(cas_panel_title)
                        .color(accent)
                        .strong()
                        .size(TYPE_MD),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if !app.document.cas_worksheet().is_empty()
                        && ui
                            .small_button(cas_clear_label)
                            .on_hover_text(cas_clear_tooltip)
                            .clicked()
                    {
                        app.clear_cas_worksheet(ui.ctx().input(|input| input.time));
                    }
                });
            });
            ui.add_space(SPACE_XS);
            ui.separator();
            ui.add_space(SPACE_SM);

            panel_scroll_content(ui, "cas_panel_scroll", None, |ui| {
                egui::Frame::none()
                    .inner_margin(egui::Margin {
                        left: SPACE_SM,
                        right: SPACE_XS,
                        top: SPACE_SM,
                        bottom: SPACE_SM,
                    })
                    .show(ui, |ui| {
                        // Acciones rápidas — pills Scandinavian dentro de sección inspector
                        draw_inspector_section(
                            ui,
                            cas_quick_section_title,
                            cas_quick_section_subtitle,
                            |ui| {
                                ui.horizontal_wrapped(|ui| {
                                    ui.spacing_mut().item_spacing = egui::vec2(SPACE_XS, SPACE_XS);
                                    for (i, &(_, snippet)) in CAS_QUICK_ACTIONS.iter().enumerate() {
                                        let label = cas_quick_label_for(i, cas_locale);
                                        let btn = egui::Button::new(
                                            egui::RichText::new(label)
                                                .size(TYPE_SM)
                                                .color(theme.text_primary),
                                        )
                                        .rounding(RADIUS_SM)
                                        .fill(theme.input_bg)
                                        .stroke(
                                            egui::Stroke::new(1.0, Color32::from_black_alpha(26)),
                                        );
                                        if ui
                                            .add(btn)
                                            .on_hover_text(cas_quick_insert_tooltip_for(
                                                snippet, cas_locale,
                                            ))
                                            .clicked()
                                        {
                                            app.input_text = snippet.to_string();
                                        }
                                    }
                                });
                            },
                        );
                        ui.add_space(SPACE_MD);

                        // Entrada — Scandinavian centered, compact, 32h
                        ui.vertical_centered(|ui| {
                            egui::Frame::none()
                                .fill(theme.input_bg)
                                .stroke(egui::Stroke::new(1.0, Color32::from_black_alpha(18)))
                                .rounding(egui::Rounding::same(RADIUS_LG))
                                .inner_margin(egui::Margin::symmetric(SPACE_MD, SPACE_SM))
                                .show(ui, |ui| {
                                    ui.set_max_width(260.0);
                                    ui.label(
                                        egui::RichText::new(cas_entry_label)
                                            .color(theme.text_tertiary)
                                            .size(TYPE_XS)
                                            .strong(),
                                    );
                                    ui.add_space(SPACE_XS);
                                    // El popup ocupa el ancho de la tarjeta.
                                    let cas_span =
                                        (ui.next_widget_position().x, ui.available_width());
                                    ui.horizontal(|ui| {
                                        let mut execute_cas = false;
                                        // Input centered, max 200, height 32
                                        let response = crate::ui::draw_command_input(
                                            ui,
                                            app,
                                            "cas_panel",
                                            [200.0, 32.0],
                                            CAS_ENTRY_HINT,
                                            true,
                                            Some(cas_span),
                                        );
                                        ui.add_space(SPACE_XS);
                                        if action_icon_button(
                                            ui,
                                            Icon::Play,
                                            accent,
                                            cas_execute_tooltip,
                                        )
                                        .clicked()
                                        {
                                            execute_cas = true;
                                        }
                                        if response.submitted {
                                            execute_cas = true;
                                        }
                                        if execute_cas && !app.input_text.is_empty() {
                                            let time = ui.ctx().input(|i| i.time);
                                            app.submit_cas_worksheet_cell(time);
                                        }
                                    });
                                    ui.add_space(SPACE_XS);
                                    ui.label(
                                        egui::RichText::new(CAS_ENTRY_KEYS_HINT)
                                            .color(txt_dim)
                                            .size(TYPE_XS),
                                    );
                                });
                        });
                        ui.add_space(SPACE_MD);

                        // Frente W-C: paso-a-paso visual (cablea a
                        // comandos reales; cero fantasma).
                        draw_wc_visual_sections(&mut *app, ui);
                        ui.add_space(SPACE_MD);

                        // Hoja de trabajo — empty state sutil y celdas con estados
                        if app.document.cas_worksheet().is_empty() {
                            egui::Frame::none()
                                .fill(theme.input_bg)
                                .stroke(egui::Stroke::new(1.0, Color32::from_black_alpha(18)))
                                .rounding(egui::Rounding::same(RADIUS_LG))
                                .inner_margin(egui::Margin::same(SPACE_LG))
                                .show(ui, |ui| {
                                    ui.vertical_centered(|ui| {
                                        ui.label(
                                            egui::RichText::new(cas_empty_title)
                                                .color(txt_col)
                                                .size(TYPE_SM)
                                                .strong(),
                                        );
                                        ui.add_space(SPACE_XS);
                                        ui.label(
                                            egui::RichText::new(cas_empty_hint)
                                                .color(txt_dim)
                                                .size(TYPE_XS),
                                        );
                                        ui.add_space(SPACE_SM);
                                        ui.label(
                                            egui::RichText::new(cas_empty_example)
                                                .color(txt_dim)
                                                .size(TYPE_SM)
                                                .monospace(),
                                        );
                                    });
                                });
                        } else {
                            ui.label(
                                egui::RichText::new(cas_sheet_title_for(
                                    app.document.cas_worksheet().len(),
                                    cas_locale,
                                ))
                                .color(theme.text_secondary)
                                .size(TYPE_SM),
                            );
                            ui.add_space(SPACE_XS);
                            for (i, entry) in app.document.cas_worksheet().iter().enumerate() {
                                let output_color = match entry.status {
                                    CasWorksheetStatus::Success => txt_col,
                                    CasWorksheetStatus::Error => theme.danger,
                                };
                                egui::Frame::none()
                                    .fill(theme.button_bg)
                                    .stroke(egui::Stroke::new(1.0, Color32::from_black_alpha(26)))
                                    .rounding(egui::Rounding::same(RADIUS_MD))
                                    .inner_margin(egui::Margin::same(SPACE_SM))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                egui::RichText::new(cas_cell_number(i))
                                                    .color(accent)
                                                    .strong()
                                                    .size(TYPE_SM),
                                            );
                                            ui.add_space(SPACE_XS);
                                            ui.label(
                                                egui::RichText::new(cas_cell_input_line(
                                                    &entry.input,
                                                ))
                                                .size(TYPE_SM)
                                                .monospace()
                                                .color(txt_col),
                                            );
                                        });
                                        ui.add_space(SPACE_XS);
                                        ui.label(
                                            egui::RichText::new(&entry.output)
                                                .size(TYPE_SM)
                                                .color(output_color),
                                        );
                                    });
                                ui.add_space(SPACE_XS);
                            }
                        }
                        ui.add_space(SPACE_SM);
                    });
            });
        });
}

/// Sección Exportar / Compartir — vive en el tab Herramientas y en
/// Archivo > Exportar. NUNCA en Vista: Vista es solo visual (cuadrícula,
/// ejes, 3D y capas). Piel pura: el diálogo rfd queda en UI thread (modal
/// nativo); render+write van a workers y el summary se aplica en
/// `poll_background_jobs`.
pub(crate) fn draw_export_section(ui: &mut egui::Ui, app: &mut GrafitoApp) {
    // B-A169: chrome de la sección archivo por dispatcher (Es→consts
    // canónicas, resto→`export_msg`); siglas SVG/PNG/TikZ/PDF intactas.
    let export_locale = app.config_locale();
    let export_section_title =
        crate::sheet_layout::export_chrome_for("section_title", export_locale);
    let export_section_subtitle =
        crate::sheet_layout::export_chrome_for("section_subtitle", export_locale);
    let export_svg_tip = crate::sheet_layout::export_chrome_for("file_svg_tip", export_locale);
    let export_png_tip = crate::sheet_layout::export_chrome_for("file_png_tip", export_locale);
    let export_tikz_tip = crate::sheet_layout::export_chrome_for("file_tikz_tip", export_locale);
    let export_pdf_tip = crate::sheet_layout::export_chrome_for("file_pdf_tip", export_locale);
    let export_copy_svg = crate::sheet_layout::export_chrome_for("file_copy_svg", export_locale);
    let export_copy_svg_tip =
        crate::sheet_layout::export_chrome_for("file_copy_svg_tip", export_locale);
    let export_copy_png = crate::sheet_layout::export_chrome_for("file_copy_png", export_locale);
    let export_copy_png_tip =
        crate::sheet_layout::export_chrome_for("file_copy_png_tip", export_locale);
    // B-A170: resto del chrome (texto + tablas + notifys + reveal + fila) por
    // dispatcher con `export_locale`. Labels `MathML`/`TikZ-eje`/`HTML`/`CSV`
    // siglas intactas (precedente B-A169): solo tips + fila se localizan.
    let export_text_mathml_tip =
        crate::sheet_layout::export_chrome_for("text_mathml_tip", export_locale);
    let export_text_tikz_tip =
        crate::sheet_layout::export_chrome_for("text_tikz_tip", export_locale);
    let export_text_html_tip =
        crate::sheet_layout::export_chrome_for("text_html_tip", export_locale);
    let export_tables_title = crate::sheet_layout::export_chrome_for("tables_title", export_locale);
    let export_csv_tip = crate::sheet_layout::export_chrome_for("csv_tip", export_locale);
    let export_notify_pdf = crate::sheet_layout::export_chrome_for("notify_pdf", export_locale);
    let export_notify_png = crate::sheet_layout::export_chrome_for("notify_png", export_locale);
    let export_notify_csv = crate::sheet_layout::export_chrome_for("notify_csv", export_locale);
    let export_reveal_label = crate::sheet_layout::export_chrome_for("reveal_label", export_locale);
    let export_reveal_tip = crate::sheet_layout::export_chrome_for("reveal_tip", export_locale);
    let export_table_untagged =
        crate::sheet_layout::export_chrome_for("table_untagged", export_locale);
    draw_inspector_section(ui, export_section_title, export_section_subtitle, |ui| {
        // Cuadrícula 3×3 escandinava: botones iguales, todo a la misma
        // altura y sin textos que envuelvan; el detalle va en tooltip.
        // Responsive como la grilla canónica: 1 columna bajo 200 px
        // (layout puro en `sheet_layout.rs`, slice 21 F14.4.4).
        let cols = export_grid_columns(ui.available_width());
        let cell_w = export_grid_cell_w(ui.available_width(), cols);
        let action = |ui: &mut egui::Ui, label: &str, tip: &str| {
            let hit = ui
                .add_sized(
                    [cell_w, PANEL_BUTTON_H],
                    // `Extend`: una sola línea siempre (nunca envuelve a dos y
                    // rompe la altura de la fila). Labels cortos + tooltip.
                    egui::Button::new(egui::RichText::new(label).size(TYPE_XS))
                        .wrap_mode(egui::TextWrapMode::Extend)
                        .rounding(RADIUS_SM),
                )
                .on_hover_text(tip)
                .clicked();
            // A 1 columna cada celda cierra su fila; a 3 mandan los
            // `end_row` manuales por fila.
            if cols == 1 {
                ui.end_row();
            }
            hit
        };
        egui::Grid::new("gc_export_grid")
            .num_columns(cols)
            .spacing([SPACE_SM, SPACE_SM])
            .show(ui, |ui| {
                // Fila 1 — archivo vectorial/ráster.
                if action(ui, EXPORT_FILE_BUTTONS[0].0, export_svg_tip) {
                    app.export_with_dialog(crate::export::ExportFormat::Svg, Some(ui.ctx()));
                }
                if action(ui, EXPORT_FILE_BUTTONS[1].0, export_png_tip) {
                    app.export_with_dialog(crate::export::ExportFormat::Png, Some(ui.ctx()));
                }
                if action(ui, EXPORT_FILE_BUTTONS[2].0, export_tikz_tip) {
                    app.export_with_dialog(crate::export::ExportFormat::Tikz, Some(ui.ctx()));
                }
                if cols != 1 {
                    ui.end_row();
                }
                // Fila 2 — PDF y portapapeles de imagen.
                if action(ui, EXPORT_FILE_BUTTONS[3].0, export_pdf_tip) {
                    // S2: sin `desktop-dialogs` devuelve `None` con toast honesto.
                    if let Some(path) =
                        app.save_file_native("PDF", &["pdf"], Some(EXPORT_PDF_DEFAULT_FILENAME))
                    {
                        app.pending_export_job = Some(crate::app::PendingExportJob {
                            receiver: spawn_pdf_export(app.document.clone(), path, ui.ctx()),
                        });
                        app.notify(export_notify_pdf, grafito_ui::toast::ToastKind::Info);
                    }
                }
                if action(ui, export_copy_svg, export_copy_svg_tip) {
                    match clipboard_svg(&app.document) {
                        Ok(svg) => {
                            let bytes = svg.len();
                            ui.ctx().output_mut(|out| {
                                out.copied_text = svg;
                            });
                            app.cas_result = export_svg_copied_text_for(bytes, export_locale);
                            app.notify(
                                app.cas_result.clone(),
                                grafito_ui::toast::ToastKind::Success,
                            );
                        }
                        Err(error) => {
                            app.cas_result = export_svg_error_text_for(&error, export_locale);
                            app.notify(app.cas_result.clone(), grafito_ui::toast::ToastKind::Error);
                        }
                    }
                }
                if action(ui, export_copy_png, export_copy_png_tip) {
                    // worker: no Ui:: — arboard bloquea; el resultado se
                    // publica en `poll_background_jobs` vía `PendingClipboardJob`.
                    app.pending_clipboard_job = Some(crate::app::PendingClipboardJob {
                        receiver: spawn_png_clipboard(app.document.clone(), ui.ctx()),
                    });
                    app.notify(export_notify_png, grafito_ui::toast::ToastKind::Info);
                }
                if cols != 1 {
                    ui.end_row();
                }
                // Fila 3 — texto puro al portapapeles (G-C).
                for (label, tip, build) in [
                    (
                        EXPORT_TEXT_BUTTONS[0].0,
                        export_text_mathml_tip,
                        crate::export::document_to_mathml
                            as fn(&Document) -> Result<String, String>,
                    ),
                    (
                        EXPORT_TEXT_BUTTONS[1].0,
                        export_text_tikz_tip,
                        crate::export::document_to_tikz_axis
                            as fn(&Document) -> Result<String, String>,
                    ),
                    (
                        EXPORT_TEXT_BUTTONS[2].0,
                        export_text_html_tip,
                        crate::export::document_to_html as fn(&Document) -> Result<String, String>,
                    ),
                ] {
                    if action(ui, label, tip) {
                        match build(&app.document) {
                            Ok(text) => {
                                let bytes = text.len();
                                ui.ctx().output_mut(|out| {
                                    out.copied_text = text;
                                });
                                app.cas_result =
                                    export_clipboard_copied_text_for(label, bytes, export_locale);
                                app.notify(
                                    app.cas_result.clone(),
                                    grafito_ui::toast::ToastKind::Success,
                                );
                            }
                            Err(error) => {
                                app.cas_result = error;
                                app.notify(
                                    app.cas_result.clone(),
                                    grafito_ui::toast::ToastKind::Error,
                                );
                            }
                        }
                    }
                }
                if cols != 1 {
                    ui.end_row();
                }
            });
        // Tablas del documento (CSV RFC 4180): una fila por tabla.
        let tables: Vec<(ObjectId, String, usize)> = app
            .document
            .objects_iter_sorted()
            .filter_map(|(id, object)| match object {
                GeoObject::DataTable(table) => Some((*id, table.label.clone(), table.xs.len())),
                _ => None,
            })
            .collect();
        if !tables.is_empty() {
            ui.add_space(SPACE_SM);
            ui.label(
                egui::RichText::new(export_tables_title)
                    .size(TYPE_XS)
                    .color(current_theme(ui.ctx()).text_secondary),
            );
        }
        for (id, label, rows) in tables {
            ui.add_space(SPACE_XS);
            ui.horizontal(|ui| {
                let name = if label.is_empty() {
                    export_table_untagged
                } else {
                    &label
                };
                // Truncado: la etiqueta larga expulsaba al botón CSV.
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(crate::sheet_layout::export_table_row_label_for(
                            name,
                            rows,
                            export_locale,
                        ))
                        .size(TYPE_SM),
                    )
                    .truncate(),
                )
                .on_hover_text(crate::sheet_layout::export_table_row_label_for(
                    name,
                    rows,
                    export_locale,
                ));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .small_button(EXPORT_CSV_LABEL)
                        .on_hover_text(export_csv_tip)
                        .clicked()
                    {
                        match datatable_csv_text(&app.document, id) {
                            Ok((table_label, csv)) => {
                                let stem = sanitize_export_stem(&table_label);
                                // S2: sin `desktop-dialogs` devuelve `None` con toast honesto.
                                if let Some(path) = app.save_file_native(
                                    "CSV",
                                    &["csv"],
                                    Some(&format!("{stem}.csv")),
                                ) {
                                    app.pending_export_job = Some(crate::app::PendingExportJob {
                                        receiver: spawn_csv_export(
                                            csv,
                                            table_label,
                                            path,
                                            ui.ctx(),
                                        ),
                                    });
                                    app.notify(
                                        export_notify_csv,
                                        grafito_ui::toast::ToastKind::Info,
                                    );
                                }
                            }
                            Err(error) => {
                                app.cas_result = error.clone();
                                app.notify(error, grafito_ui::toast::ToastKind::Error);
                            }
                        }
                    }
                });
            });
        }
        // W3 — la exportación deja de ser muda: revela la carpeta de la
        // última exportación exitosa (igual que Archivo > Exportar).
        if app.last_export_dir.is_some() {
            ui.add_space(SPACE_SM);
            if ui
                .add_sized(
                    [ui.available_width(), PANEL_BUTTON_H],
                    egui::Button::new(egui::RichText::new(export_reveal_label).size(TYPE_SM))
                        .rounding(RADIUS_SM),
                )
                .on_hover_text(export_reveal_tip)
                .clicked()
            {
                app.reveal_last_export();
            }
        }
    });
}

pub(crate) fn draw_view_panel(app: &mut GrafitoApp, ctx: &egui::Context) {
    // Panel Vista — solo visual (cuadrícula, ejes, 3D y capas). Exportar vive
    // en Herramientas, la hoja en Datos y la probabilidad en Prob.
    // B-A128: chrome General+Ejes en el idioma activo (antes consts ES).
    // B-A129: +14 (Vista 3D + Capas estático + AltaPrecisión, antes consts ES).
    // B-A130: +3 estáticas (hints de Capas + sin-etiqueta, antes consts ES).
    // B-A131: +5 estáticas (parámetro vivo, antes consts ES).
    let view_locale = app.config_locale();
    let general_title = crate::sheet_layout::view_chrome_for("general_title", view_locale);
    let show_grid_label = crate::sheet_layout::view_chrome_for("show_grid", view_locale);
    let dark_mode_label = crate::sheet_layout::view_chrome_for("dark_mode", view_locale);
    let snap_label = crate::sheet_layout::view_chrome_for("snap", view_locale);
    let axes_title = crate::sheet_layout::view_chrome_for("axes_title", view_locale);
    let x_log_label = crate::sheet_layout::view_chrome_for("x_log", view_locale);
    let y_log_label = crate::sheet_layout::view_chrome_for("y_log", view_locale);
    let number_plane_label = crate::sheet_layout::view_chrome_for("number_plane", view_locale);
    let number_plane_hint = crate::sheet_layout::view_chrome_for("number_plane_hint", view_locale);
    let section_3d = crate::sheet_layout::view_chrome_for("section_3d", view_locale);
    let ortho_hover = crate::sheet_layout::view_chrome_for("ortho_hover", view_locale);
    let layers_title = crate::sheet_layout::view_chrome_for("layers_title", view_locale);
    let assign_layer_label = crate::sheet_layout::view_chrome_for("assign_layer", view_locale);
    let assign_label = crate::sheet_layout::view_chrome_for("assign", view_locale);
    let highlight_label = crate::sheet_layout::view_chrome_for("highlight", view_locale);
    let indicate_label = crate::sheet_layout::view_chrome_for("indicate", view_locale);
    let indicate_hover = crate::sheet_layout::view_chrome_for("indicate_hover", view_locale);
    let indicating = crate::sheet_layout::view_chrome_for("indicating", view_locale);
    let hp_title = crate::sheet_layout::view_chrome_for("hp_title", view_locale);
    let hp_label = crate::sheet_layout::view_chrome_for("hp_label", view_locale);
    let hp_hint = crate::sheet_layout::view_chrome_for("hp_hint", view_locale);
    let layer_empty_hint = crate::sheet_layout::view_chrome_for("layer_empty", view_locale);
    let layer_no_selection =
        crate::sheet_layout::view_chrome_for("layer_no_selection", view_locale);
    let live_section = crate::sheet_layout::view_chrome_for("live_section", view_locale);
    let live_missing = crate::sheet_layout::view_chrome_for("live_missing", view_locale);
    let live_create = crate::sheet_layout::view_chrome_for("live_create", view_locale);
    let live_create_hover = crate::sheet_layout::view_chrome_for("live_create_hover", view_locale);
    let live_created = crate::sheet_layout::view_chrome_for("live_created", view_locale);
    let theme = current_theme(ctx);

    // Estado Capas (Q2: tabla persistente en el documento; en temp egui
    // solo la capa elegida). Se poda al dibujar para acotar memoria.
    let layer_panel_id = egui::Id::new("grafito_layer_panel_state");
    let mut layer_state: LayerPanelState = ctx
        .data_mut(|data| data.get_temp::<LayerPanelState>(layer_panel_id))
        .unwrap_or_default();
    app.document.prune_layers();
    let selection: Option<(ObjectId, String, u32)> = app.selected_object.and_then(|id| {
        app.document
            .get_object(id)
            .map(|object| (id, object.label().to_string(), app.document.layer_of(id)))
    });

    egui::SidePanel::left("view_panel")
        .show_separator_line(false)
        .exact_width(panel_left_width(ctx))
        .resizable(false)
        .frame(
            egui::Frame::none()
                .fill(theme.panel_bg)
                .stroke(egui::Stroke::NONE),
        )
        .show(ctx, |ui| {
            // Sin header: el rail dice "Vista" y el tab activo cierra.
            ui.add_space(SPACE_SM);

            panel_scroll_content(ui, "view_panel_scroll", None, |ui| {
                egui::Frame::none()
                    .inner_margin(egui::Margin {
                        left: SPACE_MD,
                        right: SPACE_XS,
                        top: SPACE_SM,
                        bottom: SPACE_LG,
                    })
                    .show(ui, |ui| {
                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                        ui.spacing_mut().item_spacing.y = SPACE_SM;
                        // General — 4 toggles básicos
                        draw_inspector_section(ui, general_title, "", |ui| {
                            // Valor EFECTIVO (el flag `__view_show_grid` del
                            // documento manda si existe, puesto por
                            // `ShowGrid[..]`): mostrar `app.show_grid` a
                            // secas quedaba rancio. Al cambiar se escribe
                            // en ambos para que no diverjan.
                            let mut show_grid = crate::view_flags::view_flag_bool(
                                &app.document,
                                crate::view_flags::VIEW_SHOW_GRID,
                                app.show_grid,
                            );
                            if grafito_ui::assistant::drill_switch(
                                ui,
                                show_grid_label,
                                "",
                                &mut show_grid,
                                true,
                            ) {
                                app.show_grid = show_grid;
                                let _ = app.document.try_set_variable(
                                    crate::view_flags::VIEW_SHOW_GRID.to_string(),
                                    if show_grid { 1.0 } else { 0.0 },
                                );
                            }
                            if grafito_ui::assistant::drill_switch(
                                ui,
                                dark_mode_label,
                                "",
                                &mut app.dark_mode,
                                true,
                            ) {
                                if app.dark_mode {
                                    DARK.apply(ui.ctx());
                                } else {
                                    LIGHT.apply(ui.ctx());
                                }
                            }
                            grafito_ui::assistant::drill_switch(
                                ui,
                                snap_label,
                                "",
                                &mut app.snap_to_grid,
                                true,
                            );
                            // D2: salida de examen con confirmación (nunca directo).
                            app.exam_mode_checkbox(ui);
                        });
                        ui.add_space(SPACE_LG);

                        // Ejes — escala logarítmica + plano numerado (F2c)
                        draw_inspector_section(ui, axes_title, "", |ui| {
                            grafito_ui::assistant::drill_switch(
                                ui,
                                x_log_label,
                                "",
                                &mut app.document.view_mut().x_log,
                                true,
                            );
                            grafito_ui::assistant::drill_switch(
                                ui,
                                y_log_label,
                                "",
                                &mut app.document.view_mut().y_log,
                                true,
                            );
                            grafito_ui::assistant::drill_switch(
                                ui,
                                number_plane_label,
                                number_plane_hint,
                                &mut app.document.number_plane_labels,
                                true,
                            );
                        });
                        ui.add_space(SPACE_LG);

                        // Vista 3D (dueño canvas.rs): perspectiva orbital u
                        // ortográficas. Escribe `app.view3d`; el dibujo y
                        // el pick del canvas 3D la leen por frame.
                        draw_inspector_section(ui, section_3d, "", |ui| {
                            let mut vista = app.view3d;
                            let combo_3d = egui::ComboBox::from_id_salt("view3d_selector")
                                .width(ui.available_width().max(0.0))
                                .selected_text(vista.name())
                                .show_ui(ui, |ui| {
                                    for candidata in crate::canvas::View3D::all() {
                                        ui.selectable_value(
                                            &mut vista,
                                            candidata,
                                            candidata.name(),
                                        );
                                    }
                                });
                            if vista != app.view3d {
                                app.view3d = vista;
                                // Ola 2: one-shot vía presupuesto.
                                app.request_repaint_budget(Duration::ZERO);
                            }
                            // Ayuda en hover, no línea permanente: la
                            // tarjeta queda quieta.
                            combo_3d.response.on_hover_text(ortho_hover);
                        });
                        ui.add_space(SPACE_LG);

                        // Parámetro vivo (F2c · ValueTracker→slider).
                        // Un parámetro nombrado (`p`) bound al slider y a la
                        // animación paramétrica: el slider escribe vía
                        // `set_live_param` (re-evalúa dependientes y sube
                        // versión; las funciones se re-muestrean por clave
                        // con hash de variables) y la animación lo lee con
                        // `live_param_value("p", fallback)` si existe.
                        draw_inspector_section(ui, live_section, "", |ui| {
                            if let Some(live) = app.document.live_param("p") {
                                let mut value = live.value;
                                ui.spacing_mut().slider_width = ui.available_width().max(0.0);
                                let slider = egui::Slider::new(&mut value, live.min..=live.max)
                                    .text("p")
                                    .clamping(egui::SliderClamping::Edits)
                                    .trailing_fill(true);
                                let response =
                                    ui.add(slider).on_hover_text(live_param_hover_text_for(
                                        value,
                                        live.min,
                                        live.max,
                                        view_locale,
                                    ));
                                // T6: un arrastre = una sola entrada de undo (el
                                // `before` vive en memoria temporal hasta soltar).
                                let gesture_key =
                                    egui::Id::new((crate::app::PANEL_GESTURE_UNDO_KEY, "live-p"));
                                if response.drag_started() || response.dragged() {
                                    crate::app::panel_gesture_begin(
                                        ui.ctx(),
                                        gesture_key,
                                        &app.document,
                                    );
                                }
                                if response.changed() {
                                    if response.dragged() {
                                        // Gesto en curso: mutar sin pushear.
                                        if let Err(error) = app.document.set_live_param("p", value)
                                        {
                                            let message =
                                                live_param_error_text_for(error, view_locale);
                                            app.cas_result = message.clone();
                                            app.notify(
                                                message,
                                                grafito_ui::toast::ToastKind::Error,
                                            );
                                        }
                                    } else {
                                        let mut snapshot = crate::app::DeferredPanelSnapshot::new(
                                            app.undo_stack.len(),
                                        );
                                        snapshot.capture(&app.document);
                                        if let Err(error) = app.document.set_live_param("p", value)
                                        {
                                            let message =
                                                live_param_error_text_for(error, view_locale);
                                            app.cas_result = message.clone();
                                            app.notify(
                                                message,
                                                grafito_ui::toast::ToastKind::Error,
                                            );
                                        }
                                        snapshot.save_if_semantically_changed(
                                            &mut app.document,
                                            &mut app.undo_stack,
                                            &mut app.redo_stack,
                                        );
                                    }
                                }
                                if response.drag_stopped() {
                                    crate::app::panel_gesture_commit(
                                        ui.ctx(),
                                        gesture_key,
                                        app.document.version,
                                        &mut app.undo_stack,
                                        &mut app.redo_stack,
                                    );
                                } else {
                                    ui.label(
                                        egui::RichText::new(live_param_readout_text_for(
                                            value,
                                            live.min,
                                            live.max,
                                            view_locale,
                                        ))
                                        .size(TYPE_XS)
                                        .color(current_theme(ui.ctx()).text_secondary),
                                    );
                                }
                            } else {
                                ui.label(
                                    egui::RichText::new(live_missing)
                                        .size(TYPE_XS)
                                        .color(current_theme(ui.ctx()).text_secondary),
                                );
                                ui.add_space(SPACE_XS);
                                if panel_primary_button(ui, live_create)
                                    .on_hover_text(live_create_hover)
                                    .clicked()
                                {
                                    let mut snapshot = crate::app::DeferredPanelSnapshot::new(
                                        app.undo_stack.len(),
                                    );
                                    snapshot.capture(&app.document);
                                    match app.document.ensure_live_param("p", -5.0, 5.0, 0.0) {
                                        Ok(_) => {
                                            app.cas_result = live_created.to_string();
                                        }
                                        Err(error) => {
                                            let message =
                                                live_param_error_text_for(error, view_locale);
                                            app.cas_result = message.clone();
                                            app.notify(
                                                message,
                                                grafito_ui::toast::ToastKind::Error,
                                            );
                                        }
                                    }
                                    snapshot.save_if_semantically_changed(
                                        &mut app.document,
                                        &mut app.undo_stack,
                                        &mut app.redo_stack,
                                    );
                                }
                            }
                        });
                        ui.add_space(SPACE_LG);

                        // Capas — orden + visibilidad (Q2, tabla persistente).
                        // Piel pura: lee &Estado, muta documento con snapshot
                        // de undo; el diálogo/trabajo pesado no aplica aquí
                        // (solo toggles y assigns en memoria).
                        draw_inspector_section(ui, layers_title, "", |ui| {
                            let used = app.document.used_layers();
                            if used.is_empty() {
                                ui.label(
                                    egui::RichText::new(layer_empty_hint)
                                        .size(TYPE_XS)
                                        .color(current_theme(ui.ctx()).text_secondary),
                                );
                            }
                            for (layer, count) in used {
                                let visible = app.document.is_layer_visible(layer);
                                ui.horizontal(|ui| {
                                    ui.add(
                                        egui::Label::new(
                                            egui::RichText::new(layer_row_label_for(
                                                layer,
                                                count,
                                                view_locale,
                                            ))
                                            .size(TYPE_SM),
                                        )
                                        .truncate(),
                                    );

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            let action =
                                                layer_toggle_label_for(visible, view_locale);
                                            if ui.small_button(action).clicked() {
                                                let mut snap =
                                                    crate::app::DeferredPanelSnapshot::new(
                                                        app.undo_stack.len(),
                                                    );
                                                snap.capture(&app.document);
                                                let touched =
                                                    app.document.set_layer_visible(layer, !visible);
                                                if let Some(text) = layer_toggle_result_for(
                                                    layer,
                                                    visible,
                                                    touched,
                                                    view_locale,
                                                ) {
                                                    app.cas_result = text;
                                                }
                                                snap.save_if_semantically_changed(
                                                    &mut app.document,
                                                    &mut app.undo_stack,
                                                    &mut app.redo_stack,
                                                );
                                            }
                                        },
                                    );
                                });
                            }
                            ui.add_space(SPACE_XS);
                            let selection_text = selection
                                .as_ref()
                                .map(|(_, label, layer)| {
                                    layer_selection_text_for(label, *layer, view_locale)
                                })
                                .unwrap_or_else(|| layer_no_selection.to_string());
                            ui.label(
                                egui::RichText::new(selection_text)
                                    .size(TYPE_XS)
                                    .color(current_theme(ui.ctx()).text_secondary),
                            );
                            ui.add_space(SPACE_XS);
                            ui.label(egui::RichText::new(assign_layer_label).size(TYPE_SM));
                            ui.horizontal(|ui| {
                                ui.add_sized(
                                    [72.0, PANEL_BUTTON_H],
                                    egui::DragValue::new(&mut layer_state.selected_layer)
                                        .range(0..=MAX_LAYERS)
                                        .speed(1),
                                );
                                let btn_w = ui.available_width().max(0.0);
                                let resp = ui.add_enabled_ui(selection.is_some(), |ui| {
                                    ui.add_sized(
                                        [btn_w, PANEL_BUTTON_H],
                                        egui::Button::new(
                                            egui::RichText::new(assign_label).size(TYPE_SM),
                                        ),
                                    )
                                });
                                if resp.inner.clicked() {
                                    if let Some((id, label, _)) = selection.as_ref() {
                                        let mut snap = crate::app::DeferredPanelSnapshot::new(
                                            app.undo_stack.len(),
                                        );
                                        snap.capture(&app.document);
                                        match app
                                            .document
                                            .set_layer(*id, layer_state.selected_layer)
                                        {
                                            Ok(()) => {
                                                app.cas_result = layer_assign_result_for(
                                                    label,
                                                    layer_state.selected_layer,
                                                    view_locale,
                                                );
                                            }
                                            Err(error) => {
                                                app.cas_result =
                                                    layer_assign_error_for(&error, view_locale);
                                                app.notify(
                                                    app.cas_result.clone(),
                                                    grafito_ui::toast::ToastKind::Error,
                                                );
                                            }
                                        }
                                        snap.save_if_semantically_changed(
                                            &mut app.document,
                                            &mut app.undo_stack,
                                            &mut app.redo_stack,
                                        );
                                    }
                                }
                            });
                            ui.add_space(SPACE_XS);
                            ui.label(egui::RichText::new(highlight_label).size(TYPE_SM));
                            let puede = selection
                                .as_ref()
                                .is_some_and(|(id, _, _)| app.document.get_object(*id).is_some());
                            {
                                let btn_w = ui.available_width().max(0.0);
                                let resp = ui.add_enabled_ui(puede, |ui| {
                                    ui.add_sized(
                                        [btn_w, PANEL_BUTTON_H],
                                        egui::Button::new(
                                            egui::RichText::new(indicate_label).size(TYPE_SM),
                                        ),
                                    )
                                });
                                if resp.inner.on_hover_text(indicate_hover).clicked() {
                                    if let Some((id, _, _)) = selection.as_ref() {
                                        match app.trigger_indicate(*id) {
                                            Ok(()) => {
                                                app.cas_result = indicating.to_string();
                                            }
                                            Err(error) => {
                                                app.cas_result =
                                                    view_indicate_error_for(&error, view_locale);
                                                app.notify(
                                                    app.cas_result.clone(),
                                                    grafito_ui::toast::ToastKind::Error,
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        });
                        ui.add_space(SPACE_LG);

                        // Alta precisión — Double-Double (avanzado, al final).
                        draw_inspector_section(ui, hp_title, "", |ui| {
                            let mut high_prec =
                                grafito_geometry::precision::is_high_precision_mode();
                            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                            if grafito_ui::assistant::drill_switch(
                                ui,
                                hp_label,
                                hp_hint,
                                &mut high_prec,
                                true,
                            ) {
                                grafito_geometry::precision::set_high_precision_mode(high_prec);
                                app.document.invalidate_all_caches();
                                app.document.bump_version();
                                if let Ok(mut cache) = app.trig_graph_cache.write() {
                                    *cache = None;
                                }
                                app.re_evaluate_constraints(&[]);
                            }
                        });
                    });
            });
        });

    // Escribe de vuelta el estado Capas (asigns del frame).
    ctx.data_mut(|data| data.insert_temp(layer_panel_id, layer_state));
}

// ── Explorador trigonométrico: grilla responsive y centrado ─────────────
// (movidos a `sheet_layout.rs`: `TRIG_*` + `trig_function_columns` +
// `trig_grid_cell_w`, slice 16 F14.4.4; acá queda solo el dibujo)

/// Etiqueta alineada por el layout (centro/derecha) sin el corrimiento de
/// egui 0.29: `Label` pinta el galley en el borde alineado en vez de `rect.min`,
/// así que se fuerza `halign(LEFT)` y el layout ubica el rect.
fn aligned_label(ui: &mut egui::Ui, text: egui::RichText) {
    ui.add(egui::Label::new(text).halign(egui::Align::LEFT));
}

/// Rótulo de sección centrado (micro tipo, mayúsculas, terciario).
fn trig_section_label(ui: &mut egui::Ui, text: &str) {
    let color = current_theme(ui.ctx()).text_tertiary;
    ui.vertical_centered(|ui| {
        aligned_label(
            ui,
            egui::RichText::new(text)
                .color(color)
                .size(TYPE_XS)
                .strong(),
        );
    });
    ui.add_space(SPACE_XS);
}

/// Chip seleccionable del explorador: pill con contraste AA en ambos estados.
fn trig_chip(ui: &mut egui::Ui, width: f32, label: &str, selected: bool) -> egui::Response {
    let theme = current_theme(ui.ctx());
    let text_color = if selected {
        theme.text_primary
    } else {
        theme.text_secondary
    };
    ui.add_sized(
        [width, TRIG_CONTROL_H],
        egui::Button::new(egui::RichText::new(label).size(TYPE_SM).color(text_color))
            .selected(selected)
            .rounding(RADIUS_SM),
    )
}

/// Tarjeta quieta (F17.2): `Theme::card_frame` — fill `card_bg` (N2 Slate en
/// oscuro, elevada cálida en claro) + borde opaco 1 px, ancho completo.
/// Antes `input_bg` + `separator` a mano: idéntico en claro, a 1.04:1 en
/// oscuro (N2 vs `input_bg`), ahora canónico vía `card_frame` para que el
/// token tenga caller productivo. La hairline 10 % no alcanza en oscuro.
fn trig_card<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let theme = current_theme(ui.ctx());
    theme
        .card_frame()
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui)
        })
        .inner
}

/// Fila etiqueta a la izquierda + valor mono a la derecha, sin desborde.
fn trig_value_row(ui: &mut egui::Ui, label: &str, value: &str) {
    let theme = current_theme(ui.ctx());
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = SPACE_SM;
        ui.label(
            egui::RichText::new(label)
                .color(theme.text_tertiary)
                .size(TYPE_XS)
                .strong(),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            aligned_label(
                ui,
                egui::RichText::new(value)
                    .color(theme.text_primary)
                    .size(TYPE_XS)
                    .monospace(),
            );
        });
    });
}

/// Header centrado: título al medio con cierre simétrico a la derecha + hairline.
/// Devuelve `true` si se pidió cerrar.
fn trig_header(
    ui: &mut egui::Ui,
    title: &str,
    accent: Color32,
    close_color: Color32,
    close_label: &str,
) -> bool {
    let side = HIT_TARGET_MIN + SPACE_SM;
    let mut close = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.add_space(side);
        let center_w = (ui.available_width() - side).max(64.0);
        ui.allocate_ui_with_layout(
            egui::vec2(center_w, 0.0),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                aligned_label(
                    ui,
                    egui::RichText::new(title)
                        .color(accent)
                        .size(TYPE_MD)
                        .strong(),
                );
            },
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if action_icon_button(ui, Icon::Close, close_color, close_label).clicked() {
                close = true;
            }
        });
    });
    ui.add_space(SPACE_XS);
    ui.painter().line_segment(
        [
            ui.cursor().min,
            ui.cursor().min + egui::vec2(ui.available_width(), 0.0),
        ],
        current_theme(ui.ctx()).hairline_stroke(),
    );
    ui.add_space(SPACE_SM);
    close
}

/// Slider a ancho completo y sin texto de valor (el valor se lee arriba, mono).
/// El raíl usa `separator` (el `button_bg` por defecto se funde con la tarjeta).
fn trig_full_slider(
    ui: &mut egui::Ui,
    value: &mut f64,
    range: std::ops::RangeInclusive<f64>,
    decimals: usize,
) -> bool {
    ui.visuals_mut().widgets.inactive.bg_fill = current_theme(ui.ctx()).separator;
    // Riel a ancho real con piso 0 (mismo `max` que el resto de sliders del
    // panel: sin esto un `available` degenerado pedía ancho negativo).
    ui.spacing_mut().slider_width = ui.available_width().max(0.0);
    ui.add(
        egui::Slider::new(value, range)
            .fixed_decimals(decimals)
            .show_value(false),
    )
    .changed()
}

/// Panel derecho: controles de la animación trigonométrica.
///
/// El círculo y la función se dibujan como overlay del canvas 2D para compartir
/// exactamente la grilla, escala y perspectiva de Geometry2D.
pub(crate) fn draw_trig_animation_panel(app: &mut GrafitoApp, ctx: &egui::Context) {
    let theme = current_theme(ctx);
    let accent = theme.accent;
    let panel_bg = theme.panel_bg;

    // Responsive: reserva un lienzo mínimo y evita el panel-muro en anchos grandes.
    let viewport_w = ctx.available_rect().width();
    let max_w = (viewport_w - TRIG_CANVAS_MIN_W)
        .min(viewport_w * 0.6)
        .clamp(TRIG_PANEL_MIN_W, TRIG_PANEL_MAX_W);

    egui::SidePanel::right("right_trig_animation")
        .show_separator_line(false)
        .default_width(TRIG_PANEL_DEFAULT_W)
        .min_width(TRIG_PANEL_MIN_W)
        .max_width(max_w)
        .resizable(true)
        .frame(
            egui::Frame::none()
                .fill(panel_bg)
                .inner_margin(egui::Margin::symmetric(SPACE_MD, SPACE_SM)),
        )
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    // Nada se recorta: etiquetas envueltas y controles con ancho exacto.
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                    ui.spacing_mut().item_spacing = egui::vec2(SPACE_SM, SPACE_XS);
                    // B-A96: chrome trig en el idioma activo (antes ES duro).
                    // `t` está ocupado por el ángulo (`let t = app.trig_angle`
                    // más abajo): alias para no sombrear el f64.
                    use grafito_ui::i18n::t as trig_t;
                    let trig_locale = app.config_locale();
                    let title = if app.perspective == crate::Perspective::Complex {
                        trig_t("trig.title_complex", trig_locale)
                    } else {
                        trig_t("trig.title_explorer", trig_locale)
                    };
                    if trig_header(
                        ui,
                        title,
                        accent,
                        theme.text_secondary,
                        inspector_chrome_for("close_anim", trig_locale),
                    ) {
                        app.set_trig_animation_visible(false);
                    }

                    if app.perspective == crate::Perspective::Complex {
                        ui.label(
                            egui::RichText::new("z(t) = cos(t) + i sin(t) = e^(it)")
                                .color(theme.text_secondary)
                                .size(TYPE_SM)
                                .strong(),
                        );
                        ui.label(
                            egui::RichText::new(trig_t("trig.complex_desc", trig_locale))
                                .color(theme.text_tertiary)
                                .size(TYPE_XS),
                        );
                        ui.add_space(SPACE_SM);
                    }

                    // ── Función activa: grilla responsive de chips ──
                    trig_section_label(ui, trig_t("trig.section_function", trig_locale));
                    let columns = trig_function_columns(ui.available_width());
                    let cell_w = trig_grid_cell_w(ui.available_width(), columns);
                    egui::Grid::new("trig_function_grid")
                        .num_columns(columns)
                        .spacing([SPACE_SM, SPACE_XS])
                        .show(ui, |ui| {
                            let total = crate::app::TRIG_FUNCTIONS.len();
                            for (idx, spec) in crate::app::TRIG_FUNCTIONS.iter().enumerate() {
                                let label = trig_function_chip_label(spec.name);
                                let selected = app.trig_function as usize == idx;
                                if trig_chip(ui, cell_w, &label, selected).clicked() {
                                    app.set_trig_function(idx as u8);
                                }
                                if (idx + 1) % columns == 0 || idx + 1 == total {
                                    ui.end_row();
                                }
                            }
                        });
                    ui.add_space(SPACE_SM);

                    let spec = GrafitoApp::trig_spec(app.trig_function);

                    // ── Velocidad: play/pausa, slider y pedido en palabras ──
                    trig_card(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = SPACE_SM;
                            let (icon, color, tip) = if app.trig_animating {
                                (
                                    Icon::Pause,
                                    accent,
                                    trig_t("trig.play_pause_tip", trig_locale),
                                )
                            } else {
                                (
                                    Icon::Play,
                                    theme.text_secondary,
                                    trig_t("trig.play_play_tip", trig_locale),
                                )
                            };
                            if action_icon_button(ui, icon, color, tip).clicked() {
                                app.trig_animating = !app.trig_animating;
                                ctx.request_repaint();
                            }
                            trig_value_row(
                                ui,
                                trig_t("trig.section_speed", trig_locale),
                                &trig_speed_text(app.trig_speed),
                            );
                        });
                        ui.add_space(SPACE_XS);
                        if trig_full_slider(ui, &mut app.trig_speed, -6.0..=6.0, 1) {
                            ctx.request_repaint();
                        }
                        ui.add_space(SPACE_SM);
                        draw_trig_speed_prompt(ui, app);
                    });

                    // ── Vista: segmento de dos opciones ──
                    ui.add_space(SPACE_SM);
                    trig_section_label(ui, trig_t("trig.section_view", trig_locale));
                    let segment_w = trig_grid_cell_w(ui.available_width(), 2);
                    egui::Grid::new("trig_view_mode_grid")
                        .num_columns(2)
                        .spacing([SPACE_SM, SPACE_XS])
                        .show(ui, |ui| {
                            let modes = [
                                (
                                    crate::app::TrigViewMode::Didactic,
                                    trig_t("trig.view_didactic", trig_locale),
                                    trig_t("trig.view_didactic_hint", trig_locale),
                                ),
                                (
                                    crate::app::TrigViewMode::Grid,
                                    trig_t("trig.view_grid", trig_locale),
                                    trig_t("trig.view_grid_hint", trig_locale),
                                ),
                            ];
                            for (mode, label, tip) in modes {
                                let selected = app.trig_view_mode == mode;
                                if trig_chip(ui, segment_w, label, selected)
                                    .on_hover_text(tip)
                                    .clicked()
                                {
                                    app.trig_view_mode = mode;
                                    ctx.request_repaint();
                                }
                            }
                            ui.end_row();
                        });

                    // ── Ángulo ──
                    ui.add_space(SPACE_SM);
                    trig_card(ui, |ui| {
                        trig_value_row(
                            ui,
                            trig_t("trig.section_angle", trig_locale),
                            &trig_angle_text(app.trig_angle),
                        );
                        ui.add_space(SPACE_XS);
                        if trig_full_slider(
                            ui,
                            &mut app.trig_angle,
                            -2.0 * std::f64::consts::PI..=2.0 * std::f64::consts::PI,
                            2,
                        ) {
                            ctx.request_repaint();
                        }
                    });

                    // ── Lectura del punto vivo ──
                    let t = app.trig_angle;
                    let fn_val = GrafitoApp::trig_value(app.trig_function, t);
                    let cos_t = t.cos();
                    let sin_t = t.sin();
                    let trace_locale = app.config_locale();
                    let value_text = trig_value_text_for(spec.name, t, fn_val, trace_locale);
                    ui.add_space(SPACE_SM);
                    trig_card(ui, |ui| {
                        ui.label(
                            egui::RichText::new(value_text)
                                .color(accent)
                                .size(TYPE_MD)
                                .strong(),
                        );
                        ui.add_space(SPACE_XS);
                        ui.label(
                            egui::RichText::new(trig_point_text_for(t, trace_locale))
                                .color(theme.text_secondary)
                                .size(TYPE_XS),
                        );
                        ui.add_space(SPACE_XS);
                        ui.label(
                            egui::RichText::new(GrafitoApp::trig_identity_for(
                                app.trig_function,
                                trace_locale,
                            ))
                            .color(theme.text_tertiary)
                            .size(TYPE_XS),
                        );
                        if app.perspective == crate::Perspective::Complex {
                            ui.add_space(SPACE_XS);
                            ui.label(
                                egui::RichText::new(trig_complex_text(t))
                                    .color(theme.text_secondary)
                                    .size(TYPE_XS),
                            );
                        }
                        // Lectura de traza: el punto vivo bajo el "puntero" animado.
                        grafito_ui::trace::draw_trace_readout(
                            ui,
                            grafito_ui::trace::trace_from_hover(cos_t, sin_t, spec.name).as_ref(),
                            trace_locale,
                        );
                    });

                    // ── Centrar la vista (acción primaria) ──
                    ui.add_space(SPACE_LG);
                    if panel_primary_button(ui, trig_t("trig.center_view", trig_locale)).clicked() {
                        app.document
                            .set_view(grafito_geometry::ViewTransform::default());
                        app.document.bump_version();
                        if let Ok(mut cache) = app.trig_graph_cache.write() {
                            *cache = None;
                        }
                    }
                    ui.add_space(SPACE_SM);
                });
        });
}

pub(crate) fn draw_empty_panel(_app: &mut GrafitoApp, ctx: &egui::Context) {
    let (_is_dark, _accent, alg_fill, _sep_col, _txt_col, _txt_dim, _hdr_col) =
        panel_theme_local(ctx);
    let theme = current_theme(ctx);

    egui::SidePanel::left("empty_panel")
        .show_separator_line(false)
        .exact_width(panel_left_width(ctx))
        .resizable(false)
        .frame(
            egui::Frame::none()
                .fill(alg_fill)
                .stroke(egui::Stroke::NONE),
        )
        .show(ctx, |ui| {
            let height = ui.available_height().max(120.0);
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), height),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.add_space(SPACE_XXL);
                    ui.label(
                        egui::RichText::new(EMPTY_PANEL_TITLE)
                            .color(theme.text_tertiary)
                            .size(TYPE_SM),
                    );
                    ui.label(
                        egui::RichText::new(EMPTY_PANEL_GUIDE)
                            .color(theme.text_secondary)
                            .size(TYPE_SM),
                    );
                },
            );
        });
}

// ══════════════════════════════════════════════════════════════════════════
// Paneles izquierdos específicos por perspectiva (Fase 2)
// ══════════════════════════════════════════════════════════════════════════

// F14.4.4 (sigue): el tab Datos vive en `crate::spreadsheet`.
pub(crate) use crate::spreadsheet::draw_data_panel;

/// Tab Prob.: probabilidad (distribuciones) + estadística (resumen).
/// Casa de las perspectivas Probabilidad y Estadística; accesible global
/// como tab 3. La calculadora de distribuciones abre desplegada cuando se
/// entra desde Probabilidad.
pub(crate) fn draw_statistics_panel(app: &mut GrafitoApp, ctx: &egui::Context) {
    let (_is_dark, _accent, alg_fill, _sep_col, txt_col, txt_dim, hdr_col) = panel_theme_local(ctx);
    // B-A105: chrome stats en el idioma activo (antes ES duro).
    use grafito_ui::i18n::t as stats_t;
    let stats_locale = app.config_locale();

    egui::SidePanel::left("stats_panel")
        .show_separator_line(false)
        .exact_width(panel_left_width(ctx))
        .resizable(false)
        .frame(
            egui::Frame::none()
                .fill(alg_fill)
                .stroke(egui::Stroke::NONE),
        )
        .show(ctx, |ui| {
            // Sin header: el rail dice "Prob." y el tab activo cierra.
            ui.add_space(SPACE_SM);
            panel_scroll_content(ui, "stats_panel_content", None, |ui| {
                egui::Frame::none()
                    .inner_margin(egui::Margin {
                        left: SPACE_MD,
                        right: SPACE_XS,
                        top: SPACE_SM,
                        bottom: SPACE_LG,
                    })
                    .show(ui, |ui| {
                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                        ui.spacing_mut().item_spacing.y = SPACE_SM;
                        // B-A105: etiquetas una vez (línea corta, estable a `fmt`).
                        let dist_label = stats_t("stats.dist_title", stats_locale);
                        let data_label = stats_t("stats.data_title", stats_locale);
                        let apply_label = stats_t("stats.apply", stats_locale);
                        let clear_label = stats_t("stats.clear", stats_locale);
                        let empty_label = stats_t("stats.empty", stats_locale);
                        let empty_hint_label = stats_t("stats.empty_hint", stats_locale);
                        let summary_label = stats_t("stats.summary", stats_locale);
                        // B-A119: empty de objetos estadísticos en el idioma
                        // activo (antes ES duro `:2942,:2943`).
                        let objects_title_label = stats_t("stats.objects_title", stats_locale);
                        let objects_empty_label = stats_t("stats.objects_empty", stats_locale);
                        // B-A123: ordenar+graficar en el idioma activo
                        // (antes ES duro `:2749,:2761,:2765,:2780,:2789,`
                        // `:2793,:2819,:2823,:2846,:2850`).
                        let sort_section_label = stats_t("stats.sort_section", stats_locale);
                        let sort_btn_label = stats_t("stats.sort_btn", stats_locale);
                        let sort_hint_label = stats_t("stats.sort_hint", stats_locale);
                        let hist_btn_label = stats_t("stats.hist_btn", stats_locale);
                        let hist_hint_label = stats_t("stats.hist_hint", stats_locale);
                        let box_btn_label = stats_t("stats.box_btn", stats_locale);
                        let box_hint_label = stats_t("stats.box_hint", stats_locale);
                        let mean_btn_label = stats_t("stats.mean_btn", stats_locale);
                        let mean_hint_label = stats_t("stats.mean_hint", stats_locale);
                        // B-A124: resumen + bloque asistente en el idioma
                        // activo (antes ES duro `:2675` header Histograma
                        // + `:2939,:2940,:2942,:2944` bloque asistente).
                        let ask_section_label = stats_t("stats.ask_section", stats_locale);
                        let ask_hint_label = stats_t("stats.ask_hint", stats_locale);
                        let ask_btn_label = stats_t("stats.ask_btn", stats_locale);
                        let ask_btn_hint_label = stats_t("stats.ask_btn_hint", stats_locale);
                        // B-A134: header Inferencia en el idioma activo
                        // (antes ES duro `:2949`).
                        let inference_title_label = stats_t("stats.inference_title", stats_locale);

                        // Calculadora de distribuciones: card propia siempre
                        // visible (sin laberinto de desplegables).
                        draw_inspector_section(ui, dist_label, "", |ui| {
                            draw_probability_section(ui, ctx, app);
                        });
                        ui.add_space(SPACE_SM);

                        // ── Tus datos: TextEdit vinculado al buffer persistente ──
                        // El buffer sólo se parsea al perder foco o al apretar "Aplicar"
                        // — antes, el editor reconstruí el string cada frame desde los
                        // valores parseados y destruía la entrada del usuario por cada
                        // coma en blanco o no-número temporal.
                        draw_inspector_section(ui, data_label, "", |ui| {
                            let theme = current_theme(ctx);
                            let te_resp = egui::Frame::none()
                                .fill(theme.input_bg)
                                .stroke(theme.hairline_stroke())
                                .rounding(egui::Rounding::same(RADIUS_SM))
                                .inner_margin(egui::Margin::same(SPACE_XS))
                                .show(ui, |ui| {
                                    ui.add_sized(
                                        [ui.available_width(), 80.0],
                                        egui::TextEdit::multiline(&mut app.statistics_input_buf)
                                            .desired_rows(4)
                                            .hint_text("1, 2, 3, 5, 4, 6")
                                            .frame(false),
                                    )
                                })
                                .inner;
                            // Anillo de foco como las celdas de la hoja.
                            if te_resp.has_focus() {
                                theme.paint_focus_ring(ui.painter(), te_resp.rect);
                            }

                            ui.add_space(SPACE_XS);
                            {
                                let theme = current_theme(ctx);
                                // Sin clamp que empuje fuera del panel angosto.
                                let bw = ((ui.available_width() - SPACE_SM) / 2.0).max(0.0);
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = SPACE_SM;
                                    let apply_btn = egui::Button::new(
                                        egui::RichText::new(apply_label)
                                            .size(TYPE_SM)
                                            .strong()
                                            .color(theme.keyboard_enter_text),
                                    )
                                    .fill(theme.keyboard_enter_bg)
                                    .stroke(egui::Stroke::NONE)
                                    .rounding(RADIUS_SM);
                                    let apply_clicked =
                                        ui.add_sized([bw, PANEL_BUTTON_H], apply_btn).clicked();
                                    let lost_focus = te_resp.lost_focus()
                                        && ui.input(|i| i.key_pressed(egui::Key::Enter));
                                    if apply_clicked || lost_focus {
                                        match parse_statistics_input(&app.statistics_input_buf) {
                                            Ok(parsed) => {
                                                app.statistics_input_error = None;
                                                if parsed != app.statistics_data {
                                                    app.statistics_data = parsed;
                                                    app.document.bump_version();
                                                }
                                            }
                                            Err(error) => app.statistics_input_error = Some(error),
                                        }
                                    }
                                    if ui
                                        .add_sized(
                                            [bw, PANEL_BUTTON_H],
                                            egui::Button::new(
                                                egui::RichText::new(clear_label).size(TYPE_SM),
                                            )
                                            .rounding(RADIUS_SM),
                                        )
                                        .clicked()
                                    {
                                        app.statistics_input_buf.clear();
                                        app.statistics_data.clear();
                                        app.statistics_input_error = None;
                                        app.document.bump_version();
                                    }
                                });
                            }

                            // Slot reservado (nbsp): el error aparece sin empujar el
                            // resumen de abajo.
                            if let Some(error) = &app.statistics_input_error {
                                ui.label(
                                    egui::RichText::new(error)
                                        .color(current_theme(ctx).danger)
                                        .size(TYPE_XS),
                                );
                            } else {
                                ui.label(egui::RichText::new("\u{a0}").size(TYPE_XS));
                            }

                            ui.add_space(SPACE_SM);
                            if app.statistics_data.is_empty() {
                                // Empty-state en tarjeta quiet (no texto suelto).
                                let theme = current_theme(ctx);
                                egui::Frame::none()
                                    .fill(theme.input_bg)
                                    .stroke(egui::Stroke::NONE)
                                    .rounding(egui::Rounding::same(RADIUS_MD))
                                    .inner_margin(egui::Margin::same(SPACE_MD))
                                    .show(ui, |ui| {
                                        ui.vertical_centered(|ui| {
                                            ui.label(
                                                egui::RichText::new(empty_label)
                                                    .color(theme.text_primary)
                                                    .size(TYPE_SM)
                                                    .strong(),
                                            );
                                            ui.add_space(SPACE_XS);
                                            ui.label(
                                                egui::RichText::new(empty_hint_label)
                                                    .color(txt_dim)
                                                    .size(TYPE_XS),
                                            );
                                        });
                                    });
                            } else {
                                let data = &app.statistics_data;
                                let summary = match statistics_summary(data) {
                                    Ok(summary) => summary,
                                    Err(error) => {
                                        ui.label(
                                            egui::RichText::new(error)
                                                .color(current_theme(ctx).danger)
                                                .size(TYPE_XS),
                                        );
                                        return;
                                    }
                                };

                                ui.label(
                                    egui::RichText::new(summary_label)
                                        .color(hdr_col)
                                        .size(TYPE_SM)
                                        .strong(),
                                );
                                ui.add_space(SPACE_XS);
                                egui::Grid::new("stats_grid")
                                    .num_columns(2)
                                    .striped(true)
                                    .spacing([SPACE_SM, SPACE_XS])
                                    .show(ui, |ui| {
                                        let mut row = |k: &str, v: String| {
                                            ui.label(
                                                egui::RichText::new(k).color(txt_dim).size(TYPE_SM),
                                            );
                                            // Valor truncado con tooltip: un texto largo
                                            // ("No representable…") envolvía la grilla.
                                            ui.add(
                                                egui::Label::new(
                                                    egui::RichText::new(v.clone())
                                                        .color(txt_col)
                                                        .size(TYPE_SM)
                                                        .strong(),
                                                )
                                                .truncate(),
                                            )
                                            .on_hover_text(v);
                                            ui.end_row();
                                        };
                                        for (k, v) in statistics_summary_rows(data.len(), &summary)
                                        {
                                            row(&k, v);
                                        }
                                    });

                                ui.add_space(SPACE_SM);
                                ui.label(
                                    egui::RichText::new(hist_btn_label)
                                        .color(hdr_col)
                                        .size(TYPE_SM)
                                        .strong(),
                                );
                                ui.add_space(2.0);
                                let bins = STATISTICS_SUMMARY_BINS;
                                let (counts, _) = statistics_histogram_counts(
                                    data,
                                    summary.minimum,
                                    summary.range,
                                    bins,
                                );
                                let max_c = (*counts.iter().max().unwrap_or(&1)).max(1) as f32;
                                let hist_h = 90.0;
                                let (hist_rect, _) = ui.allocate_exact_size(
                                    egui::vec2(ui.available_width(), hist_h + 14.0),
                                    egui::Sense::hover(),
                                );
                                if ui.is_rect_visible(hist_rect) {
                                    // Clip explícito: los conteos y min/max se pintan
                                    // a mano y sin esto dejan slivers en docks
                                    // angostos (misma clase que la tira del drawer).
                                    let painter = ui.painter().with_clip_rect(hist_rect);
                                    let plot = hist_rect.shrink2(egui::vec2(2.0, 2.0));
                                    // Plot area interna
                                    let plot_top = plot.min.y;
                                    let plot_bot = plot.max.y - 14.0;
                                    let plot_h = plot_bot - plot_top;
                                    let plot_w = plot.width();
                                    // Ejes: línea base
                                    painter.line_segment(
                                        [
                                            egui::pos2(plot.min.x, plot_bot),
                                            egui::pos2(plot.max.x, plot_bot),
                                        ],
                                        egui::Stroke::new(1.0, _sep_col.gamma_multiply(0.10)),
                                    );
                                    // Barras
                                    let bar_w = plot_w / bins as f32;
                                    for (i, c) in counts.iter().enumerate() {
                                        let h = (*c as f32 / max_c) * plot_h;
                                        let bar = egui::Rect::from_min_size(
                                            egui::pos2(
                                                plot.min.x + i as f32 * bar_w + 2.0,
                                                plot_bot - h,
                                            ),
                                            egui::vec2(bar_w - 4.0, h),
                                        );
                                        painter.rect_filled(bar, 2.0, current_theme(ctx).accent);
                                        // count label encima si > 0
                                        if *c > 0 {
                                            painter.text(
                                                egui::pos2(bar.center().x, bar.min.y - 6.0),
                                                egui::Align2::CENTER_BOTTOM,
                                                c.to_string(),
                                                egui::FontId::proportional(
                                                    grafito_ui::tokens::TYPE_XS,
                                                ),
                                                txt_dim,
                                            );
                                        }
                                    }
                                    // Etiquetas min/max en el eje
                                    plot_axis_labels(
                                        &painter,
                                        plot,
                                        plot_bot,
                                        format_statistic(summary.minimum),
                                        format_statistic(summary.maximum),
                                        txt_dim,
                                    );
                                }
                            }
                        });
                        ui.add_space(SPACE_SM);

                        // ── Ordenar y graficar: acciones directas sobre Tus datos ──
                        // Scandinavian: dos filas de dos botones quiet que usan todo
                        // el ancho; nada de chips que se envuelven ni botones mudos.
                        draw_inspector_section(ui, sort_section_label, "", |ui| {
                            if app.statistics_data.is_empty() {
                                return;
                            }
                            // Sin clamp que empuje fuera del panel angosto.
                            let bw = ((ui.available_width() - SPACE_SM) / 2.0).max(0.0);
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = SPACE_SM;
                                if ui
                                    .add_sized(
                                        [bw, PANEL_BUTTON_H],
                                        egui::Button::new(
                                            egui::RichText::new(sort_btn_label).size(TYPE_SM),
                                        )
                                        .rounding(RADIUS_SM),
                                    )
                                    .on_hover_text(sort_hint_label)
                                    .clicked()
                                {
                                    app.statistics_data.sort_by(|a, b| {
                                        a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
                                    });
                                    app.statistics_input_buf = app
                                        .statistics_data
                                        .iter()
                                        .map(|v| format!("{v}"))
                                        .collect::<Vec<_>>()
                                        .join(", ");
                                    app.statistics_input_error = None;
                                    app.document.bump_version();
                                    app.cas_result = statistics_sorted_text(
                                        app.statistics_data.len(),
                                        stats_locale,
                                    );
                                    ctx.request_repaint();
                                }
                                if ui
                                    .add_sized(
                                        [bw, PANEL_BUTTON_H],
                                        egui::Button::new(
                                            egui::RichText::new(hist_btn_label).size(TYPE_SM),
                                        )
                                        .rounding(RADIUS_SM),
                                    )
                                    .on_hover_text(hist_hint_label)
                                    .clicked()
                                {
                                    match statistics_histogram_command(&app.statistics_data) {
                                        Some(cmd) => {
                                            app.input_text = cmd;
                                            app.command_input_focus_requested = true;
                                            app.cas_result =
                                                statistics_histogram_ready_for(stats_locale)
                                                    .to_string();
                                        }
                                        None => {
                                            app.cas_result =
                                                statistics_inline_too_many_for(stats_locale)
                                                    .to_string();
                                            app.notify(
                                                app.cas_result.clone(),
                                                grafito_ui::toast::ToastKind::Info,
                                            );
                                        }
                                    }
                                    ctx.request_repaint();
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = SPACE_SM;
                                if ui
                                    .add_sized(
                                        [bw, PANEL_BUTTON_H],
                                        egui::Button::new(
                                            egui::RichText::new(box_btn_label).size(TYPE_SM),
                                        )
                                        .rounding(RADIUS_SM),
                                    )
                                    .on_hover_text(box_hint_label)
                                    .clicked()
                                {
                                    match statistics_boxplot_command(&app.statistics_data) {
                                        Some(cmd) => {
                                            app.input_text = cmd;
                                            app.command_input_focus_requested = true;
                                            app.cas_result =
                                                statistics_boxplot_ready_for(stats_locale)
                                                    .to_string();
                                        }
                                        None => {
                                            app.cas_result =
                                                statistics_inline_too_many_for(stats_locale)
                                                    .to_string();
                                            app.notify(
                                                app.cas_result.clone(),
                                                grafito_ui::toast::ToastKind::Info,
                                            );
                                        }
                                    }
                                    ctx.request_repaint();
                                }
                                if ui
                                    .add_sized(
                                        [bw, PANEL_BUTTON_H],
                                        egui::Button::new(
                                            egui::RichText::new(mean_btn_label).size(TYPE_SM),
                                        )
                                        .rounding(RADIUS_SM),
                                    )
                                    .on_hover_text(mean_hint_label)
                                    .clicked()
                                {
                                    match statistics_mean_command(&app.statistics_data) {
                                        Some(cmd) => {
                                            app.input_text = cmd;
                                            app.command_input_focus_requested = true;
                                            app.cas_result =
                                                statistics_mean_ready_for(stats_locale).to_string();
                                        }
                                        None => {
                                            app.cas_result =
                                                statistics_inline_too_many_for(stats_locale)
                                                    .to_string();
                                            app.notify(
                                                app.cas_result.clone(),
                                                grafito_ui::toast::ToastKind::Info,
                                            );
                                        }
                                    }
                                    ctx.request_repaint();
                                }
                            });
                        });
                        ui.add_space(SPACE_SM);

                        // ── Inferencia: lo que el motor ya sabe, expuesto sin laberinto ──
                        // El panel solo mostraba distribuciones + resumen; los tests
                        // (TTest/ZTest/ChiSq/ANOVA), Median/StdDev, FrequencyTable y
                        // StemPlot existían pero sin botón. Chips 2×N que encolan el
                        // comando real en la entrada (cero fantasma).
                        draw_inspector_section(ui, inference_title_label, "", |ui| {
                            let inline = statistics_inline_list(&app.statistics_data);
                            let has_inline = inline.is_some();
                            let list = inline.unwrap_or_default();
                            let chips = statistics_quick_chips_for(&list, stats_locale);
                            for pair in chips.chunks(2) {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = SPACE_SM;
                                    // Sin clamp que empuje fuera del panel angosto.
                                    let bw = ((ui.available_width() - SPACE_SM) / 2.0).max(0.0);
                                    for (label, cmd, tip) in pair {
                                        let btn = ui.add_sized(
                                            [bw, PANEL_BUTTON_H],
                                            egui::Button::new(
                                                egui::RichText::new(label.as_str()).size(TYPE_SM),
                                            )
                                            .rounding(RADIUS_SM),
                                        );
                                        if btn.on_hover_text(tip.as_str()).clicked() {
                                            if has_inline {
                                                app.input_text = cmd.clone();
                                                app.command_input_focus_requested = true;
                                                app.cas_result = statistics_quick_ready_text_for(
                                                    label.as_str(),
                                                    stats_locale,
                                                );
                                            } else {
                                                app.cas_result =
                                                    statistics_no_inline_result_for(stats_locale)
                                                        .to_string();
                                                app.notify(
                                                    app.cas_result.clone(),
                                                    grafito_ui::toast::ToastKind::Info,
                                                );
                                            }
                                            ctx.request_repaint();
                                        }
                                    }
                                });
                            }
                            if !has_inline {
                                ui.label(
                                    egui::RichText::new(statistics_no_inline_hint_for(
                                        stats_locale,
                                    ))
                                    .color(txt_dim)
                                    .size(TYPE_XS),
                                );
                            }
                        });
                        ui.add_space(SPACE_SM);

                        // ── Asistente: archivos grandes → ordenar + graficar ──
                        draw_inspector_section(ui, ask_section_label, ask_hint_label, |ui| {
                            if panel_primary_button(ui, ask_btn_label)
                                .on_hover_text(ask_btn_hint_label)
                                .clicked()
                            {
                                open_statistics_in_assistant(app, ctx);
                            }
                        });
                        ui.add_space(SPACE_SM);

                        draw_object_cards_where(
                            ui,
                            app,
                            objects_title_label,
                            objects_empty_label,
                            |obj| {
                                matches!(
                                    obj,
                                    GeoObject::Histogram(_)
                                        | GeoObject::BarChart(_)
                                        | GeoObject::PieChart(_)
                                        | GeoObject::ScatterPlot(_)
                                        | GeoObject::BoxPlot(_)
                                        | GeoObject::RegressionLine(_)
                                        | GeoObject::Function(_)
                                )
                            },
                        );
                    });
            });
        });
}

/// Panel izquierdo de Complejos. Lista objetos complejos y permite cambiar
/// el símbolo base.
pub(crate) fn draw_complex_panel(app: &mut GrafitoApp, ctx: &egui::Context) {
    use grafito_core::{GeoObject, ObjectId};
    let mut snapshot = crate::app::DeferredPanelSnapshot::new(app.undo_stack.len());
    let (_is_dark, accent, alg_fill, _sep_col, txt_col, txt_dim, hdr_col) = panel_theme_local(ctx);

    egui::SidePanel::left("complex_panel")
        .show_separator_line(false)
        .exact_width(panel_left_width(ctx))
        .resizable(false)
        .frame(
            egui::Frame::none()
                .fill(alg_fill)
                .stroke(egui::Stroke::NONE),
        )
        .show(ctx, |ui| {
            ui.add_space(SPACE_SM);
            ui.label(
                egui::RichText::new(COMPLEX_PANEL_TITLE)
                    .color(accent)
                    .size(grafito_ui::tokens::TYPE_BASE)
                    .strong(),
            );
            ui.add_space(SPACE_SM);

            // ── Barra de entrada in-panel (igual que Álgebra) ──
            // El popup de sugerencias ocupa toda la columna (no solo el campo).
            let complex_span = (ui.next_widget_position().x, ui.available_width());
            egui::Frame::none()
                .fill(current_theme(ctx).input_bg)
                .inner_margin(egui::Margin {
                    left: 8.0,
                    right: 8.0,
                    top: 6.0,
                    bottom: 6.0,
                })
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("+")
                                .color(accent)
                                .size(TYPE_MD)
                                .strong(),
                        );
                        ui.add_space(SPACE_XS);
                        let response = crate::ui::draw_command_input(
                            ui,
                            app,
                            "complex_panel",
                            [ui.available_width(), 22.0],
                            COMPLEX_PANEL_INPUT_HINT,
                            false,
                            Some(complex_span),
                        );
                        if response.submitted && !app.input_text.is_empty() {
                            let time = ui.ctx().input(|i| i.time);
                            app.submit_input_text(time);
                        }
                    });
                });
            ui.add(egui::Separator::default().spacing(0.0));
            ui.add_space(SPACE_SM);

            // ── Símbolo base ──
            ui.label(
                egui::RichText::new(COMPLEX_BASE_SYMBOL_TITLE)
                    .color(hdr_col)
                    .size(TYPE_SM),
            );
            let mut sym = app.document.complex_base_symbol.clone();
            let resp = ui.add(
                egui::TextEdit::singleline(&mut sym)
                    .desired_width(ui.available_width())
                    .hint_text(COMPLEX_BASE_SYMBOL_HINT),
            );
            if resp.lost_focus() && sym.trim() != app.document.complex_base_symbol {
                let new_sym = sym.trim().to_string();
                if !new_sym.is_empty() {
                    app.document.migrate_complex_symbol(&new_sym);
                    app.document.bump_version();
                }
            }

            ui.add_space(SPACE_SM);

            // ── Integral de contorno dibujada a mano ──────────────────────
            // f(z) + guías → arma la herramienta `ComplexContour`: el trazo
            // acumula ∮ f dz en vivo y al soltar quedan Pencil + Integral.
            ui.label(
                egui::RichText::new(COMPLEX_CONTOUR_TITLE)
                    .color(hdr_col)
                    .size(TYPE_SM)
                    .strong(),
            );
            ui.add_space(SPACE_XS);
            // Botón a la derecha con auto-ancho + campo con el resto vivo
            // (antes `available-30` cortaba el botón en ES/DE largo).
            ui.horizontal(|ui| {
                let can_prefill = app.selected_complex_expr().is_some();
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add_enabled(
                            can_prefill,
                            egui::Button::new(
                                egui::RichText::new(COMPLEX_CONTOUR_PREFILL).size(TYPE_SM),
                            )
                            .frame(false),
                        )
                        .on_hover_text(COMPLEX_CONTOUR_PREFILL_TOOLTIP)
                        .clicked()
                    {
                        if let Some(expr) = app.selected_complex_expr() {
                            app.complex_contour.settings.expr = expr;
                        }
                    }
                });
                ui.add(
                    egui::TextEdit::singleline(&mut app.complex_contour.settings.expr)
                        .desired_width(ui.available_width().max(40.0))
                        .hint_text(COMPLEX_CONTOUR_EXPR_HINT),
                );
            });
            // Switches a la derecha (S6b): una fila de 56 por opción (el par
            // horizontal de cajas no entraba en el drawer angosto).
            grafito_ui::assistant::drill_switch(
                ui,
                COMPLEX_CONTOUR_RESIDUES_LABEL,
                "",
                &mut app.complex_contour.settings.residues,
                true,
            );
            grafito_ui::assistant::drill_switch(
                ui,
                COMPLEX_CONTOUR_SNAP_LABEL,
                "",
                &mut app.complex_contour.settings.snap,
                true,
            );
            grafito_ui::assistant::drill_switch(
                ui,
                COMPLEX_CONTOUR_CLOSE_LABEL,
                "",
                &mut app.complex_contour.settings.auto_close,
                true,
            );
            grafito_ui::assistant::drill_switch(
                ui,
                COMPLEX_CONTOUR_SMOOTH_LABEL,
                "",
                &mut app.complex_contour.settings.smooth,
                true,
            );
            let mut stabilizer = app.complex_contour.settings.stabilizer;
            ui.spacing_mut().slider_width = ui.available_width().max(0.0);
            ui.add(
                egui::Slider::new(&mut stabilizer, 0.0..=0.8)
                    .text(COMPLEX_CONTOUR_STABILIZER_LABEL)
                    .show_value(false),
            );
            app.complex_contour.settings.stabilizer = stabilizer;
            let armed = app.current_tool == grafito_ui::Tool::ComplexContour;
            ui.horizontal(|ui| {
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new(COMPLEX_CONTOUR_DRAW_LABEL).size(TYPE_XS),
                        )
                        .frame(true),
                    )
                    .on_hover_text(COMPLEX_CONTOUR_DRAW_TOOLTIP)
                    .clicked()
                {
                    let _ = app
                        .arm_complex_contour(crate::complex_contour::ComplexContourMode::Freehand);
                }
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new(COMPLEX_CONTOUR_CIRCLE_LABEL).size(TYPE_XS),
                        )
                        .frame(true),
                    )
                    .on_hover_text(COMPLEX_CONTOUR_CIRCLE_TOOLTIP)
                    .clicked()
                {
                    let _ =
                        app.arm_complex_contour(crate::complex_contour::ComplexContourMode::Circle);
                }
            });
            if armed {
                ui.label(
                    egui::RichText::new(complex_contour_armed_text(
                        app.complex_contour.settings.mode,
                    ))
                    .color(accent)
                    .size(TYPE_XS),
                );
            } else if let Some(value) = &app.complex_contour.last_value {
                ui.label(
                    egui::RichText::new(complex_contour_last_value_text(value))
                        .color(txt_dim)
                        .size(TYPE_XS),
                );
            }

            ui.add_space(SPACE_SM);
            let content_height = ui.available_height();
            panel_scroll_content(ui, "complex_panel_content", Some(content_height), |ui| {
                ui.label(
                    egui::RichText::new(COMPLEX_OBJECTS_TITLE)
                        .color(hdr_col)
                        .size(TYPE_SM)
                        .strong(),
                );
                ui.add_space(SPACE_XS);
                let ids: Vec<ObjectId> = app.document.objects_iter().map(|(id, _)| *id).collect();
                let mut any_object = false;
                for id in &ids {
                    let Some(obj) = app.document.get_object(*id) else {
                        continue;
                    };
                    if !matches!(
                        obj,
                        GeoObject::Function(_)
                            | GeoObject::ImplicitCurve(_)
                            | GeoObject::ParametricCurve2D(_)
                            | GeoObject::PolarCurve(_)
                            | GeoObject::VectorField2D(_)
                            | GeoObject::ComplexGrid(_)
                            | GeoObject::ComplexMapping(_)
                            | GeoObject::Point(_)
                            | GeoObject::Line(_)
                            | GeoObject::Circle(_)
                            | GeoObject::Polygon(_)
                            | GeoObject::Ellipse(_)
                            | GeoObject::Parabola(_)
                            | GeoObject::Hyperbola(_)
                    ) {
                        continue;
                    }
                    any_object = true;
                    crate::algebra::draw_object_card(ui, app, *id);
                }
                if !any_object {
                    ui.label(
                        egui::RichText::new(COMPLEX_OBJECTS_EMPTY)
                            .color(txt_dim)
                            .size(TYPE_XS),
                    );
                }

                ui.add_space(SPACE_SM);
                ui.label(
                    egui::RichText::new(COMPLEX_QUICK_TITLE)
                        .color(hdr_col)
                        .size(TYPE_SM)
                        .strong(),
                );
                ui.add_space(2.0);
                // Atajos ejecutables: cada uno crea una visualización cuya
                // semántica coincide con su etiqueta.
                let hints: &[(&str, &str)] = &COMPLEX_QUICK_HINTS;
                for (label, payload) in hints {
                    let b = ui.add(
                        egui::Button::new(
                            egui::RichText::new(*label)
                                .monospace()
                                .size(TYPE_XS)
                                .color(txt_col),
                        )
                        .frame(false),
                    );
                    if b.clicked() {
                        app.input_text = payload.to_string();
                        let time = ui.ctx().input(|input| input.time);
                        app.submit_input_text(time);
                    }
                    b.on_hover_text(complex_quick_hover_text(payload));
                }
            });
        });
    snapshot.save_if_semantically_changed(
        &mut app.document,
        &mut app.undo_stack,
        &mut app.redo_stack,
    );
}

/// Panel izquierdo de Atractores y Dinámica.
pub(crate) fn draw_attractor_panel(app: &mut GrafitoApp, ctx: &egui::Context) {
    use grafito_core::GeoObject;
    let (_is_dark, accent, alg_fill, _sep_col, txt_col, txt_dim, hdr_col) = panel_theme_local(ctx);
    // B-A189: chrome Atractores por dispatcher (Es→consts canónicas,
    // resto→`dynamics_msg`); comando `ATTRACTOR_DEFAULT_COMMAND` +
    // lecturas `attractor_*_text` con notación intactos.
    let attractor_locale = app.config_locale();
    let attractor_panel_title = dynamics_chrome_for("attractor_panel_title", attractor_locale);
    let attractor_objects_title = dynamics_chrome_for("attractor_objects_title", attractor_locale);
    let attractor_objects_empty = dynamics_chrome_for("attractor_objects_empty", attractor_locale);
    let attractor_active_title = dynamics_chrome_for("attractor_active_title", attractor_locale);
    let attractor_inactive_guide =
        dynamics_chrome_for("attractor_inactive_guide", attractor_locale);
    let attractor_create_default =
        dynamics_chrome_for("attractor_create_default", attractor_locale);
    let attractor_commands_title =
        dynamics_chrome_for("attractor_commands_title", attractor_locale);
    let attractor_lorenz_hint = dynamics_chrome_for("attractor_lorenz_hint", attractor_locale);

    egui::SidePanel::left("attractor_panel")
        .show_separator_line(false)
        .exact_width(panel_left_width(ctx))
        .resizable(false)
        .frame(
            egui::Frame::none()
                .fill(alg_fill)
                .stroke(egui::Stroke::NONE),
        )
        .show(ctx, |ui| {
            ui.add_space(SPACE_SM);
            ui.label(
                egui::RichText::new(attractor_panel_title)
                    .color(accent)
                    .size(grafito_ui::tokens::TYPE_BASE)
                    .strong(),
            );
            ui.add_space(SPACE_SM);

            draw_object_cards_where(
                ui,
                app,
                attractor_objects_title,
                attractor_objects_empty,
                |obj| {
                    matches!(
                        obj,
                        GeoObject::Attractor3D(_)
                            | GeoObject::PhasePortrait(_)
                            | GeoObject::VectorField2D(_)
                            | GeoObject::VectorField3D(_)
                    )
                },
            );
            ui.add_space(SPACE_SM);

            let ids: Vec<_> = app.document.objects_iter().map(|(id, _)| *id).collect();
            let mut attractor_id = None;
            for id in &ids {
                if let Some(GeoObject::Attractor3D(_)) = app.document.get_object(*id) {
                    attractor_id = Some(*id);
                    break;
                }
            }

            if let Some(id) = attractor_id {
                ui.label(
                    egui::RichText::new(attractor_active_title)
                        .color(hdr_col)
                        .size(TYPE_SM)
                        .strong(),
                );
                if let Some(GeoObject::Attractor3D(a)) = app.document.get_object(id) {
                    let sigma = a.params.first().copied().unwrap_or(0.0);
                    let rho = a.params.get(1).copied().unwrap_or(0.0);
                    let beta = a.params.get(2).copied().unwrap_or(0.0);
                    ui.label(attractor_sigma_text(sigma));
                    ui.label(attractor_rho_text(rho));
                    ui.label(attractor_beta_text(beta));
                    ui.label(attractor_dt_text(a.dt));
                    ui.label(attractor_steps_text(a.steps));
                }
            } else {
                ui.label(
                    egui::RichText::new(attractor_inactive_guide)
                        .color(txt_dim)
                        .size(TYPE_XS),
                );
                ui.add_space(SPACE_XS);
                if ui
                    .button(
                        egui::RichText::new(attractor_create_default)
                            .color(accent)
                            .strong(),
                    )
                    .clicked()
                {
                    app.save_state();
                    app.execute_command_and_record(ATTRACTOR_DEFAULT_COMMAND, 0.0);
                }
            }

            ui.add_space(SPACE_SM);
            ui.label(
                egui::RichText::new(attractor_commands_title)
                    .color(hdr_col)
                    .size(TYPE_SM)
                    .strong(),
            );
            ui.label(
                egui::RichText::new(attractor_lorenz_hint)
                    .color(txt_dim)
                    .size(TYPE_XS)
                    .monospace(),
            );
            let _ = txt_col;
        });
}

/// Laboratorios de física por rama (mecánica, ondas, electromagnetismo,
/// gravitación): cada botón carga el comando en la Entrada con sus
/// parámetros a la vista para retocarlos antes de correrlo. Las unidades
/// son las que el motor ya anuncia en sus mensajes (m/s, m, s, A, W, N);
/// donde el motor no da unidades no se etiqueta ninguna.
const FISICA_LABS: &[(&str, &str, &str)] = &[
    (
        "Mecánica",
        "TrayectoriaProyectil[20, 45, 1.5]",
        "v0=20 m/s, ángulo=45°, h0=1.5 m, g=9.8 — alcance en m, vuelo en s.",
    ),
    (
        "Ondas",
        "VelocidadOnda[2.0, 3.0]",
        "λ=2.0, f=3.0 — v en m/s, T en s.",
    ),
    (
        "Ondas",
        "OndaEstacionaria[10, 3, 1]",
        "L=10, modo 3 — cuerda con nodos fijos.",
    ),
    (
        "Electromagnetismo",
        "CampoCargas[1, -1, 0, -1, 1, 0]",
        "Dipolo: campo 2D por superposición (color = |E| log).",
    ),
    (
        "Electromagnetismo",
        "LeyOhm[12, 4]",
        "V=12, R=4 — I en A, P en W.",
    ),
    (
        "Gravitación",
        "LeyGravitacion[5.972e24, 7.342e22, 3.844e8]",
        "Tierra–Luna — F en N.",
    ),
];

/// Panel izquierdo de Física (tab Herram. cuando manda `Fisica`).
/// Piel pura: solo pinta y carga texto en la Entrada, sin I/O.
pub(crate) fn draw_fisica_panel(app: &mut GrafitoApp, ctx: &egui::Context) {
    let (_is_dark, accent, alg_fill, _sep_col, _txt_col, txt_dim, hdr_col) = panel_theme_local(ctx);
    egui::SidePanel::left("fisica_panel")
        .show_separator_line(false)
        .exact_width(panel_left_width(ctx))
        .resizable(false)
        .frame(
            egui::Frame::none()
                .fill(alg_fill)
                .stroke(egui::Stroke::NONE),
        )
        .show(ctx, |ui| {
            ui.add_space(SPACE_SM);
            ui.label(
                egui::RichText::new("Física")
                    .color(accent)
                    .size(grafito_ui::tokens::TYPE_BASE)
                    .strong(),
            );
            ui.label(
                egui::RichText::new(
                    "Tocá un laboratorio para cargarlo en la Entrada,\nretocá los parámetros y Enter.",
                )
                .color(txt_dim)
                .size(TYPE_XS),
            );
            ui.add_space(SPACE_SM);
            let mut rama = "";
            for (lab_rama, comando, detalle) in FISICA_LABS {
                if *lab_rama != rama {
                    rama = lab_rama;
                    ui.label(
                        egui::RichText::new(*lab_rama)
                            .color(hdr_col)
                            .size(TYPE_SM)
                            .strong(),
                    );
                }
                if ui.button(egui::RichText::new(*comando).monospace()).clicked() {
                    app.input_text = (*comando).to_string();
                }
                ui.label(
                    egui::RichText::new(*detalle)
                        .color(txt_dim)
                        .size(TYPE_XS),
                );
                ui.add_space(SPACE_XS);
            }
        });
}

// ══════════════════════════════════════════════════════════════════════════
// Paneles derechos (Fase 3)
// ══════════════════════════════════════════════════════════════════════════

/// Panel derecho: Propiedades del objeto seleccionado (Geometry3D).
pub(crate) fn draw_right_properties_panel(app: &mut GrafitoApp, ctx: &egui::Context) {
    let theme = current_theme(ctx);
    egui::SidePanel::right("right_properties")
        .show_separator_line(false)
        .default_width(340.0)
        .min_width(DRAWER_RIGHT_MIN)
        .max_width(DRAWER_RIGHT_MAX)
        .resizable(true)
        .frame(
            egui::Frame::none()
                .fill(theme.panel_bg)
                .stroke(egui::Stroke::NONE),
        )
        .show(ctx, |ui| {
            ui.add_space(SPACE_SM);
            // B-A108: chrome del Inspector en el idioma activo (antes ES duro).
            let inspector_locale = app.config_locale();
            let inspector_panel_title = inspector_chrome_for("panel_title", inspector_locale);
            let inspector_panel_subtitle = inspector_chrome_for("panel_subtitle", inspector_locale);
            draw_right_drawer_header(
                ui,
                app,
                inspector_panel_title,
                inspector_panel_subtitle,
                theme.accent,
            );
            ui.add_space(SPACE_SM);
            draw_right_properties_contents(app, ui);
        });
}

/// Contenido reutilizable del Inspector de propiedades para un dock anfitrión.
pub(crate) fn draw_right_properties_contents(app: &mut GrafitoApp, ui: &mut egui::Ui) {
    let mut snapshot = crate::app::DeferredPanelSnapshot::new(app.undo_stack.len());
    use grafito_core::GeoObject;
    // B-A108: chrome del Inspector en el idioma activo (antes ES duro).
    // B-A109: sección 4D/N-D por dispatcher (antes consts ES directas).
    let inspector_locale = app.config_locale();
    let inspector_missing_label = inspector_chrome_for("missing_object", inspector_locale);
    let inspector_equation_label = inspector_chrome_for("equation_label", inspector_locale);
    let inspector_revert_label = inspector_chrome_for("revert", inspector_locale);
    let inspector_use_example_label = inspector_chrome_for("use_example", inspector_locale);
    let inspector_apply_label = inspector_chrome_for("apply", inspector_locale);
    let inspector_style_title_label = inspector_chrome_for("style_title", inspector_locale);
    let inspector_style_subtitle_label = inspector_chrome_for("style_subtitle", inspector_locale);
    let inspector_style_none_label = inspector_chrome_for("style_none_hint", inspector_locale);
    let inspector_layer_label_text = inspector_chrome_for("layer_label", inspector_locale);
    let poly_projection_title = inspector_chrome_for("poly_projection_title", inspector_locale);
    let poly_projection_subtitle =
        inspector_chrome_for("poly_projection_subtitle", inspector_locale);
    let poly_anim_title = inspector_chrome_for("poly_anim_title", inspector_locale);
    let poly_anim_body = inspector_chrome_for("poly_anim_body", inspector_locale);
    let poly_geometry_title = inspector_chrome_for("poly_geometry_title", inspector_locale);
    let poly_geometry_subtitle = inspector_chrome_for("poly_geometry_subtitle", inspector_locale);
    let poly_appearance_title = inspector_chrome_for("poly_appearance_title", inspector_locale);
    let poly_appearance_subtitle =
        inspector_chrome_for("poly_appearance_subtitle", inspector_locale);
    let poly_fill_omitted_hint = inspector_chrome_for("poly_fill_omitted_hint", inspector_locale);
    let poly_rotation_manual_title =
        inspector_chrome_for("poly_rotation_manual_title", inspector_locale);
    let poly_rotation_manual_guide =
        inspector_chrome_for("poly_rotation_manual_guide", inspector_locale);
    let poly_rotation_reset_label =
        inspector_chrome_for("poly_rotation_reset_label", inspector_locale);
    let poly_rotation_planes_title =
        inspector_chrome_for("poly_rotation_planes_title", inspector_locale);
    let polytope_geometry_subtitle =
        inspector_chrome_for("polytope_geometry_subtitle", inspector_locale);
    let polytope_projection_subtitle =
        inspector_chrome_for("polytope_projection_subtitle", inspector_locale);
    let polytope_appearance_subtitle =
        inspector_chrome_for("polytope_appearance_subtitle", inspector_locale);
    let polytope_fill_hint = inspector_chrome_for("polytope_fill_hint", inspector_locale);
    let polytope_rotation_guide = inspector_chrome_for("polytope_rotation_guide", inspector_locale);
    // B-A112: primitivas 3D + sliders por dispatcher (antes consts ES directas).
    let inspector_cube3d_title = inspector_chrome_for("cube3d_title", inspector_locale);
    let inspector_sphere3d_title = inspector_chrome_for("sphere3d_title", inspector_locale);
    let inspector_point3d_title = inspector_chrome_for("point3d_title", inspector_locale);
    let inspector_center_label = inspector_chrome_for("center_label", inspector_locale);
    let inspector_implicit3d_title = inspector_chrome_for("implicit3d_title", inspector_locale);
    let inspector_dock_empty_title = inspector_chrome_for("dock_empty_title", inspector_locale);
    let inspector_dock_empty_guide = inspector_chrome_for("dock_empty_guide", inspector_locale);
    let inspector_size_label = inspector_chrome_for("size_label", inspector_locale);
    let inspector_radius_label = inspector_chrome_for("radius_label", inspector_locale);
    let inspector_scale_label = inspector_chrome_for("scale_label", inspector_locale);
    let inspector_edge_width_label = inspector_chrome_for("edge_width_label", inspector_locale);
    // B-A159: apariencia 4D + cierres por dispatcher (antes ES duro).
    let inspector_edge_color_label = inspector_chrome_for("edge_color_label", inspector_locale);
    let inspector_edge_color_hover = inspector_chrome_for("edge_color_hover", inspector_locale);
    let inspector_fill_enabled_label = inspector_chrome_for("fill_enabled_label", inspector_locale);
    let inspector_fill_color_label = inspector_chrome_for("fill_color_label", inspector_locale);
    let inspector_fill_color_hover = inspector_chrome_for("fill_color_hover", inspector_locale);
    let inspector_dimension_label = inspector_chrome_for("dimension_label", inspector_locale);
    let (_is_dark, _accent, _alg_fill, _sep_col, txt_col, txt_dim, _hdr_col) =
        panel_theme_local(ui.ctx());
    let theme = current_theme(ui.ctx());

    egui::ScrollArea::vertical()
        .id_salt("right_properties_scroll")
        .auto_shrink([false, true])
        .show(ui, |ui| {
            // Wrap honesto: sin esto, las filas de texto del Inspector
            // (p. ej. la ecuación en `TYPE_MD`) se clipan en el borde
            // del dock y dejan slivers de 1-2px. Las filas
            // horizontales de controles (DragValue/Slider) no se ven
            // afectadas.
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
            let Some(id) = app.selected_object else {
                draw_inspector_empty_state(ui, inspector_locale);
                return;
            };
            let Some(mut edited_object) = app.document.get_object(id).cloned() else {
                ui.label(egui::RichText::new(inspector_missing_label).color(txt_dim));
                return;
            };
            draw_inspector_identity(ui, &edited_object, inspector_locale);
            ui.add_space(SPACE_XS);
            // D1-bis: editor de ecuación (pipeline D3). El display vive
            // en `draw_inspector_identity`; acá el campo editable con
            // validación en vivo y commit atómico + undo. El borrador
            // vive en memoria temporal egui (sobrevive frames, no se
            // persiste) claveado por objeto.
            {
                use crate::inspector_edit as ieq;
                if let Some(live_now) = app.document.get_object(id).cloned() {
                    if ieq::is_editable(&live_now) {
                        let eq_key = egui::Id::new(("inspector_equation", id));
                        let mut eq_state: ieq::InspectorEditState = ui
                            .ctx()
                            .memory(|mem| mem.data.get_temp(eq_key))
                            .unwrap_or_else(|| {
                                ieq::begin_edit(&live_now).unwrap_or(ieq::InspectorEditState {
                                    draft: String::new(),
                                    error: None,
                                    editing: false,
                                    base: String::new(),
                                })
                            });
                        // Sin edición en curso, re-sincroniza con la
                        // canónica (cambios desde Álgebra u otros).
                        if !eq_state.editing {
                            if let Some(fresh) = ieq::begin_edit(&live_now) {
                                eq_state = fresh;
                            }
                        }
                        // El drag del canvas (u otro cambio externo) pudo
                        // mover el objeto con el draft abierto: rebasea a
                        // la canónica nueva en vez de aplicar sobre
                        // rancio (con borrador fresco es no-op).
                        let _ = ieq::rebase_if_stale(&mut eq_state, &live_now);
                        ui.label(
                            egui::RichText::new(inspector_equation_label)
                                .color(txt_dim)
                                .size(TYPE_XS),
                        );
                        let mut draft = eq_state.draft.clone();
                        let eq_resp = ui.add(
                            egui::TextEdit::singleline(&mut draft)
                                .hint_text(ieq::hint_for(&live_now))
                                .desired_width(f32::INFINITY),
                        );
                        // A11Y foco visible en ecuación del inspector
                        // (anillo 2px del tema, ref `toolbar.rs:714`).
                        if eq_resp.has_focus() {
                            current_theme(ui.ctx()).paint_focus_ring(ui.painter(), eq_resp.rect);
                        }
                        if eq_resp.changed() {
                            ieq::update_draft(&mut eq_state, &draft);
                            ieq::revalidate_state(&mut eq_state, &live_now);
                        }
                        if let Some(err) = eq_state.error.clone() {
                            ui.label(
                                egui::RichText::new(err)
                                    .color(current_theme(ui.ctx()).danger)
                                    .size(TYPE_XS),
                            );
                            // W-A: inválido mantiene el último válido
                            // (el documento no se tocó) + "Revertir"
                            // vuelve a la canónica + "Usar ejemplo"
                            // inserta sintaxis válida clicable.
                            ui.horizontal(|ui| {
                                if ui.small_button(inspector_revert_label).clicked() {
                                    ieq::cancel_edit(&mut eq_state, &live_now);
                                }
                                if let Some(example) = ieq::example_for(&live_now) {
                                    if ui
                                        .small_button(inspector_use_example_label)
                                        .on_hover_text(inspector_example_tooltip_for(
                                            example,
                                            inspector_locale,
                                        ))
                                        .clicked()
                                    {
                                        ieq::update_draft(&mut eq_state, example);
                                        ieq::revalidate_state(&mut eq_state, &live_now);
                                    }
                                }
                            });
                        }
                        let canonical_now = live_now.canonical_equation_text().unwrap_or_default();
                        let can_apply = eq_state.error.is_none() && eq_state.draft != canonical_now;
                        let apply_clicked = ui
                            .add_enabled(can_apply, egui::Button::new(inspector_apply_label))
                            .clicked();
                        let enter_pressed =
                            eq_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                        if apply_clicked || enter_pressed {
                            // Con chequeo de rancio: si el objeto cambió
                            // desde que se abrió el borrador, falla
                            // honesto sin aplicar (el rebase de arriba
                            // ya refrescó lo visible por drag).
                            match ieq::commit_draft_stale_checked(&mut app.document, id, &eq_state)
                            {
                                Ok(Some(before)) => {
                                    snapshot.capture_successful_replacement(before);
                                    if let Some(updated) = app.document.get_object(id).cloned() {
                                        ieq::cancel_edit(&mut eq_state, &updated);
                                    }
                                }
                                Ok(None) => {}
                                Err(message) => {
                                    eq_state.error = Some(message.clone());
                                    app.notify(message, grafito_ui::toast::ToastKind::Error);
                                }
                            }
                        }
                        ui.ctx().memory_mut(|mem| {
                            mem.data.insert_temp(eq_key, eq_state);
                        });
                    }
                }
            }
            // Medida exacta del sólido (volumen/área) cuando el motor
            // la calcula en forma cerrada; cuádricas → `None` honesto.
            if let Some(measure) = crate::render_3d::solid_measure_text(&edited_object) {
                ui.label(
                    egui::RichText::new(inspector_measure_text_for(&measure, inspector_locale))
                        .color(txt_dim)
                        .size(TYPE_XS),
                );
            }
            ui.add_space(SPACE_MD);
            let mut changed = false;
            let mut layer_value = app.document.layer_of(id);
            let layer_before = layer_value;

            // Q2: estilos reales + capa. Trazo/marcador mutan el
            // clon y entran al undo por el `apply` común de abajo;
            // la capa vive en el documento y se aplica con su propio
            // snapshot justo después.
            draw_inspector_section(
                ui,
                inspector_style_title_label,
                inspector_style_subtitle_label,
                |ui| {
                    if let Some(current) = edited_object.line_style() {
                        let mut next = current;
                        egui::ComboBox::from_id_salt("inspector_line_style")
                            .selected_text(line_style_label_for(next, inspector_locale))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut next,
                                    LineStyle::Solid,
                                    line_style_label_for(LineStyle::Solid, inspector_locale),
                                );
                                ui.selectable_value(
                                    &mut next,
                                    LineStyle::Dashed,
                                    line_style_label_for(LineStyle::Dashed, inspector_locale),
                                );
                                ui.selectable_value(
                                    &mut next,
                                    LineStyle::Dotted,
                                    line_style_label_for(LineStyle::Dotted, inspector_locale),
                                );
                            });
                        if next != current {
                            edited_object.set_line_style(next);
                            changed = true;
                        }
                    }
                    if let Some(current) = edited_object.point_style() {
                        let mut next = current;
                        egui::ComboBox::from_id_salt("inspector_point_style")
                            .selected_text(point_style_label_for(next, inspector_locale))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut next,
                                    PointStyle::Dot,
                                    point_style_label_for(PointStyle::Dot, inspector_locale),
                                );
                                ui.selectable_value(
                                    &mut next,
                                    PointStyle::Circle,
                                    point_style_label_for(PointStyle::Circle, inspector_locale),
                                );
                                ui.selectable_value(
                                    &mut next,
                                    PointStyle::Cross,
                                    point_style_label_for(PointStyle::Cross, inspector_locale),
                                );
                                ui.selectable_value(
                                    &mut next,
                                    PointStyle::Plus,
                                    point_style_label_for(PointStyle::Plus, inspector_locale),
                                );
                            });
                        if next != current {
                            edited_object.set_point_style(next);
                            changed = true;
                        }
                    }
                    if edited_object.line_style().is_none() && edited_object.point_style().is_none()
                    {
                        ui.label(
                            egui::RichText::new(inspector_style_none_label)
                                .color(txt_dim)
                                .size(TYPE_XS),
                        );
                    }
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(inspector_layer_label_text)
                                .color(txt_dim)
                                .size(TYPE_SM),
                        );
                        ui.add(
                            egui::DragValue::new(&mut layer_value)
                                .range(0..=MAX_LAYERS)
                                .speed(1),
                        );
                    });
                },
            );
            ui.add_space(SPACE_XS);

            let label_col = theme.text_secondary;
            match &mut edited_object {
                GeoObject::Cube3D(c) => {
                    ui.label(
                        egui::RichText::new(inspector_cube3d_title)
                            .color(label_col)
                            .strong(),
                    );
                    ui.label(
                        egui::RichText::new(inspector_3d_tag_label_for(&c.label, inspector_locale))
                            .color(txt_col),
                    );
                    ui.add_space(SPACE_XS);
                    ui.label(egui::RichText::new(inspector_center_label).color(txt_dim));
                    ui.horizontal(|ui| {
                        changed |= ui
                            .add(
                                egui::DragValue::new(&mut c.center.x)
                                    .speed(INSPECTOR_CENTER_DRAG_SPEED)
                                    .prefix(INSPECTOR_AXIS_PREFIX_X),
                            )
                            .changed();
                        changed |= ui
                            .add(
                                egui::DragValue::new(&mut c.center.y)
                                    .speed(INSPECTOR_CENTER_DRAG_SPEED)
                                    .prefix(INSPECTOR_AXIS_PREFIX_Y),
                            )
                            .changed();
                        changed |= ui
                            .add(
                                egui::DragValue::new(&mut c.center.z)
                                    .speed(INSPECTOR_CENTER_DRAG_SPEED)
                                    .prefix(INSPECTOR_AXIS_PREFIX_Z),
                            )
                            .changed();
                    });
                    changed |= ui
                        .add(
                            egui::Slider::new(&mut c.size, inspector_cube_size_range())
                                .text(inspector_size_label),
                        )
                        .changed();
                }
                GeoObject::Sphere3D(s) => {
                    ui.label(
                        egui::RichText::new(inspector_sphere3d_title)
                            .color(label_col)
                            .strong(),
                    );
                    ui.label(
                        egui::RichText::new(inspector_3d_tag_label_for(&s.label, inspector_locale))
                            .color(txt_col),
                    );
                    ui.add_space(SPACE_XS);
                    ui.label(egui::RichText::new(inspector_center_label).color(txt_dim));
                    ui.horizontal(|ui| {
                        changed |= ui
                            .add(
                                egui::DragValue::new(&mut s.center.x)
                                    .speed(INSPECTOR_CENTER_DRAG_SPEED)
                                    .prefix(INSPECTOR_AXIS_PREFIX_X),
                            )
                            .changed();
                        changed |= ui
                            .add(
                                egui::DragValue::new(&mut s.center.y)
                                    .speed(INSPECTOR_CENTER_DRAG_SPEED)
                                    .prefix(INSPECTOR_AXIS_PREFIX_Y),
                            )
                            .changed();
                        changed |= ui
                            .add(
                                egui::DragValue::new(&mut s.center.z)
                                    .speed(INSPECTOR_CENTER_DRAG_SPEED)
                                    .prefix(INSPECTOR_AXIS_PREFIX_Z),
                            )
                            .changed();
                    });
                    changed |= ui
                        .add(
                            egui::Slider::new(&mut s.radius, inspector_sphere_radius_range())
                                .text(inspector_radius_label),
                        )
                        .changed();
                }
                GeoObject::Point3D(p) => {
                    ui.label(
                        egui::RichText::new(inspector_point3d_title)
                            .color(label_col)
                            .strong(),
                    );
                    ui.label(
                        egui::RichText::new(inspector_3d_tag_label_for(&p.label, inspector_locale))
                            .color(txt_col),
                    );
                    ui.add_space(SPACE_XS);
                    ui.horizontal(|ui| {
                        changed |= ui
                            .add(
                                egui::DragValue::new(&mut p.position.x)
                                    .speed(INSPECTOR_CENTER_DRAG_SPEED)
                                    .prefix(INSPECTOR_AXIS_PREFIX_X),
                            )
                            .changed();
                        changed |= ui
                            .add(
                                egui::DragValue::new(&mut p.position.y)
                                    .speed(INSPECTOR_CENTER_DRAG_SPEED)
                                    .prefix(INSPECTOR_AXIS_PREFIX_Y),
                            )
                            .changed();
                        changed |= ui
                            .add(
                                egui::DragValue::new(&mut p.position.z)
                                    .speed(INSPECTOR_CENTER_DRAG_SPEED)
                                    .prefix(INSPECTOR_AXIS_PREFIX_Z),
                            )
                            .changed();
                    });
                    changed |= ui
                        .add(
                            egui::Slider::new(&mut p.size, inspector_point_size_range())
                                .text(inspector_size_label),
                        )
                        .changed();
                }
                GeoObject::RegularPolychoron4D(polychoron) => {
                    ui.push_id(("regular_polychoron_4d", id), |ui| {
                        draw_inspector_section(
                            ui,
                            poly_projection_title,
                            poly_projection_subtitle,
                            |ui| {
                                draw_multidimensional_motion_card(
                                    ui,
                                    app,
                                    poly_anim_title,
                                    poly_anim_body,
                                    polychoron.visible && app.current_view == crate::ViewMode::D3,
                                );
                            },
                        );
                        ui.add_space(SPACE_MD);
                        draw_inspector_section(
                            ui,
                            poly_geometry_title,
                            poly_geometry_subtitle,
                            |ui| {
                                let mut kind = polychoron.kind;
                                let combo_width = ui.available_width();
                                egui::ComboBox::from_id_salt("regular_polychoron_kind")
                                    .width(combo_width)
                                    .selected_text(polychoron_kind_label_for(
                                        kind,
                                        inspector_locale,
                                    ))
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            &mut kind,
                                            RegularPolychoron::Pentachoron,
                                            polychoron_kind_label_for(
                                                RegularPolychoron::Pentachoron,
                                                inspector_locale,
                                            ),
                                        );
                                        ui.selectable_value(
                                            &mut kind,
                                            RegularPolychoron::Tesseract,
                                            polychoron_kind_label_for(
                                                RegularPolychoron::Tesseract,
                                                inspector_locale,
                                            ),
                                        );
                                        ui.selectable_value(
                                            &mut kind,
                                            RegularPolychoron::SixteenCell,
                                            polychoron_kind_label_for(
                                                RegularPolychoron::SixteenCell,
                                                inspector_locale,
                                            ),
                                        );
                                        ui.selectable_value(
                                            &mut kind,
                                            RegularPolychoron::TwentyFourCell,
                                            polychoron_kind_label_for(
                                                RegularPolychoron::TwentyFourCell,
                                                inspector_locale,
                                            ),
                                        );
                                        ui.selectable_value(
                                            &mut kind,
                                            RegularPolychoron::OneTwentyCell,
                                            polychoron_kind_label_for(
                                                RegularPolychoron::OneTwentyCell,
                                                inspector_locale,
                                            ),
                                        );
                                        ui.selectable_value(
                                            &mut kind,
                                            RegularPolychoron::SixHundredCell,
                                            polychoron_kind_label_for(
                                                RegularPolychoron::SixHundredCell,
                                                inspector_locale,
                                            ),
                                        );
                                    });
                                if kind != polychoron.kind {
                                    polychoron.kind = kind;
                                    changed = true;
                                }

                                changed |= ui
                                    .add(
                                        egui::Slider::new(
                                            &mut polychoron.scale,
                                            inspector_poly_scale_range(),
                                        )
                                        .text(inspector_scale_label),
                                    )
                                    .changed();
                                changed |= ui
                                    .add(
                                        egui::Slider::new(
                                            &mut polychoron.width,
                                            inspector_poly_width_range(),
                                        )
                                        .text(inspector_edge_width_label),
                                    )
                                    .changed();
                            },
                        );

                        ui.add_space(SPACE_MD);
                        draw_inspector_section(
                            ui,
                            poly_appearance_title,
                            poly_appearance_subtitle,
                            |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(inspector_edge_color_label);
                                    if color_picker_swatch(
                                        ui,
                                        polychoron.color,
                                        inspector_edge_color_hover,
                                    )
                                    .clicked()
                                    {
                                        app.open_object_color_picker(id);
                                    }
                                });

                                let mut fill_enabled = polychoron.fill_color.is_some();
                                if grafito_ui::assistant::scandi_checkbox(
                                    ui,
                                    inspector_fill_enabled_label,
                                    &mut fill_enabled,
                                ) {
                                    polychoron.fill_color = fill_enabled.then(|| {
                                        polychoron
                                            .fill_color
                                            .unwrap_or(Color::new(0.2, 0.5, 0.9, 0.55))
                                    });
                                    changed = true;
                                }
                                if let Some(fill_color) = polychoron.fill_color {
                                    ui.horizontal(|ui| {
                                        ui.label(inspector_fill_color_label);
                                        if color_picker_swatch(
                                            ui,
                                            fill_color,
                                            inspector_fill_color_hover,
                                        )
                                        .clicked()
                                        {
                                            app.open_regular_polychoron_fill_color_picker(id);
                                        }
                                    });
                                }
                                ui.label(
                                    egui::RichText::new(poly_fill_omitted_hint)
                                        .color(txt_dim)
                                        .size(TYPE_XS),
                                );
                            },
                        );

                        ui.add_space(SPACE_MD);
                        egui::CollapsingHeader::new(poly_rotation_manual_title)
                            .default_open(false)
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(poly_rotation_manual_guide)
                                        .color(txt_dim)
                                        .size(TYPE_SM),
                                );
                                ui.add_space(SPACE_SM);
                                if ui.button(poly_rotation_reset_label).clicked()
                                    && polychoron.rotation_angles != [0.0; 6]
                                {
                                    polychoron.rotation_angles = [0.0; 6];
                                    changed = true;
                                }
                                ui.label(
                                    egui::RichText::new(poly_rotation_planes_title)
                                        .color(txt_dim)
                                        .size(TYPE_SM),
                                );
                                egui::Grid::new("regular_polychoron_rotation_planes").show(
                                    ui,
                                    |ui| {
                                        for (angle, plane) in polychoron
                                            .rotation_angles
                                            .iter_mut()
                                            .zip(POLYCHORON_ROTATION_PLANES)
                                        {
                                            ui.label(
                                                egui::RichText::new(plane)
                                                    .monospace()
                                                    .size(TYPE_SM),
                                            );
                                            changed |= ui
                                                .add(
                                                    egui::Slider::new(
                                                        angle,
                                                        -std::f64::consts::PI
                                                            ..=std::f64::consts::PI,
                                                    )
                                                    .show_value(false)
                                                    .trailing_fill(true),
                                                )
                                                .changed();
                                            changed |= ui
                                                .add(
                                                    egui::DragValue::new(angle)
                                                        .speed(0.01)
                                                        .range(
                                                            -std::f64::consts::PI
                                                                ..=std::f64::consts::PI,
                                                        )
                                                        .fixed_decimals(2),
                                                )
                                                .changed();
                                            ui.end_row();
                                        }
                                    },
                                );
                            });
                    });
                }
                GeoObject::RegularPolytopeND(polytope) => {
                    ui.push_id(("regular_polytope_nd", id), |ui| {
                        draw_inspector_section(
                            ui,
                            poly_geometry_title,
                            polytope_geometry_subtitle,
                            |ui| {
                                let mut family = polytope.family;
                                let combo_width = ui.available_width();
                                egui::ComboBox::from_id_salt("regular_polytope_nd_family")
                                    .width(combo_width)
                                    .selected_text(polytope_nd_family_label_for(
                                        family,
                                        inspector_locale,
                                    ))
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            &mut family,
                                            RegularPolytopeFamily::Simplex,
                                            polytope_nd_family_label_for(
                                                RegularPolytopeFamily::Simplex,
                                                inspector_locale,
                                            ),
                                        );
                                        ui.selectable_value(
                                            &mut family,
                                            RegularPolytopeFamily::Hypercube,
                                            polytope_nd_family_label_for(
                                                RegularPolytopeFamily::Hypercube,
                                                inspector_locale,
                                            ),
                                        );
                                        ui.selectable_value(
                                            &mut family,
                                            RegularPolytopeFamily::CrossPolytope,
                                            polytope_nd_family_label_for(
                                                RegularPolytopeFamily::CrossPolytope,
                                                inspector_locale,
                                            ),
                                        );
                                    });
                                if family != polytope.family {
                                    polytope.family = family;
                                    changed = true;
                                }

                                let mut dimension = polytope.dimension;
                                if ui
                                    .add(
                                        egui::Slider::new(&mut dimension, 3..=10)
                                            .text(inspector_dimension_label),
                                    )
                                    .changed()
                                {
                                    if let Some(rotation_count) =
                                        RegularPolytopeNDObj::expected_rotation_angle_count(
                                            dimension,
                                        )
                                    {
                                        polytope.dimension = dimension;
                                        polytope.rotation_angles = vec![0.0; rotation_count];
                                        changed = true;
                                    }
                                }
                                changed |= ui
                                    .add(
                                        egui::Slider::new(
                                            &mut polytope.scale,
                                            inspector_poly_scale_range(),
                                        )
                                        .text(inspector_scale_label),
                                    )
                                    .changed();
                                changed |= ui
                                    .add(
                                        egui::Slider::new(
                                            &mut polytope.width,
                                            inspector_poly_width_range(),
                                        )
                                        .text(inspector_edge_width_label),
                                    )
                                    .changed();
                            },
                        );

                        // La tarjeta de animación se muestra siempre (todas las
                        // dimensiones): el interruptor global también orbita la
                        // cámara en vista 3D; la rotación 4D automática aplica
                        // a proyecciones R4 y el resto usa rotación manual.
                        ui.add_space(SPACE_MD);
                        draw_inspector_section(
                            ui,
                            poly_projection_title,
                            polytope_projection_subtitle,
                            |ui| {
                                draw_multidimensional_motion_card(
                                    ui,
                                    app,
                                    poly_anim_title,
                                    polytope_nd_projection_body_for(
                                        polytope.dimension,
                                        inspector_locale,
                                    ),
                                    polytope.visible && app.current_view == crate::ViewMode::D3,
                                );
                            },
                        );

                        ui.add_space(SPACE_MD);
                        draw_inspector_section(
                            ui,
                            poly_appearance_title,
                            polytope_appearance_subtitle,
                            |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(inspector_edge_color_label);
                                    if color_picker_swatch(
                                        ui,
                                        polytope.color,
                                        inspector_edge_color_hover,
                                    )
                                    .clicked()
                                    {
                                        app.open_object_color_picker(id);
                                    }
                                });
                                ui.label(
                                    egui::RichText::new(polytope_fill_hint)
                                        .color(txt_dim)
                                        .size(TYPE_XS),
                                );
                            },
                        );

                        ui.add_space(SPACE_MD);
                        egui::CollapsingHeader::new(poly_rotation_manual_title)
                            .default_open(false)
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(polytope_rotation_guide)
                                        .color(txt_dim)
                                        .size(TYPE_SM),
                                );
                                ui.add_space(SPACE_SM);
                                if ui.button(poly_rotation_reset_label).clicked()
                                    && polytope.rotation_angles.iter().any(|angle| *angle != 0.0)
                                {
                                    polytope.rotation_angles.fill(0.0);
                                    changed = true;
                                }
                                ui.label(
                                    egui::RichText::new(poly_rotation_planes_title)
                                        .color(txt_dim)
                                        .size(TYPE_SM),
                                );
                                let rotation_planes: Vec<_> = (0..polytope.dimension)
                                    .flat_map(|first| {
                                        ((first + 1)..polytope.dimension)
                                            .map(move |second| (first, second))
                                    })
                                    .collect();
                                egui::ScrollArea::vertical()
                                    .id_salt("regular_polytope_nd_rotation_planes")
                                    .max_height(260.0)
                                    .show(ui, |ui| {
                                        for ((first, second), angle) in rotation_planes
                                            .into_iter()
                                            .zip(polytope.rotation_angles.iter_mut())
                                        {
                                            changed |= ui
                                                .add(
                                                    egui::Slider::new(
                                                        angle,
                                                        -std::f64::consts::PI
                                                            ..=std::f64::consts::PI,
                                                    )
                                                    .text(nd_rotation_plane_label(first, second)),
                                                )
                                                .changed();
                                        }
                                    });
                            });
                    });
                }
                GeoObject::ImplicitSurface3D(s) => {
                    // Q4: Inspector live-preview + cancel (P2 con UI).
                    // Piel pura en el dibujado (cero I/O/spawn en `Ui::` salvo
                    // los handlers de botón que delegan al slot existente):
                    // "Vista previa rápida" corre el slot en background vía
                    // `maybe_submit_implicit_slot` existente; "Cancelar" llama
                    // `cancel()` de verdad (antes sin llamador prod); el
                    // overlay `last_valid` progresivo ya vive en `render_3d.rs`
                    // (sin parpadeo ante `Failed`). Todo abre algo o explica.
                    ui.label(
                        egui::RichText::new(inspector_implicit3d_title)
                            .color(label_col)
                            .strong(),
                    );
                    ui.label(
                        egui::RichText::new(implicit_3d_expr_text(&s.expr))
                            .color(txt_col)
                            .size(TYPE_SM),
                    );
                    ui.add_space(SPACE_XS);
                    ui.label(
                        egui::RichText::new(implicit_3d_box_text_for(
                            [s.x_min, s.x_max, s.y_min, s.y_max, s.z_min, s.z_max],
                            s.cells,
                            inspector_locale,
                        ))
                        .color(txt_dim)
                        .size(TYPE_XS),
                    );
                    let has_pending = app.implicit_surface_slot.has_pending();
                    let pending_cells = app.implicit_surface_slot.pending_cells();
                    let tri_count = app
                        .implicit_surface_slot
                        .last_valid()
                        .map(|mesh| mesh.triangle_count());
                    let status = crate::implicit_surface_compute::implicit_preview_status(
                        has_pending,
                        tri_count.is_some(),
                        pending_cells,
                        tri_count,
                    );
                    ui.add_space(SPACE_XS);
                    ui.label(
                        egui::RichText::new(implicit_3d_state_text_for(status, inspector_locale))
                            .color(txt_dim)
                            .size(TYPE_XS),
                    );
                    if let Some(tris) = tri_count {
                        ui.label(
                            egui::RichText::new(implicit_3d_preview_text_for(
                                tris,
                                inspector_locale,
                            ))
                            .color(txt_dim)
                            .size(TYPE_XS),
                        );
                    }
                    // B-A113: chrome de vista previa implícita en el idioma
                    // activo (antes consts ES directas). El botón Cancelar
                    // reutiliza `common.cancel` (precedente B-A66/B-A95).
                    let implicit_quick_guide_text =
                        inspector_chrome_for("implicit_quick_guide", inspector_locale);
                    let implicit_quick_label_text =
                        inspector_chrome_for("implicit_quick_label", inspector_locale);
                    let implicit_quick_hover_text =
                        inspector_chrome_for("implicit_quick_hover", inspector_locale);
                    let implicit_cancel_label_text = implicit_cancel_label_for(inspector_locale);
                    let implicit_cancel_empty_text =
                        inspector_chrome_for("implicit_cancel_empty", inspector_locale);
                    let implicit_cancel_hover_text =
                        inspector_chrome_for("implicit_cancel_hover", inspector_locale);
                    if crate::implicit_surface_compute::should_offer_quick_preview(s.cells) {
                        ui.label(
                            egui::RichText::new(implicit_quick_guide_text)
                                .color(txt_dim)
                                .size(TYPE_XS),
                        );
                    }
                    ui.add_space(SPACE_XS);
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(SPACE_SM, SPACE_XS);
                        // "Vista previa rápida": usa el productor existente.
                        // Si ya hay pendiente o válida con misma clave, el
                        // productor no re-envía (honesto: se avisa, nada mudo).
                        if ui
                            .small_button(implicit_quick_label_text)
                            .on_hover_text(implicit_quick_hover_text)
                            .clicked()
                        {
                            let was_pending = app.implicit_surface_slot.has_pending();
                            let had_valid = app.implicit_surface_slot.last_valid().is_some();
                            app.maybe_submit_implicit_slot();
                            // Ola 2: one-shot vía presupuesto (ZERO = inmediato).
                            app.request_repaint_budget(Duration::ZERO);
                            let message = implicit_quick_preview_notify(
                                was_pending,
                                had_valid,
                                app.implicit_surface_slot.has_pending(),
                            );
                            // La tabla pura decide el texto; el `ToastKind`
                            // sigue la rama del dibujo (solo el fallo es Error):
                            // el `kind` se decide sobre el canónico y el
                            // display sobre el localizado (B-A113).
                            let kind = if message == IMPLICIT_PREVIEW_START_FAILED {
                                grafito_ui::toast::ToastKind::Error
                            } else {
                                grafito_ui::toast::ToastKind::Info
                            };
                            app.notify(
                                implicit_quick_preview_notify_for(
                                    was_pending,
                                    had_valid,
                                    app.implicit_surface_slot.has_pending(),
                                    inspector_locale,
                                ),
                                kind,
                            );
                        }
                        // "Cancelar": llama `cancel()` de verdad (antes sin
                        // llamador prod). Suelta el job en vuelo sin tocar el
                        // último válido; deshabilitado con motivo si no hay nada.
                        let cancel_resp = ui.add_enabled(
                            has_pending,
                            egui::Button::new(implicit_cancel_label_text).small(),
                        );
                        if cancel_resp.clicked() {
                            app.implicit_surface_slot.cancel();
                            // Ola 2: one-shot vía presupuesto (ZERO = inmediato).
                            app.request_repaint_budget(Duration::ZERO);
                            app.notify(
                                inspector_chrome_for("implicit_cancelled", inspector_locale),
                                grafito_ui::toast::ToastKind::Info,
                            );
                        }
                        if !has_pending {
                            cancel_resp.on_disabled_hover_text(implicit_cancel_empty_text);
                        } else {
                            cancel_resp.on_hover_text(implicit_cancel_hover_text);
                        }
                    });
                    ui.add_space(SPACE_XS);
                }
                _ => {
                    // Display-only: la ecuación ya va en grande en la card de
                    // identidad. Sin nombre muerto ni texto que no sirve.
                    // D3 agrega el campo de edición; este dock no edita.
                    ui.add_space(SPACE_XS);
                    ui.separator();
                    ui.add_space(SPACE_SM);
                    ui.label(
                        egui::RichText::new(inspector_dock_empty_title)
                            .color(theme.text_secondary)
                            .size(TYPE_SM),
                    );
                    ui.label(
                        egui::RichText::new(inspector_dock_empty_guide)
                            .color(txt_dim)
                            .size(TYPE_XS),
                    );
                    ui.add_space(SPACE_MD);
                }
            }
            match apply_object_panel_edit_with_previous(
                &mut app.document,
                id,
                changed,
                move |object| {
                    *object = edited_object;
                },
            ) {
                Ok(Some(before)) => snapshot.capture_successful_replacement(before),
                Ok(None) => {}
                Err(error) => {
                    // B-A145: mensaje en el idioma activo (antes ES duro).
                    let message = crate::algebra::ctx_edit_error_text(&error, inspector_locale);
                    ui.label(
                        egui::RichText::new(&message)
                            .color(current_theme(ui.ctx()).danger)
                            .size(TYPE_XS),
                    );
                    app.cas_result = message.clone();
                    app.notify(message, grafito_ui::toast::ToastKind::Error);
                }
            }
            // Q2: la capa vive en el documento (no en el objeto).
            // Mismo pipeline validado que el reemplazo de objeto:
            // staging + `capture_successful_replacement`, un solo
            // snapshot por frame (ver test `idle_object_panel_edit…`).
            if layer_value != layer_before {
                match app.document.try_set_layer_with_previous(id, layer_value) {
                    Ok(Some(before)) => {
                        snapshot.capture_successful_replacement(before);
                    }
                    Ok(None) => {}
                    Err(error) => {
                        let message = format!("Capa: {error}");
                        ui.label(
                            egui::RichText::new(&message)
                                .color(current_theme(ui.ctx()).danger)
                                .size(TYPE_XS),
                        );
                        app.cas_result = message.clone();
                        app.notify(message, grafito_ui::toast::ToastKind::Error);
                    }
                }
            }
        });
    let _ = snapshot.save_if_semantically_changed(
        &mut app.document,
        &mut app.undo_stack,
        &mut app.redo_stack,
    );
}

// (movidos a `sheet_layout.rs`: `set_domain_coloring_mode` +
// `set_complex_mapping_animation`, slice 20 F14.4.4)

/// Panel derecho: Coloración de dominio (Complejos).
pub(crate) fn draw_right_domain_coloring_panel(app: &mut GrafitoApp, ctx: &egui::Context) {
    use grafito_core::GeoObject;
    let (_is_dark, accent, alg_fill, _sep_col, _txt_col, txt_dim, hdr_col) = panel_theme_local(ctx);

    egui::SidePanel::right("right_domain_coloring")
        .show_separator_line(false)
        .default_width(280.0)
        .min_width(200.0)
        .resizable(true)
        .frame(
            egui::Frame::none()
                .fill(alg_fill)
                .stroke(egui::Stroke::NONE),
        )
        .show(ctx, |ui| {
            ui.add_space(SPACE_SM);
            draw_right_drawer_header(
                ui,
                app,
                DOMAIN_COLORING_TITLE,
                DOMAIN_COLORING_SUBTITLE,
                accent,
            );

            let mut has_domain_coloring = false;
            let mut grid_id = None;
            for (id, obj) in app.document.objects_iter() {
                if matches!(obj, GeoObject::ComplexGrid(grid) if grid.render_mode == 1) {
                    has_domain_coloring = true;
                    grid_id = Some(*id);
                    break;
                }
            }

            if !has_domain_coloring {
                ui.label(
                    egui::RichText::new(DOMAIN_COLORING_EMPTY_GUIDE)
                        .color(txt_dim)
                        .size(TYPE_XS),
                );
            } else {
                ui.label(
                    egui::RichText::new(DOMAIN_COLORING_ACTIVE_TITLE)
                        .color(hdr_col)
                        .strong(),
                );
                ui.add_space(SPACE_XS);
                ui.label(
                    egui::RichText::new(DOMAIN_COLORING_HUE_LINE)
                        .color(txt_dim)
                        .size(TYPE_XS),
                );

                ui.add_space(SPACE_XS);
                ui.collapsing(DOMAIN_COLORING_GUIDE_TITLE, |ui| {
                    let theme = current_theme(ui.ctx());
                    egui::Frame::none()
                        .fill(theme.input_bg)
                        .rounding(egui::Rounding::same(6.0))
                        .inner_margin(egui::Margin::same(grafito_ui::tokens::SPACE_SM))
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new(DOMAIN_COLORING_COLORS_TITLE)
                                        .strong()
                                        .color(hdr_col)
                                        .size(TYPE_XS),
                                );
                                ui.label(
                                    egui::RichText::new(DOMAIN_COLORING_COLORS_BODY)
                                        .color(txt_dim)
                                        .size(grafito_ui::tokens::TYPE_XS),
                                );

                                ui.add_space(SPACE_XS);
                                ui.label(
                                    egui::RichText::new(DOMAIN_COLORING_DERIV_TITLE)
                                        .strong()
                                        .color(hdr_col)
                                        .size(TYPE_XS),
                                );
                                ui.label(
                                    egui::RichText::new(DOMAIN_COLORING_DERIV_BODY)
                                        .color(txt_dim)
                                        .size(grafito_ui::tokens::TYPE_XS),
                                );
                            });
                        });
                });
            }

            // Selector de modo de coloreado de dominio
            if let Some(id) = grid_id {
                if let Some(GeoObject::ComplexGrid(cg)) = app.document.get_object(id) {
                    ui.add_space(SPACE_SM);
                    ui.label(
                        egui::RichText::new(DOMAIN_COLORING_MODE_LABEL)
                            .color(hdr_col)
                            .size(TYPE_SM)
                            .strong(),
                    );
                    let mut mode = cg.domain_coloring_mode;
                    egui::ComboBox::from_id_salt("dc_mode_combo")
                        .selected_text(domain_coloring_mode_label(mode))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut mode, 0, domain_coloring_mode_label(0));
                            ui.selectable_value(&mut mode, 1, domain_coloring_mode_label(1));
                            ui.selectable_value(&mut mode, 2, domain_coloring_mode_label(2));
                            ui.selectable_value(&mut mode, 3, domain_coloring_mode_label(3));
                        });

                    let _ = set_domain_coloring_mode(&mut app.document, id, mode);
                }
            }

            ui.add_space(SPACE_SM);
            ui.label(
                egui::RichText::new(DOMAIN_COLORING_SYMBOL_LABEL)
                    .color(hdr_col)
                    .size(TYPE_SM)
                    .strong(),
            );
            let mut sym = app.document.complex_base_symbol.clone();
            let r = ui.add(
                egui::TextEdit::singleline(&mut sym)
                    .desired_width(ui.available_width())
                    .hint_text(DOMAIN_COLORING_SYMBOL_HINT),
            );
            if r.lost_focus() && sym.trim() != app.document.complex_base_symbol {
                let new_sym = sym.trim().to_string();
                if !new_sym.is_empty() {
                    app.document.migrate_complex_symbol(&new_sym);
                    app.document.bump_version();
                }
            }

            // Animación de homotopía si hay algún mapeo complejo
            let mut mapping_id = None;
            for (id, obj) in app.document.objects_iter() {
                if matches!(obj, GeoObject::ComplexMapping(_)) {
                    mapping_id = Some(*id);
                    break;
                }
            }

            if let Some(id) = mapping_id {
                // R3.4: títulos en el idioma activo (antes ES fijo); el catálogo
                // ya cubre PT al 100% así el panel deja de tener fallback.
                let locale = app.config_locale();
                if let Some(GeoObject::ComplexMapping(cm)) = app.document.get_object(id) {
                    // Títulos del catálogo i18n (`panel.conformal.*`) en el idioma
                    // activo (R3.4: PT total, sin fallback).
                    use grafito_ui::i18n::t;
                    ui.add_space(SPACE_SM);
                    ui.separator();
                    ui.add_space(SPACE_SM);
                    ui.label(
                        egui::RichText::new(t("panel.conformal.title", locale))
                            .color(accent)
                            .strong(),
                    );
                    ui.add_space(SPACE_XS);

                    let mut anim = cm.animate_homotopy;
                    grafito_ui::assistant::scandi_checkbox(
                        ui,
                        t("panel.conformal.animate", locale),
                        &mut anim,
                    );

                    let mut speed = cm.homotopy_speed;
                    ui.add(
                        egui::Slider::new(&mut speed, 0.2..=3.0)
                            .text(t("panel.conformal.speed", locale)),
                    );
                    let _ = set_complex_mapping_animation(&mut app.document, id, anim, speed);
                }
            }
        });
}

/// Panel derecho: Parámetros del attractor activo (Dynamics).
pub(crate) fn draw_right_parameters_panel(app: &mut GrafitoApp, ctx: &egui::Context) {
    use grafito_core::{GeoObject, ObjectId};
    let mut snapshot = crate::app::DeferredPanelSnapshot::new(app.undo_stack.len());
    let (_is_dark, accent, alg_fill, _sep_col, _txt_col, txt_dim, hdr_col) = panel_theme_local(ctx);
    // B-A189: chrome Parámetros por dispatcher (Es→consts canónicas,
    // resto→`dynamics_msg`); sliders `σ/ρ/β/dt` con notación intactos.
    // B-A190: error del commit por `params_error_text_for` con
    // `params_locale` (Es delega al canónico `params_error_text`).
    let params_locale = app.config_locale();
    let params_panel_title = dynamics_chrome_for("params_panel_title", params_locale);
    let params_panel_subtitle = dynamics_chrome_for("params_panel_subtitle", params_locale);
    let params_empty_guide = dynamics_chrome_for("params_empty_guide", params_locale);
    let params_lorenz_section = dynamics_chrome_for("params_lorenz_section", params_locale);
    let params_integration_section =
        dynamics_chrome_for("params_integration_section", params_locale);
    let params_regen_hint = dynamics_chrome_for("params_regen_hint", params_locale);
    let params_steps_label = dynamics_chrome_for("params_steps_label", params_locale);

    egui::SidePanel::right("right_parameters")
        .show_separator_line(false)
        .default_width(260.0)
        .min_width(180.0)
        .resizable(true)
        .frame(
            egui::Frame::none()
                .fill(alg_fill)
                .stroke(egui::Stroke::NONE),
        )
        .show(ctx, |ui| {
            ui.add_space(SPACE_SM);
            draw_right_drawer_header(ui, app, params_panel_title, params_panel_subtitle, accent);

            let mut attractor_id: Option<ObjectId> = None;
            for (id, obj) in app.document.objects_iter() {
                if matches!(obj, GeoObject::Attractor3D(_)) {
                    attractor_id = Some(*id);
                    break;
                }
            }

            let Some(id) = attractor_id else {
                ui.label(
                    egui::RichText::new(params_empty_guide)
                        .color(txt_dim)
                        .size(TYPE_XS),
                );
                return;
            };

            let Some(GeoObject::Attractor3D(attractor)) = app.document.get_object(id).cloned()
            else {
                return;
            };
            let mut sigma = attractor.params.first().copied().unwrap_or(0.0);
            let mut rho = attractor.params.get(1).copied().unwrap_or(0.0);
            let mut beta = attractor.params.get(2).copied().unwrap_or(0.0);
            let mut dt = attractor.dt;
            let mut steps = attractor.steps;
            let mut changed = false;

            ui.label(
                egui::RichText::new(params_lorenz_section)
                    .color(hdr_col)
                    .size(TYPE_SM)
                    .strong(),
            );
            ui.add_space(SPACE_XS);
            changed |= ui
                .add(
                    egui::Slider::new(&mut sigma, 0.1..=30.0)
                        .text("σ")
                        .trailing_fill(true),
                )
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(&mut rho, 0.1..=60.0)
                        .text("ρ")
                        .trailing_fill(true),
                )
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(&mut beta, 0.1..=10.0)
                        .text("β")
                        .trailing_fill(true),
                )
                .changed();
            ui.add_space(SPACE_XS);
            ui.label(
                egui::RichText::new(params_integration_section)
                    .color(hdr_col)
                    .size(TYPE_SM)
                    .strong(),
            );
            changed |= ui
                .add(
                    egui::Slider::new(&mut dt, 0.001..=0.05)
                        .text("dt")
                        .trailing_fill(true),
                )
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(&mut steps, 100..=20000)
                        .text(params_steps_label)
                        .trailing_fill(true)
                        .integer(),
                )
                .changed();
            ui.add_space(SPACE_XS);
            ui.label(
                egui::RichText::new(params_regen_hint)
                    .color(txt_dim)
                    .size(TYPE_XS),
            );

            match apply_object_panel_edit_with_previous(
                &mut app.document,
                id,
                changed,
                move |object| {
                    let GeoObject::Attractor3D(attractor) = object else {
                        return;
                    };
                    attractor.params.resize(3, 0.0);
                    attractor.params[0] = sigma;
                    attractor.params[1] = rho;
                    attractor.params[2] = beta;
                    attractor.dt = dt;
                    attractor.steps = steps;
                },
            ) {
                Ok(Some(before)) => snapshot.capture_successful_replacement(before),
                Ok(None) => {}
                Err(error) => {
                    let message = params_error_text_for(error, params_locale);
                    ui.label(
                        egui::RichText::new(&message)
                            .color(current_theme(ui.ctx()).danger)
                            .size(TYPE_XS),
                    );
                    app.cas_result = message.clone();
                    app.notify(message, grafito_ui::toast::ToastKind::Error);
                }
            }
        });
    let _ = snapshot.save_if_semantically_changed(
        &mut app.document,
        &mut app.undo_stack,
        &mut app.redo_stack,
    );
}

pub(crate) fn draw_right_regression_panel(app: &mut GrafitoApp, ctx: &egui::Context) {
    let theme = current_theme(ctx);

    egui::SidePanel::right("regression").show_separator_line(false)
        .resizable(true)
        .default_width(280.0)
        .min_width(200.0)
        .frame(
            egui::Frame::none()
                .fill(theme.panel_bg)
                .stroke(egui::Stroke::NONE),
        )
        .show(ctx, |ui| {
            let regression_locale = app.config_locale();
            let regression_panel_title = crate::sheet_layout::regression_chrome_for(
                "regression_panel_title",
                regression_locale,
            );
            let regression_panel_subtitle = crate::sheet_layout::regression_chrome_for(
                "regression_panel_subtitle",
                regression_locale,
            );
            let regression_fits_title = crate::sheet_layout::regression_chrome_for(
                "regression_fits_title",
                regression_locale,
            );
            let regression_fits_empty = crate::sheet_layout::regression_chrome_for(
                "regression_fits_empty",
                regression_locale,
            );
            let regression_diagnostics_title = crate::sheet_layout::regression_chrome_for(
                "regression_diagnostics_title",
                regression_locale,
            );
            let regression_diagnostics_subtitle =
                crate::sheet_layout::regression_chrome_for(
                    "regression_diagnostics_subtitle",
                    regression_locale,
                );
            let regression_create_title = crate::sheet_layout::regression_chrome_for(
                "regression_create_title",
                regression_locale,
            );
            let regression_create_subtitle = crate::sheet_layout::regression_chrome_for(
                "regression_create_subtitle",
                regression_locale,
            );
            let regression_import_label = crate::sheet_layout::regression_chrome_for(
                "regression_import_label",
                regression_locale,
            );
            let regression_import_hover = crate::sheet_layout::regression_chrome_for(
                "regression_import_hover",
                regression_locale,
            );
            let regression_model_hint = crate::sheet_layout::regression_chrome_for(
                "regression_model_hint",
                regression_locale,
            );
            let regression_no_table_guide = crate::sheet_layout::regression_chrome_for(
                "regression_no_table_guide",
                regression_locale,
            );
            draw_right_drawer_header(ui, app, regression_panel_title, regression_panel_subtitle, theme.accent);
            ui.separator();
            panel_scroll_content(ui, "regression_panel_content", None, |ui| {
                    egui::Frame::none()
                        .inner_margin(egui::Margin {
                            left: SPACE_MD,
                            right: SPACE_XS,
                            top: SPACE_SM,
                            bottom: SPACE_LG,
                        })
                        .show(ui, |ui| {
                            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                            ui.spacing_mut().item_spacing.y = SPACE_SM;

                            // 1 · Ajustes vivos del documento (tarjetas con objeto).
                            draw_object_cards_where(
                                ui,
                                app,
                                regression_fits_title,
                                regression_fits_empty,
                                |object| {
                                    matches!(
                                        object,
                                        GeoObject::DataTable(_)
                                            | GeoObject::ScatterPlot(_)
                                            | GeoObject::RegressionLine(_)
                                    ) || matches!(object, GeoObject::Function(function) if function.fit.is_some())
                                },
                            );
                            ui.add_space(SPACE_SM);

                            // 2 · Diagnósticos por ajuste (RMSE/R² + residuales).
                            let fits: Vec<(String, _)> = app
                                .document
                                .objects_iter()
                                .filter_map(|(_, object)| match object {
                                    GeoObject::Function(function) => function
                                        .fit
                                        .as_ref()
                                        .map(|fit| (function.label.clone(), fit.clone())),
                                    _ => None,
                                })
                                .collect();
                            if !fits.is_empty() {
                                draw_inspector_section(
                                    ui,
                                    regression_diagnostics_title,
                                    regression_diagnostics_subtitle,
                                    |ui| {
                                        for (label, fit) in &fits {
                                            let source_label = regression_source_label_for(
                                                app.document
                                                    .get_object(fit.source)
                                                    .map(|object| object.label()),
                                                regression_locale,
                                            );
                                            ui.label(
                                                egui::RichText::new(regression_fit_headline(
                                                    label,
                                                    fit_kind_label_for(
                                                        fit.kind,
                                                        app.config_locale(),
                                                    ),
                                                ))
                                                .color(theme.text_primary)
                                                .size(TYPE_SM)
                                                .strong(),
                                            );
                                            ui.label(
                                                egui::RichText::new(
                                                    regression_fit_diagnostics_line_for(
                                                        &source_label,
                                                        fit.diagnostics.rmse,
                                                        fit.diagnostics.r_squared,
                                                        regression_locale,
                                                    ),
                                                )
                                                .color(theme.text_tertiary)
                                                .size(TYPE_XS)
                                                .monospace(),
                                            );
                                            ui.collapsing(
                                                regression_residuals_title_for(
                                                    fit.diagnostics.residuals.len(),
                                                    regression_locale,
                                                ),
                                                |ui| {
                                                    let shown = fit
                                                        .diagnostics
                                                        .residuals
                                                        .len()
                                                        .min(REGRESSION_MAX_RESIDUALS_SHOWN);
                                                    for (index, residual) in fit
                                                        .diagnostics
                                                        .residuals
                                                        .iter()
                                                        .take(shown)
                                                        .enumerate()
                                                    {
                                                        ui.label(
                                                            egui::RichText::new(
                                                                regression_residual_line(
                                                                    index, *residual,
                                                                ),
                                                            )
                                                            .color(theme.text_tertiary)
                                                            .size(TYPE_XS)
                                                            .monospace(),
                                                        );
                                                    }
                                                    if fit.diagnostics.residuals.len() > shown {
                                                        ui.label(
                                                            egui::RichText::new(
                                                                regression_residuals_truncated_for(
                                                                    regression_locale,
                                                                ),
                                                            )
                                                            .color(theme.text_tertiary)
                                                            .size(TYPE_XS),
                                                        );
                                                    }
                                                },
                                            );
                                        }
                                    },
                                );
                                ui.add_space(SPACE_SM);
                            }

                            // 3 · Crear análisis: importar + plantillas al portapapeles.
                            draw_inspector_section(
                                ui,
                                regression_create_title,
                                regression_create_subtitle,
                                |ui| {
                                    if panel_wide_action_button(ui, regression_import_label)
                                        .on_hover_text(regression_import_hover)
                                        .clicked()
                                    {
                                        import_local_xy_table(app, ctx);
                                    }
                                    // B-A148: nombres display localizados +
                                    // comandos invariantes (`regression_templates_for`).
                                    let regression_templates =
                                        regression_templates_for(regression_locale);
                                    let items: Vec<(&str, &str)> = regression_templates
                                        .iter()
                                        .map(|(l, t)| (*l, *t))
                                        .collect();
                                    if let Some(index) =
                                        panel_button_grid(ui, "gc_regression_templates", 2, &items)
                                    {
                                        app.input_text =
                                            regression_templates[index].1.to_string();
                                        app.command_input_focus_requested = true;
                                        app.cas_result = regression_template_ready_text_for(
                                            regression_templates[index].0,
                                            regression_locale,
                                        );
                                    }
                                },
                            );
                            ui.add_space(SPACE_SM);

                            // 4 · Modelos de ajuste para la tabla seleccionada.
                            let selected_table_label = app.selected_object.and_then(|id| {
                                match app.document.get_object(id) {
                                    Some(GeoObject::DataTable(table)) => Some(table.label.clone()),
                                    _ => None,
                                }
                            });
                            if let Some(table_label) = selected_table_label {
                                draw_inspector_section(
                                    ui,
                                    &regression_section_title_for(&table_label, regression_locale),
                                    regression_model_hint,
                                    |ui| {
                                        // B-A149: etiquetas display localizadas +
                                        // comandos invariantes
                                        // (`regression_model_commands_for`).
                                        let models = regression_model_commands_for(
                                            &table_label,
                                            regression_locale,
                                        );
                                        let items: Vec<(&str, &str)> = models
                                            .iter()
                                            .map(|(label, command)| (label.as_str(), command.as_str()))
                                            .collect();
                                        if let Some(index) = panel_button_grid(
                                            ui,
                                            "gc_regression_models",
                                            2,
                                            &items,
                                        ) {
                                            app.input_text = models[index].1.clone();
                                            app.command_input_focus_requested = true;
                                            app.cas_result =
                                                regression_model_ready_text_for(
                                                    &models[index].0,
                                                    regression_locale,
                                                );
                                        }
                                    },
                                );
                            } else {
                                egui::Frame::none()
                                    .fill(theme.input_bg)
                                    .stroke(theme.hairline_stroke())
                                    .rounding(egui::Rounding::same(RADIUS_SM))
                                    .inner_margin(egui::Margin::same(SPACE_MD))
                                    .show(ui, |ui| {
                                        ui.label(
                                            egui::RichText::new(
                                                regression_no_table_guide,
                                            )
                                            .color(theme.text_tertiary)
                                            .size(TYPE_XS),
                                        );
                                    });
                            }
                        });
                });
        });
}

// ─────────────────────────────────────────────────────────────────────────
// Protocolo de Construcción (panel derecho, perspectiva Geometry2D)
// ─────────────────────────────────────────────────────────────────────────

// (movidos a `sheet_layout.rs`: `escape_latex` + `construction_log_to_latex`,
// slice 18 F14.4.4)

// Navegación view-only del protocolo de construcción (Ola 1.5): paso actual
// y reproducción. No reordena ni desactiva nada: solo selecciona el objeto
// del paso en el lienzo y resalta la tarjeta. Estado y avance puros en
// `sheet_layout.rs` (slice 18 F14.4.4); vive en `ctx.data`.
// (movidos a `sheet_layout.rs`: `ProtocolNavState` + `PROTOCOL_PLAY_INTERVAL_SECS` +
// `protocol_advance`, slice 18 F14.4.4)

/// Selecciona en el documento el objeto de salida del paso `index`.
/// Devuelve `false` si el paso no existe o su salida no es un objeto
/// etiquetado (p. ej. pasos de comandos sin geometría).
fn select_protocol_step(app: &mut GrafitoApp, index: usize) -> bool {
    let Some(step) = app.construction_log.get(index) else {
        return false;
    };
    let label = step.output.clone();
    if label.is_empty() || label.starts_with("__") {
        return false;
    }
    let target = app
        .document
        .objects_iter()
        .find(|(_, object)| !object.label().is_empty() && object.label() == label)
        .map(|(id, _)| *id);
    match target {
        Some(id) => {
            app.document.clear_selection();
            app.document.select(id);
            app.selected_object = Some(id);
            true
        }
        None => false,
    }
}

pub(crate) fn draw_construction_protocol(app: &mut GrafitoApp, ctx: &egui::Context) {
    if !app.show_construction_protocol {
        return;
    }
    let theme = current_theme(ctx);
    let (_is_dark, accent, alg_fill, _sep_col, txt_col, txt_dim, _hdr_col) = panel_theme_local(ctx);

    egui::SidePanel::right("construction_protocol")
        .show_separator_line(false)
        .resizable(true)
        .default_width(300.0)
        .min_width(200.0)
        .frame(
            egui::Frame::none()
                .fill(alg_fill)
                .stroke(egui::Stroke::NONE),
        )
        .show(ctx, |ui| {
            ui.add_space(SPACE_SM);
            ui.horizontal(|ui| {
                ui.add_space(SPACE_SM);
                draw_right_drawer_header(
                    ui,
                    app,
                    PROTOCOL_PANEL_TITLE,
                    PROTOCOL_PANEL_SUBTITLE,
                    accent,
                );
            });
            ui.add_space(2.0);
            ui.separator();

            // Cuerpo en tarjetas escandinavas: acciones arriba, pasos en
            // tarjetas con número; nada de botones sueltos ni texto flotando.
            panel_scroll_content(ui, "construction_protocol_scroll", None, |ui| {
                egui::Frame::none()
                    .inner_margin(egui::Margin {
                        left: SPACE_MD,
                        right: SPACE_XS,
                        top: SPACE_SM,
                        bottom: SPACE_LG,
                    })
                    .show(ui, |ui| {
                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                        ui.spacing_mut().item_spacing.y = SPACE_SM;

                        // Acciones: mismas celdas, mismo alto y radio que
                        // el resto de la app; deshabilitadas sin pasos.
                        let has_steps = !app.construction_log.is_empty();
                        let button_w = ((ui.available_width() - SPACE_SM) / 2.0).max(0.0);
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = SPACE_SM;
                            let export = ui.add_enabled(
                                has_steps,
                                egui::Button::new(
                                    egui::RichText::new(PROTOCOL_EXPORT_LABEL).size(TYPE_XS),
                                )
                                .min_size(egui::vec2(button_w, PANEL_BUTTON_H))
                                .rounding(RADIUS_SM),
                            );
                            if export
                                .on_hover_text(protocol_export_hover(has_steps))
                                .clicked()
                            {
                                // FileController: el diálogo rfd queda en UI thread
                                // (modal nativo); el write va a worker y se notifica
                                // en el poll.
                                let latex = construction_log_to_latex(&app.construction_log);
                                // S2: sin `desktop-dialogs` devuelve `None` con toast honesto.
                                // Sin nombre pre-cargado (igual que el rfd directo).
                                if let Some(path) = app.save_file_native("TeX", &["tex"], None) {
                                    app.pending_text_job = Some(crate::app::PendingTextWriteJob {
                                        receiver: crate::app::spawn_text_write(path, latex, ctx),
                                    });
                                    app.cas_result = PROTOCOL_EXPORTING.to_string();
                                }
                            }
                            let clear = ui.add_enabled(
                                has_steps,
                                egui::Button::new(
                                    egui::RichText::new(PROTOCOL_CLEAR_LABEL).size(TYPE_XS),
                                )
                                .min_size(egui::vec2(button_w, PANEL_BUTTON_H))
                                .rounding(RADIUS_SM),
                            );
                            if clear.on_hover_text(PROTOCOL_CLEAR_HOVER).clicked() {
                                app.construction_log.clear();
                            }
                        });

                        // Navegación view-only (Ola 1.5): ◀ ▶ y play
                        // seleccionan el objeto del paso sin tocar el
                        // documento (el reorden real sigue sin existir).
                        let nav_id = egui::Id::new("gc_protocol_nav_state");
                        let mut nav: ProtocolNavState = ctx
                            .data_mut(|data| data.get_temp::<ProtocolNavState>(nav_id))
                            .unwrap_or_default();
                        let nav_time = ctx.input(|i| i.time);
                        if nav.playing {
                            ctx.request_repaint_after(std::time::Duration::from_millis(
                                (PROTOCOL_PLAY_INTERVAL_SECS * 1_000.0) as u64,
                            ));
                        }
                        if has_steps {
                            let before = nav.cursor;
                            protocol_advance(&mut nav, app.construction_log.len(), nav_time);
                            if nav.cursor != before {
                                if let Some(index) = nav.cursor {
                                    let _ = select_protocol_step(app, index);
                                }
                            }
                            let total = app.construction_log.len();
                            let current = nav.cursor.map_or(0, |c| c + 1);
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = SPACE_XS;
                                ui.spacing_mut().interact_size.y = SHEET_CONTROL_H;
                                let nav_button = |ui: &mut egui::Ui,
                                                  glyph: &str,
                                                  tip: &str|
                                 -> bool {
                                    ui.add_sized(
                                        [32.0, SHEET_CONTROL_H],
                                        egui::Button::new(egui::RichText::new(glyph).size(TYPE_XS))
                                            .wrap_mode(egui::TextWrapMode::Extend)
                                            .rounding(RADIUS_SM),
                                    )
                                    .on_hover_text(tip)
                                    .clicked()
                                };
                                if nav_button(ui, PROTOCOL_PREV_GLYPH, PROTOCOL_PREV_TIP) {
                                    nav.cursor =
                                        Some(nav.cursor.map_or(total.saturating_sub(1), |c| {
                                            c.saturating_sub(1)
                                        }));
                                    nav.playing = false;
                                    if let Some(index) = nav.cursor {
                                        let _ = select_protocol_step(app, index);
                                    }
                                }
                                let playing = nav.playing;
                                if nav_button(ui, protocol_play_glyph(playing), PROTOCOL_PLAY_TIP) {
                                    nav.playing = !playing;
                                    if nav.playing {
                                        nav.last_advance = nav_time;
                                        if nav.cursor.is_none() {
                                            nav.cursor = Some(0);
                                            let _ = select_protocol_step(app, 0);
                                        }
                                    }
                                }
                                if nav_button(ui, PROTOCOL_LAST_GLYPH, PROTOCOL_LAST_TIP) {
                                    nav.cursor = Some(total.saturating_sub(1));
                                    nav.playing = false;
                                    let _ = select_protocol_step(app, total.saturating_sub(1));
                                }
                                let _ = ui.label(
                                    egui::RichText::new(protocol_counter_text(current, total))
                                        .color(txt_dim)
                                        .size(TYPE_XS)
                                        .monospace(),
                                );
                            });
                        }
                        // Persistencia única por frame (después de fila y
                        // tarjetas: el click de tarjeta también cuenta).
                        ctx.data_mut(|data| data.insert_temp(nav_id, nav));

                        // El protocolo es una vista fiel del historial.
                        // Reordenar o desactivar sólo su texto no modifica
                        // restricciones reales: esos controles no existen.
                        if !has_steps {
                            egui::Frame::none()
                                .fill(theme.input_bg)
                                .stroke(theme.hairline_stroke())
                                .rounding(egui::Rounding::same(RADIUS_SM))
                                .inner_margin(egui::Margin::same(SPACE_MD))
                                .show(ui, |ui| {
                                    ui.set_min_width(ui.available_width());
                                    ui.label(
                                        egui::RichText::new(PROTOCOL_EMPTY_TITLE)
                                            .color(theme.text_primary)
                                            .size(TYPE_SM)
                                            .strong(),
                                    );
                                    ui.add_space(SPACE_XS);
                                    ui.label(
                                        egui::RichText::new(PROTOCOL_EMPTY_GUIDE)
                                            .color(theme.text_tertiary)
                                            .size(TYPE_XS),
                                    );
                                });
                        } else {
                            let total = app.construction_log.len();
                            draw_inspector_section(
                                ui,
                                &protocol_steps_title(total),
                                PROTOCOL_STEPS_HINT,
                                |ui| {
                                    for i in 0..total {
                                        let (n, action, inputs, output, disabled) = {
                                            let step = &app.construction_log[i];
                                            (
                                                step.n,
                                                step.action.clone(),
                                                step.inputs.clone(),
                                                step.output.clone(),
                                                step.disabled,
                                            )
                                        };
                                        let io_line = protocol_step_io_text(&inputs, &output);
                                        let is_current = nav.cursor == Some(i);
                                        let card = egui::Frame::none()
                                            .fill(if is_current {
                                                theme.accent_muted
                                            } else {
                                                theme.input_bg
                                            })
                                            .stroke(if is_current {
                                                egui::Stroke::new(1.0, accent)
                                            } else {
                                                theme.hairline_stroke()
                                            })
                                            .rounding(egui::Rounding::same(RADIUS_SM))
                                            .inner_margin(egui::Margin::symmetric(
                                                SPACE_SM,
                                                SPACE_XS + 2.0,
                                            ))
                                            .show(ui, |ui| {
                                                ui.set_min_width(ui.available_width());
                                                ui.horizontal(|ui| {
                                                    ui.label(
                                                        egui::RichText::new(protocol_step_badge(n))
                                                            .color(accent)
                                                            .size(TYPE_XS)
                                                            .strong()
                                                            .monospace(),
                                                    );
                                                    ui.add_space(SPACE_XS);
                                                    ui.vertical(|ui| {
                                                        ui.horizontal(|ui| {
                                                            ui.label(
                                                                egui::RichText::new(&action)
                                                                    .color(txt_col)
                                                                    .strong()
                                                                    .size(TYPE_SM),
                                                            );
                                                            if disabled {
                                                                ui.label(
                                                                    egui::RichText::new(
                                                                        PROTOCOL_DISABLED_LABEL,
                                                                    )
                                                                    .color(txt_dim)
                                                                    .size(TYPE_XS)
                                                                    .italics(),
                                                                );
                                                            }
                                                        });
                                                        // Truncado: inputs de usuario sin tope.
                                                        ui.add(
                                                            egui::Label::new(
                                                                egui::RichText::new(
                                                                    io_line.clone(),
                                                                )
                                                                .size(TYPE_XS)
                                                                .color(txt_dim)
                                                                .monospace(),
                                                            )
                                                            .truncate(),
                                                        )
                                                        .on_hover_text(io_line);
                                                    });
                                                });
                                            });
                                        let card_resp = ui.interact(
                                            card.response.rect,
                                            egui::Id::new(("gc_protocol_card", i)),
                                            egui::Sense::click(),
                                        );
                                        // A11Y: la tarjeta es un click custom: se anuncia
                                        // como botón con su paso, pinta foco y actúa con
                                        // teclado (patrón mobile_menu_row).
                                        card_resp.widget_info(|| {
                                            egui::WidgetInfo::labeled(
                                                egui::WidgetType::Button,
                                                true,
                                                format!("Paso {} de {}", i + 1, total),
                                            )
                                        });
                                        if card_resp.has_focus() {
                                            theme
                                                .paint_focus_ring(ui.painter(), card.response.rect);
                                        }
                                        if card_resp.clicked()
                                            || (card_resp.has_focus()
                                                && ui.input(|i| {
                                                    i.key_pressed(egui::Key::Enter)
                                                        || i.key_pressed(egui::Key::Space)
                                                }))
                                        {
                                            nav.cursor = Some(i);
                                            nav.playing = false;
                                            let _ = select_protocol_step(app, i);
                                        }
                                        if is_current {
                                            ui.painter().rect_stroke(
                                                card.response.rect.expand(1.0),
                                                RADIUS_SM,
                                                egui::Stroke::new(1.0, accent),
                                            );
                                        }
                                        ui.add_space(SPACE_XS);
                                    }
                                },
                            );
                        }
                    });
            });
        });
}

#[cfg(test)]
mod statistics_interpolation_tests {
    use crate::sheet_layout::stable_interpolate;

    #[test]
    fn interpolation_preserves_subnormals_and_opposite_sign_extremes() {
        let minimum_subnormal = f64::from_bits(1);

        assert_eq!(
            stable_interpolate(minimum_subnormal, minimum_subnormal, 0.5),
            minimum_subnormal
        );
        assert_eq!(
            stable_interpolate(minimum_subnormal, f64::from_bits(2), 0.5),
            f64::from_bits(2)
        );
        assert_eq!(stable_interpolate(-2.0, 2.0, 0.5), 0.0);
        assert_eq!(stable_interpolate(-f64::MAX, f64::MAX, 0.5), 0.0);
        assert_eq!(stable_interpolate(f64::MAX, f64::MAX, 0.5), f64::MAX);
    }
}

#[cfg(test)]
mod drawer_slider_width_tests {
    use super::drawer_slider_width;

    #[test]
    fn slider_nunca_excede_viewport_con_margen() {
        // S2: a 360-390 px el riel no pasa de viewport-32 (margen 16+16).
        assert_eq!(drawer_slider_width(300.0, 360.0), 300.0);
        assert_eq!(drawer_slider_width(400.0, 360.0), 328.0);
        assert_eq!(drawer_slider_width(400.0, 390.0), 358.0);
        assert_eq!(drawer_slider_width(300.0, 1360.0), 300.0);
        // Degenerados: piso 0, jamás negativo.
        assert_eq!(drawer_slider_width(-10.0, 360.0), 0.0);
        assert_eq!(drawer_slider_width(300.0, 20.0), 0.0);
    }
}

#[cfg(test)]
mod domain_coloring_mutation_tests {
    use super::{set_complex_mapping_animation, set_domain_coloring_mode};
    use grafito_core::{ComplexGridObj, ComplexMappingObj, Document, GeoObject};

    #[test]
    fn unchanged_domain_coloring_mode_does_not_dirty_the_document() {
        let mut document = Document::new();
        let grid_id = document
            .try_add_object(GeoObject::ComplexGrid(ComplexGridObj::new(
                "z", -5.0, 5.0, -5.0, 5.0,
            )))
            .unwrap();
        let revision = document.version;

        assert!(!set_domain_coloring_mode(&mut document, grid_id, 0));
        assert_eq!(document.version, revision);

        assert!(set_domain_coloring_mode(&mut document, grid_id, 1));
        assert_eq!(document.version, revision + 1);
    }

    #[test]
    fn unchanged_mapping_animation_does_not_dirty_the_document() {
        let mut document = Document::new();
        let grid_id = document
            .try_add_object(GeoObject::ComplexGrid(ComplexGridObj::new(
                "z", -5.0, 5.0, -5.0, 5.0,
            )))
            .unwrap();
        let mapping_id = document
            .try_add_object(GeoObject::ComplexMapping(ComplexMappingObj::new(
                "z^2", grid_id,
            )))
            .unwrap();
        let revision = document.version;

        assert!(!set_complex_mapping_animation(
            &mut document,
            mapping_id,
            false,
            1.0,
        ));
        assert_eq!(document.version, revision);

        assert!(set_complex_mapping_animation(
            &mut document,
            mapping_id,
            true,
            1.5,
        ));
        assert_eq!(document.version, revision + 1);
    }
}

#[cfg(test)]
mod layer_panel_tests {
    use crate::sheet_layout::LayerPanelState;
    use grafito_core::Document;

    #[test]
    fn layer_panel_state_defaults_empty_on_layer_zero() {
        let state = LayerPanelState::default();
        assert_eq!(state.selected_layer, 0);
        // Q2: la tabla vive en el documento (persistente); el estado del
        // panel solo guarda la capa elegida.
        let document = Document::new();
        assert_eq!(document.used_layers(), Vec::<(u32, usize)>::new());
        assert!(document.is_layer_visible(0));
    }
}

#[cfg(test)]
mod regression_models_tests {
    use super::regression_model_commands_for;
    use grafito_ui::i18n::Locale;

    #[test]
    fn incluye_los_7_modelos_del_motor_con_logistica() {
        // B-A149: Es delega al canónico (etiquetas + comandos pineados);
        // el draw usa `regression_model_commands_for` con su locale.
        let models = regression_model_commands_for("T1", Locale::Es);
        assert_eq!(models.len(), 7, "modelos: {models:?}");
        let labels: Vec<&str> = models.iter().map(|(l, _)| l.as_str()).collect();
        assert_eq!(
            labels,
            [
                "Lineal",
                "Polinómico 2",
                "Exponencial",
                "Logarítmico",
                "Potencia",
                "Sinusoidal",
                "Logística"
            ]
        );
        let commands: Vec<&str> = models.iter().map(|(_, c)| c.as_str()).collect();
        for expected in [
            "FitLinear[T1]",
            "FitPoly[T1, 2]",
            "FitExp[T1]",
            "FitLog[T1]",
            "FitPow[T1]",
            "FitSin[T1]",
            "FitLogistic[T1]",
        ] {
            assert!(
                commands.contains(&expected),
                "falta {expected} en {commands:?}"
            );
        }
    }
}

// ══════════════════════════════════════════════════════════════════════════
// Frente G-C · Piel vs GeoGebra UI (2026-09-05) — ADITIVO
// ══════════════════════════════════════════════════════════════════════════
// Sliders Play + prompt auto, spreadsheet viva (lectura) y panel de
// probabilidad (Normal/Binomial/Poisson/T/Chi²/F con PDF/CDF honestos en f64).
//
// Reglas del frente: `fn render(&Estado) -> Frame` — estado efímero en
// `ctx.data` (cero campos nuevos en `GrafitoApp`), cero I/O/spawn en Ui::
// (los botones mutan estado en memoria o copian al portapapeles vía egui
// output, como el «Copiar SVG» existente), tokens TYPE/SPACE/RADIUS,
// Scandinavian 5/8/17, sin botones mudos. Cableado en paneles existentes
// y alcanzables (vista / trig) para que nada quede colgado.

// (movido a `sheet_layout.rs`: `ProbabilityPanelState`, slice 8 F14.4.4)

// (movido a `sheet_layout.rs`: `TrigPromptState`, slice 23 F14.4.4; el
// dibujo `draw_trig_speed_prompt` + estado efímero `ctx.data` quedan acá)

/// Curva continua + área P(X ≤ x_sel) del panel de probabilidad (C2).
/// Puro render: `density` viene del motor único; loop acotado (61 muestras).
/// Lo usan Normal, t-Student, χ² y F con el mismo sombreado.
#[allow(clippy::too_many_arguments)]
/// Etiquetas del eje del plot, cada una clipeada a su mitad: el caption
/// derecho nunca invade el centro ni lo cruza la curva.
fn plot_axis_labels(
    painter: &egui::Painter,
    plot: egui::Rect,
    plot_bot: f32,
    left: String,
    right: String,
    txt_dim: egui::Color32,
) {
    let mid_x = plot.min.x + plot.width() * 0.5;
    let font = egui::FontId::proportional(TYPE_XS);
    painter
        .with_clip_rect(egui::Rect::from_min_max(
            egui::pos2(plot.min.x, plot_bot),
            egui::pos2(mid_x, plot.max.y),
        ))
        .text(
            egui::pos2(plot.min.x, plot_bot + PROB_PLOT_LABEL_DY),
            egui::Align2::LEFT_TOP,
            left,
            font.clone(),
            txt_dim,
        );
    painter
        .with_clip_rect(egui::Rect::from_min_max(
            egui::pos2(mid_x, plot_bot),
            egui::pos2(plot.max.x, plot.max.y),
        ))
        .text(
            egui::pos2(plot.max.x, plot_bot + PROB_PLOT_LABEL_DY),
            egui::Align2::RIGHT_TOP,
            right,
            font,
            txt_dim,
        );
}

/// Caché de muestreo del plot (perf P1): las 61 muestras + ymax y las
/// masas discretas solo se recalculan cuando cambian los inputs; en idle se
/// reutilizan (antes: 61 pdfs con gamma_ln + allocs por frame a 60fps).
#[derive(Clone, Default)]
struct ProbDensityCache {
    key: (u8, [u64; 4]),
    pts: Vec<(f64, f64)>,
    ymax: f64,
}

#[derive(Clone, Default)]
struct ProbMassCache {
    key: (u8, [u64; 4]),
    masses: Vec<(u64, f64)>,
}

fn cached_density_samples(
    ctx: &egui::Context,
    key: (u8, [u64; 4]),
    compute: impl FnOnce() -> (Vec<(f64, f64)>, f64),
) -> (Vec<(f64, f64)>, f64) {
    let id = egui::Id::new("prob_density_cache");
    let mut cache: ProbDensityCache = ctx.data_mut(|data| data.get_temp(id)).unwrap_or_default();
    if cache.key != key || cache.pts.is_empty() {
        let (pts, ymax) = compute();
        cache = ProbDensityCache { key, pts, ymax };
        ctx.data_mut(|data| data.insert_temp(id, cache.clone()));
    }
    (cache.pts, cache.ymax)
}

fn cached_plot_masses(
    ctx: &egui::Context,
    key: (u8, [u64; 4]),
    compute: impl FnOnce() -> Vec<(u64, f64)>,
) -> Vec<(u64, f64)> {
    let id = egui::Id::new("prob_mass_cache");
    let mut cache: ProbMassCache = ctx.data_mut(|data| data.get_temp(id)).unwrap_or_default();
    if cache.key != key || cache.masses.is_empty() {
        let masses = compute();
        cache = ProbMassCache { key, masses };
        ctx.data_mut(|data| data.insert_temp(id, cache.clone()));
    }
    cache.masses
}

/// Pintado de densidad continua: 10 args de dibujo ineludibles
/// (geometría + colores + etiqueta); agruparlos en struct rompería los
/// call sites del plot sin beneficio. Excepción documentada, no deuda.
#[allow(clippy::too_many_arguments)]
fn paint_continuous_density(
    painter: &egui::Painter,
    plot: egui::Rect,
    cache_key: (u8, [u64; 4]),
    lo: f64,
    hi: f64,
    density: impl Fn(f64) -> f64,
    x_sel: f64,
    accent: egui::Color32,
    txt_dim: egui::Color32,
    right_label: String,
) {
    let plot_top = plot.min.y;
    let plot_bot = plot.max.y - PROB_PLOT_AXIS_RESERVE;
    let plot_h = (plot_bot - plot_top).max(1.0);
    let plot_w = plot.width().max(1.0);
    // Muestreo puro en `sheet_layout.rs` (slice 33 F14.4.4).
    let (pts, ymax) = cached_density_samples(painter.ctx(), cache_key, || {
        continuous_density_samples(density, lo, hi)
    });
    if pts.is_empty() {
        return;
    }
    let shade = accent.gamma_multiply(0.25);
    let to_px = |x: f64| plot.min.x + ((x - lo) / (hi - lo)) as f32 * plot_w;
    let to_py = |y: f64| plot_bot - (y / ymax) as f32 * plot_h;
    for window in pts.windows(2) {
        let [(x0, y0), (x1, y1)] = [window[0], window[1]];
        if x1 <= x_sel {
            let bar = egui::Rect::from_min_max(
                egui::pos2(to_px(x0), to_py(y0.max(y1))),
                egui::pos2(to_px(x1), plot_bot),
            );
            painter.rect_filled(bar, 0.0, shade);
        }
        painter.line_segment(
            [
                egui::pos2(to_px(x0), to_py(y0)),
                egui::pos2(to_px(x1), to_py(y1)),
            ],
            egui::Stroke::new(1.5, accent),
        );
    }
    if x_sel.is_finite() && x_sel >= lo && x_sel <= hi {
        let px = to_px(x_sel);
        painter.line_segment(
            [egui::pos2(px, plot_top), egui::pos2(px, plot_bot)],
            egui::Stroke::new(1.0, accent.gamma_multiply(0.6)),
        );
    }
    plot_axis_labels(
        painter,
        plot,
        plot_bot,
        format_statistic(lo),
        right_label,
        txt_dim,
    );
}

/// Color de las barras y la línea base **no seleccionadas** en los plots de
/// probabilidad (distribuciones, PMFs, CDFs).
///
/// Antes era `txt_dim.gamma_multiply(0.35)`: en tema oscuro el terciario
/// `#8C8C8C` caía a `#2E2E2E` sobre `#1A1A1A`, o sea **1.28:1** — la barra
/// no seleccionada era indistinguible del fondo, y lo que la gráfica comunicaba
/// («mirá la cola hasta acá») no se veía. En claro sobraba (18:1) y por eso el
/// bug pasaba desapercibido.
///
/// Ahora deriva del mismo acento que la serie, atenuado: mantiene la lectura
/// de «mismo dato, estado secundario» en los dos temas y hereda la identidad
/// cromática en vez de competir con ella como hacía un gris neutro.
fn unselected_bar_color(accent: egui::Color32) -> egui::Color32 {
    accent.gamma_multiply(0.45)
}

/// Etiqueta honesta dentro del plot cuando no hay curva para dibujar
/// (fuera de cota, parámetro inválido). Piel pura: solo pinta texto.
/// B-A157: prefijo localizado via `plot_message_text_for` (Es intacto).
fn paint_plot_message(
    painter: &egui::Painter,
    plot: egui::Rect,
    message: &str,
    txt_dim: egui::Color32,
    locale: grafito_ui::i18n::Locale,
) {
    painter.text(
        plot.center(),
        egui::Align2::CENTER_CENTER,
        plot_message_text_for(message, locale),
        egui::FontId::proportional(TYPE_XS),
        txt_dim,
    );
}

/// Barras discretas del plot de probabilidad con hover + a11y (S2
/// ui-polish). Pinta cada barra, expone `k` + masa vía hover y
/// `widget_info` (lector de pantalla), y escala la altura con el fundido
/// de entrada (`fade` 0→1 por distribución). Sin I/O, sin spawn; loop
/// acotado (≤256 barras, mismo tope del draw).
#[allow(clippy::too_many_arguments)]
fn paint_discrete_bars(
    ui: &egui::Ui,
    painter: &egui::Painter,
    dist: u8,
    plot_min_x: f32,
    plot_bot: f32,
    plot_h: f32,
    plot_w: f32,
    masses: &[(u64, f64)],
    k_sel: u64,
    accent: egui::Color32,
    unselected_bar: egui::Color32,
    fade: f32,
    plot_locale: grafito_ui::i18n::Locale,
) {
    if masses.is_empty() {
        return;
    }
    let ymax = discrete_ymax(masses);
    let bar_w = plot_w / masses.len() as f32;
    for (i, (k, pmf)) in masses.iter().enumerate() {
        let h = crate::sheet_layout::prob_plot_bar_h(*pmf, ymax, plot_h, fade);
        let bar = egui::Rect::from_min_size(
            egui::pos2(
                plot_min_x + i as f32 * bar_w + PROB_PLOT_BAR_INSET_X,
                plot_bot - h,
            ),
            egui::vec2(
                (bar_w - PROB_PLOT_BAR_GAP).max(PROB_PLOT_BAR_MIN),
                h.max(PROB_PLOT_BAR_MIN),
            ),
        );
        let selected = *k <= k_sel;
        painter.rect_filled(
            bar,
            PROB_PLOT_BAR_ROUNDING,
            if selected { accent } else { unselected_bar },
        );
        let bar_text = prob_plot_bar_text_for(*k, *pmf, plot_locale);
        ui.interact(
            bar,
            egui::Id::new(("prob_plot_bar", dist, *k)),
            egui::Sense::hover(),
        )
        .on_hover_text(bar_text.clone())
        .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, bar_text.clone()));
    }
}

/// Curva + área P(X ≤ x) del panel de probabilidad (F3b, extendido a
/// T/Chi²/F en C2). Puro render sobre el estado efímero: sin I/O, sin
/// spawn, loops acotados (≤61 muestras continuas, ≤256 barras discretas).
/// Reusa los wrappers de este archivo (motor único, sin duplicar).
/// B-A157: `plot_locale` para los 4 mensajes estaticos + `plot_df_*_for`
/// + prefijo (antes ES duro `:5756,:5761,:5969,:6006`).
///
/// Fila de tiles de probabilidad en grilla fija de 3 columnas: las tres
///   filas (2, 3 y 3 items) comparten los mismos anchos de columna y el mismo
///   gutter, así las columnas verticales coinciden. La celda vacía es aire.
fn prob_tile_row(
    ui: &mut egui::Ui,
    items: &[(String, String)],
    accent: egui::Color32,
    content_w: f32,
) {
    use crate::sheet_layout::{stat_tile_cols, stat_tile_width_for};
    use crate::spreadsheet::stat_tile;
    // Columnas adaptativas (3/2/1 según ancho real): a 320 siguen 3;
    // más angosto baja a 2 en vez de recortar. Una fila por chunk (antes
    // un solo horizontal con todos los chunks, que desbordaba con >3).
    let cols = stat_tile_cols(content_w);
    let tile_w = stat_tile_width_for(content_w, cols);
    for chunk in items.chunks(cols) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = SPACE_SM;
            for (label, value) in chunk {
                stat_tile(ui, tile_w, label, value, accent);
            }
            for _ in chunk.len()..cols {
                ui.add_space(tile_w);
            }
        });
    }
}

fn draw_probability_plot(
    ui: &mut egui::Ui,
    state: &ProbabilityPanelState,
    accent: egui::Color32,
    txt_dim: egui::Color32,
    plot_locale: grafito_ui::i18n::Locale,
    reduced_motion: bool,
) {
    let (rect, plot_resp) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), PROB_PLOT_H + PROB_PLOT_AXIS_RESERVE),
        egui::Sense::hover(),
    );
    // A11y: el plot anuncia su distribución al lector de pantalla; cada
    // barra discreta expone `k` + masa vía `paint_discrete_bars`.
    // Teclado: sin clic fantasma — el foco vive en el selector de
    // distribución + sliders, y los valores ya están como texto en
    // Resultado (las barras son suplemento visual/hover).
    plot_resp.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::Label,
            true,
            prob_dist_name_for(state.dist, plot_locale),
        )
    });
    if !ui.is_rect_visible(rect) {
        return;
    }
    // Transición al cambiar de distribución: cada `dist` estrena su
    // propio id de animación (0→1 la primera vez que se muestra).
    // `animate_bool` ya repinta mientras anima; el `request_repaint`
    // extra va acotado al fundido (`fade < 1`), jamás en idle.
    // animate_bool repinta mientras anima; el request_repaint explícito
    // duplicaba wakeups sobre el trabajo más pesado (plot re-escalado).
    // Movimiento reducido: el fundido al cambiar de distribución salta
    // al final (antes animaba siempre).
    let fade = if reduced_motion {
        1.0
    } else {
        ui.ctx()
            .animate_bool(egui::Id::new(("prob_plot_dist_fade", state.dist)), true)
    };
    let grow = crate::sheet_layout::prob_plot_grow(fade);
    let accent_anim = accent.gamma_multiply(grow);
    let painter = ui.painter().with_clip_rect(rect);
    let unselected_bar = unselected_bar_color(accent_anim);
    let plot = rect.shrink2(egui::vec2(PROB_PLOT_INSET, PROB_PLOT_INSET));
    let plot_bot = plot.max.y - PROB_PLOT_AXIS_RESERVE;
    let plot_h = (plot_bot - plot.min.y).max(1.0);
    let plot_w = plot.width().max(1.0);
    painter.line_segment(
        [
            egui::pos2(plot.min.x, plot_bot),
            egui::pos2(plot.max.x, plot_bot),
        ],
        egui::Stroke::new(PROB_PLOT_BASELINE_W, unselected_bar),
    );
    match state.dist {
        0 => {
            if !state.mu.is_finite() || !state.sigma.is_finite() || state.sigma <= 0.0 {
                paint_plot_message(
                    &painter,
                    plot,
                    grafito_ui::i18n::t("stats.prob_plot_sigma_positive", plot_locale),
                    txt_dim,
                    plot_locale,
                );
                return;
            }
            // Ventana pura en `sheet_layout.rs` (slice 29 F14.4.4).
            let Some((lo, hi)) = normal_plot_window(state.mu, state.sigma) else {
                paint_plot_message(
                    &painter,
                    plot,
                    grafito_ui::i18n::t("stats.prob_plot_mu_sigma_range", plot_locale),
                    txt_dim,
                    plot_locale,
                );
                return;
            };
            let (mu, sigma) = (state.mu, state.sigma);
            // Regla empírica: bandas μ±1σ/2σ/3σ detrás de la curva (detalle
            // pedagógico del caso Normal: 68.3% / 95.5% / 99.7%;
            // fracciones puras en `sheet_layout.rs`, slice 28 F14.4.4).
            for (band_lo, band_hi, alpha) in normal_band_fractions(mu, sigma, lo, hi) {
                let x0 = plot.min.x + band_lo.clamp(0.0, 1.0) * plot_w;
                let x1 = plot.min.x + band_hi.clamp(0.0, 1.0) * plot_w;
                if x1 > x0 {
                    painter.rect_filled(
                        egui::Rect::from_min_max(
                            egui::pos2(x0, plot.min.y),
                            egui::pos2(x1, plot_bot),
                        ),
                        0.0,
                        accent
                            .gamma_multiply(crate::sheet_layout::prob_plot_band_alpha(alpha, fade)),
                    );
                }
            }
            paint_continuous_density(
                &painter,
                plot,
                (0, [mu.to_bits(), sigma.to_bits(), 0, 0]),
                lo,
                hi,
                // W-A: `NaN` ante error (el filtro de `paint_continuous_density`
                // lo salta) en vez de `0.0` que dibujaba un plano falso en 0.
                |x| normal_pdf(x, mu, sigma).unwrap_or(f64::NAN),
                state.x,
                accent_anim,
                txt_dim,
                "P(X≤x) · 1σ/2σ/3σ".to_string(),
            );
        }
        1 => {
            // Ventana + clamps puros en `sheet_layout.rs` (slice 28 F14.4.4).
            let n = clamp_binomial_n(state.n);
            let k_sel = clamp_binomial_k(state.k, n);
            let (lo, hi) = binomial_plot_window(n, state.p, k_sel);
            if hi < lo {
                return;
            }
            let count = (hi - lo + 1) as usize;
            if count == 0 || count > 256 {
                return;
            }
            // Masas puras en `sheet_layout.rs` (slice 33 F14.4.4).
            let masses = cached_plot_masses(ui.ctx(), (1, [n, state.p.to_bits(), lo, hi]), || {
                binomial_plot_masses(n, state.p, lo, hi)
            });
            if masses.is_empty() {
                return;
            }
            paint_discrete_bars(
                ui,
                &painter,
                state.dist,
                plot.min.x,
                plot_bot,
                plot_h,
                plot_w,
                &masses,
                k_sel,
                accent_anim,
                unselected_bar,
                fade,
                plot_locale,
            );
            plot_axis_labels(
                &painter,
                plot,
                plot_bot,
                binomial_plot_left_label(lo),
                binomial_plot_right_label(n, lo, hi),
                txt_dim,
            );
        }
        2 => {
            // Ventana + k_sel puros en `sheet_layout.rs` (slice 29 F14.4.4).
            let k_sel = discrete_k_sel(state.k);
            let lambda = state.lambda;
            let Some(hi) = poisson_plot_hi(lambda, k_sel) else {
                return;
            };
            // Masas puras en `sheet_layout.rs` (slice 33 F14.4.4).
            let masses = cached_plot_masses(ui.ctx(), (2, [lambda.to_bits(), hi, 0, 0]), || {
                poisson_plot_masses(lambda, hi)
            });
            if masses.is_empty() {
                return;
            }
            paint_discrete_bars(
                ui,
                &painter,
                state.dist,
                plot.min.x,
                plot_bot,
                plot_h,
                plot_w,
                &masses,
                k_sel,
                accent_anim,
                unselected_bar,
                fade,
                plot_locale,
            );
            plot_axis_labels(
                &painter,
                plot,
                plot_bot,
                discrete_plot_left_label(),
                discrete_plot_right_label(hi),
                txt_dim,
            );
        }
        3 => {
            // W-A: sin `clamp` silencioso — fuera de cota se avisa, no se
            // dibuja la curva de gl=500 como si fuera la pedida.
            // B-A157: `plot_df_*_for` + prefijo localizados con `plot_locale`.
            let df = match plot_df_or_fuera_de_cota_for(state.df, "gl", plot_locale) {
                Ok(df) => df,
                Err(message) => {
                    paint_plot_message(&painter, plot, &message, txt_dim, plot_locale);
                    return;
                }
            };
            if !state.x.is_finite() {
                return;
            }
            // Ventana fija pura en `sheet_layout.rs` (slice 29 F14.4.4).
            paint_continuous_density(
                &painter,
                plot,
                (3, [df.to_bits(), 0, 0, 0]),
                T_PLOT_LO,
                T_PLOT_HI,
                |x| student_t_pdf(x, df).unwrap_or(f64::NAN),
                state.x,
                accent_anim,
                txt_dim,
                format!("P(X≤x) · t ±6 (gl={})", format_statistic(df)),
            );
        }
        4 => {
            let df = match plot_df_or_fuera_de_cota_for(state.df, "gl", plot_locale) {
                Ok(df) => df,
                Err(message) => {
                    paint_plot_message(&painter, plot, &message, txt_dim, plot_locale);
                    return;
                }
            };
            if !state.x.is_finite() {
                return;
            }
            // Ventana pura en `sheet_layout.rs` (slice 29 F14.4.4).
            let Some(hi) = chi_squared_plot_hi(df, state.x) else {
                return;
            };
            paint_continuous_density(
                &painter,
                plot,
                (4, [df.to_bits(), state.x.to_bits(), 0, 0]),
                0.0,
                hi,
                |x| chi_squared_pdf(x, df).unwrap_or(f64::NAN),
                state.x,
                accent_anim,
                txt_dim,
                format!("P(X≤x) · χ² (gl={})", format_statistic(df)),
            );
        }
        5 => {
            let (df1, df2) = match (
                plot_df_or_fuera_de_cota_for(state.df1, "gl1", plot_locale),
                plot_df_or_fuera_de_cota_for(state.df2, "gl2", plot_locale),
            ) {
                (Ok(df1), Ok(df2)) => (df1, df2),
                (Err(message), _) | (_, Err(message)) => {
                    paint_plot_message(&painter, plot, &message, txt_dim, plot_locale);
                    return;
                }
            };
            if !state.x.is_finite() {
                return;
            }
            // Ventana pura en `sheet_layout.rs` (slice 29 F14.4.4).
            let Some(hi) = f_plot_hi(state.x) else {
                return;
            };
            paint_continuous_density(
                &painter,
                plot,
                (5, [df1.to_bits(), df2.to_bits(), state.x.to_bits(), 0]),
                0.0,
                hi,
                |x| f_distribution_pdf(x, df1, df2).unwrap_or(f64::NAN),
                state.x,
                accent_anim,
                txt_dim,
                format!(
                    "P(X≤x) · F({},{})",
                    format_statistic(df1),
                    format_statistic(df2)
                ),
            );
        }
        6 => {
            // Ventana + k_sel puros en `sheet_layout.rs` (slice 29 F14.4.4).
            // B-A157: mensaje localizado con `plot_locale` (antes ES duro).
            let k_sel = discrete_k_sel(state.k);
            let prob = state.p;
            let Some(hi) = geometric_plot_hi(prob, k_sel) else {
                paint_plot_message(
                    &painter,
                    plot,
                    grafito_ui::i18n::t("stats.prob_plot_p_range", plot_locale),
                    txt_dim,
                    plot_locale,
                );
                return;
            };
            // Masas puras en `sheet_layout.rs` (slice 33 F14.4.4).
            let masses = cached_plot_masses(ui.ctx(), (6, [prob.to_bits(), hi, 0, 0]), || {
                geometric_plot_masses(prob, hi)
            });
            if masses.is_empty() {
                return;
            }
            paint_discrete_bars(
                ui,
                &painter,
                state.dist,
                plot.min.x,
                plot_bot,
                plot_h,
                plot_w,
                &masses,
                k_sel,
                accent_anim,
                unselected_bar,
                fade,
                plot_locale,
            );
            plot_axis_labels(
                &painter,
                plot,
                plot_bot,
                discrete_plot_left_label(),
                discrete_plot_right_label(hi),
                txt_dim,
            );
        }
        7 => {
            let (a, b) = (state.ua, state.ub);
            if !(a.is_finite() && b.is_finite() && a < b) {
                paint_plot_message(
                    &painter,
                    plot,
                    grafito_ui::i18n::t("stats.prob_plot_ab_order", plot_locale),
                    txt_dim,
                    plot_locale,
                );
                return;
            }
            if !state.x.is_finite() {
                return;
            }
            paint_continuous_density(
                &painter,
                plot,
                (7, [a.to_bits(), b.to_bits(), 0, 0]),
                a,
                b,
                |x| uniform_pdf(x, a, b).unwrap_or(f64::NAN),
                state.x,
                accent_anim,
                txt_dim,
                format!(
                    "P(X≤x) · U({},{})",
                    format_statistic(a),
                    format_statistic(b)
                ),
            );
        }
        _ => {}
    }
}

// (movido a `sheet_layout.rs`: `PROB_DISTS` + `distribution_moments` +
// `prob_case_command` + `prob_quantile_command`, slice 8 F14.4.4)

/// B-A121: hover del chip α del cuantil en el idioma activo (template
/// `{pct}`). Es delega al literal canónico previo (`:6201`, hover
/// Cuantil-pct con 1 decimal); resto vía `stats_msg`.
/// Pura, sin egui: testeada sin harness.
pub(crate) fn quantile_alpha_hover_text(alpha: f64, locale: grafito_ui::i18n::Locale) -> String {
    grafito_ui::i18n::t("stats.prob_quantile_alpha_hint", locale)
        .replace("{pct}", &format!("{:.1}", alpha * 100.0))
}

/// B-A122: resultado del cuantil en el idioma activo (templates
/// `{p}/{q}/{extra}`). `p`/`q` ya formateados (`{:.3}` + `format_statistic`);
/// `extra` es `""` o `" (…)"` con notación intacta (`n/gl/λ/a,b/p`).
/// Puras, sin egui: testeadas sin harness.
pub(crate) fn quantile_result_cont_text(
    p: &str,
    q: &str,
    extra: &str,
    locale: grafito_ui::i18n::Locale,
) -> String {
    grafito_ui::i18n::t("stats.prob_quantile_result_cont", locale)
        .replace("{p}", p)
        .replace("{q}", q)
        .replace("{extra}", extra)
}

/// B-A122: variante discreta (`menor k con…`). Mismo contrato que la
/// continua; `extra` es `""` o `" (…)"`.
pub(crate) fn quantile_result_disc_text(
    p: &str,
    q: &str,
    extra: &str,
    locale: grafito_ui::i18n::Locale,
) -> String {
    grafito_ui::i18n::t("stats.prob_quantile_result_disc", locale)
        .replace("{p}", p)
        .replace("{q}", q)
        .replace("{extra}", extra)
}

/// B-A123: toast "Datos ordenados (N={n})" en el idioma activo (template
/// `{n}`). Pura, sin egui: testeada sin harness. El `format!` previo con
/// ES duro vivía en `draw_statistics_panel` (`:2780`).
pub(crate) fn statistics_sorted_text(n: usize, locale: grafito_ui::i18n::Locale) -> String {
    grafito_ui::i18n::t("stats.sorted_toast", locale).replace("{n}", &n.to_string())
}

/// B-A155: hovers de la fila de parámetro en el idioma activo (templates
/// con `{label}`). Puras, sin egui: testeadas sin harness. El rótulo que
/// reciben ya viene localizado por el llamador (`draw_probability_section`).
pub(crate) fn prob_param_hover_text(label: &str, locale: grafito_ui::i18n::Locale) -> String {
    grafito_ui::i18n::t("stats.prob_param_hover", locale).replace("{label}", label)
}

/// B-A155: hover del valor en el idioma activo (template `{label}`).
/// Pura, sin egui: testeada sin harness.
pub(crate) fn prob_param_value_text(label: &str, locale: grafito_ui::i18n::Locale) -> String {
    grafito_ui::i18n::t("stats.prob_param_value_hover", locale).replace("{label}", label)
}

/// Ancho de slider sin desborde en drawer angosto: nunca excede lo real
/// (`available`) ni el viewport con 16 px de margen por lado (a 360 px un
/// riel de 300 + chrome del panel se salía por la derecha). Generaliza el
/// `slider_width = available - chrome` de la pizarra. Puro y pineado.
pub(crate) fn drawer_slider_width(panel_width: f32, viewport_width: f32) -> f32 {
    panel_width.max(0.0).min((viewport_width - 32.0).max(0.0))
}

/// Fila de parámetro escandinava: rótulo + valor editable a la derecha y
/// slider a ancho completo debajo, sin número duplicado en el medio.
/// `logarithmic` para tasas (λ); el DragValue permite tipear el valor exacto.
/// Fila de parámetro: 8 args de layout ineludibles (widget + rango +
/// formato + locale); struct rompería los call sites. Excepción documentada.
#[allow(clippy::too_many_arguments)]
fn prob_param_row(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f64,
    slider_w: f32,
    range: std::ops::RangeInclusive<f64>,
    decimals: usize,
    logarithmic: bool,
    locale: grafito_ui::i18n::Locale,
) {
    let theme = current_theme(ui.ctx());
    // Dos líneas (F13.4 como se documentó): rótulo + valor arriba, slider
    // a ancho completo abajo. En una sola línea el slider tapaba el rótulo.
    // Línea 1: rótulo a la izquierda, valor editable a la derecha.
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = SPACE_SM;
        // Rótulo con elide (DE/FR largo desbordaba la fila): el completo
        // viaja en el hover, como el inspector.
        ui.add(
            egui::Label::new(
                egui::RichText::new(label)
                    .color(theme.text_tertiary)
                    .size(TYPE_XS)
                    .strong(),
            )
            .truncate(),
        )
        .on_hover_text(prob_param_hover_text(label, locale));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let speed = ((*range.end() - *range.start()).abs() / 200.0).max(0.001);
            ui.add_sized(
                [RANGE_FIELD_W, HIT_TARGET_MIN],
                egui::DragValue::new(value)
                    .range(range.clone())
                    .speed(speed)
                    .fixed_decimals(decimals)
                    .update_while_editing(false),
            )
            .on_hover_text(prob_param_value_text(label, locale));
        });
    });
    ui.add_space(SPACE_XS);
    // Línea 2: slider a ancho completo (nunca tapa el rótulo). OJO:
    // egui ignora el tamaño de `add_sized` para el riel y usa
    // `spacing().slider_width` (default 100px): hay que fijarlo o el
    // slider sale angosto aunque el cursor avance `slider_w`.
    ui.spacing_mut().slider_width = slider_w.max(0.0);
    let mut slider = egui::Slider::new(value, range).show_value(false);
    if logarithmic {
        slider = slider.logarithmic(true);
    }
    ui.add_sized([slider_w.max(0.0), HIT_TARGET_MIN], slider);
    ui.add_space(SPACE_XS);
}

/// Sección Probabilidad: Normal / Binomial / Poisson / t-Student / χ² / F
/// / Geométrica / Uniforme / Exponencial con PDF/CDF honestos. Llamada desde el panel Vista (alcanzable) — sin
/// botones mudos: el selector cambia la distribución y cada parámetro
/// recalcula en vivo.
pub(crate) fn draw_probability_section(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    app: &mut GrafitoApp,
) {
    let id = egui::Id::new("gc_probability_state");
    let mut state: ProbabilityPanelState = ctx
        .data_mut(|data| data.get_temp::<ProbabilityPanelState>(id))
        .unwrap_or_default();
    let (_is_dark, accent, _fill, _sep, txt_col, txt_dim, hdr_col) = panel_theme_local(ctx);
    // B-A119: puente caso/cuantil + hovers en el idioma activo (antes ES
    // duro `:6124,:6143,:6149,:6301,:6305,:6320,:6326`). `app` ya está en
    // scope (sin cambio de firma, patrón B-A105); los labels son
    // `&'static` (sin borrow retenido de `app`).
    // B-A120: caso + tiles + momentos en el idioma activo (antes ES duro
    // `:6046,:6053,:6054,:6064,:6085,:6093,:6094,:6095,:6100,:6108,`
    // `:6120,:6121,:6122`).
    use grafito_ui::i18n::t as stats_t;
    let prob_locale = app.config_locale();
    let use_case_label = stats_t("stats.prob_use_case", prob_locale);
    let no_case_label = stats_t("stats.prob_no_case", prob_locale);
    let use_quantile_label = stats_t("stats.prob_use_quantile", prob_locale);
    let no_quantile_label = stats_t("stats.prob_no_quantile", prob_locale);
    let quantile_hint_label = stats_t("stats.prob_quantile_hint", prob_locale);
    let prob_case_label = stats_t("stats.prob_case", prob_locale);
    let prob_copy_label = stats_t("stats.prob_copy", prob_locale);
    let prob_copy_case_hint = stats_t("stats.prob_copy_case_hint", prob_locale);
    let prob_case_toast = stats_t("stats.prob_case_toast", prob_locale);
    let prob_cum_label = stats_t("stats.prob_cum", prob_locale);
    let prob_tail_le = stats_t("stats.prob_tail_le", prob_locale);
    let prob_tail_gt = stats_t("stats.prob_tail_gt", prob_locale);
    let prob_two_tail = stats_t("stats.prob_two_tail", prob_locale);
    let prob_f64_note = stats_t("stats.prob_f64_note", prob_locale);
    let prob_moments_label = stats_t("stats.prob_moments", prob_locale);
    let prob_mean_label = stats_t("stats.prob_mean", prob_locale);
    let prob_var_label = stats_t("stats.prob_var", prob_locale);
    let prob_sd_label = stats_t("stats.prob_sd", prob_locale);
    // B-A121: cuantil chrome en el idioma activo (antes ES duro
    // `:6189,:6201,:6209,:6294,:6298,:6313,:6318,:6337`; `:6293`
    // reutiliza `stats.prob_copy` existente).
    // Ancho útil: constante de panel 300 (marco 12+4, gutter 4 = 280),
    // PERO nunca más que lo real (`min` con `available_width` medido
    // acá, antes del scroll) ni que el viewport con 16 px de margen por
    // lado (a 360 el riel de 300 + chrome se salía por la derecha). Si la
    // ventana queda más angosta o la escala de texto crece, todo encoge
    // en vez de cortarse al borde.
    let prob_row_w = drawer_slider_width(
        PANEL_LEFT_DEFAULT - SPACE_MD - SPACE_XS - SCROLL_CONTENT_PAD_R,
        ctx.screen_rect().width(),
    )
    .min(ui.available_width().max(0.0));
    // Si `available_width` falló (0), el constante de antes.
    let prob_row_w = if prob_row_w > 0.0 {
        prob_row_w
    } else {
        PANEL_LEFT_DEFAULT - SPACE_MD - SPACE_XS - SCROLL_CONTENT_PAD_R
    };
    let prob_quantile_title = stats_t("stats.prob_quantile_title", prob_locale);
    let prob_quantile_param = stats_t("stats.prob_quantile_param", prob_locale);
    let prob_copy_quantile_hint = stats_t("stats.prob_copy_quantile_hint", prob_locale);
    let prob_quantile_toast = stats_t("stats.prob_quantile_toast", prob_locale);
    let prob_quantile_note = stats_t("stats.prob_quantile_note", prob_locale);
    let prob_quantile_note_hint = stats_t("stats.prob_quantile_note_hint", prob_locale);
    let prob_quantile_ready_toast = stats_t("stats.prob_quantile_ready_toast", prob_locale);
    // B-A133: headers Parámetros/Resultado en el idioma activo (antes ES
    // duro `:6090,:6157`).
    let prob_params_title = stats_t("stats.prob_params_title", prob_locale);
    let prob_result_title = stats_t("stats.prob_result_title", prob_locale);
    // B-A155: rótulos de parámetros en el idioma activo (vía keys
    // `stats.prob_param_*`; hovers por helpers con `{label}`).
    let prob_param_mu_label = stats_t("stats.prob_param_mu", prob_locale);
    let prob_param_sigma_label = stats_t("stats.prob_param_sigma", prob_locale);
    let prob_param_x_label = stats_t("stats.prob_param_x", prob_locale);
    let prob_param_n_label = stats_t("stats.prob_param_n", prob_locale);
    let prob_param_p_label = stats_t("stats.prob_param_p", prob_locale);
    let prob_param_k_success_label = stats_t("stats.prob_param_k_success", prob_locale);
    let prob_param_lambda_label = stats_t("stats.prob_param_lambda", prob_locale);
    let prob_param_k_events_label = stats_t("stats.prob_param_k_events", prob_locale);
    let prob_param_df_label = stats_t("stats.prob_param_df", prob_locale);
    let prob_param_df1_label = stats_t("stats.prob_param_df1", prob_locale);
    let prob_param_df2_label = stats_t("stats.prob_param_df2", prob_locale);
    let prob_param_k_failures_label = stats_t("stats.prob_param_k_failures", prob_locale);
    let prob_param_a_label = stats_t("stats.prob_param_a", prob_locale);
    let prob_param_b_label = stats_t("stats.prob_param_b", prob_locale);

    // Selector único a ancho completo (un ComboBox ordenado en vez de 9
    // chips que se envolvían y rompían la grilla del panel angosto).
    // Nombres + tips display-localizados (B-A156); el `PROB_DISTS`
    // canónico queda como dueño de índices/orden en `sheet_layout.rs`.
    let current_name = prob_dist_name_for(state.dist, prob_locale);
    let combo_dist = egui::ComboBox::from_id_salt("prob_dist_selector")
        .width(ui.available_width().max(0.0))
        .selected_text(current_name)
        .show_ui(ui, |ui| {
            for index in 0..=8u8 {
                ui.selectable_value(
                    &mut state.dist,
                    index,
                    prob_dist_name_for(index, prob_locale),
                )
                .on_hover_text(prob_dist_tip_for(index, prob_locale));
            }
        });
    if let Some(tip) = prob_dist_tooltip_for(state.dist, prob_locale) {
        // Ayuda en hover del combo, no línea permanente.
        combo_dist.response.on_hover_text(tip);
    }
    ui.add_space(SPACE_XS);

    ui.label(
        egui::RichText::new(prob_params_title)
            .color(hdr_col)
            .size(TYPE_XS)
            .strong(),
    );
    ui.add_space(SPACE_XS);

    // Parámetros por distribución: rótulo + valor editable arriba, slider
    // a ancho completo abajo (F13.4, sin número duplicado en el medio).
    match state.dist {
        0 => {
            prob_param_row(
                ui,
                prob_param_mu_label,
                &mut state.mu,
                prob_row_w,
                -10.0..=10.0,
                2,
                false,
                prob_locale,
            );
            prob_param_row(
                ui,
                prob_param_sigma_label,
                &mut state.sigma,
                prob_row_w,
                0.1..=5.0,
                2,
                false,
                prob_locale,
            );
            prob_param_row(
                ui,
                prob_param_x_label,
                &mut state.x,
                prob_row_w,
                -10.0..=10.0,
                2,
                false,
                prob_locale,
            );
        }
        1 => {
            prob_param_row(
                ui,
                prob_param_n_label,
                &mut state.n,
                prob_row_w,
                1.0..=MAX_BINOMIAL_N as f64,
                0,
                false,
                prob_locale,
            );
            prob_param_row(
                ui,
                prob_param_p_label,
                &mut state.p,
                prob_row_w,
                0.0..=1.0,
                3,
                false,
                prob_locale,
            );
            prob_param_row(
                ui,
                prob_param_k_success_label,
                &mut state.k,
                prob_row_w,
                0.0..=MAX_BINOMIAL_N as f64,
                0,
                false,
                prob_locale,
            );
        }
        2 => {
            prob_param_row(
                ui,
                prob_param_lambda_label,
                &mut state.lambda,
                prob_row_w,
                0.1..=20.0,
                2,
                true,
                prob_locale,
            );
            prob_param_row(
                ui,
                prob_param_k_events_label,
                &mut state.k,
                prob_row_w,
                0.0..=50.0,
                0,
                false,
                prob_locale,
            );
        }
        3 => {
            prob_param_row(
                ui,
                prob_param_df_label,
                &mut state.df,
                prob_row_w,
                1.0..=30.0,
                0,
                false,
                prob_locale,
            );
            prob_param_row(
                ui,
                prob_param_x_label,
                &mut state.x,
                prob_row_w,
                -6.0..=6.0,
                2,
                false,
                prob_locale,
            );
        }
        4 => {
            prob_param_row(
                ui,
                prob_param_df_label,
                &mut state.df,
                prob_row_w,
                1.0..=30.0,
                0,
                false,
                prob_locale,
            );
            prob_param_row(
                ui,
                prob_param_x_label,
                &mut state.x,
                prob_row_w,
                0.0..=20.0,
                2,
                false,
                prob_locale,
            );
        }
        5 => {
            prob_param_row(
                ui,
                prob_param_df1_label,
                &mut state.df1,
                prob_row_w,
                1.0..=30.0,
                0,
                false,
                prob_locale,
            );
            prob_param_row(
                ui,
                prob_param_df2_label,
                &mut state.df2,
                prob_row_w,
                1.0..=30.0,
                0,
                false,
                prob_locale,
            );
            prob_param_row(
                ui,
                prob_param_x_label,
                &mut state.x,
                prob_row_w,
                0.0..=10.0,
                2,
                false,
                prob_locale,
            );
        }
        6 => {
            prob_param_row(
                ui,
                prob_param_p_label,
                &mut state.p,
                prob_row_w,
                0.01..=1.0,
                3,
                false,
                prob_locale,
            );
            prob_param_row(
                ui,
                prob_param_k_failures_label,
                &mut state.k,
                prob_row_w,
                0.0..=50.0,
                0,
                false,
                prob_locale,
            );
        }
        7 => {
            prob_param_row(
                ui,
                prob_param_a_label,
                &mut state.ua,
                prob_row_w,
                -10.0..=10.0,
                2,
                false,
                prob_locale,
            );
            prob_param_row(
                ui,
                prob_param_b_label,
                &mut state.ub,
                prob_row_w,
                -10.0..=10.0,
                2,
                false,
                prob_locale,
            );
            prob_param_row(
                ui,
                prob_param_x_label,
                &mut state.x,
                prob_row_w,
                -10.0..=10.0,
                2,
                false,
                prob_locale,
            );
        }
        _ => {
            prob_param_row(
                ui,
                prob_param_lambda_label,
                &mut state.lambda,
                prob_row_w,
                0.1..=10.0,
                2,
                true,
                prob_locale,
            );
            prob_param_row(
                ui,
                prob_param_x_label,
                &mut state.x,
                prob_row_w,
                0.0..=20.0,
                2,
                false,
                prob_locale,
            );
        }
    }
    ui.add_space(SPACE_SM);
    ui.label(
        egui::RichText::new(prob_result_title)
            .color(hdr_col)
            .size(TYPE_XS)
            .strong(),
    );
    ui.add_space(SPACE_XS);

    // Caso puntual puro en `sheet_layout.rs` (slice 31 F14.4.4).
    let result: Result<(f64, f64, String), String> = prob_case_result(&state);
    match result {
        Ok((pdf, cdf, detail)) => {
            let point_label = prob_point_label_for(state.dist, prob_locale);
            // P-valores honestos: colas clampeadas a [0, 1]; `—` si el CDF
            // no es finito, en vez de inventar una probabilidad.
            // Textos puros en `sheet_layout.rs` (slice 39 F14.4.4).
            let tails = prob_tail_values(cdf);
            let (tail_left, tail_right, two_tails) = (
                prob_tail_text(tails, |t| t.0),
                prob_tail_text(tails, |t| t.1),
                prob_tail_text(tails, |t| t.2),
            );
            // Una línea como siempre: rótulo, detalle y Copiar. El
            // detalle ocupa EXACTO lo que queda medido en vivo (botón
            // primero desde la derecha, resto para el texto): la resta
            // a mano de antes ignoraba el rótulo y expulsaba al botón
            // ("Cop…"). Sin constantes, imposible desbordar.
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(prob_case_label)
                        .color(txt_dim)
                        .size(TYPE_XS)
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Botón auto-ancho al texto ("Copiar" no entraba en
                    // 64px y se partía en dos líneas); el detalle usa el
                    // resto medido en vivo: ni wrap ni desborde.
                    if ui
                        .button(egui::RichText::new(prob_copy_label).size(TYPE_XS).strong())
                        .on_hover_text(prob_copy_case_hint)
                        .clicked()
                    {
                        ctx.copy_text(prob_case_copy_text(
                            &detail,
                            point_label,
                            pdf,
                            cdf,
                            &tail_right,
                        ));
                        app.cas_result = prob_case_toast.to_string();
                    }
                    ui.add_sized(
                        egui::vec2(ui.available_width().max(40.0), PANEL_BUTTON_H),
                        egui::Label::new(
                            egui::RichText::new(&detail)
                                .color(txt_col)
                                .size(TYPE_SM)
                                .strong(),
                        )
                        .truncate(),
                    )
                    .on_hover_text(prob_case_detail_tooltip(&detail));
                });
            });
            ui.add_space(SPACE_XS);
            prob_tile_row(
                ui,
                &[
                    (point_label.to_string(), format!("{pdf:.6}")),
                    (prob_cum_label.to_string(), format!("{cdf:.6}")),
                ],
                accent,
                prob_row_w,
            );
            ui.add_space(SPACE_XS);
            prob_tile_row(
                ui,
                &[
                    (prob_tail_le.to_string(), tail_left),
                    (prob_tail_gt.to_string(), tail_right),
                    (prob_two_tail.to_string(), two_tails),
                ],
                txt_col,
                prob_row_w,
            );
            ui.label(
                egui::RichText::new(prob_f64_note)
                    .color(txt_dim)
                    .size(TYPE_XS),
            );
            ui.add_space(SPACE_XS);
            // Momentos analíticos (media/varianza/desvío) con `—` honesto si
            // no están definidos para estos parámetros.
            ui.label(
                egui::RichText::new(prob_moments_label)
                    .color(hdr_col)
                    .size(TYPE_XS)
                    .strong(),
            );
            ui.add_space(SPACE_XS);
            let (mean, variance) = distribution_moments(&state);
            let deviation = variance.map(f64::sqrt);
            // Texto puro en `sheet_layout.rs` (slice 39 F14.4.4).
            prob_tile_row(
                ui,
                &[
                    (prob_mean_label.to_string(), prob_moment_text(mean)),
                    (prob_var_label.to_string(), prob_moment_text(variance)),
                    (prob_sd_label.to_string(), prob_moment_text(deviation)),
                ],
                accent,
                prob_row_w,
            );
            ui.add_space(SPACE_XS);
            // B-A157: plot localizado con `prob_locale` (mensajes + cota + prefijo).
            draw_probability_plot(ui, &state, accent, txt_dim, prob_locale, app.reduced_motion);
            // Puente siempre visible: deshabilitado con motivo donde no
            // hay comando (t, F, Geométrica) en vez de aparecer/desaparecer
            // y saltar el layout.
            ui.add_space(SPACE_MD);
            match prob_case_command(&state) {
                Some(command) => {
                    if ui
                        .add_sized(
                            [prob_row_w, PANEL_BUTTON_H],
                            egui::Button::new(egui::RichText::new(use_case_label).size(TYPE_SM))
                                .rounding(RADIUS_SM),
                        )
                        .clicked()
                    {
                        app.input_text = command;
                        app.command_input_focus_requested = true;
                        app.cas_result =
                            "Caso de probabilidad listo en la entrada (Enter para evaluar)."
                                .to_string();
                        ctx.request_repaint();
                    }
                }
                None => {
                    ui.add_enabled_ui(false, |ui| {
                        ui.add_sized(
                            [prob_row_w, PANEL_BUTTON_H],
                            egui::Button::new(egui::RichText::new(use_case_label).size(TYPE_SM))
                                .rounding(RADIUS_SM),
                        )
                    })
                    .response
                    .on_hover_text(no_case_label);
                }
            }
        }
        Err(error) => {
            ui.label(
                egui::RichText::new(error)
                    .color(current_theme(ctx).danger)
                    .size(TYPE_XS),
            );
        }
    }
    ui.add_space(SPACE_XS);
    ui.label(
        egui::RichText::new(prob_quantile_title)
            .color(hdr_col)
            .size(TYPE_XS)
            .strong(),
    );
    // α clásicos de tabla: un clic fija el cuantil (F13.3).
    // Grilla 3×2 a ancho real (como los tiles): los 6 α siempre
    // visibles sin wrap ni scroll — el wrapped cortaba la 2ª fila.
    use crate::sheet_layout::{stat_tile_cols, stat_tile_width_for};
    let cols = stat_tile_cols(prob_row_w);
    let alpha_w = stat_tile_width_for(prob_row_w, cols);
    for chunk in PROB_QUANTILE_ALPHAS.chunks(cols) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = SPACE_SM;
            for alpha in chunk {
                let selected = (state.p_inv - *alpha).abs() < 1e-9;
                if ui
                    .add_sized(
                        egui::vec2(alpha_w, PANEL_BUTTON_H),
                        egui::SelectableLabel::new(selected, format!("{alpha:.3}")),
                    )
                    .on_hover_text(quantile_alpha_hover_text(*alpha, prob_locale))
                    .clicked()
                {
                    state.p_inv = *alpha;
                }
            }
        });
        ui.add_space(SPACE_XS);
    }
    ui.add_space(SPACE_XS);
    prob_param_row(
        ui,
        prob_quantile_param,
        &mut state.p_inv,
        prob_row_w,
        0.001..=0.999,
        3,
        false,
        prob_locale,
    );
    let quantile: Result<String, String> = (|| {
        // B-A122: resultado vía `stats.prob_quantile_result_cont/disc`
        // con `prob_locale` (antes 9 `format!` ES duros `:6240-6304`);
        // notación `P(X≤x)/P(X≤k)` + extras `(n/gl/λ/a,b/p)` intactos.
        let p_str = format!("{:.3}", state.p_inv);
        Ok(match state.dist {
            0 => {
                let q = normal_quantile_honest(state.p_inv, state.mu, state.sigma)?;
                quantile_result_cont_text(&p_str, &format_statistic(q), "", prob_locale)
            }
            1 => {
                let n = clamp_binomial_n(state.n);
                let q = binomial_quantile_honest(state.p_inv, n, state.p)?;
                quantile_result_disc_text(&p_str, &q.to_string(), &format!(" (n={n})"), prob_locale)
            }
            2 => {
                let q = poisson_quantile_honest(state.p_inv, state.lambda)?;
                quantile_result_disc_text(
                    &p_str,
                    &q.to_string(),
                    &format!(" (λ={})", format_statistic(state.lambda)),
                    prob_locale,
                )
            }
            3 => {
                let q = student_t_quantile_honest(state.p_inv, state.df)?;
                quantile_result_cont_text(
                    &p_str,
                    &format_statistic(q),
                    &format!(" (gl={})", format_statistic(state.df)),
                    prob_locale,
                )
            }
            4 => {
                let q = chi_squared_quantile_honest(state.p_inv, state.df)?;
                quantile_result_cont_text(
                    &p_str,
                    &format_statistic(q),
                    &format!(" (gl={})", format_statistic(state.df)),
                    prob_locale,
                )
            }
            5 => {
                let q = f_quantile_honest(state.p_inv, state.df1, state.df2)?;
                quantile_result_cont_text(
                    &p_str,
                    &format_statistic(q),
                    &format!(
                        " (gl1={}, gl2={})",
                        format_statistic(state.df1),
                        format_statistic(state.df2)
                    ),
                    prob_locale,
                )
            }
            6 => {
                let q = geometric_quantile_honest(state.p_inv, state.p)?;
                quantile_result_disc_text(
                    &p_str,
                    &q.to_string(),
                    &format!(" (p={})", format_statistic(state.p)),
                    prob_locale,
                )
            }
            7 => {
                let q = uniform_quantile_honest(state.p_inv, state.ua, state.ub)?;
                quantile_result_cont_text(
                    &p_str,
                    &format_statistic(q),
                    &format!(
                        " (a={}, b={})",
                        format_statistic(state.ua),
                        format_statistic(state.ub)
                    ),
                    prob_locale,
                )
            }
            _ => {
                let q = exponential_quantile_honest(state.p_inv, state.lambda)?;
                quantile_result_cont_text(
                    &p_str,
                    &format_statistic(q),
                    &format!(" (λ={})", format_statistic(state.lambda)),
                    prob_locale,
                )
            }
        })
    })();
    match quantile {
        Ok(text) => {
            // Una línea como siempre: texto + Copiar. Mismo patrón que
            // el caso (botón derecha, texto con el resto medido en
            // vivo): el string largo ya no tapa ni es tapado.
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Auto-ancho como el caso: sin wrap ni "Cop…".
                    if ui
                        .button(egui::RichText::new(prob_copy_label).size(TYPE_XS).strong())
                        .on_hover_text(prob_copy_quantile_hint)
                        .clicked()
                    {
                        ctx.copy_text(text.clone());
                        app.cas_result = prob_quantile_toast.to_string();
                    }
                    ui.add_sized(
                        egui::vec2(ui.available_width().max(40.0), PANEL_BUTTON_H),
                        egui::Label::new(
                            egui::RichText::new(text.as_str())
                                .color(txt_col)
                                .size(TYPE_SM)
                                .strong(),
                        )
                        .truncate(),
                    )
                    .on_hover_text(text.as_str());
                });
            });
            ui.label(
                egui::RichText::new(prob_quantile_note)
                    .color(txt_dim)
                    .size(TYPE_XS),
            )
            .on_hover_text(prob_quantile_note_hint);
            ui.add_space(SPACE_MD);
            match prob_quantile_command(&state) {
                Some(command) => {
                    if ui
                        .add_sized(
                            [prob_row_w, PANEL_BUTTON_H],
                            egui::Button::new(
                                egui::RichText::new(use_quantile_label).size(TYPE_SM),
                            )
                            .rounding(RADIUS_SM),
                        )
                        .on_hover_text(quantile_hint_label)
                        .clicked()
                    {
                        app.input_text = command;
                        app.command_input_focus_requested = true;
                        app.cas_result = prob_quantile_ready_toast.to_string();
                        ctx.request_repaint();
                    }
                }
                None => {
                    ui.add_enabled_ui(false, |ui| {
                        ui.add_sized(
                            [prob_row_w, PANEL_BUTTON_H],
                            egui::Button::new(
                                egui::RichText::new(use_quantile_label).size(TYPE_SM),
                            )
                            .rounding(RADIUS_SM),
                        )
                    })
                    .response
                    .on_hover_text(no_quantile_label);
                }
            }
        }
        Err(error) => {
            ui.label(
                egui::RichText::new(error)
                    .color(current_theme(ctx).danger)
                    .size(TYPE_XS),
            );
        }
    }
    ctx.data_mut(|data| data.insert_temp(id, state));
}

/// Prompt auto de velocidad: texto libre → `trig_speed` honesto.
/// Llamado desde el panel trig (alcanzable); Aplicar parsea o explica.
pub(crate) fn draw_trig_speed_prompt(ui: &mut egui::Ui, app: &mut GrafitoApp) {
    let ctx = ui.ctx().clone();
    let id = egui::Id::new("gc_trig_prompt_state");
    let mut prompt: TrigPromptState = ctx
        .data_mut(|data| data.get_temp::<TrigPromptState>(id))
        .unwrap_or_default();
    let theme = current_theme(&ctx);
    // B-A98: chrome del prompt en el idioma activo (antes consts ES).
    let prompt_locale = app.config_locale();

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = SPACE_SM;
        ui.label(
            egui::RichText::new(trig_prompt_auto_label(prompt_locale))
                .color(theme.text_tertiary)
                .size(TYPE_XS),
        );
        // Ancho exacto: label + campo + botón nunca superan la tarjeta.
        let field_w = trig_prompt_field_w(ui.available_width());
        let resp = ui.add_sized(
            [field_w, TRIG_CONTROL_H],
            egui::TextEdit::singleline(&mut prompt.text)
                .hint_text(trig_prompt_hint_text(prompt_locale)),
        );
        let enter_pressed =
            resp.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
        resp.on_hover_text(trig_prompt_field_tooltip(prompt_locale));
        if ui
            .add_sized(
                [TRIG_APPLY_W, TRIG_CONTROL_H],
                egui::Button::new(
                    egui::RichText::new(trig_prompt_apply_label(prompt_locale)).size(TYPE_SM),
                )
                .rounding(RADIUS_SM),
            )
            .on_hover_text(trig_prompt_apply_tooltip(prompt_locale))
            .clicked()
            || enter_pressed
        {
            match crate::sheet_layout::parse_slider_prompt_for(&prompt.text, prompt_locale) {
                Ok(speed) => {
                    app.trig_speed = speed;
                    prompt.error = None;
                    ctx.request_repaint();
                }
                Err(error) => prompt.error = Some(error),
            }
        }
    });
    if let Some(error) = &prompt.error {
        ui.label(egui::RichText::new(error).color(theme.danger).size(TYPE_XS));
    }
    ctx.data_mut(|data| data.insert_temp(id, prompt));
}

#[cfg(test)]
mod gc_piel_tests {
    use super::{PANEL_BUTTON_H, SHEET_CONTROL_H};
    use crate::sheet_layout::{
        binomial_cdf, binomial_pmf, binomial_quantile_honest, chi_squared_cdf, chi_squared_pdf,
        chi_squared_quantile_honest, f_distribution_cdf, f_distribution_pdf, f_quantile_honest,
        geometric_cdf, geometric_pmf, geometric_quantile_honest, normal_cdf, normal_pdf,
        normal_quantile_honest, parse_series_scalar, parse_slider_prompt, plot_df_or_fuera_de_cota,
        poisson_cdf, poisson_pmf, poisson_quantile_honest, sheet_cell_width_for, sheet_col_label,
        sheet_nav_metrics, stat_tile_cols, student_t_cdf, student_t_pdf, student_t_quantile_honest,
        uniform_cdf, uniform_pdf, uniform_quantile_honest, visible_sheet_cols,
        wc_exact_integral_command_text, wc_riemann_command_text, wc_study_command_text,
        wc_taylor_command_text, wc_taylor_remainder_line, MAX_BINOMIAL_N, MAX_GEOMETRIC_K,
        MAX_PANEL_DF, SHEET_CELL_H, SHEET_VIEW_COLS, SHEET_VIEW_ROWS,
    };
    use grafito_core::Document;
    use grafito_ui::tokens::{HIT_TARGET_MIN, SPACE_XS};
    use std::collections::BTreeMap;

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn panel_helpers_respetan_tokens_y_clamps() {
        // Botones ≥ WCAG 2.5.8 (24 px) y celdas con alto legible.
        assert!(PANEL_BUTTON_H >= HIT_TARGET_MIN);
        assert!(SHEET_CELL_H >= HIT_TARGET_MIN);
        // Controles de la hoja (chip/flechas/Ir): mismo alto, ≥ hit target.
        assert!(SHEET_CONTROL_H >= HIT_TARGET_MIN);
        // Celdas: 6 columnas repartidas, clamp 36..=96. A 300-360 px de panel
        // las 6 columnas + rótulo + espaciado entran sin scroll horizontal.
        // Grilla adaptativa: columnas visibles según ancho real (3..=6) y
        // ancho de celda que nunca desborda el contenido disponible.
        assert_eq!(visible_sheet_cols(252.0), 4); // drawer compacto medido
        assert_eq!(visible_sheet_cols(336.0), 6); // panel ancho
        assert_eq!(visible_sheet_cols(120.0), 3); // piso
        assert_eq!(visible_sheet_cols(10_000.0), SHEET_VIEW_COLS);
        for width in [252.0_f32, 300.0, 336.0, 180.0] {
            let cols = visible_sheet_cols(width);
            let cell_w = sheet_cell_width_for(width, cols);
            assert!(
                (36.0..=96.0).contains(&cell_w),
                "ancho de celda fuera de rango: {cell_w}"
            );
            let total = 32.0 + cols as f32 * (cell_w + grafito_ui::tokens::SPACE_XS);
            assert!(
                total <= width + 0.5,
                "ancho {width}: grilla {total} desborda con {cols} columnas"
            );
        }
    }

    #[test]
    fn sheet_nav_metrics_no_desbordan() {
        // F13.1: las dos filas de navegación entran sin desbordar de 112 a
        // 520 px (el panel mínimo real da ~132 px de contenido).
        for width in [
            112.0_f32, 132.0, 160.0, 180.0, 220.0, 252.0, 300.0, 360.0, 520.0,
        ] {
            let m = sheet_nav_metrics(width);
            let row1 = m.chip_w + 4.0 * m.arrow_w + 4.0 * SPACE_XS;
            let row2 = m.field_w + 2.0 * m.action_w + 2.0 * SPACE_XS;
            assert!(row1 <= width + 0.5, "fila 1 {row1} desborda {width}");
            assert!(row2 <= width + 0.5, "fila 2 {row2} desborda {width}");
            assert!(m.arrow_w >= HIT_TARGET_MIN, "flecha {width}: {}", m.arrow_w);
            assert!(m.field_w >= HIT_TARGET_MIN, "campo {width}: {}", m.field_w);
            assert!(
                m.action_w >= HIT_TARGET_MIN,
                "acción {width}: {}",
                m.action_w
            );
        }
        // En un panel angosto el chip cede antes que el hit target.
        assert!(sheet_nav_metrics(164.0).chip_w < sheet_nav_metrics(360.0).chip_w);
        assert!(sheet_nav_metrics(132.0).chip_w < 44.0);
        assert_eq!(sheet_nav_metrics(10_000.0).chip_w, 64.0);
    }

    #[test]
    fn stat_tile_cols_nunca_desborda() {
        assert_eq!(stat_tile_cols(10_000.0), 3);
        assert_eq!(stat_tile_cols(300.0), 3);
        assert_eq!(stat_tile_cols(200.0), 2);
        assert_eq!(stat_tile_cols(132.0), 1);
        assert_eq!(stat_tile_cols(1.0), 1);
    }

    #[test]
    fn normal_standard_values_are_honest() {
        let pdf = normal_pdf(0.0, 0.0, 1.0).expect("normal válida");
        assert!(
            (pdf - 0.398_942_280_401_432_7).abs() < 1e-6,
            "pdf(0) = {pdf}"
        );
        let cdf = normal_cdf(0.0, 0.0, 1.0).expect("normal válida");
        assert!((cdf - 0.5).abs() < 1e-6, "cdf(0) = {cdf}");
        let tail = normal_cdf(1.959_963_984_540_054, 0.0, 1.0).expect("cola 97.5%");
        assert!((tail - 0.975).abs() < 1e-3, "cola = {tail}");
    }

    #[test]
    fn normal_rejects_bad_sigma_and_non_finite() {
        assert!(normal_pdf(0.0, 0.0, 0.0).is_err());
        assert!(normal_pdf(0.0, 0.0, -1.0).is_err());
        assert!(normal_cdf(f64::NAN, 0.0, 1.0).is_err());
        assert!(normal_cdf(0.0, 0.0, f64::INFINITY).is_err());
    }

    #[test]
    fn binomial_fair_coin_matches_textbook() {
        // n=10, p=0.5, k=5 → 252/1024.
        let pmf = binomial_pmf(5, 10, 0.5).expect("binomial válida");
        assert!((pmf - 0.246_093_75).abs() < 1e-9, "pmf = {pmf}");
        let cdf = binomial_cdf(5, 10, 0.5).expect("cdf válida");
        assert!((cdf - 0.623_046_875).abs() < 1e-9, "cdf = {cdf}");
        assert_eq!(binomial_pmf(0, 5, 0.0).expect("p=0"), 1.0);
        assert_eq!(binomial_pmf(1, 5, 0.0).expect("p=0"), 0.0);
        assert_eq!(binomial_pmf(5, 5, 1.0).expect("p=1"), 1.0);
    }

    #[test]
    fn binomial_rejects_out_of_bounds() {
        assert!(binomial_pmf(6, 5, 0.5).is_err());
        assert!(binomial_pmf(1, 5, 1.5).is_err());
        assert!(binomial_pmf(1, MAX_BINOMIAL_N + 1, 0.5).is_err());
        assert!(binomial_cdf(6, 5, 0.5).is_err());
    }

    #[test]
    fn poisson_matches_textbook() {
        // λ=3, k=2 → 9/(2e³) ≈ 0.2240.
        let pmf = poisson_pmf(2, 3.0).expect("poisson válida");
        assert!((pmf - 0.224_041_807_655_387_75).abs() < 1e-9, "pmf = {pmf}");
        let cdf = poisson_cdf(2, 3.0).expect("cdf válida");
        assert!((cdf - 0.423_190_081_126_843_53).abs() < 1e-9, "cdf = {cdf}");
        assert!((poisson_pmf(0, 1.0).expect("k=0") - 0.367_879_441_171_442_33).abs() < 1e-9);
    }

    #[test]
    fn poisson_rejects_bad_lambda_and_caps() {
        assert!(poisson_pmf(1, 0.0).is_err());
        assert!(poisson_pmf(1, -2.0).is_err());
        assert!(poisson_pmf(1, f64::NAN).is_err());
        assert!(poisson_pmf(1, 1e9).is_err());
    }

    #[test]
    fn f3b_area_coincide_con_comando_y_cuantil_consistente() {
        // Normal: cuantil 97.5% ≈ 1.96 y su acumulada vuelve a 0.975.
        let q = normal_quantile_honest(0.975, 0.0, 1.0).expect("cuantil normal");
        assert!((q - 1.959_963_984_540_054).abs() < 1e-3, "q = {q}");
        let area = normal_cdf(q, 0.0, 1.0).expect("área en el cuantil");
        assert!((area - 0.975).abs() < 1e-3, "área = {area}");
        // Binomial: el panel (motor estadístico) coincide con la cuenta
        // combinatoria del comando `Binomial[10, 0.5, 5]` = 252/1024.
        let pmf = binomial_pmf(5, 10, 0.5).expect("pmf panel");
        assert!((pmf - 252.0 / 1024.0).abs() < 1e-9, "pmf = {pmf}");
        let q_bin = binomial_quantile_honest(0.5, 10, 0.5).expect("cuantil binomial");
        let cdf_q = binomial_cdf(q_bin, 10, 0.5).expect("cdf en cuantil");
        assert!(cdf_q >= 0.5, "cdf({q_bin}) = {cdf_q}");
        if q_bin > 0 {
            let cdf_prev = binomial_cdf(q_bin - 1, 10, 0.5).expect("cdf previa");
            assert!(cdf_prev < 0.5, "cdf({}) = {cdf_prev}", q_bin - 1);
        }
        // Poisson: coincide con `Poisson[3, 2]` = 9/(2e³) y el cuantil es
        // el menor k con acumulada ≥ p.
        let pmf_p = poisson_pmf(2, 3.0).expect("pmf poisson");
        assert!(
            (pmf_p - 9.0 / (2.0 * 3.0_f64.exp())).abs() < 1e-9,
            "pmf = {pmf_p}"
        );
        let q_pois = poisson_quantile_honest(0.5, 3.0).expect("cuantil poisson");
        let cdf_q = poisson_cdf(q_pois, 3.0).expect("cdf en cuantil");
        assert!(cdf_q >= 0.5, "cdf({q_pois}) = {cdf_q}");
        if q_pois > 0 {
            let cdf_prev = poisson_cdf(q_pois - 1, 3.0).expect("cdf previa");
            assert!(cdf_prev < 0.5, "cdf({}) = {cdf_prev}", q_pois - 1);
        }
        assert!(normal_quantile_honest(0.0, 0.0, 1.0).is_err());
        assert!(binomial_quantile_honest(1.0, 10, 0.5).is_err());
        assert!(poisson_quantile_honest(0.5, -1.0).is_err());
    }

    #[test]
    fn c2_student_t_matches_textbook() {
        // t(10): pdf(0) ≈ 0.3891, cdf(0) = 0.5, cuantil 97.5% ≈ 2.228.
        let pdf = student_t_pdf(0.0, 10.0).expect("t válida");
        assert!((pdf - 0.389_11).abs() < 1e-4, "pdf = {pdf}");
        let cdf = student_t_cdf(0.0, 10.0).expect("cdf válida");
        assert!((cdf - 0.5).abs() < 1e-9, "cdf = {cdf}");
        let q = student_t_quantile_honest(0.975, 10.0).expect("cuantil t");
        assert!((q - 2.228_14).abs() < 1e-3, "q = {q}");
        let area = student_t_cdf(q, 10.0).expect("área en el cuantil");
        assert!((area - 0.975).abs() < 1e-3, "área = {area}");
    }

    #[test]
    fn c2_chi_squared_matches_textbook() {
        // χ²(5): pdf(5) ≈ 0.1222, cuantil 95% ≈ 11.0705.
        let pdf = chi_squared_pdf(5.0, 5.0).expect("chi² válida");
        assert!((pdf - 0.122_2).abs() < 1e-3, "pdf = {pdf}");
        let q = chi_squared_quantile_honest(0.95, 5.0).expect("cuantil chi²");
        assert!((q - 11.070_5).abs() < 1e-2, "q = {q}");
        let area = chi_squared_cdf(q, 5.0).expect("área en el cuantil");
        assert!((area - 0.95).abs() < 1e-3, "área = {area}");
    }

    #[test]
    fn c2_f_distribution_matches_textbook() {
        // F(5,10): cuantil 95% ≈ 3.3258.
        let q = f_quantile_honest(0.95, 5.0, 10.0).expect("cuantil F");
        assert!((q - 3.325_8).abs() < 1e-2, "q = {q}");
        let area = f_distribution_cdf(q, 5.0, 10.0).expect("área en el cuantil");
        assert!((area - 0.95).abs() < 2e-3, "área = {area}");
        let pdf = f_distribution_pdf(1.0, 5.0, 10.0).expect("pdf F");
        assert!(pdf > 0.0 && pdf < 2.0, "pdf = {pdf}");
    }

    #[test]
    fn c2_continuous_panels_reject_bad_params() {
        assert!(student_t_pdf(0.0, 0.0).is_err());
        assert!(student_t_pdf(0.0, -2.0).is_err());
        assert!(student_t_pdf(f64::NAN, 10.0).is_err());
        assert!(student_t_pdf(0.0, MAX_PANEL_DF + 1.0).is_err());
        assert!(student_t_quantile_honest(0.0, 10.0).is_err());
        assert!(student_t_quantile_honest(1.0, 10.0).is_err());
        assert!(chi_squared_pdf(1.0, 0.0).is_err());
        assert!(chi_squared_quantile_honest(0.95, f64::NAN).is_err());
        assert!(f_distribution_pdf(1.0, 5.0, 0.0).is_err());
        assert!(f_distribution_cdf(1.0, -1.0, 10.0).is_err());
        assert!(f_quantile_honest(1.5, 5.0, 10.0).is_err());
    }

    #[test]
    fn we_geometric_matches_textbook_and_rejects() {
        // p=0.5: P(X=0)=0.5, P(X≤1)=0.75; cuantil 0.5 → 0, cuantil 0.9 → 3.
        let pmf = geometric_pmf(0, 0.5).expect("geométrica válida");
        assert!((pmf - 0.5).abs() < 1e-12, "pmf = {pmf}");
        let cdf = geometric_cdf(1, 0.5).expect("cdf válida");
        assert!((cdf - 0.75).abs() < 1e-12, "cdf = {cdf}");
        assert_eq!(geometric_quantile_honest(0.5, 0.5).expect("cuantil"), 0);
        assert_eq!(geometric_quantile_honest(0.9, 0.5).expect("cuantil"), 3);
        // p=1 degenera honesto: todo el peso en k=0.
        assert_eq!(geometric_pmf(0, 1.0).expect("p=1"), 1.0);
        assert_eq!(geometric_cdf(0, 1.0).expect("p=1"), 1.0);
        assert!(geometric_pmf(0, 0.0).is_err());
        assert!(geometric_pmf(0, 1.5).is_err());
        assert!(geometric_pmf(0, f64::NAN).is_err());
        assert!(geometric_pmf(MAX_GEOMETRIC_K + 1, 0.5).is_err());
        assert!(geometric_quantile_honest(0.0, 0.5).is_err());
        assert!(geometric_quantile_honest(0.5, 0.0).is_err());
    }

    #[test]
    fn we_uniform_matches_textbook_and_rejects() {
        // U(0,10): f(5)=0.1, P(X≤5)=0.5, cuantil 0.25 → 2.5.
        let pdf = uniform_pdf(5.0, 0.0, 10.0).expect("uniforme válida");
        assert!((pdf - 0.1).abs() < 1e-12, "pdf = {pdf}");
        assert_eq!(uniform_pdf(-1.0, 0.0, 10.0).expect("fuera"), 0.0);
        let cdf = uniform_cdf(5.0, 0.0, 10.0).expect("cdf válida");
        assert!((cdf - 0.5).abs() < 1e-12, "cdf = {cdf}");
        let q = uniform_quantile_honest(0.25, 0.0, 10.0).expect("cuantil");
        assert!((q - 2.5).abs() < 1e-12, "q = {q}");
        // Round-trip: la acumulada en el cuantil vuelve a p.
        let back = uniform_cdf(q, 0.0, 10.0).expect("área en el cuantil");
        assert!((back - 0.25).abs() < 1e-12, "área = {back}");
        assert!(uniform_pdf(5.0, 10.0, 0.0).is_err());
        assert!(uniform_pdf(5.0, 3.0, 3.0).is_err());
        assert!(uniform_cdf(f64::NAN, 0.0, 1.0).is_err());
        assert!(uniform_quantile_honest(1.0, 0.0, 1.0).is_err());
        assert!(uniform_quantile_honest(0.5, 2.0, 1.0).is_err());
    }

    #[test]
    fn wa_plot_df_fuera_de_cota_no_dibuja_curva() {
        // W-A red: `df = MAX_PANEL_DF + 1` → mensaje con "cota", no curva.
        // Antes el caller hacía `clamp` a 500 y dibujaba la curva falsa.
        let err = plot_df_or_fuera_de_cota(MAX_PANEL_DF + 1.0, "gl").expect_err("cota");
        assert!(err.contains("cota"), "mensaje honesto: {err}");
        let err1 = plot_df_or_fuera_de_cota(MAX_PANEL_DF + 1.0, "gl1").expect_err("cota");
        assert!(err1.contains("cota"), "mensaje honesto: {err1}");
        assert!(plot_df_or_fuera_de_cota(MAX_PANEL_DF, "gl").is_ok());
        assert!(plot_df_or_fuera_de_cota(0.0, "gl").is_err());
        assert!(plot_df_or_fuera_de_cota(f64::NAN, "gl").is_err());
    }

    #[test]
    fn c2_series_scalar_parses_spanish_decimals() {
        assert_eq!(parse_series_scalar("1,5", "inicio").expect("coma"), 1.5);
        assert_eq!(parse_series_scalar(" -2 ", "paso").expect("espacios"), -2.0);
        assert!(parse_series_scalar("abc", "inicio").is_err());
        assert!(parse_series_scalar("", "paso").is_err());
        assert!(parse_series_scalar("inf", "inicio").is_err());
    }

    #[test]
    fn f3b_sheet_view_respeta_presupuestos_del_core() {
        const { assert!(Document::MAX_SPREADSHEET_ROWS == 400) };
        const { assert!(Document::MAX_SPREADSHEET_COLS == 400) };
        const { assert!(Document::MAX_SPREADSHEET_RECOMPUTE_CELLS == 10_000) };
        const { assert!(SHEET_VIEW_COLS * SHEET_VIEW_ROWS <= Document::MAX_SPREADSHEET_RECOMPUTE_CELLS) };
        assert_eq!(sheet_col_label(0), "A");
        assert_eq!(sheet_col_label(5), "F");
        assert_eq!(sheet_col_label(26), "AA");
    }

    #[test]
    fn slider_prompt_parses_numbers_words_and_turns() {
        assert_eq!(parse_slider_prompt("1.5").expect("número"), 1.5);
        assert_eq!(parse_slider_prompt("medio").expect("palabra"), 0.5);
        assert_eq!(parse_slider_prompt("RÁPIDO").expect("mayúscula"), 2.0);
        let turns = parse_slider_prompt("2 vueltas en 10 s").expect("vueltas");
        assert!(
            (turns - 1.256_637_061_435_917_2).abs() < 1e-9,
            "velocidad = {turns}"
        );
        assert!(parse_slider_prompt("").is_err());
        assert!(parse_slider_prompt("100").is_err());
        assert!(parse_slider_prompt("verde").is_err());
        assert!(parse_slider_prompt("100 vueltas en 1 s").is_err());
    }

    #[test]
    fn trig_b_a161_prompt_errors_salen_de_trig_msg() {
        // B-A161: los 6 `Err` de `parse_slider_prompt`
        // (`sheet_layout.rs:2409-2452`, mostrados en rojo en
        // `draw_trig_speed_prompt :6914-6916`) salen de
        // `trig.prompt_err_*` via `parse_slider_prompt_for` + `prompt_locale`.
        // Sin egui: pines + wiring + blindaje por `include_str!`.
        use grafito_ui::i18n::{t, Locale};
        // ES pineado a los literales previos (concat a proposito: el
        // blindaje usa `include_str!` e incluye este test, precedente
        // B-A89/B-A155/B-A157).
        assert_eq!(
            t("trig.prompt_err_empty", Locale::Es),
            [
                "Escribí una velocidad: 1.5, «medio» o ",
                "«2 vueltas en 10 s»"
            ]
            .concat()
        );
        assert_eq!(
            t("trig.prompt_err_finite", Locale::Es),
            ["La velocidad debe ", "ser finita"].concat()
        );
        assert_eq!(
            t("trig.prompt_err_range", Locale::Es),
            [
                "La velocidad vive en ±6 rad/s ",
                "(mové el slider si querés más)"
            ]
            .concat()
        );
        assert_eq!(
            t("trig.prompt_err_seconds", Locale::Es),
            ["Los segundos deben ", "ser mayores que 0"].concat()
        );
        assert_eq!(
            t("trig.prompt_err_too_fast", Locale::Es),
            [
                "Eso da {speed} rad/s: más de 6 rad/s ",
                "no se sigue a ojo (bajá vueltas o subí segundos)"
            ]
            .concat()
        );
        assert_eq!(
            t("trig.prompt_err_unknown", Locale::Es),
            ["No entendí: probá 1.5, «medio» o ", "«2 vueltas en 10 s»"].concat()
        );
        assert_eq!(
            t("trig.prompt_err_finite", Locale::En),
            "Speed must be finite"
        );
        assert_eq!(
            t("trig.prompt_err_unknown", Locale::De),
            "Nicht verstanden: versuch 1.5, „mittel“ oder „2 Runden in 10 s“"
        );
        // Helper puro: Es espeja el canonico, resto localiza + interpola.
        assert_eq!(
            crate::sheet_layout::parse_slider_prompt_for("", Locale::Es).expect_err("vacio"),
            [
                "Escribí una velocidad: 1.5, «medio» o ",
                "«2 vueltas en 10 s»"
            ]
            .concat()
        );
        let fast_en =
            crate::sheet_layout::parse_slider_prompt_for("100 vueltas en 1 s", Locale::En)
                .expect_err("rapido");
        assert!(fast_en.contains("628.32"), "interpola {{speed}}: {fast_en}");
        // Wiring: el draw usa `_for` + `prompt_locale`.
        let source = include_str!("panels.rs");
        for pin in [
            "parse_slider_prompt_for(&prompt.text, prompt_locale)",
            "trig_b_a161_prompt_errors_salen_de_trig_msg",
        ] {
            assert!(source.contains(pin), "trig B-A161 cablea {pin}");
        }
        // Blindaje: la llamada sin locale ya no esta en el draw.
        // (Fragmento con contexto call-site: `include_str!` incluye este
        // test y `parse_slider_prompt_for(` contiene el prefijo viejo.)
        for needle in [
            ["match parse_slider_prompt(&prompt", ".text)"].concat(),
            ["parse_local_xy_table, parse_slider", "_prompt,"].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "trig sin llamada sin-locale en prod (B-A161): {needle}"
            );
        }
    }

    #[test]
    fn trig_b_a162_lecturas_quedan_crudas_intencional() {
        // B-A162 (barrido final saga trig): el draw usa las 5 lecturas en
        // crudo sin `locale` a propósito — notación SI/matemática idéntica
        // en los 6 idiomas (unidades `rad/s`/`rad`, `z`/`θ`/`|z|`/`arg`,
        // fórmula `z(t)`, chip `{name}(t)`; precedentes B-A114/B-A160/B-A98).
        // Lo localizable ya sale de `trig.*` (secciones B-A96, modos B-A97,
        // descripciones B-A98, errores B-A161). Sin egui: wiring + blindaje
        // por `include_str!` (fragmentos porque el `include_str!` incluye
        // este test, precedente B-A89).
        let source = include_str!("panels.rs");
        for pin in [
            "trig_speed_text(app.trig_speed)",
            "trig_angle_text(app.trig_angle)",
            "trig_complex_text(t)",
            "trig_function_chip_label(spec.name)",
            "z(t) = cos(t)",
        ] {
            assert!(
                source.contains(pin),
                "trig B-A162 usa lectura cruda en draw: {pin}"
            );
        }
        // Blindaje: las lecturas NO pasan por `trig_t()`/`t()` ni por key
        // `trig.speed_*/angle_*/complex_*` (0 keys B-A162).
        for needle in [
            ["trig_t(\"trig.spe", "ed_text\")"].concat(),
            ["trig_t(\"trig.ang", "le_text\")"].concat(),
            ["trig_t(\"trig.comp", "lex_text\")"].concat(),
            ["t(\"trig.speed", "_text\")"].concat(),
            ["t(\"trig.angle", "_text\")"].concat(),
            ["trig_msg(\"spe", "ed_text\")"].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "trig sin display-localizar lecturas: {needle}"
            );
        }
    }

    #[test]
    fn wc_command_texts_cablean_a_comandos_reales() {
        // T1: FunctionStudy[f] existe en el registry (verificado en
        // grafito-command); acá solo se pinnea el texto que arma el botón.
        assert_eq!(
            wc_study_command_text("f").as_deref(),
            Some("FunctionStudy[f]")
        );
        assert_eq!(wc_study_command_text("  ").as_deref(), None);
        // T2: RiemannSum con n redondeado + exacta definida.
        assert_eq!(
            wc_riemann_command_text("x^2", "0", "1", 100.0, "trapecio").as_deref(),
            Some("RiemannSum[x^2, x, 0, 1, 100, trapecio]")
        );
        assert_eq!(
            wc_riemann_command_text("x^2", "0", "1", 99.6, "simpson").as_deref(),
            Some("RiemannSum[x^2, x, 0, 1, 100, simpson]")
        );
        assert_eq!(
            wc_riemann_command_text("", "0", "1", 100.0, "trapecio").as_deref(),
            None
        );
        assert_eq!(
            wc_riemann_command_text("x^2", "0", "1", f32::NAN, "trapecio").as_deref(),
            None
        );
        assert_eq!(
            wc_riemann_command_text("x^2", "0", "1", 0.4, "trapecio").as_deref(),
            None
        );
        assert_eq!(
            wc_exact_integral_command_text("x^2", "0", "1").as_deref(),
            Some("Integral[x^2, x, 0, 1]")
        );
        assert_eq!(
            wc_exact_integral_command_text("", "0", "1").as_deref(),
            None
        );
        // T3: Taylor[expr, x, centro, orden] existe; orden del slider 1..=10.
        assert_eq!(
            wc_taylor_command_text("sin(x)", "0", 5, "0.5").as_deref(),
            Some("Taylor[sin(x), x, 0, 5]")
        );
        assert_eq!(
            wc_taylor_command_text("sin(x)", "0", 0, "0.5").as_deref(),
            None
        );
        assert_eq!(
            wc_taylor_command_text("sin(x)", "0", 11, "0.5").as_deref(),
            None
        );
        assert_eq!(
            wc_taylor_command_text("sin(x)", "c", 5, "0.5").as_deref(),
            None
        );
        assert_eq!(wc_taylor_command_text("", "0", 5, "0.5").as_deref(), None);
    }

    #[test]
    fn wc_taylor_remainder_line_muestra_resto_y_falla_honesto() {
        let vars = BTreeMap::new();
        let line = wc_taylor_remainder_line("sin(x)", "0", 5, "0.5", &vars).expect("resto");
        assert!(line.contains("P5(0.5)"), "rotula orden y punto: {line}");
        assert!(line.contains("resto"), "muestra resto: {line}");
        assert!(line.contains("sig."), "muestra término siguiente: {line}");
        // Expresión inválida o punto no numérico: None honesto.
        assert!(wc_taylor_remainder_line("[[[", "0", 5, "0.5", &vars).is_none());
        assert!(wc_taylor_remainder_line("sin(x)", "0", 5, "c", &vars).is_none());
    }
}
#[cfg(test)]
mod coverage_sweep_panels_pure {
    use super::*;
    use crate::sheet_layout::{
        inspector_identity_tooltip, inspector_label_text, parse_statistics_input,
        prob_distribution_is_discrete, prob_tail_values, series_preview, sheet_col_label,
        statistics_assistant_prompt, statistics_boxplot_command, statistics_data_preview,
        statistics_histogram_command, statistics_inline_list, statistics_mean_command,
        statistics_summary, summarize_range, SheetViewState, MAX_SUMMARY_CELLS,
        PANEL_DATA_MAX_WIDTH, PROB_QUANTILE_ALPHAS,
    };
    use std::collections::BTreeMap;
    #[test]
    fn regression_fit_kind_sale_de_helper() {
        // B-A146: el titular del panel Regresión sale de `fit_kind_label_for`
        // (Es = canónico `FitKind::display_name`), no del display crudo.
        // Sin egui: wiring + blindaje por `include_str!` (precedente B-A143).
        let source = include_str!("panels.rs");
        for pin in [
            "fit_kind_label_for(",
            "regression_fit_headline(",
            "use crate::algebra::fit_kind_label_for;",
        ] {
            assert!(source.contains(pin), "regresión B-A146 cablea {pin}");
        }
        assert_eq!(
            source
                .matches(["fit.kind.display", "_name()"].concat().as_str())
                .count(),
            0,
            "el titular de regresión sale del helper, no del display canónico (B-A146)"
        );
    }
    #[test]
    fn barrido_parse_statistics_input() {
        assert_eq!(
            parse_statistics_input("1, 2, 3").expect("nums"),
            vec![1.0, 2.0, 3.0]
        );
        assert!(parse_statistics_input("").expect("vacío").is_empty());
        assert!(parse_statistics_input("1, mal").is_err());
        assert!(parse_statistics_input("1,,2").is_err());
        assert!(parse_statistics_input("inf").is_err());
    }
    #[test]
    fn barrido_statistics_summary_pinnea() {
        let s = statistics_summary(&[1.0, 2.0, 3.0, 4.0]).expect("resumen");
        assert!((s.mean - 2.5).abs() < 1e-9, "media 2.5, fue {}", s.mean);
        assert!((s.median - 2.5).abs() < 1e-9);
        assert!((s.minimum - 1.0).abs() < 1e-12 && (s.maximum - 4.0).abs() < 1e-12);
        assert!(statistics_summary(&[]).is_err());
        assert!(statistics_summary(&[1.0, f64::NAN]).is_err());
        let cero = statistics_summary(&[0.0, 0.0]).expect("ceros");
        assert_eq!(cero.mean, 0.0);
    }
    #[test]
    fn barrido_inspector_textos_puros() {
        assert_eq!(
            wc_study_command_text("f").as_deref(),
            Some("FunctionStudy[f]")
        );
        assert_eq!(wc_study_command_text("  "), None);
        assert_eq!(
            wc_riemann_command_text("x^2", "0", "1", 10.0, "simpson").as_deref(),
            Some("RiemannSum[x^2, x, 0, 1, 10, simpson]")
        );
        assert_eq!(
            wc_riemann_command_text("", "0", "1", 10.0, "simpson").as_deref(),
            None
        );
        assert_eq!(
            wc_riemann_command_text("x", "0", "1", 0.0, "simpson").as_deref(),
            None
        );
        assert_eq!(
            wc_exact_integral_command_text("x^2", "0", "1").as_deref(),
            Some("Integral[x^2, x, 0, 1]")
        );
        assert_eq!(
            wc_exact_integral_command_text("", "0", "1").as_deref(),
            None
        );
        assert_eq!(
            wc_taylor_command_text("sin(x)", "0", 5, "0.5").as_deref(),
            Some("Taylor[sin(x), x, 0, 5]")
        );
        assert_eq!(
            wc_taylor_command_text("sin(x)", "0", 0, "0.5").as_deref(),
            None
        );
        let vars = BTreeMap::new();
        let line = wc_taylor_remainder_line("sin(x)", "0", 5, "0.5", &vars).expect("resto");
        assert!(line.contains("P5(0.5)"), "{line}");
        assert!(wc_taylor_remainder_line("[[[", "0", 5, "0.5", &vars).is_none());
        assert!(inspector_label_text("MiEtiqueta").is_some());
        assert!(inspector_label_text("   ").is_none());
        assert!(!inspector_identity_tooltip("Título", "f", true).is_empty());
    }
    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn barrido_datos_prob_asistente_y_comandos() {
        // Preview honesto: 24 valores + conteo, nunca trunca en silencio.
        assert_eq!(statistics_data_preview(&[]), "");
        assert!(statistics_data_preview(&[1.0, 2.0]).contains("1.000000"));
        let big: Vec<f64> = (0..30).map(|v| v as f64).collect();
        let preview = statistics_data_preview(&big);
        assert!(preview.contains("…y 6 más (N=30)"), "{preview}");
        // Prompt asistente: vacío honesto vs N + herramientas.
        let empty = statistics_assistant_prompt(&[]);
        assert!(empty.contains("vacía"), "{empty}");
        let prompt = statistics_assistant_prompt(&[3.0, 1.0, 2.0]);
        assert!(prompt.contains("3 datos"), "{prompt}");
        assert!(prompt.contains("Sort"), "{prompt}");
        assert!(prompt.contains("Histogram"), "{prompt}");
        assert!(prompt.contains("TTest"), "{prompt}");
        // Comandos inline: hasta 200 valores, más → None honesto.
        assert_eq!(
            statistics_histogram_command(&[1.0, 2.0]).as_deref(),
            Some("Histogram[{1, 2}, 10]")
        );
        assert_eq!(
            statistics_boxplot_command(&[1.0, 2.0]).as_deref(),
            Some("BoxPlot[{1, 2}]")
        );
        assert_eq!(
            statistics_mean_command(&[1.0, 2.0]).as_deref(),
            Some("Mean[{1, 2}]")
        );
        assert!(statistics_histogram_command(&[]).is_none());
        assert!(statistics_inline_list(&[]).is_none());
        let huge: Vec<f64> = vec![1.0; 201];
        assert!(statistics_inline_list(&huge).is_none());
        assert!(statistics_histogram_command(&huge).is_none());
        // Panel acotado escandinavo: 360 máx, sin muro en 1080p.
        assert_eq!(PANEL_DATA_MAX_WIDTH, 360.0);
        assert!(PANEL_DATA_MAX_WIDTH % 4.0 == 0.0);
        // Vista previa de serie v2: mismo motor que Aplicar.
        let (count, first) = series_preview("A1:A8", "1", "1", false).expect("lineal válida");
        assert_eq!(count, 8);
        assert_eq!(
            first,
            vec!["1".to_string(), "2".to_string(), "3".to_string()]
        );
        let (count_g, first_g) =
            series_preview("A1:A4", "2", "3", true).expect("geométrica válida");
        assert_eq!(count_g, 4);
        assert_eq!(
            first_g,
            vec!["2".to_string(), "6".to_string(), "18".to_string()]
        );
        assert!(series_preview("ZZZ", "1", "1", false).is_none());
        assert!(series_preview("A1:A3", "mal", "1", false).is_none());
        assert!(series_preview("A1:A3", "1", "inf", false).is_none());
    }
    #[test]
    fn grilla_exportacion_cabe_en_3_columnas() {
        // Cuadrícula 3×3 del panel Herramientas (celda medida en captura:
        // 73 px en el panel por defecto). Los labels cortos deben entrar sin
        // truncarse ni envolver; el detalle vive en el tooltip.
        let labels = [
            "SVG",
            "PNG",
            "TikZ",
            "PDF",
            "Copiar SVG",
            "Copiar PNG",
            "MathML",
            "TikZ-eje",
            "HTML",
        ];
        assert_eq!(labels.len(), 9, "la grilla es 3×3");
        let cell_w = 73.0_f32;
        for label in labels {
            let width = label.chars().count() as f32 * 6.4;
            assert!(
                width <= cell_w - 8.0,
                "«{label}» ({width:.0} px) no cabe en la celda de {cell_w} px"
            );
        }
    }
    #[test]
    fn barrido_prob_momentos_y_puente_motor() {
        let mut state = ProbabilityPanelState::default();
        // Normal por defecto: media 0, varianza 1.
        assert_eq!(distribution_moments(&state), (Some(0.0), Some(1.0)));
        assert_eq!(
            prob_case_command(&state).as_deref(),
            Some("Normal[0, 1, 0]")
        );
        assert_eq!(
            prob_quantile_command(&state).as_deref(),
            Some("InverseNormal[0.95, 0, 1]")
        );
        // t con gl=1: media y varianza no definidas (— honesto).
        state.dist = 3;
        state.df = 1.0;
        assert_eq!(distribution_moments(&state), (None, None));
        assert!(
            prob_case_command(&state).is_none(),
            "t aún sin comando forward"
        );
        state.df = 10.0;
        assert_eq!(distribution_moments(&state), (Some(0.0), Some(1.25)));
        assert_eq!(
            prob_quantile_command(&state).as_deref(),
            Some("InverseT[0.95, 10]")
        );
        // F con gl2≤4: media/varianza no definidas según el gl.
        state.dist = 5;
        state.df1 = 5.0;
        state.df2 = 4.0;
        assert_eq!(distribution_moments(&state), (Some(2.0), None));
        // Geométrica: sin comando forward ni cuantil Inverse.
        state.dist = 6;
        state.p = 0.5;
        assert_eq!(distribution_moments(&state), (Some(1.0), Some(2.0)));
        assert!(prob_case_command(&state).is_none());
        assert!(prob_quantile_command(&state).is_none());
        // Uniforme con a ≥ b: nada definido, ningún valor inventado.
        state.dist = 7;
        state.ua = 2.0;
        state.ub = 1.0;
        assert_eq!(distribution_moments(&state), (None, None));
        // Discretas vs continuas (rótulo del selector).
        assert!(prob_distribution_is_discrete(1));
        assert!(prob_distribution_is_discrete(2));
        assert!(prob_distribution_is_discrete(6));
        assert!(!prob_distribution_is_discrete(0));
        assert!(!prob_distribution_is_discrete(5));
        // "Ir a": el parser del cerebro acepta la celda canónica.
        assert_eq!(
            grafito_core::symbolic::series::parse_cell_reference("C3"),
            Some((2, 2))
        );
        assert!(grafito_core::symbolic::series::parse_cell_reference("ZZZ").is_none());
        // Navegación por celda: K10 cae en la página 1..14 con 4 columnas (I1:L14).
        let mut view = SheetViewState::default();
        view.focus_cell(9, 10, 4);
        assert_eq!(view.origin_row, 0);
        assert_eq!(view.origin_col, 8);
        assert_eq!(view.window_label(4), "I1:L14");
        // Con 6 columnas visibles la columna de página cambia (K10 → G1:L14).
        view.focus_cell(9, 10, 6);
        assert_eq!(view.origin_col, 6);
        assert_eq!(view.window_label(6), "G1:L14");
        // Cota dura del borde de la hoja 400×400 (clamp a la ventana de 8 cols).
        view.focus_cell(399, 399, 4);
        assert_eq!(view.origin_row, 386);
        assert_eq!(view.origin_col, 392);
        assert_eq!(
            view.window_label(4),
            format!("{}387:{}400", sheet_col_label(392), sheet_col_label(395))
        );
    }

    #[test]
    fn prob_tail_values_honestos() {
        assert_eq!(prob_tail_values(0.5), Some((0.5, 0.5, 1.0)));
        let (left, right, two) = prob_tail_values(0.975).expect("cdf finito");
        assert!((left - 0.975).abs() < 1e-12);
        assert!((right - 0.025).abs() < 1e-12);
        assert!((two - 0.05).abs() < 1e-12);
        // Clamp: colas subfluidas no producen negativos ni probabilidades > 1.
        assert_eq!(prob_tail_values(1.2), Some((1.0, 0.0, 0.0)));
        assert_eq!(prob_tail_values(-0.1), Some((0.0, 1.0, 0.0)));
        // No finito → `None` honesto, jamás una probabilidad inventada.
        assert_eq!(prob_tail_values(f64::NAN), None);
        assert_eq!(prob_tail_values(f64::INFINITY), None);
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn prob_alphas_rapidos_en_rango() {
        for alpha in PROB_QUANTILE_ALPHAS {
            assert!(alpha > 0.001 && alpha < 0.999, "α fuera de rango: {alpha}");
        }
        // Ascendente: los chips se leen como tabla de cuantiles.
        assert!(PROB_QUANTILE_ALPHAS.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn resumen_de_rango_honesto() {
        let mut document = Document::default();
        for (row, value) in ["1", "2", "3", "4"].iter().enumerate() {
            document
                .set_spreadsheet_cell(row, 0, (*value).to_string())
                .expect("celda válida");
        }
        let summary = summarize_range(&document, "A1:A4").expect("rango válido");
        assert_eq!(summary.count, 4);
        assert!((summary.mean - 2.5).abs() < 1e-12);
        assert!((summary.min - 1.0).abs() < 1e-12);
        assert!((summary.max - 4.0).abs() < 1e-12);
        let sd = summary.sample_sd.expect("n > 1");
        assert!((sd - 1.290_994_448_735_865_7).abs() < 1e-9, "s = {sd}");
        // Una sola celda: sin desvío muestral (— honesto), no 0 inventado.
        let single = summarize_range(&document, "A1").expect("celda sola");
        assert_eq!(single.count, 1);
        assert!(single.sample_sd.is_none());
        // Rango 2D, columna vacía y fuera de la hoja 400×400: error honesto.
        assert!(summarize_range(&document, "A1:B2").is_err());
        assert!(summarize_range(&document, "C1:C4").is_err());
        assert!(summarize_range(&document, "A1:A401").is_err());
        // Valores enormes finitos: Welford no desborda a `inf` (la suma naive sí).
        document
            .set_spreadsheet_cell(0, 1, "1e308".to_string())
            .expect("celda válida");
        document
            .set_spreadsheet_cell(1, 1, "1e308".to_string())
            .expect("celda válida");
        let huge = summarize_range(&document, "B1:B2").expect("finitos válidos");
        assert!(huge.mean.is_finite(), "media = {}", huge.mean);
        assert!(huge.sample_sd.is_some_and(f64::is_finite));
        // La cota del panel cubre todo rango 1D válido de la hoja (≤400) y
        // no excede el presupuesto de recomputación del core.
        assert!(MAX_SUMMARY_CELLS >= Document::MAX_SPREADSHEET_ROWS);
        assert!(MAX_SUMMARY_CELLS <= Document::MAX_SPREADSHEET_RECOMPUTE_CELLS);
    }
}

#[cfg(test)]
mod protocol_nav_tests {
    use super::select_protocol_step;
    use crate::sheet_layout::{protocol_advance, ProtocolNavState, PROTOCOL_PLAY_INTERVAL_SECS};

    #[test]
    fn play_avanza_de_a_un_paso_y_frena_al_final() {
        assert_eq!(PROTOCOL_PLAY_INTERVAL_SECS, 1.0);
        let mut state = ProtocolNavState {
            playing: true,
            ..Default::default()
        };
        // Primer tick: arranca en el paso 0.
        protocol_advance(&mut state, 3, 2.0);
        assert_eq!(state.cursor, Some(0));
        // Antes del intervalo no avanza.
        protocol_advance(&mut state, 3, 2.5);
        assert_eq!(state.cursor, Some(0));
        protocol_advance(&mut state, 3, 3.1);
        assert_eq!(state.cursor, Some(1));
        protocol_advance(&mut state, 3, 4.2);
        assert_eq!(state.cursor, Some(2));
        // Al llegar al final, frena solo.
        protocol_advance(&mut state, 3, 5.3);
        assert_eq!(state.cursor, Some(2));
        assert!(!state.playing);
        // Pausado no avanza aunque pase el tiempo.
        state.playing = false;
        protocol_advance(&mut state, 3, 99.0);
        assert_eq!(state.cursor, Some(2));
    }

    #[test]
    fn play_con_log_vacio_no_hace_nada() {
        let mut state = ProtocolNavState {
            playing: true,
            ..Default::default()
        };
        protocol_advance(&mut state, 0, 5.0);
        assert_eq!(state.cursor, None);
        assert!(state.playing);
    }

    #[test]
    fn trig_chrome_sale_de_trig_msg() {
        // B-A96: chrome trig (títulos + 3 secciones + centrar) sale de
        // `trig.*` con `trig_locale`; ES pineado a los literales previos
        // (`:2184-2187` títulos, `:2211` función, `:2249` velocidad,
        // `:2263` vista, `:2297` ángulo, `:2354` centrar). Sin egui.
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(
            t("trig.title_explorer", Locale::Es),
            "Explorador Trigonométrico"
        );
        assert_eq!(t("trig.title_complex", Locale::Es), "Animación Compleja");
        assert_eq!(t("trig.section_function", Locale::Es), "FUNCIÓN ACTIVA");
        assert_eq!(t("trig.section_speed", Locale::Es), "VELOCIDAD");
        assert_eq!(t("trig.section_view", Locale::Es), "VISTA");
        assert_eq!(t("trig.section_angle", Locale::Es), "ÁNGULO");
        assert_eq!(
            t("trig.center_view", Locale::Es),
            "Centrar vista en la gráfica"
        );
        // Wiring: el draw usa las 7 keys con `trig_locale` (B-A96).
        let source = include_str!("panels.rs");
        for pin in [
            "trig_t(\"trig.title_explorer\"",
            "trig_t(\"trig.title_complex\"",
            "trig_t(\"trig.section_function\"",
            "trig_t(\"trig.section_speed\"",
            "trig_t(\"trig.section_view\"",
            "trig_t(\"trig.section_angle\"",
            "trig_t(\"trig.center_view\"",
            "let trig_locale = app.config_locale()",
        ] {
            assert!(source.contains(pin), "trig cablea {pin}");
        }
        // Blindaje: viejas llamadas con literal ES duro ya no existen.
        for needle in [
            "trig_section_label(ui, \"FUNCI",
            "trig_section_label(ui, \"VISTA\"",
            "\"VELOCIDAD\",\n                                &trig_speed_text",
            "trig_value_row(ui, \"ÁNGULO\"",
            "panel_primary_button(ui, \"Centrar",
            "\"Animación Compleja\"\n                    } else {",
        ] {
            assert!(
                !source.contains(needle),
                "trig sin ES duro en prod (B-A96): {needle}"
            );
        }
    }

    #[test]
    fn trig_modos_y_play_salen_de_trig_msg() {
        // B-A97: modos Didáctica/Sobre grilla + hints y tips play/pausa
        // salen de `trig.*` con `trig_locale`; ES pineado a los literales
        // previos (`:2243` Pausar, `:2245` Iniciar, `:2276-2277` Didáctica
        // + hint, `:2281-2282` Sobre grilla + hint). Sin egui.
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(t("trig.play_pause_tip", Locale::Es), "Pausar animación");
        assert_eq!(t("trig.play_play_tip", Locale::Es), "Iniciar animación");
        assert_eq!(t("trig.view_didactic", Locale::Es), "Didáctica");
        assert_eq!(
            t("trig.view_didactic_hint", Locale::Es),
            "Círculo unitario siempre visible en una tarjeta flotante"
        );
        assert_eq!(t("trig.view_grid", Locale::Es), "Sobre grilla");
        assert_eq!(
            t("trig.view_grid_hint", Locale::Es),
            "Dibuja el círculo unitario en coordenadas reales"
        );
        // Wiring: el draw usa las 6 keys con `trig_locale` (B-A97).
        let source = include_str!("panels.rs");
        for pin in [
            "trig_t(\"trig.play_pause_tip\"",
            "trig_t(\"trig.play_play_tip\"",
            "trig_t(\"trig.view_didactic\"",
            "trig_t(\"trig.view_didactic_hint\"",
            "trig_t(\"trig.view_grid\"",
            "trig_t(\"trig.view_grid_hint\"",
        ] {
            assert!(source.contains(pin), "trig cablea {pin}");
        }
        // Blindaje: viejas llamadas con literal ES duro ya no existen. Las
        // de modos/hints van por fragmentos porque `include_str!` incluye
        // este test (los pines ES de arriba se auto-matchearían).
        for needle in [
            "accent, \"Pausar animación\"",
            "text_secondary, \"Iniciar animación\"",
        ] {
            assert!(
                !source.contains(needle),
                "trig sin ES duro en prod (B-A97): {needle}"
            );
        }
        for needle in [
            ["\"Did", "áctica\","].concat(),
            [
                "\"Círculo unitario siempre visible",
                " en una tarjeta flotante\",",
            ]
            .concat(),
            ["\"Sobre gri", "lla\","].concat(),
            ["\"Dibuja el círculo unitario", " en coordenadas reales\","].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "trig sin ES duro en prod (B-A97): {needle}"
            );
        }
    }

    #[test]
    fn trig_descripciones_prompt_identidad_salen_de_trig_msg() {
        // B-A98: desc compleja + prompt Auto + lecturas con texto +
        // identidades salen de `trig.*` con locale; ES pineado a los
        // literales previos (`panels.rs` desc, `sheet_layout.rs:723-732`
        // consts TRIG_PROMPT_*, `:760`/` :766`, `app.rs:2323-2332`).
        // Sin egui: helpers puros + blindaje de wiring por `include_str!`.
        use crate::app::GrafitoApp;
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(
            t("trig.complex_desc", Locale::Es),
            // Concat a propósito: el blindaje de abajo usa `include_str!`
            // (incluye este test) y el literal completo se auto-matchearía.
            [
                "El punto rojo recorre el círculo unitario; el punto violeta ",
                "muestra su imagen por la transformación compleja activa.",
            ]
            .concat()
        );
        assert_eq!(t("trig.prompt_auto", Locale::Es), "Auto");
        assert_eq!(t("trig.prompt_apply", Locale::Es), "Aplicar");
        assert_eq!(
            t("trig.identity_cot", Locale::Es),
            "cot θ = cos θ / sin θ; no está definida cuando sin θ = 0."
        );
        // Helpers: Es espeja el canónico, el resto resuelve.
        assert_eq!(
            GrafitoApp::trig_identity_for(0, Locale::Es),
            GrafitoApp::trig_identity(0)
        );
        assert_eq!(
            GrafitoApp::trig_identity_for(2, Locale::En),
            "tan θ = sin θ / cos θ; grows fast near cos θ = 0."
        );
        assert_eq!(
            GrafitoApp::trig_identity_for(9, Locale::De),
            GrafitoApp::trig_identity_for(0, Locale::De),
            "índice rancio cae a sin (igual que el canónico)"
        );
        // Wiring: el draw usa las keys/helpers con locale (B-A98).
        let source = include_str!("panels.rs");
        for pin in [
            "trig_t(\"trig.complex_desc\"",
            "trig_prompt_auto_label(prompt_locale)",
            "trig_prompt_hint_text(prompt_locale)",
            "trig_prompt_field_tooltip(prompt_locale)",
            "trig_prompt_apply_label(prompt_locale)",
            "trig_prompt_apply_tooltip(prompt_locale)",
            "trig_value_text_for(spec.name, t, fn_val, trace_locale)",
            "trig_point_text_for(t, trace_locale)",
            "GrafitoApp::trig_identity_for(\n                                app.trig_function,\n                                trace_locale,",
        ] {
            assert!(source.contains(pin), "trig cablea {pin}");
        }
        // Blindaje: viejos literales/consts ya no están en el draw.
        // (Fragmentos porque `include_str!` incluye este test.)
        for needle in [
            ["violeta ", "muestra su imagen"].concat(),
            ["TRIG_PROMPT_AUTO", "_LABEL)"].concat(),
            ["TRIG_PROMPT_", "HINT)"].concat(),
            ["\"Punto: (cos θ, sin θ) = (\""].concat(),
            ["trig_identity(app.trig", "_function)"].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "trig sin ES duro en prod (B-A98): {needle}"
            );
        }
    }

    #[test]
    fn stats_chrome_sale_de_stats_msg() {
        // B-A105: chrome del panel de estadística sale de `stats.*` con
        // `stats_locale`; ES pineado a los literales previos (`:2464`
        // Distribución, `:2474` Tus datos, `:2504` Aplicar, `:2532`
        // Limpiar, `:2570` Sin datos todavía, `:2578` Ingresá valores…,
        // `:2600` Resumen). Sin egui: pines + blindaje por `include_str!`.
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(t("stats.dist_title", Locale::Es), "Distribución");
        assert_eq!(t("stats.data_title", Locale::Es), "Tus datos");
        assert_eq!(t("stats.apply", Locale::Es), "Aplicar");
        assert_eq!(t("stats.clear", Locale::Es), "Limpiar");
        assert_eq!(t("stats.empty", Locale::Es), "Sin datos todavía");
        // Concat a propósito: el blindaje de abajo usa `include_str!`
        // (incluye este test) y el literal completo se auto-matchearía
        // (precedente B-A98 `trig.complex_desc`).
        assert_eq!(
            t("stats.empty_hint", Locale::Es),
            ["Ingresá valores y pulsá Apli", "car."].concat()
        );
        assert_eq!(t("stats.summary", Locale::Es), "Resumen");
        // Wiring: el draw usa las 7 keys con `stats_locale` (B-A105).
        let source = include_str!("panels.rs");
        for pin in [
            "let dist_label = stats_t(\"stats.dist_title\"",
            "let data_label = stats_t(\"stats.data_title\"",
            "let apply_label = stats_t(\"stats.apply\"",
            "let clear_label = stats_t(\"stats.clear\"",
            "let empty_label = stats_t(\"stats.empty\"",
            "let empty_hint_label = stats_t(\"stats.empty_hint\"",
            "let summary_label = stats_t(\"stats.summary\"",
            "let stats_locale = app.config_locale()",
        ] {
            assert!(source.contains(pin), "stats cablea {pin}");
        }
        // Blindaje: viejos literales ES duros ya no están en el draw.
        // (Fragmentos porque `include_str!` incluye este test.)
        for needle in [
            ["draw_inspector_section(ui, \"Distri", "bución\""].concat(),
            ["draw_inspector_section(ui, \"Tus ", "datos\""].concat(),
            ["RichText::new(\"Apli", "car\")"].concat(),
            ["RichText::new(\"Lim", "piar\")"].concat(),
            ["RichText::new(\"Sin datos to", "davía\")"].concat(),
            ["\"Ingresá valores y pulsá Apli", "car.\""].concat(),
            ["RichText::new(\"Resu", "men\")"].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "stats sin ES duro en prod (B-A105): {needle}"
            );
        }
    }

    #[test]
    fn stats_b_a119_empty_y_puente_salen_de_stats_msg() {
        // B-A119: empty de objetos estadísticos + puente caso/cuantil
        // salen de `stats.*` con `stats_locale`/`prob_locale`; ES pineado
        // a los literales previos (`:2942` Objetos estadísticos, `:2943`
        // Sin gráficos…, `:6124` Usar caso…, `:6149` Sin comando de
        // caso…, `:6301` Usar cuantil…, `:6326` Sin comando de
        // cuantil…, `:6305` Pone el Inverse*…). Sin egui: pines +
        // blindaje por `include_str!`.
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(t("stats.objects_title", Locale::Es), "Objetos estadísticos");
        // Concat a propósito: el blindaje de abajo usa `include_str!`
        // (incluye este test) y el literal completo se auto-matchearía
        // (precedente B-A105 `stats.empty_hint`).
        assert_eq!(
            t("stats.objects_empty", Locale::Es),
            [
                "Sin gráficos estad",
                "ísticos.\nProbá Histogram[...] o ScatterPlot[...].",
            ]
            .concat()
        );
        assert_eq!(
            t("stats.prob_use_case", Locale::Es),
            "Usar caso en la entrada"
        );
        // (Fragmentos también en los pines: el literal completo se
        // auto-matchearía con la aguja partida de abajo.)
        assert_eq!(
            t("stats.prob_no_case", Locale::Es),
            ["Sin comando de caso para ", "esta distribución"].concat()
        );
        assert_eq!(
            t("stats.prob_use_quantile", Locale::Es),
            "Usar cuantil en la entrada"
        );
        assert_eq!(
            t("stats.prob_no_quantile", Locale::Es),
            ["Sin comando de cuantil para ", "esta distribución"].concat()
        );
        assert_eq!(
            t("stats.prob_quantile_hint", Locale::Es),
            ["Pone el Inverse* del cuantil ", "actual en la entrada"].concat()
        );
        // Wiring: el draw usa las 7 keys con su locale (B-A119).
        let source = include_str!("panels.rs");
        for pin in [
            "let objects_title_label =",
            "stats_t(\"stats.objects_title\"",
            "stats_t(\"stats.objects_empty\"",
            "let prob_locale = app.config_locale()",
            "let use_case_label = stats_t(\"stats.prob_use_case\"",
            "let no_case_label = stats_t(\"stats.prob_no_case\"",
            "let use_quantile_label = stats_t(\"stats.prob_use_quantile\"",
            "let no_quantile_label = stats_t(\"stats.prob_no_quantile\"",
            "let quantile_hint_label = stats_t(\"stats.prob_quantile_hint\"",
            "RichText::new(use_case_label)",
            "RichText::new(use_quantile_label)",
            ".on_hover_text(no_case_label)",
            ".on_hover_text(no_quantile_label)",
            ".on_hover_text(quantile_hint_label)",
        ] {
            assert!(source.contains(pin), "stats B-A119 cablea {pin}");
        }
        // Blindaje: viejos literales ES duros ya no están en el draw.
        // (Fragmentos porque `include_str!` incluye este test.)
        for needle in [
            ["\"Objetos esta", "dísticos\","].concat(),
            ["\"Sin gráficos estadís", "ticos.\\nProbá"].concat(),
            ["RichText::new(\"Usar caso en ", "la entrada\")"].concat(),
            ["\"Sin comando de caso para ", "esta distribución\""].concat(),
            ["RichText::new(\"Usar cuantil en ", "la entrada\")"].concat(),
            ["\"Sin comando de cuantil para ", "esta distribución\""].concat(),
            ["\"Pone el Inverse* del cuantil ", "actual en la entrada\""].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "stats sin ES duro en prod (B-A119): {needle}"
            );
        }
    }

    #[test]
    fn stats_b_a120_caso_tiles_momentos_salen_de_stats_msg() {
        // B-A120: caso + tiles + momentos salen de `stats.prob_*` con
        // `prob_locale`; ES pineado a los literales previos (`:6046`
        // Caso, `:6053` Copiar, `:6054` Copia el caso…, `:6064` Caso
        // copiado…, `:6085` Acumulada, `:6093` P(X ≤ x), `:6094`
        // P(X > x), `:6095` 2·cola menor, `:6100` f64 exacto…,
        // `:6108` Momentos, `:6120` Media, `:6121` Varianza, `:6122`
        // Desvío). Sin egui: pines + blindaje por `include_str!`.
        use grafito_ui::i18n::{t, Locale};
        // Concat a propósito: el blindaje de abajo usa `include_str!`
        // (incluye este test) y el literal completo se auto-matchearía
        // (precedente B-A105/B-A119).
        assert_eq!(t("stats.prob_case", Locale::Es), ["Ca", "so"].concat());
        assert_eq!(t("stats.prob_copy", Locale::Es), ["Copi", "ar"].concat());
        assert_eq!(
            t("stats.prob_copy_case_hint", Locale::Es),
            ["Copia el caso y ", "sus probabilidades"].concat()
        );
        assert_eq!(
            t("stats.prob_case_toast", Locale::Es),
            ["Caso copiado al ", "portapapeles"].concat()
        );
        assert_eq!(t("stats.prob_cum", Locale::Es), ["Acumu", "lada"].concat());
        assert_eq!(
            t("stats.prob_tail_le", Locale::Es),
            ["P(X ≤ ", "x)"].concat()
        );
        assert_eq!(
            t("stats.prob_tail_gt", Locale::Es),
            ["P(X > ", "x)"].concat()
        );
        assert_eq!(
            t("stats.prob_two_tail", Locale::Es),
            ["2·cola me", "nor"].concat()
        );
        assert_eq!(
            t("stats.prob_f64_note", Locale::Es),
            ["f64 exacto; colas extremas ", "pueden subfluir a 0."].concat()
        );
        assert_eq!(
            t("stats.prob_moments", Locale::Es),
            ["Mome", "ntos"].concat()
        );
        assert_eq!(t("stats.prob_mean", Locale::Es), ["Me", "dia"].concat());
        assert_eq!(t("stats.prob_var", Locale::Es), ["Varian", "za"].concat());
        assert_eq!(t("stats.prob_sd", Locale::Es), ["Des", "vío"].concat());
        // Wiring: el draw usa las 13 keys con su locale (B-A120).
        let source = include_str!("panels.rs");
        for pin in [
            "let prob_case_label = stats_t(\"stats.prob_case\"",
            "let prob_copy_label = stats_t(\"stats.prob_copy\"",
            "stats_t(\"stats.prob_copy_case_hint\"",
            "stats_t(\"stats.prob_case_toast\"",
            "stats_t(\"stats.prob_cum\"",
            "stats_t(\"stats.prob_tail_le\"",
            "stats_t(\"stats.prob_tail_gt\"",
            "stats_t(\"stats.prob_two_tail\"",
            "stats_t(\"stats.prob_f64_note\"",
            "stats_t(\"stats.prob_moments\"",
            "stats_t(\"stats.prob_mean\"",
            "stats_t(\"stats.prob_var\"",
            "stats_t(\"stats.prob_sd\"",
            "RichText::new(prob_case_label)",
            "RichText::new(prob_cum_label)",
            "RichText::new(prob_moments_label)",
            ".small_button(prob_copy_label)",
            ".on_hover_text(prob_copy_case_hint)",
            "app.cas_result = prob_case_toast",
        ] {
            assert!(source.contains(pin), "stats B-A120 cablea {pin}");
        }
        // Blindaje: viejos literales ES duros del bloque caso/tiles ya no
        // están en el draw. (Fragmentos porque `include_str!` incluye
        // este test. `small_button("Copiar")` NO se blindajea: el Copiar
        // del cuantil `:6293` sigue ES duro a propósito, follow-up B-A121;
        // `RichText::new("Media") :2846` es del panel de stats, fuera de
        // scope.)
        for needle in [
            ["RichText::new(\"Ca", "so\")"].concat(),
            ["\"Copia el caso y sus prob", "abilidades\""].concat(),
            ["\"Caso copiado al portap", "apeles\""].concat(),
            ["\"Acumu", "lada\""].concat(),
            ["\"2·cola me", "nor\""].concat(),
            ["RichText::new(\"Mome", "ntos\")"].concat(),
            ["\"f64 exacto; colas extremas ", "pueden subfluir"].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "stats sin ES duro en prod (B-A120): {needle}"
            );
        }
    }

    #[test]
    fn stats_b_a121_cuantil_chrome_sale_de_stats_msg() {
        // B-A121: cuantil chrome sale de `stats.prob_quantile_*` con
        // `prob_locale` (`:6189` header, `:6201` hover α con template
        // `{pct}`, `:6209` param, `:6294` hover copia, `:6298` toast
        // copia, `:6313` nota, `:6318` hover largo, `:6337` toast
        // listo); `:6293` reutiliza `stats.prob_copy` existente. Sin
        // egui: pines + blindaje por `include_str!`.
        use grafito_ui::i18n::{t, Locale};
        // Concat a propósito: el blindaje usa `include_str!` (incluye
        // este test) y el literal completo se auto-matchearía
        // (precedente B-A105/B-A119/B-A120).
        assert_eq!(
            t("stats.prob_quantile_title", Locale::Es),
            ["Cuantil (in", "versa)"].concat()
        );
        assert_eq!(
            t("stats.prob_quantile_alpha_hint", Locale::Es),
            ["Cuantil {p", "ct}%"].concat()
        );
        assert_eq!(
            t("stats.prob_quantile_param", Locale::Es),
            ["p cuan", "til"].concat()
        );
        assert_eq!(
            t("stats.prob_copy_quantile_hint", Locale::Es),
            ["Copia el cuantil calcu", "lado"].concat()
        );
        assert_eq!(
            t("stats.prob_quantile_toast", Locale::Es),
            ["Cuantil copiado al porta", "papeles"].concat()
        );
        assert_eq!(
            t("stats.prob_quantile_note", Locale::Es),
            [
                "Inversa numérica del motor único (Geométrica acu",
                "mula pmf)."
            ]
            .concat()
        );
        assert_eq!(
            t("stats.prob_quantile_note_hint", Locale::Es),
            ["Misma inversa que InverseNormal/", "InverseT/InverseChiSquared/InverseF/InverseExponential/InverseUniform; Geométrica acumula pmf y Uniforme/Exponencial usan forma cerrada."].concat()
        );
        assert_eq!(
            t("stats.prob_quantile_ready_toast", Locale::Es),
            ["Cuantil listo en la entrada (Enter para eva", "luar)."].concat()
        );
        // Helper puro: Es delega al canónico previo `:6201`.
        assert_eq!(
            super::quantile_alpha_hover_text(0.05, Locale::Es),
            ["Cuantil ", "5.0%"].concat()
        );
        assert_eq!(
            super::quantile_alpha_hover_text(0.05, Locale::En),
            ["Quantile ", "5.0%"].concat()
        );
        // Wiring: el draw usa las 8 keys + reuse con su locale (B-A121).
        let source = include_str!("panels.rs");
        for pin in [
            "let prob_quantile_title = stats_t(\"stats.prob_quantile_title\"",
            "let prob_quantile_param = stats_t(\"stats.prob_quantile_param\"",
            "stats_t(\"stats.prob_copy_quantile_hint\"",
            "stats_t(\"stats.prob_quantile_toast\"",
            "stats_t(\"stats.prob_quantile_note\"",
            "stats_t(\"stats.prob_quantile_note_hint\"",
            "stats_t(\"stats.prob_quantile_ready_toast\"",
            "RichText::new(prob_quantile_title)",
            "quantile_alpha_hover_text(alpha, prob_locale)",
            "prob_param_row(ui, prob_quantile_param",
            ".small_button(prob_copy_label)",
            ".on_hover_text(prob_copy_quantile_hint)",
            "app.cas_result = prob_quantile_toast",
            "RichText::new(prob_quantile_note)",
            ".on_hover_text(prob_quantile_note_hint)",
            "app.cas_result = prob_quantile_ready_toast",
        ] {
            assert!(source.contains(pin), "stats B-A121 cablea {pin}");
        }
        // Blindaje: viejos literales ES duros del bloque cuantil ya no
        // están en el draw. (Fragmentos porque `include_str!` incluye
        // este test; splits distintos de los pines de arriba.)
        for needle in [
            ["RichText::new(\"Cuantil (inver", "sa)\")"].concat(),
            ["format!(\"Cuantil {:.1}%\", alpha *", " 100.0)"].concat(),
            ["prob_param_row(ui, \"p cuan", "til\""].concat(),
            ["\"Copia el cuantil calcu", "lado\""].concat(),
            ["\"Cuantil copiado al porta", "papeles\""].concat(),
            [
                "\"Inversa numérica del motor único (Geométrica acumu",
                "la pmf).\"",
            ]
            .concat(),
            [
                "\"Misma inversa que InverseNormal/InverseT/InverseChi",
                "Squared/",
            ]
            .concat(),
            ["\"Cuantil listo en la entrada (Enter para eva", "luar).\""].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "stats sin ES duro en prod (B-A121): {needle}"
            );
        }
    }

    #[test]
    fn stats_b_a155_params_salen_de_stats_msg() {
        // B-A155: rótulos de parámetros + hovers salen de
        // `stats.prob_param_*` con `prob_locale` (antes 2 `format!` con
        // ES duro en `prob_param_row` + 14 rótulos en el `match` por
        // distribución); templates `{label}`, notación intacta. Sin
        // egui: helpers + wiring + blindaje por `include_str!`.
        use grafito_ui::i18n::Locale;
        // Helpers puros: Es espeja los literales previos con el rótulo
        // ya sustituido, resto localiza. (Concat a propósito: el
        // blindaje usa `include_str!` e incluye este test.)
        assert_eq!(
            super::prob_param_hover_text(&["μ me", "dia"].concat(), Locale::Es),
            ["Parámetro μ media (arrastrá o editá ", "el valor)"].concat()
        );
        assert_eq!(
            super::prob_param_hover_text("x point", Locale::En),
            "Parameter x point (drag or edit the value)"
        );
        assert_eq!(
            super::prob_param_value_text(&["k even", "tos"].concat(), Locale::Es),
            ["Valor de k even", "tos"].concat()
        );
        assert_eq!(
            super::prob_param_value_text("x point", Locale::En),
            "Value of x point"
        );
        // Templates `{label}` en los 6 idiomas, sin fallback a la clave.
        for locale in [
            Locale::Es,
            Locale::En,
            Locale::Pt,
            Locale::It,
            Locale::Fr,
            Locale::De,
        ] {
            for text in [
                super::prob_param_hover_text("Q", locale),
                super::prob_param_value_text("Q", locale),
            ] {
                assert!(text.contains('Q'), "sin label en {locale:?}: {text}");
                assert!(
                    !text.contains("{label}"),
                    "template sin interpolar en {locale:?}: {text}"
                );
                assert!(!text.contains("stats."), "fallback en {locale:?}: {text}");
            }
        }
        // Wiring: el draw usa los 14 lets + helpers con su locale.
        let source = include_str!("panels.rs");
        for pin in [
            "pub(crate) fn prob_param_hover_text",
            "pub(crate) fn prob_param_value_text",
            "let prob_param_mu_label = stats_t(\"stats.prob_param_mu\"",
            "let prob_param_sigma_label = stats_t(\"stats.prob_param_sigma\"",
            "let prob_param_x_label = stats_t(\"stats.prob_param_x\"",
            "let prob_param_n_label = stats_t(\"stats.prob_param_n\"",
            "let prob_param_p_label = stats_t(\"stats.prob_param_p\"",
            "let prob_param_k_success_label = stats_t(\"stats.prob_param_k_success\"",
            "let prob_param_lambda_label = stats_t(\"stats.prob_param_lambda\"",
            "let prob_param_k_events_label = stats_t(\"stats.prob_param_k_events\"",
            "let prob_param_df_label = stats_t(\"stats.prob_param_df\"",
            "let prob_param_df1_label = stats_t(\"stats.prob_param_df1\"",
            "let prob_param_df2_label = stats_t(\"stats.prob_param_df2\"",
            "let prob_param_k_failures_label = stats_t(\"stats.prob_param_k_failures\"",
            "let prob_param_a_label = stats_t(\"stats.prob_param_a\"",
            "let prob_param_b_label = stats_t(\"stats.prob_param_b\"",
            "prob_param_row(ui, prob_param_mu_label",
            "prob_param_row(ui, prob_param_sigma_label",
            "prob_param_row(ui, prob_param_x_label",
            "prob_param_row(ui, prob_param_n_label",
            "prob_param_row(ui, prob_param_p_label",
            "prob_param_row(ui, prob_param_k_success_label",
            "prob_param_row(ui, prob_param_lambda_label",
            "prob_param_row(ui, prob_param_k_events_label",
            "prob_param_row(ui, prob_param_df_label",
            "prob_param_row(ui, prob_param_df1_label",
            "prob_param_row(ui, prob_param_df2_label",
            "prob_param_row(ui, prob_param_k_failures_label",
            "prob_param_row(ui, prob_param_a_label",
            "prob_param_row(ui, prob_param_b_label",
            "prob_param_row(ui, prob_quantile_param",
            "prob_param_hover_text(label, locale)",
            "prob_param_value_text(label, locale)",
            "false, prob_locale)",
        ] {
            assert!(source.contains(pin), "stats B-A155 cablea {pin}");
        }
        // Blindaje: viejos literales ES duros del bloque params ya no
        // están en el draw. (Fragmentos porque `include_str!` incluye
        // este test; contexto de call-site donde el identificador
        // sobrevive en otro uso, precedente B-A97.)
        for needle in [
            ["prob_param_row(ui, \"μ me", "dia\""].concat(),
            ["prob_param_row(ui, \"σ des", "vío\""].concat(),
            ["prob_param_row(ui, \"x pu", "nto\""].concat(),
            ["prob_param_row(ui, \"n ensa", "yos\""].concat(),
            ["prob_param_row(ui, \"p é", "xito\""].concat(),
            ["prob_param_row(ui, \"k é", "xitos\""].concat(),
            ["prob_param_row(ui, \"λ ta", "sa\""].concat(),
            ["prob_param_row(ui, \"k even", "tos\""].concat(),
            ["prob_param_row(ui, \"gl liber", "tad\""].concat(),
            ["prob_param_row(ui, \"g", "l1\""].concat(),
            ["prob_param_row(ui, \"g", "l2\""].concat(),
            ["prob_param_row(ui, \"k fa", "llos\""].concat(),
            ["prob_param_row(ui, \"a mí", "nimo\""].concat(),
            ["prob_param_row(ui, \"b má", "ximo\""].concat(),
            [
                "format!(\"Parámetro {label} (arrastrá o editá ",
                "el valor)\")",
            ]
            .concat(),
            ["format!(\"Valor ", "de {label}\")"].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "stats sin ES duro en prod (B-A155): {needle}"
            );
        }
    }

    #[test]
    fn stats_b_a156_selector_dists_sale_de_stats_msg() {
        // B-A156: selector de distribución sale de
        // `stats.prob_dist_*/kind_*/point_*` con `prob_locale` (antes
        // `PROB_DISTS` crudo en el loop del combo `:6217` + `prob_dist_name`
        // `:6212` + `prob_dist_tooltip` `:6222` + `prob_point_label`
        // `:6469`); helpers `_for` con Es al canónico. Sin egui: helpers
        // + wiring + blindaje por `include_str!`.
        use grafito_ui::i18n::Locale;
        // Helpers puros: Es espeja los canónicos de `PROB_DISTS`, resto
        // localiza. (Concat a propósito: el blindaje usa `include_str!`
        // e incluye este test, precedente B-A155.)
        assert_eq!(
            super::prob_dist_name_for(3, Locale::Es),
            ["t-Stu", "dent"].concat()
        );
        assert_eq!(
            super::prob_dist_name_for(4, Locale::Es),
            ["χ", "²"].concat()
        );
        assert_eq!(
            super::prob_dist_name_for(6, Locale::Es),
            ["Geomé", "trica"].concat()
        );
        assert_eq!(
            super::prob_dist_name_for(8, Locale::Es),
            ["Exponen", "cial"].concat()
        );
        assert_eq!(
            super::prob_dist_name_for(99, Locale::Es),
            ["Nor", "mal"].concat()
        );
        assert_eq!(
            super::prob_dist_tooltip_for(1, Locale::Es),
            Some(["Discreta · n ensayos, p éxito: ", "P(X = k) y P(X ≤ k)"].concat())
        );
        assert_eq!(super::prob_dist_tooltip_for(99, Locale::Es), None);
        assert_eq!(
            crate::sheet_layout::prob_dist_kind_label_for(2, Locale::Es),
            ["Dis", "creta"].concat()
        );
        assert_eq!(
            crate::sheet_layout::prob_dist_kind_label_for(0, Locale::En),
            "Continuous"
        );
        assert_eq!(
            super::prob_point_label_for(6, Locale::Es),
            ["Pun", "tual"].concat()
        );
        assert_eq!(super::prob_point_label_for(5, Locale::Fr), "Densité");
        // 6 idiomas resuelven los 9 índices sin vacío ni fallback.
        for locale in [
            Locale::Es,
            Locale::En,
            Locale::Pt,
            Locale::It,
            Locale::Fr,
            Locale::De,
        ] {
            for index in 0..=8u8 {
                assert!(!super::prob_dist_name_for(index, locale).is_empty());
                assert!(!super::prob_dist_tip_for(index, locale).is_empty());
                assert!(super::prob_dist_tooltip_for(index, locale).is_some());
                assert!(!crate::sheet_layout::prob_dist_kind_label_for(index, locale).is_empty());
                assert!(!super::prob_point_label_for(index, locale).is_empty());
            }
        }
        // Wiring: el draw usa helpers `_for` con `prob_locale`.
        let source = include_str!("panels.rs");
        for pin in [
            "prob_dist_name_for(state.dist, prob_locale)",
            "for index in 0..=8u8",
            "prob_dist_name_for(index, prob_locale)",
            "prob_dist_tip_for(index, prob_locale)",
            "prob_dist_tooltip_for(state.dist, prob_locale)",
            "prob_point_label_for(state.dist, prob_locale)",
        ] {
            assert!(source.contains(pin), "stats B-A156 cablea {pin}");
        }
        // Blindaje: viejos accesos crudos al canónico ES ya no están en
        // el draw. (Fragmentos vía `concat!`: `include_str!` incluye
        // este test y el literal completo se auto-matchearía, precedente
        // B-A89/B-A155.)
        for needle in [
            ["prob_dist_name(sta", "te.dist)"].concat(),
            ["prob_dist_tooltip(sta", "te.dist)"].concat(),
            ["prob_point_label(sta", "te.dist)"].concat(),
            ["for (index, name, tip) in PROB_DI", "STS"].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "stats sin ES duro en prod (B-A156): {needle}"
            );
        }
    }

    #[test]
    fn stats_b_a157_plot_messages_salen_de_stats_msg() {
        // B-A157: plot messages salen de `stats.prob_plot_*` con
        // `plot_locale` (antes 4 literales ES duros `:5756,:5761,:5969,`
        // `:6006` + `check_panel_df` + prefijo `plot_message_text`).
        // Helpers `_for` en `sheet_layout.rs` con Es al canonico.
        // Sin egui: pines + wiring + blindaje por `include_str!`.
        use grafito_ui::i18n::{t, Locale};
        // ES pineado a los literales previos (concat a proposito: el
        // blindaje usa `include_str!` e incluye este test, precedente
        // B-A89/B-A155/B-A156).
        assert_eq!(
            t("stats.prob_plot_sigma_positive", Locale::Es),
            ["σ debe ser ma", "yor que 0"].concat()
        );
        assert_eq!(
            t("stats.prob_plot_mu_sigma_range", Locale::Es),
            ["μ/σ fuera ", "de rango"].concat()
        );
        assert_eq!(
            t("stats.prob_plot_p_range", Locale::Es),
            ["p debe estar ", "en (0, 1]"].concat()
        );
        assert_eq!(
            t("stats.prob_plot_ab_order", Locale::Es),
            ["se requiere ", "a < b"].concat()
        );
        assert_eq!(
            t("stats.prob_plot_df_positive", Locale::Es),
            ["{what} debe ser posi", "tivo y finito"].concat()
        );
        assert_eq!(
            t("stats.prob_plot_df_capped", Locale::Es),
            ["{what} ≤ {max} en este panel ", "(cota de UI)"].concat()
        );
        assert_eq!(
            t("stats.prob_plot_prefix", Locale::Es),
            ["fuera de cota: ", "{message}"].concat()
        );
        assert_eq!(
            t("stats.prob_plot_sigma_positive", Locale::En),
            "σ must be greater than 0"
        );
        assert_eq!(
            t("stats.prob_plot_prefix", Locale::De),
            "außerhalb des Bereichs: {message}"
        );
        // Helpers puros: Es espeja el canonico previo.
        assert_eq!(
            crate::sheet_layout::plot_message_text_for(
                ["σ debe ser ma", "yor que 0"].concat().as_str(),
                Locale::Es
            ),
            ["fuera de cota: σ debe ser ma", "yor que 0"].concat()
        );
        assert_eq!(
            crate::sheet_layout::plot_df_or_fuera_de_cota_for(0.0, "gl", Locale::Es)
                .expect_err("cota"),
            ["gl debe ser posi", "tivo y finito"].concat()
        );
        // Wiring: el draw usa `_for` + `plot_locale` + `stats.prob_plot_*`.
        let source = include_str!("panels.rs");
        for pin in [
            "prob_plot_sigma_positive",
            "prob_plot_mu_sigma_range",
            "prob_plot_p_range",
            "prob_plot_ab_order",
            "plot_df_or_fuera_de_cota_for(state.df",
            "plot_df_or_fuera_de_cota_for(state.df1",
            "plot_message_text_for(message, locale)",
            "draw_probability_plot(",
            "app.reduced_motion,",
        ] {
            assert!(source.contains(pin), "stats B-A157 cablea {pin}");
        }
        // Blindaje: viejos literales ES y llamadas sin locale ya no estan
        // en el draw. (Fragmentos via `concat!`: `include_str!` incluye
        // este test y el literal completo se auto-matchearia.)
        for needle in [
            ["paint_plot_message(&painter, plot, \"σ debe", " ser"].concat(),
            ["paint_plot_message(&painter, plot, \"μ/σ fuera", " de"].concat(),
            ["paint_plot_message(&painter, plot, \"p debe estar", " en"].concat(),
            ["paint_plot_message(&painter, plot, \"se requiere", " a"].concat(),
            ["plot_message_text(mes", "sage)"].concat(),
            ["plot_df_or_fuera_de_cota(state.df, \"g", "l\")"].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "stats sin ES duro en prod (B-A157): {needle}"
            );
        }
    }

    #[test]
    fn stats_s2_prob_plot_consts_barras_hover_y_fade() {
        // S2 ui-polish: geometría del plot en consts `PROB_PLOT_*`
        // (`sheet_layout.rs`), barras discretas con hover + `widget_info`
        // (`k` + masa vía `stats.prob_plot_bar`) y fundido `animate_bool`
        // al cambiar de distribución (repaint acotado al fundido).
        // Sin egui: pines de valores + wiring por `include_str!`.
        use crate::sheet_layout::{
            prob_plot_band_alpha, prob_plot_bar_h, prob_plot_grow, PROB_PLOT_AXIS_RESERVE,
            PROB_PLOT_BAR_GAP, PROB_PLOT_BAR_INSET_X, PROB_PLOT_BAR_MIN, PROB_PLOT_BAR_ROUNDING,
            PROB_PLOT_BASELINE_W, PROB_PLOT_GROW_MIN, PROB_PLOT_H, PROB_PLOT_INSET,
            PROB_PLOT_LABEL_DY,
        };
        use grafito_ui::i18n::{t, Locale};
        // Verbatim previos del draw (paridad visual exacta con fade = 1).
        assert_eq!((PROB_PLOT_H, PROB_PLOT_AXIS_RESERVE), (84.0, 16.0));
        assert_eq!(
            (PROB_PLOT_INSET, PROB_PLOT_BAR_ROUNDING, PROB_PLOT_LABEL_DY),
            (4.0, 2.0, 4.0)
        );
        assert_eq!(PROB_PLOT_BAR_GAP, 4.0);
        assert_eq!(
            (
                PROB_PLOT_BAR_INSET_X,
                PROB_PLOT_BAR_MIN,
                PROB_PLOT_BASELINE_W
            ),
            (1.0, 1.0, 1.0)
        );
        const { assert!((0.0 < PROB_PLOT_GROW_MIN) && (PROB_PLOT_GROW_MIN < 1.0)) };
        // Helper puro del hover/a11y: template `{k}`/`{p}` en 6 idiomas.
        for locale in [
            Locale::Es,
            Locale::En,
            Locale::Pt,
            Locale::It,
            Locale::Fr,
            Locale::De,
        ] {
            let template = t("stats.prob_plot_bar", locale);
            assert!(template.contains("{k}"), "sin {{k}} en {locale:?}");
            assert!(template.contains("{p}"), "sin {{p}} en {locale:?}");
            let text = crate::sheet_layout::prob_plot_bar_text_for(3, 0.25, locale);
            assert!(text.contains('3'), "k ausente en {locale:?}: {text}");
            assert!(text.contains("0.2500"), "p ausente en {locale:?}: {text}");
        }
        assert_eq!(
            crate::sheet_layout::prob_plot_bar_text_for(0, 1.0, Locale::Es),
            ["k = 0 · P = ", "1.0000"].concat()
        );
        // Comportamiento del fundido: helpers puros de `sheet_layout.rs`
        // cableados en el draw (el wiring lo fija
        // `tests/s2_prob_plot_wiring.rs`, que lee el prod desde fuera y
        // no se auto-matchea). Si el prod deja de modular con `fade`,
        // estos valores cambian y el wiring falla.
        // `grow` interpola piso→1; en `fade = 1` es identidad visual.
        assert_eq!(prob_plot_grow(1.0), 1.0);
        assert_eq!(prob_plot_grow(0.0), PROB_PLOT_GROW_MIN);
        let half = prob_plot_grow(0.5);
        assert!((half - (PROB_PLOT_GROW_MIN + 1.0) / 2.0).abs() < 1e-6);
        assert!(half > PROB_PLOT_GROW_MIN && half < 1.0);
        assert!(prob_plot_grow(0.25) < prob_plot_grow(0.75));
        // Altura de barra: plena con `fade = 1`, al piso con `fade = 0`,
        // monótona en `fade` (barras y curvas crecen con el fundido).
        let full = prob_plot_bar_h(0.5, 1.0, 100.0, 1.0);
        assert!((full - 50.0).abs() < 1e-4);
        let floor = prob_plot_bar_h(0.5, 1.0, 100.0, 0.0);
        assert!((floor - 50.0 * PROB_PLOT_GROW_MIN).abs() < 1e-4);
        let mid = prob_plot_bar_h(0.5, 1.0, 100.0, 0.5);
        assert!(mid > floor && mid < full);
        // Bandas continuas: `fade = 1` deja el alfa intacto (sin salto
        // al terminar la transición), `fade = 0` las apaga.
        assert!((prob_plot_band_alpha(0.3, 1.0) - 0.3).abs() < 1e-6);
        assert_eq!(prob_plot_band_alpha(0.3, 0.0), 0.0);
        assert!(prob_plot_band_alpha(0.3, 0.5) < 0.3);
        // Repintado: lo maneja `animate_bool` (P7 perf: se eliminó el
        // `request_repaint` explícito duplicado y su helper).
        // Id de animación por distribución: dos `dist` distintas dan
        // estados independientes (cambiar de distribución re-anima).
        assert_ne!(
            egui::Id::new(("prob_plot_dist_fade", 0u8)),
            egui::Id::new(("prob_plot_dist_fade", 3u8))
        );
        // Id de barra por (`dist`, `k`): cada barra expone su propio
        // hover/`widget_info` (lector anuncia k + masa de ESA barra).
        assert_ne!(
            egui::Id::new(("prob_plot_bar", 1u8, 2u64)),
            egui::Id::new(("prob_plot_bar", 1u8, 3u64))
        );
        assert_ne!(
            egui::Id::new(("prob_plot_bar", 1u8, 2u64)),
            egui::Id::new(("prob_plot_bar", 2u8, 2u64))
        );
    }

    #[test]
    fn stats_b_a122_cuantil_resultado_sale_de_stats_msg() {
        // B-A122: resultado del cuantil sale de
        // `stats.prob_quantile_result_cont/disc` con `prob_locale`
        // (antes 9 `format!` ES duros `:6240-6304`); templates
        // `{p}/{q}/{extra}`, notación `P(X≤x)/P(X≤k)` intacta. Sin egui:
        // pines + blindaje por `include_str!`.
        use grafito_ui::i18n::{t, Locale};
        // Concat a propósito: el blindaje usa `include_str!` (incluye
        // este test) y el literal completo se auto-matchearía
        // (precedente B-A105/B-A119/B-A120/B-A121).
        assert_eq!(
            t("stats.prob_quantile_result_cont", Locale::Es),
            ["x con P(X≤x)={p} ", "→ {q}{extra}"].concat()
        );
        assert_eq!(
            t("stats.prob_quantile_result_disc", Locale::Es),
            ["menor k con P(X≤k)≥{p} ", "→ {q}{extra}"].concat()
        );
        // Helpers puros: Es espeja el canónico previo, resto localiza.
        assert_eq!(
            super::quantile_result_cont_text("0.950", "1.645", "", Locale::Es),
            ["x con P(X≤x)=0.950 ", "→ 1.645"].concat()
        );
        assert_eq!(
            super::quantile_result_cont_text("0.950", "1.645", "", Locale::En),
            ["x with P(X≤x)=0.950 ", "→ 1.645"].concat()
        );
        assert_eq!(
            super::quantile_result_disc_text("0.500", "3", " (n=10)", Locale::Es),
            ["menor k con P(X≤k)≥0.500 ", "→ 3 (n=10)"].concat()
        );
        assert_eq!(
            super::quantile_result_disc_text("0.500", "3", " (n=10)", Locale::En),
            ["smallest k with P(X≤k)≥0.500 ", "→ 3 (n=10)"].concat()
        );
        // Wiring: el draw usa los 2 templates vía helpers con su locale.
        let source = include_str!("panels.rs");
        for pin in [
            "quantile_result_cont_text(&p_str",
            "quantile_result_disc_text(&p_str",
            "let p_str = format!(\"{:.3}\", state.p_inv)",
            "stats.prob_quantile_result_cont",
            "stats.prob_quantile_result_disc",
        ] {
            assert!(source.contains(pin), "stats B-A122 cablea {pin}");
        }
        // Blindaje: viejos templates ES duros del resultado ya no están
        // en el draw. (Fragmentos; splits distintos de los pines.)
        // Los extras `" (gl=…)"`/`" (λ=…)"` quedan intactos a propósito:
        // son notación con `format_statistic`, no chrome (precedente
        // B-A98 unidades SI).
        for needle in [
            ["format!(\"x con P(X≤x)={:.3} ", "→ {})"].concat(),
            ["format!(\"menor k con P(X≤k)≥{:.3} ", "→ {q})"].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "stats sin ES duro en prod (B-A122): {needle}"
            );
        }
    }

    #[test]
    fn stats_b_a123_ordenar_graficar_sale_de_stats_msg() {
        // B-A123: ordenar+graficar sale de `stats.sort_*/hist_*/box_*/mean_*`
        // con `stats_locale` (antes 10 literales ES duros `:2749,:2761,`
        // `:2765,:2780,:2789,:2793,:2819,:2823,:2846,:2850`); template
        // `{n}` para el toast. Sin egui: pines + blindaje por
        // `include_str!`.
        use grafito_ui::i18n::{t, Locale};
        // Concat a propósito: el blindaje usa `include_str!` (incluye
        // este test) y el literal completo se auto-matchearía
        // (precedente B-A105/B-A119/B-A120/B-A121/B-A122).
        assert_eq!(t("stats.sort_section", Locale::Es), "Ordenar y graficar");
        assert_eq!(t("stats.sort_btn", Locale::Es), "Ordenar");
        assert_eq!(
            t("stats.sort_hint", Locale::Es),
            ["Ordena ascendente (Sort) ", "y actualiza el campo"].concat()
        );
        assert_eq!(
            t("stats.sorted_toast", Locale::Es),
            ["Datos ordenados ", "(N={n})"].concat()
        );
        assert_eq!(t("stats.hist_btn", Locale::Es), "Histograma");
        assert_eq!(
            t("stats.hist_hint", Locale::Es),
            "Crea Histogram[datos, 10] en la entrada"
        );
        assert_eq!(t("stats.box_btn", Locale::Es), "Caja");
        assert_eq!(
            t("stats.box_hint", Locale::Es),
            "Crea BoxPlot[datos] en la entrada"
        );
        assert_eq!(t("stats.mean_btn", Locale::Es), "Media");
        assert_eq!(
            t("stats.mean_hint", Locale::Es),
            "Crea Mean[datos] en la entrada"
        );
        // Helper puro: Es espeja el canónico previo, resto localiza.
        assert_eq!(
            super::statistics_sorted_text(7, Locale::Es),
            ["Datos ordenados ", "(N=7)"].concat()
        );
        assert_eq!(
            super::statistics_sorted_text(7, Locale::En),
            "Data sorted (N=7)"
        );
        // Wiring: el draw usa las 10 keys vía lets con su locale.
        let source = include_str!("panels.rs");
        for pin in [
            "sort_section_label",
            "sort_btn_label",
            "sort_hint_label",
            "statistics_sorted_text(",
            "hist_btn_label",
            "hist_hint_label",
            "box_btn_label",
            "box_hint_label",
            "mean_btn_label",
            "mean_hint_label",
            "stats.sort_section",
            "stats.sorted_toast",
        ] {
            assert!(source.contains(pin), "stats B-A123 cablea {pin}");
        }
        // Blindaje: viejos literales ES duros del bloque ordenar ya no
        // están en el draw. (Fragmentos; splits distintos de los pines.
        // El header del gráfico resumen `:2675` reuse `hist_btn_label`,
        // cerrado en B-A124.)
        for needle in [
            ["draw_inspector_section(ui, \"Ordenar y grafic", "ar\""].concat(),
            ["RichText::new(\"Orden", "ar\")"].concat(),
            [
                "on_hover_text(\"Ordena ascendente (Sort) y actualiza el cam",
                "po\")",
            ]
            .concat(),
            ["format!(\n                                        \"Datos ordenados (N={})\","]
                .concat(),
            [
                "on_hover_text(\"Crea Histogram[datos, 10] en la ent",
                "rada\")",
            ]
            .concat(),
            ["on_hover_text(\"Crea BoxPlot[datos] en la ent", "rada\")"].concat(),
            ["on_hover_text(\"Crea Mean[datos] en la ent", "rada\")"].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "stats sin ES duro en prod (B-A123): {needle}"
            );
        }
    }

    #[test]
    fn stats_b_a124_resumen_asistente_sale_de_stats_msg() {
        // B-A124: header del resumen reuse `hist_btn_label`
        // (key existente B-A123, 0 keys) + bloque ask_* sale de
        // `stats.ask_*` con `stats_locale`. Sin egui: pines +
        // blindaje por `include_str!`.
        use grafito_ui::i18n::{t, Locale};
        // Concat a propósito: el blindaje de abajo usa `include_str!`
        // (incluye este test) y el literal completo se auto-matchearía
        // (precedente B-A106 `motion.toast_started`).
        assert_eq!(
            t("stats.ask_section", Locale::Es),
            ["Preguntar al asis", "tente"].concat()
        );
        assert_eq!(
            t("stats.ask_hint", Locale::Es),
            ["Tirá un CSV grande en Datos y pedí orden + aná", "lisis.",].concat()
        );
        assert_eq!(
            t("stats.ask_btn", Locale::Es),
            ["Ordenar con asis", "tente"].concat()
        );
        assert_eq!(
            t("stats.ask_btn_hint", Locale::Es),
            ["Arma el prompt con tus datos y abre el asis", "tente",].concat()
        );
        assert_eq!(t("stats.ask_section", Locale::En), "Ask the assistant");
        assert_eq!(t("stats.ask_btn", Locale::Pt), "Ordenar com assistente");
        assert_eq!(
            t("stats.ask_hint", Locale::It),
            "Trascina un CSV grande in Dati e chiedi ordinamento + analisi."
        );
        assert_eq!(
            t("stats.ask_btn_hint", Locale::Fr),
            "Prépare le prompt avec vos données et ouvre l'assistant"
        );
        assert_eq!(t("stats.ask_section", Locale::De), "Assistenten fragen");
        // Wiring: el draw usa las 4 keys vía lets con su locale + el
        // header del resumen reuse `hist_btn_label` (let B-A123).
        let source = include_str!("panels.rs");
        for pin in [
            "let ask_section_label = stats_t(\"stats.ask_section\"",
            "let ask_hint_label = stats_t(\"stats.ask_hint\"",
            "let ask_btn_label = stats_t(\"stats.ask_btn\"",
            "let ask_btn_hint_label = stats_t(\"stats.ask_btn_hint\"",
            "RichText::new(hist_btn_label)",
            "ask_section_label,",
            "ask_hint_label,",
            "panel_primary_button(ui, ask_btn_label)",
            ".on_hover_text(ask_btn_hint_label)",
            "stats.ask_section",
            "stats.ask_btn_hint",
        ] {
            assert!(source.contains(pin), "stats B-A124 cablea {pin}");
        }
        // Blindaje: viejos literales ES duros del resumen + bloque
        // asistente ya no están en el draw. (Fragmentos; splits distintos
        // de los pines, precedente B-A123.)
        for needle in [
            ["RichText::new(\"Histo", "grama\")"].concat(),
            ["\"Preguntar al asis", "tente\""].concat(),
            [
                "\"Tirá un CSV grande en Datos y pedí orden + aná",
                "lisis.\"",
            ]
            .concat(),
            ["panel_primary_button(ui, \"Ordenar con asis", "tente\")"].concat(),
            ["\"Arma el prompt con tus datos y abre el asis", "tente\""].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "stats sin ES duro en prod (B-A124): {needle}"
            );
        }
    }

    #[test]
    fn stats_b_a153_quick_toasts_salen_de_helpers_for() {
        // B-A153: toasts quick-result del panel stats salen de los
        // helpers `_for` (`sheet_layout.rs`, Es delega al canónico).
        // Sin egui: pines + blindaje por `include_str!`.
        use grafito_ui::i18n::{t, Locale};
        // Concat a propósito: el blindaje de abajo usa `include_str!`
        // (incluye este test) y el literal completo se auto-matchearía
        // (precedente B-A124).
        assert_eq!(
            t("stats.quick_histogram_ready", Locale::Es),
            ["Histograma listo en la entrada (Enter para ", "crear)."].concat()
        );
        assert_eq!(
            t("stats.quick_boxplot_ready", Locale::Es),
            ["Diagrama de caja listo ", "en la entrada."].concat()
        );
        assert_eq!(
            t("stats.quick_mean_ready", Locale::Es),
            ["Media lista ", "en la entrada."].concat()
        );
        assert_eq!(
            t("stats.quick_too_many", Locale::Es),
            [
                "Demasiados datos para la entrada (máx. 200 inline): ",
                "usá Ordenar con asistente."
            ]
            .concat()
        );
        assert_eq!(
            t("stats.quick_no_inline_result", Locale::Es),
            [
                "Sin lista inline (vacío o +200 valores): ",
                "pegá menos datos o usá el asistente."
            ]
            .concat()
        );
        assert_eq!(
            t("stats.quick_no_inline_hint", Locale::Es),
            ["Sin lista inline: ", "pegá menos datos o usá el asistente."].concat()
        );
        assert_eq!(
            t("stats.quick_histogram_ready", Locale::En),
            "Histogram ready in the input (Enter to create)."
        );
        assert_eq!(
            t("stats.quick_mean_ready", Locale::Pt),
            "Média pronta na entrada."
        );
        assert_eq!(
            t("stats.quick_boxplot_ready", Locale::It),
            "Box plot pronto nell'input."
        );
        assert_eq!(
            t("stats.quick_no_inline_hint", Locale::Fr),
            "Pas de liste inline : collez moins de données ou utilisez l'assistant."
        );
        assert_eq!(
            t("stats.quick_too_many", Locale::De),
            "Zu viele Daten für die Eingabe (max. 200 inline): nutze Sortieren mit Assistent."
        );
        // Wiring: el draw usa los 7 helpers `_for` con `stats_locale`
        // (pines por llamada: `fmt` parte las cadenas largas en varias
        // líneas, precedente B-A105/B-A109).
        let source = include_str!("panels.rs");
        for pin in [
            "statistics_histogram_ready_for(stats_locale)",
            "statistics_boxplot_ready_for(stats_locale)",
            "statistics_mean_ready_for(stats_locale)",
            "statistics_inline_too_many_for(stats_locale)",
            "statistics_quick_ready_text_for(",
            "statistics_no_inline_result_for(stats_locale)",
            "statistics_no_inline_hint_for(",
            "stats.quick_histogram_ready",
            "stats.quick_ready",
        ] {
            assert!(source.contains(pin), "stats B-A153 cablea {pin}");
        }
        // Blindaje: viejos literales ES duros de los toasts quick ya no
        // están en el draw (consts canónicas intactas en sheet_layout.rs,
        // pineadas por su test). Fragmentos con splits distintos de los
        // pines (precedente B-A124).
        for needle in [
            ["STATISTICS_HISTO", "GRAM_READY.to_string()"].concat(),
            ["STATISTICS_BOX", "PLOT_READY.to_string()"].concat(),
            ["STATISTICS_MEAN", "_READY.to_string()"].concat(),
            ["STATISTICS_INLINE_", "TOO_MANY.to_string()"].concat(),
            ["STATISTICS_NO_INLINE_", "RESULT.to_string()"].concat(),
            ["RichText::new(STATISTICS_NO_INLINE", "_HINT)"].concat(),
            ["statistics_quick_ready", "_text(label)"].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "stats sin ES duro en prod (B-A153): {needle}"
            );
        }
    }

    #[test]
    fn stats_b_a163_quick_chips_salen_de_stats_msg() {
        // B-A163: labels + tips de los 6 chips rápidos de inferencia salen
        // de `statistics_quick_chips_for` con `stats_locale`. Sin egui:
        // pines + blindaje por `include_str!`.
        use grafito_ui::i18n::{t, Locale};
        // Concat a propósito: el blindaje de abajo usa `include_str!`
        // (incluye este test) y el literal completo se auto-matchearía
        // (precedente B-A153).
        assert_eq!(
            t("stats.quick_chip_median", Locale::Es),
            ["Med", "iana"].concat()
        );
        assert_eq!(
            t("stats.quick_chip_stddev", Locale::Es),
            ["Des", "vío"].concat()
        );
        assert_eq!(
            t("stats.quick_tip_median", Locale::Es),
            ["Mediana de ", "tus datos"].concat()
        );
        assert_eq!(
            t("stats.quick_tip_ztest", Locale::Es),
            ["Prueba z (σ=1 cono", "cido)"].concat()
        );
        assert_eq!(t("stats.quick_chip_median", Locale::En), "Median");
        assert_eq!(t("stats.quick_tip_stddev", Locale::Pt), "Desvio padrão");
        assert_eq!(
            t("stats.quick_tip_freq", Locale::It),
            "Tabella di frequenza"
        );
        assert_eq!(
            t("stats.quick_tip_stem", Locale::Fr),
            "Diagramme tige-feuille"
        );
        assert_eq!(
            t("stats.quick_tip_ztest", Locale::De),
            "z-Test (σ=1 bekannt)"
        );
        // Wiring: el draw usa el helper `_for` con `stats_locale`.
        let source = include_str!("panels.rs");
        for pin in [
            "statistics_quick_chips_for(&list, stats_locale)",
            "stats.quick_chip_median",
            "stats.quick_tip_ztest",
        ] {
            assert!(source.contains(pin), "stats B-A163 cablea {pin}");
        }
        // Blindaje: el draw ya no usa la variante Es-only ni literales
        // crudos (la canónica queda en `sheet_layout.rs`, pineada por su
        // test). Fragmentos con splits distintos de los pines.
        // (Nota: `RichText::new(*label)` crudo existe en otros draws
        // genéricos —helper de grilla `:228` con `TYPE_XS` y hints
        // complejos `:3309`— así que las agujas llevan el contexto
        // `TYPE_SM`/`.clicked()` propio de los chips.)
        for needle in [
            ["statistics_quick_chips(&", "list)"].concat(),
            ["RichText::new(*label).size(", "TYPE_SM)"].concat(),
            ["on_hover_text(*tip)", ".clicked()"].concat(),
            ["\"Mediana de ", "tus datos\""].concat(),
            ["\"Prueba z (σ=1 cono", "cido)\""].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "stats sin ES duro en prod (B-A163): {needle}"
            );
        }
    }

    #[test]
    fn ctx_b_a145_menu_y_error_salen_de_ctx_msg() {
        // B-A145: menú contextual 2D (6 botones) + error de edición
        // (`ctx.edit_error`) salen de `ctx.*` con su locale. Sin egui:
        // pines + blindaje por `include_str!`.
        use grafito_ui::i18n::{t, Locale};
        // Concat a propósito: el blindaje de abajo usa `include_str!`
        // (incluye este test) y el literal completo se auto-matchearía
        // (precedente B-A133).
        assert_eq!(t("ctx.rename", Locale::Es), ["Renom", "brar"].concat());
        assert_eq!(t("ctx.copy", Locale::Es), ["Cop", "iar"].concat());
        assert_eq!(t("ctx.hide", Locale::Es), ["Ocul", "tar"].concat());
        assert_eq!(t("ctx.show", Locale::Es), ["Mos", "trar"].concat());
        assert_eq!(t("ctx.delete", Locale::Es), ["Elim", "inar"].concat());
        assert_eq!(
            t("ctx.properties", Locale::Es),
            ["Propied", "ades"].concat()
        );
        assert_eq!(
            t("ctx.edit_error", Locale::Es),
            ["Propiedades: ", "{error}"].concat()
        );
        assert_eq!(t("ctx.rename", Locale::En), "Rename");
        assert_eq!(t("ctx.delete", Locale::Pt), "Excluir");
        assert_eq!(t("ctx.properties", Locale::It), "Proprietà");
        assert_eq!(t("ctx.show", Locale::Fr), "Afficher");
        assert_eq!(t("ctx.hide", Locale::De), "Ausblenden");
        // Helper puro: template `{error}` interpola, Es delega al canónico.
        assert_eq!(
            crate::algebra::ctx_edit_error_text("boom", Locale::Es),
            ["Propiedades: ", "boom"].concat()
        );
        assert_eq!(
            crate::algebra::ctx_edit_error_text("boom", Locale::En),
            "Properties: boom"
        );
        // Wiring: menú con `ctx_locale`, ediciones vía helper (fragmentos:
        // no auto-match con el `include_str!`, precedente B-A97).
        let input = include_str!("input.rs");
        for pin in [
            ["t(\"ctx.ren", "ame\", ctx_locale)"].concat(),
            ["t(\"ctx.co", "py\", ctx_locale)"].concat(),
            ["t(\"ctx.hi", "de\", ctx_locale)"].concat(),
            ["t(\"ctx.sho", "w\", ctx_locale)"].concat(),
            ["t(\"ctx.del", "ete\", ctx_locale)"].concat(),
            ["t(\"ctx.proper", "ties\", ctx_locale)"].concat(),
        ] {
            assert!(input.contains(pin.as_str()), "ctx B-A145 cablea {pin}");
        }
        let algebra = include_str!("algebra.rs");
        assert!(
            algebra.contains(
                ["ctx_edit_error_text(&error, alg_lo", "cale)"]
                    .concat()
                    .as_str()
            ),
            "ctx B-A145 cablea algebra.rs:1548 vía helper"
        );
        let panels = include_str!("panels.rs");
        assert!(
            panels.contains(
                ["ctx_edit_error_text(&error, inspector_lo", "cale)"]
                    .concat()
                    .as_str()
            ),
            "ctx B-A145 cablea panels.rs:4456 vía helper"
        );
        // Blindaje: viejos literales ES duros del menú + `format!` directo
        // del error ya no están en los draws (el fallback ES del helper es
        // canónico intencional, patrón B-A64; la aguja exige el `= ` del
        // `let message =` para no colisionar con él).
        for needle in [
            ["ui.button(\"Reno", "mbrar\")"].concat(),
            ["ui.button(\"Cop", "iar\")"].concat(),
            ["ui.button(\"Ocul", "tar\")"].concat(),
            ["ui.button(\"Mos", "trar\")"].concat(),
            ["ui.button(\"Elim", "inar\")"].concat(),
            ["ui.button(\"Propied", "ades\")"].concat(),
            ["= format!(\"Propiedades: ", "{error}\");"].concat(),
        ] {
            assert!(
                !input.contains(needle.as_str())
                    && !algebra.contains(needle.as_str())
                    && !panels.contains(needle.as_str()),
                "ctx sin ES duro en prod (B-A145): {needle}"
            );
        }
    }

    #[test]
    fn ctx_b_a174_menu_3d_sale_de_ctx_msg() {
        // B-A174: menú contextual 3D (2 botones) sale de `ctx.*` con su
        // locale. Sin egui: pines + blindaje por `include_str!` sobre
        // `input.rs` (este test vive en `panels.rs`, sin auto-match).
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(t("ctx.clear_selection", Locale::Es), "Borrar selección");
        assert_eq!(t("ctx.reset_view", Locale::Es), "Reiniciar vista");
        assert_eq!(t("ctx.clear_selection", Locale::En), "Clear selection");
        assert_eq!(t("ctx.reset_view", Locale::En), "Reset view");
        assert_eq!(t("ctx.clear_selection", Locale::Pt), "Limpar seleção");
        assert_eq!(t("ctx.reset_view", Locale::De), "Ansicht zurücksetzen");
        // Wiring: menú 3D con `ctx_locale` (fragmentos: el blindaje de abajo
        // usa `include_str!("input.rs")`, precedente B-A145).
        let input = include_str!("input.rs");
        for pin in [
            ["t(\"ctx.clear_sele", "ction\", ctx_locale)"].concat(),
            ["t(\"ctx.reset_vi", "ew\", ctx_locale)"].concat(),
        ] {
            assert!(input.contains(pin.as_str()), "ctx B-A174 cablea {pin}");
        }
        // Blindaje: viejos literales ES duros ya no están en el draw 3D.
        for needle in [
            ["ui.button(\"Borrar sele", "cción\")"].concat(),
            ["ui.button(\"Reiniciar vi", "sta\")"].concat(),
        ] {
            assert!(
                !input.contains(needle.as_str()),
                "ctx sin ES duro en prod (B-A174): {needle}"
            );
        }
    }

    #[test]
    fn ctx_b_a175_canvas_a11y_sale_de_ctx_msg() {
        // B-A175: widget_info del lienzo (lector WCAG 4.1.2) sale de `ctx.*`
        // con su locale. Sin egui: pines + blindaje por `include_str!` sobre
        // `input.rs` (este test vive en `panels.rs`, sin auto-match).
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(t("ctx.canvas_empty", Locale::Es), "Lienzo de dibujo");
        assert_eq!(
            t("ctx.canvas_with_selection", Locale::Es),
            ["Lienzo de dibujo. Hay un objeto sele", "ccionado"].concat()
        );
        assert_eq!(t("ctx.canvas_empty", Locale::En), "Drawing canvas");
        assert_eq!(t("ctx.canvas_empty", Locale::Pt), "Tela de desenho");
        assert_eq!(t("ctx.canvas_empty", Locale::De), "Zeichenfläche");
        // Wiring: lienzo con `canvas_locale` (fragmentos: el blindaje de abajo
        // usa `include_str!("input.rs")`, precedente B-A145/B-A174).
        let input = include_str!("input.rs");
        for pin in [
            ["t(\"ctx.canvas_em", "pty\", canvas_locale)"].concat(),
            ["t(\"ctx.canvas_with_sele", "ction\", canvas_locale)"].concat(),
        ] {
            assert!(input.contains(pin.as_str()), "ctx B-A175 cablea {pin}");
        }
        // Blindaje: viejos literales ES duros ya no están en `input.rs`
        // (`include_str!` solo lee `input.rs`, así que los pines ES de este
        // test no auto-matchean).
        for needle in [
            ["\"Lienzo de dibujo. Hay un objeto sele", "ccionado\""].concat(),
            ["\"Lienzo de ", "dibujo\""].concat(),
        ] {
            // El segundo fragmento también vive en el pin ES del test de
            // arriba, pero `include_str!` solo lee `input.rs`: cuenta 0.
            let count = input.matches(needle.as_str()).count();
            assert_eq!(count, 0, "ctx sin ES duro en prod (B-A175): {needle}");
        }
    }

    #[test]
    fn stats_b_a133_params_resultado_salen_de_stats_msg() {
        // B-A133: headers Parámetros/Resultado del panel prob salen de
        // `stats.prob_params_title/result_title` con `prob_locale`. Sin
        // egui: pines + blindaje por `include_str!`.
        use grafito_ui::i18n::{t, Locale};
        // Concat a propósito: el blindaje de abajo usa `include_str!`
        // (incluye este test) y el literal completo se auto-matchearía
        // (precedente B-A124).
        assert_eq!(
            t("stats.prob_params_title", Locale::Es),
            ["Pará", "metros"].concat()
        );
        assert_eq!(
            t("stats.prob_result_title", Locale::Es),
            ["Resul", "tado"].concat()
        );
        assert_eq!(t("stats.prob_params_title", Locale::En), "Parameters");
        assert_eq!(t("stats.prob_params_title", Locale::Pt), "Parâmetros");
        assert_eq!(t("stats.prob_result_title", Locale::It), "Risultato");
        assert_eq!(t("stats.prob_params_title", Locale::Fr), "Paramètres");
        assert_eq!(t("stats.prob_result_title", Locale::De), "Ergebnis");
        // Wiring: el draw usa las 2 keys vía lets con su locale.
        let source = include_str!("panels.rs");
        for pin in [
            "let prob_params_title = stats_t(\"stats.prob_params_title\"",
            "let prob_result_title = stats_t(\"stats.prob_result_title\"",
            "RichText::new(prob_params_title)",
            "RichText::new(prob_result_title)",
            "stats.prob_params_title",
            "stats.prob_result_title",
        ] {
            assert!(source.contains(pin), "stats B-A133 cablea {pin}");
        }
        // Blindaje: viejos literales ES duros ya no están en el draw.
        // (Fragmentos; splits distintos de los pines, precedente B-A124.)
        for needle in [
            ["RichText::new(\"Pará", "metros\")"].concat(),
            ["RichText::new(\"Resul", "tado\")"].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "stats sin ES duro en prod (B-A133): {needle}"
            );
        }
    }

    #[test]
    fn stats_b_a134_inferencia_sale_de_stats_msg() {
        // B-A134: header Inferencia del panel stats sale de
        // `stats.inference_title` con `stats_locale`. Sin egui: pines +
        // blindaje por `include_str!`.
        use grafito_ui::i18n::{t, Locale};
        // Concat a propósito: el blindaje de abajo usa `include_str!`
        // (incluye este test) y el literal completo se auto-matchearía
        // (precedente B-A124/B-A133).
        assert_eq!(
            t("stats.inference_title", Locale::Es),
            ["Infere", "ncia"].concat()
        );
        assert_eq!(t("stats.inference_title", Locale::En), "Inference");
        assert_eq!(t("stats.inference_title", Locale::Pt), "Inferência");
        assert_eq!(t("stats.inference_title", Locale::It), "Inferenza");
        assert_eq!(t("stats.inference_title", Locale::Fr), "Inférence");
        assert_eq!(t("stats.inference_title", Locale::De), "Inferenz");
        // Wiring: el draw usa la key vía let con su locale.
        let source = include_str!("panels.rs");
        for pin in [
            "let inference_title_label = stats_t(\"stats.inference_title\"",
            "draw_inspector_section(ui, inference_title_label,",
            "stats.inference_title",
        ] {
            assert!(source.contains(pin), "stats B-A134 cablea {pin}");
        }
        // Blindaje: viejo literal ES duro ya no está en el draw.
        // (Fragmento; split distinto del pin, precedente B-A124.)
        let needle = ["draw_inspector_section(ui, \"Infere", "ncia\","].concat();
        assert!(
            !source.contains(needle.as_str()),
            "stats sin ES duro en prod (B-A134): {needle}"
        );
    }

    #[test]
    fn stats_b_a147_regression_chrome_sale_de_stats_msg() {
        // B-A147: chrome estático del panel Regresión sale de
        // `stats.regression_*` con `regression_locale`
        // (`app.config_locale()`, sin cambio de firma, patrón
        // B-A119/B-A120). El botón Importar de `draw_spreadsheet_section`
        // (`spreadsheet.rs`) reutiliza `regression_import_label/hover`.
        // Sin egui: pines + blindaje por `include_str!`.
        use grafito_ui::i18n::{t, Locale};
        // Concat a propósito: el blindaje de abajo usa `include_str!`
        // (incluye este test) y el literal completo se auto-matchearía
        // (precedente B-A124/B-A133).
        assert_eq!(
            t("stats.regression_panel_title", Locale::Es),
            ["Regre", "sión"].concat()
        );
        assert_eq!(
            t("stats.regression_fits_empty", Locale::Es),
            ["Sin ajustes toda", "vía."].concat()
        );
        assert_eq!(
            t("stats.regression_diagnostics_title", Locale::Es),
            ["Diagnós", "ticos"].concat()
        );
        assert_eq!(
            t("stats.regression_create_title", Locale::Es),
            ["Crear aná", "lisis"].concat()
        );
        assert_eq!(
            t("stats.regression_import_label", Locale::Es),
            ["Importar CSV/", "TSV…"].concat()
        );
        assert_eq!(
            t("stats.regression_no_table_guide", Locale::Es),
            [
                "Seleccioná una tabla local en el lienzo para elegir un modelo de ",
                "ajuste."
            ]
            .concat()
        );
        assert_eq!(t("stats.regression_panel_title", Locale::En), "Regression");
        assert_eq!(
            t("stats.regression_fits_empty", Locale::Pt),
            "Sem ajustes ainda."
        );
        assert_eq!(
            t("stats.regression_create_title", Locale::It),
            "Crea analisi"
        );
        assert_eq!(
            t("stats.regression_diagnostics_title", Locale::Fr),
            "Diagnostics"
        );
        assert_eq!(
            t("stats.regression_import_label", Locale::De),
            "CSV/TSV importieren…"
        );
        // Wiring: el draw usa las 12 keys vía `regression_chrome_for`
        // con su locale (patrón B-A128).
        let source = include_str!("panels.rs");
        for pin in [
            "let regression_panel_title = crate::sheet_layout::regression_chrome_for(",
            "\"regression_panel_title\",",
            "\"regression_panel_subtitle\",",
            "\"regression_fits_title\",",
            "\"regression_fits_empty\",",
            "\"regression_diagnostics_title\",",
            "\"regression_diagnostics_subtitle\",",
            "\"regression_create_title\",",
            "\"regression_create_subtitle\",",
            "\"regression_import_label\",",
            "\"regression_import_hover\",",
            "\"regression_model_hint\",",
            "\"regression_no_table_guide\",",
            "draw_right_drawer_header(ui, app, regression_panel_title, regression_panel_subtitle,",
            "regression_fits_title,",
            "regression_diagnostics_title,",
            "regression_create_title,",
            "panel_wide_action_button(ui, regression_import_label)",
            ".on_hover_text(regression_import_hover)",
            "regression_model_hint,",
            "regression_no_table_guide,",
        ] {
            assert!(source.contains(pin), "stats B-A147 cablea {pin}");
        }
        // Blindaje: las 12 consts REGRESSION_* ya no están en el draw
        // (imports podados; consts canónicas intactas en sheet_layout.rs,
        // pineadas por su test). Splits distintos de cualquier otro pin
        // para que `include_str!` no se auto-matchee (precedente B-A124).
        for needle in [
            ["REGRESSION_PANEL_T", "ITLE"].concat(),
            ["REGRESSION_PANEL_SUBT", "ITLE"].concat(),
            ["REGRESSION_FITS_T", "ITLE"].concat(),
            ["REGRESSION_FITS_E", "MPTY"].concat(),
            ["REGRESSION_DIAGNOSTICS_T", "ITLE"].concat(),
            ["REGRESSION_DIAGNOSTICS_SUBT", "ITLE"].concat(),
            ["REGRESSION_CREATE_T", "ITLE"].concat(),
            ["REGRESSION_CREATE_SUBT", "ITLE"].concat(),
            ["REGRESSION_IMPORT_L", "ABEL"].concat(),
            ["REGRESSION_IMPORT_H", "OVER"].concat(),
            ["REGRESSION_MODEL_H", "INT"].concat(),
            ["REGRESSION_NO_TABLE_G", "UIDE"].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "stats sin ES duro en prod (B-A147): {needle}"
            );
        }
    }

    #[test]
    fn stats_b_a148_regression_dinamicos_salen_de_helpers_for() {
        // B-A148: dinámicos del panel Regresión salen de los helpers
        // `*_for` de `sheet_layout.rs` con `regression_locale`
        // (`app.config_locale()`, sin cambio de firma, patrón B-A147).
        // `regression_fit_headline` ("{label} · {kind}") y
        // `regression_residual_line` ("r{i} = …") quedan invariantes a
        // propósito (notación, precedentes B-A98/B-A114). Sin egui: pines
        // + blindaje por `include_str!`.
        use grafito_ui::i18n::{t, Locale};
        // Concat a propósito: el blindaje de abajo usa `include_str!`
        // (incluye este test) y el literal completo se auto-matchearía
        // (precedente B-A124/B-A133/B-A147).
        assert_eq!(
            t("stats.regression_deleted_source", Locale::Es),
            ["tabla elimi", "nada"].concat()
        );
        assert_eq!(
            t("stats.regression_diagnostics_line", Locale::Es),
            ["sobre {origen} · RMSE {rmse} · ", "R² {r2}"].concat()
        );
        assert_eq!(
            t("stats.regression_residuals_title", Locale::Es),
            ["Residuales (", "{n})"].concat()
        );
        assert_eq!(
            t("stats.regression_section_title", Locale::Es),
            ["Ajustar «{", "tabla}»"].concat()
        );
        assert_eq!(
            t("stats.regression_template_scatter", Locale::Es),
            ["Disper", "sión"].concat()
        );
        assert_eq!(
            t("stats.spreadsheet_import_title", Locale::Es),
            ["Importar y ", "ordenar"].concat()
        );
        assert_eq!(
            t("stats.regression_diagnostics_line", Locale::En),
            "from {origen} · RMSE {rmse} · R² {r2}"
        );
        assert_eq!(
            t("stats.regression_residuals_truncated", Locale::Pt),
            "Mostrando os primeiros {max} valores."
        );
        assert_eq!(
            t("stats.regression_model_ready", Locale::De),
            "Anpassung {model} bereit in der Eingabe (Enter zum Erstellen)."
        );
        // Wiring: el draw usa los helpers `_for` con su locale.
        let source = include_str!("panels.rs");
        for pin in [
            "regression_source_label_for(",
            "regression_fit_diagnostics_line_for(",
            "regression_residuals_title_for(",
            "regression_residuals_truncated_for(",
            "let regression_templates =",
            "regression_templates_for(regression_locale)",
            "regression_templates[index].0,",
            "regression_templates[index].1",
            "regression_template_ready_text_for(",
            "regression_section_title_for(",
            "regression_model_ready_text_for(",
        ] {
            assert!(source.contains(pin), "stats B-A148 cablea {pin}");
        }
        // Blindaje: las llamadas Es-only ya no están en el draw (delegan
        // en los helpers; canónicos intactos en sheet_layout.rs, pineados
        // por su test). Splits para que `include_str!` no se auto-matchee:
        // el `_for` vecino no contiene la aguja con `(` (precedente B-A89).
        for needle in [
            ["regression_source_la", "bel("].concat(),
            ["regression_fit_diagnostics_li", "ne("].concat(),
            ["regression_residuals_ti", "tle("].concat(),
            ["regression_residuals_truncated_no", "te("].concat(),
            ["regression_section_ti", "tle("].concat(),
            ["regression_template_ready_te", "xt("].concat(),
            ["regression_model_ready_te", "xt("].concat(),
            ["REGRESSION_TEMP", "LATES"].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "stats sin ES duro en prod (B-A148): {needle}"
            );
        }
    }

    #[test]
    fn stats_b_a149_modelos_salen_de_commands_for() {
        // B-A149: etiquetas de los 7 modelos del grid Regresión salen de
        // `regression_model_commands_for` con `regression_locale`
        // (`app.config_locale()`, sin cambio de firma, patrón B-A147/B-A148).
        // Comandos `Fit*` invariantes (precedente B-A98). Sin egui: pines +
        // blindaje por `include_str!`.
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(t("stats.regression_model_linear", Locale::Es), "Lineal");
        assert_eq!(
            t("stats.regression_model_poly2", Locale::Es),
            "Polinómico 2"
        );
        assert_eq!(
            t("stats.regression_model_logistic", Locale::Es),
            "Logística"
        );
        assert_eq!(t("stats.regression_model_poly2", Locale::En), "Quadratic");
        assert_eq!(t("stats.regression_model_power", Locale::Pt), "Potência");
        assert_eq!(
            t("stats.regression_model_logistic", Locale::De),
            "Logistisch"
        );
        // Wiring: el draw usa `regression_model_commands_for` con su locale
        // y el toast ya interpolaba la etiqueta display (`_for`, B-A148).
        let source = include_str!("panels.rs");
        for pin in [
            "regression_model_commands_for(",
            "regression_model_ready_text_for(",
        ] {
            assert!(source.contains(pin), "stats B-A149 cablea {pin}");
        }
        // Blindaje: la llamada Es-only con la etiqueta del draw ya no está
        // (delega en el helper; canónicos intactos en sheet_layout.rs,
        // pineados por su test). Split para que `include_str!` no se
        // auto-matchee (precedente B-A89).
        let needle = ["regression_model_commands", "(&table_label)"].concat();
        assert!(
            !source.contains(needle.as_str()),
            "stats sin ES duro en prod (B-A149): {needle}"
        );
    }

    #[test]
    fn view_b_a128_general_ejes_sale_de_view_chrome_for() {
        // B-A128: chrome General+Ejes del panel Vista sale del dispatcher
        // `view_chrome_for` con `view_locale`. Sin egui: pines +
        // blindaje por `include_str!`.
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(t("view.general_title", Locale::Es), "General");
        // Concat a propósito: el blindaje de abajo usa `include_str!`
        // (incluye este test) y el literal completo se auto-matchearía
        // (precedente B-A106 `motion.toast_started`).
        assert_eq!(
            t("view.show_grid", Locale::Es),
            ["Mostrar cuadr", "ícula"].concat()
        );
        assert_eq!(t("view.dark_mode", Locale::Es), "Modo oscuro");
        assert_eq!(
            t("view.snap", Locale::Es),
            ["Ajustar a cuadr", "ícula"].concat()
        );
        assert_eq!(t("view.axes_title", Locale::Es), "Ejes");
        assert_eq!(
            t("view.number_plane_hint", Locale::Es),
            [
                "Muestra los números de los ejes con pasos lindos 1/2/5×10^n",
                " y skip anti-solape",
            ]
            .concat()
        );
        assert_eq!(t("view.general_title", Locale::En), "General");
        assert_eq!(t("view.show_grid", Locale::Pt), "Mostrar grade");
        assert_eq!(t("view.dark_mode", Locale::It), "Modalità scura");
        assert_eq!(t("view.snap", Locale::Fr), "Aligner sur la grille");
        assert_eq!(t("view.axes_title", Locale::De), "Achsen");
        // Wiring: el draw usa las 9 keys vía el dispatcher con su locale.
        let source = include_str!("panels.rs");
        for pin in [
            "let view_locale = app.config_locale()",
            "view_chrome_for(\"general_title\", view_locale)",
            "view_chrome_for(\"show_grid\", view_locale)",
            "view_chrome_for(\"dark_mode\", view_locale)",
            "view_chrome_for(\"snap\", view_locale)",
            "view_chrome_for(\"axes_title\", view_locale)",
            "view_chrome_for(\"x_log\", view_locale)",
            "view_chrome_for(\"y_log\", view_locale)",
            "view_chrome_for(\"number_plane\", view_locale)",
            "view_chrome_for(\"number_plane_hint\", view_locale)",
            "draw_inspector_section(ui, general_title,",
            "draw_inspector_section(ui, axes_title,",
            "show_grid_label,",
            "dark_mode_label,",
            "snap_label,",
            "x_log_label,",
            "y_log_label,",
            "number_plane_label,",
            "number_plane_hint,",
        ] {
            assert!(source.contains(pin), "vista B-A128 cablea {pin}");
        }
        // Blindaje: las 9 consts crudas ya no están en el draw.
        // (Fragmentos; splits distintos de los pines, precedente B-A123.)
        for needle in [
            ["VIEW_GENERAL", "_TITLE"].concat(),
            ["VIEW_SHOW_GRID", "_LABEL"].concat(),
            ["VIEW_DARK_MODE", "_LABEL"].concat(),
            ["VIEW_SNAP", "_LABEL"].concat(),
            ["VIEW_AXES", "_TITLE"].concat(),
            ["VIEW_X_LOG", "_LABEL"].concat(),
            ["VIEW_Y_LOG", "_LABEL"].concat(),
            ["VIEW_NUMBER_PLANE", "_LABEL"].concat(),
            ["VIEW_NUMBER_PLANE", "_HINT"].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "vista sin ES duro en prod (B-A128): {needle}"
            );
        }
    }

    #[test]
    fn export_b_a169_archivo_sale_de_export_chrome_for() {
        // B-A169: chrome archivo de la sección Exportar sale del dispatcher
        // `export_chrome_for` con `export_locale`. Sin egui: pines +
        // blindaje por `include_str!`. Siglas SVG/PNG/TikZ/PDF intactas.
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(t("export.section_title", Locale::Es), "Exportación");
        assert_eq!(
            t("export.section_subtitle", Locale::Es),
            "Exportá a archivo (SVG/PNG/TikZ/PDF) o copiá al portapapeles."
        );
        assert_eq!(
            t("export.file_svg_tip", Locale::Es),
            "Exportar el lienzo a un archivo SVG vectorial"
        );
        assert_eq!(t("export.file_copy_svg", Locale::Es), "Copiar SVG");
        assert_eq!(t("export.file_copy_png", Locale::Es), "Copiar PNG");
        assert_eq!(t("export.section_title", Locale::En), "Export");
        assert_eq!(t("export.file_copy_svg", Locale::Pt), "Copiar SVG");
        assert_eq!(t("export.file_copy_png", Locale::It), "Copia PNG");
        assert_eq!(t("export.section_title", Locale::Fr), "Exportation");
        assert_eq!(t("export.file_copy_png", Locale::De), "PNG kopieren");
        // Wiring: el draw usa las 10 keys vía el dispatcher con su locale.
        let source = include_str!("panels.rs");
        for pin in [
            "let export_locale = app.config_locale()",
            "export_chrome_for(\"section_title\", export_locale)",
            "export_chrome_for(\"section_subtitle\", export_locale)",
            "export_chrome_for(\"file_svg_tip\", export_locale)",
            "export_chrome_for(\"file_png_tip\", export_locale)",
            "export_chrome_for(\"file_tikz_tip\", export_locale)",
            "export_chrome_for(\"file_pdf_tip\", export_locale)",
            "export_chrome_for(\"file_copy_svg\", export_locale)",
            "export_chrome_for(\"file_copy_svg_tip\", export_locale)",
            "export_chrome_for(\"file_copy_png\", export_locale)",
            "export_chrome_for(\"file_copy_png_tip\", export_locale)",
            "draw_inspector_section(ui, export_section_title,",
            "action(ui, export_copy_svg, export_copy_svg_tip)",
            "action(ui, export_copy_png, export_copy_png_tip)",
        ] {
            assert!(source.contains(pin), "export B-A169 cablea {pin}");
        }
        // Blindaje: las consts crudas ya no están en el draw (solo quedan
        // las siglas `[0..3].0` + import de `EXPORT_FILE_BUTTONS`).
        // (Fragmentos; splits distintos de los pines, precedente B-A123.)
        for needle in [
            ["EXPORT_SECTION", "_TITLE"].concat(),
            ["EXPORT_SECTION", "_SUBTITLE"].concat(),
            ["EXPORT_FILE_BUTTONS[0]", ".1"].concat(),
            ["EXPORT_FILE_BUTTONS[1]", ".1"].concat(),
            ["EXPORT_FILE_BUTTONS[2]", ".1"].concat(),
            ["EXPORT_FILE_BUTTONS[3]", ".1"].concat(),
            ["EXPORT_FILE_BUTTONS[4]", ".0"].concat(),
            ["EXPORT_FILE_BUTTONS[4]", ".1"].concat(),
            ["EXPORT_FILE_BUTTONS[5]", ".0"].concat(),
            ["EXPORT_FILE_BUTTONS[5]", ".1"].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "export sin ES duro en prod (B-A169): {needle}"
            );
        }
    }

    #[test]
    fn cas_b_a177_panel_sale_de_cas_chrome_for() {
        // B-A177: chrome estático del panel CAS sale del dispatcher
        // `cas_chrome_for` con `cas_locale` + helpers `cas_quick_label_for`/
        // `cas_quick_insert_tooltip_for`/`cas_sheet_title_for`. Sin egui:
        // pines + blindaje por `include_str!`. Notación `x^2, x`/`↵  •  Tab`
        // + `cas_cell_number`/`cas_cell_input_line` + snippets invariantes.
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(t("cas.panel_title", Locale::Es), "Cálculo Simbólico (CAS)");
        assert_eq!(t("cas.clear_label", Locale::Es), "Limpiar");
        assert_eq!(
            t("cas.clear_tooltip", Locale::Es),
            "Eliminar las celdas CAS guardadas"
        );
        assert_eq!(t("cas.entry_label", Locale::Es), "Entrada");
        assert_eq!(t("cas.execute_tooltip", Locale::Es), "Ejecutar");
        assert_eq!(t("cas.quick_section_title", Locale::Es), "Acciones rápidas");
        assert_eq!(
            t("cas.quick_section_subtitle", Locale::Es),
            "Atajos — inserta sintaxis mínima."
        );
        assert_eq!(t("cas.quick_derive", Locale::Es), "Derivar");
        assert_eq!(t("cas.quick_limit", Locale::De), "Limes");
        assert_eq!(t("cas.empty_title", Locale::Es), "Sin cálculos aún");
        assert_eq!(
            t("cas.empty_hint", Locale::Es),
            "Escribí arriba y pulsá Enter."
        );
        assert_eq!(
            t("cas.empty_example", Locale::Es),
            "Ej: Derivative[x^2, x]  ·  Solve[x^2-4, x]"
        );
        assert_eq!(t("cas.sheet_title", Locale::En), "Sheet · {count}");
        // Wiring: el draw usa las 16 keys vía dispatcher/helpers con su locale.
        let source = include_str!("panels.rs");
        for pin in [
            "let cas_locale = app.config_locale()",
            "let cas_panel_title = cas_chrome_for(\"panel_title\", cas_locale)",
            "let cas_clear_label = cas_chrome_for(\"clear_label\", cas_locale)",
            "let cas_clear_tooltip = cas_chrome_for(\"clear_tooltip\", cas_locale)",
            "let cas_quick_section_title",
            "let cas_quick_section_subtitle",
            "let cas_entry_label = cas_chrome_for(\"entry_label\", cas_locale)",
            "let cas_execute_tooltip",
            "let cas_empty_title = cas_chrome_for(\"empty_title\", cas_locale)",
            "let cas_empty_hint = cas_chrome_for(\"empty_hint\", cas_locale)",
            "let cas_empty_example",
            "cas_quick_label_for(i, cas_locale)",
            "cas_quick_insert_tooltip_for(",
            "cas_sheet_title_for(",
            "RichText::new(cas_panel_title)",
            "small_button(cas_clear_label)",
            "RichText::new(cas_empty_title)",
        ] {
            assert!(source.contains(pin), "CAS B-A177 cablea {pin}");
        }
        // Blindaje: las consts crudas ya no están en el draw (solo quedan
        // `CAS_ENTRY_HINT`/`CAS_ENTRY_KEYS_HINT` invariantes + el import de
        // `CAS_QUICK_ACTIONS` para el loop por índice).
        // (Fragmentos; splits distintos de los pines, precedente B-A123.)
        for needle in [
            ["CAS_PANEL", "_TITLE"].concat(),
            ["CAS_CLEAR", "_LABEL"].concat(),
            ["CAS_CLEAR", "_TOOLTIP"].concat(),
            ["CAS_ENTRY", "_LABEL"].concat(),
            ["CAS_EXECUTE", "_TOOLTIP"].concat(),
            ["CAS_QUICK_SECTION", "_TITLE"].concat(),
            ["CAS_QUICK_SECTION", "_SUBTITLE"].concat(),
            ["CAS_EMPTY", "_TITLE"].concat(),
            ["CAS_EMPTY", "_HINT"].concat(),
            ["CAS_EMPTY", "_EXAMPLE"].concat(),
            ["for (label, snippet) in CAS_QUICK", "_ACTIONS"].concat(),
            ["insert_tooltip(snippet", ")"].concat(),
            ["sheet_title(\n", ""].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "CAS sin ES duro en prod (B-A177): {needle}"
            );
        }
    }

    #[test]
    fn dynamics_b_a189_panels_salen_de_dynamics_chrome_for() {
        // B-A189: chrome estático de Atractores + Parámetros sale del
        // dispatcher `dynamics_chrome_for` con `attractor_locale`/
        // `params_locale`. Sin egui: pines + blindaje por `include_str!`.
        // `ATTRACTOR_DEFAULT_COMMAND` (sintaxis) + `attractor_*_text`
        // (notación) + sliders `σ/ρ/β/dt` intactos a propósito (B-A190 los
        // pinnea como invariantes); el error `params_error_text` se
        // localizó en B-A190 vía `params_error_text_for`.
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(
            t("dynamics.attractor_panel_title", Locale::Es),
            "Dinámica y Atractores"
        );
        assert_eq!(
            t("dynamics.attractor_objects_empty", Locale::Es),
            ["Sin objetos dinámicos.\n", "Probá Attractor[10, 28, 8/3]."].concat()
        );
        assert_eq!(
            t("dynamics.attractor_inactive_guide", Locale::Es),
            [
                "Sin attractor activo.\n",
                "Creá uno con:\n  Attractor[σ, ρ, β]\n",
                "(Lorenz por defecto)",
            ]
            .concat()
        );
        assert_eq!(
            t("dynamics.params_empty_guide", Locale::Es),
            [
                "Sin attractor activo. ",
                "Creá uno con:\n  Attractor[10, 28, 8/3]"
            ]
            .concat()
        );
        assert_eq!(t("dynamics.params_steps_label", Locale::En), "steps");
        // Wiring: los draws usan las 15 keys vía el dispatcher con su locale.
        let source = include_str!("panels.rs");
        for pin in [
            "let attractor_locale = app.config_locale()",
            "let attractor_panel_title = dynamics_chrome_for(\"attractor_panel_title\", attractor_locale)",
            "let attractor_objects_title",
            "let attractor_objects_empty",
            "let attractor_active_title",
            "let attractor_inactive_guide",
            "let attractor_create_default",
            "let attractor_commands_title",
            "let attractor_lorenz_hint",
            "let params_locale = app.config_locale()",
            "let params_panel_title = dynamics_chrome_for(\"params_panel_title\", params_locale)",
            "let params_panel_subtitle",
            "let params_empty_guide",
            "let params_lorenz_section",
            "let params_integration_section",
            "let params_regen_hint",
            "let params_steps_label",
            "RichText::new(attractor_panel_title)",
            "RichText::new(params_lorenz_section)",
            ".text(params_steps_label)",
        ] {
            assert!(source.contains(pin), "Dynamics B-A189 cablea {pin}");
        }
        // Blindaje: las consts crudas ya no están en el draw (solo queda
        // `ATTRACTOR_DEFAULT_COMMAND` sintaxis + el comentario B-A189).
        // (Fragmentos; splits distintos de los pines, precedente B-A123.)
        for needle in [
            ["ATTRACTOR_PANEL", "_TITLE"].concat(),
            ["ATTRACTOR_OBJECTS", "_TITLE"].concat(),
            ["ATTRACTOR_OBJECTS", "_EMPTY"].concat(),
            ["ATTRACTOR_ACTIVE", "_TITLE"].concat(),
            ["ATTRACTOR_INACTIVE", "_GUIDE"].concat(),
            ["ATTRACTOR_CREATE_DEFAULT", "_LABEL"].concat(),
            ["ATTRACTOR_COMMANDS", "_TITLE"].concat(),
            ["ATTRACTOR_LORENZ", "_HINT"].concat(),
            ["PARAMS_PANEL", "_TITLE"].concat(),
            ["PARAMS_PANEL", "_SUBTITLE"].concat(),
            ["PARAMS_EMPTY", "_GUIDE"].concat(),
            ["PARAMS_LORENZ", "_SECTION"].concat(),
            ["PARAMS_INTEGRATION", "_SECTION"].concat(),
            ["PARAMS_REGEN", "_HINT"].concat(),
            [".text(\"", "pasos\")"].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "Dynamics sin ES duro en prod (B-A189): {needle}"
            );
        }
    }

    #[test]
    fn dynamics_b_a190_error_e_invariantes() {
        // B-A190: el error del commit sale de `params_error_text_for`
        // con `params_locale` (template `{error}` ×6); sintaxis
        // `ATTRACTOR_DEFAULT_COMMAND` + lecturas `attractor_*_text` +
        // sliders `σ/ρ/β/dt` quedan crudos a propósito (comando/notación
        // SI, precedentes B-A98/B-A114/B-A162). Sin egui: pines + wiring
        // + blindaje por `include_str!` (fragmentos anti-auto-match,
        // precedente B-A89/B-A105).
        use crate::sheet_layout::params_error_text_for;
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(
            t("dynamics.params_error", Locale::Es),
            ["Parámetros: ", "{error}"].concat()
        );
        assert_eq!(
            params_error_text_for("boom", Locale::Es),
            "Parámetros: boom"
        );
        assert_eq!(
            params_error_text_for("boom", Locale::En),
            "Parameters: boom"
        );
        assert_eq!(
            params_error_text_for("boom", Locale::Fr),
            ["Paramètres : ", "boom"].concat()
        );
        // Wiring: el draw del commit usa el helper con su locale.
        let source = include_str!("panels.rs");
        for pin in [
            "let params_locale = app.config_locale()",
            "params_error_text_for(error, params_locale)",
            "ATTRACTOR_DEFAULT_COMMAND",
            "attractor_sigma_text(sigma)",
            ".text(\"",
        ] {
            assert!(source.contains(pin), "Dynamics B-A190 cablea {pin}");
        }
        // Blindaje: la llamada vieja sin locale ya no está en prod (el
        // helper `_for` no matchea por el `_` intermedio). Assert directo:
        // 1 aguja en `for` rompe `single_element_loop` (precedente B-A134).
        let needle = ["params_error_text", "("].concat();
        assert!(
            !source.contains(needle.as_str()),
            "Dynamics sin llamada vieja (B-A190): {needle}"
        );
    }

    #[test]
    fn export_b_a170_resto_sale_de_export_chrome_for() {
        // B-A170: resto de la sección Exportar (texto + tablas + notifys +
        // reveal + fila) sale del dispatcher `export_chrome_for` con
        // `export_locale` + helper `export_table_row_label_for`. Sin egui:
        // pines + blindaje por `include_str!`. Labels `MathML`/`TikZ-eje`/
        // `HTML`/`CSV` siglas intactas (precedente B-A169).
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(
            t("export.text_mathml_tip", Locale::Es),
            [
                "Copia las funciones y puntos ",
                "como MathML al portapapeles"
            ]
            .concat()
        );
        assert_eq!(
            t("export.tables_title", Locale::Es),
            ["Tablas (CSV ", "RFC 4180)"].concat()
        );
        assert_eq!(
            t("export.csv_tip", Locale::Es),
            [
                "Descarga honesta: escribe el CSV ",
                "real o informa el error"
            ]
            .concat()
        );
        assert_eq!(t("export.notify_pdf", Locale::Es), "Exportando PDF…");
        assert_eq!(t("export.reveal_label", Locale::Es), "Mostrar en carpeta");
        assert_eq!(
            t("export.table_row", Locale::Es),
            ["{name} · {rows} ", "filas"].concat()
        );
        // Wiring: el draw usa las 12 keys vía el dispatcher/helper con su locale.
        let source = include_str!("panels.rs");
        for pin in [
            "export_chrome_for(\"text_mathml_tip\", export_locale)",
            "export_chrome_for(\"text_tikz_tip\", export_locale)",
            "export_chrome_for(\"text_html_tip\", export_locale)",
            "export_chrome_for(\"tables_title\", export_locale)",
            "export_chrome_for(\"csv_tip\", export_locale)",
            "export_chrome_for(\"notify_pdf\", export_locale)",
            "export_chrome_for(\"notify_png\", export_locale)",
            "export_chrome_for(\"notify_csv\", export_locale)",
            "export_chrome_for(\"reveal_label\", export_locale)",
            "export_chrome_for(\"reveal_tip\", export_locale)",
            "export_chrome_for(\"table_untagged\", export_locale)",
            "export_table_row_label_for(",
            "action(ui, export_copy_svg, export_copy_svg_tip)",
        ] {
            assert!(source.contains(pin), "export B-A170 cablea {pin}");
        }
        // Blindaje: las consts crudas ya no están en el draw (solo quedan
        // los labels `[0..2].0` + import de `EXPORT_TEXT_BUTTONS` + sigla CSV).
        // (Fragmentos; splits distintos de los pines, precedente B-A123.)
        for needle in [
            ["EXPORT_TEXT_BUTTONS[0]", ".1"].concat(),
            ["EXPORT_TEXT_BUTTONS[1]", ".1"].concat(),
            ["EXPORT_TEXT_BUTTONS[2]", ".1"].concat(),
            ["EXPORT_TABLES", "_TITLE"].concat(),
            ["EXPORT_CSV", "_TOOLTIP"].concat(),
            ["EXPORT_NOTIFY", "_PDF"].concat(),
            ["EXPORT_NOTIFY", "_PNG"].concat(),
            ["EXPORT_NOTIFY", "_CSV"].concat(),
            ["EXPORT_REVEAL", "_LABEL"].concat(),
            ["EXPORT_REVEAL", "_TOOLTIP"].concat(),
            ["export_table_row_label", "(name, rows)"].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "export sin ES duro en prod (B-A170): {needle}"
            );
        }
    }

    #[test]
    fn export_b_a171_toasts_salen_de_helpers_for() {
        // B-A171: toasts de copiado de la sección Exportar salen de los
        // helpers `_for` con `export_locale`. Sin egui: pines + blindaje por
        // `include_str!`. Labels `MathML`/`TikZ-eje`/`HTML` intactos.
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(
            t("export.svg_copied", Locale::Es),
            ["SVG copiado al portapapeles ", "({bytes} bytes)"].concat()
        );
        assert_eq!(
            t("export.svg_error", Locale::Es),
            ["No se pudo copiar SVG", ": {error}"].concat()
        );
        assert_eq!(
            t("export.clipboard_copied", Locale::Es),
            ["{label} copiado al portapapeles ", "({bytes} bytes)"].concat()
        );
        // Wiring: el draw usa los 3 helpers con su locale.
        let source = include_str!("panels.rs");
        for pin in [
            "export_svg_copied_text_for(bytes, export_locale)",
            "export_svg_error_text_for(&error, export_locale)",
            "export_clipboard_copied_text_for(label, bytes, export_locale)",
        ] {
            assert!(source.contains(pin), "export B-A171 cablea {pin}");
        }
        // Blindaje: los canónicos Es-only ya no están en el draw.
        // (Fragmentos; splits distintos de los pines, precedente B-A123.)
        for needle in [
            ["export_svg_copied_text", "(bytes)"].concat(),
            ["export_svg_error_text", "(&error)"].concat(),
            ["export_clipboard_copied_text", "(label, bytes)"].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "export sin ES duro en prod (B-A171): {needle}"
            );
        }
    }

    #[test]
    fn export_b_a173_identificadores_quedan_crudos_intencional() {
        // B-A173 (cierre saga export): el draw usa los 3 identificadores rfd
        // en crudo sin `locale` a propósito — filtros "PDF"/"CSV" + filename
        // `format!("{stem}.csv")` (`draw_export_section`, siglas/extensiones
        // invariantes en los 6 idiomas; precedentes B-A90/B-A169/B-A162).
        // Todo el chrome ya sale de `export_chrome_for`/`export_*_for`
        // (B-A169/B-A170/B-A171). Sin egui: wiring + blindaje por
        // `include_str!` (fragmentos porque el `include_str!` incluye este
        // test, precedente B-A89).
        // S2: los `add_filter` viven en los helpers `pick`/`save` de app.rs
        // (crudos, tras `desktop-dialogs`); el draw solo pasa los literales.
        let source = include_str!("panels.rs");
        let app_source = include_str!("app.rs");
        for pin in [
            ["\"P", "DF\","].concat(),
            ["&[\"pd", "f\"],"].concat(),
            ["\"C", "SV\","].concat(),
            ["&[\"cs", "v\"],"].concat(),
            ["format!(\"{st", "em}.csv\")"].concat(),
        ] {
            assert!(
                source.contains(pin.as_str()),
                "export B-A173 usa identificador crudo en draw: {pin}"
            );
        }
        // S2: los helpers existen y el dialogo real queda tras la feature
        // (fragmentos: el contenido incluye este test, precedente B-A89).
        for frag in [
            ["fn pick_fi", "le_native"].concat(),
            ["fn save_fi", "le_native"].concat(),
        ] {
            assert!(
                app_source.contains(frag.as_str()),
                "export S2 helpers de dialogo nativo: {frag}"
            );
        }
        // Blindaje: los identificadores NO pasan por `export_msg`/`t()` ni
        // por key `export.filter*/filename*/dialog*` (0 keys B-A173).
        for needle in [
            ["export_msg(\"fil", "ter\")"].concat(),
            ["t(\"export.fil", "ter\")"].concat(),
            ["export.filter", "_pdf"].concat(),
            ["export.filen", "ame"].concat(),
            ["export.dial", "og"].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "export sin display-localizar identificadores: {needle}"
            );
        }
    }

    #[test]
    fn view_b_a129_3d_capas_hp_sale_de_view_chrome_for() {
        // B-A129: chrome 3D + Capas estático + AltaPrecisión del panel Vista
        // sale del dispatcher `view_chrome_for` con `view_locale`. Sin egui:
        // pines + blindaje por `include_str!`.
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(t("view.section_3d", Locale::Es), "Vista 3D");
        assert_eq!(t("view.layers_title", Locale::Es), "Capas");
        // Concat a propósito: el blindaje de abajo usa `include_str!`
        // (incluye este test) y el literal completo se auto-matchearía
        // (precedente B-A106 `motion.toast_started`).
        assert_eq!(
            t("view.indicate_hover", Locale::Es),
            ["Hace latir la selección ", "~1.2 s"].concat()
        );
        assert_eq!(
            t("view.indicating", Locale::Es),
            ["Indicando selección ", "(~1.2 s)"].concat()
        );
        assert_eq!(t("view.hp_title", Locale::Es), "Alta Precisión");
        assert_eq!(
            t("view.hp_label", Locale::Es),
            ["Alta Precisión ", "(Double-Double)"].concat()
        );
        assert_eq!(t("view.section_3d", Locale::En), "3D view");
        assert_eq!(t("view.layers_title", Locale::Pt), "Camadas");
        assert_eq!(t("view.assign", Locale::It), "Assegna");
        assert_eq!(t("view.hp_title", Locale::Fr), "Haute précision");
        assert_eq!(t("view.hp_title", Locale::De), "Hohe Präzision");
        // Wiring: el draw usa las 14 keys vía el dispatcher con su locale.
        let source = include_str!("panels.rs");
        for pin in [
            "view_chrome_for(\"section_3d\", view_locale)",
            "view_chrome_for(\"ortho_hover\", view_locale)",
            "view_chrome_for(\"layers_title\", view_locale)",
            "layer_toggle_label_for(visible, view_locale)",
            "view_chrome_for(\"assign_layer\", view_locale)",
            "view_chrome_for(\"assign\", view_locale)",
            "view_chrome_for(\"highlight\", view_locale)",
            "view_chrome_for(\"indicate\", view_locale)",
            "view_chrome_for(\"indicate_hover\", view_locale)",
            "view_chrome_for(\"indicating\", view_locale)",
            "view_chrome_for(\"hp_title\", view_locale)",
            "view_chrome_for(\"hp_label\", view_locale)",
            "view_chrome_for(\"hp_hint\", view_locale)",
            "draw_inspector_section(ui, section_3d,",
            "draw_inspector_section(ui, layers_title,",
            "draw_inspector_section(ui, hp_title,",
            "assign_layer_label,",
            "assign_label,",
            "highlight_label,",
            "indicate_label,",
            "indicate_hover,",
            "indicating.to_string(),",
            "hp_label,",
            "hp_hint,",
        ] {
            assert!(source.contains(pin), "vista B-A129 cablea {pin}");
        }
        // Blindaje: las 12 consts crudas ya no están en el draw.
        // (Fragmentos; splits distintos de los pines, precedente B-A123.)
        for needle in [
            ["VIEW_3D_SECTION", "_TITLE"].concat(),
            ["VIEW_3D_ORTHO", "_HOVER"].concat(),
            ["VIEW_LAYERS", "_TITLE"].concat(),
            ["LAYER_TOGGLE", "_SHOW"].concat(),
            ["LAYER_TOGGLE", "_HIDE"].concat(),
            ["VIEW_ASSIGN_LAYER", "_LABEL"].concat(),
            ["VIEW_ASSIGN", "_LABEL"].concat(),
            ["VIEW_HIGHLIGHT", "_LABEL"].concat(),
            ["VIEW_INDICATE", "_LABEL"].concat(),
            ["VIEW_INDICATE", "_HOVER"].concat(),
            ["VIEW_INDICAT", "ING"].concat(),
            ["VIEW_HIGH_PRECISION", "_TITLE"].concat(),
            ["VIEW_HIGH_PRECISION", "_LABEL"].concat(),
            ["VIEW_HIGH_PRECISION", "_HINT"].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "vista sin ES duro en prod (B-A129): {needle}"
            );
        }
    }
    #[test]
    fn view_b_a130_capas_dinamicas_salen_de_helpers_for() {
        // B-A130: Capas dinámicas + hints + indicate del panel Vista salen
        // de los helpers `_for` con `view_locale`. Sin egui: pines +
        // blindaje por `include_str!` (fragmentos, precedente B-A129).
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(
            t("view.layer_empty", Locale::Es),
            "Sin objetos: todo lo nuevo entra en la capa 0."
        );
        assert_eq!(
            t("view.layer_row", Locale::Es),
            "Capa {layer} · {count} obj"
        );
        assert_eq!(
            t("view.layer_toggled_hidden", Locale::Es),
            "Capa {layer} oculta ({touched} obj)"
        );
        assert_eq!(
            t("view.layer_selection", Locale::En),
            "Selection: {label} (layer {layer})"
        );
        assert_eq!(
            t("view.indicate_err", Locale::De),
            "Konnte nicht kennzeichnen: {error}"
        );
        // Wiring: el draw usa los 7 helpers `_for` + 2 lets del dispatcher.
        let source = include_str!("panels.rs");
        for pin in [
            "view_chrome_for(\"layer_empty\", view_locale)",
            "view_chrome_for(\"layer_no_selection\", view_locale)",
            "layer_row_label_for(layer, count, view_locale)",
            "layer_toggle_result_for(layer, visible, touched, view_locale)",
            "layer_selection_text_for(label, *layer, view_locale)",
            "layer_assign_result_for(",
            "layer_assign_error_for(&error, view_locale)",
            "view_indicate_error_for(&error, view_locale)",
            "layer_empty_hint)",
            "layer_no_selection.to_string()",
        ] {
            assert!(source.contains(pin), "vista B-A130 cablea {pin}");
        }
        // Blindaje: las fns crudas y consts ya no están en el draw.
        // (Fragmentos; los pines de arriba usan `_for`, precedente B-A123.)
        for needle in [
            ["layer_row_label(layer,", " count)"].concat(),
            ["layer_toggle_result(layer,", " visible, touched)"].concat(),
            ["layer_selection_text(label,", " *layer)"].concat(),
            ["layer_assign_res", "ult("].concat(),
            ["layer_assign_error(&", "error)"].concat(),
            ["view_indicate_error(&", "error)"].concat(),
            ["LAYER_EMPTY", "_HINT"].concat(),
            ["LAYER_NO_SELEC", "TION"].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "vista sin ES duro en prod (B-A130): {needle}"
            );
        }
    }
    #[test]
    fn view_b_a131_parametro_vivo_sale_de_helpers_for() {
        // B-A131: parámetro vivo del panel Vista sale de los helpers `_for`
        // con `view_locale`. Sin egui: pines + blindaje por `include_str!`
        // (fragmentos, precedente B-A129/B-A130).
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(t("view.live_section", Locale::Es), "Parámetro vivo");
        assert_eq!(
            t("view.live_create", Locale::Es),
            "Crear parámetro p en [-5, 5]"
        );
        assert_eq!(
            t("view.live_created", Locale::Es),
            "Parámetro `p` creado en [-5, 5]"
        );
        assert_eq!(
            t("view.live_readout", Locale::En),
            "p = {value} in [{min}, {max}]"
        );
        assert_eq!(t("view.live_error", Locale::De), "Live-Parameter: {error}");
        // Wiring: el draw usa el dispatcher (5 lets) + los 3 helpers `_for`.
        let source = include_str!("panels.rs");
        for pin in [
            "view_chrome_for(\"live_section\", view_locale)",
            "view_chrome_for(\"live_missing\", view_locale)",
            "view_chrome_for(\"live_create\", view_locale)",
            "view_chrome_for(\"live_create_hover\", view_locale)",
            "view_chrome_for(\"live_created\", view_locale)",
            "live_param_hover_text_for(value, live.min, live.max, view_locale)",
            "live_param_readout_text_for(",
            "live_param_error_text_for(error, view_locale)",
            "live_created.to_string()",
        ] {
            assert!(source.contains(pin), "vista B-A131 cablea {pin}");
        }
        // Blindaje: las consts crudas y fns sin locale ya no están en el draw.
        // (Fragmentos; los pines de arriba usan `_for`, precedente B-A123.)
        for needle in [
            ["LIVE_PARAM_SECTION", "_TITLE"].concat(),
            ["LIVE_PARAM_MISSING", "_GUIDE"].concat(),
            ["LIVE_PARAM_CREATE", "_LABEL"].concat(),
            ["LIVE_PARAM_CREATE", "_HOVER"].concat(),
            ["LIVE_PARAM_CRE", "ATED"].concat(),
            ["live_param_hover_text(", "value,"].concat(),
            ["live_param_readout_text(", "value,"].concat(),
            ["live_param_error_text(", "error)"].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "vista sin ES duro en prod (B-A131): {needle}"
            );
        }
    }
    #[test]
    fn motion_chrome_sale_de_motion_msg() {
        // B-A106: chrome de la tarjeta de movimiento 4D sale de `motion.*`
        // con `motion_locale`; ES pineado a los literales previos (`:796`
        // Pausar, `:798` Iniciar, `:822` iniciada, `:824` pausada, `:837`
        // En reproducción, `:839` Activada, `:841` En pausa, `:843` No
        // disponible, `:861` Velocidad, `:876` Velocidad de animación,
        // `:888` Restablecer, `:889` hover, `:903` hint 3D). Sin egui:
        // pines + blindaje por `include_str!`.
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(t("motion.pause", Locale::Es), "Pausar animación");
        assert_eq!(t("motion.start", Locale::Es), "Iniciar animación");
        // Concat a propósito: el blindaje de abajo usa `include_str!`
        // (incluye este test) y el literal completo se auto-matchearía
        // (precedente B-A105 `stats.empty_hint`).
        assert_eq!(
            t("motion.toast_started", Locale::Es),
            ["Animación espacial ini", "ciada."].concat()
        );
        assert_eq!(
            t("motion.toast_paused", Locale::Es),
            ["Animación espacial pau", "sada."].concat()
        );
        assert_eq!(t("motion.status_playing", Locale::Es), "En reproducción");
        assert_eq!(t("motion.status_enabled", Locale::Es), "Activada");
        assert_eq!(t("motion.status_paused", Locale::Es), "En pausa");
        assert_eq!(t("motion.status_unavailable", Locale::Es), "No disponible");
        assert_eq!(t("motion.speed", Locale::Es), "Velocidad");
        assert_eq!(t("motion.speed_text", Locale::Es), "Velocidad de animación");
        assert_eq!(t("motion.reset", Locale::Es), "Restablecer velocidad");
        assert_eq!(
            t("motion.reset_hint", Locale::Es),
            "Volver a la velocidad normal (1.00x)"
        );
        assert_eq!(
            t("motion.need_3d", Locale::Es),
            [
                "Mostrá el objeto en la vista 3D para activar la anima",
                "ción."
            ]
            .concat()
        );
        // Wiring: el draw usa las 13 keys con `motion_locale` (B-A106).
        let source = include_str!("panels.rs");
        for pin in [
            "let pause_label = motion_t(\"motion.pause\"",
            "let start_label = motion_t(\"motion.start\"",
            "let toast_started_label = motion_t(\"motion.toast_started\"",
            "let toast_paused_label = motion_t(\"motion.toast_paused\"",
            "let playing_label = motion_t(\"motion.status_playing\"",
            "let enabled_label = motion_t(\"motion.status_enabled\"",
            "let paused_label = motion_t(\"motion.status_paused\"",
            "let unavailable_label = motion_t(\"motion.status_unavailable\"",
            "let speed_label = motion_t(\"motion.speed\"",
            "let speed_text_label = motion_t(\"motion.speed_text\"",
            "let reset_label = motion_t(\"motion.reset\"",
            "let reset_hint_label = motion_t(\"motion.reset_hint\"",
            "let need_3d_label = motion_t(\"motion.need_3d\"",
            "let motion_locale = app.config_locale()",
        ] {
            assert!(source.contains(pin), "motion cablea {pin}");
        }
        // Blindaje: viejos literales ES duros ya no están en el draw.
        // Agujas con contexto del call-site viejo: los pines ES de arriba
        // y los del test trig se auto-matchearían con agujas bare
        // (precedente B-A97 con aguja por contexto de llamada).
        for needle in [
            ["if global_on {\n                \"Pausar anima", "ción\""].concat(),
            ["} else {\n                \"Iniciar anima", "ción\""].concat(),
            ["\"Animación espacial ini", "ciada.\""].concat(),
            ["\"Animación espacial pau", "sada.\""].concat(),
            ["{\n                        \"En reprodu", "cción\""].concat(),
            ["global_on {\n                        \"Acti", "vada\""].concat(),
            ["can_animate {\n                        \"En pa", "usa\""].concat(),
            ["} else {\n                        \"No dispo", "nible\""].concat(),
            ["RichText::new(\"Velo", "cidad\")"].concat(),
            [".text(\"Velocidad de anima", "ción\")"].concat(),
            ["small_button(\"Restablecer veloci", "dad\")"].concat(),
            ["on_hover_text(\"Volver a la velocidad normal (1.00", "x)\")"].concat(),
            [
                "RichText::new(\n                        \"Mostrá el objeto en la vista 3D para activar la anima",
                "ción.\",\n                    )",
            ]
            .concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "motion sin ES duro en prod (B-A106): {needle}"
            );
        }
    }

    #[test]
    fn wc_chrome_sale_de_wc_msg() {
        // B-A107: chrome de la tarjeta W-C estudio/integral/taylor sale de
        // `wc.*` con `wc_locale`; ES pineado a los literales previos
        // (`:989` Función:, `:990` Estudiar, `:994` hint ceros/extremos,
        // `:1007` hover integrando, `:1027` hint Simpson n par, `:1033`
        // Aproximar, `:1044` Exacta, `:1053` hint hoja ambas celdas,
        // `:1061` hover función, `:1063` centro:, `:1068` slider orden,
        // `:1069` Polinomio, `:1091` Sin resto). Sin egui: pines +
        // blindaje por `include_str!`.
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(t("wc.study_function", Locale::Es), "Función:");
        assert_eq!(t("wc.study_button", Locale::Es), "Estudiar");
        // Concat a propósito: el blindaje de abajo usa `include_str!`
        // (incluye este test) y el literal completo se auto-matchearía
        // (precedente B-A105 `stats.empty_hint`).
        assert_eq!(
            t("wc.study_hint", Locale::Es),
            [
                "Ceros, extremos, asíntotas + tabla de signos. Marca puntos en el can",
                "vas."
            ]
            .concat()
        );
        assert_eq!(
            t("wc.integral_hover", Locale::Es),
            "Integrando en x, ej. x^2"
        );
        assert_eq!(
            t("wc.integral_simpson_hint", Locale::Es),
            [
                "Simpson requiere n par (el comando lo rechaza hones",
                "to)."
            ]
            .concat()
        );
        assert_eq!(t("wc.integral_approx", Locale::Es), "Aproximar");
        assert_eq!(t("wc.integral_exact", Locale::Es), "Exacta");
        assert_eq!(
            t("wc.integral_sheet_hint", Locale::Es),
            ["La hoja muestra ambas celdas: aproximado vs exa", "cto."].concat()
        );
        assert_eq!(t("wc.taylor_hover", Locale::Es), "Función en x, ej. sin(x)");
        assert_eq!(t("wc.taylor_center", Locale::Es), "centro:");
        assert_eq!(t("wc.taylor_order", Locale::Es), "orden");
        assert_eq!(t("wc.taylor_poly", Locale::Es), "Polinomio");
        assert_eq!(
            t("wc.taylor_no_remainder", Locale::Es),
            ["Sin resto: revisá la expre", "sión."].concat()
        );
        // Wiring: los 3 draws usan las 13 keys con `wc_locale` (B-A107).
        let source = include_str!("panels.rs");
        for pin in [
            "let function_label = wc_t(\"wc.study_function\"",
            "let study_label = wc_t(\"wc.study_button\"",
            "let hint_label = wc_t(\"wc.study_hint\"",
            "let hover_label = wc_t(\"wc.integral_hover\"",
            "let simpson_hint_label = wc_t(\"wc.integral_simpson_hint\"",
            "let approx_label = wc_t(\"wc.integral_approx\"",
            "let exact_label = wc_t(\"wc.integral_exact\"",
            "let sheet_hint_label = wc_t(\"wc.integral_sheet_hint\"",
            "let hover_label = wc_t(\"wc.taylor_hover\"",
            "let center_label = wc_t(\"wc.taylor_center\"",
            "let order_label = wc_t(\"wc.taylor_order\"",
            "let poly_label = wc_t(\"wc.taylor_poly\"",
            "let no_remainder_label = wc_t(\"wc.taylor_no_remainder\"",
            "let wc_locale = app.config_locale()",
        ] {
            assert!(source.contains(pin), "wc cablea {pin}");
        }
        // Blindaje: viejos literales ES duros ya no están en el draw.
        // (Fragmentos porque `include_str!` incluye este test.)
        for needle in [
            ["ui.label(\"Fun", "ción:\")"].concat(),
            ["small_button(\"Estu", "diar\")"].concat(),
            [
                "\"Ceros, extremos, asíntotas + tabla de signos. Marca puntos en el can",
                "vas.\"",
            ]
            .concat(),
            ["on_hover_text(\"Integrando en x, ej.", " x^2\")"].concat(),
            ["small_button(\"Apro", "ximar\")"].concat(),
            ["small_button(\"Exa", "cta\")"].concat(),
            [
                "\"La hoja muestra ambas celdas: aproximado vs exa",
                "cto.\"",
            ]
            .concat(),
            [
                "\"Simpson requiere n par (el comando lo rechaza hones",
                "to).\"",
            ]
            .concat(),
            ["on_hover_text(\"Función en x, ej.", " sin(x)\")"].concat(),
            ["ui.label(\"cen", "tro:\")"].concat(),
            [".text(\"or", "den\")"].concat(),
            ["small_button(\"Poli", "nomio\")"].concat(),
            ["\"Sin resto: revisá la expre", "sión.\""].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "wc sin ES duro en prod (B-A107): {needle}"
            );
        }
    }

    #[test]
    fn wc_section_headers_salen_de_wc_msg() {
        // B-A132: headers de sección de `draw_wc_visual_sections` salen de
        // `wc.section_*` con `app.config_locale()`; ES pineado a los
        // literales previos (`:1107-1125` Recorrido/Integral/Taylor +
        // subtítulos). Sin egui: pines + blindaje por `include_str!`.
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(
            t("wc.section_study_title", Locale::Es),
            ["Recorrido de fun", "ción"].concat()
        );
        assert_eq!(
            t("wc.section_integral_title", Locale::Es),
            ["Integral numé", "rica"].concat()
        );
        assert_eq!(
            t("wc.section_taylor_title", Locale::Es),
            ["Taylor vi", "sual"].concat()
        );
        // Concat a propósito: el blindaje de abajo usa `include_str!`
        // (incluye este test) y el literal completo se auto-matchearía
        // (precedente B-A105 `stats.empty_hint`).
        assert_eq!(
            t("wc.section_study_subtitle", Locale::Es),
            [
                "Paso a paso visual: signos, ceros, extremos, asín",
                "totas.",
            ]
            .concat()
        );
        assert_eq!(
            t("wc.section_integral_subtitle", Locale::Es),
            ["Riemann, trapecio o Simpson con n ba", "rras."].concat()
        );
        assert_eq!(
            t("wc.section_taylor_subtitle", Locale::Es),
            ["Orden 1..10 con resto obser", "vado."].concat()
        );
        // Wiring: `draw_wc_visual_sections` usa las 6 keys con locale.
        let source = include_str!("panels.rs");
        for pin in [
            "let study_title = wc_t(\"wc.section_study_title\"",
            "let study_subtitle = wc_t(\"wc.section_study_subtitle\"",
            "let integral_title = wc_t(\"wc.section_integral_title\"",
            "let integral_subtitle = wc_t(\"wc.section_integral_subtitle\"",
            "let taylor_title = wc_t(\"wc.section_taylor_title\"",
            "let taylor_subtitle = wc_t(\"wc.section_taylor_subtitle\"",
            "let wc_section_locale = app.config_locale()",
        ] {
            assert!(source.contains(pin), "wc headers cablea {pin}");
        }
        // Blindaje: viejos literales ES duros ya no están en el draw.
        // (Fragmentos porque `include_str!` incluye este test.)
        for needle in [
            ["\"Recorrido de fun", "ción\""].concat(),
            [
                "\"Paso a paso visual: signos, ceros, extremos, asín",
                "totas.\"",
            ]
            .concat(),
            ["\"Integral numé", "rica\""].concat(),
            ["\"Riemann, trapecio o Simpson con n ba", "rras.\""].concat(),
            ["\"Taylor vi", "sual\""].concat(),
            ["\"Orden 1..10 con resto obser", "vado.\""].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "wc headers sin ES duro en prod (B-A132): {needle}"
            );
        }
    }

    #[test]
    fn inspector_chrome_sale_de_inspector_msg() {
        // B-A108: chrome del panel Inspector base sale de `inspector.*`
        // con `inspector_locale`; ES pineado a los literales previos de
        // `sheet_layout.rs` (`:2922` Inspector, `:2923` subtítulo,
        // `:2924` vacío, `:2925` hint, `:2927` inexistente, `:2928`
        // ecuación, `:2929` Revertir, `:2930` ejemplo, `:2931` aplicar,
        // `:2932` identidad, `:2933` sin ecuación, `:2934` Estilo,
        // `:2935` subtítulo estilo, `:2936` hint sin estilo, `:2938`
        // Capa:). Sin egui: pines + blindaje por `include_str!`.
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(t("inspector.panel_title", Locale::Es), "Inspector");
        assert_eq!(t("inspector.revert", Locale::Es), "Revertir");
        assert_eq!(t("inspector.use_example", Locale::Es), "Usar ejemplo");
        assert_eq!(t("inspector.apply", Locale::Es), "Aplicar ecuación");
        assert_eq!(t("inspector.identity_title", Locale::Es), "Ecuación");
        assert_eq!(t("inspector.style_title", Locale::Es), "Estilo");
        assert_eq!(t("inspector.layer_label", Locale::Es), "Capa:");
        assert_eq!(
            t("inspector.panel_subtitle", Locale::Es),
            ["Propiedades del objeto seleccio", "nado."].concat()
        );
        assert_eq!(
            t("inspector.empty_title", Locale::Es),
            ["Tocá un objeto del lienzo para editarlo ", "acá."].concat()
        );
        assert_eq!(
            t("inspector.empty_hint", Locale::Es),
            [
                "Acá ajustás su geometría, estilo y controles ",
                "avanzados."
            ]
            .concat()
        );
        assert_eq!(
            t("inspector.missing_object", Locale::Es),
            "Objeto inexistente."
        );
        assert_eq!(
            t("inspector.equation_label", Locale::Es),
            "Ecuación (editable)"
        );
        assert_eq!(
            t("inspector.no_equation", Locale::Es),
            ["Ecuación no disponible para ", "este tipo"].concat()
        );
        assert_eq!(
            t("inspector.style_subtitle", Locale::Es),
            ["Trazo, marcador y capa de dibujo ", "(0 = fondo)."].concat()
        );
        assert_eq!(
            t("inspector.style_none_hint", Locale::Es),
            ["Este tipo no tiene trazo ni marcador ", "con estilo."].concat()
        );
        // Wiring: header + contenido usan las 15 keys vía el dispatcher
        // `inspector_chrome_for` (Es→const canónica, resto→i18n).
        let source = include_str!("panels.rs");
        for pin in [
            "inspector_chrome_for(\"panel_title\", inspector_locale)",
            "inspector_chrome_for(\"panel_subtitle\", inspector_locale)",
            "inspector_chrome_for(\"empty_title\", locale)",
            "inspector_chrome_for(\"empty_hint\", locale)",
            "let inspector_missing_label = inspector_chrome_for(\"missing_object\"",
            "let inspector_equation_label = inspector_chrome_for(\"equation_label\"",
            "let inspector_revert_label = inspector_chrome_for(\"revert\"",
            "let inspector_use_example_label = inspector_chrome_for(\"use_example\"",
            "let inspector_apply_label = inspector_chrome_for(\"apply\"",
            "inspector_chrome_for(\"identity_title\", locale)",
            "inspector_chrome_for(\"no_equation\", locale)",
            "let inspector_style_title_label = inspector_chrome_for(\"style_title\"",
            "let inspector_style_subtitle_label = inspector_chrome_for(\"style_subtitle\"",
            "let inspector_style_none_label = inspector_chrome_for(\"style_none_hint\"",
            "let inspector_layer_label_text = inspector_chrome_for(\"layer_label\"",
            "let inspector_locale = app.config_locale()",
            "draw_inspector_empty_state(ui, inspector_locale)",
            "draw_inspector_identity(ui, &edited_object, inspector_locale)",
        ] {
            assert!(source.contains(pin), "inspector cablea {pin}");
        }
        // Blindaje: las 15 consts ES ya no se referencian en el draw.
        // (Fragmentos porque `include_str!` incluye este test.)
        for needle in [
            ["INSPECTOR_PAN", "EL_TITLE"].concat(),
            ["INSPECTOR_PAN", "EL_SUBTITLE"].concat(),
            ["INSPECTOR_EMP", "TY_TITLE"].concat(),
            ["INSPECTOR_EMP", "TY_HINT"].concat(),
            ["INSPECTOR_MISS", "ING_OBJECT"].concat(),
            ["INSPECTOR_EQUA", "TION_LABEL"].concat(),
            ["INSPECTOR_REV", "ERT_LABEL"].concat(),
            ["INSPECTOR_USE_EX", "AMPLE_LABEL"].concat(),
            ["INSPECTOR_APP", "LY_LABEL"].concat(),
            ["INSPECTOR_IDEN", "TITY_TITLE"].concat(),
            ["INSPECTOR_NO_EQU", "ATION"].concat(),
            ["INSPECTOR_STY", "LE_TITLE"].concat(),
            ["INSPECTOR_STY", "LE_SUBTITLE"].concat(),
            ["INSPECTOR_STY", "LE_NONE_HINT"].concat(),
            ["INSPECTOR_LAY", "ER_LABEL"].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "inspector sin ES duro en prod (B-A108): {needle}"
            );
        }
    }

    #[test]
    fn inspector_poly_chrome_sale_de_inspector_msg() {
        // B-A109: sección 4D/N-D sale de `inspector.poly*` con
        // `inspector_locale`; ES pineado a los literales previos de
        // `sheet_layout.rs` (`:4033` Proyección, `:4034` subtítulo,
        // `:4036` anim, `:4037` body, `:4039` Geometría, `:4040` Forma,
        // `:4041` Apariencia, `:4042` subtítulo, `:4043` hint relleno,
        // `:4045` manual, `:4046` guía, `:4048` reset, `:4049` planos,
        // `:4054` familia, `:4055` proyección N-D, `:4057` dim4, `:4059`
        // otra, `:4060` aristas, `:4061` fill N-D, `:4063` guía).
        // Sin egui: pines + blindaje por `include_str!`.
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(
            t("inspector.poly_projection_title", Locale::Es),
            "Proyección"
        );
        assert_eq!(
            t("inspector.poly_anim_title", Locale::Es),
            "Animación de proyección"
        );
        assert_eq!(t("inspector.poly_geometry_title", Locale::Es), "Geometría");
        assert_eq!(
            t("inspector.poly_appearance_title", Locale::Es),
            "Apariencia"
        );
        assert_eq!(
            t("inspector.poly_rotation_manual_title", Locale::Es),
            "Rotación manual"
        );
        assert_eq!(
            t("inspector.poly_rotation_reset_label", Locale::Es),
            "Restablecer rotaciones"
        );
        assert_eq!(
            t("inspector.poly_projection_subtitle", Locale::Es),
            [
                "Controlá la vista dinámica sin alterar ",
                "la construcción."
            ]
            .concat()
        );
        assert_eq!(
            t("inspector.poly_anim_body", Locale::Es),
            [
                "La cámara y el politopo 4D giran sin alterar ",
                "el documento."
            ]
            .concat()
        );
        assert_eq!(
            t("inspector.poly_fill_omitted_hint", Locale::Es),
            [
                "El relleno se omite en Vista previa y ",
                "durante el movimiento."
            ]
            .concat()
        );
        assert_eq!(
            t("inspector.poly_rotation_manual_guide", Locale::Es),
            [
                "Ajustá los seis planos sólo cuando necesites ",
                "una vista específica."
            ]
            .concat()
        );
        assert_eq!(
            t("inspector.polytope_geometry_subtitle", Locale::Es),
            "Familia, dimensión y escala"
        );
        assert_eq!(
            t("inspector.polytope_projection_subtitle", Locale::Es),
            ["La cámara y la proyección 4D ", "comparten velocidad."].concat()
        );
        assert_eq!(
            t("inspector.polytope_anim_dim4_body", Locale::Es),
            ["La cámara y la proyección 4D ", "usan la misma velocidad."].concat()
        );
        assert_eq!(
            t("inspector.polytope_anim_other_body", Locale::Es),
            "La cámara orbita sin alterar el documento."
        );
        assert_eq!(
            t("inspector.polytope_fill_hint", Locale::Es),
            [
                "Los politopos N-D genéricos se muestran solo como aristas; ",
                "el relleno se omite en Vista previa y durante el movimiento.",
            ]
            .concat()
        );
        // Wiring: las ramas 4D/N-D usan las 20 keys vía el dispatcher.
        // (Pines por llamada `inspector_chrome_for("...", inspector_locale)`:
        // los `let` largos los parte `fmt` en dos líneas, precedente B-A105.)
        let source = include_str!("panels.rs");
        for pin in [
            "inspector_chrome_for(\"poly_projection_title\", inspector_locale)",
            "inspector_chrome_for(\"poly_projection_subtitle\", inspector_locale)",
            "inspector_chrome_for(\"poly_anim_title\", inspector_locale)",
            "inspector_chrome_for(\"poly_anim_body\", inspector_locale)",
            "inspector_chrome_for(\"poly_geometry_title\", inspector_locale)",
            "inspector_chrome_for(\"poly_geometry_subtitle\", inspector_locale)",
            "inspector_chrome_for(\"poly_appearance_title\", inspector_locale)",
            "inspector_chrome_for(\"poly_appearance_subtitle\", inspector_locale)",
            "inspector_chrome_for(\"poly_fill_omitted_hint\", inspector_locale)",
            "inspector_chrome_for(\"poly_rotation_manual_title\", inspector_locale)",
            "inspector_chrome_for(\"poly_rotation_manual_guide\", inspector_locale)",
            "inspector_chrome_for(\"poly_rotation_reset_label\", inspector_locale)",
            "inspector_chrome_for(\"poly_rotation_planes_title\", inspector_locale)",
            "inspector_chrome_for(\"polytope_geometry_subtitle\", inspector_locale)",
            "inspector_chrome_for(\"polytope_projection_subtitle\", inspector_locale)",
            "inspector_chrome_for(\"polytope_appearance_subtitle\", inspector_locale)",
            "inspector_chrome_for(\"polytope_fill_hint\", inspector_locale)",
            "inspector_chrome_for(\"polytope_rotation_guide\", inspector_locale)",
            "polytope_nd_projection_body_for(",
        ] {
            assert!(source.contains(pin), "inspector 4D cablea {pin}");
        }
        // Blindaje: las 20 consts ES ya no se referencian en el draw
        // (`POLYCHORON_ROTATION_PLANES` queda: notación "(rad)" intacta).
        // (Fragmentos porque `include_str!` incluye este test.)
        for needle in [
            ["POLYCHORON_PROJEC", "TION_TITLE"].concat(),
            ["POLYCHORON_PROJEC", "TION_SUBTITLE"].concat(),
            ["POLYCHORON_A", "NIM_TITLE"].concat(),
            ["POLYCHORON_A", "NIM_BODY"].concat(),
            ["POLYCHORON_GEO", "METRY_TITLE"].concat(),
            ["POLYCHORON_GEO", "METRY_SUBTITLE"].concat(),
            ["POLYCHORON_APPEA", "RANCE_TITLE"].concat(),
            ["POLYCHORON_APPEA", "RANCE_SUBTITLE"].concat(),
            ["POLYCHORON_FILL_OMI", "TTED_HINT"].concat(),
            ["POLYCHORON_ROTATION_MA", "NUAL_TITLE"].concat(),
            ["POLYCHORON_ROTATION_MA", "NUAL_GUIDE"].concat(),
            ["POLYCHORON_ROTATION_RE", "SET_LABEL"].concat(),
            ["POLYCHORON_ROTATION_PLA", "NES_TITLE"].concat(),
            ["POLYTOPE_ND_GEO", "METRY_SUBTITLE"].concat(),
            ["POLYTOPE_ND_PROJEC", "TION_SUBTITLE"].concat(),
            ["POLYTOPE_ND_APPEA", "RANCE_SUBTITLE"].concat(),
            ["POLYTOPE_ND_FILL", "_HINT"].concat(),
            ["POLYTOPE_ND_ROTA", "TION_GUIDE"].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "inspector 4D sin ES duro en prod (B-A109): {needle}"
            );
        }
    }

    #[test]
    fn inspector_b_a114_notacion_queda_invariante() {
        // B-A114 (barrido final saga inspector): el draw usa estos 4 sets
        // crudos sin `locale` a propósito — notación SI/matemática idéntica
        // en los 6 idiomas (unidades `(rad)`, prefijos `x=`, fórmula
        // `F(x, y, z)`; precedentes B-A90/B-A98/B-A112). Si se localizan,
        // actualizar este pin + `sheet_inspector_notation_sets_...`.
        // Sin egui: blindaje por `include_str!` (fragmentos porque el
        // `include_str!` incluye este test, precedente B-A89).
        let source = include_str!("panels.rs");
        for pin in [
            [".zip(POLYCHORON_ROTA", "TION_PLANES)"].concat(),
            [".prefix(INSPECTOR_AXIS", "_PREFIX_X)"].concat(),
            [".prefix(INSPECTOR_AXIS", "_PREFIX_Y)"].concat(),
            [".prefix(INSPECTOR_AXIS", "_PREFIX_Z)"].concat(),
            ["implicit_3d_expr", "_text("].concat(),
            ["nd_rotation_plane", "_label("].concat(),
        ] {
            assert!(
                source.contains(pin.as_str()),
                "notación cruda en draw: {pin}"
            );
        }
    }

    #[test]
    fn prob_motor_b_a160_err_se_muestra_crudo_intencional() {
        // B-A160 (cierre saga prob): la rama `Err(error)` del cuantil
        // muestra el `Err` canónico ES del motor en crudo, sin `t()` — el
        // motor es canónico a propósito (ver `sheet_layout.rs` header +
        // `sheet_prob_motor_b_a160_canonico_es_intencional`): draw
        // localizado (B-A122/B-A157) + widgets con clamp hacen el path
        // defensivo. Sin egui: wiring + blindaje por `include_str!`.
        let source = include_str!("panels.rs");
        for pin in [
            ["Err(er", "ror) =>"].concat(),
            ["egui::RichText::new(", "error)"].concat(),
        ] {
            assert!(
                source.contains(pin.as_str()),
                "cuantil muestra el Err del motor en crudo: {pin}"
            );
        }
        // Blindaje: el error del cuantil NO pasa por `stats_t`/`t()` ni por
        // key `stats.prob_err*` (0 keys B-A160). (Fragmentos vía `concat!`:
        // `include_str!` incluye este test, precedente B-A89.)
        for needle in [
            ["stats_t(\"stats.prob_err", "\")"].concat(),
            ["t(\"stats.prob_err", "\")"].concat(),
            ["prob_err_qua", "ntile_p"].concat(),
            ["prob_err_fi", "nite"].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "sin display-localizar el motor: {needle}"
            );
        }
    }

    #[test]
    fn inspector_b_a110_labels_salen_de_inspector_msg() {
        // B-A110: visibilidad/ejemplo/medida/kind/familia salen de
        // `inspector.*` con `locale`/`inspector_locale`; ES pineado a los
        // literales previos de `sheet_layout.rs` (Visible/Oculto, ejemplo
        // con `{example}`, medida con `{measure}`, 6 kinds, 3 familias).
        // Sin egui: pines + blindaje por `include_str!`.
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(t("inspector.visibility_visible", Locale::Es), "Visible");
        assert_eq!(t("inspector.visibility_hidden", Locale::Es), "Oculto");
        assert_eq!(
            t("inspector.example_tooltip", Locale::Es),
            ["Inserta sintaxis válida: ", "{example}"].concat()
        );
        assert_eq!(
            t("inspector.measure", Locale::Es),
            ["Medida: ", "{measure}"].concat()
        );
        assert_eq!(
            t("inspector.polychoron_kind_tesseract", Locale::Es),
            "Teseracto"
        );
        assert_eq!(
            t("inspector.polytope_family_simplex", Locale::Es),
            ["Sím", "plex"].concat()
        );
        // Wiring: identidad + ecuación + medida + combos usan `_for`.
        let source = include_str!("panels.rs");
        for pin in [
            "inspector_visibility_label_for(visible, locale)",
            "inspector_identity_tooltip_for(&headline, &label, visible, locale)",
            "inspector_example_tooltip_for(example, inspector_locale)",
            "inspector_measure_text_for(&measure, inspector_locale)",
            "polychoron_kind_label_for(kind, inspector_locale)",
            "polychoron_kind_label_for(",
            "polytope_nd_family_label_for(family, inspector_locale)",
            "polytope_nd_family_label_for(",
        ] {
            assert!(source.contains(pin), "inspector B-A110 cablea {pin}");
        }
        // Blindaje: los 5 canónicos ES ya no se llaman en el draw
        // (quedan solo en imports + wrappers `_for` del dueño + este test
        // partido en fragmentos).
        // (Fragmentos porque `include_str!` incluye este test.)
        for needle in [
            ["inspector_visibility_la", "bel(visible)"].concat(),
            ["inspector_example_tool", "tip(example)"].concat(),
            ["inspector_measure_te", "xt(&measure)"].concat(),
            ["polychoron_kind_la", "bel(kind)"].concat(),
            ["polytope_nd_family_la", "bel(family)"].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "inspector B-A110 sin ES duro en prod: {needle}"
            );
        }
    }

    #[test]
    fn inspector_b_a111_labels_salen_de_inspector_msg() {
        // B-A111: prefijo Etiqueta + estilos trazo/punto salen de
        // `inspector.*` con `locale`/`inspector_locale`; ES pineado a los
        // literales previos de `sheet_layout.rs` (Etiqueta, Continuo,
        // Cruz +). Sin egui: pines + blindaje por `include_str!`.
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(
            t("inspector.label_prefix", Locale::Es),
            ["Etiqueta: ", "{label}"].concat()
        );
        assert_eq!(t("inspector.line_solid", Locale::Es), "Continuo");
        assert_eq!(t("inspector.point_plus", Locale::Es), "Cruz +");
        // Wiring: identidad + estilos + tags 3D usan `_for`.
        let source = include_str!("panels.rs");
        for pin in [
            "inspector_label_text_for(&label, locale)",
            "line_style_label_for(next, inspector_locale)",
            "point_style_label_for(next, inspector_locale)",
            "inspector_3d_tag_label_for(&c.label, inspector_locale)",
            "inspector_3d_tag_label_for(&s.label, inspector_locale)",
            "inspector_3d_tag_label_for(&p.label, inspector_locale)",
        ] {
            assert!(source.contains(pin), "inspector B-A111 cablea {pin}");
        }
        // Blindaje: los 4 canónicos ES ya no se llaman en el draw
        // (quedan solo en imports + wrappers `_for` del dueño + este test
        // partido en fragmentos).
        // (Fragmentos porque `include_str!` incluye este test.)
        for needle in [
            ["inspector_label_te", "xt(&label)"].concat(),
            ["line_style_la", "bel(next)"].concat(),
            ["point_style_la", "bel(next)"].concat(),
            ["inspector_3d_tag_la", "bel(&c.label)"].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "inspector B-A111 sin ES duro en prod: {needle}"
            );
        }
    }

    #[test]
    fn inspector_b_a112_labels_salen_de_inspector_msg() {
        // B-A112: primitivas 3D + sliders salen de `inspector.*` con
        // `inspector_locale`; ES pineado a los literales previos de
        // `sheet_layout.rs` (Cubo/Esfera/Punto 3D, Centro, Superficie,
        // dock ×2, tamaño/radio/Escala/Grosor). Sin egui: pines +
        // blindaje por `include_str!`.
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(t("inspector.cube3d_title", Locale::Es), "Cubo 3D");
        assert_eq!(
            t("inspector.implicit3d_title", Locale::Es),
            "Superficie implícita 3D"
        );
        assert_eq!(
            t("inspector.dock_empty_title", Locale::Es),
            "Sin controles dedicados en este dock."
        );
        assert_eq!(t("inspector.size_label", Locale::Es), "tamaño");
        assert_eq!(
            t("inspector.edge_width_label", Locale::Es),
            "Grosor de aristas"
        );
        // Wiring: títulos + sliders por dispatcher, implícitos por `_for`.
        let source = include_str!("panels.rs");
        for pin in [
            "inspector_chrome_for(\"cube3d_title\", inspector_locale)",
            "inspector_chrome_for(\"sphere3d_title\", inspector_locale)",
            "inspector_chrome_for(\"point3d_title\", inspector_locale)",
            "inspector_chrome_for(\"center_label\", inspector_locale)",
            "inspector_chrome_for(\"implicit3d_title\", inspector_locale)",
            "inspector_chrome_for(\"dock_empty_title\", inspector_locale)",
            "inspector_chrome_for(\"size_label\", inspector_locale)",
            "inspector_chrome_for(\"radius_label\", inspector_locale)",
            "inspector_chrome_for(\"scale_label\", inspector_locale)",
            "inspector_chrome_for(\"edge_width_label\", inspector_locale)",
            "implicit_3d_box_text_for(",
            "implicit_3d_state_text_for(status, inspector_locale)",
            "implicit_3d_preview_text_for(tris, inspector_locale)",
        ] {
            assert!(source.contains(pin), "inspector B-A112 cablea {pin}");
        }
        // Blindaje: las 11 consts + 3 fns canónicas ya no se llaman en el
        // draw (quedan solo en wrappers `_for` del dueño + este test
        // partido en fragmentos).
        // (Fragmentos porque `include_str!` incluye este test.)
        for needle in [
            ["INSPECTOR_CUBE", "3D_TITLE"].concat(),
            ["INSPECTOR_SPHERE", "3D_TITLE"].concat(),
            ["INSPECTOR_POINT", "3D_TITLE"].concat(),
            ["INSPECTOR_CEN", "TER_LABEL"].concat(),
            ["INSPECTOR_IMPLICIT", "3D_TITLE"].concat(),
            ["INSPECTOR_DOCK_EMPTY", "_TITLE"].concat(),
            ["INSPECTOR_DOCK_EMPTY", "_GUIDE"].concat(),
            ["INSPECTOR_SI", "ZE_TEXT"].concat(),
            ["INSPECTOR_RAD", "IUS_TEXT"].concat(),
            ["INSPECTOR_SCA", "LE_TEXT"].concat(),
            ["INSPECTOR_EDGE_WI", "DTH_TEXT"].concat(),
            ["implicit_3d_box_te", "xt("].concat(),
            ["implicit_3d_state_te", "xt("].concat(),
            ["implicit_3d_preview_te", "xt("].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "inspector B-A112 sin ES duro en prod: {needle}"
            );
        }
    }

    #[test]
    fn inspector_b_a113_preview_labels_salen_de_inspector_msg() {
        // B-A113: vista previa implícita sale de `inspector.*` (10 keys)
        // con `inspector_locale`; el botón Cancelar reutiliza
        // `common.cancel` (precedente B-A66/B-A95). ES pineado a los
        // literales previos de `sheet_layout.rs:4204-4218`. Sin egui:
        // pines + blindaje por `include_str!`.
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(
            t("inspector.implicit_quick_guide", Locale::Es),
            "Celdas altas: el fino tarda; la vista previa gruesa (12³) llega primero."
        );
        assert_eq!(
            t("inspector.implicit_quick_label", Locale::Es),
            "Vista previa rápida"
        );
        assert_eq!(
            t("inspector.implicit_start_failed", Locale::Es),
            "No se pudo iniciar la vista previa (revisá la superficie)."
        );
        assert_eq!(
            t("inspector.implicit_cancelled", Locale::Es),
            "Vista previa cancelada."
        );
        // Wiring: guía/label/hover + notify por `_for` + cancel por
        // `common.cancel` + hovers por dispatcher.
        let source = include_str!("panels.rs");
        for pin in [
            "inspector_chrome_for(\"implicit_quick_guide\"",
            "inspector_chrome_for(\"implicit_quick_label\"",
            "inspector_chrome_for(\"implicit_quick_hover\"",
            "inspector_chrome_for(\"implicit_cancel_empty\"",
            "inspector_chrome_for(\"implicit_cancel_hover\"",
            "inspector_chrome_for(\"implicit_cancelled\"",
            "implicit_quick_preview_notify_for(",
            "implicit_cancel_label_for(inspector_locale)",
        ] {
            assert!(source.contains(pin), "inspector B-A113 cablea {pin}");
        }
        // Blindaje: las 7 consts ES ya no se llaman en el draw (quedan
        // solo en el dispatcher `_for` del dueño + este test partido en
        // fragmentos). `IMPLICIT_PREVIEW_START_FAILED` sigue en prod a
        // propósito (el `kind` Error se decide sobre el canónico).
        // (Fragmentos porque `include_str!` incluye este test.)
        for needle in [
            ["IMPLICIT_QUICK_PRE", "VIEW_GUIDE"].concat(),
            ["IMPLICIT_QUICK_PRE", "VIEW_LABEL"].concat(),
            ["IMPLICIT_QUICK_PRE", "VIEW_HOVER"].concat(),
            ["IMPLICIT_PREVIEW_CAN", "CELLED"].concat(),
            ["IMPLICIT_PREVIEW_CAN", "CEL_EMPTY"].concat(),
            ["IMPLICIT_PREVIEW_CAN", "CEL_HOVER"].concat(),
            ["IMPLICIT_PREVIEW_CAN", "CEL_LABEL"].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "inspector B-A113 sin ES duro en prod: {needle}"
            );
        }
    }

    #[test]
    fn inspector_b_a159_apariencia_y_cierres_salen_de_inspector_msg() {
        // B-A159: apariencia 4D + cierres salen de `inspector.*` (8 keys)
        // con `inspector_locale`/`trig_locale`; `trig_header` suma param
        // `close_label` (1 caller). ES pineado a los literales previos de
        // `panels.rs` (`:4004` Color de aristas, `:4008` Cambiar…,
        // `:4019` Relleno habilitado, `:4031` Color de relleno,
        // `:4035` Cambiar…, `:4163` Dimensión, `:527` Cerrar panel,
        // `:2178` Cerrar animación). Sin egui: pines + blindaje.
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(
            t("inspector.edge_color_label", Locale::Es),
            ["Color de ar", "istas"].concat()
        );
        assert_eq!(
            t("inspector.fill_enabled_label", Locale::Es),
            ["Relleno habi", "litado"].concat()
        );
        assert_eq!(
            t("inspector.dimension_label", Locale::Es),
            ["Dimen", "sión"].concat()
        );
        assert_eq!(
            t("inspector.close_anim", Locale::Es),
            ["Cerrar ani", "mación"].concat()
        );
        // Wiring: lets por dispatcher + cierres + param del header trig.
        let source = include_str!("panels.rs");
        for pin in [
            "inspector_chrome_for(\"edge_color_label\"",
            "inspector_chrome_for(\"edge_color_hover\"",
            "inspector_chrome_for(\"fill_enabled_label\"",
            "inspector_chrome_for(\"fill_color_label\"",
            "inspector_chrome_for(\"fill_color_hover\"",
            "inspector_chrome_for(\"dimension_label\"",
            "inspector_chrome_for(\"close_context\"",
            "inspector_chrome_for(\"close_anim\"",
            "close_label: &str",
            "inspector_edge_color_label",
            "inspector_dimension_label",
        ] {
            assert!(source.contains(pin), "inspector B-A159 cablea {pin}");
        }
        // Blindaje: los 8 ES ya no se llaman en el draw (quedan solo en
        // las consts canónicas del dueño `sheet_layout.rs` + este test
        // partido en fragmentos por `include_str!` auto-match).
        for needle in [
            ["\"Color de ar", "istas\""].concat(),
            ["\"Cambiar color de ar", "istas\""].concat(),
            ["\"Relleno habi", "litado\""].concat(),
            ["\"Color de re", "lleno\""].concat(),
            ["\"Cambiar color de re", "lleno\""].concat(),
            ["\".text(\"Dimen", "sión\")"].concat(),
            ["\"Cerrar panel con", "textual\""].concat(),
            ["\"Cerrar ani", "mación\""].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "inspector B-A159 sin ES duro en prod: {needle}"
            );
        }
    }

    #[test]
    fn click_de_paso_selecciona_el_objeto_del_documento() {
        let mut app = crate::app::dummy_grafito_app();
        app.execute_command_and_record("A=(1,2)", 0.0);
        // El comando ya registra su propio paso: acá controlamos el log a mano.
        app.construction_log.clear();
        app.record_construction_step("Punto", vec![], "A");
        assert!(select_protocol_step(&mut app, 0), "paso 0 selecciona A");
        let selected = app.selected_object.expect("selección viva");
        assert_eq!(
            app.document
                .get_object(selected)
                .map(|object| object.label().to_string())
                .as_deref(),
            Some("A")
        );
        // Paso inexistente o salida no etiquetada: false sin pánico.
        assert!(!select_protocol_step(&mut app, 9));
        app.record_construction_step("Comando", vec![], "");
        assert!(!select_protocol_step(&mut app, 1));
    }
}
