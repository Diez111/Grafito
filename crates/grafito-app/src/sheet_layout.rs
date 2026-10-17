//! Layout + datos + tablas + prob + import CSV + hoja CAS + estados puros
//! + textos del Inspector + protocolo puro + setters complejos + grilla export
//! + estado visual W-C + estados efímeros trig/capas + objetos de import
//! + lecturas del panel trigonométrico + textos puros de Capas
//! + geometría pura del plot de probabilidad + ventanas restantes del plot
//! + resultado puntual del caso de probabilidad + muestreo puro del plot
//! + textos puros de la sección Regresión + chips de inferencia
//! + textos del caso de probabilidad
//! + token de alto de botones (`PANEL_BUTTON_H` = 28.0)
//! + textos de copiado/export de `draw_export_section`
//! + textos del prompt de velocidad trigonométrica
//! + acciones rápidas + textos vacíos del panel CAS
//! + textos del header + entrada + celdas del panel CAS
//! + header de la sección de acciones rápidas del panel CAS
//! + textos de `draw_export_section` (slice 46) + textos del panel Complejos
//!   (slice 47) + textos del panel Atractores (slice 48)
//! + textos del Inspector propiedades/identidad/estilo (slice 49)
//! + textos del panel Parámetros dinámicos (slice 50)
//! + textos del panel Coloración de dominio (slice 51)
//! + textos del panel vacío (slice 52)
//! + textos General+Ejes de `draw_view_panel` (slice 53)
//! + textos Vista-3D + parámetro vivo de `draw_view_panel` (slice 54)
//! + textos Capas/Indicar/Alta-precisión de `draw_view_panel` (slice 55)
//! + textos de `draw_construction_protocol` (slice 56)
//! + textos del panel de regresión (slice 57)
//! + textos 3D + fallback del dock de `draw_right_properties_contents`
//!   (slice 58)
//! + botones de vista previa implícita 3D de `draw_right_properties_contents`
//!   (slice 59)
//! + textos 4D/N-D del Inspector de `draw_right_properties_contents`
//!   (slice 60)
//! + primitivas 3D/4D del Inspector de `draw_right_properties_contents`
//!   (slice 61: prefijos de ejes + velocidad + textos tamaño/radio/
//!   Escala/Grosor + rangos cubo/esfera/punto/escala/ancho)
//!   (slices 1-52 de F14.4.4).
//!
//! Extracción del split de `panels.rs`: solo funciones/estado puros de la
//! hoja, sin egui, sin I/O y sin estado de app. Todo lo movido acá se
//! comporta byte a byte igual que en `panels.rs`: el dibujo
//! (`draw_sheet_editable_grid`, `draw_sheet_summary_card`,
//! `draw_sheet_series_row`) y los tests (`gc_piel_tests`) consumen estos
//! mismos símbolos vía import. Slice 13 mueve las definiciones de
//! `SheetEditState`/`SheetSummaryState`/`SheetSeriesState` (datos puros sin
//! egui: el uso `ctx.data` + dibujo queda en `panels.rs` a propósito).
//! Slice 14 mueve los textos puros de la vista de tablas
//! (`sheet_table_rows`/`sheet_table_shown`/`sheet_table_subtitle` +
//! overflows de filas/tablas: mismas cadenas que el dibujo).
//! Slice 15 mueve los anchos puros de la hoja (`stat_tile_width_for` +
//! `sheet_series_field_w` + `sheet_summary_field_w`: mismas fórmulas que
//! el dibujo, sin egui).
//! Slice 16 mueve el layout puro del explorador trigonométrico
//! (`TRIG_*` + `trig_function_columns` + `trig_grid_cell_w`: mismas
//! fórmulas que el dibujo, sin egui).
//! Slice 17 mueve los textos puros del Inspector (`inspector_equation_text` +
//! `inspector_type_caption` + `inspector_label_text` +
//! `inspector_identity_tooltip` + `line_style_label` + `point_style_label`:
//! mismas cadenas que el dibujo, sin egui; `inspector_type_caption` conserva
//! la rama `quadric_uses_placeholder` del render).
//! Slice 18 mueve lo puro del protocolo de construcción (`escape_latex` +
//! `construction_log_to_latex` + `ProtocolNavState` +
//! `PROTOCOL_PLAY_INTERVAL_SECS` + `protocol_advance`: mismas cadenas y
//! misma máquina de pasos que el dibujo, sin egui; `select_protocol_step`
//! queda en `panels.rs` porque muta el documento).
//! Slice 19 mueve la primitiva de edición de los paneles
//! (`apply_object_panel_edit` + `apply_object_panel_edit_with_previous`:
//! reemplazo estadificado con snapshot previo, sin egui ni I/O; los call
//! sites del dibujo quedan en `panels.rs`/`algebra.rs` por import).
//! Slice 20 mueve los setters puros del panel de complejos
//! (`set_domain_coloring_mode` + `set_complex_mapping_animation`:
//! comparan antes de tocar vía `get_object`, mutan vía
//! `get_object_mut_touch`, sin egui ni I/O; el dibujo
//! `draw_right_domain_coloring_panel` queda en `panels.rs`).
//! Slice 21 mueve el layout puro de la grilla de exportación
//! (`export_grid_columns` + `export_grid_cell_w`: mismas fórmulas que
//! `draw_export_section`, sin egui; 3 columnas en panel normal, 1 bajo
//! 200 px con celdas + gaps sin desbordar).
//! Slice 22 mueve el estado visual W-C (`WC_QUADRATURE_METHODS` +
//! `WC_RIEMANN_SLIDER_MAX` + `WcVisualState` con su `Default` de ejemplos
//! que evalúan + `wc_quadrature_method` con clamp: mismas cadenas y mismo
//! default que el dibujo, sin egui; el dibujo (`draw_wc_*`) y el estado
//! efímero `ctx.data` quedan en `panels.rs` a propósito).
//! Slice 23 mueve los estados efímeros trig/capas (`LayerPanelState` +
//! `TrigPromptState`: mismos defaults que `ctx.data` usa vía
//! `unwrap_or_default()`, sin egui; el dibujo queda en `panels.rs`).
//! Slice 24 mueve el armado puro tabla→objetos (`local_xy_table_objects`:
//! misma construcción `DataTable` + `ScatterPlot` linkeada que
//! `commit_local_xy_table`, sin egui ni I/O; el commit con undo queda en
//! `panels.rs` por import).
//! Slice 25 mueve las lecturas puras del panel trigonométrico
//! (`trig_function_chip_label` + `trig_speed_text` + `trig_angle_text` +
//! `trig_value_text` + `trig_point_text` + `trig_complex_text`: mismas
//! cadenas que `draw_trig_animation_panel`, sin egui; el dibujo queda en
//! `panels.rs`).
//! Slice 26 mueve los textos puros de la sección Capas de `draw_view_panel`
//! (`LAYER_EMPTY_HINT` + `LAYER_NO_SELECTION` + `layer_display_name` +
//! `layer_row_label` + `layer_toggle_result` + `layer_selection_text` +
//! `layer_assign_result` + `layer_assign_error`: mismas cadenas que el
//! dibujo, sin egui; el dibujo queda en `panels.rs`).
//! Slice 27 mueve el ancho puro del campo del prompt trig
//! (`trig_prompt_field_w`: misma fórmula que `draw_trig_speed_prompt`,
//! sin egui; el dibujo queda en `panels.rs`).
//! Slice 28 mueve la geometría pura del plot de probabilidad
//! (`clamp_binomial_n` + `clamp_binomial_k` + `binomial_plot_window` +
//! `binomial_plot_left_label`/`binomial_plot_right_label` +
//! `normal_band_fractions`: mismos clamps, ventana, bandas y rótulos que
//! `draw_probability_plot`, sin egui; el dibujo queda en `panels.rs`).
//! Slice 29 mueve las ventanas puras restantes del plot (`normal_plot_window`
//!     + `T_PLOT_LO/HI` + `discrete_k_sel` + `poisson_plot_hi` +
//!     `geometric_plot_hi` + `discrete_plot_left_label`/
//!     `discrete_plot_right_label` + `chi_squared_plot_hi` + `f_plot_hi`:
//!     mismas ventanas, clamps y rótulos que `draw_probability_plot`,
//!     sin egui; el dibujo queda en `panels.rs`).
//! Slice 30 mueve la tabla de ejemplo (`example_sheet_table` +
//! `example_sheet_table_loaded_text` + `example_sheet_table_error_text`:
//! mismo `DataTableObj` x: 0..4 / y: x² con etiqueta "ejemplo" y mismas
//! cadenas que `draw_spreadsheet_section`, sin egui; el commit con undo
//! y los `notify` quedan en `panels.rs`).
//! Slice 31 mueve el resultado puntual del caso (`prob_case_result`:
//! mismas 9 ramas, mismos detalles y mismos errores honestos que el
//! closure inline de `draw_probability_section`, sin egui; el dibujo
//! queda en `panels.rs`).
//! Slice 32 mueve los textos puros del selector de distribución
//! (`prob_dist_name` + `prob_dist_kind_label` + `prob_dist_tooltip` +
//! `prob_point_label`: mismo nombre con caída "Normal", mismo tooltip
//! "{kind} · {tip}" con `None` si el índice es desconocido y mismo
//! rótulo "Puntual"/"Densidad" que `draw_probability_section`, sin
//! egui; el combo y el dibujo quedan en `panels.rs`).
//! Slice 33 mueve el muestreo puro del plot (`continuous_density_samples` +
//! `discrete_ymax` + `binomial_plot_masses`/`poisson_plot_masses`/
//! `geometric_plot_masses`: mismas 61 muestras, mismo `max(0.0)` + filtro
//! finito, mismo piso 1e-12 y mismo tope 256 que `paint_continuous_density`
//! y las ramas 1/2/6 de `draw_probability_plot`, sin egui; el pintado
//! queda en `panels.rs`).
//! Slice 34 mueve el ancho máximo Datos/Prob + el texto del aviso del plot
//! (`PANEL_DATA_MAX_WIDTH` + `panel_data_max_width_for`: mismo clamp que
//! `draw_data_panel`/`draw_statistics_panel`, sin egui; + `plot_message_text`:
//! misma cadena "fuera de cota: {mensaje}" que `paint_plot_message`, sin
//! egui; el `SidePanel` y el pintor quedan en `panels.rs`).
//! Slice 35 mueve el prompt puro de tablas→asistente
//! (`data_tables_assistant_prompt`: mismo resumen hasta 4 tablas con
//! preview de 12 valores + misma guía sin tablas que
//! `open_data_tables_in_assistant`, sin egui ni I/O; el drawer queda en
//! `panels.rs`).
//! Slice 36 mueve los textos puros de `draw_right_regression_panel`
//! (`REGRESSION_*` + `regression_*`: mismas cadenas que el dibujo, sin
//! egui; el `SidePanel` y los `RichText`/colores quedan en `panels.rs`).
//! Slice 37 mueve los chips puros de inferencia del panel de estadística
//! (`statistics_quick_chips` + `statistics_quick_ready_text` +
//! `STATISTICS_*_READY`/`*_TOO_MANY`/`*_NO_INLINE_*`: mismas 6 tuplas y
//! mismas cadenas que `draw_statistics_panel`, sin egui; los botones,
//! el `notify` y el drawer quedan en `panels.rs`).
//! Slice 38 mueve las filas de la grilla "Resumen" + el binner del
//! histograma (`statistics_summary_rows` + `statistics_histogram_counts` +
//! `STATISTICS_SUMMARY_BINS`/`STATISTICS_SUMMARY_UNREPRESENTABLE`: mismo
//! orden, mismas cadenas y misma fórmula que `draw_statistics_panel`,
//! sin egui; la grilla, el pintor y los botones quedan en `panels.rs`).
//! Slice 39 mueve los textos puros de la fila del caso de probabilidad
//! (`prob_tail_text` + `prob_moment_text` + `prob_case_copy_text` +
//! `prob_case_detail_tooltip`: mismas caídas "—", mismo "{:.6}" y mismas
//! cadenas que `draw_probability_section`, sin egui; los botones Copiar,
//! los tiles y el pintor quedan en `panels.rs`).
//! Slice 40 mueve el token de alto de botones (`PANEL_BUTTON_H` = 28.0:
//! mismo valor que `panels.rs`, sin egui; los ~20 call sites del dibujo
//! lo consumen por import y `SHEET_CONTROL_H`/`TRIG_CONTROL_H` quedan
//! pineados al mismo ritmo vertical).
//! Slice 41 mueve los textos puros de copiado/export de
//! `draw_export_section` (`export_svg_copied_text` +
//! `export_svg_error_text` + `export_clipboard_copied_text` +
//! `export_table_row_label`: mismas cadenas que el dibujo, sin egui;
//! los botones, los `notify` y los workers quedan en `panels.rs`).

use crate::app::ConstructionStep;
use crate::complex_contour::ComplexContourMode;

use grafito_core::symbolic::series as spreadsheet_series;
use grafito_core::{
    DataTableObj, Document, GeoObject, LineStyle, ObjectId, PointStyle, ScatterPlotObj,
};
use grafito_geometry::{RegularPolychoron, RegularPolytopeFamily};
use grafito_ui::i18n::{t as trig_i18n_t, Locale};
use grafito_ui::tokens::{HIT_TARGET_MIN, SPACE_SM, SPACE_XS};
use std::collections::{BTreeMap, HashMap, VecDeque};

/// Ventana visible de la hoja vinculada (F3b). La hoja real vive en el
/// documento (`Document::MAX_SPREADSHEET_ROWS/COLS = 400×400`,
/// `MAX_SPREADSHEET_RECOMPUTE_CELLS = 10_000`): la UI solo muestra esta
/// ventana navegable de hasta 8×14 por rendimiento (`SheetViewState` guarda
/// el origen; las columnas visibles bajan a 3..=4 si el panel es angosto
/// para que nunca se corten); el resto se alcanza con las flechas, con
/// `Ir a A1` o con `FillColumn`/`FillCells`/`FillRow` y la serie de abajo
/// (`FillSeries`, mismo motor). No es infinita como Excel a propósito: el
/// presupuesto 400×400/10k del core mantiene la recomputación acotada y
/// honesta.
pub(crate) const SHEET_VIEW_COLS: usize = 8;
pub(crate) const SHEET_VIEW_ROWS: usize = 14;

/// Alto de celdas editables de la hoja (slice 12: movido verbatim de
/// `panels.rs`; token de layout, sin egui).
pub(crate) const SHEET_CELL_H: f32 = 24.0;
/// Alto ÚNICO de los controles de la hoja (chip de rango, flechas, «Ir a» y
/// sus botones): un solo token para que todos midan exactamente lo mismo.
/// 28 = `PANEL_BUTTON_H`: un solo ritmo vertical en todo el panel.
pub(crate) const SHEET_CONTROL_H: f32 = 28.0;

/// Alto de botones de panel (≥ `HIT_TARGET_MIN` 24, WCAG 2.5.8).
/// Verbatim de `panels.rs` (slice 40 F14.4.4): dueño nuevo del token que
/// ya citaban `SHEET_CONTROL_H` y `TRIG_CONTROL_H` como literal 28.0.
pub(crate) const PANEL_BUTTON_H: f32 = 28.0;

/// Ancho responsive de celda de hoja: reparte el ancho entre 6 columnas.
/// Columnas visibles de la hoja según el ancho real del contenido (3..=6).
/// La grilla entra siempre sin scroll horizontal: celdas ≥ 44 px mientras
/// el ancho lo permita (táctil WCAG 2.5.8), mínimo duro 36 px.
pub(crate) fn visible_sheet_cols(available_width: f32) -> usize {
    (((available_width - 32.0) / 48.0).floor() as i64).clamp(3, SHEET_VIEW_COLS as i64) as usize
}

/// Ancho de celda para `cols` columnas visibles: descuenta rótulo de fila,
/// espaciado y margen del marco; nunca desborda el ancho disponible.
pub(crate) fn sheet_cell_width_for(available_width: f32, cols: usize) -> f32 {
    let cols = cols.max(1) as f32;
    ((available_width - 32.0 - cols * SPACE_XS) / cols).clamp(36.0, 96.0)
}

/// Origen (esquina superior izquierda) de la ventana navegable de la hoja.
/// Vive en `ctx.data` como `SheetEditState`: cero campos nuevos en
/// `GrafitoApp`, cero I/O.
#[derive(Debug, Clone, Default)]
pub(crate) struct SheetViewState {
    pub(crate) origin_row: usize,
    pub(crate) origin_col: usize,
}

impl SheetViewState {
    /// Recorta el origen a la hoja real del documento.
    pub(crate) fn clamped(&self) -> Self {
        let max_col = Document::MAX_SPREADSHEET_COLS.saturating_sub(SHEET_VIEW_COLS);
        let max_row = Document::MAX_SPREADSHEET_ROWS.saturating_sub(SHEET_VIEW_ROWS);
        Self {
            origin_row: self.origin_row.min(max_row),
            origin_col: self.origin_col.min(max_col),
        }
    }

    /// Etiqueta del rango visible (`C3:H10`) para `cols` columnas visibles.
    pub(crate) fn window_label(&self, cols: usize) -> String {
        let cols = cols.clamp(1, SHEET_VIEW_COLS);
        let first = format!(
            "{}{}",
            sheet_col_label(self.origin_col),
            self.origin_row + 1
        );
        let last = format!(
            "{}{}",
            sheet_col_label(self.origin_col + cols - 1),
            self.origin_row + SHEET_VIEW_ROWS
        );
        format!("{first}:{last}")
    }

    /// Mueve la ventana para que la celda `(row, col)` quede en la página
    /// visible (esquina superior izquierda del bloque de `cols`×14), con el
    /// mismo clamp que la navegación con flechas.
    pub(crate) fn focus_cell(&mut self, row: usize, col: usize, cols: usize) {
        let cols = cols.clamp(1, SHEET_VIEW_COLS);
        self.origin_row = (row / SHEET_VIEW_ROWS) * SHEET_VIEW_ROWS;
        self.origin_col = (col / cols) * cols;
        *self = self.clamped();
    }
}

/// Borradores + errores por celda de la hoja editable (slice 13: movido
/// verbatim de `panels.rs`; datos puros sin egui — vive en `ctx.data` pero
/// la fuente canónica sigue en `Document.spreadsheet`; acá solo el texto
/// en edición y el último error honesto por celda).
#[derive(Debug, Clone, Default)]
pub(crate) struct SheetEditState {
    pub(crate) drafts: HashMap<(usize, usize), String>,
    pub(crate) errors: HashMap<(usize, usize), String>,
    /// Celda en edición (muestra la fórmula). El resto muestra el valor
    /// calculado, como una planilla real: clic para editar, Enter para confirmar.
    pub(crate) editing: Option<(usize, usize)>,
    /// Texto del salto directo ("Ir a C3") y su último error honesto.
    pub(crate) goto: String,
    pub(crate) goto_error: Option<String>,
}

/// Estado efímero de la fila de autorrelleno por serie (slice 13: movido
/// verbatim de `panels.rs`; vive en `ctx.data`, sin I/O): rango 1D +
/// inicio/paso como texto + modo.
#[derive(Debug, Clone)]
pub(crate) struct SheetSeriesState {
    pub(crate) range: String,
    pub(crate) start: String,
    pub(crate) step: String,
    pub(crate) geometric: bool,
    pub(crate) error: Option<String>,
}

impl Default for SheetSeriesState {
    fn default() -> Self {
        Self {
            range: "A1:A8".to_string(),
            start: "1".to_string(),
            step: "1".to_string(),
            geometric: false,
            error: None,
        }
    }
}

/// Estado efímero del resumen de rango (slice 13: movido verbatim de
/// `panels.rs`; rango escrito + último resultado).
#[derive(Debug, Clone)]
pub(crate) struct SheetSummaryState {
    pub(crate) range: String,
    pub(crate) summary: Option<RangeSummary>,
    pub(crate) error: Option<String>,
}

impl Default for SheetSummaryState {
    fn default() -> Self {
        Self {
            range: "A1:A8".to_string(),
            summary: None,
            error: None,
        }
    }
}

pub(crate) fn sheet_col_label(col: usize) -> String {
    let mut column = col;
    let mut letters = String::new();
    loop {
        letters.push(char::from(b'A' + (column % 26) as u8));
        if column < 26 {
            break;
        }
        column = column / 26 - 1;
    }
    letters.chars().rev().collect()
}

/// Métricas puras de la navegación de la hoja (F13.1): reparte el ancho
/// disponible entre el chip de rango, las cuatro flechas (fila 1) y
/// «Ir a» + Ir + A1 (fila 2). La suma de cada fila nunca desborda el panel
/// real (contrato ≥ 112 px): el chip cede primero (0..64), luego las flechas
/// bajan a `HIT_TARGET_MIN`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SheetNavMetrics {
    pub(crate) chip_w: f32,
    pub(crate) arrow_w: f32,
    pub(crate) field_w: f32,
    pub(crate) action_w: f32,
}

pub(crate) fn sheet_nav_metrics(available: f32) -> SheetNavMetrics {
    // Nunca se inventa ancho: si el panel es más angosto que 156 px el chip
    // cede por debajo de 44 (hasta 0) en vez de desbordar el marco.
    let available = available.max(HIT_TARGET_MIN);
    let chip_w = (available - 4.0 * HIT_TARGET_MIN - 4.0 * SPACE_XS).clamp(0.0, 64.0);
    let arrow_w = ((available - chip_w - 4.0 * SPACE_XS) / 4.0).max(HIT_TARGET_MIN);
    let action_w =
        ((available - 2.0 * SPACE_XS - HIT_TARGET_MIN) / 3.0).clamp(HIT_TARGET_MIN, 56.0);
    let field_w = (available - 2.0 * SPACE_XS - 2.0 * action_w).max(HIT_TARGET_MIN);
    SheetNavMetrics {
        chip_w,
        arrow_w,
        field_w,
        action_w,
    }
}

/// Cota del resumen de rango: más allá el panel lo rechaza con error
/// honesto en vez de reconstruir miles de celdas por frame.
pub(crate) const MAX_SUMMARY_CELLS: usize = 2000;

/// Resumen estadístico 1D de un rango de la hoja (n, media, s muestral,
/// mín, máx). Pura sobre `Document`; `Err` honesto si el rango es inválido,
/// 2D, excede [`MAX_SUMMARY_CELLS`] o no tiene valores numéricos.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RangeSummary {
    pub(crate) count: usize,
    pub(crate) mean: f64,
    pub(crate) sample_sd: Option<f64>,
    pub(crate) min: f64,
    pub(crate) max: f64,
}

pub(crate) fn summarize_range(document: &Document, range: &str) -> Result<RangeSummary, String> {
    let cells = spreadsheet_series::parse_series_range(range)
        .map_err(|error| error.replace("FillSeries:", "Resumen:"))?;
    if cells.len() > MAX_SUMMARY_CELLS {
        return Err(format!(
            "Resumen: {} celdas excede el máximo {MAX_SUMMARY_CELLS}",
            cells.len()
        ));
    }
    let mut count = 0usize;
    let mut mean = 0.0f64;
    let mut m2 = 0.0f64;
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    // Welford en una pasada: la suma naive desbordaba a `inf` con valores
    // grandes finitos (A1=A2=1e308) y mostraba media/desvío `inf`/`NaN`.
    for value in cells
        .iter()
        .filter_map(|(row, col)| document.eval_spreadsheet_cell(*row, *col))
        .filter(|value| value.is_finite())
    {
        count += 1;
        let delta = value - mean;
        mean += delta / count as f64;
        m2 += delta * (value - mean);
        min = min.min(value);
        max = max.max(value);
    }
    if count == 0 {
        return Err("Resumen: el rango no tiene valores numéricos".to_string());
    }
    if !mean.is_finite() || !m2.is_finite() {
        return Err("Resumen: los valores desbordan la precisión de media/varianza".to_string());
    }
    let sample_sd = (count > 1).then(|| (m2 / (count - 1) as f64).sqrt());
    Ok(RangeSummary {
        count,
        mean,
        sample_sd,
        min,
        max,
    })
}

/// Prefijo localizado de `summarize_range_for` para errores del parser
/// (`FillSeries: …` → `Resumen:/Summary:/…`). Es delega al canónico
/// `"Resumen:"` (paridad byte-idéntica, patrón B-A98).
pub(crate) fn sheet_summary_prefix_for(locale: Locale) -> &'static str {
    match locale {
        Locale::Es => "Resumen:",
        Locale::En => "Summary:",
        Locale::Pt => "Resumo:",
        Locale::It => "Riepilogo:",
        Locale::Fr => "Résumé :",
        Locale::De => "Zusammenfassung:",
    }
}

/// `summarize_range` en el idioma pedido (B-A154, templates `{n}/{max}`).
/// Es delega al canónico [`summarize_range`] (paridad byte-idéntica;
/// `…`/`—`/backticks intactos como notación).
pub(crate) fn summarize_range_for(
    document: &Document,
    range: &str,
    locale: Locale,
) -> Result<RangeSummary, String> {
    use grafito_ui::i18n::Locale as L;
    if locale == L::Es {
        return summarize_range(document, range);
    }
    let cells = spreadsheet_series::parse_series_range(range)
        .map_err(|error| error.replace("FillSeries:", sheet_summary_prefix_for(locale)))?;
    if cells.len() > MAX_SUMMARY_CELLS {
        return Err(
            grafito_ui::i18n::stats_msg("sheet_summary_err_too_many", locale)
                .replace("{n}", &cells.len().to_string())
                .replace("{max}", &MAX_SUMMARY_CELLS.to_string()),
        );
    }
    let mut count = 0usize;
    let mut mean = 0.0f64;
    let mut m2 = 0.0f64;
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    for value in cells
        .iter()
        .filter_map(|(row, col)| document.eval_spreadsheet_cell(*row, *col))
        .filter(|value| value.is_finite())
    {
        count += 1;
        let delta = value - mean;
        mean += delta / count as f64;
        m2 += delta * (value - mean);
        min = min.min(value);
        max = max.max(value);
    }
    if count == 0 {
        return Err(grafito_ui::i18n::stats_msg("sheet_summary_err_empty", locale).to_string());
    }
    if !mean.is_finite() || !m2.is_finite() {
        return Err(grafito_ui::i18n::stats_msg("sheet_summary_err_overflow", locale).to_string());
    }
    let sample_sd = (count > 1).then(|| (m2 / (count - 1) as f64).sqrt());
    Ok(RangeSummary {
        count,
        mean,
        sample_sd,
        min,
        max,
    })
}

/// Escalar numérico del formulario de serie (acepta coma decimal).
/// `Err` honesto en español, nunca se inventa un valor.
pub(crate) fn parse_series_scalar(text: &str, what: &str) -> Result<f64, String> {
    let value: f64 = text
        .trim()
        .replace(',', ".")
        .parse()
        .map_err(|_| format!("Serie: {what} debe ser un número"))?;
    if value.is_finite() {
        Ok(value)
    } else {
        Err(format!("Serie: {what} debe ser finito"))
    }
}

/// Edits atómicos de la serie (parse + motor, sin commit ni undo).
/// Pura: misma validación que `Aplicar serie` del panel; el llamador
/// aplica con `stage_spreadsheet_cell_edits` de forma atómica.
pub(crate) fn build_series_edits(
    range: &str,
    start_text: &str,
    step_text: &str,
    geometric: bool,
) -> Result<Vec<(usize, usize, String)>, String> {
    let start = parse_series_scalar(start_text, "inicio")?;
    let step = parse_series_scalar(step_text, "paso")?;
    let kind = if geometric {
        spreadsheet_series::SeriesKind::Geometric
    } else {
        spreadsheet_series::SeriesKind::Linear
    };
    spreadsheet_series::build_fill_series(range, start, step, kind)
}

/// Texto de la vista previa en vivo (`{n} celdas: a, b, c…`).
/// Puro: mismo formato que el dibujo de la fila de serie.
pub(crate) fn format_series_preview_text(count: usize, first: &[String]) -> String {
    let mut text = format!("{count} celdas: {}", first.join(", "));
    if count > first.len() {
        text.push('…');
    }
    text
}

/// Texto de éxito tras aplicar (`Serie: N celda(s) rellenadas en RANGO`).
/// Puro: mismo formato que `Aplicar serie` (rango recortado).
pub(crate) fn format_series_success(count: usize, range: &str) -> String {
    format!("Serie: {count} celda(s) rellenadas en {}", range.trim())
}

/// Vista previa en el idioma pedido (B-A150, templates `{count}/{list}`).
/// Es delega al canónico [`format_series_preview_text`] (paridad
/// byte-idéntica, `…` de truncado intacto como notación).
pub(crate) fn format_series_preview_text_for(
    count: usize,
    first: &[String],
    locale: grafito_ui::i18n::Locale,
) -> String {
    use grafito_ui::i18n::Locale;
    if locale == Locale::Es {
        format_series_preview_text(count, first)
    } else {
        let mut text = grafito_ui::i18n::stats_msg("series_preview", locale)
            .replace("{count}", &count.to_string())
            .replace("{list}", &first.join(", "));
        if count > first.len() {
            text.push('…');
        }
        text
    }
}

/// Éxito tras aplicar en el idioma pedido (B-A150, templates
/// `{count}/{range}`). Es delega al canónico [`format_series_success`].
pub(crate) fn format_series_success_for(
    count: usize,
    range: &str,
    locale: grafito_ui::i18n::Locale,
) -> String {
    use grafito_ui::i18n::Locale;
    if locale == Locale::Es {
        format_series_success(count, range)
    } else {
        grafito_ui::i18n::stats_msg("series_success", locale)
            .replace("{count}", &count.to_string())
            .replace("{range}", range.trim())
    }
}

/// Hover del chip de rango de la hoja en el idioma pedido (B-A167, template
/// `{ventana}` con la etiqueta de `SheetViewState::window_label`, notación
/// intacta). Es delega al canónico por construcción (idéntico al `format!`
/// previo del draw en `spreadsheet.rs:109-112`).
pub(crate) fn sheet_chip_hint_text_for(
    window_label: &str,
    locale: grafito_ui::i18n::Locale,
) -> String {
    use grafito_ui::i18n::Locale;
    if locale == Locale::Es {
        format!(
            "Celda en edición (accent) o esquina visible de la hoja 400×400. Ventana {window_label} · 10 000 celdas vivas."
        )
    } else {
        grafito_ui::i18n::stats_msg("sheet_chip_hint", locale).replace("{ventana}", window_label)
    }
}

/// Vista previa honesta de la serie (cantidad + primeros 3 valores).
/// Pura: usa el mismo motor que `Aplicar`; `None` si rango/inicio/paso
/// no validan (el error detallado lo da `Aplicar`, acá no se inventa nada).
pub(crate) fn series_preview(
    range: &str,
    start_text: &str,
    step_text: &str,
    geometric: bool,
) -> Option<(usize, Vec<String>)> {
    let edits = build_series_edits(range, start_text, step_text, geometric).ok()?;
    if edits.is_empty() {
        return None;
    }
    let first: Vec<String> = edits
        .iter()
        .take(3)
        .map(|(_, _, value)| value.clone())
        .collect();
    Some((edits.len(), first))
}

/// Filas por tabla en la hoja viva (lectura): el resto se avisa.
pub(crate) const MAX_SPREADSHEET_ROWS: usize = 50;
/// Tablas visibles en la hoja viva: el resto se avisa.
pub(crate) const MAX_SPREADSHEET_TABLES: usize = 8;

/// Fila de la hoja viva: una tabla del documento con sus columnas clonadas
/// (lectura puntual por frame; las celdas editables están arriba).
#[derive(Debug, Clone)]
pub(crate) struct SheetTable {
    pub(crate) id: ObjectId,
    pub(crate) label: String,
    pub(crate) x_name: String,
    pub(crate) y_name: String,
    pub(crate) xs: Vec<f64>,
    pub(crate) ys: Vec<f64>,
}

/// Colecta pura de las tablas del documento para la hoja viva.
/// Mismo recorrido que el dibujo (`objects_iter_sorted` + `DataTable`).
pub(crate) fn collect_sheet_tables(document: &Document) -> Vec<SheetTable> {
    document
        .objects_iter_sorted()
        .filter_map(|(id, object)| match object {
            GeoObject::DataTable(table) => Some(SheetTable {
                id: *id,
                label: table.label.clone(),
                x_name: table.x_name.clone(),
                y_name: table.y_name.clone(),
                xs: table.xs.clone(),
                ys: table.ys.clone(),
            }),
            _ => None,
        })
        .collect()
}

/// Título visible de la tabla: etiqueta o marcador honesto si está vacía.
pub(crate) fn sheet_table_title(label: &str) -> String {
    if label.is_empty() {
        "<sin etiqueta>".to_string()
    } else {
        label.to_string()
    }
}

/// Título visible en el idioma pedido (B-A152, display-localizado).
/// Es delega al canónico [`sheet_table_title`] (paridad byte-idéntica,
/// precedente B-A98/B-A150); resto → `stats.sheet_table_untitled`.
pub(crate) fn sheet_table_title_for(label: &str, locale: Locale) -> String {
    use grafito_ui::i18n::Locale as L;
    if locale == L::Es {
        sheet_table_title(label)
    } else if label.is_empty() {
        grafito_ui::i18n::stats_msg("sheet_table_untitled", locale).to_string()
    } else {
        label.to_string()
    }
}

/// Nombre de columna acotado a 24 caracteres + `…` (sin tope empujaba el panel).
pub(crate) fn truncate_table_name(name: &str) -> String {
    if name.chars().count() > 24 {
        format!("{}…", name.chars().take(23).collect::<String>())
    } else {
        name.to_string()
    }
}

/// Filas reales de una tabla viva: el mínimo de ambas columnas (verbatim de
/// `draw_spreadsheet_section`; `xs`/`ys` pueden desalinearse por edición).
pub(crate) fn sheet_table_rows(xs_len: usize, ys_len: usize) -> usize {
    xs_len.min(ys_len)
}

/// Filas visibles por tabla: acotadas a `MAX_SPREADSHEET_ROWS`, el resto se
/// avisa con [`sheet_table_rows_overflow`].
pub(crate) fn sheet_table_shown(rows: usize) -> usize {
    rows.min(MAX_SPREADSHEET_ROWS)
}

/// Subtítulo de la card de tabla (`ejemplo · 5 filas`). Misma expresión que
/// el dibujo; extraído para testear sin egui.
pub(crate) fn sheet_table_subtitle(title: &str, rows: usize) -> String {
    format!("{title} · {rows} filas")
}

/// Subtítulo en el idioma pedido (B-A152, templates `{title}/{rows}`).
/// Es delega al canónico [`sheet_table_subtitle`]; resto interpola
/// `stats.sheet_table_subtitle` (`·` intacto, precedente B-A150).
pub(crate) fn sheet_table_subtitle_for(title: &str, rows: usize, locale: Locale) -> String {
    use grafito_ui::i18n::Locale as L;
    if locale == L::Es {
        sheet_table_subtitle(title, rows)
    } else {
        grafito_ui::i18n::stats_msg("sheet_table_subtitle", locale)
            .replace("{title}", title)
            .replace("{rows}", &rows.to_string())
    }
}

/// Aviso de filas recortadas por tabla. `None` si entra todo (el dibujo no
/// pinta nada en ese caso).
pub(crate) fn sheet_table_rows_overflow(rows: usize, shown: usize) -> Option<String> {
    if rows > shown {
        Some(format!(
            "…y {} filas más (vista acotada a {MAX_SPREADSHEET_ROWS})",
            rows - shown
        ))
    } else {
        None
    }
}

/// Aviso de filas en el idioma pedido (B-A152, templates `{rest}/{max}`).
/// Es delega al canónico [`sheet_table_rows_overflow`]; `None` si entra todo.
pub(crate) fn sheet_table_rows_overflow_for(
    rows: usize,
    shown: usize,
    locale: Locale,
) -> Option<String> {
    use grafito_ui::i18n::Locale as L;
    if rows <= shown {
        return None;
    }
    if locale == L::Es {
        return sheet_table_rows_overflow(rows, shown);
    }
    Some(
        grafito_ui::i18n::stats_msg("sheet_table_rows_overflow", locale)
            .replace("{rest}", &(rows - shown).to_string())
            .replace("{max}", &MAX_SPREADSHEET_ROWS.to_string()),
    )
}

/// Aviso de tablas recortadas de la sección. `None` si entran todas (el
/// dibujo no pinta nada en ese caso).
pub(crate) fn sheet_hidden_tables_overflow(total_tables: usize) -> Option<String> {
    let hidden = total_tables.saturating_sub(MAX_SPREADSHEET_TABLES);
    if hidden > 0 {
        Some(format!(
            "…y {hidden} tablas más (vista acotada a {MAX_SPREADSHEET_TABLES})"
        ))
    } else {
        None
    }
}

/// Aviso de tablas en el idioma pedido (B-A152, templates `{hidden}/{max}`).
/// Es delega al canónico [`sheet_hidden_tables_overflow`]; `None` si entran todas.
pub(crate) fn sheet_hidden_tables_overflow_for(
    total_tables: usize,
    locale: Locale,
) -> Option<String> {
    use grafito_ui::i18n::Locale as L;
    let hidden = total_tables.saturating_sub(MAX_SPREADSHEET_TABLES);
    if hidden == 0 {
        return None;
    }
    if locale == L::Es {
        return sheet_hidden_tables_overflow(total_tables);
    }
    Some(
        grafito_ui::i18n::stats_msg("sheet_hidden_tables_overflow", locale)
            .replace("{hidden}", &hidden.to_string())
            .replace("{max}", &MAX_SPREADSHEET_TABLES.to_string()),
    )
}

/// Modelos del panel de regresión (TABLE-REG, F10.W4): los 7 del motor
/// (`FitKind` en geometry + `Fit*` en registry). Función pura para testear
/// sin egui; la UI solo la consume.
pub(crate) fn regression_model_commands(table_label: &str) -> Vec<(String, String)> {
    vec![
        (
            REGRESSION_MODEL_LINEAR_NAME.to_string(),
            format!("FitLinear[{table_label}]"),
        ),
        (
            REGRESSION_MODEL_POLY2_NAME.to_string(),
            format!("FitPoly[{table_label}, 2]"),
        ),
        (
            REGRESSION_MODEL_EXP_NAME.to_string(),
            format!("FitExp[{table_label}]"),
        ),
        (
            REGRESSION_MODEL_LOG_NAME.to_string(),
            format!("FitLog[{table_label}]"),
        ),
        (
            REGRESSION_MODEL_POWER_NAME.to_string(),
            format!("FitPow[{table_label}]"),
        ),
        (
            REGRESSION_MODEL_SIN_NAME.to_string(),
            format!("FitSin[{table_label}]"),
        ),
        (
            REGRESSION_MODEL_LOGISTIC_NAME.to_string(),
            format!("FitLogistic[{table_label}]"),
        ),
    ]
}

/// Nombres display de los 7 modelos (B-A149, canónicos ES para la rama
/// Es del dispatcher; resto → `stats_msg`). Los comandos `Fit*` son
/// invariantes (precedente B-A98/B-A148).
pub(crate) const REGRESSION_MODEL_LINEAR_NAME: &str = "Lineal";
pub(crate) const REGRESSION_MODEL_POLY2_NAME: &str = "Polinómico 2";
pub(crate) const REGRESSION_MODEL_EXP_NAME: &str = "Exponencial";
pub(crate) const REGRESSION_MODEL_LOG_NAME: &str = "Logarítmico";
pub(crate) const REGRESSION_MODEL_POWER_NAME: &str = "Potencia";
pub(crate) const REGRESSION_MODEL_SIN_NAME: &str = "Sinusoidal";
pub(crate) const REGRESSION_MODEL_LOGISTIC_NAME: &str = "Logística";

/// Columnas de la fila de tiles del resumen: 3 si hay ancho para tres
/// tiles de 64 px, 2 o 1 si el panel se angosta. Nunca devuelve 0.
/// Pura sobre el ancho real; el dibujo (`stat_tile_row` en `panels.rs`)
/// solo la consume.
pub(crate) fn stat_tile_cols(available: f32) -> usize {
    const TILE_MIN_W: f32 = 64.0;
    (((available + SPACE_SM) / (TILE_MIN_W + SPACE_SM)).floor() as usize).clamp(1, 3)
}

/// Ancho de cada tile en una fila de `count` tiles sobre `available`.
/// Verbatim de `stat_tile_row` en `panels.rs`: reparte el ancho real
/// descontando el espaciado, piso duro 0. El dibujo siempre llama con
/// `count >= 1` (chunk de `chunks(cols)`, `cols >= 1`); con `count < 1`
/// devuelve 0 en vez de dividir por cero.
pub(crate) fn stat_tile_width_for(available: f32, count: usize) -> f32 {
    if count == 0 {
        return 0.0;
    }
    let count_f = count as f32;
    ((available - (count_f - 1.0) * SPACE_SM) / count_f).max(0.0)
}

/// Ancho de cada campo corto (Inicio/Paso) de la fila de serie.
/// Verbatim de `draw_sheet_series_row` en `panels.rs`: mitad del ancho
/// real menos el espaciado, piso 0 (a panel angosto cede, no desborda).
pub(crate) fn sheet_series_field_w(available: f32) -> f32 {
    ((available - SPACE_SM) / 2.0).max(0.0)
}

/// Ancho del botón «Resumir» de la card de resumen (slice 15: sale del
/// dibujo como token para que el helper de abajo lo reuse).
pub(crate) const SHEET_SUMMARY_BTN_W: f32 = 84.0;

/// Ancho del campo de rango de la card «Resumen de rango».
/// Verbatim de `draw_sheet_summary_card` en `panels.rs`: ancho real menos
/// botón y espaciado, piso `HIT_TARGET_MIN` (táctil).
pub(crate) fn sheet_summary_field_w(available: f32) -> f32 {
    (available - SHEET_SUMMARY_BTN_W - SPACE_SM).max(HIT_TARGET_MIN)
}

/// Ancho mínimo del lienzo 2D con el explorador trigonométrico abierto:
/// el panel nunca lo aplasta más allá de esto.
/// Verbatim de `panels.rs` (slice 16 F14.4.4): token de layout, sin egui.
pub(crate) const TRIG_CANVAS_MIN_W: f32 = 320.0;
/// Ancho por defecto del explorador trigonométrico (drawer derecho).
/// Verbatim de `panels.rs` (slice 16 F14.4.4).
pub(crate) const TRIG_PANEL_DEFAULT_W: f32 = 300.0;
/// Piso y techo del explorador trigonométrico: legible en ventanas chicas,
/// sin muro en anchas. Verbatim de `panels.rs` (slice 16 F14.4.4).
pub(crate) const TRIG_PANEL_MIN_W: f32 = 248.0;
/// Verbatim de `panels.rs` (slice 16 F14.4.4).
pub(crate) const TRIG_PANEL_MAX_W: f32 = 400.0;
/// Ancho mínimo de un chip de función trigonométrica
/// (texto + padding 16×8 del sistema). Verbatim de `panels.rs`.
pub(crate) const TRIG_CHIP_MIN_W: f32 = 74.0;
/// Ancho del botón «Aplicar» del prompt de velocidad trigonométrica
/// (texto + padding). Verbatim de `panels.rs`.
pub(crate) const TRIG_APPLY_W: f32 = 76.0;
/// Alto único de los controles del explorador trigonométrico
/// (≥ `HIT_TARGET_MIN`). Verbatim `PANEL_BUTTON_H` (= 28.0) de `panels.rs`:
/// se pinea como literal para no acoplar este módulo puro al dibujo.
pub(crate) const TRIG_CONTROL_H: f32 = 28.0;

/// Columnas del selector de funciones trigonométricas según el ancho real:
/// 6 en una fila si entran, 3×2 en paneles normales y 2×3 en el piso.
/// Nunca hay recorte. Verbatim de `panels.rs` (slice 16 F14.4.4).
pub(crate) fn trig_function_columns(available: f32) -> usize {
    let fits = |cols: usize| {
        available >= cols as f32 * TRIG_CHIP_MIN_W + (cols.saturating_sub(1) as f32) * SPACE_SM
    };
    if fits(6) {
        6
    } else if fits(3) {
        3
    } else {
        2
    }
}

/// Ancho exacto de cada celda del grid trigonométrico para `columns`
/// columnas separadas por `SPACE_SM`: celdas + gaps nunca superan
/// `available`. Verbatim de `panels.rs` (slice 16 F14.4.4).
pub(crate) fn trig_grid_cell_w(available: f32, columns: usize) -> f32 {
    let cols = columns.max(1) as f32;
    ((available - (cols - 1.0) * SPACE_SM) / cols).max(0.0)
}

/// Ancho del campo de texto del prompt de velocidad trigonométrica.
/// Verbatim de `draw_trig_speed_prompt` en `panels.rs` (slice 27 F14.4.4):
/// ancho real menos botón Aplicar y espaciados, piso 0 (a panel angosto
/// cede, no desborda; el botón conserva su ancho fijo `TRIG_APPLY_W`).
pub(crate) fn trig_prompt_field_w(available: f32) -> f32 {
    (available - TRIG_APPLY_W - 2.0 * SPACE_SM).max(0.0)
}

/// Textos puros del prompt de velocidad trigonométrica.
/// Verbatim de `draw_trig_speed_prompt` en `panels.rs` (slice 42 F14.4.4,
/// sin egui): rótulo "Auto" + hint del campo + tooltips + botón Aplicar.
pub(crate) const TRIG_PROMPT_AUTO_LABEL: &str = "Auto";
/// Verbatim de `panels.rs` (slice 42 F14.4.4).
pub(crate) const TRIG_PROMPT_HINT: &str = "1.5 · medio";
/// Verbatim de `panels.rs` (slice 42 F14.4.4).
pub(crate) const TRIG_PROMPT_FIELD_TOOLTIP: &str =
    "Pedí una velocidad con palabras o números y pulsa Aplicar";
/// Verbatim de `panels.rs` (slice 42 F14.4.4).
pub(crate) const TRIG_PROMPT_APPLY_LABEL: &str = "Aplicar";
/// Verbatim de `panels.rs` (slice 42 F14.4.4).
pub(crate) const TRIG_PROMPT_APPLY_TOOLTIP: &str = "Convierte el pedido en velocidad del slider";

/// Etiqueta del chip de función trigonométrica (`sin(t)`). Verbatim de
/// `draw_trig_animation_panel` en `panels.rs` (slice 25 F14.4.4, sin egui).
pub(crate) fn trig_function_chip_label(name: &str) -> String {
    format!("{name}(t)")
}

/// Lectura de velocidad del panel trigonométrico (`0.5 rad/s`). Verbatim de
/// `draw_trig_animation_panel` en `panels.rs` (slice 25 F14.4.4, sin egui).
pub(crate) fn trig_speed_text(speed: f64) -> String {
    format!("{speed:.1} rad/s")
}

/// Lectura de ángulo del panel trigonométrico (`0.00 rad`). Verbatim de
/// `draw_trig_animation_panel` en `panels.rs` (slice 25 F14.4.4, sin egui).
pub(crate) fn trig_angle_text(angle: f64) -> String {
    format!("{angle:.2} rad")
}

/// Lectura del punto vivo (`sin(0.00) = 0.0000` o `tan(1.57) no está
/// definido`). Verbatim de `draw_trig_animation_panel` en `panels.rs`
/// (slice 25 F14.4.4, sin egui; `fn_val` ya viene de `trig_value`).
pub(crate) fn trig_value_text(name: &str, t: f64, fn_val: f64) -> String {
    if fn_val.is_finite() {
        format!("{name}({t:.2}) = {fn_val:.4}")
    } else {
        format!("{name}({t:.2}) no está definido")
    }
}

/// Lectura del punto (`(cos θ, sin θ)`). Verbatim de
/// `draw_trig_animation_panel` en `panels.rs` (slice 25 F14.4.4, sin egui).
pub(crate) fn trig_point_text(t: f64) -> String {
    format!("Punto: (cos θ, sin θ) = ({:.3}, {:.3})", t.cos(), t.sin())
}

/// Lectura compleja (`z` unitario + argumento). Verbatim de
/// `draw_trig_animation_panel` en `panels.rs` (slice 25 F14.4.4, sin egui;
/// solo se muestra en perspectiva Complex).
pub(crate) fn trig_complex_text(t: f64) -> String {
    format!(
        "z = {:.3} {:+.3}i  |z| = 1  arg(z) = {t:.2}",
        t.cos(),
        t.sin()
    )
}

// ── B-A98: variantes display-localizadas del chrome trigonométrico ──
// Los canónicos Es (`TRIG_PROMPT_*`, `trig_value_text`, `trig_point_text`)
// quedan intactos y pineados por test; el Es sale del canónico por
// construcción (paridad byte-idéntica) y el resto de `trig.*`.
// Unidades SI (`rad/s`, `rad`), fórmula `z(t)`, notación `z` y chip
// `{name}(t)` no se traducen: no tienen clave.
// B-A162: `trig_speed_text`/`trig_angle_text`/`trig_complex_text` +
// `trig_function_chip_label` + fórmula `z(t)` son notación SI/matemática
// invariante a propósito (0 keys, precedente B-A114/B-A160): el draw ya
// localiza lo localizable (`section_speed`/`section_angle`/`value_undefined`/
// `point_label`/identidades B-A98, errores `prompt_err_*` B-A161) y las
// lecturas son números con unidad + `z`/`θ`/`|z|`/`arg` idénticos en los 6
// idiomas. Si se localizan, actualizar este pin +
// `trig_b_a162_lecturas_quedan_crudas_intencional` en `panels.rs`.

/// Etiqueta "Auto" del prompt de velocidad (`trig.prompt_auto`).
pub(crate) fn trig_prompt_auto_label(locale: Locale) -> &'static str {
    if locale == Locale::Es {
        TRIG_PROMPT_AUTO_LABEL
    } else {
        trig_i18n_t("trig.prompt_auto", locale)
    }
}

/// Hint del campo del prompt (`trig.prompt_hint`).
pub(crate) fn trig_prompt_hint_text(locale: Locale) -> &'static str {
    if locale == Locale::Es {
        TRIG_PROMPT_HINT
    } else {
        trig_i18n_t("trig.prompt_hint", locale)
    }
}

/// Tooltip del campo del prompt (`trig.prompt_field_tooltip`).
pub(crate) fn trig_prompt_field_tooltip(locale: Locale) -> &'static str {
    if locale == Locale::Es {
        TRIG_PROMPT_FIELD_TOOLTIP
    } else {
        trig_i18n_t("trig.prompt_field_tooltip", locale)
    }
}

/// Etiqueta del botón Aplicar (`trig.prompt_apply`).
pub(crate) fn trig_prompt_apply_label(locale: Locale) -> &'static str {
    if locale == Locale::Es {
        TRIG_PROMPT_APPLY_LABEL
    } else {
        trig_i18n_t("trig.prompt_apply", locale)
    }
}

/// Tooltip del botón Aplicar (`trig.prompt_apply_tooltip`).
pub(crate) fn trig_prompt_apply_tooltip(locale: Locale) -> &'static str {
    if locale == Locale::Es {
        TRIG_PROMPT_APPLY_TOOLTIP
    } else {
        trig_i18n_t("trig.prompt_apply_tooltip", locale)
    }
}

/// Lectura del punto vivo localizada (`trig.value_undefined` en la rama
/// no finita; la rama finita es notación, idéntica en los 6 idiomas).
pub(crate) fn trig_value_text_for(name: &str, t: f64, fn_val: f64, locale: Locale) -> String {
    if locale == Locale::Es {
        trig_value_text(name, t, fn_val)
    } else if fn_val.is_finite() {
        format!("{name}({t:.2}) = {fn_val:.4}")
    } else {
        format!(
            "{name}({t:.2}) {}",
            trig_i18n_t("trig.value_undefined", locale)
        )
    }
}

/// Lectura del punto localizada (`trig.point_label`; el resto es notación).
pub(crate) fn trig_point_text_for(t: f64, locale: Locale) -> String {
    if locale == Locale::Es {
        trig_point_text(t)
    } else {
        format!(
            "{}: (cos θ, sin θ) = ({:.3}, {:.3})",
            trig_i18n_t("trig.point_label", locale),
            t.cos(),
            t.sin()
        )
    }
}

/// Aviso cuando el documento no tiene objetos en ninguna capa. Verbatim de
/// `draw_view_panel` en `panels.rs` (slice 26 F14.4.4, sin egui).
pub(crate) const LAYER_EMPTY_HINT: &str = "Sin objetos: todo lo nuevo entra en la capa 0.";

/// Aviso cuando no hay objeto seleccionado en el lienzo. Verbatim de
/// `draw_view_panel` en `panels.rs` (slice 26 F14.4.4, sin egui).
pub(crate) const LAYER_NO_SELECTION: &str = "Sin selección: elegí un objeto en el lienzo.";

/// Nombre visible de una etiqueta de objeto: vacía → "<sin etiqueta>".
/// Verbatim de `draw_view_panel` en `panels.rs` (slice 26 F14.4.4, sin egui).
pub(crate) fn layer_display_name(label: &str) -> &str {
    if label.is_empty() {
        "<sin etiqueta>"
    } else {
        label
    }
}

/// Nombre visible en el idioma activo (B-A130): Es delega a la canónica
/// `layer_display_name`, resto a `view_msg("layer_untagged")` (patrón
/// `layer_toggle_label_for` B-A129; storage `label` intacto, solo display).
pub(crate) fn layer_display_name_for(label: &str, locale: grafito_ui::i18n::Locale) -> &str {
    use grafito_ui::i18n::Locale;
    if label.is_empty() {
        if locale == Locale::Es {
            layer_display_name(label)
        } else {
            grafito_ui::i18n::view_msg("layer_untagged", locale)
        }
    } else {
        label
    }
}

/// Fila de la lista de capas (`Capa 2 · 5 obj`). Verbatim de
/// `draw_view_panel` en `panels.rs` (slice 26 F14.4.4, sin egui).
pub(crate) fn layer_row_label(layer: u32, count: usize) -> String {
    format!("Capa {layer} · {count} obj")
}

/// Fila en el idioma activo (B-A130): Es delega a la canónica por
/// construcción, resto interpola el template `view.layer_row`
/// (`{layer}`/`{count}`; patrón B-A64/B-A75).
pub(crate) fn layer_row_label_for(
    layer: u32,
    count: usize,
    locale: grafito_ui::i18n::Locale,
) -> String {
    use grafito_ui::i18n::Locale;
    if locale == Locale::Es {
        layer_row_label(layer, count)
    } else {
        grafito_ui::i18n::view_msg("layer_row", locale)
            .replacen("{layer}", &layer.to_string(), 1)
            .replacen("{count}", &count.to_string(), 1)
    }
}

/// Resultado de alternar la visibilidad de una capa: `None` si no tocó
/// ningún objeto (el dibujo no escribe `cas_result` en ese caso).
/// Verbatim de `draw_view_panel` en `panels.rs` (slice 26 F14.4.4, sin egui).
pub(crate) fn layer_toggle_result(layer: u32, was_visible: bool, touched: usize) -> Option<String> {
    if touched > 0 {
        Some(format!(
            "Capa {layer} {} ({touched} obj)",
            if was_visible { "oculta" } else { "visible" }
        ))
    } else {
        None
    }
}

/// Resultado en el idioma activo (B-A130): Es delega a la canónica,
/// resto a `view.layer_toggled_shown/hidden` (`{layer}`/`{touched}`).
pub(crate) fn layer_toggle_result_for(
    layer: u32,
    was_visible: bool,
    touched: usize,
    locale: grafito_ui::i18n::Locale,
) -> Option<String> {
    use grafito_ui::i18n::Locale;
    if touched == 0 {
        return None;
    }
    if locale == Locale::Es {
        return layer_toggle_result(layer, was_visible, touched);
    }
    let key = if was_visible {
        "layer_toggled_hidden"
    } else {
        "layer_toggled_shown"
    };
    Some(
        grafito_ui::i18n::view_msg(key, locale)
            .replacen("{layer}", &layer.to_string(), 1)
            .replacen("{touched}", &touched.to_string(), 1),
    )
}

/// Línea de selección de la sección Capas. Verbatim de `draw_view_panel`
/// en `panels.rs` (slice 26 F14.4.4, sin egui).
pub(crate) fn layer_selection_text(label: &str, layer: u32) -> String {
    format!("Selección: {} (capa {layer})", layer_display_name(label))
}

/// Línea en el idioma activo (B-A130): Es delega a la canónica, resto al
/// template `view.layer_selection` (`{label}`/`{layer}`; el display del
/// nombre sale de `layer_display_name_for`).
pub(crate) fn layer_selection_text_for(
    label: &str,
    layer: u32,
    locale: grafito_ui::i18n::Locale,
) -> String {
    use grafito_ui::i18n::Locale;
    if locale == Locale::Es {
        return layer_selection_text(label, layer);
    }
    let shown = layer_display_name_for(label, locale);
    grafito_ui::i18n::view_msg("layer_selection", locale)
        .replacen("{label}", shown, 1)
        .replacen("{layer}", &layer.to_string(), 1)
}

/// Confirmación de asignar a capa (`'C' → capa 2`). Verbatim de
/// `draw_view_panel` en `panels.rs` (slice 26 F14.4.4, sin egui).
pub(crate) fn layer_assign_result(label: &str, layer: u32) -> String {
    format!("'{}' → capa {layer}", layer_display_name(label))
}

/// Confirmación en el idioma activo (B-A130): Es delega a la canónica,
/// resto al template `view.layer_assign_ok` (`{label}`/`{layer}`).
pub(crate) fn layer_assign_result_for(
    label: &str,
    layer: u32,
    locale: grafito_ui::i18n::Locale,
) -> String {
    use grafito_ui::i18n::Locale;
    if locale == Locale::Es {
        return layer_assign_result(label, layer);
    }
    let shown = layer_display_name_for(label, locale);
    grafito_ui::i18n::view_msg("layer_assign_ok", locale)
        .replacen("{label}", shown, 1)
        .replacen("{layer}", &layer.to_string(), 1)
}

/// Error honesto al asignar a capa. Verbatim de `draw_view_panel` en
/// `panels.rs` (slice 26 F14.4.4, sin egui).
pub(crate) fn layer_assign_error(error: &str) -> String {
    format!("No se pudo asignar: {error}")
}

/// Error en el idioma activo (B-A130): Es delega a la canónica, resto al
/// template `view.layer_assign_err` (`{error}`).
pub(crate) fn layer_assign_error_for(error: &str, locale: grafito_ui::i18n::Locale) -> String {
    use grafito_ui::i18n::Locale;
    if locale == Locale::Es {
        return layer_assign_error(error);
    }
    grafito_ui::i18n::view_msg("layer_assign_err", locale).replacen("{error}", error, 1)
}

/// Título de la sección General de `draw_view_panel`. Verbatim de
/// `panels.rs` (slice 53 F14.4.4, sin egui).
pub(crate) const VIEW_GENERAL_TITLE: &str = "General";

/// Etiqueta del toggle de cuadrícula. Verbatim de `draw_view_panel` en
/// `panels.rs` (slice 53 F14.4.4, sin egui).
pub(crate) const VIEW_SHOW_GRID_LABEL: &str = "Mostrar cuadrícula";

/// Etiqueta del toggle de modo oscuro. Verbatim de `draw_view_panel` en
/// `panels.rs` (slice 53 F14.4.4, sin egui).
pub(crate) const VIEW_DARK_MODE_LABEL: &str = "Modo oscuro";

/// Etiqueta del toggle de ajuste a cuadrícula. Verbatim de
/// `draw_view_panel` en `panels.rs` (slice 53 F14.4.4, sin egui).
pub(crate) const VIEW_SNAP_LABEL: &str = "Ajustar a cuadrícula";

/// Título de la sección Ejes de `draw_view_panel`. Verbatim de `panels.rs`
/// (slice 53 F14.4.4, sin egui).
pub(crate) const VIEW_AXES_TITLE: &str = "Ejes";

/// Etiqueta del toggle de escala log en X. Verbatim de `draw_view_panel`
/// en `panels.rs` (slice 53 F14.4.4, sin egui).
pub(crate) const VIEW_X_LOG_LABEL: &str = "Eje X logarítmico";

/// Etiqueta del toggle de escala log en Y. Verbatim de `draw_view_panel`
/// en `panels.rs` (slice 53 F14.4.4, sin egui).
pub(crate) const VIEW_Y_LOG_LABEL: &str = "Eje Y logarítmico";

/// Etiqueta del plano numerado (ticks + etiquetas + origen). Verbatim de
/// `draw_view_panel` en `panels.rs` (slice 53 F14.4.4, sin egui).
pub(crate) const VIEW_NUMBER_PLANE_LABEL: &str = "Plano numerado (ticks + etiquetas + origen)";

/// Hint del plano numerado. Verbatim de `draw_view_panel` en `panels.rs`
/// (slice 53 F14.4.4, sin egui).
pub(crate) const VIEW_NUMBER_PLANE_HINT: &str =
    "Muestra los números de los ejes con pasos lindos 1/2/5×10^n y skip anti-solape";

/// Chrome del panel Vista General+Ejes por sufijo (B-A128, patrón
/// `inspector_chrome_for`): Es delega a la const canónica de arriba, resto
/// a `view_msg`. Sufijo desconocido → el sufijo.
/// B-A129: +14 sufijos (Vista 3D + Capas estático + AltaPrecisión,
/// consts `:4025-4026,4053-4064`).
/// B-A130: +3 estáticas (hints de Capas + sin-etiqueta, `:862-878`).
/// B-A131: +5 estáticas (parámetro vivo, `:4153-4158`).
pub(crate) fn view_chrome_for(suffix: &'static str, locale: Locale) -> &'static str {
    use grafito_ui::i18n::view_msg;
    if locale == Locale::Es {
        match suffix {
            "general_title" => VIEW_GENERAL_TITLE,
            "show_grid" => VIEW_SHOW_GRID_LABEL,
            "dark_mode" => VIEW_DARK_MODE_LABEL,
            "snap" => VIEW_SNAP_LABEL,
            "axes_title" => VIEW_AXES_TITLE,
            "x_log" => VIEW_X_LOG_LABEL,
            "y_log" => VIEW_Y_LOG_LABEL,
            "number_plane" => VIEW_NUMBER_PLANE_LABEL,
            "number_plane_hint" => VIEW_NUMBER_PLANE_HINT,
            "section_3d" => VIEW_3D_SECTION_TITLE,
            "ortho_hover" => VIEW_3D_ORTHO_HOVER,
            "layers_title" => VIEW_LAYERS_TITLE,
            "layer_show" => LAYER_TOGGLE_SHOW,
            "layer_hide" => LAYER_TOGGLE_HIDE,
            "assign_layer" => VIEW_ASSIGN_LAYER_LABEL,
            "assign" => VIEW_ASSIGN_LABEL,
            "highlight" => VIEW_HIGHLIGHT_LABEL,
            "indicate" => VIEW_INDICATE_LABEL,
            "indicate_hover" => VIEW_INDICATE_HOVER,
            "indicating" => VIEW_INDICATING,
            "hp_title" => VIEW_HIGH_PRECISION_TITLE,
            "hp_label" => VIEW_HIGH_PRECISION_LABEL,
            "hp_hint" => VIEW_HIGH_PRECISION_HINT,
            "layer_empty" => LAYER_EMPTY_HINT,
            "layer_no_selection" => LAYER_NO_SELECTION,
            "layer_untagged" => "<sin etiqueta>",
            "live_section" => LIVE_PARAM_SECTION_TITLE,
            "live_missing" => LIVE_PARAM_MISSING_GUIDE,
            "live_create" => LIVE_PARAM_CREATE_LABEL,
            "live_create_hover" => LIVE_PARAM_CREATE_HOVER,
            "live_created" => LIVE_PARAM_CREATED,
            _ => suffix,
        }
    } else {
        view_msg(suffix, locale)
    }
}

/// Chrome de la sección Exportar archivo + resto (B-A169 + B-A170). Es delega
/// a las consts canónicas de sección, botones de archivo, tips de texto,
/// título de tablas, tooltip CSV, 3 notifys y reveal (paridad byte-idéntica,
/// patrón B-A98/B-A128); resto va a `export_msg`. Siglas
/// `SVG/PNG/TikZ/PDF/MathML/HTML/CSV` intactas como labels (notación,
/// precedente B-A98/B-A114): solo tips + `Copiar SVG/PNG` + fila.
///
/// B-A173 (cierre saga export): los únicos literales que quedan en el draw
/// (`panels.rs:1440,1580-1581`) son identificadores rfd/extensiones
/// invariantes — `.add_filter("PDF", &["pdf"])`, `.add_filter("CSV",
/// &["csv"])`, `format!("{stem}.csv")` + `EXPORT_PDF_DEFAULT_FILENAME` /
/// `EXPORT_CSV_LABEL` — idénticos en los 6 idiomas por diseño (precedentes
/// "ICS" B-A90 + siglas B-A169 + notación SI B-A162): 0 keys, sin
/// display-localizar. Si se les da key, actualizar los 2 pins B-A173.
pub(crate) fn export_chrome_for(suffix: &'static str, locale: Locale) -> &'static str {
    use grafito_ui::i18n::export_msg;
    if locale == Locale::Es {
        match suffix {
            "section_title" => EXPORT_SECTION_TITLE,
            "section_subtitle" => EXPORT_SECTION_SUBTITLE,
            "file_svg_tip" => EXPORT_FILE_BUTTONS[0].1,
            "file_png_tip" => EXPORT_FILE_BUTTONS[1].1,
            "file_tikz_tip" => EXPORT_FILE_BUTTONS[2].1,
            "file_pdf_tip" => EXPORT_FILE_BUTTONS[3].1,
            "file_copy_svg" => EXPORT_FILE_BUTTONS[4].0,
            "file_copy_svg_tip" => EXPORT_FILE_BUTTONS[4].1,
            "file_copy_png" => EXPORT_FILE_BUTTONS[5].0,
            "file_copy_png_tip" => EXPORT_FILE_BUTTONS[5].1,
            "text_mathml_tip" => EXPORT_TEXT_BUTTONS[0].1,
            "text_tikz_tip" => EXPORT_TEXT_BUTTONS[1].1,
            "text_html_tip" => EXPORT_TEXT_BUTTONS[2].1,
            "tables_title" => EXPORT_TABLES_TITLE,
            "csv_tip" => EXPORT_CSV_TOOLTIP,
            "notify_pdf" => EXPORT_NOTIFY_PDF,
            "notify_png" => EXPORT_NOTIFY_PNG,
            "notify_csv" => EXPORT_NOTIFY_CSV,
            "reveal_label" => EXPORT_REVEAL_LABEL,
            "reveal_tip" => EXPORT_REVEAL_TOOLTIP,
            _ => suffix,
        }
    } else {
        export_msg(suffix, locale)
    }
}

/// Los 5 tiles del «Resumen de rango» (N, Media, Desvío s, Mín, Máx) a
/// partir de un [`RangeSummary`]. El formateo numérico lo inyecta la Piel
/// (`format_statistic` en `panels.rs`): este módulo no conoce egui ni
/// formatos. `sample_sd` ausente (n = 1) se muestra como "—", igual que
/// la celda sin resolver de la grilla editable.
pub(crate) fn sheet_summary_tiles(
    summary: &RangeSummary,
    format: impl Fn(f64) -> String,
) -> [(String, String); 5] {
    let sd = summary
        .sample_sd
        .map(&format)
        .unwrap_or_else(|| "—".to_string());
    [
        ("N".to_string(), format!("{}", summary.count)),
        ("Media".to_string(), format(summary.mean)),
        ("Desvío s".to_string(), sd),
        ("Mín".to_string(), format(summary.min)),
        ("Máx".to_string(), format(summary.max)),
    ]
}

/// Los 5 tiles en el idioma pedido (B-A154). Es delega al canónico
/// [`sheet_summary_tiles`] (paridad byte-idéntica, patrón B-A98/B-A150);
/// `—` intacta como notación.
pub(crate) fn sheet_summary_tiles_for(
    summary: &RangeSummary,
    format: impl Fn(f64) -> String,
    locale: Locale,
) -> [(String, String); 5] {
    use grafito_ui::i18n::Locale as L;
    if locale == L::Es {
        return sheet_summary_tiles(summary, format);
    }
    let sd = summary
        .sample_sd
        .map(&format)
        .unwrap_or_else(|| "—".to_string());
    [
        (
            grafito_ui::i18n::stats_msg("sheet_summary_tile_n", locale).to_string(),
            format!("{}", summary.count),
        ),
        (
            grafito_ui::i18n::stats_msg("sheet_summary_tile_mean", locale).to_string(),
            format(summary.mean),
        ),
        (
            grafito_ui::i18n::stats_msg("sheet_summary_tile_sd", locale).to_string(),
            sd,
        ),
        (
            grafito_ui::i18n::stats_msg("sheet_summary_tile_min", locale).to_string(),
            format(summary.min),
        ),
        (
            grafito_ui::i18n::stats_msg("sheet_summary_tile_max", locale).to_string(),
            format(summary.max),
        ),
    ]
}

/// Destino del "Ir a celda": wrapper puro del parser del cerebro
/// (`parse_cell_reference`) con el error honesto que hoy arma el dibujo.
/// Texto vacío → guía de uso; inválido → `«X» no es una celda válida` con
/// el texto recortado, igual que `draw_sheet_editable_grid` en `panels.rs`.
/// El parser ya recorta internamente: acá solo se arma el mensaje.
pub(crate) fn parse_sheet_goto(text: &str) -> Result<(usize, usize), String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err("Ir a: escribí una celda (p. ej. C3)".to_string());
    }
    spreadsheet_series::parse_cell_reference(text)
        .ok_or_else(|| format!("«{}» no es una celda válida", trimmed))
}

/// Texto del chip de navegación: celda en edición (`C3`) o ventana visible
/// (`C3:H10`). Misma expresión que el dibujo de la grilla editable;
/// extraída para testear sin egui.
pub(crate) fn sheet_chip_text(
    editing: Option<(usize, usize)>,
    view: &SheetViewState,
    cols: usize,
) -> String {
    editing
        .map(|(row, col)| format!("{}{}", sheet_col_label(col), row + 1))
        .unwrap_or_else(|| view.window_label(cols))
}

/// Clase visual de una celda de la hoja en modo lectura (slice 12): sin
/// egui para testear sin UI. `Empty` = fuente vacía (sin texto ni hover);
/// `Value` = valor calculado con `format`; `Error` = "—" con el error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SheetCellKind {
    Empty,
    Value,
    Error,
}

/// Texto + hover base + clase de una celda en lectura (slice 12, verbatim
/// de `draw_sheet_editable_grid`): el error manda ("—" + texto del error);
/// fuente vacía → vacío; valor `Some` → `format(v)` + la fórmula como hover
/// (o vacío si no es fórmula); `None` → "—" sin hover. El fallback del
/// hover (`C3 = valor` cuando no es fórmula) queda en el dibujo.
pub(crate) fn sheet_cell_display(
    computed: Option<f64>,
    source: &str,
    error: Option<&str>,
    format: impl Fn(f64) -> String,
) -> (String, String, SheetCellKind) {
    if let Some(message) = error {
        return ("—".to_string(), message.to_string(), SheetCellKind::Error);
    }
    if source.trim().is_empty() {
        return (String::new(), String::new(), SheetCellKind::Empty);
    }
    match computed {
        Some(value) => {
            let hover = if source.trim_start().starts_with('=') {
                source.to_string()
            } else {
                String::new()
            };
            (format(value), hover, SheetCellKind::Value)
        }
        None => ("—".to_string(), String::new(), SheetCellKind::Error),
    }
}

/// Entrada 1D de Prob./Datos: coma o salto de línea, finitos estrictos.
/// Vacía → `Ok(vec![])` (el panel muestra el vacío honesto, no se inventa).
pub(crate) fn parse_statistics_input(input: &str) -> Result<Vec<f64>, String> {
    let input = input.trim();
    if input.is_empty() {
        return Ok(Vec::new());
    }

    input
        .split([',', '\n'])
        .enumerate()
        .map(|(index, token)| {
            let token = token.trim();
            if token.is_empty() {
                return Err(format!("Dato {}: falta un valor", index + 1));
            }
            let value = token
                .parse::<f64>()
                .map_err(|_| format!("Dato {}: '{token}' no es un número válido", index + 1))?;
            if !value.is_finite() {
                return Err(format!("Dato {}: el valor debe ser finito", index + 1));
            }
            Ok(value)
        })
        .collect()
}

/// Muestra honesta de datos para el asistente: primeros 24 valores con
/// `:.6`, más `…y N más` si hay más. Nunca inventa ni trunca en silencio.
pub(crate) fn statistics_data_preview(data: &[f64]) -> String {
    const PREVIEW_N: usize = 24;
    let shown: Vec<String> = data
        .iter()
        .take(PREVIEW_N)
        .map(|v| format!("{v:.6}"))
        .collect();
    if data.len() > PREVIEW_N {
        format!(
            "{} …y {} más (N={})",
            shown.join(", "),
            data.len() - PREVIEW_N,
            data.len()
        )
    } else {
        shown.join(", ")
    }
}

/// Prompt para ordenar/analizar con el asistente: lleva N + preview +
/// comandos disponibles. Puro y testeable; el caller lo pone en
/// `assistant.problem` y abre el workspace.
pub(crate) fn statistics_assistant_prompt(data: &[f64]) -> String {
    if data.is_empty() {
        return "Tengo datos para ordenar pero la lista está vacía. \
            ¿Cómo los pego (uno por línea o comas) en Prob. → Tus datos, o importo un CSV en Datos?"
            .to_string();
    }
    format!(
        "Ordená y analizá estos {} datos: [{}]. \
        Pasos: 1) ordenalos con Sort, 2) resumen (Media, Mediana, Desvío, Q1/Q3/IQR), \
        3) sugerí gráfico (Histogram/BoxPlot) y 4) proponé inferencia (TTest/ZTest/ChiSqTest/ANOVA/Correlation/LinearRegression) \
        con comandos listos para pegar en la entrada.",
        data.len(),
        statistics_data_preview(data)
    )
}

/// Prompt para ordenar/analizar tablas 2D de Datos con el asistente:
/// resume hasta 4 tablas (etiqueta + N + preview x/y de 12 valores) en un
/// prompt accionable. Si no hay tablas, el prompt explica cómo importar.
/// Movido verbatim de `open_data_tables_in_assistant` en `panels.rs`
/// (slice 35): puro sobre el documento, sin egui ni I/O; el caller pone el
/// texto en `assistant.problem` y abre el workspace.
pub(crate) fn data_tables_assistant_prompt(document: &Document) -> String {
    let mut parts: Vec<String> = Vec::new();
    for (_, object) in document.objects_iter_sorted().take(64) {
        if parts.len() >= 4 {
            break;
        }
        if let GeoObject::DataTable(table) = object {
            let n = table.xs.len().min(table.ys.len());
            if n == 0 {
                continue;
            }
            let xs: Vec<String> = table
                .xs
                .iter()
                .take(12)
                .map(|v| format!("{v:.4}"))
                .collect();
            let ys: Vec<String> = table
                .ys
                .iter()
                .take(12)
                .map(|v| format!("{v:.4}"))
                .collect();
            let more = if n > 12 {
                format!(" …+{} más", n - 12)
            } else {
                String::new()
            };
            let label = if table.label.is_empty() {
                "<sin etiqueta>"
            } else {
                &table.label
            };
            parts.push(format!(
                "«{label}» N={n} x=[{}{more}] y=[{}{more}]",
                xs.join(", "),
                ys.join(", "),
                more = more
            ));
        }
    }
    if parts.is_empty() {
        "Quiero ordenar muchos datos pero no hay tablas. \
        ¿Importo un CSV/TSV de dos columnas en Datos → Importar, o pego listas con DataTable[{1,2,3}, {2,4,6}]? \
        Después ordená con Sort, resumí y proponé Histogram/ScatterPlot/LinearRegression con comandos listos."
            .to_string()
    } else {
        format!(
            "Ordená y analizá estas tablas: {}. \
            Pasos por tabla: 1) Sort si hace falta, 2) resumen y correlación, \
            3) gráfico (ScatterPlot/Histogram/BoxPlot/BarChart/PieChart) y 4) ajuste (FitLinear/FitPoly/FitExp/FitLog/FitPow/FitSin) \
            o inferencia (TTest/ANOVA) con comandos listos para pegar.",
            parts.join(" · ")
        )
    }
}

/// Lista inline honesta para comandos (hasta 200 valores; más → `None` para
/// no reventar la entrada y derivar al asistente/hoja).
pub(crate) fn statistics_inline_list(data: &[f64]) -> Option<String> {
    const MAX_INLINE: usize = 200;
    if data.is_empty() || data.len() > MAX_INLINE {
        return None;
    }
    Some(
        data.iter()
            .map(|v| format!("{v}"))
            .collect::<Vec<_>>()
            .join(", "),
    )
}

pub(crate) fn statistics_histogram_command(data: &[f64]) -> Option<String> {
    statistics_inline_list(data).map(|list| format!("Histogram[{{{list}}}, 10]"))
}

pub(crate) fn statistics_boxplot_command(data: &[f64]) -> Option<String> {
    statistics_inline_list(data).map(|list| format!("BoxPlot[{{{list}}}]"))
}

pub(crate) fn statistics_mean_command(data: &[f64]) -> Option<String> {
    statistics_inline_list(data).map(|list| format!("Mean[{{{list}}}]"))
}

/// Chips puros de inferencia del panel de estadística (slice 37 F14.4.4):
/// mismas 6 tuplas (rótulo, comando, tooltip) que el closure inline de
/// `draw_statistics_panel` en `panels.rs`, sin egui.
pub(crate) fn statistics_quick_chips(list: &str) -> [(&'static str, String, &'static str); 6] {
    [
        (
            "Mediana",
            format!("Median[{{{list}}}]"),
            "Mediana de tus datos",
        ),
        ("Desvío", format!("StdDev[{{{list}}}]"), "Desvío estándar"),
        (
            "Frec.",
            format!("FrequencyTable[{{{list}}}]"),
            "Tabla de frecuencias",
        ),
        (
            "Tallo",
            format!("StemPlot[{{{list}}}]"),
            "Diagrama tallo-hoja",
        ),
        (
            "TTest μ=0",
            format!("TTest[{{{list}}}, 0]"),
            "Prueba t contra μ=0",
        ),
        (
            "ZTest",
            format!("ZTest[{{{list}}}, 0, 1]"),
            "Prueba z (σ=1 conocido)",
        ),
    ]
}

/// Chips de inferencia en el idioma pedido (B-A163, display-localizado).
/// Es delega a los literales canónicos de [`statistics_quick_chips`]
/// (paridad byte-idéntica, precedente B-A98/B-A150); resto va por
/// `stats.quick_chip_*/quick_tip_*`. Los comandos
/// (`Median/StdDev/FrequencyTable/StemPlot/TTest/ZTest`) son invariantes
/// (precedente B-A98/B-A148: identificadores de comando, no chrome).
pub(crate) fn statistics_quick_chips_for(
    list: &str,
    locale: Locale,
) -> [(String, String, String); 6] {
    use grafito_ui::i18n::Locale as L;
    if locale == L::Es {
        let canonical = statistics_quick_chips(list);
        canonical.map(|(label, cmd, tip)| (label.to_string(), cmd, tip.to_string()))
    } else {
        let label = |suffix: &'static str| -> String {
            grafito_ui::i18n::stats_msg(suffix, locale).to_string()
        };
        [
            (
                label("quick_chip_median"),
                format!("Median[{{{list}}}]"),
                label("quick_tip_median"),
            ),
            (
                label("quick_chip_stddev"),
                format!("StdDev[{{{list}}}]"),
                label("quick_tip_stddev"),
            ),
            (
                label("quick_chip_freq"),
                format!("FrequencyTable[{{{list}}}]"),
                label("quick_tip_freq"),
            ),
            (
                label("quick_chip_stem"),
                format!("StemPlot[{{{list}}}]"),
                label("quick_tip_stem"),
            ),
            (
                label("quick_chip_ttest"),
                format!("TTest[{{{list}}}, 0]"),
                label("quick_tip_ttest"),
            ),
            (
                label("quick_chip_ztest"),
                format!("ZTest[{{{list}}}, 0, 1]"),
                label("quick_tip_ztest"),
            ),
        ]
    }
}

/// Texto "listo en la entrada" de los chips (misma cadena que el dibujo).
pub(crate) fn statistics_quick_ready_text(label: &str) -> String {
    format!("{label} listo en la entrada (Enter para evaluar).")
}

/// Textos fijos del panel de estadística (mismas cadenas que el dibujo).
pub(crate) const STATISTICS_HISTOGRAM_READY: &str =
    "Histograma listo en la entrada (Enter para crear).";
pub(crate) const STATISTICS_BOXPLOT_READY: &str = "Diagrama de caja listo en la entrada.";
pub(crate) const STATISTICS_MEAN_READY: &str = "Media lista en la entrada.";
pub(crate) const STATISTICS_INLINE_TOO_MANY: &str =
    "Demasiados datos para la entrada (máx. 200 inline): usá Ordenar con asistente.";
pub(crate) const STATISTICS_NO_INLINE_RESULT: &str =
    "Sin lista inline (vacío o +200 valores): pegá menos datos o usá el asistente.";
pub(crate) const STATISTICS_NO_INLINE_HINT: &str =
    "Sin lista inline: pegá menos datos o usá el asistente.";

/// Toasts quick-result en el idioma pedido (B-A153, sin templates).
/// Es delega a las consts canónicas; resto va por `stats_msg`
/// (precedente B-A148 `regression_chrome_for`).
pub(crate) fn statistics_histogram_ready_for(locale: Locale) -> &'static str {
    use grafito_ui::i18n::Locale as L;
    if locale == L::Es {
        STATISTICS_HISTOGRAM_READY
    } else {
        grafito_ui::i18n::stats_msg("quick_histogram_ready", locale)
    }
}

/// Ver [`statistics_histogram_ready_for`].
pub(crate) fn statistics_boxplot_ready_for(locale: Locale) -> &'static str {
    use grafito_ui::i18n::Locale as L;
    if locale == L::Es {
        STATISTICS_BOXPLOT_READY
    } else {
        grafito_ui::i18n::stats_msg("quick_boxplot_ready", locale)
    }
}

/// Ver [`statistics_histogram_ready_for`].
pub(crate) fn statistics_mean_ready_for(locale: Locale) -> &'static str {
    use grafito_ui::i18n::Locale as L;
    if locale == L::Es {
        STATISTICS_MEAN_READY
    } else {
        grafito_ui::i18n::stats_msg("quick_mean_ready", locale)
    }
}

/// Ver [`statistics_histogram_ready_for`].
pub(crate) fn statistics_inline_too_many_for(locale: Locale) -> &'static str {
    use grafito_ui::i18n::Locale as L;
    if locale == L::Es {
        STATISTICS_INLINE_TOO_MANY
    } else {
        grafito_ui::i18n::stats_msg("quick_too_many", locale)
    }
}

/// Ver [`statistics_histogram_ready_for`].
pub(crate) fn statistics_no_inline_result_for(locale: Locale) -> &'static str {
    use grafito_ui::i18n::Locale as L;
    if locale == L::Es {
        STATISTICS_NO_INLINE_RESULT
    } else {
        grafito_ui::i18n::stats_msg("quick_no_inline_result", locale)
    }
}

/// Ver [`statistics_histogram_ready_for`].
pub(crate) fn statistics_no_inline_hint_for(locale: Locale) -> &'static str {
    use grafito_ui::i18n::Locale as L;
    if locale == L::Es {
        STATISTICS_NO_INLINE_HINT
    } else {
        grafito_ui::i18n::stats_msg("quick_no_inline_hint", locale)
    }
}

/// Texto "listo en la entrada" en el idioma pedido (B-A153, template
/// `{label}`). Es delega al canónico [`statistics_quick_ready_text`];
/// resto interpola `stats.quick_ready` (precedente B-A150).
pub(crate) fn statistics_quick_ready_text_for(label: &str, locale: Locale) -> String {
    use grafito_ui::i18n::Locale as L;
    if locale == L::Es {
        statistics_quick_ready_text(label)
    } else {
        grafito_ui::i18n::stats_msg("quick_ready", locale).replace("{label}", label)
    }
}

/// Resumen 1D del panel de estadística (suma opcional si desborda, media,
/// mediana, varianza poblacional, desvío, mín/máx, rango, Q1/Q3/IQR).
/// Puro sobre el slice; `Err` honesto si hay vacíos, no-finitos o el
/// verdadero valor no es representable en f64.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StatisticsSummary {
    pub sum: Option<f64>,
    pub mean: f64,
    pub median: f64,
    pub variance: f64,
    pub standard_deviation: f64,
    pub minimum: f64,
    pub maximum: f64,
    pub range: f64,
    pub q1: f64,
    pub q3: f64,
    pub iqr: f64,
}

pub(crate) fn stable_interpolate(a: f64, b: f64, fraction: f64) -> f64 {
    if fraction <= 0.0 || a == b {
        return a;
    }
    if fraction >= 1.0 {
        return b;
    }

    let delta = b - a;
    if delta.is_finite() {
        fraction.mul_add(delta, a)
    } else {
        // Opposite-sign extremes need the weighted form to avoid overflow.
        (1.0 - fraction).mul_add(a, fraction * b)
    }
}

pub(crate) fn statistics_summary(data: &[f64]) -> Result<StatisticsSummary, String> {
    if data.is_empty() {
        return Err("Estadística: se requiere al menos un dato".to_string());
    }
    if data.iter().any(|value| !value.is_finite()) {
        return Err("Estadística: todos los datos deben ser finitos".to_string());
    }

    let count = data.len() as f64;
    let scale = data.iter().map(|value| value.abs()).fold(0.0, f64::max);
    let (sum, mean, variance, standard_deviation) = if scale == 0.0 {
        (Some(0.0), 0.0, 0.0, 0.0)
    } else {
        let mut normalized_sum = 0.0;
        let mut compensation = 0.0;
        let mut running_mean = 0.0;
        let mut m2 = 0.0;

        for (index, value) in data.iter().enumerate() {
            let normalized = *value / scale;

            let corrected = normalized - compensation;
            let next_sum = normalized_sum + corrected;
            compensation = (next_sum - normalized_sum) - corrected;
            normalized_sum = next_sum;

            let sample_count = (index + 1) as f64;
            let delta = normalized - running_mean;
            running_mean += delta / sample_count;
            let delta_after = normalized - running_mean;
            m2 += delta * delta_after;
        }

        let mean = (normalized_sum / count) * scale;
        if !mean.is_finite() {
            return Err("Estadística: la media no es representable en f64".to_string());
        }

        let normalized_variance = (m2 / count).clamp(0.0, 1.0);
        let standard_deviation = normalized_variance.sqrt() * scale;
        if !standard_deviation.is_finite() {
            return Err("Estadística: el desvío no es representable en f64".to_string());
        }
        let variance = standard_deviation * standard_deviation;
        if normalized_variance > 0.0 && (!variance.is_finite() || variance == 0.0) {
            return Err(
                "Estadística: la varianza verdadera no es representable en f64".to_string(),
            );
        }

        let unscaled_sum = normalized_sum * scale;
        (
            unscaled_sum.is_finite().then_some(unscaled_sum),
            mean,
            variance,
            standard_deviation,
        )
    };

    let mut sorted = data.to_vec();
    sorted.sort_by(f64::total_cmp);
    let minimum = sorted[0];
    let maximum = sorted[sorted.len() - 1];
    let range = maximum - minimum;
    if !range.is_finite() {
        return Err("Estadística: el rango verdadero no es representable en f64".to_string());
    }

    let quantile = |probability: f64| -> Result<f64, String> {
        let position = probability * (sorted.len() as f64 - 1.0);
        let lower = position.floor() as usize;
        let upper = (lower + 1).min(sorted.len() - 1);
        let value = stable_interpolate(sorted[lower], sorted[upper], position - lower as f64);
        value
            .is_finite()
            .then_some(value)
            .ok_or_else(|| "Estadística: un cuantil no es representable en f64".to_string())
    };

    let median = quantile(0.5)?;
    let q1 = quantile(0.25)?;
    let q3 = quantile(0.75)?;
    let iqr = q3 - q1;
    if !iqr.is_finite() {
        return Err("Estadística: el IQR verdadero no es representable en f64".to_string());
    }

    Ok(StatisticsSummary {
        sum,
        mean,
        median,
        variance,
        standard_deviation,
        minimum,
        maximum,
        range,
        q1,
        q3,
        iqr,
    })
}

/// Formato compacto de estadísticos: 3 decimales, científica fuera de
/// `[1e-4, 1e6)`. Lo usan la tarjeta de resumen de la hoja, el panel de
/// estadística y las etiquetas de Prob.
pub(crate) fn format_statistic(value: f64) -> String {
    let magnitude = value.abs();
    if magnitude >= 1.0e6 || (magnitude > 0.0 && magnitude < 1.0e-4) {
        format!("{value:.3e}")
    } else {
        format!("{value:.3}")
    }
}

// --- extraído verbatim de panels.rs:2706-2740 (slice 38 F14.4.4) ---
/// Texto cuando la suma verdadera no entra en f64 (misma cadena que la
/// grilla "Resumen" de `draw_statistics_panel`).
pub(crate) const STATISTICS_SUMMARY_UNREPRESENTABLE: &str = "No representable (desbordamiento)";
/// Cantidad de barras del histograma del panel (mismo literal que el dibujo).
pub(crate) const STATISTICS_SUMMARY_BINS: usize = 10;

/// Filas de la grilla "Resumen" (mismo orden y mismo formato que el dibujo:
/// N/Suma/Media/Mediana/Desvío/Varianza/Mín/Máx/Rango/Q1/Q3/IQR, con la
/// caída "No representable…" cuando la suma no entra en f64).
pub(crate) fn statistics_summary_rows(
    data_len: usize,
    summary: &StatisticsSummary,
) -> Vec<(String, String)> {
    vec![
        ("N".to_string(), format!("{}", data_len)),
        (
            "Suma".to_string(),
            summary
                .sum
                .map(format_statistic)
                .unwrap_or_else(|| STATISTICS_SUMMARY_UNREPRESENTABLE.to_string()),
        ),
        ("Media".to_string(), format_statistic(summary.mean)),
        ("Mediana".to_string(), format_statistic(summary.median)),
        (
            "Desvío".to_string(),
            format_statistic(summary.standard_deviation),
        ),
        ("Varianza".to_string(), format_statistic(summary.variance)),
        ("Mín".to_string(), format_statistic(summary.minimum)),
        ("Máx".to_string(), format_statistic(summary.maximum)),
        ("Rango".to_string(), format_statistic(summary.range)),
        ("Q1".to_string(), format_statistic(summary.q1)),
        ("Q3".to_string(), format_statistic(summary.q3)),
        ("IQR".to_string(), format_statistic(summary.iqr)),
    ]
}

/// Conteo por barra del histograma (misma fórmula que el dibujo: ancho
/// `range.max(1e-9)/bins`, índice `((v-min)/bw).floor()` con clamp a
/// `bins-1`). `bins == 0` es guarda defensiva (el dibujo siempre usa 10).
pub(crate) fn statistics_histogram_counts(
    data: &[f64],
    minimum: f64,
    range: f64,
    bins: usize,
) -> (Vec<u32>, f64) {
    if bins == 0 {
        return (Vec::new(), 0.0);
    }
    let bw = range.max(1e-9) / bins as f64;
    let mut counts = vec![0u32; bins];
    for v in data {
        let idx = (((v - minimum) / bw).floor() as usize).min(bins - 1);
        counts[idx] += 1;
    }
    (counts, bw)
}

// ══════════════════════════════════════════════════════════════
// Slice 7 — probabilidad pura (distribuciones + cotas + slider prompt).
// Movido verbatim de `panels.rs` sin cambio de comportamiento: el dibujo
// (`draw_probability_section`, plots) y los tests (`gc_piel_tests`)
// consumen estos mismos símbolos vía import. `ProbabilityPanelState` +
// `distribution_moments`/`prob_case_command`/`prob_quantile_command`
// quedan en `panels.rs` a propósito (estado egui, no matemática pura).
// ══════════════════════════════════════════════════════════════

// --- extraído verbatim de panels.rs:6065-6070 (slice 7 F14.4.4) ---
/// Cota de `n` en Binomial: CDF suma `n` términos por frame.
pub(crate) const MAX_BINOMIAL_N: u64 = 2_000;
/// Cota de `λ` en Poisson: `exp(-λ)` subfluye igual, pero el loop es O(k).
pub(crate) const MAX_POISSON_LAMBDA: f64 = 1_000_000.0;
/// Cota de `k` en Poisson.
pub(crate) const MAX_POISSON_K: u64 = 100_000;

// --- extraído verbatim de panels.rs:6072-6564 (slice 7 F14.4.4) ---
/// Motor único de probabilidad (F3b, extendido a T/Chi²/F en C2): este
/// panel NO duplica matemática. Delega en `grafito_geometry::statistics`
/// —el mismo motor que usan los comandos `Normal`/`Binomial`/`Poisson`/
/// `InverseNormal`/`InverseT`/`InverseChiSquared`/`InverseF`— y solo agrega
/// el `Err` honesto en español + las cotas de UI. `NaN` del motor ⇒ `Err`.
///
/// B-A160: los `Err` de este motor son canónicos ES a propósito (0 keys,
/// precedente B-A114/B-A146 `commands.rs:856`): el draw ya está localizado
/// (cuantil B-A122, plot B-A157 con `_for`) y los widgets clampan
/// (`DragValue.range` + `Slider` + `clamp_binomial_n/k`), así que el `Err`
/// crudo del cuantil (`panels.rs` rama `Err(error)`) es path defensivo
/// inalcanzable por UI normal (solo tests/API directa). `parse_slider_prompt`
/// (input libre del prompt trig) queda fuera: follow-up B-A161.
fn check_probability_point(value: f64, what: &str) -> Result<f64, String> {
    if !value.is_finite() {
        return Err(format!("{what} debe ser un número finito"));
    }
    Ok(value)
}

fn finish_probability_scalar(value: f64, what: &str) -> Result<f64, String> {
    if !value.is_finite() {
        return Err(format!("{what} no es finito con estos parámetros"));
    }
    Ok(value)
}

/// Densidad Normal(μ, σ) en x. `Err` honesto si σ ≤ 0 o hay no-finitos.
pub(crate) fn normal_pdf(x: f64, mu: f64, sigma: f64) -> Result<f64, String> {
    let (x, mu, sigma) = (
        check_probability_point(x, "x")?,
        check_probability_point(mu, "μ")?,
        check_probability_point(sigma, "σ")?,
    );
    if sigma <= 0.0 {
        return Err("σ debe ser mayor que 0".to_string());
    }
    finish_probability_scalar(
        grafito_geometry::statistics::normal_pdf(x, mu, sigma),
        "La densidad",
    )
}

/// Acumulada Normal(μ, σ) en x: P(X ≤ x).
pub(crate) fn normal_cdf(x: f64, mu: f64, sigma: f64) -> Result<f64, String> {
    let (x, mu, sigma) = (
        check_probability_point(x, "x")?,
        check_probability_point(mu, "μ")?,
        check_probability_point(sigma, "σ")?,
    );
    if sigma <= 0.0 {
        return Err("σ debe ser mayor que 0".to_string());
    }
    finish_probability_scalar(
        grafito_geometry::statistics::normal_cdf(x, mu, sigma),
        "La acumulada",
    )
}

/// Cuantil Normal(μ, σ) en p: el x con P(X ≤ x) = p. Misma inversa que
/// `InverseNormal[p, μ, σ]` (bisección honesta del motor, sin duplicar).
pub(crate) fn normal_quantile_honest(p: f64, mu: f64, sigma: f64) -> Result<f64, String> {
    if !(p.is_finite() && 0.0 < p && p < 1.0) {
        return Err("p debe estar en el intervalo (0, 1)".to_string());
    }
    check_probability_point(mu, "μ")?;
    check_probability_point(sigma, "σ")?;
    if sigma <= 0.0 {
        return Err("σ debe ser mayor que 0".to_string());
    }
    finish_probability_scalar(
        grafito_geometry::statistics::normal_quantile(p, mu, sigma),
        "El cuantil",
    )
}

fn check_binomial_args(k: u64, n: u64, p: f64) -> Result<(u32, f64, u32), String> {
    if !p.is_finite() {
        return Err("p debe ser un número finito".to_string());
    }
    if !(0.0..=1.0).contains(&p) {
        return Err("p debe estar entre 0 y 1".to_string());
    }
    if n > MAX_BINOMIAL_N {
        return Err(format!("n ≤ {} en este panel (cota de UI)", MAX_BINOMIAL_N));
    }
    if k > n {
        return Err("k no puede superar a n".to_string());
    }
    let n32 = u32::try_from(n).map_err(|_| "n fuera de rango".to_string())?;
    let k32 = u32::try_from(k).map_err(|_| "k fuera de rango".to_string())?;
    Ok((n32, p, k32))
}

/// Masa Binomial(n, p) en k: P(X = k). Delega en el motor.
pub(crate) fn binomial_pmf(k: u64, n: u64, p: f64) -> Result<f64, String> {
    let (n32, p, k32) = check_binomial_args(k, n, p)?;
    finish_probability_scalar(
        grafito_geometry::statistics::binomial_pmf(n32, p, k32),
        "La puntual",
    )
}

/// Acumulada Binomial(n, p) en k: P(X ≤ k). Delega en el motor.
pub(crate) fn binomial_cdf(k: u64, n: u64, p: f64) -> Result<f64, String> {
    let (n32, p, k32) = check_binomial_args(k, n, p)?;
    finish_probability_scalar(
        grafito_geometry::statistics::binomial_cdf(n32, p, k32),
        "La acumulada",
    )
}

/// Cuantil Binomial: el menor k con P(X ≤ k) ≥ p. Acumula `pmf` una sola vez
/// (O(n), n ≤ 2000 por cota) en vez de sumar CDFs anidadas.
pub(crate) fn binomial_quantile_honest(p: f64, n: u64, prob: f64) -> Result<u64, String> {
    if !(p.is_finite() && 0.0 < p && p < 1.0) {
        return Err("p debe estar en el intervalo (0, 1)".to_string());
    }
    if n > MAX_BINOMIAL_N {
        return Err(format!("n ≤ {} en este panel (cota de UI)", MAX_BINOMIAL_N));
    }
    if !prob.is_finite() || !(0.0..=1.0).contains(&prob) {
        return Err("p del modelo debe estar entre 0 y 1".to_string());
    }
    let mut acc = 0.0;
    for k in 0..=n {
        acc += binomial_pmf(k, n, prob)?;
        if acc >= p {
            return Ok(k);
        }
    }
    Ok(n)
}

fn check_poisson_args(k: u64, lambda: f64) -> Result<(f64, u32), String> {
    if !lambda.is_finite() {
        return Err("λ debe ser un número finito".to_string());
    }
    if lambda <= 0.0 {
        return Err("λ debe ser mayor que 0".to_string());
    }
    if lambda > MAX_POISSON_LAMBDA {
        return Err(format!(
            "λ ≤ {} en este panel (cota de UI)",
            MAX_POISSON_LAMBDA
        ));
    }
    if k > MAX_POISSON_K {
        return Err(format!("k ≤ {} en este panel (cota de UI)", MAX_POISSON_K));
    }
    let k32 =
        u32::try_from(k.min(u64::from(u32::MAX))).map_err(|_| "k fuera de rango".to_string())?;
    Ok((lambda, k32))
}

/// Masa Poisson(λ) en k: P(X = k). Delega en el motor.
pub(crate) fn poisson_pmf(k: u64, lambda: f64) -> Result<f64, String> {
    let (lambda, k32) = check_poisson_args(k, lambda)?;
    finish_probability_scalar(
        grafito_geometry::statistics::poisson_pmf(lambda, k32),
        "La puntual",
    )
}

/// Acumulada Poisson(λ) en k: P(X ≤ k). Delega en el motor.
pub(crate) fn poisson_cdf(k: u64, lambda: f64) -> Result<f64, String> {
    let (lambda, k32) = check_poisson_args(k, lambda)?;
    finish_probability_scalar(
        grafito_geometry::statistics::poisson_cdf(lambda, k32),
        "La acumulada",
    )
}

/// Cuantil Poisson: el menor k con P(X ≤ k) ≥ p. Itera `pmf` con cota de
/// 10 001 pasos (la del motor discreto); si no alcanza, `Err` honesto.
pub(crate) fn poisson_quantile_honest(p: f64, lambda: f64) -> Result<u64, String> {
    if !(p.is_finite() && 0.0 < p && p < 1.0) {
        return Err("p debe estar en el intervalo (0, 1)".to_string());
    }
    if !lambda.is_finite() || lambda <= 0.0 {
        return Err("λ debe ser mayor que 0".to_string());
    }
    if lambda > MAX_POISSON_LAMBDA {
        return Err(format!(
            "λ ≤ {} en este panel (cota de UI)",
            MAX_POISSON_LAMBDA
        ));
    }
    let mut acc = 0.0;
    let cap = MAX_POISSON_K.min(10_000);
    for k in 0..=cap {
        acc += poisson_pmf(k, lambda)?;
        if acc >= p {
            return Ok(k);
        }
    }
    Err("La cola supera la cota de 10 001 pasos en este panel".to_string())
}

/// Cota de grados de libertad en el panel (frente C2): el motor acepta
/// cualquier gl > 0, pero la curva se dibuja con 61 muestras por frame.
pub(crate) const MAX_PANEL_DF: f64 = 500.0;

fn check_panel_df(value: f64, what: &str) -> Result<f64, String> {
    if !value.is_finite() || value <= 0.0 {
        return Err(format!("{what} debe ser positivo y finito"));
    }
    if value > MAX_PANEL_DF {
        return Err(format!(
            "{what} ≤ {MAX_PANEL_DF} en este panel (cota de UI)"
        ));
    }
    Ok(value)
}

fn check_quantile_p(p: f64) -> Result<f64, String> {
    if p.is_finite() && 0.0 < p && p < 1.0 {
        Ok(p)
    } else {
        Err("p debe estar en el intervalo (0, 1)".to_string())
    }
}

/// Densidad t-Student(gl) en x. Mismo motor que `InverseT`.
pub(crate) fn student_t_pdf(x: f64, df: f64) -> Result<f64, String> {
    let x = check_probability_point(x, "x")?;
    let df = check_panel_df(df, "gl")?;
    finish_probability_scalar(
        grafito_geometry::statistics::student_t_pdf(x, df),
        "La densidad",
    )
}

/// Acumulada t-Student(gl) en x: P(X ≤ x).
pub(crate) fn student_t_cdf(x: f64, df: f64) -> Result<f64, String> {
    let x = check_probability_point(x, "x")?;
    let df = check_panel_df(df, "gl")?;
    finish_probability_scalar(
        grafito_geometry::statistics::student_t_cdf(x, df),
        "La acumulada",
    )
}

/// Cuantil t-Student: el x con P(X ≤ x) = p. Misma inversa que
/// `InverseT[p, gl]` (bisección honesta del motor, sin duplicar).
pub(crate) fn student_t_quantile_honest(p: f64, df: f64) -> Result<f64, String> {
    let p = check_quantile_p(p)?;
    let df = check_panel_df(df, "gl")?;
    finish_probability_scalar(
        grafito_geometry::statistics::student_t_quantile(p, df),
        "El cuantil",
    )
}

/// Densidad χ²(gl) en x. Mismo motor que `InverseChiSquared`.
pub(crate) fn chi_squared_pdf(x: f64, df: f64) -> Result<f64, String> {
    let x = check_probability_point(x, "x")?;
    let df = check_panel_df(df, "gl")?;
    finish_probability_scalar(
        grafito_geometry::statistics::chi_squared_pdf(x, df),
        "La densidad",
    )
}

/// Acumulada χ²(gl) en x: P(X ≤ x).
pub(crate) fn chi_squared_cdf(x: f64, df: f64) -> Result<f64, String> {
    let x = check_probability_point(x, "x")?;
    let df = check_panel_df(df, "gl")?;
    finish_probability_scalar(
        grafito_geometry::statistics::chi_squared_cdf(x, df),
        "La acumulada",
    )
}

/// Cuantil χ²: el x con P(X ≤ x) = p. Misma inversa que
/// `InverseChiSquared[p, gl]`.
pub(crate) fn chi_squared_quantile_honest(p: f64, df: f64) -> Result<f64, String> {
    let p = check_quantile_p(p)?;
    let df = check_panel_df(df, "gl")?;
    finish_probability_scalar(
        grafito_geometry::statistics::chi_squared_quantile(p, df),
        "El cuantil",
    )
}

/// Densidad F(gl1, gl2) en x. Mismo motor que `InverseF`.
pub(crate) fn f_distribution_pdf(x: f64, df1: f64, df2: f64) -> Result<f64, String> {
    let x = check_probability_point(x, "x")?;
    let df1 = check_panel_df(df1, "gl1")?;
    let df2 = check_panel_df(df2, "gl2")?;
    finish_probability_scalar(
        grafito_geometry::statistics::f_distribution_pdf(x, df1, df2),
        "La densidad",
    )
}

/// Acumulada F(gl1, gl2) en x: P(X ≤ x).
pub(crate) fn f_distribution_cdf(x: f64, df1: f64, df2: f64) -> Result<f64, String> {
    let x = check_probability_point(x, "x")?;
    let df1 = check_panel_df(df1, "gl1")?;
    let df2 = check_panel_df(df2, "gl2")?;
    finish_probability_scalar(
        grafito_geometry::statistics::f_distribution_cdf(x, df1, df2),
        "La acumulada",
    )
}

/// Cuantil F: el x con P(X ≤ x) = p. Misma inversa que
/// `InverseF[p, gl1, gl2]`.
pub(crate) fn f_quantile_honest(p: f64, df1: f64, df2: f64) -> Result<f64, String> {
    let p = check_quantile_p(p)?;
    let df1 = check_panel_df(df1, "gl1")?;
    let df2 = check_panel_df(df2, "gl2")?;
    finish_probability_scalar(
        grafito_geometry::statistics::f_quantile(p, df1, df2),
        "El cuantil",
    )
}

/// Cota de `k` en Geométrica: el loop del cuantil es O(k).
pub(crate) const MAX_GEOMETRIC_K: u64 = 100_000;

fn check_geometric_args(k: u64, p: f64) -> Result<(f64, u32), String> {
    if !p.is_finite() {
        return Err("p debe ser un número finito".to_string());
    }
    if !(0.0 < p && p <= 1.0) {
        return Err("p debe estar en el intervalo (0, 1]".to_string());
    }
    if k > MAX_GEOMETRIC_K {
        return Err(format!("k ≤ {MAX_GEOMETRIC_K} en este panel (cota de UI)"));
    }
    let k32 =
        u32::try_from(k.min(u64::from(u32::MAX))).map_err(|_| "k fuera de rango".to_string())?;
    Ok((p, k32))
}

/// Masa Geométrica(p) en k: P(X = k), k = fallos antes del primer éxito.
/// Delega en el motor (`geometric_pmf`), que no valida: el `Err` honesto
/// en español + las cotas viven acá.
pub(crate) fn geometric_pmf(k: u64, p: f64) -> Result<f64, String> {
    let (p, k32) = check_geometric_args(k, p)?;
    finish_probability_scalar(
        grafito_geometry::statistics::geometric_pmf(p, k32),
        "La puntual",
    )
}

/// Acumulada Geométrica(p) en k: P(X ≤ k). Delega en el motor.
pub(crate) fn geometric_cdf(k: u64, p: f64) -> Result<f64, String> {
    let (p, k32) = check_geometric_args(k, p)?;
    finish_probability_scalar(
        grafito_geometry::statistics::geometric_cdf(p, k32),
        "La acumulada",
    )
}

/// Cuantil Geométrica: el menor k con P(X ≤ k) ≥ p. Acumula `pmf` con la
/// misma cota del panel discreto; si no alcanza, `Err` honesto.
pub(crate) fn geometric_quantile_honest(p: f64, prob: f64) -> Result<u64, String> {
    let p = check_quantile_p(p)?;
    if !(prob.is_finite() && 0.0 < prob && prob <= 1.0) {
        return Err("p del modelo debe estar en el intervalo (0, 1]".to_string());
    }
    let cap = MAX_GEOMETRIC_K.min(10_000);
    let mut acc = 0.0;
    for k in 0..=cap {
        acc += geometric_pmf(k, prob)?;
        if acc >= p {
            return Ok(k);
        }
    }
    Err("La cola supera la cota de 10 001 pasos en este panel".to_string())
}

fn check_uniform_args(x: f64, a: f64, b: f64) -> Result<(f64, f64, f64), String> {
    let x = check_probability_point(x, "x")?;
    let a = check_probability_point(a, "a")?;
    let b = check_probability_point(b, "b")?;
    if a >= b {
        return Err("se requiere a < b".to_string());
    }
    Ok((x, a, b))
}

/// Densidad Uniforme(a, b) en x: 1/(b−a) dentro de [a, b], 0 fuera.
/// Delega en el motor; el `Err` honesto (a < b finitos) vive acá.
pub(crate) fn uniform_pdf(x: f64, a: f64, b: f64) -> Result<f64, String> {
    let (x, a, b) = check_uniform_args(x, a, b)?;
    finish_probability_scalar(
        grafito_geometry::statistics::uniform_pdf(x, a, b),
        "La densidad",
    )
}

/// Acumulada Uniforme(a, b) en x: P(X ≤ x). Delega en el motor.
pub(crate) fn uniform_cdf(x: f64, a: f64, b: f64) -> Result<f64, String> {
    let (x, a, b) = check_uniform_args(x, a, b)?;
    finish_probability_scalar(
        grafito_geometry::statistics::uniform_cdf(x, a, b),
        "La acumulada",
    )
}

/// Cuantil Uniforme: forma cerrada a + p·(b−a), sin iterar.
pub(crate) fn uniform_quantile_honest(p: f64, a: f64, b: f64) -> Result<f64, String> {
    let p = check_quantile_p(p)?;
    let a = check_probability_point(a, "a")?;
    let b = check_probability_point(b, "b")?;
    if a >= b {
        return Err("se requiere a < b".to_string());
    }
    finish_probability_scalar(a + p * (b - a), "El cuantil")
}

/// Densidad Exponencial(λ) en x: λ·exp(-λx) para x≥0, 0 fuera.
/// `Err` honesto si λ ≤ 0 o hay no-finitos.
pub(crate) fn exponential_pdf(x: f64, lambda: f64) -> Result<f64, String> {
    let x = check_probability_point(x, "x")?;
    let lambda = check_probability_point(lambda, "λ")?;
    if lambda <= 0.0 {
        return Err("λ debe ser mayor que 0".to_string());
    }
    finish_probability_scalar(
        grafito_geometry::statistics::exponential_pdf(x, lambda),
        "La densidad",
    )
}

/// Acumulada Exponencial(λ) en x: P(X ≤ x). Delega en el motor.
pub(crate) fn exponential_cdf(x: f64, lambda: f64) -> Result<f64, String> {
    let x = check_probability_point(x, "x")?;
    let lambda = check_probability_point(lambda, "λ")?;
    if lambda <= 0.0 {
        return Err("λ debe ser mayor que 0".to_string());
    }
    finish_probability_scalar(
        grafito_geometry::statistics::exponential_cdf(x, lambda),
        "La acumulada",
    )
}

/// Cuantil Exponencial: forma cerrada -ln(1-p)/λ, sin iterar.
pub(crate) fn exponential_quantile_honest(p: f64, lambda: f64) -> Result<f64, String> {
    if !(p.is_finite() && 0.0 < p && p < 1.0) {
        return Err("p debe estar en el intervalo (0, 1)".to_string());
    }
    let lambda = check_probability_point(lambda, "λ")?;
    if lambda <= 0.0 {
        return Err("λ debe ser mayor que 0".to_string());
    }
    finish_probability_scalar(-(1.0 - p).ln() / lambda, "El cuantil")
}

/// Prompt auto de velocidad para sliders Play. Acepta:
/// número («1.5»), palabra («lento/medio/rápido») o «N vueltas en S s».
/// Todo lo demás es `Err` honesto (nunca se inventa una velocidad).
pub(crate) fn parse_slider_prompt(input: &str) -> Result<f64, String> {
    let text = input.trim().to_lowercase();
    if text.is_empty() {
        return Err("Escribí una velocidad: 1.5, «medio» o «2 vueltas en 10 s»".to_string());
    }
    if let Ok(value) = text.replace(',', ".").parse::<f64>() {
        if !value.is_finite() {
            return Err("La velocidad debe ser finita".to_string());
        }
        if value.abs() > 6.0 {
            return Err("La velocidad vive en ±6 rad/s (mové el slider si querés más)".to_string());
        }
        return Ok(value);
    }
    match text.as_str() {
        "lento" | "lenta" | "slow" => return Ok(0.2),
        "medio" | "media" | "normal" => return Ok(0.5),
        "rapido" | "rápido" | "rapida" | "rápida" | "fast" => return Ok(2.0),
        _ => {}
    }
    // «N vueltas en S s»: números con punto o coma decimal.
    let numbers: Vec<f64> = text
        .split(|c: char| !(c.is_ascii_digit() || c == '.' || c == ',' || c == '-'))
        .filter(|token| !token.is_empty())
        .filter_map(|token| token.replace(',', ".").parse::<f64>().ok())
        .filter(|value| value.is_finite() && *value > 0.0)
        .collect();
    if (text.contains("vuelta") || text.contains("lap") || text.contains("giro"))
        && numbers.len() >= 2
    {
        let (turns, seconds) = (numbers[0], numbers[1]);
        if seconds <= 0.0 {
            return Err("Los segundos deben ser mayores que 0".to_string());
        }
        let speed = turns * 2.0 * std::f64::consts::PI / seconds;
        if speed > 6.0 {
            return Err(format!(
                "Eso da {speed:.2} rad/s: más de 6 rad/s no se sigue a ojo (bajá vueltas o subí segundos)"
            ));
        }
        return Ok(speed);
    }
    Err("No entendí: probá 1.5, «medio» o «2 vueltas en 10 s»".to_string())
}

/// Variante localizada de [`parse_slider_prompt`] (B-A161): misma
/// aceptación (número/palabra/vueltas) con los 6 `Err` por
/// `trig.prompt_err_*` en el `locale` pedido. Es delega al canónico por
/// construcción (paridad pineada); resto→i18n con template `{speed}` solo
/// en too_fast (notación `rad/s`/`±6`/números intacta).
pub(crate) fn parse_slider_prompt_for(input: &str, locale: Locale) -> Result<f64, String> {
    use grafito_ui::i18n::Locale as L;
    if locale == L::Es {
        return parse_slider_prompt(input);
    }
    let text = input.trim().to_lowercase();
    if text.is_empty() {
        return Err(trig_i18n_t("trig.prompt_err_empty", locale).to_string());
    }
    if let Ok(value) = text.replace(',', ".").parse::<f64>() {
        if !value.is_finite() {
            return Err(trig_i18n_t("trig.prompt_err_finite", locale).to_string());
        }
        if value.abs() > 6.0 {
            return Err(trig_i18n_t("trig.prompt_err_range", locale).to_string());
        }
        return Ok(value);
    }
    match text.as_str() {
        "lento" | "lenta" | "slow" => return Ok(0.2),
        "medio" | "media" | "normal" => return Ok(0.5),
        "rapido" | "rápido" | "rapida" | "rápida" | "fast" => return Ok(2.0),
        _ => {}
    }
    // «N vueltas en S s»: números con punto o coma decimal.
    let numbers: Vec<f64> = text
        .split(|c: char| !(c.is_ascii_digit() || c == '.' || c == ',' || c == '-'))
        .filter(|token| !token.is_empty())
        .filter_map(|token| token.replace(',', ".").parse::<f64>().ok())
        .filter(|value| value.is_finite() && *value > 0.0)
        .collect();
    if (text.contains("vuelta") || text.contains("lap") || text.contains("giro"))
        && numbers.len() >= 2
    {
        let (turns, seconds) = (numbers[0], numbers[1]);
        if seconds <= 0.0 {
            return Err(trig_i18n_t("trig.prompt_err_seconds", locale).to_string());
        }
        let speed = turns * 2.0 * std::f64::consts::PI / seconds;
        if speed > 6.0 {
            return Err(trig_i18n_t("trig.prompt_err_too_fast", locale)
                .replace("{speed}", &format!("{speed:.2}")));
        }
        return Ok(speed);
    }
    Err(trig_i18n_t("trig.prompt_err_unknown", locale).to_string())
}

// --- extraído verbatim de panels.rs:6694-6699 (slice 7 F14.4.4) ---
/// Resuelve gl para el plot sin `clamp` silencioso: fuera de cota es `Err`
/// honesto (el caller dibuja "fuera de cota" en vez de una curva falsa).
/// W-A: `df = MAX_PANEL_DF + 1` → `Err`, no curva clampada a 500.
/// B-A157: variante localizada (templates `{what}/{max}` via
/// `stats.prob_plot_df_*`; `what` es identificador `gl/gl1/gl2` intacto,
/// precedente B-A155). Es delega al canonico por construccion.
pub(crate) fn plot_df_or_fuera_de_cota_for(
    value: f64,
    what: &str,
    locale: grafito_ui::i18n::Locale,
) -> Result<f64, String> {
    if !value.is_finite() || value <= 0.0 {
        return Err(
            grafito_ui::i18n::t("stats.prob_plot_df_positive", locale).replace("{what}", what)
        );
    }
    if value > MAX_PANEL_DF {
        return Err(grafito_ui::i18n::t("stats.prob_plot_df_capped", locale)
            .replace("{what}", what)
            .replace("{max}", &format!("{}", MAX_PANEL_DF)));
    }
    Ok(value)
}
/// Wrapper Es del plot (solo tests; prod usa `_for` con `plot_locale`).
/// Precedente B-A143/B-A111: sin caller prod va con `#[cfg(test)]`.
#[cfg(test)]
pub(crate) fn plot_df_or_fuera_de_cota(value: f64, what: &str) -> Result<f64, String> {
    plot_df_or_fuera_de_cota_for(value, what, grafito_ui::i18n::Locale::Es)
}

// --- extraído verbatim de panels.rs:7117-7120 (slice 7 F14.4.4) ---
/// ¿Distribución discreta? Rótulo honesto junto a la ayuda del selector.
pub(crate) fn prob_distribution_is_discrete(dist: u8) -> bool {
    matches!(dist, 1 | 2 | 6)
}

// --- extraído verbatim de panels.rs:7205-7220 (slice 7 F14.4.4) ---
/// Colas honestas del caso: `(P(X ≤ x), P(X > x), 2·cola menor)`. Clampa a
/// `[0, 1]` porque `1 − cdf` puede subfluir fuera de rango en colas extremas;
/// `None` si el CDF no es finito: jamás se inventa una probabilidad.
pub(crate) fn prob_tail_values(cdf: f64) -> Option<(f64, f64, f64)> {
    if !cdf.is_finite() {
        return None;
    }
    let cdf = cdf.clamp(0.0, 1.0);
    let right = (1.0 - cdf).clamp(0.0, 1.0);
    let two = (2.0 * cdf.min(right)).clamp(0.0, 1.0);
    Some((cdf, right, two))
}

// --- extraído verbatim de panels.rs:5643 + 6384 (slice 28 F14.4.4) ---
/// n binomial clampado del estado crudo del slider (1..=MAX_BINOMIAL_N).
/// El mismo clamp vivía en el plot (rama 1) y en el cuantil (caso 1):
/// dueño único, sin drift.
pub(crate) fn clamp_binomial_n(n_raw: f64) -> u64 {
    n_raw.round().clamp(1.0, MAX_BINOMIAL_N as f64) as u64
}

// --- extraído verbatim de panels.rs:5644 (slice 28 F14.4.4) ---
/// k binomial clampado del estado crudo del slider (0..=n).
pub(crate) fn clamp_binomial_k(k_raw: f64, n: u64) -> u64 {
    (k_raw.round().max(0.0) as u64).min(n)
}

// --- extraído verbatim de panels.rs:5645-5658 (slice 28 F14.4.4) ---
/// Ventana (lo, hi) del plot binomial: 0..=n si n ≤ 61, si no 61 barras
/// alrededor de la media (±4σ, piso 8) extendida hasta cubrir k_sel.
/// p se clampa a [0, 1] como hacía el dibujo.
pub(crate) fn binomial_plot_window(n: u64, p: f64, k_sel: u64) -> (u64, u64) {
    let mean = n as f64 * p.clamp(0.0, 1.0);
    let sd = (n as f64 * p.clamp(0.0, 1.0) * (1.0 - p.clamp(0.0, 1.0))).sqrt();
    let (mut lo, mut hi) = if n <= 61 {
        (0u64, n)
    } else {
        let span = (4.0 * sd).ceil().max(8.0) as u64;
        let lo = (mean.floor().max(0.0) as u64)
            .saturating_sub(span / 2)
            .min(n);
        let hi = (lo + 60).min(n);
        (hi.saturating_sub(60).min(lo), hi)
    };
    lo = lo.min(k_sel);
    hi = hi.max(k_sel);
    (lo, hi)
}

// --- extraído verbatim de panels.rs:5694-5711 (slice 28 F14.4.4) ---
/// Rótulo izquierdo del eje del plot binomial.
pub(crate) fn binomial_plot_left_label(lo: u64) -> String {
    format!("k={lo}")
}

/// Rótulo derecho del plot binomial (ventana solo si n > 61).
pub(crate) fn binomial_plot_right_label(n: u64, lo: u64, hi: u64) -> String {
    if n > 61 {
        format!("ventana {lo}..={hi} de 0..={n} · ≤k sombreado")
    } else {
        "k ≤ seleccionado sombreado".to_string()
    }
}

// --- extraído verbatim de panels.rs:5612-5614 (slice 28 F14.4.4) ---
/// Fracciones (lo, hi, alfa) de las bandas μ±kσ de la regla empírica
/// (k=3/2/1 con alfa 0.05/0.07/0.10): el dibujo las lleva a píxeles con
/// clamp [0, 1] × ancho del plot.
pub(crate) fn normal_band_fractions(mu: f64, sigma: f64, lo: f64, hi: f64) -> [(f32, f32, f32); 3] {
    [(3.0_f64, 0.05_f32), (2.0, 0.07), (1.0, 0.10)].map(|(k, alpha)| {
        let band_lo = ((mu - k * sigma - lo) / (hi - lo)) as f32;
        let band_hi = ((mu + k * sigma - lo) / (hi - lo)) as f32;
        (band_lo, band_hi, alpha)
    })
}

// --- extraído verbatim de panels.rs:5602-5610 (slice 29 F14.4.4) ---
/// Ventana (lo, hi) del plot Normal: μ±4σ. `None` si σ no es positiva
/// finita o la ventana no es finita (el dibujo distingue los dos mensajes:
/// primer guard "σ debe ser mayor que 0", este `None` → "μ/σ fuera de rango").
pub(crate) fn normal_plot_window(mu: f64, sigma: f64) -> Option<(f64, f64)> {
    if !mu.is_finite() || !sigma.is_finite() || sigma <= 0.0 {
        return None;
    }
    let (lo, hi) = (mu - 4.0 * sigma, mu + 4.0 * sigma);
    if !(lo.is_finite() && hi.is_finite() && hi > lo) {
        return None;
    }
    Some((lo, hi))
}

/// Ventana fija del plot t-Student (±6, literales verbatim de la rama 3).
pub(crate) const T_PLOT_LO: f64 = -6.0;
/// Ventana fija del plot t-Student (±6, literales verbatim de la rama 3).
pub(crate) const T_PLOT_HI: f64 = 6.0;

// --- geometría del plot de probabilidad (S2 ui-polish) ---
// Dueños únicos de los pads/geometría antes literales en
// `draw_probability_plot` (`panels.rs`): altura, reserva del eje,
// inset, línea base, barras y etiquetas. Doctrina Nordic Slate:
// ningún color/espaciado literal nuevo en el draw.
/// Altura del área de curva/barras del plot de probabilidad.
pub(crate) const PROB_PLOT_H: f32 = 84.0;
/// Reserva vertical para eje + etiquetas bajo el plot.
pub(crate) const PROB_PLOT_AXIS_RESERVE: f32 = 16.0;
/// Inset del marco interno del plot (`shrink2`).
pub(crate) const PROB_PLOT_INSET: f32 = 4.0;
/// Ancho de la línea base del eje.
pub(crate) const PROB_PLOT_BASELINE_W: f32 = 1.0;
/// Inset lateral de cada barra discreta.
pub(crate) const PROB_PLOT_BAR_INSET_X: f32 = 1.0;
/// Separación total entre barras vecinas (`bar_w - gap`).
pub(crate) const PROB_PLOT_BAR_GAP: f32 = 4.0;
/// Ancho/alto mínimo de barra visible.
pub(crate) const PROB_PLOT_BAR_MIN: f32 = 1.0;
/// Redondeo de las barras discretas.
pub(crate) const PROB_PLOT_BAR_ROUNDING: f32 = 2.0;
/// Desplazamiento vertical de las etiquetas del eje.
pub(crate) const PROB_PLOT_LABEL_DY: f32 = 4.0;
/// Piso del fundido de entrada: las barras crecen desde esta fracción
/// de su altura hasta el total (`fade` 0→1 por distribución).
pub(crate) const PROB_PLOT_GROW_MIN: f32 = 0.25;

/// Factor de crecimiento del fundido de entrada del plot (`fade` 0→1
/// por distribución): las barras/curvas crecen desde `PROB_PLOT_GROW_MIN`
/// hasta el total; en `fade = 1` es identidad. Pura, sin egui.
/// Dueña única de la fórmula antes duplicada en `paint_discrete_bars` +
/// `draw_probability_plot` (`panels.rs`, S2 ui-polish).
pub(crate) fn prob_plot_grow(fade: f32) -> f32 {
    PROB_PLOT_GROW_MIN + (1.0 - PROB_PLOT_GROW_MIN) * fade
}

/// Alfa de las bandas μ±σ modulado por el fundido: en `fade = 1` deja
/// el alfa intacto (sin salto al terminar la transición). Pura, sin egui.
pub(crate) fn prob_plot_band_alpha(alpha: f32, fade: f32) -> f32 {
    alpha * fade
}

/// Altura de una barra discreta: fracción de masa por altura del plot,
/// escalada por el fundido. Pura, sin egui.
pub(crate) fn prob_plot_bar_h(pmf: f64, ymax: f64, plot_h: f32, fade: f32) -> f32 {
    (pmf / ymax) as f32 * plot_h * prob_plot_grow(fade)
}

/// Texto accesible de una barra discreta del plot (`k` + masa).
/// Template `stats.prob_plot_bar` (`k = {k} · P = {p}`, notación
/// idéntica en los 6 idiomas); `p` con 4 decimales. Pura, sin egui.
/// La usa el hover + `widget_info` de cada barra (S2 ui-polish).
pub(crate) fn prob_plot_bar_text_for(k: u64, pmf: f64, locale: grafito_ui::i18n::Locale) -> String {
    grafito_ui::i18n::t("stats.prob_plot_bar", locale)
        .replace("{k}", &k.to_string())
        .replace("{p}", &format!("{pmf:.4}"))
}

// --- extraído verbatim de panels.rs:5699 + 5832 (slice 29 F14.4.4) ---
/// k discreto seleccionado del estado crudo del slider (ramas Poisson 2 y
/// Geométrica 6 del plot: misma expresión en ambas, dueño único).
pub(crate) fn discrete_k_sel(k_raw: f64) -> u64 {
    k_raw.round().max(0.0) as u64
}

// --- extraído verbatim de panels.rs:5700-5705 (slice 29 F14.4.4) ---
/// Ancho (hi, desde 0) del plot Poisson: media×3+10 con piso 8, tope 60,
/// extendido hasta cubrir k_sel y capado por `MAX_POISSON_K`.
/// `None` si λ no es finita positiva (el dibujo retorna en silencio).
pub(crate) fn poisson_plot_hi(lambda: f64, k_sel: u64) -> Option<u64> {
    if !lambda.is_finite() || lambda <= 0.0 {
        return None;
    }
    let hi_default = (lambda * 3.0 + 10.0).ceil().max(8.0) as u64;
    Some(hi_default.min(60).max(k_sel).min(MAX_POISSON_K))
}

// --- extraído verbatim de panels.rs:5833-5840 (slice 29 F14.4.4) ---
/// Ancho (hi, desde 0) del plot Geométrica: media×3+10 con piso 8, tope 60,
/// extendido hasta cubrir k_sel y capado por `MAX_GEOMETRIC_K`.
/// `None` si p ∉ (0, 1] finita (el dibujo avisa "p debe estar en (0, 1]").
pub(crate) fn geometric_plot_hi(prob: f64, k_sel: u64) -> Option<u64> {
    if !(prob.is_finite() && 0.0 < prob && prob <= 1.0) {
        return None;
    }
    let mean = (1.0 - prob) / prob;
    let hi_default = (mean * 3.0 + 10.0).ceil().max(8.0) as u64;
    Some(hi_default.min(60).max(k_sel).min(MAX_GEOMETRIC_K))
}

/// Rótulo izquierdo de los plots discretos Poisson/Geométrica (verbatim).
pub(crate) fn discrete_plot_left_label() -> String {
    "k=0".to_string()
}

/// Rótulo derecho de los plots discretos Poisson/Geométrica (verbatim en
/// ambas ramas: misma cadena, dueño único).
pub(crate) fn discrete_plot_right_label(hi: u64) -> String {
    format!("0..={hi} · ≤k sombreado")
}

// --- extraído verbatim de panels.rs:5778-5784 (slice 29 F14.4.4) ---
/// Ancho (hi, desde 0) del plot χ²: gl×4+8 extendido hasta cubrir x con
/// piso 8. `None` si x no es finita o hi no lo es (df ya validado por
/// `plot_df_or_fuera_de_cota` en el dibujo; si df no fuera finita, hi
/// tampoco lo es y este `None` lo cubre).
pub(crate) fn chi_squared_plot_hi(df: f64, x: f64) -> Option<f64> {
    if !x.is_finite() {
        return None;
    }
    let hi = (df * 4.0 + 8.0).max(x).max(8.0);
    if !hi.is_finite() {
        None
    } else {
        Some(hi)
    }
}

// --- extraído verbatim de panels.rs:5808-5814 (slice 29 F14.4.4) ---
/// Ancho (hi, desde 0) del plot F: 6 extendido hasta cubrir x con piso 1.
/// `None` si x no es finita o hi no lo es (gl1/gl2 ya validados por
/// `plot_df_or_fuera_de_cota` en el dibujo).
pub(crate) fn f_plot_hi(x: f64) -> Option<f64> {
    if !x.is_finite() {
        return None;
    }
    let hi = 6.0f64.max(x).max(1.0);
    if !hi.is_finite() {
        None
    } else {
        Some(hi)
    }
}

/// Cuantiles rápidos del panel (α clásicos de tabla). Todos dentro de
/// `(0.001, 0.999)`, el rango válido del slider de `p_inv`.
pub(crate) const PROB_QUANTILE_ALPHAS: [f64; 6] = [0.01, 0.05, 0.10, 0.90, 0.95, 0.99];

// --- extraído verbatim de panels.rs:6073-6113 + 6591-6698 (slice 8 F14.4.4) ---
/// Estado efímero del panel de probabilidad (vive en `ctx.data`, sin I/O).
/// `dist`: 0 Normal, 1 Binomial, 2 Poisson, 3 t-Student, 4 χ², 5 F (C2),
/// 6 Geométrica, 7 Uniforme (W-E: el motor ya las trae), 8 Exponencial (P4).
#[derive(Debug, Clone)]
pub(crate) struct ProbabilityPanelState {
    pub(crate) dist: u8,
    pub(crate) mu: f64,
    pub(crate) sigma: f64,
    pub(crate) x: f64,
    pub(crate) n: f64,
    pub(crate) p: f64,
    pub(crate) k: f64,
    pub(crate) lambda: f64,
    pub(crate) p_inv: f64,
    pub(crate) df: f64,
    pub(crate) df1: f64,
    pub(crate) df2: f64,
    pub(crate) ua: f64,
    pub(crate) ub: f64,
}

impl Default for ProbabilityPanelState {
    fn default() -> Self {
        Self {
            dist: 0,
            mu: 0.0,
            sigma: 1.0,
            x: 0.0,
            n: 10.0,
            p: 0.5,
            k: 5.0,
            lambda: 3.0,
            p_inv: 0.95,
            df: 10.0,
            df1: 5.0,
            df2: 10.0,
            ua: 0.0,
            ub: 1.0,
        }
    }
}

/// Distribuciones de la calculadora de probabilidad: (id, nombre, ayuda).
/// El ComboBox de la card "Distribución" las muestra en este orden.
pub(crate) const PROB_DISTS: [(u8, &str, &str); 9] = [
    (0u8, "Normal", "Gaussiana μ, σ: densidad y acumulada"),
    (1u8, "Binomial", "n ensayos, p éxito: P(X = k) y P(X ≤ k)"),
    (2u8, "Poisson", "Tasa λ: P(X = k) y P(X ≤ k)"),
    (
        3u8,
        "t-Student",
        "gl grados de libertad: densidad y acumulada",
    ),
    (4u8, "χ²", "gl grados de libertad: densidad y acumulada"),
    (5u8, "F", "gl1, gl2: densidad y acumulada"),
    (
        6u8,
        "Geométrica",
        "p éxito: fallos antes del primer éxito, P(X = k) y P(X ≤ k)",
    ),
    (7u8, "Uniforme", "a, b: densidad 1/(b−a) y acumulada"),
    (
        8u8,
        "Exponencial",
        "λ tasa: densidad λ·exp(-λx) y acumulada",
    ),
];

/// Momentos analíticos (media, varianza) de la distribución seleccionada.
/// `None` = no definido con esos parámetros (p. ej. t con gl ≤ 2, F con
/// gl2 ≤ 4, Uniforme con a ≥ b): el panel muestra `—` sin inventar.
pub(crate) fn distribution_moments(state: &ProbabilityPanelState) -> (Option<f64>, Option<f64>) {
    match state.dist {
        0 => (Some(state.mu), Some(state.sigma * state.sigma)),
        1 => {
            let n = state.n.round().max(0.0);
            let p = state.p;
            (Some(n * p), Some(n * p * (1.0 - p)))
        }
        2 => (Some(state.lambda), Some(state.lambda)),
        3 => {
            let df = state.df;
            let mean = (df > 1.0).then_some(0.0);
            let variance = (df > 2.0).then_some(df / (df - 2.0));
            (mean, variance)
        }
        4 => (Some(state.df), Some(2.0 * state.df)),
        5 => {
            let (df1, df2) = (state.df1, state.df2);
            let mean = (df2 > 2.0).then_some(df2 / (df2 - 2.0));
            let variance = (df2 > 4.0).then_some(
                2.0 * df2 * df2 * (df1 + df2 - 2.0) / (df1 * (df2 - 2.0).powi(2) * (df2 - 4.0)),
            );
            (mean, variance)
        }
        6 => {
            let p = state.p;
            (Some((1.0 - p) / p), Some((1.0 - p) / (p * p)))
        }
        7 => {
            let (a, b) = (state.ua, state.ub);
            if a < b {
                (Some((a + b) / 2.0), Some((b - a).powi(2) / 12.0))
            } else {
                (None, None)
            }
        }
        _ => {
            let lambda = state.lambda;
            (Some(1.0 / lambda), Some(1.0 / (lambda * lambda)))
        }
    }
}

/// Comando real para llevar el caso actual a la entrada. Solo donde el
/// motor tiene comando forward (Normal/Binomial/Poisson/ChiSquared/Uniform/
/// Exponential): t, F y Geométrica devuelven `None` — nunca botón fantasma.
pub(crate) fn prob_case_command(state: &ProbabilityPanelState) -> Option<String> {
    Some(match state.dist {
        0 => format!("Normal[{}, {}, {}]", state.mu, state.sigma, state.x),
        1 => {
            let n = state.n.round().max(1.0);
            let k = state.k.round().clamp(0.0, n);
            format!("Binomial[{n}, {}, {k}]", state.p)
        }
        2 => {
            let k = state.k.round().max(0.0);
            format!("Poisson[{}, {k}]", state.lambda)
        }
        4 => format!("ChiSquared[{}, {}]", state.df, state.x),
        7 => format!("Uniform[{}, {}, {}]", state.ua, state.ub, state.x),
        8 => format!("Exponential[{}, {}]", state.lambda, state.x),
        _ => return None,
    })
}

/// Comando real de cuantil (Inverse*) para la entrada; `None` donde el
/// motor no lo expone (Binomial, Poisson, Geométrica).
pub(crate) fn prob_quantile_command(state: &ProbabilityPanelState) -> Option<String> {
    let p = state.p_inv;
    Some(match state.dist {
        0 => format!("InverseNormal[{p}, {}, {}]", state.mu, state.sigma),
        3 => format!("InverseT[{p}, {}]", state.df),
        4 => format!("InverseChiSquared[{p}, {}]", state.df),
        5 => format!("InverseF[{p}, {}, {}]", state.df1, state.df2),
        7 => format!("InverseUniform[{p}, {}, {}]", state.ua, state.ub),
        8 => format!("InverseExponential[{p}, {}]", state.lambda),
        _ => return None,
    })
}

// --- extraído verbatim de panels.rs:6073-6186 (slice 31 F14.4.4) ---
/// Resultado puntual del caso de probabilidad.
///
/// Densidad/puntual + acumulada + detalle para el tooltip y el copiado.
/// Lógica pura movida verbatim del closure inline de `draw_probability_section`
/// (mismas 9 ramas y mismos errores honestos del motor; el dibujo queda
/// en `panels.rs`).
pub(crate) fn prob_case_result(
    state: &ProbabilityPanelState,
) -> Result<(f64, f64, String), String> {
    Ok(match state.dist {
        0 => {
            let (pdf, cdf) = (
                normal_pdf(state.x, state.mu, state.sigma)?,
                normal_cdf(state.x, state.mu, state.sigma)?,
            );
            (pdf, cdf, format!("f({})", format_statistic(state.x)))
        }
        1 => {
            let (n, k) = (
                state.n.round() as u64,
                state.k.round().min(state.n.round()) as u64,
            );
            let (pmf, cdf) = (binomial_pmf(k, n, state.p)?, binomial_cdf(k, n, state.p)?);
            (pmf, cdf, format!("P(X = {k}) · n = {n}"))
        }
        2 => {
            let k = state.k.round().max(0.0) as u64;
            let (pmf, cdf) = (poisson_pmf(k, state.lambda)?, poisson_cdf(k, state.lambda)?);
            (
                pmf,
                cdf,
                format!("P(X = {k}) · λ = {}", format_statistic(state.lambda)),
            )
        }
        3 => {
            let (pdf, cdf) = (
                student_t_pdf(state.x, state.df)?,
                student_t_cdf(state.x, state.df)?,
            );
            (
                pdf,
                cdf,
                format!(
                    "f({}) · t(gl={})",
                    format_statistic(state.x),
                    format_statistic(state.df)
                ),
            )
        }
        4 => {
            let (pdf, cdf) = (
                chi_squared_pdf(state.x, state.df)?,
                chi_squared_cdf(state.x, state.df)?,
            );
            (
                pdf,
                cdf,
                format!(
                    "f({}) · χ²(gl={})",
                    format_statistic(state.x),
                    format_statistic(state.df)
                ),
            )
        }
        5 => {
            let (pdf, cdf) = (
                f_distribution_pdf(state.x, state.df1, state.df2)?,
                f_distribution_cdf(state.x, state.df1, state.df2)?,
            );
            (
                pdf,
                cdf,
                format!(
                    "f({}) · F({},{})",
                    format_statistic(state.x),
                    format_statistic(state.df1),
                    format_statistic(state.df2)
                ),
            )
        }
        6 => {
            let k = state.k.round().max(0.0) as u64;
            let (pmf, cdf) = (geometric_pmf(k, state.p)?, geometric_cdf(k, state.p)?);
            (
                pmf,
                cdf,
                format!("P(X = {k}) · p = {}", format_statistic(state.p),),
            )
        }
        7 => {
            let (pdf, cdf) = (
                uniform_pdf(state.x, state.ua, state.ub)?,
                uniform_cdf(state.x, state.ua, state.ub)?,
            );
            (
                pdf,
                cdf,
                format!(
                    "f({}) · U({},{})",
                    format_statistic(state.x),
                    format_statistic(state.ua),
                    format_statistic(state.ub)
                ),
            )
        }
        _ => {
            let (pdf, cdf) = (
                exponential_pdf(state.x, state.lambda)?,
                exponential_cdf(state.x, state.lambda)?,
            );
            (
                pdf,
                cdf,
                format!(
                    "f({}) · Exp(λ={})",
                    format_statistic(state.x),
                    format_statistic(state.lambda)
                ),
            )
        }
    })
}

// --- extraído verbatim de panels.rs:5971-5993 + 6075-6079 (slice 32 F14.4.4) ---
// B-A156: los wrappers Es (`prob_dist_name/tooltip/kind_label/point_label`)
// se eliminan — su único consumidor era el draw, hoy display-localizado
// vía `prob_dist_*_for` (Es delega al canónico por construcción, mismo
// `PROB_DISTS` + misma caída "Normal" + misma asimetría `None`/continuo
// en índice desconocido). El pin histórico vive en
// `sheet_prob_selector_pinnea_nombres_tipos_y_rotulos` (vía `_for` Es).

// --- B-A156: display localizado del selector de distribución (i18n
// `stats.prob_dist_*/kind_*/point_*`; Es delega al canónico por
// construcción, resto → `stats_msg`; patrón B-A98/B-A144). Los wrappers
// Es previos se eliminaron (su único consumidor era el draw, hoy vía
// `_for`; el pin histórico `sheet_prob_selector_...` corre en Es).
/// Nombre de la distribución en el `locale` ("Normal" si el índice es
/// desconocido, misma caída que `prob_dist_name`).
pub(crate) fn prob_dist_name_for(dist: u8, locale: grafito_ui::i18n::Locale) -> &'static str {
    match dist {
        0 => grafito_ui::i18n::stats_msg("prob_dist_normal", locale),
        1 => grafito_ui::i18n::stats_msg("prob_dist_binomial", locale),
        2 => grafito_ui::i18n::stats_msg("prob_dist_poisson", locale),
        3 => grafito_ui::i18n::stats_msg("prob_dist_t", locale),
        4 => grafito_ui::i18n::stats_msg("prob_dist_chi2", locale),
        5 => grafito_ui::i18n::stats_msg("prob_dist_f", locale),
        6 => grafito_ui::i18n::stats_msg("prob_dist_geometric", locale),
        7 => grafito_ui::i18n::stats_msg("prob_dist_uniform", locale),
        8 => grafito_ui::i18n::stats_msg("prob_dist_exponential", locale),
        _ => grafito_ui::i18n::stats_msg("prob_dist_normal", locale),
    }
}

/// Tip de la distribución en el `locale` (crudo de `PROB_DISTS` solo en
/// Es por construcción del catálogo).
pub(crate) fn prob_dist_tip_for(dist: u8, locale: grafito_ui::i18n::Locale) -> &'static str {
    match dist {
        0 => grafito_ui::i18n::stats_msg("prob_dist_tip_normal", locale),
        1 => grafito_ui::i18n::stats_msg("prob_dist_tip_binomial", locale),
        2 => grafito_ui::i18n::stats_msg("prob_dist_tip_poisson", locale),
        3 => grafito_ui::i18n::stats_msg("prob_dist_tip_t", locale),
        4 => grafito_ui::i18n::stats_msg("prob_dist_tip_chi2", locale),
        5 => grafito_ui::i18n::stats_msg("prob_dist_tip_f", locale),
        6 => grafito_ui::i18n::stats_msg("prob_dist_tip_geometric", locale),
        7 => grafito_ui::i18n::stats_msg("prob_dist_tip_uniform", locale),
        8 => grafito_ui::i18n::stats_msg("prob_dist_tip_exponential", locale),
        _ => grafito_ui::i18n::stats_msg("prob_dist_tip_normal", locale),
    }
}

/// "Discreta"/"Continua" en el `locale` (misma rama que
/// `prob_dist_kind_label`).
pub(crate) fn prob_dist_kind_label_for(dist: u8, locale: grafito_ui::i18n::Locale) -> &'static str {
    if prob_distribution_is_discrete(dist) {
        grafito_ui::i18n::stats_msg("prob_kind_discrete", locale)
    } else {
        grafito_ui::i18n::stats_msg("prob_kind_continuous", locale)
    }
}

/// Tooltip "{kind} · {tip}" en el `locale`; `None` si el índice es
/// desconocido (misma asimetría que `prob_dist_tooltip`).
pub(crate) fn prob_dist_tooltip_for(dist: u8, locale: grafito_ui::i18n::Locale) -> Option<String> {
    if PROB_DISTS.iter().any(|(index, _, _)| *index == dist) {
        Some(format!(
            "{} · {}",
            prob_dist_kind_label_for(dist, locale),
            prob_dist_tip_for(dist, locale)
        ))
    } else {
        None
    }
}

/// "Puntual"/"Densidad" en el `locale` (misma rama que
/// `prob_point_label`).
pub(crate) fn prob_point_label_for(dist: u8, locale: grafito_ui::i18n::Locale) -> &'static str {
    if prob_distribution_is_discrete(dist) {
        grafito_ui::i18n::stats_msg("prob_point_discrete", locale)
    } else {
        grafito_ui::i18n::stats_msg("prob_point_continuous", locale)
    }
}

// --- extraído verbatim de panels.rs:197-203 + 281-404 (slice 9 F14.4.4) ---
/// Tabla XY de dos columnas importada de un CSV/TSV local. El resultado no
/// conserva ruta ni metadatos de origen a propósito.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LocalXYTable {
    pub x_name: String,
    pub y_name: String,
    pub xs: Vec<f64>,
    pub ys: Vec<f64>,
}

/// Parses an explicitly selected local two-column CSV/TSV payload. The result
/// deliberately carries no file path, timestamps, or other source metadata.
pub(crate) fn parse_local_xy_table(input: &str, delimiter: u8) -> Result<LocalXYTable, String> {
    let delimiter = char::from(delimiter);
    let rows = input
        .lines()
        .enumerate()
        .filter_map(|(index, line)| (!line.trim().is_empty()).then_some((index + 1, line)))
        .map(|(line_number, line)| {
            parse_delimited_row(line, delimiter)
                .map_err(|error| format!("Fila {line_number}: {error}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if rows.is_empty() {
        return Err("El archivo no contiene filas de datos".to_string());
    }

    let mut rows: VecDeque<_> = rows.into();
    let Some(first) = rows.pop_front() else {
        return Err("El archivo no contiene filas de datos".to_string());
    };
    if first.len() != 2 {
        return Err("Cada fila debe tener exactamente dos columnas".to_string());
    }
    let first_values = parse_local_xy_values(&first);
    let (x_name, y_name, mut xs, mut ys) = match first_values {
        Ok((x, y)) => ("x".to_string(), "y".to_string(), vec![x], vec![y]),
        Err(error) => {
            if first.iter().any(|cell| cell.parse::<f64>().is_ok()) {
                return Err(format!("Fila 1: {error}"));
            }
            if first[0].is_empty() || first[1].is_empty() {
                return Err("Los encabezados de columna no pueden estar vacíos".to_string());
            }
            (first[0].clone(), first[1].clone(), Vec::new(), Vec::new())
        }
    };

    for (index, row) in rows.into_iter().enumerate() {
        if row.len() != 2 {
            return Err(format!(
                "Fila {}: se esperaban exactamente dos columnas",
                index + 2
            ));
        }
        let (x, y) =
            parse_local_xy_values(&row).map_err(|error| format!("Fila {}: {error}", index + 2))?;
        xs.push(x);
        ys.push(y);
        if xs.len() > grafito_core::validation::MAX_DATA_TABLE_ROWS {
            return Err(format!(
                "El archivo supera el máximo de {} filas",
                grafito_core::validation::MAX_DATA_TABLE_ROWS
            ));
        }
    }
    if xs.len() < 2 {
        return Err("Se necesitan al menos dos pares numéricos finitos".to_string());
    }

    Ok(LocalXYTable {
        x_name,
        y_name,
        xs,
        ys,
    })
}

fn parse_delimited_row(line: &str, delimiter: char) -> Result<Vec<String>, String> {
    let mut cells = Vec::new();
    let mut current = String::new();
    let mut chars = line.chars().peekable();
    let mut in_quotes = false;

    while let Some(character) = chars.next() {
        if in_quotes {
            if character == '"' {
                if chars.peek() == Some(&'"') {
                    current.push('"');
                    let _ = chars.next();
                } else {
                    in_quotes = false;
                }
            } else {
                current.push(character);
            }
        } else if character == '"' {
            if !current.trim().is_empty() {
                return Err("las comillas deben iniciar una celda".to_string());
            }
            current.clear();
            in_quotes = true;
        } else if character == delimiter {
            cells.push(current.trim().to_string());
            current.clear();
        } else {
            current.push(character);
        }
    }
    if in_quotes {
        return Err("comillas sin cerrar".to_string());
    }
    cells.push(current.trim().to_string());
    if let Some(first) = cells.first_mut() {
        *first = first.trim_start_matches('\u{feff}').to_string();
    }
    Ok(cells)
}

fn parse_local_xy_values(row: &[String]) -> Result<(f64, f64), String> {
    if row.len() != 2 {
        return Err("se esperaban exactamente dos columnas".to_string());
    }
    let x = row[0]
        .parse::<f64>()
        .map_err(|_| format!("'{}' no es un número válido", row[0]))?;
    let y = row[1]
        .parse::<f64>()
        .map_err(|_| format!("'{}' no es un número válido", row[1]))?;
    if !x.is_finite() || !y.is_finite() {
        return Err("los valores deben ser finitos".to_string());
    }
    Ok((x, y))
}

/// Armado puro tabla→objetos del import CSV/TSV local: `DataTable` con los
/// nombres y vectores tal cual + `ScatterPlot` con los mismos puntos y
/// linkeada al id de la tabla. Verbatim de `commit_local_xy_table` en
/// `panels.rs` (slice 24 F14.4.4, sin cambio de comportamiento; el commit
/// con undo queda allá porque muta el documento vía `app`).
pub(crate) fn local_xy_table_objects(table: LocalXYTable) -> (GeoObject, GeoObject, ObjectId) {
    let data_table = DataTableObj::new(
        table.x_name,
        table.y_name,
        table.xs.clone(),
        table.ys.clone(),
    );
    let data_id = data_table.id;
    let scatter = ScatterPlotObj::new(table.xs, table.ys).linked_to(data_id);
    (
        GeoObject::DataTable(data_table),
        GeoObject::ScatterPlot(scatter),
        data_id,
    )
}

/// Texto del comando de estudio de la hoja CAS (`None` si la etiqueta está
/// vacía). Puro, testeable.
/// (movido desde `panels.rs`: slice 10 F14.4.4, sin cambio de comportamiento)
pub(crate) fn wc_study_command_text(label: &str) -> Option<String> {
    let label = label.trim();
    if label.is_empty() {
        None
    } else {
        Some(format!("FunctionStudy[{label}]"))
    }
}

/// Texto del comando T2 (`None` si falta tramo o n fuera de 1..=1_000_000).
/// La variable es `x` (se dice en el hint de la sección). Puro, testeable.
/// (movido desde `panels.rs`: slice 10 F14.4.4, sin cambio de comportamiento)
pub(crate) fn wc_riemann_command_text(
    expr: &str,
    a: &str,
    b: &str,
    n: f32,
    metodo: &str,
) -> Option<String> {
    let (expr, a, b, metodo) = (expr.trim(), a.trim(), b.trim(), metodo.trim());
    if expr.is_empty() || a.is_empty() || b.is_empty() || metodo.is_empty() {
        return None;
    }
    if !n.is_finite() {
        return None;
    }
    let n = n.round() as usize;
    if !(1..=1_000_000).contains(&n) {
        return None;
    }
    Some(format!("RiemannSum[{expr}, x, {a}, {b}, {n}, {metodo}]"))
}

/// Texto del comando de la integral exacta T2 (para comparar en la hoja).
/// Puro, testeable.
/// (movido desde `panels.rs`: slice 10 F14.4.4, sin cambio de comportamiento)
pub(crate) fn wc_exact_integral_command_text(expr: &str, a: &str, b: &str) -> Option<String> {
    let (expr, a, b) = (expr.trim(), a.trim(), b.trim());
    if expr.is_empty() || a.is_empty() || b.is_empty() {
        None
    } else {
        Some(format!("Integral[{expr}, x, {a}, {b}]"))
    }
}

/// Texto del comando T3 (`None` si la expresión no cierra o el orden sale
/// del rango del slider 1..=10). Puro, testeable.
/// (movido desde `panels.rs`: slice 10 F14.4.4, sin cambio de comportamiento)
pub(crate) fn wc_taylor_command_text(
    expr: &str,
    centro: &str,
    orden: i32,
    x: &str,
) -> Option<String> {
    let (expr, centro, x) = (expr.trim(), centro.trim(), x.trim());
    if expr.is_empty() || centro.is_empty() || x.is_empty() {
        return None;
    }
    if !(1..=10).contains(&orden) {
        return None;
    }
    if centro.parse::<f64>().is_err() || x.parse::<f64>().is_err() {
        return None;
    }
    Some(format!("Taylor[{expr}, x, {centro}, {orden}]"))
}

/// Línea de resto observado T3 con el motor existente. `None` honesto si
/// algo no evalúa (la sección muestra "revisá la expresión").
/// (movido desde `panels.rs`: slice 10 F14.4.4, sin cambio de comportamiento)
pub(crate) fn wc_taylor_remainder_line(
    expr: &str,
    centro: &str,
    orden: i32,
    x: &str,
    vars: &BTreeMap<String, f64>,
) -> Option<String> {
    let centro: f64 = centro.trim().parse().ok()?;
    let x: f64 = x.trim().parse().ok()?;
    let orden_usize: usize = usize::try_from(orden).ok()?;
    let r = grafito_command::commands::taylor_remainder_observed(
        expr.trim(),
        "x",
        centro,
        orden_usize,
        x,
        vars,
    )?;
    let siguiente = r
        .termino_siguiente
        .map(|t| format!("{t:.2e}"))
        .unwrap_or_else(|| "—".to_string());
    Some(format!(
        "P{orden}({x}) ≈ {:.6} · f = {:.6} · resto {:.2e} · sig. {siguiente}",
        r.approx, r.exact, r.resto_observado
    ))
}

/// Ecuación en grande del Inspector (titular de la card de identidad).
/// Verbatim de `panels.rs` (slice 17 F14.4.4): pura, sin egui.
pub(crate) fn inspector_equation_text(obj: &GeoObject) -> Option<String> {
    fn non_empty(raw: &str) -> Option<String> {
        let trimmed = raw.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    }
    match obj {
        GeoObject::Function(f) => non_empty(&f.expr).map(|expr| format!("y = {expr}")),
        GeoObject::ImplicitCurve(c) => {
            let lhs = c.expr_lhs.trim();
            let rhs = c.expr_rhs.trim();
            let op = match c.operator {
                grafito_core::RelationOperator::Eq => "=",
                grafito_core::RelationOperator::Less => "<",
                grafito_core::RelationOperator::Greater => ">",
                grafito_core::RelationOperator::LessEq => "≤",
                grafito_core::RelationOperator::GreaterEq => "≥",
            };
            match (lhs.is_empty(), rhs.is_empty()) {
                (true, true) => None,
                (true, false) => Some(rhs.to_string()),
                (false, true) => Some(lhs.to_string()),
                (false, false) => Some(format!("{lhs} {op} {rhs}")),
            }
        }
        GeoObject::ParametricCurve2D(c) => {
            let (x, y) = (c.expr_x.trim(), c.expr_y.trim());
            if x.is_empty() && y.is_empty() {
                None
            } else {
                Some(format!("({x}, {y})"))
            }
        }
        GeoObject::ParametricCurve3D(c) => {
            let (x, y, z) = (c.expr_x.trim(), c.expr_y.trim(), c.expr_z.trim());
            if x.is_empty() && y.is_empty() && z.is_empty() {
                None
            } else {
                Some(format!("({x}, {y}, {z})"))
            }
        }
        GeoObject::PolarCurve(c) => non_empty(&c.expr_r).map(|expr| format!("r = {expr}")),
        GeoObject::Surface3D(s) => {
            if s.is_parametric {
                let (x, y, z) = (s.expr_x.trim(), s.expr_y.trim(), s.expr_z.trim());
                if x.is_empty() && y.is_empty() && z.is_empty() {
                    None
                } else {
                    Some(format!("({x}, {y}, {z})"))
                }
            } else if s.is_complex {
                non_empty(&s.expr).map(|expr| format!("|{expr}|"))
            } else {
                non_empty(&s.expr).map(|expr| format!("z = {expr}"))
            }
        }
        _ => None,
    }
}

/// Caption terciario del tipo de objeto, en español rioplatense.
/// El nombre muerto (`Surface3D`, …) nunca va en grande: solo acá,
/// en `TYPE_XS`. Puro y testeable headless.
/// Verbatim de `panels.rs` (slice 17 F14.4.4, sin cambio de comportamiento).
pub(crate) fn inspector_type_caption(obj: &GeoObject) -> &'static str {
    match obj {
        GeoObject::Point(_) => "punto",
        GeoObject::Line(_) => "recta",
        GeoObject::Circle(_) => "círculo",
        GeoObject::Polygon(_) => "polígono",
        GeoObject::Polyline(_) => "polilínea",
        GeoObject::Pencil(p) if p.is_dynamic_locus() => "lugar geométrico",
        GeoObject::Pencil(_) => "trazo",
        GeoObject::Function(_) => "función",
        GeoObject::Text(_) => "texto",
        GeoObject::Ellipse(_) => "elipse",
        GeoObject::Parabola(_) => "parábola",
        GeoObject::Hyperbola(_) => "hipérbola",
        GeoObject::Arc(_) => "arco",
        GeoObject::Sector(_) => "sector",
        GeoObject::BezierCurve(_) => "curva Bézier",
        GeoObject::Spline(_) => "spline",
        GeoObject::Point3D(_) => "punto 3D",
        GeoObject::Segment3D(_) => "segmento 3D",
        GeoObject::Plane3D(_) => "plano 3D",
        GeoObject::Line3D(_) => "recta 3D",
        GeoObject::Sphere3D(_) => "esfera 3D",
        GeoObject::Cube3D(_) => "cubo 3D",
        GeoObject::Tetrahedron3D(_) => "tetraedro 3D",
        GeoObject::Pyramid3D(_) => "pirámide 3D",
        GeoObject::Cone3D(_) => "cono 3D",
        GeoObject::Cylinder3D(_) => "cilindro 3D",
        GeoObject::Torus3D(_) => "toro 3D",
        GeoObject::MoebiusStrip(_) => "cinta de Möbius",
        GeoObject::Surface3D(s) if s.is_complex => "superficie compleja 3D",
        GeoObject::Surface3D(_) => "superficie 3D",
        GeoObject::Prism3D(_) => "prisma 3D",
        GeoObject::Quadric3D(quadric) if grafito_render::quadric_uses_placeholder(quadric) => {
            "cuádrica 3D · vista aproximada"
        }
        GeoObject::Quadric3D(_) => "cuádrica 3D",
        GeoObject::ImplicitSurface3D(_) => "superficie implícita 3D",
        GeoObject::ParametricCurve2D(_) => "curva paramétrica 2D",
        GeoObject::ParametricCurve3D(_) => "curva paramétrica 3D",
        GeoObject::PolarCurve(_) => "curva polar",
        GeoObject::ImplicitCurve(_) => "curva implícita",
        GeoObject::VectorField2D(_) => "campo vectorial 2D",
        GeoObject::ComplexGrid(_) => "grilla compleja",
        GeoObject::ComplexMapping(_) => "mapeo complejo",
        GeoObject::ComplexIntegral(_) => "integral compleja",
        GeoObject::Attractor3D(_) => "atractor 3D",
        GeoObject::Fractal2D(_) => "fractal 2D",
        GeoObject::RegularPolychoron4D(_) => "polícoro regular 4D",
        GeoObject::RegularPolytopeND(_) => "politopo regular N-D",
        GeoObject::HyperSurface4D(_) => "hiperficie 4D",
        GeoObject::VectorField3D(_) => "campo vectorial 3D",
        GeoObject::Histogram(_) => "histograma",
        GeoObject::BarChart(_) => "gráfico de barras",
        GeoObject::PieChart(_) => "gráfico de torta",
        GeoObject::ScatterPlot(_) => "dispersión",
        GeoObject::BoxPlot(_) => "diagrama de caja",
        GeoObject::RegressionLine(_) => "recta de regresión",
        GeoObject::DataTable(_) => "tabla de datos",
        GeoObject::PhasePortrait(_) => "retrato de fase",
        GeoObject::Transformed(_) => "objeto transformado",
        // `GeoObject` es non-exhaustive: variantes futuras caen al nombre
        // honesto en inglés hasta tener caption en español.
        &_ => obj.name(),
    }
}

/// Texto de la fila de etiqueta del Inspector. Nunca devuelve una letra
/// suelta: la etiqueta cruda (p. ej. `S`, autolabel de `Surface3D` según
/// `document.rs:1193-1202` — primera letra del nombre del tipo) se presenta
/// con prefijo `Etiqueta: ` para que no parezca texto truncado.
/// `None` si no hay etiqueta que mostrar. Puro y testeable headless.
/// Verbatim de `panels.rs` (slice 17 F14.4.4).
pub(crate) fn inspector_label_text(label: &str) -> Option<String> {
    let trimmed = label.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(format!("Etiqueta: {trimmed}"))
    }
}

/// Tooltip honesto de la card de identidad: texto completo sin truncar.
/// `headline` es la ecuación en grande (o el caption si no hay ecuación).
/// Verbatim de `panels.rs` (slice 17 F14.4.4).
pub(crate) fn inspector_identity_tooltip(headline: &str, label: &str, visible: bool) -> String {
    let state = inspector_visibility_label(visible);
    match inspector_label_text(label) {
        Some(labeled) => format!("{headline} · {labeled} · {state}"),
        None => format!("{headline} · {state}"),
    }
}

/// Tooltip de la card de identidad en el idioma pedido (B-A110 + B-A111).
/// Localiza la palabra de estado vía [`inspector_visibility_label_for`] y el
/// prefijo `Etiqueta: ` vía [`inspector_label_text_for`] (B-A111).
pub(crate) fn inspector_identity_tooltip_for(
    headline: &str,
    label: &str,
    visible: bool,
    locale: Locale,
) -> String {
    if locale == Locale::Es {
        inspector_identity_tooltip(headline, label, visible)
    } else {
        let state = inspector_visibility_label_for(visible, locale);
        match inspector_label_text_for(label, locale) {
            Some(labeled) => format!("{headline} · {labeled} · {state}"),
            None => format!("{headline} · {state}"),
        }
    }
}

/// Etiqueta ES del estilo de trazo para el Inspector (Q2).
/// Verbatim de `panels.rs` (slice 17 F14.4.4; era privada, ahora `pub(crate)`).
pub(crate) fn line_style_label(style: LineStyle) -> &'static str {
    match style {
        LineStyle::Solid => "Continuo",
        LineStyle::Dashed => "Rayado",
        LineStyle::Dotted => "Punteado",
    }
}

/// Etiqueta ES de la forma de punto para el Inspector (Q2).
/// Verbatim de `panels.rs` (slice 17 F14.4.4; era privada, ahora `pub(crate)`).
pub(crate) fn point_style_label(style: PointStyle) -> &'static str {
    match style {
        PointStyle::Dot => "Punto",
        PointStyle::Circle => "Aro",
        PointStyle::Cross => "Cruz",
        PointStyle::Plus => "Cruz +",
    }
}

/// Prefijo `Etiqueta: …` en el idioma pedido (B-A111, template `{label}`).
/// Es delega al canónico [`inspector_label_text`] (con `trim`/None intactos);
/// el resto interpola el template i18n.
pub(crate) fn inspector_label_text_for(label: &str, locale: Locale) -> Option<String> {
    if locale == Locale::Es {
        inspector_label_text(label)
    } else {
        let trimmed = label.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(
                grafito_ui::i18n::inspector_msg("label_prefix", locale).replace("{label}", trimmed),
            )
        }
    }
}

/// Etiqueta del estilo de trazo en el idioma pedido (B-A111).
/// Es delega al canónico [`line_style_label`]; el resto sale de i18n.
pub(crate) fn line_style_label_for(style: LineStyle, locale: Locale) -> &'static str {
    if locale == Locale::Es {
        line_style_label(style)
    } else {
        let suffix = match style {
            LineStyle::Solid => "line_solid",
            LineStyle::Dashed => "line_dashed",
            LineStyle::Dotted => "line_dotted",
        };
        grafito_ui::i18n::inspector_msg(suffix, locale)
    }
}

/// Etiqueta de la forma de punto en el idioma pedido (B-A111).
/// Es delega al canónico [`point_style_label`]; el resto sale de i18n.
pub(crate) fn point_style_label_for(style: PointStyle, locale: Locale) -> &'static str {
    if locale == Locale::Es {
        point_style_label(style)
    } else {
        let suffix = match style {
            PointStyle::Dot => "point_dot",
            PointStyle::Circle => "point_circle",
            PointStyle::Cross => "point_cross",
            PointStyle::Plus => "point_plus",
        };
        grafito_ui::i18n::inspector_msg(suffix, locale)
    }
}

/// Textos puros del panel Inspector de propiedades (header + vacío +
/// ecuación editable + identidad + estilo).
/// Verbatim de `panels.rs` (`draw_right_properties_panel`,
/// `draw_right_properties_contents`, `draw_inspector_identity`,
/// `draw_inspector_empty_state`; slice 49 F14.4.4; puros, sin egui).
pub(crate) const INSPECTOR_PANEL_TITLE: &str = "Inspector";
pub(crate) const INSPECTOR_PANEL_SUBTITLE: &str = "Propiedades del objeto seleccionado.";
pub(crate) const INSPECTOR_EMPTY_TITLE: &str = "Tocá un objeto del lienzo para editarlo acá.";
pub(crate) const INSPECTOR_EMPTY_HINT: &str =
    "Acá ajustás su geometría, estilo y controles avanzados.";
pub(crate) const INSPECTOR_MISSING_OBJECT: &str = "Objeto inexistente.";
pub(crate) const INSPECTOR_EQUATION_LABEL: &str = "Ecuación (editable)";
pub(crate) const INSPECTOR_REVERT_LABEL: &str = "Revertir";
pub(crate) const INSPECTOR_USE_EXAMPLE_LABEL: &str = "Usar ejemplo";
pub(crate) const INSPECTOR_APPLY_LABEL: &str = "Aplicar ecuación";
pub(crate) const INSPECTOR_IDENTITY_TITLE: &str = "Ecuación";
pub(crate) const INSPECTOR_NO_EQUATION: &str = "Ecuación no disponible para este tipo";
pub(crate) const INSPECTOR_STYLE_TITLE: &str = "Estilo";
pub(crate) const INSPECTOR_STYLE_SUBTITLE: &str = "Trazo, marcador y capa de dibujo (0 = fondo).";
pub(crate) const INSPECTOR_STYLE_NONE_HINT: &str =
    "Este tipo no tiene trazo ni marcador con estilo.";
pub(crate) const INSPECTOR_LAYER_LABEL: &str = "Capa:";

/// Chrome del panel Inspector base en el idioma pedido (B-A108 + B-A109 + B-A110 + B-A111 + B-A112 + B-A113 + B-A159).
/// Es delega a la const canónica (pineada por el test de este módulo);
/// el resto sale de `grafito-ui` i18n (`inspector.*`, 88 keys).
/// Sufijo desconocido → el sufijo (mismo contrato que `inspector_msg`).
pub(crate) fn inspector_chrome_for(suffix: &'static str, locale: Locale) -> &'static str {
    use grafito_ui::i18n::inspector_msg;
    if locale == Locale::Es {
        match suffix {
            "panel_title" => INSPECTOR_PANEL_TITLE,
            "panel_subtitle" => INSPECTOR_PANEL_SUBTITLE,
            "empty_title" => INSPECTOR_EMPTY_TITLE,
            "empty_hint" => INSPECTOR_EMPTY_HINT,
            "missing_object" => INSPECTOR_MISSING_OBJECT,
            "equation_label" => INSPECTOR_EQUATION_LABEL,
            "revert" => INSPECTOR_REVERT_LABEL,
            "use_example" => INSPECTOR_USE_EXAMPLE_LABEL,
            "apply" => INSPECTOR_APPLY_LABEL,
            "identity_title" => INSPECTOR_IDENTITY_TITLE,
            "no_equation" => INSPECTOR_NO_EQUATION,
            "style_title" => INSPECTOR_STYLE_TITLE,
            "style_subtitle" => INSPECTOR_STYLE_SUBTITLE,
            "style_none_hint" => INSPECTOR_STYLE_NONE_HINT,
            "layer_label" => INSPECTOR_LAYER_LABEL,
            "poly_projection_title" => POLYCHORON_PROJECTION_TITLE,
            "poly_projection_subtitle" => POLYCHORON_PROJECTION_SUBTITLE,
            "poly_anim_title" => POLYCHORON_ANIM_TITLE,
            "poly_anim_body" => POLYCHORON_ANIM_BODY,
            "poly_geometry_title" => POLYCHORON_GEOMETRY_TITLE,
            "poly_geometry_subtitle" => POLYCHORON_GEOMETRY_SUBTITLE,
            "poly_appearance_title" => POLYCHORON_APPEARANCE_TITLE,
            "poly_appearance_subtitle" => POLYCHORON_APPEARANCE_SUBTITLE,
            "poly_fill_omitted_hint" => POLYCHORON_FILL_OMITTED_HINT,
            "poly_rotation_manual_title" => POLYCHORON_ROTATION_MANUAL_TITLE,
            "poly_rotation_manual_guide" => POLYCHORON_ROTATION_MANUAL_GUIDE,
            "poly_rotation_reset_label" => POLYCHORON_ROTATION_RESET_LABEL,
            "poly_rotation_planes_title" => POLYCHORON_ROTATION_PLANES_TITLE,
            "polytope_geometry_subtitle" => POLYTOPE_ND_GEOMETRY_SUBTITLE,
            "polytope_projection_subtitle" => POLYTOPE_ND_PROJECTION_SUBTITLE,
            "polytope_anim_dim4_body" => POLYTOPE_ND_ANIM_DIM4_BODY,
            "polytope_anim_other_body" => POLYTOPE_ND_ANIM_OTHER_BODY,
            "polytope_appearance_subtitle" => POLYTOPE_ND_APPEARANCE_SUBTITLE,
            "polytope_fill_hint" => POLYTOPE_ND_FILL_HINT,
            "polytope_rotation_guide" => POLYTOPE_ND_ROTATION_GUIDE,
            "visibility_visible" => "Visible",
            "visibility_hidden" => "Oculto",
            "polychoron_kind_pentachoron" => "Pentácoron (5-celda)",
            "polychoron_kind_tesseract" => "Teseracto",
            "polychoron_kind_sixteen" => "16-celda",
            "polychoron_kind_twentyfour" => "24-celda",
            "polychoron_kind_onetwenty" => "120-celda",
            "polychoron_kind_sixhundred" => "600-celda",
            "polytope_family_simplex" => "Símplex",
            "polytope_family_hypercube" => "Hipercubo",
            "polytope_family_cross" => "Politopo cruzado",
            "cube3d_title" => INSPECTOR_CUBE3D_TITLE,
            "sphere3d_title" => INSPECTOR_SPHERE3D_TITLE,
            "point3d_title" => INSPECTOR_POINT3D_TITLE,
            "center_label" => INSPECTOR_CENTER_LABEL,
            "implicit3d_title" => INSPECTOR_IMPLICIT3D_TITLE,
            "dock_empty_title" => INSPECTOR_DOCK_EMPTY_TITLE,
            "dock_empty_guide" => INSPECTOR_DOCK_EMPTY_GUIDE,
            "size_label" => INSPECTOR_SIZE_TEXT,
            "radius_label" => INSPECTOR_RADIUS_TEXT,
            "scale_label" => INSPECTOR_SCALE_TEXT,
            "edge_width_label" => INSPECTOR_EDGE_WIDTH_TEXT,
            "edge_color_label" => INSPECTOR_EDGE_COLOR_LABEL,
            "edge_color_hover" => INSPECTOR_EDGE_COLOR_HOVER,
            "fill_enabled_label" => INSPECTOR_FILL_ENABLED_LABEL,
            "fill_color_label" => INSPECTOR_FILL_COLOR_LABEL,
            "fill_color_hover" => INSPECTOR_FILL_COLOR_HOVER,
            "dimension_label" => INSPECTOR_DIMENSION_LABEL,
            "close_context" => INSPECTOR_CLOSE_CONTEXT,
            "close_anim" => INSPECTOR_CLOSE_ANIM,
            "implicit_quick_guide" => IMPLICIT_QUICK_PREVIEW_GUIDE,
            "implicit_quick_label" => IMPLICIT_QUICK_PREVIEW_LABEL,
            "implicit_quick_hover" => IMPLICIT_QUICK_PREVIEW_HOVER,
            "implicit_running" => IMPLICIT_PREVIEW_RUNNING,
            "implicit_already_shown" => IMPLICIT_PREVIEW_ALREADY_SHOWN,
            "implicit_already_running" => IMPLICIT_PREVIEW_ALREADY_RUNNING,
            "implicit_start_failed" => IMPLICIT_PREVIEW_START_FAILED,
            "implicit_cancelled" => IMPLICIT_PREVIEW_CANCELLED,
            "implicit_cancel_empty" => IMPLICIT_PREVIEW_CANCEL_EMPTY,
            "implicit_cancel_hover" => IMPLICIT_PREVIEW_CANCEL_HOVER,
            _ => suffix,
        }
    } else {
        inspector_msg(suffix, locale)
    }
}

/// Caja implícita 3D en el idioma pedido (B-A112, templates `{ranges}`/`{cells}`).
/// Es delega al canónico [`implicit_3d_box_text`]; el resto interpola i18n
/// con los mismos números (`{:.2}` + `³`, como el canónico).
pub(crate) fn implicit_3d_box_text_for(bounds: [f64; 6], cells: usize, locale: Locale) -> String {
    if locale == Locale::Es {
        let [x_min, x_max, y_min, y_max, z_min, z_max] = bounds;
        implicit_3d_box_text(x_min, x_max, y_min, y_max, z_min, z_max, cells)
    } else {
        let [x_min, x_max, y_min, y_max, z_min, z_max] = bounds;
        let ranges = format!(
            "[{x_min:.2}, {x_max:.2}] × [{y_min:.2}, {y_max:.2}] × [{z_min:.2}, {z_max:.2}]"
        );
        grafito_ui::i18n::inspector_msg("implicit_box", locale)
            .replace("{ranges}", &ranges)
            .replace("{cells}", &cells.to_string())
    }
}

/// Estado implícito 3D en el idioma pedido (B-A112, template `{status}`).
/// Es delega al canónico [`implicit_3d_state_text`]; el resto interpola i18n.
pub(crate) fn implicit_3d_state_text_for(status: &str, locale: Locale) -> String {
    if locale == Locale::Es {
        implicit_3d_state_text(status)
    } else {
        grafito_ui::i18n::inspector_msg("implicit_state", locale).replace("{status}", status)
    }
}

/// Vista previa implícita 3D en el idioma pedido (B-A112, template `{tris}`).
/// Es delega al canónico [`implicit_3d_preview_text`]; el resto interpola i18n.
pub(crate) fn implicit_3d_preview_text_for(tris: usize, locale: Locale) -> String {
    if locale == Locale::Es {
        implicit_3d_preview_text(tris)
    } else {
        grafito_ui::i18n::inspector_msg("implicit_preview", locale)
            .replace("{tris}", &tris.to_string())
    }
}

/// Cuerpo de la tarjeta de proyección N-D en el idioma pedido (B-A109).
/// Es delega a la const canónica vía [`polytope_nd_projection_body`]
/// (wrapper intacto para los tests B-A previos); el resto sale de i18n.
pub(crate) fn polytope_nd_projection_body_for(dimension: usize, locale: Locale) -> &'static str {
    if locale == Locale::Es {
        polytope_nd_projection_body(dimension)
    } else if dimension == 4 {
        grafito_ui::i18n::inspector_msg("polytope_anim_dim4_body", locale)
    } else {
        grafito_ui::i18n::inspector_msg("polytope_anim_other_body", locale)
    }
}

/// Rótulo Visible/Oculto del badge de identidad. Verbatim de `panels.rs`.
pub(crate) fn inspector_visibility_label(visible: bool) -> &'static str {
    if visible {
        "Visible"
    } else {
        "Oculto"
    }
}

/// Tooltip del botón "Usar ejemplo": inserta sintaxis válida clicable.
/// Verbatim de `panels.rs` (`format!("Inserta sintaxis válida: {example}")`).
pub(crate) fn inspector_example_tooltip(example: &str) -> String {
    format!("Inserta sintaxis válida: {example}")
}

/// Línea "Medida: …" del sólido cuando el motor la calcula en forma cerrada.
/// Verbatim de `panels.rs` (`format!("Medida: {measure}")`).
pub(crate) fn inspector_measure_text(measure: &str) -> String {
    format!("Medida: {measure}")
}

/// Rótulo Visible/Oculto en el idioma pedido (B-A110).
/// Es delega al canónico [`inspector_visibility_label`] (wrapper intacto
/// para los tests previos); el resto sale de i18n.
pub(crate) fn inspector_visibility_label_for(visible: bool, locale: Locale) -> &'static str {
    if locale == Locale::Es {
        inspector_visibility_label(visible)
    } else if visible {
        grafito_ui::i18n::inspector_msg("visibility_visible", locale)
    } else {
        grafito_ui::i18n::inspector_msg("visibility_hidden", locale)
    }
}

/// Tooltip "Usar ejemplo" en el idioma pedido (B-A110, template `{example}`).
/// Es delega al canónico [`inspector_example_tooltip`]; el resto interpola
/// el template i18n.
pub(crate) fn inspector_example_tooltip_for(example: &str, locale: Locale) -> String {
    if locale == Locale::Es {
        inspector_example_tooltip(example)
    } else {
        grafito_ui::i18n::inspector_msg("example_tooltip", locale).replace("{example}", example)
    }
}

/// Línea "Medida: …" en el idioma pedido (B-A110, template `{measure}`).
/// Es delega al canónico [`inspector_measure_text`]; el resto interpola
/// el template i18n.
pub(crate) fn inspector_measure_text_for(measure: &str, locale: Locale) -> String {
    if locale == Locale::Es {
        inspector_measure_text(measure)
    } else {
        grafito_ui::i18n::inspector_msg("measure", locale).replace("{measure}", measure)
    }
}

/// Escapa un texto para interpolarlo en LaTeX (Q2).
/// Verbatim de `panels.rs` (slice 18 F14.4.4; era privada, ahora `pub(crate)`).
pub(crate) fn escape_latex(s: &str) -> String {
    s.replace('\\', "\\textbackslash{}")
        .replace('_', "\\_")
        .replace('%', "\\%")
        .replace('&', "\\&")
        .replace('#', "\\#")
        .replace('$', "\\$")
        .replace('{', "\\{")
        .replace('}', "\\}")
}

/// Genera una lista enumerada en LaTeX a partir del registro de construcción.
/// Verbatim de `panels.rs` (slice 18 F14.4.4; era privada, ahora `pub(crate)`).
pub(crate) fn construction_log_to_latex(log: &[ConstructionStep]) -> String {
    let mut s = String::new();
    s.push_str("% Protocolo de Construcción — Grafito\n");
    s.push_str("\\begin{enumerate}\n");
    for step in log {
        let inputs = if step.inputs.is_empty() {
            "\\textemdash".to_string()
        } else {
            step.inputs.join(", ")
        };
        let output = if step.output.is_empty() {
            "\\textemdash".to_string()
        } else {
            step.output.clone()
        };
        let disabled = if step.disabled {
            " (deshabilitado)"
        } else {
            ""
        };
        s.push_str(&format!(
            "  \\item {}{}: {} $\\rightarrow$ {}\n",
            escape_latex(&step.action),
            disabled,
            escape_latex(&inputs),
            escape_latex(&output),
        ));
    }
    s.push_str("\\end{enumerate}\n");
    s
}

/// Navegación view-only del protocolo de construcción (Ola 1.5): paso actual
/// y reproducción. No reordena ni desactiva nada: solo selecciona el objeto
/// del paso en el lienzo y resalta la tarjeta. Vive en `ctx.data`.
/// Verbatim de `panels.rs` (slice 18 F14.4.4; era privada, ahora `pub(crate)`
/// con campos `pub(crate)` porque el dibujo los lee/escribe por import).
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ProtocolNavState {
    /// Índice 0-based sobre `construction_log` (None = sin paso activo).
    pub(crate) cursor: Option<usize>,
    pub(crate) playing: bool,
    pub(crate) last_advance: f64,
}

/// Intervalo de avance del play (segundos por paso).
/// Verbatim de `panels.rs` (slice 18 F14.4.4; era privada, ahora `pub(crate)`).
pub(crate) const PROTOCOL_PLAY_INTERVAL_SECS: f64 = 1.0;

/// Avanza la reproducción si corresponde. Pura sobre el estado (testeable con
/// tiempos sintéticos): arranca en el paso 0, avanza de a uno y frena al final.
/// Verbatim de `panels.rs` (slice 18 F14.4.4; era privada, ahora `pub(crate)`).
pub(crate) fn protocol_advance(state: &mut ProtocolNavState, len: usize, time: f64) {
    if !state.playing || len == 0 {
        return;
    }
    if time - state.last_advance < PROTOCOL_PLAY_INTERVAL_SECS {
        return;
    }
    state.last_advance = time;
    match state.cursor {
        None => state.cursor = Some(0),
        Some(cursor) if cursor + 1 >= len => {
            state.cursor = Some(len - 1);
            state.playing = false;
        }
        Some(cursor) => state.cursor = Some(cursor + 1),
    }
}

/// Aplica una edición del panel de propiedades sobre un objeto por id.
/// Primitiva de mutación de todos los paneles (propiedades + álgebra):
/// si `changed` es falso o el objeto no existe, no toca el documento;
/// si la edición deja el objeto igual, el reemplazo estadificado devuelve
/// `None` (sin bump de versión); si cambia, reemplaza con snapshot previo
/// para el undo diferido. Verbatim de `panels.rs` (slice 19 F14.4.4; en
/// origen era `#[cfg(test)]` —solo la usan los tests— y se preserva).
#[cfg(test)]
pub(crate) fn apply_object_panel_edit(
    document: &mut Document,
    id: ObjectId,
    changed: bool,
    edit: impl FnOnce(&mut GeoObject),
) -> Result<bool, String> {
    Ok(apply_object_panel_edit_with_previous(document, id, changed, edit)?.is_some())
}

/// Variante con el documento previo (para `capture_successful_replacement`).
/// Verbatim de `panels.rs` (slice 19 F14.4.4).
pub(crate) fn apply_object_panel_edit_with_previous(
    document: &mut Document,
    id: ObjectId,
    changed: bool,
    edit: impl FnOnce(&mut GeoObject),
) -> Result<Option<Document>, String> {
    if !changed {
        return Ok(None);
    }
    let Some(object) = document.get_object(id) else {
        return Ok(None);
    };
    let mut edited = object.clone();
    edit(&mut edited);
    document.try_replace_object_with_previous(id, edited)
}

/// Setter del modo de coloración de dominio sobre `ComplexGrid`.
/// Verbatim de `panels.rs` (slice 20 F14.4.4): compara antes de tocar
/// (no-op si el modo ya es el pedido o el objeto no es `ComplexGrid`).
pub(crate) fn set_domain_coloring_mode(document: &mut Document, id: ObjectId, mode: u8) -> bool {
    let needs_update = matches!(
        document.get_object(id),
        Some(GeoObject::ComplexGrid(grid)) if grid.domain_coloring_mode != mode
    );
    if !needs_update {
        return false;
    }
    if let Some(GeoObject::ComplexGrid(grid)) = document.get_object_mut_touch(id) {
        grid.domain_coloring_mode = mode;
        true
    } else {
        false
    }
}

/// Setter de la animación de homotopía sobre `ComplexMapping`.
/// Verbatim de `panels.rs` (slice 20 F14.4.4): compara antes de tocar
/// (no-op si flags/velocidad ya coinciden o no es `ComplexMapping`).
pub(crate) fn set_complex_mapping_animation(
    document: &mut Document,
    id: ObjectId,
    animate_homotopy: bool,
    homotopy_speed: f32,
) -> bool {
    let needs_update = matches!(
        document.get_object(id),
        Some(GeoObject::ComplexMapping(mapping))
            if mapping.animate_homotopy != animate_homotopy
                || mapping.homotopy_speed != homotopy_speed
    );
    if !needs_update {
        return false;
    }
    if let Some(GeoObject::ComplexMapping(mapping)) = document.get_object_mut_touch(id) {
        mapping.animate_homotopy = animate_homotopy;
        mapping.homotopy_speed = homotopy_speed;
        true
    } else {
        false
    }
}

/// Columnas de la grilla de exportación según el ancho real: 3 en panel
/// normal, 1 bajo 200 px (responsive como la grilla canónica).
/// Verbatim de `draw_export_section` en `panels.rs` (slice 21 F14.4.4).
pub(crate) fn export_grid_columns(available: f32) -> usize {
    if available < 200.0 {
        1
    } else {
        3
    }
}

/// Ancho exacto de cada celda de la grilla de exportación para `columns`
/// columnas separadas por `SPACE_SM`: celdas + gaps nunca superan
/// `available`. Verbatim de `draw_export_section` en `panels.rs`
/// (slice 21 F14.4.4; misma forma que `trig_grid_cell_w` del slice 16).
pub(crate) fn export_grid_cell_w(available: f32, columns: usize) -> f32 {
    let cols = columns.max(1) as f32;
    ((available - (cols - 1.0) * SPACE_SM) / cols).max(0.0)
}

/// Texto de éxito al copiar el SVG real al portapapeles.
/// Verbatim de `draw_export_section` en `panels.rs` (slice 41 F14.4.4).
pub(crate) fn export_svg_copied_text(bytes: usize) -> String {
    format!("SVG copiado al portapapeles ({bytes} bytes)")
}

/// Texto de error honesto si el SVG no se pudo copiar.
/// Verbatim de `draw_export_section` en `panels.rs` (slice 41 F14.4.4).
pub(crate) fn export_svg_error_text(error: impl std::fmt::Display) -> String {
    format!("No se pudo copiar SVG: {error}")
}

/// Texto de éxito genérico al copiar MathML/TikZ-eje/HTML al portapapeles.
/// Verbatim de `draw_export_section` en `panels.rs` (slice 41 F14.4.4).
pub(crate) fn export_clipboard_copied_text(label: &str, bytes: usize) -> String {
    format!("{label} copiado al portapapeles ({bytes} bytes)")
}

/// Texto de éxito al copiar el SVG en el idioma pedido (B-A171). Es delega
/// al canónico [`export_svg_copied_text`] (paridad byte-idéntica, patrón
/// B-A98/B-A170); resto interpola el template `export.svg_copied` con
/// `{bytes}`. `bytes` es conteo, notación intacta.
pub(crate) fn export_svg_copied_text_for(bytes: usize, locale: Locale) -> String {
    use grafito_ui::i18n::t;
    if locale == Locale::Es {
        export_svg_copied_text(bytes)
    } else {
        t("export.svg_copied", locale).replace("{bytes}", &bytes.to_string())
    }
}

/// Texto de error honesto si el SVG no se pudo copiar, en el idioma pedido
/// (B-A171). Es delega al canónico [`export_svg_error_text`] (paridad
/// byte-idéntica, patrón B-A98/B-A170); resto interpola el template
/// `export.svg_error` con `{error}`. El detalle del error va crudo.
pub(crate) fn export_svg_error_text_for(error: impl std::fmt::Display, locale: Locale) -> String {
    use grafito_ui::i18n::t;
    if locale == Locale::Es {
        export_svg_error_text(error)
    } else {
        t("export.svg_error", locale).replace("{error}", &error.to_string())
    }
}

/// Texto de éxito genérico al copiar MathML/TikZ-eje/HTML en el idioma pedido
/// (B-A171). Es delega al canónico [`export_clipboard_copied_text`]
/// (paridad byte-idéntica, patrón B-A98/B-A170); resto interpola el template
/// `export.clipboard_copied` con `{label}`/`{bytes}`. El `label` es
/// identificador (`MathML`/`TikZ-eje`/`HTML`, precedente B-A169) y `bytes`
/// conteo: notación intacta.
pub(crate) fn export_clipboard_copied_text_for(
    label: &str,
    bytes: usize,
    locale: Locale,
) -> String {
    use grafito_ui::i18n::t;
    if locale == Locale::Es {
        export_clipboard_copied_text(label, bytes)
    } else {
        t("export.clipboard_copied", locale)
            .replace("{label}", label)
            .replace("{bytes}", &bytes.to_string())
    }
}

/// Rótulo de la fila de tabla CSV ("{nombre} · {n} filas").
/// Misma expresión que `sheet_table_subtitle` y que `draw_export_section`
/// en `panels.rs` (slice 41 F14.4.4; duplicado a propósito por dominio:
/// export vs hoja, igual que `export_grid_cell_w` duplica la forma de
/// `trig_grid_cell_w` de los slices 16/21).
pub(crate) fn export_table_row_label(name: &str, rows: usize) -> String {
    format!("{name} · {rows} filas")
}

/// Rótulo de la fila de tabla CSV en el idioma pedido (B-A170). Es delega
/// al canónico [`export_table_row_label`] (paridad byte-idéntica, patrón
/// B-A98/B-A150); resto interpola el template `export.table_row` con
/// `{name}`/`{rows}`. `·`/`filas` localizadas, notación intacta.
pub(crate) fn export_table_row_label_for(name: &str, rows: usize, locale: Locale) -> String {
    use grafito_ui::i18n::t;
    if locale == Locale::Es {
        export_table_row_label(name, rows)
    } else {
        t("export.table_row", locale)
            .replace("{name}", name)
            .replace("{rows}", &rows.to_string())
    }
}

/// Título y bajada de la sección Exportar.
/// Verbatim de `draw_export_section` en `panels.rs` (slice 46 F14.4.4).
pub(crate) const EXPORT_SECTION_TITLE: &str = "Exportación";
pub(crate) const EXPORT_SECTION_SUBTITLE: &str =
    "Exportá a archivo (SVG/PNG/TikZ/PDF) o copiá al portapapeles.";

/// Botones de archivo de la grilla 3×3 (rótulo + tooltip).
/// Verbatim de `draw_export_section` en `panels.rs` (slice 46 F14.4.4).
pub(crate) const EXPORT_FILE_BUTTONS: [(&str, &str); 6] = [
    ("SVG", "Exportar el lienzo a un archivo SVG vectorial"),
    (
        "PNG",
        "Exportar el lienzo a PNG (tiny-skia) donde elijas",
    ),
    ("TikZ", "Exportar el lienzo a código TikZ"),
    (
        "PDF",
        "Vectorial multipágina: una página por hoja de pizarra con contenido (rectas, círculos, polígonos y texto Helvetica), hasta 64 hojas.",
    ),
    (
        "Copiar SVG",
        "Copia el SVG real del lienzo al portapapeles",
    ),
    (
        "Copiar PNG",
        "Copia el PNG real del lienzo al portapapeles del sistema (para Word/Moodle)",
    ),
];

/// Botones de texto puro al portapapeles (rótulo + tooltip).
/// Verbatim de `draw_export_section` en `panels.rs` (slice 46 F14.4.4).
pub(crate) const EXPORT_TEXT_BUTTONS: [(&str, &str); 3] = [
    (
        "MathML",
        "Copia las funciones y puntos como MathML al portapapeles",
    ),
    (
        "TikZ-eje",
        "Copia un entorno pgfplots axis con tus funciones al portapapeles",
    ),
    (
        "HTML",
        "Copia una página autónoma con el SVG del lienzo al portapapeles",
    ),
];

/// Título de la lista de tablas CSV + botón CSV y su tooltip.
/// Verbatim de `draw_export_section` en `panels.rs` (slice 46 F14.4.4).
pub(crate) const EXPORT_TABLES_TITLE: &str = "Tablas (CSV RFC 4180)";
pub(crate) const EXPORT_CSV_LABEL: &str = "CSV";
pub(crate) const EXPORT_CSV_TOOLTIP: &str =
    "Descarga honesta: escribe el CSV real o informa el error";

/// Nombre por defecto del diálogo PDF + avisos de workers.
/// Verbatim de `draw_export_section` en `panels.rs` (slice 46 F14.4.4).
pub(crate) const EXPORT_PDF_DEFAULT_FILENAME: &str = "grafito_export.pdf";
pub(crate) const EXPORT_NOTIFY_PDF: &str = "Exportando PDF…";
pub(crate) const EXPORT_NOTIFY_PNG: &str = "Copiando PNG al portapapeles…";
pub(crate) const EXPORT_NOTIFY_CSV: &str = "Exportando CSV…";

/// Botón "Mostrar en carpeta" + su tooltip.
/// Verbatim de `draw_export_section` en `panels.rs` (slice 46 F14.4.4).
pub(crate) const EXPORT_REVEAL_LABEL: &str = "Mostrar en carpeta";
pub(crate) const EXPORT_REVEAL_TOOLTIP: &str = "Abre la carpeta de tu última exportación.";

/// Título + entrada del panel Complejos.
/// Verbatim de `draw_complex_panel` en `panels.rs` (slice 47 F14.4.4).
pub(crate) const COMPLEX_PANEL_TITLE: &str = "Números Complejos";
pub(crate) const COMPLEX_PANEL_INPUT_HINT: &str = "DomainColoring[1/z, -2, 2, -2, 2, 160]";
pub(crate) const COMPLEX_BASE_SYMBOL_TITLE: &str = "Símbolo base";
pub(crate) const COMPLEX_BASE_SYMBOL_HINT: &str = "z";

/// Contorno dibujado a mano: títulos, hints y rótulos de guías.
/// Verbatim de `draw_complex_panel` en `panels.rs` (slice 47 F14.4.4).
pub(crate) const COMPLEX_CONTOUR_TITLE: &str = "Integral de contorno";
pub(crate) const COMPLEX_CONTOUR_EXPR_HINT: &str = "f(z) — ej. 1/z";
pub(crate) const COMPLEX_CONTOUR_PREFILL: &str = "↧";
pub(crate) const COMPLEX_CONTOUR_PREFILL_TOOLTIP: &str = "Usar f(z) del objeto seleccionado";
pub(crate) const COMPLEX_CONTOUR_RESIDUES_LABEL: &str = "Residuos (Gauss)";
pub(crate) const COMPLEX_CONTOUR_SNAP_LABEL: &str = "Imán";
pub(crate) const COMPLEX_CONTOUR_CLOSE_LABEL: &str = "Cierre automático";
pub(crate) const COMPLEX_CONTOUR_SMOOTH_LABEL: &str = "Suavizar";
pub(crate) const COMPLEX_CONTOUR_STABILIZER_LABEL: &str = "Estabilizador";
pub(crate) const COMPLEX_CONTOUR_DRAW_LABEL: &str = "Dibujar contorno";
pub(crate) const COMPLEX_CONTOUR_DRAW_TOOLTIP: &str =
    "Dibujá con el mouse/stylus: el valor ∮ f(z) dz aparece en vivo; Esc cancela";
pub(crate) const COMPLEX_CONTOUR_CIRCLE_LABEL: &str = "Círculo";
pub(crate) const COMPLEX_CONTOUR_CIRCLE_TOOLTIP: &str =
    "Clic en el centro (con imán) y arrastrá el radio";

/// Texto del modo armado del contorno (misma rama que el dibujo).
/// Verbatim de `draw_complex_panel` en `panels.rs` (slice 47 F14.4.4).
pub(crate) fn complex_contour_armed_text(mode: ComplexContourMode) -> &'static str {
    match mode {
        ComplexContourMode::Freehand => "Dibujá sobre el lienzo · Esc para salir",
        ComplexContourMode::Circle => "Centro y radio sobre el lienzo · Esc para salir",
    }
}

/// Línea del último valor publicado del contorno.
/// Verbatim de `draw_complex_panel` en `panels.rs` (slice 47 F14.4.4).
pub(crate) fn complex_contour_last_value_text(value: &str) -> String {
    format!("Último valor: {value}")
}

/// Sección de objetos + comandos rápidos del panel Complejos.
/// Verbatim de `draw_complex_panel` en `panels.rs` (slice 47 F14.4.4).
pub(crate) const COMPLEX_OBJECTS_TITLE: &str = "Objetos";
pub(crate) const COMPLEX_OBJECTS_EMPTY: &str =
    "Sin objetos.\nProbá: x^2 + y^2 < 1\no: DomainColoring[1/z, -2, 2, -2, 2, 160]";
pub(crate) const COMPLEX_QUICK_TITLE: &str = "Comandos rápidos";
pub(crate) const COMPLEX_QUICK_HINTS: [(&str, &str); 5] = [
    (
        "Coloración de dominio: 1/z",
        "DomainColoring[1/z, -2, 2, -2, 2, 160]",
    ),
    (
        "Rejilla transformada: 1/z",
        "ComplexGrid[1/z, -2, 2, -2, 2, 16]",
    ),
    ("ComplexMapping[1/z, I]", "ComplexMapping[1/z, I]"),
    (
        "Coloración de dominio: exp(z)",
        "DomainColoring[exp(z), -2, 2, -2, 2, 160]",
    ),
    ("ComplexSymbol[w]", "ComplexSymbol[w]"),
];

/// Hover del atajo ejecutable ("Click para ejecutar: {payload}").
/// Verbatim de `draw_complex_panel` en `panels.rs` (slice 47 F14.4.4).
pub(crate) fn complex_quick_hover_text(payload: &str) -> String {
    format!("Click para ejecutar: {payload}")
}

/// Métodos de cuadratura del panel W-C integral (clave de comando + nombre
/// ES del combo). Verbatim de `panels.rs` (slice 22 F14.4.4).
pub(crate) const WC_QUADRATURE_METHODS: [(&str, &str); 3] = [
    ("midpoint", "Punto medio"),
    ("trapecio", "Trapecio"),
    ("simpson", "Simpson"),
];

/// Presupuesto UX del slider n (el comando banca hasta 1_000_000).
/// Verbatim de `panels.rs` (slice 22 F14.4.4).
pub(crate) const WC_RIEMANN_SLIDER_MAX: f32 = 10_000.0;

/// Estado efímero de las 3 secciones W-C (temp egui; `Clone` para el
/// get/insert de `IdTypeMap`, `Default` con ejemplos que evalúan).
/// Verbatim de `panels.rs` (slice 22 F14.4.4; el dibujo queda allá).
#[derive(Debug, Clone)]
pub(crate) struct WcVisualState {
    pub(crate) estudio_label: String,
    pub(crate) integral_expr: String,
    pub(crate) integral_a: String,
    pub(crate) integral_b: String,
    pub(crate) integral_n: f32,
    pub(crate) integral_metodo: usize,
    pub(crate) taylor_expr: String,
    pub(crate) taylor_centro: String,
    pub(crate) taylor_orden: i32,
    pub(crate) taylor_x: String,
}

impl Default for WcVisualState {
    fn default() -> Self {
        Self {
            estudio_label: "f".to_string(),
            integral_expr: "x^2".to_string(),
            integral_a: "0".to_string(),
            integral_b: "1".to_string(),
            integral_n: 100.0,
            integral_metodo: 0,
            taylor_expr: "sin(x)".to_string(),
            taylor_centro: "0".to_string(),
            taylor_orden: 5,
            taylor_x: "0.5".to_string(),
        }
    }
}

/// Método de cuadratura por índice con clamp (el combo escribe 0..=2 pero el
/// estado efímero puede venir rancio de una sesión previa): nunca paniquea,
/// a diferencia del indexado directo que hacía `panels.rs` en la rama
/// Simpson/Aproximar. Misma tabla que el dibujo.
pub(crate) fn wc_quadrature_method(idx: usize) -> (&'static str, &'static str) {
    WC_QUADRATURE_METHODS[idx.min(WC_QUADRATURE_METHODS.len() - 1)]
}

/// Clave de comando (`midpoint`/`trapecio`/`simpson`) por índice con clamp.
pub(crate) fn wc_quadrature_key(idx: usize) -> &'static str {
    wc_quadrature_method(idx).0
}

/// Estado del panel Capas (Q2): solo la capa elegida para asignar. La
/// tabla vive en el documento (`Document::layers`, persistente y con undo);
/// en temp egui queda únicamente el `selected_layer`. Las ids de documentos
/// cerrados se podan al dibujar vía [`Document::prune_layers`].
/// Verbatim de `panels.rs` (slice 23 F14.4.4; el dibujo queda allá).
#[derive(Debug, Clone, Default)]
pub(crate) struct LayerPanelState {
    pub(crate) selected_layer: u32,
}

/// Estado efímero del prompt auto trig (texto + último error honesto).
/// Verbatim de `panels.rs` (slice 23 F14.4.4; el dibujo
/// `draw_trig_speed_prompt` queda allá).
#[derive(Debug, Clone, Default)]
pub(crate) struct TrigPromptState {
    pub(crate) text: String,
    pub(crate) error: Option<String>,
}

/// Tabla de ejemplo de `draw_spreadsheet_section` (botón "Cargar ejemplo").
/// Verbatim de `panels.rs` (slice 30 F14.4.4; el commit con undo y los
/// `notify` quedan allá): x: 0..4, y: x², etiqueta "ejemplo".
pub(crate) fn example_sheet_table() -> GeoObject {
    GeoObject::DataTable(
        DataTableObj::new(
            "x",
            "y",
            vec![0.0, 1.0, 2.0, 3.0, 4.0],
            vec![0.0, 1.0, 4.0, 9.0, 16.0],
        )
        .with_label("ejemplo"),
    )
}

/// Texto de éxito al cargar la tabla de ejemplo (misma cadena que el dibujo).
pub(crate) fn example_sheet_table_loaded_text() -> &'static str {
    "Tabla «ejemplo» cargada"
}

/// Éxito en el idioma pedido (B-A152, display-localizado).
/// Es delega al canónico [`example_sheet_table_loaded_text`]
/// (`«ejemplo»` intacto, precedente B-A98); resto → `stats.sheet_example_loaded`.
pub(crate) fn example_sheet_table_loaded_text_for(locale: Locale) -> &'static str {
    use grafito_ui::i18n::Locale as L;
    if locale == L::Es {
        example_sheet_table_loaded_text()
    } else {
        grafito_ui::i18n::stats_msg("sheet_example_loaded", locale)
    }
}

/// Texto honesto al fallar la carga del ejemplo (misma cadena que el dibujo).
pub(crate) fn example_sheet_table_error_text(error: impl std::fmt::Display) -> String {
    format!("No se pudo cargar el ejemplo: {error}")
}

/// Error en el idioma pedido (B-A152, template `{error}`).
/// Es delega al canónico [`example_sheet_table_error_text`]; resto interpola
/// `stats.sheet_example_error`.
pub(crate) fn example_sheet_table_error_text_for(
    error: impl std::fmt::Display,
    locale: Locale,
) -> String {
    use grafito_ui::i18n::Locale as L;
    if locale == L::Es {
        example_sheet_table_error_text(error)
    } else {
        grafito_ui::i18n::stats_msg("sheet_example_error", locale)
            .replace("{error}", &error.to_string())
    }
}

/// Muestreo puro del plot continuo (`paint_continuous_density` en
/// `panels.rs`, slice 33 F14.4.4): 61 muestras equiespaciadas, `max(0.0)`
/// y filtro de no-finitos, `ymax` con piso 1e-12. Misma aritmética que el
/// dibujo, sin egui; el pintado queda en `panels.rs`.
pub(crate) const CONTINUOUS_PLOT_SAMPLES: usize = 61;
/// Tope de barras discretas del plot (misma guarda que el dibujo: 256).
pub(crate) const DISCRETE_PLOT_MAX_BARS: usize = 256;
/// Piso de `ymax` para no dividir por cero (mismo `1e-12` del dibujo).
pub(crate) const PLOT_YMAX_FLOOR: f64 = 1e-12;

pub(crate) fn continuous_density_samples(
    density: impl Fn(f64) -> f64,
    lo: f64,
    hi: f64,
) -> (Vec<(f64, f64)>, f64) {
    let mut pts: Vec<(f64, f64)> = Vec::with_capacity(CONTINUOUS_PLOT_SAMPLES);
    for i in 0..CONTINUOUS_PLOT_SAMPLES {
        let x = lo + (hi - lo) * i as f64 / (CONTINUOUS_PLOT_SAMPLES - 1) as f64;
        let y = density(x).max(0.0);
        if y.is_finite() {
            pts.push((x, y));
        }
    }
    let ymax = pts
        .iter()
        .map(|(_, y)| *y)
        .fold(0.0f64, f64::max)
        .max(PLOT_YMAX_FLOOR);
    (pts, ymax)
}

/// `ymax` discreto (mismo fold con piso 1e-12 que las 3 ramas del dibujo).
pub(crate) fn discrete_ymax(masses: &[(u64, f64)]) -> f64 {
    masses
        .iter()
        .map(|(_, y)| *y)
        .fold(0.0f64, f64::max)
        .max(PLOT_YMAX_FLOOR)
}

/// Masas Binomial(n, p) en [lo, hi] (misma `unwrap_or(0.0).max(0.0)` +
/// filtro finito + tope 256 que la rama 1 del dibujo).
pub(crate) fn binomial_plot_masses(n: u64, p: f64, lo: u64, hi: u64) -> Vec<(u64, f64)> {
    let mut masses: Vec<(u64, f64)> = Vec::new();
    for k in lo..=hi {
        let pmf = binomial_pmf(k, n, p).unwrap_or(0.0).max(0.0);
        if pmf.is_finite() {
            masses.push((k, pmf));
        }
        if masses.len() >= DISCRETE_PLOT_MAX_BARS {
            break;
        }
    }
    masses
}

/// Masas Poisson(λ) en [0, hi] (misma construcción que la rama 2).
pub(crate) fn poisson_plot_masses(lambda: f64, hi: u64) -> Vec<(u64, f64)> {
    (0..=hi)
        .filter_map(|k| {
            let pmf = poisson_pmf(k, lambda).unwrap_or(0.0).max(0.0);
            pmf.is_finite().then_some((k, pmf))
        })
        .collect()
}

/// Masas Geométrica(p) en [0, hi] (misma construcción que la rama 6).
pub(crate) fn geometric_plot_masses(prob: f64, hi: u64) -> Vec<(u64, f64)> {
    (0..=hi)
        .filter_map(|k| {
            let pmf = geometric_pmf(k, prob).unwrap_or(0.0).max(0.0);
            pmf.is_finite().then_some((k, pmf))
        })
        .collect()
}

/// Ancho máximo real de Datos/Prob (mismo que `draw_data_panel` +
/// `draw_statistics_panel` en `panels.rs`): evita que el 45% del viewport
/// en pantallas anchas estire el panel a 800+ px. 360 = múltiplo de base 4.
#[cfg(test)]
pub(crate) const PANEL_DATA_MAX_WIDTH: f32 = 360.0;

/// Clamp del ancho máximo Datos/Prob (misma fórmula que los dos
/// `SidePanel::left("data_panel"/"stats_panel")` del dibujo, sin egui).
#[cfg(test)]
pub(crate) fn panel_data_max_width_for(viewport_w: f32) -> f32 {
    (viewport_w * grafito_ui::tokens::PANEL_LEFT_MAX_FRACTION)
        .clamp(grafito_ui::tokens::PANEL_LEFT_MIN, PANEL_DATA_MAX_WIDTH)
}

/// Texto del aviso dentro del plot cuando no hay curva para dibujar
/// (misma cadena que `paint_plot_message` del dibujo, sin egui).
/// B-A157: variante localizada (template `{message}` via
/// `stats.prob_plot_prefix`; Es delega al canonico por construccion).
pub(crate) fn plot_message_text_for(message: &str, locale: grafito_ui::i18n::Locale) -> String {
    grafito_ui::i18n::t("stats.prob_plot_prefix", locale).replace("{message}", message)
}
/// Wrapper Es del aviso (solo tests; prod usa `_for` con `plot_locale`).
/// Precedente B-A143/B-A111: sin caller prod va con `#[cfg(test)]`.
#[cfg(test)]
pub(crate) fn plot_message_text(message: &str) -> String {
    plot_message_text_for(message, grafito_ui::i18n::Locale::Es)
}

/// Textos puros de la sección Regresión de `draw_right_regression_panel`
/// en `panels.rs` (slice 36): mismas cadenas que el dibujo, sin egui.
/// El dibujo conserva el `SidePanel`, los `RichText`/colores, el `collapsing`
/// y los call sites con undo; acá solo vive el texto que se pinea.
pub(crate) const REGRESSION_DELETED_SOURCE: &str = "tabla eliminada";

/// Tope de residuales pintados (misma `min(24)` que el dibujo).
pub(crate) const REGRESSION_MAX_RESIDUALS_SHOWN: usize = 24;

/// Plantillas de `Crear análisis` (mismas 3 que el `const TEMPLATES` inline
/// del dibujo: nombre para el botón + comando para la entrada).
pub(crate) const REGRESSION_TEMPLATES: [(&str, &str); 3] = [
    ("Dispersión", "ScatterPlot[{1, 2, 3}, {1, 4, 9}]"),
    ("Regresión", "LinearRegression[{1, 2, 3}, {1, 4, 9}]"),
    ("Tabla local", "DataTable[{0, 1, 2}, {1, 3, 5}]"),
];

/// Nombres display de las 3 plantillas (B-A148, canónicos ES para la rama
/// Es del dispatcher; resto → `stats_msg`).
pub(crate) const REGRESSION_TEMPLATE_SCATTER_NAME: &str = "Dispersión";
pub(crate) const REGRESSION_TEMPLATE_REGRESSION_NAME: &str = "Regresión";
pub(crate) const REGRESSION_TEMPLATE_LOCAL_TABLE_NAME: &str = "Tabla local";

/// Título/subtítulo de la sección de import de `draw_spreadsheet_section`
/// (B-A148, canónicos ES; resto → `stats_msg`). El botón Importar ya
/// reutiliza `regression_import_label/hover` (B-A147).
pub(crate) const SPREADSHEET_IMPORT_TITLE: &str = "Importar y ordenar";
pub(crate) const SPREADSHEET_IMPORT_SUBTITLE: &str = "Máx. 2 MB.";

/// Etiqueta de origen del ajuste: la del objeto o la caída honesta cuando
/// la tabla fue eliminada (mismo `unwrap_or_else` que el dibujo).
pub(crate) fn regression_source_label(label: Option<&str>) -> String {
    label.unwrap_or(REGRESSION_DELETED_SOURCE).to_string()
}

/// Titular del ajuste (misma `"{label} · {kind}"` que el dibujo; el caller
/// pasa ya computado `fit.kind.display_name()`).
pub(crate) fn regression_fit_headline(label: &str, kind_display: &str) -> String {
    format!("{label} · {kind_display}")
}

/// Línea de diagnósticos (misma `"sobre {origen} · RMSE {:.6} · R² {:.6}"`).
pub(crate) fn regression_fit_diagnostics_line(
    source_label: &str,
    rmse: f64,
    r_squared: f64,
) -> String {
    format!("sobre {source_label} · RMSE {rmse:.6} · R² {r_squared:.6}")
}

/// Título del desplegable de residuales (mismo `"Residuales ({n})"`).
pub(crate) fn regression_residuals_title(count: usize) -> String {
    format!("Residuales ({count})")
}

/// Fila de residual (misma `"r{i} = {v:.6}"` con índice 1-based).
pub(crate) fn regression_residual_line(index0: usize, residual: f64) -> String {
    format!("r{} = {residual:.6}", index0 + 1)
}

/// Nota cuando se recorta la lista (misma cadena que el dibujo).
pub(crate) fn regression_residuals_truncated_note() -> &'static str {
    "Se muestran los primeros 24 valores."
}

/// Título de la sección de modelos (mismo `"Ajustar «{tabla}»"`).
pub(crate) fn regression_section_title(table_label: &str) -> String {
    format!("Ajustar «{table_label}»")
}

/// Texto de éxito al mandar una plantilla a la entrada (misma cadena).
pub(crate) fn regression_template_ready_text(name: &str) -> String {
    format!("{name} listo en la entrada (Enter para crear).")
}

/// Texto de éxito al mandar un modelo a la entrada (misma cadena).
pub(crate) fn regression_model_ready_text(model: &str) -> String {
    format!("Ajuste {model} listo en la entrada (Enter para crear).")
}

// --- extraído verbatim de panels.rs:5933-5940 + 5956-5958 + 5972-5974 + 6013-6017 (slice 39 F14.4.4) ---
/// Texto de una cola de `draw_probability_section`: `"{valor:.6}"` o `"—"`
/// si el CDF no es finito (misma caída honesta que el closure inline).
pub(crate) fn prob_tail_text(
    tails: Option<(f64, f64, f64)>,
    pick: fn((f64, f64, f64)) -> f64,
) -> String {
    tails
        .map(pick)
        .map(|value| format!("{value:.6}"))
        .unwrap_or_else(|| "—".to_string())
}

/// Texto de un momento de `draw_probability_section`: `format_statistic` o
/// `"—"` si no está definido (misma caída que el closure inline).
pub(crate) fn prob_moment_text(value: Option<f64>) -> String {
    value
        .map(format_statistic)
        .unwrap_or_else(|| "—".to_string())
}

/// Texto que el botón "Copiar" del caso pone en el portapapeles (misma
/// interpolación que `draw_probability_section`, sin egui).
pub(crate) fn prob_case_copy_text(
    detail: &str,
    point_label: &str,
    pdf: f64,
    cdf: f64,
    tail_right: &str,
) -> String {
    format!("{detail} · {point_label} = {pdf:.6} · Acumulada = {cdf:.6} · P(X>x) = {tail_right}")
}

/// Tooltip del detalle del caso (misma cadena que el dibujo, sin egui).
pub(crate) fn prob_case_detail_tooltip(detail: &str) -> String {
    format!("{detail}\nCaso evaluado en vivo con este motor")
}

// --- extraído verbatim de panels.rs:1125-1130 + 1143 + 1224-1238 + 1248-1251 (slice 43 F14.4.4) ---
/// Acciones rápidas del panel CAS: mismas 4 tuplas rótulo/snippet que
/// `draw_cas_panel`, sin egui.
pub(crate) const CAS_QUICK_ACTIONS: [(&str, &str); 4] = [
    ("Derivar", "Derivative["),
    ("Integrar", "Integral["),
    ("Resolver", "Solve["),
    ("Límite", "Limit["),
];

/// Tooltip del botón rápido: misma interpolación `"Insertar {snippet}"`
/// que `draw_cas_panel`, sin egui.
pub(crate) fn cas_quick_insert_tooltip(snippet: &str) -> String {
    format!("Insertar {snippet}")
}

/// Título del empty-state de la hoja CAS (misma cadena que el dibujo).
pub(crate) const CAS_EMPTY_TITLE: &str = "Sin cálculos aún";
/// Hint del empty-state de la hoja CAS (misma cadena que el dibujo).
pub(crate) const CAS_EMPTY_HINT: &str = "Escribe arriba y pulsa Enter.";
/// Ejemplo del empty-state de la hoja CAS (misma cadena que el dibujo).
pub(crate) const CAS_EMPTY_EXAMPLE: &str = "Ej: Derivative[x^2, x]  ·  Solve[x^2-4, x]";

/// Título de la hoja con N celdas: misma cadena `"Hoja · {n}"` que
/// `draw_cas_panel`, sin egui.
pub(crate) fn cas_sheet_title(count: usize) -> String {
    format!("Hoja · {count}")
}

// --- extraído verbatim de panels.rs:1089 + 1097-1098 + 1160 + 1177 + 1182 + 1197 + 1262 + 1269 (slice 44 F14.4.4) ---
/// Título del panel CAS (misma cadena que `draw_cas_panel`, sin egui).
pub(crate) const CAS_PANEL_TITLE: &str = "Cálculo Simbólico (CAS)";
/// Rótulo del botón que vacía la hoja CAS (misma cadena que el dibujo).
pub(crate) const CAS_CLEAR_LABEL: &str = "Limpiar";
/// Hover del botón que vacía la hoja CAS (misma cadena que el dibujo).
pub(crate) const CAS_CLEAR_TOOLTIP: &str = "Eliminar las celdas CAS guardadas";
/// Rótulo de la tarjeta de entrada CAS (misma cadena que el dibujo).
pub(crate) const CAS_ENTRY_LABEL: &str = "Entrada";
/// Placeholder de la entrada CAS (misma cadena que el dibujo).
pub(crate) const CAS_ENTRY_HINT: &str = "x^2, x";
/// Tooltip del botón ejecutar CAS (misma cadena que el dibujo).
pub(crate) const CAS_EXECUTE_TOOLTIP: &str = "Ejecutar";
/// Hint de teclas de la tarjeta de entrada CAS (misma cadena que el dibujo).
pub(crate) const CAS_ENTRY_KEYS_HINT: &str = "↵  •  Tab";
/// Número de la celda i-ésima: mismo `format!("{}", i + 1)` que el dibujo.
pub(crate) fn cas_cell_number(index: usize) -> String {
    format!("{}", index + 1)
}
/// Línea de entrada de la celda: mismo `format!("> {input}")` que el dibujo.
pub(crate) fn cas_cell_input_line(input: &str) -> String {
    format!("> {input}")
}

/// Chrome estático del panel CAS (B-A177). Es delega a las consts canónicas
/// (paridad byte-idéntica, patrón B-A98/B-A128); resto va a `cas_msg`.
/// `CAS_ENTRY_HINT` "x^2, x" + `CAS_ENTRY_KEYS_HINT` "↵  •  Tab" +
/// `cas_cell_number`/`cas_cell_input_line` + snippets `Derivative[/Integral[/
/// `Solve[/Limit[` invariantes (notación/sintaxis/comandos, B-A98/B-A114).
pub(crate) fn cas_chrome_for(suffix: &'static str, locale: Locale) -> &'static str {
    use grafito_ui::i18n::cas_msg;
    if locale == Locale::Es {
        match suffix {
            "panel_title" => CAS_PANEL_TITLE,
            "clear_label" => CAS_CLEAR_LABEL,
            "clear_tooltip" => CAS_CLEAR_TOOLTIP,
            "entry_label" => CAS_ENTRY_LABEL,
            "execute_tooltip" => CAS_EXECUTE_TOOLTIP,
            "quick_section_title" => CAS_QUICK_SECTION_TITLE,
            "quick_section_subtitle" => CAS_QUICK_SECTION_SUBTITLE,
            "quick_derive" => CAS_QUICK_ACTIONS[0].0,
            "quick_integrate" => CAS_QUICK_ACTIONS[1].0,
            "quick_solve" => CAS_QUICK_ACTIONS[2].0,
            "quick_limit" => CAS_QUICK_ACTIONS[3].0,
            "empty_title" => CAS_EMPTY_TITLE,
            "empty_hint" => CAS_EMPTY_HINT,
            "empty_example" => CAS_EMPTY_EXAMPLE,
            _ => suffix,
        }
    } else {
        cas_msg(suffix, locale)
    }
}

/// Tooltip "Insertar {snippet}" localizado (B-A177). Es delega al canónico
/// `cas_quick_insert_tooltip`; resto interpola `cas.quick_insert_tooltip`.
/// El `snippet` (`Derivative[`…) es sintaxis invariante (B-A98).
pub(crate) fn cas_quick_insert_tooltip_for(snippet: &str, locale: Locale) -> String {
    if locale == Locale::Es {
        cas_quick_insert_tooltip(snippet)
    } else {
        grafito_ui::i18n::cas_msg("quick_insert_tooltip", locale).replace("{snippet}", snippet)
    }
}

/// Título "Hoja · {count}" localizado (B-A177). Es delega al canónico
/// `cas_sheet_title`; resto interpola `cas.sheet_title`.
pub(crate) fn cas_sheet_title_for(count: usize, locale: Locale) -> String {
    if locale == Locale::Es {
        cas_sheet_title(count)
    } else {
        grafito_ui::i18n::cas_msg("sheet_title", locale).replace("{count}", &count.to_string())
    }
}

/// Rótulo del i-ésimo botón rápido CAS localizado (B-A177). Es delega a
/// `CAS_QUICK_ACTIONS[i].0` vía el dispatcher; resto al `cas_msg`.
/// Índice fuera de rango → "" (el draw itera 0..4, inalcanzable).
pub(crate) fn cas_quick_label_for(index: usize, locale: Locale) -> &'static str {
    const SUFFIXES: [&str; 4] = [
        "quick_derive",
        "quick_integrate",
        "quick_solve",
        "quick_limit",
    ];
    match SUFFIXES.get(index) {
        Some(suffix) => cas_chrome_for(suffix, locale),
        None => "",
    }
}

// --- extraído verbatim de panels.rs:1123-1124 (slice 45 F14.4.4) ---
/// Título de la sección de acciones rápidas CAS (misma cadena que el dibujo).
pub(crate) const CAS_QUICK_SECTION_TITLE: &str = "Acciones rápidas";
/// Bajada de la sección de acciones rápidas CAS (misma cadena que el dibujo).
pub(crate) const CAS_QUICK_SECTION_SUBTITLE: &str = "Atajos — inserta sintaxis mínima.";

// --- extraído verbatim de panels.rs:3303-3369 (slice 48 F14.4.4) ---
/// Título + sección de objetos del panel Atractores (mismas cadenas que el dibujo).
pub(crate) const ATTRACTOR_PANEL_TITLE: &str = "Dinámica y Atractores";
pub(crate) const ATTRACTOR_OBJECTS_TITLE: &str = "Objetos dinámicos";
pub(crate) const ATTRACTOR_OBJECTS_EMPTY: &str =
    "Sin objetos dinámicos.\nProbá Attractor[10, 28, 8/3].";
/// Título de la tarjeta del attractor activo (misma cadena que el dibujo).
pub(crate) const ATTRACTOR_ACTIVE_TITLE: &str = "Attractor activo";
/// Líneas de parámetros del attractor activo (mismos `format!` que el dibujo).
pub(crate) fn attractor_sigma_text(sigma: f64) -> String {
    format!("sigma = {sigma:.3}")
}
pub(crate) fn attractor_rho_text(rho: f64) -> String {
    format!("rho = {rho:.3}")
}
pub(crate) fn attractor_beta_text(beta: f64) -> String {
    format!("beta = {beta:.3}")
}
pub(crate) fn attractor_dt_text(dt: f64) -> String {
    format!("dt = {dt:.4}")
}
pub(crate) fn attractor_steps_text(steps: usize) -> String {
    format!("pasos = {steps}")
}
/// Guía sin attractor + botón de creación por defecto (mismas cadenas que el dibujo).
pub(crate) const ATTRACTOR_INACTIVE_GUIDE: &str =
    "Sin attractor activo.\nCreá uno con:\n  Attractor[σ, ρ, β]\n(Lorenz por defecto)";
pub(crate) const ATTRACTOR_CREATE_DEFAULT_LABEL: &str = "Crear Lorenz por defecto";
pub(crate) const ATTRACTOR_DEFAULT_COMMAND: &str = "Attractor[10, 28, 8/3]";
/// Sección de comandos del panel Atractores (mismas cadenas que el dibujo).
pub(crate) const ATTRACTOR_COMMANDS_TITLE: &str = "Comandos";
pub(crate) const ATTRACTOR_LORENZ_HINT: &str = "- Lorenz: Attractor[sigma, rho, beta]";

// --- extraído verbatim de panels.rs:4507-4526 + 4546 + 4575 + 4597 + 4621 (slice 50 F14.4.4) ---
/// Textos puros del panel Parámetros dinámicos (mismas cadenas que
/// `draw_right_parameters_panel`, sin egui; los `SidePanel`/sliders/botones
/// y el commit con undo quedan en `panels.rs`).
pub(crate) const PARAMS_PANEL_TITLE: &str = "Parámetros dinámicos";
pub(crate) const PARAMS_PANEL_SUBTITLE: &str = "Lorenz y sistemas vivos.";
pub(crate) const PARAMS_EMPTY_GUIDE: &str =
    "Sin attractor activo. Creá uno con:\n  Attractor[10, 28, 8/3]";
pub(crate) const PARAMS_LORENZ_SECTION: &str = "Lorenz sigma, rho, beta";
pub(crate) const PARAMS_INTEGRATION_SECTION: &str = "Integración";
pub(crate) const PARAMS_REGEN_HINT: &str = "El canvas se regenera cada cambio.";
/// Rótulo del slider de pasos del panel Parámetros (misma cadena que el dibujo).
pub(crate) const PARAMS_STEPS_LABEL: &str = "pasos";

/// Línea de error honesto del commit de parámetros.
/// Verbatim de `panels.rs` (`format!("Parámetros: {error}")`).
pub(crate) fn params_error_text(error: impl std::fmt::Display) -> String {
    format!("Parámetros: {error}")
}

/// Línea de error del commit de parámetros por locale (B-A190).
/// Es delega al canónico `params_error_text` (paridad byte-idéntica,
/// patrón B-A98); resto interpola `dynamics.params_error` (`{error}`).
pub(crate) fn params_error_text_for(error: impl std::fmt::Display, locale: Locale) -> String {
    if locale == Locale::Es {
        params_error_text(error)
    } else {
        grafito_ui::i18n::t("dynamics.params_error", locale).replace("{error}", &error.to_string())
    }
}

/// Chrome estático de Atractores + Parámetros (B-A189 + B-A190). Es delega a las
/// consts canónicas (paridad byte-idéntica, patrón B-A98/B-A128/B-A177);
/// resto va a `dynamics_msg`. `ATTRACTOR_DEFAULT_COMMAND`
/// "Attractor[10, 28, 8/3]" invariante (sintaxis de comando, B-A98/B-A148) +
/// `attractor_sigma/rho/beta/dt/steps_text` "sigma = …" invariantes
/// (lecturas con notación, B-A98/B-A162) + sliders "σ"/"ρ"/"β"/"dt"
/// invariantes (notación SI, B-A114) + `params_error_text` canónico Es
/// intacto (wrapper de paridad; el draw usa `params_error_text_for`,
/// B-A190).
pub(crate) fn dynamics_chrome_for(suffix: &'static str, locale: Locale) -> &'static str {
    use grafito_ui::i18n::dynamics_msg;
    if locale == Locale::Es {
        match suffix {
            "attractor_panel_title" => ATTRACTOR_PANEL_TITLE,
            "attractor_objects_title" => ATTRACTOR_OBJECTS_TITLE,
            "attractor_objects_empty" => ATTRACTOR_OBJECTS_EMPTY,
            "attractor_active_title" => ATTRACTOR_ACTIVE_TITLE,
            "attractor_inactive_guide" => ATTRACTOR_INACTIVE_GUIDE,
            "attractor_create_default" => ATTRACTOR_CREATE_DEFAULT_LABEL,
            "attractor_commands_title" => ATTRACTOR_COMMANDS_TITLE,
            "attractor_lorenz_hint" => ATTRACTOR_LORENZ_HINT,
            "params_panel_title" => PARAMS_PANEL_TITLE,
            "params_panel_subtitle" => PARAMS_PANEL_SUBTITLE,
            "params_empty_guide" => PARAMS_EMPTY_GUIDE,
            "params_lorenz_section" => PARAMS_LORENZ_SECTION,
            "params_integration_section" => PARAMS_INTEGRATION_SECTION,
            "params_regen_hint" => PARAMS_REGEN_HINT,
            "params_steps_label" => PARAMS_STEPS_LABEL,
            _ => suffix,
        }
    } else {
        dynamics_msg(suffix, locale)
    }
}

// --- extraído verbatim de panels.rs:4360 + 4373-4399 + 4409-4424 + 4431 + 4436 (slice 51 F14.4.4) ---
/// Textos puros del panel Coloración de dominio (mismas cadenas que
/// `draw_right_domain_coloring_panel`, sin egui; el `SidePanel`, los combos,
/// el `TextEdit` del símbolo y el drawer quedan en `panels.rs`).
pub(crate) const DOMAIN_COLORING_TITLE: &str = "Coloración de dominio";
pub(crate) const DOMAIN_COLORING_SUBTITLE: &str = "Fase y módulo en el plano.";
pub(crate) const DOMAIN_COLORING_EMPTY_GUIDE: &str =
    "Sin coloración de dominio. Creá una con:\n  DomainColoring[1/z, -2, 2, -2, 2, 160]\nFase y módulo de f(z).";
pub(crate) const DOMAIN_COLORING_ACTIVE_TITLE: &str = "Coloración por fase habilitada";
pub(crate) const DOMAIN_COLORING_HUE_LINE: &str = "Tono = arg(f(z)).";
pub(crate) const DOMAIN_COLORING_GUIDE_TITLE: &str = "Guia de Interpretacion";
pub(crate) const DOMAIN_COLORING_COLORS_TITLE: &str = "Colores y Magnitud:";
pub(crate) const DOMAIN_COLORING_COLORS_BODY: &str =
    "- Tono: Fase o angulo arg(f(z)).\n- Brillo: Magnitud |f(z)|. Negro = Raiz (0), Blanco = Polo (inf).";
pub(crate) const DOMAIN_COLORING_DERIV_TITLE: &str = "Derivabilidad y Wirtinger:";
pub(crate) const DOMAIN_COLORING_DERIV_BODY: &str =
    "- Al graficar deriv_z_conj(f), f(z) es holomorfa solo en zonas negras (donde d/dzbar = 0, Cauchy-Riemann).\n- Las zonas coloreadas representan donde NO es derivable.";
pub(crate) const DOMAIN_COLORING_MODE_LABEL: &str = "Modo de coloración";
pub(crate) const DOMAIN_COLORING_SYMBOL_LABEL: &str = "Símbolo base";
pub(crate) const DOMAIN_COLORING_SYMBOL_HINT: &str = "z";

/// Rótulo del modo de coloración de dominio (misma tabla que el
/// `ComboBox` de `draw_right_domain_coloring_panel`).
pub(crate) fn domain_coloring_mode_label(mode: u8) -> &'static str {
    match mode {
        0 => "HSL Clásico (Fase + Módulo)",
        1 => "Retrato de Fase Puro",
        2 => "Rejilla Polar Conforme",
        3 => "Rejilla Cartesiana Conforme",
        _ => "HSL Clásico",
    }
}

// --- extraído verbatim de panels.rs:2452 + 2458 (slice 52 F14.4.4) ---
/// Textos puros del panel vacío (mismas cadenas que `draw_empty_panel`,
/// sin egui; el `SidePanel` centrado queda en `panels.rs`).
pub(crate) const EMPTY_PANEL_TITLE: &str = "Sin panel aquí";
pub(crate) const EMPTY_PANEL_GUIDE: &str = "Cambiá de perspectiva o abrí un panel desde «Paneles».";

// --- extraído verbatim de panels.rs:1679+1703+1718+1730-1733+1753-1754+1795-1798+1808+1814-1815+1828+1831-1832 (slice 54 F14.4.4) ---
/// Textos puros Vista-3D + parámetro vivo de `draw_view_panel` (mismas
/// cadenas que el dibujo, sin egui; el `SidePanel`/combo/slider/undo
/// quedan en `panels.rs`).
pub(crate) const VIEW_3D_SECTION_TITLE: &str = "Vista 3D";
pub(crate) const VIEW_3D_ORTHO_HOVER: &str = "En ortográficas el zoom y el paneo se conservan; orbitar con botón derecho vuelve a perspectiva.";
pub(crate) const LIVE_PARAM_SECTION_TITLE: &str = "Parámetro vivo";
pub(crate) const LIVE_PARAM_MISSING_GUIDE: &str =
    "Todavía no hay parámetro `p`: las funciones con `p` usan su valor por defecto.";
pub(crate) const LIVE_PARAM_CREATE_LABEL: &str = "Crear parámetro p en [-5, 5]";
pub(crate) const LIVE_PARAM_CREATE_HOVER: &str = "Crea la variable `p` con slider";
pub(crate) const LIVE_PARAM_CREATED: &str = "Parámetro `p` creado en [-5, 5]";

/// Hover del slider vivo (misma cadena que `draw_view_panel`).
pub(crate) fn live_param_hover_text(value: f64, min: f64, max: f64) -> String {
    format!("p = {value:.3} en [{min:.3}, {max:.3}] · las funciones con `p` se re-muestrean solas")
}

/// Lectura bajo el slider vivo (misma cadena que `draw_view_panel`).
pub(crate) fn live_param_readout_text(value: f64, min: f64, max: f64) -> String {
    format!("p = {value:.3} en [{min:.3}, {max:.3}]")
}

/// Error honesto del commit del parámetro vivo (mismo prefijo que el dibujo).
pub(crate) fn live_param_error_text(error: impl std::fmt::Display) -> String {
    format!("Parámetro vivo: {error}")
}

/// Hover del slider vivo en el idioma activo (B-A131): Es delega a la
/// canónica `live_param_hover_text`, resto al template `view.live_hover`
/// (`{value}`/`{min}`/`{max}` con 3 decimales como el dibujo; patrón
/// `layer_row_label_for` B-A130).
pub(crate) fn live_param_hover_text_for(
    value: f64,
    min: f64,
    max: f64,
    locale: grafito_ui::i18n::Locale,
) -> String {
    use grafito_ui::i18n::Locale;
    if locale == Locale::Es {
        live_param_hover_text(value, min, max)
    } else {
        grafito_ui::i18n::view_msg("live_hover", locale)
            .replacen("{value}", &format!("{value:.3}"), 1)
            .replacen("{min}", &format!("{min:.3}"), 1)
            .replacen("{max}", &format!("{max:.3}"), 1)
    }
}

/// Lectura bajo el slider vivo en el idioma activo (B-A131): Es delega a
/// la canónica `live_param_readout_text`, resto al template
/// `view.live_readout` (`{value}`/`{min}`/`{max}`).
pub(crate) fn live_param_readout_text_for(
    value: f64,
    min: f64,
    max: f64,
    locale: grafito_ui::i18n::Locale,
) -> String {
    use grafito_ui::i18n::Locale;
    if locale == Locale::Es {
        live_param_readout_text(value, min, max)
    } else {
        grafito_ui::i18n::view_msg("live_readout", locale)
            .replacen("{value}", &format!("{value:.3}"), 1)
            .replacen("{min}", &format!("{min:.3}"), 1)
            .replacen("{max}", &format!("{max:.3}"), 1)
    }
}

/// Error honesto del commit del parámetro vivo en el idioma activo
/// (B-A131): Es delega a la canónica `live_param_error_text`, resto al
/// template `view.live_error` (`{error}`; patrón `view_indicate_error_for`).
pub(crate) fn live_param_error_text_for(
    error: impl std::fmt::Display,
    locale: grafito_ui::i18n::Locale,
) -> String {
    use grafito_ui::i18n::Locale;
    if locale == Locale::Es {
        return live_param_error_text(error);
    }
    grafito_ui::i18n::view_msg("live_error", locale).replacen("{error}", &error.to_string(), 1)
}

// --- extraído verbatim de panels.rs:1861+1889-1890+1937+1955+2001+2014+2020+2027+2031-2033+2050+2058-2060 (slice 55 F14.4.4) ---
/// Textos puros Capas/Indicar/Alta-precisión de `draw_view_panel` (mismas
/// cadenas que el dibujo, sin egui; el `SidePanel`/botones/sliders/undo
/// quedan en `panels.rs`).
pub(crate) const VIEW_LAYERS_TITLE: &str = "Capas";
pub(crate) const LAYER_TOGGLE_SHOW: &str = "Mostrar";
pub(crate) const LAYER_TOGGLE_HIDE: &str = "Ocultar";
pub(crate) const VIEW_ASSIGN_LAYER_LABEL: &str = "Asignar a capa:";
pub(crate) const VIEW_ASSIGN_LABEL: &str = "Asignar";
pub(crate) const VIEW_HIGHLIGHT_LABEL: &str = "Resaltar:";
pub(crate) const VIEW_INDICATE_LABEL: &str = "Indicar";
pub(crate) const VIEW_INDICATE_HOVER: &str = "Hace latir la selección ~1.2 s";
pub(crate) const VIEW_INDICATING: &str = "Indicando selección (~1.2 s)";
pub(crate) const VIEW_HIGH_PRECISION_TITLE: &str = "Alta Precisión";
pub(crate) const VIEW_HIGH_PRECISION_LABEL: &str = "Alta Precisión (Double-Double)";
pub(crate) const VIEW_HIGH_PRECISION_HINT: &str = "Usa aritmética Double-Double (~106 bits / 32 dígitos) para evaluar expresiones simbólicas sin pérdida de precisión.";

/// Rótulo del toggle de visibilidad (misma ternaria que `draw_view_panel`).
pub(crate) fn layer_toggle_label(visible: bool) -> &'static str {
    if visible {
        LAYER_TOGGLE_HIDE
    } else {
        LAYER_TOGGLE_SHOW
    }
}

/// Rótulo del toggle de visibilidad en el idioma activo (B-A129): Es delega
/// a la canónica `layer_toggle_label`, resto a `view_msg` (ternaria
/// Mostrar/Ocultar localizada; el draw ya no usa la canónica directa).
pub(crate) fn layer_toggle_label_for(visible: bool, locale: Locale) -> &'static str {
    if locale == Locale::Es {
        layer_toggle_label(visible)
    } else if visible {
        grafito_ui::i18n::view_msg("layer_hide", locale)
    } else {
        grafito_ui::i18n::view_msg("layer_show", locale)
    }
}

/// Error honesto del pulso de indicación (mismo formato que el dibujo).
pub(crate) fn view_indicate_error(error: impl std::fmt::Display) -> String {
    format!("No se pudo indicar: {error}")
}

/// Error en el idioma activo (B-A130): Es delega a la canónica, resto al
/// template `view.indicate_err` (`{error}`; patrón `layer_assign_error_for`).
pub(crate) fn view_indicate_error_for(error: impl std::fmt::Display, locale: Locale) -> String {
    if locale == Locale::Es {
        return view_indicate_error(error);
    }
    grafito_ui::i18n::view_msg("indicate_err", locale).replacen("{error}", &error.to_string(), 1)
}

// --- extraído verbatim de panels.rs:4865+4893+4899-4903+4921+4927+4933+4981+4994+5004+5009+5033+5041+5049-5052+5116+5128 (slice 56 F14.4.4) ---
/// Textos puros del protocolo de construcción (`draw_construction_protocol`:
/// mismas cadenas que el dibujo, sin egui; el `SidePanel`/botones/nav/drawer
/// quedan en `panels.rs`; la máquina de pasos ya vive acá desde el slice 18).
pub(crate) const PROTOCOL_PANEL_TITLE: &str = "Protocolo de Construcción";
pub(crate) const PROTOCOL_PANEL_SUBTITLE: &str = "Pasos de construcción.";
pub(crate) const PROTOCOL_EXPORT_LABEL: &str = "Exportar LaTeX";
pub(crate) const PROTOCOL_EXPORT_HOVER_READY: &str = "Guarda el protocolo como archivo .tex";
pub(crate) const PROTOCOL_EXPORTING: &str = "Exportando protocolo a LaTeX…";
pub(crate) const PROTOCOL_CLEAR_LABEL: &str = "Limpiar";
pub(crate) const PROTOCOL_CLEAR_HOVER: &str = "Vacía el historial de pasos";
pub(crate) const PROTOCOL_PREV_GLYPH: &str = "◀";
pub(crate) const PROTOCOL_PREV_TIP: &str = "Paso anterior";
pub(crate) const PROTOCOL_PLAY_GLYPH: &str = "▶";
pub(crate) const PROTOCOL_PAUSE_GLYPH: &str = "⏸";
pub(crate) const PROTOCOL_PLAY_TIP: &str = "Reproducir el protocolo paso a paso";
pub(crate) const PROTOCOL_LAST_GLYPH: &str = "▶|";
pub(crate) const PROTOCOL_LAST_TIP: &str = "Último paso";
pub(crate) const PROTOCOL_EMPTY_TITLE: &str = "Sin pasos todavía";
pub(crate) const PROTOCOL_EMPTY_GUIDE: &str = "Creá objetos o restricciones para verlos acá.";
pub(crate) const PROTOCOL_STEPS_HINT: &str =
    "Vista fiel del historial; no reordena ni desactiva restricciones reales.";
pub(crate) const PROTOCOL_DISABLED_LABEL: &str = "deshabilitado";
pub(crate) const PROTOCOL_EMPTY_CELL: &str = "—";

/// Hover del botón exportar (misma ternaria que `draw_construction_protocol`).
pub(crate) fn protocol_export_hover(has_steps: bool) -> &'static str {
    if has_steps {
        PROTOCOL_EXPORT_HOVER_READY
    } else {
        PROTOCOL_EMPTY_TITLE
    }
}

/// Glifo play/pausa (misma ternaria que la fila de navegación).
pub(crate) fn protocol_play_glyph(playing: bool) -> &'static str {
    if playing {
        PROTOCOL_PAUSE_GLYPH
    } else {
        PROTOCOL_PLAY_GLYPH
    }
}

/// Título de la sección de pasos (mismo formato que el dibujo).
pub(crate) fn protocol_steps_title(total: usize) -> String {
    format!("Pasos ({total})")
}

/// Contador de la fila de navegación (mismo formato que el dibujo).
pub(crate) fn protocol_counter_text(current: usize, total: usize) -> String {
    format!("{current}/{total}")
}

/// Insignia del número de paso (mismo formato que la tarjeta).
pub(crate) fn protocol_step_badge(n: usize) -> String {
    format!("{n:02}")
}

/// Línea `inputs → output` de la tarjeta (mismos fallbacks `—` que el dibujo).
pub(crate) fn protocol_step_io_text(inputs: &[String], output: &str) -> String {
    let inputs_str = if inputs.is_empty() {
        PROTOCOL_EMPTY_CELL.to_string()
    } else {
        inputs.join(", ")
    };
    let output_str = if output.is_empty() {
        PROTOCOL_EMPTY_CELL.to_string()
    } else {
        output.to_string()
    };
    format!("{inputs_str} → {output_str}")
}

// --- extraído verbatim de panels.rs:4605+4623-4624+4651-4652+4729-4733+4768+4797+6834-6835 (slice 57 F14.4.4) ---
/// Textos puros del panel de regresión (`draw_right_regression_panel` +
/// botón de `draw_spreadsheet_section`: mismas cadenas que el dibujo, sin
/// egui; el `SidePanel`/botones/grillas quedan en `panels.rs`; los helpers
/// de formato (`regression_fit_headline`, …) ya viven acá desde el
/// slice 36).
pub(crate) const REGRESSION_PANEL_TITLE: &str = "Regresión";
pub(crate) const REGRESSION_PANEL_SUBTITLE: &str = "Ajuste a datos del documento.";
pub(crate) const REGRESSION_FITS_TITLE: &str = "Ajustes del documento";
pub(crate) const REGRESSION_FITS_EMPTY: &str = "Sin ajustes todavía.";
pub(crate) const REGRESSION_DIAGNOSTICS_TITLE: &str = "Diagnósticos";
pub(crate) const REGRESSION_DIAGNOSTICS_SUBTITLE: &str =
    "RMSE y R² por ajuste; residuales recortados a 24.";
pub(crate) const REGRESSION_CREATE_TITLE: &str = "Crear análisis";
pub(crate) const REGRESSION_CREATE_SUBTITLE: &str =
    "CSV/TSV de dos columnas o tabla local desde listas. La ruta nunca se guarda.";
pub(crate) const REGRESSION_IMPORT_LABEL: &str = "Importar CSV/TSV…";
pub(crate) const REGRESSION_IMPORT_HOVER: &str = "Dos columnas x,y · la ruta nunca se guarda";
pub(crate) const REGRESSION_MODEL_HINT: &str = "El comando del modelo va a la entrada.";
pub(crate) const REGRESSION_NO_TABLE_GUIDE: &str =
    "Seleccioná una tabla local en el lienzo para elegir un modelo de ajuste.";

/// Chrome estático del panel Regresión por sufijo (B-A147 + B-A148): Es → const
/// canónica de arriba, resto → `stats_msg` (display-localizado, storage
/// intacto; patrón `view_chrome_for` B-A128). B-A148 suma los 3 nombres de
/// plantilla + título/subtítulo del import de la hoja. Los dinámicos con
/// números (`regression_fit_diagnostics_line`, `regression_residuals_*`,
/// `regression_template/model_ready_text`, `regression_section_title`,
/// `regression_source_label`) viven en los helpers `*_for` de abajo
/// (Es delega al canónico, resto interpola `stats_msg`; patrón B-A98/B-A130).
/// `regression_fit_headline` ("{label} · {kind}") y `regression_residual_line`
/// ("r{i} = …") quedan invariantes a propósito (notación, B-A98/B-A114).
pub(crate) fn regression_chrome_for(
    suffix: &'static str,
    locale: grafito_ui::i18n::Locale,
) -> &'static str {
    use grafito_ui::i18n::Locale;
    if locale == Locale::Es {
        match suffix {
            "regression_panel_title" => REGRESSION_PANEL_TITLE,
            "regression_panel_subtitle" => REGRESSION_PANEL_SUBTITLE,
            "regression_fits_title" => REGRESSION_FITS_TITLE,
            "regression_fits_empty" => REGRESSION_FITS_EMPTY,
            "regression_diagnostics_title" => REGRESSION_DIAGNOSTICS_TITLE,
            "regression_diagnostics_subtitle" => REGRESSION_DIAGNOSTICS_SUBTITLE,
            "regression_create_title" => REGRESSION_CREATE_TITLE,
            "regression_create_subtitle" => REGRESSION_CREATE_SUBTITLE,
            "regression_import_label" => REGRESSION_IMPORT_LABEL,
            "regression_import_hover" => REGRESSION_IMPORT_HOVER,
            "regression_model_hint" => REGRESSION_MODEL_HINT,
            "regression_no_table_guide" => REGRESSION_NO_TABLE_GUIDE,
            "regression_template_scatter" => REGRESSION_TEMPLATE_SCATTER_NAME,
            "regression_template_regression" => REGRESSION_TEMPLATE_REGRESSION_NAME,
            "regression_template_local_table" => REGRESSION_TEMPLATE_LOCAL_TABLE_NAME,
            "regression_model_linear" => REGRESSION_MODEL_LINEAR_NAME,
            "regression_model_poly2" => REGRESSION_MODEL_POLY2_NAME,
            "regression_model_exp" => REGRESSION_MODEL_EXP_NAME,
            "regression_model_log" => REGRESSION_MODEL_LOG_NAME,
            "regression_model_power" => REGRESSION_MODEL_POWER_NAME,
            "regression_model_sin" => REGRESSION_MODEL_SIN_NAME,
            "regression_model_logistic" => REGRESSION_MODEL_LOGISTIC_NAME,
            "spreadsheet_import_title" => SPREADSHEET_IMPORT_TITLE,
            "spreadsheet_import_subtitle" => SPREADSHEET_IMPORT_SUBTITLE,
            _ => suffix,
        }
    } else {
        grafito_ui::i18n::stats_msg(suffix, locale)
    }
}

/// Plantillas de `Crear análisis` en el idioma pedido (B-A148): nombre
/// display localizado + comando invariante (`ScatterPlot`/`LinearRegression`/
/// `DataTable` son comandos, no se traducen; precedente B-A98).
pub(crate) fn regression_templates_for(
    locale: grafito_ui::i18n::Locale,
) -> [(&'static str, &'static str); 3] {
    [
        (
            regression_chrome_for("regression_template_scatter", locale),
            REGRESSION_TEMPLATES[0].1,
        ),
        (
            regression_chrome_for("regression_template_regression", locale),
            REGRESSION_TEMPLATES[1].1,
        ),
        (
            regression_chrome_for("regression_template_local_table", locale),
            REGRESSION_TEMPLATES[2].1,
        ),
    ]
}

/// Modelos del panel de regresión en el idioma pedido (B-A149): etiqueta
/// display localizada + comando invariante (`Fit*` son comandos, no se
/// traducen; precedente B-A98/B-A148). Es delega a las consts canónicas
/// (paridad byte-idéntica con `regression_model_commands`).
pub(crate) fn regression_model_commands_for(
    table_label: &str,
    locale: grafito_ui::i18n::Locale,
) -> Vec<(String, String)> {
    let labels = [
        regression_chrome_for("regression_model_linear", locale),
        regression_chrome_for("regression_model_poly2", locale),
        regression_chrome_for("regression_model_exp", locale),
        regression_chrome_for("regression_model_log", locale),
        regression_chrome_for("regression_model_power", locale),
        regression_chrome_for("regression_model_sin", locale),
        regression_chrome_for("regression_model_logistic", locale),
    ];
    let canonical = regression_model_commands(table_label);
    labels
        .into_iter()
        .zip(canonical)
        .map(|(label, (_, command))| (label.to_string(), command))
        .collect()
}

/// Etiqueta de origen del ajuste en el idioma pedido (B-A148, template sin
/// placeholder: la del objeto o la caída honesta "tabla eliminada").
/// Es delega al canónico [`regression_source_label`].
pub(crate) fn regression_source_label_for(
    label: Option<&str>,
    locale: grafito_ui::i18n::Locale,
) -> String {
    use grafito_ui::i18n::Locale;
    if locale == Locale::Es {
        regression_source_label(label)
    } else {
        label
            .unwrap_or(grafito_ui::i18n::stats_msg(
                "regression_deleted_source",
                locale,
            ))
            .to_string()
    }
}

/// Línea de diagnósticos en el idioma pedido (B-A148, templates
/// `{origen}/{rmse}/{r2}`; `·`, `RMSE` y `R²` intactos como notación).
/// Es delega al canónico [`regression_fit_diagnostics_line`].
pub(crate) fn regression_fit_diagnostics_line_for(
    source_label: &str,
    rmse: f64,
    r_squared: f64,
    locale: grafito_ui::i18n::Locale,
) -> String {
    use grafito_ui::i18n::Locale;
    if locale == Locale::Es {
        regression_fit_diagnostics_line(source_label, rmse, r_squared)
    } else {
        grafito_ui::i18n::stats_msg("regression_diagnostics_line", locale)
            .replace("{origen}", source_label)
            .replace("{rmse}", &format!("{rmse:.6}"))
            .replace("{r2}", &format!("{r_squared:.6}"))
    }
}

/// Título del desplegable de residuales en el idioma pedido (B-A148,
/// template `{n}`). Es delega al canónico [`regression_residuals_title`].
pub(crate) fn regression_residuals_title_for(
    count: usize,
    locale: grafito_ui::i18n::Locale,
) -> String {
    use grafito_ui::i18n::Locale;
    if locale == Locale::Es {
        regression_residuals_title(count)
    } else {
        grafito_ui::i18n::stats_msg("regression_residuals_title", locale)
            .replace("{n}", &count.to_string())
    }
}

/// Nota de recorte de residuales en el idioma pedido (B-A148, template
/// `{max}` con [`REGRESSION_MAX_RESIDUALS_SHOWN`]).
/// Es delega al canónico [`regression_residuals_truncated_note`].
pub(crate) fn regression_residuals_truncated_for(locale: grafito_ui::i18n::Locale) -> String {
    use grafito_ui::i18n::Locale;
    if locale == Locale::Es {
        regression_residuals_truncated_note().to_string()
    } else {
        grafito_ui::i18n::stats_msg("regression_residuals_truncated", locale)
            .replace("{max}", &REGRESSION_MAX_RESIDUALS_SHOWN.to_string())
    }
}

/// Título de la sección de modelos en el idioma pedido (B-A148, template
/// `{tabla}`). Es delega al canónico [`regression_section_title`].
pub(crate) fn regression_section_title_for(
    table_label: &str,
    locale: grafito_ui::i18n::Locale,
) -> String {
    use grafito_ui::i18n::Locale;
    if locale == Locale::Es {
        regression_section_title(table_label)
    } else {
        grafito_ui::i18n::stats_msg("regression_section_title", locale)
            .replace("{tabla}", table_label)
    }
}

/// Texto de éxito de plantilla en el idioma pedido (B-A148, template
/// `{name}` con el nombre display ya localizado). Es delega al canónico
/// [`regression_template_ready_text`].
pub(crate) fn regression_template_ready_text_for(
    name_display: &str,
    locale: grafito_ui::i18n::Locale,
) -> String {
    use grafito_ui::i18n::Locale;
    if locale == Locale::Es {
        regression_template_ready_text(name_display)
    } else {
        grafito_ui::i18n::stats_msg("regression_template_ready", locale)
            .replace("{name}", name_display)
    }
}

/// Texto de éxito de modelo en el idioma pedido (B-A148, template `{model}`
/// con la etiqueta del modelo tal cual la da `regression_model_commands_for`
/// (B-A149: display localizado, comando invariante).
/// Es delega al canónico [`regression_model_ready_text`].
pub(crate) fn regression_model_ready_text_for(
    model_display: &str,
    locale: grafito_ui::i18n::Locale,
) -> String {
    use grafito_ui::i18n::Locale;
    if locale == Locale::Es {
        regression_model_ready_text(model_display)
    } else {
        grafito_ui::i18n::stats_msg("regression_model_ready", locale)
            .replace("{model}", model_display)
    }
}

// --- extraído verbatim de panels.rs:3629+3632+3661+3664+3693+4084+4089+4096+4116+4123+4211+4216 (slice 58 F14.4.4) ---
/// Textos puros 3D del Inspector (`draw_right_properties_contents` ramas
/// `Cube3D`/`Sphere3D`/`Point3D`/`ImplicitSurface3D` + fallback del dock:
/// mismas cadenas y mismos `format!` que el dibujo, sin egui; el
/// `SidePanel`/`ScrollArea`/sliders/botones/drawer quedan en `panels.rs`).
/// `inspector_3d_tag_label` es incondicional a propósito: las 3 ramas 3D
/// hacen `format!("Etiqueta: {}", label)` sin el `trim`/None de
/// `inspector_label_text` (slice 17).
pub(crate) const INSPECTOR_CUBE3D_TITLE: &str = "Cubo 3D";
pub(crate) const INSPECTOR_SPHERE3D_TITLE: &str = "Esfera 3D";
pub(crate) const INSPECTOR_POINT3D_TITLE: &str = "Punto 3D";
pub(crate) const INSPECTOR_CENTER_LABEL: &str = "Centro";
pub(crate) const INSPECTOR_IMPLICIT3D_TITLE: &str = "Superficie implícita 3D";
pub(crate) const INSPECTOR_DOCK_EMPTY_TITLE: &str = "Sin controles dedicados en este dock.";
pub(crate) const INSPECTOR_DOCK_EMPTY_GUIDE: &str = "Editá este objeto desde el panel de Álgebra.";

pub(crate) fn inspector_3d_tag_label(label: &str) -> String {
    format!("Etiqueta: {label}")
}

/// Etiqueta 3D (`Etiqueta: …`) en el idioma pedido (B-A111, template `{label}`).
/// Es delega al canónico [`inspector_3d_tag_label`]; el resto interpola i18n.
/// Incondicional a propósito (sin `trim`/None, como el canónico).
pub(crate) fn inspector_3d_tag_label_for(label: &str, locale: Locale) -> String {
    if locale == Locale::Es {
        inspector_3d_tag_label(label)
    } else {
        grafito_ui::i18n::inspector_msg("label_prefix", locale).replace("{label}", label)
    }
}

pub(crate) fn implicit_3d_expr_text(expr: &str) -> String {
    format!("F(x, y, z) = {expr}")
}

pub(crate) fn implicit_3d_box_text(
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    z_min: f64,
    z_max: f64,
    cells: usize,
) -> String {
    format!("Caja [{x_min:.2}, {x_max:.2}] × [{y_min:.2}, {y_max:.2}] × [{z_min:.2}, {z_max:.2}] · {cells}³ celdas")
}

pub(crate) fn implicit_3d_state_text(status: &str) -> String {
    format!("Estado: {status}")
}

pub(crate) fn implicit_3d_preview_text(tris: usize) -> String {
    format!("Vista previa: {tris} triángulos (el canvas la muestra mientras refina)")
}

// Slice 59: textos puros de los botones de vista previa implícita 3D de
// `draw_right_properties_contents` en `panels.rs` (rama `ImplicitSurface3D`):
// mismas cadenas que el dibujo, sin egui. El `SidePanel`/botones/drawer +
// el slot en background quedan en panels a propósito; la decisión del aviso
// (`implicit_quick_preview_notify`) es pura sobre el estado del slot
// (antes/después de `maybe_submit_implicit_slot`), misma tabla que el `if`
// del dibujo.
pub(crate) const IMPLICIT_QUICK_PREVIEW_GUIDE: &str =
    "Celdas altas: el fino tarda; la vista previa gruesa (12³) llega primero.";
pub(crate) const IMPLICIT_QUICK_PREVIEW_LABEL: &str = "Vista previa rápida";
pub(crate) const IMPLICIT_QUICK_PREVIEW_HOVER: &str = "Genera el grueso 12³ en segundo plano";
pub(crate) const IMPLICIT_PREVIEW_RUNNING: &str = "Vista previa en curso…";
pub(crate) const IMPLICIT_PREVIEW_ALREADY_SHOWN: &str =
    "Ya hay vista previa mostrándose en el canvas.";
pub(crate) const IMPLICIT_PREVIEW_ALREADY_RUNNING: &str = "Ya se está generando la vista previa…";
pub(crate) const IMPLICIT_PREVIEW_START_FAILED: &str =
    "No se pudo iniciar la vista previa (revisá la superficie).";
pub(crate) const IMPLICIT_PREVIEW_CANCELLED: &str = "Vista previa cancelada.";
pub(crate) const IMPLICIT_PREVIEW_CANCEL_LABEL: &str = "Cancelar";
pub(crate) const IMPLICIT_PREVIEW_CANCEL_EMPTY: &str =
    "No hay vista previa en curso para cancelar.";
pub(crate) const IMPLICIT_PREVIEW_CANCEL_HOVER: &str = "Detiene la vista previa en curso";

pub(crate) fn implicit_quick_preview_notify(
    was_pending: bool,
    had_valid: bool,
    now_pending: bool,
) -> &'static str {
    if now_pending && !was_pending {
        IMPLICIT_PREVIEW_RUNNING
    } else if had_valid {
        IMPLICIT_PREVIEW_ALREADY_SHOWN
    } else if was_pending {
        IMPLICIT_PREVIEW_ALREADY_RUNNING
    } else {
        IMPLICIT_PREVIEW_START_FAILED
    }
}

/// Etiqueta del botón Cancelar de la vista previa en el idioma pedido
/// (B-A113). Es delega a la const canónica
/// [`IMPLICIT_PREVIEW_CANCEL_LABEL`]; el resto reutiliza la key existente
/// `common.cancel` (precedente B-A66/B-A95, sin key propia). Puro y headless.
pub(crate) fn implicit_cancel_label_for(locale: Locale) -> &'static str {
    if locale == Locale::Es {
        IMPLICIT_PREVIEW_CANCEL_LABEL
    } else {
        grafito_ui::i18n::t("common.cancel", locale)
    }
}
/// Aviso de vista previa implícita en el idioma pedido (B-A113).
/// Es delega al canónico [`implicit_quick_preview_notify`] (misma tabla);
/// el resto sale de i18n (`inspector.implicit_*`). Puro y headless.
pub(crate) fn implicit_quick_preview_notify_for(
    was_pending: bool,
    had_valid: bool,
    now_pending: bool,
    locale: Locale,
) -> &'static str {
    if locale == Locale::Es {
        implicit_quick_preview_notify(was_pending, had_valid, now_pending)
    } else if now_pending && !was_pending {
        grafito_ui::i18n::inspector_msg("implicit_running", locale)
    } else if had_valid {
        grafito_ui::i18n::inspector_msg("implicit_already_shown", locale)
    } else if was_pending {
        grafito_ui::i18n::inspector_msg("implicit_already_running", locale)
    } else {
        grafito_ui::i18n::inspector_msg("implicit_start_failed", locale)
    }
}

// Slice 60: textos puros del Inspector 4D/N-D de
// `draw_right_properties_contents` en `panels.rs` (ramas
// `RegularPolychoron4D`/`RegularPolytopeND`): mismas cadenas que el dibujo,
// sin egui. El `SidePanel`/`ScrollArea`/sliders/combos/botones/drawer y el
// commit con undo quedan en panels a propósito; las etiquetas de
// kind/familia y el cuerpo de proyección N-D son puras sobre el modelo.
pub(crate) const POLYCHORON_PROJECTION_TITLE: &str = "Proyección";
pub(crate) const POLYCHORON_PROJECTION_SUBTITLE: &str =
    "Controlá la vista dinámica sin alterar la construcción.";
pub(crate) const POLYCHORON_ANIM_TITLE: &str = "Animación de proyección";
pub(crate) const POLYCHORON_ANIM_BODY: &str =
    "La cámara y el politopo 4D giran sin alterar el documento.";
pub(crate) const POLYCHORON_GEOMETRY_TITLE: &str = "Geometría";
pub(crate) const POLYCHORON_GEOMETRY_SUBTITLE: &str = "Forma y escala";
pub(crate) const POLYCHORON_APPEARANCE_TITLE: &str = "Apariencia";
pub(crate) const POLYCHORON_APPEARANCE_SUBTITLE: &str = "Estilo de aristas y relleno";
pub(crate) const POLYCHORON_FILL_OMITTED_HINT: &str =
    "El relleno se omite en Vista previa y durante el movimiento.";
pub(crate) const POLYCHORON_ROTATION_MANUAL_TITLE: &str = "Rotación manual";
pub(crate) const POLYCHORON_ROTATION_MANUAL_GUIDE: &str =
    "Ajustá los seis planos sólo cuando necesites una vista específica.";
pub(crate) const POLYCHORON_ROTATION_RESET_LABEL: &str = "Restablecer rotaciones";
pub(crate) const POLYCHORON_ROTATION_PLANES_TITLE: &str = "Planos de rotación";
/// Etiquetas de los seis planos de rotación 4D (mismo orden que el dibujo).
pub(crate) const POLYCHORON_ROTATION_PLANES: [&str; 6] = [
    "xy (rad)", "xz (rad)", "xw (rad)", "yz (rad)", "yw (rad)", "zw (rad)",
];
pub(crate) const POLYTOPE_ND_GEOMETRY_SUBTITLE: &str = "Familia, dimensión y escala";
pub(crate) const POLYTOPE_ND_PROJECTION_SUBTITLE: &str =
    "La cámara y la proyección 4D comparten velocidad.";
pub(crate) const POLYTOPE_ND_ANIM_DIM4_BODY: &str =
    "La cámara y la proyección 4D usan la misma velocidad.";
pub(crate) const POLYTOPE_ND_ANIM_OTHER_BODY: &str = "La cámara orbita sin alterar el documento.";
pub(crate) const POLYTOPE_ND_APPEARANCE_SUBTITLE: &str = "Estilo de aristas";
pub(crate) const POLYTOPE_ND_FILL_HINT: &str =
    "Los politopos N-D genéricos se muestran solo como aristas; el relleno se omite en Vista previa y durante el movimiento.";
pub(crate) const POLYTOPE_ND_ROTATION_GUIDE: &str =
    "Los ajustes avanzados permanecen separados de los controles principales.";

/// Etiqueta del kind 4D (mismo `match` que el `selected_text` + los
/// `selectable_value` del dibujo en `panels.rs`).
pub(crate) fn polychoron_kind_label(kind: RegularPolychoron) -> &'static str {
    match kind {
        RegularPolychoron::Pentachoron => "Pentácoron (5-celda)",
        RegularPolychoron::Tesseract => "Teseracto",
        RegularPolychoron::SixteenCell => "16-celda",
        RegularPolychoron::TwentyFourCell => "24-celda",
        RegularPolychoron::OneTwentyCell => "120-celda",
        RegularPolychoron::SixHundredCell => "600-celda",
    }
}

/// Etiqueta de la familia N-D (mismo `match` que el dibujo en `panels.rs`).
pub(crate) fn polytope_nd_family_label(family: RegularPolytopeFamily) -> &'static str {
    match family {
        RegularPolytopeFamily::Simplex => "Símplex",
        RegularPolytopeFamily::Hypercube => "Hipercubo",
        RegularPolytopeFamily::CrossPolytope => "Politopo cruzado",
    }
}

/// Etiqueta del kind 4D en el idioma pedido (B-A110).
/// Es delega al canónico [`polychoron_kind_label`]; el resto sale de i18n.
pub(crate) fn polychoron_kind_label_for(kind: RegularPolychoron, locale: Locale) -> &'static str {
    if locale == Locale::Es {
        polychoron_kind_label(kind)
    } else {
        let suffix = match kind {
            RegularPolychoron::Pentachoron => "polychoron_kind_pentachoron",
            RegularPolychoron::Tesseract => "polychoron_kind_tesseract",
            RegularPolychoron::SixteenCell => "polychoron_kind_sixteen",
            RegularPolychoron::TwentyFourCell => "polychoron_kind_twentyfour",
            RegularPolychoron::OneTwentyCell => "polychoron_kind_onetwenty",
            RegularPolychoron::SixHundredCell => "polychoron_kind_sixhundred",
        };
        grafito_ui::i18n::inspector_msg(suffix, locale)
    }
}

/// Etiqueta de la familia N-D en el idioma pedido (B-A110).
/// Es delega al canónico [`polytope_nd_family_label`]; el resto sale de i18n.
pub(crate) fn polytope_nd_family_label_for(
    family: RegularPolytopeFamily,
    locale: Locale,
) -> &'static str {
    if locale == Locale::Es {
        polytope_nd_family_label(family)
    } else {
        let suffix = match family {
            RegularPolytopeFamily::Simplex => "polytope_family_simplex",
            RegularPolytopeFamily::Hypercube => "polytope_family_hypercube",
            RegularPolytopeFamily::CrossPolytope => "polytope_family_cross",
        };
        grafito_ui::i18n::inspector_msg(suffix, locale)
    }
}

/// Cuerpo de la tarjeta de proyección N-D (misma rama `dimension == 4`
/// que el dibujo en `panels.rs`).
pub(crate) fn polytope_nd_projection_body(dimension: usize) -> &'static str {
    if dimension == 4 {
        POLYTOPE_ND_ANIM_DIM4_BODY
    } else {
        POLYTOPE_ND_ANIM_OTHER_BODY
    }
}

/// Rótulo del plano de rotación N-D genérico (mismo `format!` que el
/// `Slider::text` del dibujo en `panels.rs`).
pub(crate) fn nd_rotation_plane_label(first: usize, second: usize) -> String {
    format!("x{}/x{} (rad)", first + 1, second + 1)
}

/// Controles primitivos 3D/4D del Inspector (slice 61): mismos prefijos de
/// ejes, velocidad de DragValue, textos de sliders y rangos que el dibujo
/// en `panels.rs` (ramas `Cube3D`/`Sphere3D`/`Point3D` + sliders
/// Escala/Grosor de `RegularPolychoron4D`/`RegularPolytopeND`), sin egui.
/// Los `SidePanel`/`DragValue`/`Slider`/combos/botones + commit con undo
/// quedan en `panels.rs` a propósito.
pub(crate) const INSPECTOR_AXIS_PREFIX_X: &str = "x=";
pub(crate) const INSPECTOR_AXIS_PREFIX_Y: &str = "y=";
pub(crate) const INSPECTOR_AXIS_PREFIX_Z: &str = "z=";
/// Velocidad de los DragValue de centro/posición 3D (mismo `0.1` que el dibujo).
pub(crate) const INSPECTOR_CENTER_DRAG_SPEED: f64 = 0.1;
pub(crate) const INSPECTOR_SIZE_TEXT: &str = "tamaño";
pub(crate) const INSPECTOR_RADIUS_TEXT: &str = "radio";
pub(crate) const INSPECTOR_SCALE_TEXT: &str = "Escala";
pub(crate) const INSPECTOR_EDGE_WIDTH_TEXT: &str = "Grosor de aristas";
/// Chrome de apariencia 4D + cierres en el idioma pedido (B-A159).
/// Es delega a la const canónica; el resto sale de `grafito-ui` i18n.
pub(crate) const INSPECTOR_EDGE_COLOR_LABEL: &str = "Color de aristas";
pub(crate) const INSPECTOR_EDGE_COLOR_HOVER: &str = "Cambiar color de aristas";
pub(crate) const INSPECTOR_FILL_ENABLED_LABEL: &str = "Relleno habilitado";
pub(crate) const INSPECTOR_FILL_COLOR_LABEL: &str = "Color de relleno";
pub(crate) const INSPECTOR_FILL_COLOR_HOVER: &str = "Cambiar color de relleno";
pub(crate) const INSPECTOR_DIMENSION_LABEL: &str = "Dimensión";
pub(crate) const INSPECTOR_CLOSE_CONTEXT: &str = "Cerrar panel contextual";
pub(crate) const INSPECTOR_CLOSE_ANIM: &str = "Cerrar animación";
/// Rangos de los sliders primitivos (mismos literales que el dibujo;
/// `f64`/`f32` según el tipo del campo del modelo: cubo/esfera/escala
/// son `f64`, punto/ancho son `f32`).
pub(crate) const INSPECTOR_CUBE_SIZE_MIN: f64 = 0.1;
pub(crate) const INSPECTOR_CUBE_SIZE_MAX: f64 = 10.0;
pub(crate) const INSPECTOR_SPHERE_RADIUS_MIN: f64 = 0.1;
pub(crate) const INSPECTOR_SPHERE_RADIUS_MAX: f64 = 10.0;
pub(crate) const INSPECTOR_POINT_SIZE_MIN: f32 = 1.0;
pub(crate) const INSPECTOR_POINT_SIZE_MAX: f32 = 20.0;
pub(crate) const INSPECTOR_POLY_SCALE_MIN: f64 = 0.01;
pub(crate) const INSPECTOR_POLY_SCALE_MAX: f64 = 10.0;
pub(crate) const INSPECTOR_POLY_WIDTH_MIN: f32 = 0.5;
pub(crate) const INSPECTOR_POLY_WIDTH_MAX: f32 = 10.0;

/// Rangos como `RangeInclusive` listos para `egui::Slider::new`
/// (mismos extremos que las consts `*_MIN`/`*_MAX` de arriba).
pub(crate) fn inspector_cube_size_range() -> std::ops::RangeInclusive<f64> {
    INSPECTOR_CUBE_SIZE_MIN..=INSPECTOR_CUBE_SIZE_MAX
}
pub(crate) fn inspector_sphere_radius_range() -> std::ops::RangeInclusive<f64> {
    INSPECTOR_SPHERE_RADIUS_MIN..=INSPECTOR_SPHERE_RADIUS_MAX
}
pub(crate) fn inspector_point_size_range() -> std::ops::RangeInclusive<f32> {
    INSPECTOR_POINT_SIZE_MIN..=INSPECTOR_POINT_SIZE_MAX
}
pub(crate) fn inspector_poly_scale_range() -> std::ops::RangeInclusive<f64> {
    INSPECTOR_POLY_SCALE_MIN..=INSPECTOR_POLY_SCALE_MAX
}
pub(crate) fn inspector_poly_width_range() -> std::ops::RangeInclusive<f32> {
    INSPECTOR_POLY_WIDTH_MIN..=INSPECTOR_POLY_WIDTH_MAX
}

#[cfg(test)]
mod tests {
    use super::{
        apply_object_panel_edit, apply_object_panel_edit_with_previous, binomial_plot_left_label,
        binomial_plot_right_label, binomial_plot_window, binomial_pmf, build_series_edits,
        chi_squared_plot_hi, clamp_binomial_k, clamp_binomial_n, collect_sheet_tables,
        construction_log_to_latex, discrete_k_sel, discrete_plot_left_label,
        discrete_plot_right_label, distribution_moments, escape_latex, example_sheet_table,
        example_sheet_table_error_text, example_sheet_table_loaded_text, export_grid_cell_w,
        export_grid_columns, f_plot_hi, format_series_preview_text, format_series_success,
        format_statistic, geometric_plot_hi, geometric_pmf, inspector_equation_text,
        inspector_identity_tooltip, inspector_label_text, inspector_type_caption,
        layer_assign_error, layer_assign_result, layer_display_name, layer_row_label,
        layer_selection_text, layer_toggle_result, line_style_label, local_xy_table_objects,
        normal_band_fractions, normal_cdf, normal_pdf, normal_plot_window, parse_local_xy_table,
        parse_sheet_goto, parse_slider_prompt, parse_statistics_input, plot_df_or_fuera_de_cota,
        point_style_label, poisson_plot_hi, poisson_pmf, prob_case_command, prob_case_copy_text,
        prob_case_detail_tooltip, prob_case_result, prob_distribution_is_discrete,
        prob_moment_text, prob_quantile_command, prob_tail_text, prob_tail_values,
        protocol_advance, regression_model_commands, series_preview, set_complex_mapping_animation,
        set_domain_coloring_mode, sheet_cell_display, sheet_cell_width_for, sheet_chip_text,
        sheet_col_label, sheet_hidden_tables_overflow, sheet_nav_metrics, sheet_series_field_w,
        sheet_summary_field_w, sheet_summary_tiles, sheet_table_rows, sheet_table_rows_overflow,
        sheet_table_shown, sheet_table_subtitle, sheet_table_title, stable_interpolate,
        stat_tile_cols, stat_tile_width_for, statistics_assistant_prompt,
        statistics_boxplot_command, statistics_data_preview, statistics_histogram_command,
        statistics_inline_list, statistics_mean_command, statistics_quick_chips,
        statistics_quick_ready_text, statistics_summary, trig_angle_text, trig_complex_text,
        trig_function_chip_label, trig_function_columns, trig_grid_cell_w, trig_point_text,
        trig_point_text_for, trig_prompt_apply_label, trig_prompt_apply_tooltip,
        trig_prompt_auto_label, trig_prompt_field_tooltip, trig_prompt_field_w,
        trig_prompt_hint_text, trig_speed_text, trig_value_text, trig_value_text_for,
        truncate_table_name, visible_sheet_cols, wc_exact_integral_command_text, wc_quadrature_key,
        wc_quadrature_method, wc_riemann_command_text, wc_study_command_text,
        wc_taylor_command_text, wc_taylor_remainder_line, LayerPanelState, ProbabilityPanelState,
        ProtocolNavState, SheetCellKind, SheetEditState, SheetSeriesState, SheetSummaryState,
        TrigPromptState, WcVisualState, LAYER_EMPTY_HINT, LAYER_NO_SELECTION, MAX_BINOMIAL_N,
        MAX_GEOMETRIC_K, MAX_PANEL_DF, MAX_POISSON_K, MAX_SPREADSHEET_ROWS, MAX_SPREADSHEET_TABLES,
        PROB_DISTS, PROB_QUANTILE_ALPHAS, PROTOCOL_PLAY_INTERVAL_SECS, SHEET_CELL_H,
        SHEET_CONTROL_H, SHEET_SUMMARY_BTN_W, SHEET_VIEW_COLS, STATISTICS_BOXPLOT_READY,
        STATISTICS_HISTOGRAM_READY, STATISTICS_INLINE_TOO_MANY, STATISTICS_MEAN_READY,
        STATISTICS_NO_INLINE_HINT, STATISTICS_NO_INLINE_RESULT, TRIG_APPLY_W, TRIG_CANVAS_MIN_W,
        TRIG_CHIP_MIN_W, TRIG_CONTROL_H, TRIG_PANEL_DEFAULT_W, TRIG_PANEL_MAX_W, TRIG_PANEL_MIN_W,
        TRIG_PROMPT_APPLY_LABEL, TRIG_PROMPT_APPLY_TOOLTIP, TRIG_PROMPT_AUTO_LABEL,
        TRIG_PROMPT_FIELD_TOOLTIP, TRIG_PROMPT_HINT, T_PLOT_HI, T_PLOT_LO, WC_QUADRATURE_METHODS,
        WC_RIEMANN_SLIDER_MAX,
    };
    use grafito_ui::tokens::{HIT_TARGET_MIN, SPACE_SM, SPACE_XS};
    use std::collections::BTreeMap;

    #[test]
    fn sheet_layout_no_desborda_y_pinnea_rangos() {
        // Paridad con `panels::gc_piel_tests`: mismos pines, dueño nuevo.
        assert_eq!(visible_sheet_cols(252.0), 4);
        assert_eq!(visible_sheet_cols(336.0), 6);
        assert_eq!(visible_sheet_cols(120.0), 3);
        assert_eq!(visible_sheet_cols(10_000.0), SHEET_VIEW_COLS);
        for width in [252.0_f32, 300.0, 336.0, 180.0] {
            let cols = visible_sheet_cols(width);
            let cell_w = sheet_cell_width_for(width, cols);
            assert!((36.0..=96.0).contains(&cell_w));
            let total = 32.0 + cols as f32 * (cell_w + SPACE_XS);
            assert!(total <= width + 0.5);
        }
        for width in [112.0_f32, 132.0, 160.0, 252.0, 360.0, 520.0] {
            let m = sheet_nav_metrics(width);
            let row1 = m.chip_w + 4.0 * m.arrow_w + 4.0 * SPACE_XS;
            let row2 = m.field_w + 2.0 * m.action_w + 2.0 * SPACE_XS;
            assert!(row1 <= width + 0.5);
            assert!(row2 <= width + 0.5);
            assert!(m.arrow_w >= HIT_TARGET_MIN);
            assert!(m.field_w >= HIT_TARGET_MIN);
            assert!(m.action_w >= HIT_TARGET_MIN);
        }
        assert_eq!(sheet_col_label(0), "A");
        assert_eq!(sheet_col_label(25), "Z");
        assert_eq!(sheet_col_label(26), "AA");
    }

    #[test]
    fn sheet_tablas_en_lectura_pinnea_titulos_y_modelos() {
        use grafito_core::{DataTableObj, Document, GeoObject};
        assert_eq!(MAX_SPREADSHEET_ROWS, 50);
        assert_eq!(MAX_SPREADSHEET_TABLES, 8);
        assert_eq!(sheet_table_title(""), "<sin etiqueta>");
        assert_eq!(sheet_table_title("T1"), "T1");
        assert_eq!(truncate_table_name("x"), "x");
        let largo = "a".repeat(30);
        let cortado = truncate_table_name(&largo);
        assert_eq!(cortado.chars().count(), 24);
        assert!(cortado.ends_with('…'));
        let models = regression_model_commands("T1");
        assert_eq!(models.len(), 7);
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
            assert!(commands.contains(&expected), "falta {expected}");
        }
        let mut document = Document::new();
        assert!(collect_sheet_tables(&document).is_empty());
        let table = GeoObject::DataTable(
            DataTableObj::new("x", "y", vec![0.0, 1.0], vec![0.0, 1.0]).with_label("T1"),
        );
        document.try_add_object(table).expect("tabla válida");
        let tablas = collect_sheet_tables(&document);
        assert_eq!(tablas.len(), 1);
        assert_eq!(tablas[0].label, "T1");
        assert_eq!(tablas[0].xs, vec![0.0, 1.0]);
        assert_eq!(sheet_table_title(&tablas[0].label), "T1");
    }

    #[test]
    fn sheet_summary_tiles_pinnea_los_5_tiles_y_columnas() {
        // Paridad con `panels::gc_piel_tests::stat_tile_cols_nunca_desborda`
        // y con el armado inline de `draw_sheet_summary_card`.
        assert_eq!(stat_tile_cols(10_000.0), 3);
        assert_eq!(stat_tile_cols(300.0), 3);
        assert_eq!(stat_tile_cols(200.0), 2);
        assert_eq!(stat_tile_cols(132.0), 1);
        assert_eq!(stat_tile_cols(1.0), 1);
        let summary = super::RangeSummary {
            count: 3,
            mean: 2.0,
            sample_sd: Some(1.0),
            min: 1.0,
            max: 3.0,
        };
        let tiles = sheet_summary_tiles(&summary, |v| format!("{v:.1}"));
        assert_eq!(tiles.len(), 5);
        let labels: Vec<&str> = tiles.iter().map(|(l, _)| l.as_str()).collect();
        assert_eq!(labels, ["N", "Media", "Desvío s", "Mín", "Máx"]);
        let values: Vec<&str> = tiles.iter().map(|(_, v)| v.as_str()).collect();
        assert_eq!(values, ["3", "2.0", "1.0", "1.0", "3.0"]);
        // n = 1: sin desvío muestral, el tile muestra "—" (no se inventa).
        let single = super::RangeSummary {
            count: 1,
            mean: 5.0,
            sample_sd: None,
            min: 5.0,
            max: 5.0,
        };
        let tiles = sheet_summary_tiles(&single, |v| format!("{v:.1}"));
        assert_eq!(tiles[2], ("Desvío s".to_string(), "—".to_string()));
    }

    #[test]
    fn sheet_goto_y_chip_pinnea_el_dibujo() {
        // Paridad con `draw_sheet_editable_grid`: mismo parser del cerebro
        // y mismo mensaje honesto con el texto recortado.
        assert_eq!(parse_sheet_goto("C3"), Ok((2, 2)));
        assert_eq!(parse_sheet_goto("  c3  "), Ok((2, 2)));
        assert_eq!(parse_sheet_goto("A1"), Ok((0, 0)));
        assert_eq!(
            parse_sheet_goto(""),
            Err("Ir a: escribí una celda (p. ej. C3)".to_string())
        );
        assert_eq!(
            parse_sheet_goto("   "),
            Err("Ir a: escribí una celda (p. ej. C3)".to_string())
        );
        assert_eq!(
            parse_sheet_goto("hola"),
            Err("«hola» no es una celda válida".to_string())
        );
        assert_eq!(
            parse_sheet_goto("  C0  "),
            Err("«C0» no es una celda válida".to_string())
        );
        // Chip: celda en edición o ventana visible, misma expresión.
        let view = super::SheetViewState {
            origin_row: 2,
            origin_col: 2,
        };
        assert_eq!(sheet_chip_text(Some((2, 2)), &view, 4), "C3");
        assert_eq!(sheet_chip_text(None, &view, 4), view.window_label(4));
    }

    #[test]
    fn sheet_estadistica_pura_pinnea_parse_resumen_y_comandos() {
        // Paridad con `panels::coverage_sweep_panels_pure` y
        // `panels::statistics_interpolation_tests`: mismas expresiones.
        assert_eq!(
            parse_statistics_input("1, 2\n3").expect("nums"),
            vec![1.0, 2.0, 3.0]
        );
        assert!(parse_statistics_input("").expect("vacío").is_empty());
        assert!(parse_statistics_input("1,,2").is_err());
        assert!(parse_statistics_input("inf").is_err());
        let summary = statistics_summary(&[1.0, 2.0, 3.0, 4.0]).expect("resumen");
        assert!((summary.mean - 2.5).abs() < 1e-9);
        assert!((summary.median - 2.5).abs() < 1e-9);
        assert!(statistics_summary(&[]).is_err());
        assert!(statistics_summary(&[1.0, f64::NAN]).is_err());
        assert_eq!(stable_interpolate(-2.0, 2.0, 0.5), 0.0);
        assert_eq!(stable_interpolate(f64::MAX, f64::MAX, 0.5), f64::MAX);
        assert_eq!(statistics_data_preview(&[]), "");
        assert!(statistics_data_preview(&[1.0, 2.0]).contains("1.000000"));
        assert!(statistics_assistant_prompt(&[]).contains("vacía"));
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
        assert!(statistics_inline_list(&[]).is_none());
        assert_eq!(format_statistic(2.5), "2.500");
        assert_eq!(format_statistic(2_500_000.0), "2.500e6");
    }

    #[test]
    fn sheet_stats_chips_pinnea_comandos_y_textos_del_dibujo() {
        // Paridad con `draw_statistics_panel` en `panels.rs`: mismas 6
        // tuplas y mismas cadenas, dueño nuevo (slice 37). Los botones,
        // el `notify` y el drawer quedan en panels.
        let chips = statistics_quick_chips("1, 2");
        assert_eq!(chips.len(), 6);
        assert_eq!(chips[0].0, "Mediana");
        assert_eq!(chips[0].1, "Median[{1, 2}]");
        assert_eq!(chips[0].2, "Mediana de tus datos");
        assert_eq!(chips[1].1, "StdDev[{1, 2}]");
        assert_eq!(chips[2].1, "FrequencyTable[{1, 2}]");
        assert_eq!(chips[3].1, "StemPlot[{1, 2}]");
        assert_eq!(chips[4].1, "TTest[{1, 2}, 0]");
        assert_eq!(chips[5].1, "ZTest[{1, 2}, 0, 1]");
        assert_eq!(
            statistics_quick_ready_text("Mediana"),
            "Mediana listo en la entrada (Enter para evaluar)."
        );
        assert_eq!(
            STATISTICS_HISTOGRAM_READY,
            "Histograma listo en la entrada (Enter para crear)."
        );
        assert_eq!(
            STATISTICS_BOXPLOT_READY,
            "Diagrama de caja listo en la entrada."
        );
        assert_eq!(STATISTICS_MEAN_READY, "Media lista en la entrada.");
        assert!(STATISTICS_INLINE_TOO_MANY.contains("200 inline"));
        assert!(STATISTICS_NO_INLINE_RESULT.contains("+200 valores"));
        assert!(STATISTICS_NO_INLINE_HINT.contains("pegá menos datos"));
    }

    #[test]
    fn sheet_quick_b_a163_chips_localizan() {
        // B-A163: `_for` delega Es al canónico (paridad byte-idéntica) y
        // localiza el resto vía `stats.quick_chip_*/quick_tip_*`;
        // comandos invariantes (precedente B-A98/B-A148) + notación μ/σ
        // intacta.
        use super::{statistics_quick_chips, statistics_quick_chips_for};
        use grafito_ui::i18n::Locale;
        let es = statistics_quick_chips_for("1, 2", Locale::Es);
        let canonical = statistics_quick_chips("1, 2");
        for i in 0..6 {
            assert_eq!(es[i].0, canonical[i].0);
            assert_eq!(es[i].1, canonical[i].1);
            assert_eq!(es[i].2, canonical[i].2);
        }
        // Comandos invariantes en los 6 idiomas.
        for locale in [
            Locale::Es,
            Locale::En,
            Locale::Pt,
            Locale::It,
            Locale::Fr,
            Locale::De,
        ] {
            let chips = statistics_quick_chips_for("1, 2", locale);
            assert_eq!(chips[0].1, "Median[{1, 2}]");
            assert_eq!(chips[1].1, "StdDev[{1, 2}]");
            assert_eq!(chips[2].1, "FrequencyTable[{1, 2}]");
            assert_eq!(chips[3].1, "StemPlot[{1, 2}]");
            assert_eq!(chips[4].1, "TTest[{1, 2}, 0]");
            assert_eq!(chips[5].1, "ZTest[{1, 2}, 0, 1]");
            for (_, _, tip) in chips.iter() {
                assert!(!tip.is_empty(), "tip vacío en {locale:?}");
                assert!(!tip.contains("stats."), "fallback en {locale:?}");
            }
        }
        assert_eq!(
            statistics_quick_chips_for("1, 2", Locale::En)[0].0,
            "Median"
        );
        assert_eq!(
            statistics_quick_chips_for("1, 2", Locale::De)[5].2,
            "z-Test (σ=1 bekannt)"
        );
    }

    #[test]
    fn sheet_prob_pura_pinnea_distribuciones_y_cotas() {
        // Paridad con `panels::gc_piel_tests`: mismas expresiones, dueño nuevo.
        let pdf = normal_pdf(0.0, 0.0, 1.0).expect("normal válida");
        assert!((pdf - 0.398_942_280_401_432_7).abs() < 1e-6);
        assert!((normal_cdf(0.0, 0.0, 1.0).expect("cdf") - 0.5).abs() < 1e-6);
        assert!(normal_pdf(0.0, 0.0, 0.0).is_err());
        let pmf = binomial_pmf(5, 10, 0.5).expect("binomial válida");
        assert!((pmf - 252.0 / 1024.0).abs() < 1e-9);
        assert!(binomial_pmf(1, MAX_BINOMIAL_N + 1, 0.5).is_err());
        let pp = poisson_pmf(0, 1.0).expect("poisson válida");
        assert!((pp - 0.367_879_441_171_442_33).abs() < 1e-9);
        assert_eq!(prob_tail_values(0.5), Some((0.5, 0.5, 1.0)));
        assert_eq!(prob_tail_values(f64::NAN), None);
        assert!(prob_distribution_is_discrete(1));
        assert!(!prob_distribution_is_discrete(0));
        assert!(PROB_QUANTILE_ALPHAS.windows(2).all(|w| w[0] < w[1]));
        assert!(plot_df_or_fuera_de_cota(MAX_PANEL_DF, "gl").is_ok());
        assert!(plot_df_or_fuera_de_cota(MAX_PANEL_DF + 1.0, "gl").is_err());
        assert!(geometric_pmf(MAX_GEOMETRIC_K + 1, 0.5).is_err());
        assert_eq!(parse_slider_prompt("medio").expect("palabra"), 0.5);
        assert!(parse_slider_prompt("verde").is_err());
    }

    #[test]
    fn sheet_prob_comandos_pinnea_momentos_y_casos() {
        // Paridad con `panels::probability_commands_tests`: mismas expresiones.
        assert_eq!(PROB_DISTS.len(), 9);
        assert!(PROB_DISTS.iter().map(|(id, _, _)| *id).eq(0..=8));
        let state = ProbabilityPanelState::default();
        assert_eq!(distribution_moments(&state), (Some(0.0), Some(1.0)));
        assert_eq!(
            prob_case_command(&state).as_deref(),
            Some("Normal[0, 1, 0]")
        );
        assert_eq!(
            prob_quantile_command(&state).as_deref(),
            Some("InverseNormal[0.95, 0, 1]")
        );
        let t = ProbabilityPanelState {
            dist: 3,
            df: 1.0,
            ..ProbabilityPanelState::default()
        };
        assert_eq!(distribution_moments(&t), (None, None));
        assert!(prob_case_command(&t).is_none());
        let u = ProbabilityPanelState {
            dist: 7,
            ua: 2.0,
            ub: 1.0,
            ..ProbabilityPanelState::default()
        };
        assert_eq!(distribution_moments(&u), (None, None));
    }

    #[test]
    fn sheet_import_csv_puro_pinnea_parseo_y_errores_honestos() {
        // Paridad con `panels::local_data_import_tests`: mismas expresiones.
        let csv = parse_local_xy_table("\"time\",\"distance\"\n0,1\n1,3\n2,5\n", b',')
            .expect("CSV con encabezado parsea");
        assert_eq!(csv.x_name, "time");
        assert_eq!(csv.y_name, "distance");
        assert_eq!(csv.xs, vec![0.0, 1.0, 2.0]);
        assert_eq!(csv.ys, vec![1.0, 3.0, 5.0]);
        let tsv = parse_local_xy_table("0\t1\n1\t3\n", b'\t').expect("TSV sin encabezado parsea");
        assert_eq!(tsv.x_name, "x");
        assert_eq!(tsv.xs, vec![0.0, 1.0]);
        for input in ["x,y\n0,1\n1\n", "x,y\n0,NaN\n", "x,y\n0,1\n"] {
            let error = parse_local_xy_table(input, b',').expect_err("fila inválida se rechaza");
            assert!(!error.is_empty());
        }
        // Comillas: escapada `""` y BOM inicial se toleran igual que en panels.
        let quoted = parse_local_xy_table("\"a\"\"b\",y\n0,1\n1,2\n", b',')
            .expect("comilla escapada parsea");
        assert_eq!(quoted.x_name, "a\"b");
        let bom =
            parse_local_xy_table("\u{feff}x,y\n0,1\n1,2\n", b',').expect("BOM inicial se recorta");
        assert_eq!(bom.x_name, "x");
    }

    #[test]
    fn sheet_hoja_cas_pura_pinnea_textos_de_comando() {
        // Paridad con `panels::gc_piel_tests`: mismas expresiones, dueño nuevo.
        assert_eq!(
            wc_study_command_text("f").as_deref(),
            Some("FunctionStudy[f]")
        );
        assert_eq!(wc_study_command_text("  "), None);
        assert_eq!(
            wc_riemann_command_text("x^2", "0", "1", 100.0, "trapecio").as_deref(),
            Some("RiemannSum[x^2, x, 0, 1, 100, trapecio]")
        );
        assert_eq!(
            wc_riemann_command_text("x^2", "0", "1", 99.6, "simpson").as_deref(),
            Some("RiemannSum[x^2, x, 0, 1, 100, simpson]")
        );
        assert_eq!(
            wc_riemann_command_text("", "0", "1", 100.0, "trapecio"),
            None
        );
        assert_eq!(
            wc_riemann_command_text("x^2", "0", "1", f32::NAN, "trapecio"),
            None
        );
        assert_eq!(
            wc_riemann_command_text("x^2", "0", "1", 0.4, "trapecio"),
            None
        );
        assert_eq!(
            wc_exact_integral_command_text("x^2", "0", "1").as_deref(),
            Some("Integral[x^2, x, 0, 1]")
        );
        assert_eq!(wc_exact_integral_command_text("", "0", "1"), None);
        assert_eq!(
            wc_taylor_command_text("sin(x)", "0", 5, "0.5").as_deref(),
            Some("Taylor[sin(x), x, 0, 5]")
        );
        assert_eq!(wc_taylor_command_text("sin(x)", "0", 0, "0.5"), None);
        assert_eq!(wc_taylor_command_text("sin(x)", "0", 11, "0.5"), None);
        assert_eq!(wc_taylor_command_text("sin(x)", "c", 5, "0.5"), None);
        assert_eq!(wc_taylor_command_text("", "0", 5, "0.5"), None);
        let vars = BTreeMap::new();
        let line = wc_taylor_remainder_line("sin(x)", "0", 5, "0.5", &vars).expect("resto");
        assert!(line.starts_with("P5(0.5) ≈ "));
        assert!(wc_taylor_remainder_line("[[[", "0", 5, "0.5", &vars).is_none());
        assert!(wc_taylor_remainder_line("sin(x)", "0", 5, "c", &vars).is_none());
    }

    #[test]
    fn sheet_serie_aplicar_pinnea_edits_y_textos() {
        // Paridad con `draw_sheet_series_row`: mismo motor que Aplicar + textos.
        let edits = build_series_edits("A1:A8", "1", "1", false).expect("lineal válida");
        assert_eq!(edits.len(), 8);
        assert_eq!(edits[0], (0, 0, "1".to_string()));
        assert_eq!(edits[7].2, "8".to_string());
        let geom = build_series_edits("A1:A4", "2", "3", true).expect("geométrica válida");
        assert_eq!(
            geom.iter().map(|(_, _, v)| v.clone()).collect::<Vec<_>>(),
            vec!["2", "6", "18", "54"]
        );
        // Coma decimal y errores honestos (mismos mensajes que el panel).
        assert_eq!(
            build_series_edits("A1:A2", "1,5", "1", false).expect("coma")[0].2,
            "1.5"
        );
        for (range, start, step) in [
            ("ZZZ", "1", "1"),
            ("A1:A3", "mal", "1"),
            ("A1:A3", "1", "inf"),
        ] {
            assert!(build_series_edits(range, start, step, false).is_err());
        }
        // Vista previa delega al mismo builder (paridad con `series_preview`).
        let (count, first) = series_preview("A1:A8", "1", "1", false).expect("preview");
        assert_eq!(
            format_series_preview_text(count, &first),
            "8 celdas: 1, 2, 3…"
        );
        assert_eq!(
            format_series_preview_text(2, &["1".to_string(), "2".to_string()]),
            "2 celdas: 1, 2"
        );
        assert_eq!(
            format_series_success(3, "  A1:A3 "),
            "Serie: 3 celda(s) rellenadas en A1:A3"
        );
    }

    #[test]
    fn sheet_celda_lectura_pinnea_display_y_tokens() {
        // Paridad con `draw_sheet_editable_grid`: misma rama de lectura.
        assert_eq!(SHEET_CELL_H, 24.0);
        assert_eq!(SHEET_CONTROL_H, 28.0);
        let fmt = format_statistic;
        // Vacía → vacío.
        assert_eq!(
            sheet_cell_display(None, "   ", None, fmt),
            (String::new(), String::new(), SheetCellKind::Empty)
        );
        // Valor con fórmula → hover con la fuente.
        let (text, hover, kind) = sheet_cell_display(Some(2.5), "=A1+B1", None, fmt);
        assert_eq!(kind, SheetCellKind::Value);
        assert_eq!(text, fmt(2.5));
        assert_eq!(hover, "=A1+B1");
        // Valor sin fórmula → hover vacío (el dibujo pone `C3 = valor`).
        let (text, hover, kind) = sheet_cell_display(Some(2.5), "2.5", None, fmt);
        assert_eq!((kind, hover.as_str()), (SheetCellKind::Value, ""));
        assert_eq!(text, fmt(2.5));
        // Sin valor y sin error → "—" sin hover.
        assert_eq!(
            sheet_cell_display(None, "=1/0", None, fmt),
            ("—".to_string(), String::new(), SheetCellKind::Error)
        );
        // El error manda sobre todo (misma precedencia que el dibujo).
        assert_eq!(
            sheet_cell_display(Some(1.0), "=A1", Some("C3: roto"), fmt),
            (
                "—".to_string(),
                "C3: roto".to_string(),
                SheetCellKind::Error
            )
        );
    }

    #[test]
    fn sheet_estados_efimeros_pinnea_defaults_del_dibujo() {
        // Paridad con `draw_sheet_editable_grid`/`draw_sheet_series_row`/
        // `draw_sheet_summary_card`: mismos defaults que `ctx.data` usa vía
        // `unwrap_or_default()` (slice 13 mueve solo las definiciones).
        let edit = SheetEditState::default();
        assert!(edit.editing.is_none());
        assert!(edit.drafts.is_empty());
        assert!(edit.errors.is_empty());
        assert!(edit.goto.is_empty());
        assert!(edit.goto_error.is_none());
        let serie = SheetSeriesState::default();
        assert_eq!(
            (
                serie.range.as_str(),
                serie.start.as_str(),
                serie.step.as_str()
            ),
            ("A1:A8", "1", "1")
        );
        assert!(!serie.geometric);
        assert!(serie.error.is_none());
        let resumen = SheetSummaryState::default();
        assert_eq!(resumen.range.as_str(), "A1:A8");
        assert!(resumen.summary.is_none());
        assert!(resumen.error.is_none());
    }

    #[test]
    fn sheet_tabla_vista_pinnea_subtitulo_y_avisos() {
        // Paridad con `draw_spreadsheet_section`: mismas cadenas y cotas.
        assert_eq!(sheet_table_rows(5, 3), 3);
        assert_eq!(sheet_table_rows(0, 4), 0);
        assert_eq!(sheet_table_shown(5), 5);
        assert_eq!(sheet_table_shown(500), MAX_SPREADSHEET_ROWS);
        assert_eq!(sheet_table_subtitle("ejemplo", 5), "ejemplo · 5 filas");
        assert_eq!(
            sheet_table_subtitle("<sin etiqueta>", 0),
            "<sin etiqueta> · 0 filas"
        );
        assert_eq!(sheet_table_rows_overflow(5, 5), None);
        assert_eq!(
            sheet_table_rows_overflow(51, 50),
            Some("…y 1 filas más (vista acotada a 50)".to_string())
        );
        assert_eq!(sheet_hidden_tables_overflow(8), None);
        assert_eq!(sheet_hidden_tables_overflow(0), None);
        assert_eq!(
            sheet_hidden_tables_overflow(9),
            Some("…y 1 tablas más (vista acotada a 8)".to_string())
        );
    }

    #[test]
    fn sheet_anchos_puros_pinnea_las_formulas_del_dibujo() {
        // Paridad con `draw_sheet_series_row` / `draw_sheet_summary_card` /
        // `stat_tile_row` en `panels.rs`: mismas fórmulas, dueño nuevo.
        assert_eq!(SHEET_SUMMARY_BTN_W, 84.0);
        // Serie: mitad del ancho menos espaciado, piso 0.
        assert_eq!(sheet_series_field_w(300.0), (300.0 - SPACE_SM) / 2.0);
        assert_eq!(sheet_series_field_w(0.0), 0.0);
        // Resumen: ancho menos botón y espaciado, piso táctil.
        assert_eq!(
            sheet_summary_field_w(300.0),
            300.0 - SHEET_SUMMARY_BTN_W - SPACE_SM
        );
        assert_eq!(sheet_summary_field_w(0.0), HIT_TARGET_MIN);
        // Tiles: reparto exacto del disponible, piso 0, cero con count 0.
        assert_eq!(
            stat_tile_width_for(300.0, 3),
            (300.0 - 2.0 * SPACE_SM) / 3.0
        );
        assert_eq!(stat_tile_width_for(300.0, 1), 300.0);
        assert_eq!(stat_tile_width_for(0.0, 2), 0.0);
        assert_eq!(stat_tile_width_for(300.0, 0), 0.0);
    }

    #[test]
    fn sheet_trig_layout_puro_pinnea_columnas_y_celdas() {
        // Paridad con `draw_trig_animation_panel` en `panels.rs`: mismas
        // cotas y fórmulas, dueño nuevo (slice 16).
        assert_eq!(TRIG_CANVAS_MIN_W, 320.0);
        assert_eq!(TRIG_PANEL_DEFAULT_W, 300.0);
        assert_eq!(TRIG_PANEL_MIN_W, 248.0);
        assert_eq!(TRIG_PANEL_MAX_W, 400.0);
        assert_eq!(TRIG_CHIP_MIN_W, 74.0);
        assert_eq!(TRIG_APPLY_W, 76.0);
        // `TRIG_CONTROL_H` es `PANEL_BUTTON_H` (= 28.0) pineado como literal
        // para no acoplar este módulo puro al dibujo.
        assert_eq!(TRIG_CONTROL_H, 28.0);
        assert_eq!(TRIG_CONTROL_H, SHEET_CONTROL_H);
        // 6 chips entran en ancho sobrado, 3 en panel normal, 2 en el piso.
        assert_eq!(trig_function_columns(10_000.0), 6);
        assert_eq!(trig_function_columns(300.0), 3);
        assert_eq!(trig_function_columns(0.0), 2);
        // Celdas + gaps nunca superan el disponible en anchos reales, piso 0,
        // columns 0 no divide por cero.
        for (available, columns) in [(300.0_f32, 6), (300.0, 3), (300.0, 2)] {
            let cell_w = trig_grid_cell_w(available, columns);
            let total = columns as f32 * cell_w + (columns as f32 - 1.0) * SPACE_SM;
            assert!(total <= available + 1e-4, "{available} {columns} {total}");
            assert!(cell_w >= 0.0);
        }
        assert_eq!(trig_grid_cell_w(300.0, 0), trig_grid_cell_w(300.0, 1));
        assert_eq!(trig_grid_cell_w(0.0, 3), 0.0);
    }

    #[test]
    fn sheet_inspector_textos_puros_pinnea_titular_y_etiquetas() {
        // Paridad con `draw_inspector_identity` en `panels.rs`: mismas
        // cadenas, dueño nuevo (slice 17).
        use grafito_core::{GeoObject, LineStyle, ParametricCurve3DObj, PointStyle, Surface3DObj};
        let surface = GeoObject::Surface3D(Surface3DObj::new("x^2+y^2", (-2.0, 2.0), (-2.0, 2.0)));
        assert_eq!(
            inspector_equation_text(&surface).as_deref(),
            Some("z = x^2+y^2")
        );
        let curve =
            GeoObject::ParametricCurve3D(ParametricCurve3DObj::new("t", "t^2", "t^3", 0.0, 1.0));
        assert_eq!(
            inspector_equation_text(&curve).as_deref(),
            Some("(t, t^2, t^3)")
        );
        assert_eq!(inspector_type_caption(&surface), "superficie 3D");
        assert_eq!(inspector_label_text(""), None);
        assert_eq!(inspector_label_text("S").as_deref(), Some("Etiqueta: S"));
        let tip = inspector_identity_tooltip("z = x^2+y^2", "S", true);
        assert!(tip.contains("z = x^2+y^2"));
        assert!(tip.contains("Etiqueta: S"));
        assert!(tip.contains("Visible"));
        assert_eq!(line_style_label(LineStyle::Solid), "Continuo");
        assert_eq!(point_style_label(PointStyle::Plus), "Cruz +");
    }

    #[test]
    fn sheet_protocolo_puro_pinnea_latex_y_reproduccion() {
        // Paridad con `draw_construction_protocol` en `panels.rs`: mismas
        // cadenas LaTeX y misma máquina de pasos, dueño nuevo (slice 18).
        use crate::app::ConstructionStep;
        assert_eq!(escape_latex("100%_a&b"), "100\\%\\_a\\&b");
        assert_eq!(escape_latex("a{b}\\c"), "a\\{b\\}\\textbackslash\\{\\}c");
        let log = vec![
            ConstructionStep {
                n: 0,
                action: "Punto".to_string(),
                inputs: vec![],
                output: "A".to_string(),
                disabled: false,
                timestamp: 0.0,
            },
            ConstructionStep {
                n: 1,
                action: "Recta_50%".to_string(),
                inputs: vec!["A".to_string(), "B".to_string()],
                output: "".to_string(),
                disabled: true,
                timestamp: 1.0,
            },
        ];
        let latex = construction_log_to_latex(&log);
        assert!(latex.starts_with("% Protocolo de Construcción — Grafito\n"));
        assert!(latex.contains("\\begin{enumerate}"));
        // Ojo: el `\textemdash` de vacíos pasa por `escape_latex` (verbatim
        // de `panels.rs`): la barra se escapa a `\textbackslash\{\}`.
        assert!(latex.contains("\\item Punto: \\textbackslash\\{\\}textemdash $\\rightarrow$ A"));
        assert!(latex.contains("(deshabilitado)"));
        assert!(latex.contains("Recta\\_50\\%"));
        assert!(latex.ends_with("\\end{enumerate}\n"));
        // Reproducción: arranca en 0, avanza de a uno, frena al final.
        assert_eq!(PROTOCOL_PLAY_INTERVAL_SECS, 1.0);
        let mut state = ProtocolNavState {
            playing: true,
            ..Default::default()
        };
        protocol_advance(&mut state, 3, 2.0);
        assert_eq!(state.cursor, Some(0));
        protocol_advance(&mut state, 3, 2.5);
        assert_eq!(state.cursor, Some(0));
        protocol_advance(&mut state, 3, 3.1);
        assert_eq!(state.cursor, Some(1));
        protocol_advance(&mut state, 0, 99.0);
        assert_eq!(state.cursor, Some(1));
        assert!(state.playing);
    }

    #[test]
    fn sheet_edicion_panel_pinnea_reemplazo_estadificado() {
        // Paridad con `tests::changed_object_panel_edit_bumps_version_once`:
        // misma primitiva, dueño nuevo (slice 19).
        use grafito_core::{Cube3DObj, Document, GeoObject, ObjectId};
        use grafito_geometry::Point3D;

        let mut document = Document::new();
        let cube = document.add_object(GeoObject::Cube3D(Cube3DObj::new(
            Point3D::new(0.0, 0.0, 0.0),
            2.0,
        )));
        let version_before = document.version;
        // Idle (changed=false) no toca el documento ni corre el closure.
        assert!(
            !apply_object_panel_edit(&mut document, cube, false, |_| panic!(
                "idle edit must not run"
            ))
            .expect("idle es no-op")
        );
        assert_eq!(document.version, version_before);
        // Id inexistente con changed=true: Ok(None)/false, sin pánico.
        let missing = ObjectId::new();
        assert!(
            apply_object_panel_edit_with_previous(&mut document, missing, true, |_| {})
                .expect("faltante es no-op")
                .is_none()
        );
        // Edición real: bump único de versión + previo para el undo.
        let before = apply_object_panel_edit_with_previous(&mut document, cube, true, |object| {
            let GeoObject::Cube3D(cube) = object else {
                panic!("fixture sigue cubo");
            };
            cube.size = 3.0;
        })
        .expect("edición válida")
        .expect("devuelve el previo");
        assert_eq!(document.version, version_before.wrapping_add(1));
        assert!(matches!(
            before.get_object(cube),
            Some(GeoObject::Cube3D(cube)) if cube.size == 2.0
        ));
        assert!(!apply_object_panel_edit(&mut document, cube, true, |_| {})
            .expect("candidata sin cambio es no-op"));
    }

    #[test]
    fn sheet_complejos_setters_pinnea_noop_y_mutacion() {
        // Paridad con `draw_right_domain_coloring_panel` en `panels.rs`:
        // mismos setters, dueño nuevo (slice 20).
        use grafito_core::{ComplexGridObj, ComplexMappingObj, Document, GeoObject, ObjectId};

        let mut document = Document::new();
        let grid = document.add_object(GeoObject::ComplexGrid(ComplexGridObj::new(
            "z", -1.0, 1.0, -1.0, 1.0,
        )));
        let mapping =
            document.add_object(GeoObject::ComplexMapping(ComplexMappingObj::new("z", grid)));
        // Mismo valor → no-op.
        assert!(!set_domain_coloring_mode(&mut document, grid, 0));
        // Cambio real → true y queda aplicado.
        assert!(set_domain_coloring_mode(&mut document, grid, 2));
        assert!(matches!(
            document.get_object(grid),
            Some(GeoObject::ComplexGrid(g)) if g.domain_coloring_mode == 2
        ));
        // Repetir → no-op.
        assert!(!set_domain_coloring_mode(&mut document, grid, 2));
        // Tipo equivocado y faltante → no-op.
        assert!(!set_domain_coloring_mode(&mut document, mapping, 1));
        assert!(!set_domain_coloring_mode(&mut document, ObjectId::new(), 1));
        // Homotopía: defaults (false, 1.0) → no-op.
        assert!(!set_complex_mapping_animation(
            &mut document,
            mapping,
            false,
            1.0
        ));
        assert!(set_complex_mapping_animation(
            &mut document,
            mapping,
            true,
            2.0
        ));
        assert!(matches!(
            document.get_object(mapping),
            Some(GeoObject::ComplexMapping(m))
                if m.animate_homotopy && m.homotopy_speed == 2.0
        ));
        assert!(!set_complex_mapping_animation(
            &mut document,
            mapping,
            true,
            2.0
        ));
        // Solo velocidad cambia → también muta.
        assert!(set_complex_mapping_animation(
            &mut document,
            mapping,
            true,
            3.0
        ));
        assert!(!set_complex_mapping_animation(
            &mut document,
            grid,
            true,
            3.0
        ));
        assert!(!set_complex_mapping_animation(
            &mut document,
            ObjectId::new(),
            true,
            1.0
        ));
    }

    #[test]
    fn sheet_export_grid_pinnea_columnas_y_celdas() {
        // Paridad con `draw_export_section` en `panels.rs`: mismas fórmulas,
        // dueño nuevo (slice 21). 3×3 en panel normal, 1 bajo 200 px.
        assert_eq!(export_grid_columns(360.0), 3);
        assert_eq!(export_grid_columns(200.0), 3);
        assert_eq!(export_grid_columns(199.9), 1);
        assert_eq!(export_grid_columns(0.0), 1);
        for (available, columns) in [(360.0_f32, 3), (252.0, 3), (199.0, 1), (0.0, 1)] {
            let cols = export_grid_columns(available);
            assert_eq!(cols, columns);
            let cell_w = export_grid_cell_w(available, cols);
            let total = cols as f32 * cell_w + (cols as f32 - 1.0) * SPACE_SM;
            assert!(total <= available + 1e-4, "{available} {cols} {total}");
            assert!(cell_w >= 0.0);
        }
        assert_eq!(export_grid_cell_w(300.0, 0), export_grid_cell_w(300.0, 1));
        assert_eq!(export_grid_cell_w(0.0, 3), 0.0);
    }

    #[test]
    fn sheet_wc_visual_pinnea_estado_y_metodos() {
        // Paridad con `draw_wc_*` en `panels.rs`: mismas cadenas y mismo
        // default, dueño nuevo (slice 22). El dibujo queda en panels.
        assert_eq!(WC_QUADRATURE_METHODS.len(), 3);
        assert_eq!(WC_QUADRATURE_METHODS[0], ("midpoint", "Punto medio"));
        assert_eq!(WC_QUADRATURE_METHODS[2], ("simpson", "Simpson"));
        assert_eq!(WC_RIEMANN_SLIDER_MAX, 10_000.0);
        let wc = WcVisualState::default();
        assert_eq!(wc.estudio_label, "f");
        assert_eq!(wc.integral_expr, "x^2");
        assert_eq!(wc.integral_n, 100.0);
        assert_eq!(wc.integral_metodo, 0);
        assert_eq!(wc.taylor_orden, 5);
        // Helper con clamp: índice rancio no paniquea, cae a Simpson.
        assert_eq!(wc_quadrature_method(0), ("midpoint", "Punto medio"));
        assert_eq!(wc_quadrature_method(2), ("simpson", "Simpson"));
        assert_eq!(wc_quadrature_method(99), ("simpson", "Simpson"));
        assert_eq!(wc_quadrature_key(99), "simpson");
        assert_eq!(wc_quadrature_key(usize::MAX), "simpson");
    }

    #[test]
    fn sheet_trig_y_capas_pinnea_estados_efimeros() {
        // Paridad con `draw_trig_speed_prompt`/`draw_view_panel` en
        // `panels.rs`: mismos defaults que `ctx.data` usa vía
        // `unwrap_or_default()` (slice 23 mueve solo las definiciones).
        let capas = LayerPanelState::default();
        assert_eq!(capas.selected_layer, 0);
        let prompt = TrigPromptState::default();
        assert!(prompt.text.is_empty());
        assert!(prompt.error.is_none());
    }

    #[test]
    fn sheet_import_objetos_pinnea_tabla_y_dispersion_linkeada() {
        // Paridad con `commit_local_xy_table` en `panels.rs`: mismo armado
        // `DataTable` + `ScatterPlot` linkeada (slice 24 mueve solo el
        // armado puro; el commit con undo queda en panels).
        use super::LocalXYTable;
        use grafito_core::GeoObject;
        let tabla = LocalXYTable {
            x_name: "t".to_string(),
            y_name: "d".to_string(),
            xs: vec![0.0, 1.0, 2.0],
            ys: vec![1.0, 3.0, 5.0],
        };
        let (data_obj, scatter_obj, data_id) = local_xy_table_objects(tabla);
        let GeoObject::DataTable(data) = &data_obj else {
            panic!("se esperaba DataTable");
        };
        assert_eq!(data.id, data_id);
        assert_eq!(data.x_name, "t");
        assert_eq!(data.y_name, "d");
        assert_eq!(data.xs, vec![0.0, 1.0, 2.0]);
        assert_eq!(data.ys, vec![1.0, 3.0, 5.0]);
        let GeoObject::ScatterPlot(scatter) = &scatter_obj else {
            panic!("se esperaba ScatterPlot");
        };
        assert_eq!(scatter.source_data, Some(data_id));
        assert_eq!(scatter.xs, vec![0.0, 1.0, 2.0]);
        assert_eq!(scatter.ys, vec![1.0, 3.0, 5.0]);
    }

    #[test]
    fn sheet_trig_lecturas_pinnea_textos_del_dibujo() {
        // Paridad con `draw_trig_animation_panel` en `panels.rs`: mismas
        // cadenas, dueño nuevo (slice 25). El dibujo queda en panels.
        assert_eq!(trig_function_chip_label("sin"), "sin(t)");
        assert_eq!(trig_speed_text(0.5), "0.5 rad/s");
        assert_eq!(trig_angle_text(0.0), "0.00 rad");
        assert_eq!(trig_value_text("sin", 0.0, 0.0), "sin(0.00) = 0.0000");
        assert_eq!(
            trig_value_text("tan", 1.57, f64::INFINITY),
            "tan(1.57) no está definido"
        );
        assert_eq!(
            trig_point_text(0.0),
            format!(
                "Punto: (cos θ, sin θ) = ({:.3}, {:.3})",
                0.0_f64.cos(),
                0.0_f64.sin()
            )
        );
        assert_eq!(
            trig_complex_text(0.0),
            format!(
                "z = {:.3} {:+.3}i  |z| = 1  arg(z) = {:.2}",
                0.0_f64.cos(),
                0.0_f64.sin(),
                0.0
            )
        );
    }

    #[test]
    fn sheet_trig_b_a162_lecturas_son_notacion_si_invariante() {
        // B-A162 (barrido final saga trig): `trig_speed_text` /
        // `trig_angle_text` / `trig_complex_text` + chip `{name}(t)` son
        // notación SI/matemática idéntica en los 6 idiomas por diseño
        // (precedentes B-A114/B-A160/B-A98: unidades `rad/s`/`rad`, `z`/
        // `θ`/`|z|`/`arg`, fórmula `z(t)`) y NO tienen key `trig.*`. Si
        // algún día se les da key, este test obliga a actualizar la
        // decisión + `trig_b_a162_...` en `panels.rs`.
        assert_eq!(trig_speed_text(1.0), "1.0 rad/s");
        assert_eq!(trig_speed_text(-6.0), "-6.0 rad/s");
        assert!(trig_speed_text(0.5).ends_with(" rad/s"));
        assert_eq!(trig_angle_text(1.0), "1.00 rad");
        assert!(trig_angle_text(0.0).ends_with(" rad"));
        assert!(!trig_angle_text(0.0).ends_with("/s"));
        let complex = trig_complex_text(0.0);
        for needle in ["z =", "|z| = 1", "arg(z) =", "i"] {
            assert!(
                complex.contains(needle),
                "notación compleja invariante: {needle} en {complex}"
            );
        }
        assert_eq!(trig_function_chip_label("cos"), "cos(t)");
        // Sin key `trig.speed_*/angle_*/complex_*` para lecturas: notación sin
        // display-localizar (TRIG_KEYS hoy 33 con los 6 `prompt_err_*` B-A161;
        // `complex_desc` es descripción de vista B-A98, no lectura).
        assert_eq!(grafito_ui::i18n::TRIG_KEYS.len(), 33);
        for suffix in grafito_ui::i18n::TRIG_KEYS {
            for denied in [
                "speed_text",
                "speed_readout",
                "angle_text",
                "angle_readout",
                "complex_text",
                "complex_readout",
                "readout_speed",
                "readout_angle",
            ] {
                assert!(*suffix != denied, "lectura sin key a propósito: {suffix}");
            }
        }
    }

    #[test]
    fn sheet_trig_b_a98_helpers_localizan_sin_romper_el_canonico() {
        // B-A98: los helpers `_for`/`_label` salen de `trig.*`; el Es es
        // byte-idéntico al canónico por construcción (rama Es delega).
        use grafito_ui::i18n::Locale;
        // Paridad Es con los canónicos pineados arriba.
        assert_eq!(trig_prompt_auto_label(Locale::Es), TRIG_PROMPT_AUTO_LABEL);
        assert_eq!(trig_prompt_hint_text(Locale::Es), TRIG_PROMPT_HINT);
        assert_eq!(
            trig_prompt_field_tooltip(Locale::Es),
            TRIG_PROMPT_FIELD_TOOLTIP
        );
        assert_eq!(trig_prompt_apply_label(Locale::Es), TRIG_PROMPT_APPLY_LABEL);
        assert_eq!(
            trig_prompt_apply_tooltip(Locale::Es),
            TRIG_PROMPT_APPLY_TOOLTIP
        );
        assert_eq!(
            trig_value_text_for("tan", 1.57, f64::INFINITY, Locale::Es),
            trig_value_text("tan", 1.57, f64::INFINITY)
        );
        assert_eq!(trig_point_text_for(0.0, Locale::Es), trig_point_text(0.0));
        // El resto de idiomas resuelve sin vacíos ni fallback a la clave.
        for locale in [Locale::En, Locale::Pt, Locale::It, Locale::Fr, Locale::De] {
            assert!(!trig_prompt_auto_label(locale).is_empty());
            assert!(!trig_prompt_hint_text(locale).is_empty());
            assert!(!trig_prompt_field_tooltip(locale).is_empty());
            assert!(!trig_prompt_apply_label(locale).is_empty());
            assert!(!trig_prompt_apply_tooltip(locale).is_empty());
            let undef = trig_value_text_for("tan", 1.57, f64::INFINITY, locale);
            assert!(
                undef.contains("tan(1.57)"),
                "notación intacta en {locale:?}"
            );
            assert_ne!(undef, trig_value_text("tan", 1.57, f64::INFINITY));
            assert!(trig_point_text_for(0.0, locale).contains("(cos θ, sin θ)"));
        }
        // Pines por idioma (detectan swaps de overlay).
        assert_eq!(trig_prompt_apply_label(Locale::En), "Apply");
        assert_eq!(trig_prompt_hint_text(Locale::En), "1.5 · medium");
        assert_eq!(trig_prompt_auto_label(Locale::De), "Auto");
        assert_eq!(
            trig_value_text_for("tan", 1.57, f64::INFINITY, Locale::Pt),
            "tan(1.57) não está definido"
        );
        assert!(trig_point_text_for(0.0, Locale::Fr).starts_with("Point:"));
    }

    #[test]
    fn sheet_capas_textos_puros_pinnea_filas_y_resultados() {
        // Paridad con `draw_view_panel` en `panels.rs`: mismas cadenas,
        // dueño nuevo (slice 26). El dibujo queda en panels.
        assert_eq!(
            LAYER_EMPTY_HINT,
            "Sin objetos: todo lo nuevo entra en la capa 0."
        );
        assert_eq!(
            LAYER_NO_SELECTION,
            "Sin selección: elegí un objeto en el lienzo."
        );
        assert_eq!(layer_display_name(""), "<sin etiqueta>");
        assert_eq!(layer_display_name("C"), "C");
        assert_eq!(layer_row_label(2, 5), "Capa 2 · 5 obj");
        assert_eq!(layer_row_label(0, 1), "Capa 0 · 1 obj");
        assert_eq!(
            layer_toggle_result(2, true, 3),
            Some("Capa 2 oculta (3 obj)".to_string())
        );
        assert_eq!(
            layer_toggle_result(2, false, 1),
            Some("Capa 2 visible (1 obj)".to_string())
        );
        assert_eq!(layer_toggle_result(2, true, 0), None);
        assert_eq!(layer_selection_text("C", 2), "Selección: C (capa 2)");
        assert_eq!(
            layer_selection_text("", 0),
            "Selección: <sin etiqueta> (capa 0)"
        );
        assert_eq!(layer_assign_result("C", 2), "'C' → capa 2");
        assert_eq!(layer_assign_result("", 1), "'<sin etiqueta>' → capa 1");
        assert_eq!(
            layer_assign_error("capa 99 fuera de cota"),
            "No se pudo asignar: capa 99 fuera de cota"
        );
    }

    #[test]
    fn sheet_trig_prompt_campo_pinnea_formula_del_dibujo() {
        // Paridad con `draw_trig_speed_prompt` en `panels.rs`: misma
        // fórmula, dueño nuevo (slice 27). Campo + botón nunca desbordan.
        assert_eq!(TRIG_APPLY_W, 76.0);
        assert_eq!(
            trig_prompt_field_w(300.0),
            300.0 - TRIG_APPLY_W - 2.0 * SPACE_SM
        );
        assert_eq!(trig_prompt_field_w(0.0), 0.0);
        for available in [200.0_f32, 252.0, 300.0, 360.0] {
            let field_w = trig_prompt_field_w(available);
            let total = field_w + TRIG_APPLY_W + 2.0 * SPACE_SM;
            assert!(total <= available + 1e-4, "{available} {total}");
            assert!(field_w >= 0.0);
        }
    }

    #[test]
    fn sheet_prob_plot_pinnea_ventana_bandas_y_rotulos() {
        // Paridad con `draw_probability_plot` en `panels.rs`: mismos
        // clamps, ventana, bandas y rótulos, dueño nuevo (slice 28).
        // El dibujo queda en panels.
        assert_eq!(clamp_binomial_n(10.4), 10);
        assert_eq!(clamp_binomial_n(0.0), 1);
        assert_eq!(clamp_binomial_n(1e9), MAX_BINOMIAL_N);
        assert_eq!(clamp_binomial_k(5.0, 10), 5);
        assert_eq!(clamp_binomial_k(99.0, 10), 10);
        assert_eq!(clamp_binomial_k(-3.0, 10), 0);
        assert_eq!(binomial_plot_window(10, 0.5, 5), (0, 10));
        assert_eq!(binomial_plot_left_label(0), "k=0");
        assert_eq!(
            binomial_plot_right_label(10, 0, 10),
            "k ≤ seleccionado sombreado"
        );
        // n > 61: ventana de 61 alrededor de la media, extendida a k_sel.
        let n = clamp_binomial_n(200.0);
        let k_sel = clamp_binomial_k(190.0, n);
        let (lo, hi) = binomial_plot_window(n, 0.5, k_sel);
        assert!(hi - lo >= 60 && lo <= k_sel && hi >= k_sel);
        assert_eq!(
            binomial_plot_right_label(n, lo, hi),
            format!("ventana {lo}..={hi} de 0..={n} · ≤k sombreado")
        );
        // Bandas μ=0 σ=1 en [-4, 4]: k=3 → (1/8, 7/8), k=1 → (3/8, 5/8).
        let bands = normal_band_fractions(0.0, 1.0, -4.0, 4.0);
        assert!((bands[0].0 - 0.125_f32).abs() < 1e-6);
        assert!((bands[0].1 - 0.875_f32).abs() < 1e-6);
        assert_eq!(bands[0].2, 0.05_f32);
        assert!((bands[2].0 - 0.375_f32).abs() < 1e-6);
        assert!((bands[2].1 - 0.625_f32).abs() < 1e-6);
        assert_eq!(bands[2].2, 0.10_f32);
    }

    #[test]
    fn sheet_prob_ventanas_restantes_pinnea_plot() {
        // Paridad con `draw_probability_plot` en `panels.rs`: mismas
        // ventanas, clamps y rótulos, dueño nuevo (slice 29).
        // El dibujo queda en panels.
        assert_eq!(normal_plot_window(0.0, 1.0), Some((-4.0, 4.0)));
        assert_eq!(normal_plot_window(0.0, 0.0), None);
        assert_eq!(normal_plot_window(f64::NAN, 1.0), None);
        assert_eq!(normal_plot_window(0.0, f64::INFINITY), None);
        assert_eq!((T_PLOT_LO, T_PLOT_HI), (-6.0, 6.0));
        assert_eq!(discrete_k_sel(5.4), 5);
        assert_eq!(discrete_k_sel(-3.0), 0);
        // Poisson λ=3, k_sel=3: hi_default=(9+10).ceil()=19 → 19.
        assert_eq!(poisson_plot_hi(3.0, 3), Some(19));
        assert_eq!(poisson_plot_hi(0.0, 0), None);
        assert_eq!(poisson_plot_hi(f64::NAN, 0), None);
        // λ enorme cubre k_sel y respeta el tope de UI.
        let hi_big = poisson_plot_hi(1_000_000.0, 0).expect("λ válida");
        assert!(hi_big <= MAX_POISSON_K);
        // Geométrica p=0.5 (media 1): hi_default=(3+10).ceil()=13.
        assert_eq!(geometric_plot_hi(0.5, 0), Some(13));
        assert_eq!(geometric_plot_hi(0.0, 0), None);
        assert_eq!(geometric_plot_hi(1.5, 0), None);
        assert_eq!(geometric_plot_hi(f64::NAN, 0), None);
        assert_eq!(discrete_plot_left_label(), "k=0");
        assert_eq!(discrete_plot_right_label(19), "0..=19 · ≤k sombreado");
        // χ² gl=10, x=5: hi=48; F x=5: hi=6; x no finita → None.
        assert_eq!(chi_squared_plot_hi(10.0, 5.0), Some(48.0));
        assert_eq!(chi_squared_plot_hi(10.0, f64::NAN), None);
        assert_eq!(f_plot_hi(5.0), Some(6.0));
        assert_eq!(f_plot_hi(10.0), Some(10.0));
        assert_eq!(f_plot_hi(f64::NAN), None);
    }

    #[test]
    fn sheet_ejemplo_pinnea_tabla_y_textos() {
        // Paridad con `draw_spreadsheet_section` en `panels.rs`: misma
        // tabla x: 0..4 / y: x² con etiqueta "ejemplo" y mismas cadenas,
        // dueño nuevo (slice 30). El commit con undo queda en panels.
        let table = example_sheet_table();
        match &table {
            grafito_core::GeoObject::DataTable(obj) => {
                assert_eq!(obj.label, "ejemplo");
                assert_eq!(obj.x_name, "x");
                assert_eq!(obj.y_name, "y");
                assert_eq!(obj.xs, vec![0.0, 1.0, 2.0, 3.0, 4.0]);
                assert_eq!(obj.ys, vec![0.0, 1.0, 4.0, 9.0, 16.0]);
            }
            otro => panic!("ejemplo debe ser DataTable, fue {otro:?}"),
        }
        assert_eq!(example_sheet_table_loaded_text(), "Tabla «ejemplo» cargada");
        assert_eq!(
            example_sheet_table_error_text("duplicada"),
            "No se pudo cargar el ejemplo: duplicada"
        );
    }

    #[test]
    fn sheet_prob_resultado_pinnea_ramas_y_errores_honestos() {
        // Paridad con el closure inline de `draw_probability_section` en
        // `panels.rs`: mismas 9 ramas, mismos detalles y mismos errores
        // honestos, dueño nuevo (slice 31). El dibujo queda en panels.
        let normal = ProbabilityPanelState::default();
        let (pdf, cdf, detail) = prob_case_result(&normal).expect("normal default calcula");
        assert!((pdf - 0.398_942_280_401_432_7).abs() < 1e-6);
        assert!((cdf - 0.5).abs() < 1e-6);
        assert_eq!(detail, format!("f({})", super::format_statistic(0.0)));
        let binomial = ProbabilityPanelState {
            dist: 1,
            n: 10.0,
            p: 0.5,
            k: 5.0,
            ..ProbabilityPanelState::default()
        };
        let (pmf, cdf_b, detail_b) = prob_case_result(&binomial).expect("binomial calcula");
        assert!((pmf - 0.246_093_75).abs() < 1e-9);
        assert!((cdf_b - 0.623_046_875).abs() < 1e-9);
        assert_eq!(detail_b, "P(X = 5) · n = 10");
        let mala = ProbabilityPanelState {
            sigma: 0.0,
            ..ProbabilityPanelState::default()
        };
        let error = prob_case_result(&mala).expect_err("σ no positiva se rechaza");
        assert!(!error.is_empty());
    }

    #[test]
    fn sheet_prob_selector_pinnea_nombres_tipos_y_rotulos() {
        use super::{
            prob_dist_kind_label_for, prob_dist_name_for, prob_dist_tooltip_for,
            prob_point_label_for, PROB_DISTS,
        };
        use grafito_ui::i18n::Locale as L;
        // Paridad con los inline de `draw_probability_section` en
        // `panels.rs`: mismo nombre con caída "Normal", mismo tooltip
        // "{kind} · {tip}" y mismo rótulo "Puntual"/"Densidad", dueño
        // nuevo (slice 32). El combo y el dibujo quedan en panels.
        // B-A156: vía `_for` con Es (los wrappers Es se eliminaron; Es
        // delega al canónico por construcción).
        for (index, name, tip) in PROB_DISTS {
            assert_eq!(prob_dist_name_for(index, L::Es), name);
            let discreta = matches!(index, 1 | 2 | 6);
            assert_eq!(
                prob_dist_kind_label_for(index, L::Es),
                if discreta { "Discreta" } else { "Continua" }
            );
            assert_eq!(
                prob_dist_tooltip_for(index, L::Es),
                Some(format!(
                    "{} · {tip}",
                    if discreta { "Discreta" } else { "Continua" }
                ))
            );
            assert_eq!(
                prob_point_label_for(index, L::Es),
                if discreta { "Puntual" } else { "Densidad" }
            );
        }
        // Índice desconocido: el nombre cae a "Normal" pero el tipo se
        // trata como continuo (misma asimetría que el inline original:
        // `find` falla para el nombre/tooltip, `is_discrete(99)` es falso).
        assert_eq!(prob_dist_name_for(99, L::Es), "Normal");
        assert_eq!(prob_dist_tooltip_for(99, L::Es), None);
        assert_eq!(prob_dist_kind_label_for(99, L::Es), "Continua");
        assert_eq!(prob_point_label_for(99, L::Es), "Densidad");
    }

    #[test]
    fn sheet_prob_b_a156_helpers_for_delegan_es_al_canonico() {
        use super::{
            prob_dist_kind_label_for, prob_dist_name_for, prob_dist_tooltip_for,
            prob_point_label_for, PROB_DISTS,
        };
        use grafito_ui::i18n::Locale as L;
        // Paridad Es: los helpers `_for` devuelven el canónico (mismo
        // `PROB_DISTS` + mismas ramas discreta/continua que los wrappers
        // Es). Resto de idiomas: resuelven sin fallback a la clave.
        for (index, name, tip) in PROB_DISTS {
            assert_eq!(prob_dist_name_for(index, L::Es), name);
            let discreta = matches!(index, 1 | 2 | 6);
            assert_eq!(
                prob_dist_kind_label_for(index, L::Es),
                if discreta { "Discreta" } else { "Continua" }
            );
            assert_eq!(
                prob_dist_tooltip_for(index, L::Es),
                Some(format!(
                    "{} · {tip}",
                    if discreta { "Discreta" } else { "Continua" }
                ))
            );
            assert_eq!(
                prob_point_label_for(index, L::Es),
                if discreta { "Puntual" } else { "Densidad" }
            );
            for locale in [L::En, L::Pt, L::It, L::Fr, L::De] {
                assert!(!prob_dist_name_for(index, locale).is_empty());
                assert!(prob_dist_tooltip_for(index, locale).is_some());
                assert!(!prob_dist_kind_label_for(index, locale).is_empty());
                assert!(!prob_point_label_for(index, locale).is_empty());
            }
        }
        assert_eq!(prob_dist_name_for(99, L::Es), "Normal");
        assert_eq!(prob_dist_tooltip_for(99, L::Es), None);
        assert_eq!(prob_dist_kind_label_for(99, L::En), "Continuous");
        assert_eq!(prob_point_label_for(99, L::De), "Dichte");
    }

    #[test]
    fn sheet_plot_muestreo_puro_pinnea_masas_y_ymax() {
        use super::{
            binomial_plot_masses, continuous_density_samples, discrete_ymax, geometric_plot_masses,
            poisson_plot_masses, CONTINUOUS_PLOT_SAMPLES, DISCRETE_PLOT_MAX_BARS, PLOT_YMAX_FLOOR,
        };
        // Paridad con `paint_continuous_density` + ramas 1/2/6 de
        // `draw_probability_plot` en `panels.rs`: mismas 61 muestras,
        // mismo `max(0.0)` + filtro finito, mismo piso 1e-12 y mismo
        // tope 256, dueño nuevo (slice 33). El pintado queda en panels.
        assert_eq!(CONTINUOUS_PLOT_SAMPLES, 61);
        assert_eq!(DISCRETE_PLOT_MAX_BARS, 256);
        assert_eq!(PLOT_YMAX_FLOOR, 1e-12);
        let (pts, ymax) = continuous_density_samples(|x| x, 0.0, 1.0);
        assert_eq!(pts.len(), 61);
        assert!((ymax - 1.0).abs() < 1e-12);
        // `NaN.max(0.0)` es 0.0 en Rust (ignora NaN): la densidad NaN
        // dibuja plano en 0, no vacío (misma semántica que el dibujo).
        let (nans, _) = continuous_density_samples(|_| f64::NAN, 0.0, 1.0);
        assert_eq!(nans.len(), 61);
        // Vacío real solo con infinitos (el filtro los salta).
        let (vacia, ymax_vacia) = continuous_density_samples(|_| f64::INFINITY, 0.0, 1.0);
        assert!(vacia.is_empty());
        assert_eq!(ymax_vacia, 1e-12);
        let bin = binomial_plot_masses(10, 0.5, 0, 10);
        assert_eq!(bin.len(), 11);
        assert!((discrete_ymax(&bin) - 0.246_093_75).abs() < 1e-9);
        let poi = poisson_plot_masses(3.0, 10);
        assert_eq!(poi.len(), 11);
        assert!(discrete_ymax(&poi) > 0.0);
        let geo = geometric_plot_masses(0.5, 5);
        assert_eq!(geo.len(), 6);
        assert!((geo[0].1 - 0.5).abs() < 1e-12);
        assert!(discrete_ymax(&[]).eq(&1e-12));
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn sheet_panel_datos_ancho_y_aviso_pinnea_dibujo() {
        use super::{panel_data_max_width_for, plot_message_text, PANEL_DATA_MAX_WIDTH};
        // Paridad con `draw_data_panel`/`draw_statistics_panel` +
        // `paint_plot_message` en `panels.rs`: mismo clamp 240..=360 y
        // misma cadena "fuera de cota: {mensaje}", dueño nuevo
        // (slice 34). El `SidePanel` y el pintor quedan en panels.
        assert_eq!(PANEL_DATA_MAX_WIDTH, 360.0);
        assert!(PANEL_DATA_MAX_WIDTH % 4.0 == 0.0);
        // Viewport angosto: el piso 240 manda (igual que el clamp inline).
        assert_eq!(panel_data_max_width_for(100.0), 240.0);
        assert_eq!(panel_data_max_width_for(0.0), 240.0);
        // Viewport ancho: el techo 360 manda (800 px × 0.45 = 360).
        assert_eq!(panel_data_max_width_for(800.0), 360.0);
        assert_eq!(panel_data_max_width_for(10_000.0), 360.0);
        // Zona media: 600 × 0.45 = 270 pasa intacto.
        assert!((panel_data_max_width_for(600.0) - 270.0).abs() < 1e-6);
        assert_eq!(
            plot_message_text("σ debe ser mayor que 0"),
            "fuera de cota: σ debe ser mayor que 0"
        );
        assert_eq!(plot_message_text(""), "fuera de cota: ");
    }

    #[test]
    fn sheet_prob_b_a157_plot_helpers_localizan() {
        use super::{plot_df_or_fuera_de_cota_for, plot_message_text_for, MAX_PANEL_DF};
        use grafito_ui::i18n::{t, Locale as L};
        // Paridad Es: `_for` delega al canonico (mismos mensajes que los
        // literales previos de `panels.rs:5756,:5761,:5969,:6006` +
        // `check_panel_df :2148-2154` + prefijo `:4282`).
        assert_eq!(
            plot_message_text_for("σ debe ser mayor que 0", L::Es),
            "fuera de cota: σ debe ser mayor que 0"
        );
        assert_eq!(plot_message_text_for("", L::Es), "fuera de cota: ");
        assert!(plot_df_or_fuera_de_cota_for(MAX_PANEL_DF, "gl", L::Es).is_ok());
        assert_eq!(
            plot_df_or_fuera_de_cota_for(0.0, "gl", L::Es).expect_err("cota"),
            "gl debe ser positivo y finito"
        );
        assert_eq!(
            plot_df_or_fuera_de_cota_for(MAX_PANEL_DF + 1.0, "gl1", L::Es).expect_err("cota"),
            "gl1 ≤ 500 en este panel (cota de UI)"
        );
        // 6 idiomas resuelven sin vacio ni fallback + templates intactos.
        for locale in [L::Es, L::En, L::Pt, L::It, L::Fr, L::De] {
            assert!(!plot_message_text_for("x", locale).is_empty());
            assert!(!plot_df_or_fuera_de_cota_for(0.0, "gl", locale)
                .expect_err("cota")
                .is_empty());
            let prefix = t("stats.prob_plot_prefix", locale);
            assert!(prefix.contains("{message}"));
            let positive = t("stats.prob_plot_df_positive", locale);
            assert!(positive.contains("{what}"));
            let capped = t("stats.prob_plot_df_capped", locale);
            assert!(capped.contains("{what}"));
            assert!(capped.contains("{max}"));
        }
        assert_eq!(
            t("stats.prob_plot_sigma_positive", L::En),
            "σ must be greater than 0"
        );
        assert_eq!(
            t("stats.prob_plot_prefix", L::De),
            "außerhalb des Bereichs: {message}"
        );
    }

    #[test]
    fn sheet_trig_b_a161_prompt_errors_localizan() {
        use super::{parse_slider_prompt, parse_slider_prompt_for};
        use grafito_ui::i18n::{t, Locale as L};
        // Paridad Es: `_for` delega al canonico (mismos 6 `Err` que los
        // literales previos de `parse_slider_prompt :2409-2452`).
        for input in [
            "",
            "inf",
            "100",
            "2 vueltas en 0 s",
            "100 vueltas en 1 s",
            "verde",
        ] {
            assert_eq!(
                parse_slider_prompt_for(input, L::Es).expect_err("canonico"),
                parse_slider_prompt(input).expect_err("canonico"),
            );
        }
        assert_eq!(
            parse_slider_prompt_for("", L::Es).expect_err("vacio"),
            "Escribí una velocidad: 1.5, «medio» o «2 vueltas en 10 s»"
        );
        assert_eq!(
            parse_slider_prompt_for("100 vueltas en 1 s", L::Es).expect_err("rapido"),
            "Eso da 628.32 rad/s: más de 6 rad/s no se sigue a ojo (bajá vueltas o subí segundos)"
        );
        // Ok intactos en los 6 idiomas (misma aceptacion).
        for locale in [L::Es, L::En, L::Pt, L::It, L::Fr, L::De] {
            assert_eq!(parse_slider_prompt_for("1.5", locale).expect("numero"), 1.5);
            assert_eq!(
                parse_slider_prompt_for("medio", locale).expect("palabra"),
                0.5
            );
            assert!(parse_slider_prompt_for("", locale).is_err());
            assert!(parse_slider_prompt_for("verde", locale).is_err());
            for key in [
                "trig.prompt_err_empty",
                "trig.prompt_err_finite",
                "trig.prompt_err_range",
                "trig.prompt_err_seconds",
                "trig.prompt_err_too_fast",
                "trig.prompt_err_unknown",
            ] {
                assert!(!t(key, locale).is_empty(), "{key} vacio en {locale:?}");
            }
            assert!(t("trig.prompt_err_too_fast", locale).contains("{speed}"));
        }
        assert_eq!(t("trig.prompt_err_finite", L::En), "Speed must be finite");
        assert_eq!(
            t("trig.prompt_err_unknown", L::De),
            "Nicht verstanden: versuch 1.5, „mittel“ oder „2 Runden in 10 s“"
        );
        // Interpolacion `{speed}` con el valor formateado.
        let fast = parse_slider_prompt_for("100 vueltas en 1 s", L::En).expect_err("rapido");
        assert!(fast.contains("628.32"), "interpola {{speed}}: {fast}");
        assert!(!fast.contains("{speed}"), "sin placeholder crudo: {fast}");
    }

    #[test]
    fn sheet_tablas_prompt_pinnea_resumen_y_guia() {
        use super::data_tables_assistant_prompt;
        use grafito_core::{DataTableObj, Document, GeoObject};
        // Paridad con `open_data_tables_in_assistant` en `panels.rs`:
        // mismo resumen (etiqueta + N + preview) y misma guía sin
        // tablas, dueño nuevo (slice 35). El drawer queda en panels.
        let vacio = Document::new();
        let guia = data_tables_assistant_prompt(&vacio);
        assert!(guia.contains("no hay tablas"), "guía sin tablas: {guia}");
        assert!(guia.contains("DataTable[{1,2,3}, {2,4,6}]"));
        let mut una = Document::new();
        una.try_add_object(GeoObject::DataTable(
            DataTableObj::new("x", "y", vec![1.0, 2.0], vec![3.0, 4.0]).with_label("T1"),
        ))
        .expect("tabla válida");
        let prompt = data_tables_assistant_prompt(&una);
        assert!(prompt.contains("«T1» N=2"), "resumen etiquetado: {prompt}");
        assert!(prompt.contains("1.0000"));
        assert!(prompt.contains("FitLinear"));
        // La rama `n == 0 → continue` del dueño nuevo es defensiva: la
        // validación rechaza tablas de <2 filas al insertar, así que por
        // documento solo llegan tablas con N≥2 (verificado acá).
        let hueca = GeoObject::DataTable(DataTableObj::new("x", "y", vec![], vec![]));
        assert!(Document::new().try_add_object(hueca).is_err());
        // Más de 12 valores trunca con "…+N más" (misma cadena que el
        // dibujo). Nota honesta: el insert auto-etiqueta («D»), así que la
        // rama "<sin etiqueta>" del dueño nuevo es defensiva (solo se
        // alcanzaría sin pasar por el documento).
        let mut larga = Document::new();
        let xs: Vec<f64> = (0..15).map(|i| i as f64).collect();
        let ys: Vec<f64> = (0..15).map(|i| i as f64 * 2.0).collect();
        larga
            .try_add_object(GeoObject::DataTable(DataTableObj::new("x", "y", xs, ys)))
            .expect("tabla larga válida");
        let truncado = data_tables_assistant_prompt(&larga);
        assert!(truncado.contains("N=15"), "quince filas: {truncado}");
        assert!(truncado.contains("…+3 más"));
    }

    #[test]
    fn sheet_regresion_textos_puros_pinnea_dibujo() {
        use super::{
            regression_fit_diagnostics_line, regression_fit_headline, regression_model_ready_text,
            regression_residual_line, regression_residuals_title,
            regression_residuals_truncated_note, regression_section_title, regression_source_label,
            regression_template_ready_text, REGRESSION_MAX_RESIDUALS_SHOWN, REGRESSION_TEMPLATES,
        };
        // Paridad con `draw_right_regression_panel` en `panels.rs`: mismas
        // cadenas que el dibujo, dueño nuevo (slice 36). El `SidePanel`,
        // los `RichText`/colores y el `collapsing` quedan en panels.
        assert_eq!(REGRESSION_MAX_RESIDUALS_SHOWN, 24);
        assert_eq!(REGRESSION_TEMPLATES.len(), 3);
        assert_eq!(REGRESSION_TEMPLATES[0].0, "Dispersión");
        assert_eq!(
            REGRESSION_TEMPLATES[1].1,
            "LinearRegression[{1, 2, 3}, {1, 4, 9}]"
        );
        assert_eq!(regression_source_label(Some("T1")), "T1");
        assert_eq!(regression_source_label(None), "tabla eliminada");
        assert_eq!(regression_fit_headline("f", "Lineal"), "f · Lineal");
        assert_eq!(
            regression_fit_diagnostics_line("T1", 0.5, 0.25),
            "sobre T1 · RMSE 0.500000 · R² 0.250000"
        );
        assert_eq!(regression_residuals_title(3), "Residuales (3)");
        assert_eq!(regression_residual_line(0, 0.123456789), "r1 = 0.123457");
        assert_eq!(
            regression_residuals_truncated_note(),
            "Se muestran los primeros 24 valores."
        );
        assert_eq!(regression_section_title("T1"), "Ajustar «T1»");
        assert_eq!(
            regression_template_ready_text("Dispersión"),
            "Dispersión listo en la entrada (Enter para crear)."
        );
        assert_eq!(
            regression_model_ready_text("Lineal"),
            "Ajuste Lineal listo en la entrada (Enter para crear)."
        );
    }

    #[test]
    fn sheet_stats_resumen_pinnea_filas_y_barras() {
        use super::{
            format_statistic, statistics_histogram_counts, statistics_summary,
            statistics_summary_rows, STATISTICS_SUMMARY_BINS, STATISTICS_SUMMARY_UNREPRESENTABLE,
        };
        // Paridad con `draw_statistics_panel` en `panels.rs`: mismo orden
        // de 12 filas, misma caída de suma y misma fórmula de binner,
        // dueño nuevo (slice 38). La grilla, el pintor y los botones
        // quedan en panels.
        assert_eq!(STATISTICS_SUMMARY_BINS, 10);
        assert_eq!(
            STATISTICS_SUMMARY_UNREPRESENTABLE,
            "No representable (desbordamiento)"
        );
        let data = [1.0, 2.0, 3.0, 4.0];
        let summary = statistics_summary(&data).expect("resumen válido");
        let filas = statistics_summary_rows(data.len(), &summary);
        let claves: Vec<&str> = filas.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(
            claves,
            [
                "N", "Suma", "Media", "Mediana", "Desvío", "Varianza", "Mín", "Máx", "Rango", "Q1",
                "Q3", "IQR"
            ]
        );
        assert_eq!(filas[0].1, "4");
        assert_eq!(filas[2].1, format_statistic(summary.mean));
        assert_eq!(filas[5].1, format_statistic(summary.variance));
        // Binner: 4 datos en 10 barras suman 4 y caen en barras distintas
        // (mín en la primera, máx en la última por el clamp).
        let (conteos, bw) = statistics_histogram_counts(&data, summary.minimum, summary.range, 10);
        assert_eq!(conteos.len(), 10);
        assert_eq!(conteos.iter().sum::<u32>(), 4);
        assert!((bw - summary.range.max(1e-9) / 10.0).abs() < 1e-12);
        assert!(conteos[0] >= 1);
        assert!(conteos[9] >= 1);
        // Rango 0: el piso 1e-9 evita división por cero, todo cae en la
        // primera barra (misma fórmula que el dibujo).
        let (planos, _) = statistics_histogram_counts(&[5.0, 5.0], 5.0, 0.0, 10);
        assert_eq!(planos.iter().sum::<u32>(), 2);
        assert_eq!(planos[0], 2);
        // Guarda defensiva: 0 barras no hace pánico (el dibujo usa 10).
        assert_eq!(
            statistics_histogram_counts(&data, summary.minimum, summary.range, 0),
            (Vec::new(), 0.0)
        );
    }

    #[test]
    fn sheet_prob_caso_textos_pinnea_colas_momentos_y_copiado() {
        // Paridad con `draw_probability_section` en `panels.rs`: mismas
        // caídas "—", mismo "{:.6}" y mismas cadenas de copiado/tooltip,
        // dueño nuevo (slice 39). Los botones Copiar, los tiles y el
        // pintor quedan en panels.
        let tails = Some((0.5, 0.5, 1.0));
        assert_eq!(prob_tail_text(tails, |t| t.0), "0.500000");
        assert_eq!(prob_tail_text(tails, |t| t.1), "0.500000");
        assert_eq!(prob_tail_text(tails, |t| t.2), "1.000000");
        assert_eq!(prob_tail_text(None, |t| t.0), "—");
        assert_eq!(prob_moment_text(Some(2.0)), format_statistic(2.0));
        assert_eq!(prob_moment_text(None), "—");
        assert_eq!(
            prob_case_copy_text("Normal(0,1)", "Densidad", 0.25, 0.5, "0.500000"),
            "Normal(0,1) · Densidad = 0.250000 · Acumulada = 0.500000 · P(X>x) = 0.500000"
        );
        assert_eq!(
            prob_case_detail_tooltip("Normal(0,1)"),
            "Normal(0,1)\nCaso evaluado en vivo con este motor"
        );
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn sheet_boton_panel_pinnea_ritmo_vertical() {
        // Paridad con `panels::gc_piel_tests::panel_helpers_respetan_tokens_y_clamps`:
        // mismo token `PANEL_BUTTON_H` = 28.0, dueño nuevo (slice 40). Los
        // ~20 call sites del dibujo lo consumen por import, sin cambio visual.
        use super::{PANEL_BUTTON_H, SHEET_CONTROL_H, TRIG_CONTROL_H};
        assert_eq!(PANEL_BUTTON_H, 28.0);
        assert_eq!(SHEET_CONTROL_H, PANEL_BUTTON_H);
        assert_eq!(TRIG_CONTROL_H, PANEL_BUTTON_H);
        assert!(PANEL_BUTTON_H >= HIT_TARGET_MIN);
    }

    #[test]
    fn sheet_export_textos_pinnea_copiado_y_filas() {
        // Paridad con `draw_export_section` en `panels.rs`: mismas cadenas
        // de copiado y mismo rótulo "{nombre} · {n} filas", dueño nuevo
        // (slice 41). Los botones, los `notify` y los workers quedan en panels.
        use super::{
            export_clipboard_copied_text, export_svg_copied_text, export_svg_error_text,
            export_table_row_label, sheet_table_subtitle,
        };
        assert_eq!(
            export_svg_copied_text(123),
            "SVG copiado al portapapeles (123 bytes)"
        );
        assert_eq!(export_svg_error_text("boom"), "No se pudo copiar SVG: boom");
        assert_eq!(
            export_clipboard_copied_text("MathML", 45),
            "MathML copiado al portapapeles (45 bytes)"
        );
        assert_eq!(
            export_clipboard_copied_text("TikZ-eje", 7),
            "TikZ-eje copiado al portapapeles (7 bytes)"
        );
        assert_eq!(export_table_row_label("T1", 5), "T1 · 5 filas");
        assert_eq!(
            export_table_row_label("<sin etiqueta>", 2),
            "<sin etiqueta> · 2 filas"
        );
        // El rótulo de export comparte expresión con la hoja (dominio
        // distinto, misma cadena: precedente slices 16/21).
        assert_eq!(
            export_table_row_label("T1", 5),
            sheet_table_subtitle("T1", 5)
        );
    }

    #[test]
    fn sheet_trig_prompt_textos_pinnea_rotulos_del_dibujo() {
        // Paridad con `draw_trig_speed_prompt` en `panels.rs`: mismos
        // rótulos, hint y tooltips, dueño nuevo (slice 42). El `TextEdit`,
        // el botón y el estado efímero quedan en panels.
        use super::{
            TRIG_PROMPT_APPLY_LABEL, TRIG_PROMPT_APPLY_TOOLTIP, TRIG_PROMPT_AUTO_LABEL,
            TRIG_PROMPT_FIELD_TOOLTIP, TRIG_PROMPT_HINT,
        };
        assert_eq!(TRIG_PROMPT_AUTO_LABEL, "Auto");
        assert_eq!(TRIG_PROMPT_HINT, "1.5 · medio");
        assert_eq!(
            TRIG_PROMPT_FIELD_TOOLTIP,
            "Pedí una velocidad con palabras o números y pulsa Aplicar"
        );
        assert_eq!(TRIG_PROMPT_APPLY_LABEL, "Aplicar");
        assert_eq!(
            TRIG_PROMPT_APPLY_TOOLTIP,
            "Convierte el pedido en velocidad del slider"
        );
    }

    #[test]
    fn sheet_cas_rapidas_pinnea_acciones_y_vacio_del_dibujo() {
        // Paridad con `draw_cas_panel` en `panels.rs`: mismas 4 tuplas,
        // mismo tooltip "Insertar {snippet}", mismos textos de vacío y
        // mismo título "Hoja · {n}", dueño nuevo (slice 43). El `SidePanel`,
        // los botones y el drawer quedan en panels.
        use super::{
            cas_quick_insert_tooltip, cas_sheet_title, CAS_EMPTY_EXAMPLE, CAS_EMPTY_HINT,
            CAS_EMPTY_TITLE, CAS_QUICK_ACTIONS,
        };
        assert_eq!(
            CAS_QUICK_ACTIONS,
            [
                ("Derivar", "Derivative["),
                ("Integrar", "Integral["),
                ("Resolver", "Solve["),
                ("Límite", "Limit["),
            ]
        );
        assert_eq!(
            cas_quick_insert_tooltip("Derivative["),
            "Insertar Derivative["
        );
        assert_eq!(CAS_EMPTY_TITLE, "Sin cálculos aún");
        assert_eq!(CAS_EMPTY_HINT, "Escribe arriba y pulsa Enter.");
        assert_eq!(
            CAS_EMPTY_EXAMPLE,
            "Ej: Derivative[x^2, x]  ·  Solve[x^2-4, x]"
        );
        assert_eq!(cas_sheet_title(0), "Hoja · 0");
        assert_eq!(cas_sheet_title(3), "Hoja · 3");
    }

    #[test]
    fn sheet_cas_panel_textos_pinnea_titulo_entrada_y_celdas() {
        // Paridad con `draw_cas_panel` en `panels.rs`: mismo título,
        // mismos rótulos de limpiar/entrada/ejecutar, mismo placeholder
        // "x^2, x", mismo hint "↵  •  Tab" y mismas líneas de celda
        // ("{i+1}" y "> {input}"), dueño nuevo (slice 44). El `SidePanel`,
        // los botones y el drawer quedan en panels.
        use super::{
            cas_cell_input_line, cas_cell_number, CAS_CLEAR_LABEL, CAS_CLEAR_TOOLTIP,
            CAS_ENTRY_HINT, CAS_ENTRY_KEYS_HINT, CAS_ENTRY_LABEL, CAS_EXECUTE_TOOLTIP,
            CAS_PANEL_TITLE,
        };
        assert_eq!(CAS_PANEL_TITLE, "Cálculo Simbólico (CAS)");
        assert_eq!(CAS_CLEAR_LABEL, "Limpiar");
        assert_eq!(CAS_CLEAR_TOOLTIP, "Eliminar las celdas CAS guardadas");
        assert_eq!(CAS_ENTRY_LABEL, "Entrada");
        assert_eq!(CAS_ENTRY_HINT, "x^2, x");
        assert_eq!(CAS_EXECUTE_TOOLTIP, "Ejecutar");
        assert_eq!(CAS_ENTRY_KEYS_HINT, "↵  •  Tab");
        assert_eq!(cas_cell_number(0), "1");
        assert_eq!(cas_cell_number(9), "10");
        assert_eq!(cas_cell_input_line("x^2"), "> x^2");
        assert_eq!(cas_cell_input_line(""), "> ");
    }

    #[test]
    fn sheet_cas_quick_section_pinnea_titulo_y_bajada() {
        // Paridad con `draw_cas_panel` en `panels.rs`: mismo título y misma
        // bajada de la sección de acciones rápidas, dueño nuevo (slice 45).
        // El `SidePanel`, los botones y el drawer quedan en panels.
        use super::{CAS_QUICK_SECTION_SUBTITLE, CAS_QUICK_SECTION_TITLE};
        assert_eq!(CAS_QUICK_SECTION_TITLE, "Acciones rápidas");
        assert_eq!(
            CAS_QUICK_SECTION_SUBTITLE,
            "Atajos — inserta sintaxis mínima."
        );
    }

    #[test]
    fn sheet_export_textos_pinnea_titulo_botones_y_avisos() {
        // Paridad con `draw_export_section` en `panels.rs`: mismo título y
        // bajada, mismos 6+3 rótulos/tooltips de la grilla, mismo título de
        // tablas, mismo botón CSV, mismo nombre PDF, mismos 3 avisos y mismo
        // botón de revelado, dueño nuevo (slice 46). Los `SidePanel`, la
        // grilla, los diálogos rfd, los workers y el `notify` quedan en panels.
        use super::{
            EXPORT_CSV_LABEL, EXPORT_CSV_TOOLTIP, EXPORT_FILE_BUTTONS, EXPORT_NOTIFY_CSV,
            EXPORT_NOTIFY_PDF, EXPORT_NOTIFY_PNG, EXPORT_PDF_DEFAULT_FILENAME, EXPORT_REVEAL_LABEL,
            EXPORT_REVEAL_TOOLTIP, EXPORT_SECTION_SUBTITLE, EXPORT_SECTION_TITLE,
            EXPORT_TABLES_TITLE, EXPORT_TEXT_BUTTONS,
        };
        assert_eq!(EXPORT_SECTION_TITLE, "Exportación");
        assert_eq!(
            EXPORT_SECTION_SUBTITLE,
            "Exportá a archivo (SVG/PNG/TikZ/PDF) o copiá al portapapeles."
        );
        assert_eq!(EXPORT_FILE_BUTTONS.len(), 6);
        assert_eq!(EXPORT_FILE_BUTTONS[0].0, "SVG");
        assert_eq!(
            EXPORT_FILE_BUTTONS[0].1,
            "Exportar el lienzo a un archivo SVG vectorial"
        );
        assert_eq!(EXPORT_FILE_BUTTONS[1].0, "PNG");
        assert_eq!(
            EXPORT_FILE_BUTTONS[1].1,
            "Exportar el lienzo a PNG (tiny-skia) donde elijas"
        );
        assert_eq!(EXPORT_FILE_BUTTONS[2].0, "TikZ");
        assert_eq!(EXPORT_FILE_BUTTONS[2].1, "Exportar el lienzo a código TikZ");
        assert_eq!(EXPORT_FILE_BUTTONS[3].0, "PDF");
        assert!(EXPORT_FILE_BUTTONS[3].1.contains("hasta 64 hojas."));
        assert_eq!(EXPORT_FILE_BUTTONS[4].0, "Copiar SVG");
        assert_eq!(
            EXPORT_FILE_BUTTONS[4].1,
            "Copia el SVG real del lienzo al portapapeles"
        );
        assert_eq!(EXPORT_FILE_BUTTONS[5].0, "Copiar PNG");
        assert!(EXPORT_FILE_BUTTONS[5].1.contains("Word/Moodle"));
        assert_eq!(EXPORT_TEXT_BUTTONS.len(), 3);
        assert_eq!(EXPORT_TEXT_BUTTONS[0].0, "MathML");
        assert_eq!(EXPORT_TEXT_BUTTONS[1].0, "TikZ-eje");
        assert_eq!(EXPORT_TEXT_BUTTONS[2].0, "HTML");
        assert_eq!(EXPORT_TABLES_TITLE, "Tablas (CSV RFC 4180)");
        assert_eq!(EXPORT_CSV_LABEL, "CSV");
        assert_eq!(
            EXPORT_CSV_TOOLTIP,
            "Descarga honesta: escribe el CSV real o informa el error"
        );
        assert_eq!(EXPORT_PDF_DEFAULT_FILENAME, "grafito_export.pdf");
        assert_eq!(EXPORT_NOTIFY_PDF, "Exportando PDF…");
        assert_eq!(EXPORT_NOTIFY_PNG, "Copiando PNG al portapapeles…");
        assert_eq!(EXPORT_NOTIFY_CSV, "Exportando CSV…");
        assert_eq!(EXPORT_REVEAL_LABEL, "Mostrar en carpeta");
        assert_eq!(
            EXPORT_REVEAL_TOOLTIP,
            "Abre la carpeta de tu última exportación."
        );
    }

    #[test]
    fn sheet_complex_textos_pinnea_panel() {
        // Paridad con `draw_complex_panel` en `panels.rs`: mismos títulos,
        // hints, rótulos de guías, textos de armado/último valor, sección
        // de objetos y 5 atajos ejecutables, dueño nuevo (slice 47). El
        // `SidePanel`, los `TextEdit`/botones y el drawer quedan en panels.
        use super::{
            complex_contour_armed_text, complex_contour_last_value_text, complex_quick_hover_text,
            COMPLEX_BASE_SYMBOL_HINT, COMPLEX_BASE_SYMBOL_TITLE, COMPLEX_CONTOUR_CIRCLE_LABEL,
            COMPLEX_CONTOUR_CIRCLE_TOOLTIP, COMPLEX_CONTOUR_CLOSE_LABEL,
            COMPLEX_CONTOUR_DRAW_LABEL, COMPLEX_CONTOUR_DRAW_TOOLTIP, COMPLEX_CONTOUR_EXPR_HINT,
            COMPLEX_CONTOUR_PREFILL, COMPLEX_CONTOUR_PREFILL_TOOLTIP,
            COMPLEX_CONTOUR_RESIDUES_LABEL, COMPLEX_CONTOUR_SMOOTH_LABEL,
            COMPLEX_CONTOUR_SNAP_LABEL, COMPLEX_CONTOUR_STABILIZER_LABEL, COMPLEX_CONTOUR_TITLE,
            COMPLEX_OBJECTS_EMPTY, COMPLEX_OBJECTS_TITLE, COMPLEX_PANEL_INPUT_HINT,
            COMPLEX_PANEL_TITLE, COMPLEX_QUICK_HINTS, COMPLEX_QUICK_TITLE,
        };
        use crate::complex_contour::ComplexContourMode;
        assert_eq!(COMPLEX_PANEL_TITLE, "Números Complejos");
        assert_eq!(
            COMPLEX_PANEL_INPUT_HINT,
            "DomainColoring[1/z, -2, 2, -2, 2, 160]"
        );
        assert_eq!(COMPLEX_BASE_SYMBOL_TITLE, "Símbolo base");
        assert_eq!(COMPLEX_BASE_SYMBOL_HINT, "z");
        assert_eq!(COMPLEX_CONTOUR_TITLE, "Integral de contorno");
        assert_eq!(COMPLEX_CONTOUR_EXPR_HINT, "f(z) — ej. 1/z");
        assert_eq!(COMPLEX_CONTOUR_PREFILL, "↧");
        assert_eq!(
            COMPLEX_CONTOUR_PREFILL_TOOLTIP,
            "Usar f(z) del objeto seleccionado"
        );
        assert_eq!(COMPLEX_CONTOUR_RESIDUES_LABEL, "Residuos (Gauss)");
        assert_eq!(COMPLEX_CONTOUR_SNAP_LABEL, "Imán");
        assert_eq!(COMPLEX_CONTOUR_CLOSE_LABEL, "Cierre automático");
        assert_eq!(COMPLEX_CONTOUR_SMOOTH_LABEL, "Suavizar");
        assert_eq!(COMPLEX_CONTOUR_STABILIZER_LABEL, "Estabilizador");
        assert_eq!(COMPLEX_CONTOUR_DRAW_LABEL, "Dibujar contorno");
        assert!(COMPLEX_CONTOUR_DRAW_TOOLTIP.contains("Esc cancela"));
        assert_eq!(COMPLEX_CONTOUR_CIRCLE_LABEL, "Círculo");
        assert!(COMPLEX_CONTOUR_CIRCLE_TOOLTIP.contains("imán"));
        assert_eq!(
            complex_contour_armed_text(ComplexContourMode::Freehand),
            "Dibujá sobre el lienzo · Esc para salir"
        );
        assert_eq!(
            complex_contour_armed_text(ComplexContourMode::Circle),
            "Centro y radio sobre el lienzo · Esc para salir"
        );
        assert_eq!(complex_contour_last_value_text("1"), "Último valor: 1");
        assert_eq!(COMPLEX_OBJECTS_TITLE, "Objetos");
        assert!(COMPLEX_OBJECTS_EMPTY.contains("Sin objetos."));
        assert_eq!(COMPLEX_QUICK_TITLE, "Comandos rápidos");
        assert_eq!(COMPLEX_QUICK_HINTS.len(), 5);
        assert_eq!(
            COMPLEX_QUICK_HINTS[0].1,
            "DomainColoring[1/z, -2, 2, -2, 2, 160]"
        );
        assert_eq!(
            complex_quick_hover_text(COMPLEX_QUICK_HINTS[0].1),
            "Click para ejecutar: DomainColoring[1/z, -2, 2, -2, 2, 160]"
        );
    }

    #[test]
    fn sheet_attractor_textos_pinnea_panel() {
        // Paridad con `draw_attractor_panel` en `panels.rs`: mismo título,
        // misma sección de objetos con su vacío, mismo título activo con sus
        // 5 líneas de parámetros, misma guía inactiva con su botón y comando
        // por defecto, y misma sección de comandos, dueño nuevo (slice 48).
        // El `SidePanel`, las tarjetas, los botones y el drawer quedan en panels.
        use super::{
            attractor_beta_text, attractor_dt_text, attractor_rho_text, attractor_sigma_text,
            attractor_steps_text, ATTRACTOR_ACTIVE_TITLE, ATTRACTOR_COMMANDS_TITLE,
            ATTRACTOR_CREATE_DEFAULT_LABEL, ATTRACTOR_DEFAULT_COMMAND, ATTRACTOR_INACTIVE_GUIDE,
            ATTRACTOR_LORENZ_HINT, ATTRACTOR_OBJECTS_EMPTY, ATTRACTOR_OBJECTS_TITLE,
            ATTRACTOR_PANEL_TITLE,
        };
        assert_eq!(ATTRACTOR_PANEL_TITLE, "Dinámica y Atractores");
        assert_eq!(ATTRACTOR_OBJECTS_TITLE, "Objetos dinámicos");
        assert!(ATTRACTOR_OBJECTS_EMPTY.contains("Attractor[10, 28, 8/3]"));
        assert_eq!(ATTRACTOR_ACTIVE_TITLE, "Attractor activo");
        assert_eq!(attractor_sigma_text(10.0), "sigma = 10.000");
        assert_eq!(attractor_rho_text(28.0), "rho = 28.000");
        assert_eq!(attractor_beta_text(8.0 / 3.0), "beta = 2.667");
        assert_eq!(attractor_dt_text(0.01), "dt = 0.0100");
        assert_eq!(attractor_steps_text(5000), "pasos = 5000");
        assert!(ATTRACTOR_INACTIVE_GUIDE.contains("Attractor[σ, ρ, β]"));
        assert_eq!(ATTRACTOR_CREATE_DEFAULT_LABEL, "Crear Lorenz por defecto");
        assert_eq!(ATTRACTOR_DEFAULT_COMMAND, "Attractor[10, 28, 8/3]");
        assert_eq!(ATTRACTOR_COMMANDS_TITLE, "Comandos");
        assert_eq!(
            ATTRACTOR_LORENZ_HINT,
            "- Lorenz: Attractor[sigma, rho, beta]"
        );
    }

    #[test]
    fn sheet_inspector_props_pinnea_header_vacio_identidad_y_estilo() {
        // Paridad con `draw_right_properties_panel`,
        // `draw_right_properties_contents`, `draw_inspector_identity` y
        // `draw_inspector_empty_state` en `panels.rs`: mismo header, mismo
        // vacío, misma etiqueta de ecuación editable con sus botones, mismo
        // badge Visible/Oculto, misma medida y misma sección de estilo,
        // dueño nuevo (slice 49). El `SidePanel`, los `TextEdit`/botones,
        // el drawer y los combos quedan en panels.
        use super::{
            inspector_example_tooltip, inspector_measure_text, inspector_visibility_label,
            INSPECTOR_APPLY_LABEL, INSPECTOR_EMPTY_HINT, INSPECTOR_EMPTY_TITLE,
            INSPECTOR_EQUATION_LABEL, INSPECTOR_IDENTITY_TITLE, INSPECTOR_LAYER_LABEL,
            INSPECTOR_MISSING_OBJECT, INSPECTOR_NO_EQUATION, INSPECTOR_PANEL_SUBTITLE,
            INSPECTOR_PANEL_TITLE, INSPECTOR_REVERT_LABEL, INSPECTOR_STYLE_NONE_HINT,
            INSPECTOR_STYLE_SUBTITLE, INSPECTOR_STYLE_TITLE, INSPECTOR_USE_EXAMPLE_LABEL,
        };
        assert_eq!(INSPECTOR_PANEL_TITLE, "Inspector");
        assert_eq!(
            INSPECTOR_PANEL_SUBTITLE,
            "Propiedades del objeto seleccionado."
        );
        assert_eq!(
            INSPECTOR_EMPTY_TITLE,
            "Tocá un objeto del lienzo para editarlo acá."
        );
        assert_eq!(
            INSPECTOR_EMPTY_HINT,
            "Acá ajustás su geometría, estilo y controles avanzados."
        );
        assert_eq!(INSPECTOR_MISSING_OBJECT, "Objeto inexistente.");
        assert_eq!(INSPECTOR_EQUATION_LABEL, "Ecuación (editable)");
        assert_eq!(INSPECTOR_REVERT_LABEL, "Revertir");
        assert_eq!(INSPECTOR_USE_EXAMPLE_LABEL, "Usar ejemplo");
        assert_eq!(
            inspector_example_tooltip("y = x^2"),
            "Inserta sintaxis válida: y = x^2"
        );
        assert_eq!(INSPECTOR_APPLY_LABEL, "Aplicar ecuación");
        assert_eq!(INSPECTOR_IDENTITY_TITLE, "Ecuación");
        assert_eq!(inspector_visibility_label(true), "Visible");
        assert_eq!(inspector_visibility_label(false), "Oculto");
        assert_eq!(
            INSPECTOR_NO_EQUATION,
            "Ecuación no disponible para este tipo"
        );
        assert_eq!(inspector_measure_text("1.0"), "Medida: 1.0");
        assert_eq!(INSPECTOR_STYLE_TITLE, "Estilo");
        assert_eq!(
            INSPECTOR_STYLE_SUBTITLE,
            "Trazo, marcador y capa de dibujo (0 = fondo)."
        );
        assert_eq!(
            INSPECTOR_STYLE_NONE_HINT,
            "Este tipo no tiene trazo ni marcador con estilo."
        );
        assert_eq!(INSPECTOR_LAYER_LABEL, "Capa:");
    }

    #[test]
    fn sheet_inspector_chrome_for_delega_es_al_canonico() {
        // B-A108: el dispatcher `inspector_chrome_for` devuelve la const
        // canónica en Es (paridad byte-identica con el catálogo i18n) y
        // delega al `inspector_msg` de `grafito-ui` en el resto.
        use super::inspector_chrome_for;
        use grafito_ui::i18n::{t, Locale};
        for suffix in [
            "panel_title",
            "panel_subtitle",
            "empty_title",
            "empty_hint",
            "missing_object",
            "equation_label",
            "revert",
            "use_example",
            "apply",
            "identity_title",
            "no_equation",
            "style_title",
            "style_subtitle",
            "style_none_hint",
            "layer_label",
        ] {
            let key: &'static str = Box::leak(format!("inspector.{suffix}").into_boxed_str());
            assert_eq!(
                inspector_chrome_for(suffix, Locale::Es),
                t(key, Locale::Es),
                "Es delega al canónico: {suffix}"
            );
            assert_eq!(
                inspector_chrome_for(suffix, Locale::En),
                t(key, Locale::En),
                "resto delega a i18n: {suffix}"
            );
        }
        assert_eq!(inspector_chrome_for("nope", Locale::Es), "nope");
        assert_eq!(inspector_chrome_for("nope", Locale::En), "nope");
    }

    #[test]
    fn sheet_inspector_poly_chrome_for_delega_es_al_canonico() {
        // B-A109: el dispatcher `inspector_chrome_for` devuelve la const
        // canónica 4D/N-D en Es y delega a i18n en el resto; el helper
        // `polytope_nd_projection_body_for` elige por dimensión.
        use super::{inspector_chrome_for, polytope_nd_projection_body_for};
        use grafito_ui::i18n::{t, Locale};
        for suffix in [
            "poly_projection_title",
            "poly_projection_subtitle",
            "poly_anim_title",
            "poly_anim_body",
            "poly_geometry_title",
            "poly_geometry_subtitle",
            "poly_appearance_title",
            "poly_appearance_subtitle",
            "poly_fill_omitted_hint",
            "poly_rotation_manual_title",
            "poly_rotation_manual_guide",
            "poly_rotation_reset_label",
            "poly_rotation_planes_title",
            "polytope_geometry_subtitle",
            "polytope_projection_subtitle",
            "polytope_anim_dim4_body",
            "polytope_anim_other_body",
            "polytope_appearance_subtitle",
            "polytope_fill_hint",
            "polytope_rotation_guide",
        ] {
            let key: &'static str = Box::leak(format!("inspector.{suffix}").into_boxed_str());
            assert_eq!(
                inspector_chrome_for(suffix, Locale::Es),
                t(key, Locale::Es),
                "Es delega al canónico: {suffix}"
            );
            assert_eq!(
                inspector_chrome_for(suffix, Locale::En),
                t(key, Locale::En),
                "resto delega a i18n: {suffix}"
            );
        }
        assert_eq!(
            polytope_nd_projection_body_for(4, Locale::Es),
            t("inspector.polytope_anim_dim4_body", Locale::Es)
        );
        assert_eq!(
            polytope_nd_projection_body_for(5, Locale::Es),
            t("inspector.polytope_anim_other_body", Locale::Es)
        );
        assert_eq!(
            polytope_nd_projection_body_for(4, Locale::En),
            t("inspector.polytope_anim_dim4_body", Locale::En)
        );
        assert_eq!(
            polytope_nd_projection_body_for(3, Locale::En),
            t("inspector.polytope_anim_other_body", Locale::En)
        );
    }

    #[test]
    fn sheet_inspector_b_a159_apariencia_y_cierres_delegan_es_al_canonico() {
        // B-A159: dispatcher extendido (8 estáticas: apariencia 4D +
        // cierres) delega Es al canónico y resto a i18n.
        use super::inspector_chrome_for;
        use grafito_ui::i18n::{t, Locale};
        for suffix in [
            "edge_color_label",
            "edge_color_hover",
            "fill_enabled_label",
            "fill_color_label",
            "fill_color_hover",
            "dimension_label",
            "close_context",
            "close_anim",
        ] {
            let key: &'static str = Box::leak(format!("inspector.{suffix}").into_boxed_str());
            assert_eq!(
                inspector_chrome_for(suffix, Locale::Es),
                t(key, Locale::Es),
                "Es delega al canónico: {suffix}"
            );
            assert_eq!(
                inspector_chrome_for(suffix, Locale::En),
                t(key, Locale::En),
                "resto delega a i18n: {suffix}"
            );
        }
    }

    #[test]
    fn sheet_prob_motor_b_a160_canonico_es_intencional() {
        // B-A160 (cierre saga prob): los `Err` del motor puro son canónicos
        // ES a propósito (0 keys, precedente B-A114/B-A146): el draw ya está
        // localizado (cuantil B-A122, plot B-A157) y los widgets clampan, así
        // que el `Err` crudo es path defensivo. Si se localizan, actualizar
        // este pin + `prob_motor_err_se_muestra_crudo` en `panels.rs`.
        use super::{
            binomial_pmf, check_probability_point, exponential_cdf, finish_probability_scalar,
            geometric_quantile_honest, normal_cdf, normal_pdf, normal_quantile_honest,
            student_t_pdf, uniform_cdf,
        };
        // Mensajes canónicos pineados (ES exacto del motor).
        assert_eq!(
            check_probability_point(f64::NAN, "x").expect_err("no-finito"),
            ["{what} debe ser un número", " finito"]
                .concat()
                .replace("{what}", "x")
        );
        assert_eq!(
            finish_probability_scalar(f64::INFINITY, "El cuantil").expect_err("no-finito"),
            ["{what} no es finito con estos", " parámetros"]
                .concat()
                .replace("{what}", "El cuantil")
        );
        assert_eq!(
            normal_pdf(0.0, 0.0, 0.0).expect_err("sigma 0"),
            ["σ debe ser ", "mayor que 0"].concat()
        );
        assert_eq!(
            normal_cdf(0.0, 0.0, -1.0).expect_err("sigma <0"),
            ["σ debe ser ", "mayor que 0"].concat()
        );
        assert_eq!(
            normal_quantile_honest(0.0, 0.0, 1.0).expect_err("p=0"),
            "p debe estar en el intervalo (0, 1)"
        );
        assert_eq!(
            binomial_pmf(6, 5, 0.5).expect_err("k>n"),
            "k no puede superar a n"
        );
        assert_eq!(
            student_t_pdf(0.0, 0.0).expect_err("gl=0"),
            ["{what} debe ser positivo", " y finito"]
                .concat()
                .replace("{what}", "gl")
        );
        assert_eq!(
            uniform_cdf(1.0, 2.0, 2.0).expect_err("a>=b"),
            ["se requiere", " a < b"].concat()
        );
        assert_eq!(
            exponential_cdf(1.0, -2.0).expect_err("lambda<0"),
            ["λ debe ser ", "mayor que 0"].concat()
        );
        assert_eq!(
            geometric_quantile_honest(1.0, 0.5).expect_err("p=1"),
            "p debe estar en el intervalo (0, 1)"
        );
        // Sin key `stats.prob_err*`: canónico sin display-localizar.
        for suffix in grafito_ui::i18n::STAT_KEYS {
            assert!(
                !suffix.starts_with("prob_err"),
                "motor canónico sin key a propósito: {suffix}"
            );
        }
    }

    #[test]
    fn sheet_inspector_b_a110_helpers_delegan_es_al_canonico() {
        // B-A110: dispatcher extendido (11 estáticas) + helpers `_for`
        // (visibilidad/ejemplo/medida/kind/familia/tooltip) delegan Es al
        // canónico y resto a i18n; templates `{example}`/`{measure}` ×6.
        use super::{
            inspector_chrome_for, inspector_example_tooltip, inspector_example_tooltip_for,
            inspector_identity_tooltip, inspector_identity_tooltip_for, inspector_measure_text,
            inspector_measure_text_for, inspector_visibility_label, inspector_visibility_label_for,
            polychoron_kind_label, polychoron_kind_label_for, polytope_nd_family_label,
            polytope_nd_family_label_for,
        };
        use grafito_geometry::{RegularPolychoron, RegularPolytopeFamily};
        use grafito_ui::i18n::{t, Locale};
        for suffix in [
            "visibility_visible",
            "visibility_hidden",
            "polychoron_kind_pentachoron",
            "polychoron_kind_tesseract",
            "polychoron_kind_sixteen",
            "polychoron_kind_twentyfour",
            "polychoron_kind_onetwenty",
            "polychoron_kind_sixhundred",
            "polytope_family_simplex",
            "polytope_family_hypercube",
            "polytope_family_cross",
        ] {
            let key: &'static str = Box::leak(format!("inspector.{suffix}").into_boxed_str());
            assert_eq!(
                inspector_chrome_for(suffix, Locale::Es),
                t(key, Locale::Es),
                "Es delega al canónico: {suffix}"
            );
            assert_eq!(
                inspector_chrome_for(suffix, Locale::En),
                t(key, Locale::En),
                "resto delega a i18n: {suffix}"
            );
        }
        // Paridad dispatcher ↔ funciones canónicas.
        assert_eq!(
            inspector_chrome_for("visibility_visible", Locale::Es),
            inspector_visibility_label(true)
        );
        assert_eq!(
            inspector_chrome_for("visibility_hidden", Locale::Es),
            inspector_visibility_label(false)
        );
        assert_eq!(
            inspector_chrome_for("polychoron_kind_pentachoron", Locale::Es),
            polychoron_kind_label(RegularPolychoron::Pentachoron)
        );
        assert_eq!(
            inspector_chrome_for("polytope_family_cross", Locale::Es),
            polytope_nd_family_label(RegularPolytopeFamily::CrossPolytope)
        );
        // Helpers `_for`: Es = canónico byte-idéntico.
        assert_eq!(
            inspector_visibility_label_for(true, Locale::Es),
            inspector_visibility_label(true)
        );
        assert_eq!(
            inspector_example_tooltip_for("y = x^2", Locale::Es),
            inspector_example_tooltip("y = x^2")
        );
        assert_eq!(
            inspector_measure_text_for("1.0", Locale::Es),
            inspector_measure_text("1.0")
        );
        assert_eq!(
            polychoron_kind_label_for(RegularPolychoron::Tesseract, Locale::Es),
            polychoron_kind_label(RegularPolychoron::Tesseract)
        );
        assert_eq!(
            polytope_nd_family_label_for(RegularPolytopeFamily::Simplex, Locale::Es),
            polytope_nd_family_label(RegularPolytopeFamily::Simplex)
        );
        assert_eq!(
            inspector_identity_tooltip_for("E", "A", true, Locale::Es),
            inspector_identity_tooltip("E", "A", true)
        );
        // Resto: sale de i18n con templates interpolados.
        assert_eq!(
            inspector_visibility_label_for(false, Locale::En),
            t("inspector.visibility_hidden", Locale::En)
        );
        assert_eq!(
            inspector_example_tooltip_for("y = x^2", Locale::En),
            t("inspector.example_tooltip", Locale::En).replace("{example}", "y = x^2")
        );
        assert_eq!(
            inspector_measure_text_for("1.0", Locale::De),
            t("inspector.measure", Locale::De).replace("{measure}", "1.0")
        );
        assert_eq!(
            polychoron_kind_label_for(RegularPolychoron::SixteenCell, Locale::Pt),
            t("inspector.polychoron_kind_sixteen", Locale::Pt)
        );
        assert_eq!(
            polytope_nd_family_label_for(RegularPolytopeFamily::Hypercube, Locale::Fr),
            t("inspector.polytope_family_hypercube", Locale::Fr)
        );
    }

    #[test]
    fn sheet_inspector_b_a111_helpers_delegan_es_al_canonico() {
        // B-A111: prefijo Etiqueta + estilos trazo/punto delegan Es al
        // canónico y resto a i18n; templates `{label}` ×6.
        use super::{
            inspector_3d_tag_label, inspector_3d_tag_label_for, inspector_label_text,
            inspector_label_text_for, line_style_label, line_style_label_for, point_style_label,
            point_style_label_for,
        };
        use grafito_core::{LineStyle, PointStyle};
        use grafito_ui::i18n::{t, Locale};
        // Es = canónico byte-idéntico.
        assert_eq!(
            inspector_label_text_for("S", Locale::Es),
            inspector_label_text("S")
        );
        assert_eq!(inspector_label_text_for("   ", Locale::Es), None);
        assert_eq!(
            line_style_label_for(LineStyle::Solid, Locale::Es),
            line_style_label(LineStyle::Solid)
        );
        assert_eq!(
            point_style_label_for(PointStyle::Plus, Locale::Es),
            point_style_label(PointStyle::Plus)
        );
        assert_eq!(
            inspector_3d_tag_label_for("C", Locale::Es),
            inspector_3d_tag_label("C")
        );
        // Resto: sale de i18n con templates interpolados.
        assert_eq!(
            inspector_label_text_for("S", Locale::En),
            Some(t("inspector.label_prefix", Locale::En).replace("{label}", "S"))
        );
        assert_eq!(
            line_style_label_for(LineStyle::Dashed, Locale::Pt),
            t("inspector.line_dashed", Locale::Pt)
        );
        assert_eq!(
            point_style_label_for(PointStyle::Circle, Locale::It),
            t("inspector.point_circle", Locale::It)
        );
        assert_eq!(
            inspector_3d_tag_label_for("C", Locale::De),
            t("inspector.label_prefix", Locale::De).replace("{label}", "C")
        );
        assert_eq!(inspector_label_text_for("   ", Locale::Fr), None);
    }

    #[test]
    fn sheet_inspector_b_a112_helpers_delegan_es_al_canonico() {
        // B-A112: primitivas 3D + sliders delegan Es al canónico y resto
        // a i18n; templates `{ranges}`/`{cells}`/`{status}`/`{tris}` ×6.
        use super::{
            implicit_3d_box_text, implicit_3d_box_text_for, implicit_3d_preview_text,
            implicit_3d_preview_text_for, implicit_3d_state_text, implicit_3d_state_text_for,
            inspector_chrome_for, INSPECTOR_CENTER_LABEL, INSPECTOR_CUBE3D_TITLE,
            INSPECTOR_DOCK_EMPTY_GUIDE, INSPECTOR_DOCK_EMPTY_TITLE, INSPECTOR_EDGE_WIDTH_TEXT,
            INSPECTOR_IMPLICIT3D_TITLE, INSPECTOR_POINT3D_TITLE, INSPECTOR_RADIUS_TEXT,
            INSPECTOR_SCALE_TEXT, INSPECTOR_SIZE_TEXT, INSPECTOR_SPHERE3D_TITLE,
        };
        use grafito_ui::i18n::{t, Locale};
        // Dispatcher: Es = const canónica byte-idéntica.
        assert_eq!(
            inspector_chrome_for("cube3d_title", Locale::Es),
            INSPECTOR_CUBE3D_TITLE
        );
        assert_eq!(
            inspector_chrome_for("sphere3d_title", Locale::Es),
            INSPECTOR_SPHERE3D_TITLE
        );
        assert_eq!(
            inspector_chrome_for("point3d_title", Locale::Es),
            INSPECTOR_POINT3D_TITLE
        );
        assert_eq!(
            inspector_chrome_for("center_label", Locale::Es),
            INSPECTOR_CENTER_LABEL
        );
        assert_eq!(
            inspector_chrome_for("implicit3d_title", Locale::Es),
            INSPECTOR_IMPLICIT3D_TITLE
        );
        assert_eq!(
            inspector_chrome_for("dock_empty_title", Locale::Es),
            INSPECTOR_DOCK_EMPTY_TITLE
        );
        assert_eq!(
            inspector_chrome_for("dock_empty_guide", Locale::Es),
            INSPECTOR_DOCK_EMPTY_GUIDE
        );
        assert_eq!(
            inspector_chrome_for("size_label", Locale::Es),
            INSPECTOR_SIZE_TEXT
        );
        assert_eq!(
            inspector_chrome_for("radius_label", Locale::Es),
            INSPECTOR_RADIUS_TEXT
        );
        assert_eq!(
            inspector_chrome_for("scale_label", Locale::Es),
            INSPECTOR_SCALE_TEXT
        );
        assert_eq!(
            inspector_chrome_for("edge_width_label", Locale::Es),
            INSPECTOR_EDGE_WIDTH_TEXT
        );
        // Helpers: Es = canónico byte-idéntico.
        assert_eq!(
            implicit_3d_box_text_for([0.0, 1.0, 0.0, 1.0, 0.0, 1.0], 12, Locale::Es),
            implicit_3d_box_text(0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 12)
        );
        assert_eq!(
            implicit_3d_state_text_for("ok", Locale::Es),
            implicit_3d_state_text("ok")
        );
        assert_eq!(
            implicit_3d_preview_text_for(7, Locale::Es),
            implicit_3d_preview_text(7)
        );
        // Resto: sale de i18n con templates interpolados.
        assert_eq!(
            inspector_chrome_for("cube3d_title", Locale::En),
            t("inspector.cube3d_title", Locale::En)
        );
        assert_eq!(
            inspector_chrome_for("dock_empty_guide", Locale::Pt),
            t("inspector.dock_empty_guide", Locale::Pt)
        );
        assert_eq!(
            implicit_3d_state_text_for("ok", Locale::Fr),
            t("inspector.implicit_state", Locale::Fr).replace("{status}", "ok")
        );
        assert_eq!(
            implicit_3d_preview_text_for(7, Locale::De),
            t("inspector.implicit_preview", Locale::De).replace("{tris}", "7")
        );
        assert_eq!(
            implicit_3d_box_text_for([0.0, 1.0, 0.0, 1.0, 0.0, 1.0], 12, Locale::It),
            t("inspector.implicit_box", Locale::It)
                .replace("{ranges}", "[0.00, 1.00] × [0.00, 1.00] × [0.00, 1.00]")
                .replace("{cells}", "12")
        );
    }

    #[test]
    fn sheet_inspector_b_a113_preview_delega_es_al_canonico() {
        // B-A113: vista previa implícita delega Es al canónico y resto a
        // i18n; la tabla `notify_for` espeja la canónica en las 4 ramas.
        use super::{
            implicit_cancel_label_for, implicit_quick_preview_notify,
            implicit_quick_preview_notify_for, inspector_chrome_for,
            IMPLICIT_PREVIEW_ALREADY_RUNNING, IMPLICIT_PREVIEW_ALREADY_SHOWN,
            IMPLICIT_PREVIEW_CANCELLED, IMPLICIT_PREVIEW_CANCEL_EMPTY,
            IMPLICIT_PREVIEW_CANCEL_HOVER, IMPLICIT_PREVIEW_CANCEL_LABEL, IMPLICIT_PREVIEW_RUNNING,
            IMPLICIT_PREVIEW_START_FAILED, IMPLICIT_QUICK_PREVIEW_GUIDE,
            IMPLICIT_QUICK_PREVIEW_HOVER, IMPLICIT_QUICK_PREVIEW_LABEL,
        };
        use grafito_ui::i18n::{t, Locale};
        // Dispatcher: Es = const canónica byte-idéntica.
        assert_eq!(
            inspector_chrome_for("implicit_quick_guide", Locale::Es),
            IMPLICIT_QUICK_PREVIEW_GUIDE
        );
        assert_eq!(
            inspector_chrome_for("implicit_quick_label", Locale::Es),
            IMPLICIT_QUICK_PREVIEW_LABEL
        );
        assert_eq!(
            inspector_chrome_for("implicit_quick_hover", Locale::Es),
            IMPLICIT_QUICK_PREVIEW_HOVER
        );
        assert_eq!(
            inspector_chrome_for("implicit_running", Locale::Es),
            IMPLICIT_PREVIEW_RUNNING
        );
        assert_eq!(
            inspector_chrome_for("implicit_already_shown", Locale::Es),
            IMPLICIT_PREVIEW_ALREADY_SHOWN
        );
        assert_eq!(
            inspector_chrome_for("implicit_already_running", Locale::Es),
            IMPLICIT_PREVIEW_ALREADY_RUNNING
        );
        assert_eq!(
            inspector_chrome_for("implicit_start_failed", Locale::Es),
            IMPLICIT_PREVIEW_START_FAILED
        );
        assert_eq!(
            inspector_chrome_for("implicit_cancelled", Locale::Es),
            IMPLICIT_PREVIEW_CANCELLED
        );
        assert_eq!(
            inspector_chrome_for("implicit_cancel_empty", Locale::Es),
            IMPLICIT_PREVIEW_CANCEL_EMPTY
        );
        assert_eq!(
            inspector_chrome_for("implicit_cancel_hover", Locale::Es),
            IMPLICIT_PREVIEW_CANCEL_HOVER
        );
        // Cancelar: Es = const canónica, resto = `common.cancel` (sin key).
        assert_eq!(
            implicit_cancel_label_for(Locale::Es),
            IMPLICIT_PREVIEW_CANCEL_LABEL
        );
        assert_eq!(
            implicit_cancel_label_for(Locale::En),
            t("common.cancel", Locale::En)
        );
        assert_eq!(
            implicit_cancel_label_for(Locale::De),
            t("common.cancel", Locale::De)
        );
        // `notify_for`: Es = canónico en las 4 ramas de la tabla.
        for (was, had, now) in [
            (false, false, true),
            (false, true, false),
            (true, false, false),
            (false, false, false),
        ] {
            assert_eq!(
                implicit_quick_preview_notify_for(was, had, now, Locale::Es),
                implicit_quick_preview_notify(was, had, now)
            );
        }
        // Resto: sale de i18n (misma tabla, 4 ramas).
        assert_eq!(
            implicit_quick_preview_notify_for(false, false, true, Locale::En),
            t("inspector.implicit_running", Locale::En)
        );
        assert_eq!(
            implicit_quick_preview_notify_for(false, true, false, Locale::Pt),
            t("inspector.implicit_already_shown", Locale::Pt)
        );
        assert_eq!(
            implicit_quick_preview_notify_for(true, false, false, Locale::It),
            t("inspector.implicit_already_running", Locale::It)
        );
        assert_eq!(
            implicit_quick_preview_notify_for(false, false, false, Locale::Fr),
            t("inspector.implicit_start_failed", Locale::Fr)
        );
        assert_eq!(
            implicit_quick_preview_notify_for(false, false, true, Locale::De),
            t("inspector.implicit_running", Locale::De)
        );
        assert_eq!(
            inspector_chrome_for("implicit_quick_guide", Locale::De),
            t("inspector.implicit_quick_guide", Locale::De)
        );
        assert_eq!(inspector_chrome_for("nope", Locale::Es), "nope");
    }

    #[test]
    fn sheet_params_textos_pinnea_panel() {
        // Paridad con `draw_right_parameters_panel` en `panels.rs`: mismo
        // título/subtítulo, misma guía sin attractor, mismas secciones
        // Lorenz/Integración, mismo hint de regeneración y misma línea de
        // error honesto, dueño nuevo (slice 50). El `SidePanel`, los
        // sliders, el commit con undo y el drawer quedan en panels.
        use super::{
            params_error_text, PARAMS_EMPTY_GUIDE, PARAMS_INTEGRATION_SECTION,
            PARAMS_LORENZ_SECTION, PARAMS_PANEL_SUBTITLE, PARAMS_PANEL_TITLE, PARAMS_REGEN_HINT,
        };
        assert_eq!(PARAMS_PANEL_TITLE, "Parámetros dinámicos");
        assert_eq!(PARAMS_PANEL_SUBTITLE, "Lorenz y sistemas vivos.");
        assert_eq!(
            PARAMS_EMPTY_GUIDE,
            "Sin attractor activo. Creá uno con:\n  Attractor[10, 28, 8/3]"
        );
        assert_eq!(PARAMS_LORENZ_SECTION, "Lorenz sigma, rho, beta");
        assert_eq!(PARAMS_INTEGRATION_SECTION, "Integración");
        assert_eq!(PARAMS_REGEN_HINT, "El canvas se regenera cada cambio.");
        assert_eq!(params_error_text("boom"), "Parámetros: boom");
    }

    #[test]
    fn sheet_dynamics_chrome_for_delega_es_al_canonico() {
        // B-A189: Es delega a las consts canónicas `ATTRACTOR_*`/`PARAMS_*`
        // (paridad byte-idéntica); resto va a `dynamics_msg`.
        use super::{
            dynamics_chrome_for, ATTRACTOR_ACTIVE_TITLE, ATTRACTOR_COMMANDS_TITLE,
            ATTRACTOR_CREATE_DEFAULT_LABEL, ATTRACTOR_INACTIVE_GUIDE, ATTRACTOR_LORENZ_HINT,
            ATTRACTOR_OBJECTS_EMPTY, ATTRACTOR_OBJECTS_TITLE, ATTRACTOR_PANEL_TITLE,
            PARAMS_EMPTY_GUIDE, PARAMS_INTEGRATION_SECTION, PARAMS_LORENZ_SECTION,
            PARAMS_PANEL_SUBTITLE, PARAMS_PANEL_TITLE, PARAMS_REGEN_HINT, PARAMS_STEPS_LABEL,
        };
        use grafito_ui::i18n::Locale;
        for (suffix, canonical) in [
            ("attractor_panel_title", ATTRACTOR_PANEL_TITLE),
            ("attractor_objects_title", ATTRACTOR_OBJECTS_TITLE),
            ("attractor_objects_empty", ATTRACTOR_OBJECTS_EMPTY),
            ("attractor_active_title", ATTRACTOR_ACTIVE_TITLE),
            ("attractor_inactive_guide", ATTRACTOR_INACTIVE_GUIDE),
            ("attractor_create_default", ATTRACTOR_CREATE_DEFAULT_LABEL),
            ("attractor_commands_title", ATTRACTOR_COMMANDS_TITLE),
            ("attractor_lorenz_hint", ATTRACTOR_LORENZ_HINT),
            ("params_panel_title", PARAMS_PANEL_TITLE),
            ("params_panel_subtitle", PARAMS_PANEL_SUBTITLE),
            ("params_empty_guide", PARAMS_EMPTY_GUIDE),
            ("params_lorenz_section", PARAMS_LORENZ_SECTION),
            ("params_integration_section", PARAMS_INTEGRATION_SECTION),
            ("params_regen_hint", PARAMS_REGEN_HINT),
            ("params_steps_label", PARAMS_STEPS_LABEL),
        ] {
            assert_eq!(dynamics_chrome_for(suffix, Locale::Es), canonical);
            assert_eq!(
                dynamics_chrome_for(suffix, Locale::En),
                grafito_ui::i18n::dynamics_msg(suffix, Locale::En)
            );
        }
        assert_eq!(PARAMS_STEPS_LABEL, "pasos");
        assert_eq!(dynamics_chrome_for("nope", Locale::Es), "nope");
    }

    #[test]
    fn sheet_params_error_b_a190_localiza_con_template() {
        // B-A190: `params_error_text_for` delega Es al canónico
        // `params_error_text` (paridad byte-idéntica, patrón B-A98);
        // resto interpola `dynamics.params_error` (`{error}`).
        use super::{params_error_text, params_error_text_for};
        use grafito_ui::i18n::Locale;
        assert_eq!(
            params_error_text_for("boom", Locale::Es),
            params_error_text("boom")
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
            params_error_text_for("boom", Locale::Pt),
            "Parâmetros: boom"
        );
        assert_eq!(params_error_text_for("boom", Locale::De), "Parameter: boom");
        assert_eq!(
            params_error_text_for("boom", Locale::Fr),
            ["Paramètres : ", "boom"].concat()
        );
    }

    #[test]
    fn sheet_domain_coloring_textos_pinnea_panel() {
        // Paridad con `draw_right_domain_coloring_panel` en `panels.rs`:
        // mismo título/subtítulo, misma guía vacía, mismo título activo,
        // misma línea de tono, misma guía con sus dos secciones, mismo
        // rótulo de modo con sus 4 modos + caída, y mismo rótulo/hint de
        // símbolo, dueño nuevo (slice 51). El `SidePanel`, el `ComboBox`,
        // el `TextEdit` del símbolo y el drawer quedan en panels.
        use super::{
            domain_coloring_mode_label, DOMAIN_COLORING_ACTIVE_TITLE, DOMAIN_COLORING_COLORS_BODY,
            DOMAIN_COLORING_COLORS_TITLE, DOMAIN_COLORING_DERIV_BODY, DOMAIN_COLORING_DERIV_TITLE,
            DOMAIN_COLORING_EMPTY_GUIDE, DOMAIN_COLORING_GUIDE_TITLE, DOMAIN_COLORING_HUE_LINE,
            DOMAIN_COLORING_MODE_LABEL, DOMAIN_COLORING_SUBTITLE, DOMAIN_COLORING_SYMBOL_HINT,
            DOMAIN_COLORING_SYMBOL_LABEL, DOMAIN_COLORING_TITLE,
        };
        assert_eq!(DOMAIN_COLORING_TITLE, "Coloración de dominio");
        assert_eq!(DOMAIN_COLORING_SUBTITLE, "Fase y módulo en el plano.");
        assert_eq!(
            DOMAIN_COLORING_EMPTY_GUIDE,
            "Sin coloración de dominio. Creá una con:\n  DomainColoring[1/z, -2, 2, -2, 2, 160]\nFase y módulo de f(z)."
        );
        assert_eq!(
            DOMAIN_COLORING_ACTIVE_TITLE,
            "Coloración por fase habilitada"
        );
        assert_eq!(DOMAIN_COLORING_HUE_LINE, "Tono = arg(f(z)).");
        assert_eq!(DOMAIN_COLORING_GUIDE_TITLE, "Guia de Interpretacion");
        assert_eq!(DOMAIN_COLORING_COLORS_TITLE, "Colores y Magnitud:");
        assert!(DOMAIN_COLORING_COLORS_BODY.contains("Negro = Raiz (0)"));
        assert_eq!(DOMAIN_COLORING_DERIV_TITLE, "Derivabilidad y Wirtinger:");
        assert!(DOMAIN_COLORING_DERIV_BODY.contains("Cauchy-Riemann"));
        assert_eq!(DOMAIN_COLORING_MODE_LABEL, "Modo de coloración");
        assert_eq!(domain_coloring_mode_label(0), "HSL Clásico (Fase + Módulo)");
        assert_eq!(domain_coloring_mode_label(1), "Retrato de Fase Puro");
        assert_eq!(domain_coloring_mode_label(2), "Rejilla Polar Conforme");
        assert_eq!(domain_coloring_mode_label(3), "Rejilla Cartesiana Conforme");
        assert_eq!(domain_coloring_mode_label(99), "HSL Clásico");
        assert_eq!(DOMAIN_COLORING_SYMBOL_LABEL, "Símbolo base");
        assert_eq!(DOMAIN_COLORING_SYMBOL_HINT, "z");
    }

    #[test]
    fn sheet_empty_panel_textos_pinnea_panel() {
        // Paridad con `draw_empty_panel` en `panels.rs`: mismo título y
        // misma guía, dueño nuevo (slice 52). El `SidePanel` centrado
        // queda en panels.
        use super::{EMPTY_PANEL_GUIDE, EMPTY_PANEL_TITLE};
        assert_eq!(EMPTY_PANEL_TITLE, "Sin panel aquí");
        assert_eq!(
            EMPTY_PANEL_GUIDE,
            "Cambiá de perspectiva o abrí un panel desde «Paneles»."
        );
    }

    #[test]
    fn sheet_view_general_ejes_pinnea_textos_del_dibujo() {
        // Paridad con `draw_view_panel` en `panels.rs`: mismos títulos
        // General/Ejes y mismas etiquetas de toggles, dueño nuevo
        // (slice 53). Los `SidePanel`/`scandi_checkbox`/`exam_mode_checkbox`
        // y el resto del dibujo quedan en panels.
        use super::{
            VIEW_AXES_TITLE, VIEW_DARK_MODE_LABEL, VIEW_GENERAL_TITLE, VIEW_NUMBER_PLANE_HINT,
            VIEW_NUMBER_PLANE_LABEL, VIEW_SHOW_GRID_LABEL, VIEW_SNAP_LABEL, VIEW_X_LOG_LABEL,
            VIEW_Y_LOG_LABEL,
        };
        assert_eq!(VIEW_GENERAL_TITLE, "General");
        assert_eq!(VIEW_SHOW_GRID_LABEL, "Mostrar cuadrícula");
        assert_eq!(VIEW_DARK_MODE_LABEL, "Modo oscuro");
        assert_eq!(VIEW_SNAP_LABEL, "Ajustar a cuadrícula");
        assert_eq!(VIEW_AXES_TITLE, "Ejes");
        assert_eq!(VIEW_X_LOG_LABEL, "Eje X logarítmico");
        assert_eq!(VIEW_Y_LOG_LABEL, "Eje Y logarítmico");
        assert_eq!(
            VIEW_NUMBER_PLANE_LABEL,
            "Plano numerado (ticks + etiquetas + origen)"
        );
        assert_eq!(
            VIEW_NUMBER_PLANE_HINT,
            "Muestra los números de los ejes con pasos lindos 1/2/5×10^n y skip anti-solape"
        );
    }

    #[test]
    fn sheet_view_chrome_for_delega_es_al_canonico() {
        // B-A128: el dispatcher Es delega a las consts canónicas de arriba;
        // resto va a `view_msg`. Paridad Es×9 + En×9 + unknown.
        // B-A129: +14 (Vista 3D + Capas estático + AltaPrecisión).
        // B-A130: +3 (hints de Capas + sin-etiqueta, `:862-878`).
        use super::view_chrome_for;
        use grafito_ui::i18n::Locale;
        for (suffix, canonico) in [
            ("general_title", super::VIEW_GENERAL_TITLE),
            ("show_grid", super::VIEW_SHOW_GRID_LABEL),
            ("dark_mode", super::VIEW_DARK_MODE_LABEL),
            ("snap", super::VIEW_SNAP_LABEL),
            ("axes_title", super::VIEW_AXES_TITLE),
            ("x_log", super::VIEW_X_LOG_LABEL),
            ("y_log", super::VIEW_Y_LOG_LABEL),
            ("number_plane", super::VIEW_NUMBER_PLANE_LABEL),
            ("number_plane_hint", super::VIEW_NUMBER_PLANE_HINT),
            ("section_3d", super::VIEW_3D_SECTION_TITLE),
            ("ortho_hover", super::VIEW_3D_ORTHO_HOVER),
            ("layers_title", super::VIEW_LAYERS_TITLE),
            ("layer_show", super::LAYER_TOGGLE_SHOW),
            ("layer_hide", super::LAYER_TOGGLE_HIDE),
            ("assign_layer", super::VIEW_ASSIGN_LAYER_LABEL),
            ("assign", super::VIEW_ASSIGN_LABEL),
            ("highlight", super::VIEW_HIGHLIGHT_LABEL),
            ("indicate", super::VIEW_INDICATE_LABEL),
            ("indicate_hover", super::VIEW_INDICATE_HOVER),
            ("indicating", super::VIEW_INDICATING),
            ("hp_title", super::VIEW_HIGH_PRECISION_TITLE),
            ("hp_label", super::VIEW_HIGH_PRECISION_LABEL),
            ("hp_hint", super::VIEW_HIGH_PRECISION_HINT),
            ("layer_empty", super::LAYER_EMPTY_HINT),
            ("layer_no_selection", super::LAYER_NO_SELECTION),
            ("layer_untagged", "<sin etiqueta>"),
        ] {
            assert_eq!(view_chrome_for(suffix, Locale::Es), canonico);
            assert_eq!(
                view_chrome_for(suffix, Locale::En),
                grafito_ui::i18n::view_msg(suffix, Locale::En)
            );
        }
        assert_eq!(view_chrome_for("nope", Locale::Es), "nope");
    }

    #[test]
    fn sheet_export_chrome_for_delega_es_al_canonico() {
        // B-A169: Es delega a las consts canónicas `EXPORT_SECTION_TITLE`/
        // `SUBTITLE`/`EXPORT_FILE_BUTTONS`; resto va a `export_msg`.
        // Paridad Es×10 + En×10 + unknown. Siglas SVG/PNG/TikZ/PDF intactas.
        use super::export_chrome_for;
        use grafito_ui::i18n::Locale;
        for (suffix, canonico) in [
            ("section_title", super::EXPORT_SECTION_TITLE),
            ("section_subtitle", super::EXPORT_SECTION_SUBTITLE),
            ("file_svg_tip", super::EXPORT_FILE_BUTTONS[0].1),
            ("file_png_tip", super::EXPORT_FILE_BUTTONS[1].1),
            ("file_tikz_tip", super::EXPORT_FILE_BUTTONS[2].1),
            ("file_pdf_tip", super::EXPORT_FILE_BUTTONS[3].1),
            ("file_copy_svg", super::EXPORT_FILE_BUTTONS[4].0),
            ("file_copy_svg_tip", super::EXPORT_FILE_BUTTONS[4].1),
            ("file_copy_png", super::EXPORT_FILE_BUTTONS[5].0),
            ("file_copy_png_tip", super::EXPORT_FILE_BUTTONS[5].1),
        ] {
            assert_eq!(export_chrome_for(suffix, Locale::Es), canonico);
            assert_eq!(
                export_chrome_for(suffix, Locale::En),
                grafito_ui::i18n::export_msg(suffix, Locale::En)
            );
        }
        assert_eq!(export_chrome_for("nope", Locale::Es), "nope");
        assert_eq!(export_chrome_for("section_title", Locale::Pt), "Exportação");
        assert_eq!(
            export_chrome_for("file_copy_png", Locale::De),
            "PNG kopieren"
        );
    }

    #[test]
    fn sheet_export_b_a170_resto_delega_es_al_canonico() {
        // B-A170: Es delega a las consts canónicas `EXPORT_TEXT_BUTTONS` (tips) +
        // `EXPORT_TABLES_TITLE`/`EXPORT_CSV_TOOLTIP` + 3 `EXPORT_NOTIFY_*` +
        // `EXPORT_REVEAL_LABEL/TOOLTIP`; resto va a `export_msg`. Paridad Es×12 +
        // En×12 + templates + unknown. Labels `MathML`/`TikZ-eje`/`HTML`/`CSV`
        // siglas intactas (precedente B-A169).
        use super::export_chrome_for;
        use grafito_ui::i18n::Locale;
        for (suffix, canonico) in [
            ("text_mathml_tip", super::EXPORT_TEXT_BUTTONS[0].1),
            ("text_tikz_tip", super::EXPORT_TEXT_BUTTONS[1].1),
            ("text_html_tip", super::EXPORT_TEXT_BUTTONS[2].1),
            ("tables_title", super::EXPORT_TABLES_TITLE),
            ("csv_tip", super::EXPORT_CSV_TOOLTIP),
            ("notify_pdf", super::EXPORT_NOTIFY_PDF),
            ("notify_png", super::EXPORT_NOTIFY_PNG),
            ("notify_csv", super::EXPORT_NOTIFY_CSV),
            ("reveal_label", super::EXPORT_REVEAL_LABEL),
            ("reveal_tip", super::EXPORT_REVEAL_TOOLTIP),
        ] {
            assert_eq!(export_chrome_for(suffix, Locale::Es), canonico);
            assert_eq!(
                export_chrome_for(suffix, Locale::En),
                grafito_ui::i18n::export_msg(suffix, Locale::En)
            );
        }
        // Fila: Es delega al `format!` canónico, resto al template localizado.
        assert_eq!(
            super::export_table_row_label_for("T1", 5, Locale::Es),
            super::export_table_row_label("T1", 5)
        );
        assert_eq!(
            super::export_table_row_label_for("T1", 5, Locale::En),
            "T1 · 5 rows"
        );
        assert_eq!(
            super::export_table_row_label_for("T1", 5, Locale::De),
            "T1 · 5 Zeilen"
        );
        assert_eq!(export_chrome_for("nope", Locale::Es), "nope");
        assert_eq!(
            export_chrome_for("tables_title", Locale::Pt),
            "Tabelas (CSV RFC 4180)"
        );
        assert_eq!(
            export_chrome_for("reveal_label", Locale::De),
            "Im Ordner anzeigen"
        );
    }

    #[test]
    fn sheet_cas_b_a177_chrome_delega_es_al_canonico() {
        // B-A177: Es delega a las consts canónicas CAS; resto va a `cas_msg`.
        // Paridad Es×14 + En×14 + templates + unknown. `CAS_ENTRY_HINT`/
        // `CAS_ENTRY_KEYS_HINT` + `cas_cell_number`/`cas_cell_input_line` +
        // snippets invariantes (notación, B-A98/B-A114).
        use super::cas_chrome_for;
        use grafito_ui::i18n::Locale;
        for (suffix, canonico) in [
            ("panel_title", super::CAS_PANEL_TITLE),
            ("clear_label", super::CAS_CLEAR_LABEL),
            ("clear_tooltip", super::CAS_CLEAR_TOOLTIP),
            ("entry_label", super::CAS_ENTRY_LABEL),
            ("execute_tooltip", super::CAS_EXECUTE_TOOLTIP),
            ("quick_section_title", super::CAS_QUICK_SECTION_TITLE),
            ("quick_section_subtitle", super::CAS_QUICK_SECTION_SUBTITLE),
            ("quick_derive", super::CAS_QUICK_ACTIONS[0].0),
            ("quick_integrate", super::CAS_QUICK_ACTIONS[1].0),
            ("quick_solve", super::CAS_QUICK_ACTIONS[2].0),
            ("quick_limit", super::CAS_QUICK_ACTIONS[3].0),
            ("empty_title", super::CAS_EMPTY_TITLE),
            ("empty_hint", super::CAS_EMPTY_HINT),
            ("empty_example", super::CAS_EMPTY_EXAMPLE),
        ] {
            assert_eq!(cas_chrome_for(suffix, Locale::Es), canonico);
            assert_eq!(
                cas_chrome_for(suffix, Locale::En),
                grafito_ui::i18n::cas_msg(suffix, Locale::En)
            );
        }
        // Rótulos rápidos por índice: Es delega a la tupla canónica.
        for (i, (label, _)) in super::CAS_QUICK_ACTIONS.iter().enumerate() {
            assert_eq!(super::cas_quick_label_for(i, Locale::Es), *label);
            assert!(!super::cas_quick_label_for(i, Locale::De).is_empty());
        }
        assert_eq!(super::cas_quick_label_for(9, Locale::Es), "");
        // Templates: Es delega al `format!` canónico, resto al localizado.
        assert_eq!(
            super::cas_quick_insert_tooltip_for("Solve[", Locale::Es),
            super::cas_quick_insert_tooltip("Solve[")
        );
        assert_eq!(
            super::cas_quick_insert_tooltip_for("Solve[", Locale::En),
            "Insert Solve["
        );
        assert_eq!(
            super::cas_sheet_title_for(3, Locale::Es),
            super::cas_sheet_title(3)
        );
        assert_eq!(super::cas_sheet_title_for(3, Locale::En), "Sheet · 3");
        assert_eq!(super::cas_sheet_title_for(3, Locale::De), "Blatt · 3");
        assert_eq!(cas_chrome_for("nope", Locale::Es), "nope");
        assert_eq!(
            cas_chrome_for("panel_title", Locale::Pt),
            "Cálculo Simbólico (CAS)"
        );
        assert_eq!(cas_chrome_for("clear_label", Locale::De), "Leeren");
    }

    #[test]
    fn sheet_export_b_a171_toasts_delegan_es_al_canonico() {
        // B-A171: Es delega a los canónicos `export_svg_copied_text`/
        // `export_svg_error_text`/`export_clipboard_copied_text`; resto va a
        // los templates `export.svg_copied/svg_error/clipboard_copied`.
        // Paridad Es + resto 6 idiomas + templates + labels intactos.
        use grafito_ui::i18n::Locale;
        assert_eq!(
            super::export_svg_copied_text_for(123, Locale::Es),
            super::export_svg_copied_text(123)
        );
        assert_eq!(
            super::export_svg_copied_text_for(123, Locale::En),
            "SVG copied to the clipboard (123 bytes)"
        );
        assert_eq!(
            super::export_svg_copied_text_for(7, Locale::De),
            "SVG in die Zwischenablage kopiert (7 Bytes)"
        );
        assert_eq!(
            super::export_svg_error_text_for("boom", Locale::Es),
            super::export_svg_error_text("boom")
        );
        assert_eq!(
            super::export_svg_error_text_for("boom", Locale::En),
            "Could not copy SVG: boom"
        );
        assert_eq!(
            super::export_svg_error_text_for("boom", Locale::Pt),
            "Não foi possível copiar o SVG: boom"
        );
        assert_eq!(
            super::export_clipboard_copied_text_for("MathML", 45, Locale::Es),
            super::export_clipboard_copied_text("MathML", 45)
        );
        assert_eq!(
            super::export_clipboard_copied_text_for("TikZ-eje", 7, Locale::En),
            "TikZ-eje copied to the clipboard (7 bytes)"
        );
        assert_eq!(
            super::export_clipboard_copied_text_for("HTML", 9, Locale::Fr),
            "HTML copié dans le presse-papiers (9 octets)"
        );
    }

    #[test]
    fn sheet_export_b_a173_identificadores_son_siglas_invariantes() {
        // B-A173 (cierre saga export): los únicos literales que quedan en el
        // draw son identificadores rfd/extensiones invariantes — filtros
        // "PDF"/"CSV", `format!("{stem}.csv")` (`panels.rs:1440,1580-1581`)
        // + `EXPORT_PDF_DEFAULT_FILENAME` / `EXPORT_CSV_LABEL` — idénticos en
        // los 6 idiomas por diseño (precedentes B-A90/B-A169/B-A162). 0 keys:
        // `EXPORT_KEYS` 25 sin `filter/filename`. Si se localizan, actualizar
        // este pin + `export_b_a173_...` en `panels.rs`.
        assert_eq!(super::EXPORT_CSV_LABEL, "CSV");
        assert_eq!(super::EXPORT_PDF_DEFAULT_FILENAME, "grafito_export.pdf");
        assert_eq!(format!("{}.csv", "T1"), "T1.csv");
        assert_eq!(grafito_ui::i18n::EXPORT_KEYS.len(), 25);
        for suffix in grafito_ui::i18n::EXPORT_KEYS {
            for denied in ["filter", "filename", "dialog", "extension"] {
                assert!(
                    !suffix.contains(denied),
                    "export sin key de identificador a propósito: {suffix}"
                );
            }
        }
        // S8: +1 `classroom.close` (cierre del bottom-sheet de Aula).
        // S4: +1 `anim.export_weight_estimate` (estimado previo al ffmpeg).
        // V3: +6 (`cal.fsrs_reviews`, `cal.review_now`, `mastery.title`,
        // `mastery.todo/doing/done`: capa FSRS + kanban auto del tab Plan).
        // F1 vault: +12 (`sort_*`, `expand_all`, `auto_reveal*`, `ctx_reveal`,
        // `move_*`: header compacto + auto-reveal + mover-con-filtro).
        // 2026-10-12: purga study/vault/center muertos (−176: 106 study + 58
        // vault + 12 center; viven vault.backlinks_title + 2 center) → 1441;
        // 2026-10-13: purga onboarding/center muertos (−8) → 1433.
        // 2026-10-14: +3 `assistant.empty.suggest_*` (caminos del empty-state) → 1436.
        // 2026-10-14: +2 `toast.first_*` (celebración contenida) → 1438.
        // 2026-10-14: +1 `assistant.header.streak` + 1 `exam.modal_verdict`
        // (racha visible + veredicto cálido, nivel reflexivo) → 1440.
        // 2026-10-14: +1 `algebra.empty_action` + 2 `anim.export_quick*`
        // (empty cálido con siembra + GIF rápido a Descargas) → 1443.
        // 2026-10-14: +2 `assistant.steps.animate*` («Ver animación» de la
        // tarjeta de pasos) → 1445.
        // 2026-10-15: −4 `assistant.empty.*` (chips del empty-state fuera
        // — directiva usuario: sin opciones, avatar solo centrado) → 1443.
        // 2026-10-15: +11 `classroom.role_*/present*/ask/follow/attending/exit_focus/curtain`
        // (roles Profesor/Alumno en Aula: selector + Presentar/Preguntar/Seguir + overlay foco) → 1454.
        // 2026-10-15: +5 `assistant.steps.read*/stop*` (botón «Leer» TTS de la
        // tarjeta de pasos: voz espeak-ng solo al pulsar) → 1459.
        // 2026-10-15: +4 `assistant.composer.dictate*/stop/recording/missing`
        // (mic STT del composer: dicta local con whisper-cli, inserta sin
        // enviar, hover honesto con comandos) → 1463.
        // 2026-10-16: +11 `agent.*` (transcriptor del agente) + 1
        // `teach.reduced_motion_active` + 2 `menu.file.export_lesson*` + 2
        // `menu.file.lesson_vault*` (lección al vault en 1 clic) − 1
        // `center.menu_reopen` muerta → 1480.
        assert_eq!(grafito_ui::i18n::MSG_COUNT, 1480);
    }

    #[test]
    fn sheet_layer_toggle_label_for_delega_es_al_canonico() {
        // B-A129: Es delega a la canónica `layer_toggle_label`, resto a
        // `view_msg` (ternaria Mostrar/Ocultar localizada).
        use super::{layer_toggle_label, layer_toggle_label_for};
        use grafito_ui::i18n::Locale;
        assert_eq!(
            layer_toggle_label_for(true, Locale::Es),
            layer_toggle_label(true)
        );
        assert_eq!(
            layer_toggle_label_for(false, Locale::Es),
            layer_toggle_label(false)
        );
        assert_eq!(
            layer_toggle_label_for(true, Locale::En),
            grafito_ui::i18n::view_msg("layer_hide", Locale::En)
        );
        assert_eq!(
            layer_toggle_label_for(false, Locale::En),
            grafito_ui::i18n::view_msg("layer_show", Locale::En)
        );
        assert_eq!(layer_toggle_label_for(true, Locale::Pt), "Ocultar");
        assert_eq!(layer_toggle_label_for(false, Locale::De), "Anzeigen");
    }

    #[test]
    fn sheet_view_b_a130_helpers_delegan_es_al_canonico() {
        // B-A130: Es delega a las canónicas de Capas (`sheet_layout.rs:862-914`
        // + `:4105`); resto interpola los templates `view.*` (`{layer}`/
        // `{count}`/`{touched}`/`{label}`/`{error}`). Wrappers Es intactos.
        use super::{
            layer_assign_error, layer_assign_error_for, layer_assign_result,
            layer_assign_result_for, layer_display_name, layer_display_name_for, layer_row_label,
            layer_row_label_for, layer_selection_text, layer_selection_text_for,
            layer_toggle_result, layer_toggle_result_for, view_chrome_for, view_indicate_error,
            view_indicate_error_for, LAYER_EMPTY_HINT, LAYER_NO_SELECTION,
        };
        use grafito_ui::i18n::Locale;
        // Dispatcher: 3 estáticas nuevas.
        assert_eq!(view_chrome_for("layer_empty", Locale::Es), LAYER_EMPTY_HINT);
        assert_eq!(
            view_chrome_for("layer_no_selection", Locale::Es),
            LAYER_NO_SELECTION
        );
        assert_eq!(
            view_chrome_for("layer_untagged", Locale::Es),
            "<sin etiqueta>"
        );
        assert_eq!(
            view_chrome_for("layer_empty", Locale::En),
            grafito_ui::i18n::view_msg("layer_empty", Locale::En)
        );
        // Paridad Es ×7 + None por tocado 0.
        assert_eq!(
            layer_display_name_for("", Locale::Es),
            layer_display_name("")
        );
        assert_eq!(layer_display_name_for("C", Locale::Es), "C");
        assert_eq!(layer_row_label_for(2, 5, Locale::Es), layer_row_label(2, 5));
        assert_eq!(
            layer_toggle_result_for(2, true, 3, Locale::Es),
            layer_toggle_result(2, true, 3)
        );
        assert_eq!(layer_toggle_result_for(2, true, 0, Locale::Es), None);
        assert_eq!(
            layer_selection_text_for("C", 2, Locale::Es),
            layer_selection_text("C", 2)
        );
        assert_eq!(
            layer_assign_result_for("C", 2, Locale::Es),
            layer_assign_result("C", 2)
        );
        assert_eq!(
            layer_assign_error_for("boom", Locale::Es),
            layer_assign_error("boom")
        );
        assert_eq!(
            view_indicate_error_for("boom", Locale::Es),
            view_indicate_error("boom")
        );
        // Resto: templates sin placeholders crudos + untagged localizado.
        assert_eq!(layer_display_name_for("", Locale::En), "<untagged>");
        assert_eq!(layer_display_name_for("", Locale::Pt), "<sem rótulo>");
        assert_eq!(layer_row_label_for(2, 5, Locale::En), "Layer 2 · 5 objs");
        assert_eq!(
            layer_toggle_result_for(2, true, 3, Locale::En).as_deref(),
            Some("Layer 2 hidden (3 objs)")
        );
        assert_eq!(
            layer_toggle_result_for(2, false, 3, Locale::En).as_deref(),
            Some("Layer 2 shown (3 objs)")
        );
        assert_eq!(layer_toggle_result_for(2, true, 0, Locale::En), None);
        assert_eq!(
            layer_selection_text_for("C", 2, Locale::En),
            "Selection: C (layer 2)"
        );
        assert_eq!(
            layer_selection_text_for("", 2, Locale::Pt),
            "Seleção: <sem rótulo> (camada 2)"
        );
        assert_eq!(
            layer_assign_result_for("C", 2, Locale::It),
            "'C' → livello 2"
        );
        assert_eq!(
            layer_assign_error_for("boom", Locale::Fr),
            "Impossible d'assigner : boom"
        );
        assert_eq!(
            view_indicate_error_for("boom", Locale::De),
            "Konnte nicht kennzeichnen: boom"
        );
        for text in [
            layer_row_label_for(2, 5, Locale::En),
            layer_toggle_result_for(2, true, 3, Locale::En).unwrap(),
            layer_selection_text_for("C", 2, Locale::En),
            layer_assign_result_for("C", 2, Locale::En),
            layer_assign_error_for("boom", Locale::En),
            view_indicate_error_for("boom", Locale::En),
        ] {
            assert!(!text.contains("{layer}"), "template sin interpolar: {text}");
            assert!(!text.contains("{count}"), "template sin interpolar: {text}");
            assert!(
                !text.contains("{touched}"),
                "template sin interpolar: {text}"
            );
        }
    }

    #[test]
    fn sheet_view_b_a131_helpers_delegan_es_al_canonico() {
        // B-A131: Es delega a las canónicas del parámetro vivo
        // (`sheet_layout.rs:4153-4173`); resto interpola los templates
        // `view.live_*` (`{value}`/`{min}`/`{max}` con 3 decimales como el
        // dibujo, `{error}`). Wrappers Es intactos.
        use super::{
            live_param_error_text, live_param_error_text_for, live_param_hover_text,
            live_param_hover_text_for, live_param_readout_text, live_param_readout_text_for,
            view_chrome_for, LIVE_PARAM_CREATED, LIVE_PARAM_CREATE_HOVER, LIVE_PARAM_CREATE_LABEL,
            LIVE_PARAM_MISSING_GUIDE, LIVE_PARAM_SECTION_TITLE,
        };
        use grafito_ui::i18n::Locale;
        // Dispatcher: 5 estáticas nuevas.
        assert_eq!(
            view_chrome_for("live_section", Locale::Es),
            LIVE_PARAM_SECTION_TITLE
        );
        assert_eq!(
            view_chrome_for("live_missing", Locale::Es),
            LIVE_PARAM_MISSING_GUIDE
        );
        assert_eq!(
            view_chrome_for("live_create", Locale::Es),
            LIVE_PARAM_CREATE_LABEL
        );
        assert_eq!(
            view_chrome_for("live_create_hover", Locale::Es),
            LIVE_PARAM_CREATE_HOVER
        );
        assert_eq!(
            view_chrome_for("live_created", Locale::Es),
            LIVE_PARAM_CREATED
        );
        assert_eq!(
            view_chrome_for("live_section", Locale::En),
            grafito_ui::i18n::view_msg("live_section", Locale::En)
        );
        // Paridad Es ×3.
        assert_eq!(
            live_param_hover_text_for(1.0, -5.0, 5.0, Locale::Es),
            live_param_hover_text(1.0, -5.0, 5.0)
        );
        assert_eq!(
            live_param_readout_text_for(1.0, -5.0, 5.0, Locale::Es),
            live_param_readout_text(1.0, -5.0, 5.0)
        );
        assert_eq!(
            live_param_error_text_for("boom", Locale::Es),
            live_param_error_text("boom")
        );
        // Resto: templates sin placeholders crudos.
        assert_eq!(
            live_param_hover_text_for(1.0, -5.0, 5.0, Locale::En),
            "p = 1.000 in [-5.000, 5.000] · functions with `p` resample themselves"
        );
        assert_eq!(
            live_param_readout_text_for(1.0, -5.0, 5.0, Locale::Pt),
            "p = 1.000 em [-5.000, 5.000]"
        );
        assert_eq!(
            live_param_error_text_for("boom", Locale::De),
            "Live-Parameter: boom"
        );
        for text in [
            live_param_hover_text_for(1.0, -5.0, 5.0, Locale::En),
            live_param_readout_text_for(1.0, -5.0, 5.0, Locale::En),
            live_param_error_text_for("boom", Locale::En),
        ] {
            assert!(!text.contains("{value}"), "template sin interpolar: {text}");
            assert!(!text.contains("{min}"), "template sin interpolar: {text}");
            assert!(!text.contains("{max}"), "template sin interpolar: {text}");
            assert!(!text.contains("{error}"), "template sin interpolar: {text}");
        }
    }

    #[test]
    fn sheet_vista3d_y_parametro_vivo_pinnea_textos_del_dibujo() {
        // Paridad con `draw_view_panel` en `panels.rs`: mismos títulos
        // Vista-3D + parámetro vivo y mismas cadenas del slider vivo,
        // dueño nuevo (slice 54). El `SidePanel`/combo/slider/undo
        // quedan en panels.
        use super::{
            live_param_error_text, live_param_hover_text, live_param_readout_text,
            LIVE_PARAM_CREATED, LIVE_PARAM_CREATE_HOVER, LIVE_PARAM_CREATE_LABEL,
            LIVE_PARAM_MISSING_GUIDE, LIVE_PARAM_SECTION_TITLE, VIEW_3D_ORTHO_HOVER,
            VIEW_3D_SECTION_TITLE,
        };
        assert_eq!(VIEW_3D_SECTION_TITLE, "Vista 3D");
        assert_eq!(
            VIEW_3D_ORTHO_HOVER,
            "En ortográficas el zoom y el paneo se conservan; orbitar con botón derecho vuelve a perspectiva."
        );
        assert_eq!(LIVE_PARAM_SECTION_TITLE, "Parámetro vivo");
        assert_eq!(
            LIVE_PARAM_MISSING_GUIDE,
            "Todavía no hay parámetro `p`: las funciones con `p` usan su valor por defecto."
        );
        assert_eq!(LIVE_PARAM_CREATE_LABEL, "Crear parámetro p en [-5, 5]");
        assert_eq!(LIVE_PARAM_CREATE_HOVER, "Crea la variable `p` con slider");
        assert_eq!(LIVE_PARAM_CREATED, "Parámetro `p` creado en [-5, 5]");
        assert_eq!(
            live_param_hover_text(1.23456, -5.0, 5.0),
            "p = 1.235 en [-5.000, 5.000] · las funciones con `p` se re-muestrean solas"
        );
        assert_eq!(
            live_param_readout_text(0.0, -5.0, 5.0),
            "p = 0.000 en [-5.000, 5.000]"
        );
        assert_eq!(
            live_param_error_text("rango vacío"),
            "Parámetro vivo: rango vacío"
        );
    }

    #[test]
    fn sheet_capas_indicar_alta_pinnea_textos_del_dibujo() {
        // Paridad con `draw_view_panel` en `panels.rs`: mismos títulos
        // Capas/Alta-precisión y mismas cadenas de asignar/indicar,
        // dueño nuevo (slice 55). El `SidePanel`/botones/sliders/undo
        // quedan en panels.
        use super::{
            layer_toggle_label, view_indicate_error, LAYER_TOGGLE_HIDE, LAYER_TOGGLE_SHOW,
            VIEW_ASSIGN_LABEL, VIEW_ASSIGN_LAYER_LABEL, VIEW_HIGHLIGHT_LABEL,
            VIEW_HIGH_PRECISION_HINT, VIEW_HIGH_PRECISION_LABEL, VIEW_HIGH_PRECISION_TITLE,
            VIEW_INDICATE_HOVER, VIEW_INDICATE_LABEL, VIEW_INDICATING, VIEW_LAYERS_TITLE,
        };
        assert_eq!(VIEW_LAYERS_TITLE, "Capas");
        assert_eq!(LAYER_TOGGLE_HIDE, "Ocultar");
        assert_eq!(LAYER_TOGGLE_SHOW, "Mostrar");
        assert_eq!(layer_toggle_label(true), "Ocultar");
        assert_eq!(layer_toggle_label(false), "Mostrar");
        assert_eq!(VIEW_ASSIGN_LAYER_LABEL, "Asignar a capa:");
        assert_eq!(VIEW_ASSIGN_LABEL, "Asignar");
        assert_eq!(VIEW_HIGHLIGHT_LABEL, "Resaltar:");
        assert_eq!(VIEW_INDICATE_LABEL, "Indicar");
        assert_eq!(VIEW_INDICATE_HOVER, "Hace latir la selección ~1.2 s");
        assert_eq!(VIEW_INDICATING, "Indicando selección (~1.2 s)");
        assert_eq!(
            view_indicate_error("sin objeto"),
            "No se pudo indicar: sin objeto"
        );
        assert_eq!(VIEW_HIGH_PRECISION_TITLE, "Alta Precisión");
        assert_eq!(VIEW_HIGH_PRECISION_LABEL, "Alta Precisión (Double-Double)");
        assert_eq!(
            VIEW_HIGH_PRECISION_HINT,
            "Usa aritmética Double-Double (~106 bits / 32 dígitos) para evaluar expresiones simbólicas sin pérdida de precisión."
        );
    }

    #[test]
    fn sheet_protocolo_textos_pinnea_panel() {
        // Paridad con `draw_construction_protocol` en `panels.rs`: mismos
        // títulos/acciones/nav/vacío/pasos, dueño nuevo (slice 56). El
        // `SidePanel`/botones/nav/drawer quedan en panels.
        use super::{
            protocol_counter_text, protocol_export_hover, protocol_play_glyph, protocol_step_badge,
            protocol_step_io_text, protocol_steps_title, PROTOCOL_CLEAR_HOVER,
            PROTOCOL_CLEAR_LABEL, PROTOCOL_DISABLED_LABEL, PROTOCOL_EMPTY_GUIDE,
            PROTOCOL_EMPTY_TITLE, PROTOCOL_EXPORTING, PROTOCOL_EXPORT_LABEL, PROTOCOL_LAST_GLYPH,
            PROTOCOL_LAST_TIP, PROTOCOL_PANEL_SUBTITLE, PROTOCOL_PANEL_TITLE, PROTOCOL_PAUSE_GLYPH,
            PROTOCOL_PLAY_GLYPH, PROTOCOL_PLAY_TIP, PROTOCOL_PREV_GLYPH, PROTOCOL_PREV_TIP,
            PROTOCOL_STEPS_HINT,
        };
        assert_eq!(PROTOCOL_PANEL_TITLE, "Protocolo de Construcción");
        assert_eq!(PROTOCOL_PANEL_SUBTITLE, "Pasos de construcción.");
        assert_eq!(PROTOCOL_EXPORT_LABEL, "Exportar LaTeX");
        assert_eq!(
            protocol_export_hover(true),
            "Guarda el protocolo como archivo .tex"
        );
        assert_eq!(protocol_export_hover(false), PROTOCOL_EMPTY_TITLE);
        assert_eq!(PROTOCOL_EXPORTING, "Exportando protocolo a LaTeX…");
        assert_eq!(PROTOCOL_CLEAR_LABEL, "Limpiar");
        assert_eq!(PROTOCOL_CLEAR_HOVER, "Vacía el historial de pasos");
        assert_eq!(PROTOCOL_PREV_GLYPH, "◀");
        assert_eq!(PROTOCOL_PREV_TIP, "Paso anterior");
        assert_eq!(protocol_play_glyph(true), PROTOCOL_PAUSE_GLYPH);
        assert_eq!(protocol_play_glyph(false), PROTOCOL_PLAY_GLYPH);
        assert_eq!(PROTOCOL_PLAY_TIP, "Reproducir el protocolo paso a paso");
        assert_eq!(PROTOCOL_LAST_GLYPH, "▶|");
        assert_eq!(PROTOCOL_LAST_TIP, "Último paso");
        assert_eq!(protocol_counter_text(2, 7), "2/7");
        assert_eq!(PROTOCOL_EMPTY_TITLE, "Sin pasos todavía");
        assert_eq!(
            PROTOCOL_EMPTY_GUIDE,
            "Creá objetos o restricciones para verlos acá."
        );
        assert_eq!(protocol_steps_title(3), "Pasos (3)");
        assert_eq!(
            PROTOCOL_STEPS_HINT,
            "Vista fiel del historial; no reordena ni desactiva restricciones reales."
        );
        assert_eq!(protocol_step_badge(7), "07");
        assert_eq!(PROTOCOL_DISABLED_LABEL, "deshabilitado");
        assert_eq!(
            protocol_step_io_text(&["A".to_string(), "B".to_string()], "C"),
            "A, B → C"
        );
        assert_eq!(protocol_step_io_text(&[], ""), "— → —");
    }

    #[test]
    fn sheet_regression_panel_textos_pinnea_panel() {
        // Paridad con `draw_right_regression_panel` + botón de
        // `draw_spreadsheet_section` en `panels.rs`: mismos
        // títulos/secciones/acciones/guías, dueño nuevo (slice 57). El
        // `SidePanel`/botones/grillas quedan en panels; el formato de
        // diagnósticos vive acá desde el slice 36.
        use super::{
            REGRESSION_CREATE_SUBTITLE, REGRESSION_CREATE_TITLE, REGRESSION_DIAGNOSTICS_SUBTITLE,
            REGRESSION_DIAGNOSTICS_TITLE, REGRESSION_FITS_EMPTY, REGRESSION_FITS_TITLE,
            REGRESSION_IMPORT_HOVER, REGRESSION_IMPORT_LABEL, REGRESSION_MODEL_HINT,
            REGRESSION_NO_TABLE_GUIDE, REGRESSION_PANEL_SUBTITLE, REGRESSION_PANEL_TITLE,
            REGRESSION_TEMPLATE_LOCAL_TABLE_NAME, REGRESSION_TEMPLATE_REGRESSION_NAME,
            REGRESSION_TEMPLATE_SCATTER_NAME, SPREADSHEET_IMPORT_SUBTITLE,
            SPREADSHEET_IMPORT_TITLE,
        };
        assert_eq!(REGRESSION_PANEL_TITLE, "Regresión");
        assert_eq!(REGRESSION_PANEL_SUBTITLE, "Ajuste a datos del documento.");
        assert_eq!(REGRESSION_FITS_TITLE, "Ajustes del documento");
        assert_eq!(REGRESSION_FITS_EMPTY, "Sin ajustes todavía.");
        assert_eq!(REGRESSION_DIAGNOSTICS_TITLE, "Diagnósticos");
        assert_eq!(
            REGRESSION_DIAGNOSTICS_SUBTITLE,
            "RMSE y R² por ajuste; residuales recortados a 24."
        );
        assert_eq!(REGRESSION_CREATE_TITLE, "Crear análisis");
        assert_eq!(
            REGRESSION_CREATE_SUBTITLE,
            "CSV/TSV de dos columnas o tabla local desde listas. La ruta nunca se guarda."
        );
        assert_eq!(REGRESSION_IMPORT_LABEL, "Importar CSV/TSV…");
        assert_eq!(
            REGRESSION_IMPORT_HOVER,
            "Dos columnas x,y · la ruta nunca se guarda"
        );
        assert_eq!(
            REGRESSION_MODEL_HINT,
            "El comando del modelo va a la entrada."
        );
        assert_eq!(
            REGRESSION_NO_TABLE_GUIDE,
            "Seleccioná una tabla local en el lienzo para elegir un modelo de ajuste."
        );
        // B-A148: nombres de plantilla + header del import de la hoja.
        assert_eq!(REGRESSION_TEMPLATE_SCATTER_NAME, "Dispersión");
        assert_eq!(REGRESSION_TEMPLATE_REGRESSION_NAME, "Regresión");
        assert_eq!(REGRESSION_TEMPLATE_LOCAL_TABLE_NAME, "Tabla local");
        assert_eq!(SPREADSHEET_IMPORT_TITLE, "Importar y ordenar");
        assert_eq!(SPREADSHEET_IMPORT_SUBTITLE, "Máx. 2 MB.");
    }

    #[test]
    fn sheet_regression_chrome_for_delega_es_al_canonico() {
        // B-A147: el dispatcher Es delega a las consts canónicas de arriba;
        // resto va a `stats_msg`. Paridad Es×12 + En×12 + unknown.
        // B-A148: +5 (3 nombres de plantilla + título/subtítulo del import
        // de la hoja).
        // B-A149: +7 (etiquetas de los 7 modelos del grid Regresión).
        use super::regression_chrome_for;
        use grafito_ui::i18n::Locale;
        for (suffix, canonico) in [
            ("regression_panel_title", super::REGRESSION_PANEL_TITLE),
            (
                "regression_panel_subtitle",
                super::REGRESSION_PANEL_SUBTITLE,
            ),
            ("regression_fits_title", super::REGRESSION_FITS_TITLE),
            ("regression_fits_empty", super::REGRESSION_FITS_EMPTY),
            (
                "regression_diagnostics_title",
                super::REGRESSION_DIAGNOSTICS_TITLE,
            ),
            (
                "regression_diagnostics_subtitle",
                super::REGRESSION_DIAGNOSTICS_SUBTITLE,
            ),
            ("regression_create_title", super::REGRESSION_CREATE_TITLE),
            (
                "regression_create_subtitle",
                super::REGRESSION_CREATE_SUBTITLE,
            ),
            ("regression_import_label", super::REGRESSION_IMPORT_LABEL),
            ("regression_import_hover", super::REGRESSION_IMPORT_HOVER),
            ("regression_model_hint", super::REGRESSION_MODEL_HINT),
            (
                "regression_no_table_guide",
                super::REGRESSION_NO_TABLE_GUIDE,
            ),
            (
                "regression_template_scatter",
                super::REGRESSION_TEMPLATE_SCATTER_NAME,
            ),
            (
                "regression_template_regression",
                super::REGRESSION_TEMPLATE_REGRESSION_NAME,
            ),
            (
                "regression_template_local_table",
                super::REGRESSION_TEMPLATE_LOCAL_TABLE_NAME,
            ),
            ("spreadsheet_import_title", super::SPREADSHEET_IMPORT_TITLE),
            (
                "spreadsheet_import_subtitle",
                super::SPREADSHEET_IMPORT_SUBTITLE,
            ),
            (
                "regression_model_linear",
                super::REGRESSION_MODEL_LINEAR_NAME,
            ),
            ("regression_model_poly2", super::REGRESSION_MODEL_POLY2_NAME),
            ("regression_model_exp", super::REGRESSION_MODEL_EXP_NAME),
            ("regression_model_log", super::REGRESSION_MODEL_LOG_NAME),
            ("regression_model_power", super::REGRESSION_MODEL_POWER_NAME),
            ("regression_model_sin", super::REGRESSION_MODEL_SIN_NAME),
            (
                "regression_model_logistic",
                super::REGRESSION_MODEL_LOGISTIC_NAME,
            ),
        ] {
            assert_eq!(regression_chrome_for(suffix, Locale::Es), canonico);
            assert_eq!(
                regression_chrome_for(suffix, Locale::En),
                grafito_ui::i18n::stats_msg(suffix, Locale::En)
            );
        }
        assert_eq!(
            regression_chrome_for("desconocido", Locale::Es),
            "desconocido"
        );
        assert_eq!(
            regression_chrome_for("desconocido", Locale::En),
            "desconocido"
        );
    }

    #[test]
    fn sheet_regression_b_a148_helpers_delegan_es_al_canonico() {
        // B-A148: los helpers `_for` delegan Es al canónico (paridad
        // byte-idéntica con `sheet_regresion_textos_puros_pinnea_dibujo`);
        // resto interpola `stats_msg` sin placeholders crudos. Comandos de
        // `REGRESSION_TEMPLATES` intactos en los 6 idiomas.
        use super::{
            regression_fit_diagnostics_line, regression_fit_diagnostics_line_for,
            regression_model_ready_text, regression_model_ready_text_for,
            regression_residuals_title, regression_residuals_title_for,
            regression_residuals_truncated_for, regression_residuals_truncated_note,
            regression_section_title, regression_section_title_for, regression_source_label,
            regression_source_label_for, regression_template_ready_text,
            regression_template_ready_text_for, regression_templates_for,
            REGRESSION_DELETED_SOURCE, REGRESSION_TEMPLATES,
        };
        use grafito_ui::i18n::Locale;
        // Paridad Es ×7.
        assert_eq!(
            regression_source_label_for(Some("T1"), Locale::Es),
            regression_source_label(Some("T1"))
        );
        assert_eq!(
            regression_source_label_for(None, Locale::Es),
            REGRESSION_DELETED_SOURCE
        );
        assert_eq!(
            regression_fit_diagnostics_line_for("T1", 0.5, 0.25, Locale::Es),
            regression_fit_diagnostics_line("T1", 0.5, 0.25)
        );
        assert_eq!(
            regression_residuals_title_for(3, Locale::Es),
            regression_residuals_title(3)
        );
        assert_eq!(
            regression_residuals_truncated_for(Locale::Es),
            regression_residuals_truncated_note()
        );
        assert_eq!(
            regression_section_title_for("T1", Locale::Es),
            regression_section_title("T1")
        );
        assert_eq!(
            regression_template_ready_text_for("Dispersión", Locale::Es),
            regression_template_ready_text("Dispersión")
        );
        assert_eq!(
            regression_model_ready_text_for("Lineal", Locale::Es),
            regression_model_ready_text("Lineal")
        );
        // Resto: templates interpolados, comandos intactos.
        assert_eq!(
            regression_source_label_for(None, Locale::En),
            "deleted table"
        );
        assert_eq!(regression_source_label_for(Some("T1"), Locale::De), "T1");
        assert_eq!(
            regression_fit_diagnostics_line_for("T1", 0.5, 0.25, Locale::En),
            "from T1 · RMSE 0.500000 · R² 0.250000"
        );
        assert_eq!(
            regression_residuals_title_for(3, Locale::Pt),
            "Resíduos (3)"
        );
        assert_eq!(
            regression_residuals_truncated_for(Locale::Fr),
            "Affichage des 24 premières valeurs."
        );
        assert_eq!(
            regression_section_title_for("T1", Locale::It),
            "Adatta «T1»"
        );
        assert_eq!(
            regression_template_ready_text_for("Scatter", Locale::En),
            "Scatter ready in the input (Enter to create)."
        );
        assert_eq!(
            regression_model_ready_text_for("Lineal", Locale::De),
            "Anpassung Lineal bereit in der Eingabe (Enter zum Erstellen)."
        );
        let templates_es = regression_templates_for(Locale::Es);
        assert_eq!(templates_es[0].0, REGRESSION_TEMPLATES[0].0);
        assert_eq!(templates_es[1].1, REGRESSION_TEMPLATES[1].1);
        assert_eq!(templates_es[2].1, REGRESSION_TEMPLATES[2].1);
        for locale in [
            Locale::Es,
            Locale::En,
            Locale::Pt,
            Locale::It,
            Locale::Fr,
            Locale::De,
        ] {
            let templates = regression_templates_for(locale);
            assert_eq!(templates.len(), 3);
            // Comandos invariantes en los 6 idiomas (precedente B-A98).
            assert_eq!(templates[0].1, "ScatterPlot[{1, 2, 3}, {1, 4, 9}]");
            assert_eq!(templates[1].1, "LinearRegression[{1, 2, 3}, {1, 4, 9}]");
            assert_eq!(templates[2].1, "DataTable[{0, 1, 2}, {1, 3, 5}]");
            assert!(!templates[0].0.is_empty());
            for text in [
                regression_fit_diagnostics_line_for("T1", 0.5, 0.25, locale),
                regression_residuals_title_for(3, locale),
                regression_residuals_truncated_for(locale),
                regression_section_title_for("T1", locale),
                regression_template_ready_text_for("X", locale),
                regression_model_ready_text_for("Y", locale),
            ] {
                assert!(
                    !text.contains("{origen}"),
                    "placeholder crudo en {locale:?}"
                );
                assert!(!text.contains("{rmse}"), "placeholder crudo en {locale:?}");
                assert!(!text.contains("{r2}"), "placeholder crudo en {locale:?}");
                assert!(!text.contains("{n})"), "placeholder crudo en {locale:?}");
                assert!(!text.contains("{max}"), "placeholder crudo en {locale:?}");
                assert!(!text.contains("{tabla}"), "placeholder crudo en {locale:?}");
                assert!(!text.contains("{name}"), "placeholder crudo en {locale:?}");
                assert!(!text.contains("{model}"), "placeholder crudo en {locale:?}");
            }
        }
    }

    #[test]
    fn sheet_regression_b_a149_modelos_localizan() {
        // B-A149: `regression_model_commands_for` delega Es a las consts
        // canónicas (paridad byte-idéntica con `regression_model_commands`)
        // y localiza el display en el resto (comandos `Fit*` invariantes en
        // los 6 idiomas, precedente B-A98/B-A148). Sufijo desconocido del
        // dispatcher → el sufijo (igual que `regression_chrome_for`).
        use super::{
            regression_chrome_for, regression_model_commands, regression_model_commands_for,
            REGRESSION_MODEL_EXP_NAME, REGRESSION_MODEL_LINEAR_NAME,
            REGRESSION_MODEL_LOGISTIC_NAME, REGRESSION_MODEL_LOG_NAME, REGRESSION_MODEL_POLY2_NAME,
            REGRESSION_MODEL_POWER_NAME, REGRESSION_MODEL_SIN_NAME,
        };
        use grafito_ui::i18n::Locale;
        let canonical = regression_model_commands("T1");
        let es = regression_model_commands_for("T1", Locale::Es);
        assert_eq!(es.len(), 7);
        for ((label, command), (canon_label, canon_command)) in es.iter().zip(canonical.iter()) {
            assert_eq!(label, canon_label);
            assert_eq!(command, canon_command);
        }
        assert_eq!(es[0].0, REGRESSION_MODEL_LINEAR_NAME);
        assert_eq!(es[1].0, REGRESSION_MODEL_POLY2_NAME);
        assert_eq!(es[2].0, REGRESSION_MODEL_EXP_NAME);
        assert_eq!(es[3].0, REGRESSION_MODEL_LOG_NAME);
        assert_eq!(es[4].0, REGRESSION_MODEL_POWER_NAME);
        assert_eq!(es[5].0, REGRESSION_MODEL_SIN_NAME);
        assert_eq!(es[6].0, REGRESSION_MODEL_LOGISTIC_NAME);
        assert_eq!(
            regression_model_commands_for("T1", Locale::En)[1].0,
            "Quadratic"
        );
        assert_eq!(
            regression_model_commands_for("T1", Locale::De)[6].0,
            "Logistisch"
        );
        assert_eq!(
            regression_chrome_for("desconocido", Locale::Es),
            "desconocido"
        );
        assert_eq!(
            regression_chrome_for("desconocido", Locale::En),
            "desconocido"
        );
        for locale in [
            Locale::Es,
            Locale::En,
            Locale::Pt,
            Locale::It,
            Locale::Fr,
            Locale::De,
        ] {
            let models = regression_model_commands_for("T1", locale);
            assert_eq!(models.len(), 7);
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
                assert!(commands.contains(&expected), "falta {expected}");
            }
            for (label, _) in &models {
                assert!(!label.is_empty(), "etiqueta vacía en {locale:?}");
            }
        }
    }

    #[test]
    fn sheet_series_b_a150_preview_y_success_localizan() {
        // B-A150: `_for` delega Es al canónico (paridad byte-idéntica) y
        // localiza el resto con templates `{count}/{list}/{range}`; `…` de
        // truncado intacto como notación (precedente B-A98).
        use super::{
            format_series_preview_text, format_series_preview_text_for, format_series_success,
            format_series_success_for,
        };
        use grafito_ui::i18n::Locale;
        let first = vec!["1".to_string(), "2".to_string(), "3".to_string()];
        assert_eq!(
            format_series_preview_text_for(3, &first, Locale::Es),
            format_series_preview_text(3, &first)
        );
        assert_eq!(
            format_series_success_for(2, "A1:A8", Locale::Es),
            format_series_success(2, "A1:A8")
        );
        assert_eq!(format_series_preview_text(3, &first), "3 celdas: 1, 2, 3");
        assert_eq!(
            format_series_success(2, "A1:A8"),
            "Serie: 2 celda(s) rellenadas en A1:A8"
        );
        // Truncado: `…` se conserva en Es y en el resto.
        let first_corto = vec!["1".to_string()];
        assert!(format_series_preview_text(3, &first_corto).ends_with('…'));
        assert!(format_series_preview_text_for(3, &first_corto, Locale::En).ends_with('…'));
        for locale in [
            Locale::Es,
            Locale::En,
            Locale::Pt,
            Locale::It,
            Locale::Fr,
            Locale::De,
        ] {
            let preview = format_series_preview_text_for(2, &first, locale);
            assert!(
                !preview.contains("{count}"),
                "placeholder crudo en {locale:?}"
            );
            assert!(
                !preview.contains("{list}"),
                "placeholder crudo en {locale:?}"
            );
            assert!(!preview.contains("stats."), "fallback en {locale:?}");
            let success = format_series_success_for(2, "A1:A8", locale);
            assert!(
                !success.contains("{count}"),
                "placeholder crudo en {locale:?}"
            );
            assert!(
                !success.contains("{range}"),
                "placeholder crudo en {locale:?}"
            );
            assert!(!success.contains("stats."), "fallback en {locale:?}");
        }
        assert_eq!(
            format_series_preview_text_for(2, &first, Locale::En),
            "2 cells: 1, 2, 3"
        );
        assert_eq!(
            format_series_success_for(2, "A1:A8", Locale::De),
            "Serie: 2 Zelle(n) in A1:A8 gefüllt"
        );
    }

    #[test]
    fn sheet_chip_b_a167_hint_localiza() {
        // B-A167: `_for` construye Es idéntico al `format!` previo del draw
        // (`spreadsheet.rs:109-112`) y localiza el resto con template
        // `{ventana}`; notación `400×400`/`10 000`/`(accent)` intacta
        // (precedente B-A98/B-A114/B-A150).
        use super::sheet_chip_hint_text_for;
        use grafito_ui::i18n::Locale;
        assert_eq!(
            sheet_chip_hint_text_for("C3:H10", Locale::Es),
            [
                "Celda en edición (accent) o esquina visible de la hoja 400×400. ",
                "Ventana C3:H10 · 10 000 celdas vivas."
            ]
            .concat()
        );
        for locale in [
            Locale::Es,
            Locale::En,
            Locale::Pt,
            Locale::It,
            Locale::Fr,
            Locale::De,
        ] {
            let text = sheet_chip_hint_text_for("C3:H10", locale);
            assert!(text.contains("C3:H10"), "ventana cruda en {locale:?}");
            assert!(
                !text.contains("{ventana}"),
                "placeholder crudo en {locale:?}"
            );
            assert!(!text.contains("stats."), "fallback en {locale:?}");
            assert!(text.contains("400×400"), "notación en {locale:?}");
        }
        assert_eq!(
            sheet_chip_hint_text_for("C3:H10", Locale::En),
            [
                "Cell being edited (accent) or visible corner of the 400×400 sheet. ",
                "Window C3:H10 · 10,000 live cells."
            ]
            .concat()
        );
        assert_eq!(
            sheet_chip_hint_text_for("C3:H10", Locale::De),
            "Zelle in Bearbeitung (accent) oder sichtbare Ecke des 400×400-Blatts. Fenster C3:H10 · 10.000 aktive Zellen."
        );
    }

    #[test]
    fn sheet_tables_b_a152_helpers_localizan() {
        // B-A152: `_for` delega Es al canónico (paridad byte-idéntica) y
        // localiza el resto con templates `{title}/{rows}/{rest}/{max}/
        // {hidden}/{error}`; notación `DataTable[…]`/`x²`/`«ejemplo»`/`·`
        // intacta (precedente B-A98/B-A150).
        use super::{
            example_sheet_table_error_text, example_sheet_table_error_text_for,
            example_sheet_table_loaded_text, example_sheet_table_loaded_text_for,
            sheet_hidden_tables_overflow, sheet_hidden_tables_overflow_for,
            sheet_table_rows_overflow, sheet_table_rows_overflow_for, sheet_table_subtitle,
            sheet_table_subtitle_for, sheet_table_title, sheet_table_title_for,
        };
        use grafito_ui::i18n::Locale;
        assert_eq!(sheet_table_title_for("", Locale::Es), sheet_table_title(""));
        assert_eq!(
            sheet_table_title_for("T1", Locale::Es),
            sheet_table_title("T1")
        );
        assert_eq!(sheet_table_title("",), "<sin etiqueta>");
        assert_eq!(
            sheet_table_subtitle_for("ejemplo", 5, Locale::Es),
            sheet_table_subtitle("ejemplo", 5)
        );
        assert_eq!(sheet_table_subtitle("ejemplo", 5), "ejemplo · 5 filas");
        assert_eq!(
            sheet_table_rows_overflow_for(60, 50, Locale::Es),
            sheet_table_rows_overflow(60, 50)
        );
        assert_eq!(
            sheet_hidden_tables_overflow_for(10, Locale::Es),
            sheet_hidden_tables_overflow(10)
        );
        assert_eq!(
            example_sheet_table_loaded_text_for(Locale::Es),
            example_sheet_table_loaded_text()
        );
        assert_eq!(
            example_sheet_table_error_text_for("x", Locale::Es),
            example_sheet_table_error_text("x")
        );
        assert_eq!(sheet_table_rows_overflow(50, 50), None);
        assert_eq!(sheet_hidden_tables_overflow(8), None);
        for locale in [
            Locale::Es,
            Locale::En,
            Locale::Pt,
            Locale::It,
            Locale::Fr,
            Locale::De,
        ] {
            let sub = sheet_table_subtitle_for("T1", 5, locale);
            assert!(!sub.contains("{title}"), "placeholder crudo en {locale:?}");
            assert!(!sub.contains("{rows}"), "placeholder crudo en {locale:?}");
            assert!(!sub.contains("stats."), "fallback en {locale:?}");
            let over = sheet_table_rows_overflow_for(60, 50, locale).expect("overflow");
            assert!(!over.contains("{rest}"), "placeholder crudo en {locale:?}");
            assert!(!over.contains("{max}"), "placeholder crudo en {locale:?}");
            assert!(!over.contains("stats."), "fallback en {locale:?}");
            let hidden = sheet_hidden_tables_overflow_for(10, locale).expect("hidden");
            assert!(
                !hidden.contains("{hidden}"),
                "placeholder crudo en {locale:?}"
            );
            assert!(!hidden.contains("{max}"), "fallback en {locale:?}");
            let err = example_sheet_table_error_text_for("dup", locale);
            assert!(!err.contains("{error}"), "placeholder crudo en {locale:?}");
            assert!(!err.contains("stats."), "fallback en {locale:?}");
        }
        assert_eq!(sheet_table_title_for("", Locale::En), "<untagged>");
        assert_eq!(sheet_table_subtitle_for("T1", 5, Locale::En), "T1 · 5 rows");
        assert_eq!(
            example_sheet_table_loaded_text_for(Locale::De),
            "Tabelle „ejemplo“ geladen"
        );
    }

    #[test]
    fn sheet_quick_b_a153_helpers_localizan() {
        // B-A153: `_for` delega Es al canónico (paridad byte-idéntica) y
        // localiza el resto con template `{label}`; notación
        // `(Enter …)`/`200 inline` intacta (precedente B-A98/B-A150).
        use super::{
            statistics_boxplot_ready_for, statistics_histogram_ready_for,
            statistics_inline_too_many_for, statistics_mean_ready_for,
            statistics_no_inline_hint_for, statistics_no_inline_result_for,
            statistics_quick_ready_text, statistics_quick_ready_text_for, STATISTICS_BOXPLOT_READY,
            STATISTICS_HISTOGRAM_READY, STATISTICS_INLINE_TOO_MANY, STATISTICS_MEAN_READY,
            STATISTICS_NO_INLINE_HINT, STATISTICS_NO_INLINE_RESULT,
        };
        use grafito_ui::i18n::Locale;
        assert_eq!(
            statistics_histogram_ready_for(Locale::Es),
            STATISTICS_HISTOGRAM_READY
        );
        assert_eq!(
            statistics_boxplot_ready_for(Locale::Es),
            STATISTICS_BOXPLOT_READY
        );
        assert_eq!(statistics_mean_ready_for(Locale::Es), STATISTICS_MEAN_READY);
        assert_eq!(
            statistics_inline_too_many_for(Locale::Es),
            STATISTICS_INLINE_TOO_MANY
        );
        assert_eq!(
            statistics_no_inline_result_for(Locale::Es),
            STATISTICS_NO_INLINE_RESULT
        );
        assert_eq!(
            statistics_no_inline_hint_for(Locale::Es),
            STATISTICS_NO_INLINE_HINT
        );
        assert_eq!(
            statistics_quick_ready_text_for("Mediana", Locale::Es),
            statistics_quick_ready_text("Mediana")
        );
        assert_eq!(
            statistics_quick_ready_text("Mediana"),
            "Mediana listo en la entrada (Enter para evaluar)."
        );
        for locale in [
            Locale::Es,
            Locale::En,
            Locale::Pt,
            Locale::It,
            Locale::Fr,
            Locale::De,
        ] {
            let ready = statistics_quick_ready_text_for("Mediana", locale);
            assert!(
                !ready.contains("{label}"),
                "placeholder crudo en {locale:?}"
            );
            assert!(!ready.contains("stats."), "fallback en {locale:?}");
            assert!(!statistics_histogram_ready_for(locale).is_empty());
            assert!(!statistics_inline_too_many_for(locale).is_empty());
        }
        assert_eq!(
            statistics_histogram_ready_for(Locale::En),
            "Histogram ready in the input (Enter to create)."
        );
        assert_eq!(
            statistics_quick_ready_text_for("Mediana", Locale::De),
            "Mediana bereit in der Eingabe (Enter zum Auswerten)."
        );
    }

    #[test]
    fn sheet_summary_b_a154_helpers_localizan() {
        // B-A154: `_for` delega Es al canónico (paridad byte-idéntica) y
        // localiza el resto con templates `{n}/{max}`; notación `—`
        // intacta (precedente B-A98/B-A150/B-A153).
        use super::{
            sheet_summary_prefix_for, sheet_summary_tiles, sheet_summary_tiles_for,
            summarize_range, summarize_range_for,
        };
        use grafito_core::Document;
        use grafito_ui::i18n::Locale;
        assert_eq!(sheet_summary_prefix_for(Locale::Es), "Resumen:");
        assert_eq!(sheet_summary_prefix_for(Locale::En), "Summary:");
        // Paridad Es: tiles idénticos al canónico.
        let summary = super::RangeSummary {
            count: 3,
            mean: 2.0,
            sample_sd: Some(1.0),
            min: 1.0,
            max: 3.0,
        };
        assert_eq!(
            sheet_summary_tiles_for(&summary, |v| format!("{v:.1}"), Locale::Es),
            sheet_summary_tiles(&summary, |v| format!("{v:.1}")),
        );
        // Resto: etiquetas localizadas sin fallback.
        assert_eq!(
            sheet_summary_tiles_for(&summary, |v| format!("{v:.1}"), Locale::En)[1].0,
            "Mean",
        );
        assert_eq!(
            sheet_summary_tiles_for(&summary, |v| format!("{v:.1}"), Locale::De)[2].0,
            "Stichpr.-s",
        );
        // Errores: Es delega al canónico; resto interpola templates.
        let document = Document::default();
        assert_eq!(
            summarize_range(&document, "C1:C4").unwrap_err(),
            "Resumen: el rango no tiene valores numéricos",
        );
        assert_eq!(
            summarize_range_for(&document, "C1:C4", Locale::Es).unwrap_err(),
            "Resumen: el rango no tiene valores numéricos",
        );
        let err_en = summarize_range_for(&document, "C1:C4", Locale::En).unwrap_err();
        assert_eq!(err_en, "Summary: the range has no numeric values");
        let err_de = summarize_range_for(&document, "A1:A401", Locale::De).unwrap_err();
        assert!(err_de.contains("Zusammenfassung:"), "prefijo DE: {err_de}");
        for locale in [
            Locale::Es,
            Locale::En,
            Locale::Pt,
            Locale::It,
            Locale::Fr,
            Locale::De,
        ] {
            let tiles = sheet_summary_tiles_for(&summary, |v| format!("{v:.1}"), locale);
            assert_eq!(tiles.len(), 5);
            assert!(!tiles[1].0.contains("stats."), "fallback en {locale:?}");
        }
    }

    #[test]
    fn sheet_inspector_3d_pinnea_titulos_y_textos() {
        // Paridad con `draw_right_properties_contents` en `panels.rs`
        // (ramas Cube3D/Sphere3D/Point3D/ImplicitSurface3D + fallback del
        // dock): mismos títulos/cadenas/formatos, dueño nuevo (slice 58).
        // El `SidePanel`/`ScrollArea`/sliders/botones/drawer quedan en
        // panels; los botones de vista previa y sus notifies viven en el
        // slice 59 (`sheet_implicit_preview_pinnea_botones_y_avisos`).
        use super::{
            implicit_3d_box_text, implicit_3d_expr_text, implicit_3d_preview_text,
            implicit_3d_state_text, inspector_3d_tag_label, INSPECTOR_CENTER_LABEL,
            INSPECTOR_CUBE3D_TITLE, INSPECTOR_DOCK_EMPTY_GUIDE, INSPECTOR_DOCK_EMPTY_TITLE,
            INSPECTOR_IMPLICIT3D_TITLE, INSPECTOR_POINT3D_TITLE, INSPECTOR_SPHERE3D_TITLE,
        };
        assert_eq!(INSPECTOR_CUBE3D_TITLE, "Cubo 3D");
        assert_eq!(INSPECTOR_SPHERE3D_TITLE, "Esfera 3D");
        assert_eq!(INSPECTOR_POINT3D_TITLE, "Punto 3D");
        assert_eq!(INSPECTOR_CENTER_LABEL, "Centro");
        assert_eq!(INSPECTOR_IMPLICIT3D_TITLE, "Superficie implícita 3D");
        assert_eq!(inspector_3d_tag_label("C"), "Etiqueta: C");
        assert_eq!(
            implicit_3d_expr_text("x^2+y^2+z^2-1"),
            "F(x, y, z) = x^2+y^2+z^2-1"
        );
        assert_eq!(
            implicit_3d_box_text(-1.0, 1.0, -1.0, 1.0, -1.0, 1.0, 24),
            "Caja [-1.00, 1.00] × [-1.00, 1.00] × [-1.00, 1.00] · 24³ celdas"
        );
        assert_eq!(implicit_3d_state_text("lista"), "Estado: lista");
        assert_eq!(
            implicit_3d_preview_text(1500),
            "Vista previa: 1500 triángulos (el canvas la muestra mientras refina)"
        );
        assert_eq!(
            INSPECTOR_DOCK_EMPTY_TITLE,
            "Sin controles dedicados en este dock."
        );
        assert_eq!(
            INSPECTOR_DOCK_EMPTY_GUIDE,
            "Editá este objeto desde el panel de Álgebra."
        );
    }

    #[test]
    fn sheet_implicit_preview_pinnea_botones_y_avisos() {
        // Paridad con `draw_right_properties_contents` en `panels.rs`
        // (rama `ImplicitSurface3D`, botones de vista previa): mismas
        // cadenas que el dibujo, dueño nuevo (slice 59). El `SidePanel`/
        // botones/drawer + el slot en background quedan en panels; la
        // tabla de avisos vive acá como `implicit_quick_preview_notify`.
        use super::{
            implicit_quick_preview_notify, IMPLICIT_PREVIEW_ALREADY_RUNNING,
            IMPLICIT_PREVIEW_ALREADY_SHOWN, IMPLICIT_PREVIEW_CANCELLED,
            IMPLICIT_PREVIEW_CANCEL_EMPTY, IMPLICIT_PREVIEW_CANCEL_HOVER,
            IMPLICIT_PREVIEW_CANCEL_LABEL, IMPLICIT_PREVIEW_RUNNING, IMPLICIT_PREVIEW_START_FAILED,
            IMPLICIT_QUICK_PREVIEW_GUIDE, IMPLICIT_QUICK_PREVIEW_HOVER,
            IMPLICIT_QUICK_PREVIEW_LABEL,
        };
        assert_eq!(
            IMPLICIT_QUICK_PREVIEW_GUIDE,
            "Celdas altas: el fino tarda; la vista previa gruesa (12³) llega primero."
        );
        assert_eq!(IMPLICIT_QUICK_PREVIEW_LABEL, "Vista previa rápida");
        assert_eq!(
            IMPLICIT_QUICK_PREVIEW_HOVER,
            "Genera el grueso 12³ en segundo plano"
        );
        assert_eq!(IMPLICIT_PREVIEW_RUNNING, "Vista previa en curso…");
        assert_eq!(
            IMPLICIT_PREVIEW_ALREADY_SHOWN,
            "Ya hay vista previa mostrándose en el canvas."
        );
        assert_eq!(
            IMPLICIT_PREVIEW_ALREADY_RUNNING,
            "Ya se está generando la vista previa…"
        );
        assert_eq!(
            IMPLICIT_PREVIEW_START_FAILED,
            "No se pudo iniciar la vista previa (revisá la superficie)."
        );
        assert_eq!(IMPLICIT_PREVIEW_CANCELLED, "Vista previa cancelada.");
        assert_eq!(IMPLICIT_PREVIEW_CANCEL_LABEL, "Cancelar");
        assert_eq!(
            IMPLICIT_PREVIEW_CANCEL_EMPTY,
            "No hay vista previa en curso para cancelar."
        );
        assert_eq!(
            IMPLICIT_PREVIEW_CANCEL_HOVER,
            "Detiene la vista previa en curso"
        );
        // Misma tabla que el `if` del dibujo en `panels.rs`.
        assert_eq!(
            implicit_quick_preview_notify(false, false, true),
            IMPLICIT_PREVIEW_RUNNING
        );
        assert_eq!(
            implicit_quick_preview_notify(false, true, false),
            IMPLICIT_PREVIEW_ALREADY_SHOWN
        );
        assert_eq!(
            implicit_quick_preview_notify(true, false, true),
            IMPLICIT_PREVIEW_ALREADY_RUNNING
        );
        assert_eq!(
            implicit_quick_preview_notify(false, false, false),
            IMPLICIT_PREVIEW_START_FAILED
        );
    }

    #[test]
    fn sheet_inspector_4d_nd_pinnea_textos_del_dibujo() {
        // Paridad con `draw_right_properties_contents` en `panels.rs`
        // (ramas `RegularPolychoron4D`/`RegularPolytopeND`): mismos
        // títulos/subtítulos/cuerpos y mismas etiquetas de kind/familia
        // que el dibujo, dueño nuevo (slice 60). El `SidePanel`/
        // combos/sliders/botones/drawer + commit con undo quedan en panels.
        use super::{
            nd_rotation_plane_label, polychoron_kind_label, polytope_nd_family_label,
            polytope_nd_projection_body, POLYCHORON_ANIM_BODY, POLYCHORON_ANIM_TITLE,
            POLYCHORON_APPEARANCE_SUBTITLE, POLYCHORON_APPEARANCE_TITLE,
            POLYCHORON_FILL_OMITTED_HINT, POLYCHORON_GEOMETRY_SUBTITLE, POLYCHORON_GEOMETRY_TITLE,
            POLYCHORON_PROJECTION_SUBTITLE, POLYCHORON_PROJECTION_TITLE,
            POLYCHORON_ROTATION_MANUAL_GUIDE, POLYCHORON_ROTATION_MANUAL_TITLE,
            POLYCHORON_ROTATION_PLANES, POLYCHORON_ROTATION_PLANES_TITLE,
            POLYCHORON_ROTATION_RESET_LABEL, POLYTOPE_ND_ANIM_DIM4_BODY,
            POLYTOPE_ND_ANIM_OTHER_BODY, POLYTOPE_ND_APPEARANCE_SUBTITLE, POLYTOPE_ND_FILL_HINT,
            POLYTOPE_ND_GEOMETRY_SUBTITLE, POLYTOPE_ND_PROJECTION_SUBTITLE,
            POLYTOPE_ND_ROTATION_GUIDE,
        };
        use grafito_geometry::{RegularPolychoron, RegularPolytopeFamily};
        assert_eq!(POLYCHORON_PROJECTION_TITLE, "Proyección");
        assert_eq!(
            POLYCHORON_PROJECTION_SUBTITLE,
            "Controlá la vista dinámica sin alterar la construcción."
        );
        assert_eq!(POLYCHORON_ANIM_TITLE, "Animación de proyección");
        assert_eq!(
            POLYCHORON_ANIM_BODY,
            "La cámara y el politopo 4D giran sin alterar el documento."
        );
        assert_eq!(POLYCHORON_GEOMETRY_TITLE, "Geometría");
        assert_eq!(POLYCHORON_GEOMETRY_SUBTITLE, "Forma y escala");
        assert_eq!(POLYCHORON_APPEARANCE_TITLE, "Apariencia");
        assert_eq!(
            POLYCHORON_APPEARANCE_SUBTITLE,
            "Estilo de aristas y relleno"
        );
        assert_eq!(
            POLYCHORON_FILL_OMITTED_HINT,
            "El relleno se omite en Vista previa y durante el movimiento."
        );
        assert_eq!(POLYCHORON_ROTATION_MANUAL_TITLE, "Rotación manual");
        assert_eq!(
            POLYCHORON_ROTATION_MANUAL_GUIDE,
            "Ajustá los seis planos sólo cuando necesites una vista específica."
        );
        assert_eq!(POLYCHORON_ROTATION_RESET_LABEL, "Restablecer rotaciones");
        assert_eq!(POLYCHORON_ROTATION_PLANES_TITLE, "Planos de rotación");
        assert_eq!(
            POLYCHORON_ROTATION_PLANES,
            ["xy (rad)", "xz (rad)", "xw (rad)", "yz (rad)", "yw (rad)", "zw (rad)"]
        );
        assert_eq!(
            polychoron_kind_label(RegularPolychoron::Pentachoron),
            "Pentácoron (5-celda)"
        );
        assert_eq!(
            polychoron_kind_label(RegularPolychoron::Tesseract),
            "Teseracto"
        );
        assert_eq!(
            polychoron_kind_label(RegularPolychoron::SixteenCell),
            "16-celda"
        );
        assert_eq!(
            polychoron_kind_label(RegularPolychoron::TwentyFourCell),
            "24-celda"
        );
        assert_eq!(
            polychoron_kind_label(RegularPolychoron::OneTwentyCell),
            "120-celda"
        );
        assert_eq!(
            polychoron_kind_label(RegularPolychoron::SixHundredCell),
            "600-celda"
        );
        assert_eq!(POLYTOPE_ND_GEOMETRY_SUBTITLE, "Familia, dimensión y escala");
        assert_eq!(
            polytope_nd_family_label(RegularPolytopeFamily::Simplex),
            "Símplex"
        );
        assert_eq!(
            polytope_nd_family_label(RegularPolytopeFamily::Hypercube),
            "Hipercubo"
        );
        assert_eq!(
            polytope_nd_family_label(RegularPolytopeFamily::CrossPolytope),
            "Politopo cruzado"
        );
        assert_eq!(
            POLYTOPE_ND_PROJECTION_SUBTITLE,
            "La cámara y la proyección 4D comparten velocidad."
        );
        assert_eq!(
            POLYTOPE_ND_ANIM_DIM4_BODY,
            "La cámara y la proyección 4D usan la misma velocidad."
        );
        assert_eq!(
            POLYTOPE_ND_ANIM_OTHER_BODY,
            "La cámara orbita sin alterar el documento."
        );
        assert_eq!(polytope_nd_projection_body(4), POLYTOPE_ND_ANIM_DIM4_BODY);
        assert_eq!(polytope_nd_projection_body(5), POLYTOPE_ND_ANIM_OTHER_BODY);
        assert_eq!(POLYTOPE_ND_APPEARANCE_SUBTITLE, "Estilo de aristas");
        assert!(POLYTOPE_ND_FILL_HINT.contains("solo como aristas"));
        assert_eq!(
            POLYTOPE_ND_ROTATION_GUIDE,
            "Los ajustes avanzados permanecen separados de los controles principales."
        );
        assert_eq!(nd_rotation_plane_label(0, 1), "x1/x2 (rad)");
        assert_eq!(nd_rotation_plane_label(3, 5), "x4/x6 (rad)");
    }

    #[test]
    fn sheet_inspector_notation_sets_son_invariantes_de_locale() {
        // B-A114 (barrido final saga inspector): estos 4 sets son notación
        // SI/matemática idéntica en los 6 idiomas por diseño (precedentes
        // B-A90/B-A98/B-A112: unidades `(rad)`, prefijos de eje `x=`/`y=`/
        // `z=`, fórmula `F(x, y, z)`) y NO tienen key `inspector.*`. Si algún
        // día se les da key, este test obliga a actualizar la decisión.
        use super::{
            implicit_3d_expr_text, nd_rotation_plane_label, INSPECTOR_AXIS_PREFIX_X,
            INSPECTOR_AXIS_PREFIX_Y, INSPECTOR_AXIS_PREFIX_Z, POLYCHORON_ROTATION_PLANES,
        };
        assert_eq!(
            POLYCHORON_ROTATION_PLANES,
            ["xy (rad)", "xz (rad)", "xw (rad)", "yz (rad)", "yw (rad)", "zw (rad)"]
        );
        for plane in POLYCHORON_ROTATION_PLANES {
            assert!(plane.contains("(rad)"), "plano 4D con unidad SI: {plane}");
        }
        assert_eq!(INSPECTOR_AXIS_PREFIX_X, "x=");
        assert_eq!(INSPECTOR_AXIS_PREFIX_Y, "y=");
        assert_eq!(INSPECTOR_AXIS_PREFIX_Z, "z=");
        assert_eq!(
            implicit_3d_expr_text("x^2+y^2+z^2-1"),
            "F(x, y, z) = x^2+y^2+z^2-1"
        );
        assert_eq!(nd_rotation_plane_label(0, 1), "x1/x2 (rad)");
        for needle in ["xy (rad)", "x=", "F(x, y, z)"] {
            assert!(
                !grafito_ui::i18n::INSPECTOR_KEYS.contains(&needle),
                "notación sin key a propósito: {needle}"
            );
        }
    }

    #[test]
    fn sheet_inspector_primitivas_pinnea_prefijos_y_rangos() {
        // Paridad con `draw_right_properties_contents` en `panels.rs`
        // (ramas `Cube3D`/`Sphere3D`/`Point3D` + sliders Escala/Grosor de
        // `RegularPolychoron4D`/`RegularPolytopeND`): mismos prefijos de
        // ejes, velocidad de DragValue, textos y rangos que el dibujo,
        // dueño nuevo (slice 61). Los `SidePanel`/`DragValue`/`Slider`/
        // combos/botones + commit con undo quedan en panels.
        use super::{
            inspector_cube_size_range, inspector_point_size_range, inspector_poly_scale_range,
            inspector_poly_width_range, inspector_sphere_radius_range, INSPECTOR_AXIS_PREFIX_X,
            INSPECTOR_AXIS_PREFIX_Y, INSPECTOR_AXIS_PREFIX_Z, INSPECTOR_CENTER_DRAG_SPEED,
            INSPECTOR_CUBE_SIZE_MAX, INSPECTOR_CUBE_SIZE_MIN, INSPECTOR_EDGE_WIDTH_TEXT,
            INSPECTOR_POINT_SIZE_MAX, INSPECTOR_POINT_SIZE_MIN, INSPECTOR_POLY_SCALE_MAX,
            INSPECTOR_POLY_SCALE_MIN, INSPECTOR_POLY_WIDTH_MAX, INSPECTOR_POLY_WIDTH_MIN,
            INSPECTOR_RADIUS_TEXT, INSPECTOR_SCALE_TEXT, INSPECTOR_SIZE_TEXT,
            INSPECTOR_SPHERE_RADIUS_MAX, INSPECTOR_SPHERE_RADIUS_MIN,
        };
        assert_eq!(INSPECTOR_AXIS_PREFIX_X, "x=");
        assert_eq!(INSPECTOR_AXIS_PREFIX_Y, "y=");
        assert_eq!(INSPECTOR_AXIS_PREFIX_Z, "z=");
        assert_eq!(INSPECTOR_CENTER_DRAG_SPEED, 0.1);
        assert_eq!(INSPECTOR_SIZE_TEXT, "tamaño");
        assert_eq!(INSPECTOR_RADIUS_TEXT, "radio");
        assert_eq!(INSPECTOR_SCALE_TEXT, "Escala");
        assert_eq!(INSPECTOR_EDGE_WIDTH_TEXT, "Grosor de aristas");
        assert_eq!(
            (INSPECTOR_CUBE_SIZE_MIN, INSPECTOR_CUBE_SIZE_MAX),
            (0.1, 10.0)
        );
        assert_eq!(
            (INSPECTOR_SPHERE_RADIUS_MIN, INSPECTOR_SPHERE_RADIUS_MAX),
            (0.1, 10.0)
        );
        assert_eq!(
            (INSPECTOR_POINT_SIZE_MIN, INSPECTOR_POINT_SIZE_MAX),
            (1.0, 20.0)
        );
        assert_eq!(
            (INSPECTOR_POLY_SCALE_MIN, INSPECTOR_POLY_SCALE_MAX),
            (0.01, 10.0)
        );
        assert_eq!(
            (INSPECTOR_POLY_WIDTH_MIN, INSPECTOR_POLY_WIDTH_MAX),
            (0.5, 10.0)
        );
        assert_eq!(
            inspector_cube_size_range(),
            INSPECTOR_CUBE_SIZE_MIN..=INSPECTOR_CUBE_SIZE_MAX
        );
        assert_eq!(
            inspector_sphere_radius_range(),
            INSPECTOR_SPHERE_RADIUS_MIN..=INSPECTOR_SPHERE_RADIUS_MAX
        );
        assert_eq!(
            inspector_point_size_range(),
            INSPECTOR_POINT_SIZE_MIN..=INSPECTOR_POINT_SIZE_MAX
        );
        assert_eq!(
            inspector_poly_scale_range(),
            INSPECTOR_POLY_SCALE_MIN..=INSPECTOR_POLY_SCALE_MAX
        );
        assert_eq!(
            inspector_poly_width_range(),
            INSPECTOR_POLY_WIDTH_MIN..=INSPECTOR_POLY_WIDTH_MAX
        );
    }
}
