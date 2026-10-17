//! OCR local foto→texto vía el CLI `tesseract` (sin nuevas dependencias cargo).
//!
//! Foto→OCR→solver acotado: cuando el usuario adjunta una imagen con una
//! cuenta o ecuación, este módulo extrae el texto con el binario
//! `tesseract` del sistema (`spa+eng`, `--psm 6`) y lo normaliza a una
//! forma que el solver en lenguaje natural (`solver_nl`) pueda entender
//! (`×`→`*`, `√x`→`sqrt(x)`, …).
//!
//! Contrato de seguridad (NO auto-ejecutar jamás):
//! - El OCR solo rellena `ImageAttachment.transcription` vía
//!   `EditableTranscription::edit` (que deja `reviewed = false`).
//! - El usuario debe revisar el texto y pedir explícitamente
//!   «resolvé … paso a paso»; el solver solo corre desde texto revisado.
//! - Si `tesseract` falta, falla o no detecta nada, se devuelve `Err`
//!   honesto y el adjunto viaja igual con transcripción vacía.

use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Idiomas pedidos a tesseract (español + inglés, como hay en esta máquina).
const OCR_LANGS: &str = "spa+eng";
/// Modo de segmentación: bloque uniforme de texto (cuenta fotografiada).
const OCR_PSM: &str = "6";
/// Timeout de espera del proceso tesseract antes de matarlo.
pub const OCR_TIMEOUT: Duration = Duration::from_secs(30);
/// Intervalo de sondeo mientras tesseract corre.
const OCR_POLL_INTERVAL: Duration = Duration::from_millis(50);

/// Contador para nombres únicos de archivos temporales (sin dependencias).
static OCR_TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// `true` si hay un `tesseract` ejecutable en el `PATH` (equivalente a
/// `which tesseract`). Nunca panica: a falta de `PATH` devuelve `false`.
pub fn is_tesseract_available() -> bool {
    executable_in_path("tesseract").is_some()
}

