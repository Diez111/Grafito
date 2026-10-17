# docs/architecture.md — Grafito v1.3.0 (fuente de verdad: `Cargo.toml`/tag/dist; auditoría extrema 2026-09-18; paridad GeoGebra: nominal completa, profundidad parcial con stubs honestos)

## 1. Vision
Grafito es pizarra geometrica con **Cerebro** (Rust puro) y **Piel** (egui/wgpu).
- Cerebro nunca miente: invariants validados, type-safety, Result/Option.
- Piel solo renderiza &Estado: sin I/O, sin spawn.
- Animador es puente nativo 100% Rust (69 plantillas: 13 canónicas + 52 dominio + 4 extra (incl. sup-laplace-3d) — verificado 2026-09-28 — + ffmpeg-sidecar; `engines/python` eliminado; notebooks de `lab/colab_upload/` eliminados, port del lab a `grafito-lab` en curso).

## 2. DAG de crates (workspace resolver 2)

```
grafito-geometry -+
grafito-complex  -+-> grafito-core --> grafito-command --> grafito-app
grafito-render   -+          |                     ^
grafito-anim     ------------+-> grafito-assistant +
grafito-whiteboard ----------+
grafito-ui       ------------> grafito-app  (Piel)
grafito-profile / pedagogy / plugins / assistant-types / agent
```

- **Cerebro puro** (sin egui, sin wgpu): core, geometry, command, complex, whiteboard, profile, pedagogy, plugins, assistant, assistant-types, agent
- **Puente**: anim (IPC stdio JSON v1)
- **Piel**: ui (tokens, theme, toolbar, assistant, animation), app (app.rs, panels.rs, canvas.rs, render_2d, anim_native/ui)
- **Infra**: packaging/deb, .github/workflows/ci.yml (17 jobs)

## 3. Principios (Slash Commands)

- **/j-space**: todo plan en Markdown antes de codigo (Plans.md, Tasks.md, progress.md, docs/architecture.md)
- **/statem**: flujos como enum Estado + transiciones tipadas que no compilan si son ilegales
- **/rust-design**: newtype (AnimJobId, Resolution, AnimDuration), Result/Option, clippy -D warnings =0, sin unwrap en prod
- **/rust-ui**: UI = fn render(&Estado) -> Frame, I/O en background thread
- **/vibecoder-guide**: error de compilador -> explicacion en lenguaje negocio + menu 2-3 opciones

## 4. Statems

### 4.1 AnimJob (grafito-anim/src/engine.rs)

```
Idle -> Spawning -> AwaitingHello{deadline} -> AwaitingPong{deadline} -> Ready
     -> Running{job_id, deadline} -> Cancelling{job_id} -> ShuttingDown{deadline}
     -> Completed{media_path} | Failed{code,msg} | TimedOut | Cancelled
```
- Transiciones via &mut self con can_submit()==Ready, deadline absoluta, poll 200ms para cancel.
- Validaciones: spawn NUL/workdir/bin path, line_cap 64KB OOM-safe, diagnostics cap 64 lineas, validate_media_path path_escape, Drop no-bloqueante (kill+try_wait).

### 4.2 DocumentLifecycle (grafito-core, ValidatedDocument wrapper — en core; migración de app pendiente, ver `app.rs` TODO)

```
Empty -> Loading -> Validating -> Ready -> Mutating -> Persisting -> Ready
```
- `ValidatedDocument` (`validation.rs:40`): wrapper fail-closed `try_new(doc)` que ejecuta `validate_document` antes de serializar/persistir (auditoría 2026-09-17: `commit`/`restore` validan directo con `validate_document`; el wrapper es la puerta de serialización, no la única llamada). `Document` ya es `BTreeMap` total en sus mapas de dominio y `ConstraintGraph` migró a `BTreeMap/BTreeSet` (save byte-idéntico, test `constraints_serializan_en_orden_determinista`); `semantic_document_baseline` refuerza orden.
- MAX limits: MAX_OBJECT_COUNT 5000, MAX_EXPR_LENGTH 2000, MAX_TRANSFORM_DEPTH 64, MAX_DOCUMENT_SIZE 10M.
- Hash determinista: sorted vars antes de hashear (object.rs:2605 fix), BTreeMap en serializacion (deuda migratoria).
- Transformed try_new valida prepare_function_ast("z") y rechaza "0" singular.

### 4.3 AssistantRuntime (grafito-app/src/assistant.rs)

