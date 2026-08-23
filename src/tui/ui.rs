//! Rendering for the TUI. Currently a placeholder screen; Task 6 will add
//! the split-pane layout and attribute inspector.

use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use super::App;

pub fn draw(frame: &mut Frame, app: &App) {
    let area: Rect = frame.area();
    let block = Block::default().title("BlakeSync").borders(Borders::ALL);
    let text = format!(
        "{} entries loaded. Press 'q' to quit.",
        app.entries.len()
    );
    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
}
