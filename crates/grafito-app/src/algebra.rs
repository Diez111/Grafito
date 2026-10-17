//! Algebra side panel: object list, inline property editors, variables,
//! and command input preview.

use crate::{commands, GrafitoApp, ViewMode};
use egui::Color32;
use grafito_core::{GeoObject, ObjectId};
use grafito_ui::i18n::{t, Locale};
use grafito_ui::icons::{action_icon_button, draw_icon, Icon};
use grafito_ui::theme::current_theme;
use grafito_ui::tokens::{
    ICON_SM, ICON_XL, POPUP_ACTION_H, POPUP_MIN_W, RADIUS_SM, SPACE_LG, SPACE_MD, SPACE_SM,
    SPACE_XL, SPACE_XS, TYPE_SM, TYPE_XS, ZOOM_ICON_HIT,
};

pub(crate) const OBJECT_COLOR_TARGET_SIZE: egui::Vec2 =
    egui::Vec2::new(ZOOM_ICON_HIT, ZOOM_ICON_HIT);

/// Ejemplo que siembra el botón del vacío de Álgebra (1 clic → objeto real).
///
/// Es sintaxis válida de la entrada (el mismo camino que tipear + Enter:
/// `submit_input_text` la ejecuta y registra con undo).
pub(crate) const ALGEBRA_EMPTY_EXAMPLE: &str = "y = x^2";

const _ASSERT_HIT_SQUARE_32: () = assert!(ICON_XL == ZOOM_ICON_HIT && ZOOM_ICON_HIT == 32.0);

/// Slider de grosor/tamaño de objeto con el estilo único del panel Álgebra
/// (fill de arrastre + rango explícito). Un solo hogar para los 8 sliders de
/// estilo que antes duplicaban `trailing_fill(true)` con rangos sueltos.
fn width_slider(value: &mut f32, min: f32, max: f32) -> egui::Slider<'_> {
    egui::Slider::new(value, min..=max).trailing_fill(true)
}

pub(crate) fn variable_meta_for_display(
    document: &grafito_core::Document,
    name: &str,
) -> grafito_core::VariableMeta {
    document
        .variable_meta(name)
        .cloned()
        .unwrap_or(grafito_core::VariableMeta {
            position: grafito_geometry::Point2::new(0.0, 0.0),
            min: -5.0,
            max: 5.0,
            step: 0.1,
            visible: true,
            animating: false,
            animation_speed: 1.0,
            animation_mode: grafito_core::AnimationMode::PingPong,
        })
}

pub(crate) fn apply_variable_meta_panel_edit(
    document: &mut grafito_core::Document,
    name: &str,
    candidate: grafito_core::VariableMeta,
    snapshot: &mut crate::app::DeferredPanelSnapshot,
) -> Result<bool, String> {
    let Some(before) = document.try_replace_variable_meta_with_previous(name, candidate)? else {
        return Ok(false);
    };
    snapshot.capture_successful_replacement(before);
    Ok(true)
}

fn color32_from_object_color(color: grafito_geometry::Color, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(
        (color.r * 255.0).clamp(0.0, 255.0) as u8,
        (color.g * 255.0).clamp(0.0, 255.0) as u8,
        (color.b * 255.0).clamp(0.0, 255.0) as u8,
        alpha,
    )
}

fn is_internal_trig_name(name: &str) -> bool {
    name == "TrigGraph" || name == "TrigValue" || name == "trig_angle" || name.starts_with("trig_")
}

/// Etiqueta localizada del kind de policoron regular 4D (pura, sin I/O).
/// Claves `algebra.poly_kind_*`; Es reproduce el canónico previo de
/// `regular_polychoron_summary` ("Pentácoron/Teseracto/16-celda/… 4D",
/// notación `4D` intacta en los 6 idiomas). B-A144.
pub(crate) fn polychoron_kind_label_for(
    kind: grafito_geometry::RegularPolychoron,
    locale: Locale,
) -> &'static str {
    use grafito_geometry::RegularPolychoron as RP;
    match kind {
        RP::Pentachoron => t("algebra.poly_kind_pentachoron", locale),
        RP::Tesseract => t("algebra.poly_kind_tesseract", locale),
        RP::SixteenCell => t("algebra.poly_kind_sixteen", locale),
        RP::TwentyFourCell => t("algebra.poly_kind_twentyfour", locale),
        RP::OneTwentyCell => t("algebra.poly_kind_onetwenty", locale),
        RP::SixHundredCell => t("algebra.poly_kind_sixhundred", locale),
    }
}

/// Etiqueta localizada de la familia de politopo regular N-D (pura, sin I/O).
/// Claves `algebra.poly_family_*`; Es reproduce el canónico previo
/// (las 3 familias del `match` original). B-A144.
pub(crate) fn polytope_family_label_for(
    family: grafito_geometry::RegularPolytopeFamily,
    locale: Locale,
) -> &'static str {
    use grafito_geometry::RegularPolytopeFamily as Fam;
    match family {
        Fam::Simplex => t("algebra.poly_family_simplex", locale),
        Fam::Hypercube => t("algebra.poly_family_hypercube", locale),
        Fam::CrossPolytope => t("algebra.poly_family_cross", locale),
    }
}

/// Etiqueta localizada del kind de ajuste (pura, sin I/O).
/// Claves `algebra.fit_kind_*`; Es reproduce el canónico previo de
/// `FitKind::display_name` (`statistics.rs:54`: lineal/polinómico/…).
/// El mensaje de comando (`commands.rs:856` "Ajuste …") queda canónico ES
/// a propósito: es contenido de comando, no chrome. B-A146.
pub(crate) fn fit_kind_label_for(
    kind: grafito_geometry::statistics::FitKind,
    locale: Locale,
) -> &'static str {
    use grafito_geometry::statistics::FitKind as FK;
    match kind {
        FK::Linear => t("algebra.fit_kind_linear", locale),
        FK::Polynomial { .. } => t("algebra.fit_kind_poly", locale),
        FK::Exponential => t("algebra.fit_kind_exp", locale),
        FK::Logarithmic => t("algebra.fit_kind_log", locale),
        FK::Power => t("algebra.fit_kind_power", locale),
        FK::Sinusoidal => t("algebra.fit_kind_sin", locale),
        FK::Logistic => t("algebra.fit_kind_logistic", locale),
        FK::Growth => t("algebra.fit_kind_growth", locale),
        FK::Implicit => t("algebra.fit_kind_implicit", locale),
    }
}

/// Resumen de policoron 4D localizado (puro, sin I/O).
/// Template `algebra.poly_projected_4d` con `{name}/{scale}`; si el template
/// perdiera algún placeholder, degrada al formato ES canónico (mismo patrón
/// que `algebra_summary_vertices_full_text`). B-A144.
pub(crate) fn polychoron_summary_for(
    polychoron: &grafito_core::RegularPolychoron4DObj,
    locale: Locale,
) -> String {
    let name = polychoron_kind_label_for(polychoron.kind, locale);
    let template = t("algebra.poly_projected_4d", locale);
    if template.contains("{name}") && template.contains("{scale}") {
        template
            .replace("{name}", name)
            .replace("{scale}", &format!("{:.2}", polychoron.scale))
    } else {
        format!(
            "{name} centrado y proyectado · escala={:.2}",
            polychoron.scale
        )
    }
}

/// Resumen de politopo regular N-D localizado (puro, sin I/O).
/// Template `algebra.poly_projected_nd` con `{family}/{dim}/{scale}`;
/// fallback ES canónico si falta algún placeholder. Notación `{}D` intacta
/// en los 6 idiomas. B-A144.
pub(crate) fn polytope_nd_summary_for(
    polytope: &grafito_core::RegularPolytopeNDObj,
    locale: Locale,
) -> String {
    let family = polytope_family_label_for(polytope.family, locale);
    let template = t("algebra.poly_projected_nd", locale);
    if template.contains("{family}") && template.contains("{dim}") && template.contains("{scale}") {
        template
            .replace("{family}", family)
            .replace("{dim}", &polytope.dimension.to_string())
            .replace("{scale}", &format!("{:.2}", polytope.scale))
    } else {
        format!(
            "{family} {}D centrado y proyectado · escala={:.2}",
            polytope.dimension, polytope.scale
        )
    }
}

/// Resumen de cono infinito localizado (puro, sin I/O).
/// Template `algebra.cone_infinite` con `{x}/{y}/{z}/{deg}`; fallback ES
/// canónico. Notación `θ`/`°`/`(clip ±50)` intacta (precedente B-A114). B-A144.
pub(crate) fn infinite_cone_summary_for(
    cone: &grafito_core::InfiniteCone3DObj,
    locale: Locale,
) -> String {
    let template = t("algebra.cone_infinite", locale);
    if template.contains("{x}")
        && template.contains("{y}")
        && template.contains("{z}")
        && template.contains("{deg}")
    {
        template
            .replace("{x}", &format!("{:.1}", cone.apex.x))
            .replace("{y}", &format!("{:.1}", cone.apex.y))
            .replace("{z}", &format!("{:.1}", cone.apex.z))
            .replace("{deg}", &format!("{:.1}", cone.half_angle_rad.to_degrees()))
    } else {
        format!(
            "ápice=({:.1},{:.1},{:.1}) θ={:.1}° (clip ±50)",
            cone.apex.x,
            cone.apex.y,
            cone.apex.z,
            cone.half_angle_rad.to_degrees()
        )
    }
}

/// Resumen de cilindro infinito localizado (puro, sin I/O).
/// Template `algebra.cylinder_infinite` con `{r}/{x}/{y}/{z}`; fallback ES
/// canónico. Notación `r=`/`→`/`(clip ±50)` intacta. B-A144.
pub(crate) fn infinite_cylinder_summary_for(
    cyl: &grafito_core::InfiniteCylinder3DObj,
    locale: Locale,
) -> String {
    let template = t("algebra.cylinder_infinite", locale);
    if template.contains("{r}")
        && template.contains("{x}")
        && template.contains("{y}")
        && template.contains("{z}")
    {
        template
            .replace("{r}", &format!("{:.2}", cyl.radius))
            .replace("{x}", &format!("{:.1}", cyl.direction.x))
            .replace("{y}", &format!("{:.1}", cyl.direction.y))
            .replace("{z}", &format!("{:.1}", cyl.direction.z))
    } else {
        format!(
            "r={:.2} eje→({:.1},{:.1},{:.1}) (clip ±50)",
            cyl.radius, cyl.direction.x, cyl.direction.y, cyl.direction.z
        )
    }
}

