//! Fuentes UTNianos (foro MyBB) para el asistente: buscar y citar (Fase 1, solo lectura).
//!
//! Integración puntual y honesta con `www.utnianos.com.ar`, sin API pública:
//! - `buscar_resumen{materia, tipo, query}` busca con el backend web existente
//!   acotado a `site:utnianos.com.ar` (el `search.php` interno está bloqueado
//!   por `robots.txt` y jamás se toca), filtra a slugs estables `tema-...` y
//!   rankea en memoria. Los candidatos salen con `verificado: false`: la URL
//!   sola no prueba nada.
//! - `citar_fuente{url}` hace UN fetch puntual al tema, extrae título, autor,
//!   fecha y un fragmento mínimo (tope [`UTNIANO_FRAGMENT_MAX_CHARS`], jamás
//!   el post entero) más la fecha de acceso. Ante login, 403/404 o redirect a
//!   login falla honesto en español. Nunca usa credenciales ni descarga masiva.
//!
//! Reglas pineadas:
//! - Toda URL citada debe pasar por [`normalize_utnianos_url`]: host exacto del
//!   foro + slug `tema-...`. Dominios ajenos y rutas bloqueadas por robots
//!   (`attachment.php`, `search.php`, `member.php`) se rechazan ANTES de
//!   cualquier fetch, así que una URL inventada jamás llega a la red.
//! - El parseo/rankeo es puro y determinista (sin red): los tests usan fixtures
//!   HTML locales en `tests/fixtures/`, jamás fetch real en tests.

use serde::Serialize;

use crate::web::{sanitize_html_text, WebSearchResult};

/// Hosts válidos del foro (sin subdominios inventados ni espejos).
pub const UTNIANO_HOSTS: [&str; 2] = ["www.utnianos.com.ar", "utnianos.com.ar"];
/// Máximo de candidatos devueltos (paridad con `WEB_SEARCH_MAX_RESULTS`).
pub const UTNIANO_MAX_RESULTS: usize = 5;
/// Cap de caracteres de cada campo libre de búsqueda.
pub const UTNIANO_QUERY_MAX_CHARS: usize = 256;
/// Cap de caracteres del título citado.
pub const UTNIANO_TITLE_MAX_CHARS: usize = 200;
/// Cap de caracteres del nombre del autor citado.
pub const UTNIANO_AUTHOR_MAX_CHARS: usize = 64;
/// Cap de caracteres del fragmento mínimo (cita, no republicación).
pub const UTNIANO_FRAGMENT_MAX_CHARS: usize = 400;
/// Cap de caracteres de una URL aceptada.
pub const UTNIANO_URL_MAX_CHARS: usize = 2_000;
/// Timeout del fetch puntual.
pub const UTNIANO_TIMEOUT_MS: u64 = 8_000;
/// Cap del cuerpo leído del tema (OOM-safe).
#[cfg(feature = "assistant-net")]
const UTNIANO_RESPONSE_MAX_BYTES: usize = 512 * 1024;
/// UA explícito, mismo estilo que el resto del transporte.
#[cfg(feature = "assistant-net")]
const UTNIANO_USER_AGENT: &str = "Mozilla/5.0 (compatible; Grafito/1.1; +https://www.grafito.org/)";

/// Candidato de `buscar_resumen`: NO verificado (ver `verificado`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UtnianoCandidate {
    /// Título del tema, sin HTML.
    pub titulo: String,
    /// Autor sólo si el snippet lo trae; si no, `None` (verificar con `citar_fuente`).
    pub autor: Option<String>,
    /// URL estable `tema-...` del foro.
    pub url: String,
    /// Fecha sólo si el snippet la trae; si no, `None`.
    pub fecha: Option<String>,
    /// Agradecimientos sólo si el snippet los trae; si no, `None`.
    pub agradecimientos: Option<u32>,
    /// Resumen del snippet, sin HTML.
    pub fragmento: String,
    /// Siempre `false` en búsqueda: la URL sola no prueba que el tema exista.
    pub verificado: bool,
}

