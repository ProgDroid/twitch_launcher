#![crate_type = "bin"]

mod app;
mod input_mapping;
mod terminal;

use crate::{
    app::App,
    terminal::{display::Tui, event::Event},
};
use std::io;
use tui::backend::CrosstermBackend;

// TODO allow setting?
const TICK_INTERVAL: u64 = 250;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let mut app = App::new().await;

    let backend = CrosstermBackend::new(io::stderr());
    let mut tui = Tui::new(backend, TICK_INTERVAL)?;

    tui.init()?;

    while app.is_running() {
        tui.draw(&mut app)?;

        match tui.events.next().await {
            Some(Event::Tick) => app.tick().await,
            Some(Event::Key(key_event)) => app.handle_input(key_event),
            Some(Event::Mouse(_) | Event::Resize(_, _)) | None => {}
        }
    }

    tui.exit()?;
    Ok(())
}
