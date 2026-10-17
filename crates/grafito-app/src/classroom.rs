//! Aula loopback F0 sin red — ClassroomPanel + ShareCode + QR real.
//!
//! Fase C1: el panel existe testeado pero no estaba cableado. Este módulo es
//! Piel pura: `ui()` no hace I/O ni spawn, solo renderiza `&Estado`.
//! El QR es real y escaneable vía `qrcode` 0.14 (`generate_qr_modules`):
//! codifica `grafito://aula/{código}`; si el código es vacío/inválido o el
//! encoder falla, el panel muestra fallback de texto honesto (sin QR falso).

use egui::{Color32, Rect, Vec2};
use grafito_ui::i18n::{t as classroom_t, Locale};
use grafito_ui::tokens::{
    PANEL_LEFT_DEFAULT, SPACE_LG, SPACE_SM, SPACE_XL, SPACE_XS, SPACE_XXL, TYPE_BASE, TYPE_SM,
    TYPE_XS,
};

/// Lado del QR del ShareCode — derivado de tokens (sin hardcodes):
/// `PANEL_LEFT_DEFAULT − SPACE_XXL − SPACE_XL − SPACE_LG` = 180.
pub const CLASSROOM_QR_SIDE: f32 = PANEL_LEFT_DEFAULT - SPACE_XXL - SPACE_XL - SPACE_LG;

/// Código para compartir en lobby/live. Validación newtype.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShareCode(pub String);

impl ShareCode {
    /// Crea un ShareCode validando caracteres permitidos y longitud 1..=32.
    /// Solo alfanumérico, '-' y '_' (compatible con URL/query).
    ///
    /// El tope es 32 — no 64 — a propósito: el núcleo (`ClassroomCode` en
    /// `grafito-classroom/src/session.rs`) solo acepta `1..=32`, y un código
    /// UI de 33..=64 era `InvalidCode` al cruzar a sesión. Validar el cruce
    /// acá evita un QR que la sesión rechaza.
    pub fn new(code: impl Into<String>) -> Option<Self> {
        let s = code.into();
        if s.is_empty() || s.len() > 32 {
            return None;
        }
        if !s
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return None;
        }
        Some(Self(s))
    }

    /// Genera un código loopback para F0 sin red. Determinista por proceso+tiempo,
    /// sin I/O. No usa `rand` para evitar dep extra.
    pub fn generate() -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let pid = std::process::id() as u128;
        let mut seed = nanos ^ (pid.rotate_left(7).wrapping_add(0x9e3779b97f4a7c15));
        if seed == 0 {
            seed = 0x9e3779b97f4a7c15;
        }
        const ALPH: &[u8] = b"ABCDEFGHJKMNPQRSTVWXYZ23456789";
        let mut out = String::with_capacity(10);
        out.push_str("AULA-");
        let mut v = seed;
        for _ in 0..6 {
            let idx = (v % ALPH.len() as u128) as usize;
            out.push(ALPH[idx] as char);
            v = v.wrapping_div(ALPH.len() as u128);
            if v == 0 {
                v = seed.wrapping_add(0x9e3779b97f4a7c15);
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            }
        }
        // SAFETY: construcción garantiza alfanumérico + '-' y len 11
        Self(out)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Rol local de UX en el Aula (preferencia de sesión, sin cuentas ni red).
///
/// Decisión de default: `Unset` ("Sin rol") = el comportamiento actual,
/// sin forzar elección. El selector invita a elegir; sin rol no se muestra
/// ninguna sección de rol y el panel queda intacto. El rol vive solo en
/// memoria (`ClassroomPanel`): no se persiste en `AppConfig`/disco a
/// propósito (alcance cerrado: sin preferencia natural previa, sesión sola).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClassroomRole {
    /// Sin rol elegido: comportamiento previo al cambio (default).
    #[default]
    Unset,
    /// Profesor: Presentar + exportar PNG/SVG.
    Teacher,
    /// Alumno: Preguntar + Seguir.
    Student,
}

/// Modo de foco del Aula (canvas a pantalla completa, sesión local).
///
/// Ambos ocultan los paneles del shell y restauran con Esc. `Present` es
/// del Profesor (con cortina + exportes a mano); `Attend` es del Alumno
/// (modo atención, solo canvas). Ninguno toca el lockdown de examen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassroomFocus {
    /// Profesor presentando: overlay con cortina + PNG/SVG + salir.
    Present,
    /// Alumno siguiendo: overlay mínimo (modo atención) + salir.
    Attend,
}

/// Foto de los paneles visibles antes de entrar en foco (para restaurar
/// con Esc exactamente lo que había: drawers + asistente + teclado).
/// Pura y `Copy`: el snapshot entra por valor y sale por valor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ClassroomShellSnapshot {
    pub left_drawer_open: bool,
    pub compact_drawer_open: bool,
    pub right_drawer_open: bool,
    pub assistant_visible: bool,
    pub keyboard_visible: bool,
}

/// Estado del panel Aula (F0 loopback sin red).
#[derive(Debug, Clone)]
pub struct ClassroomPanel {
    opt_in: bool,
    share_code: Option<ShareCode>,
    is_host: bool,
    /// Rol de UX elegido (sesión sola, default [`ClassroomRole::Unset`]).
    role: ClassroomRole,
    /// Modo de foco activo, si lo hay (Presentar/Seguir).
    focus: Option<ClassroomFocus>,
    /// Foto pre-foco para restaurar con Esc (solo con `focus.is_some()`).
    saved_shell: Option<ClassroomShellSnapshot>,
    /// Outbox offline persistida en disco (D2): se carga LAZY al abrir el
    /// Aula (`ensure_outbox_loaded`, llamada desde `draw_classroom_panel`)
    /// y se guarda en background (`persist_outbox_background`). Antes se
    /// leía en cada arranque aunque Aula jamás se abriera (I/O sync + parse
    /// de hasta 320 KiB en `new`). Vacía hasta la primera apertura.
    outbox: grafito_classroom::OfflineOutbox,
    /// Aviso honesto de carga corrupta (se muestra una vez en el panel).
    outbox_notice: Option<String>,
    /// La outbox ya se leyó del disco en esta sesión (una sola vez).
    outbox_loaded: bool,
    /// Idioma para el aviso de carga (se guarda en `new`, se usa al cargar).
    outbox_locale: Locale,
    /// Borrador del chat offline (UI state, acotado a 2048 en `set_chat_draft`).
    chat_draft: String,
    /// Aviso del último envío ("se enviará al reconectar…" o error honesto).
    chat_notice: Option<String>,
}

impl Default for ClassroomPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl ClassroomPanel {
    pub fn new() -> Self {
        Self::new_with_locale(Locale::Es)
    }

