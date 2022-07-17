use crate::app::{App, Result};
use crate::event::Handler;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use std::io;
use tui::backend::Backend;
use tui::Terminal;

#[derive(Debug)]
pub struct Tui<B: Backend> {
    terminal: Terminal<B>,
    pub events: Handler,
}

impl<B: Backend> Tui<B> {
    pub fn new(terminal: Terminal<B>, events: Handler) -> Self {
        Self { terminal, events }
    }

    pub fn init(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stderr(), EnterAlternateScreen, EnableMouseCapture)?;
        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        Ok(())
    }

    pub fn draw(&mut self, app: &mut App) -> Result<()> {
        self.terminal.draw(|frame| app.render(frame))?;
        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        terminal::disable_raw_mode()?;
        crossterm::execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture)?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}