/// Cita de `citar_fuente`: verificada por fetch puntual.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UtnianoCitation {
    /// Título del tema (`<title>`), sin HTML.
    pub titulo: String,
    /// Autor del primer post (link `member.php`), si se pudo extraer.
    pub autor: Option<String>,
    /// Fecha del primer post (`dd-mm-aaaa` o `aaaa-mm-dd`), si se pudo extraer.
    pub fecha: Option<String>,
    /// URL verificada por fetch.
    pub url: String,
    /// Fragmento mínimo del primer post (tope [`UTNIANO_FRAGMENT_MAX_CHARS`]).
    pub fragmento: String,
    /// Fecha de acceso en UTC (`aaaa-mm-dd`).
    pub fecha_acceso: String,
    /// Siempre `true`: hubo fetch 2xx + parseo.
    pub verificado: bool,
}

/// Valida y normaliza una URL candidata a cita (pura, sin red).
///
/// Acepta sólo `http(s)` con host exacto del foro y slug estable `tema-...`.
/// Rechaza antes de cualquier fetch: dominios ajenos (URLs inventadas),
/// credenciales embebidas, rutas bloqueadas por `robots.txt`
/// (`attachment.php`, `search.php`, `member.php`) y páginas que no son temas.
/// El `Err` es honesto y en español para mostrar al modelo/usuario.
pub fn normalize_utnianos_url(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("citar_fuente necesita una URL no vacía; conseguila con buscar_resumen".into());
    }
    if trimmed.contains('\0') {
        return Err("la URL contiene un carácter NUL y no es citable".into());
    }
    if trimmed.chars().any(char::is_control) {
        return Err("la URL contiene caracteres de control y no es citable".into());
    }
    let lower = trimmed.to_ascii_lowercase();
    if !(lower.starts_with("http://") || lower.starts_with("https://")) {
        return Err(format!(
            "solo se citan URLs http(s) del foro UTNianos; '{trimmed}' no es una fuente verificable"
        ));
    }
    let parsed = url::Url::parse(trimmed)
        .map_err(|_| "la URL no se pudo interpretar; no es citable".to_string())?;
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(
            "la URL trae credenciales embebidas; Grafito nunca usa login, pedí un aporte público"
                .into(),
        );
    }
    let host = parsed.host_str().unwrap_or_default().to_ascii_lowercase();
    if !UTNIANO_HOSTS.contains(&host.as_str()) {
        return Err(format!(
            "'{trimmed}' no es del foro UTNianos (hosts válidos: {}); conseguí la URL con buscar_resumen y verificala con citar_fuente",
            UTNIANO_HOSTS.join(", ")
        ));
    }
    let path = parsed.path().to_ascii_lowercase();
    if path.contains("attachment.php") {
        return Err(
            "attachment.php está bloqueado por robots.txt y exige descarga directa; Grafito no descarga adjuntos, citá el tema que lo contiene"
                .into(),
        );
    }
    if path.contains("search.php") {
        return Err(
            "search.php está bloqueado por robots.txt; buscá con buscar_resumen en vez de la búsqueda interna del foro"
                .into(),
        );
    }
    if path.contains("member.php") {
        return Err(
            "member.php está bloqueado por robots.txt; no se citan perfiles, citá el tema del aporte"
                .into(),
        );
    }
    if !path.contains("tema-") {
        return Err(format!(
            "'{trimmed}' no es un tema del foro (falta el slug estable `tema-...`); no es un aporte citable"
        ));
    }
    if trimmed.chars().count() > UTNIANO_URL_MAX_CHARS {
        return Err(format!(
            "la URL supera {UTNIANO_URL_MAX_CHARS} caracteres y no es citable"
        ));
    }
    Ok(trimmed.to_string())
}

/// Arma la query acotada al foro (`site:` + materia + tipo + query).
///
/// Pura: valida NUL/vacío y capa por chars sin partir multibyte.
pub fn utnianos_site_query(query: &str, materia: &str, tipo: &str) -> Result<String, String> {
    for (nombre, valor) in [("query", query), ("materia", materia), ("tipo", tipo)] {
        if valor.contains('\0') {
            return Err(format!(
                "buscar_resumen: '{nombre}' contiene un carácter NUL"
            ));
        }
    }
    if query.trim().is_empty() {
        return Err("buscar_resumen necesita una query no vacía".into());
    }
    let mut parts = vec!["site:utnianos.com.ar".to_string()];
    for part in [materia, tipo, query] {
        let collapsed = collapse_whitespace(part);
        if !collapsed.is_empty() {
            parts.push(collapsed);
        }
    }
    let joined = parts.join(" ");
    Ok(truncate_chars(&joined, UTNIANO_QUERY_MAX_CHARS).to_string())
}