/// Resumen de una línea por objeto para la tarjeta del panel Álgebra.
/// Variante con `locale` (B-A143): los conteos salen de los helpers
/// `algebra_summary_*_text`; la notación de medidas (`L=/P=/A=/V=/r=/R=`,
/// `BarChart/PieChart`, `bins`, `=`) queda intacta a propósito.
pub(crate) fn object_expression_summary_for(obj: &GeoObject, locale: Locale) -> String {
    match obj {
        GeoObject::Function(f) => f.fit.as_ref().map_or_else(
            || f.expr.clone(),
            |fit| {
                format!(
                    "{} · {} · RMSE={:.3} · R²={:.3}",
                    f.expr,
                    fit_kind_label_for(fit.kind, locale),
                    fit.diagnostics.rmse,
                    fit.diagnostics.r_squared
                )
            },
        ),
        GeoObject::Point(p) => format!("({:.2}, {:.2})", p.position.x, p.position.y),
        GeoObject::Line(l) => {
            let dx = l.end.x - l.start.x;
            let dy = l.end.y - l.start.y;
            let len = (dx * dx + dy * dy).sqrt();
            format!(
                "({:.2}, {:.2}) <-> ({:.2}, {:.2})  L={:.3}",
                l.start.x, l.start.y, l.end.x, l.end.y, len
            )
        }
        GeoObject::Circle(c) => {
            let area = std::f64::consts::PI * c.radius * c.radius;
            let perim = 2.0 * std::f64::consts::PI * c.radius;
            format!("r={:.2}  A={:.3}  P={:.3}", c.radius, area, perim)
        }
        GeoObject::Ellipse(e) => {
            let area = std::f64::consts::PI * e.rx * e.ry;
            format!("rx={:.2} ry={:.2}  A={:.3}", e.rx, e.ry, area)
        }
        GeoObject::Polygon(p) => {
            let n = p.vertices.len();
            let perim = if n >= 2 {
                let mut sum = 0.0;
                for i in 0..n {
                    let a = p.vertices[i];
                    let b = p.vertices[(i + 1) % n];
                    let dx = b.x - a.x;
                    let dy = b.y - a.y;
                    sum += (dx * dx + dy * dy).sqrt();
                }
                sum
            } else {
                0.0
            };
            let area = if n >= 3 {
                let mut s = 0.0;
                for i in 0..n {
                    let j = (i + 1) % n;
                    s += p.vertices[i].x * p.vertices[j].y - p.vertices[j].x * p.vertices[i].y;
                }
                s.abs() * 0.5
            } else {
                0.0
            };
            algebra_summary_vertices_full_text(n, perim, area, locale)
        }
        GeoObject::Polyline(l) => {
            // Cadena abierta: longitud sin cerrar (sin último→primero).
            let mut len = 0.0;
            for w in l.points.windows(2) {
                let dx = w[1].x - w[0].x;
                let dy = w[1].y - w[0].y;
                len += (dx * dx + dy * dy).sqrt();
            }
            algebra_summary_polyline_text(l.points.len(), len, locale)
        }
        GeoObject::Point3D(p) => format!(
            "({:.2}, {:.2}, {:.2})",
            p.position.x, p.position.y, p.position.z
        ),
        GeoObject::Sphere3D(s) => {
            let area = 4.0 * std::f64::consts::PI * s.radius * s.radius;
            let vol = 4.0 / 3.0 * std::f64::consts::PI * s.radius * s.radius * s.radius;
            format!("r={:.2}  A={:.3}  V={:.3}", s.radius, area, vol)
        }
        GeoObject::Cube3D(c) => {
            let vol = c.size * c.size * c.size;
            format!("size={:.2}  V={:.3}", c.size, vol)
        }
        GeoObject::Tetrahedron3D(t) => {
            let vol = t.edge_length.powi(3) / (6.0 * 2.0_f64.sqrt());
            format!("edge={:.2}  V={:.3}", t.edge_length, vol)
        }
        GeoObject::Platonic3D(p) => {
            format!("{} edge={:.2}", p.kind.as_str(), p.edge_length)
        }
        GeoObject::InfiniteCone3D(c) => infinite_cone_summary_for(c, locale),
        GeoObject::InfiniteCylinder3D(c) => infinite_cylinder_summary_for(c, locale),
        GeoObject::RegularPolychoron4D(polychoron) => polychoron_summary_for(polychoron, locale),
        GeoObject::RegularPolytopeND(polytope) => polytope_nd_summary_for(polytope, locale),
        GeoObject::Cylinder3D(cy) => {
            let dx = cy.top_center.x - cy.base_center.x;
            let dy = cy.top_center.y - cy.base_center.y;
            let dz = cy.top_center.z - cy.base_center.z;
            let h = (dx * dx + dy * dy + dz * dz).sqrt();
            let vol = std::f64::consts::PI * cy.radius * cy.radius * h;
            format!("r={:.2} h={:.2}  V={:.3}", cy.radius, h, vol)
        }
        GeoObject::Cone3D(co) => {
            let dx = co.apex.x - co.base_center.x;
            let dy = co.apex.y - co.base_center.y;
            let dz = co.apex.z - co.base_center.z;
            let h = (dx * dx + dy * dy + dz * dz).sqrt();
            let vol = 1.0 / 3.0 * std::f64::consts::PI * co.radius * co.radius * h;
            format!("r={:.2} h={:.2}  V={:.3}", co.radius, h, vol)
        }
        GeoObject::Pyramid3D(py) => {
            let dx = py.apex.x - py.base_center.x;
            let dy = py.apex.y - py.base_center.y;
            let dz = py.apex.z - py.base_center.z;
            let h = (dx * dx + dy * dy + dz * dz).sqrt();
            let vol = py.base_size * py.base_size * h / 3.0;
            format!("b={:.2} h={:.2}  V={:.3}", py.base_size, h, vol)
        }
        GeoObject::Torus3D(t) => format!("R={:.2} r={:.2}", t.r_major, t.r_minor),
        GeoObject::ImplicitSurface3D(s) => {
            let short: String = s.expr.chars().take(48).collect();
            let ellipsis = if s.expr.chars().count() > 48 {
                "…"
            } else {
                ""
            };
            format!("{short}{ellipsis}=0 [{}³]", s.cells)
        }
        GeoObject::Segment3D(s) => {
            let dx = s.b.x - s.a.x;
            let dy = s.b.y - s.a.y;
            let dz = s.b.z - s.a.z;
            let len = (dx * dx + dy * dy + dz * dz).sqrt();
            format!("L={:.3}", len)
        }
        GeoObject::ParametricCurve2D(p) => format!("({}, {})", p.expr_x, p.expr_y),
        GeoObject::PolarCurve(p) => format!("r = {}", p.expr_r),
        GeoObject::VectorField2D(v) => format!("({}, {})", v.expr_u, v.expr_v),
        GeoObject::ComplexGrid(c) => c.expr.clone(),
        GeoObject::ComplexMapping(c) => c.expr.clone(),
        GeoObject::ImplicitCurve(ic) => {
            let op = match ic.operator {
                grafito_core::RelationOperator::Eq => "=",
                grafito_core::RelationOperator::Less => "<",
                grafito_core::RelationOperator::LessEq => "<=",
                grafito_core::RelationOperator::Greater => ">",
                grafito_core::RelationOperator::GreaterEq => ">=",
            };
            format!("{} {} {}", ic.expr_lhs, op, ic.expr_rhs)
        }
        GeoObject::Histogram(h) => algebra_summary_histogram_text(h.data.len(), h.bins, locale),
        GeoObject::BarChart(b) => algebra_summary_barchart_text(b.data.len(), locale),
        GeoObject::PieChart(p) => {
            let total: f64 = p.data.iter().sum();
            if total.is_finite() {
                format!("PieChart[total={:.3}]", total)
            } else {
                algebra_summary_pie_data_text(p.data.len(), locale)
            }
        }
        GeoObject::ScatterPlot(s) => {
            algebra_summary_points_text(s.xs.len().min(s.ys.len()), locale)
        }
        GeoObject::BoxPlot(b) => algebra_summary_data_text(b.data.len(), locale),
        GeoObject::RegressionLine(r) => format!("y = {:.3}x + {:.3}", r.slope, r.intercept),
        GeoObject::DataTable(table) => algebra_summary_table_text(
            table.xs.len(),
            table.x_name.as_str(),
            table.y_name.as_str(),
            locale,
        ),
        GeoObject::PhasePortrait(p) => format!("({}, {})", p.expr_dx, p.expr_dy),
        _ => String::new(),
    }
}

/// Wrapper ES de [`object_expression_summary_for`] (canónico intacto para
/// callers sin locale y tests previos). Solo tests: prod siempre pasa el
/// `locale` real. B-A143.
#[cfg(test)]
pub(crate) fn object_expression_summary(obj: &GeoObject) -> String {
    object_expression_summary_for(obj, Locale::Es)
}

/// Texto a11y "Eliminar {label}" localizado (puro, sin I/O).
/// Template `algebra.card_delete_a11y` con `{label}`; si el template perdiera
/// el placeholder, degrada al formato ES canónico (mismo patrón que
/// `export_summary_text` en ui). B-A139.
pub(crate) fn algebra_delete_a11y_text(label: &str, locale: Locale) -> String {
    let template = t("algebra.card_delete_a11y", locale);
    if template.contains("{label}") {
        template.replace("{label}", label)
    } else {
        format!("Eliminar {label}")
    }
}

/// Texto a11y "Visibilidad de {label}" localizado (puro, sin I/O).
/// Template `algebra.card_visibility_a11y` con `{label}`; fallback ES
/// canónico si el template perdiera el placeholder. B-A139.
pub(crate) fn algebra_visibility_a11y_text(label: &str, locale: Locale) -> String {
    let template = t("algebra.card_visibility_a11y", locale);
    if template.contains("{label}") {
        template.replace("{label}", label)
    } else {
        format!("Visibilidad de {label}")
    }
}

/// Texto a11y "Color de {label}" localizado (puro, sin I/O).
/// Template `algebra.card_color_a11y` con `{label}`; fallback ES canónico
/// si el template perdiera el placeholder. B-A139.
pub(crate) fn algebra_color_a11y_text(label: &str, locale: Locale) -> String {
    let template = t("algebra.card_color_a11y", locale);
    if template.contains("{label}") {
        template.replace("{label}", label)
    } else {
        format!("Color de {label}")
    }
}

/// Texto a11y del slider de variable localizado (puro, sin I/O).
/// Template `algebra.var_slider_a11y` con `{name}/{v}/{min}/{max}`; si el
/// template perdiera algún placeholder, degrada al formato ES canónico
/// (mismo patrón que `algebra_delete_a11y_text`). B-A141.
pub(crate) fn algebra_var_slider_a11y_text(
    name: &str,
    v: f64,
    min: f64,
    max: f64,
    locale: Locale,
) -> String {
    let template = t("algebra.var_slider_a11y", locale);
    if template.contains("{name}")
        && template.contains("{v}")
        && template.contains("{min}")
        && template.contains("{max}")
    {
        template
            .replace("{name}", name)
            .replace("{v}", &format!("{v}"))
            .replace("{min}", &format!("{min}"))
            .replace("{max}", &format!("{max}"))
    } else {
        format!("{name}, {v} entre {min} y {max}")
    }
}

/// Hover del slider de variable localizado (puro, sin I/O).
/// Template `algebra.var_slider_hover` con `{name}/{v}/{min}/{max}`;
/// fallback ES canónico si falta algún placeholder. B-A141.
pub(crate) fn algebra_var_slider_hover_text(
    name: &str,
    v: f64,
    min: f64,
    max: f64,
    locale: Locale,
) -> String {
    let template = t("algebra.var_slider_hover", locale);
    if template.contains("{name}")
        && template.contains("{v}")
        && template.contains("{min}")
        && template.contains("{max}")
    {
        template
            .replace("{name}", name)
            .replace("{v}", &format!("{v}"))
            .replace("{min}", &format!("{min}"))
            .replace("{max}", &format!("{max}"))
    } else {
        format!("{name}: {v} (rango {min}…{max})")
    }
}

/// Error al aplicar un cambio de variable localizado (puro, sin I/O).
/// Template `algebra.var_set_error` con `{error}`; si el template perdiera
/// el placeholder, degrada al formato ES canónico (mismo patrón que
/// `algebra_delete_a11y_text`). B-A142.
pub(crate) fn algebra_var_set_error_text(error: &str, locale: Locale) -> String {
    let template = t("algebra.var_set_error", locale);
    if template.contains("{error}") {
        template.replace("{error}", error)
    } else {
        format!("Variable: {error}")
    }
}

/// Error al aplicar una edición del panel localizado (puro, sin I/O).
/// Template `ctx.edit_error` con `{error}`; si el template perdiera el
/// placeholder, degrada al formato ES canónico (mismo patrón que
/// `algebra_delete_a11y_text`). B-A145.
pub(crate) fn ctx_edit_error_text(error: &str, locale: Locale) -> String {
    let template = t("ctx.edit_error", locale);
    if template.contains("{error}") {
        template.replace("{error}", error)
    } else {
        format!("Propiedades: {error}")
    }
}

/// Lectura `nombre = valor` de variable localizada (pura, sin I/O).
/// Template `algebra.var_readout` con `{name}/{val}` (`=` intacto en los 6
/// idiomas por notación); fallback ES canónico si falta algún placeholder.
/// B-A142.
pub(crate) fn algebra_var_readout_text(name: &str, val_str: &str, locale: Locale) -> String {
    let template = t("algebra.var_readout", locale);
    if template.contains("{name}") && template.contains("{val}") {
        template.replace("{name}", name).replace("{val}", val_str)
    } else {
        format!("{name} = {val_str}")
    }
}

/// Resúmenes de tarjeta del panel Álgebra localizados (puros, sin I/O).
/// Templates `algebra.summary_*` con `{n}/{perim}/{area}/{len}/{bins}/{x}/{y}`;
/// si el template perdiera algún placeholder, degrada al formato ES canónico
/// (mismo patrón que `algebra_var_readout_text`). Notación intacta en los 6
/// idiomas: `L=/P=/A=` (medidas), `BarChart/PieChart` (identificadores),
/// `bins` (término técnico) y `=`/`·` (precedentes B-A98/B-A114). B-A143.
pub(crate) fn algebra_summary_points_text(n: usize, locale: Locale) -> String {
    let template = t("algebra.summary_points", locale);
    if template.contains("{n}") {
        template.replace("{n}", &n.to_string())
    } else {
        format!("{n} puntos", n = n)
    }
}

pub(crate) fn algebra_summary_vertices_text(n: usize, locale: Locale) -> String {
    let template = t("algebra.summary_vertices", locale);
    if template.contains("{n}") {
        template.replace("{n}", &n.to_string())
    } else {
        format!("{n} vértices", n = n)
    }
}

pub(crate) fn algebra_summary_vertices_full_text(
    n: usize,
    perim: f64,
    area: f64,
    locale: Locale,
) -> String {
    let template = t("algebra.summary_vertices_full", locale);
    if template.contains("{n}") && template.contains("{perim}") && template.contains("{area}") {
        template
            .replace("{n}", &n.to_string())
            .replace("{perim}", &format!("{perim:.3}"))
            .replace("{area}", &format!("{area:.3}"))
    } else {
        format!("{n} vértices  P={perim:.3}  A={area:.3}", n = n)
    }
}

pub(crate) fn algebra_summary_polyline_text(n: usize, len: f64, locale: Locale) -> String {
    let template = t("algebra.summary_polyline", locale);
    if template.contains("{n}") && template.contains("{len}") {
        template
            .replace("{n}", &n.to_string())
            .replace("{len}", &format!("{len:.3}"))
    } else {
        format!("{n} puntos  L={len:.3}", n = n)
    }
}

pub(crate) fn algebra_summary_locus_text(n: usize, locale: Locale) -> String {
    let template = t("algebra.summary_locus", locale);
    if template.contains("{n}") {
        template.replace("{n}", &n.to_string())
    } else {
        format!("Locus: {n} puntos", n = n)
    }
}

pub(crate) fn algebra_summary_data_text(n: usize, locale: Locale) -> String {
    let template = t("algebra.summary_data", locale);
    if template.contains("{n}") {
        template.replace("{n}", &n.to_string())
    } else {
        format!("{n} datos", n = n)
    }
}