```
Idle -> Composing -> Thinking -> AwaitingAuthorization -> Animating{job_id} -> Failed | Cancelled
(terminal hoy: Failed; sin Done — verificado AS5; local corre síncrono en UI, resto en thread)
```
> Sin `Verifying`: el preflight de propuestas corre síncrono dentro de `Thinking` (worker, vía `set_proposal_preflight_results` en `grafito-ui/src/assistant.rs`); la variante muerta se eliminó en auditoría en vez de mapear un flag ficticio (test `lifecycle_sin_variantes_muertas`).
- Cada job remoto/proposal/model/agent/anim en thread con CancellationToken y RequestBudget (max_input_chars 8192, max_steps 8, timeout_ms 60000 = 60s — `crates/grafito-assistant-types/src/lib.rs:198-209`; AttachmentLimits max_bytes 512 KiB + max_total_bytes 1 MiB — `lib.rs:245-255`). Excepción: local síncrono en UI. Spark streaming: body cap 256 KiB con parcial `truncated` + aviso (`STREAM_ACCUM_MAX_BYTES` + `push_stream_capped`, `crates/grafito-app/src/assistant.rs:2836-2843`, usos `:3050-3075`); loop agente input cap 48K (`MAX_RESPONSES_INPUT_CHARS`, `crates/grafito-assistant/src/agent.rs:4760-4765`, pin `:6241-6246`).
- Protocolos por modelo (OpenCodeGo) — `crates/grafito-assistant/src/lib.rs:1560` (`remote_protocol`, tabla `go_model_protocol` `:1576-1590`): Chat Completions default (`chat_completion_endpoint`, `:1491`), Responses API `POST {base}/responses` para Muse Spark 1.2/1.3 (`responses_endpoint`, `:1497`; router `uses_responses_api` = `muse-spark`/`gpt-`/`grok-`, `:76-78`; por Chat devuelven 500 instantáneo con cualquier payload, verificado 2026-09-04 — comentario en `:67-70`), Anthropic Messages `POST {base}/messages` para mimo-2.5-vl (`messages_endpoint`, `:1507`; match en `:1578-1586`), Fusion (draft+audit) para `fusion` (`:1578`; `FUSION_AUDIT_MODEL = "deepseek-v4-pro"`, `:144`). Catálogo vigente 2026-09-08 (docs Zen): `muse-spark-1.3`, `muse-spark-1.2` (pagos) + `muse-spark-1.3-contributor-free` (gratis, exige sesión); IDs viejos siguen ruteando por `contains`. El tier gratuito exige sesión → HTTP 400 `MissingSessionID` hace fallback automático a deepseek con aviso de una línea (`is_session_or_account_error`, `crates/grafito-app/src/assistant.rs:9521`, callers `:9500-9545`). Modo agente con tools aún no soportado en Spark → fallback sólo-sesión a deepseek (`should_fallback_agent_spark_to_deepseek`, `crates/grafito-app/src/assistant.rs:9500-9505` + `is_agent_spark_responses_unsupported_error :9492`; remoto `should_fallback_remote_spark_to_deepseek :9538`; hint 500 de Spark en `:9643`). Ruteo completo por tabla `go_model_protocol` (spark/gpt/grok→Responses, minimax/qwen3.6-3.8→Messages, resto→Chat), ver §13.
- Modelos: default `deepseek-v4-flash` (`default_assistant_model`, `crates/grafito-app/src/utils.rs:126`); `qwen3.8-max` / `kimi-k3` sólo como sugerencia en el hint de error 500 de Spark (`crates/grafito-app/src/assistant.rs:9643`, sin tests dedicados); `mimo-2.5-vl` visión (`assistant.rs:55`, `grafito-assistant/src/lib.rs:65`); `fusion` (`lib.rs:66`). Tests wire: `spark_models_route_to_responses_api`, `responses_payload_shape_matches_verified_wire_format`, `responses_wire_uses_bearer_and_response_shape` (`lib.rs:5896,:6108,:6303`); fallback sesión (`should_fallback_*`, `assistant.rs:9488-9545`, test `:10739`).
- I/O nunca en UI thread; UI solo renderiza &AssistantState.
- **Gate de contexto (rediseño F24 2026-09-17)**: el TEXTO de una respuesta remota NUNCA se descarta si cambia el documento/foco mientras el proveedor piensa (aviso Info de una línea); las propuestas se re-verifican contra el documento ACTUAL en el preflight (`start_remote_proposal` captura `preflight_revision/digest/focus`); si cambia durante la verificación, se descartan las propuestas con aviso honesto y el texto se publica (`propuestas_rancias_se_descartan_sin_perder_el_texto`). El Apply sigue fail-closed por `PlanBasis` (`crates/grafito-command/src/assistant_plan.rs:216`, `validate_basis`). Clarificación `ask_user` usa el `call_id` REAL del wire vía `AgentEvent::Clarification` (elimina la heurística `ask_user-{len}`).
- **Presupuesto de prompt**: `bounded_context_prompt` (grafito-assistant/src/lib.rs) trunca el bloque de objetos/variables al presupuesto con nota honesta (un doc denso ya no rompe la consulta con `assistant input exceeds`); el bloque de documento/web viaja entre `<datos_no_confiables>` con directiva de sistema (anti prompt-injection).
- **Telemetría local opt-in**: `GRAFITO_USAGE_LOG=1` (o ruta) → JSONL `$XDG_DATA_HOME/grafito/assistant_usage.jsonl` con kind/provider/model/chars/tokens reales/ms/stale_context, sin contenido ni claves (`grafito-app/src/usage_log.rs`).
- **F18 streaming tipado + razonamiento + tokens**: el preview monolítico se retiró; el motor emite `StreamDelta { Reasoning | Text | Status }` por `stream_progress_sender` (sufijos por flujo, `try_send` best-effort sobre canal 128) — `crates/grafito-assistant/src/lib.rs` (`StreamDelta`, `stream_progress_sender`, `read_responses_sse_stream`). **Los tres protocolos streamean**: Responses, Chat y Anthropic Messages (`content_block_delta` con `text_delta`/`thinking_delta`, `message_start`/`message_delta` con usage mergeado y `message_stop`; `request_anthropic_completion_streaming`; mimo-2.5-vl ya no espera muda). `reasoning_content` / `response.reasoning_*` alimentan el bloque plegable por turno (`ConversationTurn.reasoning` + `reasoning_ms`, `grafito-assistant-types`; cap `MAX_CONVERSATION_REASONING_CHARS` 8192) y jamás entran al texto final ni al historial remoto. `usage` real (input/output/reasoning/cached/total) se parsea en Responses (`response.completed`), Chat (chunk final con `stream_options.include_usage`), Anthropic y Fusion (`parse_token_usage`), viaja en `RemoteCompletion.usage` y se muestra por turno + sesión; sin datos se oculta (nunca se inventa). Modo razonador opt-in: `AssistantRequest.reasoning_effort` → wire `reasoning_effort` (Chat) / `reasoning.effort` (Responses) con degradación honesta si el proveedor responde 400/422 (`strip_reasoning_knobs` + reintento único en `post_json_with_reasoning_fallback`) + directiva `REASONING_BINDING_DIRECTIVE` en el system prompt (cubre el modo agente). Buscar en internet opt-in: `AssistantRequest.web_search` → worker pre-flight `web::web_search` (DuckDuckGo HTML + fallback Instant Answer, sin claves, 5 resultados, snippet 400, timeout 8s, `web.rs`), contexto inyectado al prompt (cap `MAX_WEB_CONTEXT_CHARS` 4096 contado al input) y tool `web_search` opt-in en el agente (`agent.rs`; fuera de `all_safe_tool_schemas` a propósito). UI: disclosure por turno con auto-expand mientras piensa y auto-colapso al responder (`reasoning_open` por id; header `secondary` en vivo → `tertiary` quieto), chip de tokens con hover, total de sesión en el header, botón ghost **Copiar** en el turno cerrado, chips **Razonar**/**Buscar** (ghost pill `composer_mode_chip`, piso táctil 24px) en el composer y checkboxes en Configuración (`AssistantUiAction::ReasoningModeChanged/WebSearchChanged`, persistidos en `AppConfig.assistant_reasoning_enabled/assistant_web_search_enabled`). Fix visual "chat oculto": `scroll_to_cursor` forzado cuando el transcript sigue el fondo, tolerancia 2px, `ASSISTANT_TRANSCRIPT_MIN_HEIGHT` 56px que capa el composer en viewports chicos. Perf: `AssistantBlocksCache` devuelve `Arc<Vec<...>>` (hit sin clonar), el reveal no clona el contenido del último turno y `humanize_prose_text` devuelve `Cow` (sin copia cuando no hay identificadores de control).

### 4.4 Interval/ExprEval (grafito-geometry)

```
Raw -> Parsed -> Validated -> Evaluated | Failed
```
- Cache LRU 128, midpoint lo+(hi-lo)/2, safe_sample valida finite y MAX_SAMPLES 100k.

## 5. Type-Safety

- **AnimJobId**: newtype String try_new regex ^[A-Za-z0-9_-]{1,64}
- **Resolution**: try_new 64..=4096, as_tuple()
- **AnimDuration**: try_new 0.1..=60s (P0.1 long-form; `protocol.rs:263`), as_millis()
- **AnimParams**: validate template/concept/spec + params finite + duration/resolution
- **ExportFormat**: Gif/PngSequence/Mp4
- **WireMessage**: Hello{protocol_version, capabilities} | Progress | Result | Error{code,msg} | Pong ; versionado ANIM_PROTOCOL_VERSION=1
- **Matrix**: try_* con MAX_MATRIX_DIMENSION 1000, MAX_ELEMENTS 1M, debug_assert en zeros/identity, SVD singular_value_tolerance

## 6. Generador de Animaciones v2

### 6.1 Protocolo (protocol.rs + render nativo en anim_native.rs; motor `engines/python` eliminado)
- JSON v1 sobre stdio, line_cap 64KB, backpressure sync_channel(128)
- Histórico — motor python retirado (`engines/python` eliminado del árbol): sandbox AST con whitelist + denylist, placeholder hex estable, `manim_is_available` con cache, render con Manim por template con `media_path` verificado, progreso 30->60->100, fallback a placeholder con log en stderr. Hoy el render es nativo (`anim_native.rs`, 69 plantillas: 13 canónicas + 52 dominio + 4 extra (incl. sup-laplace-3d) — verificado 2026-09-28) + ffmpeg-sidecar con `FfmpegMissing` honesto.

### 6.2 Engine (engine.rs)
- Config: command argv, working_dir, idle_timeout 8s, job_timeout 90s, line_cap 64KB
- Metodos: spawn, wait_ready (hello+pong), submit (solo Ready), recv_event (filtra job_id), shutdown (cooperativo 8s), cancel() (Running->Cancelling->Cancelled), run_job (efimero con cancel poll 200ms, deadline absoluta, validate_media_path), diagnostics(), state()
- Tests: 5 tests en engine::tests (stub POSIX sh — F0 sin Python, health_check, media_path reject, propagate errors, timeout, statem reject)

### 6.3 Nativo fallback (anim_native.rs)
- render_native_animation_frames (parabola tangente), render_pitagoras_frames, render_integral_frames, render_taylor_frames, render_conformal_frames, dispatcher render_anim_by_template(template,w,h)
- Tests: 5 tests (bounded, distinct, integral, taylor, conformal, dispatcher fallback)
- UI (anim_ui.rs): AnimPreviewState {template, concept, progress, status, media_path, frames, source_turn}, draw_anim_panel con bar_width 420, max_height 84, textura id unico por frame, sin I/O, build_anim_params con Resolution/Duration

## 7. Piel (grafito-ui / grafito-app)

- **Tokens** (grafito-ui/src/tokens.rs): TYPE_XS..XXL (11..28, ratio 1.25), SPACE_XS..XXL (4..32, base 4), RADIUS_SM..LG, ICON_SM..XL — unica fuente de verdad. Nordic Slate F15.1: N0 `#131518` canvas / N1 `#1B1F26` chrome / N2 `#232935` / N3 `#2B323E` / borde `#2F3642` 1px 40-60% mid 0.50, sin negros planos (`tokens.rs:656-672`, `theme.rs:173-180` + `DARK :465-483`, pins `tokens.rs:1087`/`theme.rs:1065,1117`).
- **Assistant panel** (`crates/grafito-ui/src/assistant.rs:102-107`): SidePanel 300..520 (default 400) o TopBottomPanel bottom-sheet cuando el viewport < 740px (`ASSISTANT_SIDE_PANEL_MIN_VIEWPORT_WIDTH = 440 + 300`, `:105-107`; `assistant_uses_bottom_sheet`, `:4923-4924`; draw `SidePanel/TopBottomPanel`, `:4854-4874`), composer con clamp 88..260, sin ScrollArea envolvente (fix overflow), wrapping, clip.
- **App shell** (grafito-app/src/app.rs 10615L, assistant.rs 15299L, panels.rs 11387L + spreadsheet.rs 1408L + sheet_layout.rs 9999L tras el split F14.4.4 — puros sin egui, wiring `lib.rs:52-54`, import `panels.rs:7` —, app/src/anim_native.rs 17301L — el dispatcher vive en grafito-app, ya no en grafito-anim —, ui/assistant.rs 24571L, command/command_registry.rs 12063L y command/commands.rs 35148L, medidos 2026-10-05 con `wc -l`): eframe::App::update dispatch (god function, deuda P1), GrafitoApp 136 campos (god object), `MAX_UNDO` 50 + `MAX_UNDO_BYTES` 50 MiB con `VecDeque<Document/ChangeSet>` (`pop_front` O(1), `Vec` previo era O(n) shift — corregido), `controllers.rs` reales `DocumentController/ViewController/AssistantController` con `VecDeque` + tests (F14: `GrafitoApp` ya delega por puente `from_parts`/`into_parts` en snapshot/undo/redo/replace/view/examen; wiring fino P2), `AssistantTurnState` en `assistant_jobs.rs` (derivado del runtime con `derive_from`; con consumidor productivo `assistant_is_busy()` usado por `attach_assistant_image` desde F14.4.2 — wiring fino P2 queda), ViewMode/Perspective/CanvasMode redundancia (deuda P1), repaint intervals 150ms settle, 33ms multidimensional 30Hz, 16ms whiteboard 60Hz. Animación sin Python (`engines/python` eliminado del árbol): 69 plantillas nativas (13 canónicas + 52 dominio + 4 extra (incl. sup-laplace-3d), sync 69↔69↔69 en `grafito-anim/src/protocol.rs::CANONICAL_TEMPLATES` ↔ `grafito-app/src/anim_native.rs::NATIVE_TEMPLATES` ↔ `grafito-app/src/anim_ui.rs::PLANTILLAS_COMBO`, + 52 dominio linalg/am1/am2/edo/stats/3d/4d/graphs/chaos — verificado 2026-09-27) + MP4 vía ffmpeg-sidecar (`grafito-app/src/anim_native.rs:716,736,865,942`, `FfmpegMissing` honesto sin `ffmpeg` en PATH). CAS experto opt-in: feature `cas-nativo` con `alkahest-cas 3` (`grafito-assistant/Cargo.toml:44`).
- **Atajos verificados** (handlers en `grafito-app/src/shortcuts.rs` — F14 los movió de `app.rs`; menús `draw_file_menu ui.rs:249` + `draw_view_menu :397` + `draw_top_bar :821`; etiquetas toolbar en `grafito-ui/src/toolbar.rs:30-177`): Ctrl+N/O/S + Ctrl+Shift+S archivo (`lifecycle.rs:20-31`), Ctrl+Z/Y deshacer/rehacer (+Shift en Ctrl+Y = herramienta YIntercept, `shortcuts.rs:22-36`), Supr eliminar, Esc cancelar (overlay-aware con `consume_key` + fallback único, F14), F1-F6 herramientas 2D (`shortcuts.rs:41-67`), F8 Esfera 3D + F9 Cubo 3D (`shortcuts.rs:69-78`), R/E/I/X/N/S/Y/V/M/G herramientas sin modificadores (`:79-122`), Ctrl+A Analizar (`:124-125`), Shift+L/K/J toggles log X/Y/ambos (`:133-144`), G snap (`:146-149`), Ctrl+K paleta (`:186`), Ctrl+T tema (`shortcuts.rs:203`), Ctrl+P Lápiz + Ctrl+E Borrador (`shortcuts.rs:213,:218`), Ctrl+Shift+1..9,0 perspectivas (10, `shortcuts.rs:163-184`). Cero fantasmas desde BUILD 2026-09-04.
- **Responsive shell**: rail 68px (`RAIL_WIDTH`, `tokens.rs:183`; items 60px insetados 4px por lado para que el borde no clipee; `app/src/ui.rs:1032,1059`) visible sólo en Medium/Wide (≥1360, `BREAKPOINT_COMPACT` `tokens.rs:159`; `CANVAS_FOCUS_MAX_WIDTH`/`WIDE_MIN_WIDTH` + derivación Compact/Medium/Wide `app/src/lib.rs:427,430,442-447`) — colapsado en Compact (≤1360, `is_compact_viewport` `tokens.rs:525-528`; `COMPACT_TOP_CHROME_MAX_WIDTH` `app/src/ui.rs:32`); drawer derecho 292..440 con clamp (`clamp_drawer_right_width`, `tokens.rs:173-178,546`; dock 3D `draw_geometry_utility_dock` `app/src/ui.rs:1383-1388`; Inspector `panels.rs:675`); panel izquierdo min 240 + max 45% viewport (`PANEL_LEFT_MIN`, `PANEL_LEFT_MAX_FRACTION`, `tokens.rs:165-171`; call sites `panels.rs:1131,1602,2418` + `algebra.rs:479,619`).
- **Onboarding** (campo `show_onboarding` `app.rs:1960-1961`, gating `!config.onboarding_completed` en `new()` `:2692`; `OnboardingChoice` `:9263` + `draw_onboarding_window` `:9304` + `apply_onboarding_choice` `:9519-9532` con `onboarding_choice_persists` `:9532`; `utils.rs:57-59`): bienvenida first-run fullscreen (`Area` `Foreground`, columna `WELCOME_COLUMN_WIDTH` —ver comentario `:9304-9345`—, 1 sola pantalla `WELCOME_PAGES=1`, copy `welcome_page_copy` (qué es Grafito + Mili); botones "Probar ahora" (siembra `y=x^2` + abre a Mili con el saludo) y "Empezar vacío", checkbox "No mostrar de nuevo"; Skip/Esc = empezar vacío); el copy "18 grupos" (5/8/18) vive en `bullet_university` (`i18n.rs:249`) con claves en 6 idiomas pero sin uso — deuda menor honesta; persiste `onboarding_completed=true` SOLO `NeverShow` ("No mostrar de nuevo"), el resto solo oculta en sesión.
- **Paleta de comandos** (`grafito-ui/src/command_palette.rs`): fuzzy subsecuencia sin tildes (`fuzzy_match`, `:224-251`), bilingüe es/en (`filtered_commands`, `:275-296`), footer en español con conteo "N de M · ↑↓ navegar · Enter abrir · Esc cerrar" (`:394-403`), 15 acciones UI en español con clave inglesa estable (`UI_ACTIONS`; test actualizado).
- **Estudio FSRS + calendario + breadcrumb vault** (F16.1/F17.1/F17.2/B-A11) — HISTÓRICO, módulos eliminados 2026-10-06 (reference; el código ya no existe): motor puro `grafito-study/src/fsrs.rs` (`fsrs_init :68`, `due_days_in_month :183`, `retention_after` R(t,S)=(1+19/81·t/S)^-0.5, `due_day` saturante i32) + fechas Hinnant sin chrono `study/src/calendar.rs:355,379`; Piel `study_views.rs:113` `FSRS_DUE_AMBER` (250,204,21) + `:118` `fsrs_due_in_cell` + dot 6px fila propia `:1204` con hover en `cal_month_grid :1037` vía `draw_calendar :853` con param `fsrs_due` (wiring `app.rs:5098-5112`, deck real B-A27 `fsrs_deck` + `MarkReviewed` 1-4 en el modal `study_views.rs:1360-1373`); top bar `vault.rs:750` `TOP_BAR_BREADCRUMB_MAX_CHARS` 64 + `:757` `top_bar_vault_breadcrumb_label` pura (None-sin-selección, colapso medio `Vault / … / cola`) + `:778` `draw_top_bar_vault_breadcrumb` solo-lectura con hover ruta plena (wiring `app/src/ui.rs:893`).

## 8. Presupuestos y Limites

| Dominio | Constante | Valor | Ubicacion verificada |
|---------|-----------|-------|----------------------|
| Documento | MAX_DOCUMENT_SIZE_BYTES | 10M | validation.rs |
| Documento | MAX_OBJECT_COUNT | 5000 | validation.rs |
| Expr | MAX_EXPR_LENGTH | 2000 | validation.rs |
| Matriz | MAX_MATRIX_DIMENSION | 1000 | matrices.rs |
| Matriz | MAX_MATRIX_ELEMENTS | 1M | matrices.rs |
| Cache | MAX_COMPILED_EXPR_CACHE | 128 (medido 2026-09-23: 512/1024 dan Δ<3% con working set realista W=64 — negativo documentado en `docs/profiling.md` §2.11) | expr.rs:978-996 |
| Interval | MAX_SAMPLES | 100k | interval.rs |
| Interval | MAX_INTERVAL_PREC_DIGITS | 15 (`prec > 15` satura; `Interval::new(prec, lo, hi)` redondea outward real; `prec = 0` preserva el comportamiento previo) | interval.rs:9,35-55 |
| Anim | MAX_CANVAS (retirado con `engines/python`; vigente: Resolution 64..=4096) | 4096 | anim/src/protocol.rs:228 |
| Anim | MIN_CANVAS (retirado con `engines/python`; vigente: Resolution 64..=4096) | 64 | anim/src/protocol.rs:228 |
| Anim | MAX_EXPR_LEN (retirado con `engines/python`; vigente: MATH_EXPR_MAX_BYTES 200) | 500 | anim/src/guion.rs:61 |
| Anim | MAX_NODES (retirado con `engines/python`, sin equivalente nativo) | 200 | — |
| Anim | line_cap | 64KB | engine.rs |
| Anim | diagnostics cap | 64 lineas | engine.rs |
| Anim | AnimDuration | 0.1..=60s (P0.1 long-form) | anim/src/protocol.rs:263 |
| Anim | Resolution | 64..=4096 por lado | anim/src/protocol.rs (tests :3366-3371) |
| Anim | VIDEO_LONGFORM_MAX_FRAMES | 1500 (Mp4/Webm; Gif/PngSequence cap 64) | anim/src/protocol.rs:341,345,438 |
| Anim | PREVIEW_SHORT_MAX_FRAMES (GIF) | 64 | anim/src/protocol.rs:341 |
| Anim | LONGFORM_CHUNK_MAX_BYTES | 64 MiB (streaming por chunks, sin Vec total) | anim/src/protocol.rs:349,364-389 |
| Anim | MAX_TIMELINE_DURATION_MS | 60_000 (60 s) | anim/src/protocol.rs:931 |
| Anim | AudioTrack offset_ms / gain | 0..=60000 / 0.0..=2.0 finita | anim/src/protocol.rs:468,470,503-543 |
| Anim | CAPTION_MAX_OUTPUT_BYTES (SRT/ASS) | 256 KiB (PREVENTIVA: `empuja_acotado`/`empuja_karaoke` cortan al cruzarla sin materializar la salida; el peor caso de wire 2000×500×200 ≈ 200 MB aborta con el acumulado en el tope) | anim/src/captions.rs:34,350-377 |
| Anim | PLAYER_MAX_TOTAL_BYTES (set jugado) | 64 MiB lógicos estimados (`estimate_placed_bytes`); el peor caso compuesto (96 frames × 32 capas × 4096 pts ≈ 192 MB) da `Err` honesto en `try_play` y corte en `play` (antes clonaba la escena base CADA frame sin chequeo real) | anim/src/player.rs:307-372 (test `:1977`) |
| Anim | protocolo v1 con rango real | 1..=2 tolerado (v2 degrada a v1 con campos nuevos ignorados; fuera del rango → `UnsupportedVersion{min:1,max:2}` tipado, antes `== 1` duro) | anim/src/protocol.rs:35-42,1213-1222 |
| Anim | MATH_EXPR_MAX_BYTES | 200 bytes (unidad unificada con `grafito-pedagogy::verify_math_expr`; antes chars y una expr multibyte de 200 chars pasaba el pre-gate para caer en el gate real) | anim/src/guion.rs:61,224 |
| Anim | VOICEOVER_MAX_PALABRAS (por paso) | 40 | anim/src/guion.rs:64 |
| Anim | SHORT_MIN/MAX_PALABRAS (short total) | 110 / 130 | anim/src/guion.rs:66,68 |
| Anim | run_command texto | no vacío, ≤2000 chars, sin NUL, una línea | assistant-types/src/lib.rs:17 + assistant/src/agent.rs:2381-2399 |
| Assistant | RequestBudget max_input_chars | 8192 | assistant-types/src/lib.rs:201 (+validate cap :214) |
| Assistant | RequestBudget max_output_chars | 2048 | assistant-types/src/lib.rs:202 (+validate cap :217) |
| Assistant | RequestBudget max_steps | 8 | assistant-types/src/lib.rs:203 (+validate cap :220) |
| Assistant | RequestBudget timeout_ms | 60000 (60s) | assistant-types/src/lib.rs:204-206 (+rango 100..=120000 :223) |
| Assistant | AttachmentLimits max_bytes | 512 KiB | assistant-types/src/lib.rs:248 |
| Assistant | AttachmentLimits max_total_bytes | 1 MiB | assistant-types/src/lib.rs:251 |
| Assistant | AttachmentLimits max_pixels / max_total_pixels | 1 MiP / 2 MiP | assistant-types/src/lib.rs:249,252 |
| Assistant | AttachmentLimits max_attachments | 2 | assistant-types/src/lib.rs:250 |
| Assistant | ConversationTurn reasoning cap | 8192 (`MAX_CONVERSATION_REASONING_CHARS`, espeja `REASONING_MAX_CHARS` del motor) | assistant-types/src/lib.rs + assistant/src/lib.rs |
| Assistant | Web search | 5 resultados / snippet 400 / query 256 / contexto 4096 / timeout 8s, sin claves | assistant/src/web.rs |
| Assistant | Fallback razonador | 400/422 con "reasoning" → 1 reintento sin el campo | assistant/src/lib.rs (`strip_reasoning_knobs`, `post_json_with_reasoning_fallback`) |
| Assistant | tools seguras (`all_safe_tool_schemas`, assistant) | 39 (3 base + 8 pedag + 24 math + 2 harness1 + 2 harness2) | assistant/src/agent.rs (math 24 = `math_tool_schemas`, `:3549`; `all_safe` `:4445`; asserts `:8023,:8026`; harness2: search_topp39 + export_dimacs) |
| Assistant | tools seguras (`all_safe_tool_schemas`, agent hoja) | 9 (3 base + 6 pedag, sin math/harness) | agent/src/tools.rs:3073 |
| Tex | TEX_INPUT_MAX_BYTES | 8 KiB | tex/src/lib.rs:37 |
| Comandos | COMMANDS registrados | 764 (`command!(`) | command/src/command_registry.rs (blindaje `registry_counts_match_documented_architecture` == 764, `:11777`; Harness Fase A suma 7 Discreta: UnitPairs/DistinctDistances/UnitGraphEdges/ChromaticCheck/HalvingEdges/EmptyTriangle/Topp39Scan; Frente fantasma+nuevos 2026-09-23: +12 = 7 con handler sin spec (ImproperIntegral/SeriesSum/SequenceLimit/DoubleIntegral/LagrangeMultipliers/MeanValueCheck/SubspaceSum) + 5 nuevos delegando al motor (PolyGCD/Resultant/Residue/PrincipalPart/StepByStep); Frente fantasma-2 2026-09-23: +71 visibles (Matrices +20, AM2 +13, Crear +7, AM1 +7, CAS +7, 3D +7, Construir +3, Dinámica +3, Estadística +2, Análisis +1, Complejos +1 — `Image` NO se registra: stub sin modelo persistente de imagen, blindado en `orphan_detection_reports_counts` `:11404`; deltas históricos `:11832-11946`); Olas G1/O1/O3/O5/O6 2026-10-04: 747→764; EDO-con-pasos 2026-10-15: +2 CAS con spec (SolveODE1/SolveODEHomog, pin test 852/815/830) (medido `grep -c "command!"` = 764; `bridge.rs:9,:159` y `protocol.rs:129,:191` ya dicen 764) |
| Comandos | palette-visible | 727 (37 ocultos) + 15 acciones UI = 742 en paleta | command_registry.rs + grafito-ui/src/command_palette.rs (R3.1: Rename stub→visible; 3D-A2: +Vista3D; P0.2: +41 álgebra; P0.3: +7 complejo; P0.4: +3 demostración; P0.5: +8 optimización+gráficos; P1: +82 listas/estadística/probabilidad; P1b: +1 List persistible; P2: +25 medidas/geometría; P3b: +11 scripting + Execute real; P4: +105 CAS/listas/geometría/stats/display; P5: +18 cierre nominal (Slope/SetValue/GroebnerLexDeg/SD/SampleSD/SampleVariance/SetSeed/Seed/Turtle*); Harness Fase A: +7 Discreta; Fantasma+nuevos 2026-09-23: +12 — `Gauss` pierde el alias `residue` (canónico nuevo `Residue`; `residuos` queda, `command_registry.rs:1973-1991` Gauss + `:2004-2013` Residue canónico); Fantasma-2 2026-09-23: +71; Olas G1/O1/O3/O5/O6 2026-10-04: paleta 710→727; EDO-con-pasos 2026-10-15: +2 CAS (SolveODE1/SolveODEHomog) —detalle por frente en `:11832-11946`, pin `registry_counts_match_documented_architecture` `:11777` 764/727/25 en verde) |
| Comandos | categorías visibles | 25 (`VALID_CATEGORIES`, `command_registry.rs:10193-10219`; sin cambio en el frente 2026-09-23) | command_registry.rs (G-F audit) |
| Toolbar | ToolGroupId / UNIVERSITY | 18 (PRIMARY 5, SECONDARY 8) | grafito-ui/src/toolbar.rs:192 (enum) + PRIMARY `:355` / SECONDARY `:364` / UNIVERSITY `:376` (+pins `:2169,:2256`; F3a 17→18) |
| Toolbar | ToolGroupId / ALL_GROUPS | 15 clásico intencional (UNIVERSITY 18 suma Dynamics/ThreeD/FourD; disclosure progresivo, no bug) | grafito-ui/src/toolbar.rs:325 (ALL_GROUPS) + comentario `:301-306` |
| Toolbar | Tool variantes | 89 | grafito-ui/src/lib.rs `pub enum Tool` (Ola 1.6 suma `Text`; contorno complejo suma `ComplexContour`) |
| App | Perspectivas | 10 (Ctrl+Shift+1..9,0) | grafito-app/src/lib.rs:90-111 + app.rs:4236-4242 |
| Workspace | crates | 20 members + root; 21 dirs en `crates/` (vault/study/tabs fuera 2026-10-12; solver Photomath local en `app/solver_nl` + `StepByStep` con fence) | `Cargo.toml` members (app, agent, anim, assistant, assistant-types, classroom, command, complex, core, geometry, ggb, lab, mcp, pedagogy, plugins, profile, render, tex, ui, whiteboard) + `crates/grafito-release-tests/` no listado |
| Lab MCP | `grafito-mcp` tools/resources | 71 tools (9 lab incl. execute + 37 proxedas assistant vía `SafeGrafitoDispatcher`; 2 Lean sidecar; 2 policy dream-loop; 1 gpu_probe WGSL verificado RADV+llvmpipe; 2 Colab offload; 5 estudio; 2 CAD; 11 vault; `web_search` y harness-2 viejo excluidos) + 4 resources + 5 templates por stdio | `crates/grafito-mcp/src/{lib,tools,protocol,ledger,sat,bridge,lean,policy,gpu,colab}.rs` (total dinámico verificado en `protocol.rs:497-513` y `tests/stdio_contract.rs:63-72`; 37 proxedas = all_safe 39 − harness-2 viejo + web_search en `bridge.rs:19-37,281-299`; MCP 2024-11-05; ledger XDG `lab_ledger.jsonl` + `lab_colab.jsonl`; `execute_command` multi-step ≤32 con catch_unwind; SHA-lock anti-spec-gaming; SplitMix bit-idéntico Colab pineado; guardia PII; topes 100k pts/2M aristas/500k vars/50 MiB/100k seeds/SAT 24h/Lean 600s/Colab 200k pts) |
| Colab App | pareo 1-clic + jobs | Dueño ÚNICO del proxy `colab-mcp` (opencode.json NO lo trae: doble pareo confunde); ventana Herramientas > Colab Pro… con estado, log, ejecutor descubierto, jobs e import | `crates/grafito-app/src/colab_link.rs` (cliente MCP mínimo por stdio con timeouts, mock por trait en tests) + `colab_panel.rs` (ventana; cero I/O en dibujado, worker en background); i18n 1463×6 (purgas 2026-10-12 −176 study/vault/center + 2026-10-13 −8 onboarding/center + 2026-10-14 +2 racha/veredicto + 2026-10-14 +3 empty cálido/GIF rápido) con 29 claves `colab.*` + 142 `profile.*` B-A41/B-A42/B-A43/B-A44/B-A45/B-A46/B-A47/B-A48/B-A49/B-A50/B-A51 + 110 `settings.*` B-A52/B-A53/B-A54/B-A55/B-A56/B-A57/B-A58/B-A60 + 10 `anim.export.*` B-A61 + 6 `anim.export.*` B-A62 + 22 `anim.export.*` B-A63 + 4 `anim.export.*` B-A64 validación export + 6 `anim.job_*` B-A66 progreso del job + 15 `anim.media_*` B-A67 player/header/export + 1 `anim.media_*` B-A68 preparando + 2 `assistant.*` B-A68 saludo/mora + 4 `assistant.math.*` B-A69 math-tex + 5 `teach.*` B-A71 marcos + pizarra + 7 `teach.*` B-A73 ejercicio + overlay nav + 14 `teach.*` B-A74 feedback + ejercicio resto + overlay nav/hovers + 4 `teach.*` B-A75 progreso + distractores + 5 `teach.*` B-A76 errores del job + 2 `teach.*` B-A78 toast + hint morph + 1 `teach.*` B-A79 overlay reserva + 2 `trace.*` B-A94 hover + vacío + 2 `assistant.error.*` B-A95 reintento + reinicio + 7 `trig.*` B-A96 chrome + 6 `trig.*` B-A97 modos + play/pausa + 14 `trig.*` B-A98 descripciones + prompt + lecturas + identidades + 6 `trig.prompt_err_*` B-A161 errores prompt + 22 `assistant.*` B-A101 turno/propuesta/auth/clarify/plan + 1 `vault.*` (título backlinks) + 2 `center.*` (whiteboard + reopen) + 7 `stats.*` B-A105 panel estadística + 13 `motion.*` B-A106 tarjeta movimiento 4D + 13 `wc.*` B-A107 tarjeta W-C + 6 `wc.section_*` B-A132 headers W-C + 15 `inspector.*` B-A108 panel Inspector base + 20 `inspector.poly*` B-A109 sección 4D/N-D + 13 `inspector.*` B-A110 visibilidad/kind/familia/ejemplo/medida + 8 `inspector.*` B-A111 prefijo Etiqueta + estilos + 14 `inspector.*` B-A112 primitivas 3D + sliders + 10 `inspector.*` B-A113 vista previa implícita + 8 `picker.*` B-A117 color picker + 7 `stats.*` B-A119 empty + puente caso/cuantil + 13 `stats.*` B-A120 caso + tiles + momentos + 8 `stats.*` B-A121 cuantil chrome + 2 `stats.*` B-A122 cuantil resultado + 10 `stats.*` B-A123 ordenar+graficar + 4 `stats.*` B-A124 resumen+asistente + 9 `view.*` B-A128 Vista General+Ejes + 14 `view.*` B-A129 Vista 3D+Capas+AltaPrecisión + 10 `view.*` B-A130 Capas dinámicas + hints + indicate + 8 `view.*` B-A131 parámetro vivo + 2 `stats.*` B-A133 headers prob + 1 `stats.*` B-A134 header Inferencia + 12 `colab.*` B-A136 panel Colab + 7 `colab.status_*` + 5 `colab.import_*` B-A137 status + notas + 10 `algebra.*` B-A138 drawer + vacío + hovers + 8 `algebra.*` B-A139 entrada + teclado + anim + a11y + 8 `algebra.var_*` B-A141 sliders + rango + 2 `algebra.var_*` B-A142 error + readout + 10 `algebra.summary_*` B-A143 resúmenes con notación + 13 `algebra.poly_*/cone/cylinder_infinite` B-A144 politopos + cono/cilindro + 7 `ctx.*` B-A145 menú contextual + error + 2 `ctx.*` B-A174 menú 3D + 9 `algebra.fit_kind_*` B-A146 kinds de ajuste + 12 `stats.regression_*` B-A147 panel Regresión + 12 `stats.regression_*` B-A148 dinámicos Regresión + header import + 7 `stats.regression_model_*` B-A149 etiquetas modelos + 10 `stats.series_*` B-A150 fila serie + 8 `stats.sheet_*` B-A151 hoja chrome + 10 `stats.sheet_*` B-A152 hoja tablas + 7 `stats.quick_*` B-A153 toasts quick-result + 11 `stats.sheet_summary_*`/`sheet_linked_long_hint` B-A154 resumen card + 16 `stats.prob_param_*` B-A155 params prob + 22 `stats.prob_dist_*/kind_*/point_*` B-A156 selector dists + 8 `stats.prob_plot_*` B-A157 plot messages +1 `stats.prob_plot_bar` S2 + 8 `inspector.*` B-A159 apariencia 4D + cierres + 12 `stats.quick_chip_*/quick_tip_*` B-A163 chips inferencia + 2 `stats.sheet_goto_*` B-A164 goto hoja + 4 `stats.sheet_nav_*` B-A166 flechas hoja + 1 `stats.sheet_chip_hint` B-A167 chip hoja + 10 `export.*` B-A169 sección archivo + 12 `export.*` B-A170 resto + 3 `export.*` B-A171 toasts copiado + 2 `ctx.*` B-A175 canvas a11y + 16 `cas.*` B-A177 panel CAS + 9 `exam.*` B-A180 examen resto + 1 `exam.*` B-A181 vista bloqueada + 9 `recovery.*` B-A183 modales recovery + 11 `custom.dlg_*` B-A184 diálogo custom-tool + 13 `rail.*` B-A185 rail sidebar + 14 `shell.*` B-A186 shell chrome + 9 `save.*` B-A187 diálogo save + 57 `hint.*` B-A188 hints barra + 2 `shell.*` B-A188 ejemplo + 16 `dynamics.*` B-A189+B-A190 Atractores + Parámetros + 1 `rail.*` B-A197 tab Aula + 5 `classroom.*` B-A198 panel externo + 16 `classroom.*` B-A199 panel interno + 3 `classroom.outbox_*` B-A201 outbox + 3 `exam.*` B-A206 bloqueo/oculto + 3 `assistant.empty.suggest_*` caminos empty-state 2026-10-14 + 2 `toast.first_*` celebración contenida 2026-10-14 + 1 `assistant.header.streak` racha visible + 1 `exam.modal_verdict` veredicto cálido 2026-10-14 + 1 `algebra.empty_action` siembra ejemplo + 2 `anim.export_quick*` GIF rápido 2026-10-14 + 11 `classroom.*` roles Aula 2026-10-15 + 5 `assistant.steps.read/stop*` TTS Leer 2026-10-15 + 4 `assistant.composer.dictate*` mic STT 2026-10-15 |
| Complejo | Contorno a mano (cuadratura) | ≤ `MAX_CONTOUR_POINTS` 10 000 puntos; ≥ 2 puntos; cuadratura ≤ `MAX_QUADRATURE_SEGMENTS` 2 048 segmentos × Gauss–Legendre 16 | complex/src/math/complex_calculus.rs (`contour_integral`, `ContourAccumulator`, `circle_contour_integral`) |
| Complejo | Guías del contorno | cierre automático ≤ 12 px, muestras cada ≥ 1.5 px, estabilizador 0.0–0.8, suavizado Catmull-Rom ×6 | app/src/complex_contour.rs (`CLOSURE_RADIUS_PX`, `MIN_SAMPLE_STEP_PX`) |
| UI | BREAKPOINT_COMPACT | 1360 | tokens.rs:159 (is_compact_viewport :525-528) |
| UI | PANEL_LEFT_DEFAULT | 260 (min 240, max 45% viewport via PANEL_LEFT_MAX_FRACTION) | tokens.rs:166 (pin `:772`, usos `app/src/panels.rs:1601`, `app/src/algebra.rs:478,618`) |
| UI | PANEL_LEFT_MIN | 240 | tokens.rs:168 (pin `:773`) |
| UI | PANEL_LEFT_MAX_FRACTION | 0.45 (clamp + panel_left_max_width) | tokens.rs:171 (pin `:774`, helper `:534-535`, clamp `:541`) |
| UI | DRAWER_RIGHT_DEFAULT | 344 (min 292, max 440) | tokens.rs:174 (pin `:790`, uso `app/src/ui.rs:1386`) |
| UI | DRAWER_RIGHT_MIN | 292 | tokens.rs:176 (pin `:791`) |
| UI | DRAWER_RIGHT_MAX | 440 | tokens.rs:178 (pin `:792`, clamp `:546-547`) |
| UI | RAIL_WIDTH | 68 | tokens.rs:183 (pin `:800`, uso `app/src/ui.rs:1059`) |
| UI | Nordic Slate F15.1 | N0 `#131518` canvas / N1 `#1B1F26` chrome / N2 `#232935` / N3 `#2B323E` / borde `#2F3642` 1px 40-60% mid 0.50, sin negros planos | tokens.rs:656-672 + theme.rs:173-180,465-483 (pins `:1087`, `:1065,1117`) |
| UI | TOP_BAR_HEIGHT | 48 | tokens.rs:555 (pin `:746`, uso `app/src/ui.rs:842`, `PALETTE_POS_Y :445`, `TOAST_TOP_OFFSET :626`) |
| UI (histórico, módulo eliminado 2026-10-06) | Vault breadcrumb top bar (B-A11/F17.1) | solo-lectura, cap 64 chars, colapso medio `Vault / … / cola`, None = no dibuja nada, hover ruta plena, cero I/O | ui/vault.rs:750 (`TOP_BAR_BREADCRUMB_MAX_CHARS`), `:757` (label pura), `:778` (draw) + wiring `app/src/ui.rs:893` |
| Study (histórico, módulo eliminado 2026-10-06) | FSRS-lite core (F16.1) | R(t,S)=(1+19/81·t/S)^-0.5, `INIT_STABILITY 1.0` día, `INIT_DIFFICULTY 5.0` (1..=10), `MIN/MAX_INTERVAL 1/365` días, `due_day` saturante i32, `review` 1..=4 | study/src/fsrs.rs:23-41 (consts), `:68` (init), `:97` (retention), `:183` (due_days_in_month) + calendar `:355,:379` Hinnant sin chrono |
| Study (histórico, módulo eliminado 2026-10-06) | Calendario dot FSRS (F17.2 + B-A27) | dot ámbar 6px fila propia con hover solo si `fsrs_due_in_cell(in_month, day, set)`; `FSRS_DUE_AMBER` (250,204,21); deck real (`fsrs_deck` en `app.rs:1920`, helper puro `fsrs_mark_reviewed` `:65` cap 1024 `MAX_FSRS_DECK` `:56`, persiste `.grafito/fsrs_deck.json` `vault_jobs.rs:199,236`, botón Repaso 1-4 en el modal) | ui/study_views.rs:113 (ámbar), `:118` (predicado puro), `:1037` (grid), `:1204` (dot), `:79` (`MarkReviewed`), `:1360` (botones) + wiring `app/src/app.rs:5098-5140` |
| UI | SPLASH_LOGO_SIZE | 128 | tokens.rs:205 (pin `:805`, `:807` %4) |
| UI | ASSISTANT_PANEL width | 300..520 (default 400); bottom-sheet si viewport < 740 | assistant.rs:102-107 + `assistant_uses_bottom_sheet` :4923-4924 + draw :4854-4874 |
| UI | Tessellation egui (rayon) | 1-2 ms/frame, 10K verts (egui/rayon tessellation paralela) | Cargo.toml `egui = { features = ["rayon"] }` + app.rs:6 presupuestos |
| GPU | domain_coloring_compute | 250k cells/dispatch (500×500, MAX_CELLS 250k) | grafito-render/domain_coloring_compute.rs:13 + lib.rs |
| App | MAX_UNDO | 50 (VecDeque pop_front O(1)) | app.rs:33 + controllers.rs:19 |
| App | MAX_UNDO_BYTES | 50 MiB (VecDeque, pop_front O(1), Document::estimated_bytes) | app.rs:40 + controllers.rs:21 |
| App | undo_stack | VecDeque<Document> + VecDeque\<ChangeSet\> (pop_front O(1), fix Vec::remove(0) O(n)) | app.rs + controllers.rs |
| Core | ValidatedDocument | fail-closed wrapper try_new | validation.rs:40 |
| Assets | mora.png / mora.svg | <32 KiB PNG embebido via include_bytes! (fallback dibujado si falla) | assets/mora.png, assets/mora.svg, app.rs:4707 |

