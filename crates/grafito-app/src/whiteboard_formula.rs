//! Botón `Fórmula` de la pizarra: diálogo de texto → `WhiteboardElement::Image`.
//!
//! Camino real (ver `grafito_whiteboard::formula`): valida el texto con el
//! presupuesto del modelo + el parser subset de `draw_math` (ANTES de
//! rasterizar: el rechazo conserva el borrador) y lo inserta como `Image`
//! SOLO si el raster produce PNG válido. El raster es puro y no promueve
//! `grafito-tex` al runtime de la app (veto intacto):
//! `grafito_ui::formula_raster::rasterize_formula_rgba` (el `Context` de
//! `draw_math` solo cachea texturas) + codificación PNG con el `image` ya
//! declarado aquí. Jamás imagen rota: sin PNG no hay `Image`.
//!
//! La inserción sigue el MISMO camino que archivo
//! (`insert_whiteboard_image_from_file` en `whiteboard_ui.rs`): rect
//! centrado en la vista con el tamaño en píxeles, `WhiteboardElement::new_image`
//! (valida 4 MiB + PNG/JPEG + lado ≤ 2048), `try_add`, selección y toast.

use grafito_whiteboard::{
    formula::{formula_raster_available, validate_formula_source, FORMULA_UNSUPPORTED_MSG},
    validate_image_bytes, WhiteboardElement,
};
use image::ImageEncoder;

/// Estado efímero del diálogo (memoria temporal de egui, no persiste ni va
/// al documento: como la selección de la sesión).
#[derive(Debug, Clone, Default)]
pub(crate) struct FormulaDialog {
    /// Diálogo visible.
    pub open: bool,
    /// Borrador del usuario (se conserva si el envío es rechazado).
    pub text: String,
}

fn formula_dialog_id() -> egui::Id {
    egui::Id::new("whiteboard_formula_dialog")
}

/// Lee el estado del diálogo (cerrado + vacío si nunca se abrió).
pub(crate) fn formula_dialog_get(ctx: &egui::Context) -> FormulaDialog {
    ctx.data_mut(|mapa| mapa.get_temp::<FormulaDialog>(formula_dialog_id()))
        .unwrap_or_default()
}

/// Guarda el estado del diálogo.
pub(crate) fn formula_dialog_set(ctx: &egui::Context, state: FormulaDialog) {
    ctx.data_mut(|mapa| mapa.insert_temp(formula_dialog_id(), state));
}

/// Resultado del envío del diálogo (puro, sin egui salvo el parser).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FormulaSubmit {
    /// Presupuesto + parser ok y raster disponible (el gate
    /// `formula_raster_available()` es `true`: el raster puro existe).
    Ready,
    /// Presupuesto o parser falló (mensaje en español para el toast).
    Rejected { message: String },
    /// Válida pero sin raster: el llamador avisa [`FORMULA_UNSUPPORTED_MSG`].
    Unsupported,
}

/// Valida la fuente: presupuesto ([`validate_formula_source`]) y después el
/// parser math existente (subset de `draw_math`, sin tex). Solo `Ready`
/// autoriza a rasterizar; lo demás es toast, jamás `Image`.
pub(crate) fn submit_whiteboard_formula(source: &str) -> FormulaSubmit {
    if let Err(error) = validate_formula_source(source) {
        return FormulaSubmit::Rejected {
            message: error.to_string(),
        };
    }
    if !grafito_ui::assistant::math_subset_supported(source) {
        return FormulaSubmit::Rejected {
            message: "no se pudo leer la fórmula (usá el asistente)".to_string(),
        };
    }
    if !formula_raster_available() {
        return FormulaSubmit::Unsupported;
    }
    FormulaSubmit::Ready
}

/// Costura del raster: convierte la fuente validada a PNG y devuelve sus
/// bytes. Puro salvo el `Vec` de salida: RGBA del helper de `grafito-ui`
/// (mismo motor que `draw_math`, sin `Context`) + PNG con el `image` ya
/// declarado en este crate. `Err` honesto en español (el motor aún rechaza
/// lo que el subset acepta: `\frac{}{}`, `a>b<c`, `x&y`); jamás píxeles
/// inventados.
fn rasterize_formula_to_png(source: &str) -> Result<Vec<u8>, String> {
    let mapa = grafito_ui::formula_raster::rasterize_formula_rgba(source)?;
    let mut png = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(
            &mapa.rgba,
            mapa.width,
            mapa.height,
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|error| format!("no se pudo codificar la fórmula: {error}"))?;
    Ok(png)
}