/// Busca un ejecutable en el `PATH` sin salir del `std`.
fn executable_in_path(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        if dir.as_os_str().is_empty() {
            continue;
        }
        #[cfg(windows)]
        {
            for ext in ["exe", "bat", "cmd"] {
                let candidate = dir.join(format!("{name}.{ext}"));
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
        #[cfg(not(windows))]
        {
            let candidate = dir.join(name);
            if candidate.is_file() && is_executable(&candidate) {
                return Some(candidate);
            }
        }
    }
    None
}

/// Bit ejecutable en unix (en Windows `is_file` + extensión ya filtra).
#[cfg(unix)]
fn is_executable(path: &std::path::Path) -> bool {
    use std::os::unix::fs::PermissionsExt as _;
    std::fs::metadata(path)
        .map(|meta| meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

/// En Windows todo binario válido pasa por extensión; el chequeo de modo
/// no aplica.
#[cfg(windows)]
fn is_executable(_path: &std::path::Path) -> bool {
    true
}

/// Normaliza texto crudo de OCR a entrada apta para el solver:
///
/// - `×`→`*`, `÷`→`/`, `−`/`–`/`—`→`-`, `∫`→`integral`
/// - `√x`→`sqrt(x)`, `√(…)`→`sqrt(…)` (ver [`wrap_sqrt_operands`])
/// - colapsa cualquier racha de blancos (espacios, saltos, `form feed` de
///   tesseract) a un solo espacio y recorta los bordes.
pub fn normalize_ocr_math(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        match ch {
            '×' => out.push('*'),
            '÷' => out.push('/'),
            '−' | '–' | '—' => out.push('-'),
            '∫' => out.push_str("integral"),
            other => out.push(other),
        }
    }
    let out = wrap_sqrt_operands(&out);
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Reescribe `√` + operando como `sqrt(operando)`.
///
/// - `√x` → `sqrt(x)`; `√9` → `sqrt(9)` (corre alfanumérico con `.`/`_`)
/// - `√(x+1)` → `sqrt(x+1)` (paréntesis balanceados)
/// - `√` sin operando (o fin de texto) → `sqrt` a secas, sin inventar nada.
fn wrap_sqrt_operands(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len() + 8);
    let mut i = 0;
    while i < chars.len() {
        if chars[i] != '√' {
            out.push(chars[i]);
            i += 1;
            continue;
        }
        i += 1;
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        if i < chars.len() && chars[i] == '(' {
            let start = i;
            let mut depth = 0;
            while i < chars.len() {
                if chars[i] == '(' {
                    depth += 1;
                } else if chars[i] == ')' {
                    depth -= 1;
                    if depth == 0 {
                        i += 1;
                        break;
                    }
                }
                i += 1;
            }
            // Honesto: con `(` sin cierre no se inventa el `)`; se emite
            // `sqrt` + resto crudo (el solver lo rechaza en vez de calcular
            // sobre una expresión fraguada).
            let inner: String = chars[start..i.min(chars.len())].iter().collect();
            out.push_str("sqrt");
            out.push_str(&inner);
        } else {
            let start = i;
            while i < chars.len()
                && (chars[i].is_alphanumeric() || chars[i] == '.' || chars[i] == '_')
            {
                i += 1;
            }
            if start < i {
                let inner: String = chars[start..i].iter().collect();
                out.push_str("sqrt(");
                out.push_str(&inner);
                out.push(')');
            } else {
                out.push_str("sqrt");
            }
        }
    }
    out
}

/// Comando instalable copiable para el OCR local (Arch; incluye datos
/// de español e inglés, los idiomas que pide [`OCR_LANGS`]).
pub const TESSERACT_INSTALL_CMD: &str =
    "sudo pacman -S tesseract tesseract-data-spa tesseract-data-eng";

/// Mensaje honesto cuando `tesseract` falta: causa + comando instalable
/// copiable + alternativa (pegar la transcripción en el chat). Puro, para
/// que el drain del adjunto y los tests lo compartan sin duplicar texto.
pub fn tesseract_missing_message() -> String {
    format!(
        "Tesseract no está instalado; no se pudo extraer texto de la imagen. Instalalo con `{TESSERACT_INSTALL_CMD}` o pegá la transcripción del ejercicio en el chat."
    )
}

/// Corre `tesseract img stdout -l spa+eng --psm 6` sobre bytes PNG/JPEG y
/// devuelve el texto normalizado ([`normalize_ocr_math`]).
///
/// - Escribe los bytes a un temporal (solo `std`), espera hasta
///   [`OCR_TIMEOUT`] y mata el proceso si se excede.
/// - `Err` honesto (español) si: entrada vacía o con formato inválido,
///   `tesseract` ausente, fallo de spawn/timeout, o salida vacía.
/// - Nunca panica; borra el temporal en todos los caminos.
pub fn ocr_bytes_to_text(png_or_jpeg_bytes: &[u8]) -> Result<String, String> {
    if png_or_jpeg_bytes.is_empty() {
        return Err("No hay imagen para reconocer.".to_string());
    }
    let ext = sniff_raster_ext(png_or_jpeg_bytes)
        .ok_or_else(|| "La imagen debe ser PNG o JPEG.".to_string())?;
    if !is_tesseract_available() {
        return Err(tesseract_missing_message());
    }
    let tmp = unique_tmp_path(ext);
    if let Err(error) = write_tmp(&tmp, png_or_jpeg_bytes) {
        remove_tmp(&tmp);
        return Err(error);
    }
    let result = run_tesseract(&tmp);
    remove_tmp(&tmp);
    result
}

/// Adivina la extensión por número mágico (`None` = no es PNG ni JPEG).
fn sniff_raster_ext(bytes: &[u8]) -> Option<&'static str> {
    if bytes.len() >= 8 && bytes[..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
        Some("png")
    } else if bytes.len() >= 3 && bytes[..3] == [0xFF, 0xD8, 0xFF] {
        Some("jpg")
    } else {
        None
    }
}

/// Ruta temporal única para esta imagen (`std` + pid + contador atómico).
fn unique_tmp_path(ext: &str) -> PathBuf {
    let id = OCR_TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("grafito-ocr-{}-{id}.{ext}", std::process::id()))
}

fn write_tmp(path: &std::path::Path, bytes: &[u8]) -> Result<(), String> {
    // `create_new`: si la ruta ya existe (colisión o symlink plantado),
    // falla honesto en vez de truncar algo ajeno.
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .and_then(|mut file| file.write_all(bytes))
        .map_err(|_| "No se pudo preparar la imagen para el OCR.".to_string())
}

