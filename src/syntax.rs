use std::{path::Path, str::FromStr, sync::OnceLock};

use eframe::egui::{self, Color32, FontFamily, FontId, TextFormat};
use syntect::{
    easy::HighlightLines,
    highlighting::{Color, ScopeSelectors, StyleModifier, Theme, ThemeItem, ThemeSettings},
    parsing::SyntaxSet,
};

static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
static DARK_THEME: OnceLock<Theme> = OnceLock::new();
static LIGHT_THEME: OnceLock<Theme> = OnceLock::new();

fn syntax_set() -> &'static SyntaxSet {
    SYNTAX_SET.get_or_init(|| {
        syntect::dumps::from_binary(include_bytes!("../assets/sublemon-syntaxes.packdump"))
    })
}

fn theme(light: bool) -> &'static Theme {
    let cell = if light { &LIGHT_THEME } else { &DARK_THEME };
    cell.get_or_init(|| {
        let (foreground, background, palette) = if light {
            (
                Color {
                    r: 45,
                    g: 48,
                    b: 55,
                    a: 255,
                },
                Color {
                    r: 250,
                    g: 250,
                    b: 252,
                    a: 255,
                },
                [
                    ("comment", [99, 112, 132]),
                    ("string", [151, 73, 39]),
                    ("constant.numeric", [137, 88, 24]),
                    ("constant.language", [137, 88, 24]),
                    ("keyword", [160, 55, 26]),
                    ("storage", [160, 55, 26]),
                    ("entity.name", [39, 83, 139]),
                    ("support.function", [24, 99, 119]),
                    ("variable", [54, 63, 78]),
                    ("punctuation", [79, 88, 102]),
                    ("invalid", [180, 30, 35]),
                ],
            )
        } else {
            (
                Color {
                    r: 204,
                    g: 205,
                    b: 207,
                    a: 255,
                },
                Color {
                    r: 49,
                    g: 54,
                    b: 64,
                    a: 255,
                },
                [
                    ("comment", [125, 147, 173]),
                    ("string", [214, 157, 133]),
                    ("constant.numeric", [209, 163, 111]),
                    ("constant.language", [209, 163, 111]),
                    ("keyword", [230, 153, 116]),
                    ("storage", [230, 153, 116]),
                    ("entity.name", [178, 202, 235]),
                    ("support.function", [160, 194, 219]),
                    ("variable", [194, 205, 222]),
                    ("punctuation", [176, 187, 204]),
                    ("invalid", [255, 126, 126]),
                ],
            )
        };
        let scopes = palette
            .into_iter()
            .filter_map(|(scope, [r, g, b])| {
                Some(ThemeItem {
                    scope: ScopeSelectors::from_str(scope).ok()?,
                    style: StyleModifier {
                        foreground: Some(Color { r, g, b, a: 255 }),
                        ..Default::default()
                    },
                })
            })
            .collect();
        Theme {
            name: Some("Sublemon Dark".to_owned()),
            author: Some("Sublemon".to_owned()),
            settings: ThemeSettings {
                foreground: Some(foreground),
                background: Some(background),
                ..Default::default()
            },
            scopes,
        }
    })
}

pub fn layout(
    text: &str,
    path: Option<&Path>,
    wrap_width: f32,
    light: bool,
) -> Option<egui::text::LayoutJob> {
    let syntaxes = syntax_set();
    let syntax = path
        .and_then(|path| syntaxes.find_syntax_for_file(path).ok().flatten())
        .unwrap_or_else(|| syntaxes.find_syntax_plain_text());
    let theme = theme(light);
    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut job = egui::text::LayoutJob::default();
    job.wrap.max_width = wrap_width;
    job.keep_trailing_whitespace = true;
    let mut lines = text.split('\n').peekable();
    while let Some(line) = lines.next() {
        let highlighted = highlighter.highlight_line(line, syntaxes).ok()?;
        for (style, span) in highlighted {
            let font_id = FontId::new(14.0, FontFamily::Monospace);
            let format = TextFormat {
                font_id,
                color: Color32::from_rgba_unmultiplied(
                    style.foreground.r,
                    style.foreground.g,
                    style.foreground.b,
                    style.foreground.a,
                ),
                italics: style
                    .font_style
                    .contains(syntect::highlighting::FontStyle::ITALIC),
                ..Default::default()
            };
            job.append(span, 0.0, format);
        }
        if lines.peek().is_some() {
            job.append(
                "\n",
                0.0,
                TextFormat {
                    font_id: FontId::new(14.0, FontFamily::Monospace),
                    color: if light {
                        Color32::from_rgb(45, 48, 55)
                    } else {
                        Color32::from_rgb(204, 205, 207)
                    },
                    ..Default::default()
                },
            );
        }
    }
    Some(job)
}