pub(crate) fn algebra_summary_histogram_text(n: usize, bins: usize, locale: Locale) -> String {
    let template = t("algebra.summary_histogram", locale);
    if template.contains("{n}") && template.contains("{bins}") {
        template
            .replace("{n}", &n.to_string())
            .replace("{bins}", &bins.to_string())
    } else {
        format!("{n} datos · {bins} bins", n = n, bins = bins)
    }
}

pub(crate) fn algebra_summary_barchart_text(n: usize, locale: Locale) -> String {
    let template = t("algebra.summary_barchart", locale);
    if template.contains("{n}") {
        template.replace("{n}", &n.to_string())
    } else {
        format!("BarChart[{n} datos]", n = n)
    }
}

pub(crate) fn algebra_summary_pie_data_text(n: usize, locale: Locale) -> String {
    let template = t("algebra.summary_pie_data", locale);
    if template.contains("{n}") {
        template.replace("{n}", &n.to_string())
    } else {
        format!("PieChart[{n} datos]", n = n)
    }
}

pub(crate) fn algebra_summary_table_text(n: usize, x: &str, y: &str, locale: Locale) -> String {
    let template = t("algebra.summary_table", locale);
    if template.contains("{n}") && template.contains("{x}") && template.contains("{y}") {
        template
            .replace("{n}", &n.to_string())
            .replace("{x}", x)
            .replace("{y}", y)
    } else {
        format!("{n} pares · {x} / {y}", n = n)
    }
}

pub(crate) fn draw_object_card(ui: &mut egui::Ui, app: &mut GrafitoApp, oid: ObjectId) {
    let theme = current_theme(ui.ctx());
    let alg_locale: Locale = app.config_locale();
    let Some(obj) = app.document.get_object(oid) else {
        return;
    };
    let obj_label = obj.label().to_string();
    let obj_name = obj.name().to_string();
    let obj_vis = obj.is_visible();
    let obj_supports_visibility = !matches!(obj, GeoObject::DataTable(_));
    let obj_col = color32_from_object_color(
        obj.color(),
        (obj.color().a.clamp(0.0, 1.0) * 255.0).round() as u8,
    );
    let obj_expr = object_expression_summary_for(obj, alg_locale);

    let is_sel = app.selected_object == Some(oid);
    let frame_fill = if is_sel {
        theme.accent_muted
    } else {
        theme.button_bg
    };
    let border = if is_sel {
        egui::Stroke::new(1.0, theme.accent)
    } else {
        theme.hairline_stroke()
    };

    let mut row_clicked = false;
    let mut delete = false;
    ui.add_space(SPACE_XS);
    egui::Frame::none()
        .fill(frame_fill)
        .rounding(RADIUS_SM)
        .stroke(border)
        .inner_margin(egui::Margin::symmetric(SPACE_SM, SPACE_SM))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let (del_rect, del_resp) = ui.allocate_exact_size(
                        egui::vec2(ZOOM_ICON_HIT, ZOOM_ICON_HIT),
                        egui::Sense::click(),
                    );
                    del_resp.widget_info(|| {
                        egui::WidgetInfo::labeled(
                            egui::WidgetType::Button,
                            true,
                            algebra_delete_a11y_text(&obj_label, alg_locale),
                        )
                    });
                    if del_resp.has_focus() {
                        theme.paint_focus_ring(ui.painter(), del_rect);
                    }
                    if ui.is_rect_visible(del_rect) {
                        draw_icon(
                            ui.painter(),
                            del_rect.shrink(4.0),
                            Icon::Delete,
                            theme.text_secondary,
                        );
                    }
                    if del_resp
                        .on_hover_text(t("algebra.card_delete", alg_locale))
                        .clicked()
                    {
                        delete = true;
                    }

                    if obj_supports_visibility {
                        let (eye_rect, eye_resp) = ui.allocate_exact_size(
                            egui::vec2(ZOOM_ICON_HIT, ZOOM_ICON_HIT),
                            egui::Sense::click(),
                        );
                        eye_resp.widget_info(|| {
                            egui::WidgetInfo::labeled(
                                egui::WidgetType::Button,
                                true,
                                algebra_visibility_a11y_text(&obj_label, alg_locale),
                            )
                        });
                        if ui.is_rect_visible(eye_rect) {
                            draw_icon(
                                ui.painter(),
                                eye_rect.shrink(4.0),
                                if obj_vis { Icon::Eye } else { Icon::EyeOff },
                                theme.text_secondary,
                            );
                        }
                        if eye_resp
                            .on_hover_text(t("algebra.card_visibility", alg_locale))
                            .clicked()
                        {
                            app.save_state();
                            if let Some(o) = app.document.get_object_mut(oid) {
                                o.set_visible(!obj_vis);
                                app.document.bump_version();
                            }
                        }
                    } else {
                        ui.add_space(ZOOM_ICON_HIT);
                    }

                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        let dot_alpha = if obj_vis {
                            obj_col.a()
                        } else {
                            obj_col.a().min(80)
                        };
                        let dot_col = Color32::from_rgba_unmultiplied(
                            obj_col.r(),
                            obj_col.g(),
                            obj_col.b(),
                            dot_alpha,
                        );
                        let (dot_r, dot_resp) =
                            ui.allocate_exact_size(OBJECT_COLOR_TARGET_SIZE, egui::Sense::click());
                        dot_resp.widget_info(|| {
                            egui::WidgetInfo::labeled(
                                egui::WidgetType::Button,
                                true,
                                algebra_color_a11y_text(&obj_label, alg_locale),
                            )
                        });
                        ui.painter().circle_filled(dot_r.center(), 6.0, dot_col);
                        if dot_resp.hovered() {
                            ui.painter().circle_stroke(
                                dot_r.center(),
                                6.0,
                                egui::Stroke::new(1.0, theme.text_primary),
                            );
                        }
                        if dot_resp
                            .on_hover_text(t("algebra.card_color", alg_locale))
                            .clicked()
                        {
                            app.open_object_color_picker(oid);
                            row_clicked = true;
                        }
                        ui.add_space(SPACE_XS);

                        let txt = if !obj_expr.is_empty() {
                            format!("{}: {}", obj_label, obj_expr)
                        } else {
                            format!("{}: {}", obj_label, obj_name)
                        };
                        // Truncado con tooltip completo + anillo de foco: sin
                        // tooltip el recorte no se recuperaba.
                        let lbl_resp = ui.add(
                            egui::Label::new(
                                egui::RichText::new(txt.clone())
                                    .size(TYPE_SM)
                                    .color(theme.text_primary),
                            )
                            .sense(egui::Sense::click())
                            .truncate(),
                        );
                        if lbl_resp.has_focus() {
                            theme.paint_focus_ring(ui.painter(), lbl_resp.rect);
                        }
                        let lbl_resp = lbl_resp.on_hover_text(txt);
                        if lbl_resp.clicked() {
                            row_clicked = true;
                        }
                        if lbl_resp.double_clicked()
                            && !obj_expr.is_empty()
                            && (obj_name == "Function" || obj_name == "Point")
                        {
                            app.input_text = format!("{}={}", obj_label, obj_expr);
                        }
                    });
                });
            });
        });

    if row_clicked {
        app.selected_object = if is_sel { None } else { Some(oid) };
    }
    if delete {
        app.save_state();
        app.document.remove_object(oid);
        if app.selected_object == Some(oid) {
            app.selected_object = None;
        }
    }
    ui.add_space(SPACE_XS);
}

