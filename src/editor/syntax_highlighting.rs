use egui::text::LayoutJob;

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
    Whitespace,
}

#[derive(Clone, Copy, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct CodeTheme {
    formats: enum_map::EnumMap<TokenType, egui::TextFormat>,
}

impl Default for CodeTheme {
    fn default() -> Self {
        Self::dark()
    }
}

impl CodeTheme {
    pub fn from_memory(ctx: &egui::Context) -> Self {
        ctx.memory()
            .data
            .get_persisted(egui::Id::new("dark"))
            .unwrap_or_else(CodeTheme::dark)
    }

    pub fn store_in_memory(&self, ctx: &egui::Context) {
        ctx.memory()
            .data
            .insert_persisted(egui::Id::new("dark"), *self);
    }
}

impl CodeTheme {
    pub fn dark() -> Self {
        let text_style = egui::TextStyle::Monospace;
        use egui::{Color32, TextFormat};
        Self {
            formats: enum_map::enum_map![
                TokenType::Comment => TextFormat::simple(text_style, Color32::from_gray(120)),
		TokenType::Normal => TextFormat::simple(text_style, Color32::from_gray(200)),
		TokenType::Boolean => TextFormat::simple(text_style, Color32::from_rgb(0, 200, 100)),
                TokenType::Keyword => TextFormat::simple(text_style, Color32::from_rgb(200, 20, 200)),
                TokenType::StringLiteral => TextFormat::simple(text_style, egui::Color32::from_rgb(200, 200, 10)),
                TokenType::Function => TextFormat::simple(text_style, Color32::from_rgb(220, 20, 100)),
                TokenType::Whitespace => TextFormat::simple(text_style, Color32::TRANSPARENT),
		TokenType::Linebreak => TextFormat::simple(text_style, Color32::TRANSPARENT),
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
                let end = text.find('\n').unwrap_or_else(|| text.len());
                job.append(&text[..end], 0.0, theme.formats[TokenType::Comment]);
                text = &text[end..];
            } else if text.starts_with('"') {
                let end = text[1..]
                    .find('"')
                    .map(|i| i + 2)
                    .or_else(|| text.find('\n'))
                    .unwrap_or_else(|| text.len());
                job.append(&text[..end], 0.0, theme.formats[TokenType::StringLiteral]);
                text = &text[end..];
            } else if text.starts_with(':') {
                let end = text[1..]
                    .find(|c: char| !c.is_ascii_alphanumeric())
                    .map_or_else(|| text.len(), |i| i + 1);
                job.append(&text[..end], 0.0, theme.formats[TokenType::Keyword]);
                text = &text[end..];
            } else if text.starts_with('#') {
                let end = 2;
                job.append(&text[..end], 0.0, theme.formats[TokenType::Boolean]);
                text = &text[end..];
            } else if text.starts_with(|c: char| c.is_ascii_alphanumeric()) {
                let end = text[1..]
                    .find(|c: char| !c.is_ascii_alphanumeric())
                    .map_or_else(|| text.len(), |i| i + 1);
                let word = &text[..end];
                let tt = if is_function(word) {
                    TokenType::Function
                } else {
                    TokenType::Normal
                };
                job.append(word, 0.0, theme.formats[tt]);
                text = &text[end..];
            } else if text.starts_with(|c: char| c.is_ascii_whitespace()) {
                let end = text[1..]
                    .find(|c: char| !c.is_ascii_whitespace())
                    .map_or_else(|| text.len(), |i| i + 1);
                job.append(&text[..end], 0.0, theme.formats[TokenType::Whitespace]);
                text = &text[end..];
            } else {
                let mut it = text.char_indices();
                it.next();
                let end = it.next().map_or(text.len(), |(idx, _chr)| idx);
                job.append(&text[..end], 0.0, theme.formats[TokenType::Normal]);
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
            | "export-dot"
            | "step-part"
            | "friendship"
            | "tmod"
            | "latency"
            | "global-resources"
            | "learn"
            | "delay"
            | "reverb"
            | "pear"
            | "nuc"
            | "fully"
            | "flower"
            | "sx"
            | "cyc"
            | "xspread"
            | "xdup"
            | "life"
            | "ls"
            | "every"
            | "defpart"
            | "infer"
            | "clear"
            | "once"
            | "cub"
            | "cmp"
            | "chop"
            | "rnd"
            | "rep"
            | "inh"
            | "exh"
            | "inexh"
            | "stages"
    )
}