/// Rect destino centrado en la vista con el tamaño en píxeles (puro): el
/// mismo encuadre que la inserción por archivo (mundo @zoom 1, 1px = 1u).
pub(crate) fn formula_image_rect(
    view_center: (f64, f64),
    width: u32,
    height: u32,
) -> ((f64, f64), (f64, f64)) {
    (
        (
            view_center.0 - f64::from(width) / 2.0,
            view_center.1 - f64::from(height) / 2.0,
        ),
        (
            view_center.0 + f64::from(width) / 2.0,
            view_center.1 + f64::from(height) / 2.0,
        ),
    )
}

/// Envía el borrador del diálogo: valida, rasteriza si hay raster e inserta
/// por el camino de archivo. D2 lockdown: en examen no entra nada (igual que
/// Imagen/Compartir). Devuelve `true` si el diálogo debe cerrarse (insertada
/// o bloqueo de examen); `false` si queda abierto con el borrador intacto:
/// para corregir (rechazada), para llevarlo al asistente (sin raster) o
/// porque el motor no la maquetó (raster fallido: aviso honesto, sin `Image`).
pub(crate) fn insert_whiteboard_formula(app: &mut crate::GrafitoApp, source: &str) -> bool {
    match submit_whiteboard_formula(source) {
        FormulaSubmit::Rejected { message } => {
            app.notify(message, grafito_ui::toast::ToastKind::Error);
            false
        }
        FormulaSubmit::Unsupported => {
            app.notify(
                FORMULA_UNSUPPORTED_MSG.to_string(),
                grafito_ui::toast::ToastKind::Error,
            );
            false
        }
        FormulaSubmit::Ready => {
            if app.exam_blocks("Insertar fórmula") {
                return true;
            }
            let bytes = match rasterize_formula_to_png(source) {
                Ok(bytes) => bytes,
                Err(causa) => {
                    // El subset la aceptó pero el motor no la maquetó:
                    // aviso honesto y borrador intacto (igual que sin
                    // raster), jamás `Image` a medias.
                    app.notify(causa, grafito_ui::toast::ToastKind::Error);
                    return false;
                }
            };
            insert_formula_png(app, bytes);
            true
        }
    }
}