/// Vault ELIMINADO 2026-10-06 (directiva usuario: ELIMINAR vistas).
/// Sin drawer, sin breadcrumb, sin rail (ui.rs ya sin entradas). El mensaje
/// honesto vive en el asistente (`assistant.rs` navegación) y en el fallback
/// del `sidebar_tab` en `app.rs`. Crates `grafito-vault` intactos (follow-up).
pub(crate) fn draw_algebra_panel(app: &mut GrafitoApp, ctx: &egui::Context) {
    let mut snapshot = crate::app::DeferredPanelSnapshot::new(app.undo_stack.len());
    let theme = current_theme(ctx);
    let alg_locale: Locale = app.config_locale();
    let accent = theme.accent;
    let alg_fill = theme.panel_bg;
    let _sep_col = theme.separator;
    let txt_col = theme.text_primary;
    let _txt_dim = theme.text_tertiary;

    egui::SidePanel::left("algebra_panel").show_separator_line(false)
    .exact_width(crate::panels::panel_left_width(ctx))
    .resizable(false)
    .frame(egui::Frame::none().fill(alg_fill).stroke(egui::Stroke::NONE))
    .show(ctx, |ui| {
        // Sin header: el rail ya dice "Álgebra" y el clic en el tab activo
        // cierra (igual que la X que había acá). El conteo vive en la barra
        // inferior. La entrada abre directo con aire.
        ui.add_space(SPACE_SM);
        // Input row — cuadrado RADIUS_SM 8 (no pill), aire 8, gap 8 entre +/input/botón, sin espacio vacío a la derecha
        // El popup de sugerencias ocupa toda la columna (no solo el campo).
        let input_span = (ui.next_widget_position().x, ui.available_width());
        egui::Frame::none()
            .fill(theme.input_bg)
            .stroke(theme.hairline_stroke())
            .rounding(egui::Rounding::same(RADIUS_SM))
            .inner_margin(egui::Margin::symmetric(SPACE_SM, SPACE_SM))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = SPACE_SM;
                    let (plus_rect, _) =
                        ui.allocate_exact_size(egui::vec2(ICON_SM, ICON_SM), egui::Sense::hover());
                    if ui.is_rect_visible(plus_rect) {
                        draw_icon(ui.painter(), plus_rect, Icon::Plus, accent);
                    }
                    // Input ocupa todo el ancho disponible entre + y botón — sin hueco a la derecha
                    let button_w = SPACE_XL + SPACE_XS;
                    let input_width =
                        (ui.available_width() - button_w - SPACE_SM).max(0.0);
                    let response = crate::ui::draw_command_input(
                        ui,
                        app,
                        "algebra_panel",
                        [input_width, ZOOM_ICON_HIT],
                        // Los ejemplos viven acá (desaparecen al escribir),
                        // no en el empty-state fijo de abajo.
                        t("algebra.input_hint", alg_locale),
                        false,
                        Some(input_span),
                    );
                    if action_icon_button(
                        ui,
                        Icon::Grid,
                        if app.keyboard_visible {
                            accent
                        } else {
                            theme.text_secondary
                        },
                        if app.keyboard_visible {
                            t("algebra.keyboard_hide", alg_locale)
                        } else {
                            t("algebra.keyboard_show", alg_locale)
                        },
                    )
                    .clicked()
                {
                        app.keyboard_visible = !app.keyboard_visible;
                        // Elección explícita: sobrevive a cambios de perspectiva.
                        app.keyboard_visible_explicit = true;
                        if !app.keyboard_visible {
                            app.keyboard_expanded = false;
                        }
                }
                    if response.changed {
                        app.preview_object = commands::parse_preview(&app.input_text);
                    }
                    if app.input_text.is_empty() {
                        app.preview_object = None;
                    }
                    if response.submitted {
                        let time = ui.ctx().input(|i| i.time);
                        app.submit_input_text(time);
                    }
                });
            });
        ui.add_space(SPACE_SM);
        // Separador canónico (la línea manual a ancho completo tocaba los
        // bordes mientras el contenido lleva inset).
        ui.separator();
        ui.add_space(SPACE_SM);

        // ── Object list — compact, 1 line each ──────────────────────
        egui::Frame::none()
            .inner_margin(egui::Margin {
                left: SPACE_SM,
                right: SPACE_SM,
                top: SPACE_SM,
                bottom: SPACE_SM,
            })
            .show(ui, |ui| {
                // Aire derecho 24 para la flotante (ver tokens).
                let outer_w = ui.available_width();
                egui::ScrollArea::vertical().auto_shrink([false;2]).scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible).show(ui, |ui| {
            let inner_w = grafito_ui::tokens::scroll_content_width(outer_w).max(0.0);
            ui.allocate_ui_with_layout(
                egui::vec2(inner_w, ui.available_height().max(1.0)),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
            let mut delete_id: Option<ObjectId> = None;
            let ids: Vec<ObjectId> = app.document.objects_iter().map(|(id,_)| *id).collect();
            let mut drawn_rows = 0usize;
            for oid in ids {
                // `is_sel` antes del bloque: el caché de expresiones y el
                // resumen de polígono dependen de si la fila está expandida.
                let is_sel = app.selected_object == Some(oid);
                let (obj_label, obj_name, obj_vis, obj_col, obj_expr) = {
                    let Some(obj) = app.document.get_object(oid) else { continue; };
                    if is_internal_trig_name(obj.label()) {
                        continue;
                    }

                    // Filtra por el espacio de render centralizado en GeoObject.
                    let is_3d_object = obj.is_3d();
                    let is_3d_view = app.current_view == ViewMode::D3;
                    if is_3d_object != is_3d_view {
                        continue;
                    }

                    // El Pencil libre no es analizable; un Locus usa el mismo
                    // almacenamiento de polilínea pero sí es una construcción.
                    if matches!(obj, grafito_core::GeoObject::Pencil(pencil) if !pencil.is_dynamic_locus()) {
                        continue;
                    }

                    let o_col = obj.color();
                    let col = Color32::from_rgba_unmultiplied(
                        (o_col.r * 255.0).clamp(0.0, 255.0) as u8,
                        (o_col.g * 255.0).clamp(0.0, 255.0) as u8,
                        (o_col.b * 255.0).clamp(0.0, 255.0) as u8,
                        (o_col.a.clamp(0.0, 1.0) * 255.0).round() as u8,
                    );
                    // Caché de la cadena de resumen por (versión, expandida): el
                    // `match` + `format!` por fila por frame es el costo
                    // dominante de la lista a N grande. Con versión quieta
                    // (hover, scroll, panel abierto sin editar) es hit; al
                    // mutar se recomputa una vez. Una entrada por objeto, con
                    // poda si crece sin cota (objetos borrados).
                    let cached_expr = app.algebra_expr_cache.get(&oid).and_then(
                        |(v, s, l, e)| {
                            (*v == app.document.version
                                && *s == is_sel
                                && *l == alg_locale.code())
                            .then(|| e.clone())
                        },
                    );
                    let expr = match cached_expr {
                        Some(e) => e,
                        None => {
                            let computed: String = match obj {
                        grafito_core::GeoObject::Function(f) => f.expr.clone(),
                        grafito_core::GeoObject::Point(p) => format!("({:.2}, {:.2})", p.position.x, p.position.y),
                        grafito_core::GeoObject::Line(l) => {
                            let dx = l.end.x - l.start.x;
                            let dy = l.end.y - l.start.y;
                            let len = (dx * dx + dy * dy).sqrt();
                            format!(
                                "({:.2}, {:.2}) <-> ({:.2}, {:.2})  L={:.3}",
                                l.start.x, l.start.y, l.end.x, l.end.y, len
                            )
                        }
                        grafito_core::GeoObject::Circle(c) => {
                            let area = std::f64::consts::PI * c.radius * c.radius;
                            let perim = 2.0 * std::f64::consts::PI * c.radius;
                            format!(
                                "r={:.2}  A={:.3}  P={:.3}",
                                c.radius, area, perim
                            )
                        }
                        grafito_core::GeoObject::Ellipse(e) => {
                            let area = std::f64::consts::PI * e.rx * e.ry;
                            format!(
                                "rx={:.2} ry={:.2}  A={:.3}",
                                e.rx, e.ry, area
                            )
                        }
                        grafito_core::GeoObject::Polygon(p) => {
                            let n = p.vertices.len();
                            // O(N·V) diferido: en fila colapsada solo el conteo
                            // (50M sqrt/frame en el peor caso); P/A al expandir.
                            if !is_sel {
                                algebra_summary_vertices_text(n, alg_locale)
                            } else {
                                let perim = if n >= 2 {
                                    let mut sum = 0.0;
                                    for i in 0..n {
                                        let a = p.vertices[i];
                                        let b = p.vertices[(i + 1) % n];
                                        let dx = b.x - a.x;
                                        let dy = b.y - a.y;
                                        sum += (dx * dx + dy * dy).sqrt();
                                    }
                                    sum
                                } else {
                                    0.0
                                };
                                let area = if n >= 3 {
                                    let mut s = 0.0;
                                    for i in 0..n {
                                        let j = (i + 1) % n;
                                        s += p.vertices[i].x * p.vertices[j].y
                                            - p.vertices[j].x * p.vertices[i].y;
                                    }
                                    s.abs() * 0.5
                                } else {
                                    0.0
                                };
                                algebra_summary_vertices_full_text(n, perim, area, alg_locale)
                            }
                        }
                        grafito_core::GeoObject::Polyline(l) => {
                            // Cadena abierta: longitud sin cerrar (también O(V),
                            // mismo diferido que el polígono).
                            if !is_sel {
                                algebra_summary_points_text(l.points.len(), alg_locale)
                            } else {
                                let mut len = 0.0;
                                for w in l.points.windows(2) {
                                    let dx = w[1].x - w[0].x;
                                    let dy = w[1].y - w[0].y;
                                    len += (dx * dx + dy * dy).sqrt();
                                }
                                algebra_summary_polyline_text(l.points.len(), len, alg_locale)
                            }
                        }
                        grafito_core::GeoObject::Pencil(p) if p.is_dynamic_locus() => {
                            algebra_summary_locus_text(p.points.len(), alg_locale)
                        }
                        grafito_core::GeoObject::Pencil(p) => {
                            algebra_summary_points_text(p.points.len(), alg_locale)
                        }
                        grafito_core::GeoObject::Point3D(p) => format!("({:.2}, {:.2}, {:.2})", p.position.x, p.position.y, p.position.z),
                        grafito_core::GeoObject::Sphere3D(s) => {
                            let area = 4.0 * std::f64::consts::PI * s.radius * s.radius;
                            let vol = 4.0 / 3.0 * std::f64::consts::PI * s.radius * s.radius * s.radius;
                            format!("r={:.2}  A={:.3}  V={:.3}", s.radius, area, vol)
                        }
                        grafito_core::GeoObject::Cube3D(c) => {
                            let vol = c.size * c.size * c.size;
                            format!("size={:.2}  V={:.3}", c.size, vol)
                        }
                        grafito_core::GeoObject::Tetrahedron3D(t) => {
                            let vol = t.edge_length.powi(3) / (6.0 * 2.0_f64.sqrt());
                            format!("edge={:.2}  V={:.3}", t.edge_length, vol)
                        }
                        grafito_core::GeoObject::Platonic3D(p) => {
                            format!("{} edge={:.2}", p.kind.as_str(), p.edge_length)
                        }
                        grafito_core::GeoObject::InfiniteCone3D(c) => {
                            infinite_cone_summary_for(c, alg_locale)
                        }
                        grafito_core::GeoObject::InfiniteCylinder3D(c) => {
                            infinite_cylinder_summary_for(c, alg_locale)
                        }
                        grafito_core::GeoObject::RegularPolychoron4D(polychoron) => {
                            polychoron_summary_for(polychoron, alg_locale)
                        }
                        grafito_core::GeoObject::RegularPolytopeND(polytope) => {
                            polytope_nd_summary_for(polytope, alg_locale)
                        }
                        grafito_core::GeoObject::Cylinder3D(cy) => {
                            let dx = cy.top_center.x - cy.base_center.x;
                            let dy = cy.top_center.y - cy.base_center.y;
                            let dz = cy.top_center.z - cy.base_center.z;
                            let h = (dx * dx + dy * dy + dz * dz).sqrt();
                            let vol = std::f64::consts::PI * cy.radius * cy.radius * h;
                            format!("r={:.2} h={:.2}  V={:.3}", cy.radius, h, vol)
                        }
                        grafito_core::GeoObject::Cone3D(co) => {
                            let dx = co.apex.x - co.base_center.x;
                            let dy = co.apex.y - co.base_center.y;
                            let dz = co.apex.z - co.base_center.z;
                            let h = (dx * dx + dy * dy + dz * dz).sqrt();
                            let vol = 1.0 / 3.0 * std::f64::consts::PI * co.radius * co.radius * h;
                            format!("r={:.2} h={:.2}  V={:.3}", co.radius, h, vol)
                        }
                        grafito_core::GeoObject::Pyramid3D(py) => {
                            let dx = py.apex.x - py.base_center.x;
                            let dy = py.apex.y - py.base_center.y;
                            let dz = py.apex.z - py.base_center.z;
                            let h = (dx * dx + dy * dy + dz * dz).sqrt();
                            let vol = py.base_size * py.base_size * h / 3.0;
                            format!("b={:.2} h={:.2}  V={:.3}", py.base_size, h, vol)
                        }
                        grafito_core::GeoObject::Torus3D(t) => {
                            format!("R={:.2} r={:.2}", t.r_major, t.r_minor)
                        }
                        grafito_core::GeoObject::Segment3D(s) => {
                            let dx = s.b.x - s.a.x;
                            let dy = s.b.y - s.a.y;
                            let dz = s.b.z - s.a.z;
                            let len = (dx * dx + dy * dy + dz * dz).sqrt();
                            format!("L={:.3}", len)
                        }
                        _ => String::new(),
                            };
                            if app.algebra_expr_cache.len() > 20_000 {
                                app.algebra_expr_cache.clear();
                            }
                            app.algebra_expr_cache.insert(
                                oid,
                                (
                                    app.document.version,
                                    is_sel,
                                    alg_locale.code(),
                                    computed.clone(),
                                ),
                            );
                            computed
                        }
                    };
                    (obj.label().to_string(), obj.name().to_string(), obj.is_visible(), col, expr)
                };
                drawn_rows += 1;

                let frame_fill = if is_sel {
                    theme.accent_muted
                } else {
                    theme.button_bg
                };
                let border = if is_sel {
                    egui::Stroke::new(1.0, theme.accent)
                } else {
                    theme.hairline_stroke()
                };

                let mut row_clicked = false;
                ui.add_space(SPACE_XS);
                egui::Frame::none()
                    .fill(frame_fill)
                    .rounding(RADIUS_SM)
                    .stroke(border)
                    .inner_margin(egui::Margin::symmetric(SPACE_SM, SPACE_SM))
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.horizontal(|ui| {
                            // Right-side controls drawn first
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let (del_rect, del_resp) = ui.allocate_exact_size(
                                    egui::vec2(ZOOM_ICON_HIT, ZOOM_ICON_HIT),
                                    egui::Sense::click(),
                                );
                                del_resp.widget_info(|| {
                                    egui::WidgetInfo::labeled(
                                        egui::WidgetType::Button,
                                        true,
                                        algebra_delete_a11y_text(&obj_label, alg_locale),
                                    )
                                });
                                if del_resp.has_focus() {
                                    theme.paint_focus_ring(ui.painter(), del_rect);
                                }
                                if ui.is_rect_visible(del_rect) {
                                    draw_icon(ui.painter(), del_rect.shrink(4.0), Icon::Delete, theme.text_secondary);
                                }
                                if del_resp.on_hover_text(t("algebra.card_delete", alg_locale)).clicked() {
                                    delete_id = Some(oid);
                                }
                                let (eye_rect, eye_resp) = ui.allocate_exact_size(
                                    egui::vec2(ZOOM_ICON_HIT, ZOOM_ICON_HIT),
                                    egui::Sense::click(),
                                );
                                eye_resp.widget_info(|| {
                                    egui::WidgetInfo::labeled(
                                        egui::WidgetType::Button,
                                        true,
                                        algebra_visibility_a11y_text(&obj_label, alg_locale),
                                    )
                                });
                                if ui.is_rect_visible(eye_rect) {
                                    draw_icon(
                                        ui.painter(),
                                        eye_rect.shrink(4.0),
                                        if obj_vis { Icon::Eye } else { Icon::EyeOff },
                                        theme.text_secondary,
                                    );
                                }
                                if eye_resp.on_hover_text(t("algebra.card_visibility", alg_locale)).clicked() {
                                    snapshot.capture(&app.document);
                                    if let Some(o) = app.document.get_object_mut_touch(oid) {
                                        let v = o.is_visible(); o.set_visible(!v);
                                    }
                                }

                                // Left-side controls in remaining space
                                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                    let dot_alpha = if obj_vis { obj_col.a() } else { obj_col.a().min(80) };
                                    let dot_col = Color32::from_rgba_unmultiplied(
                                        obj_col.r(), obj_col.g(), obj_col.b(), dot_alpha);
                                    let (dot_r, dot_resp) = ui.allocate_exact_size(OBJECT_COLOR_TARGET_SIZE, egui::Sense::click());
                                    dot_resp.widget_info(|| {
                                        egui::WidgetInfo::labeled(
                                            egui::WidgetType::Button,
                                            true,
                                            algebra_color_a11y_text(&obj_label, alg_locale),
                                        )
                                    });
                                    ui.painter().circle_filled(dot_r.center(), 6.0, dot_col);
                                    if dot_resp.hovered() {
                                        ui.painter().circle_stroke(dot_r.center(), 6.0, egui::Stroke::new(1.0, theme.text_primary));
                                    }
                                    let dot_resp = dot_resp.on_hover_text(t("algebra.card_color", alg_locale));
                                    if dot_resp.clicked() {
                                        app.open_object_color_picker(oid);
                                        row_clicked = true;
                                    }
                                    ui.add_space(SPACE_XS);

                                    let txt = if !obj_expr.is_empty() {
                                        format!("{}: {}", obj_label, obj_expr)
                                    } else {
                                        format!("{}: {}", obj_label, obj_name)
                                    };
                                    let lbl_resp = ui.add(egui::Label::new(
                                        egui::RichText::new(txt.clone()).size(TYPE_SM).color(txt_col)).sense(egui::Sense::click()).truncate());
                                    if lbl_resp.has_focus() {
                                        theme.paint_focus_ring(ui.painter(), lbl_resp.rect);
                                    }
                                    // El hint 3D viaja en el mismo tooltip (no como
                                    // label fijo): un solo hover, sin pisar el texto.
                                    let lbl_resp = if is_sel
                                        && app.current_view == ViewMode::D3
                                    {
                                        lbl_resp.on_hover_text(format!(
                                            "{txt}\nAbrí el Inspector para editar este objeto 3D."
                                        ))
                                    } else {
                                        lbl_resp.on_hover_text(txt)
                                    };
                                    if lbl_resp.clicked() { row_clicked = true; }
                                    if lbl_resp.double_clicked() && !obj_expr.is_empty() && (obj_name == "Function" || obj_name == "Point") {
                                        app.input_text = format!("{}={}", obj_label, obj_expr);
                                    }
                                });
                            });
                        });

                        // Properties Panel (Inline)
                        if is_sel && app.current_view != ViewMode::D3 {
                            // Edit a detached copy so idle controls cannot bump the
                            // document revision while an assistant request is pending.
                            if let Some(mut edited) = app.document.get_object(oid).cloned() {
                                ui.add_space(SPACE_XS);
                                // T6: respuesta del slider de estilo para
                                // coalescing por gesto (1 undo por drag).
                                let mut style_resp: Option<egui::Response> = None;
                                ui.scope(|ui| {
                                    // Sin overrides de light mode: confiamos en
                                    // los tokens del theme LIGHT definidos en
                                    // grafito-ui/src/theme.rs.
                                    match &mut edited {
                                        GeoObject::Line(l) => {
                                            style_resp = Some(
                                                ui.horizontal(|ui| {
                                                ui.add_space(SPACE_LG);
                                                ui.label(egui::RichText::new("w").size(TYPE_XS).color(theme.text_tertiary));
                                                    ui.spacing_mut().slider_width = ui.available_width().max(0.0);
                                                    ui.add(width_slider(&mut l.width, 0.5, 10.0))
                                                })
                                                .inner,
                                            );
                                        }
                                        GeoObject::Circle(c) => {
                                            style_resp = Some(
                                                ui.horizontal(|ui| {
                                                ui.add_space(SPACE_LG);
                                                ui.label(egui::RichText::new("w").size(TYPE_XS).color(theme.text_tertiary));
                                                    ui.spacing_mut().slider_width = ui.available_width().max(0.0);
                                                    ui.add(width_slider(&mut c.width, 0.5, 10.0))
                                                })
                                                .inner,
                                            );
                                        }
                                        GeoObject::Function(f) => {
                                            style_resp = Some(
                                                ui.horizontal(|ui| {
                                                ui.add_space(SPACE_LG);
                                                ui.label(egui::RichText::new("w").size(TYPE_XS).color(theme.text_tertiary));
                                                    ui.spacing_mut().slider_width = ui.available_width().max(0.0);
                                                    ui.add(width_slider(&mut f.width, 0.5, 10.0))
                                                })
                                                .inner,
                                            );
                                        }
                                        GeoObject::Point(p) => {
                                            style_resp = Some(
                                                ui.horizontal(|ui| {
                                                ui.add_space(SPACE_LG);
                                                ui.label(egui::RichText::new("pt").size(TYPE_XS).color(theme.text_tertiary));
                                                    ui.spacing_mut().slider_width = ui.available_width().max(0.0);
                                                    ui.add(width_slider(&mut p.size, 1.0, 20.0))
                                                })
                                                .inner,
                                            );
                                        }
                                        GeoObject::Point3D(p) => {
                                            style_resp = Some(
                                                ui.horizontal(|ui| {
                                                ui.add_space(SPACE_LG);
                                                ui.label(egui::RichText::new("pt").size(TYPE_XS).color(theme.text_tertiary));
                                                    ui.spacing_mut().slider_width = ui.available_width().max(0.0);
                                                    ui.add(width_slider(&mut p.size, 1.0, 20.0))
                                                })
                                                .inner,
                                            );
                                        }
                                        GeoObject::Polygon(poly) => {
                                            style_resp = Some(
                                                ui.horizontal(|ui| {
                                                ui.add_space(SPACE_LG);
                                                ui.label(egui::RichText::new("w").size(TYPE_XS).color(theme.text_tertiary));
                                                    ui.spacing_mut().slider_width = ui.available_width().max(0.0);
                                                    ui.add(width_slider(&mut poly.width, 0.5, 10.0))
                                                })
                                                .inner,
                                            );
                                        }
                                        GeoObject::Polyline(line) => {
                                            style_resp = Some(
                                                ui.horizontal(|ui| {
                                                ui.add_space(SPACE_LG);
                                                ui.label(egui::RichText::new("w").size(TYPE_XS).color(theme.text_tertiary));
                                                    ui.spacing_mut().slider_width = ui.available_width().max(0.0);
                                                    ui.add(width_slider(&mut line.width, 0.5, 10.0))
                                                })
                                                .inner,
                                            );
                                        }
                                        GeoObject::Pencil(pencil) => {
                                            style_resp = Some(
                                                ui.horizontal(|ui| {
                                                ui.add_space(SPACE_LG);
                                                ui.label(egui::RichText::new("pen").size(TYPE_XS).color(theme.text_tertiary));
                                                    ui.spacing_mut().slider_width = ui.available_width().max(0.0);
                                                    ui.add(width_slider(&mut pencil.width, 0.5, 20.0))
                                                })
                                                .inner,
                                            );
                                        }
                                        _ => {}
                                    }
                                });
                                // T6: gesto de estilo — captura una vez al
                                // iniciar el arrastre, commitea al soltar.
                                let style_key = egui::Id::new((
                                    crate::app::PANEL_GESTURE_UNDO_KEY,
                                    ("style", oid),
                                ));
                                if style_resp
                                    .as_ref()
                                    .is_some_and(|resp| resp.drag_started() || resp.dragged())
                                {
                                    crate::app::panel_gesture_begin(
                                        ui.ctx(),
                                        style_key,
                                        &app.document,
                                    );
                                }
                                let style_dragged = style_resp
                                    .as_ref()
                                    .is_some_and(|resp| resp.dragged());
                                let changed = app
                                    .document
                                    .get_object(oid)
                                    .is_some_and(|object| object != &edited);
                                match crate::panels::apply_object_panel_edit_with_previous(
                                    &mut app.document,
                                    oid,
                                    changed,
                                    |object| *object = edited,
                                ) {
                                    Ok(Some(before)) => {
                                        if !style_dragged {
                                            snapshot.capture_successful_replacement(before);
                                        }
                                        // else: el gesto commitea al soltar;
                                        // se descarta el `before` del frame.
                                    }
                                    Ok(None) => {}
                                    Err(error) => {
                                        // B-A145: mensaje en el idioma activo
                                        // (antes `format!("Propiedades: …")` ES duro).
                                        let message =
                                            ctx_edit_error_text(&error, alg_locale);
                                        app.cas_result = message.clone();
                                        app.notify(message, grafito_ui::toast::ToastKind::Error);
                                    }
                                }
                                if style_resp
                                    .as_ref()
                                    .is_some_and(|resp| resp.drag_stopped())
                                {
                                    crate::app::panel_gesture_commit(
                                        ui.ctx(),
                                        style_key,
                                        app.document.version,
                                        &mut app.undo_stack,
                                        &mut app.redo_stack,
                                    );
                                }
                            }
                        }

                    });

                if row_clicked {
                    app.selected_object = if is_sel { None } else { Some(oid) };
                }
                ui.add_space(SPACE_XS);
            }
            if let Some(id) = delete_id {
                snapshot.capture(&app.document);
                app.document.remove_object(id);
                if app.selected_object == Some(id) { app.selected_object = None; }
            }

            // Vacío quiet: misma tarjeta que Datos (no lista en blanco).
            if drawn_rows == 0 {
                ui.add_space(SPACE_XS);
                egui::Frame::none()
                    .fill(theme.button_bg)
                    .stroke(theme.hairline_stroke())
                    .rounding(RADIUS_SM)
                    .inner_margin(egui::Margin::same(SPACE_MD))
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new(t("algebra.empty_title", alg_locale))
                                    .color(theme.text_primary)
                                    .size(TYPE_SM)
                                    .strong(),
                            );
                            ui.add_space(SPACE_XS);
                            ui.label(
                                egui::RichText::new(t("algebra.empty_hint", alg_locale))
                                    .color(theme.text_tertiary)
                                    .size(TYPE_XS),
                            );
                            // Empty cálido: el botón siembra el ejemplo (1 clic
                            // → objeto real por el mismo camino que tipear +
                            // Enter: `submit_input_text` ejecuta y registra).
                            ui.add_space(SPACE_SM);
                            if ui
                                .button(
                                    egui::RichText::new(t("algebra.empty_action", alg_locale))
                                        .size(TYPE_SM)
                                        .strong(),
                                )
                                .clicked()
                            {
                                app.input_text = ALGEBRA_EMPTY_EXAMPLE.to_string();
                                let time = ui.ctx().input(|input| input.time);
                                app.submit_input_text(time);
                            }
                        });
                    });
                ui.add_space(SPACE_XS);
            }

            // Variables (sin guía: el vacío de objetos ya enseña la
            // entrada; una línea más era ruido).
            if !app.document.variables.is_empty() {
                ui.add_space(SPACE_SM);

                let vars: Vec<(String, f64)> = app
                    .document
                    .variables
                    .clone()
                    .into_iter()
                    // Ola 0.5: `__view_*` son estado de vista (ShowGrid & cía.),
                    // no variables del usuario.
                    .filter(|(name, _)| {
                        !is_internal_trig_name(name) && !name.starts_with("__")
                    })
                    .collect();
                let mut var_to_delete = None;
                let mut vars_left = vars.len();
                for (name, val) in &vars {
                    let mut v = *val;

                    let metadata = variable_meta_for_display(&app.document, name);
                    let (mut animating, mut min, mut max, step, mut speed) = (
                        metadata.animating,
                        metadata.min,
                        metadata.max,
                        metadata.step,
                        metadata.animation_speed,
                    );

                    egui::Frame::none()
                        .fill(theme.button_bg)
                        .rounding(RADIUS_SM)
                        .stroke(theme.hairline_stroke())
                        .inner_margin(egui::Margin::symmetric(SPACE_SM, SPACE_SM))
                        .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    // Top row: name = value and options
                                    ui.horizontal(|ui| {
                                    let val_str = if v.fract() == 0.0 { format!("{v:.0}") } else { format!("{v:.2}") };
                                    // Nombre truncado con tooltip: un nombre largo
                                    // expulsaba el engranaje/combo/play fuera.
                                    ui.add(
                                        egui::Label::new(
                                            egui::RichText::new(algebra_var_readout_text(
                                                name, &val_str, alg_locale,
                                            ))
                                                .size(TYPE_SM)
                                                .strong()
                                                .color(txt_col),
                                        )
                                        .truncate(),
                                    )
                                    .on_hover_text(algebra_var_readout_text(name, &val_str, alg_locale));

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        let settings_id = ui.make_persistent_id(("variable_settings", name));
                                        let settings_button = action_icon_button(
                                            ui,
                                            Icon::Settings,
                                            theme.text_secondary,
                                            t("algebra.var_range_title", alg_locale),
                                        );
                                        if settings_button.clicked() {
                                            ui.memory_mut(|mem| mem.toggle_popup(settings_id));
                                        }
                                        egui::popup::popup_below_widget(
                                            ui,
                                            settings_id,
                                            &settings_button,
                                            egui::popup::PopupCloseBehavior::CloseOnClickOutside,
                                            |ui| {
                                            // Tres tarjetas a sangre (mín, máx,
                                            // Borrar): mismo alto, mismo radio y
                                            // ancho completo; pila uniforme sin
                                            // etiquetas sueltas ni aire muerto.
                                            let width =
                                                ui.available_width().clamp(0.0, POPUP_MIN_W);
                                            ui.set_min_width(width);
                                            ui.spacing_mut().item_spacing.y = SPACE_XS;
                                            let (min_copy, max_copy) = (min, max);
                                            for (value, limit, is_min, tag_key) in [
                                                (
                                                    &mut min,
                                                    max_copy,
                                                    true,
                                                    "var_min",
                                                ),
                                                (
                                                    &mut max,
                                                    min_copy,
                                                    false,
                                                    "var_max",
                                                ),
                                            ] {
                                                let bound = if limit.is_finite() {
                                                    limit
                                                } else if is_min {
                                                    f64::INFINITY
                                                } else {
                                                    f64::NEG_INFINITY
                                                };
                                                let range = if is_min {
                                                    f64::NEG_INFINITY..=bound
                                                } else {
                                                    bound..=f64::INFINITY
                                                };
                                                ui.label(
                                                    egui::RichText::new(t(
                                                        if tag_key == "var_min" {
                                                            "algebra.var_min"
                                                        } else {
                                                            "algebra.var_max"
                                                        },
                                                        alg_locale,
                                                    ))
                                                        .size(TYPE_XS)
                                                        .color(theme.text_tertiary),
                                                );
                                                ui.add_sized(
                                                    [width, POPUP_ACTION_H],
                                                    egui::DragValue::new(value)
                                                        .speed(0.1)
                                                        .range(range)
                                                        .clamp_existing_to_range(false),
                                                );
                                            }
                                            if !min.is_finite() || !max.is_finite() || min >= max {
                                                ui.label(
                                                    egui::RichText::new(t(
                                                        "algebra.var_range_error",
                                                        alg_locale,
                                                    ))
                                                    .size(TYPE_XS)
                                                    .color(theme.danger),
                                                );
                                            }
                                            // Acción destructiva con el mismo
                                            // molde de tarjeta: tinta danger
                                            // sobre fondo tenue, sin borde.
                                            let borrar = egui::Button::new(
                                                egui::RichText::new(t(
                                                    "algebra.var_delete",
                                                    alg_locale,
                                                ))
                                                    .size(TYPE_SM)
                                                    .strong()
                                                    .color(theme.danger),
                                            )
                                            .fill(theme.danger.gamma_multiply(0.14))
                                            .stroke(egui::Stroke::NONE)
                                            .rounding(RADIUS_SM)
                                            .frame(true);
                                            if ui
                                                .add_sized([width, POPUP_ACTION_H], borrar)
                                                .clicked()
                                            {
                                                var_to_delete = Some(name.clone());
                                                ui.close_menu();
                                            }
                                        },
                                        );

                                        // Selector de velocidad
                                        let speed_abs = speed.abs();
                                        let speed_label = if speed_abs == 1.0 { "1x" } 
                                                          else if speed_abs == 1.5 { "1.5x" }
                                                          else if speed_abs == 2.0 { "2x" }
                                                          else if speed_abs == 0.5 { "0.5x" }
                                                          else { "1x" };

                                        egui::ComboBox::from_id_salt(format!("speed_{}", name))
                                            .selected_text(egui::RichText::new(speed_label).size(TYPE_SM).color(theme.text_secondary))
                                            .width(48.0)
                                            .show_ui(ui, |ui| {
                                                let mut new_speed_abs = speed_abs;
                                                ui.selectable_value(&mut new_speed_abs, 0.5, "0.5x");
                                                ui.selectable_value(&mut new_speed_abs, 1.0, "1x");
                                                ui.selectable_value(&mut new_speed_abs, 1.5, "1.5x");
                                                ui.selectable_value(&mut new_speed_abs, 2.0, "2x");
                                                if new_speed_abs != speed_abs {
                                                    speed = if speed < 0.0 { -new_speed_abs } else { new_speed_abs };
                                                }
                                            });

                                        // Play/Pause button
                                        let tooltip = if animating {
                                            t("algebra.var_anim_stop", alg_locale)
                                        } else {
                                            t("algebra.var_anim_start", alg_locale)
                                        };
                                        if action_icon_button(
                                            ui,
                                            if animating { Icon::Pause } else { Icon::Play },
                                            if animating { theme.accent } else { theme.text_secondary },
                                            tooltip,
                                        )
                                        .clicked()
                                        {
                                            animating = !animating;
                                            if speed == 0.0 { speed = 1.0; } // Ensure it moves when played
                                        }
                                    });
                                });

                                ui.add_space(SPACE_XS);

                                // Bottom row: min, slider, max
                                if min.is_finite() && max.is_finite() && min < max {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            egui::RichText::new(format!("{min}"))
                                                .size(TYPE_SM)
                                                .color(theme.text_tertiary),
                                        );

                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.label(
                                                egui::RichText::new(format!("{max}"))
                                                    .size(TYPE_SM)
                                                    .color(theme.text_tertiary),
                                            );

                                            let mut sl_resp = None;
                                            ui.scope(|ui| {
                                                let visuals = ui.visuals_mut();
                                                visuals.selection.bg_fill = theme.accent;

                                            let mut slider = egui::Slider::new(&mut v, min..=max)
                                                .show_value(false)
                                                .clamping(egui::SliderClamping::Edits)
                                                .trailing_fill(true);

                                            if !animating {
                                                slider = slider.step_by(step);
                                            }

                                            // Sin mínimo que empuje: en card angosta cede
                                            // en vez de pintar fuera. Y `slider_width`
                                            // en spacing: egui ignora el `add_sized`
                                            // para el riel (default 100px).
                                            let slider_width = ui.available_width().max(0.0);
                                            ui.spacing_mut().slider_width = slider_width;
                                            sl_resp = Some(ui.add_sized([slider_width, ui.spacing().interact_size.y], slider));
                                        });

                                            if let Some(sl_resp) = sl_resp {
                                                sl_resp.widget_info(|| {
                                                    egui::WidgetInfo::labeled(
                                                        egui::WidgetType::Slider,
                                                        true,
                                                        algebra_var_slider_a11y_text(
                                                            name, v, min, max, alg_locale,
                                                        ),
                                                    )
                                                });
                                                let _ = sl_resp.clone().on_hover_text(
                                                    algebra_var_slider_hover_text(
                                                        name, v, min, max, alg_locale,
                                                    ),
                                                );
                                                if sl_resp.dragged() && animating {
                                                    animating = false;
                                                }
                                                if sl_resp.changed() && !animating {
                                                    snapshot.capture(&app.document);
                                                    if let Err(error) =
                                                        app.document.try_set_variable(name.clone(), v)
                                                    {
                                                        let message = algebra_var_set_error_text(
                                                            &error.to_string(),
                                                            alg_locale,
                                                        );
                                                        app.cas_result = message.clone();
                                                        app.notify(message, grafito_ui::toast::ToastKind::Error);
                                                    }
                                                }
                                            }
                                        });
                                    });
                                } else {
                                    ui.label(
                                        egui::RichText::new(t(
                                            "algebra.var_range_fix",
                                            alg_locale,
                                        ))
                                        .size(TYPE_XS)
                                        .color(theme.danger),
                                    );
                                }
                            });
                        });

                    let meta_changed = metadata.animating != animating
                        || metadata.min != min
                        || metadata.max != max
                        || metadata.animation_speed != speed;
                    if meta_changed {
                        let mut candidate = metadata;
                        candidate.animating = animating;
                        candidate.min = min;
                        candidate.max = max;
                        candidate.animation_speed = speed;
                        if let Err(error) = apply_variable_meta_panel_edit(
                            &mut app.document,
                            name,
                            candidate,
                            &mut snapshot,
                        ) {
                            let message =
                                algebra_var_set_error_text(&error.to_string(), alg_locale);
                            app.cas_result = message.clone();
                            app.notify(message, grafito_ui::toast::ToastKind::Error);
                        }
                    }

                    // Separador entre tarjetas, nunca colgando al final.
                    if vars_left > 1 {
                        ui.add_space(SPACE_XS);
                        ui.separator();
                    }
                    vars_left -= 1;
                }
                if let Some(to_del) = var_to_delete {
                    snapshot.capture(&app.document);
                    app.document.remove_variable(&to_del);
                }
            }
                },
                );
                });
        });
    });
    let _ = snapshot.save_if_semantically_changed(
        &mut app.document,
        &mut app.undo_stack,
        &mut app.redo_stack,
    );
}

