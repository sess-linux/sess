use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{execute, ExecutableCommand};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Terminal;
use std::io;

use crate::core::model::SessionSummary;
use crate::core::storage::{self, human_size};

pub enum PickerResult {
    Open(String),
    Delete(String),
    Quit,
}

/// Fixed width of the relative size bar, in characters. Matches the approved mockup:
/// each session shows a bar proportional to the heaviest item in the list.
const BAR_WIDTH: usize = 12;

pub fn run() -> Result<PickerResult> {
    let sessions = storage::list()?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = ListState::default();
    if !sessions.is_empty() {
        state.select(Some(0));
    }

    let total_bytes: u64 = sessions.iter().map(|s| s.size_bytes).sum();
    let max_bytes = sessions.iter().map(|s| s.size_bytes).max().unwrap_or(1).max(1);

    let result = loop {
        terminal.draw(|f| {
            let area = f.size();
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(2), Constraint::Min(0), Constraint::Length(1)])
                .split(area);

            draw_summary(f, layout[0], sessions.len(), total_bytes);
            draw_list(f, layout[1], &sessions, max_bytes, &mut state);
            draw_footer(f, layout[2]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break PickerResult::Quit,
                KeyCode::Down | KeyCode::Char('j') => move_selection(&mut state, sessions.len(), 1),
                KeyCode::Up | KeyCode::Char('k') => move_selection(&mut state, sessions.len(), -1),
                KeyCode::Enter => {
                    if let Some(i) = state.selected() {
                        if let Some(s) = sessions.get(i) {
                            break PickerResult::Open(s.name.clone());
                        }
                    }
                }
                KeyCode::Char('d') => {
                    if let Some(i) = state.selected() {
                        if let Some(s) = sessions.get(i) {
                            break PickerResult::Delete(s.name.clone());
                        }
                    }
                }
                _ => {}
            }
        }
    };

    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(result)
}

fn move_selection(state: &mut ListState, len: usize, delta: i32) {
    if len == 0 {
        return;
    }
    let current = state.selected().unwrap_or(0) as i32;
    let next = (current + delta).rem_euclid(len as i32);
    state.select(Some(next as usize));
}

fn draw_summary(f: &mut ratatui::Frame, area: Rect, count: usize, total_bytes: u64) {
    let text = format!(" sessions: {count}   total space: {} ", human_size(total_bytes));
    let p = Paragraph::new(text)
        .style(Style::default().fg(Color::Gray))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
    f.render_widget(p, area);
}

fn size_bar(bytes: u64, max_bytes: u64) -> String {
    let ratio = (bytes as f64 / max_bytes as f64).clamp(0.0, 1.0);
    let filled = ((ratio * BAR_WIDTH as f64).round() as usize).max(if bytes > 0 { 1 } else { 0 });
    let empty = BAR_WIDTH.saturating_sub(filled);
    format!("{}{}", "#".repeat(filled), "-".repeat(empty))
}

fn draw_list(
    f: &mut ratatui::Frame,
    area: Rect,
    sessions: &[SessionSummary],
    max_bytes: u64,
    state: &mut ListState,
) {
    if sessions.is_empty() {
        let p = Paragraph::new("no saved sessions yet. `sess save <name>` to create one.")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(p, area);
        return;
    }

    let items: Vec<ListItem> = sessions
        .iter()
        .map(|s| {
            let bar = size_bar(s.size_bytes, max_bytes);
            let line = Line::from(vec![
                Span::styled(format!("{:<20}", s.name), Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(format!("{:>2} panes  ", s.pane_count), Style::default().fg(Color::Gray)),
                Span::styled(bar, Style::default().fg(Color::DarkGray)),
                Span::raw(format!("  {:>8}", human_size(s.size_bytes))),
                Span::styled(
                    format!("   {}", s.created_at.format("%Y-%m-%d %H:%M")),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");

    f.render_stateful_widget(list, area, state);
}

fn draw_footer(f: &mut ratatui::Frame, area: Rect) {
    let footer = Line::from(vec![
        Span::styled("enter", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" open   "),
        Span::styled("d", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" delete   "),
        Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" quit"),
    ]);
    f.render_widget(
        Paragraph::new(footer).style(Style::default().fg(Color::DarkGray)),
        area,
    );
}