/// Filtra resultados web a temas UTNianos citables y los rankea (puro, sin red).
///
/// Descarta todo lo que [`normalize_utnianos_url`] rechaza (dominios ajenos,
/// `attachment.php`/`search.php`/`member.php`, páginas sin `tema-...`) y
/// ordena por coincidencia con query/materia/tipo (estable: a igual puntaje
/// conserva el orden del backend). Capa a [`UTNIANO_MAX_RESULTS`].
/// Autor/fecha/agradecimientos salen `None`: el snippet no los trae de forma
/// fiable; se completan con `citar_fuente` (`verificado: false` lo recuerda).
pub fn select_utnianos_candidates(
    results: &[WebSearchResult],
    query: &str,
    materia: &str,
    tipo: &str,
) -> Vec<UtnianoCandidate> {
    let mut scored: Vec<(u32, usize, UtnianoCandidate)> = Vec::new();
    for (index, result) in results.iter().enumerate() {
        let Ok(url) = normalize_utnianos_url(&result.url) else {
            continue;
        };
        let titulo =
            truncate_chars(&sanitize_html_text(&result.title), UTNIANO_TITLE_MAX_CHARS).to_string();
        if titulo.is_empty() {
            continue;
        }
        let fragmento = truncate_chars(
            &sanitize_html_text(&result.snippet),
            UTNIANO_FRAGMENT_MAX_CHARS,
        )
        .to_string();
        let score = utnianos_score(&titulo, &fragmento, query, materia, tipo);
        scored.push((
            score,
            index,
            UtnianoCandidate {
                titulo,
                autor: None,
                url,
                fecha: None,
                agradecimientos: None,
                fragmento,
                verificado: false,
            },
        ));
    }
    scored.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));
    scored.truncate(UTNIANO_MAX_RESULTS);
    scored
        .into_iter()
        .map(|(_, _, candidate)| candidate)
        .collect()
}

/// Puntaje de un candidato: coincidencias en título pesan más que en snippet;
/// materia y tipo suman como contexto de la materia pedida.
fn utnianos_score(title: &str, snippet: &str, query: &str, materia: &str, tipo: &str) -> u32 {
    let title = title.to_lowercase();
    let snippet = snippet.to_lowercase();
    let mut score = 0u32;
    for token in word_tokens(query) {
        if title.contains(&token) {
            score = score.saturating_add(10);
        } else if snippet.contains(&token) {
            score = score.saturating_add(3);
        }
    }
    for token in word_tokens(materia) {
        if title.contains(&token) {
            score = score.saturating_add(5);
        } else if snippet.contains(&token) {
            score = score.saturating_add(2);
        }
    }
    for token in word_tokens(tipo) {
        if title.contains(&token) {
            score = score.saturating_add(3);
        } else if snippet.contains(&token) {
            score = score.saturating_add(1);
        }
    }
    score
}

/// Tokens minúsculos alfanuméricos de largo ≥ 2 (`AM1`, `final`, ...).
fn word_tokens(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|character: char| !character.is_alphanumeric())
        .filter(|token| token.chars().count() >= 2)
        .map(str::to_string)
        .collect()
}

/// Colapsa espacios/saltos/tabs a un solo espacio y recorta bordes.
fn collapse_whitespace(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut pending_space = false;
    for character in text.chars() {
        if character.is_whitespace() {
            pending_space = true;
        } else {
            if pending_space && !out.is_empty() {
                out.push(' ');
            }
            pending_space = false;
            out.push(character);
        }
    }
    out
}

/// Recorta a `max_chars` chars sin partir multibyte.
fn truncate_chars(text: &str, max_chars: usize) -> &str {
    match text.char_indices().nth(max_chars) {
        Some((boundary, _)) => &text[..boundary],
        None => text,
    }
}

/// Marcas de muro de login en el HTML de un tema (minúsculas).
const UTNIANO_LOGIN_MARKERS: [&str; 6] = [
    "member.php?action=login",
    "name=\"password\"",
    "name='password'",
    "iniciar sesi",
    "logueate",
    "identificate",
];