#[cfg(test)]
mod algebra_b_a138_tests {
    #[test]
    fn algebra_drawer_vacio_y_hovers_salen_de_algebra_msg() {
        // B-A138: drawer + vacío + hovers tarjeta del panel Álgebra salen de
        // `t("algebra.*", locale)` con `app.config_locale()`. Sin egui: pines +
        // blindaje por `include_str!`.
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(
            t("algebra.drawer_no_folder", Locale::Es),
            ["sin carp", "eta"].concat()
        );
        assert_eq!(
            t("algebra.drawer_change", Locale::Es),
            ["Cambiar", "..."].concat()
        );
        // Concat a propósito: el blindaje usa `include_str!` (incluye este
        // test) y el literal completo se auto-matchearía (precedente B-A105).
        assert_eq!(
            t("algebra.drawer_change_hint", Locale::Es),
            ["Elegí la carpeta del vault ", "en tu PC"].concat()
        );
        assert_eq!(
            t("algebra.drawer_new_note", Locale::Es),
            ["Nueva ", "nota"].concat()
        );
        assert_eq!(
            t("algebra.drawer_new_folder", Locale::Es),
            ["Nueva carp", "eta"].concat()
        );
        assert_eq!(
            t("algebra.empty_title", Locale::Es),
            ["Sin objetos tod", "avía"].concat()
        );
        assert_eq!(
            t("algebra.empty_hint", Locale::Es),
            ["Empezá tranqui: escribí ", "y=x² en la entrada."].concat()
        );
        assert_eq!(
            t("algebra.empty_action", Locale::Es),
            ["Graficá ", "y=x²"].concat()
        );
        assert_eq!(
            t("algebra.card_delete", Locale::Es),
            ["Elim", "inar"].concat()
        );
        assert_eq!(
            t("algebra.card_visibility", Locale::Es),
            ["Visibil", "idad"].concat()
        );
        assert_eq!(
            t("algebra.card_color", Locale::Es),
            ["Cambiar col", "or"].concat()
        );
        assert_eq!(t("algebra.drawer_no_folder", Locale::En), "no folder");
        assert_eq!(
            t("algebra.drawer_change_hint", Locale::Pt),
            "Escolha a pasta do cofre no seu PC"
        );
        assert_eq!(
            t("algebra.empty_title", Locale::It),
            "Nessun oggetto ancora"
        );
        assert_eq!(
            t("algebra.empty_hint", Locale::Fr),
            "Commence tranquille : écris y=x² dans la saisie."
        );
        assert_eq!(t("algebra.card_color", Locale::De), "Farbe ändern");
        // Wiring: Vault ELIMINADO 2026-10-06 — drawer vault fuera, solo panel Álgebra.
        // Los 5 keys drawer_* ya no se usan en draws (solo asserts de copy, no wiring).
        let source = include_str!("algebra.rs");
        assert!(
            source.contains("Vault ELIMINADO 2026-10-06"),
            "falta pin de eliminación Vault en algebra"
        );
        assert!(
            !source.contains("t(\"algebra.drawer_no_folder\", drawer_locale)"),
            "drawer vault sigue vivo tras eliminar vistas"
        );
        for pin in [
            "let alg_locale: Locale = app.config_locale()",
            "t(\"algebra.empty_title\", alg_locale)",
            "t(\"algebra.empty_hint\", alg_locale)",
            "t(\"algebra.empty_action\", alg_locale)",
            "ALGEBRA_EMPTY_EXAMPLE",
            "t(\"algebra.card_delete\", alg_locale)",
            "t(\"algebra.card_visibility\", alg_locale)",
            "t(\"algebra.card_color\", alg_locale)",
        ] {
            assert!(source.contains(pin), "álgebra B-A138 cablea {pin}");
        }
        // Blindaje: los literales crudos ya no están en el draw.
        // (Fragmentos con `concat!`; splits distintos de los pines.)
        // Empty cálido: el botón tampoco deja ES duro (usa la key i18n).
        for needle in [
            ["si", "n carpeta"].concat(),
            ["Cambi", "ar..."].concat(),
            ["Elegí la carpeta", " del vault en tu PC"].concat(),
            ["Nue", "va nota"].concat(),
            ["Nueva ", "carpeta"].concat(),
            ["Sin obj", "etos todavía"].concat(),
            ["Escribí en la entrada", " para crear el primero."].concat(),
            ["on_hover_text(\"Elim", "inar\")"].concat(),
            ["on_hover_text(\"Visibil", "idad\")"].concat(),
            ["on_hover_text(\"Cambiar col", "or\")"].concat(),
            ["Graficá ", "y=x²"].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "álgebra sin ES duro en prod (B-A138): {needle}"
            );
        }
    }

    #[test]
    fn algebra_empty_action_siembra_ejemplo_valido() {
        // Empty cálido: el botón siembra `ALGEBRA_EMPTY_EXAMPLE` por el
        // mismo camino que tipear + Enter (`submit_input_text`): crea el
        // objeto y limpia la entrada. Sin egui (lógica pura cabezal).
        use super::ALGEBRA_EMPTY_EXAMPLE;
        let mut app = crate::app::dummy_grafito_app();
        assert_eq!(app.document.object_count(), 0);
        app.input_text = ALGEBRA_EMPTY_EXAMPLE.to_string();
        app.submit_input_text(0.0);
        assert_eq!(
            app.document.object_count(),
            1,
            "el ejemplo del vacío crea el objeto"
        );
        assert!(app.input_text.is_empty(), "la entrada se limpia al crear");
    }
}

