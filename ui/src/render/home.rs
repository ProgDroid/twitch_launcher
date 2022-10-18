use crate::{
    panel::Home,
    render::common::{
        generate_app_layout, generate_background_widget, generate_input_box, generate_keys_widget,
        generate_tabs_widget, generate_title, HORIZONTAL_MARGIN, VERTICAL_MARGIN,
    },
    theme::{Elevation, Theme},
};
use std::cmp::{max, min};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    terminal::Frame,
    text::{Span, Spans},
    widgets::{Block, List, ListItem, ListState},
};
use twitch::channel::{status::Status, Channel};

// TODO if favourites file doesn't exist, nothing works. This shouldn't happen

#[allow(
    clippy::trivially_copy_pass_by_ref,
    clippy::indexing_slicing,
    clippy::too_many_arguments
)]
pub fn home<B: Backend>(
    theme: &Theme,
    frame: &mut Frame<'_, B>,
    channel_highlight: &usize,
    channels: &[Channel],
    typing: bool,
    search_input: &[char],
    focused_panel: &Home,
    keybinds: &[String],
) {
    let area = frame.size();

    frame.render_widget(
        generate_background_widget(theme.background.as_tui_colour()),
        area,
    );

    let app_layout = generate_app_layout(area);

    frame.render_widget(
        generate_background_widget(theme.elevation(Elevation::Level1).as_tui_colour()),
        app_layout[2],
    );

    let content_area = generate_content_area_layout(app_layout[2]);

    frame.render_widget(generate_tabs_widget(0, theme), app_layout[0]);

    let list_chunks = generate_favourites_layout(content_area[0]);

    let favourites_focused = *focused_panel == Home::Favourites;
    let search_focused = *focused_panel == Home::Search;

    frame.render_widget(
        generate_title(
            "Favourites",
            theme.elevation(Elevation::Level2).as_tui_colour(),
            (&theme.primary).as_tui_colour(),
            favourites_focused,
        ),
        list_chunks[0],
    );

    let mut list_state: ListState = ListState::default();
    list_state.select(Some(*channel_highlight));

    frame.render_stateful_widget(
        generate_favourites_widget(theme, channels, content_area[0].width, favourites_focused),
        list_chunks[1],
        &mut list_state,
    );

    let search_chunks_with_margin = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(HORIZONTAL_MARGIN), Constraint::Min(1)].as_ref())
        .split(content_area[1]);

    frame.render_widget(
        generate_background_widget(theme.elevation(Elevation::Level2).as_tui_colour()),
        search_chunks_with_margin[1],
    );

    let search_chunks = generate_search_layout(search_chunks_with_margin[1]);

    frame.render_widget(
        generate_input_box(
            theme,
            search_input,
            typing,
            search_focused,
            String::from("Search Channel"),
            theme.elevation(Elevation::Level2).as_tui_colour(),
        ),
        search_chunks[1],
    );

    frame.render_widget(generate_keys_widget(theme, keybinds), app_layout[3]);
}

fn generate_content_area_layout(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Horizontal)
        .horizontal_margin(HORIZONTAL_MARGIN)
        .vertical_margin(VERTICAL_MARGIN)
        .constraints([Constraint::Percentage(30), Constraint::Min(1)].as_ref())
        .split(area)
}

fn generate_favourites_layout(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
        .split(area)
}

#[allow(clippy::integer_arithmetic, clippy::as_conversions)]
fn generate_favourites_widget<'a>(
    theme: &Theme,
    channels: &'a [Channel],
    width: u16,
    focused: bool,
) -> List<'a> {
    let online_style = Style::default().fg(theme.secondary.as_tui_colour());
    let offline_style = Style::default().fg(theme.text_dimmed.as_tui_colour());
    let unknown_status_style = Style::default().fg(theme.text.as_tui_colour());
    let awaiting_status_style = Style::default().fg(theme.text.as_tui_colour());
    let text_style = Style::default().fg(theme.text.as_tui_colour());

    let items: Vec<ListItem<'a>> = channels
        .iter()
        .map(|entry| {
            let status_style = match entry.status {
                Status::Awaiting => awaiting_status_style,
                Status::Online => online_style,
                Status::Offline => offline_style,
                Status::Unknown => unknown_status_style,
            };

            ListItem::new(Spans::from(vec![
                Span::styled(
                    format!(
                        " {:text_width$}",
                        entry.friendly_name.as_str(),
                        text_width = min(
                            max(width as usize, entry.status.message().len() - 5)
                                - entry.status.message().len()
                                - 5,
                            25,
                        ),
                    ),
                    text_style,
                ),
                Span::styled(entry.status.message(), status_style),
            ]))
        })
        .collect();

    let mut block_style = Style::default().fg(theme.text.as_tui_colour());

    if !focused {
        block_style = block_style.add_modifier(Modifier::DIM);
    }

    // TODO allow theme to have highlight style? Or make constant

    List::new(items)
        .block(Block::default().style(block_style))
        .style(Style::default().bg(theme.elevation(Elevation::Level2).as_tui_colour()))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(" >")
}

#[allow(clippy::indexing_slicing)]
fn generate_search_layout(area: Rect) -> Vec<Rect> {
    let search_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(7), // Left Margin
                Constraint::Min(1),        // Search Bar
                Constraint::Percentage(5), // Right Margin
            ]
            .as_ref(),
        )
        .split(area);

    Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(42), // Top Margin
                Constraint::Length(3),      // Search Bar
                Constraint::Min(1),         // Bottom Margin
            ]
            .as_ref(),
        )
        .split(search_chunks[1])
}
