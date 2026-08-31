use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{execute, ExecutableCommand};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Terminal;
use std::io;

use crate::core::state::{self, SessionState, StateKind};
use crate::core::{capture, storage};

pub enum PickerResult {
    /// The only action that needs to leave the TUI: attaching hands the
    /// terminal over to tmux, which the alternate screen can't coexist with.
    Open(String),
    Quit,
}

const BAR_WIDTH: usize = 12;

/// What the picker is currently doing, beyond simple navigation. Rename and
/// duplicate need a line of text input; a stale/broken selection needs a
/// choice or a message before anything happens.
enum Mode {
    Normal,
    Prompt {
        kind: PromptKind,
        target: String,
        buffer: String,
    },
    Confirm {
        target: String,
        message: String,
    },
    Message {
        text: String,
    },
}

enum PromptKind {
    Rename,
    Duplicate,
}

pub fn run() -> Result<PickerResult> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut sessions = state::all()?;
    let mut list_state = ListState::default();
    if !sessions.is_empty() {
        list_state.select(Some(0));
    }
    let mut mode = Mode::Normal;
    let mut status: Option<String> = None;

    let result = loop {
        let total_bytes: u64 = sessions.iter().filter_map(|s| s.size_bytes).sum();
        let max_bytes = sessions
            .iter()
            .filter_map(|s| s.size_bytes)
            .max()
            .unwrap_or(1)
            .max(1);

        terminal.draw(|f| {
            let area = f.size();
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(2),
                    Constraint::Min(0),
                    Constraint::Length(2),
                ])
                .split(area);

            draw_summary(f, layout[0], &sessions, total_bytes);
            draw_list(f, layout[1], &sessions, max_bytes, &mut list_state);
            draw_footer(f, layout[2], &mode, &status);
        })?;

        if let Event::Key(key) = event::read()? {
            match &mut mode {
                Mode::Normal => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break PickerResult::Quit,
                    KeyCode::Down | KeyCode::Char('j') => {
                        move_selection(&mut list_state, sessions.len(), 1);
                        status = None;
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        move_selection(&mut list_state, sessions.len(), -1);
                        status = None;
                    }
                    KeyCode::Enter => {
                        if let Some(session) = selected(&sessions, &list_state) {
                            match session.kind {
                                StateKind::Running | StateKind::Saved => {
                                    break PickerResult::Open(session.name.clone());
                                }
                                StateKind::Stale => {
                                    mode = Mode::Confirm {
                                        target: session.name.clone(),
                                        message: session
                                            .detail
                                            .clone()
                                            .unwrap_or_else(|| "this session looks stale.".into()),
                                    };
                                }
                                StateKind::Broken => {
                                    mode = Mode::Message {
                                        text: session.detail.clone().unwrap_or_else(|| {
                                            "this saved session is broken.".into()
                                        }),
                                    };
                                }
                            }
                        }
                    }
                    KeyCode::Char('d') => {
                        if let Some(session) = selected(&sessions, &list_state) {
                            let name = session.name.clone();
                            match storage::delete(&name) {
                                Ok(()) => status = Some(format!("deleted '{name}'.")),
                                Err(e) => status = Some(format!("could not delete '{name}': {e}")),
                            }
                            sessions = state::all()?;
                            clamp_selection(&mut list_state, sessions.len());
                        }
                    }
                    KeyCode::Char('r') => {
                        if let Some(session) = selected(&sessions, &list_state) {
                            mode = Mode::Prompt {
                                kind: PromptKind::Rename,
                                target: session.name.clone(),
                                buffer: String::new(),
                            };
                        }
                    }
                    KeyCode::Char('c') => {
                        if let Some(session) = selected(&sessions, &list_state) {
                            mode = Mode::Prompt {
                                kind: PromptKind::Duplicate,
                                target: session.name.clone(),
                                buffer: String::new(),
                            };
                        }
                    }
                    KeyCode::Char('s') => {
                        status = Some(save_current());
                        sessions = state::all()?;
                        clamp_selection(&mut list_state, sessions.len());
                    }
                    _ => {}
                },
                Mode::Prompt {
                    kind,
                    target,
                    buffer,
                } => match key.code {
                    KeyCode::Esc => mode = Mode::Normal,
                    KeyCode::Enter => {
                        let new_name = buffer.trim().to_string();
                        let target = target.clone();
                        let outcome = if new_name.is_empty() {
                            Err("name can't be empty".to_string())
                        } else {
                            match kind {
                                PromptKind::Rename => {
                                    storage::rename(&target, &new_name).map_err(|e| e.to_string())
                                }
                                PromptKind::Duplicate => storage::load(&target)
                                    .and_then(|mut snap| {
                                        snap.name = new_name.clone();
                                        storage::save(&snap, false)
                                    })
                                    .map_err(|e| e.to_string()),
                            }
                        };
                        status = Some(match outcome {
                            Ok(()) => format!("'{target}' -> '{new_name}'."),
                            Err(e) => format!("could not complete: {e}"),
                        });
                        sessions = state::all()?;
                        clamp_selection(&mut list_state, sessions.len());
                        mode = Mode::Normal;
                    }
                    KeyCode::Backspace => {
                        buffer.pop();
                    }
                    KeyCode::Char(c) => buffer.push(c),
                    _ => {}
                },
                Mode::Confirm { target, .. } => match key.code {
                    KeyCode::Char('o') => {
                        let name = target.clone();
                        break PickerResult::Open(name);
                    }
                    KeyCode::Char('d') => {
                        let name = target.clone();
                        match storage::delete(&name) {
                            Ok(()) => status = Some(format!("deleted '{name}'.")),
                            Err(e) => status = Some(format!("could not delete '{name}': {e}")),
                        }
                        sessions = state::all()?;
                        clamp_selection(&mut list_state, sessions.len());
                        mode = Mode::Normal;
                    }
                    KeyCode::Char('c') | KeyCode::Esc => mode = Mode::Normal,
                    _ => {}
                },
                Mode::Message { .. } => {
                    mode = Mode::Normal;
                }
            }
        }
    };

    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(result)
}