#[cfg(test)]
mod algebra_b_a139_tests {
    #[test]
    fn algebra_entrada_teclado_anim_y_a11y_salen_de_algebra_msg() {
        // B-A139: entrada + teclado + anim variable + a11y con template {label}
        // salen de `t("algebra.*", locale)` con `app.config_locale()`. Sin egui:
        // pines + blindaje por `include_str!` (precedente B-A138).
        use super::{
            algebra_color_a11y_text, algebra_delete_a11y_text, algebra_visibility_a11y_text,
        };
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(
            t("algebra.input_hint", Locale::Es),
            ["Escribí ", "algo…"].concat()
        );
        assert_eq!(
            t("algebra.keyboard_hide", Locale::Es),
            ["Ocultar tecl", "ado"].concat()
        );
        assert_eq!(
            t("algebra.keyboard_show", Locale::Es),
            ["Mostrar tecl", "ado"].concat()
        );
        assert_eq!(
            t("algebra.var_anim_stop", Locale::Es),
            ["Detener anim", "ación"].concat()
        );
        assert_eq!(
            t("algebra.var_anim_start", Locale::Es),
            ["Animar vari", "able"].concat()
        );
        assert_eq!(
            t("algebra.card_delete_a11y", Locale::Es),
            ["Eliminar ", "{label}"].concat()
        );
        assert_eq!(
            t("algebra.card_visibility_a11y", Locale::Es),
            ["Visibilidad de ", "{label}"].concat()
        );
        assert_eq!(
            t("algebra.card_color_a11y", Locale::Es),
            ["Color de ", "{label}"].concat()
        );
        assert_eq!(t("algebra.input_hint", Locale::En), "Type something…");
        assert_eq!(
            t("algebra.keyboard_show", Locale::Pt),
            ["Mostrar ", "teclado"].concat()
        );
        assert_eq!(t("algebra.var_anim_stop", Locale::It), "Ferma animazione");
        assert_eq!(
            t("algebra.var_anim_start", Locale::Fr),
            "Animer la variable"
        );
        assert_eq!(t("algebra.card_delete_a11y", Locale::De), "Löschen {label}");
        // Helpers puros: Es delega al canónico por construcción.
        assert_eq!(
            algebra_delete_a11y_text("M", Locale::Es),
            ["Eliminar ", "M"].concat()
        );
        assert_eq!(algebra_delete_a11y_text("M", Locale::En), "Delete M");
        assert_eq!(
            algebra_visibility_a11y_text("M", Locale::Es),
            ["Visibilidad de ", "M"].concat()
        );
        assert_eq!(algebra_color_a11y_text("M", Locale::De), "Farbe von M");
        // Wiring: entrada + toggles + tooltip anim + 6 widget_info por helpers.
        let source = include_str!("algebra.rs");
        for pin in [
            "t(\"algebra.input_hint\", alg_locale)",
            "t(\"algebra.keyboard_hide\", alg_locale)",
            "t(\"algebra.keyboard_show\", alg_locale)",
            "t(\"algebra.var_anim_stop\", alg_locale)",
            "t(\"algebra.var_anim_start\", alg_locale)",
            "algebra_delete_a11y_text(&obj_label, alg_locale)",
            "algebra_visibility_a11y_text(&obj_label, alg_locale)",
            "algebra_color_a11y_text(&obj_label, alg_locale)",
        ] {
            assert!(source.contains(pin), "álgebra B-A139 cablea {pin}");
        }
        // Blindaje: los 8 literales crudos ya no están en el draw (los
        // fallbacks `Eliminar {label}` viven solo en los helpers puros, con
        // `{label}` — acá se buscan los viejos `{obj_label}` + splits
        // distintos de los pines para evitar auto-match por `include_str!`).
        for needle in [
            ["Escribí a", "lgo…"].concat(),
            ["Ocultar tecl", "ado\""].concat(),
            ["Mostrar tecl", "ado\""].concat(),
            ["Detener anim", "ación\""].concat(),
            ["Animar vari", "able\""].concat(),
            ["Eliminar {obj_", "label}"].concat(),
            ["Visibilidad de {obj_", "label}"].concat(),
            ["Color de {obj_", "label}"].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "álgebra sin ES duro en prod (B-A139): {needle}"
            );
        }
    }
}

