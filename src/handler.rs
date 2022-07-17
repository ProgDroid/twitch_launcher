use crate::{
    app::{App, Result},
    channel::{Channel, Status},
    keybind::{KeyBindFn, Keybind},
    panel::{Home, Panel},
    state::{Event, State, StateMachine},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[must_use]
pub fn keybinds_startup() -> Vec<Keybind> {
    quit_binds()
}

#[must_use]
pub fn keybinds_home() -> Vec<Keybind> {
    let mut binds = quit_binds();

    binds.append(&mut tab_move_binds());

    binds.append(&mut highlight_move_binds());

    binds.append(&mut select_binds());

    binds.append(&mut panel_move_binds());

    binds
}

#[must_use]
#[inline]
pub const fn keybinds_exit() -> Vec<Keybind> {
    Vec::new()
}

#[must_use]
pub fn keybinds_typing() -> Vec<Keybind> {
    vec![
        Keybind {
            triggers: vec![
                KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
                KeyEvent::new(KeyCode::Char('C'), KeyModifiers::CONTROL),
            ],
            action: stop_app,
        },
        Keybind {
            triggers: vec![KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)],
            action: stop_typing,
        },
        Keybind {
            triggers: vec![KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)],
            action: submit_search,
        },
        Keybind {
            triggers: vec![KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)],
            action: remove_from_search_input,
        },
    ]
}

#[allow(clippy::wildcard_enum_match_arm)]
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> Result<()> {
    if let Some(keybind) = app
        .state
        .keybinds()
        .iter()
        .find(|&bind| bind.triggers.contains(&key_event))
    {
        return (keybind.action)(key_event, app);
    }

    match app.state {
        StateMachine::Home { typing, .. } if typing => add_to_search_input(key_event, app),
        _ => Ok(()),
    }
}

#[allow(clippy::unnecessary_wraps)]
fn stop_app(_: KeyEvent, app: &mut App) -> Result<()> {
    app.events.push_back(Event::Exited);
    Ok(())
}

// TODO custom type for tab/channel highlight?

#[allow(clippy::integer_arithmetic)]
const fn index_add(current_value: usize, size: usize) -> usize {
    (current_value + 1) % size
}

#[allow(clippy::integer_arithmetic)]
const fn index_subtract(current_value: usize, size: usize) -> usize {
    (current_value + size - 1) % size
}

fn cycle_tabs(key_event: KeyEvent, app: &mut App) -> Result<()> {
    if get_tab_right_keys().contains(&key_event) {
        return tab_right(key_event, app);
    }

    if get_tab_left_keys().contains(&key_event) {
        return tab_left(key_event, app);
    }

    Ok(())
}

#[allow(
    clippy::unnecessary_wraps,
    clippy::single_match,
    clippy::wildcard_enum_match_arm
)]
fn tab_right(_: KeyEvent, app: &mut App) -> Result<()> {
    match app.state {
        StateMachine::Home {
            ref mut tab,
            tab_titles,
            ..
        } => *tab = index_add(*tab, tab_titles.len()),
        _ => {}
    }

    Ok(())
}

#[allow(
    clippy::unnecessary_wraps,
    clippy::single_match,
    clippy::wildcard_enum_match_arm
)]
fn tab_left(_: KeyEvent, app: &mut App) -> Result<()> {
    match app.state {
        StateMachine::Home {
            ref mut tab,
            tab_titles,
            ..
        } => *tab = index_subtract(*tab, tab_titles.len()),
        _ => {}
    }

    Ok(())
}

fn cycle_highlights(key_event: KeyEvent, app: &mut App) -> Result<()> {
    if get_highlights_down_keys().contains(&key_event) {
        return highlight_down(key_event, app);
    }

    if get_highlights_up_keys().contains(&key_event) {
        return highlight_up(key_event, app);
    }

    Ok(())
}

#[allow(
    clippy::pattern_type_mismatch,
    clippy::single_match,
    clippy::unnecessary_wraps,
    clippy::wildcard_enum_match_arm
)]
fn highlight_down(_: KeyEvent, app: &mut App) -> Result<()> {
    match &mut app.state {
        StateMachine::Home {
            ref mut channel_highlight,
            channels,
            focused_panel,
            ..
        } => {
            if *focused_panel == Home::Favourites {
                *channel_highlight = index_add(*channel_highlight, channels.len());
            }
        }
        _ => {}
    }

    Ok(())
}

#[allow(
    clippy::pattern_type_mismatch,
    clippy::single_match,
    clippy::unnecessary_wraps,
    clippy::wildcard_enum_match_arm
)]
fn highlight_up(_: KeyEvent, app: &mut App) -> Result<()> {
    match &mut app.state {
        StateMachine::Home {
            ref mut channel_highlight,
            channels,
            focused_panel,
            ..
        } => {
            if *focused_panel == Home::Favourites {
                *channel_highlight = index_subtract(*channel_highlight, channels.len());
            }
        }
        _ => {}
    }

    Ok(())
}

