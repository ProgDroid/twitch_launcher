use crate::{
    channel::{Channel, Status},
    keybind::Keybind,
    panel::Home,
    popup::{Choice, Popup, Popups},
    state::TabTitles,
    theme::{Elevation, Theme},
};
use std::cmp::{max, min};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    terminal::Frame,
    text::{Span, Spans},
    widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph, Tabs, Wrap},
};

const HORIZONTAL_MARGIN: u16 = 2;
const VERTICAL_MARGIN: u16 = 1;

const POPUP_PERCENTAGE_X: u16 = 60;
const POPUP_PERCENTAGE_Y: u16 = 20;

#[repr(usize)]
enum Tab {
    Home = 0,
    Follows = 1,
}

impl From<usize> for Tab {
    fn from(input: usize) -> Self {
        match input {
            0 => Self::Home,
            1 => Self::Follows,
            _ => {
                eprintln!("Menu Item does not exist, going Home");
                Self::Home
            }
        }
    }
}

#[allow(clippy::trivially_copy_pass_by_ref, clippy::indexing_slicing)]
pub fn startup_animation<B: Backend>(theme: &Theme, frame: &mut Frame<'_, B>, timer: &u64) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .horizontal_margin(HORIZONTAL_MARGIN)
        .vertical_margin(VERTICAL_MARGIN)
        .constraints(
            [
                Constraint::Percentage(49), // Empty Area
                Constraint::Min(1),         // Message
            ]
            .as_ref(),
        )
        .split(frame.size());

    frame.render_widget(
        Paragraph::new(format!("Starting{}", animate_ellipsis(timer)))
            .block(
                Block::default().style(
                    Style::default()
                        .fg(theme.text.as_tui_colour())
                        .add_modifier(Modifier::ITALIC),
                ),
            )
            .alignment(Alignment::Left),
        chunks[1],
    );
}

#[allow(clippy::trivially_copy_pass_by_ref, clippy::indexing_slicing)]
pub fn account_missing<B: Backend>(theme: &Theme, frame: &mut Frame<'_, B>, timer: &u64) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .horizontal_margin(HORIZONTAL_MARGIN)
        .vertical_margin(VERTICAL_MARGIN)
        .constraints(
            [
                Constraint::Percentage(49), // Empty Area
                Constraint::Min(1),         // Message
            ]
            .as_ref(),
        )
        .split(frame.size());

    frame.render_widget(
        Paragraph::new(format!(
            "Account not loaded, please configure{}",
            animate_ellipsis(timer)
        ))
        .block(Block::default().style(Style::default()))
        .style(Style::default().fg(theme.text.as_tui_colour()))
        .alignment(Alignment::Left),
        chunks[1],
    );
}

// TODO consider breaking this up

#[allow(
    clippy::too_many_arguments,
    clippy::trivially_copy_pass_by_ref,
    clippy::indexing_slicing
)]
pub fn home<B: Backend>(
    theme: &Theme,
    frame: &mut Frame<'_, B>,
    tab: &usize,
    tab_titles: &TabTitles,
    channel_highlight: &usize,
    channels: &[Channel],
    typing: &bool,
    search_input: &[char],
    focused_panel: &Home,
    keybinds: &[Keybind],
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

    let menu = tab_titles
        .iter()
        .map(|t| Spans::from(vec![Span::raw(*t)]))
        .collect();

    frame.render_widget(generate_tabs_widget(menu, tab, theme), app_layout[0]);

    match Tab::from(*tab) {
        Tab::Home => {
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
                generate_favourites_widget(
                    theme,
                    channels,
                    content_area[0].width,
                    favourites_focused,
                ),
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
                generate_search_box(
                    theme,
                    search_input,
                    typing,
                    search_focused,
                    String::from("Search Channel"),
                    theme.elevation(Elevation::Level2).as_tui_colour(),
                ),
                search_chunks[1],
            );
        }
        Tab::Follows => {
            // frame.render_widget(
            //     generate_follows_widget(theme, &channels, content_area[0].width),
            //     content_area[0],
            // );
        }
    }

    frame.render_widget(generate_keys_widget(theme, keybinds), app_layout[3]);
}

#[allow(clippy::indexing_slicing)]
pub fn popup<B: Backend>(
    theme: &Theme,
    frame: &mut Frame<'_, B>,
    popup: &Popup,
    keybinds: &[Keybind],
) {
    let area = frame.size();

    let popup_area = generate_popup_layout(area);

    frame.render_widget(
        generate_background_widget(theme.background.as_tui_colour()),
        popup_area[1],
    );

    let popup_sections = match popup.variant {
        Popups::Choice { .. } => generate_choice_popup_layout(popup_area[1]),
        Popups::Input { .. } => generate_input_popup_layout(popup_area[1]),
        Popups::TimedInfo { .. } => generate_timed_info_popup_layout(popup_area[1]),
    };

    let paragraph = Paragraph::new(popup.message.as_str())
        .block(
            Block::default()
                .title((&popup.title).as_str())
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.secondary.as_tui_colour())),
        )
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, popup_sections[0]);

    match popup.variant {
        Popups::Choice {
            ref options,
            selected,
        } => {
            let mut list_state: ListState = ListState::default();
            list_state.select(Some(selected));

            frame.render_stateful_widget(
                generate_choice_popup_list(theme, options),
                popup_sections[1],
                &mut list_state,
            );
        }
        Popups::Input { ref input, typing } => frame.render_widget(
            generate_search_box(
                theme,
                input,
                &typing,
                true,
                String::from(""),
                theme.background.as_tui_colour(),
            ),
            popup_sections[1],
        ),
        Popups::TimedInfo { duration, timer } => frame.render_widget(
            generate_timed_popup_progress_bar(theme, &duration, &timer),
            popup_sections[1],
        ),
    }

    frame.render_widget(generate_keys_widget(theme, keybinds), popup_area[3]);
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

