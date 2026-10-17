//! Visor markdown céntrico (Piel pura, sin deps nuevas).
//!
//! Subset honesto sin `pulldown-cmark`/`comrak`/`egui_commonmark`: parser de
//! líneas que emite `Vec<MdBlock>` y `draw_md_blocks` que los pinta con
//! `RichText`/tokens. Base reutilizable: `AssistantMessageBlock` y el layout
//! de `assistant.rs` (tablas vía `draw_markdown_table`, math vía
//! `draw_math_with_locale` = `grafito-tex` con fallback al subset).
//!
//! Wikilinks `[[nota]]`/`[[nota|alias]]` → botón (`MdAction::OpenLink`),
//! `#tag` → chip (`MdAction::OpenTag`). La edición sigue en
//! `TextEdit` (Fuente/Preview): esto solo MUESTRA.

use egui::{Color32, RichText};

use crate::i18n::Locale;
use crate::tokens::{RADIUS_MD, SPACE_SM, SPACE_XS, TYPE_LG, TYPE_MD, TYPE_SM, TYPE_XS};

/// Bloque del subset (lo que el visor sabe pintar).
#[derive(Debug, Clone, PartialEq)]
pub enum MdBlock {
    Heading { level: usize, text: String },
    Bullet(String),
    Ordered { number: u32, text: String },
    Quote(String),
    Paragraph(String),
    Code { language: String, text: String },
    Math(String),
    Table(Vec<Vec<String>>),
}

/// Tramo inline dentro de un bloque de texto.
#[derive(Debug, Clone, PartialEq)]
pub enum MdInline {
    Text(String),
    Bold(String),
    Italic(String),
    Code(String),
    Link { target: String, label: String },
    Tag(String),
}

/// Acción del visor hacia la app (navegación, nunca edición).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MdAction {
    OpenLink(String),
    OpenTag(String),
}

