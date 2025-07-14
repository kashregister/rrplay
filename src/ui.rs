use crate::app::App;
use crate::app::Mode;
use crate::app::Song;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    prelude::*,
    style::{Color, Stylize},
    widgets::{Block, BorderType, Borders, Gauge, Paragraph, Widget},
};

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

        let top_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                // Index
                Constraint::Percentage(5),
                // File Name
                Constraint::Percentage(50),
                // Duration
                Constraint::Percentage(5),
                // Artist
                Constraint::Percentage(15),
                // Album
                Constraint::Percentage(15),
                // Genre
                Constraint::Percentage(10),
            ])
            .split(layout[0]);
        let bottom_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(18),
                Constraint::Percentage(72),
                Constraint::Percentage(10),
            ])
            .split(layout[1]);

        let title_block = if self.mode == Mode::Select {
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
                .title("Undefined")
                .title_alignment(Alignment::Center)
        };
        let borderless_block = Block::new().borders(Borders::NONE);

        let array_test: Vec<Vec<Line<'_>>> = {
            let mut tmp_results: Vec<Vec<Line<'_>>> = vec![
                vec![
                    "".into(),
                    Span::styled("Index", Style::default().add_modifier(Modifier::BOLD)).into(),
                ],
                vec![
                    "".into(),
                    Span::styled("Title", Style::default().add_modifier(Modifier::BOLD)).into(),
                ],
                vec![
                    "".into(),
                    Span::styled("Length", Style::default().add_modifier(Modifier::BOLD)).into(),
                ],
                vec![
                    "".into(),
                    Span::styled("Artist", Style::default().add_modifier(Modifier::BOLD)).into(),
                ],
                vec![
                    "".into(),
                    Span::styled("Album", Style::default().add_modifier(Modifier::BOLD)).into(),
                ],
                vec![
                    "".into(),
                    Span::styled("Genre", Style::default().add_modifier(Modifier::BOLD)).into(),
                ],
            ];
            let mut to_iter: Vec<Song> = if self.mode == Mode::Sitback {
                self.queue.clone()
            } else {
                self.search_results.clone()
            };
            to_iter.truncate({
                if self.terminal_size.1 > 10 {
                    (self.terminal_size.1 - 6).into()
                } else {
                    self.terminal_size.1.into()
                }
            });

            for (i, song) in to_iter.iter().enumerate() {
                for (n, item) in &mut tmp_results.iter_mut().enumerate() {
                    item.push(
                        Span::styled(
                            {
                                match n {
                                    0 => (i + 1 as usize).to_string(),
                                    1 => song.title.clone(),
                                    2 => generate_label(song.duration.as_secs()),
                                    3 => song.artist.clone(),
                                    4 => song.album.clone(),
                                    5 => song.genre.clone(),
                                    _ => "Undefined".to_string(),
                                }
                            },
                            Style::default()
                                .add_modifier({
                                    if i as i32
                                        == self.search_results.len() as i32
                                            - 1
                                            - self.select_index as i32
                                        && self.mode == Mode::Select
                                    {
                                        Modifier::BOLD
                                    } else if self.mode == Mode::Sitback {
                                        if i == 0 {
                                            Modifier::SLOW_BLINK
                                        } else {
                                        Modifier::DIM
                                        }
                                    } else {
                                        Modifier::DIM
                                    }
                                })
                                .fg({
                                    if song.is_valid {
                                        Color::Green
                                    } else {
                                        Color::Red
                                    }
                                }),
                        )
                        .into(),
                    )
                }
            }
            tmp_results.clone()
        };
        for (i, n) in array_test.iter().enumerate() {
            Paragraph::new(n.clone())
                .block(borderless_block.clone())
                .centered()
                .render(top_layout[i], buf);
        }
        // Paragraph::new(detailed_results.0)
        //     .block(borderless_block.clone())
        //     .centered()
        //     .render(top_layout[0], buf);
        //
        // Paragraph::new(detailed_results.1)
        //     .block(borderless_block.clone())
        //     .centered()
        //     .render(top_layout[1], buf);
        // Paragraph::new(detailed_results.2)
        //     .block(borderless_block.clone())
        //     .centered()
        //     .render(top_layout[2], buf);
        //
        // Paragraph::new(detailed_results.3)
        //     .block(borderless_block.clone())
        //     .centered()
        //     .render(top_layout[3], buf);
        //
        // Paragraph::new(detailed_results.4)
        //     .block(borderless_block.clone())
        //     .centered()
        //     .render(top_layout[4], buf);
        // Paragraph::new(detailed_results.5)
        //     .block(borderless_block.clone())
        //     .centered()
        //     .render(top_layout[5], buf);

        Paragraph::new("")
            .block(title_block)
            .centered()
            .render(layout[0], buf);
        // temp_paragraph.render();
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
                    .ratio(self.sink.get_pos().as_secs_f64() / self.queue[0].duration.as_secs_f64())
                    .label(label)
                    .render(bottom_layout[1], buf);
            } else if self.queue.len() > 0 && self.sink.is_paused() {
                let label = Span::styled(
                    generate_label(self.sink.get_pos().as_secs()),
                    Style::new().italic().bold().fg(Color::DarkGray),
                );
                Gauge::default()
                    .block(mode_block)
                    .gauge_style(Style::new().red())
                    .ratio(self.sink.get_pos().as_secs_f64() / self.queue[0].duration.as_secs_f64())
                    .label(label)
                    .render(bottom_layout[1], buf);
            } else {
                Paragraph::new("Empty queue")
                    .block(mode_block)
                    .fg(Color::White)
                    .bg(Color::Black)
                    .centered()
                    .render(bottom_layout[1], buf);
            }
        } else if self.mode == Mode::Search {
            Paragraph::new(self.query.clone())
                .block(mode_block)
                .fg(Color::White)
                .bg(Color::Black)
                .centered()
                .render(bottom_layout[1], buf);
        } else if self.mode == Mode::Select {
            Paragraph::new(self.query.clone())
                .block(mode_block)
                .fg(Color::White)
                .bg(Color::Black)
                .centered()
                .render(bottom_layout[1], buf);
        };
        let volume_paragraph = Paragraph::new(((self.sink.volume() * 100.0) as usize).to_string())
            .block(status_volume_block)
            .fg(Color::White)
            .bg(Color::Black)
            .centered();
        volume_paragraph.render(bottom_layout[2], buf);

        if self.sink.is_paused() {
            Paragraph::new("Paused")
                .block(status_playing_block)
                .fg(Color::White)
                .bg(Color::Black)
                .centered()
                .render(bottom_layout[0], buf);
        } else {
            Paragraph::new("Playing")
                .block(status_playing_block)
                .fg(Color::White)
                .bg(Color::Black)
                .centered()
                .render(bottom_layout[0], buf);
        };

        // status_paragraph.render(bottom_layout[0], buf);
    }
}
