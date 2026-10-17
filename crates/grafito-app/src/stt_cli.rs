//! Dictado por voz del composer vía `whisper-cli` local (sin nuevas
//! dependencias cargo).
//!
//! Pulsar el mic del composer graba con `arecord` (fallback `parec` si
//! existe) y al pulsar de nuevo (o al tope) transcribe con `whisper-cli`
//! (`-l es` pineado, `--no-timestamps`) e INSERTA el texto en el composer:
//! jamás auto-envía (el usuario revisa y manda, mismo contrato que el OCR:
//! el texto transcrito es borrador, no submit).
//!
//! Solo `std::process` + binarios externos, mismo patrón que
//! [`crate::tts_cli`] y [`crate::ocr_cli`]: presencia por `PATH`, error
//! honesto con comando instalable si falta algo.
//!
//! Contrato STT (PROHIBIDO auto-enviar):
//! - `toggle`/`poll` viven en la app: el draw solo emite la intención y la
//!   app ejecuta fuera (el draw jamás spawnea ni graba).
//! - Grabar sin indicador está prohibido: la app levanta el flag
//!   `stt_recording` del panel al spawnear y lo baja al terminar (el
//!   composer muestra «Grabando…» mientras tanto).
//! - Sin hilos propios: la grabación y la transcripción viven en `Child`
//!   que la app sondea con `try_wait` por frame (no bloquea la UI);
//!   `stop`/`poll` hacen kill+wait (sin zombies) y `on_exit` los mata
//!   (nada sobrevive al cerrar).
//! - Tope [`STT_MAX_SECS`] pineado: al agotarse, la app detiene la
//!   grabación y transcribe sola (sin grabar de más en silencio).
//! - Audio temporal en `tempdir` con borrado en todos los caminos (jamás
//!   persiste: `Drop` de [`SttRecording`]/[`SttTranscribeJob`] mata y
//!   borra por si algún camino olvida cerrarlo).
//!
//! Modelo esperado (XDG, con override por env):
//! - Default: `$XDG_DATA_HOME/grafito/models/ggml-base.bin`, o
//!   `~/.local/share/grafito/models/ggml-base.bin` si no hay `XDG_DATA_HOME`.
//! - Override: `GRAFITO_WHISPER_MODEL` con la ruta al `.bin` (documentado
//!   acá y en el hover del mic apagado).

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Binario de transcripción (en `PATH`; Arch: `whisper-cpp`).
pub const WHISPER_BIN: &str = "whisper-cli";
/// Idioma pineado de la transcripción (español).
pub const WHISPER_LANG: &str = "es";
/// Nombre del modelo esperado bajo `grafito/models/`.
pub const WHISPER_MODEL_FILE: &str = "ggml-base.bin";
/// Variable de entorno para overriding la ruta del modelo.
pub const WHISPER_MODEL_ENV: &str = "GRAFITO_WHISPER_MODEL";
/// Tope de grabación en segundos (pineado: el mic jamás graba de más).
pub const STT_MAX_SECS: u64 = 30;
/// Timeout de espera de la transcripción antes de matarla (el modelo base
/// transcribe 30 s de audio en ~1 s; 60 s es margen honesto, no eternidad).
pub const STT_TIMEOUT: Duration = Duration::from_secs(60);
/// Intervalo de sondeo mientras `whisper-cli` corre (solo en la API
/// bloqueante, que corre en el job de transcripción o en tests).
#[cfg_attr(not(test), allow(dead_code))]
const STT_POLL_INTERVAL: Duration = Duration::from_millis(50);
/// Texto máximo a insertar en el composer en chars (cota del input; el
/// resto se recorta en borde de char, jamás a medias).
pub const STT_MAX_TRANSCRIPT_CHARS: usize = 2000;

/// Comando instalable copiable para el motor STT (Arch).
pub const WHISPER_INSTALL_CMD: &str = "sudo pacman -S whisper-cpp";

/// Contador para nombres únicos de WAV temporales (sin dependencias).
static STT_TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Grabador disponible en este sistema. Hoy los grabadores son Linux-only
/// (`arecord`/`parec`): en Windows el enum compila para que `SttRecording`
/// sea idéntico en ambos targets pero las variantes quedan sin construir —
/// el dictado degrada honesto a "no hay grabador" (mismo patrón que
/// `PiperMissing`).
#[cfg_attr(windows, allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecorderKind {
    /// `arecord` (ALSA).
    Arecord,
    /// `parec` (PulseAudio; fallback si no hay `arecord`).
    Parec,
}

