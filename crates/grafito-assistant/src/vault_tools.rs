//! Puente Obsidian del modo agente: el vault como espacio de trabajo.
//!
//! Cuatro superficies seguras (`list`/`read`/`write`/`move` para
//! reorganizar) con sandbox estricto: todo path es relativo a `vault_root`
//! (env `GRAFITO_VAULT_DIR`, default `~/Documentos/GrafitoVault`), sin
//! escapes (`..`, raíz absoluta, NUL), con topes de bytes y entradas.
//! Fail-closed: error tipado honesto `E_VAULT_*`, jamás fallback silencioso
//! ni I/O fuera del root. El caller (`grafito-app`) las agrega dinámicamente
//! al catálogo del agente cuando el vault está configurado — igual que
//! `web_search` — para no mover los pins `all_safe`(39)/MCP(71).
//!
//! El I/O corre en el worker del agente (nunca en la UI), con la misma
//! justificación que `web.rs`: herramientas opt-in que lo necesitan.

use grafito_agent::schema::{ToolCall, ToolResult, ToolSchema};
use serde_json::{json, Value};

/// Tope de entradas por `vault_list_dir`.
pub const VAULT_LIST_MAX_ENTRIES: usize = 512;
/// Tope de bytes leídos por `vault_read_note`.
pub const VAULT_READ_MAX_BYTES: u64 = 256 * 1024;
/// Tope de bytes escritos por `vault_write_note`.
pub const VAULT_WRITE_MAX_BYTES: usize = 64 * 1024;
/// Componentes máximos de un subpath.
pub const VAULT_MAX_COMPONENTS: usize = 24;
/// Nombre máximo por componente.
pub const VAULT_COMPONENT_MAX_LEN: usize = 128;

/// Root del vault: `GRAFITO_VAULT_DIR` gana; default `~/Documentos/GrafitoVault`.
///
/// Sin efectos más allá de leer env; el caller decide si expone las tools.
pub fn vault_root() -> Result<std::path::PathBuf, String> {
    if let Ok(dir) = std::env::var("GRAFITO_VAULT_DIR") {
        let dir = dir.trim();
        if !dir.is_empty() {
            return Ok(std::path::PathBuf::from(dir));
        }
    }
    let home = std::env::var("HOME")
        .map_err(|_| "E_VAULT_ROOT: HOME no definido; configurá GRAFITO_VAULT_DIR".to_string())?;
    Ok(std::path::PathBuf::from(home)
        .join("Documentos")
        .join("GrafitoVault"))
}

/// ¿El vault está configurado/existe? La app expone las tools si esto da true.
pub fn vault_configured() -> bool {
    std::env::var("GRAFITO_VAULT_DIR").is_ok_and(|d| !d.trim().is_empty())
        || vault_root().map(|root| root.is_dir()).unwrap_or(false)
}

/// Resolve fail-closed de un subpath relativo al root dado.
///
/// Rechaza: `..` (traversal), raíces absolutas, NUL, tilde, componentes
/// vacíos; `.` redundante se tolera; si existe ⇒ canonicalize + spoke-check
/// del root (symlink escape detectado). No existe ⇒ el target se quedará
/// bajo el root (el caller crea padres donde corresponde).
pub fn vault_resolve_in(
    root: &std::path::Path,
    subpath: &str,
) -> Result<std::path::PathBuf, String> {
    if subpath.contains('\0') {
        return Err("E_VAULT_PATH: el path no puede contener NUL".to_string());
    }
    if subpath.contains("..") {
        return Err("E_VAULT_PATH: '..' no está permitido (sandbox del vault)".to_string());
    }
    let trimmed = subpath.trim();
    if trimmed.starts_with('/') || trimmed.starts_with('\\') {
        return Err("E_VAULT_PATH: el path debe ser relativo al vault".to_string());
    }
    if trimmed.starts_with('~') {
        return Err("E_VAULT_PATH: '~' no se expande (usá el root del vault)".to_string());
    }
    let rel = std::path::Path::new(trimmed);
    if rel.components().count() > VAULT_MAX_COMPONENTS {
        return Err(format!(
            "E_VAULT_PATH: máximo {VAULT_MAX_COMPONENTS} componentes"
        ));
    }
    let mut target = root.to_path_buf();
    for comp in rel.components() {
        match comp {
            std::path::Component::Normal(seg) => {
                let nombre = seg.to_string_lossy();
                if nombre.chars().count() > VAULT_COMPONENT_MAX_LEN {
                    return Err(format!(
                        "E_VAULT_PATH: componente con más de {VAULT_COMPONENT_MAX_LEN} caracteres"
                    ));
                }
                if nombre.chars().any(char::is_control) {
                    return Err("E_VAULT_PATH: caracteres de control no permitidos".to_string());
                }
                target.push(seg);
            }
            std::path::Component::CurDir => {}
            other => {
                return Err(format!(
                "E_VAULT_PATH: componente no permitido ({other:?}); sólo paths relativos al vault"
            ))
            }
        }
    }
    if let Ok(canon) = target.canonicalize() {
        let root_canon = root
            .canonicalize()
            .map_err(|e| format!("E_VAULT_ROOT: {e}"))?;
        if !canon.starts_with(&root_canon) {
            return Err("E_VAULT_ESCAPE: el path resuelve fuera del vault".to_string());
        }
        return Ok(canon);
    }
    Ok(target)
}