/// Extrae la cita verificada del HTML puntual de un tema (puro, sin red).
///
/// Falla honesto si el HTML es un muro de login, si no hay título o si no hay
/// cuerpo de post (el fetch 2xx ya lo garantiza el transporte).
pub fn parse_utniano_topic_html(html: &str, url: &str) -> Result<UtnianoCitation, String> {
    let verified_url = normalize_utnianos_url(url)?;
    if html.len() > 2 * 1024 * 1024 {
        return Err("el tema es demasiado grande para citarlo de forma puntual".into());
    }
    let lower = html.to_lowercase();
    if UTNIANO_LOGIN_MARKERS
        .iter()
        .any(|marker| lower.contains(marker))
    {
        return Err(
            "el tema exige login (muro de autenticación): Grafito no usa credenciales, pedí otro aporte público"
                .into(),
        );
    }
    let titulo = extract_topic_title(html).ok_or_else(|| {
        "el tema no trae título legible; no se puede citar sin título".to_string()
    })?;
    let autor = extract_member_author(html);
    let fecha = extract_spanish_date(html);
    let fragmento = extract_first_post_body(html).ok_or_else(|| {
        "el tema no trae cuerpo de post legible; no se puede citar sin fragmento".to_string()
    })?;
    Ok(UtnianoCitation {
        titulo,
        autor,
        fecha,
        url: verified_url,
        fragmento,
        fecha_acceso: utnianos_today_utc(),
        verificado: true,
    })
}

/// `<title>` saneado y capado; `None` si vacío o ausente.
fn extract_topic_title(html: &str) -> Option<String> {
    let start = find_ascii_case_insensitive(html, "<title")?;
    let tag_len = scan_tag_len(&html[start..])?;
    let after = &html[start.checked_add(tag_len)?..];
    let end = find_ascii_case_insensitive(after, "</title>")?;
    let title =
        truncate_chars(&sanitize_html_text(&after[..end]), UTNIANO_TITLE_MAX_CHARS).to_string();
    if title.is_empty() {
        None
    } else {
        Some(title)
    }
}