/// Parsea el subset a bloques. Puro, sin pánico, lineal.
///
/// - Encabezado: `#`..`######` + espacio (`#tag` sin espacio es tag, no
///   título). Niveles `> 6` se capean a 6.
/// - Cita: `>` + espacio opcional. Viñeta: `-`/`*`/`+` + espacio.
///   Ordenada: dígitos + `.`/`)` + espacio (número `u32` saturante).
/// - Tabla: racha de ≥2 líneas con `|`; la fila separadora (`---`) se salta
///   y las filas se rellenan al ancho máximo (nunca paniquea el `Grid`).
///   Una sola línea con `|` queda como párrafo (no es tabla).
/// - Code fence sin cerrar: se muestra igual como código (visor, no stream).
/// - Math display: `$$…$$` en una línea o bloque `$$ … $$` multilínea.
#[must_use]
pub fn parse_md_blocks(src: &str) -> Vec<MdBlock> {
    let lines: Vec<&str> = src.lines().collect();
    let mut blocks = Vec::new();
    let mut paragraph: Vec<&str> = Vec::new();
    let mut index = 0;

    let flush = |blocks: &mut Vec<MdBlock>, paragraph: &mut Vec<&str>| {
        if !paragraph.is_empty() {
            blocks.push(MdBlock::Paragraph(paragraph.join("\n")));
            paragraph.clear();
        }
    };

    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim();
        if trimmed.is_empty() {
            flush(&mut blocks, &mut paragraph);
            index += 1;
            continue;
        }
        // Code fence.
        if let Some(language) = trimmed.strip_prefix("```") {
            flush(&mut blocks, &mut paragraph);
            let mut body: Vec<&str> = Vec::new();
            index += 1;
            while index < lines.len() && !lines[index].trim_start().starts_with("```") {
                body.push(lines[index]);
                index += 1;
            }
            // Consume el cierre si existe; sin cierre se muestra igual.
            if index < lines.len() {
                index += 1;
            }
            blocks.push(MdBlock::Code {
                language: language.trim().to_ascii_lowercase(),
                text: body.join("\n"),
            });
            continue;
        }
        // Math display multilínea `$$ … $$`.
        if trimmed == "$$" {
            flush(&mut blocks, &mut paragraph);
            let mut body: Vec<&str> = Vec::new();
            index += 1;
            while index < lines.len() && lines[index].trim() != "$$" {
                body.push(lines[index]);
                index += 1;
            }
            if index < lines.len() {
                index += 1;
            }
            let math = body.join("\n");
            if math.trim().is_empty() {
                paragraph.push(line);
            } else {
                blocks.push(MdBlock::Math(math.trim().to_string()));
            }
            continue;
        }
        // Math display en una línea `$$…$$`.
        if let Some(math) = trimmed
            .strip_prefix("$$")
            .and_then(|v| v.strip_suffix("$$"))
            .filter(|v| !v.trim().is_empty())
        {
            flush(&mut blocks, &mut paragraph);
            blocks.push(MdBlock::Math(math.trim().to_string()));
            index += 1;
            continue;
        }
        // Encabezado (`# ` con espacio; `#tag` es tag, no título).
        if let Some((level, text)) = parse_md_heading(trimmed) {
            flush(&mut blocks, &mut paragraph);
            blocks.push(MdBlock::Heading { level, text });
            index += 1;
            continue;
        }
        // Cita.
        if let Some(text) = trimmed.strip_prefix('>') {
            flush(&mut blocks, &mut paragraph);
            blocks.push(MdBlock::Quote(
                text.strip_prefix(' ').unwrap_or(text).to_string(),
            ));
            index += 1;
            continue;
        }
        // Viñeta.
        if let Some(text) = parse_md_bullet(trimmed) {
            flush(&mut blocks, &mut paragraph);
            blocks.push(MdBlock::Bullet(text));
            index += 1;
            continue;
        }
        // Ordenada.
        if let Some((number, text)) = parse_md_ordered(trimmed) {
            flush(&mut blocks, &mut paragraph);
            blocks.push(MdBlock::Ordered { number, text });
            index += 1;
            continue;
        }
        // Tabla: racha de ≥2 líneas con `|`.
        if trimmed.contains('|') {
            let mut run: Vec<&str> = Vec::new();
            while index < lines.len() {
                let candidate = lines[index].trim();
                if candidate.is_empty() || !candidate.contains('|') {
                    break;
                }
                run.push(lines[index]);
                index += 1;
            }
            if run.len() >= 2 {
                flush(&mut blocks, &mut paragraph);
                let rows = parse_md_table_rows(&run);
                if rows.is_empty() {
                    for row_line in run {
                        blocks.push(MdBlock::Paragraph(row_line.trim().to_string()));
                    }
                } else {
                    blocks.push(MdBlock::Table(rows));
                }
                continue;
            }
            // Una sola línea con `|`: cae al párrafo (no es tabla).
            paragraph.push(line);
            continue;
        }
        paragraph.push(line);
        index += 1;
    }
    flush(&mut blocks, &mut paragraph);
    blocks
}

fn parse_md_heading(trimmed: &str) -> Option<(usize, String)> {
    let hashes = trimmed.chars().take_while(|c| *c == '#').count();
    if !(1..=6).contains(&hashes) {
        return None;
    }
    let rest = &trimmed[hashes..];
    let text = rest.strip_prefix(' ')?;
    Some((hashes, (*text).to_string()))
}

fn parse_md_bullet(trimmed: &str) -> Option<String> {
    for marker in ["- ", "* ", "+ "] {
        if let Some(text) = trimmed.strip_prefix(marker) {
            return Some(text.to_string());
        }
    }
    None
}

fn parse_md_ordered(trimmed: &str) -> Option<(u32, String)> {
    let digits = trimmed.chars().take_while(char::is_ascii_digit).count();
    if digits == 0 || digits > 9 {
        return None;
    }
    let rest = &trimmed[digits..];
    let text = rest
        .strip_prefix(". ")
        .or_else(|| rest.strip_prefix(") "))?;
    let number: u32 = trimmed[..digits].parse().unwrap_or(1);
    Some((number, (*text).to_string()))
}

fn is_md_separator_row(cells: &[String]) -> bool {
    !cells.is_empty()
        && cells.iter().all(|c| {
            let c = c.trim().trim_matches(':');
            !c.is_empty() && c.chars().all(|ch| ch == '-')
        })
}