/// Saves the tmux session sess itself is currently running inside, if any —
/// the same default target as plain `sess save`.
fn save_current() -> String {
    let Ok(session_name) = capture::current_session_name() else {
        return "can't save: not inside a tmux session.".to_string();
    };
    match capture::capture_session(&session_name) {
        Ok(snapshot) => match storage::save(&snapshot, true) {
            Ok(()) => format!("saved '{session_name}'."),
            Err(e) => format!("could not save '{session_name}': {e}"),
        },
        Err(e) => format!("could not capture '{session_name}': {e}"),
    }
}

fn selected<'a>(sessions: &'a [SessionState], state: &ListState) -> Option<&'a SessionState> {
    state.selected().and_then(|i| sessions.get(i))
}

fn move_selection(state: &mut ListState, len: usize, delta: i32) {
    if len == 0 {
        return;
    }
    let current = state.selected().unwrap_or(0) as i32;
    let next = (current + delta).rem_euclid(len as i32);
    state.select(Some(next as usize));
}

fn clamp_selection(state: &mut ListState, len: usize) {
    if len == 0 {
        state.select(None);
    } else {
        let current = state.selected().unwrap_or(0).min(len - 1);
        state.select(Some(current));
    }
}

fn draw_summary(f: &mut ratatui::Frame, area: Rect, sessions: &[SessionState], total_bytes: u64) {
    let running = sessions
        .iter()
        .filter(|s| s.kind == StateKind::Running)
        .count();
    let text = format!(
        " sessions: {}   running: {running}   total space: {} ",
        sessions.len(),
        storage::human_size(total_bytes)
    );
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

fn state_color(kind: StateKind) -> Color {
    match kind {
        StateKind::Running => Color::Green,
        StateKind::Saved => Color::Gray,
        StateKind::Stale => Color::Yellow,
        StateKind::Broken => Color::Red,
    }
}

fn draw_list(
    f: &mut ratatui::Frame,
    area: Rect,
    sessions: &[SessionState],
    max_bytes: u64,
    state: &mut ListState,
) {
    if sessions.is_empty() {
        let p = Paragraph::new("no sessions yet. `sess start <name>` to create one.")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(p, area);
        return;
    }

    let items: Vec<ListItem> = sessions
        .iter()
        .map(|s| {
            let bytes = s.size_bytes.unwrap_or(0);
            let bar = if s.size_bytes.is_some() {
                size_bar(bytes, max_bytes)
            } else {
                "-".repeat(BAR_WIDTH)
            };
            let size_text = s
                .size_bytes
                .map(storage::human_size)
                .unwrap_or_else(|| "-".to_string());
            let panes_text = s
                .pane_count
                .map(|n| format!("{n:>2}p"))
                .unwrap_or_else(|| "  -".to_string());

            let line = Line::from(vec![
                Span::styled(
                    format!("{:<8}", s.kind.to_string()),
                    Style::default()
                        .fg(state_color(s.kind))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{:<20}", s.name),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!("{panes_text}  "), Style::default().fg(Color::Gray)),
                Span::styled(bar, Style::default().fg(Color::DarkGray)),
                Span::raw(format!("  {:>8}", size_text)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");

    f.render_stateful_widget(list, area, state);
}

fn draw_footer(f: &mut ratatui::Frame, area: Rect, mode: &Mode, status: &Option<String>) {
    let line = match mode {
        Mode::Normal => {
            if let Some(msg) = status {
                Line::from(Span::styled(msg.clone(), Style::default().fg(Color::Cyan)))
            } else {
                Line::from(vec![
                    Span::styled("enter", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" open   "),
                    Span::styled("d", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" delete   "),
                    Span::styled("r", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" rename   "),
                    Span::styled("c", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" clone   "),
                    Span::styled("s", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" save current   "),
                    Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" quit"),
                ])
            }
        }
        Mode::Prompt {
            kind,
            target,
            buffer,
        } => {
            let label = match kind {
                PromptKind::Rename => "rename",
                PromptKind::Duplicate => "clone",
            };
            Line::from(vec![
                Span::styled(
                    format!("{label} '{target}' to: "),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    format!("{buffer}_"),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
            ])
        }
        Mode::Confirm { target, message } => Line::from(vec![Span::styled(
            format!("'{target}' is stale: {message}  [o]pen anyway  [d]elete  [c]ancel"),
            Style::default().fg(Color::Yellow),
        )]),
        Mode::Message { text } => Line::from(vec![Span::styled(
            format!("{text}  (press any key)"),
            Style::default().fg(Color::Red),
        )]),
    };

    f.render_widget(Paragraph::new(line), area);
}
