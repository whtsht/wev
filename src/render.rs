use crate::{dom::NodeType, style::StyledNode};
use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::{Buffer, CrosstermBackend, Rect, Terminal},
    widgets::{Block, Borders, Paragraph, Widget},
};
use std::io::{stdout, Result};

pub fn render(typed_node: &StyledNode, area: Rect, buf: &mut Buffer) {
    match typed_node.node_type {
        NodeType::Element(_) => {
            let block = Block::default().borders(Borders::ALL);
            typed_node
                .children
                .iter()
                .for_each(|child| render(child, block.inner(area), buf));
            block.render(area, buf);
        }
        NodeType::Text(text) => Paragraph::new(&*text.data).render(area, buf),
    }
}

pub fn start(typed_node: &StyledNode) -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    loop {
        terminal.draw(|frame| render(typed_node, frame.size(), frame.buffer_mut()))?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
