use crate::{
    channel::{Channel, ChannelStatus},
    panel::HomePanel,
    popup::Popup,
    theme::{Elevation, Theme},
};
use chrono::{Datelike, Local};
use std::cmp::min;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    terminal::Frame,
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs},
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
    fn from(input: usize) -> Tab {
        match input {
            0 => Tab::Home,
            1 => Tab::Follows,
            _ => {
                eprintln!("Menu Item does not exist, going Home");
                Tab::Home
            }
        }
    }
}

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
        Paragraph::new(format!("Starting{}", animate_ellipsis(&timer)))
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
            animate_ellipsis(&timer)
        ))
        .block(Block::default().style(Style::default()))
        .style(Style::default().fg(theme.text.as_tui_colour()))
        .alignment(Alignment::Left),
        chunks[1],
    );
}

// ? TODO this might need splitting
pub fn render_home<B: Backend>(
    theme: &Theme,
    frame: &mut Frame<'_, B>,
    tab: &usize,
    tab_titles: &[&'static str; 2],
    channel_highlight: &usize,
    channels: &Vec<Channel>,
    popup: &Option<Popup>,
    search_input: &Vec<char>,
    focused_panel: &HomePanel,
) {
    let area = frame.size();

    if let Some(p) = popup {
        let area = generate_popup_layout(area);

        frame.render_widget(
            generate_background_widget(theme.background.as_tui_colour()),
            area,
        );

        let paragraph = Paragraph::new(p.message.as_str())
            .block(
                Block::default()
                    .title((&p.title).as_str())
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.secondary.as_tui_colour())),
            )
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
        return;
    }

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
            // TODO make use of focused_panel here
            let list_chunks = generate_favourites_layout(content_area[0]);

            frame.render_widget(
                generate_title(
                    "Favourites",
                    theme.elevation(Elevation::Level2).as_tui_colour(),
                    (&theme.primary).as_tui_colour(),
                ),
                list_chunks[0],
            );

            // TODO should this be a part of State?
            let mut list_state: ListState = ListState::default();
            list_state.select(Some(*channel_highlight));

            frame.render_stateful_widget(
                generate_favourites_widget(theme, &channels, content_area[0].width),
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
                generate_channel_search(theme, search_input),
                search_chunks[1],
            );
        }
        Tab::Follows => {
            frame.render_widget(
                generate_follows_widget(theme, &channels, content_area[0].width),
                content_area[0],
            );
        }
    }

    frame.render_widget(generate_info_widget(theme), app_layout[3]);
}

fn generate_favourites_widget<'a>(
    theme: &Theme,
    channels: &'a Vec<Channel>,
    width: u16,
) -> List<'a> {
    let items: Vec<ListItem<'a>> = channels
        .iter()
        .map(|a| {
            let status_style = match a.status {
                ChannelStatus::Online => Style::default().fg(theme.secondary.as_tui_colour()),
                ChannelStatus::Offline => Style::default()
                    .fg((&theme.text).as_tui_colour())
                    .add_modifier(Modifier::DIM),
                ChannelStatus::Unknown => Style::default().fg(theme.text.as_tui_colour()),
            };

            ListItem::new(Spans::from(vec![
                Span::styled(
                    format!(
                        " {:text_width$}",
                        a.friendly_name.as_str(),
                        text_width = min((width as usize) - a.status.message().len() - 5, 25),
                    ),
                    Style::default().fg(theme.text.as_tui_colour()),
                ),
                Span::styled(format!("{}", a.status.message()), status_style),
            ]))
        })
        .collect();

    List::new(items)
        .block(Block::default().style(Style::default().fg(theme.text.as_tui_colour())))
        .style(Style::default().bg(theme.elevation(Elevation::Level2).as_tui_colour()))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(" >")
}

#[allow(unused_variables)] // TODO remove this when not needed
fn generate_follows_widget<'a>(
    theme: &Theme,
    channels: &Vec<Channel>,
    width: u16,
) -> Paragraph<'a> {
    Paragraph::new(vec![Spans::from(vec![Span::raw("Test Follows")])])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .style(Style::default().fg(theme.text.as_tui_colour()))
                .title(""),
        )
}

fn animate_ellipsis(timer: &u64) -> String {
    format!(
        "{}",
        (0..((timer / 2) % 4)).map(|_| ".").collect::<String>()
    )
}

fn generate_title<'a>(title: &str, bg_colour: Color, text_colour: Color) -> Paragraph<'a> {
    Paragraph::new(vec![
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![
            Span::raw(" "),
            Span::styled(
                format!("{}", title),
                Style::default()
                    .fg(text_colour)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ])
    .block(Block::default().style(Style::default().bg(bg_colour)))
    .style(Style::default())
    .alignment(Alignment::Left)
}

fn generate_popup_layout<'a>(r: Rect) -> Rect {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - POPUP_PERCENTAGE_Y) / 2),
                Constraint::Percentage(POPUP_PERCENTAGE_Y),
                Constraint::Percentage((100 - POPUP_PERCENTAGE_Y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - POPUP_PERCENTAGE_X) / 2),
                Constraint::Percentage(POPUP_PERCENTAGE_X),
                Constraint::Percentage((100 - POPUP_PERCENTAGE_X) / 2),
            ]
            .as_ref(),
        )
        .split(layout[1])[1]
}

fn generate_channel_search<'a>(theme: &Theme, search_input: &Vec<char>) -> Paragraph<'a> {
    let input_string: String = search_input.iter().collect();

    Paragraph::new(input_string).block(
        Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(
                "Search Channel",
                Style::default().fg(theme.text.as_tui_colour()),
            ))
            .border_style(Style::default().fg(theme.primary.as_tui_colour()))
            .style(Style::default().bg(theme.elevation(Elevation::Level2).as_tui_colour())),
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
                Constraint::Length(2), // Footer
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

fn generate_info_widget<'a>(theme: &Theme) -> Paragraph<'a> {
    let info_text = vec![Spans::from(vec![Span::styled(
        format!("Twitch Launcher {} ", Local::now().year()),
        Style::default().add_modifier(Modifier::ITALIC),
    )])];

    Paragraph::new(info_text)
        .style(Style::default().fg(theme.primary.as_tui_colour()))
        .alignment(Alignment::Right)
        .block(Block::default().style(Style::default()).title(""))
}
