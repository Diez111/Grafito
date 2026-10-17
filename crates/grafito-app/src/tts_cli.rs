//! Lectura en voz alta de pasos vía el CLI `espeak-ng` (sin nuevas
//! dependencias cargo).
//!
//! La tarjeta ```grafito-steps gana un botón «Leer»: al pulsarlo, los pasos
//! visibles se hablan en español con el binario `espeak-ng` del sistema
//! (`-v es` pineado); la segunda pulsación detiene (toggle, jamás dos
//! audios solapados). Mismo patrón que [`crate::ocr_cli`]: solo
//! `std::process` + binario externo, presencia por `PATH`, comando
//! instalable honesto si falta.
//!
//! Contrato de voz (PROHIBIDO auto-play):
//! - Nada habla solo: `speak_steps_text` / `spawn_speech` solo se llaman
//!   desde el handler de `ToggleSpeakSteps` (clic explícito en «Leer»).
//!   El draw jamás spawnea (emite la acción y la app ejecuta fuera).
//! - Sin sonidos de marca ni celebración: solo la lectura pedida.
//! - Sin hilos propios: el habla vive en un `Child` que la app sondea con
//!   `try_wait` por frame (no bloquea la UI); `stop` hace kill+wait
//!   (sin zombies) y `on_exit` lo mata (nada sobrevive al cerrar).
//! - Timeout [`TTS_TIMEOUT`]: si espeak se cuelga, se mata.
//!
//! LÍMITE documentado (sin verbalizador matemático): no hay un motor que
//! convierta fórmulas a prosa (`x^2/(1+x)` no se vuelve «equis al cuadrado
//! sobre uno más equis»). [`sanitize_speech_text`] aplica prosa mínima a
//! los símbolos más comunes (`×`→«por», `√`→«raíz de», `→`→«pasa a», …)
//! y el resto del texto del paso se lee TAL CUAL (letras, dígitos,
//! paréntesis). Si un día hay verbalizador, se reemplaza el saneado; jamás
//! se inventa contenido para que «suene mejor».

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// Binario de síntesis (en `PATH`; Arch: `espeak-ng`).
pub const ESPEAK_BIN: &str = "espeak-ng";
/// Voz española pineada (`es` = español de España, verificado con
/// `espeak-ng --voices`; la variante `es-419` existe pero NO se usa: un
/// solo pin, sin giros por máquina).
pub const ESPEAK_VOICE: &str = "es";
/// Timeout de espera del habla antes de matarla (2000 chars ≈ 300
/// palabras ≈ 2 min a 150 ppm; la app también lo aplica en su sondeo).
pub const TTS_TIMEOUT: Duration = Duration::from_secs(120);
/// Intervalo de sondeo mientras espeak habla (bloqueante solo en
/// [`speak_steps_text`], que corre en hilo worker o test).
#[cfg_attr(not(test), allow(dead_code))]
const TTS_POLL_INTERVAL: Duration = Duration::from_millis(50);
/// Texto máximo a hablar en chars (cota de tiempo y de `argv`; el resto se
/// recorta en borde de char, jamás a medias).
pub const TTS_MAX_TEXT_CHARS: usize = 2000;

/// Comando instalable copiable para la voz local (Arch; `es` viene en el
/// paquete base, sin datos extra como tesseract).
pub const ESPEAK_INSTALL_CMD: &str = "sudo pacman -S espeak-ng";

/// `true` si hay un `espeak-ng` ejecutable en el `PATH`. Nunca panica.
pub fn is_espeak_available() -> bool {
    executable_in_path(ESPEAK_BIN).is_some()
}

/// Busca un ejecutable en el `PATH` sin salir del `std` (espejo del helper
/// privado de [`crate::ocr_cli`]: se duplica a propósito para no abrir
/// visibilidad interna entre módulos).
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

