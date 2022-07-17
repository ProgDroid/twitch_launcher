#![crate_type = "bin"]
#![crate_name = "twitch_launcher"]

use std::io;
use tui::backend::CrosstermBackend;
use tui::Terminal;
use twitch_launcher::app::{App, Result};
use twitch_launcher::event::{Event, EventHandler};
use twitch_launcher::handler::handle_key_events;
use twitch_launcher::tui::Tui;

const TICK_INTERVAL: u64 = 250;

#[tokio::main]
async fn main() -> Result<()> {
    let mut app = App::new(TICK_INTERVAL).await;

    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(TICK_INTERVAL);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    while app.running {
        tui.draw(&mut app)?;

        match tui.events.next()? {
            Event::Tick => app.tick().await,
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    tui.exit()?;
    Ok(())
}
