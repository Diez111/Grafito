//! Los plugins de ejemplo del repo (`plugins/`) deben cargar y validar
//! fail-closed contra las capacidades reales. F5: incluye el enforce de
//! `min_app_version` (los ejemplos declaran `1.0.0`, la app corre `1.0.0`).

use grafito_plugins::{PluginRegistry, ValidationContext};
use std::path::Path;

/// Capacidades reales mínimas que usan los plugins del repo.
fn ctx() -> ValidationContext<'static> {
    ValidationContext {
        resolvable_command_ids: &|id| matches!(id, "Function" | "Tetrahedron"),
        known_tools: &["evaluate_expr", "grafito_docs", "ask_user"],
        known_scenes: &["derivative-slope", "graph-trace", "riemann"],
    }
}

#[test]
fn example_plugins_load_without_errors() {
    let plugins_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../plugins");
    if !plugins_dir.is_dir() {
        // Sin árbol de plugins (crate empaquetado): nada que verificar.
        return;
    }
    let registry = PluginRegistry::load(&plugins_dir, &ctx());
    assert!(
        !registry.plugins.is_empty(),
        "el repo debe traer al menos un plugin de ejemplo"
    );
    for plugin in &registry.plugins {
        assert!(
            plugin.error.is_none(),
            "plugin '{}' no debe fallar la validación: {:?}",
            plugin.manifest.plugin.id,
            plugin.error
        );
    }
    let ids: Vec<&str> = registry
        .plugins
        .iter()
        .map(|p| p.manifest.plugin.id.as_str())
        .collect();
    for expected in ["j-space", "tutor.packs-utn", "ejercicios.secundaria"] {
        assert!(
            ids.contains(&expected),
            "falta el plugin de ejemplo {expected}: {ids:?}"
        );
    }
}
