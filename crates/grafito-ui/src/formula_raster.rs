//! Raster puro fórmula→RGBA para la pizarra (sin `egui::Context` ni `Painter`).
//!
//! Camino real del botón `Fórmula`: `grafito-tex::latex_to_rgba` es puro
//! (display-list de cajas/reglas + `tiny-skia`, sin `Context` ni texturas:
//! el `Context` de `draw_math` solo cachea la textura y mide el DPR, ver
//! `rasterizar_formula_tex_for` en `assistant.rs`). Este helper lo expone
//! con tipos propios para que `grafito-app` —que no declara `grafito-tex`
//! en runtime— rasterice sin promover el crate: la app codifica el PNG con
//! su `image` ya declarado y lo inserta por el camino de archivo.
//!
//! Tinta negra opaca y `TEX_DEFAULT_FONT_PX` (24 px, igual que `draw_math` a
//! DPR 1): la `Image` de pizarra guarda píxeles, no puntos lógicos.
//!
//! Contrato honesto: cualquier `Err` (vacío, presupuesto, parse, layout,
//! dimensiones) es `String` en español del motor; el llamador avisa con
//! toast y jamás construye `Image`.

/// Bitmap RGBA recto de una fórmula (salida pura del raster).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormulaBitmap {
    /// Bytes RGBA no premultiplicados (`width*height*4`), fila a fila.
    pub rgba: Vec<u8>,
    /// Ancho en píxeles (>0).
    pub width: u32,
    /// Alto en píxeles (>0).
    pub height: u32,
}

/// Rasteriza la fuente validada a RGBA (puro, sin egui).
///
/// El llamador valida antes el presupuesto (`validate_formula_source` en
/// `grafito-whiteboard`) y el subset (`math_subset_supported` aquí mismo);
/// aun así el motor puede rechazar lo que el subset acepta (`\frac{}{}`,
/// `a>b<c`, `x&y`: el subset ve texto y el motor maqueta MathML/XML real)
/// y ese `Err` honesto significa "sin imagen", jamás píxeles inventados.
pub fn rasterize_formula_rgba(source: &str) -> Result<FormulaBitmap, String> {
    let mapa = grafito_tex::latex_to_rgba(source, grafito_tex::TEX_DEFAULT_FONT_PX)
        .map_err(|error| error.to_string())?;
    if mapa.width == 0 || mapa.height == 0 {
        return Err("la fórmula rasterizó con dimensiones nulas".to_string());
    }
    let esperado = mapa
        .width
        .checked_mul(mapa.height)
        .and_then(|pixeles| pixeles.checked_mul(4));
    if esperado != Some(mapa.rgba.len()) {
        return Err("la fórmula rasterizó con dimensiones incoherentes".to_string());
    }
    let width = u32::try_from(mapa.width)
        .map_err(|_| "la fórmula rasterizó demasiado ancha".to_string())?;
    let height = u32::try_from(mapa.height)
        .map_err(|_| "la fórmula rasterizó demasiado alta".to_string())?;
    Ok(FormulaBitmap {
        rgba: mapa.rgba,
        width,
        height,
    })
}

#[cfg(test)]
mod tests {
    use super::rasterize_formula_rgba;
    use crate::assistant::math_subset_supported;

    #[test]
    fn subset_valido_rasteriza_con_dimensiones_y_tinta() {
        // El subset del botón y el motor cubren lo mismo en la práctica.
        for fuente in ["x^2", "\\frac{1}{2}", "\\sqrt{x}+\\alpha", "x^2 + 1"] {
            assert!(math_subset_supported(fuente), "subset: {fuente}");
            let mapa = rasterize_formula_rgba(fuente);
            assert!(mapa.is_ok(), "raster: {fuente}");
            let mapa = mapa.unwrap();
            assert!(mapa.width > 0 && mapa.height > 0);
            assert_eq!(
                mapa.rgba.len(),
                mapa.width as usize * mapa.height as usize * 4
            );
            // Hay tinta real (jamás lienzo vacío con `Ok`).
            assert!(
                mapa.rgba.chunks_exact(4).any(|px| px[3] > 0),
                "sin alfa en {fuente}"
            );
        }
    }

    #[test]
    fn subset_acepta_pero_motor_rechaza_honesto() {
        // El subset ve texto (`\frac{}{}` son dos grupos vacíos, `&` una
        // letra más) y el motor MathML/XML real lo rechaza: `Err` en
        // español, jamás imagen.
        for fuente in ["\\frac{}{}", "a>b<c", "x&y", "$x^2$"] {
            assert!(math_subset_supported(fuente), "subset acepta {fuente:?}");
            assert!(
                rasterize_formula_rgba(fuente).is_err(),
                "motor rechaza {fuente:?}"
            );
        }
    }

    #[test]
    fn entradas_invalidas_dan_err_sin_panics() {
        for fuente in [
            "",
            "   ",
            "\\frac{1}{",
            "x^",
            "\\comandoinexistente",
            "x\0y",
        ] {
            assert!(!math_subset_supported(fuente) || rasterize_formula_rgba(fuente).is_err());
            // El helper solo: vacío/NUL/sintaxis rota del motor → `Err`.
            if ["", "   ", "\\frac{1}{", "x\0y"].contains(&fuente) {
                assert!(rasterize_formula_rgba(fuente).is_err(), "{fuente:?}");
            }
        }
        // Sobre el presupuesto tex (8 KiB): `TooLong` honesto.
        let larga = "x+".repeat(8 * 1024);
        assert!(rasterize_formula_rgba(&larga).is_err());
    }
}
