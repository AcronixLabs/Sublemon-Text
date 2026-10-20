use std::path::{Path, PathBuf};

use eframe::egui::{self, Color32, Context, FontFamily, FontId, Id, TextFormat};

#[derive(Clone)]
struct CachedLayout {
    revision: u64,
    path: Option<PathBuf>,
    width: u32,
    light: bool,
    galley: std::sync::Arc<egui::Galley>,
}

const LARGE_FILE_PLAIN_TEXT_LIMIT: usize = 512 * 1024;

pub fn layout_cached(
    context: &Context,
    text: &str,
    revision: u64,
    path: Option<&Path>,
    wrap_width: f32,
    light: bool,
) -> std::sync::Arc<egui::Galley> {
    let id = Id::new("sublemon_syntax_galley");
    let path = path.map(Path::to_path_buf);
    let width = wrap_width.to_bits();
    if let Some(cached) = context.data(|data| data.get_temp::<CachedLayout>(id))
        && cached.revision == revision
        && cached.path == path
        && cached.width == width
        && cached.light == light
    {
        return cached.galley;
    }

    let foreground = foreground(light);
    let job = if text.len() > LARGE_FILE_PLAIN_TEXT_LIMIT {
        // Syntax tokenization and per-token layout are expensive for huge buffers.
        // Keep a single style run so editing remains responsive.
        egui::text::LayoutJob::simple(
            text.to_owned(),
            FontId::new(14.0, FontFamily::Monospace),
            foreground,
            wrap_width,
        )
    } else {
        highlighted_layout(text, path.as_deref(), wrap_width, light)
    };
    let galley = context.fonts_mut(|fonts| fonts.layout_job(job));
    context.data_mut(|data| {
        data.insert_temp(
            id,
            CachedLayout {
                revision,
                path,
                width,
                light,
                galley: galley.clone(),
            },
        )
    });
    galley
}

fn foreground(light: bool) -> Color32 {
    if light {
        Color32::from_rgb(45, 48, 55)
    } else {
        Color32::from_rgb(204, 205, 207)
    }
}

fn highlighted_layout(
    text: &str,
    path: Option<&Path>,
    width: f32,
    light: bool,
) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    job.wrap.max_width = width;
    job.keep_trailing_whitespace = true;
    let extension = path
        .and_then(Path::extension)
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let base = foreground(light);
    let mut lines = text.split('\n').peekable();
    while let Some(line) = lines.next() {
        append_line(&mut job, line, &extension, light, base);
        if lines.peek().is_some() {
            job.append(
                "\n",
                0.0,
                TextFormat {
                    font_id: FontId::new(14.0, FontFamily::Monospace),
                    color: base,
                    ..Default::default()
                },
            );
        }
    }
    job
}

fn append_line(
    job: &mut egui::text::LayoutJob,
    line: &str,
    extension: &str,
    light: bool,
    base: Color32,
) {
    let comment = if light {
        Color32::from_rgb(99, 112, 132)
    } else {
        Color32::from_rgb(125, 147, 173)
    };
    let string = if light {
        Color32::from_rgb(151, 73, 39)
    } else {
        Color32::from_rgb(214, 157, 133)
    };
    let keyword = if light {
        Color32::from_rgb(160, 55, 26)
    } else {
        Color32::from_rgb(230, 153, 116)
    };
    let number = if light {
        Color32::from_rgb(137, 88, 24)
    } else {
        Color32::from_rgb(209, 163, 111)
    };
    let entity = if light {
        Color32::from_rgb(39, 83, 139)
    } else {
        Color32::from_rgb(178, 202, 235)
    };
    let punctuation = if light {
        Color32::from_rgb(79, 88, 102)
    } else {
        Color32::from_rgb(176, 187, 204)
    };
    let line_comment = if matches!(
        extension,
        "py" | "pyi" | "sh" | "bash" | "yaml" | "yml" | "toml" | "rb" | "pl" | "r"
    ) {
        '#'
    } else {
        '\0'
    };

    let mut chars = line.char_indices().peekable();
    while let Some((start, ch)) = chars.next() {
        if (line_comment != '\0' && ch == line_comment)
            || (ch == '/' && chars.peek().is_some_and(|(_, next)| *next == '/'))
        {
            append_span(job, &line[start..], comment);
            return;
        }
        if ch == '/' && chars.peek().is_some_and(|(_, next)| *next == '*') {
            append_span(job, &line[start..], comment);
            return;
        }
        if matches!(ch, '"' | '\'' | '`') {
            let quote = ch;
            let mut end = line.len();
            let mut escaped = false;
            for (offset, next) in chars.by_ref() {
                if !escaped && next == quote {
                    end = offset + next.len_utf8();
                    break;
                }
                escaped = !escaped && next == '\\';
                if next != '\\' {
                    escaped = false;
                }
            }
            append_span(job, &line[start..end], string);
            // The iterator has advanced through the string; continue at the next token.
            continue;
        }
        if ch.is_ascii_alphabetic() || ch == '_' {
            let mut end = start + ch.len_utf8();
            while let Some((offset, next)) = chars.peek().copied() {
                if next.is_ascii_alphanumeric() || next == '_' {
                    chars.next();
                    end = offset + next.len_utf8();
                } else {
                    break;
                }
            }
            let token = &line[start..end];
            let color = if is_keyword(token, extension) {
                keyword
            } else if chars.peek().is_some_and(|(_, next)| *next == '(') {
                entity
            } else {
                base
            };
            append_span(job, token, color);
            continue;
        }
        if ch.is_ascii_digit() {
            let mut end = start + ch.len_utf8();
            while let Some((offset, next)) = chars.peek().copied() {
                if next.is_ascii_alphanumeric() || matches!(next, '.' | '_') {
                    chars.next();
                    end = offset + next.len_utf8();
                } else {
                    break;
                }
            }
            append_span(job, &line[start..end], number);
            continue;
        }
        append_span(job, &line[start..start + ch.len_utf8()], punctuation);
    }
}

fn append_span(job: &mut egui::text::LayoutJob, text: &str, color: Color32) {
    if text.is_empty() {
        return;
    }
    job.append(
        text,
        0.0,
        TextFormat {
            font_id: FontId::new(14.0, FontFamily::Monospace),
            color,
            ..Default::default()
        },
    );
}

fn is_keyword(token: &str, extension: &str) -> bool {
    const COMMON: &[&str] = &[
        "as",
        "async",
        "await",
        "break",
        "case",
        "catch",
        "class",
        "const",
        "continue",
        "default",
        "do",
        "else",
        "enum",
        "export",
        "extends",
        "false",
        "final",
        "for",
        "from",
        "function",
        "if",
        "import",
        "in",
        "interface",
        "let",
        "match",
        "module",
        "mut",
        "new",
        "null",
        "of",
        "package",
        "pub",
        "return",
        "self",
        "static",
        "struct",
        "switch",
        "this",
        "throw",
        "trait",
        "true",
        "try",
        "type",
        "use",
        "var",
        "void",
        "while",
        "yield",
    ];
    const PYTHON: &[&str] = &[
        "and", "def", "elif", "except", "exec", "from", "global", "is", "lambda", "nonlocal",
        "not", "or", "pass", "raise", "with",
    ];
    const RUST: &[&str] = &["crate", "dyn", "impl", "loop", "ref", "where"];
    COMMON.contains(&token)
        || (extension == "py" && PYTHON.contains(&token))
        || (extension == "rs" && RUST.contains(&token))
}
