use std::time::Duration;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(6),
            Constraint::Length(3),
        ])
        .split(frame.area());

    render_header(frame, chunks[0], app);
    render_playlist(frame, chunks[1], app);
    render_now_playing(frame, chunks[2], app);
    render_footer(frame, chunks[3], app);
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let title = if app.search_mode {
        format!(" R-PLAY MUSIC PLAYER  |  Search: {}_ ", app.search_query)
    } else if !app.search_query.is_empty() {
        format!(
            " R-PLAY MUSIC PLAYER  |  Filter: '{}' ({} tracks) ",
            app.search_query,
            app.filtered_indices.len()
        )
    } else {
        format!(" R-PLAY MUSIC PLAYER  |  {} tracks ", app.tracks.len())
    };

    let header = Paragraph::new(Line::from(Span::styled(
        title,
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, area);
}

fn render_playlist(frame: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = if app.filtered_indices.is_empty() {
        vec![ListItem::new("  (No matching tracks)")]
    } else {
        app.filtered_indices
            .iter()
            .map(|&idx| {
                let track = &app.tracks[idx];
                let is_playing = app.current_track_index == Some(idx);
                let marker = if is_playing { "▶ " } else { "  " };
                let dur = track
                    .duration
                    .map(format_duration)
                    .unwrap_or_else(|| "--:--".to_string());
                let text = format!("{}{} [{}]", marker, track.display_name(), dur);
                let style = if is_playing {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(Line::from(Span::styled(text, style)))
            })
            .collect()
    };

    let mut state = ListState::default();
    if !app.filtered_indices.is_empty() {
        state.select(Some(app.selected_index));
    }

    let list = List::new(items)
        .block(Block::default().title(" Playlist ").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(list, area, &mut state);
}

fn render_now_playing(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(" Now Playing ")
        .borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    let (title_line, total_dur) = match app.current_track() {
        Some(track) => {
            let album = track
                .album
                .as_deref()
                .map(|a| format!(" ({a})"))
                .unwrap_or_default();
            (format!("{}{}", track.display_name(), album), track.duration)
        }
        None => (app.status_message.clone(), None),
    };

    let state_icon = if app.current_track().is_none() {
        "■ Stopped"
    } else if app.is_paused() {
        "⏸ Paused"
    } else {
        "▶ Playing"
    };

    let info_line = format!(
        "{}  |  Vol: {}%  |  Shuffle: {}  |  Repeat: {}",
        state_icon,
        app.volume_percent(),
        if app.shuffle { "On" } else { "Off" },
        app.repeat.label()
    );

    let elapsed = app.playback_position();
    let ratio = match total_dur {
        Some(total) if !total.is_zero() => {
            (elapsed.as_secs_f64() / total.as_secs_f64()).clamp(0.0, 1.0)
        }
        _ => 0.0,
    };

    let time_label = format!(
        "{} / {}",
        format_duration(elapsed),
        total_dur
            .map(format_duration)
            .unwrap_or_else(|| "--:--".to_string())
    );

    frame.render_widget(
        Paragraph::new(Span::styled(
            title_line,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        rows[0],
    );
    frame.render_widget(
        Paragraph::new(Span::styled(info_line, Style::default().fg(Color::Cyan))),
        rows[1],
    );
    frame.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(Color::Green).bg(Color::DarkGray))
            .ratio(ratio)
            .label(time_label),
        rows[2],
    );
    frame.render_widget(
        Paragraph::new(Span::styled(
            app.status_message.as_str(),
            Style::default().fg(Color::Gray),
        )),
        rows[3],
    );
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let text = if app.search_mode {
        " Type to filter | Enter: Apply | Esc: Clear & Exit "
    } else {
        " Space: Play/Pause | Enter: Play | ↑/↓: Nav | N/P: Next/Prev | +/-: Vol | S: Shuffle | R: Repeat | /: Search | Q: Quit "
    };

    let footer = Paragraph::new(Line::from(Span::styled(
        text,
        Style::default().fg(Color::DarkGray),
    )))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, area);
}

fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    format!("{:02}:{:02}", secs / 60, secs % 60)
}