/// Autor del primer post: texto del primer link a `member.php` (puro texto,
/// jamás se fetchea ese link bloqueado por robots).
fn extract_member_author(html: &str) -> Option<String> {
    for anchor in collect_raw_anchors(html) {
        if anchor.href.to_ascii_lowercase().contains("member.php") {
            let name = truncate_chars(&sanitize_html_text(&anchor.text), UTNIANO_AUTHOR_MAX_CHARS)
                .to_string();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }
    None
}

/// Primera fecha `dd-mm-aaaa` (o `aaaa-mm-dd`) con día/mes/año plausibles.
fn extract_spanish_date(html: &str) -> Option<String> {
    let bytes = html.as_bytes();
    let mut index = 0usize;
    while index + 10 <= bytes.len() {
        if let Some(date) =
            day_month_year_at(bytes, index).or_else(|| year_month_day_at(bytes, index))
        {
            return Some(date);
        }
        index = index.saturating_add(1);
    }
    None
}

/// `dd-mm-aaaa` (separador `-` o `/`) en `bytes[index..]`; normaliza a `-`.
fn day_month_year_at(bytes: &[u8], index: usize) -> Option<String> {
    let digit = |position: usize| bytes.get(index.checked_add(position)?).copied();
    let day = two_digits(digit(0)?, digit(1)?)?;
    let separator = digit(2)?;
    if separator != b'-' && separator != b'/' {
        return None;
    }
    let month = two_digits(digit(3)?, digit(4)?)?;
    if digit(5)? != separator {
        return None;
    }
    let year = four_digits(digit(6)?, digit(7)?, digit(8)?, digit(9)?)?;
    if (1..=31).contains(&day) && (1..=12).contains(&month) && (1900..=2100).contains(&year) {
        Some(format!("{day:02}-{month:02}-{year:04}"))
    } else {
        None
    }
}

/// `aaaa-mm-dd` (separador `-` o `/`) en `bytes[index..]`.
fn year_month_day_at(bytes: &[u8], index: usize) -> Option<String> {
    let digit = |position: usize| bytes.get(index.checked_add(position)?).copied();
    let year = four_digits(digit(0)?, digit(1)?, digit(2)?, digit(3)?)?;
    let separator = digit(4)?;
    if separator != b'-' && separator != b'/' {
        return None;
    }
    let month = two_digits(digit(5)?, digit(6)?)?;
    if digit(7)? != separator {
        return None;
    }
    let day = two_digits(digit(8)?, digit(9)?)?;
    if (1..=31).contains(&day) && (1..=12).contains(&month) && (1900..=2100).contains(&year) {
        Some(format!("{year:04}-{month:02}-{day:02}"))
    } else {
        None
    }
}

fn two_digits(high: u8, low: u8) -> Option<u32> {
    if high.is_ascii_digit() && low.is_ascii_digit() {
        Some(u32::from(high - b'0') * 10 + u32::from(low - b'0'))
    } else {
        None
    }
}

fn four_digits(a: u8, b: u8, c: u8, d: u8) -> Option<u32> {
    if [a, b, c, d].iter().all(|byte| byte.is_ascii_digit()) {
        Some(
            u32::from(a - b'0') * 1000
                + u32::from(b - b'0') * 100
                + u32::from(c - b'0') * 10
                + u32::from(d - b'0'),
        )
    } else {
        None
    }
}

/// Fragmento mínimo: primer `post_body` (o `<p>`) con texto sustancial,
/// saneado y capado a [`UTNIANO_FRAGMENT_MAX_CHARS`] con `…`.
fn extract_first_post_body(html: &str) -> Option<String> {
    for body in collect_class_texts(html, "post_body") {
        let clean = sanitize_html_text(&body);
        if clean.chars().count() >= 20 {
            return Some(cap_fragment(&clean));
        }
    }
    for paragraph in collect_tag_texts(html, "p") {
        let clean = sanitize_html_text(&paragraph);
        if clean.chars().count() >= 20 {
            return Some(cap_fragment(&clean));
        }
    }
    None
}

/// Capa el fragmento a [`UTNIANO_FRAGMENT_MAX_CHARS`] chars con `…` final.
fn cap_fragment(text: &str) -> String {
    if text.chars().count() <= UTNIANO_FRAGMENT_MAX_CHARS {
        return text.to_string();
    }
    let mut out: String = text
        .chars()
        .take(UTNIANO_FRAGMENT_MAX_CHARS.saturating_sub(1))
        .collect();
    out = out.trim_end().to_string();
    out.push('…');
    out
}

/// Anchor crudo (`href` + HTML interno) para extraer autores sin fetch.
struct RawAnchor {
    href: String,
    text: String,
}

/// Colecta hasta 512 anchors `<a href=...>…</a>` (tolerante a mayúsculas).
fn collect_raw_anchors(html: &str) -> Vec<RawAnchor> {
    let mut out = Vec::new();
    let mut rest = html;
    while let Some(start) = find_ascii_case_insensitive(rest, "<a") {
        let tag_open = &rest[start..];
        if !matches!(
            tag_open.as_bytes().get(2).copied().unwrap_or(b' '),
            b' ' | b'\t' | b'\n' | b'\r' | b'>' | b'/'
        ) {
            rest = &tag_open[1..];
            continue;
        }
        let Some(tag_len) = scan_tag_len(tag_open) else {
            break;
        };
        let tag = &tag_open[..tag_len];
        let after_tag = &tag_open[tag_len..];
        let Some(close) = find_ascii_case_insensitive(after_tag, "</a>") else {
            rest = after_tag;
            continue;
        };
        let inner = after_tag[..close].to_string();
        rest = &after_tag[close.saturating_add(4)..];
        let Some(href) = attribute_value(tag, "href") else {
            continue;
        };
        out.push(RawAnchor { href, text: inner });
        if out.len() >= 512 {
            break;
        }
    }
    out
}

/// Textos internos de los elementos cuya `class` contiene `class_name`.
fn collect_class_texts(html: &str, class_name: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find('<') {
        let tag_open = &rest[start..];
        let Some(tag_len) = scan_tag_len(tag_open) else {
            break;
        };
        let tag = &tag_open[..tag_len];
        let Some(class) = attribute_value(tag, "class") else {
            rest = &tag_open[tag_len..];
            continue;
        };
        rest = &tag_open[tag_len..];
        if class.split_whitespace().any(|token| token == class_name) {
            if let Some(end) = find_ascii_case_insensitive(rest, "</div>") {
                out.push(rest[..end].to_string());
                rest = &rest[end.saturating_add(6)..];
                if out.len() >= 32 {
                    break;
                }
            }
        }
    }
    out
}

/// Textos internos de los `<tag>…</tag>` (sólo `p`: párrafos de fallback).
fn collect_tag_texts(html: &str, tag: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = html;
    let close = format!("</{tag}>");
    while let Some(start) = find_ascii_case_insensitive(rest, &format!("<{tag}")) {
        let tag_open = &rest[start..];
        let Some(tag_len) = scan_tag_len(tag_open) else {
            break;
        };
        let after_tag = &tag_open[tag_len..];
        let Some(end) = find_ascii_case_insensitive(after_tag, &close) else {
            rest = after_tag;
            continue;
        };
        out.push(after_tag[..end].to_string());
        rest = &after_tag[end.saturating_add(close.len())..];
        if out.len() >= 32 {
            break;
        }
    }
    out
}

/// `find` ASCII case-insensitive (aguja ASCII) sin asignar.
fn find_ascii_case_insensitive(haystack: &str, needle: &str) -> Option<usize> {
    let haystack = haystack.as_bytes();
    let needle = needle.as_bytes();
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    (0..=haystack.len().saturating_sub(needle.len()))
        .find(|start| haystack[*start..*start + needle.len()].eq_ignore_ascii_case(needle))
}

/// Largo del tag que arranca en `input` (incluye el `>`), respetando comillas.
fn scan_tag_len(input: &str) -> Option<usize> {
    let mut quote: Option<u8> = None;
    for (index, byte) in input.bytes().enumerate() {
        match (quote, byte) {
            (None, b'"') | (None, b'\'') => quote = Some(byte),
            (Some(open), close) if open == close => quote = None,
            (None, b'>') => return Some(index.saturating_add(1)),
            _ => {}
        }
    }
    None
}

/// Valor de un atributo del tag, con comillas simples/dobles o sin comillas.
fn attribute_value(tag: &str, name: &str) -> Option<String> {
    let mut search_from = 0usize;
    while search_from < tag.len() {
        let relative = find_ascii_case_insensitive(&tag[search_from..], name)?;
        let position = search_from.saturating_add(relative);
        let before_ok = position == 0
            || tag.as_bytes()[position.saturating_sub(1)].is_ascii_whitespace()
            || tag.as_bytes()[position.saturating_sub(1)] == b'<';
        if before_ok {
            let after = tag[position.saturating_add(name.len())..].trim_start();
            if let Some(value_part) = after.strip_prefix('=') {
                let value_part = value_part.trim_start();
                let quote = *value_part.as_bytes().first()?;
                if quote == b'"' || quote == b'\'' {
                    let rest = &value_part[1..];
                    let close = rest.find(quote as char)?;
                    return Some(rest[..close].to_string());
                }
                let end = value_part
                    .find(|character: char| character.is_whitespace() || character == '>')
                    .unwrap_or(value_part.len());
                return Some(value_part[..end].to_string());
            }
        }
        search_from = position.saturating_add(name.len());
    }
    None
}

/// Fecha UTC de hoy (`aaaa-mm-dd`) para la fecha de acceso de la cita.
pub fn utnianos_today_utc() -> String {
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(elapsed) => {
            let days = i64::try_from(elapsed.as_secs() / 86_400).unwrap_or(i64::MAX);
            civil_date_from_epoch_days(days)
        }
        Err(_) => "desconocida".to_string(),
    }
}