/// Resolve con el vault configurado (env/default).
pub fn vault_resolve(subpath: &str) -> Result<std::path::PathBuf, String> {
    vault_resolve_in(&vault_root()?, subpath)
}

/// `vault_list_dir`: (name, kind, bytes) acotado a 512, sin orden garantizado.
pub fn vault_list_dir_in(root: &std::path::Path, subpath: &str) -> Result<Value, String> {
    let path = vault_resolve_in(root, subpath)?;
    let meta =
        std::fs::metadata(&path).map_err(|e| format!("E_VAULT_LIST: {e} ({})", path.display()))?;
    if !meta.is_dir() {
        return Err(format!(
            "E_VAULT_LIST: {} no es una carpeta",
            path.display()
        ));
    }
    let mut out: Vec<Value> = Vec::new();
    let mut cortado = false;
    for entry in std::fs::read_dir(&path).map_err(|e| format!("E_VAULT_LIST: {e}"))? {
        if out.len() >= VAULT_LIST_MAX_ENTRIES {
            cortado = true;
            break;
        }
        let Ok(entry) = entry else { continue };
        let Ok(meta) = entry.metadata() else { continue };
        let name = entry.file_name().to_string_lossy().to_string();
        let bytes = meta.is_file().then_some(meta.len());
        out.push(json!({
            "name": name,
            "kind": if meta.is_dir() { "dir" } else { "file" },
            "bytes": bytes,
        }));
    }
    let mut payload = json!({
        "ok": true,
        "path": path.display().to_string(),
        "count": out.len(),
        "entries": out,
    });
    if cortado {
        payload["note"] = json!(format!("cortado a {VAULT_LIST_MAX_ENTRIES} entradas"));
    }
    Ok(payload)
}

/// `vault_read_note`: texto ≤256 KiB (aviso de recorte si excede).
pub fn vault_read_note_in(root: &std::path::Path, subpath: &str) -> Result<Value, String> {
    let path = vault_resolve_in(root, subpath)?;
    let meta = std::fs::metadata(&path).map_err(|e| format!("E_VAULT_READ: {e}"))?;
    if !meta.is_file() {
        return Err(format!("E_VAULT_READ: {} no es un archivo", path.display()));
    }
    if meta.len() > VAULT_READ_MAX_BYTES {
        return Err(format!(
            "E_VAULT_READ: {} bytes exceden el tope {VAULT_READ_MAX_BYTES}; dividí la nota",
            meta.len()
        ));
    }
    let text = std::fs::read_to_string(&path).map_err(|e| format!("E_VAULT_READ: {e}"))?;
    let truncated = text.chars().count() > 24_000;
    let contenido: String = if truncated {
        text.chars().take(24_000).collect()
    } else {
        text
    };
    Ok(json!({
        "ok": true,
        "path": path.display().to_string(),
        "bytes": meta.len(),
        "truncated": truncated,
        "content": contenido,
    }))
}

