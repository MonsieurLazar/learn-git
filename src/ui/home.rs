use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};

use super::{logo, theme};

pub fn draw(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::border())
        .style(theme::panel_background());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let main_and_footer =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(inner);

    let main_area = main_and_footer[0];
    let footer_area = main_and_footer[1];

    let left_and_right =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_area);

    let logo_area = left_and_right[0];
    let text_area = left_and_right[1];

    logo::draw(frame, logo_area);

    let right_text = Paragraph::new(vec![
        Line::from("Welcome!").style(theme::accent_text()),
        Line::from(""),
        Line::from("Descriere mai inteligenta").style(theme::soft_accent_text()),
    ])
    .alignment(Alignment::Left)
    .wrap(Wrap { trim: true });

    let right_area = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(4),
        Constraint::Fill(1),
    ])
    .split(text_area)[1];

    let footer_text = Paragraph::new(Line::from("Press enter to continue!"))
        .alignment(Alignment::Center)
        .style(theme::soft_accent_text());

    frame.render_widget(right_text, right_area);
    frame.render_widget(footer_text, footer_area);
}