    /// Variante localizada de [`Self::new`] (B-A201, patrón B-A76):
    /// `new()` queda como wrapper Es intacto (tests); el arranque real
    /// usa `load_outbox_for(locale)` para el aviso de display.
    pub fn new_with_locale(locale: Locale) -> Self {
        // Sin I/O: la outbox se lee al abrir el Aula (ver
        // `ensure_outbox_loaded`). El arranque no paga disco por un panel
        // que casi nadie abre en la primera sesión.
        Self {
            opt_in: false,
            share_code: None,
            is_host: false,
            outbox: grafito_classroom::OfflineOutbox::default(),
            outbox_notice: None,
            outbox_loaded: false,
            outbox_locale: locale,
            chat_draft: String::new(),
            chat_notice: None,
            role: ClassroomRole::Unset,
            focus: None,
            saved_shell: None,
        }
    }

    /// Rol de UX elegido (default [`ClassroomRole::Unset`]: sin forzar).
    pub fn role(&self) -> ClassroomRole {
        self.role
    }

    /// Fija el rol (sesión sola: no persiste en disco, ver [`ClassroomRole`]).
    pub fn set_role(&mut self, role: ClassroomRole) {
        self.role = role;
    }

    /// Modo de foco activo (`None` = canvas normal).
    pub fn focus_mode(&self) -> Option<ClassroomFocus> {
        self.focus
    }

    /// ¿Hay foco activo (Presentar o Seguir)?
    pub fn in_focus(&self) -> bool {
        self.focus.is_some()
    }

    /// Entra en foco guardando la foto del shell. Si ya había foco, solo
    /// cambia el modo y conserva la foto ORIGINAL (Esc restaura siempre al
    /// estado previo al primer `enter_focus`, no al intermedio).
    pub fn enter_focus(&mut self, mode: ClassroomFocus, snapshot: ClassroomShellSnapshot) {
        if self.focus.is_none() {
            self.saved_shell = Some(snapshot);
        }
        self.focus = Some(mode);
    }

    /// Sale del foco devolviendo la foto para restaurar (o `None` si no
    /// había foco). Limpia el modo y la foto: doble salida es no-op.
    pub fn exit_focus(&mut self) -> Option<ClassroomShellSnapshot> {
        self.focus = None;
        self.saved_shell.take()
    }

    /// Lee la outbox del disco una sola vez (llamar al abrir el Aula).
    /// Idempotente: tras la primera lectura es no-op.
    pub fn ensure_outbox_loaded(&mut self) {
        if self.outbox_loaded {
            return;
        }
        self.outbox_loaded = true;
        let (outbox, notice) = crate::utils::load_outbox_for(self.outbox_locale);
        if let Some(aviso) = &notice {
            log::warn!("Outbox offline: {aviso}");
        }
        self.outbox = outbox;
        self.outbox_notice = notice;
    }

    /// Sincroniza el opt-in desde `GrafitoApp.advanced_red_opt_in` (llamado por frame).
    pub fn set_opt_in(&mut self, opt: bool) {
        self.opt_in = opt;
        if opt && self.share_code.is_none() {
            self.share_code = Some(ShareCode::generate());
        }
    }

    pub fn is_opt_in(&self) -> bool {
        self.opt_in
    }

    /// ¿Debe mostrarse el tab "Aula" en el sidebar? Solo si opt-in.
    pub fn should_show_aula_tab(&self) -> bool {
        self.opt_in
    }

    pub fn share_code(&self) -> Option<&ShareCode> {
        self.share_code.as_ref()
    }

    pub fn set_share_code(&mut self, code: ShareCode) {
        self.share_code = Some(code);
    }

    pub fn set_host(&mut self, host: bool) {
        self.is_host = host;
    }

    pub fn is_host(&self) -> bool {
        self.is_host
    }

    /// Outbox offline (lectura para la UI; la mutación futura persiste aparte).
    pub fn outbox(&self) -> &grafito_classroom::OfflineOutbox {
        &self.outbox
    }

    /// Pendientes en la outbox (visible en el panel, P1b).
    pub fn pending_count(&self) -> usize {
        self.outbox.len()
    }

    /// Borrador del chat (lo edita el TextEdit del panel).
    pub fn chat_draft(&self) -> &str {
        &self.chat_draft
    }

    /// Fija el borrador acotando a 2048 bytes (presupuesto `MAX_OFFLINE_BODY_BYTES`).
    pub fn set_chat_draft(&mut self, draft: impl Into<String>) {
        let mut text = draft.into();
        while text.len() > grafito_classroom::MAX_OFFLINE_BODY_BYTES {
            text.pop();
        }
        self.chat_draft = text;
    }

    /// Aviso del último envío del chat, si hubo.
    pub fn chat_notice(&self) -> Option<&str> {
        self.chat_notice.as_deref()
    }

    /// Reloj del caller para la outbox (secs desde UNIX_EPOCH, 0 si falla).
    fn outbox_now_secs() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
}

/// Etiqueta del rol de UX en el Aula, por locale (patrón B-A199/B-A98).
/// Sin template: cada rol resuelve su clave (`role_teacher`/`role_student`/
/// `role_none`) en los 6 idiomas.
pub fn classroom_role_label_for(role: ClassroomRole, locale: Locale) -> String {
    let key = match role {
        ClassroomRole::Teacher => "classroom.role_teacher",
        ClassroomRole::Student => "classroom.role_student",
        ClassroomRole::Unset => "classroom.role_none",
    };
    classroom_t(key, locale).to_string()
}

/// Título de la sala con código, por locale. Es idéntico al canónico
/// previo con `{code}` (B-A199, patrón B-A98).
pub fn classroom_room_code_for(code: &str, locale: Locale) -> String {
    classroom_t("classroom.room_code", locale).replace("{code}", code)
}

/// Estado de la cola offline, por locale. Es idéntico al canónico previo
/// con `{n}` (B-A199).
pub fn classroom_queue_status_for(pending: usize, locale: Locale) -> String {
    classroom_t("classroom.queue_status", locale).replace("{n}", &pending.to_string())
}

/// Aviso de mensaje encolado, por locale. Es idéntico al canónico previo
/// con `{id}` y `{n}` (B-A199).
pub fn classroom_chat_queued_for(id: u64, pending: usize, locale: Locale) -> String {
    classroom_t("classroom.chat_queued", locale)
        .replace("{id}", &id.to_string())
        .replace("{n}", &pending.to_string())
}

/// Aviso de fallo de enqueue, por locale. Es idéntico al canónico previo
/// con `{motivo}` (B-A199).
pub fn classroom_chat_failed_for(motivo: &str, locale: Locale) -> String {
    classroom_t("classroom.chat_failed", locale).replace("{motivo}", motivo)
}

