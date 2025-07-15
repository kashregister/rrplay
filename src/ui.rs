use crate::app::*;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    prelude::*,
    style::{Color, Stylize},
    widgets::{Block, BorderType, Borders, Clear, Gauge, Paragraph, Widget, Wrap},
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
                // File path
                Constraint::Percentage(15),
                // Title
                Constraint::Percentage(40),
                // Artist
                Constraint::Percentage(15),
                // Album
                Constraint::Percentage(15),
                // Genre
                Constraint::Percentage(10),
                // Duration
                Constraint::Percentage(5),
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

        let title_block = Block::bordered()
            .title({
                if self.mode == Mode::Select || self.mode == Mode::Search {
                    "Search results"
                } else if self.mode == Mode::Sitback {
                    "Queue"
                } else {
                    "Undefined"
                }
            })
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Plain)
            .border_style({
                if self.mode == Mode::Select {
                    Style::new().yellow()
                } else {
                    Style::new()
                }
            });
        let borderless_block = Block::new().borders(Borders::NONE);
        let array_test: Vec<Vec<Line<'_>>> = {
            let mut tmp_results: Vec<Vec<Line<'_>>> = vec![
                vec![
                    "".into(),
                    Span::styled(
                        "[1] Path",
                        Style::default()
                            .fg(
                                if self.search_by == SearchBy::FilePath
                                    && (self.mode == Mode::Search || self.mode == Mode::Select)
                                {
                                    Color::Yellow
                                } else {
                                    Color::White
                                },
                            )
                            .add_modifier(Modifier::BOLD),
                    )
                    .into(),
                ],
                vec![
                    "".into(),
                    Span::styled(
                        "[2] Title",
                        Style::default()
                            .fg(
                                if self.search_by == SearchBy::Title
                                    && (self.mode == Mode::Search || self.mode == Mode::Select)
                                {
                                    Color::Yellow
                                } else {
                                    Color::White
                                },
                            )
                            .add_modifier(Modifier::BOLD),
                    )
                    .into(),
                ],
                vec![
                    "".into(),
                    Span::styled(
                        "[3] Artist",
                        Style::default()
                            .fg(
                                if self.search_by == SearchBy::Artist
                                    && (self.mode == Mode::Search || self.mode == Mode::Select)
                                {
                                    Color::Yellow
                                } else {
                                    Color::White
                                },
                            )
                            .add_modifier(Modifier::BOLD),
                    )
                    .into(),
                ],
                vec![
                    "".into(),
                    Span::styled(
                        "[4] Album",
                        Style::default()
                            .fg(
                                if self.search_by == SearchBy::Album
                                    && (self.mode == Mode::Search || self.mode == Mode::Select)
                                {
                                    Color::Yellow
                                } else {
                                    Color::White
                                },
                            )
                            .add_modifier(Modifier::BOLD),
                    )
                    .into(),
                ],
                vec![
                    "".into(),
                    Span::styled(
                        "[5] Genre",
                        Style::default()
                            .fg(
                                if self.search_by == SearchBy::Genre
                                    && (self.mode == Mode::Search || self.mode == Mode::Select)
                                {
                                    Color::Yellow
                                } else {
                                    Color::White
                                },
                            )
                            .add_modifier(Modifier::BOLD),
                    )
                    .into(),
                ],
                vec![
                    "".into(),
                    Span::styled("Length", Style::default().add_modifier(Modifier::BOLD)).into(),
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

            // TODO: Use a scrollable table instead of a paragraph
            for (i, song) in to_iter.iter().enumerate() {
                for (n, item) in &mut tmp_results.iter_mut().enumerate() {
                    item.push(
                        Span::styled(
                            {
                                match n {
                                    0 => {
                                        let mut cut_str = " ".to_string();
                                        cut_str.push_str(&song.file_path);
                                        cut_str
                                    }
                                    1 => song.title.clone(),
                                    2 => song.artist.clone(),
                                    3 => song.album.clone(),
                                    4 => song.genre.clone(),
                                    5 => generate_label(song.duration.as_secs()),
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
                                            Modifier::UNDERLINED
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

        Paragraph::new("")
            .block(title_block)
            .centered()
            .render(layout[0], buf);

        let mode_block = Block::bordered()
            .title(if self.mode == Mode::Search {
                "Query"
            } else if self.mode == Mode::Sitback {
                ""
            } else if self.mode == Mode::Select {
                "Query"
            } else {
                "Undefined"
            })
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Plain)
            .border_style(if self.mode == Mode::Search {
                Style::new().yellow()
            } else {
                Style::new()
            });

        let status_volume_block = Block::bordered()
            .title("Volume")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Plain);

        let status_playing_block = Block::bordered()
            .title("Status")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Plain);

        if self.mode == Mode::Sitback {
            if self.queue.len() > 0 {
                let label = Span::styled(
                    generate_label(self.sink.get_pos().as_secs()),
                    Style::new().italic().bold().fg(Color::DarkGray),
                );
                Gauge::default()
                    .block(mode_block)
                    .gauge_style(if self.sink.is_paused() {
                        Style::new().red()
                    } else {
                        Style::new().green()
                    })
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
        } else if self.mode == Mode::Search || self.mode == Mode::Select {
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

        Paragraph::new({
            if self.queue.len() == 0 {
                "N/A"
            } else {
                if self.sink.is_paused() {
                    "Paused"
                } else {
                    "Playing"
                }
            }
        })
        .block(status_playing_block)
        .fg(Color::White)
        .bg(Color::Black)
        .centered()
        .render(bottom_layout[0], buf);

        if self.help_display {
            let help_block = Block::new()
                .title("Help desk")
                .title_style(Style::new().white().bold())
                .borders(Borders::ALL)
                .border_style(Style::new());
            let popup_area = Rect {
                x: area.width / 4,
                y: area.height / 3,
                width: area.width / 2,
                height: area.height / 3,
            };
            Clear.render(popup_area, buf);

            if let Some(sources_ok) = self.sources.clone() {
                let mut sources: Vec<Line<'_>> = sources_ok
                    .into_iter()
                    .map(|s| {
                        Span::styled(
                            s.0,
                            Style::default().fg(if s.1 { Color::Green } else { Color::Red }),
                        )
                        .into()
                    })
                    .collect();
                let mut start = Vec::from([Span::styled("Sources:", Style::default()).into()]);
                start.append(&mut sources);
                _ = Paragraph::new(start)
                    .style(Style::new())
                    .block(help_block.clone())
                    .render(popup_area, buf);
            } else {
                Paragraph::new({
                    let mut ret = "No sources found...\nAdd some!\nFile location:\n".to_string();
                    if let Some(cfg_dir) = dirs::config_dir() {
                        let str = cfg_dir.join("rrplay").join("config").display().to_string();
                        ret.push_str(&str);
                    }
                    ret
                })
                .wrap(Wrap { trim: true })
                .style(Style::new())
                .block(help_block)
                .render(popup_area, buf);
            }
        }
    }
}