## 9. Verificacion CI (17 jobs) — .github/workflows/ci.yml

MSRV 1.92 (`rust-version.workspace = "1.92"`) verificada en matriz `toolchain: ['1.92', stable]` para `check`, `test`, `lint`; `cargo metadata --locked` con 1.92 valida lockfile completo; docs advierten 1.92 en `Cargo.toml`, `ci.yml`, `CONTRIBUTING.md`, `AGENTS.md`, `README*.md`, `packaging/README.md` ( packaging-fixtures.sh lo exige).

| # | Job | Comando / descripcion | Runner / notas |
|---|-----|------------------------|----------------|
| 1 | `check` | `cargo check --workspace --locked` + `cargo check -p grafito-app --target x86_64-pc-windows-gnu --all-features --locked` (solo 1.92) | matrix 1.92 + stable, apt cache libgmp/mpfr/mpc/dbus, mingw-w64 para 1.92 |
| 2 | `test` | `cargo test --workspace --all-targets --all-features --locked` | matrix 1.92 + stable |
| 3 | `gpu-compute` | `cargo test -p grafito-render --test gpu_compute --locked` **required** | `WGPU_BACKEND=vulkan`, `GRAFITO_REQUIRE_GPU_TESTS=1`, `mesa-vulkan-drivers` + `libvulkan1`, no longer `WGPU_BACKEND=gl headless` ni SKIP; falla si GPU no disponible |
| 4 | `examples` | `cargo check --workspace --examples --locked` | stable |
| 5 | `benches` | `cargo check --workspace --benches --locked` | stable (separado de examples desde 14-job split) |
| 6 | `docs` | `cargo doc --workspace --no-deps --locked` con `RUSTDOCFLAGS=-D warnings` | stable |
| 7 | `lint` | `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | matrix 1.92 + stable, components clippy |
| 8 | `release-build` | `cargo build --workspace --release --locked` | PR \|\| push main (`if: pull_request \|\| push refs/heads/main`), ubuntu-22.04 |
| 9 | `fmt` | `cargo fmt --all -- --check` | stable, rustfmt |
| 10 | `all-targets` | `cargo check --workspace --all-targets --all-features --locked` | stable |
| 11 | `cross-platform-smoke` | `cargo test -p grafito-app --test app_smoke --locked` | matrix os: ubuntu/windows/macos, fail-fast false |
| 12 | `supply-chain` | `cargo audit 0.22.2` + `cargo deny 0.20.2` + `verify_advisory_exceptions.py` | cache cargo-tools |
| 13 | `workflow-lint` | `actionlint 1.7.7` + `shellcheck` + `bash -n` + `bash packaging/tests/packaging-fixtures.sh` + Debian version mapping (`1.2.20~beta < 1.2.20`) | valida packaging fixtures como gate |
| 14 | `coverage` | `cargo llvm-cov --workspace --all-targets --all-features --locked` con gate `--fail-under-lines 71`, semanal/manual (`schedule` lunes 03:00 / `workflow_dispatch`, job en `ci.yml:390`, gate `if:` en `:395`), SIN fallback `cargo test` (F5: duplicaba el job `test`; GPU required con `GRAFITO_REQUIRE_GPU_TESTS=1`, ya no SKIP) | stable + llvm-tools-preview, artefacto lcov 14 días |
| 15 | `bench-regression` | `cargo bench --workspace --benches --locked` (criterion, baseline `main` INFORMATIVO sin gate >10% hasta baseline estable; F5: benches SSE-truncado (`assistant/benches/sse_truncation.rs`) + frames nativos integral/morph (`anim/benches/native_frames.rs`) + `evaluate_cached_hot`/`miss_w200` (`geometry/benches/expr_bench.rs:30,37`) existen, corren con harness `--test` fail-closed y publican su base). **Solo `schedule` + `workflow_dispatch` (job en `ci.yml:461`, gate `if:` en `:476`): NO es gate de PR; sin comparador `critcmp` por diseño** (flaky entre runners; ver Notas) | stable, artefacto target/criterion 7 días |
| 16 | `mutation` | `cargo mutants 24.11.1 --workspace --timeout 60 --in-place` (semanal a proposito: 60min + muta el arbol, no apto como gate de PR). **Solo `schedule` lunes 03:00 + `workflow_dispatch` (job en `ci.yml:518`, gate `if:` en `:525`): NO es gate de PR** | sólo `schedule` semanal / `workflow_dispatch`, 60 min |
| 17 | `package-debian` | `desktop-file-validate` + `packaging/build-deb.sh` + `dpkg-deb --info/--ctrl-tarfile` ownership `root:root`, permisos, `lintian --fail-on error`, `dpkg --install` + `/usr/bin/grafito --help` + purge | ubuntu-22.04, Needs `dpkg-dev lintian desktop-file-utils` |

Notas:
- Ola 4 honesto: `coverage`, `bench-regression` y `mutation` son `schedule` /
  `workflow_dispatch`, NUNCA gates de PR (verificado `ci.yml:395,476,525`).
  `bench-regression` no tiene comparador `critcmp`: comparar >10% de verdad
  exigiría baseline estable + descarga del artefacto de `main` y es flaky entre
  runners; hasta tenerlo el job es informativo (compila, corre harness `--test`,
  imprime números base, sube artefacto) sin gate.
- `gpu-compute` ahora **requerido** con `WGPU_BACKEND=vulkan` (antes `gl` headless SKIP); `GRAFITO_REQUIRE_GPU_TESTS=1` hace fail-closed si el adapter no esta disponible.
- Packaging fixtures (`packaging/tests/packaging-fixtures.sh`) es gate en `workflow-lint`: verifica iconos `16..512` + scalable `hicolor/scalable/apps/grafito.svg`, `grafito-icon.svg`, abort si falta asset, y `desktop Icon=grafito`, mas plugins `usr/share/grafito/plugins` (`j-space`), `postrm` parse, MSRV 1.92 docs, MSVC static CRT, e icon asset existencia; `assets/mora.png/.svg` existen y se embeben via `include_bytes!` (verificado en `app.rs:4870` test `<32 KiB`).
- Baseline 2026-08-20: 7/8 PASS (gpu_compute SKIP headless, release-build SKIP 45m). Desde 14-job split: gpu-compute ya no SKIP, package-debian y workflow-lint son blocking. F14 2026-09-13: `ci.yml` real 637 líneas (medido 2026-10-05 con `wc -l`; era 608 en 2026-09-13, +29 por comentarios F5/budget-gpu), gate coverage 71% (no 75), `release-build` corre también en PR, `mutation` limitado por `cargo-mutants.toml` a 5 crates (`core/geometry/command/render/app`).
- Perf Ola 5/6 (2026-09-17): mediciones, decisiones y negativos en `docs/profiling.md` (walk vs flat, mimalloc **revertido**, `desired_maximum_frame_latency: 1`, GPU async, RAM); PGO reproducible con `scripts/pgo.sh` (`--interactive`, `--install`, fail-closed si no hay `llvm-profdata`); spike egui 0.32 diferido con conteo real de rupturas en `docs/adr-0004-egui-032-spike.md`; `bench-regression` compila además `expr_bench`/`ode_bench`/`cas_matrices_bench`/`native_rgba` (sigue informativo, sin `critcmp`).

## 10. Snapshot histórico v1.2.35-beta — Paridad parcial GeoGebra: subset verificado, resto stub honesto (2026-08-26; la versión vigente es 1.3.0)

**Pedagogía multi-nivel (primaria→ingeniería)**
- `grafito-pedagogy::Curriculum` 68 LOs (histórico de esta snapshot: 45 = AM1 8, AM2 7, Álgebra 7, Prob 6, Secundaria 12, Primaria 5; desde entonces sumaron AM2 9 + Física I/II 6+6, Discreta 6 y 3 avanzados — ver `curriculum.rs`)
- `UdlProfile {Concrete,Graphic,Symbolic,Formal}` + `level_value()` (Primary 2, Secondary 8, AM1 12...)
- `SocraticFsm` Review→HeuristicQ→AwaitStudent→Rectify→Summarize (Telling<5%), `ScaffoldEngine` ya usa `history`
- `ExerciseGenerator::generate_with_seed` wyhash paramétrico (a,b,c) + `ValidatorKind` + `FeedbackEngine` 10 misconceptions (Sign/Distributive/ChainRule/Fraction/Domain/Notation/Exponent/Algebra/Concept/None)

**Perfil adaptativo**
- `BKT` bayesiano + `Scheduler` Leitner `next_interval=86400*2^(level-1)*(2-mastery)` → `BranchState {next_review_epoch, box_level, bkt_p_known}` + `branches_due()` + `recommend_next_with_scheduler()`
- `WorkingMemory` sesión (steps_tried, misconception_counts) sincronizado `assistant.rs:520` con `StudentProfile`

**Asistente OpenCode Go socrático**
- `PedagogyDispatcher` 6 tools puras (`scaffold`, `generate_exercise`, `assess_answer`, `get_curriculum`, `suggest_next`, `generate_animation`) + 3 base = 9 `all_safe_tool_schemas()` OpenAI-compat (`agent.rs` 680L, 14 tests)
- `TeachingTopic` 14 variantes, `teaching_ui::whiteboard_elements_for_hint` 14 mappings (fracción, vector, matriz, prob, serie, trig, cónica...), `anim_native` templates pedagógicos, `AssistantExerciseCard` inline `grafito-exercise`

**UI Scandinavian sin laberinto**
- `toolbar.rs` `PRIMARY 5` `SECONDARY 8` `UNIVERSITY 18` (F3a suma Transform; `ALL_GROUPS` 15 diverge — ver §8) + `toolbar_groups_for_level_value(u32)` + `udl.rs` helper sin depender de pedagogy; `filter_groups_by_level`
- `AssistantPanelState` `max_composer 88..260` quiet, tokens 64%/44%/10%/5% documentados, `whiteboard:WhiteboardDoc` persistente en `Document` (serde, cota 500 elementos), `spreadsheet` `=A1+B1` stripping, `AppConfig::onboarding_completed`

**Gates:** `cargo fmt 0` `clippy -D warnings 0` `test --workspace ~1500 verdes`

## 11. Riesgos residuales

- BTreeMap determinismo total (cerebro-audit 2026-09-10): `objects`, `next_label_number`, `spreadsheet_coordinate_points`, `variables` (`VarMap`), `variable_meta` (`VarMetaMap`), `live_sequences`, `variables_assumptions` + params de constraints (`Constraint.params`, `Document` params fns) + `Exercise.params` son BTreeMap; ~210 firmas `&HashMap<String,f64>` migradas en core/geometry/command/render/complex/pedagogy/app (+ tests/benches/examples). Quedan `HashMap` solo fuera del dominio variables: índice solver `var_index`, mapas tipados `DD`/`Complex64`, intervalos simbólicos, escalares de planilla, cachés UI.
- `Transformed` Jacobian det pendiente
- `fill_compute`: pipeline real en `grafito-render/src/fill_compute.rs` (`Renderer::ensure_fill_compute`), sin caller de app todavía (campo `None`: ahorra ~128 MiB; habilitar lazy si `ImplicitCurve != Eq`)
- App God Object `app.rs 10615L` + `assistant.rs 15299L` (medido 2026-10-05): `controllers.rs` reales con tests y `GrafitoApp` delega por puente `from_parts`/`into_parts` (wiring fino P2); `AssistantTurnState` vive en `assistant_jobs.rs` y deriva del runtime, con consumidor productivo `assistant_is_busy()` en `attach_assistant_image` (F14.4.2; wiring fino P2 queda)

## 12. Próximos pasos

1. `ValidatedMatrix` (la migración `Document {variables, variable_meta, live_sequences, variables_assumptions}` a `BTreeMap` ya se completó 2026-09-10)
2. `WhiteboardDoc` export SVG + `GeoObject::Whiteboard` independiente
3. `fill_compute` lazy con caller de app + `DomainColoring` en export
4. Classroom P2P opt-in (feature flag)

## 13. Tabla doc→código (BUILD 2026-09-04 — todo número con origen verificado)

| Afirmación en docs | Origen verificado en código |
|---|---|
| RequestBudget 8192 / 2048 / 8 / 60s | `crates/grafito-assistant-types/src/lib.rs:198-209` |
| AttachmentLimits 512 KiB / 1 MiB / 1-2 MiP / 2 adjuntos | `crates/grafito-assistant-types/src/lib.rs:245-255` |
| 764 comandos (`command!(`), 727 visibles + 15 UI = 742 en paleta | `crates/grafito-command/src/command_registry.rs` (R3.1: Rename visible; 3D-A2: +Vista3D; P0.2: +41 álgebra; P0.3: +7 complejo; P0.4: +3 demostración; P0.5: +8 optimización+gráficos; P1: +82 listas/estadística/probabilidad; P1b: +1 List persistible; P2: +25 medidas/geometría; P3b: +11 scripting + Execute real; P4: +105 CAS/listas/geometría/stats/display; P5: +18 cierre nominal (Slope/SetValue/GroebnerLexDeg/SD/SampleSD/SampleVariance/SetSeed/Seed/Turtle*); Harness Fase A: +7 Discreta; Fantasma+nuevos 2026-09-23: +12 (7 con handler sin spec + 5 nuevos PolyGCD/Resultant/Residue/PrincipalPart/StepByStep); Fantasma-2 2026-09-23: +71 — `Image` NO se registra (stub sin modelo persistente, blindado `:11404`); deltas por frente `:11832-11946`, blindaje `registry_counts_match_documented_architecture` `:11777` 764/727/25 en verde 2026-10-04; Olas G1/O1/O3/O5/O6: 747→764, paleta 710→727; EDO-con-pasos 2026-10-15: +2 CAS con spec (SolveODE1/SolveODEHomog)) |
| 15 acciones UI + fuzzy + footer es | `crates/grafito-ui/src/command_palette.rs` |
| 39 tools assistant (3 base + 8 pedag + 24 math + 2 harness1 + 2 harness2) / 9 en agent hoja | `crates/grafito-assistant/src/agent.rs:3549` (`math_tool_schemas` 24), `:4445` (`all_safe_tool_schemas` 39), asserts `:8023,:8026`; `crates/grafito-agent/src/tools.rs:3300` (3+6=9, assert `:3844`) |
| MCP `tools/list` 71 (9 lab incl. execute + 37 proxedas + 2 Lean + 2 policy + 1 GPU + 2 Colab + 5 estudio + 2 CAD + 11 vault); proxied 37 (= all_safe 39 − harness-2 viejo) | `crates/grafito-mcp/src/protocol.rs:144` (`all_tool_defs`, total dinámico pineado `:505-513`), `tests/stdio_contract.rs` (contrato punta a punta), `bridge.rs:281-299` (proxy = all_safe − excluidos) |
| Taylor con UNA implementación (dedup 2026-09-23): wrapper fino `Option<String>` sobre el `Result` canónico | `crates/grafito-geometry/src/matrices.rs:877-885` → `symbolic.rs:5537` |
| Financieras dedupe 2026-09-23: núcleo canónico `finance_*` (`pub(crate)`) + fachadas de aula que delegan | `crates/grafito-geometry/src/cas_extra.rs:767-853`; `stats_extra.rs:326,344,364` |
| `Interval::new(prec, lo, hi)` con redondeo outward real (antes `prec` se ignoraba: la API mentía); `prec = 0` preserva el comportamiento previo | `crates/grafito-geometry/src/interval.rs:9,35-55` |
| Contratos públicos multipolinomio (2026-09-23): `poly_gcd_subresultant`, `BiPoly`, `sylvester_resultant` | `crates/grafito-geometry/src/symbolic.rs:2576`, `solve.rs:779`, `solve.rs:1026` |
| **Cambio de semántica** `sylvester_resultant` a convención de signo estándar/textbook (2026-09-23): `Resultant[x²+1, x-1, x]` ahora da `2` (antes `-2`); `Res_y(y²+1, y−1) = 2` | `crates/grafito-geometry/src/solve.rs:1005-1024` (doc + NOTA DE SEMÁNTICA), corrección de signo `:1050-1054`; pin `crates/grafito-geometry/tests/api_contract.rs:29-54` (el solver interno `solve_system_2x2` solo usa raíces: no le afecta) |
| Polygon: degeneración por área ABSOLUTA de regiones (fix 2026-09-24): antes el shoelace CON SIGNO daba área neta 0 en toda curva auto-intersecada y la rechazaba ("degenerate or colinear" en cualquier parametrización: Lissajous, lemniscata, lazos) | `crates/grafito-core/src/validation.rs:971-1001` (fan de triángulos abanicados desde el vértice 0, invariante ante traslación); tests `:2538-2625` (Lissajous(3,2) + 3 lazos pasan; colineales siguen rechazándose) |
| `captions`: cota de salida 256 KiB PREVENTIVA (fix 2026-09-24): antes se construía TODO y se medía después (peor caso de wire ≈ 200 MB transitorios); ahora aborta al cruzar el tope | `crates/grafito-anim/src/captions.rs:350-377` (`empuja_acotado`/`empuja_karaoke`), `to_srt`/`to_ass` `:281-348`; tests `:652-733` |
| `player`: presupuesto propio del set jugado 64 MiB estimados (fix 2026-09-24); fondo compartido QUEDA como propuesta (un `Arc` plano por frame no ahorra las copias: haría falta representación de dos partes = contrato Piel) | `crates/grafito-anim/src/player.rs:307-372` (`PLAYER_MAX_TOTAL_BYTES`, `estimate_placed_bytes`, `empuja_frame`), análisis en el doc del módulo `:35-53`; test peor caso `:1977` |
| `short_script`/`long_script` pasan las 69 canónicas literales (13 con copy propio + resto universal) + copy por familia (6) + golden hash FNV-1a del `GuionTexto` serializado (fix 2026-09-24; antes 8/13 plantillas caían a `universal` y los 6 beats eran idénticos para todo concepto; catálogo ampliado a 69 el 2026-09-28: 13 canónicas + 52 dominio + 4 extra) | `crates/grafito-anim/src/guion.rs:931-1157` (`BeatsCorto`, `BEATS_*`, `familia_corto`, `short_script`), tests `:1737` (13 canónicas), `:1770` (beats por familia), `:1808` (golden) |
| Protocolo anim v1 con rango real de negociación 1..=2 + trazas de handshake (fix 2026-09-24; antes `== 1` duro y `Ok(_) => {}` descartaba sin traza) | `crates/grafito-anim/src/protocol.rs:35-42,1213-1222`; `crates/grafito-anim/src/engine.rs:451-458` (`anota_diagnostico`), `:520-528,:572-580` (diagnósticos), `:1067-1095` (versión fuera de rango → `Error` tipado); tests `engine.rs:1752,1790` |
| `math_expr` medido en BYTES, unidad unificada con el gate de pedagogía (fix 2026-09-24; el contador de chars permitía pasar el pre-gate y caer en `verify_math_expr`) | `crates/grafito-anim/src/guion.rs:61,224` (`MATH_EXPR_MAX_BYTES`) vs `crates/grafito-pedagogy/src/teaching.rs:284-294`; test `guion.rs:1577` |
| Regla de olas: UNA representación de expresión — `evaluate_cached` → `compile_flat_ops` → `run_opcodes_flat` (nunca portar `Expression` hash-consed ni `Expr` boxeado de otras fuentes) | `crates/grafito-geometry/src/expr.rs:1019,2256,1976`; números en `docs/profiling.md` §2.1 |
| `MAX_COMPILED_EXPR_CACHE` 128 medido (512/1024: Δ<3% con W=64 realista; 50×+ solo en working sets sintéticos) | `crates/grafito-geometry/src/expr.rs:978-996` + `docs/profiling.md` §2.11 |
| 18 grupos toolbar (PRIMARY 5, SECONDARY 8, UNIVERSITY 18; ALL_GROUPS 15 diverge — ver §8) | `crates/grafito-ui/src/toolbar.rs:192` (enum) + `:355/:364/:376` (PRIMARY/SECONDARY/UNIVERSITY) + `:325` (ALL_GROUPS), pins `:2169,:2256` |
| 89 herramientas (`Tool`) | `crates/grafito-ui/src/lib.rs` `pub enum Tool` (Ola 1.6 suma `Text`; contorno complejo suma `ComplexContour`) |
| 10 perspectivas (Ctrl+Shift+1..9,0) | `crates/grafito-app/src/lib.rs:90-111`, `shortcuts.rs:163-184` |
| 22 crates workspace | `crates/` (22 members + root `grafito-release-tests`; +lab R reclamos) |
| 17 jobs CI | `.github/workflows/ci.yml` (637 líneas, medido 2026-10-05; gate coverage 71%) |
| Spark vía Responses API (`POST {base}/responses`) | `crates/grafito-assistant/src/lib.rs:67-70` (comentario 500), `:1497` (`responses_endpoint`), `:76-78` (router), `:1560` (`remote_protocol`) |
| Ruteo Go por tabla `go_model_protocol` (docs Go 2026-09-13): spark/gpt/grok→Responses, minimax/qwen3.6-3.8→Messages, resto→Chat | `crates/grafito-assistant/src/lib.rs` (tabla + test `go_model_routing_table_matches_go_docs_endpoints`); agente usa la misma tabla |
| Catálogo Go vigente incl. `deepseek-v4.1-flash` + botón Actualizar + texto libre | `crates/grafito-ui/src/assistant.rs` (`OPENCODE_MODELS`, `RefreshModels`, `model_draft`, `sanitize_custom_model_id`); auto-refresh al cambiar proveedor |
| `max_tokens` Chat con headroom ×2 para razonamiento (piso 1024) | `crates/grafito-assistant/src/lib.rs:completion_token_limit_for_chars` + `loop_engine.rs:completion_token_budget`; test `chat_token_limit_doubles_chars_with_reasoning_headroom` |
| Fallback agente con pregunta guardada + toast de truncado + body de sesión | `assistant_jobs.rs` (`AssistantAgentJob.question`, `outcome.truncated`); `agent.rs:agent_http_status_error` (cuerpo cap 200) |
| F20: `reasoning.summary=auto` siempre en Responses + budget ×4 (verificado contra el endpoint real: sin summary el item llega con `summary:[]` y un "2+2" consume ~2045 reasoning tokens → `incomplete`) | `crates/grafito-assistant/src/lib.rs` (`build_responses_payload`, `responses_token_limit_for_chars`, test `responses_reasoning_summary_always_requested_with_quadrupled_budget`) |
| F20: etapas sin jerga (`Pensando…`, sin KiB) + tarjeta de espera oculta con contenido en vivo | `crates/grafito-ui/src/assistant.rs` (`RemoteStage::label`, `pending_card_visible`, `provisional_turn_has_visible_content`) |
| F20: composer sólo-iconos + teclado con foco y toggle persistente + composer sin recorte | `crates/grafito-ui/src/icons.rs` (`Send/Brain/Copy/Paperclip`), `assistant.rs` (`composer_icon_toggle`, BASE 100, sheet con piso usable), `app/src/keyboard.rs` (inserción según `composer_focused`, `keyboard_visible_explicit`) |
| Modelo default `deepseek-v4-flash` | `crates/grafito-app/src/utils.rs:126` (`default_assistant_model`) |
| Fallback sesión spark→deepseek | `crates/grafito-app/src/assistant.rs:9488-9545` (`should_fallback_agent_spark_to_deepseek :9500`, `should_fallback_remote_spark_to_deepseek :9538`), test `:10739` |
| Ctrl+T tema | `crates/grafito-app/src/shortcuts.rs:203` + menú `draw_view_menu ui.rs:397-422` (`shortcut_text` Ctrl+T `:418`) |
| Ctrl+P/E lápiz/borrador, F8/F9 esfera/cubo | `crates/grafito-app/src/shortcuts.rs:69-78,213,218` |
| Onboarding first-run fullscreen (`Area` `Foreground`, 1 pantalla `WELCOME_PAGES=1`) | `crates/grafito-app/src/app.rs:1960-1961` (campo `show_onboarding`), gating `:2692`, `OnboardingChoice :9263` + `draw_onboarding_window :9304` + `apply :9519-9532`, `utils.rs:57-59` (`onboarding_completed`), `WELCOME_PAGES :2168-2169` |
| Rail 68px, drawer 292..440, panel izq 240+45% | `crates/grafito-ui/src/tokens.rs:159,165-171,173-178,183,525-528`; `app/src/ui.rs:32,1032,1059,1383-1388`; `app/src/lib.rs:427,430,442-447`; `app/src/panels.rs:675,1131,1602,2418` |
| AnimDuration 0.1..=60s, Resolution 64..=4096 | `crates/grafito-anim/src/protocol.rs:263`, tests `:3321-3371` |
| Long-form 1500 frames (GIF 64), chunks 64 MiB streaming, timeline 60 s | `crates/grafito-anim/src/protocol.rs:341,345,349,364-389,438,931` |
| AudioTrack offset 0..=60000, gain 0.0..=2.0 | `crates/grafito-anim/src/protocol.rs:468,470,503-543` |
| Captions SRT/ASS ≤256 KiB; voiceover ≤40 palabras/paso; short 110-130 | `crates/grafito-anim/src/captions.rs:33`; `guion.rs:64,66,68` |
| run_command ≤2000 chars + allowlist/undo (solo no destructivos) | `crates/grafito-assistant-types/src/lib.rs:17,988-997`; `assistant/src/agent.rs:2381-2432` |
| solid_measure_3d + generate_short_script (4 beats) | `crates/grafito-assistant/src/agent.rs:1828-1907,2234-2237,2705-2720` |
| Voiceover Piper sidecar honesto + captions sidecar/quemadas | `crates/grafito-app/src/voice.rs:39-56,102-134`; `grafito-ui/src/assistant.rs:997-998,1016-1017` |
| Write por trazo + Rectangle/Ellipse/Arc | `crates/grafito-anim/src/player.rs:23,469,482`; `scene.rs:533-542` |
| LaTeX offline SVG/raster (tex, STIX) + draw_math | `crates/grafito-tex/src/lib.rs:1,157,247,352`; `grafito-ui/src/assistant.rs:10649-10820` |
| 69 plantillas nativas (pineado en `NATIVE_TEMPLATES == CANONICAL_TEMPLATES == 69`; 13 históricas + 52 dominio + 4 extra = 69) + `media.title` 14 ES/EN/PT/IT/FR/DE | `crates/grafito-anim/src/protocol.rs` (`CANONICAL_TEMPLATES` 69) + `crates/grafito-app/src/anim_native.rs` (`NATIVE_TEMPLATES` 69) + `crates/grafito-ui/src/i18n.rs` (`media.title.*` 14: +`subspace`/`fractal`); el corto/largo `short_script`/`long_script` pasan las 69 literales (13 con copy propio + resto universal; antes 5 + fallback) con copy por familia |
| 22 members + root; 23 dirs en crates/ | `Cargo.toml` members + `ls crates/` |
| F18: `StreamDelta` (Reasoning/Text/Status) + `usage` real por proveedor + reasoning plegable por turno | `crates/grafito-assistant/src/lib.rs` (`StreamDelta`, `stream_progress_sender`, `parse_token_usage`, `post_json_with_reasoning_fallback`, `request_anthropic_completion_streaming`); `crates/grafito-app/src/assistant.rs` (`drain_remote_stream_preview`); `crates/grafito-ui/src/assistant.rs` (`draw_reasoning_disclosure`, `draw_turn_metrics`, `composer_mode_chip`) |
| F18: búsqueda web sin claves (DDG HTML + Instant Answer) | `crates/grafito-assistant/src/web.rs` (23 tests) + tool opt-in `web_search` (`agent.rs`) |
| F24: texto remoto nunca se descarta por contexto rancio; propuestas re-verificadas contra el doc actual; aviso Info | `crates/grafito-app/src/assistant_jobs.rs` (`poll_assistant_jobs`: `accepts_remote_context` → aviso; `proposals_stale` → descarta propuestas sin perder texto), tests `texto_remoto_se_publica_aunque_cambie_el_documento`, `propuestas_rancias_se_descartan_sin_perder_el_texto` |
| F24: clarificación `ask_user` con `call_id` real (sin heurística de longitud) | `crates/grafito-agent/src/loop_engine.rs` (`AgentEvent::Clarification`), `crates/grafito-assistant/src/agent.rs` (emisión), `crates/grafito-app/src/assistant_jobs.rs` (forwarder al canal lateral) |
| F24: contexto del prompt truncado por presupuesto con nota honesta + delimitadores anti-inyección | `crates/grafito-assistant/src/lib.rs` (`bounded_context_prompt`, `UNTRUSTED_DATA_*`, tests `documento_denso_se_recorta_por_presupuesto_sin_fallar`, `contexto_y_web_van_delimitados_como_datos_no_confiables`) |
| F24: telemetría local opt-in de tokens (JSONL, sin PII) | `crates/grafito-app/src/usage_log.rs` + llamadas en `assistant_jobs.rs` (chat/proposal) |
| F24: determinismo `ConstraintGraph` (BTreeMap/BTreeSet) + memos frescos por clon staged/restore | `crates/grafito-core/src/constraints.rs`, `document.rs` (`detached_clone_for_staging`, `ChangeSet::restore`), test `constraints_serializan_en_orden_determinista` |
| F26: composer 32px (`ASSISTANT_COMPOSER_ACTION_SIZE`) + `fit_icon_square` anti-estirado | `crates/grafito-ui/src/icons.rs` (`draw_icon`, `fit_icon_square`, test), `crates/grafito-ui/src/assistant.rs` (`composer_icon_toggle`, BASE 160/colapso 138) |
| F26: `is_explicit_demo_request` (pedido de ejemplo nunca exige repair) + setting `assistant_socratic_enabled` con checkbox | `crates/grafito-pedagogy/src/scaffold.rs`, `socratic.rs` (`requires_repair_despite_exploratory`), `crates/grafito-app/src/utils.rs` (config), `assistant_jobs.rs` (`session_guard` + repairs), `crates/grafito-ui/src/assistant.rs` (checkbox + `SocraticModeChanged`) |
| Harness opencode: instructions 20k→3.5k tokens/sesión; `filesystem` MCP off; `small_model` deepseek-v4.1-flash; agentes baratos en routing; watcher ignora `.opencode/**` | `opencode.json` + `.opencode/agents/*.md` + `.opencode/skills/grafito-architecture/SKILL.md` |
| HISTÓRICO (módulos eliminados 2026-10-06) Estudio FSRS-lite + calendario ámbar + breadcrumb vault (F16.1/F17.1/F17.2/B-A11): `fsrs_init`/`due_days_in_month` puros, dot 6px solo con vencimientos, breadcrumb 64 cap solo-lectura | `crates/grafito-study/src/fsrs.rs:68,183` + `calendar.rs:355,379`; `crates/grafito-ui/src/study_views.rs:113,118,853,1037,1204` + `vault.rs:750,757,778`; wiring `crates/grafito-app/src/app.rs:5040-5052` + `ui.rs:893` |

## 14. Paridad GeoGebra 2026 — frente F10-C (BUILD 2026-09-05, rama f10-plan-total)

> Plan de cierre ejecutado 2026-09-16 (P0–P4, ver ADR-0003) y auditoría extrema
> 2026-09-18 (Olas 0–2): **643 comandos** (606 visibles + 15 UI = 621 en paleta).
> Harness de problemas abiertos Fase A (2026-09-20): **650 comandos** (613 visibles
> + 15 UI = 628 en paleta), 23 tools (harness2: search_topp39 + export_dimacs),
> motor `geometry::search` + loop TOPP 39 con hash re-verificado (protocolo en
> `docs/OPEN_PROBLEMS_LAB.md`);
> i18n ES/EN/PT/IT/FR/DE (1463 claves, purgas 2026-10-12 −176 + 2026-10-13 −8 onboarding/center + 2026-10-14 +2 racha/veredicto + 2026-10-14 +3 empty cálido/GIF rápido: `MSG_COUNT` en `crates/grafito-ui/src/i18n.rs:102`; menús y panel del asistente incluidos, +5 `profile.*` B-A41 +7 `profile.*` B-A42 +12 `profile.*` B-A43 formas Silueta +21 `profile.*` B-A44 Rostro +17 `profile.*` B-A45 Acento display y Fondo +12 `profile.*` B-A46 Rostro headers + sliders +15 `profile.*` B-A47 preview + Animación +8 `profile.*` B-A48 Personalidad headers + Afinar tono +16 `profile.*` B-A49 presets Personalidad +14 `profile.*` B-A50 Idioma + Instrucciones + Objetivo +15 `profile.*` B-A51 bloque Memoria +10 `settings.*` B-A52 tab Ajustes Proveedor/Modelo/discovery +23 `settings.*` B-A53 tab Ajustes checkboxes + Clave + plugins +3 `settings.*` B-A54 tab Ajustes buscador + cerrar +18 `settings.*` B-A55 tab Ajustes nav secciones + entradas + hints +10 `settings.*` B-A56 tab Ajustes Apariencia + grupos Atajos +16 `settings.*` B-A57 tab Ajustes descripciones file+edit+view + vacío +21 `settings.*` B-A58 tab Ajustes descripciones tools +9 `settings.*` B-A60 tab Ajustes General +10 `anim.export.*` B-A61 diálogo export animación +6 `anim.export.*` B-A62 diálogo export resto + 22 `anim.export.*` B-A63 hints display + más-acciones + labels de enums + 4 `anim.export.*` B-A64 validación export + 6 `anim.job_*` B-A66 progreso del job + 15 `anim.media_*` B-A67 player/header/export + 1 `anim.media_*` B-A68 preparando + 2 `assistant.*` B-A68 saludo/mora + 4 `assistant.math.*` B-A69 math-tex + 5 `teach.*` B-A71 marcos + pizarra + 7 `teach.*` B-A73 ejercicio + overlay nav + 14 `teach.*` B-A74 feedback + ejercicio resto + overlay nav/hovers + 4 `teach.*` B-A75 progreso + distractores + 5 `teach.*` B-A76 errores del job + 2 `teach.*` B-A78 toast + hint morph + 1 `teach.*` B-A79 overlay reserva + 2 `trace.*` B-A94 hover + vacío + 2 `assistant.error.*` B-A95 reintento + reinicio + 7 `trig.*` B-A96 chrome + 6 `trig.*` B-A97 modos + play/pausa + 14 `trig.*` B-A98 descripciones + prompt + lecturas + identidades + 6 `trig.prompt_err_*` B-A161 errores prompt + 22 `assistant.*` B-A101 turno/propuesta/auth/clarify/plan + 1 `vault.*` (título backlinks) + 2 `center.*` (whiteboard + reopen) + 7 `stats.*` B-A105 panel estadística + 13 `motion.*` B-A106 tarjeta movimiento 4D + 13 `wc.*` B-A107 tarjeta W-C + 6 `wc.section_*` B-A132 headers W-C + 15 `inspector.*` B-A108 panel Inspector base + 20 `inspector.poly*` B-A109 sección 4D/N-D + 13 `inspector.*` B-A110 visibilidad/kind/familia/ejemplo/medida + 8 `inspector.*` B-A111 prefijo Etiqueta + estilos + 14 `inspector.*` B-A112 primitivas 3D + sliders + 10 `inspector.*` B-A113 vista previa implícita + 8 `picker.*` B-A117 color picker + 7 `stats.*` B-A119 empty + puente caso/cuantil + 13 `stats.*` B-A120 caso + tiles + momentos + 8 `stats.*` B-A121 cuantil chrome + 2 `stats.*` B-A122 cuantil resultado + 10 `stats.*` B-A123 ordenar+graficar + 4 `stats.*` B-A124 resumen+asistente + 9 `view.*` B-A128 Vista General+Ejes + 14 `view.*` B-A129 Vista 3D+Capas+AltaPrecisión + 10 `view.*` B-A130 Capas dinámicas + hints + indicate + 8 `view.*` B-A131 parámetro vivo + 2 `stats.*` B-A133 headers prob + 1 `stats.*` B-A134 header Inferencia + 12 `colab.*` B-A136 panel Colab + 7 `colab.status_*` + 5 `colab.import_*` B-A137 status + notas + 10 `algebra.*` B-A138 drawer + vacío + hovers + 8 `algebra.*` B-A139 entrada + teclado + anim + a11y + 8 `algebra.var_*` B-A141 sliders + rango + 2 `algebra.var_*` B-A142 error + readout + 10 `algebra.summary_*` B-A143 resúmenes con notación + 13 `algebra.poly_*/cone/cylinder_infinite` B-A144 politopos + cono/cilindro + 7 `ctx.*` B-A145 menú contextual + error + 2 `ctx.*` B-A174 menú 3D + 9 `algebra.fit_kind_*` B-A146 kinds de ajuste + 12 `stats.regression_*` B-A147 panel Regresión + 12 `stats.regression_*` B-A148 dinámicos Regresión + header import + 7 `stats.regression_model_*` B-A149 etiquetas modelos + 10 `stats.series_*` B-A150 fila serie + 8 `stats.sheet_*` B-A151 hoja chrome + 10 `stats.sheet_*` B-A152 hoja tablas + 7 `stats.quick_*` B-A153 toasts quick-result + 11 `stats.sheet_summary_*`/`sheet_linked_long_hint` B-A154 resumen card + 16 `stats.prob_param_*` B-A155 params prob + 22 `stats.prob_dist_*/kind_*/point_*` B-A156 selector dists + 8 `stats.prob_plot_*` B-A157 plot messages +1 `stats.prob_plot_bar` S2 + 8 `inspector.*` B-A159 apariencia 4D + cierres + 12 `stats.quick_chip_*/quick_tip_*` B-A163 chips inferencia + 2 `stats.sheet_goto_*` B-A164 goto hoja + 4 `stats.sheet_nav_*` B-A166 flechas hoja + 1 `stats.sheet_chip_hint` B-A167 chip hoja + 10 `export.*` B-A169 sección archivo + 12 `export.*` B-A170 resto + 3 `export.*` B-A171 toasts copiado + 2 `ctx.*` B-A175 canvas a11y + 16 `cas.*` B-A177 panel CAS + 9 `exam.*` B-A180 examen resto + 1 `exam.*` B-A181 vista bloqueada + 9 `recovery.*` B-A183 modales recovery + 11 `custom.dlg_*` B-A184 diálogo custom-tool + 13 `rail.*` B-A185 rail sidebar + 14 `shell.*` B-A186 shell chrome + 9 `save.*` B-A187 diálogo save + 57 `hint.*` B-A188 hints barra + 2 `shell.*` B-A188 ejemplo + 16 `dynamics.*` B-A189+B-A190 Atractores + Parámetros + 1 `rail.*` B-A197 tab Aula + 5 `classroom.*` B-A198 panel externo + 16 `classroom.*` B-A199 panel interno + 3 `classroom.outbox_*` B-A201 outbox + 3 `exam.*` B-A206 bloqueo/oculto + 3 `assistant.empty.suggest_*` caminos empty-state 2026-10-14 + 2 `toast.first_*` celebración contenida 2026-10-14 + 1 `assistant.header.streak` racha visible + 1 `exam.modal_verdict` veredicto cálido 2026-10-14 + 1 `algebra.empty_action` siembra ejemplo + 2 `anim.export_quick*` GIF rápido 2026-10-14 + 11 `classroom.*` roles Aula 2026-10-15 + 5 `assistant.steps.read/stop*` TTS Leer 2026-10-15 + 4 `assistant.composer.dictate*` mic STT 2026-10-15),
> export `.ggb` + PDF multipágina + P2P iroh tras flag. GeoGebra lista ~502:
> cobertura nominal ≈100% (502/502 nombres resuelven) con profundidad parcial
> declarada abajo como subset honesto (Groebner/GroebnerLex/DegRevLex ya usan el
> Buchberger real, cotas 12 polinomios/6 variables/384 S-polinomios).

Cerebro puro en `crates/grafito-core/src/symbolic/` (`csv.rs`, `solids.rs`,
`exchange.rs`, `mod.rs` con `groebner_gate`); piel fina en
`crates/grafito-app/src/render_3d.rs` (`OrthoProjection`,
`project_point_ortho`, `solid_measure_text`); helps honestos en
`crates/grafito-command/src/command_registry.rs` (Groebner×3 con Buchberger
real, Net L).
Sin tocar geometría exacta, A11Y ni perf; sin `unwrap` (gates §9).

| Categoría | Grafito hoy (archivo) | GeoGebra | Esfuerzo |
|---|---|---|---|
| Capas | `symbolic/exchange.rs` (`LayerTable` 0..=255 + visibilidad) | capas con orden | S cerrado (API; wiring panel P2) |
| Bar/Pie | `command/src/commands.rs` (`BarChart`/`PieChart` reales desde lista o `DataTable[.xs\|.ys]` vía `parse_chart_data_arg`; `exchange.rs` conserva `bar_chart_stub`/`pie_chart_stub` solo como validación sin documento) | BarChart/PieChart | S cerrado (R3.5: rango de planilla) |
| Tabla viva lectura | `symbolic/exchange.rs` (`datatable_rows`/`cell`/`to_csv` sobre `DataTableObj`) | spreadsheet viva | S cerrado (edición P2) |
| Volumen/área 3D | `symbolic/solids.rs` (esfera/cubo/cilindro/cono/toro/tetra/pirámide/prisma exactos; cuádrica → `None` + `solid_measure_status`) | Volume/Area 3D | S cerrado |
| Vistas ortográficas | `symbolic/solids.rs` (`OrthoView` alzado/planta/perfil) + `render_3d.rs` (`OrthoProjection`, píxeles egui) | vistas 3D | S cerrado (cableado cámara P2) |
| Groebner | Buchberger real cableado a `Groebner`/`GroebnerLex`/`GroebnerDegRevLex` (`cas_gate` → `cas_motor::cas_groebner_ordered`, órdenes lex/grlex/grevlex, cotas 12/6/384 en `geometry/cas.rs`); `symbolic/mod.rs::groebner_gate` queda como vía legacy 2×2 solo para tests | CAS Groebner | S cerrado |
| PDF | `app/src/export.rs` (`pdf_page_ops` vía `printpdf 0.12`: rectas/círculos/polígonos/texto Helvetica, 1 página por hoja con contenido, tope `MAX_PDF_PAGES` 64; `document_to_pdf` queda referencia histórica) | export PDF | S cerrado |
| CSV RFC 4180 | `symbolic/csv.rs` (`to_csv` CRLF + `parse_csv` con `""`, cotas 20k filas/10M) | import/export CSV | S cerrado (wiring UI P2) |
| Clipboard SVG/PNG | `app/src/export.rs` (SVG real punto/círculo/polígono/texto; PNG vía `clipboard_png_bytes` + `arboard` "Copiar PNG", headless honesto) | copiar SVG/PNG | S cerrado |
| Álgebra exacta (Ola 2.5) | `geometry/exact_linalg.rs` (`ExactMatrix` sobre `ExactRational`, det/inversa/RREF hasta 32×32; overflow honesto) cableada a `Determinant`/`Inverse`/`ReducedRowEchelonForm` con fallback numérico (`exact.rs::from_f64_decimal`) | CAS lineal | S cerrado |
| Análisis simbólico (Ola 2.1) | `geometry/analysis.rs`: derivadas exactas del CAS (1.ª/2.ª/3.ª) con fallback numérico; clasificación de extremos por segunda derivada | Análisis de curvas | S cerrado |
| Factor (Ola 2.4) | `symbolic.rs`: raíces racionales + Kronecker acotado (enteros, grado ≤ 6); irreducibles sobre Q se devuelven tal cual; fuera de cota sigue error honesto | Factor | S cerrado |
| Cuádricas (Ola 2.6) | `symbolic/solids.rs`: esfera/elipsoide informan volumen exacto + área numérica (Simpson 128×128) con estado honesto; hiperboloides siguen `None` | Volume/Area 3D | S cerrado |
| Scripting automático (Ola 2.7) | `OnLoad` corre al abrir (`replace_document`) y `OnUpdate` tras cada commit mutante (`app.rs::run_on_update_scripts`, tope 16, guard de reentrada); allowlist benigna pinneada por test | GUI scripting | S cerrado |
| Gruntz / Risch / marching-tetra / Net / iroh / CRDT | Reales y cableados: Risch-Norman (`geometry/integral.rs:640` `risch_norman_integrate`), Gruntz (`geometry/cas.rs:672` `gruntz_limit`; consumido por `Limit` desde F14), marching-tetra (`geometry/polytopes.rs:4796` `implicit_surface_mesh`), Net (spec `command_registry.rs:4168` + dispatch `commands.rs:6412` + `run_net_command :23140` vía `PolyhedronNet::unfold`), Buchberger (entradas públicas `cas.rs:2783`/`2796` `buchberger_basis`/`_ordered`), P2P real `IrohTransport` tras flag `aula-iroh` (`classroom/src/iroh_transport.rs`, ping/pong verificado); stub honesto solo cifrado-sesión/outbox-persistente (`classroom/stubs.rs`) | CAS y P2P | S cerrado (wiring); UI de sala P2P = P3c diferido |

### 14.1 Frente C3 — voz, guion corto, trazo, LaTeX, run_command (sync 2026-09-11)

Puro y acotado, sin tocar geometría exacta ni A11Y; sin `unwrap` (gates §9).

| Pieza | Grafito hoy (archivo) | Nota |
|---|---|---|
| Voiceover Piper | `app/src/voice.rs` (sidecar `piper` en disco; sin binario → `PiperMissing` honesto, jamás silencio; UI `VozMode::Piper` + hint `assistant.rs:997-998`) | MP4 vía ffmpeg-sidecar o mudo honesto |
| Captions | `anim/src/captions.rs` (SRT/ASS ≤256 KiB) + sidecar `.srt` / quemadas vía ffmpeg (`voice.rs:102-134`) | Sin ffmpeg queda el sidecar |
| Short script | `anim/src/guion.rs:933` (`short_script`, 4 beats, 110-130 palabras) + tool `generate_short_script` (`assistant/src/agent.rs:1828-1907`) | Determinista byte-idéntico |
| Write + figuras | `anim/src/player.rs` (`WriteAnim` por trazo) + `scene.rs:533-542` (`Rectangle`/`Ellipse`/`Arc`) | Fallback opacidad honesta sin trazo |
| LaTeX offline | `grafito-tex` (pulldown-latex→formulary→SVG/tiny-skia, STIX embebida, 8 KiB) + `draw_math` (`ui/src/assistant.rs:10649-10820`) | Subset + aviso si falta fuente |
| run_command | bridge puro ≤2000 chars (`assistant-types:988-997`, `agent.rs:2381-2399`); app valida allowlist y aplica con undo, solo no destructivos | Propuesta, nunca ejecuta solo |
| solid_measure_3d | `assistant/src/agent.rs:2705-2720` sobre `symbolic::solids` exactos | Error honesto si no computable |
| Long-form streaming | chunks 64 MiB (`protocol.rs:349,364-389`), 1500 frames, timeline 60 s | Sin `Vec` total en RAM |