/// `vault_write_note`: escribe/reescribe (crea padres); content ≤64 KiB.
///
/// Con `bak:true` y contenido anterior distinto deja hermana `.bak` del
/// contenido previo (jamás pisar sin copia cuando el caller pidió resguardo).
/// El contenido debe ser UTF-8 (`fs::write` falla con el motivo al revés).
pub fn vault_write_note_in(
    root: &std::path::Path,
    subpath: &str,
    content: &str,
    with_bak: bool,
) -> Result<Value, String> {
    if content.len() > VAULT_WRITE_MAX_BYTES {
        return Err(format!(
            "E_VAULT_WRITE: {} bytes exceden el tope {VAULT_WRITE_MAX_BYTES}",
            content.len()
        ));
    }
    let path = vault_resolve_in(root, subpath)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("E_VAULT_WRITE: no se pudo crear la carpeta: {e}"))?;
    }
    let mut backup = None;
    if with_bak {
        if let Ok(prev) = std::fs::read_to_string(&path) {
            if prev != content {
                let bak = format!("{}.bak", path.display());
                if std::fs::write(&bak, prev).is_ok() {
                    backup = Some(bak);
                }
            }
        }
    }
    std::fs::write(&path, content).map_err(|e| format!("E_VAULT_WRITE: {e}"))?;
    Ok(json!({
        "ok": true,
        "path": path.display().to_string(),
        "bytes": content.len(),
        "backup": backup,
    }))
}

/// `vault_move_file`: reorganizar notas/carpetas dentro del vault.
/// Sin clobber salvo `overwrite:true`.
pub fn vault_move_file_in(
    root: &std::path::Path,
    from: &str,
    to: &str,
    overwrite: bool,
) -> Result<Value, String> {
    let from_p = vault_resolve_in(root, from)?;
    let to_p = vault_resolve_in(root, to)?;
    if !from_p.exists() {
        return Err(format!(
            "E_VAULT_MOVE: origen no existe: {}",
            from_p.display()
        ));
    }
    if from_p == to_p {
        return Err("E_VAULT_MOVE: origen y destino son el mismo".to_string());
    }
    if to_p.exists() && !overwrite {
        return Err(format!(
            "E_VAULT_MOVE: destino existe: {}; pasá overwrite:true si lo querés pisar",
            to_p.display()
        ));
    }
    // Reorganización amable: crea las carpetas del destino si faltan
    // (siempre dentro del sandbox: `to_p` ya fue resuelto fail-closed).
    if let Some(parent) = to_p.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("E_VAULT_MOVE: no se pudo crear la carpeta destino: {e}"))?;
    }
    std::fs::rename(&from_p, &to_p).map_err(|e| format!("E_VAULT_MOVE: {e}"))?;
    Ok(json!({
        "ok": true,
        "from": from_p.display().to_string(),
        "to": to_p.display().to_string(),
    }))
}

// ── Dispatch del agente ──────────────────────────────────────────────────

/// Despacha los 4 tool calls del vault con el root del vault configurado.
pub fn dispatch(call: &ToolCall) -> ToolResult {
    let root = match vault_root() {
        Ok(root) => root,
        Err(e) => return ToolResult::text(&call.id, false, e),
    };
    if let Err(e) = root.canonicalize() {
        return ToolResult::text(
            &call.id,
            false,
            format!("E_VAULT_ROOT: la carpeta del vault no existe todavía ({e}); creala o definí GRAFITO_VAULT_DIR"),
        );
    }
    match call.name.as_str() {
        "vault_list_dir" => {
            let subpath = call
                .arguments
                .get("subpath")
                .and_then(Value::as_str)
                .unwrap_or("");
            result_of(call, vault_list_dir_in(&root, subpath))
        }
        "vault_read_note" => {
            let Some(subpath) = call
                .arguments
                .get("path")
                .and_then(Value::as_str)
                .map(|s| s.to_string())
            else {
                return ToolResult::text(&call.id, false, "E_VAULT_READ: falta `path`");
            };
            result_of(call, vault_read_note_in(&root, &subpath))
        }
        "vault_write_note" => {
            let Some(subpath) = call
                .arguments
                .get("path")
                .and_then(Value::as_str)
                .map(|s| s.to_string())
            else {
                return ToolResult::text(&call.id, false, "E_VAULT_WRITE: falta `path`");
            };
            let Some(content) = call
                .arguments
                .get("content")
                .and_then(Value::as_str)
                .map(|s| s.to_string())
            else {
                return ToolResult::text(&call.id, false, "E_VAULT_WRITE: falta `content`");
            };
            let with_bak = call
                .arguments
                .get("bak")
                .and_then(Value::as_bool)
                .unwrap_or(true);
            result_of(
                call,
                vault_write_note_in(&root, &subpath, &content, with_bak),
            )
        }
        "vault_move_file" => {
            let Some(from) = call
                .arguments
                .get("from")
                .and_then(Value::as_str)
                .map(|s| s.to_string())
            else {
                return ToolResult::text(&call.id, false, "E_VAULT_MOVE: falta `from`");
            };
            let Some(to) = call
                .arguments
                .get("to")
                .and_then(Value::as_str)
                .map(|s| s.to_string())
            else {
                return ToolResult::text(&call.id, false, "E_VAULT_MOVE: falta `to`");
            };
            let overwrite = call
                .arguments
                .get("overwrite")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            result_of(call, vault_move_file_in(&root, &from, &to, overwrite))
        }
        other => ToolResult::text(
            &call.id,
            false,
            format!("tool '{other}' no es del vault (list/read/write/move)"),
        ),
    }
}

