use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::Color;
use ratatui::symbols::Marker;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::widgets::canvas::{Canvas, Circle, Context, Line as CanvasLine};

use super::theme;

/// The logo is authored in a 100x100 coordinate box. Canvas scales it to the
/// available area, so these numbers only describe the drawing, not its size.
const LOGO_W: f64 = 100.0;
const LOGO_H: f64 = 100.0;

const MIN_WIDTH: u16 = 20;
const MIN_HEIGHT: u16 = 8;

pub fn draw(frame: &mut Frame, area: Rect) {
    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        let fallback = Paragraph::new(Line::from("learn-git"))
            .alignment(Alignment::Center)
            .style(theme::accent_text());
        let fallback_area = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .split(area)[1];
        frame.render_widget(fallback, fallback_area);
        return;
    }

    // Braille gives 2x4 dots per terminal cell. Those dots are roughly square
    // in a typical monospace font, which is what lets us reason about aspect
    // ratio in dot space instead of cell space.
    let dots_x = f64::from(area.width) * 2.0;
    let dots_y = f64::from(area.height) * 4.0;

    // One scale for both axes, so circles stay circles.
    let scale = (dots_x / LOGO_W).min(dots_y / LOGO_H);
    if scale <= 0.0 {
        return;
    }

    // Expand the visible bounds to the full dot grid. The logo is centered in
    // that grid, which letterboxes it instead of stretching it.
    let x_span = dots_x / scale;
    let y_span = dots_y / scale;
    let cx = LOGO_W / 2.0;
    let cy = LOGO_H / 2.0;

    let canvas = Canvas::default()
        .marker(Marker::Braille)
        .background_color(theme::panel_color())
        .x_bounds([cx - x_span / 2.0, cx + x_span / 2.0])
        .y_bounds([cy - y_span / 2.0, cy + y_span / 2.0])
        .paint(|ctx| {
            let main = theme::accent_color();
            let branch = theme::soft_accent_color();

            // Main branch.
            ctx.draw(&CanvasLine {
                x1: 32.0,
                y1: 10.0,
                x2: 32.0,
                y2: 90.0,
                color: main,
            });

            // Branch curving off to the right, then up.
            curve(ctx, (32.0, 53.0), (32.0, 68.0), (68.0, 68.0), branch);
            ctx.draw(&CanvasLine {
                x1: 68.0,
                y1: 68.0,
                x2: 68.0,
                y2: 90.0,
                color: branch,
            });

            // Commit nodes.
            for (x, y, color) in [
                (32.0, 10.0, main),
                (32.0, 45.0, main),
                (32.0, 90.0, main),
                (68.0, 68.0, branch),
                (68.0, 90.0, branch),
            ] {
                ctx.draw(&Circle {
                    x,
                    y,
                    radius: 5.0,
                    color,
                });
            }
        });

    frame.render_widget(canvas, area);
}

/// Draw a quadratic Bezier curve as a series of short line segments.
fn curve(ctx: &mut Context, p0: (f64, f64), p1: (f64, f64), p2: (f64, f64), color: Color) {
    let steps = 24;
    let mut prev = p0;

    for i in 1..=steps {
        let t = f64::from(i) / f64::from(steps);
        let u = 1.0 - t;

        let x = u * u * p0.0 + 2.0 * u * t * p1.0 + t * t * p2.0;
        let y = u * u * p0.1 + 2.0 * u * t * p1.1 + t * t * p2.1;

        ctx.draw(&CanvasLine {
            x1: prev.0,
            y1: prev.1,
            x2: x,
            y2: y,
            color,
        });
        prev = (x, y);
    }
}