/// `true` si hay motor STT usable: binario `whisper-cli` en `PATH` **y**
/// modelo `.bin` existente. Nunca panica.
pub fn is_whisper_available() -> bool {
    executable_in_path(WHISPER_BIN).is_some() && whisper_model_path().is_file()
}

/// `true` si hay con qué grabar (`arecord` o `parec` en `PATH`). Nunca panica.
pub fn is_recorder_available() -> bool {
    find_recorder().is_some()
}

/// `true` si hay micrófono de captura visible para `arecord -l`.
///
/// Honesto y barato: si `arecord` falta o su lista no menciona tarjetas,
/// no hay mic que ofrecer (el botón va apagado). Nunca panica; ante
/// cualquier fallo devuelve `false` (fail-closed).
pub fn is_mic_available() -> bool {
    let Ok(mut child) = Command::new("arecord")
        .arg("-l")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    else {
        return false;
    };
    let done = wait_child_bounded(&mut child, Duration::from_secs(5));
    let output = child.wait_with_output();
    let Ok(output) = output else {
        return false;
    };
    if !done {
        return false;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    text.lines().any(|line| line.contains("card"))
}

/// Espera acotada a un hijo ya spawneado: `true` si terminó antes del
/// tope (el llamador cosecha con `wait_with_output`); si se excede lo
/// mata (kill+wait, sin zombies) y devuelve `false`.
fn wait_child_bounded(child: &mut Child, timeout: Duration) -> bool {
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return true,
            Ok(None) => {
                if started.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return false;
                }
                std::thread::sleep(STT_POLL_INTERVAL);
            }
            Err(_) => return false,
        }
    }
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

/// Ruta del modelo: override por env [`WHISPER_MODEL_ENV`] si está
/// definida (aunque sea relativa: se respeta tal cual), si no XDG
/// (`$XDG_DATA_HOME/grafito/models/ggml-base.bin`, o `~/.local/share/…`
/// sin `XDG_DATA_HOME`). Pura salvo la lectura del env en el wrapper.
pub fn whisper_model_path() -> PathBuf {
    resolve_model_path(std::env::var_os(WHISPER_MODEL_ENV).as_ref())
}

/// Versión pura (testeable sin tocar el entorno del proceso).
pub fn resolve_model_path(env_override: Option<&std::ffi::OsString>) -> PathBuf {
    if let Some(custom) = env_override {
        if !custom.is_empty() {
            return PathBuf::from(custom);
        }
    }
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")))
        .unwrap_or_else(|| PathBuf::from("/tmp"));
    base.join("grafito/models").join(WHISPER_MODEL_FILE)
}

/// Mensaje honesto cuando falta motor o modelo: causa + comando
/// instalable copiable + ruta esperada del modelo. Puro, para que el
/// handler del mic y los tests lo compartan.
pub fn whisper_missing_message() -> String {
    let model = whisper_model_path();
    format!(
        "Sin dictado por voz; instalá el motor con `{WHISPER_INSTALL_CMD}` y poné el modelo en `{}` (o definí `{WHISPER_MODEL_ENV}`).",
        model.display()
    )
}

/// Mensaje honesto cuando no hay con qué grabar (ni `arecord` ni `parec`).
/// Puro.
pub fn recorder_missing_message() -> String {
    "Sin grabación de audio; instalá ALSA con `sudo pacman -S alsa-utils` o PulseAudio con `sudo pacman -S pulseaudio`.".to_string()
}

/// Localiza el grabador sin tocar el entorno (testeable con `PATH`
/// explícito).
fn find_recorder() -> Option<RecorderKind> {
    std::env::var_os("PATH").and_then(|path| find_recorder_in_path(&path))
}

/// Versión pura de [`find_recorder`] (tests herméticos, sin mutar env).
fn find_recorder_in_path(path_var: &std::ffi::OsStr) -> Option<RecorderKind> {
    for dir in std::env::split_paths(path_var) {
        if dir.as_os_str().is_empty() {
            continue;
        }
        #[cfg(windows)]
        {
            let _ = dir;
            return None;
        }
        #[cfg(not(windows))]
        {
            if dir.join("arecord").is_file() {
                return Some(RecorderKind::Arecord);
            }
        }
    }
    for dir in std::env::split_paths(path_var) {
        if dir.as_os_str().is_empty() {
            continue;
        }
        #[cfg(not(windows))]
        {
            if dir.join("parec").is_file() {
                return Some(RecorderKind::Parec);
            }
        }
    }
    None
}