/// Bit ejecutable en unix.
#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt as _;
    std::fs::metadata(path)
        .map(|meta| meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

/// Mensaje honesto cuando `espeak-ng` falta: causa + comando instalable
/// copiable. Puro, para que el handler del toggle y los tests lo compartan
/// (mismo patrón que `tesseract_missing_message`, sin la alternativa de
/// transcripción: acá no hay texto que revisar, solo instalar o no leer).
pub fn espeak_missing_message() -> String {
    format!(
        "Sin voz para leer los pasos; instalá espeak-ng con `{ESPEAK_INSTALL_CMD}` o seguí leyendo en pantalla."
    )
}

/// Sanea texto de pasos a prosa mínima hablable (puro, testeable).
///
/// - Cada línea no vacía es una frase (pausa entre pasos al unir con
///   «. »); líneas en blanco se caen (un turno vacío sanea a `""` y el
///   handler avisa en vez de hablar un punto); `\r` y tabulador →
///   espacio; comillas `"`/`'` se quitan (ruido «comillas» en el habla).
/// - Símbolos comunes → palabras (`×`→«por», `√`→«raíz de»,
///   `→`→«pasa a», …; ver tabla abajo). El ASCII `+`/`-`/paréntesis se
///   deja tal cual (espeak-es ya los dice bien).
/// - Colapsa blancos y recorta a [`TTS_MAX_TEXT_CHARS`] en borde de char.
///
/// LÍMITE (ver docs del módulo): lo no mapeado se lee tal cual; no se
/// verbalizan fracciones, matrices ni límites como prosa real.
pub fn sanitize_speech_text(raw: &str) -> String {
    let mut frases = Vec::new();
    for linea in raw.split('\n') {
        let mut out = String::with_capacity(linea.len().min(TTS_MAX_TEXT_CHARS + 16));
        for ch in linea.chars() {
            match ch {
                '\r' | '\t' => out.push(' '),
                '"' | '\'' => {}
                '×' | '*' => out.push_str(" por "),
                '÷' | '/' => out.push_str(" dividido "),
                '−' | '–' | '—' => out.push_str(" menos "),
                '√' => out.push_str(" raíz de "),
                '∫' => out.push_str(" integral de "),
                '∞' => out.push_str(" infinito "),
                'π' => out.push_str(" pi "),
                '≠' => out.push_str(" distinto de "),
                '≤' => out.push_str(" menor o igual que "),
                '≥' => out.push_str(" mayor o igual que "),
                '±' => out.push_str(" más menos "),
                '≈' => out.push_str(" aproximadamente "),
                '→' => out.push_str(" pasa a "),
                '·' => out.push_str(", "),
                '^' => out.push_str(" elevado a "),
                '_' => out.push(' '),
                '%' => out.push_str(" por ciento "),
                '=' => out.push_str(" igual "),
                other => out.push(other),
            }
        }
        let colapsada = out.split_whitespace().collect::<Vec<_>>().join(" ");
        if !colapsada.is_empty() {
            frases.push(colapsada);
        }
    }
    // «fin de frase.» + separador «. » pega dos puntos: se colapsa a uno.
    let mut unido = frases.join(". ");
    while unido.contains("..") {
        unido = unido.replace("..", ".");
    }
    let unido = unido.trim().to_string();
    if unido.chars().count() <= TTS_MAX_TEXT_CHARS {
        return unido;
    }
    // Corte honesto en borde de char (`chars().take` jamás parte UTF-8).
    unido
        .chars()
        .take(TTS_MAX_TEXT_CHARS)
        .collect::<String>()
        .trim_end()
        .to_string()
}

/// `argv` pineado para espeak: `["-v", voz, texto]` (voz española por
/// defecto vía [`ESPEAK_VOICE`]). Puro: el test lo pinea contra regresión
/// de flags (un flag extra como `-s` cambiaría la voz percibida).
pub fn espeak_argv(text: &str, voice: &str) -> Vec<String> {
    vec!["-v".to_string(), voice.to_string(), text.to_string()]
}

/// Habla `text` con `voz` de forma BLOQUEANTE (hilo worker o test):
/// spawnea `espeak-ng -v voz texto`, lo espera hasta [`TTS_TIMEOUT`] y lo
/// mata si se excede (kill+wait, sin zombies).
///
/// - `Err` honesto (español) si: texto vacío (tras sanear), voz vacía,
///   binario ausente, fallo de spawn/timeout, o salida con error.
/// - Nunca panica. El audio sale por el dispositivo (stdout/stderr a null).
///
/// API requerida del módulo: hoy la ejerce el funeral de la sesión (hilo
/// worker / tests); la UI interactiva usa `spawn_speech` + sondeo por
/// frame para no bloquearse. `not(test)` silencia el aviso en builds sin
/// tests (precedente `app.rs:1118`).
#[cfg_attr(not(test), allow(dead_code))]
pub fn speak_steps_text(text: &str, voice: &str) -> Result<(), String> {
    speak_steps_text_with_bin(text, voice, Path::new(ESPEAK_BIN))
}

/// Idem con binario explícito (tests herméticos + distros sin `PATH`).
#[cfg_attr(not(test), allow(dead_code))]
pub fn speak_steps_text_with_bin(text: &str, voice: &str, bin: &Path) -> Result<(), String> {
    let clean = sanitize_speech_text(text);
    if clean.is_empty() {
        return Err("No hay texto para leer.".to_string());
    }
    if voice.trim().is_empty() {
        return Err("Voz vacía: usá `es`.".to_string());
    }
    let mut child = spawn_speech_with_bin(&clean, voice, bin)?;
    let started = Instant::now();
    loop {
        match child
            .try_wait()
            .map_err(|_| "Falló la espera de la voz.".to_string())?
        {
            Some(status) => {
                // `wait` tras `try_wait` listo: cosecha el zombie sin
                // bloquear (ya terminó) y deja el código para el mensaje.
                let _ = child.wait();
                if status.success() {
                    return Ok(());
                }
                return Err(format!(
                    "La voz terminó con error (código {}).",
                    status.code().unwrap_or(-1)
                ));
            }
            None => {
                if started.elapsed() >= TTS_TIMEOUT {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err("La lectura tardó demasiado y se detuvo.".to_string());
                }
                std::thread::sleep(TTS_POLL_INTERVAL);
            }
        }
    }
}

/// Spawnea `espeak-ng -v es texto` SIN esperar (la app lo sondea por frame
/// con `try_wait`: la UI jamás se bloquea). Audio al dispositivo,
/// stdio a null. `Err` honesto si el texto queda vacío o no hay binario.
pub fn spawn_speech(text: &str) -> Result<Child, String> {
    spawn_speech_with_bin(text, ESPEAK_VOICE, Path::new(ESPEAK_BIN))
}

/// Idem con voz y binario explícitos (tests herméticos).
pub fn spawn_speech_with_bin(text: &str, voice: &str, bin: &Path) -> Result<Child, String> {
    let clean = sanitize_speech_text(text);
    if clean.is_empty() {
        return Err("No hay texto para leer.".to_string());
    }
    if voice.trim().is_empty() {
        return Err("Voz vacía: usá `es`.".to_string());
    }
    Command::new(bin)
        .args(espeak_argv(&clean, voice.trim()))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                espeak_missing_message()
            } else {
                format!("No se pudo iniciar la voz: {error}")
            }
        })
}

