# Obsidian Vault — instrucciones del plugin

El vault vive en `~/Documentos/GrafitoVault` (`GRAFITO_VAULT_DIR` lo
redirige). Es markdown plano que Obsidian abre directo: Grafito nunca
convierte ni migra, solo lee y escribe archivos `.md`.

## Formato de nota

- Frontmatter YAML mínimo: `título`, `materia`, `fecha`, `tags`.
- Wiki-links `[[nota]]` o `[[nota|alias]]` para conectar ideas.
- Tags `#materia/nivel` (ej. `#matematica/am1`, `#ingles/b2`, `#electronica/digital`).
- Una idea por línea en `Mapas/` (el layout radial GPU las usa tal cual).

## Flujo de estudio

1. Resumir: el asistente escribe `Resúmenes/<tema>.md` con frontmatter y
   cita la fuente en el cuerpo. Corto y re-legible, no transcripciones.
   Las cuentas se verifican con `evaluate_expr` antes de escribirlas.
2. Conectar: agrega `[[links]]` a notas relacionadas y `#tags` de materia.
3. Mapear: `study_mindmap` convierte una lista anidada en nodos con ángulo.
4. Evaluar: `quiz_grade` califica con el banco; lo no sabido vuelve a
   `Exámenes/` como repaso. Todo intento queda en el vault, nada solo en chat.
5. Agendar: `cal_op` + export ICS para llevar las sesiones al calendario.

## Límites honestos

- Nota máx 1 MiB, frontmatter máx 8 KiB, 2000 archivos por carga.
- Sin vault en disco → error `vault: no existe…`, jamás contenido inventado.
- Google Calendar es import manual por ICS hasta F3b (OAuth con client propio).