/// Aviso de outbox corrupta, por locale. Es idéntico al canónico
/// `offline.rs:302` (B-A201, patrón B-A98).
pub fn classroom_outbox_corrupt_for(locale: Locale) -> String {
    classroom_t("classroom.outbox_corrupt", locale).to_string()
}

/// Aviso de mensajes descartados al cargar, por locale. Es idéntico al
/// canónico `offline.rs:307` con `{n}` (B-A201).
pub fn classroom_outbox_discarded_for(n: usize, locale: Locale) -> String {
    classroom_t("classroom.outbox_discarded", locale).replace("{n}", &n.to_string())
}

/// Aviso de lectura fallida de la outbox, por locale. Es idéntico al
/// canónico `utils.rs:429` (B-A201).
pub fn classroom_outbox_read_error_for(locale: Locale) -> String {
    classroom_t("classroom.outbox_read_error", locale).to_string()
}

impl ClassroomPanel {
    /// Productor real P1b: encola un mensaje de chat offline.
    ///
    /// F0 sin red → todo envío va a la outbox con aviso
    /// "se enviará al reconectar". Vacío → `Err` honesto sin encolar.
    /// Llena (128) o cuerpo inválido → `Err` honesto con motivo.
    /// En éxito limpia el borrador, avisa y persiste en background
    /// (I/O fuera del hilo UI vía `persist_outbox_background`).
    /// B-A199: `locale` solo localiza los `chat_notice` de display;
    /// los `Err` quedan canónicos ES a propósito (contenido motor,
    /// precedente B-A184 `ggbscript.rs`).
    pub fn enqueue_chat_message(&mut self, body: &str, locale: Locale) -> Result<u64, String> {
        let trimmed = body.trim();
        if trimmed.is_empty() {
            self.chat_notice = Some(classroom_t("classroom.chat_empty", locale).to_string());
            return Err("el mensaje no debe estar vacío".to_string());
        }
        let now = Self::outbox_now_secs();
        match self.outbox.enqueue("chat", trimmed, now) {
            Ok(id) => {
                self.chat_draft.clear();
                self.chat_notice = Some(classroom_chat_queued_for(id, self.outbox.len(), locale));
                self.persist_outbox_background();
                Ok(id)
            }
            Err(grafito_classroom::ClassroomError::QueueFull) => {
                self.chat_notice = Some(classroom_t("classroom.chat_full", locale).to_string());
                Err("cola offline llena (128)".to_string())
            }
            Err(err) => {
                let motivo = format!("{err:?}");
                self.chat_notice = Some(classroom_chat_failed_for(&motivo, locale));
                Err(motivo)
            }
        }
    }

    /// Aviso honesto de carga (corrupción/descarte), si hubo.
    pub fn outbox_notice(&self) -> Option<&str> {
        self.outbox_notice.as_deref()
    }

    /// Consume el aviso de carga (para mostrarlo una sola vez como toast).
    pub fn take_outbox_notice(&mut self) -> Option<String> {
        self.outbox_notice.take()
    }

    /// Guarda la outbox en disco en background (cero bloqueo del UI thread).
    ///
    /// Clona el estado y escribe en un hilo `grafito-outbox`; si falla,
    /// solo log (la memoria sigue siendo la fuente de verdad).
    pub fn persist_outbox_background(&self) {
        let snapshot = self.outbox.clone();
        let path = crate::utils::outbox_path();
        let _ = std::thread::Builder::new()
            .name("grafito-outbox".to_string())
            .spawn(move || {
                if let Err(motivo) = crate::utils::save_outbox_to_path(&snapshot, &path) {
                    log::warn!("No se pudo persistir la outbox offline: {motivo}");
                }
            });
    }

    /// Render puro del contenido del panel (sin I/O, sin spawn). Llamado desde
    /// `draw_classroom_panel` que crea el SidePanel izquierdo.
    /// B-A199: `locale` solo display (patrón B-A41/B-A52); el payload
    /// `grafito://aula` y los avisos del crate quedan intactos.
    pub fn ui(&self, ui: &mut egui::Ui, locale: Locale) {
        ui.add_space(SPACE_SM);
        ui.label(
            egui::RichText::new(classroom_t("classroom.panel_title", locale))
                .size(TYPE_BASE)
                .strong(),
        );
        ui.add_space(SPACE_SM);
        ui.separator();
        ui.add_space(SPACE_SM);
        if !self.opt_in {
            ui.label(
                egui::RichText::new(classroom_t("classroom.disabled_msg", locale))
                    .size(TYPE_XS)
                    .color(ui.visuals().weak_text_color()),
            );
            ui.add_space(SPACE_SM);
            ui.label(
                egui::RichText::new(classroom_t("classroom.optin_hint", locale))
                    .size(TYPE_XS)
                    .weak(),
            );
            return;
        }
        if let Some(code) = &self.share_code {
            ui.label(
                egui::RichText::new(classroom_room_code_for(code.as_str(), locale))
                    .size(TYPE_SM)
                    .strong(),
            );
            ui.add_space(SPACE_XS);
            ui.label(
                egui::RichText::new(classroom_t("classroom.qr_hint", locale))
                    .size(TYPE_XS)
                    .color(ui.visuals().weak_text_color()),
            );
            ui.add_space(SPACE_SM);
            draw_share_qr(ui, code.as_str(), CLASSROOM_QR_SIDE, locale);
            ui.add_space(SPACE_SM);
            ui.label(
                egui::RichText::new(format!("grafito://aula/{}", code.as_str()))
                    .size(TYPE_XS)
                    .monospace()
                    .color(ui.visuals().weak_text_color()),
            );
            ui.add_space(SPACE_SM);
            if self.is_host {
                ui.label(
                    egui::RichText::new(classroom_t("classroom.host_mode", locale))
                        .size(TYPE_XS)
                        .color(grafito_ui::theme::current_theme(ui.ctx()).success),
                );
            } else {
                ui.label(
                    egui::RichText::new(classroom_t("classroom.guest_mode", locale))
                        .size(TYPE_XS)
                        .weak(),
                );
            }
        } else {
            ui.label(classroom_t("classroom.generating", locale));
        }
        ui.add_space(SPACE_SM);
        ui.separator();
        ui.add_space(SPACE_XS);
        ui.label(
            egui::RichText::new(classroom_t("classroom.offline_note", locale))
                .size(TYPE_XS)
                .weak(),
        );
        // P1b: outbox visible — pendientes + último aviso (carga o envío).
        ui.add_space(SPACE_SM);
        ui.separator();
        ui.add_space(SPACE_XS);
        ui.label(
            egui::RichText::new(classroom_queue_status_for(self.outbox.len(), locale))
                .size(TYPE_XS)
                .weak(),
        );
        if let Some(aviso) = self.outbox_notice.as_deref() {
            ui.label(
                egui::RichText::new(aviso)
                    .size(TYPE_XS)
                    .color(ui.visuals().weak_text_color()),
            );
        }
        if let Some(aviso) = self.chat_notice.as_deref() {
            ui.label(egui::RichText::new(aviso).size(TYPE_XS).weak());
        }
        ui.add_space(SPACE_XS);
        ui.label(
            egui::RichText::new(classroom_t("classroom.chat_hint", locale))
                .size(TYPE_XS)
                .weak(),
        );
    }
}

