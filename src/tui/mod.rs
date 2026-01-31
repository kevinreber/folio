use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs, Wrap},
    Frame, Terminal,
};
use std::io;

use crate::db::Database;
use crate::types::{Activity, Importance};

/// Application state for TUI
pub struct App {
    /// Current tab index
    pub current_tab: usize,
    /// Activities list
    pub activities: Vec<Activity>,
    /// List state for activities
    pub list_state: ListState,
    /// Whether to quit
    pub should_quit: bool,
    /// Status message
    pub status_message: Option<String>,
    /// Search query
    pub search_query: String,
    /// Filtered activities (for search)
    pub filtered_activities: Vec<usize>,
    /// Help visible
    pub show_help: bool,
}

impl App {
    pub fn new(db: &Database) -> Result<Self> {
        let activities = db.list_activities(Some(100))?;
        let filtered_activities: Vec<usize> = (0..activities.len()).collect();

        let mut list_state = ListState::default();
        if !activities.is_empty() {
            list_state.select(Some(0));
        }

        Ok(Self {
            current_tab: 0,
            activities,
            list_state,
            should_quit: false,
            status_message: None,
            search_query: String::new(),
            filtered_activities,
            show_help: false,
        })
    }

    pub fn selected_activity(&self) -> Option<&Activity> {
        self.list_state
            .selected()
            .and_then(|i| self.filtered_activities.get(i))
            .and_then(|&idx| self.activities.get(idx))
    }

    pub fn next(&mut self) {
        if self.filtered_activities.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.filtered_activities.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.filtered_activities.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_activities.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn next_tab(&mut self) {
        self.current_tab = (self.current_tab + 1) % 3;
    }

    pub fn previous_tab(&mut self) {
        if self.current_tab == 0 {
            self.current_tab = 2;
        } else {
            self.current_tab -= 1;
        }
    }

    pub fn update_filter(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_activities = (0..self.activities.len()).collect();
        } else {
            let query = self.search_query.to_lowercase();
            self.filtered_activities = self.activities
                .iter()
                .enumerate()
                .filter(|(_, a)| {
                    a.title.to_lowercase().contains(&query)
                        || a.project.as_ref().map(|p| p.to_lowercase().contains(&query)).unwrap_or(false)
                        || a.description.as_ref().map(|d| d.to_lowercase().contains(&query)).unwrap_or(false)
                })
                .map(|(i, _)| i)
                .collect();
        }

        // Reset selection if needed
        if self.list_state.selected().map(|i| i >= self.filtered_activities.len()).unwrap_or(true) {
            if !self.filtered_activities.is_empty() {
                self.list_state.select(Some(0));
            } else {
                self.list_state.select(None);
            }
        }
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some(msg.into());
    }
}

/// Run the TUI application
pub fn run() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let db = Database::open()?;
    let mut app = App::new(&db)?;

    // Run main loop
    let res = run_app(&mut terminal, &mut app, &db);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    db: &Database,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            if app.show_help {
                app.show_help = false;
                continue;
            }

            match key.code {
                KeyCode::Char('q') => app.should_quit = true,
                KeyCode::Char('?') => app.show_help = true,
                KeyCode::Tab => app.next_tab(),
                KeyCode::BackTab => app.previous_tab(),
                KeyCode::Down | KeyCode::Char('j') => app.next(),
                KeyCode::Up | KeyCode::Char('k') => app.previous(),
                KeyCode::Char('/') => {
                    // Enter search mode - simplified for now
                    app.search_query.clear();
                    app.set_status("Search: type to filter");
                }
                KeyCode::Esc => {
                    app.search_query.clear();
                    app.update_filter();
                    app.status_message = None;
                }
                KeyCode::Char('d') => {
                    // Delete selected
                    if let Some(activity) = app.selected_activity() {
                        let id = activity.id.clone();
                        if db.delete_activity(&id).is_ok() {
                            app.set_status(format!("Deleted activity {}", &id[..8]));
                            app.activities = db.list_activities(Some(100)).unwrap_or_default();
                            app.update_filter();
                        }
                    }
                }
                KeyCode::Char('r') => {
                    // Refresh
                    app.activities = db.list_activities(Some(100)).unwrap_or_default();
                    app.update_filter();
                    app.set_status("Refreshed");
                }
                KeyCode::Char(c) => {
                    // Add to search query
                    app.search_query.push(c);
                    app.update_filter();
                }
                KeyCode::Backspace => {
                    app.search_query.pop();
                    app.update_filter();
                }
                _ => {}
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Tabs
            Constraint::Min(0),     // Content
            Constraint::Length(3),  // Status bar
        ])
        .split(f.size());

    // Tabs
    let tab_titles = ["Activities", "Accomplishments", "Stats"];
    let tabs = Tabs::new(tab_titles.iter().map(|t| Line::from(*t)).collect::<Vec<_>>())
        .block(Block::default().borders(Borders::ALL).title("Folio"))
        .select(app.current_tab)
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    f.render_widget(tabs, chunks[0]);

    // Content based on tab
    match app.current_tab {
        0 => render_activities(f, app, chunks[1]),
        1 => render_accomplishments(f, app, chunks[1]),
        2 => render_stats(f, app, chunks[1]),
        _ => {}
    }

    // Status bar
    let status = if !app.search_query.is_empty() {
        format!("Search: {} | {} results", app.search_query, app.filtered_activities.len())
    } else if let Some(ref msg) = app.status_message {
        msg.clone()
    } else {
        "Press ? for help | q to quit | Tab to switch views".to_string()
    };

    let status_bar = Paragraph::new(status)
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status_bar, chunks[2]);

    // Help overlay
    if app.show_help {
        render_help(f);
    }
}

