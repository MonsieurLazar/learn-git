use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, LineGauge, Paragraph};

use crate::update::UpdateState;
use crate::version::VERSION;

use super::theme;

pub fn draw(frame: &mut Frame, area: Rect, state: &UpdateState) {
    frame.render_widget(Block::default().style(theme::panel_alt_background()), area);

    let version = format!(" v{VERSION}");

    match state {
        UpdateState::Downloading {
            latest,
            downloaded,
            total,
        } => {
            let chunks = Layout::horizontal([
                Constraint::Length(version.len() as u16),
                Constraint::Min(10),
                Constraint::Length(5),
            ])
            .split(area);

            frame.render_widget(
                Paragraph::new(version).style(theme::muted_text()),
                chunks[0],
            );

            let ratio = total
                .filter(|total| *total > 0)
                .map(|total| *downloaded as f64 / total as f64)
                .unwrap_or(0.0)
                .clamp(0.0, 1.0);

            let gauge = LineGauge::default()
                .ratio(ratio)
                .label(Line::from(format!(" downloading v{latest}")))
                .filled_style(theme::accent_text())
                .unfilled_style(theme::muted_text());

            frame.render_widget(gauge, chunks[1]);

            let percent = Paragraph::new(format!("{:>3}% ", (ratio * 100.0) as u16))
                .alignment(Alignment::Right)
                .style(theme::muted_text());

            frame.render_widget(percent, chunks[2]);
        }
        _ => {
            let (message, style) = match state {
                UpdateState::Available { release } => (
                    format!("v{} available · ctrl+u to update ", release.version),
                    theme::soft_accent_text(),
                ),
                UpdateState::Ready { latest } => (
                    format!("v{latest} downloaded · ctrl+u to restart "),
                    theme::soft_accent_text(),
                ),
                UpdateState::Failed { message } => {
                    (format!("{message} · ctrl+u to retry "), theme::muted_text())
                }
                _ => (String::new(), theme::muted_text()),
            };

            let chunks =
                Layout::horizontal([Constraint::Length(version.len() as u16), Constraint::Min(0)])
                    .split(area);

            frame.render_widget(
                Paragraph::new(version).style(theme::muted_text()),
                chunks[0],
            );
            frame.render_widget(
                Paragraph::new(message)
                    .alignment(Alignment::Right)
                    .style(style),
                chunks[1],
            );
        }
    }
}
