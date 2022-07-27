use crate::{
    render::common::{generate_background_widget, generate_keys_widget, generate_search_box},
    theme::{Elevation, Theme},
};
use input::keybind::KeyBind;
use std::fmt::Display;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    terminal::Frame,
    text::{Span, Spans},
    widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph},
};

pub const POPUP_PERCENTAGE_X: u16 = 60;
pub const POPUP_PERCENTAGE_Y: u16 = 20;

pub fn choice<B: Backend, T: Display + Clone>(
    theme: &Theme,
    frame: &mut Frame<'_, B>,
    keybinds: &[KeyBind<T>],
    title: &str,
    message: &str,
    selected: usize,
    options: &[String],
) {
    let area = self::popup(
        theme,
        frame,
        keybinds,
        title,
        message,
        generate_choice_popup_layout,
    );

    let mut list_state: ListState = ListState::default();
    list_state.select(Some(selected));

    frame.render_stateful_widget(
        generate_choice_popup_list(theme, options),
        area,
        &mut list_state,
    );
}

pub fn input<B: Backend, T: Display + Clone>(
    theme: &Theme,
    frame: &mut Frame<'_, B>,
    keybinds: &[KeyBind<T>],
    title: &str,
    message: &str,
    input: &[char],
    typing: bool,
) {
    let area = self::popup(
        theme,
        frame,
        keybinds,
        title,
        message,
        generate_input_popup_layout,
    );

    frame.render_widget(
        generate_search_box(
            theme,
            input,
            typing,
            true,
            String::from(""),
            theme.background.as_tui_colour(),
        ),
        area,
    );
}

pub fn timed_info<B: Backend, T: Display + Clone>(
    theme: &Theme,
    frame: &mut Frame<'_, B>,
    keybinds: &[KeyBind<T>],
    title: &str,
    message: &str,
    duration: u64,
    timer: u64,
) {
    let area = self::popup(
        theme,
        frame,
        keybinds,
        title,
        message,
        generate_timed_info_popup_layout,
    );

    frame.render_widget(
        generate_timed_popup_progress_bar(theme, duration, timer),
        area,
    );
}

#[allow(clippy::indexing_slicing)]
#[must_use]
fn popup<B: Backend, T: Display + Clone>(
    theme: &Theme,
    frame: &mut Frame<'_, B>,
    keybinds: &[KeyBind<T>],
    title: &str,
    message: &str,
    layout_function: fn(Rect) -> Vec<Rect>,
) -> Rect {
    let area = frame.size();

    let popup_area = generate_popup_layout(area);

    frame.render_widget(
        generate_background_widget(theme.background.as_tui_colour()),
        popup_area[1],
    );

    let popup_sections = layout_function(popup_area[1]);

    let paragraph = Paragraph::new(message)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.secondary.as_tui_colour())),
        )
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, popup_sections[0]);

    frame.render_widget(generate_keys_widget(theme, keybinds), popup_area[3]);

    popup_sections[1]
}

fn generate_choice_popup_layout(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            vec![
                Constraint::Percentage(67), // message
                Constraint::Percentage(33), // options
            ]
            .as_ref(),
        )
        .split(area)
}

fn generate_choice_popup_list<'a>(theme: &Theme, options: &[String]) -> List<'a> {
    let text_style = Style::default().fg(theme.text.as_tui_colour());

    let items: Vec<ListItem<'a>> = options
        .iter()
        .map(|entry| {
            ListItem::new(Spans::from(Span::styled(
                format!(" {}", entry.as_str()),
                text_style,
            )))
        })
        .collect();

    List::new(items)
        .block(Block::default().style(Style::default().fg(theme.text.as_tui_colour())))
        .style(Style::default().bg(theme.elevation(Elevation::Level2).as_tui_colour()))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(" >")
}

#[allow(
    clippy::trivially_copy_pass_by_ref,
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::integer_arithmetic,
    clippy::integer_division
)]
fn generate_timed_popup_progress_bar<'a>(theme: &Theme, duration: u64, timer: u64) -> Gauge<'a> {
    Gauge::default()
        .block(Block::default())
        .gauge_style(Style::default().fg(theme.text_dimmed.as_tui_colour()))
        .label("")
        .percent(((duration - timer) * 100 / duration) as u16) // don't care for loss of precision here
}

fn generate_input_popup_layout(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            vec![
                Constraint::Percentage(50), // message
                Constraint::Percentage(50), // input box
            ]
            .as_ref(),
        )
        .split(area)
}

fn generate_timed_info_popup_layout(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            vec![
                Constraint::Percentage(95), // message
                Constraint::Percentage(5),  // progress bar
            ]
            .as_ref(),
        )
        .split(area)
}

#[allow(
    clippy::integer_arithmetic,
    clippy::integer_division,
    clippy::indexing_slicing
)]
fn generate_popup_layout(r: Rect) -> Vec<Rect> {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - POPUP_PERCENTAGE_X) / 2),
                Constraint::Percentage(POPUP_PERCENTAGE_X),
                Constraint::Percentage((100 - POPUP_PERCENTAGE_X) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((87 - POPUP_PERCENTAGE_Y) / 2),
                Constraint::Percentage(POPUP_PERCENTAGE_Y),
                Constraint::Percentage((87 - POPUP_PERCENTAGE_Y) / 2),
                Constraint::Min(3), // Footer
            ]
            .as_ref(),
        )
        .split(layout[1])
}
