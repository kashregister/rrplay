use crate::app::App;
use crossterm::execute;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use ratatui::prelude::*;
use std::io::stdout;
use std::panic::set_hook;
use std::panic::take_hook;

pub mod app;
pub mod event;
pub mod ui;
pub mod widgets;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    init_panic_hook();
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::default().run(terminal).await;
    ratatui::restore();
    result
}

pub fn init_panic_hook() {
    let original_hook = take_hook();
    set_hook(Box::new(move |panic_info| {
        // intentionally ignore errors here since we're already in a panic
        let _ = restore_tui();
        original_hook(panic_info);
    }));
}

pub fn init_tui() -> color_eyre::Result<Terminal<impl Backend>> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout()))?)
}

pub fn restore_tui() -> color_eyre::Result<()> {
    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen)?;
    Ok(())
}
