//! Pegar imagen del portapapeles en la pizarra (arboard directo).
//!
//! Veredicto arboard (arboard 3.6.1 pineado en `Cargo.lock`, feature
//! `image-data` por defecto): SIRVE en este entorno (Wayland,
//! `WAYLAND_DISPLAY=wayland-1` + X11 `DISPLAY=:1`). Evidencia medida con un
//! probe mínimo (`arboard::Clipboard::new` + `set_image`/`get_image`):
//! `Clipboard::new()` abre sin error, `get_image` con el portapapeles
//! vacío devuelve `Err(ContentNotAvailable)` honesto y el roundtrip
//! `set_image` 2×2 RGBA → `get_image` devuelve los mismos 16 bytes
//! (`width`/`height`/`bytes` fila-mayor RGBA8). `get_image` entrega RGBA
//! crudo (`ImageData { width, height, bytes }`), sin dimensiones hostiles
//! que temer más allá de las cotas ya existentes.
//!
//! Camino de inserción: EL MISMO que archivo
//! (`insert_whiteboard_image_from_file` en `whiteboard_ui.rs`): el RGBA se
//! codifica a PNG en memoria con el crate `image` YA declarado en
//! `grafito-app` (solo codec, como `whiteboard_png`), y desde ahí
//! `validate_image_bytes` (4 MiB / lado ≤ 2048 / PNG-JPEG por cabecera) +
//! `WhiteboardElement::new_image` + `try_add` + selección. Jamás bypass:
//! lo que excede se rechaza con el mismo mensaje que por archivo.
//!
//! Ramas honestas (nada de imagen rota): portapapeles OS no disponible o
//! sin imagen → `Err` en español CON la causa de arboard, que el llamador
//! muestra como toast; sin PNG válido no hay `Image`.
//!
//! Lectura síncrona en el hilo UI (como la inserción por archivo): `get_image`
//! es un solo roundtrip al servidor gráfico; la escritura al portapapeles
//! (`whiteboard_png::spawn_whiteboard_png_clipboard`) sí usa worker porque
//! Wayland bloquea al retener el contenido. Sin la feature `desktop-dialogs`
//! (p. ej. Android) el mismo `Err` honesto que el resto de la UI.

use grafito_whiteboard::{validate_image_bytes, WhiteboardElement};

use crate::whiteboard_formula::formula_image_rect;
use crate::whiteboard_ui::WhiteboardSession;

/// RGBA fila-mayor tal como lo entrega `arboard::ImageData` (4 canales por
/// píxel, `ancho×alto×4` bytes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ClipboardRgba {
    /// Ancho en píxeles (> 0).
    pub width: u32,
    /// Alto en píxeles (> 0).
    pub height: u32,
    /// Píxeles fila-mayor RGBA8 (`width×height×4` bytes exactos).
    pub bytes: Vec<u8>,
}

/// Lee el portapapeles OS como RGBA (`arboard::Clipboard::get_image`,
/// versión pineada 3.6.1). Vacío o con otro formato → `Err` honesto con la
/// causa; dimensiones nulas o búfer incompleto → `Err` sin pánico.
#[cfg(feature = "desktop-dialogs")]
pub(crate) fn read_clipboard_rgba() -> Result<ClipboardRgba, String> {
    let mut clipboard = arboard::Clipboard::new()
        .map_err(|error| format!("No se pudo pegar: portapapeles no disponible ({error})"))?;
    let image = clipboard
        .get_image()
        .map_err(|error| format!("No se pudo pegar: el portapapeles no trae imagen ({error})"))?;
    normalize_image_data(image.width, image.height, image.bytes.as_ref())
}

/// Sin `desktop-dialogs` (p. ej. Android): `Err` honesto, sin pánico.
#[cfg(not(feature = "desktop-dialogs"))]
pub(crate) fn read_clipboard_rgba() -> Result<ClipboardRgba, String> {
    Err("No se pudo pegar: portapapeles OS no disponible en esta compilación".to_string())
}