/// Payload escaneable del QR para un código de sala. `None` honesto si el
/// código es vacío o inválido (`ShareCode::new` lo rechaza: longitud 1..=32,
/// sólo ASCII alfanumérico + `-`/`_`).
pub fn qr_payload_for_code(code: &str) -> Option<String> {
    ShareCode::new(code)?;
    Some(format!("grafito://aula/{code}"))
}

/// Genera la grid booleana de un QR REAL con `qrcode` 0.14 (core, sin
/// features de imagen/svg: sólo `QrCode::new` + `into_colors`).
///
/// Codifica `grafito://aula/{código}`. Retorna `None` honesto con código
/// vacío/inválido o si el encoder falla (p. ej. payload excede la capacidad
/// QR). Grid cuadrada de lado `width()` (≥21 según versión), `true` = módulo
/// oscuro. Pura, sin I/O, testeable.
pub fn generate_qr_modules(code: &str) -> Option<Vec<Vec<bool>>> {
    let payload = qr_payload_for_code(code)?;
    let qr = qrcode::QrCode::new(payload.as_bytes()).ok()?;
    let width = qr.width();
    if width == 0 {
        return None;
    }
    let colors = qr.into_colors();
    if colors.len() != width.saturating_mul(width) {
        return None;
    }
    let mut grid = Vec::with_capacity(width);
    for row in colors.chunks(width) {
        grid.push(
            row.iter()
                .map(|color| *color == qrcode::Color::Dark)
                .collect(),
        );
    }
    Some(grid)
}

/// Dibuja el QR REAL del `code` con rects egui. Si `generate_qr_modules`
/// falla (código vacío/inválido o encoder sin capacidad), fallback honesto
/// de texto en vez de un QR falso que no escanea.
/// B-A199: fallback por `classroom.qr_unavailable` con `locale`.
pub fn draw_share_qr(ui: &mut egui::Ui, code: &str, size: f32, locale: Locale) {
    let Some(grid) = generate_qr_modules(code) else {
        ui.label(
            egui::RichText::new(classroom_t("classroom.qr_unavailable", locale))
                .size(TYPE_XS)
                .color(ui.visuals().weak_text_color()),
        );
        return;
    };
    let n = grid.len() as f32;
    let cell = size / n.max(1.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, SPACE_XS, Color32::WHITE);
    painter.rect_stroke(
        rect,
        SPACE_XS,
        egui::Stroke::new(1.0, Color32::from_gray(200)),
    );
    for (y, row) in grid.iter().enumerate() {
        for (x, &filled) in row.iter().enumerate() {
            if !filled {
                continue;
            }
            let r = Rect::from_min_size(
                egui::pos2(rect.min.x + x as f32 * cell, rect.min.y + y as f32 * cell),
                Vec2::splat(cell),
            );
            painter.rect_filled(r, 0.0, Color32::BLACK);
        }
    }
    // Borde fino alrededor del QR para contraste
    painter.rect_stroke(rect, SPACE_XS, egui::Stroke::new(1.0, Color32::BLACK));
}

/// Entry point para `GrafitoApp` (left drawer). Crea SidePanel y delega a `ClassroomPanel::ui`.
///
/// Piel pura: dentro del `show` (Ui::) solo se renderiza y se REGISTRAN
/// acciones; la mutación + persistencia (`save_app_config`, I/O) se aplica
/// DESPUÉS del cierre. El sync `set_opt_in` vive por frame en `update`
/// (`app.rs`), nunca en Ui::.
/// Fracción del viewport para el bottom-sheet de Aula en Nivel 0.
/// 55 % deja media pantalla de canvas visible (compromiso M3: la hoja
/// muestra el contenido sin tapar el trabajo).
pub const AULA_SHEET_FRACTION: f32 = 0.55;

/// Altura del sheet de Aula. Pura y pineada.
pub fn aula_sheet_height(viewport_height: f32) -> f32 {
    (viewport_height * AULA_SHEET_FRACTION).max(0.0)
}

/// Intenciones recogidas por los contenidos (aplicación fuera de `Ui::`).
#[derive(Default)]
struct ClassroomOutcome {
    opt_in_toggled: Option<bool>,
    regenerate_code: bool,
    toggle_host: bool,
    send_chat: bool,
    set_role: Option<ClassroomRole>,
    enter_present: bool,
    enter_attend: bool,
    ask_assistant: bool,
    export_png: bool,
    export_svg: bool,
}

/// Sección de acciones según el rol elegido (registra intenciones, sin I/O).
///
/// - Profesor: "Presentar" (canvas fullscreen + cortina + puntero visible,
///   Esc restaura) + atajos a exportar PNG/SVG (reusan `export_with_dialog`,
///   con su lockdown de examen intacto).
/// - Alumno: "Preguntar" (abre el asistente con foco + saludo de Mili si
///   el transcript está vacío, mismo camino que el onboarding) + "Seguir"
///   (modo atención: solo canvas, Esc restaura).
/// - `Unset`: nada (comportamiento previo, sin forzar elección).
fn draw_classroom_role_section(
    ui: &mut egui::Ui,
    aula_locale: Locale,
    role: ClassroomRole,
    out: &mut ClassroomOutcome,
) {
    match role {
        ClassroomRole::Unset => {}
        ClassroomRole::Teacher => {
            ui.add_space(SPACE_XS);
            if ui
                .button(grafito_ui::i18n::t("classroom.present", aula_locale))
                .clicked()
            {
                out.enter_present = true;
            }
            // Reuso `palette.action.export_*` (0 claves nuevas para exportar).
            let export_svg = grafito_ui::i18n::t("palette.action.export_svg", aula_locale);
            let export_png = grafito_ui::i18n::t("palette.action.export_png", aula_locale);
            ui.horizontal_wrapped(|ui| {
                if ui.button(export_svg).clicked() {
                    out.export_svg = true;
                }
                if ui.button(export_png).clicked() {
                    out.export_png = true;
                }
            });
        }
        ClassroomRole::Student => {
            ui.add_space(SPACE_XS);
            ui.horizontal_wrapped(|ui| {
                if ui
                    .button(grafito_ui::i18n::t("classroom.ask", aula_locale))
                    .clicked()
                {
                    out.ask_assistant = true;
                }
                if ui
                    .button(grafito_ui::i18n::t("classroom.follow", aula_locale))
                    .clicked()
                {
                    out.enter_attend = true;
                }
            });
        }
    }
}