fn generate_choice_popup_list<'a>(theme: &Theme, options: &[Choice]) -> List<'a> {
    let text_style = Style::default().fg(theme.text.as_tui_colour());

    let items: Vec<ListItem<'a>> = options
        .iter()
        .map(|entry| {
            ListItem::new(Spans::from(Span::styled(
                format!(" {}", entry.display_text.as_str()),
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
    clippy::integer_arithmetic
)]
fn generate_timed_popup_progress_bar<'a>(theme: &Theme, duration: &u64, timer: &u64) -> Gauge<'a> {
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

// #[allow(unused_variables)] // TODO remove this when not needed
// fn generate_follows_widget<'a>(
//     theme: &Theme,
//     channels: &Vec<Channel>,
//     width: u16,
// ) -> Paragraph<'a> {
//     Paragraph::new(vec![Spans::from(vec![Span::raw("Test Follows")])])
//         .alignment(Alignment::Center)
//         .block(
//             Block::default()
//                 .style(Style::default().fg(theme.text.as_tui_colour()))
//                 .title(""),
//         )
// }

#[allow(clippy::trivially_copy_pass_by_ref)]
fn animate_ellipsis(timer: &u64) -> String {
    (0..((timer / 2) % 4)).map(|_| ".").collect::<String>()
}

fn generate_title<'a>(
    title: &str,
    bg_colour: Color,
    text_colour: Color,
    focused: bool,
) -> Paragraph<'a> {
    let mut block_style = Style::default().bg(bg_colour);

    if !focused {
        block_style = block_style.add_modifier(Modifier::DIM);
    }

    Paragraph::new(vec![
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![
            Span::raw(" "),
            Span::styled(
                title.to_owned(),
                Style::default()
                    .fg(text_colour)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ])
    .block(Block::default().style(block_style))
    .style(Style::default())
    .alignment(Alignment::Left)
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

#[allow(clippy::trivially_copy_pass_by_ref)]
fn generate_search_box<'a>(
    theme: &Theme,
    search_input: &[char],
    typing: &bool,
    focused: bool,
    title: String,
    background: Color,
) -> Paragraph<'a> {
    let input_string: String = search_input.iter().collect();

    let mut block_style = Style::default().bg(background);

    if !focused {
        block_style = block_style.add_modifier(Modifier::DIM);
    }

    Paragraph::new(Spans::from(vec![
        Span::from(input_string),
        Span::styled(
            if *typing {
                String::from(&theme.cursor.cursor)
            } else {
                String::from("")
            },
            Style::default()
                .fg(theme.secondary.as_tui_colour())
                .add_modifier(theme.cursor.modifier),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(
                title,
                Style::default().fg(theme.text.as_tui_colour()),
            ))
            .border_style(Style::default().fg(theme.primary.as_tui_colour()))
            .style(block_style),
    )
}

fn generate_background_widget<'a>(colour: Color) -> Block<'a> {
    Block::default()
        .title("")
        .style(Style::default().bg(colour))
}

fn generate_app_layout(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .horizontal_margin(HORIZONTAL_MARGIN)
        .vertical_margin(VERTICAL_MARGIN)
        .constraints(
            [
                Constraint::Length(3), // Header
                Constraint::Length(1), // Empty Area
                Constraint::Min(0),    // Content Area
                Constraint::Length(3), // Footer
            ]
            .as_ref(),
        )
        .split(area)
}

fn generate_content_area_layout(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Horizontal)
        .horizontal_margin(HORIZONTAL_MARGIN)
        .vertical_margin(VERTICAL_MARGIN)
        .constraints([Constraint::Percentage(30), Constraint::Min(1)].as_ref())
        .split(area)
}

fn generate_tabs_widget<'a>(menu: Vec<Spans<'a>>, tab: &usize, theme: &Theme) -> Tabs<'a> {
    Tabs::new(menu)
        .select(*tab)
        .block(Block::default().title(""))
        .style(
            Style::default()
                .fg(theme.text.as_tui_colour())
                .bg(theme.elevation(Elevation::Level1).as_tui_colour()),
        )
        .highlight_style(
            Style::default()
                .fg(theme.primary.as_tui_colour())
                .add_modifier(Modifier::BOLD),
        )
        .divider(Span::raw(""))
}

fn generate_favourites_layout(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
        .split(area)
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

fn generate_keys_widget<'a>(theme: &Theme, keybinds: &[Keybind]) -> Paragraph<'a> {
    let info_text: Vec<Span> = keybinds
        .iter()
        .enumerate()
        .map(|(i, bind)| {
            let spacer = if i == 0 { "" } else { " | " };

            Span::styled(
                format!("{}{}", spacer, bind),
                Style::default().add_modifier(Modifier::ITALIC),
            )
        })
        .collect();

    Paragraph::new(Spans::from(info_text))
        .style(Style::default().fg(theme.text_dimmed.as_tui_colour()))
        .block(Block::default().style(Style::default()).title(""))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
}
