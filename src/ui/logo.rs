use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::Color;
use ratatui::symbols::Marker;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::widgets::canvas::{Canvas, Painter, Shape};

use super::theme;

/// The logo is authored in a 100x100 coordinate box. Canvas scales it to the
/// available area, so these numbers only describe the drawing, not its size.
const LOGO_W: f64 = 100.0;
const LOGO_H: f64 = 100.0;

const MIN_WIDTH: u16 = 20;
const MIN_HEIGHT: u16 = 8;

// Geometry of the mark, in logo units.
const TRUNK_X: f64 = 28.0;
const BOTTOM_Y: f64 = 13.0;
const TOP_Y: f64 = 87.0;
const TIP_X: f64 = 72.0;
const MID_Y: f64 = 50.0;
const BRANCH_JOIN_TOP: f64 = 63.0;
const BRANCH_JOIN_BOTTOM: f64 = 37.0;
const BRANCH_CTRL_TOP: f64 = 80.0;
const BRANCH_CTRL_BOTTOM: f64 = 20.0;

/// Half-width of the strokes. Thickness is expressed in logo units, so the
/// lines actually grow with the logo instead of staying one dot thin.
const TRUNK_HALF: f64 = 2.5;
const BRANCH_HALF: f64 = 2.2;

/// Commit nodes are rings instead of dots, which reads much better once the
/// logo is big: a solid blob just looks like a smudge at braille resolution.
const PIN_OUTER: f64 = 8.2;
const PIN_HOLE: f64 = 4.6;
const PIN_SCALE: f64 = 1.15;
const MID_PIN_SCALE: f64 = 1.0;

/// A dark contour around every shape gives the mark a crisp edge and a little
/// depth, without the dithering noise a translucent glow would create.
const CONTOUR: f64 = 1.0;
const SHEEN: f64 = 0.18;

/// Keep a little air between the mark and the panel border.
const PADDING: f64 = 1.06;

const WARM_HIGHLIGHT: Color = Color::Rgb(255, 238, 206);
const COOL_HIGHLIGHT: Color = Color::Rgb(225, 248, 250);

/// Braille dots, as (column, row, bit) inside one terminal cell.
const DOTS: [(usize, usize, u8); 8] = [
    (0, 0, 0x01),
    (0, 1, 0x02),
    (1, 0, 0x04),
    (1, 1, 0x08),
    (0, 2, 0x10),
    (0, 3, 0x20),
    (1, 2, 0x40),
    (1, 3, 0x80),
];

#[derive(Clone, Copy)]
enum Tone {
    Trunk,
    Branch,
}

impl Tone {
    fn color(self, y: f64) -> Color {
        match self {
            Tone::Trunk => {
                let t = ((y - BOTTOM_Y) / (TOP_Y - BOTTOM_Y)).clamp(0.0, 1.0);
                // Deep amber at the bottom, warm gold at the top.
                mix(
                    mix(theme::accent_color(), theme::panel_color(), 0.32),
                    mix(theme::accent_color(), WARM_HIGHLIGHT, 0.5),
                    t,
                )
            }
            Tone::Branch => {
                let t = ((y - MID_Y).abs() / (TOP_Y - MID_Y)).clamp(0.0, 1.0);
                // Deep teal at the trunk, frosty aqua at the tips.
                mix(
                    mix(theme::soft_accent_color(), theme::panel_color(), 0.32),
                    mix(theme::soft_accent_color(), COOL_HIGHLIGHT, 0.55),
                    t,
                )
            }
        }
    }

    fn highlight(self) -> Color {
        match self {
            Tone::Trunk => WARM_HIGHLIGHT,
            Tone::Branch => COOL_HIGHLIGHT,
        }
    }

    fn half_width(self) -> f64 {
        match self {
            Tone::Trunk => TRUNK_HALF,
            Tone::Branch => BRANCH_HALF,
        }
    }
}

