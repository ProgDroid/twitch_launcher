use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
use std::time::{Duration, Instant};
use tokio::{
    spawn,
    sync::mpsc::{unbounded_channel, UnboundedReceiver},
};

#[derive(Clone, Copy, Debug)]
pub enum Event {
    Tick,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
}

pub struct Handler {
    receiver: UnboundedReceiver<Event>,
}

impl Handler {
    #[allow(clippy::shadow_reuse, clippy::expect_used)]
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (sender, receiver) = unbounded_channel();

        spawn(async move {
            let mut last_tick = Instant::now();
            loop {
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or(tick_rate);

                if event::poll(timeout).expect("no events available") {
                    let result = match event::read().expect("unable to read event") {
                        CrosstermEvent::Key(e) => sender.send(Event::Key(e)),
                        CrosstermEvent::Mouse(e) => sender.send(Event::Mouse(e)),
                        CrosstermEvent::Resize(w, h) => sender.send(Event::Resize(w, h)),
                    };

                    if result.is_err() {
                        break;
                    }
                }

                if last_tick.elapsed() >= tick_rate {
                    let result = sender.send(Event::Tick);

                    if result.is_err() {
                        break;
                    }

                    last_tick = Instant::now();
                }
            }
        });

        Self { receiver }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.receiver.recv().await
    }
}
