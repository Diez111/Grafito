//! Fórmula→pizarra: validación pura + gate del raster math→imagen.
//!
//! Camino real (sin dependencias nuevas): `grafito-tex::latex_to_rgba` SÍ
//! es puro (display-list de cajas/reglas + `tiny-skia`; el `egui::Context`
//! de `draw_math` solo cachea la textura y mide el DPR). `grafito-ui` —que
//! ya declara `grafito-tex` en runtime— lo expone como helper puro
//! (`formula_raster::rasterize_formula_rgba`) y `grafito-app` lo usa sin
//! promover `grafito-tex` a su runtime (veto intacto: la app solo pone su
//! `image` ya declarado para codificar el PNG).
//!
//! El botón `Fórmula` valida el texto (presupuesto aquí + parser subset
//! del llamador, que corre ANTES de rasterizar) e inserta como
//! `WhiteboardElement::Image` SOLO si el raster produce PNG válido (vía
//! `insert_formula_png`, con los límites de imagen ya existentes).
//! [`formula_raster_available`] es `true` porque ese camino funciona de
//! punta a punta (testeado en `grafito-app` y `grafito-ui`). El motor aún
//! puede rechazar lo que el subset acepta (`\frac{}{}`, `a>b<c`, `x&y`:
//! el subset ve texto y el motor maqueta MathML/XML real); ese `Err`
//! honesto avisa [`FORMULA_UNSUPPORTED_MSG`] con el borrador intacto.
//! Jamás imagen rota: sin PNG no hay `Image`.

/// Entrada máxima del diálogo de fórmula en bytes (2 KiB: una fórmula cabe
/// de sobra; espejo chico de `TEX_INPUT_MAX_BYTES` 8 KiB y muy por debajo de
/// `MAX_TEXT_BYTES` 64 KiB del `Text`).
pub const FORMULA_MAX_BYTES: usize = 2 * 1024;

/// Aviso honesto cuando la fórmula es válida pero no hay raster (el llamador
/// lo muestra tal cual; el texto exacto está pineado en tests).
pub const FORMULA_UNSUPPORTED_MSG: &str = "fórmula no soportada en pizarra (usá el asistente)";

/// Error de presupuesto de la fuente de fórmula (español, sin panics).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormulaError {
    /// Vacía o solo espacios (tras validar el tope de bytes, como tex).
    Empty,
    /// Sobre [`FORMULA_MAX_BYTES`] bytes.
    TooLong { got: usize },
    /// Contiene NUL (corta XML/SVG por la mitad si pasara).
    Nul,
}

impl std::fmt::Display for FormulaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "fórmula vacía"),
            Self::TooLong { got } => {
                write!(f, "fórmula de {got} bytes, máximo {FORMULA_MAX_BYTES}")
            }
            Self::Nul => write!(f, "la fórmula contiene NUL"),
        }
    }
}

impl std::error::Error for FormulaError {}

/// Valida el presupuesto de la fuente (pura): primero `TooLong` en bytes y
/// después `Empty`, para que un relleno gigante de espacios no se disfrace de
/// "vacío" (mismo orden que `check_tex_input` en `grafito-tex`). La validez
/// sintáctica la chequea el llamador con el parser math existente (subset de
/// `draw_math`); aquí no hay parser que declarar.
pub fn validate_formula_source(source: &str) -> Result<(), FormulaError> {
    let len = source.len();
    if len > FORMULA_MAX_BYTES {
        return Err(FormulaError::TooLong { got: len });
    }
    if source.trim().is_empty() {
        return Err(FormulaError::Empty);
    }
    if source.contains('\0') {
        return Err(FormulaError::Nul);
    }
    Ok(())
}

/// ¿Hay raster math→PNG disponible en este alcance? `true`: el camino puro
/// existe (`grafito-ui::formula_raster` sobre `grafito-tex::latex_to_rgba`,
/// testeado de punta a punta). El llamador inserta como `Image` SOLO si
/// esto es `true` Y el raster devuelve PNG válido; si no, avisa
/// [`FORMULA_UNSUPPORTED_MSG`]. Función (no const) para no romper la firma
/// si el camino cambia.
pub fn formula_raster_available() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::{
        formula_raster_available, validate_formula_source, FormulaError, FORMULA_MAX_BYTES,
        FORMULA_UNSUPPORTED_MSG,
    };

    #[test]
    fn presupuesto_pineado() {
        assert_eq!(FORMULA_MAX_BYTES, 2 * 1024);
    }

    #[test]
    fn mensaje_no_soportada_pineado() {
        assert_eq!(
            FORMULA_UNSUPPORTED_MSG,
            "fórmula no soportada en pizarra (usá el asistente)"
        );
    }

    #[test]
    fn raster_disponible_via_ui() {
        // El camino puro existe (ver encabezado): el botón se muestra y el
        // llamador inserta SOLO con PNG válido, jamás bytes inventados.
        assert!(formula_raster_available());
    }

    #[test]
    fn fuentes_validas_pasan() {
        assert_eq!(validate_formula_source("x^2"), Ok(()));
        assert_eq!(validate_formula_source("\\frac{1}{2}"), Ok(()));
        assert_eq!(validate_formula_source("  x  "), Ok(()));
        // Justo en el tope (no vacío): pasa.
        let tope = format!("x{:width$}", "", width = FORMULA_MAX_BYTES - 1);
        assert_eq!(tope.len(), FORMULA_MAX_BYTES);
        assert_eq!(validate_formula_source(&tope), Ok(()));
    }

    #[test]
    fn vacia_nul_y_larga_fallan_honesto() {
        assert_eq!(validate_formula_source(""), Err(FormulaError::Empty));
        assert_eq!(validate_formula_source("   \n "), Err(FormulaError::Empty));
        assert_eq!(validate_formula_source("x\0y"), Err(FormulaError::Nul));
        let larga = "x+".repeat(FORMULA_MAX_BYTES);
        assert!(matches!(
            validate_formula_source(&larga),
            Err(FormulaError::TooLong { .. })
        ));
        // Relleno gigante de espacios: TooLong, no Empty disfrazado.
        let relleno = " ".repeat(FORMULA_MAX_BYTES + 1);
        assert!(matches!(
            validate_formula_source(&relleno),
            Err(FormulaError::TooLong { .. })
        ));
        if let Err(FormulaError::TooLong { got }) = validate_formula_source(&larga) {
            assert_eq!(got, larga.len());
        } else {
            panic!("se esperaba TooLong con el conteo");
        }
    }

    #[test]
    fn display_en_espanol_sin_panics() {
        assert_eq!(format!("{}", FormulaError::Empty), "fórmula vacía");
        assert!(format!("{}", FormulaError::TooLong { got: 9999 }).contains("9999"));
        assert_eq!(format!("{}", FormulaError::Nul), "la fórmula contiene NUL");
    }
}
