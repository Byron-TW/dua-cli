use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Paragraph, Text, Widget},
};

pub struct Header;

impl Header {
    pub fn render(&self, bg_color: Color, area: Rect, buf: &mut Buffer) {
        let text_color = Color::Black;
        let standard = Style {
            fg: text_color,
            bg: bg_color,
            ..Default::default()
        };
        let modified = |text: &'static str, modifier| {
            Text::Styled(
                text.into(),
                Style {
                    modifier,
                    ..standard
                },
            )
        };
        let bold = |text: &'static str| modified(text, Modifier::BOLD);
        let italic = |text: &'static str| modified(text, Modifier::UNDERLINED);
        let text = |text: &'static str| Text::Styled(text.into(), standard);

        let lines = [
            bold(" D"),
            text("isk "),
            bold("U"),
            text("sage "),
            bold("A"),
            text("nalyzer v"),
            text(env!("CARGO_PKG_VERSION")),
            text("    "),
            italic("(press "),
            modified("?", Modifier::BOLD | Modifier::UNDERLINED),
            italic(" for help)"),
        ];
        Paragraph::new(lines.iter())
            .style(Style {
                fg: text_color,
                bg: bg_color,
                ..Default::default()
            })
            .draw(area, buf);
    }
}
