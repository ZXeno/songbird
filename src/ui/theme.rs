use iced::widget::text;
use iced::{Color, Font, Theme};

// Text Styling
pub(super) const MONO: Font = Font::MONOSPACE;
pub(super) const UI_TEXT_SIZE: f32 = 12.0;
pub(super) const UI_TEXT_SIZE_SM: f32 = 10.0;
pub(super) const HEADER_TEXT_SIZE: f32 = 16.0;
pub(super) const TEXT_SCALE_ALPHA: f32 = 0.8;
pub(super) const HOVERED_SCALE_ALPHA: f32 = 0.15;

// General Colorization
pub(super) const SUCCESS: Color = Color::from_rgb(0.30, 0.78, 0.47);
pub(super) const WARNING: Color = Color::from_rgb(0.90, 0.72, 0.25);
pub(super) const DANGER: Color = Color::from_rgb(0.90, 0.36, 0.36);
pub(super) const MUTED: Color = Color::from_rgb(0.55, 0.58, 0.65);

// UI element standardization formatting
pub(super) const RADIUS: f32 = 2.0;

pub(super) fn tinted_text<'a>(
    content: impl text::IntoFragment<'a>,
    color: Color,
) -> text::Text<'a, Theme, iced::Renderer> {
    text(content).style(move |_| text::Style { color: Some(color) })
}

pub(super) fn truncate_label(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        value.to_owned()
    } else {
        let cut: String = value.chars().take(max_chars).collect();
        format!("{cut}...")
    }
}
