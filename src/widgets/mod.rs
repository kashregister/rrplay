pub mod popup_manual;
pub mod popup_notif;

use ratatui::style::Color;
#[derive(Clone)]
pub struct PopupNotif {
    pub message: Vec<(String, Color)>,
    pub title: String,
    pub border_color: Color,
    pub duration_ticks: Option<usize>,
    pub index: usize,
}

#[derive(Clone)]
pub struct PopupManual {
    pub message: Vec<(String, Color)>,
    pub title: String,
    pub bottom_hint: String,
    pub border_color: Color,
}
