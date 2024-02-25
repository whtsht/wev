use crate::layout::{LayoutObject, LayoutObjectType};
use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{Paragraph, Widget},
};
use std::io::{stdout, Result};

pub fn render(object: &LayoutObject, buf: &mut Buffer) {
    match &object.ty {
        LayoutObjectType::Texts(texts) => {
            texts
                .iter()
                .for_each(|t| Paragraph::new(t.data).render(t.area, buf));
        }
        LayoutObjectType::Block { children } => {
            children.iter().for_each(|n| render(n, buf));
        }
    }
}

pub fn start(object: &LayoutObject) -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    loop {
        terminal.draw(|frame| render(object, frame.buffer_mut()))?;

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
