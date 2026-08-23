//! Rendering for the TUI: a split-pane layout with a file list on the left
//! and an attribute inspector (size, mtime, hash) for the selected file on
//! the right. Keybindings and tagging (Task 7) will build on top of this.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::engine::{CompareStatus, ComparisonEntry};

use super::App;

pub fn draw(frame: &mut Frame, app: &App) {
    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(frame.area());

    draw_file_list(frame, panes[0], app);
    draw_inspector(frame, panes[1], app);
}

fn draw_file_list(frame: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app.entries.iter().map(|entry| ListItem::new(list_row(entry))).collect();

    let list = List::new(items)
        .block(Block::default().title("Files").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    let mut state = ListState::default();
    if !app.entries.is_empty() {
        state.select(Some(app.selected));
    }

    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_inspector(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().title("Inspector").borders(Borders::ALL);
    let text = inspector_lines(app.entries.get(app.selected)).join("\n");
    frame.render_widget(Paragraph::new(text).block(block), area);
}

/// Formats a single row of the file list pane.
pub fn list_row(entry: &ComparisonEntry) -> String {
    format!("{:<11} {}", status_label(entry.status), entry.rel_path.display())
}

/// Short, fixed-width label for a comparison status.
pub fn status_label(status: CompareStatus) -> &'static str {
    match status {
        CompareStatus::SourceOnly => "SOURCE-ONLY",
        CompareStatus::DestOnly => "DEST-ONLY",
        CompareStatus::Same => "SAME",
        CompareStatus::Modified => "MODIFIED",
    }
}

/// Builds the lines of text shown in the attribute inspector for the
/// currently selected entry, if any.
pub fn inspector_lines(entry: Option<&ComparisonEntry>) -> Vec<String> {
    let Some(entry) = entry else {
        return vec!["No files to review.".to_string()];
    };

    let mut lines = vec![
        format!("Path:   {}", entry.rel_path.display()),
        format!("Status: {}", status_label(entry.status)),
    ];

    match &entry.source {
        Some(src) => {
            lines.push(format!("Source size:  {} bytes", src.size));
            lines.push(format!("Source mtime: {}", format_mtime(src.mtime)));
        }
        None => lines.push("Source: (missing)".to_string()),
    }

    match &entry.dest {
        Some(dst) => {
            lines.push(format!("Dest size:    {} bytes", dst.size));
            lines.push(format!("Dest mtime:   {}", format_mtime(dst.mtime)));
        }
        None => lines.push("Dest: (missing)".to_string()),
    }

    lines.push("BLAKE3 hash: press 'h' to compute".to_string());
    lines
}

fn format_mtime(time: std::time::SystemTime) -> String {
    match time.duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => format!("{}s since epoch", d.as_secs()),
        Err(_) => "before epoch".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::FileEntry;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use std::path::PathBuf;
    use std::time::SystemTime;

    fn sample_entry() -> ComparisonEntry {
        ComparisonEntry {
            rel_path: PathBuf::from("a.txt"),
            status: CompareStatus::Modified,
            source: Some(FileEntry {
                rel_path: PathBuf::from("a.txt"),
                size: 10,
                mtime: SystemTime::now(),
            }),
            dest: Some(FileEntry {
                rel_path: PathBuf::from("a.txt"),
                size: 12,
                mtime: SystemTime::now(),
            }),
        }
    }

    #[test]
    fn status_label_covers_all_variants() {
        assert_eq!(status_label(CompareStatus::SourceOnly), "SOURCE-ONLY");
        assert_eq!(status_label(CompareStatus::DestOnly), "DEST-ONLY");
        assert_eq!(status_label(CompareStatus::Same), "SAME");
        assert_eq!(status_label(CompareStatus::Modified), "MODIFIED");
    }

    #[test]
    fn inspector_lines_reports_no_selection() {
        assert_eq!(inspector_lines(None), vec!["No files to review.".to_string()]);
    }

    #[test]
    fn inspector_lines_reports_size_and_status_for_selection() {
        let entry = sample_entry();
        let lines = inspector_lines(Some(&entry));
        assert!(lines.iter().any(|l| l.contains("MODIFIED")));
        assert!(lines.iter().any(|l| l.contains("10 bytes")));
        assert!(lines.iter().any(|l| l.contains("12 bytes")));
    }

    #[test]
    fn draw_does_not_panic_for_empty_or_populated_state() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        let empty_app = App::new(vec![]);
        terminal.draw(|frame| draw(frame, &empty_app)).unwrap();

        let populated_app = App::new(vec![sample_entry()]);
        terminal.draw(|frame| draw(frame, &populated_app)).unwrap();
    }
}