/// `argv` de grabación pineado (16 kHz mono 16-bit, formato que el modelo
/// espera; `-d` = tope, jamás graba de más). Puro.
pub fn recorder_argv(kind: RecorderKind, wav_path: &Path, secs: u64) -> Vec<String> {
    let secs = secs.clamp(1, STT_MAX_SECS).to_string();
    match kind {
        RecorderKind::Arecord => vec![
            "-d".to_string(),
            secs,
            "-f".to_string(),
            "S16_LE".to_string(),
            "-r".to_string(),
            "16000".to_string(),
            "-c".to_string(),
            "1".to_string(),
            wav_path.to_string_lossy().into_owned(),
        ],
        RecorderKind::Parec => vec![
            "--format=s16le".to_string(),
            "--rate=16000".to_string(),
            "--channels=1".to_string(),
            "--file-format=wav".to_string(),
            wav_path.to_string_lossy().into_owned(),
        ],
    }
}

/// Spawnea la grabación SIN esperar (la app la sondea por frame; al tope
/// [`STT_MAX_SECS`] o a la 2ª pulsación la detiene y transcribe). Audio a
/// `wav_path` (temporal, lo borra la transcripción al terminar).
/// `Err` honesto si no hay grabador o falla el spawn.
pub fn spawn_recording(wav_path: &Path, secs: u64) -> Result<Child, String> {
    let kind = find_recorder().ok_or_else(recorder_missing_message)?;
    spawn_recording_with(kind, wav_path, secs)
}

/// Idem con grabador explícito (tests herméticos).
fn spawn_recording_with(kind: RecorderKind, wav_path: &Path, secs: u64) -> Result<Child, String> {
    let bin = match kind {
        RecorderKind::Arecord => "arecord",
        RecorderKind::Parec => "parec",
    };
    Command::new(bin)
        .args(recorder_argv(kind, wav_path, secs))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                recorder_missing_message()
            } else {
                format!("No se pudo empezar a grabar: {error}")
            }
        })
}

/// Graba `secs` segundos de forma BLOQUEANTE (job worker o test):
/// spawnea el grabador, lo espera hasta `secs + 5 s` y lo mata si se
/// excede (kill+wait, sin zombies). `Err` honesto si no hay grabador,
/// falla el spawn o el grabador sale con error.
#[cfg_attr(not(test), allow(dead_code))]
pub fn record_wav_secs(wav_path: &Path, secs: u64) -> Result<(), String> {
    let kind = find_recorder().ok_or_else(recorder_missing_message)?;
    let mut child = spawn_recording_with(kind, wav_path, secs)?;
    let ok = wait_child_bounded(
        &mut child,
        Duration::from_secs(secs.clamp(1, STT_MAX_SECS) + 5),
    );
    if !ok {
        return Err("La grabación tardó demasiado y se detuvo.".to_string());
    }
    // `wait` tras espera acotada exitosa: cosecha el zombie sin bloquear.
    let status = child
        .wait()
        .map_err(|_| "Falló la espera de la grabación.".to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "La grabación terminó con error (código {}): revisá el micrófono.",
            status.code().unwrap_or(-1)
        ))
    }
}

/// `argv` pineado de `whisper-cli` (modelo + español + WAV + sin
/// timings). Puro: el test lo pinea contra regresión de flags.
pub fn whisper_argv(model: &Path, wav_path: &Path) -> Vec<String> {
    vec![
        "-m".to_string(),
        model.to_string_lossy().into_owned(),
        "-l".to_string(),
        WHISPER_LANG.to_string(),
        "-f".to_string(),
        wav_path.to_string_lossy().into_owned(),
        "--no-timestamps".to_string(),
    ]
}

