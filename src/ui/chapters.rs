use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Padding, Paragraph, Wrap};

use super::theme;

const CHAPTERS: [(&str, &str); 8] = [
    ("Getting started", "install Git, set your name and email"),
    (
        "The three trees",
        "working directory, staging area, repository",
    ),
    ("Your first commit", "init, add, status and commit"),
    ("Branching", "create, switch and compare branches"),
    ("Merging", "fast-forwards, three-way merges and conflicts"),
    ("Remotes", "clone, fetch, pull and push"),
    ("Undoing changes", "restore, reset and revert"),
    ("Rewriting history", "amend, rebase and cherry-pick"),
];

pub fn draw(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::border())
        .style(theme::panel_background());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .split(inner);

    let header = Paragraph::new(vec![
        Line::from("Chapters").style(theme::accent_text()),
        Line::from("A short, hands-on path through Git").style(theme::muted_text()),
        Line::from(""),
    ]);
    frame.render_widget(header, chunks[0]);

    let columns = Layout::horizontal([Constraint::Percentage(62), Constraint::Percentage(38)])
        .split(chunks[1]);

    let list = Paragraph::new(
        CHAPTERS
            .iter()
            .enumerate()
            .map(|(index, (title, summary))| {
                Line::from(vec![
                    Span::styled(format!("{:>2}. ", index + 1), theme::accent_text()),
                    Span::styled(*title, theme::soft_accent_text()),
                    Span::styled(format!(" — {summary}"), theme::muted_text()),
                ])
            })
            .collect::<Vec<_>>(),
    )
    .wrap(Wrap { trim: true });
    frame.render_widget(list, columns[0]);

    let aside = Block::default()
        .borders(Borders::LEFT)
        .border_style(theme::border())
        .padding(Padding::left(2));
    let aside_inner = aside.inner(columns[1]);
    frame.render_widget(aside, columns[1]);

    let notes = Paragraph::new(vec![
        Line::from("How to use").style(theme::accent_text()),
        Line::from(""),
        Line::from("Work through the chapters in order, or jump to whatever you need.")
            .style(theme::muted_text()),
        Line::from(""),
        Line::from("Each lesson is short and hands-on: read it, then try the commands in a scratch repository.")
            .style(theme::muted_text()),
        Line::from(""),
        Line::from(vec![
            Span::styled("Esc", theme::soft_accent_text()),
            Span::styled("  go back to home", theme::muted_text()),
        ]),
        Line::from(vec![
            Span::styled("q", theme::soft_accent_text()),
            Span::styled("    quit learn-git", theme::muted_text()),
        ]),
    ])
    .wrap(Wrap { trim: true });
    frame.render_widget(notes, aside_inner);

    let footer_text = Paragraph::new(Line::from("Esc or Backspace to go back · q to quit"))
        .alignment(Alignment::Center)
        .style(theme::soft_accent_text());
    frame.render_widget(footer_text, chunks[2]);
}