fn parse_md_table_rows(run: &[&str]) -> Vec<Vec<String>> {
    let mut rows: Vec<Vec<String>> = Vec::new();
    for line in run {
        let inner = line.trim().trim_matches('|');
        let cells: Vec<String> = inner.split('|').map(|c| c.trim().to_string()).collect();
        if cells.len() < 2 {
            return Vec::new();
        }
        if is_md_separator_row(&cells) {
            continue;
        }
        rows.push(cells);
    }
    if rows.is_empty() {
        return Vec::new();
    }
    let width = rows.iter().map(Vec::len).max().unwrap_or(1);
    for row in &mut rows {
        while row.len() < width {
            row.push(String::new());
        }
    }
    rows
}

/// Parsea tramos inline: `**fuerte**`, `*suave*`, `` `código` ``,
/// `[[destino]]`/`[[destino|alias]]`, `#tag`.
///
/// Sin cierre → texto literal (nunca jerga a medias, nunca pánico).
/// `#` solo es tag si lo precede inicio/blanco/puntuación (no `C#`).
#[must_use]
pub fn parse_md_inline(s: &str) -> Vec<MdInline> {
    let mut out: Vec<MdInline> = Vec::new();
    let mut text = String::new();
    let mut rest = s;
    let mut prev_is_word = false;

    let push_text = |out: &mut Vec<MdInline>, text: &mut String| {
        if !text.is_empty() {
            out.push(MdInline::Text(std::mem::take(text)));
        }
    };

    while !rest.is_empty() {
        if let Some(after) = rest.strip_prefix("**") {
            if let Some(end) = after.find("**") {
                push_text(&mut out, &mut text);
                out.push(MdInline::Bold(after[..end].to_string()));
                rest = &after[end + 2..];
                prev_is_word = true;
                continue;
            }
            text.push_str("**");
            rest = after;
            prev_is_word = false;
            continue;
        }
        if let Some(after) = rest.strip_prefix('`') {
            if let Some(end) = after.find('`') {
                push_text(&mut out, &mut text);
                out.push(MdInline::Code(after[..end].to_string()));
                rest = &after[end + 1..];
                prev_is_word = true;
                continue;
            }
            text.push('`');
            rest = after;
            continue;
        }
        if let Some(after) = rest.strip_prefix('*') {
            if let Some(end) = after.find('*') {
                push_text(&mut out, &mut text);
                out.push(MdInline::Italic(after[..end].to_string()));
                rest = &after[end + 1..];
                prev_is_word = true;
                continue;
            }
            text.push('*');
            rest = after;
            continue;
        }
        if let Some(after) = rest.strip_prefix("[[") {
            if let Some(end) = after.find("]]") {
                let inner = &after[..end];
                let (target, label) = match inner.split_once('|') {
                    Some((t, l)) => (t.trim(), l.trim()),
                    None => (inner.trim(), inner.trim()),
                };
                if !target.is_empty() {
                    push_text(&mut out, &mut text);
                    out.push(MdInline::Link {
                        target: target.to_string(),
                        label: if label.is_empty() {
                            target.to_string()
                        } else {
                            label.to_string()
                        },
                    });
                    rest = &after[end + 2..];
                    prev_is_word = true;
                    continue;
                }
            }
            text.push_str("[[");
            rest = after;
            continue;
        }
        if let Some(after) = rest.strip_prefix('#') {
            let len = after
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '-' | '_'))
                .map(char::len_utf8)
                .sum::<usize>();
            if len > 0 && !prev_is_word {
                push_text(&mut out, &mut text);
                out.push(MdInline::Tag(after[..len].to_string()));
                rest = &after[len..];
                prev_is_word = true;
                continue;
            }
            text.push('#');
            rest = after;
            prev_is_word = false;
            continue;
        }
        let mut chars = rest.char_indices();
        let (_, first) = chars.next().unwrap_or((0, '\0'));
        if first == '\0' {
            break;
        }
        text.push(first);
        prev_is_word = first.is_alphanumeric();
        if let Some((next_byte, _)) = chars.next() {
            rest = &rest[next_byte..];
        } else {
            rest = &rest[rest.len()..];
        }
    }
    push_text(&mut out, &mut text);
    out
}

