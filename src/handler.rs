use crate::{
    app::{App, AppResult},
    channel::{Channel, ChannelStatus},
    panel::{HomePanel, Panel},
    popup::Popup,
    state::{Event, State, StateMachine},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub type KeyBindFn = fn(KeyEvent, &mut App) -> AppResult<()>;

pub fn keybinds_startup(key_event: KeyEvent) -> Option<KeyBindFn> {
    match key_event.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => Some(stop_app),
        KeyCode::Char('c') | KeyCode::Char('C') if key_event.modifiers == KeyModifiers::CONTROL => {
            Some(stop_app)
        }
        _ => None,
    }
}

pub fn keybinds_home(key_event: KeyEvent) -> Option<KeyBindFn> {
    match key_event.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => Some(stop_app),
        KeyCode::Char('c') | KeyCode::Char('C') if key_event.modifiers == KeyModifiers::CONTROL => {
            Some(stop_app)
        }
        KeyCode::Tab => Some(tab_right),
        KeyCode::BackTab => Some(tab_left),
        KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Down => Some(highlight_down),
        KeyCode::Char('w') | KeyCode::Char('W') | KeyCode::Up => Some(highlight_up),
        KeyCode::Enter | KeyCode::Char(' ') => Some(select),
        KeyCode::Char('a') | KeyCode::Char('A') | KeyCode::Left => Some(panel_left),
        KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Right => Some(panel_right),
        _ => None,
    }
}

pub fn keybinds_exit(_: KeyEvent) -> Option<KeyBindFn> {
    None
}

pub fn keybinds_typing(key_event: KeyEvent) -> Option<KeyBindFn> {
    match key_event.code {
        KeyCode::Char('c') | KeyCode::Char('C') if key_event.modifiers == KeyModifiers::CONTROL => {
            Some(stop_app)
        }
        KeyCode::Esc => Some(stop_typing),
        KeyCode::Enter => Some(submit_search),
        KeyCode::Backspace => Some(remove_from_search_input),
        _ => Some(add_to_search_input),
    }
}

pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match app.state.keybinds(key_event) {
        Some(function) => (function)(key_event, app),
        None => Ok(()),
    }
}

fn stop_app(_: KeyEvent, app: &mut App) -> AppResult<()> {
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
        StateMachine::Home {
            channels,
            channel_highlight,
            focused_panel,
            ref mut typing,
            ..
        } => match focused_panel {
            HomePanel::Favourites => {
                app.events.push_back(Event::ChannelSelected(
                    channels[*channel_highlight].clone(),
                    true,
                ));
            }
            HomePanel::Search => {
                *typing = true;
            }
        },
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

fn stop_typing(_: KeyEvent, app: &mut App) -> AppResult<()> {
    match &mut app.state {
        StateMachine::Home { typing, .. } => {
            *typing = false;
        }
        _ => {}
    }

    Ok(())
}

fn submit_search(_: KeyEvent, app: &mut App) -> AppResult<()> {
    match &mut app.state {
        StateMachine::Home {
            typing,
            search_input,
            channels,
            ..
        } => {
            *typing = false;

            let handle: String = search_input.iter().collect();

            // TODO allow setting to determine whether to launch on browser or locally
            if let Some(index) = channels.iter().position(|channel| channel.handle == handle) {
                app.events.push_back(Event::ChannelSelected(
                    channels[index].clone(),
                    true, /* chat */
                ));
            } else {
                let channel = Channel {
                    friendly_name: String::new(),
                    handle,
                    status: ChannelStatus::Unknown,
                };

                app.events
                    .push_back(Event::ChannelSelected(channel, true /* chat */));
            }
        }
        _ => {}
    }

    Ok(())
}

fn remove_from_search_input(_: KeyEvent, app: &mut App) -> AppResult<()> {
    match &mut app.state {
        StateMachine::Home {
            typing,
            search_input,
            ..
        } => {
            if !*typing {
                return Ok(());
            }

            search_input.pop();
        }
        _ => {}
    }

    Ok(())
}

fn add_to_search_input(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match &mut app.state {
        StateMachine::Home {
            typing,
            search_input,
            ..
        } => {
            if !*typing {
                return Ok(());
            }

            if let KeyCode::Char(c) = key_event.code {
                search_input.push(c);
            }
        }
        _ => {}
    }

    Ok(())
}