/// Parsea la salida de `whisper-cli --no-timestamps` a texto insertable:
///
/// - Se queda solo con texto: líneas de timings (`[00:00:00.000 --> …]`)
///   y vacías se caen (defensa por si algún día el flag cambia).
/// - Colapsa blancos y recorta a [`STT_MAX_TRANSCRIPT_CHARS`] en borde
///   de char. Vacío → `""` (el llamador avisa en vez de insertar nada).
pub fn parse_whisper_stdout(raw: &str) -> String {
    let mut piezas = Vec::new();
    for linea in raw.lines() {
        let linea = linea.trim();
        if linea.is_empty() {
            continue;
        }
        // Timing `[00:00:00.000 --> 00:00:02.000] texto`: jamás va al
        // composer (el dictado es solo texto). Con `--no-timestamps` no
        // debería aparecer, pero si el flag cambia no se cuela nada.
        if linea.starts_with('[') && linea.contains("-->") {
            continue;
        }
        piezas.push(linea);
    }
    let unido = piezas.join(" ");
    let colapsado = unido.split_whitespace().collect::<Vec<_>>().join(" ");
    if colapsado.chars().count() <= STT_MAX_TRANSCRIPT_CHARS {
        return colapsado;
    }
    colapsado
        .chars()
        .take(STT_MAX_TRANSCRIPT_CHARS)
        .collect::<String>()
        .trim_end()
        .to_string()
}

/// Spawnea `whisper-cli` SIN esperar (la app lo sondea por frame con
/// `try_wait`: la UI jamás se bloquea). `stdout` con pipe (la app lo lee
/// al terminar), `stderr` a null. `Err` honesto si falta binario/modelo.
pub fn spawn_transcription(wav_path: &Path) -> Result<Child, String> {
    spawn_transcription_with_bin(Path::new(WHISPER_BIN), &whisper_model_path(), wav_path)
}

/// Idem con binario y modelo explícitos (tests herméticos + distros sin
/// `PATH`).
pub fn spawn_transcription_with_bin(
    bin: &Path,
    model: &Path,
    wav_path: &Path,
) -> Result<Child, String> {
    if !model.is_file() {
        return Err(whisper_missing_message());
    }
    Command::new(bin)
        .args(whisper_argv(model, wav_path))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                whisper_missing_message()
            } else {
                format!("No se pudo iniciar la transcripción: {error}")
            }
        })
}

/// Transcribe `wav_path` de forma BLOQUEANTE (job de transcripción o
/// test): spawnea `whisper-cli`, lo espera hasta [`STT_TIMEOUT`] y lo mata
/// si se excede (kill+wait, sin zombies). Devuelve el texto parseado
/// ([`parse_whisper_stdout`]); `Err` honesto si falta motor/modelo, falla
/// el spawn/timeout, o no se entendió nada. Nunca panica.
#[cfg_attr(not(test), allow(dead_code))]
pub fn transcribe_wav(wav_path: &Path) -> Result<String, String> {
    transcribe_wav_with_bin(Path::new(WHISPER_BIN), &whisper_model_path(), wav_path)
}

/// Idem con binario y modelo explícitos (tests herméticos).
#[cfg_attr(not(test), allow(dead_code))]
pub fn transcribe_wav_with_bin(
    bin: &Path,
    model: &Path,
    wav_path: &Path,
) -> Result<String, String> {
    let mut child = spawn_transcription_with_bin(bin, model, wav_path)?;
    let started = Instant::now();
    loop {
        match child
            .try_wait()
            .map_err(|_| "Falló la espera de la transcripción.".to_string())?
        {
            Some(_) => {
                let output = child
                    .wait_with_output()
                    .map_err(|_| "No se pudo leer la salida de la transcripción.".to_string())?;
                let text = parse_whisper_stdout(&String::from_utf8_lossy(&output.stdout));
                if text.is_empty() {
                    return Err("No se entendió nada en la grabación; probá de nuevo.".to_string());
                }
                return Ok(text);
            }
            None => {
                if started.elapsed() >= STT_TIMEOUT {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err("La transcripción tardó demasiado y se canceló.".to_string());
                }
                std::thread::sleep(STT_POLL_INTERVAL);
            }
        }
    }
}

/// Ruta WAV temporal única (`std` + pid + contador atómico). El dueño la
/// borra al transcribir (ver [`SttRecording`]/[`SttTranscribeJob`]).
pub fn unique_wav_path() -> PathBuf {
    let id = STT_TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("grafito-stt-{}-{id}.wav", std::process::id()))
}