/// Estilo de un flujo inline (encabezado/cita/párrafo comparten el motor).
struct MdFlowStyle {
    size: f32,
    color: Color32,
    italic: bool,
}

fn md_flow_formats(
    theme: &crate::theme::Theme,
    style: &MdFlowStyle,
) -> (egui::TextFormat, egui::TextFormat) {
    let base = egui::TextFormat {
        font_id: egui::FontId::proportional(style.size),
        color: style.color,
        italics: style.italic,
        ..Default::default()
    };
    let code = egui::TextFormat {
        font_id: egui::FontId::monospace(style.size),
        color: theme.code_text,
        background: theme.accent_muted,
        ..Default::default()
    };
    // Negrita con la misma familia proporcional (igual que el transcript
    // del asistente): el peso del bloque lo da el tamaño, no la jerga.
    (base, code)
}

fn md_flow_flush(ui: &mut egui::Ui, job: &mut egui::text::LayoutJob) {
    if job.text.is_empty() {
        return;
    }
    ui.add(
        egui::Label::new(std::mem::take(job))
            .wrap()
            .selectable(true),
    );
}

/// Pinta un texto con inline-formatting: links → botón, tags → chip.
fn draw_md_flow(
    ui: &mut egui::Ui,
    raw: &str,
    style: &MdFlowStyle,
    on_action: &mut impl FnMut(MdAction),
) {
    let theme = crate::theme::current_theme(ui.ctx());
    let (base, code_fmt) = md_flow_formats(theme, style);
    ui.horizontal_wrapped(|ui| {
        let mut job = egui::text::LayoutJob::default();
        for span in parse_md_inline(raw) {
            match span {
                MdInline::Text(t) => {
                    job.append(&t, 0.0, base.clone());
                }
                MdInline::Bold(t) | MdInline::Italic(t) => {
                    // Misma familia proporcional que el transcript: el
                    // énfasis viaja en el peso del bloque, no en jerga.
                    job.append(&t, 0.0, base.clone());
                }
                MdInline::Code(t) => {
                    job.append(&t, 0.0, code_fmt.clone());
                }
                MdInline::Link { target, label } => {
                    md_flow_flush(ui, &mut job);
                    let link = ui.add(
                        egui::Button::new(RichText::new(label.clone()).size(style.size).strong())
                            .frame(false),
                    );
                    if link.clicked() {
                        on_action(MdAction::OpenLink(target.clone()));
                    }
                    link.on_hover_text(format!("[[{target}]]"));
                }
                MdInline::Tag(tag) => {
                    md_flow_flush(ui, &mut job);
                    let chip = ui.add(
                        egui::Button::new(
                            RichText::new(format!("#{tag}"))
                                .size(style.size)
                                .color(theme.accent),
                        )
                        .frame(false),
                    );
                    if chip.clicked() {
                        on_action(MdAction::OpenTag(tag.clone()));
                    }
                }
            }
        }
        md_flow_flush(ui, &mut job);
    });
}

