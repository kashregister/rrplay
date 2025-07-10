use crate::app::App;
use crate::app::Mode;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    prelude::*,
    style::{Color, Stylize},
    widgets::{Block, BorderType, Gauge, Paragraph, Widget},
};
use std::path::Path;

fn format_seconds(seconds: u64) -> (u64, u64) {
    let mut secs = seconds.clone();
    let min = secs / 60 as u64;
    secs -= min * 60;
    (min, secs)
}

fn generate_label(seconds: u64) -> String {
    let playing = format_seconds(seconds);
    if playing.0 == 0 {
        format!("{}s", playing.1)
    } else {
        format!("{}m {}s", playing.0, playing.1)
    }
}

impl Widget for &App {
    /// Renders the user interface widgets.
    ///
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui/ratatui/tree/master/examples
    //

    fn render(self, area: Rect, buf: &mut Buffer) {
        // Color for the focused window
        // const focus_color: Color = Color::Blue;

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(99), Constraint::Length(3)])
            .split(area);

        let inner_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(18),
                Constraint::Percentage(72),
                Constraint::Percentage(10),
            ])
            .split(layout[1]);

        let block = if self.mode == Mode::Select {
            Block::bordered()
                .title("Search results")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Plain)
                .border_style(Style::new().yellow())
        } else if self.mode == Mode::Search {
            Block::bordered()
                .title("Search results")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Plain)
        } else if self.mode == Mode::Sitback {
            Block::bordered()
                .title("Queue")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Plain)
        } else {
            Block::bordered()
                .title("Queue")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Plain)
        };

        let mut joined_results: Vec<Line<'_>> = Vec::new();
        let binding = self.search_results.clone();
        let binding_queue = self.queue.clone();
        if self.mode == Mode::Select {
            for (i, song) in binding.iter().enumerate() {
                if let Some(filename) = Path::new(song.clone().as_str()).file_name() {
                    if let Ok(fname) = filename.to_owned().into_string() {
                        if i == self.search_results.len() - 1 - self.select_index {
                            joined_results.push(
                                Span::styled(fname, Style::default().add_modifier(Modifier::BOLD))
                                    .into(),
                            );
                        } else {
                            joined_results.push(
                                Span::styled(fname, Style::default().add_modifier(Modifier::DIM))
                                    .into(),
                            );
                        }
                    }
                }
            }
        } else if self.mode == Mode::Search {
            for song in binding {
                if let Some(filename) = Path::new(song.clone().as_str()).file_name() {
                    if let Ok(fname) = filename.to_owned().into_string() {
                        joined_results.push(
                            Span::styled(fname, Style::default().add_modifier(Modifier::DIM))
                                .into(),
                        );
                    }
                }
            }
        } else {
            for (i, song) in binding_queue.iter().enumerate() {
                if i == 0 {
                    joined_results.push(
                        Span::styled(
                            song.0.clone(),
                            Style::default().add_modifier(Modifier::BOLD),
                        )
                        .into(),
                    );
                } else {
                    joined_results.push(
                        Span::styled(song.0.clone(), Style::default().add_modifier(Modifier::DIM))
                            .into(),
                    );
                }
            }
        }

        let paragraph = Paragraph::new(joined_results)
            .block(block)
            .fg(Color::White)
            .bg(Color::Black)
            .centered();
        paragraph.render(layout[0], buf);

        let mode_block = if self.mode == Mode::Search {
            Block::bordered()
                .title("Query")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Plain)
                .border_style(Style::new().yellow())
        } else if self.mode == Mode::Sitback {
            Block::bordered()
                // .title("Sitback")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Plain)
        } else if self.mode == Mode::Select {
            Block::bordered()
                .title("Query")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Plain)
        } else {
            Block::bordered()
                .title("Undefined")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Plain)
        };

        let status_volume_block = Block::bordered()
            .title("Volume")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Plain);

        let status_playing_block = Block::bordered()
            .title("Status")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Plain);
        // .border_style(Style::new().red());

        if self.mode == Mode::Sitback {
            if self.queue.len() > 0 && self.sink.is_paused() == false {
                let label = Span::styled(
                    generate_label(self.sink.get_pos().as_secs()),
                    Style::new().italic().bold().fg(Color::DarkGray),
                );
                Gauge::default()
                    .block(mode_block)
                    .gauge_style(Style::new().green())
                    .ratio(self.sink.get_pos().as_secs_f64() / self.queue[0].1.as_secs_f64())
                    .label(label)
                    .render(inner_layout[1], buf);
            } else if self.queue.len() > 0 && self.sink.is_paused() {
                let label = Span::styled(
                    generate_label(self.sink.get_pos().as_secs()),
                    Style::new().italic().bold().fg(Color::DarkGray),
                );
                Gauge::default()
                    .block(mode_block)
                    .gauge_style(Style::new().red())
                    .ratio(self.sink.get_pos().as_secs_f64() / self.queue[0].1.as_secs_f64())
                    .label(label)
                    .render(inner_layout[1], buf);
            } else {
                Paragraph::new("Empty queue")
                    .block(mode_block)
                    .fg(Color::White)
                    .bg(Color::Black)
                    .centered()
                    .render(inner_layout[1], buf);
            }
        } else if self.mode == Mode::Search {
            Paragraph::new(self.query.clone())
                .block(mode_block)
                .fg(Color::White)
                .bg(Color::Black)
                .centered()
                .render(inner_layout[1], buf);
        } else if self.mode == Mode::Select {
            Paragraph::new(self.query.clone())
                .block(mode_block)
                .fg(Color::White)
                .bg(Color::Black)
                .centered()
                .render(inner_layout[1], buf);
        };
        let volume_paragraph = Paragraph::new(((self.sink.volume() * 100.0) as usize).to_string())
            .block(status_volume_block)
            .fg(Color::White)
            .bg(Color::Black)
            .centered();
        volume_paragraph.render(inner_layout[2], buf);

        if self.sink.is_paused() {
            Paragraph::new("Paused")
                .block(status_playing_block)
                .fg(Color::White)
                .bg(Color::Black)
                .centered()
                .render(inner_layout[0], buf);
        } else {
            Paragraph::new("Playing")
                .block(status_playing_block)
                .fg(Color::White)
                .bg(Color::Black)
                .centered()
                .render(inner_layout[0], buf);
        };

        // status_paragraph.render(inner_layout[0], buf);
    }
}
