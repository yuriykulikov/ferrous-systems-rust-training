use crate::Model;
use std::io;
use std::io::Stdout;
use std::sync::{Arc, Mutex};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};

/// Draws contents of the [Model] in a TUI
/// TODO write integration tests by replacing the backend
pub fn draw_tui(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    model: Arc<Mutex<Model>>,
) -> io::Result<()> {
    let model = model.lock().unwrap();
    let composed = model.composed();
    let selected_channel_idx = model.selected_channel_idx();
    let selected_channel_name = model.selected_channel_name();
    terminal.draw(|rect| {
        let size = rect.size();
        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Min(2),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(size);

        let horizontal_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
            .split(vertical_layout[1]);

        let mut list_state = ListState::default();
        list_state.select(Some(selected_channel_idx));
        rect.render_stateful_widget(
            render_channels(model.channels()),
            horizontal_layout[0],
            &mut list_state,
        );

        rect.render_widget(
            render_compose(composed.as_ref(), selected_channel_name.as_str()),
            vertical_layout[0],
        );
        rect.render_widget(
            render_emails(model.emails_for_selected_channel()),
            horizontal_layout[1],
        );
    })?;
    Ok(())
}

/// Renders emails as a flat list in a box
fn render_emails<'a>(emails: Vec<String>) -> Paragraph<'a> {
    let email_spans: Vec<Spans> = emails
        .iter()
        .map(|mail| Spans::from(vec![Span::raw(mail.to_owned())]))
        .collect();

    Paragraph::new(email_spans)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Emails")
                .border_type(BorderType::Plain),
        )
}

/// Renders a box with new email or new channel content
fn render_compose<'a>(compose: &'a str, channel: &'a str) -> Paragraph<'a> {
    let title = match channel {
        "+" => "Create a new channel".to_string(),
        existing => format!("Append an email to {}", existing),
    };

    Paragraph::new(vec![Spans::from(vec![Span::raw(compose)])])
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title(title)
                .border_type(BorderType::Plain),
        )
}

/// Renders available channels as a list
/// Selected element is highlighted
fn render_channels<'a>(channels: Vec<String>) -> List<'a> {
    let items: Vec<_> = channels
        .iter()
        .map(|channel| {
            ListItem::new(Spans::from(vec![Span::styled(
                channel.clone(),
                Style::default(),
            )]))
        })
        .collect();

    let channel_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Channels")
        .border_type(BorderType::Plain);

    List::new(items)
        .block(channel_block)
        .highlight_symbol("> ")
        .highlight_style(
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
}