/// Pinta bloques ya parseados (la edición sigue en `TextEdit`).
///
/// Tablas → `draw_markdown_table` del asistente; math `$$` → `grafito-tex`
/// con fallback honesto al subset. `block_base` numera las tablas para el
/// `id_salt` interno (estable por documento).
pub fn draw_md_blocks(
    ui: &mut egui::Ui,
    blocks: &[MdBlock],
    locale: Locale,
    mut on_action: impl FnMut(MdAction),
) {
    let theme = crate::theme::current_theme(ui.ctx());
    let mut table_count = 0;
    for block in blocks {
        match block {
            MdBlock::Heading { level, text } => {
                let size = match level {
                    1 => TYPE_LG,
                    2 => TYPE_MD,
                    _ => TYPE_SM,
                };
                draw_md_flow(
                    ui,
                    text,
                    &MdFlowStyle {
                        size,
                        color: theme.text_primary,
                        italic: false,
                    },
                    &mut on_action,
                );
                ui.add_space(SPACE_XS);
            }
            MdBlock::Bullet(text) => {
                ui.horizontal_top(|ui| {
                    ui.label(RichText::new("·").color(theme.accent).size(TYPE_SM));
                    draw_md_flow(
                        ui,
                        text,
                        &MdFlowStyle {
                            size: TYPE_SM,
                            color: theme.text_primary,
                            italic: false,
                        },
                        &mut on_action,
                    );
                });
            }
            MdBlock::Ordered { number, text } => {
                ui.horizontal_top(|ui| {
                    ui.label(
                        RichText::new(format!("{number}."))
                            .strong()
                            .size(TYPE_SM)
                            .color(theme.accent),
                    );
                    draw_md_flow(
                        ui,
                        text,
                        &MdFlowStyle {
                            size: TYPE_SM,
                            color: theme.text_primary,
                            italic: false,
                        },
                        &mut on_action,
                    );
                });
            }
            MdBlock::Quote(text) => {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = SPACE_SM;
                    let (bar, _) =
                        ui.allocate_exact_size(egui::vec2(4.0, 16.0), egui::Sense::hover());
                    ui.painter().rect_filled(bar, 1.0, theme.accent);
                    draw_md_flow(
                        ui,
                        text,
                        &MdFlowStyle {
                            size: TYPE_SM,
                            color: theme.text_secondary,
                            italic: true,
                        },
                        &mut on_action,
                    );
                });
            }
            MdBlock::Paragraph(text) => {
                draw_md_flow(
                    ui,
                    text,
                    &MdFlowStyle {
                        size: TYPE_SM,
                        color: theme.text_primary,
                        italic: false,
                    },
                    &mut on_action,
                );
                ui.add_space(SPACE_XS);
            }
            MdBlock::Code { language, text } => {
                let title = if language.is_empty() {
                    "código".to_string()
                } else {
                    language.clone()
                };
                egui::Frame::none()
                    .fill(theme.input_bg)
                    .stroke(egui::Stroke::new(1.0, theme.separator.gamma_multiply(0.10)))
                    .rounding(RADIUS_MD)
                    .inner_margin(egui::Margin::same(SPACE_SM))
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.label(
                            RichText::new(title)
                                .strong()
                                .size(TYPE_XS)
                                .color(theme.accent),
                        );
                        ui.label(
                            RichText::new(text)
                                .monospace()
                                .size(TYPE_SM)
                                .color(theme.text_primary),
                        );
                    });
                ui.add_space(SPACE_XS);
            }
            MdBlock::Math(math) => {
                crate::assistant::draw_math_with_locale(ui, math, locale);
                ui.add_space(SPACE_XS);
            }
            MdBlock::Table(rows) => {
                crate::assistant::draw_markdown_table(ui, rows, usize::MAX, table_count);
                table_count += 1;
                ui.add_space(SPACE_XS);
            }
        }
    }
}

/// Sección "Citado por" del visor: `links` ya viene resuelto por el llamador.
/// Vacío → nada (el llamador decide si mostrar el panel).
pub fn draw_md_backlinks(
    ui: &mut egui::Ui,
    links: &[String],
    locale: Locale,
    mut on_action: impl FnMut(MdAction),
) {
    if links.is_empty() {
        return;
    }
    let theme = crate::theme::current_theme(ui.ctx());
    ui.separator();
    ui.label(
        RichText::new(crate::i18n::vault_msg("backlinks_title", locale))
            .size(TYPE_XS)
            .strong()
            .color(theme.text_secondary),
    );
    for link in links.iter().take(8) {
        if ui
            .add(egui::Button::new(RichText::new(link).size(TYPE_SM)).frame(false))
            .clicked()
        {
            on_action(MdAction::OpenLink(link.clone()));
        }
    }
}

