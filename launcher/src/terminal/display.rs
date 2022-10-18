use crate::app::App;
use crate::terminal::event::Handler;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use tui::{backend::Backend, Terminal};

pub struct Tui<B: Backend> {
    terminal: Terminal<B>,
    pub events: Handler,
}

impl<B: Backend> Tui<B> {
    pub fn new(backend: B, tick_interval: u64) -> io::Result<Self> {
        let terminal = Terminal::new(backend)?;
        let events = Handler::new(tick_interval);

        Ok(Self { terminal, events })
    }

    pub fn init(&mut self) -> Result<(), std::io::Error> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        Ok(())
    }

    pub fn draw(&mut self, app: &mut App) -> Result<(), std::io::Error> {
        self.terminal.draw(|frame| app.render(frame))?;
        Ok(())
    }

    pub fn exit(&mut self) -> Result<(), std::io::Error> {
        terminal::disable_raw_mode()?;
        crossterm::execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}