/// Contenidos del panel Aula, independientes del contenedor (S8): el mismo
/// cuerpo vive en el `SidePanel` desktop y en el bottom-sheet móvil.
fn draw_classroom_contents(
    ui: &mut egui::Ui,
    app: &mut crate::app::GrafitoApp,
    aula_locale: Locale,
    panel_was_opt_in: bool,
    opt_in: &mut bool,
    out: &mut ClassroomOutcome,
) {
    // Toggle opt-in: solo registra la intención (Piel pura, sin I/O).
    // Switch a la derecha (S6b): el header "Aula" queda como título y el
    // "Activar" es el rótulo del switch (antes checkbox en fila horizontal).
    ui.label(
        egui::RichText::new(grafito_ui::i18n::t("rail.tab_aula", aula_locale))
            .size(TYPE_SM)
            .strong(),
    );
    if grafito_ui::assistant::drill_switch(
        ui,
        grafito_ui::i18n::t("classroom.opt_in_toggle", aula_locale),
        "",
        opt_in,
        true,
    ) {
        out.opt_in_toggled = Some(*opt_in);
    }
    ui.separator();
    app.classroom.ui(ui, aula_locale);
    ui.add_space(SPACE_SM);
    if panel_was_opt_in {
        // Roles Profesor/Alumno (preferencia local de UX, sesión sola): el
        // selector no fuerza elección — `Unset` deja el panel como estaba.
        // Piel pura: los radios mutan un local y solo REGISTRAN `set_role`.
        ui.label(
            egui::RichText::new(grafito_ui::i18n::t("classroom.role_label", aula_locale))
                .size(TYPE_SM)
                .strong(),
        );
        let mut role = app.classroom.role();
        ui.horizontal_wrapped(|ui| {
            ui.radio_value(
                &mut role,
                ClassroomRole::Teacher,
                classroom_role_label_for(ClassroomRole::Teacher, aula_locale),
            );
            ui.radio_value(
                &mut role,
                ClassroomRole::Student,
                classroom_role_label_for(ClassroomRole::Student, aula_locale),
            );
            ui.radio_value(
                &mut role,
                ClassroomRole::Unset,
                classroom_role_label_for(ClassroomRole::Unset, aula_locale),
            );
        });
        if role != app.classroom.role() {
            out.set_role = Some(role);
        }
        draw_classroom_role_section(ui, aula_locale, role, out);
        ui.horizontal(|ui| {
            if ui
                .button(grafito_ui::i18n::t("classroom.regen_code", aula_locale))
                .clicked()
            {
                out.regenerate_code = true;
            }
            if ui
                .button(grafito_ui::i18n::t("classroom.toggle_host", aula_locale))
                .clicked()
            {
                out.toggle_host = true;
            }
        });
        // P1b: chat offline — el TextEdit muta solo el borrador (UI
        // state, sin I/O); el enqueue + persist van FUERA de Ui::.
        ui.add_space(SPACE_SM);
        ui.separator();
        ui.add_space(SPACE_XS);
        ui.label(
            egui::RichText::new(grafito_ui::i18n::t("classroom.chat_title", aula_locale))
                .size(TYPE_SM)
                .strong(),
        );
        ui.horizontal(|ui| {
            let draft = &mut app.classroom.chat_draft;
            let resp = ui.text_edit_singleline(draft);
            if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                out.send_chat = true;
            }
            if ui
                .button(grafito_ui::i18n::t("classroom.chat_send", aula_locale))
                .clicked()
            {
                out.send_chat = true;
            }
        });
    }
}

/// Aplica las intenciones FUERA de `Ui::` (contexto `update`, sin layout).
/// `ctx` solo viaja a los exportes (diálogo nativo, mismo camino que el
/// menú Archivo → Exportar, con su `exam_blocks` intacto).
fn apply_classroom_outcome(
    app: &mut crate::app::GrafitoApp,
    aula_locale: Locale,
    ctx: &egui::Context,
    out: ClassroomOutcome,
) {
    if let Some(opt_in) = out.opt_in_toggled {
        app.advanced_red_opt_in = opt_in;
        app.classroom.set_opt_in(opt_in);
        app.save_app_config();
    }
    if out.regenerate_code {
        app.classroom.set_share_code(ShareCode::generate());
    }
    if out.toggle_host {
        let cur = app.classroom.is_host();
        app.classroom.set_host(!cur);
    }
    if out.send_chat {
        let body = app.classroom.chat_draft.clone();
        let _ = app.classroom.enqueue_chat_message(&body, aula_locale);
    }
    if let Some(role) = out.set_role {
        // Sesión sola a propósito: el rol NO viaja a `save_app_config`
        // (alcance cerrado: preferencia local de UX, sin persistencia).
        app.classroom.set_role(role);
    }
    if out.enter_present {
        enter_classroom_focus(app, ClassroomFocus::Present);
    }
    if out.enter_attend {
        enter_classroom_focus(app, ClassroomFocus::Attend);
    }
    if out.ask_assistant {
        ask_assistant_from_classroom(app);
    }
    if out.export_svg {
        app.export_with_dialog(crate::export::ExportFormat::Svg, Some(ctx));
    }
    if out.export_png {
        app.export_with_dialog(crate::export::ExportFormat::Png, Some(ctx));
    }
}

/// Entra en foco del Aula: foto del shell + ocultar paneles (solo canvas).
///
/// No toca el documento, la cortina ni el lockdown de examen: solo esconde
/// drawers + asistente + teclado. El puntero queda visible (egui no lo
/// oculta; este camino jamás pide `CursorIcon::None`).
pub fn enter_classroom_focus(app: &mut crate::app::GrafitoApp, mode: ClassroomFocus) {
    let snapshot = ClassroomShellSnapshot {
        left_drawer_open: app.left_drawer_open,
        compact_drawer_open: app.compact_drawer_open,
        right_drawer_open: app.right_drawer_open,
        assistant_visible: app.assistant_visible,
        keyboard_visible: app.keyboard_visible,
    };
    app.classroom.enter_focus(mode, snapshot);
    app.left_drawer_open = false;
    app.compact_drawer_open = false;
    app.right_drawer_open = false;
    app.assistant_visible = false;
    app.keyboard_visible = false;
}

