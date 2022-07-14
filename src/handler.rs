use crate::{
    app::{App, AppResult},
    channel::{Channel, ChannelStatus},
    keybind::{KeyBindFn, Keybind},
    panel::{HomePanel, Panel},
    state::{Event, State, StateMachine},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn keybinds_startup() -> Vec<Keybind> {
    quit_binds()
}

pub fn keybinds_home() -> Vec<Keybind> {
    let mut binds = quit_binds();

    binds.append(&mut tab_move_binds());

    binds.append(&mut highlight_move_binds());

    binds.append(&mut select_binds());

    binds.append(&mut panel_move_binds());

    binds
}

pub fn keybinds_exit() -> Vec<Keybind> {
    Vec::new()
}

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

pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    if let Some(keybind) = app
        .state
        .keybinds()
        .iter()
        .find(|&bind| bind.triggers.contains(&key_event))
    {
        return (keybind.action)(key_event, app);
    }

    match app.state {
        StateMachine::Home { typing, .. } if typing => {
            return add_to_search_input(key_event, app);
        }
        _ => return Ok(()),
    }
}

fn stop_app(_: KeyEvent, app: &mut App) -> AppResult<()> {
    app.events.push_back(Event::Exited);
    Ok(())
}

fn cycle_tabs(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    if get_tab_right_keys().contains(&key_event) {
        return tab_right(key_event, app);
    }

    if get_tab_left_keys().contains(&key_event) {
        return tab_left(key_event, app);
    }

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

fn cycle_highlights(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    if get_highlights_down_keys().contains(&key_event) {
        return highlight_down(key_event, app);
    }

    if get_highlights_up_keys().contains(&key_event) {
        return highlight_up(key_event, app);
    }

    Ok(())
}

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

fn cycle_panels(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    if get_panels_left_keys().contains(&key_event) {
        return panel_left(key_event, app);
    }

    if get_panels_right_keys().contains(&key_event) {
        return panel_right(key_event, app);
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
                    status: ChannelStatus::Unknown,
                };

                app.events
                    .push_back(Event::ChannelSelected(channel, true /* TODO chat */));
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
        triggers: triggers,
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
            triggers: triggers,
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
        triggers: triggers,
        action: cycle_panels,
    }]
}

pub fn function_to_string(function: KeyBindFn) -> String {
    let func: usize = function as usize;

    if func == stop_app as usize {
        return String::from("Exit");
    }

    if func == cycle_tabs as usize {
        return String::from("Cycle Tabs");
    }

    if func == cycle_highlights as usize {
        return String::from("Cycle List");
    }

    if func == select as usize {
        return String::from("Select Current");
    }

    if func == cycle_panels as usize {
        return String::from("Cycle Panels");
    }

    if func == stop_typing as usize {
        return String::from("Cancel");
    }

    if func == submit_search as usize {
        return String::from("Submit");
    }

    if func == remove_from_search_input as usize {
        return String::from("Delete");
    }

    return String::from("Unknown");
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
    let mut existing_highlight_bottom_keys = get_highlights_down_keys();

    for key_event in &mut existing_highlight_bottom_keys {
        key_event.modifiers = KeyModifiers::CONTROL;
    }

    existing_highlight_bottom_keys
}

fn get_highlights_top_keys() -> Vec<KeyEvent> {
    let mut existing_highlight_up_keys = get_highlights_up_keys();

    for key_event in &mut existing_highlight_up_keys {
        key_event.modifiers = KeyModifiers::CONTROL;
    }

    existing_highlight_up_keys
}

fn top_bottom_highlights(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    if get_highlights_bottom_keys().contains(&key_event) {
        return highlight_bottom(key_event, app);
    }

    if get_highlights_top_keys().contains(&key_event) {
        return highlight_top(key_event, app);
    }

    Ok(())
}

fn highlight_bottom(_: KeyEvent, app: &mut App) -> AppResult<()> {
    match &mut app.state {
        StateMachine::Home {
            ref mut channel_highlight,
            channels,
            focused_panel,
            ..
        } => {
            if *focused_panel == HomePanel::Favourites {
                *channel_highlight = channels.len() - 1;
            }
        }
        _ => {}
    }

    Ok(())
}

fn highlight_top(_: KeyEvent, app: &mut App) -> AppResult<()> {
    match &mut app.state {
        StateMachine::Home {
            ref mut channel_highlight,
            focused_panel,
            ..
        } => {
            if *focused_panel == HomePanel::Favourites {
                *channel_highlight = 0;
            }
        }
        _ => {}
    }

    Ok(())
}
