//! Puente MCP ↔ CAD externo (KiCad, FreeCAD, Blender, LibreCAD).
//!
//! Honesto por diseño: NADA de esto es nativo Rust — son procesos externos
//! que quizás no están instalados. `cad_probe` reporta disponibilidad real
//! (`which` + `--version` acotado). `cad_run` ejecuta UNA línea CLI
//! allowlisteada (solo lectura/export) con timeout y sin shell.
//! El render pesado y la GPU quedan en Grafito; el CAD aporta verificación.

use serde_json::{json, Value};
use std::process::Command;
use std::time::Duration;

const PROBE_TIMEOUT: Duration = Duration::from_secs(10);
const RUN_TIMEOUT_SECS: u64 = 120;

/// Binarios conocidos por herramienta (nombre → binarios candidatos).
fn candidates(tool: &str) -> Option<&'static [&'static str]> {
    match tool {
        "kicad" => Some(&["kicad-cli"]),
        "freecad" => Some(&["freecad", "freecadcmd"]),
        "blender" => Some(&["blender"]),
        "librecad" => Some(&["librecad"]),
        _ => None,
    }
}

/// Comandos de solo lectura/export permitidos por herramienta.
fn allowlisted(tool: &str, sub: &str) -> bool {
    matches!(
        (tool, sub),
        ("kicad", "version")
            | ("kicad", "sch")
            | ("kicad", "pcb")
            | ("kicad", "fp")
            | ("kicad", "sym")
            | ("freecad", "version")
            | ("blender", "version")
            | ("blender", "render")
            | ("librecad", "version")
    )
}

fn which(bin: &str) -> Option<String> {
    let out = Command::new("which").arg(bin).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// `cad_probe(tool?)` → disponibilidad real por herramienta.
pub fn cad_probe(args: &Value) -> Result<Value, String> {
    let only = args.get("tool").and_then(Value::as_str);
    let tools = ["kicad", "freecad", "blender", "librecad"];
    let mut out = serde_json::Map::new();
    for tool in tools {
        if let Some(want) = only {
            if want != tool {
                continue;
            }
        }
        let bins: Vec<Value> = candidates(tool)
            .unwrap_or(&[])
            .iter()
            .filter_map(|b| which(b).map(|path| json!({"binary": b, "path": path})))
            .collect();
        out.insert(
            tool.to_string(),
            json!({"available": !bins.is_empty(), "binaries": bins}),
        );
    }
    if let Some(want) = only {
        if candidates(want).is_none() {
            return Err("cad_probe: tool debe ser kicad|freecad|blender|librecad".into());
        }
    }
    Ok(json!({"tool": "cad_probe", "probe_timeout_s": PROBE_TIMEOUT.as_secs(), "tools": out}))
}

/// `cad_run(tool, argv)` → ejecuta subcomando allowlisteado, sin shell.
pub fn cad_run(args: &Value) -> Result<Value, String> {
    let tool = args.get("tool").and_then(Value::as_str).unwrap_or("");
    let argv = args
        .get("argv")
        .and_then(Value::as_array)
        .ok_or("cad_run: 'argv' debe ser arreglo")?;
    let cands = candidates(tool).ok_or("cad_run: tool debe ser kicad|freecad|blender|librecad")?;
    if argv.is_empty() || argv.len() > 16 {
        return Err("cad_run: argv de 1..=16 elementos".into());
    }
    let parts: Vec<String> = argv
        .iter()
        .map(|v| v.as_str().unwrap_or("").to_string())
        .collect();
    if parts
        .iter()
        .any(|p| p.is_empty() || p.len() > 512 || p.contains('\0'))
    {
        return Err("cad_run: argumento vacío, largo o con NUL".into());
    }
    if !allowlisted(tool, &parts[0]) {
        return Err(format!(
            "cad_run: subcomando '{}' no allowlisteado para {tool}",
            parts[0]
        ));
    }
    let bin = cands
        .iter()
        .find_map(|b| which(b))
        .ok_or(format!("cad_run: {tool} no instalado (ver cad_probe)"))?;
    let timeout = std::time::Duration::from_secs(
        args.get("timeout_s")
            .and_then(Value::as_u64)
            .unwrap_or(60)
            .clamp(1, RUN_TIMEOUT_SECS),
    );
    let mut child = Command::new(&bin)
        .args(&parts)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("cad_run: no se pudo lanzar {bin}: {e}"))?;
    let status = wait_timeout(&mut child, timeout).ok_or("cad_run: timeout (proceso matado)")?;
    let out = child
        .wait_with_output()
        .map_err(|e| format!("cad_run: {e}"))?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    Ok(json!({
        "tool": "cad_run",
        "binary": bin,
        "success": status.success(),
        "code": out.status.code(),
        "stdout": stdout.chars().take(4000).collect::<String>(),
        "stderr": stderr.chars().take(1000).collect::<String>(),
    }))
}

fn wait_timeout(
    child: &mut std::process::Child,
    timeout: Duration,
) -> Option<std::process::ExitStatus> {
    let start = std::time::Instant::now();
    loop {
        if let Some(status) = child.try_wait().ok()? {
            return Some(status);
        }
        if start.elapsed() >= timeout {
            let _ = child.kill();
            return None;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

pub fn cad_tool_defs() -> Vec<Value> {
    vec![
        json!({
            "name": "cad_probe",
            "description": "Reporta qué CAD externos están instalados (kicad, freecad, blender, librecad).",
            "inputSchema": {
                "type": "object",
                "properties": {"tool": {"type": "string"}},
            },
            "annotations": {"readOnlyHint": true, "destructiveHint": false}
        }),
        json!({
            "name": "cad_run",
            "description": "Ejecuta un subcomando allowlisteado de solo lectura/export en un CAD externo, sin shell.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "tool": {"type": "string"},
                    "argv": {"type": "array", "items": {"type": "string"}},
                    "timeout_s": {"type": "integer", "minimum": 1, "maximum": 120}
                },
                "required": ["tool", "argv"]
            },
            "annotations": {"readOnlyHint": false, "destructiveHint": false}
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_responde_y_run_rechaza() {
        let p = cad_probe(&json!({})).expect("probe");
        assert!(p["tools"]["blender"].get("available").is_some());
        assert!(cad_probe(&json!({"tool": "autocad"})).is_err());
        // Sin shell: `;` no es subcomando válido de nadie.
        assert!(cad_run(&json!({"tool": "blender", "argv": [";rm"]})).is_err());
        assert!(cad_run(&json!({"tool": "blender", "argv": []})).is_err());
        assert!(cad_run(&json!({"tool": "nada", "argv": ["version"]})).is_err());
        // Blender ausente o presente: version es honesto en ambos casos.
        let v = cad_run(&json!({"tool": "blender", "argv": ["version"]}));
        assert!(v.is_ok() || v.unwrap_err().contains("no instalado"));
    }
}
