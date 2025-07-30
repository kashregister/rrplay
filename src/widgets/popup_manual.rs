use crate::widgets::PopupManual;
use ratatui::{
    prelude::{Alignment, Buffer, Rect, Stylize},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

impl Default for PopupManual {
    fn default() -> Self {
        PopupManual {
            title: "Popup".into(),
            message: vec![("This is a manual popup".into(), Color::White)],
            bottom_hint: "Press <Esc> to close the window".into(),
            border_color: Color::Red,
        }
    }
}

impl Widget for PopupManual {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let help_block = Block::new()
            .title(self.title)
            .title_style(Style::new().white().bold())
            .borders(Borders::ALL)
            .border_style(Style::new().fg(self.border_color));

        let popup_area = Rect {
            x: area.width / 4,
            y: area.height / 3,
            width: area.width / 2,
            height: area.height / 3,
        };

        if popup_area.height < 1 {
            return;
        }
        let text_for_render: Vec<Line<'_>> = self
            .message
            .into_iter()
            .map(|s| Span::styled(s.0, Style::default().fg(s.1)).into())
            .collect();

        Clear.render(popup_area, buf);
        Paragraph::new(text_for_render)
            .style(Style::new())
            .block(help_block)
            .render(popup_area, buf);
        let mut hint_popup_area = popup_area;
        hint_popup_area.y += popup_area.height - 1;
        hint_popup_area.x += 1;

        Paragraph::new(self.bottom_hint)
            .alignment(Alignment::Left)
            .render(hint_popup_area, buf);
    }
}