/// Días desde la época UNIX → `aaaa-mm-dd` (algoritmo civil de Hinnant).
fn civil_date_from_epoch_days(days: i64) -> String {
    let shifted = days.saturating_add(719_468);
    let era = if shifted >= 0 {
        shifted / 146_097
    } else {
        shifted.saturating_sub(146_096) / 146_097
    };
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = if month_prime < 10 {
        month_prime + 3
    } else {
        month_prime - 9
    };
    let full_year = if month <= 2 { year + 1 } else { year };
    format!("{full_year:04}-{month:02}-{day:02}")
}

// ---------------------------------------------------------------------------
// Transporte (sólo con `assistant-net`; sin red: stubs honestos)
// ---------------------------------------------------------------------------

/// Busca resúmenes en UTNianos vía `site:` sobre el backend web existente.
///
/// El `search.php` interno del foro está bloqueado por `robots.txt` y jamás
/// se toca: la búsqueda sale por el backend de `crate::web` ya auditado.
#[cfg(feature = "assistant-net")]
pub fn buscar_utnianos(
    query: &str,
    materia: &str,
    tipo: &str,
) -> Result<Vec<UtnianoCandidate>, String> {
    if query.trim().is_empty() {
        return Err("buscar_resumen necesita una query no vacía".into());
    }
    let site_query = utnianos_site_query(query, materia, tipo)?;
    let results = crate::web::web_search(&site_query)?;
    Ok(select_utnianos_candidates(&results, query, materia, tipo))
}