// ── Habla en curso (la app lo guarda; sondeo sin hilos) ─────────────────

/// Habla en curso de la tarjeta de pasos: el `Child` lo sondea la app por
/// frame ([`StepsTtsSpeech::poll_done`], no bloquea); `stop` hace
/// kill+wait (sin zombies, sin solape: el handler mata antes de hablar).
#[derive(Debug)]
pub struct StepsTtsSpeech {
    /// Turno del transcript que se está leyendo (la 2ª pulsación en ESTE
    /// turno detiene; en otro turno cambia el habla).
    pub turn: usize,
    /// Proceso espeak en curso.
    pub child: Child,
    /// Arranque (para el timeout aunque el sondeo por frame se pause).
    pub started: Instant,
}

impl StepsTtsSpeech {
    /// Detiene el habla: kill+wait (cosecha el zombie). Consume el habla.
    pub fn stop(mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }

    /// Sondeo no bloqueante: `true` = terminó (éxito, fallo o timeout con
    /// kill+wait) y la app debe soltar el habla + repintar. Nunca panica.
    pub fn poll_done(&mut self) -> bool {
        match self.child.try_wait() {
            Ok(Some(_)) => true,
            Ok(None) => {
                if self.started.elapsed() >= TTS_TIMEOUT {
                    let _ = self.child.kill();
                    let _ = self.child.wait();
                    true
                } else {
                    false
                }
            }
            Err(_) => true,
        }
    }
}

