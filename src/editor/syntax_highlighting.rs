use crate::parser;
use egui::text::LayoutJob;
use egui::FontId;

/// Memoized Code highlighting
pub fn highlight(ctx: &egui::Context, theme: &CodeTheme, code: &str) -> LayoutJob {
    impl egui::util::cache::ComputerMut<(&CodeTheme, &str), LayoutJob> for Highlighter {
        fn compute(&mut self, (theme, code): (&CodeTheme, &str)) -> LayoutJob {
            self.highlight(theme, code)
        }
    }

    type HighlightCache<'a> = egui::util::cache::FrameCache<LayoutJob, Highlighter>;

    let mut memory = ctx.memory();
    let highlight_cache = memory.caches.cache::<HighlightCache<'_>>();
    highlight_cache.get((theme, code))
}

// ----------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(enum_map::Enum)]
enum TokenType {
    Comment,
    Normal,
    Linebreak,
    Keyword,
    Boolean,
    StringLiteral,
    Function,
    Command,
    GenMod,
    Whitespace,
}

#[derive(Clone, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct CodeTheme {
    formats: enum_map::EnumMap<TokenType, egui::TextFormat>,
}

impl Default for CodeTheme {
    fn default() -> Self {
        Self::dark(15.0)
    }
}

impl CodeTheme {
    pub fn dark(font_size: f32) -> Self {
        let text_style = FontId::monospace(font_size);
        use egui::{Color32, TextFormat};
        Self {
            formats: enum_map::enum_map![
                        TokenType::Comment => TextFormat::simple(text_style.clone(), Color32::from_gray(120)),
                TokenType::Normal => TextFormat::simple(text_style.clone(), Color32::from_gray(200)),
                TokenType::Boolean => TextFormat::simple(text_style.clone(), Color32::from_rgb(0, 200, 100)),
                        TokenType::Keyword => TextFormat::simple(text_style.clone(), Color32::from_rgb(200, 20, 200)),
                        TokenType::StringLiteral => TextFormat::simple(text_style.clone(), egui::Color32::from_rgb(200, 200, 10)),
                    TokenType::Function => TextFormat::simple(text_style.clone(), Color32::from_rgb(220, 20, 100)),
            TokenType::Command => TextFormat::simple(text_style.clone(), Color32::from_rgb(100, 220, 110)),
            TokenType::GenMod => TextFormat::simple(text_style.clone(), Color32::from_rgb(190, 190, 140)),
                        TokenType::Whitespace => TextFormat::simple(text_style.clone(), Color32::TRANSPARENT),
                TokenType::Linebreak => TextFormat::simple(text_style.clone(), Color32::TRANSPARENT),
                    ],
        }
    }
}

#[derive(Default)]
struct Highlighter {}

impl Highlighter {
    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
    fn highlight(&self, theme: &CodeTheme, mut text: &str) -> LayoutJob {
        let mut job = LayoutJob::default();

        while !text.is_empty() {
            if text.starts_with(";;") {
                let end = text.find('\n').unwrap_or(text.len());
                job.append(&text[..end], 0.0, theme.formats[TokenType::Comment].clone());
                text = &text[end..];
            } else if text.starts_with('"') {
                let end = text[1..]
                    .find('"')
                    .map(|i| i + 2)
                    .or_else(|| text.find('\n'))
                    .unwrap_or(text.len());
                job.append(
                    &text[..end],
                    0.0,
                    theme.formats[TokenType::StringLiteral].clone(),
                );
                text = &text[end..];
            } else if text.starts_with(':') {
                let end = text[1..]
                    .find(|c: char| !c.is_ascii_alphanumeric())
                    .map_or_else(|| text.len(), |i| i + 1);
                job.append(&text[..end], 0.0, theme.formats[TokenType::Keyword].clone());
                text = &text[end..];
            } else if text.starts_with('#') {
                // avoid crash by checking text length
                let end = if text.len() > 1 { 2 } else { 1 };
                job.append(&text[..end], 0.0, theme.formats[TokenType::Boolean].clone());
                text = &text[end..];
            } else if text.starts_with(|c: char| c.is_ascii_alphanumeric()) {
                let end = text[1..]
                    .find(|c: char| !parser::valid_function_name_char(c))
                    .map_or_else(|| text.len(), |i| i + 1);
                let word = &text[..end];
                let tt = if is_function(word) {
                    TokenType::Function
                } else if is_command(word) {
                    TokenType::Command
                } else if is_genmod(word) {
                    TokenType::GenMod
                } else {
                    TokenType::Normal
                };
                job.append(word, 0.0, theme.formats[tt].clone());
                text = &text[end..];
            } else if text.starts_with(|c: char| c.is_ascii_whitespace()) {
                let end = text[1..]
                    .find(|c: char| !c.is_ascii_whitespace())
                    .map_or_else(|| text.len(), |i| i + 1);
                job.append(
                    &text[..end],
                    0.0,
                    theme.formats[TokenType::Whitespace].clone(),
                );
                text = &text[end..];
            } else {
                let mut it = text.char_indices();
                it.next();
                let end = it.next().map_or(text.len(), |(idx, _chr)| idx);
                job.append(&text[..end], 0.0, theme.formats[TokenType::Normal].clone());
                text = &text[end..];
            }
        }

        job
    }
}
fn is_function(word: &str) -> bool {
    matches!(
        word,
        "apple"
            | "friendship"
            | "learn"
            | "pear"
            | "nuc"
            | "fully"
            | "flower"
            | "sx"
            | "cyc"
            | "xspread"
            | "xdup"
            | "life"
            | "lin"
            | "loop"
            | "ls"
            | "list"
            | "every"
            | "infer"
            | "once"
            | "cmp"
            | "chop"
            | "inh"
            | "exh"
            | "inexh"
            | "stages"
    )
}

fn is_command(word: &str) -> bool {
    matches!(
        word,
        "apple"
            | "export-dot"
            | "step-part"
            | "tmod"
            | "latency"
            | "global-resources"
            | "delay"
            | "reverb"
            | "default-duration"
            | "bpm"
            | "defpart"
            | "clear"
            | "rec"
            | "stop-rec"
    )
}

fn is_genmod(word: &str) -> bool {
    matches!(
        word,
        "haste"
            | "shrink"
            | "grow"
            | "keep"
            | "rep"
            | "rnd"
            | "relax"
            | "blur"
            | "sharpen"
            | "skip"
            | "rewind"
            | "reverse"
            | "solidify"
    )
}
