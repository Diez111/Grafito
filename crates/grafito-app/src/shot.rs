//! Hook de verificación visual para capturas bajo Xvfb (NO es feature).
//!
//! Lee `GRAFITO_SHOT` al arrancar y deja la vista correspondiente abierta
//! para que el operador saque la captura con `import -window root` (en el
//! box no hay xdotool: la vista debe abrirse sola, sin clicks). Todo es
//! DEMO solo en memoria: NUNCA escribe al disco ni persiste el perfil.
//! Valor ausente o desconocido = arranque normal, sin tocar nada.
//!
//! J2: el vault salió de `main`; `GRAFITO_SHOT=vault` ya no abre nada
//! (arranque normal = Canvas). Se conserva el hook mínimo para no romper
//! el contrato de arranque.

use crate::app::GrafitoApp;

/// Valores aceptados de `GRAFITO_SHOT`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Shot {
    /// Abre el Canvas (vista por defecto, sin tocar nada más).
    Canvas,
}

/// Variable de entorno que dispara el hook.
pub(crate) const SHOT_ENV: &str = "GRAFITO_SHOT";

/// Tamaño de ventana de las capturas (representativo, 16:9).
pub(crate) const SHOT_WINDOW_SIZE: [f32; 2] = [1600.0, 900.0];
/// Tamaño normal sin hook (fuente única del `ViewportBuilder` inicial).
pub(crate) const DEFAULT_WINDOW_SIZE: [f32; 2] = [1280.0, 720.0];

/// Tamaño inicial de la ventana nativa (puro, sin egui): 1600x900 si
/// `GRAFITO_SHOT` trae un valor conocido, 1280x720 si no. Es la misma
/// fuente que `apply_shot` + el `ViewportCommand` del primer frame: el
/// comando solo queda de respaldo por si el WM ignora el tamaño inicial.
/// Fuente única para no repetir el `if shot` en cada camino.
pub(crate) fn initial_window_size() -> [f32; 2] {
    window_size_for(shot_from_env())
}

/// Mapeo puro testeable: shot conocido = 1600x900, resto = 1280x720.
fn window_size_for(shot: Option<Shot>) -> [f32; 2] {
    if shot.is_some() {
        SHOT_WINDOW_SIZE
    } else {
        DEFAULT_WINDOW_SIZE
    }
}

/// Parsea el valor crudo de la variable (puro, sin I/O: lo testeable).
/// Desconocido o vacío = `None` (arranque normal = Canvas).
/// `vault` es legado de cuando el vault vivía en `main` (J2): hoy es
/// arranque normal, que ya muestra el Canvas.
pub(crate) fn shot_from_value(raw: &str) -> Option<Shot> {
    match raw.trim() {
        "canvas" => Some(Shot::Canvas),
        _ => None,
    }
}

/// Lee `GRAFITO_SHOT` del entorno (puro I/O de arranque, sin egui).
/// Ausente o inválida = `None` (arranque normal).
pub(crate) fn shot_from_env() -> Option<Shot> {
    std::env::var(SHOT_ENV)
        .ok()
        .and_then(|raw| shot_from_value(&raw))
}

/// Aplica el hook si la variable está puesta. Idempotente y sin I/O:
/// solo muta estado en memoria; jamás toca el disco ni el perfil.
pub(crate) fn maybe_apply_shot(app: &mut GrafitoApp) {
    if let Some(shot) = shot_from_env() {
        apply_shot(app, shot);
    }
}

/// Aplica un `Shot` ya parseado (seam testeable headless, sin screenshots).
pub(crate) fn apply_shot(app: &mut GrafitoApp, shot: Shot) {
    // El onboarding tapa la vista con el carrusel: en sesión demo se baja
    // en memoria (el archivo de config no se toca).
    app.show_onboarding = false;
    // Capturas representativas: ventana 1600x900 en el primer frame
    // (el `update` lo consume una sola vez vía `ViewportCommand`).
    app.shot_viewport_resize = true;
    match shot {
        Shot::Canvas => apply_canvas(app),
    }
}

/// `canvas`: sin sembrar nada (sin tabs: el Canvas es la única vista).
fn apply_canvas(_app: &mut GrafitoApp) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valores_conocidos_y_desconocido() {
        assert_eq!(shot_from_value("canvas"), Some(Shot::Canvas));
        assert_eq!(shot_from_value("nada"), None);
        assert_eq!(shot_from_value(""), None);
        assert_eq!(shot_from_value(" VAULT "), None);
        // J2: `vault` es legado — arranque normal (= Canvas), sin drawer.
        assert_eq!(shot_from_value("vault"), None);
        assert_eq!(shot_from_value("plan"), None);
        assert_eq!(shot_from_value("quiz"), None);
    }

    #[test]
    fn canvas_enfoca_sin_io() {
        let mut app = crate::app::dummy_grafito_app();
        apply_shot(&mut app, Shot::Canvas);
        assert!(!app.show_onboarding);
    }

    #[test]
    fn todo_shot_baja_onboarding_y_pide_ventana_representativa() {
        // El `dummy` trae `show_onboarding=false`: se simula el first-run
        // real (`new()` con `!onboarding_completed`) para que el pin no sea
        // vacuo. Un solo lugar (`apply_shot`): los caminos heredan.
        let mut app = crate::app::dummy_grafito_app();
        app.show_onboarding = true;
        app.shot_viewport_resize = false;
        apply_shot(&mut app, Shot::Canvas);
        assert!(!app.show_onboarding);
        assert!(app.shot_viewport_resize);
    }

    #[test]
    fn ventana_inicial_1600x900_solo_con_shot_conocido() {
        // Sin variable el entorno del test arranca normal (1280x720).
        // El mapeo con valor se pinea en puro (`window_size_for`), sin
        // tocar el env del proceso (los tests corren en un solo proceso).
        assert_eq!(SHOT_WINDOW_SIZE, [1600.0, 900.0]);
        assert_eq!(DEFAULT_WINDOW_SIZE, [1280.0, 720.0]);
        assert_eq!(window_size_for(Some(Shot::Canvas)), SHOT_WINDOW_SIZE);
        assert_eq!(window_size_for(None), DEFAULT_WINDOW_SIZE);
    }
}