fn remove_tmp(path: &std::path::Path) {
    let _ = std::fs::remove_file(path);
}

/// Spawnea tesseract, lo espera hasta [`OCR_TIMEOUT`] (lo mata si se
/// excede) y normaliza su `stdout`.
fn run_tesseract(img_path: &std::path::Path) -> Result<String, String> {
    let mut child = Command::new("tesseract")
        .arg(img_path)
        .arg("stdout")
        .arg("-l")
        .arg(OCR_LANGS)
        .arg("--psm")
        .arg(OCR_PSM)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| "No se pudo iniciar Tesseract.".to_string())?;
    let started = Instant::now();
    loop {
        match child
            .try_wait()
            .map_err(|_| "Falló la espera de Tesseract.".to_string())?
        {
            Some(status) => {
                let output = child
                    .wait_with_output()
                    .map_err(|_| "No se pudo leer la salida de Tesseract.".to_string())?;
                let text = normalize_ocr_math(&String::from_utf8_lossy(&output.stdout));
                if !text.is_empty() {
                    return Ok(text);
                }
                if !status.success() {
                    let detail: String = String::from_utf8_lossy(&output.stderr)
                        .split_whitespace()
                        .take(12)
                        .collect::<Vec<_>>()
                        .join(" ");
                    return Err(if detail.is_empty() {
                        "Tesseract no pudo leer la imagen.".to_string()
                    } else {
                        format!("Tesseract no pudo leer la imagen: {detail}")
                    });
                }
                return Err("Tesseract no detectó texto en la imagen.".to_string());
            }
            None => {
                if started.elapsed() >= OCR_TIMEOUT {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err("Tesseract tardó demasiado y se canceló.".to_string());
                }
                std::thread::sleep(OCR_POLL_INTERVAL);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{is_tesseract_available, normalize_ocr_math, ocr_bytes_to_text};

    #[test]
    fn normaliza_operadores_unicode() {
        assert_eq!(normalize_ocr_math("2 × 3"), "2 * 3");
        assert_eq!(normalize_ocr_math("8 ÷ 2"), "8 / 2");
        assert_eq!(normalize_ocr_math("5 − 3"), "5 - 3");
        assert_eq!(normalize_ocr_math("5 – 3"), "5 - 3");
        assert_eq!(normalize_ocr_math("5 — 3"), "5 - 3");
    }

    #[test]
    fn normaliza_raiz_e_integral() {
        assert_eq!(normalize_ocr_math("√x"), "sqrt(x)");
        assert_eq!(normalize_ocr_math("√(x+1)"), "sqrt(x+1)");
        assert_eq!(normalize_ocr_math("√9 + √y"), "sqrt(9) + sqrt(y)");
        assert_eq!(normalize_ocr_math("∫ x dx"), "integral x dx");
        // `√` huérfana no inventa operando; `(` sin cierre no inventa `)`.
        assert_eq!(normalize_ocr_math("√"), "sqrt");
        assert_eq!(normalize_ocr_math("√(x+1"), "sqrt(x+1");
    }

    #[test]
    fn colapsa_blancos_y_recorta() {
        // Ruido típico de tesseract: doble espacio, saltos y `form feed`.
        assert_eq!(
            normalize_ocr_math("  x^2  +  3x  =  0\n\x0c"),
            "x^2 + 3x = 0"
        );
        assert_eq!(normalize_ocr_math("   "), "");
        assert_eq!(normalize_ocr_math("2+2\n4"), "2+2 4");
    }

    #[test]
    fn detecta_tesseract_sin_panico() {
        // Solo presencia: en CI sin binario devuelve `false`, acá `true`.
        let _ = is_tesseract_available();
    }

    #[test]
    fn tesseract_ausente_trae_comando_instalable_y_alternativa() {
        // Sin tesseract el mensaje trae el comando copiable + la
        // alternativa (pegar la transcripción), jamás un mudo.
        let msg = super::tesseract_missing_message();
        assert!(
            msg.contains("sudo pacman -S tesseract tesseract-data-spa tesseract-data-eng"),
            "{msg}"
        );
        assert!(msg.contains(super::TESSERACT_INSTALL_CMD), "{msg}");
        assert!(msg.contains("transcripción"), "alternativa: {msg}");
    }

    #[test]
    fn rechaza_entrada_vacia_y_formato_invalido() {
        assert!(ocr_bytes_to_text(&[]).is_err());
        assert!(ocr_bytes_to_text(b"esto no es una imagen").is_err());
        assert!(ocr_bytes_to_text(&[0x47, 0x49, 0x46, 0x38]).is_err());
    }

    /// Integración REAL contra el binario tesseract presente: renderiza
    /// "2+3" con DejaVu + `ab_glyph`, lo codifica a PNG con `image` y lo
    /// pasa por [`ocr_bytes_to_text`]. Sin fuente o sin binario, degrada a
    /// asserts honestos de presencia (no falla en CI mínima).
    #[test]
    fn ocr_real_contra_binario_tesseract() {
        if !is_tesseract_available() {
            return;
        }
        let version = std::process::Command::new("tesseract")
            .arg("--version")
            .output()
            .map(|out| {
                format!(
                    "{}{}",
                    String::from_utf8_lossy(&out.stdout),
                    String::from_utf8_lossy(&out.stderr)
                )
            })
            .unwrap_or_default();
        assert!(
            version.to_lowercase().contains("tesseract"),
            "tesseract --version no se reconoce a sí mismo"
        );
        let Some(font_bytes) = load_test_font() else {
            return;
        };
        let png = render_test_png(&font_bytes, "2+3");
        let text = ocr_bytes_to_text(&png).expect("tesseract debe leer el PNG sintético");
        let compacto: String = text.split_whitespace().collect();
        assert!(
            compacto.contains("2+3"),
            "OCR no reconoció '2+3' en {text:?}"
        );
    }

    /// Fuente del sistema para el PNG sintético (`None` = CI sin fuentes).
    fn load_test_font() -> Option<Vec<u8>> {
        [
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        ]
        .iter()
        .find_map(|path| std::fs::read(path).ok())
    }

    /// Renderiza texto negro sobre blanco a PNG (solo `ab_glyph` + `image`,
    /// ambas dependencias existentes del crate; caret manual como en
    /// `anim_native.rs`, sin `layout`).
    fn render_test_png(font_bytes: &[u8], text: &str) -> Vec<u8> {
        use ab_glyph::{Font as _, FontRef, PxScale, ScaleFont as _};
        use image::ImageEncoder as _;
        let font = FontRef::try_from_slice(font_bytes).expect("fuente de test válida");
        let px = 120.0;
        let scaled = font.as_scaled(PxScale::from(px));
        let width = 640u32;
        let height = 220u32;
        let baseline = 160.0;
        let mut buf = vec![255u8; (width * height * 3) as usize];
        let mut caret_x = 30.0;
        let mut prev_id: Option<ab_glyph::GlyphId> = None;
        for ch in text.chars() {
            let gid = font.glyph_id(ch);
            if let Some(prev) = prev_id {
                caret_x += scaled.kern(prev, gid);
            }
            prev_id = Some(gid);
            let glyph = ab_glyph::Glyph {
                id: gid,
                scale: PxScale::from(px),
                position: ab_glyph::point(caret_x, baseline),
            };
            if let Some(outlined) = font.outline_glyph(glyph) {
                let bounds = outlined.px_bounds();
                outlined.draw(|x, y, coverage| {
                    if coverage > 0.5 {
                        let gx = bounds.min.x as i32 + x as i32;
                        let gy = bounds.min.y as i32 + y as i32;
                        if gx >= 0 && gy >= 0 && gx < width as i32 && gy < height as i32 {
                            let base = (gy as usize * width as usize + gx as usize) * 3;
                            buf[base] = 0;
                            buf[base + 1] = 0;
                            buf[base + 2] = 0;
                        }
                    }
                });
            }
            caret_x += scaled.h_advance(gid);
        }
        let mut png = Vec::new();
        image::codecs::png::PngEncoder::new(&mut png)
            .write_image(&buf, width, height, image::ExtendedColorType::Rgb8)
            .expect("codificar PNG de test");
        png
    }
}