/// Sale del foco restaurando la foto pre-foco (Esc o botón Salir).
/// No-op sin foco activo.
pub fn exit_classroom_focus(app: &mut crate::app::GrafitoApp) {
    if let Some(foto) = app.classroom.exit_focus() {
        app.left_drawer_open = foto.left_drawer_open;
        app.compact_drawer_open = foto.compact_drawer_open;
        app.right_drawer_open = foto.right_drawer_open;
        app.assistant_visible = foto.assistant_visible;
        app.keyboard_visible = foto.keyboard_visible;
    }
}

/// Alumno → "Preguntar": abre el asistente con foco + saludo de Mili si
/// el transcript está vacío.
///
/// Reusa el camino del onboarding (`onboarding_try_example` en `app.rs`:
/// mismo flag `assistant_visible`, misma clave `assistant.greeting.fallback`)
/// menos la siembra del ejemplo (el alumno pregunta sobre SU canvas, no
/// sobre `y = x^2`). El foco viaja en `composer_focus_request` (one-shot
/// que el composer consume al dibujarse). Sesión local, sin red.
pub fn ask_assistant_from_classroom(app: &mut crate::app::GrafitoApp) {
    app.assistant_visible = true;
    app.assistant.composer_focus_request = true;
    if app.assistant.conversation.is_empty() {
        let locale = app.config_locale();
        let saludo = grafito_ui::i18n::t("assistant.greeting.fallback", locale).to_string();
        app.assistant
            .push_turn(grafito_assistant_types::ConversationTurn::assistant(saludo));
    }
}

/// Frame del foco del Aula: Esc restaura + overlay mínimo con salidas.
///
/// Llamar DESPUÉS del Esc de examen en `update` (`app.rs`): el lockdown
/// conserva prioridad (en examen, Esc pide confirmación de salida primero
/// y el foco sigue). Sin foco es no-op.
///
/// Overlay: barra flotante arriba-centro (`Foreground`, no roba canvas).
/// Presentar trae cortina (toggle real de la pizarra, solo si está abierta),
/// PNG/SVG (mismo camino con lockdown) y Salir; Seguir trae solo Salir.
/// Piel pura: los botones solo REGISTRAN y se aplican fuera del `show`.
pub fn classroom_focus_frame(app: &mut crate::app::GrafitoApp, ctx: &egui::Context) {
    let Some(mode) = app.classroom.focus_mode() else {
        return;
    };
    if ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
        exit_classroom_focus(app);
        return;
    }
    // B-A197: `locale` + flags pre-borrow (el closure de `show` muta `app`).
    let locale = app.config_locale();
    let whiteboard_open = app.whiteboard_open;
    let mut do_exit = false;
    let mut do_curtain = false;
    let mut do_png = false;
    let mut do_svg = false;
    let titulo = match mode {
        ClassroomFocus::Present => classroom_t("classroom.presenting", locale),
        ClassroomFocus::Attend => classroom_t("classroom.attending", locale),
    };
    egui::Area::new(egui::Id::new("classroom_focus_bar"))
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 8.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(titulo).size(TYPE_SM).strong());
                    if mode == ClassroomFocus::Present {
                        // Cortina disponible presentando: toggle real del slot
                        // único de la pizarra (no se inventa cortina en el
                        // canvas principal; sin pizarra abierta no hay botón).
                        if whiteboard_open
                            && ui
                                .button(classroom_t("classroom.curtain", locale))
                                .clicked()
                        {
                            do_curtain = true;
                        }
                        if ui
                            .button(grafito_ui::i18n::t("palette.action.export_svg", locale))
                            .clicked()
                        {
                            do_svg = true;
                        }
                        if ui
                            .button(grafito_ui::i18n::t("palette.action.export_png", locale))
                            .clicked()
                        {
                            do_png = true;
                        }
                    }
                    if ui
                        .button(classroom_t("classroom.exit_focus", locale))
                        .clicked()
                    {
                        do_exit = true;
                    }
                });
            });
        });
    if do_curtain {
        app.whiteboard.toggle_curtain();
    }
    if do_svg {
        app.export_with_dialog(crate::export::ExportFormat::Svg, Some(ctx));
    }
    if do_png {
        app.export_with_dialog(crate::export::ExportFormat::Png, Some(ctx));
    }
    if do_exit {
        exit_classroom_focus(app);
    }
}