/// Borra un temporal (best-effort, jamás panica).
pub fn remove_tmp(path: &Path) {
    let _ = std::fs::remove_file(path);
}

// ── Grabación / transcripción en curso (la app las guarda; sondeo sin hilos) ──

/// Grabación en curso: el `Child` lo sondea la app por frame; al tope o a
/// la 2ª pulsación se detiene ([`SttRecording::stop`]: kill+wait + ruta
/// del WAV para transcribir). `Drop` mata y borra por si algún camino
/// olvida cerrarla (el audio jamás persiste).
pub struct SttRecording {
    /// Proceso grabador en curso (`arecord -d 30 …`).
    pub child: Child,
    /// WAV temporal que el grabador está escribiendo.
    pub wav_path: PathBuf,
    /// Arranque (para el tope aunque el sondeo por frame se pause).
    pub started: Instant,
    /// Segundos pedidos al grabador (`-d`, acotado a [`STT_MAX_SECS`]).
    pub secs: u64,
}

impl SttRecording {
    /// `true` si ya se agotó el tope (la app debe detener y transcribir).
    pub fn is_overdue(&self) -> bool {
        self.started.elapsed() >= Duration::from_secs(self.secs.clamp(1, STT_MAX_SECS))
    }

    /// Detiene la grabación: kill+wait (cosecha el zombie) y devuelve la
    /// ruta del WAV para transcribir. Consume la grabación (el `Drop`
    /// posterior ya no tiene nada que matar ni borrar).
    pub fn stop(mut self) -> PathBuf {
        // `arecord` ante SIGKILL puede dejar el header con tamaños
        // parciales; igual se transcribe lo grabado (si no se entiende
        // nada, el error es honesto, jamás silencio).
        let _ = self.child.kill();
        let _ = self.child.wait();
        // Evita que el `Drop` borre el WAV que el llamador va a usar:
        // se deja un path vacío que `remove_tmp` ignora en benigno.
        std::mem::replace(&mut self.wav_path, PathBuf::new())
    }
}

impl Drop for SttRecording {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        if !self.wav_path.as_os_str().is_empty() {
            remove_tmp(&self.wav_path);
        }
    }
}

/// Transcripción en curso: el `Child` lo sondea la app por frame
/// ([`SttTranscribeJob::poll`], no bloquea). Al terminar borra el WAV en
/// todos los caminos (éxito, fallo o timeout: jamás persiste).
pub struct SttTranscribeJob {
    /// Proceso `whisper-cli` en curso (stdout con pipe).
    pub child: Child,
    /// WAV temporal a borrar al terminar.
    pub wav_path: PathBuf,
    /// Arranque (para el timeout aunque el sondeo por frame se pause).
    pub started: Instant,
}

impl SttTranscribeJob {
    /// Sondeo no bloqueante: `None` = sigue transcribiendo;
    /// `Some(Ok(texto))` = texto listo para INSERTAR (no enviar);
    /// `Some(Err)` = aviso honesto. En todos los `Some` el WAV ya está
    /// borrado y el zombie cosechado. Nunca panica.
    pub fn poll(&mut self) -> Option<Result<String, String>> {
        match self.child.try_wait() {
            Ok(Some(_)) => {
                // `wait_with_output` consume el `Child`; acá solo hay
                // `&mut`: se drena el pipe ya cerrado (retorna al acto
                // porque el proceso terminó) y se cosecha con `wait`.
                let mut raw = Vec::new();
                if let Some(mut stdout) = self.child.stdout.take() {
                    use std::io::Read as _;
                    let _ = stdout.read_to_end(&mut raw);
                }
                let _ = self.child.wait();
                let text = parse_whisper_stdout(&String::from_utf8_lossy(&raw));
                let result = if text.is_empty() {
                    Err("No se entendió nada en la grabación; probá de nuevo.".to_string())
                } else {
                    Ok(text)
                };
                remove_tmp(&self.wav_path);
                self.wav_path = PathBuf::new();
                Some(result)
            }
            Ok(None) => {
                if self.started.elapsed() >= STT_TIMEOUT {
                    let _ = self.child.kill();
                    let _ = self.child.wait();
                    remove_tmp(&self.wav_path);
                    self.wav_path = PathBuf::new();
                    Some(Err(
                        "La transcripción tardó demasiado y se canceló.".to_string()
                    ))
                } else {
                    None
                }
            }
            Err(_) => {
                remove_tmp(&self.wav_path);
                self.wav_path = PathBuf::new();
                Some(Err("Falló la espera de la transcripción.".to_string()))
            }
        }
    }
}