#[allow(
    clippy::pattern_type_mismatch,
    clippy::single_match,
    clippy::unnecessary_wraps,
    clippy::wildcard_enum_match_arm,
    clippy::indexing_slicing
)]
fn select(_: KeyEvent, app: &mut App) -> Result<()> {
    match &mut app.state {
        StateMachine::Home {
            channels,
            channel_highlight,
            focused_panel,
            ref mut typing,
            ..
        } => match focused_panel {
            Home::Favourites => {
                app.events.push_back(Event::ChannelSelected(
                    channels[*channel_highlight].clone(),
                    true,
                ));
            }
            Home::Search => {
                *typing = true;
            }
        },
        _ => {}
    }

    Ok(())
}

fn cycle_panels(key_event: KeyEvent, app: &mut App) -> Result<()> {
    if get_panels_left_keys().contains(&key_event) {
        return panel_left(key_event, app);
    }

    if get_panels_right_keys().contains(&key_event) {
        return panel_right(key_event, app);
    }

    Ok(())
}

#[allow(
    clippy::pattern_type_mismatch,
    clippy::single_match,
    clippy::unnecessary_wraps,
    clippy::wildcard_enum_match_arm
)]
fn panel_left(_: KeyEvent, app: &mut App) -> Result<()> {
    match &mut app.state {
        StateMachine::Home { focused_panel, .. } => {
            *focused_panel = focused_panel.left();
        }
        _ => {}
    }

    Ok(())
}

// TODO should there be a state function to handle these changes? or events??

#[allow(
    clippy::pattern_type_mismatch,
    clippy::single_match,
    clippy::unnecessary_wraps,
    clippy::wildcard_enum_match_arm
)]
fn panel_right(_: KeyEvent, app: &mut App) -> Result<()> {
    match &mut app.state {
        StateMachine::Home { focused_panel, .. } => {
            *focused_panel = focused_panel.right();
        }
        _ => {}
    }

    Ok(())
}

#[allow(
    clippy::pattern_type_mismatch,
    clippy::single_match,
    clippy::unnecessary_wraps,
    clippy::wildcard_enum_match_arm
)]
fn stop_typing(_: KeyEvent, app: &mut App) -> Result<()> {
    match &mut app.state {
        StateMachine::Home { typing, .. } => {
            *typing = false;
        }
        _ => {}
    }

    Ok(())
}

#[allow(
    clippy::pattern_type_mismatch,
    clippy::single_match,
    clippy::unnecessary_wraps,
    clippy::wildcard_enum_match_arm,
    clippy::indexing_slicing
)]
fn submit_search(_: KeyEvent, app: &mut App) -> Result<()> {
    match &mut app.state {
        StateMachine::Home {
            typing,
            search_input,
            channels,
            ..
        } => {
            if search_input.is_empty() {
                return Ok(());
            }

            *typing = false;

            let handle: String = search_input.iter().collect();

            // TODO allow setting to determine whether to launch on browser or locally
            if let Some(index) = channels.iter().position(|channel| channel.handle == handle) {
                app.events.push_back(Event::ChannelSelected(
                    channels[index].clone(),
                    true, /* TODO chat */
                ));
            } else {
                let channel = Channel {
                    friendly_name: String::new(),
                    handle,
                    status: Status::Unknown,
                };

                app.events
                    .push_back(Event::ChannelSelected(channel, true /* TODO chat */));
            }
        }
        _ => {}
    }

    Ok(())
}

// TODO add CTRL backspace?

#[allow(
    clippy::pattern_type_mismatch,
    clippy::single_match,
    clippy::unnecessary_wraps,
    clippy::wildcard_enum_match_arm
)]
fn remove_from_search_input(_: KeyEvent, app: &mut App) -> Result<()> {
    match app.state {
        StateMachine::Home {
            typing,
            ref mut search_input,
            ..
        } => {
            if typing {
                search_input.pop();
            }
        }
        _ => {}
    }

    Ok(())
}

