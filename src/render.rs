use crate::layout::LayoutObject;
use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Widget},
};
use std::io::{stdout, Result};

pub fn render(object: &LayoutObject, area: Rect, buf: &mut Buffer) {
    match object {
        LayoutObject::Box {
            direction,
            children,
            border,
        } => {
            let border = if *border { Borders::ALL } else { Borders::NONE };
            let block = Block::new().borders(border);

            let constraints = vec![Constraint::Length(5); children.len()];
            let layout = Layout::default()
                .direction(*direction)
                .constraints(constraints)
                .split(block.inner(area));

            children.iter().enumerate().for_each(|(idx, child)| {
                render(child, layout[idx], buf);
            });
            block.render(area, buf);
        }
        LayoutObject::Text(text) => Paragraph::new(*text).render(area, buf),
    }
}

pub fn start(object: &LayoutObject) -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    loop {
        terminal.draw(|frame| render(object, frame.size(), frame.buffer_mut()))?;

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