/// Inserta bytes PNG ya rasterizados por el camino de archivo (centro de la
/// vista, `new_image`, `try_add`, selección, toast). Solo se llama con PNG
/// válido del raster; lo inválido da toast, jamás imagen rota.
fn insert_formula_png(app: &mut crate::GrafitoApp, bytes: Vec<u8>) {
    let (width, height) = match validate_image_bytes(&bytes) {
        Ok(dims) => dims,
        Err(causa) => {
            app.notify(
                format!("La fórmula no entra: {causa}"),
                grafito_ui::toast::ToastKind::Error,
            );
            return;
        }
    };
    let zoom = if app.whiteboard.zoom.is_finite() && app.whiteboard.zoom > 0.0 {
        app.whiteboard.zoom
    } else {
        1.0
    };
    let center = (-app.whiteboard.pan.0 / zoom, -app.whiteboard.pan.1 / zoom);
    let element =
        match WhiteboardElement::new_image(bytes, formula_image_rect(center, width, height)) {
            Ok(element) => element,
            Err(causa) => {
                app.notify(
                    format!("La fórmula no entra: {causa}"),
                    grafito_ui::toast::ToastKind::Error,
                );
                return;
            }
        };
    match app.whiteboard.try_add(element) {
        Ok(()) => {
            app.whiteboard.doc.select_at(center, f64::INFINITY);
            app.notify("Fórmula insertada.", grafito_ui::toast::ToastKind::Success);
        }
        Err(mensaje) => app.notify(mensaje, grafito_ui::toast::ToastKind::Error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn submit_rechaza_presupuesto_roto() {
        assert!(matches!(
            submit_whiteboard_formula(""),
            FormulaSubmit::Rejected { message } if message.contains("vacía")
        ));
        assert!(matches!(
            submit_whiteboard_formula("   "),
            FormulaSubmit::Rejected { .. }
        ));
        assert!(matches!(
            submit_whiteboard_formula("x\0y"),
            FormulaSubmit::Rejected { .. }
        ));
        let larga = "x+".repeat(2048);
        assert!(matches!(
            submit_whiteboard_formula(&larga),
            FormulaSubmit::Rejected { message } if message.contains("bytes")
        ));
    }

    #[test]
    fn submit_rechaza_lo_que_el_subset_no_lee() {
        // Sintaxis rota: el parser del fallback la rechaza (toast de
        // corrección, diálogo abierto, jamás imagen).
        assert!(matches!(
            submit_whiteboard_formula("\\frac{1}{"),
            FormulaSubmit::Rejected { message } if message.contains("leer")
        ));
        assert!(matches!(
            submit_whiteboard_formula("x^"),
            FormulaSubmit::Rejected { .. }
        ));
    }

    #[test]
    fn submit_valida_llega_a_ready_con_gate_real() {
        // Sintaxis ok del subset + gate real: `Ready` (el raster puro
        // existe; el texto de sin-raster queda pineado por si el camino cae).
        assert!(formula_raster_available());
        assert_eq!(submit_whiteboard_formula("x^2"), FormulaSubmit::Ready);
        assert_eq!(
            submit_whiteboard_formula("\\frac{1}{2}"),
            FormulaSubmit::Ready
        );
        assert_eq!(
            FORMULA_UNSUPPORTED_MSG,
            "fórmula no soportada en pizarra (usá el asistente)"
        );
    }

    #[test]
    fn rasterize_da_png_valido_con_dimensiones_e_insertable() {
        // Punta a punta: fuente válida → PNG con firma + dimensiones > 0 por
        // el camino de archivo + `Image` construible (lo que
        // `insert_formula_png` exige antes de `try_add`).
        for fuente in ["x^2", "\\frac{1}{2}", "\\sqrt{x}+\\alpha"] {
            let png = rasterize_formula_to_png(fuente);
            assert!(png.is_ok(), "raster: {fuente}");
            let png = png.unwrap();
            assert!(png.starts_with(&[137, 80, 78, 71, 13, 10, 26, 10]));
            let (ancho, alto) = validate_image_bytes(&png).unwrap();
            assert!(ancho > 0 && alto > 0, "{fuente}: {ancho}x{alto}");
            let rect = formula_image_rect((0.0, 0.0), ancho, alto);
            assert!(WhiteboardElement::new_image(png, rect).is_ok());
        }
    }

    #[test]
    fn rasterize_rechaza_honesto_sin_imagen() {
        // Sintaxis rota: ni subset ni motor → `Err`, jamás PNG.
        for fuente in ["", "\\frac{1}{", "x^"] {
            assert!(rasterize_formula_to_png(fuente).is_err());
        }
        // El subset acepta pero el motor no maqueta (grupos vacíos,
        // `&`/`<` literales): `submit` da `Ready` y el raster falla honesto
        // (el `insert` avisa y conserva el borrador, sin `Image`).
        assert_eq!(
            submit_whiteboard_formula("\\frac{}{}"),
            FormulaSubmit::Ready
        );
        assert!(rasterize_formula_to_png("\\frac{}{}").is_err());
        assert_eq!(submit_whiteboard_formula("a>b<c"), FormulaSubmit::Ready);
        assert!(rasterize_formula_to_png("a>b<c").is_err());
    }

    #[test]
    fn rect_centrado_como_archivo() {
        // 100×50 px centrados en (0,0): mismo encuadre que por archivo.
        let rect = formula_image_rect((0.0, 0.0), 100, 50);
        assert_eq!(rect, ((-50.0, -25.0), (50.0, 25.0)));
        // El rect produce un `Image` válido con PNG mínimo (el camino de
        // inserción no se rompe cuando el raster exista).
        let mut png = vec![137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13];
        png.extend_from_slice(b"IHDR");
        png.extend_from_slice(&100u32.to_be_bytes());
        png.extend_from_slice(&50u32.to_be_bytes());
        png.extend_from_slice(&[8, 2, 0, 0, 0]);
        assert!(WhiteboardElement::new_image(png, rect).is_ok());
    }

    #[test]
    fn dialogo_persiste_en_memoria_temporal() {
        let ctx = egui::Context::default();
        assert!(!formula_dialog_get(&ctx).open);
        formula_dialog_set(
            &ctx,
            FormulaDialog {
                open: true,
                text: "x^2".to_string(),
            },
        );
        let leido = formula_dialog_get(&ctx);
        assert!(leido.open);
        assert_eq!(leido.text, "x^2");
    }
}