/// Decisión pura del toggle «Leer» (sin I/O, sin binario: testeable con
/// gate en vez de mock).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TtsToggle {
    /// Nada suena: empezar a hablar el turno pulsado.
    Start,
    /// Suena ESTE turno: detener (2ª pulsación).
    Stop,
    /// Suena OTRO turno: detener ese y empezar este (jamás solapar).
    Switch,
}

/// Qué hace la pulsación en `pressed` si suena `current` (`None` = silencio).
/// Pura.
pub fn tts_toggle_decision(current: Option<usize>, pressed: usize) -> TtsToggle {
    match current {
        None => TtsToggle::Start,
        Some(sounding) if sounding == pressed => TtsToggle::Stop,
        Some(_) => TtsToggle::Switch,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saneado_mapea_simbolos_a_prosa_minima() {
        assert_eq!(sanitize_speech_text("2 × 3"), "2 por 3");
        assert_eq!(sanitize_speech_text("8 ÷ 2"), "8 dividido 2");
        assert_eq!(sanitize_speech_text("2*x"), "2 por x");
        assert_eq!(sanitize_speech_text("1/2"), "1 dividido 2");
        assert_eq!(sanitize_speech_text("5 − 3"), "5 menos 3");
        assert_eq!(sanitize_speech_text("√x"), "raíz de x");
        assert_eq!(sanitize_speech_text("∫ x dx"), "integral de x dx");
        assert_eq!(sanitize_speech_text("x → 2*x"), "x pasa a 2 por x");
        assert_eq!(sanitize_speech_text("x^2"), "x elevado a 2");
        assert_eq!(sanitize_speech_text("x_1"), "x 1");
        assert_eq!(sanitize_speech_text("50%"), "50 por ciento");
        assert_eq!(sanitize_speech_text("x = 2"), "x igual 2");
        assert_eq!(sanitize_speech_text("π ≈ 3.14"), "pi aproximadamente 3.14");
        assert_eq!(sanitize_speech_text("a ≠ b"), "a distinto de b");
        assert_eq!(
            sanitize_speech_text("a ≤ b, b ≥ a"),
            "a menor o igual que b, b mayor o igual que a"
        );
        assert_eq!(sanitize_speech_text("x ± 1"), "x más menos 1");
        assert_eq!(sanitize_speech_text("∞"), "infinito");
    }

    #[test]
    fn saneado_pasos_multilinea_con_pausa_y_sin_basura() {
        // Cada línea no vacía es una frase (pausa entre pasos); comillas
        // fuera; blancos colapsados; puntos dobles («fin de línea.» +
        // separador) a uno; líneas en blanco se caen (vacío → sin habla).
        assert_eq!(
            sanitize_speech_text("Paso 1.\n\"x^2\"\n  pasa  "),
            "Paso 1. x elevado a 2. pasa"
        );
        assert_eq!(sanitize_speech_text("  a   b\tc  "), "a b c");
        assert_eq!(sanitize_speech_text(""), "");
        assert_eq!(sanitize_speech_text("   \n  "), "");
        assert_eq!(sanitize_speech_text("\n\n"), "");
    }

    #[test]
    fn saneado_trunca_en_borde_de_char() {
        let largo = "á".repeat(TTS_MAX_TEXT_CHARS + 40);
        let cut = sanitize_speech_text(&largo);
        assert_eq!(cut.chars().count(), TTS_MAX_TEXT_CHARS);
        assert!(cut.is_char_boundary(cut.len()));
        // Corto queda intacto (sin truncar de más).
        assert_eq!(sanitize_speech_text("hola"), "hola");
    }

    #[test]
    fn comando_pineado_voz_y_args() {
        // Binario + voz española pineados (un cambio acá cambia la voz).
        assert_eq!(ESPEAK_BIN, "espeak-ng");
        assert_eq!(ESPEAK_VOICE, "es");
        assert_eq!(
            espeak_argv("hola", ESPEAK_VOICE),
            vec!["-v".to_string(), "es".to_string(), "hola".to_string()]
        );
        // La voz viaja como parámetro (el pin es el default, no el único).
        assert_eq!(
            espeak_argv("hola", "es-419"),
            vec!["-v".to_string(), "es-419".to_string(), "hola".to_string()]
        );
    }

    #[test]
    fn sin_binario_habla_imposible_con_comando_instalable() {
        // Binario inexistente → `Err` con el comando copiable (sin tocar
        // el `PATH` real: el gate es el binario explícito).
        let faltante = Path::new("/nonexistent-grafito-test-dir/espeak-ng-falso");
        let error = speak_steps_text_with_bin("hola", ESPEAK_VOICE, faltante)
            .expect_err("sin binario no hay habla");
        assert!(error.contains(ESPEAK_INSTALL_CMD), "{error}");
        let error_spawn =
            spawn_speech_with_bin("hola", ESPEAK_VOICE, faltante).expect_err("sin spawn");
        assert!(error_spawn.contains(ESPEAK_INSTALL_CMD), "{error_spawn}");
        // Mensaje compartido con el toast: causa + comando + alternativa.
        let msg = espeak_missing_message();
        assert!(msg.contains(ESPEAK_INSTALL_CMD), "{msg}");
        assert_eq!(ESPEAK_INSTALL_CMD, "sudo pacman -S espeak-ng");
    }

    #[test]
    fn hablar_rechaza_vacio_y_voz_vacia() {
        let faltante = Path::new("/nonexistent-grafito-test-dir/espeak-ng-falso");
        // La API bloqueante requerida también rechaza vacío sin spawnear
        // (sin audio en tests: el `Err` llega antes del `Command`).
        assert!(speak_steps_text("", ESPEAK_VOICE).is_err());
        assert!(speak_steps_text_with_bin("", ESPEAK_VOICE, faltante).is_err());
        assert!(speak_steps_text_with_bin("   ", ESPEAK_VOICE, faltante).is_err());
        assert!(speak_steps_text_with_bin("hola", "", faltante).is_err());
        assert!(speak_steps_text_with_bin("hola", "  ", faltante).is_err());
    }

    #[test]
    fn toggle_sin_solape_matriz_completa() {
        // Gate puro (sin mock ni binario): silencio→Start, mismo→Stop,
        // otro→Switch (el handler mata antes de hablar: jamás solapa).
        assert_eq!(tts_toggle_decision(None, 0), TtsToggle::Start);
        assert_eq!(tts_toggle_decision(None, 7), TtsToggle::Start);
        assert_eq!(tts_toggle_decision(Some(3), 3), TtsToggle::Stop);
        assert_eq!(tts_toggle_decision(Some(0), 0), TtsToggle::Stop);
        assert_eq!(tts_toggle_decision(Some(2), 5), TtsToggle::Switch);
        assert_eq!(tts_toggle_decision(Some(5), 2), TtsToggle::Switch);
    }

    #[test]
    fn detecta_espeak_sin_panico() {
        // Solo presencia: en CI sin binario `false`, acá `true`.
        let _ = is_espeak_available();
    }

    /// Integración REAL contra el binario espeak-ng presente, SIN audio:
    /// solo `--version` (hablar en tests rompería el silencio del CI y
    /// sería auto-play fuera de un clic). Sin binario, degrada honesto.
    #[test]
    fn espeak_real_solo_version_sin_audio() {
        if !is_espeak_available() {
            return;
        }
        let salida = std::process::Command::new(ESPEAK_BIN)
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
            salida.to_lowercase().contains("espeak"),
            "espeak-ng --version no se reconoce a sí mismo"
        );
        // La voz pineada existe en este sistema (verificado por la tarea).
        let voces = std::process::Command::new(ESPEAK_BIN)
            .arg("--voices")
            .output()
            .map(|out| String::from_utf8_lossy(&out.stdout).into_owned())
            .unwrap_or_default();
        assert!(
            voces
                .lines()
                .any(|linea| linea.split_whitespace().nth(1) == Some("es")),
            "la voz `es` pineada falta en espeak-ng --voices"
        );
    }
}
