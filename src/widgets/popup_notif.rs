use crate::widgets::PopupNotif;

use ratatui::{
    prelude::{Alignment, Buffer, Rect, Stylize},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
};
impl Default for PopupNotif {
    fn default() -> Self {
        PopupNotif {
            title: "".into(),
            message: vec![("This is a notification popup!".into(), Color::White)],
            border_color: Color::Green,
            duration_ticks: Some(30),
            index: 1,
        }
    }
}

impl Widget for PopupNotif {
    fn render(self, area: Rect, buf: &mut Buffer) -> () {
        if let Some(ticks) = self.duration_ticks {
            let ratio = (4, 5);
            if self.index > ratio.1.into() {
                return;
            }
            let help_block = Block::new()
                .title(self.title)
                .title_style(Style::new().white().bold())
                .borders(Borders::ALL)
                .border_style(Style::new().fg(self.border_color));

            let mut popup_area = Rect {
                x: area.width,
                y: area.height,
                width: area.width / ratio.0,
                height: area.height / ratio.1,
            };
            popup_area.x -= area.width / ratio.0;
            popup_area.y -= (area.height / ratio.1) * self.index as u16;

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
                .wrap(Wrap { trim: false })
                .render(popup_area, buf);
            let mut hint_popup_area = popup_area;
            hint_popup_area.y += popup_area.height - 1;
            hint_popup_area.x += 1;

            Paragraph::new(ticks.to_string())
                .alignment(Alignment::Left)
                .render(hint_popup_area, buf);
        }
    }
}