#[cfg(test)]
mod algebra_b_a141_tests {
    #[test]
    fn algebra_sliders_rango_salen_de_algebra_msg() {
        // B-A141: sliders variables + popup rango salen de `t("algebra.var_*")`
        // y helpers puros con templates {name}/{v}/{min}/{max}. Sin egui:
        // pines + blindaje por `include_str!` (precedente B-A138/B-A139).
        use super::{algebra_var_slider_a11y_text, algebra_var_slider_hover_text};
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(
            t("algebra.var_range_title", Locale::Es),
            ["Configurar rango ", "de la variable"].concat()
        );
        assert_eq!(t("algebra.var_min", Locale::Es), ["m", "ín"].concat());
        assert_eq!(t("algebra.var_max", Locale::Es), ["m", "áx"].concat());
        assert_eq!(
            t("algebra.var_range_error", Locale::Es),
            ["Mínimo < máximo, ", "ambos finitos."].concat()
        );
        assert_eq!(t("algebra.var_delete", Locale::Es), ["Bor", "rar"].concat());
        assert_eq!(
            t("algebra.var_slider_a11y", Locale::Es),
            ["{name}, {v} entre ", "{min} y {max}"].concat()
        );
        assert_eq!(
            t("algebra.var_slider_hover", Locale::Es),
            ["{name}: {v} (rango ", "{min}…{max})"].concat()
        );
        assert_eq!(
            t("algebra.var_range_fix", Locale::Es),
            ["Corrige el rango ", "antes de editar el valor."].concat()
        );
        assert_eq!(
            t("algebra.var_range_title", Locale::En),
            "Configure variable range"
        );
        assert_eq!(
            t("algebra.var_slider_hover", Locale::Pt),
            "{name}: {v} (intervalo {min}…{max})"
        );
        assert_eq!(
            t("algebra.var_range_fix", Locale::De),
            "Korrigiere den Bereich, bevor du den Wert bearbeitest."
        );
        // Helpers puros: Es delega al canónico por construcción.
        assert_eq!(
            algebra_var_slider_a11y_text("p", 1.0, -5.0, 5.0, Locale::Es),
            ["p, 1 entre -5 ", "y 5"].concat()
        );
        assert_eq!(
            algebra_var_slider_a11y_text("p", 1.0, -5.0, 5.0, Locale::En),
            "p, 1 between -5 and 5"
        );
        assert_eq!(
            algebra_var_slider_hover_text("p", 1.0, -5.0, 5.0, Locale::Es),
            ["p: 1 (rango -5…", "5)"].concat()
        );
        assert_eq!(
            algebra_var_slider_hover_text("p", 1.0, -5.0, 5.0, Locale::De),
            "p: 1 (Bereich -5…5)"
        );
        // Wiring: popup + slider usan las 8 keys con su locale.
        let source = include_str!("algebra.rs");
        for pin in [
            "t(\"algebra.var_range_title\", alg_locale)",
            "\"algebra.var_min\"",
            "\"algebra.var_max\"",
            "t(\"algebra.var_range_error\",",
            "t(\"algebra.var_delete\",",
            "algebra_var_slider_a11y_text(",
            "algebra_var_slider_hover_text(",
            "t(\"algebra.var_range_fix\",",
        ] {
            assert!(source.contains(pin), "álgebra B-A141 cablea {pin}");
        }
        // Blindaje: los 6 literales no-template ya no están en el draw. Los 2
        // templates del slider (`{name}, {v} entre…` / `{name}: {v} (rango…)`)
        // viven como fallback ES en los helpers puros (mismo patrón B-A139:
        // allá el blindaje buscaba `{obj_label}` viejo vs `{label}` nuevo;
        // acá placeholders viejos y nuevos son idénticos, así que el draw se
        // pinnea por wiring + salida de helpers, no por ausencia de substring).
        for needle in [
            ["Configurar rango", " de la variable\""].concat(),
            ["\"m", "ín\""].concat(),
            ["\"m", "áx\""].concat(),
            ["Mínimo < máx", "imo, ambos finitos.\""].concat(),
            ["RichText::new(\"Bo", "rrar\")"].concat(),
            ["Corrige el ra", "ngo antes de editar el valor.\""].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "álgebra sin ES duro en prod (B-A141): {needle}"
            );
        }
    }
}

#[cfg(test)]
mod algebra_b_a142_tests {
    #[test]
    fn algebra_error_y_readout_salen_de_algebra_msg() {
        // B-A142: error al aplicar variable + readout `nombre = valor`
        // salen de `t("algebra.var_*")` y helpers puros con templates
        // {error}/{name}/{val}. Sin egui: pines + blindaje por
        // `include_str!` (precedente B-A138/B-A139/B-A141). La velocidad
        // "1x"/"0.5x"/"1.5x"/"2x" es notación invariante (precedente B-A114).
        use super::{algebra_var_readout_text, algebra_var_set_error_text};
        use grafito_ui::i18n::{t, Locale};
        assert_eq!(
            t("algebra.var_set_error", Locale::Es),
            ["Variable: ", "{error}"].concat()
        );
        assert_eq!(
            t("algebra.var_readout", Locale::Es),
            ["{name} = ", "{val}"].concat()
        );
        assert_eq!(t("algebra.var_set_error", Locale::En), "Variable: {error}");
        assert_eq!(
            t("algebra.var_set_error", Locale::Pt),
            ["Variável: ", "{error}"].concat()
        );
        assert_eq!(
            t("algebra.var_readout", Locale::It),
            ["{name} = ", "{val}"].concat()
        );
        assert_eq!(
            t("algebra.var_set_error", Locale::Fr),
            ["Variable : ", "{error}"].concat()
        );
        assert_eq!(
            t("algebra.var_readout", Locale::De),
            ["{name} = ", "{val}"].concat()
        );
        // Helpers puros: Es delega al canónico por construcción.
        assert_eq!(
            algebra_var_set_error_text("rango inválido", Locale::Es),
            ["Variable: ", "rango inválido"].concat()
        );
        assert_eq!(
            algebra_var_set_error_text("bad range", Locale::En),
            "Variable: bad range"
        );
        assert_eq!(
            algebra_var_set_error_text("x", Locale::Fr),
            ["Variable : ", "x"].concat()
        );
        assert_eq!(
            algebra_var_readout_text("p", "1", Locale::Es),
            ["p = ", "1"].concat()
        );
        assert_eq!(algebra_var_readout_text("p", "1", Locale::De), "p = 1");
        // Wiring: readout + errores usan helpers/keys con su locale.
        let source = include_str!("algebra.rs");
        for pin in [
            "algebra_var_readout_text(",
            "algebra_var_set_error_text(",
            "&error.to_string(),",
            "t(\"algebra.var_set_error\",",
            "t(\"algebra.var_readout\",",
        ] {
            assert!(source.contains(pin), "álgebra B-A142 cablea {pin}");
        }
        // Blindaje: los literales crudos del draw migraron a los helpers
        // puros (mismo patrón B-A141: allá el blindaje buscaba
        // `{obj_label}` viejo vs `{label}` nuevo; acá placeholders viejos
        // y nuevos son idénticos, así que se pinnea que cada `format!`
        // queda exactamente 1 vez —el fallback del helper— y el draw
        // sale por wiring + salida de helpers).
        assert_eq!(
            source.matches("format!(\"Variable: {error}\")").count(),
            1,
            "el único Variable-format es el fallback del helper (B-A142)"
        );
        assert_eq!(
            source.matches("format!(\"{name} = {val_str}\")").count(),
            1,
            "el único readout-format es el fallback del helper (B-A142)"
        );
        // Notación de velocidad invariante: sin key, sin locale.
        for speed in ["\"1x\"", "\"0.5x\"", "\"1.5x\"", "\"2x\""] {
            assert!(
                source.contains(speed),
                "velocidad invariante presente (B-A142): {speed}"
            );
        }
    }
}