#[allow(
    clippy::pattern_type_mismatch,
    clippy::single_match,
    clippy::unnecessary_wraps,
    clippy::wildcard_enum_match_arm
)]
fn add_to_search_input(key_event: KeyEvent, app: &mut App) -> Result<()> {
    match app.state {
        StateMachine::Home {
            typing,
            ref mut search_input,
            ..
        } => {
            if !typing {
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

fn quit_binds() -> Vec<Keybind> {
    vec![Keybind {
        triggers: vec![
            KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('Q'), KeyModifiers::SHIFT),
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
            KeyEvent::new(
                KeyCode::Char('C'),
                KeyModifiers::CONTROL | KeyModifiers::SHIFT,
            ),
        ],
        action: stop_app,
    }]
}

fn tab_move_binds() -> Vec<Keybind> {
    let mut triggers: Vec<KeyEvent> = get_tab_right_keys();

    triggers.append(&mut get_tab_left_keys());

    vec![Keybind {
        triggers,
        action: cycle_tabs,
    }]
}

fn highlight_move_binds() -> Vec<Keybind> {
    let mut triggers: Vec<KeyEvent> = get_highlights_down_keys();

    triggers.append(&mut get_highlights_up_keys());

    let mut top_bottom_triggers = get_highlights_bottom_keys();

    top_bottom_triggers.append(&mut get_highlights_top_keys());

    vec![
        Keybind {
            triggers,
            action: cycle_highlights,
        },
        Keybind {
            triggers: top_bottom_triggers,
            action: top_bottom_highlights,
        },
    ]
}

fn select_binds() -> Vec<Keybind> {
    vec![Keybind {
        triggers: vec![
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE),
        ],
        action: select,
    }]
}

fn panel_move_binds() -> Vec<Keybind> {
    let mut triggers: Vec<KeyEvent> = get_panels_left_keys();

    triggers.append(&mut get_panels_right_keys());

    vec![Keybind {
        triggers,
        action: cycle_panels,
    }]
}

#[allow(clippy::as_conversions, clippy::fn_to_numeric_cast_any)]
pub fn function_to_string(function: KeyBindFn) -> String {
    let func: usize = function as usize;

    match func {
        f if f == stop_app as usize => String::from("Exit"),
        f if f == cycle_tabs as usize => String::from("Cycle Tabs"),
        f if f == cycle_highlights as usize => String::from("Cycle List"),
        f if f == select as usize => String::from("Select Current"),
        f if f == cycle_panels as usize => String::from("Cycle Panels"),
        f if f == stop_typing as usize => String::from("Cancel"),
        f if f == submit_search as usize => String::from("Submit"),
        f if f == remove_from_search_input as usize => String::from("Delete"),
        f if f == top_bottom_highlights as usize => String::from("Go to List Top/Bottom"),
        _ => String::from("Unknown"),
    }
}

fn get_tab_right_keys() -> Vec<KeyEvent> {
    vec![KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)]
}

fn get_tab_left_keys() -> Vec<KeyEvent> {
    vec![KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT)]
}

fn get_highlights_down_keys() -> Vec<KeyEvent> {
    vec![
        KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE),
        KeyEvent::new(KeyCode::Char('S'), KeyModifiers::SHIFT),
        KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
    ]
}

fn get_highlights_up_keys() -> Vec<KeyEvent> {
    vec![
        KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE),
        KeyEvent::new(KeyCode::Char('W'), KeyModifiers::SHIFT),
        KeyEvent::new(KeyCode::Up, KeyModifiers::NONE),
    ]
}

fn get_panels_left_keys() -> Vec<KeyEvent> {
    vec![
        KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE),
        KeyEvent::new(KeyCode::Char('A'), KeyModifiers::SHIFT),
        KeyEvent::new(KeyCode::Left, KeyModifiers::NONE),
    ]
}

fn get_panels_right_keys() -> Vec<KeyEvent> {
    vec![
        KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE),
        KeyEvent::new(KeyCode::Char('D'), KeyModifiers::SHIFT),
        KeyEvent::new(KeyCode::Right, KeyModifiers::NONE),
    ]
}

fn get_highlights_bottom_keys() -> Vec<KeyEvent> {
    get_highlights_down_keys()
        .iter_mut()
        .map(|event| {
            event.modifiers = KeyModifiers::CONTROL;
            *event
        })
        .collect::<Vec<KeyEvent>>()
}

fn get_highlights_top_keys() -> Vec<KeyEvent> {
    get_highlights_up_keys()
        .iter_mut()
        .map(|event| {
            event.modifiers = KeyModifiers::CONTROL;
            *event
        })
        .collect::<Vec<KeyEvent>>()
}

fn top_bottom_highlights(key_event: KeyEvent, app: &mut App) -> Result<()> {
    if get_highlights_bottom_keys().contains(&key_event) {
        return highlight_bottom(key_event, app);
    }

    if get_highlights_top_keys().contains(&key_event) {
        return highlight_top(key_event, app);
    }

    Ok(())
}

#[allow(
    clippy::pattern_type_mismatch,
    clippy::single_match,
    clippy::unnecessary_wraps,
    clippy::wildcard_enum_match_arm,
    clippy::integer_arithmetic
)]
fn highlight_bottom(_: KeyEvent, app: &mut App) -> Result<()> {
    match &mut app.state {
        StateMachine::Home {
            ref mut channel_highlight,
            channels,
            focused_panel,
            ..
        } => {
            if *focused_panel == Home::Favourites {
                *channel_highlight = channels.len() - 1;
            }
        }
        _ => {}
    }

    Ok(())
}

#[allow(
    clippy::pattern_type_mismatch,
    clippy::single_match,
    clippy::unnecessary_wraps,
    clippy::wildcard_enum_match_arm
)]
fn highlight_top(_: KeyEvent, app: &mut App) -> Result<()> {
    match &mut app.state {
        StateMachine::Home {
            ref mut channel_highlight,
            focused_panel,
            ..
        } => {
            if *focused_panel == Home::Favourites {
                *channel_highlight = 0;
            }
        }
        _ => {}
    }

    Ok(())
}
