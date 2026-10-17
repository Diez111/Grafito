# Animación liviana — GIF nativo + código puro (no MP4 pesado)

> Verdad medida 2026-10-08: `assets/` 976K sin MP4/GIF/WebM/MOV. Los únicos 2 MP4
> del repo están en `.agents/skills/` (demos de agentes, 18M) y NO son de Grafito.
> Export de animación usa crate `gif` 0.13 (`encode_frames_to_gif_bytes`,
> `export_frames_to_gif_file`, `spawn_gif_export`). MP4 solo vía ffmpeg externo
> honesto (`anim_native.rs:708-830`), jamás embebido.

## Decisión

1. **GIF nativo** para compartir (12fps/64 frames/5MiB por defecto, configurable).
2. **Código puro / animación nativa liviana** para entender: cada animación se
   guarda como receta (templates canónicos 69 en `protocol.rs:1759`) re-ejecutable,
   no como video. Pesa KB, es editable y pedagógica.
3. **MP4/WebM solo bajo demanda** vía ffmpeg del sistema, nunca en `assets/`.
   Cuando haya encoder propio 60fps/4K (Ola F), seguirá siendo export, no asset.

## Por qué supera a MP4

- Peso: receta nativa KB vs MP4 MB; GIF 5MiB tope vs demo 18M.
- Pedagógico: receta = pasos + parámetros (pausa, cambia, re-ejecuta). Video = caja negra.
- 100% Rust: `gif` crate puro, sin binarios, sin Python, reproducible.
- Recursos: render por frames bajo demanda, sin decodificar video en memoria.

## Reglas

- Ningún MP4/MOV/WebM en `assets/`, `docs/`, `crates/`. Solo GIF ≤5MiB o receta `.json`.
- Cada ejemplo animado: receta nativa + GIF opcional, jamás solo video.
- Export default: GIF. MP4 solo si el usuario lo pide explícito y hay ffmpeg.
- Blindaje local: `bash scripts/check-assets.sh` (falla si hay `*.mp4`/`*.mov`/`*.webm` >5MiB en `assets/`/`docs/`/`crates/`). Correrlo antes de pushear; sin gancho en workflows (con script + docs alcanza).