fn render_activities(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Activity list
    let items: Vec<ListItem> = app.filtered_activities
        .iter()
        .filter_map(|&i| app.activities.get(i))
        .map(|a| {
            let importance_color = match a.importance {
                Importance::High => Color::Red,
                Importance::Medium => Color::Yellow,
                Importance::Low => Color::Gray,
            };

            let content = Line::from(vec![
                Span::styled("● ", Style::default().fg(importance_color)),
                Span::raw(&a.title),
            ]);

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Activities"))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, chunks[0], &mut app.list_state.clone());

    // Detail view
    let detail = if let Some(activity) = app.selected_activity() {
        let mut lines = vec![
            Line::from(vec![
                Span::styled("Title: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&activity.title),
            ]),
            Line::from(vec![
                Span::styled("Date: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(activity.timestamp.format("%Y-%m-%d %H:%M").to_string()),
            ]),
            Line::from(vec![
                Span::styled("Source: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(activity.source.to_string()),
            ]),
            Line::from(vec![
                Span::styled("Importance: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(activity.importance.to_string()),
            ]),
        ];

        if let Some(ref project) = activity.project {
            lines.push(Line::from(vec![
                Span::styled("Project: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(project),
            ]));
        }

        if let Some(ref desc) = activity.description {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("Description:", Style::default().add_modifier(Modifier::BOLD))));
            lines.push(Line::from(desc.as_str()));
        }

        if let Some(impact) = activity.metadata.get("impact").and_then(|v| v.as_str()) {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("Impact: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(impact),
            ]));
        }

        Text::from(lines)
    } else {
        Text::from("Select an activity to view details")
    };

    let detail_widget = Paragraph::new(detail)
        .block(Block::default().borders(Borders::ALL).title("Details"))
        .wrap(Wrap { trim: true });

    f.render_widget(detail_widget, chunks[1]);
}

fn render_accomplishments(f: &mut Frame, _app: &App, area: Rect) {
    let text = Paragraph::new("Accomplishments view - Coming soon!\n\nThis will show your polished accomplishments with STAR format stories.")
        .block(Block::default().borders(Borders::ALL).title("Accomplishments"))
        .wrap(Wrap { trim: true });
    f.render_widget(text, area);
}

fn render_stats(f: &mut Frame, app: &App, area: Rect) {
    let total = app.activities.len();
    let high = app.activities.iter().filter(|a| matches!(a.importance, Importance::High)).count();
    let medium = app.activities.iter().filter(|a| matches!(a.importance, Importance::Medium)).count();
    let low = app.activities.iter().filter(|a| matches!(a.importance, Importance::Low)).count();

    let mut projects: std::collections::HashSet<_> = app.activities
        .iter()
        .filter_map(|a| a.project.clone())
        .collect();

    let stats_text = format!(
        "Total Activities: {}\n\n\
         By Importance:\n  High: {}\n  Medium: {}\n  Low: {}\n\n\
         Projects: {}",
        total, high, medium, low, projects.len()
    );

    let text = Paragraph::new(stats_text)
        .block(Block::default().borders(Borders::ALL).title("Statistics"))
        .wrap(Wrap { trim: true });
    f.render_widget(text, area);
}

fn render_help(f: &mut Frame) {
    let area = centered_rect(60, 60, f.size());

    let help_text = vec![
        "Keyboard Shortcuts",
        "",
        "Navigation:",
        "  j/↓     Move down",
        "  k/↑     Move up",
        "  Tab     Next tab",
        "  S-Tab   Previous tab",
        "",
        "Actions:",
        "  /       Search",
        "  d       Delete selected",
        "  r       Refresh",
        "  Esc     Clear search",
        "  q       Quit",
        "",
        "Press any key to close this help",
    ];

    let text = Paragraph::new(help_text.join("\n"))
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .style(Style::default().bg(Color::Black));

    f.render_widget(ratatui::widgets::Clear, area);
    f.render_widget(text, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
