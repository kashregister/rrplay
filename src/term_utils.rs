use crossterm::*;
use crossterm::{
    cursor::{self, MoveTo, MoveToColumn},
    style::{ResetColor, SetAttribute, SetBackgroundColor},
    terminal::{self, ClearType},
};
use std::io;
use std::io::Write;
// clear the entire terminal
pub fn t_clear_all() {
    io::stdout()
        .execute(terminal::Clear(ClearType::All))
        .unwrap();
}
// clear the whole line
pub fn t_clear_line() {
    io::stdout()
        .execute(terminal::Clear(ClearType::CurrentLine))
        .unwrap();
}
// move to the very top (top left)
pub fn t_mv_start() {
    io::stdout().execute(MoveTo(0, 0)).unwrap();
}
// move to the very bottom
pub fn t_mv_end() {
    let t_sz = terminal::size().unwrap();
    io::stdout().execute(MoveTo(0, t_sz.1)).unwrap();
}
// move to the start of the current line
pub fn t_mv_sol() {
    io::stdout().execute(MoveToColumn(0)).unwrap();
}
// change the lines style for the song we are hovering over
pub fn t_bg_gray() {
    io::stdout()
        .execute(SetBackgroundColor(crossterm::style::Color::DarkGrey))
        .unwrap();
    io::stdout()
        .execute(SetAttribute(crossterm::style::Attribute::Bold))
        .unwrap();
}
// make text bold

pub fn t_txt_bold() {
    io::stdout()
        .execute(SetAttribute(crossterm::style::Attribute::Bold))
        .unwrap();
}

// make text not bold
pub fn t_txt_nobold() {
    io::stdout()
        .execute(SetAttribute(crossterm::style::Attribute::NoBold))
        .unwrap();
}
// reset the terminals styling
pub fn t_bg_reset() {
    io::stdout().execute(ResetColor).unwrap();
}

pub fn t_flush() {
    io::stdout().flush().unwrap();
}

pub fn t_cursor_show() {
    io::stdout().execute(cursor::Show).unwrap();
}
pub fn t_cursor_hide() {
    io::stdout().execute(cursor::Hide).unwrap();
}
