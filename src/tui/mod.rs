//! TUI entry point: terminal setup/teardown and the main event loop.

mod app;
mod ui;

pub use app::App;

use std::io::{self, Stdout};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::engine::ComparisonEntry;
use crate::executor;

/// Initializes the terminal, runs the review loop until the user quits,
/// then restores the terminal to its original state.
pub fn run(
    source_root: PathBuf,
    dest_root: PathBuf,
    entries: Vec<ComparisonEntry>,
    dry_run: bool,
) -> anyhow::Result<()> {
    let mut terminal = init_terminal()?;
    let mut app = App::new(source_root.clone(), dest_root.clone(), entries);

    let result = event_loop(&mut terminal, &mut app, &source_root, &dest_root, dry_run);

    restore_terminal(terminal)?;
    result
}

fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
    source_root: &Path,
    dest_root: &Path,
    dry_run: bool,
) -> anyhow::Result<()> {
    while !app.should_quit {
        terminal.draw(|frame| ui::draw(frame, app))?;

        if app.execute_requested {
            app.execute_requested = false;
            run_execution(terminal, app, source_root, dest_root, dry_run)?;
            continue;
        }

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                app.on_key(key);
            }
        }
    }
    Ok(())
}

/// Plans and applies the current tags, redrawing after every step so the
/// progress gauge animates, then logs each result and leaves a summary in
/// `app.status_message`.
fn run_execution(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
    source_root: &Path,
    dest_root: &Path,
    dry_run: bool,
) -> anyhow::Result<()> {
    let actions = executor::plan_actions(&app.entries, &app.tags);
    app.progress = Some((0, actions.len()));

    let results = executor::execute_plan(&actions, source_root, dest_root, dry_run, |done, total| {
        app.progress = Some((done, total));
        let _ = terminal.draw(|frame| ui::draw(frame, app));
    })?;

    for result in &results {
        log_result(result, dry_run);
    }

    let applied = results.iter().filter(|r| r.applied).count();
    app.progress = None;
    app.status_message = Some(format!(
        "{}{} of {} action(s) applied",
        if dry_run { "[dry-run] " } else { "" },
        applied,
        results.len()
    ));

    Ok(())
}

fn log_result(result: &executor::ActionResult, dry_run: bool) {
    if dry_run {
        log::info!("[dry-run] {}: {}", result.rel_path.display(), result.message);
    } else {
        log::info!("{}: {}", result.rel_path.display(), result.message);
    }
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
