# Portabilidad PC + Android (100 % Rust nativo + GPU)

> Stack: eframe 0.29 + wgpu 22 + egui 0.29. Sin Python, sin OpenSSL
> (reqwest usa rustls). Núcleo didáctico puro-Rust sin OS.

## S1 — check Android del núcleo: VERDE

```text
cargo check -p grafito-core -p grafito-geometry -p grafito-command \
  -p grafito-pedagogy -p grafito-tex \
  --target aarch64-linux-android --locked   # ok (22.47 s)
cargo check -p grafito-ggb -p grafito-complex \
  --target aarch64-linux-android --locked   # ok (4.32 s)
```

Todo el motor didáctico (comandos, geometría, pedagogía, TeX,
intercambio GGB, animación) compila para Android sin tocar una línea.

## Bloqueadores (severidad)

1. **Entry Android inexistente (CRÍTICO):** `main()` + `run_native`
   desktop; cero `cfg android`, cero actividad, cero manifiesto.
2. **`rfd` + rutas POSIX (CRÍTICO funcional):** sin Storage Access
   Framework; guardar/cargar necesita abstracción (scoped storage).
3. **Keyring/clipboard (ALTO):** D-Bus no existe; usar
   `--no-default-features` + clave de sesión (stubs ya existen).
4. **Touch real (ALTO UX):** `input.rs` solo ratón; pinch-zoom, pan a
   2 dedos y targets ≥44 px sin cablear.
5. **GPU fragmentada (ALTO):** Vulkan en gama alta / GLES en gama baja;
   MSAA 4× caro en tile-based; validar los 9 WGSL contra GLES.
   Mitiga el fallback CPU honesto a 250 ms (`gpu_readback.rs`).
6. **Sidecars `ffmpeg/piper` (MEDIO):** degradar a error honesto;
   GIF nativo (`gif` puro-Rust) + recetas = camino portable.
7. **Tamaño APK (MEDIO):** 37.4 MiB desktop como referencia;
   `--no-default-features` (−3 MiB), `cas-nativo` OFF, splits por ABI.
8. **Permisos/red (MEDIO-BAJO):** solo `INTERNET` opcional; talkback
   y teclado virtual sin verificar.
9. **Ventana mínima/breakpoints (BAJO):** `min 960×600` no aplica a
   fullscreen; calibrar `Compact <1360` a 360×640.

## Slices S2–S11 (regla: ningún slice toca registry ni plantillas)

- **S2** — matriz de features no-portables por crate.
- **S2** — HECHO (`59c8cb3`): feature `desktop-dialogs` (default ON) con `rfd`/`arboard` opcionales; 14 sitios tras `pick/save_file_native` + `copy_png_to_os_clipboard` con fallback honesto; app 1384 default + 1381 no-default verdes, clippy x2 limpio.
- **S3** — abstracción de storage (desktop idéntico + backend Android).
- **S4** — `rfd` tras la abstracción + stub honesto móvil.
- **S5** — entry Android mínimo (APK que abre/cierra, pause/resume).
  Sonda `check -p grafito-app --no-default-features --target aarch64-linux-android`: falla en `android-activity 0.6.1` (transitiva de winit) — `Either game-activity or native-activity must be enabled as features` + `activity_impl` ausente. Dirección: dependencia directa a `winit` con `android-native-activity` (o `android-activity/native-activity` pineada a la versión de winit) + `android_main` mínimo. Sin esto no hay APK.
- **S6** — política GPU móvil (Vulkan→GLES, MSAA adaptativo, 9 WGSL).
- **S7** — touch canvas (pinch + pan 2 dedos + targets 44).
- **S8** — degradación sidecars (error honesto + GIF/receta).
- **S9** — clipboard/keyring móvil (archivo + sesión).
- **S10** — dieta APK + CI Android (número por ABI + CI verde).
- **S11** — responsive 360×640 + a11y (layout + checklist manual).

Ya portable gratis: tessellation CPU (lyon/tiny-skia), WGSL
arch-independientes, timeouts GPU→CPU, features `assistant-net`/
`aula-iroh`/`cas-nativo` como interruptores, TLS sin OpenSSL,
`Compact` como base responsive, GIF+receta como formato móvil.