/// Sin red: `Err` honesto (el build no puede buscar).
#[cfg(not(feature = "assistant-net"))]
pub fn buscar_utnianos(
    _query: &str,
    _materia: &str,
    _tipo: &str,
) -> Result<Vec<UtnianoCandidate>, String> {
    Err("buscar_resumen necesita el build con red (feature assistant-net apagada)".into())
}

/// Cita un tema con UN fetch puntual + parseo (la URL queda verificada).
#[cfg(feature = "assistant-net")]
pub fn citar_utniano_url(url: &str) -> Result<UtnianoCitation, String> {
    let validated = normalize_utnianos_url(url)?;
    let body = fetch_utniano_topic_body(&validated)?;
    parse_utniano_topic_html(&body, &validated)
}

/// Sin red: `Err` honesto (el build no puede verificar fuentes).
#[cfg(not(feature = "assistant-net"))]
pub fn citar_utniano_url(_url: &str) -> Result<UtnianoCitation, String> {
    Err("citar_fuente necesita el build con red (feature assistant-net apagada)".into())
}

/// GET puntual al tema con el cliente compartido (sin redirects, con timeout).
#[cfg(feature = "assistant-net")]
fn fetch_utniano_topic_body(url: &str) -> Result<String, String> {
    use std::io::Read;
    use std::time::Duration;

    let client = crate::shared_http_client()?;
    let response = client
        .get(url)
        .header("User-Agent", UTNIANO_USER_AGENT)
        .timeout(Duration::from_millis(UTNIANO_TIMEOUT_MS))
        .send()
        .map_err(|error| format!("no se pudo traer el tema de UTNianos: {error}"))?;
    let status = response.status().as_u16();
    if status == 401 || status == 403 {
        return Err(format!(
            "UTNianos devolvió HTTP {status}: el tema requiere login o está restringido; Grafito nunca usa credenciales, pedí otro aporte público"
        ));
    }
    if status == 404 {
        return Err(
            "UTNianos devolvió HTTP 404: esa URL no es un tema público; conseguí la URL con buscar_resumen"
                .into(),
        );
    }
    if (300..400).contains(&status) {
        return Err(format!(
            "UTNianos redirigió (HTTP {status}), probablemente al login: el tema no es público; pedí otro aporte"
        ));
    }
    if !(200..300).contains(&status) {
        return Err(format!(
            "UTNianos devolvió HTTP {status}: no se pudo verificar la fuente, probá más tarde"
        ));
    }
    let mut bytes = Vec::new();
    response
        .take(UTNIANO_RESPONSE_MAX_BYTES as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("no se pudo leer el tema de UTNianos: {error}"))?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::web::parse_ddg_html;

    const SEARCH_FIXTURE: &str = include_str!("../tests/fixtures/utnianos_search.html");
    const TOPIC_FIXTURE: &str = include_str!("../tests/fixtures/utniano_topic.html");
    const LOGIN_FIXTURE: &str = include_str!("../tests/fixtures/utniano_login.html");

    #[test]
    fn normalize_acepta_tema_y_rechaza_inventadas() {
        assert_eq!(
            normalize_utnianos_url(
                "https://www.utnianos.com.ar/tema-final-am1-febrero-2024-aporte"
            )
            .expect("tema válido"),
            "https://www.utnianos.com.ar/tema-final-am1-febrero-2024-aporte"
        );
        // Dominio ajeno con forma de tema: inventada, se rechaza sin red.
        assert!(normalize_utnianos_url("https://example.com/tema-falso-inventado-am1").is_err());
        // Host parecido pero no exacto.
        assert!(normalize_utnianos_url("https://foro.utnianos.com.ar/tema-algo").is_err());
        // Página del foro sin slug tema-: no es aporte citable.
        assert!(
            normalize_utnianos_url("https://www.utnianos.com.ar/foro-materias-generales").is_err()
        );
        // Esquema no http(s).
        assert!(normalize_utnianos_url("ftp://www.utnianos.com.ar/tema-algo").is_err());
        // Credenciales embebidas: nunca login.
        assert!(
            normalize_utnianos_url("https://usuario:clave@www.utnianos.com.ar/tema-algo").is_err()
        );
        assert!(normalize_utnianos_url("").is_err());
        assert!(normalize_utnianos_url("   ").is_err());
    }

    #[test]
    fn normalize_bloquea_rutas_de_robots_antes_del_fetch() {
        let attachment =
            normalize_utnianos_url("https://www.utnianos.com.ar/attachment.php?aid=12345")
                .expect_err("attachment.php debe fallar");
        assert!(
            attachment.contains("robots.txt"),
            "error honesto: {attachment}"
        );
        let search =
            normalize_utnianos_url("https://www.utnianos.com.ar/search.php?action=results&sid=abc")
                .expect_err("search.php debe fallar");
        assert!(search.contains("robots.txt"), "error honesto: {search}");
        let member =
            normalize_utnianos_url("https://www.utnianos.com.ar/member.php?action=profile&uid=7")
                .expect_err("member.php debe fallar");
        assert!(member.contains("robots.txt"), "error honesto: {member}");
    }

    #[test]
    fn select_filtra_y_rankea_desde_html_local() {
        let parsed = parse_ddg_html(SEARCH_FIXTURE);
        // El backend capa a 5: entran los 2 temas, el dominio ajeno, la
        // página sin slug y el attachment (search/member quedan fuera del
        // cap pero se cubren en `normalize_bloquea_rutas_de_robots`).
        assert_eq!(parsed.len(), 5);
        let candidatos = select_utnianos_candidates(
            &parsed,
            "final am1 febrero",
            "Análisis Matemático I",
            "final",
        );
        // Sólo los 2 temas pasan: caen el dominio ajeno, la página sin slug
        // tema- y el attachment bloqueado por robots.
        assert_eq!(candidatos.len(), 2);
        assert!(
            candidatos[0].url.contains("tema-final-am1"),
            "el final rankea primero: {}",
            candidatos[0].url
        );
        assert!(candidatos[1].url.contains("tema-resumen-algebra"));
        for candidato in &candidatos {
            assert!(!candidato.verificado);
            assert!(candidato.autor.is_none());
            assert!(!candidato.titulo.contains('<'));
        }
        assert!(candidatos[0].titulo.contains("Final AM1"));
    }

    #[test]
    fn parse_topic_extrae_cita_verificada() {
        let cita = parse_utniano_topic_html(
            TOPIC_FIXTURE,
            "https://www.utnianos.com.ar/tema-final-am1-febrero-2024-aporte",
        )
        .expect("fixture de tema válido");
        assert!(cita.titulo.contains("Final AM1"));
        assert_eq!(cita.autor.as_deref(), Some("ProfeUTN"));
        assert_eq!(cita.fecha.as_deref(), Some("12-02-2024"));
        assert!(cita.fragmento.contains("Taylor"));
        assert!(
            cita.fragmento.chars().count() <= UTNIANO_FRAGMENT_MAX_CHARS,
            "fragmento mínimo, no republicación"
        );
        assert_eq!(cita.fecha_acceso.len(), 10);
        assert!(cita.verificado);
        assert_eq!(
            cita.url,
            "https://www.utnianos.com.ar/tema-final-am1-febrero-2024-aporte"
        );
    }

    #[test]
    fn parse_login_falla_honesto_sin_credenciales() {
        let error = parse_utniano_topic_html(
            LOGIN_FIXTURE,
            "https://www.utnianos.com.ar/tema-restringido-xyz",
        )
        .expect_err("el muro de login debe fallar");
        assert!(error.contains("login"), "error honesto: {error}");
    }

    #[test]
    fn civil_date_convierte_vectores_conocidos() {
        assert_eq!(civil_date_from_epoch_days(0), "1970-01-01");
        assert_eq!(civil_date_from_epoch_days(19782), "2024-02-29");
        assert_eq!(civil_date_from_epoch_days(20736), "2026-10-10");
    }

    #[test]
    fn site_query_acota_al_foro_y_rechaza_vacia() {
        let query = utnianos_site_query("taylor con resto", "AM1", "final").expect("válida");
        assert!(
            query.starts_with("site:utnianos.com.ar"),
            "query inesperada: {query}"
        );
        assert!(query.contains("taylor"));
        assert!(utnianos_site_query("   ", "AM1", "final").is_err());
        assert!(utnianos_site_query("hola\0", "", "").is_err());
        let larga = "palabra ".repeat(100);
        let capada = utnianos_site_query(&larga, "", "").expect("se recorta");
        assert_eq!(capada.chars().count(), UTNIANO_QUERY_MAX_CHARS);
    }
}
