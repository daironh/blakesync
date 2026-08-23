//! TUI entry point: terminal setup/teardown and the main event loop.

mod app;
mod ui;

pub use app::App;

use std::io::{self, Stdout};
use std::path::PathBuf;
use std::time::Duration;

use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::engine::ComparisonEntry;

/// Initializes the terminal, runs the review loop until the user quits,
/// then restores the terminal to its original state.
pub fn run(source_root: PathBuf, dest_root: PathBuf, entries: Vec<ComparisonEntry>) -> anyhow::Result<()> {
    let mut terminal = init_terminal()?;
    let mut app = App::new(source_root, dest_root, entries);

    let result = event_loop(&mut terminal, &mut app);

    restore_terminal(terminal)?;
    result
}

fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
) -> anyhow::Result<()> {
    while !app.should_quit {
        terminal.draw(|frame| ui::draw(frame, app))?;
        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                app.on_key(key);
            }
        }
    }
    Ok(())
}

fn init_terminal() -> anyhow::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal(mut terminal: Terminal<CrosstermBackend<Stdout>>) -> anyhow::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