/// Backlinks puros: qué rutas citan a `target` con `[[link]]`.
/// `docs` = `(ruta, links)`; compara stem o ruta exacta, case-insensitive.
#[must_use]
pub fn backlinks<'a>(docs: &[(&'a str, Vec<String>)], target: &str) -> Vec<&'a str> {
    let want = target.trim().to_lowercase();
    let stem = want
        .rsplit('/')
        .next()
        .unwrap_or(&want)
        .trim_end_matches(".md")
        .to_string();
    let mut out: Vec<&str> = docs
        .iter()
        .filter(|(path, links)| {
            *path != target
                && links.iter().any(|l| {
                    let l = l.to_lowercase();
                    l == want || l == stem
                })
        })
        .map(|(path, _)| *path)
        .collect();
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn md_view_h1_bold_bullet_quote_code() {
        let src = "# Energía\n\nTexto **fuerte** y *suave* con `E=mc2`.\n\n- uno\n- dos\n\n> cita sabia\n\n```rust\nlet a = 1;\n```\n\n$$x^2$$\n";
        let blocks = parse_md_blocks(src);
        assert_eq!(
            blocks[0],
            MdBlock::Heading {
                level: 1,
                text: "Energía".to_string()
            }
        );
        assert!(matches!(blocks[1], MdBlock::Paragraph(_)));
        let spans = parse_md_inline("Texto **fuerte** y *suave* con `E=mc2`.");
        assert_eq!(spans[0], MdInline::Text("Texto ".to_string()));
        assert_eq!(spans[1], MdInline::Bold("fuerte".to_string()));
        assert_eq!(spans[2], MdInline::Text(" y ".to_string()));
        assert_eq!(spans[3], MdInline::Italic("suave".to_string()));
        assert_eq!(spans[4], MdInline::Text(" con ".to_string()));
        assert_eq!(spans[5], MdInline::Code("E=mc2".to_string()));
        assert_eq!(spans[6], MdInline::Text(".".to_string()));
        assert_eq!(blocks[2], MdBlock::Bullet("uno".to_string()));
        assert_eq!(blocks[3], MdBlock::Bullet("dos".to_string()));
        assert_eq!(blocks[4], MdBlock::Quote("cita sabia".to_string()));
        assert_eq!(
            blocks[5],
            MdBlock::Code {
                language: "rust".to_string(),
                text: "let a = 1;".to_string()
            }
        );
        assert_eq!(blocks[6], MdBlock::Math("x^2".to_string()));
        assert_eq!(blocks.len(), 7);
        // Sin cierre → literal, nunca jerga a medias ni pánico.
        assert_eq!(
            parse_md_inline("esto **queda"),
            vec![MdInline::Text("esto **queda".to_string())]
        );
        // `#tag` sin espacio NO es título.
        let tag_doc = parse_md_blocks("#parcial de Física\n");
        assert!(matches!(tag_doc[0], MdBlock::Paragraph(_)));
        let tag_spans = parse_md_inline("#parcial de Física");
        assert_eq!(tag_spans[0], MdInline::Tag("parcial".to_string()));
    }

    #[test]
    fn md_view_wikilink_tag_clicables() {
        let spans = parse_md_inline("Ver [[Física|apuntes]] y [[Óptica]] (#parcial).");
        assert_eq!(
            spans[1],
            MdInline::Link {
                target: "Física".to_string(),
                label: "apuntes".to_string(),
            }
        );
        assert_eq!(
            spans[3],
            MdInline::Link {
                target: "Óptica".to_string(),
                label: "Óptica".to_string(),
            }
        );
        assert_eq!(spans[5], MdInline::Tag("parcial".to_string()));
        // `C#` no es tag; `[[sin cierre` es literal.
        let tricky = parse_md_inline("C# no tag, [[sin cierre");
        assert!(tricky.iter().all(|s| !matches!(s, MdInline::Tag(_))));
        assert!(tricky.iter().all(|s| !matches!(s, MdInline::Link { .. })));
        // Tabla mínima + backlinks del vault.
        let table = parse_md_blocks("| a | b |\n| 1 | 2 |\n");
        assert_eq!(
            table[0],
            MdBlock::Table(vec![
                vec!["a".to_string(), "b".to_string()],
                vec!["1".to_string(), "2".to_string()],
            ])
        );
        let docs = vec![
            ("Resúmenes/mate.md", vec!["Física".to_string()]),
            ("Física.md", Vec::new()),
        ];
        let cited = backlinks(&docs, "Física");
        assert_eq!(cited, vec!["Resúmenes/mate.md"]);
    }
}