/// Envuelve el Result de una op como ToolResult del call.
fn result_of(call: &ToolCall, result: Result<Value, String>) -> ToolResult {
    match result {
        Ok(v) => ToolResult::text(&call.id, true, v.to_string()),
        Err(e) => ToolResult::text(&call.id, false, e),
    }
}

/// Schemas de las 4 herramientas (se exponen sólo si `vault_configured()`).
pub fn vault_tool_schemas() -> Vec<ToolSchema> {
    vec![
        ToolSchema::new(
            "vault_list_dir",
            "Lista el contenido de una carpeta de tu vault Obsidian (sólo dentro del vault configurado). Usala para ubicar notas o carpetas antes de escribir.",
            json!({
                "type": "object",
                "properties": {
                    "subpath": {"type": "string", "description": "Subpath relativo al vault (vacío = raíz)"}
                },
                "required": []
            }),
        ),
        ToolSchema::new(
            "vault_read_note",
            "Lee una nota del vault Obsidian (texto, hasta 256 KiB) para citarla o extenderla.",
            json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Path relativo al vault, ej. Resúmenes/derivada.md"}
                },
                "required": ["path"]
            }),
        ),
        ToolSchema::new(
            "vault_write_note",
            "Crea o actualiza una nota Markdown en el vault Obsidian (hasta 64 KiB). Incluí bloques ```grafito``` re-ejecutables y YAML title/tags cuando te lo pidan.",
            json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Path relativo al vault con nombre de nota, ej. Resúmenes/derivada.md"},
                    "content": {"type": "string", "description": "Contenido completo en Markdown de la nota"},
                    "bak": {"type": "boolean", "description": "Guardar .bak si sobreescribe distinto (default true)"}
                },
                "required": ["path", "content"]
            }),
        )
        .with_consent(true),
        ToolSchema::new(
            "vault_move_file",
            "Mueve o renombra una nota/carpeta dentro del vault (reorganización). No pisa destino salvo overwrite:true.",
            json!({
                "type": "object",
                "properties": {
                    "from": {"type": "string", "description": "Path relativo del origen"},
                    "to": {"type": "string", "description": "Path relativo del destino"},
                    "overwrite": {"type": "boolean", "description": "Pisar el destino si existe (default false)"}
                },
                "required": ["from", "to"]
            }),
        )
        .with_consent(true),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("grafito-vault-{}-{tag}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    #[test]
    fn sandbox_rejecta_traversal_absoluto_nul_tilde() {
        let root = temp_root("sandbox");
        for (subpath, why) in [
            ("../afuera.md", "traversal literal"),
            ("carpeta/../../otro.md", "traversal anidado"),
            ("/etc/passwd", "absoluto"),
            ("a\0b.md", "NUL"),
            ("~/x.md", "tilde"),
        ] {
            assert!(
                vault_resolve_in(&root, subpath).is_err(),
                "debe rechazar {subpath} ({why})"
            );
        }
    }

    #[test]
    fn symlink_escape_fuera_del_root_se_detecta() {
        let root = temp_root("symlink");
        let outside = temp_root("symlink-out");
        let _ = std::fs::write(outside.join("nota-fuera.md"), "x");
        #[cfg(unix)]
        {
            let _ = std::os::unix::fs::symlink(&outside, root.join("escape"));
            assert!(
                vault_resolve_in(&root, "escape/nota-fuera.md").is_err(),
                "symlink saliendo del vault → E_VAULT_ESCAPE"
            );
        }
    }

    #[test]
    fn componentes_sobran_largos_y_curdir() {
        let root = temp_root("comps");
        let largo = format!("{}.md", "a".repeat(VAULT_COMPONENT_MAX_LEN + 1));
        assert!(vault_resolve_in(&root, &largo).is_err());
        let deep = (0..=VAULT_MAX_COMPONENTS)
            .map(|i| format!("c{i}"))
            .collect::<Vec<_>>()
            .join("/");
        assert!(
            vault_resolve_in(&root, &deep).is_err(),
            "demasiado profundo"
        );
        assert!(
            vault_resolve_in(&root, "./carpeta/nota.md").is_ok(),
            "CurDir ok"
        );
    }

    #[test]
    fn write_read_move_roundtrip_con_bak() {
        let root = temp_root("roundtrip");
        let contenido = "# nota\n\n```grafito\ny=sin(x)\n```\n";
        assert!(vault_write_note_in(&root, "Resúmenes/derivada.md", contenido, true).is_ok());
        let leida = vault_read_note_in(&root, "Resúmenes/derivada.md").expect("read");
        assert_eq!(leida["content"], contenido);
        let segunda = vault_write_note_in(&root, "Resúmenes/derivada.md", "# nota v2\n", true)
            .expect("write v2");
        assert!(segunda["backup"].as_str().is_some(), ".bak guardado");
        assert!(root.join("Resúmenes/derivada.md.bak").exists());
        let moved = vault_move_file_in(
            &root,
            "Resúmenes/derivada.md",
            "Archivo/2026/derivada.md",
            false,
        )
        .expect("move");
        assert_eq!(moved["ok"], true);
        assert!(!root.join("Resúmenes/derivada.md").exists());
        assert!(root.join("Archivo/2026/derivada.md").exists());
    }

    #[test]
    fn read_y_write_respetan_los_topes() {
        let root = temp_root("bounds");
        let grande_lectura = "x".repeat(VAULT_READ_MAX_BYTES as usize + 1);
        let _ = std::fs::write(root.join("grande.md"), &grande_lectura);
        assert!(vault_read_note_in(&root, "grande.md").is_err(), "tope read");
        let grande_escritura = "x".repeat(VAULT_WRITE_MAX_BYTES + 1);
        assert!(
            vault_write_note_in(&root, "nocrear.md", &grande_escritura, false).is_err(),
            "tope write"
        );
        assert!(!root.join("nocrear.md").exists(), "fail-closed sin crear");
    }

    #[test]
    fn list_dir_del_root_de_prueba() {
        let root = temp_root("list");
        let _ = std::fs::create_dir_all(root.join("Resúmenes"));
        let _ = std::fs::write(root.join("índice.md"), "# índice");
        let v = vault_list_dir_in(&root, "").expect("list");
        let names = v["entries"]
            .as_array()
            .expect("entries")
            .iter()
            .map(|i| i["name"].as_str().unwrap_or_default().to_string())
            .collect::<Vec<_>>();
        assert!(names.contains(&"Resúmenes".to_string()));
        assert!(names.contains(&"índice.md".to_string()));
        assert_eq!(v["ok"], true);
    }

    #[test]
    fn schemas_cubren_las_4_y_dispatch_conoce_los_nombres() {
        let schemas = vault_tool_schemas();
        assert_eq!(schemas.len(), 4);
        let names: Vec<String> = schemas.iter().map(|s| s.name.clone()).collect();
        assert_eq!(
            names,
            [
                "vault_list_dir".to_string(),
                "vault_read_note".to_string(),
                "vault_write_note".to_string(),
                "vault_move_file".to_string(),
            ]
        );
        let call = ToolCall {
            id: "v1".to_string(),
            name: "vault_totalmente_otra".to_string(),
            arguments: serde_json::json!({}),
        };
        let r = dispatch(&call);
        assert!(!r.ok, "tool desconocida del vault → honesta");
        assert!(r.content.contains("no es del vault"));
    }
}
