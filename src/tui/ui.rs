//! Rendering for the TUI: a split-pane layout with a file list on the left
//! and an attribute inspector (size, mtime, hash, tag) for the selected
//! file on the right, plus a bottom status bar that doubles as a progress
//! gauge while the executor (Task 8/9) is running.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::engine::{CompareStatus, ComparisonEntry};

use super::app::Tag;
use super::App;

const HELP_TEXT: &str = "j/k move  s=keep source  d=keep dest  i=ignore  h=hash  Enter=execute  q=quit";

pub fn draw(frame: &mut Frame, app: &App) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(frame.area());

    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(rows[0]);

    draw_file_list(frame, panes[0], app);
    draw_inspector(frame, panes[1], app);
    draw_status_bar(frame, rows[1], app);
}

fn draw_file_list(frame: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .entries
        .iter()
        .map(|entry| ListItem::new(list_row(entry, app.tags.get(&entry.rel_path).copied())))
        .collect();

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
    let text = inspector_lines(app).join("\n");
    frame.render_widget(Paragraph::new(text).block(block), area);
}

/// Renders a live progress gauge while the executor is running, otherwise
/// falls back to the last status message (or a keybinding hint).
fn draw_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    if let Some((done, total)) = app.progress {
        let ratio = if total == 0 { 1.0 } else { done as f64 / total as f64 };
        let gauge = Gauge::default().ratio(ratio.clamp(0.0, 1.0)).label(format!("{done}/{total}"));
        frame.render_widget(gauge, area);
    } else {
        let text = app.status_message.clone().unwrap_or_else(|| HELP_TEXT.to_string());
        frame.render_widget(Paragraph::new(text), area);
    }
}

/// Formats a single row of the file list pane, prefixed with the file's
/// tag marker (`[S]`/`[D]`/`[X]`/`[ ]`).
pub fn list_row(entry: &ComparisonEntry, tag: Option<Tag>) -> String {
    format!(
        "[{}] {:<11} {}",
        tag_marker(tag),
        status_label(entry.status),
        entry.rel_path.display()
    )
}

fn tag_marker(tag: Option<Tag>) -> char {
    match tag {
        Some(Tag::KeepSource) => 'S',
        Some(Tag::KeepDest) => 'D',
        Some(Tag::Ignore) => 'X',
        None => ' ',
    }
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

/// Human-readable label for a tag, used in the attribute inspector.
pub fn tag_label(tag: Tag) -> &'static str {
    match tag {
        Tag::KeepSource => "KEEP SOURCE",
        Tag::KeepDest => "KEEP DEST",
        Tag::Ignore => "IGNORE",
    }
}

/// Builds the lines of text shown in the attribute inspector for the
/// currently selected entry, including its tag and any cached hash.
pub fn inspector_lines(app: &App) -> Vec<String> {
    let Some(entry) = app.entries.get(app.selected) else {
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

    match app.hash_cache.get(&entry.rel_path) {
        Some((source_hash, dest_hash)) => {
            lines.push(format!("Source hash: {}", format_hash(*source_hash)));
            lines.push(format!("Dest hash:   {}", format_hash(*dest_hash)));
        }
        None => lines.push("BLAKE3 hash: press 'h' to compute".to_string()),
    }

    match app.tags.get(&entry.rel_path) {
        Some(tag) => lines.push(format!("Tag: {}", tag_label(*tag))),
        None => lines.push("Tag: (untagged) - s=keep source, d=keep dest, i=ignore".to_string()),
    }

    lines
}

fn format_hash(hash: Option<blake3::Hash>) -> String {
    hash.map(|h| h.to_hex().to_string()).unwrap_or_else(|| "n/a".to_string())
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

    fn app_with(entries: Vec<ComparisonEntry>) -> App {
        App::new(PathBuf::from("/src"), PathBuf::from("/dst"), entries)
    }

    #[test]
    fn status_label_covers_all_variants() {
        assert_eq!(status_label(CompareStatus::SourceOnly), "SOURCE-ONLY");
        assert_eq!(status_label(CompareStatus::DestOnly), "DEST-ONLY");
        assert_eq!(status_label(CompareStatus::Same), "SAME");
        assert_eq!(status_label(CompareStatus::Modified), "MODIFIED");
    }

    #[test]
    fn tag_marker_reflects_tag_state() {
        assert_eq!(list_row(&sample_entry(), None).chars().nth(1), Some(' '));
        assert_eq!(list_row(&sample_entry(), Some(Tag::KeepSource)).chars().nth(1), Some('S'));
        assert_eq!(list_row(&sample_entry(), Some(Tag::KeepDest)).chars().nth(1), Some('D'));
        assert_eq!(list_row(&sample_entry(), Some(Tag::Ignore)).chars().nth(1), Some('X'));
    }

    #[test]
    fn inspector_lines_reports_no_selection() {
        assert_eq!(inspector_lines(&app_with(vec![])), vec!["No files to review.".to_string()]);
    }

    #[test]
    fn inspector_lines_reports_size_status_and_untagged_hint() {
        let app = app_with(vec![sample_entry()]);
        let lines = inspector_lines(&app);
        assert!(lines.iter().any(|l| l.contains("MODIFIED")));
        assert!(lines.iter().any(|l| l.contains("10 bytes")));
        assert!(lines.iter().any(|l| l.contains("12 bytes")));
        assert!(lines.iter().any(|l| l.contains("press 'h' to compute")));
        assert!(lines.iter().any(|l| l.contains("untagged")));
    }

    #[test]
    fn inspector_lines_reports_tag_once_assigned() {
        let mut app = app_with(vec![sample_entry()]);
        app.tags.insert(PathBuf::from("a.txt"), Tag::KeepDest);
        let lines = inspector_lines(&app);
        assert!(lines.iter().any(|l| l.contains("KEEP DEST")));
    }

    #[test]
    fn draw_does_not_panic_for_empty_or_populated_state() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        let empty_app = app_with(vec![]);
        terminal.draw(|frame| draw(frame, &empty_app)).unwrap();

        let populated_app = app_with(vec![sample_entry()]);
        terminal.draw(|frame| draw(frame, &populated_app)).unwrap();
    }
}
