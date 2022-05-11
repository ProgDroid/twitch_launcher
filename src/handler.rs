use crate::{
    app::{App, AppResult},
    panel::{HomePanel, Panel},
    popup::Popup,
    state::{Event, State, StateMachine},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub type KeyBindFn = fn(KeyEvent, &mut App) -> AppResult<()>;

pub fn keybinds_startup(key_event: KeyEvent) -> Option<KeyBindFn> {
    match key_event.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => Some(stop_app),
        KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                Some(stop_app)
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn keybinds_home(key_event: KeyEvent) -> Option<KeyBindFn> {
    match key_event.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => Some(stop_app),
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                Some(stop_app)
            } else {
                None
            }
        }
        KeyCode::Tab => Some(tab_right),
        KeyCode::BackTab => Some(tab_left),
        KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Down => Some(highlight_down),
        KeyCode::Char('w') | KeyCode::Char('W') | KeyCode::Up => Some(highlight_up),
        KeyCode::Enter | KeyCode::Char(' ') => Some(select),
        KeyCode::Char('p') => Some(test_popup),
        KeyCode::Char('a') | KeyCode::Char('A') | KeyCode::Left => Some(panel_left),
        KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Right => Some(panel_right),
        _ => None,
    }
}

pub fn keybinds_exit(_: KeyEvent) -> Option<KeyBindFn> {
    None
}

pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match app.state.keybinds(key_event) {
        Some(function) => (function)(key_event, app),
        None => Ok(()),
    }
}

fn stop_app(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers != KeyModifiers::CONTROL {
                return Ok(());
            }
        }
        _ => {}
    }

    app.events.push_back(Event::Exited);
    Ok(())
}

fn tab_right(_: KeyEvent, app: &mut App) -> AppResult<()> {
    match app.state {
        StateMachine::Home {
            ref mut tab,
            tab_titles,
            ..
        } => *tab = (*tab + 1) % tab_titles.len(),
        _ => {}
    }

    Ok(())
}

fn tab_left(_: KeyEvent, app: &mut App) -> AppResult<()> {
    match app.state {
        StateMachine::Home {
            ref mut tab,
            tab_titles,
            ..
        } => {
            if *tab == 0 {
                *tab = tab_titles.len() - 1;
                return Ok(());
            }

            *tab -= 1;
        }
        _ => {}
    }

    Ok(())
}

// ? TODO Maybe add CTRL modifier to go all the way to the bottom?
fn highlight_down(_: KeyEvent, app: &mut App) -> AppResult<()> {
    match &mut app.state {
        StateMachine::Home {
            ref mut channel_highlight,
            channels,
            focused_panel,
            ..
        } => {
            if *focused_panel == HomePanel::Favourites {
                *channel_highlight = (*channel_highlight + 1) % channels.len()
            }
        }
        _ => {}
    }

    Ok(())
}

// ? TODO Should these be made more generic? Sub function to do calculation and just passing relevant things in
fn highlight_up(_: KeyEvent, app: &mut App) -> AppResult<()> {
    match &mut app.state {
        StateMachine::Home {
            ref mut channel_highlight,
            channels,
            focused_panel,
            ..
        } => {
            if *focused_panel == HomePanel::Favourites {
                if *channel_highlight == 0 {
                    *channel_highlight = channels.len() - 1;
                    return Ok(());
                }

                *channel_highlight -= 1;
            }
        }
        _ => {}
    }

    Ok(())
}

fn select(_: KeyEvent, app: &mut App) -> AppResult<()> {
    match &mut app.state {
        StateMachine::Home { focused_panel, .. } => match focused_panel {
            HomePanel::Favourites => {
                app.events.push_back(Event::ChannelSelected);
            }
            HomePanel::Search => {}
        },
        _ => {}
    }

    Ok(())
}

fn test_popup(_: KeyEvent, app: &mut App) -> AppResult<()> {
    match app.state {
        StateMachine::Home { ref mut popup, .. } => {
            *popup = Some(Popup {
                title: String::from("Test"),
                message: String::from("This is a test popup"),
            });
        }
        _ => {}
    }

    Ok(())
}

fn panel_left(_: KeyEvent, app: &mut App) -> AppResult<()> {
    match &mut app.state {
        StateMachine::Home { focused_panel, .. } => {
            *focused_panel = focused_panel.left();
        }
        _ => {}
    }

    Ok(())
}

fn panel_right(_: KeyEvent, app: &mut App) -> AppResult<()> {
    match &mut app.state {
        StateMachine::Home { focused_panel, .. } => {
            *focused_panel = focused_panel.right();
        }
        _ => {}
    }

    Ok(())
}