#[cfg(test)]
mod algebra_b_a143_tests {
    #[test]
    fn algebra_resumenes_con_notacion_salen_de_helpers() {
        // B-A143: resúmenes "{n} puntos/vértices/datos/pares" + "Locus" con
        // notación `L=/P=/A=`, `BarChart/PieChart`, `bins` intacta salen de
        // `algebra_summary_*_text` y `object_expression_summary_for(obj,
        // locale)`. Sin egui: pines + blindaje por `include_str!`
        // (precedente B-A138/B-A141/B-A142).
        use super::{
            algebra_summary_barchart_text, algebra_summary_data_text,
            algebra_summary_histogram_text, algebra_summary_locus_text,
            algebra_summary_pie_data_text, algebra_summary_points_text,
            algebra_summary_polyline_text, algebra_summary_table_text,
            algebra_summary_vertices_full_text, algebra_summary_vertices_text,
            object_expression_summary_for,
        };
        use grafito_core::{GeoObject, PolylineObj};
        use grafito_geometry::Point2;
        use grafito_ui::i18n::{t, Locale};
        // Helpers puros: Es reproduce el canónico previo.
        assert_eq!(
            algebra_summary_points_text(3, Locale::Es),
            ["3 pun", "tos"].concat()
        );
        assert_eq!(
            algebra_summary_vertices_text(4, Locale::Es),
            ["4 vér", "tices"].concat()
        );
        assert_eq!(
            algebra_summary_vertices_full_text(4, 12.0, 6.0, Locale::Es),
            ["4 vértices  P=12.0", "00  A=6.000"].concat()
        );
        assert_eq!(
            algebra_summary_polyline_text(5, 2.5, Locale::Es),
            ["5 puntos  L=2.5", "00"].concat()
        );
        assert_eq!(
            algebra_summary_locus_text(7, Locale::Es),
            ["Locus: 7 pun", "tos"].concat()
        );
        assert_eq!(
            algebra_summary_data_text(9, Locale::Es),
            ["9 da", "tos"].concat()
        );
        assert_eq!(
            algebra_summary_histogram_text(9, 10, Locale::Es),
            ["9 datos · 10 b", "ins"].concat()
        );
        assert_eq!(
            algebra_summary_barchart_text(9, Locale::Es),
            ["BarChart[9 da", "tos]"].concat()
        );
        assert_eq!(
            algebra_summary_pie_data_text(9, Locale::Es),
            ["PieChart[9 da", "tos]"].concat()
        );
        assert_eq!(
            algebra_summary_table_text(8, "x", "y", Locale::Es),
            ["8 pares · x ", "/ y"].concat()
        );
        // Resto de idiomas: notación intacta, sustantivo localizado.
        assert_eq!(algebra_summary_points_text(3, Locale::En), "3 points");
        assert_eq!(algebra_summary_vertices_text(4, Locale::It), "4 vertici");
        assert_eq!(algebra_summary_locus_text(7, Locale::De), "Locus: 7 Punkte");
        assert_eq!(algebra_summary_data_text(9, Locale::Pt), "9 dados");
        assert_eq!(
            algebra_summary_histogram_text(9, 10, Locale::En),
            "9 data · 10 bins"
        );
        assert_eq!(
            algebra_summary_table_text(8, "x", "y", Locale::Fr),
            "8 paires · x / y"
        );
        // `_for` sobre objeto real: 3-4-5 → L=5.000, Es y En.
        let line = GeoObject::Polyline(PolylineObj::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(3.0, 4.0),
        ]));
        assert_eq!(
            object_expression_summary_for(&line, Locale::Es),
            ["2 puntos  L=5.0", "00"].concat()
        );
        assert_eq!(
            object_expression_summary_for(&line, Locale::En),
            "2 points  L=5.000"
        );
        assert_eq!(
            t("algebra.summary_locus", Locale::Es),
            ["Locus: {n} pun", "tos"].concat()
        );
        // Wiring: draws + `_for` + caché con locale.
        let source = include_str!("algebra.rs");
        for pin in [
            "object_expression_summary_for(",
            "object_expression_summary_for(obj, alg_locale)",
            "algebra_summary_points_text(",
            "algebra_summary_vertices_text(",
            "algebra_summary_vertices_full_text(n, perim, area, alg_locale)",
            "algebra_summary_polyline_text(l.points.len(), len, alg_locale)",
            "algebra_summary_locus_text(p.points.len(), alg_locale)",
            "algebra_summary_histogram_text(h.data.len(), h.bins, locale)",
            "algebra_summary_barchart_text(b.data.len(), locale)",
            "algebra_summary_pie_data_text(p.data.len(), locale)",
            "algebra_summary_table_text(",
            "t(\"algebra.summary_points\",",
            "t(\"algebra.summary_table\",",
            "*l == alg_locale.code()",
            "alg_locale.code(),",
        ] {
            assert!(source.contains(pin), "álgebra B-A143 cablea {pin}");
        }
        // Blindaje: los 8 estilos viejos `{}` ya no están en prod. Los 2
        // fallbacks `{n}` (vértices colapsado/full) conservan la forma ES
        // con args explícitos y se pinnean por conteo exacto 1 c/u
        // (patrón B-A142). Las formas cortas (`"{} puntos"`, `"{} datos"`)
        // no se buscan por ausencia: son substring de las agujas largas en
        // este mismo archivo y el `include_str!` se auto-matchearía
        // (precedente B-A89/B-A105); esos sitios quedan cubiertos por
        // wiring + salida de helpers.
        for needle in [
            ["{} puntos  L={", "len:.3}\""].concat(),
            ["{} puntos  L={", ":.3}\""].concat(),
            ["Locus: {} pun", "tos\""].concat(),
            ["{} datos · {", "} bins\""].concat(),
            ["BarChart[{} da", "tos]\""].concat(),
            ["PieChart[{} da", "tos]\""].concat(),
            ["{} pares · {} / {", "}\""].concat(),
            ["{} vértices  P={", ":.3}\""].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "álgebra sin ES duro viejo en prod (B-A143): {needle}"
            );
        }
        assert_eq!(
            source.matches("format!(\"{n} vértices\", n = n)").count(),
            1,
            "el único vértices-colapsado es el fallback del helper (B-A143)"
        );
        assert_eq!(
            source
                .matches("format!(\"{n} vértices  P={perim:.3}  A={area:.3}\", n = n)")
                .count(),
            1,
            "el único vértices-full es el fallback del helper (B-A143)"
        );
    }

    #[test]
    fn algebra_politopos_cono_cilindro_salen_de_helpers_for() {
        // B-A144: kinds 4D + familias N-D + cono/cilindro infinito salen de
        // `polychoron_kind_label_for`/`polytope_family_label_for` +
        // `polychoron_summary_for`/`polytope_nd_summary_for` +
        // `infinite_cone/cylinder_summary_for` y
        // `object_expression_summary_for(obj, locale)`. Sin egui: pines +
        // blindaje por `include_str!` (precedente B-A138/B-A143).
        use super::{
            infinite_cone_summary_for, infinite_cylinder_summary_for,
            object_expression_summary_for, polychoron_kind_label_for, polychoron_summary_for,
            polytope_family_label_for, polytope_nd_summary_for,
        };
        use grafito_core::{
            GeoObject, InfiniteCone3DObj, InfiniteCylinder3DObj, RegularPolychoron4DObj,
            RegularPolytopeNDObj,
        };
        use grafito_geometry::{Point3D, RegularPolychoron, RegularPolytopeFamily};
        use grafito_ui::i18n::{t, Locale};
        // Labels: Es reproduce el canónico previo.
        assert_eq!(
            polychoron_kind_label_for(RegularPolychoron::Pentachoron, Locale::Es),
            ["Pentá", "coron 4D"].concat()
        );
        assert_eq!(
            polychoron_kind_label_for(RegularPolychoron::Tesseract, Locale::En),
            "Tesseract 4D"
        );
        assert_eq!(
            polychoron_kind_label_for(RegularPolychoron::SixteenCell, Locale::De),
            "16-Zeller 4D"
        );
        assert_eq!(
            polytope_family_label_for(RegularPolytopeFamily::Simplex, Locale::Es),
            ["Sím", "plex"].concat()
        );
        assert_eq!(
            polytope_family_label_for(RegularPolytopeFamily::CrossPolytope, Locale::Fr),
            "Polytope croisé"
        );
        assert_eq!(
            polytope_family_label_for(RegularPolytopeFamily::Hypercube, Locale::It),
            "Ipercubo"
        );
        // Summaries Es: canónico previo byte-idéntico.
        let tesseract = RegularPolychoron4DObj::new(RegularPolychoron::Tesseract);
        assert_eq!(
            polychoron_summary_for(&tesseract, Locale::Es),
            ["Teseracto 4D centrado y proyectado · ", "escala=1.00"].concat()
        );
        assert_eq!(
            polychoron_summary_for(&tesseract, Locale::En),
            "Tesseract 4D centered and projected · scale=1.00"
        );
        let simplex = RegularPolytopeNDObj::new(RegularPolytopeFamily::Simplex, 3);
        assert_eq!(
            polytope_nd_summary_for(&simplex, Locale::Es),
            ["Símplex 3D centrado y proyectado · ", "escala=1.00"].concat()
        );
        assert_eq!(
            polytope_nd_summary_for(&simplex, Locale::Pt),
            "Simplexo 3D centrado e projetado · escala=1.00"
        );
        let cone = InfiniteCone3DObj::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(0.0, 0.0, 1.0),
            0.5,
        );
        assert_eq!(
            infinite_cone_summary_for(&cone, Locale::Es),
            ["ápice=(0.0,0.0,0.0) θ=28.6", "° (clip ±50)"].concat()
        );
        assert_eq!(
            infinite_cone_summary_for(&cone, Locale::De),
            "Apex=(0.0,0.0,0.0) θ=28.6° (clip ±50)"
        );
        let cyl = InfiniteCylinder3DObj::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            2.0,
        );
        assert_eq!(
            infinite_cylinder_summary_for(&cyl, Locale::Es),
            ["r=2.00 eje→(1.0,0.0,0.0", ") (clip ±50)"].concat()
        );
        assert_eq!(
            infinite_cylinder_summary_for(&cyl, Locale::Fr),
            "r=2.00 axe→(1.0,0.0,0.0) (clip ±50)"
        );
        // `_for` sobre objeto real: Teseracto Es y En.
        let obj = GeoObject::RegularPolychoron4D(RegularPolychoron4DObj::new(
            RegularPolychoron::Tesseract,
        ));
        assert_eq!(
            object_expression_summary_for(&obj, Locale::Es),
            ["Teseracto 4D centrado y proyectado · ", "escala=1.00"].concat()
        );
        assert_eq!(
            object_expression_summary_for(&obj, Locale::En),
            "Tesseract 4D centered and projected · scale=1.00"
        );
        assert_eq!(
            t("algebra.poly_kind_sixhundred", Locale::Es),
            ["600-ce", "lda 4D"].concat()
        );
        // Wiring: draws + `_for` + `t("algebra.poly_*"/"algebra.cone/cylinder")`.
        let source = include_str!("algebra.rs");
        for pin in [
            "polychoron_summary_for(polychoron, locale)",
            "polytope_nd_summary_for(polytope, locale)",
            "infinite_cone_summary_for(c, locale)",
            "infinite_cylinder_summary_for(c, locale)",
            "polychoron_summary_for(polychoron, alg_locale)",
            "polytope_nd_summary_for(polytope, alg_locale)",
            "infinite_cone_summary_for(c, alg_locale)",
            "infinite_cylinder_summary_for(c, alg_locale)",
            "t(\"algebra.poly_kind_pentachoron\",",
            "t(\"algebra.poly_projected_4d\",",
            "t(\"algebra.poly_projected_nd\",",
            "t(\"algebra.cone_infinite\",",
            "t(\"algebra.cylinder_infinite\",",
        ] {
            assert!(source.contains(pin), "álgebra B-A144 cablea {pin}");
        }
        // Blindaje: los literales ES viejos ya no están en prod. Kinds (6) y
        // familias (3) salieron a keys: ausencia total (pines del test por
        // `concat!` partido, precedente B-A89). Los 4 templates conservan su
        // forma ES solo en el fallback de cada helper: conteo exacto 1 c/u
        // (patrón B-A142/B-A143).
        for needle in [
            ["Pentá", "coron 4D\""].concat(),
            ["Teser", "acto 4D\""].concat(),
            ["16-ce", "lda 4D\""].concat(),
            ["24-ce", "lda 4D\""].concat(),
            ["120-ce", "lda 4D\""].concat(),
            ["600-ce", "lda 4D\""].concat(),
            ["\"Sím", "plex\""].concat(),
            ["\"Hiper", "cubo\""].concat(),
            ["Politopo cru", "zado\""].concat(),
        ] {
            assert!(
                !source.contains(needle.as_str()),
                "álgebra sin ES duro viejo en prod (B-A144): {needle}"
            );
        }
        assert_eq!(
            source
                .matches("\"{name} centrado y proyectado · escala={:.2}\",")
                .count(),
            1,
            "el único proyectado-4d es el fallback del helper (B-A144)"
        );
        assert_eq!(
            source
                .matches("\"ápice=({:.1},{:.1},{:.1}) θ={:.1}° (clip ±50)\",")
                .count(),
            1,
            "el único cono-infinito es el fallback del helper (B-A144)"
        );
        assert_eq!(
            source
                .matches("\"r={:.2} eje→({:.1},{:.1},{:.1}) (clip ±50)\",")
                .count(),
            1,
            "el único cilindro-infinito es el fallback del helper (B-A144)"
        );
    }

    #[test]
    fn algebra_fit_kind_sale_de_helper_for() {
        // B-A146: kinds de ajuste salen de `fit_kind_label_for` (Es reproduce
        // el canónico previo de `FitKind::display_name`) y
        // `object_expression_summary_for(obj, locale)`; `PlatonicKind::as_str`
        // queda invariante (identificadores EN, precedente B-A114). Sin egui:
        // pines + blindaje por `include_str!` (precedente B-A138/B-A144).
        use super::fit_kind_label_for;
        use grafito_core::PlatonicKind;
        use grafito_geometry::statistics::FitKind as FK;
        use grafito_ui::i18n::Locale;
        // Es pineado al canónico previo por `concat!` partido (el blindaje
        // usa `include_str!`, precedente B-A89/B-A144).
        assert_eq!(
            fit_kind_label_for(FK::Linear, Locale::Es),
            ["lin", "eal"].concat()
        );
        assert_eq!(
            fit_kind_label_for(FK::Polynomial { degree: 2 }, Locale::Es),
            ["polin", "ómico"].concat()
        );
        assert_eq!(
            fit_kind_label_for(FK::Exponential, Locale::Es),
            ["expon", "encial"].concat()
        );
        assert_eq!(
            fit_kind_label_for(FK::Logarithmic, Locale::Es),
            ["logar", "ítmico"].concat()
        );
        assert_eq!(
            fit_kind_label_for(FK::Power, Locale::Es),
            ["pot", "encia"].concat()
        );
        assert_eq!(
            fit_kind_label_for(FK::Sinusoidal, Locale::Es),
            ["sinus", "oidal"].concat()
        );
        assert_eq!(
            fit_kind_label_for(FK::Logistic, Locale::Es),
            ["logís", "tico"].concat()
        );
        assert_eq!(
            fit_kind_label_for(FK::Growth, Locale::Es),
            ["creci", "miento"].concat()
        );
        assert_eq!(
            fit_kind_label_for(FK::Implicit, Locale::Es),
            ["implí", "cito"].concat()
        );
        assert_eq!(fit_kind_label_for(FK::Linear, Locale::En), "linear");
        assert_eq!(fit_kind_label_for(FK::Logistic, Locale::En), "logistic");
        assert_eq!(
            fit_kind_label_for(FK::Polynomial { degree: 3 }, Locale::Pt),
            "polinomial"
        );
        assert_eq!(fit_kind_label_for(FK::Power, Locale::It), "potenza");
        assert_eq!(fit_kind_label_for(FK::Sinusoidal, Locale::Fr), "sinusoïdal");
        assert_eq!(fit_kind_label_for(FK::Growth, Locale::De), "Wachstum");
        assert_eq!(fit_kind_label_for(FK::Implicit, Locale::De), "implizit");
        // `PlatonicKind::as_str` son identificadores EN: invariantes de locale
        // a propósito (precedente B-A114 notación/unidades).
        assert_eq!(PlatonicKind::Dodecahedron.as_str(), "Dodecahedron");
        assert_eq!(PlatonicKind::Icosahedron.as_str(), "Icosahedron");
        assert_eq!(PlatonicKind::Octahedron.as_str(), "Octahedron");
        // Wiring: helper + resumen de tarjeta + `t("algebra.fit_kind_*")`.
        let source = include_str!("algebra.rs");
        for pin in [
            "fit_kind_label_for(fit.kind, locale)",
            "t(\"algebra.fit_kind_linear\",",
            "t(\"algebra.fit_kind_poly\",",
            "t(\"algebra.fit_kind_implicit\",",
        ] {
            assert!(source.contains(pin), "álgebra B-A146 cablea {pin}");
        }
        // Blindaje: el call-site crudo ya no está en prod (aguja partida:
        // `include_str!` incluye este test, precedente B-A89/B-A141).
        assert_eq!(
            source
                .matches(["fit.kind.display", "_name()"].concat().as_str())
                .count(),
            0,
            "el kind del ajuste sale del helper, no del display canónico (B-A146)"
        );
    }
}