pub fn draw_classroom_panel(app: &mut crate::app::GrafitoApp, ctx: &egui::Context) {
    // La outbox vive acá, no en el arranque: este draw solo corre cuando
    // el Aula se muestra (opt-in + tab/sheet visible).
    app.classroom.ensure_outbox_loaded();
    let mut out = ClassroomOutcome::default();
    let opt_in_snapshot = app.advanced_red_opt_in;
    let panel_was_opt_in = app.classroom.is_opt_in();
    let mut opt_in = opt_in_snapshot;
    // B-A197: `locale` pre-borrow (el closure de `show` muta vía `app`).
    let aula_locale = app.config_locale();
    // S8 Nivel 0: Aula como bottom-sheet (el drawer lateral está oculto
    // en <480 por S4). Mismo contenido, otro contenedor; desktop intacto.
    if crate::ui::mobile_level0(ctx.screen_rect().width()) {
        let sheet_h = aula_sheet_height(ctx.screen_rect().height());
        egui::TopBottomPanel::bottom("aula_sheet")
            .resizable(true)
            .default_height(sheet_h)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Cierre explícito: el sheet no tiene el rail al lado.
                    ui.horizontal(|ui| {
                        ui.strong(grafito_ui::i18n::t("rail.tab_aula", aula_locale));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .button(grafito_ui::i18n::t("classroom.close", aula_locale))
                                .clicked()
                            {
                                app.left_drawer_open = false;
                                app.compact_drawer_open = false;
                            }
                        });
                    });
                    ui.separator();
                    draw_classroom_contents(
                        ui,
                        app,
                        aula_locale,
                        panel_was_opt_in,
                        &mut opt_in,
                        &mut out,
                    );
                });
            });
    } else {
        egui::SidePanel::left("aula_panel")
            .exact_width(PANEL_LEFT_DEFAULT)
            .resizable(false)
            .show(ctx, |ui| {
                draw_classroom_contents(
                    ui,
                    app,
                    aula_locale,
                    panel_was_opt_in,
                    &mut opt_in,
                    &mut out,
                );
            });
    }
    apply_classroom_outcome(app, aula_locale, ctx, out);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aula_sheet_ocupa_media_pantalla_sin_negativos() {
        // S8: 55 % del viewport, nunca negativo.
        assert_eq!(aula_sheet_height(800.0), 440.0);
        assert_eq!(aula_sheet_height(640.0), 352.0);
        assert_eq!(aula_sheet_height(0.0), 0.0);
        assert!(aula_sheet_height(320.0) < 320.0);
    }

    #[test]
    fn share_code_validation_accepts_alphanumeric_dash_underscore() {
        assert!(ShareCode::new("AULA-123").is_some());
        assert!(ShareCode::new("abc_123").is_some());
        assert!(ShareCode::new("").is_none());
        assert!(ShareCode::new("a".repeat(33)).is_none());
        assert!(ShareCode::new("bad code!").is_none());
        assert!(ShareCode::new("ok-CODE_99").is_some());
    }

    #[test]
    fn share_code_generate_is_valid_and_nonempty() {
        let c = ShareCode::generate();
        assert!(!c.as_str().is_empty());
        assert!(ShareCode::new(c.as_str()).is_some());
        assert!(c.as_str().starts_with("AULA-"));
        assert_eq!(c.as_str().len(), 11);
    }

    #[test]
    fn classroom_panel_opt_in_controls_aula_tab_visibility() {
        let mut panel = ClassroomPanel::new();
        assert!(
            !panel.should_show_aula_tab(),
            "sin opt-in no se muestra tab Aula"
        );
        assert!(!panel.is_opt_in());
        panel.set_opt_in(true);
        assert!(panel.should_show_aula_tab(), "con opt-in tab visible");
        assert!(panel.is_opt_in());
        assert!(panel.share_code().is_some(), "al activar genera ShareCode");
        panel.set_opt_in(false);
        assert!(!panel.should_show_aula_tab());
        // ShareCode persiste tras desactivar (no se borra, solo se oculta tab)
        assert!(panel.share_code().is_some());
    }

    #[test]
    fn classroom_panel_set_opt_in_generates_share_code_once() {
        let mut panel = ClassroomPanel::new();
        panel.set_opt_in(true);
        let first = panel.share_code().unwrap().as_str().to_owned();
        panel.set_opt_in(true);
        let second = panel.share_code().unwrap().as_str().to_owned();
        assert_eq!(first, second, "no regenerar si ya existe");
        let custom = ShareCode::new("CUSTOM-01").unwrap();
        panel.set_share_code(custom.clone());
        assert_eq!(panel.share_code().unwrap(), &custom);
    }

    #[test]
    fn qr_grid_is_deterministic_and_has_finder_patterns() {
        let g1 = generate_qr_modules("AULA-123").expect("QR real genera");
        let g2 = generate_qr_modules("AULA-123").expect("QR real genera");
        assert_eq!(g1, g2, "mismo código -> misma grid");
        let g3 = generate_qr_modules("AULA-999").expect("QR real genera");
        assert_ne!(g1, g3, "código distinto -> grid distinta");
        let n = g1.len();
        assert!(n >= 21, "versión QR mínima 21×21, fue {n}");
        assert!(g1.iter().all(|row| row.len() == n), "grid cuadrada");
        // Finder top-left del QR real: esquinas borde negro, centro negro
        assert!(g1[0][0]);
        assert!(g1[0][6]);
        assert!(g1[6][0]);
        assert!(g1[3][3], "centro finder");
        // Dark module del estándar (fila N-8, col 8)
        assert!(g1[n - 8][8]);
    }

    #[test]
    fn qr_grid_different_codes_produce_different_data_area() {
        let a = generate_qr_modules("code-a").expect("QR real genera");
        let b = generate_qr_modules("code-b").expect("QR real genera");
        assert_eq!(a.len(), b.len());
        // Comparar zona central (10,10) debería diferir con alta prob
        let mut diff = 0;
        for y in 8..13 {
            for x in 8..13 {
                if a[y][x] != b[y][x] {
                    diff += 1;
                }
            }
        }
        assert!(
            diff > 0,
            "zona de datos debe diferir entre códigos distintos"
        );
    }

    #[test]
    fn classroom_ui_is_pure_and_does_not_panic() {
        // No test de egui render real (requiere ctx), solo verificar que construir panel
        // y generar QR no paniquea en modo headless.
        let mut panel = ClassroomPanel::new();
        panel.set_opt_in(true);
        let code = panel.share_code().unwrap().as_str().to_owned();
        let grid = generate_qr_modules(&code).expect("QR real genera");
        assert!(grid.len() >= 21);
    }

    #[test]
    fn aula_tab_visible_with_opt_in() {
        let mut panel = ClassroomPanel::new();
        panel.set_opt_in(true);
        assert!(panel.is_opt_in());
        assert!(
            panel.should_show_aula_tab(),
            "con opt-in el tab Aula debe ser visible"
        );
    }

    #[test]
    fn aula_tab_hidden_without_opt_in() {
        let panel = ClassroomPanel::new();
        assert!(!panel.is_opt_in());
        assert!(
            !panel.should_show_aula_tab(),
            "sin opt-in el tab Aula debe estar oculto"
        );
        let mut panel = ClassroomPanel::new();
        panel.set_opt_in(true);
        panel.set_opt_in(false);
        assert!(
            !panel.should_show_aula_tab(),
            "al revocar el opt-in el tab se oculta"
        );
    }

    #[test]
    fn qr_side_derives_from_tokens() {
        assert_eq!(
            CLASSROOM_QR_SIDE,
            grafito_ui::tokens::PANEL_LEFT_DEFAULT
                - grafito_ui::tokens::SPACE_XXL
                - grafito_ui::tokens::SPACE_XL
                - grafito_ui::tokens::SPACE_LG
        );
        assert_eq!(CLASSROOM_QR_SIDE % 4.0, 0.0);
    }

    #[test]
    fn qr_payload_encodes_join_uri_for_valid_code() {
        assert_eq!(
            qr_payload_for_code("AULA-ABC123").as_deref(),
            Some("grafito://aula/AULA-ABC123")
        );
        assert_eq!(qr_payload_for_code(""), None);
        assert_eq!(qr_payload_for_code("bad code!"), None);
        assert_eq!(qr_payload_for_code(&"a".repeat(33)), None);
    }

    #[test]
    fn qr_real_generates_deterministic_scannable_grid() {
        let first = generate_qr_modules("AULA-ABC123").expect("QR real genera");
        let second = generate_qr_modules("AULA-ABC123").expect("QR real genera");
        assert_eq!(first, second, "mismo código -> misma grid");
        let n = first.len();
        assert!(n >= 21, "versión QR mínima 21×21, fue {n}");
        assert!(
            first.iter().all(|row| row.len() == n),
            "grid cuadrada {n}×{n}"
        );
        // Finder top-left del QR real: esquina y centro oscuros.
        assert!(first[0][0]);
        assert!(first[3][3]);
        // Hay módulos oscuros y claros (no vacía ni sólida).
        let dark = first.iter().flatten().filter(|cell| **cell).count();
        assert!(dark > 0 && dark < n * n);
        // Código distinto -> grid distinta.
        let other = generate_qr_modules("AULA-XYZ999").expect("QR real genera");
        assert_ne!(first, other);
    }

    #[test]
    fn qr_real_fails_honestly_with_empty_or_invalid_code() {
        assert_eq!(generate_qr_modules(""), None);
        assert_eq!(generate_qr_modules("bad code!"), None);
        assert_eq!(generate_qr_modules(&"a".repeat(33)), None);
    }

    #[test]
    fn outbox_notice_se_consume_una_sola_vez() {
        let mut panel = ClassroomPanel::new();
        // En este box el aviso depende del disco real; el invariante es el
        // consumo único, valga `Some` o `None`.
        let primera = panel.take_outbox_notice();
        assert!(panel.take_outbox_notice().is_none());
        assert!(panel.outbox_notice().is_none());
        let _ = primera;
    }

    #[test]
    fn outbox_arranca_vacia_y_acotada() {
        let panel = ClassroomPanel::new();
        assert!(panel.outbox().len() <= grafito_classroom::MAX_OFFLINE_QUEUE);
    }

    #[test]
    fn chat_offline_encola_con_aviso_de_reconexion() {
        // P1b: productor real — enviar con red caída encola + avisa.
        // B-A199: `enqueue` con `locale` (Es pineado al aviso previo).
        use grafito_ui::i18n::Locale;
        let mut panel = ClassroomPanel::new();
        panel.outbox.clear();
        let base = panel.pending_count();
        let id = panel
            .enqueue_chat_message("hola aula", Locale::Es)
            .expect("chat se encola");
        assert!(id >= 1);
        assert_eq!(panel.pending_count(), base + 1);
        let aviso = panel.chat_notice().expect("aviso visible");
        assert!(
            aviso.contains("se enviará al reconectar"),
            "aviso P1b esperado, fue: {aviso}"
        );
        assert!(
            panel.chat_draft().is_empty(),
            "borrador se limpia tras encolar"
        );
    }

    #[test]
    fn chat_offline_vacio_no_encola_y_avisa_honesto() {
        use grafito_ui::i18n::Locale;
        let mut panel = ClassroomPanel::new();
        panel.outbox.clear();
        let base = panel.pending_count();
        assert!(panel.enqueue_chat_message("   ", Locale::Es).is_err());
        assert_eq!(panel.pending_count(), base, "vacío no encola");
        assert!(panel.chat_notice().is_some());
    }

    #[test]
    fn chat_draft_se_acota_a_2048_bytes() {
        let mut panel = ClassroomPanel::new();
        panel.set_chat_draft("x".repeat(5000));
        assert!(panel.chat_draft().len() <= grafito_classroom::MAX_OFFLINE_BODY_BYTES);
    }

    #[test]
    fn rol_default_es_sin_rol_y_sobrevive_al_opt_in() {
        // Decisión: default = el actual (Unset), sin forzar elección. Sin
        // rol no hay sección de rol y el panel queda intacto.
        let mut panel = ClassroomPanel::new();
        assert_eq!(panel.role(), ClassroomRole::Unset);
        assert!(!panel.in_focus());
        assert_eq!(panel.focus_mode(), None);
        panel.set_opt_in(true);
        assert_eq!(
            panel.role(),
            ClassroomRole::Unset,
            "el sync de opt-in por frame no pisa el rol"
        );
        panel.set_role(ClassroomRole::Teacher);
        assert_eq!(panel.role(), ClassroomRole::Teacher);
        panel.set_role(ClassroomRole::Student);
        assert_eq!(panel.role(), ClassroomRole::Student);
    }

    #[test]
    fn rol_etiquetas_resuelven_en_seis_idiomas() {
        use grafito_ui::i18n::Locale;
        assert_eq!(
            classroom_role_label_for(ClassroomRole::Unset, Locale::Es),
            "Sin rol"
        );
        assert_eq!(
            classroom_role_label_for(ClassroomRole::Teacher, Locale::Es),
            "Profesor"
        );
        assert_eq!(
            classroom_role_label_for(ClassroomRole::Student, Locale::Es),
            "Alumno"
        );
        assert_eq!(
            classroom_role_label_for(ClassroomRole::Teacher, Locale::En),
            "Teacher"
        );
        assert_eq!(
            classroom_role_label_for(ClassroomRole::Student, Locale::Pt),
            "Aluno"
        );
        assert_eq!(
            classroom_role_label_for(ClassroomRole::Teacher, Locale::It),
            "Insegnante"
        );
        assert_eq!(
            classroom_role_label_for(ClassroomRole::Student, Locale::Fr),
            "Élève"
        );
        assert_eq!(
            classroom_role_label_for(ClassroomRole::Teacher, Locale::De),
            "Lehrer"
        );
    }

    #[test]
    fn foco_guarda_foto_cambia_modo_y_sale_una_vez() {
        let mut panel = ClassroomPanel::new();
        let foto = ClassroomShellSnapshot {
            left_drawer_open: true,
            compact_drawer_open: false,
            right_drawer_open: true,
            assistant_visible: true,
            keyboard_visible: false,
        };
        panel.enter_focus(ClassroomFocus::Present, foto);
        assert!(panel.in_focus());
        assert_eq!(panel.focus_mode(), Some(ClassroomFocus::Present));
        // Re-entrar (p. ej. Present → Seguir sin Esc) conserva la foto
        // ORIGINAL: Esc restaura al estado previo al primer enter.
        let otra = ClassroomShellSnapshot::default();
        panel.enter_focus(ClassroomFocus::Attend, otra);
        assert_eq!(panel.focus_mode(), Some(ClassroomFocus::Attend));
        assert_eq!(
            panel.exit_focus(),
            Some(foto),
            "restaura la primera foto, no la intermedia"
        );
        assert!(!panel.in_focus());
        // Doble salida es no-op.
        assert_eq!(panel.exit_focus(), None);
    }

    #[test]
    fn rol_es_sesion_sola_sin_persistencia_en_disco() {
        // Alcance cerrado: el rol NO viaja a `AppConfig` (sin preferencia
        // natural previa: sesión sola + reportado). Blindaje estilo repo:
        // ningún campo de rol en la config persistida ni guardado.
        let utils = include_str!("utils.rs");
        for needle in [
            "classroom_role",
            "ClassroomRole",
            "role_teacher",
            "role_student",
        ] {
            assert!(
                !utils.contains(needle),
                "utils.rs (AppConfig) sin rol de Aula: {needle}"
            );
        }
        let source = include_str!("classroom.rs");
        assert_eq!(
            source.matches("\n        app.save_app_config();").count(),
            1,
            "una sola persistencia real: el opt-in preexistente; el rol no persiste"
        );
        // Y el default recién creado sigue siendo Unset.
        assert_eq!(ClassroomPanel::new().role(), ClassroomRole::Unset);
    }
}