impl Drop for SttTranscribeJob {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        if !self.wav_path.as_os_str().is_empty() {
            remove_tmp(&self.wav_path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    #[test]
    fn parseo_solo_texto_sin_timings() {
        // Con `--no-timestamps` llega texto plano (una línea por segmento).
        assert_eq!(parse_whisper_stdout("hola mundo\n"), "hola mundo");
        assert_eq!(
            parse_whisper_stdout("  hola   mundo  \n\nsegunda línea\n"),
            "hola mundo segunda línea"
        );
        // Defensa: si algún día el flag cambia, los timings no van al
        // composer (línea `[00:.. --> 00:..]` entera afuera).
        assert_eq!(
            parse_whisper_stdout("[00:00:00.000 --> 00:00:02.000] hola mundo\n[00:00:02.000 --> 00:00:04.000] chau\n"),
            ""
        );
        // Vacío/blancos → "" (el llamador avisa en vez de insertar nada).
        assert_eq!(parse_whisper_stdout(""), "");
        assert_eq!(parse_whisper_stdout("   \n  \n"), "");
    }

    #[test]
    fn parseo_trunca_en_borde_de_char() {
        let largo = "á".repeat(STT_MAX_TRANSCRIPT_CHARS + 40);
        let cut = parse_whisper_stdout(&largo);
        assert_eq!(cut.chars().count(), STT_MAX_TRANSCRIPT_CHARS);
        assert!(cut.is_char_boundary(cut.len()));
        assert_eq!(parse_whisper_stdout("corto"), "corto");
    }

    #[test]
    fn topes_pineados_treinta_segundos_y_dos_mil_chars() {
        // Tope de grabación pineado (el mic jamás graba de más).
        assert_eq!(STT_MAX_SECS, 30);
        assert_eq!(STT_MAX_TRANSCRIPT_CHARS, 2000);
        // `recorder_argv` acota `-d` al tope aunque pidan más.
        let wav = Path::new("/tmp/x.wav");
        let argv = recorder_argv(RecorderKind::Arecord, wav, 999);
        assert!(argv.contains(&"30".to_string()), "{argv:?}");
        assert!(!argv.contains(&"999".to_string()), "{argv:?}");
        // Formato que el modelo espera: 16 kHz mono 16-bit.
        assert!(argv.contains(&"16000".to_string()), "{argv:?}");
        assert!(argv.contains(&"S16_LE".to_string()), "{argv:?}");
    }

    #[test]
    fn comando_whisper_pineado_modelo_idioma_y_sin_timings() {
        let model = Path::new("/m/ggml-base.bin");
        let wav = Path::new("/tmp/g.wav");
        let argv = whisper_argv(model, wav);
        assert_eq!(
            argv,
            vec![
                "-m".to_string(),
                "/m/ggml-base.bin".to_string(),
                "-l".to_string(),
                "es".to_string(),
                "-f".to_string(),
                "/tmp/g.wav".to_string(),
                "--no-timestamps".to_string(),
            ]
        );
        assert_eq!(WHISPER_BIN, "whisper-cli");
        assert_eq!(WHISPER_LANG, "es");
    }

    #[test]
    fn modelo_en_xdg_con_override_por_env_documentado() {
        // Override manda (ruta tal cual, sin rebasear).
        assert_eq!(
            resolve_model_path(Some(&OsString::from("/srv/mio.bin"))),
            PathBuf::from("/srv/mio.bin")
        );
        // Vacío no vale (cae al default, jamás a ruta vacía).
        assert_ne!(resolve_model_path(Some(&OsString::new())), PathBuf::new());
        // Default documentado bajo XDG.
        assert_eq!(WHISPER_MODEL_ENV, "GRAFITO_WHISPER_MODEL");
        assert_eq!(WHISPER_MODEL_FILE, "ggml-base.bin");
        let def = resolve_model_path(None);
        assert!(
            def.ends_with("grafito/models/ggml-base.bin"),
            "ruta esperada: {def:?}"
        );
    }

    #[test]
    fn sin_binario_transcripcion_imposible_con_comando_instalable() {
        // Binario y modelo inexistentes → `Err` con el comando copiable
        // (sin tocar el `PATH` real: el gate es bin+modelo explícitos).
        let faltante = Path::new("/nonexistent-grafito-test-dir/whisper-cli-falso");
        let modelo_faltante = Path::new("/nonexistent-grafito-test-dir/ggml-base.bin");
        let wav = Path::new("/tmp/falso.wav");
        let error = spawn_transcription_with_bin(faltante, modelo_faltante, wav)
            .expect_err("sin motor/modelo no hay transcripción");
        assert!(error.contains(WHISPER_INSTALL_CMD), "{error}");
        // El mensaje compartido con el hover trae comando + ruta del modelo.
        let msg = whisper_missing_message();
        assert!(msg.contains(WHISPER_INSTALL_CMD), "{msg}");
        assert!(msg.contains("ggml-base.bin"), "{msg}");
        assert!(msg.contains(WHISPER_MODEL_ENV), "{msg}");
        assert_eq!(WHISPER_INSTALL_CMD, "sudo pacman -S whisper-cpp");
        // Sin grabador el mensaje trae su propio comando honesto.
        let rec = recorder_missing_message();
        assert!(rec.contains("alsa-utils"), "{rec}");
    }

    #[test]
    fn grabador_arecord_antes_que_parec_y_ninguno_da_none() {
        use std::os::unix::fs::PermissionsExt as _;
        let base = std::env::temp_dir().join("grafito-stt-test-rec");
        let solo_parec = base.join("parec solo");
        let _ = std::fs::remove_dir_all(&base);
        // Con ambos presentes manda `arecord` (fallback parec solo si falta).
        let ambos = base.join("ambos");
        std::fs::create_dir_all(&ambos).expect("dir de test");
        for bin in ["arecord", "parec"] {
            let path = ambos.join(bin);
            std::fs::write(&path, b"#!/bin/sh\n").expect("bin falso");
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
                .expect("permiso de test");
        }
        let path_var: OsString = OsString::from(ambos.to_string_lossy().into_owned());
        assert_eq!(
            find_recorder_in_path(&path_var),
            Some(RecorderKind::Arecord)
        );
        // Solo `parec` → fallback honesto.
        std::fs::create_dir_all(&solo_parec).expect("dir de test");
        let parec = base.join("parec solo").join("parec");
        std::fs::write(&parec, b"#!/bin/sh\n").expect("bin falso");
        std::fs::set_permissions(&parec, std::fs::Permissions::from_mode(0o755))
            .expect("permiso de test");
        let path_var = OsString::from(solo_parec.to_string_lossy().into_owned());
        assert_eq!(find_recorder_in_path(&path_var), Some(RecorderKind::Parec));
        // Vacío → `None` (sin grabador no se graba).
        assert_eq!(
            find_recorder_in_path(&OsString::from("/nonexistent-grafito-stt")),
            None
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn detecta_motor_y_grabador_sin_panico() {
        // Solo presencia: en CI sin binarios `false`, acá `true`.
        let _ = is_whisper_available();
        let _ = is_recorder_available();
        let _ = is_mic_available();
    }

    /// Integración REAL contra `whisper-cli` presente: transcribe 1 s de
    /// silencio/ambiente (sin mic puede salir vacío → `Err` honesto, sin
    /// mic se salta). El WAV temporal se borra en todos los caminos.
    #[test]
    fn whisper_real_transcribe_un_segundo_sin_colgarse() {
        if executable_in_path(WHISPER_BIN).is_none() || !whisper_model_path().is_file() {
            return;
        }
        if find_recorder().is_none() {
            return;
        }
        let wav = unique_wav_path();
        // Sin grabador funcional (CI sin mic) `record` falla honesto y el
        // test degrada sin romper (igual que el micro de espeak).
        if record_wav_secs(&wav, 1).is_err() {
            remove_tmp(&wav);
            assert!(!wav.exists(), "el temporal se borra también al fallar");
            return;
        }
        assert!(wav.is_file(), "arecord debe dejar el WAV");
        // Silencio → `Ok` con algo o `Err` honesto («no se entendió»);
        // jamás panic ni cuelgue (timeout+kill internos).
        let _ = transcribe_wav(&wav);
        remove_tmp(&wav);
        assert!(!wav.exists(), "el audio temporal jamás persiste");
    }
}