/// Normaliza dimensiones + búfer crudos a [`ClipboardRgba`] (puro,
/// testeable sin display): lo nulo o incompleto es `Err` honesto, jamás
/// imagen a medias. `usize` porque así los entrega arboard 3.6.1.
/// Solo existe con arboard o en tests (sin `desktop-dialogs` ni tests no
/// hay llamador prod: el `read` devuelve `Err` directo).
#[cfg(any(feature = "desktop-dialogs", test))]
fn normalize_image_data(
    width: usize,
    height: usize,
    bytes: &[u8],
) -> Result<ClipboardRgba, String> {
    if width == 0 || height == 0 {
        return Err("No se pudo pegar: el portapapeles trae imagen vacía".to_string());
    }
    let width_u32 = u32::try_from(width).map_err(|_| {
        "No se pudo pegar: la imagen del portapapeles es demasiado ancha".to_string()
    })?;
    let height_u32 = u32::try_from(height).map_err(|_| {
        "No se pudo pegar: la imagen del portapapeles es demasiado alta".to_string()
    })?;
    let expected = (width as u64)
        .checked_mul(height as u64)
        .and_then(|pixels| pixels.checked_mul(4));
    if expected != Some(bytes.len() as u64) {
        return Err("No se pudo pegar: el búfer del portapapeles está incompleto".to_string());
    }
    Ok(ClipboardRgba {
        width: width_u32,
        height: height_u32,
        bytes: bytes.to_vec(),
    })
}

/// Codifica el RGBA a PNG en memoria con el crate `image` ya declarado
/// (solo codec, igual que `whiteboard_png::encode_png_rgba`). `Err`
/// honesto si el búfer no cuadra o el codificador falla.
pub(crate) fn encode_clipboard_rgba(image: &ClipboardRgba) -> Result<Vec<u8>, String> {
    if image.width == 0 || image.height == 0 {
        return Err("No se pudo pegar: el portapapeles trae imagen vacía".to_string());
    }
    let buffer = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(
        image.width,
        image.height,
        image.bytes.clone(),
    )
    .ok_or_else(|| "No se pudo pegar: el búfer del portapapeles está incompleto".to_string())?;
    let mut out = Vec::new();
    image::DynamicImage::ImageRgba8(buffer)
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .map_err(|error| format!("No se pudo pegar: el codificador PNG falló ({error})"))?;
    if out.is_empty() {
        return Err("No se pudo pegar: el codificador PNG devolvió cero bytes".to_string());
    }
    Ok(out)
}

/// Inserta el RGBA en la sesión por el MISMO camino que archivo: PNG en
/// memoria → `validate_image_bytes` (4 MiB / 2048 / formato, los límites
/// existentes sin tocar) → rect 1:1 centrado en la vista (el mismo
/// [`formula_image_rect`] que fórmula/archivo) → `new_image` (revalida) →
/// `try_add` (cota 500/hoja) → selección. Devuelve el resumen para el
/// toast de éxito; lo rechazado es `Err` en español con la causa.
pub(crate) fn insert_clipboard_rgba(
    session: &mut WhiteboardSession,
    image: &ClipboardRgba,
) -> Result<String, String> {
    let png = encode_clipboard_rgba(image)?;
    // Límites pineados del tipo, SIN duplicar números: la única fuente es
    // `validate_image_bytes`, igual que la inserción por archivo.
    let (width, height) =
        validate_image_bytes(&png).map_err(|causa| format!("La imagen no entra: {causa}"))?;
    let zoom = if session.zoom.is_finite() && session.zoom > 0.0 {
        session.zoom
    } else {
        1.0
    };
    let center = (-session.pan.0 / zoom, -session.pan.1 / zoom);
    let element = WhiteboardElement::new_image(png, formula_image_rect(center, width, height))
        .map_err(|causa| format!("La imagen no entra: {causa}"))?;
    session.try_add(element)?;
    session.doc.select_at(center, f64::INFINITY);
    Ok("Imagen pegada desde el portapapeles.".to_string())
}

