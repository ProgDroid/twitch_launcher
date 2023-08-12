use std::cmp::{max, min};

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, List, ListItem, ListState},
    Frame,
};
use twitch::channel::List as ChannelList;

use crate::theme::{Elevation, Theme};

use super::common::{
    generate_app_layout, generate_background_widget, generate_keys_widget, generate_tabs_widget,
    generate_title, HORIZONTAL_MARGIN, VERTICAL_MARGIN,
};

pub fn lists<B: Backend>(
    theme: &Theme,
    frame: &mut Frame<'_, B>,
    highlight: usize,
    lists: &[ChannelList],
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

    frame.render_widget(generate_tabs_widget(1, theme), app_layout[0]);

    let list_chunks = generate_lists_layout(content_area[0]);

    frame.render_widget(
        generate_title(
            "Channel Lists",
            theme.elevation(Elevation::Level2).as_tui_colour(),
            theme.primary.as_tui_colour(),
            true,
        ),
        list_chunks[0],
    );

    let mut list_state: ListState = ListState::default();
    list_state.select(Some(highlight));

    frame.render_stateful_widget(
        generate_lists_widget(theme, lists, content_area[0].width, true),
        list_chunks[1],
        &mut list_state,
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

fn generate_lists_layout(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
        .split(area)
}

fn generate_lists_widget<'a>(
    theme: &Theme,
    lists: &'a [ChannelList],
    width: u16,
    focused: bool,
) -> List<'a> {
    let channel_count_style = Style::default().fg(theme.secondary.as_tui_colour());
    let text_style = Style::default().fg(theme.text.as_tui_colour());

    let items: Vec<ListItem<'a>> = lists
        .iter()
        .map(|entry| {
            let channel_count = entry.channels.len();
            let count_message = format!(
                "{channel_count} channel{}",
                if channel_count == 1 { "" } else { "s" }
            );

            ListItem::new(Spans::from(vec![
                Span::styled(
                    format!(
                        " {:text_width$}",
                        entry.name,
                        text_width = min(
                            max(width as usize, count_message.len() - 5) - count_message.len() - 5,
                            25,
                        ),
                    ),
                    text_style,
                ),
                Span::styled(count_message, channel_count_style),
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
