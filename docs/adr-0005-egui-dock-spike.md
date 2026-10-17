# ADR-0005 — Spike `egui_dock` Editor/Canvas/Kanban (F15.2): diferido con evidencia

## Estado
Spike ejecutado 2026-10-05. **Diferido**: no se mergea a `main`. Queda cuantificado
y listo para retomar en la rama `spike/egui-dock-F15.2` (commit `033f1f6`).

## Contexto
F15 Studio pide split Editor/Canvas/Kanban simultáneos y reordenables. La duda: ¿vale
`egui_dock` contra el layout actual (`SidePanel` + `CentralPanel` + `WorkspaceDockTab`
Inspector/Assistant en `grafito-app`, breakpoints en `ShellLayout`)?

## Evidencia (probe real, no estimación)
Worktree `/tmp/opencode/egui-dock-spike` sobre HEAD `4c2cf7e`, `egui_dock 0.14.0`
(último compatible con egui 0.29, `req ^0.29`) en `grafito-ui`, módulo
`dock_spike.rs` (solo `#[cfg(test)]`, cero cambio productivo): `DockState` con
Canvas al centro + `split_left(root, 0.25, [Editor])` + `split_right(root, 0.7,
[Kanban])`, 2 tests (split expone las 3 tabs + pin de tamaños).

Memoria medida en este box (x86_64, Rust 1.92):

| Estado | Stack | Heap | Alloc |
|---|---|---|---|
| Actual (`CurrentDockModel`: tab + 2 flags + clase) | 4 B | 0 | ninguna (`Copy`) |
| `DockState<StudioTab>` (3 tabs, 7 nodos) | 112 B (28×) | ~0.6 KiB | sí |

Desglose medido: `Node<StudioTab>` 72 B × 7 nodos = 504 B + `Tree` 40 B +
`Surface` + 3 tabs de 1 B + `Translations` (3 `String` en inglés con heap propio).
El heap del dock (~0.6 KiB) es irrelevante frente a una escena; el punto es que el
estado actual no aloca nada y el dock sí (nodos + `Strings` del chrome).

Costo de dependencias (medido): lock +2 paquetes (`egui_dock 0.14.0` +
`duplicate 2.0.1` proc-macro); cierre `cargo tree -p grafito-ui` 189→193;
MSRV intacta (`egui_dock` rust 1.76, `duplicate` 1.65, ambas < 1.92); **sin**
upgrade de egui (0.29 se queda, a diferencia de ADR-0004).

Gates del spike en su rama: `cargo fmt -p grafito-ui` 0,
`cargo clippy -p grafito-ui --all-targets --locked -- -D warnings` 0,
`cargo test -p grafito-ui --locked --lib` 438/0 (436 base + 2 del spike).

## Costos de integración (inspección, no medidos)
1. El canvas de Grafito es wgpu propio (texturas), no widgets egui: viviría como
   `Image` dentro de `TabViewer::ui`, con rewire de hover/picking/input.
2. `DockArea` tomaría el área central: `ShellLayout` (Compact/Medium/Wide) y los
   breakpoints de drawers se reescriben.
3. El chrome del dock es inglés fijo (`Translations::english`); el i18n propio
   (1593 claves × 6 idiomas) tendría que duplicar "Close"/"Eject"/tooltips vía
   `with_translations`.
4. Tab bars + drop targets por frame sobre un canvas ya pesado (ver
   `docs/profiling.md` §2: los costos están en sampler/alloc/texturas).
5. Persistencia del layout trae `serde` + `egui/serde` (hoy desactivados).

## Decisión
Diferir. El costo absoluto (~0.6 KiB) no es el problema; la integración es
multi-día (canvas wgpu en tabs, ShellLayout, foco/shortcuts, i18n del chrome)
sin un cuello UX medido que la exija: `WorkspaceDockTab` + `ShellLayout` ya
cubren Inspector/Assistant. Nada se mergea; `main` solo recibe este ADR.

## Retomada (cuando Studio exija paneles reordenables por el usuario)
1. Partir de `spike/egui-dock-F15.2` (`033f1f6`) como contrato.
2. Prototipar el canvas wgpu como tab (textura en `TabViewer::ui` + picking)
   antes de tocar `ShellLayout`.
3. Cablear `with_translations` al `locale` (`config_locale`) + persistencia con
   `serde` acotada, y cerrar con los 4 gates + E2E `computer-control`.
