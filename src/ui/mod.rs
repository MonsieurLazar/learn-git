mod home;
mod status_bar;
pub mod theme;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::widgets::Block;

use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    frame.render_widget(Block::default().style(theme::app_background()), area);

    let chunks = Layout::vertical([Constraint::Min(3), Constraint::Length(1)]).split(area);

    home::draw(frame, chunks[0]);
    status_bar::draw(frame, chunks[1], &app.state);
}