const PINS: [(f64, f64, Tone, f64); 5] = [
    (TRUNK_X, BOTTOM_Y, Tone::Trunk, PIN_SCALE),
    (TRUNK_X, MID_Y, Tone::Trunk, MID_PIN_SCALE),
    (TRUNK_X, TOP_Y, Tone::Trunk, PIN_SCALE),
    (TIP_X, BOTTOM_Y, Tone::Branch, PIN_SCALE),
    (TIP_X, TOP_Y, Tone::Branch, PIN_SCALE),
];

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
    let resolution = (usize::from(area.width) * 2, usize::from(area.height) * 4);
    let dots_x = resolution.0 as f64;
    let dots_y = resolution.1 as f64;

    // One scale for both axes, so circles stay circles.
    let scale = (dots_x / LOGO_W).min(dots_y / LOGO_H) / PADDING;
    if scale <= 0.0 {
        return;
    }

    // Expand the visible bounds to the full dot grid. The logo is centered in
    // that grid, which letterboxes it instead of stretching it.
    let x_span = dots_x / scale;
    let y_span = dots_y / scale;
    let cx = LOGO_W / 2.0;
    let cy = LOGO_H / 2.0;

    let logo = Logo::new(resolution, x_span, y_span);

    let canvas = Canvas::default()
        .marker(Marker::Braille)
        .background_color(theme::panel_color())
        .x_bounds([cx - x_span / 2.0, cx + x_span / 2.0])
        .y_bounds([cy - y_span / 2.0, cy + y_span / 2.0])
        .paint(move |ctx| ctx.draw(&logo));

    frame.render_widget(canvas, area);
}

/// A rasterized version of the mark. Instead of drawing one-dot lines, every
/// braille dot is evaluated against the shapes' signed distance fields, which
/// gives us thick strokes, rounded caps, smooth joins and shading for free.
struct Logo {
    resolution: (usize, usize),
    x_span: f64,
    y_span: f64,
    branches: [Vec<(f64, f64)>; 2],
}

impl Logo {
    fn new(resolution: (usize, usize), x_span: f64, y_span: f64) -> Self {
        Self {
            resolution,
            x_span,
            y_span,
            branches: [
                bezier(
                    (TRUNK_X, BRANCH_JOIN_TOP),
                    (TRUNK_X, BRANCH_CTRL_TOP),
                    (TIP_X, BRANCH_CTRL_TOP),
                    (TIP_X, TOP_Y),
                ),
                bezier(
                    (TRUNK_X, BRANCH_JOIN_BOTTOM),
                    (TRUNK_X, BRANCH_CTRL_BOTTOM),
                    (TIP_X, BRANCH_CTRL_BOTTOM),
                    (TIP_X, BOTTOM_Y),
                ),
            ],
        }
    }

    /// Center of a dot in logo coordinates. The grid origin is top-left,
    /// while the logo itself uses bottom-left like the canvas does.
    fn point(&self, ix: usize, iy: usize) -> (f64, f64) {
        let (rx, ry) = self.resolution;
        let x = LOGO_W / 2.0 - self.x_span / 2.0
            + (ix as f64 + 0.5) / rx as f64 * self.x_span;
        let y = LOGO_H / 2.0 + self.y_span / 2.0
            - (iy as f64 + 0.5) / ry as f64 * self.y_span;
        (x, y)
    }

    /// Color of a single braille dot, or `None` for empty space.
    fn sample(&self, x: f64, y: f64) -> Option<Color> {
        let trunk = [(TRUNK_X, BOTTOM_Y), (TRUNK_X, TOP_Y)];
        let mut nearest: Option<(f64, Tone)> = None;

        let branches = self
            .branches
            .iter()
            .map(|points| (points.as_slice(), Tone::Branch));
        for (points, tone) in std::iter::once((&trunk[..], Tone::Trunk)).chain(branches) {
            let distance = polyline_distance(x, y, points);
            let sdf = distance - tone.half_width();
            if nearest.is_none_or(|(best, _)| sdf < best) {
                nearest = Some((sdf, tone));
            }
        }

        let mut color = nearest.and_then(|(sdf, tone)| {
            let base = tone.color(y);
            if sdf <= 0.0 {
                // Lightest along the middle of the stroke, like a soft tube.
                let depth = ((sdf + tone.half_width()) / tone.half_width()).clamp(0.0, 1.0);
                Some(mix(base, tone.highlight(), SHEEN * (1.0 - depth)))
            } else if sdf < CONTOUR {
                Some(mix(base, theme::panel_color(), 0.66))
            } else {
                None
            }
        });

        for &(px, py, tone, scale) in &PINS {
            let distance = ((x - px).powi(2) + (y - py).powi(2)).sqrt();
            let outer = PIN_OUTER * scale;
            let hole = PIN_HOLE * scale;
            let base = tone.color(py);

            if distance <= hole {
                // Empty hole, so the trunk is cut cleanly at each pin.
                color = Some(theme::panel_color());
            } else if distance <= outer {
                let t = ((distance - hole) / (outer - hole)).clamp(0.0, 1.0);
                let inner = mix(base, tone.highlight(), 0.5);
                let rim = mix(base, theme::panel_color(), 0.1);
                color = Some(mix(inner, rim, t));
            } else if distance < outer + CONTOUR {
                let contour =
                    mix(mix(base, tone.highlight(), 0.2), theme::panel_color(), 0.66);
                if color.is_none_or(|current| luminance(contour) > luminance(current)) {
                    color = Some(contour);
                }
            }
        }

        color
    }
}