/// Pega la imagen del portapapeles en la pizarra con toasts honestos.
/// D2 lockdown: en examen no entra nada (igual que Imagen/Compartir); el
/// llamador ya lo filtra, aquí solo toasts. Nada de imagen rota: sin RGBA
/// o sin PNG válido no se toca la sesión.
pub(crate) fn paste_whiteboard_image_from_clipboard(app: &mut crate::GrafitoApp) {
    use grafito_ui::toast::ToastKind;
    let image = match read_clipboard_rgba() {
        Ok(image) => image,
        Err(causa) => {
            app.notify(causa, ToastKind::Error);
            return;
        }
    };
    match insert_clipboard_rgba(&mut app.whiteboard, &image) {
        Ok(resumen) => app.notify(resumen, ToastKind::Success),
        Err(mensaje) => app.notify(mensaje, ToastKind::Error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use grafito_whiteboard::MAX_IMAGE_BYTES;

    /// RGBA sólido `w`×`h` (un solo color: el PNG comprime chico y el test
    /// corre rápido; el ruido solo se usa donde se quiere PNG grande).
    fn solid_rgba(width: u32, height: u32, pixel: [u8; 4]) -> ClipboardRgba {
        let mut bytes = Vec::with_capacity(width as usize * height as usize * 4);
        for _ in 0..width as usize * height as usize {
            bytes.extend_from_slice(&pixel);
        }
        ClipboardRgba {
            width,
            height,
            bytes,
        }
    }

    /// Inserción desde bytes RGBA: 2×2 rojo/verde/azul/blanco → PNG en
    /// memoria vía `image` → entra por `new_image`+`try_add` y queda
    /// seleccionado (mismo camino que archivo, jamás bypass).
    #[test]
    fn clipboard_rgba_inserts_through_new_image() {
        let image = ClipboardRgba {
            width: 2,
            height: 2,
            bytes: vec![
                255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
            ],
        };
        // El PNG intermedio es válido y cuadra con el RGBA (codec `image`).
        let png = encode_clipboard_rgba(&image).expect("2×2 codifica");
        let decoded = image::load_from_memory(&png)
            .expect("PNG válido")
            .to_rgba8();
        assert_eq!((decoded.width(), decoded.height()), (2, 2));
        assert_eq!(decoded.into_raw(), image.bytes);
        let mut session = WhiteboardSession::default();
        let resumen = insert_clipboard_rgba(&mut session, &image).expect("2×2 entra");
        assert!(
            resumen.contains("portapapeles"),
            "toast de éxito honesto, fue: {resumen}"
        );
        assert_eq!(session.doc.len(), 1);
        assert_eq!(session.doc.selected(), Some(0));
        // Rect 1:1 centrado en la vista (zoom 1, pan 0): 2×2 mundo.
        assert_eq!(
            session.doc.elements()[0].bounds(),
            Some(((-1.0, -1.0), (1.0, 1.0)))
        );
    }

    /// Portapapeles vacío → `Err` honesto con causa (mock del borde puro:
    /// dimensiones nulas y búfer incompleto, sin display).
    #[test]
    fn clipboard_empty_errors_honestly() {
        let vacia = ClipboardRgba {
            width: 0,
            height: 0,
            bytes: Vec::new(),
        };
        let err = encode_clipboard_rgba(&vacia).expect_err("vacía no codifica");
        assert!(err.contains("vacía"), "causa honesta, fue: {err}");
        let rota = ClipboardRgba {
            width: 2,
            height: 2,
            bytes: vec![1, 2, 3],
        };
        let err = encode_clipboard_rgba(&rota).expect_err("incompleta no codifica");
        assert!(err.contains("incompleto"), "causa honesta, fue: {err}");
        // El borde arboard crudo también rechaza lo nulo/incompleto.
        assert!(normalize_image_data(0, 0, &[]).is_err());
        assert!(normalize_image_data(2, 2, &[1, 2, 3]).is_err());
        let ok = normalize_image_data(2, 1, &[0; 8]).expect("2×1 cuadra");
        assert_eq!((ok.width, ok.height), (2, 1));
        // Nada tocó la sesión ante lo inválido.
        let mut session = WhiteboardSession::default();
        assert!(insert_clipboard_rgba(&mut session, &vacia).is_err());
        assert!(insert_clipboard_rgba(&mut session, &rota).is_err());
        assert!(session.doc.is_empty());
    }

    /// Límites reutilizados, jamás bypass: lado > 2048 se rechaza con el
    /// mismo mensaje que por archivo (sin reescalado silencioso).
    #[test]
    fn clipboard_side_limit_reused() {
        // 3000×10 sólido: el PNG es chico pero el lado excede 2048.
        let image = solid_rgba(3000, 10, [200, 30, 30, 255]);
        let png = encode_clipboard_rgba(&image).expect("codifica igual");
        assert!(
            validate_image_bytes(&png).is_err(),
            "el límite existe antes de insertar"
        );
        let mut session = WhiteboardSession::default();
        let err = insert_clipboard_rgba(&mut session, &image).expect_err("lado fuera");
        assert!(
            err.contains("lado máximo"),
            "misma causa que archivo, fue: {err}"
        );
        assert!(session.doc.is_empty());
    }

    /// Tope de 4 MiB reutilizado: ruido 1200×1200 (el PNG apenas comprime,
    /// ~5.7 MiB > 4 MiB) se rechaza con la causa de bytes, sin OOM.
    #[test]
    fn clipboard_bytes_limit_reused() {
        let mut bytes = Vec::with_capacity(1200 * 1200 * 4);
        let mut state = 0x1234_5678u32;
        for _ in 0..1200 * 1200 {
            // LCG determinista: ruido sin `rand` (0 deps nuevas).
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            bytes.extend_from_slice(&state.to_le_bytes());
        }
        let image = ClipboardRgba {
            width: 1200,
            height: 1200,
            bytes,
        };
        let png = encode_clipboard_rgba(&image).expect("el ruido codifica");
        assert!(
            png.len() > MAX_IMAGE_BYTES,
            "el PNG de ruido supera 4 MiB ({})",
            png.len()
        );
        let mut session = WhiteboardSession::default();
        let err = insert_clipboard_rgba(&mut session, &image).expect_err("bytes fuera");
        assert!(
            err.contains("excede el máximo"),
            "misma causa que archivo, fue: {err}"
        );
        assert!(session.doc.is_empty());
    }

    /// Hoja llena (500/500): el pegado respeta la cota como archivo.
    #[test]
    fn clipboard_paste_respects_page_cap() {
        use crate::whiteboard_ui::MAX_WHITEBOARD_ELEMENTS_PER_PAGE;
        use grafito_whiteboard::WhiteboardElement;
        let image = solid_rgba(2, 2, [255, 0, 0, 255]);
        let mut session = WhiteboardSession::default();
        for _ in 0..MAX_WHITEBOARD_ELEMENTS_PER_PAGE {
            session
                .try_add(WhiteboardElement::Rectangle {
                    min: (0.0, 0.0),
                    max: (1.0, 1.0),
                    fill: None,
                })
                .expect("la hoja se llena");
        }
        let err = insert_clipboard_rgba(&mut session, &image).expect_err("hoja llena");
        assert!(err.contains("máximo"), "cota 500 como archivo, fue: {err}");
        assert_eq!(session.doc.len(), MAX_WHITEBOARD_ELEMENTS_PER_PAGE);
    }
}
