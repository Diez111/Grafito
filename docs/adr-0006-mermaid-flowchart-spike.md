# ADR-0006 — Spike `mermaid flowchart` con `tiny-skia` (F17.3): diferido con evidencia

## Estado
Spike ejecutado 2026-10-05. **Diferido**: no se mergea a `main`. Queda cuantificado
y listo para retomar en la rama `spike/mermaid-flowchart-F17.3` (commit `b3bc813`,
base `810fbaf`).

## Contexto
El vault muestra bloques ````mermaid` como texto. La duda: ¿cuánto cuesta el
subset `graph TD/LR` (parse + layout + raster) con `tiny-skia` (ya en el árbol
vía `grafito-tex`) contra `resvg` (stack SVG completo)? Una variante sola.

## Evidencia (probe real, no estimación)
Worktree `/tmp/opencode/mermaid-spike` sobre HEAD `810fbaf`,
`tiny-skia = "=0.11.4"` (misma versión pineada que `grafito-tex`, `crates/grafito-tex/Cargo.toml:23`)
en `grafito-ui`, módulo `mermaid_spike.rs` (564 L, solo `#[cfg(test)]`, cero
cambio productivo): parser del subset (`graph`/`flowchart` + `TD`/`LR`,
nodos `A`/`A[txt]`/`A(txt)`/`A{txt}`, aristas ` -->`/`---` con cadenas
`A-->B-->C`, topes 32 nodos / 64 aristas / id 16 / label 64 truncado) +
layout en grilla por profundidad BFS desde raíces (ciclos al final en orden
de inserción) + raster a `Pixmap` (fondo N0, cajas con borde 1 px, aristas
en L de 2 px; **sin texto**: el shaping queda como follow-up). 6 tests
(parse TD/LR+labels, 7 errores honestos, layout determinista con tamaño
pineado, raster con fondo y centro de caja, tamaños pineados).

Memoria medida en este box (x86_64, Rust 1.92):

| Estado | Stack | Heap ejemplo 3 nodos TD 152×200 |
|---|---|---|
| `MermaidGraph` vacío | 56 B | nodos+aristas (`String`/`Vec`, cientos de B) |
| `MermaidLayout` vacío | 32 B | cajas `Vec<[f32; 4]>` (48 B p/3 nodos) |
| `Pixmap` 152×200 | — | 121 600 B (`w*h*4`, pineado) |

Bug cazado por el test: las cadenas `A-->B-->C` (el `muchos` del tope de
nodos) no parseaban (`splitn(2)` partía solo la primera arista y el resto
caía a `BadToken`): ahora se normaliza `---`→`-->` y se encadenan eslabones
por pares.

Costo de dependencias (medido): lock 856→856 paquetes (**+0**, solo +1 arista
`tiny-skia` en `grafito-ui`); cierre `cargo tree -p grafito-ui --prefix none | sort -u`
189 (= base, mismo set); MSRV intacta (`tiny-skia` 0.11.4 sin `rust-version`,
anterior a F3 y ya en el binario vía `grafito-tex`); release **+0 bytes**
(el spike es solo `#[cfg(test)]`; de mergearse como prod, `tiny-skia` ya
está en el binario y el código propio son ~564 L sin shaping).

Gates del spike en su rama: `cargo fmt -p grafito-ui` 0,
`cargo clippy -p grafito-ui --all-targets --locked -- -D warnings` 0,
`cargo test -p grafito-ui --locked --lib` 443/0 (437 base + 6 del spike).

## Variante descartada (inspección, no medida en el árbol)
`resvg` 0.46.0 (edition 2024, rust 1.87, MSRV ok) trae stack SVG completo
(`usvg` con `text`/`system-fonts`/`memmap-fonts`, `gif`, `image-webp`,
`zune-jpeg`, `pico-args`, `rgb`, `svgtypes`, más `tiny-skia`) + `[[bin]]` y
examples: desproporcionado para un subset flowchart sin SVG ni texto (el
spike ni dibuja glifos). No se agregó al árbol a propósito.

## Decisión
Diferir. El costo absoluto es nulo en deps y ~119 KiB de pixmap por diagrama
chico (irrelevante frente a una escena); la integración real es el shaping
de labels (traer `ttf-parser`/fuente al vault o reusar el pipeline de
`grafito-tex`), el cableado a ```mermaid en el visor y el caché por
`(vault_rev, hash_bloque)`. Nada se mergea; `main` solo recibe este ADR.

## Retomada (cuando Studio exija diagramas en notas)
1. Partir de `spike/mermaid-flowchart-F17.3` (`b3bc813`) como contrato.
2. Sumar shaping (reusar `TintaContorno`/`ttf-parser` 0.25.1 de `grafito-tex`)
   antes de tocar el visor del vault.
3. Cachear por `(vault_rev, hash_bloque)` con cota 1 MiP y cerrar con los
   4 gates + E2E `computer-control`.