impl Shape for Logo {
    fn draw(&self, painter: &mut Painter) {
        let (rx, ry) = self.resolution;

        for cell_y in 0..ry / 4 {
            for cell_x in 0..rx / 2 {
                let mut pattern = 0u8;
                let mut chosen: Option<Color> = None;
                let mut brightest = f64::NEG_INFINITY;

                for &(dx, dy, bit) in &DOTS {
                    let (x, y) = self.point(cell_x * 2 + dx, cell_y * 4 + dy);
                    let Some(color) = self.sample(x, y) else {
                        continue;
                    };
                    // Holes and empty space are the panel itself; leaving them
                    // unpainted keeps the background clean.
                    if color == theme::panel_color() {
                        continue;
                    }

                    pattern |= bit;
                    let luma = luminance(color);
                    if luma > brightest {
                        brightest = luma;
                        chosen = Some(color);
                    }
                }

                let Some(color) = chosen else {
                    continue;
                };

                // A cell only has one foreground color, so the whole cell is
                // painted with its most prominent dot's shade.
                for &(dx, dy, bit) in &DOTS {
                    if pattern & bit != 0 {
                        painter.paint(cell_x * 2 + dx, cell_y * 4 + dy, color);
                    }
                }
            }
        }
    }
}

fn polyline_distance(x: f64, y: f64, points: &[(f64, f64)]) -> f64 {
    points
        .windows(2)
        .map(|pair| segment_distance(x, y, pair[0], pair[1]))
        .fold(f64::INFINITY, f64::min)
}

fn segment_distance(x: f64, y: f64, a: (f64, f64), b: (f64, f64)) -> f64 {
    let (ax, ay) = a;
    let (bx, by) = b;
    let (dx, dy) = (bx - ax, by - ay);
    let len_sq = dx * dx + dy * dy;

    let t = if len_sq == 0.0 {
        0.0
    } else {
        (((x - ax) * dx + (y - ay) * dy) / len_sq).clamp(0.0, 1.0)
    };

    let cx = ax + t * dx;
    let cy = ay + t * dy;
    ((x - cx).powi(2) + (y - cy).powi(2)).sqrt()
}

/// Sample a cubic Bezier into short segments, so its distance field is cheap
/// to evaluate and the polyline stays visually smooth at braille resolution.
fn bezier(p0: (f64, f64), p1: (f64, f64), p2: (f64, f64), p3: (f64, f64)) -> Vec<(f64, f64)> {
    let steps = 32;
    let mut points = Vec::with_capacity(steps + 1);

    for i in 0..=steps {
        let t = i as f64 / steps as f64;
        let u = 1.0 - t;
        let x = u * u * u * p0.0 + 3.0 * u * u * t * p1.0 + 3.0 * u * t * t * p2.0 + t * t * t * p3.0;
        let y = u * u * u * p0.1 + 3.0 * u * u * t * p1.1 + 3.0 * u * t * t * p2.1 + t * t * t * p3.1;
        points.push((x, y));
    }

    points
}

fn mix(a: Color, b: Color, t: f64) -> Color {
    let t = t.clamp(0.0, 1.0);
    let (Color::Rgb(ar, ag, ab), Color::Rgb(br, bg, bb)) = (a, b) else {
        return b;
    };

    let channel = |a: u8, b: u8| -> u8 {
        (f64::from(a) + (f64::from(b) - f64::from(a)) * t).round() as u8
    };

    Color::Rgb(channel(ar, br), channel(ag, bg), channel(ab, bb))
}

fn luminance(color: Color) -> f64 {
    let Color::Rgb(r, g, b) = color else {
        return 0.0;
    };
    0.2126 * f64::from(r) + 0.7152 * f64::from(g) + 0.0722 * f64::from(b)
}

