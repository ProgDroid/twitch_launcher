use crate::channel::{Channel, ChannelStatus};
use crate::theme::{Elevation, Theme};
// use crate::twitch_account::TwitchAccount;
use chrono::{Datelike, Local};
use std::cmp::min;
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::terminal::Frame;
use tui::text::{Span, Spans};
use tui::widgets::{Block, List, ListItem, ListState, Paragraph, Tabs};

const HORIZONTAL_MARGIN: u16 = 2;
const VERTICAL_MARGIN: u16 = 1;

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
            .block(Block::default().style(Style::default().fg(theme.text.as_tui_colour())))
            .style(Style::default().fg(Color::White))
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

pub fn render_home<B: Backend>(
    theme: &Theme,
    frame: &mut Frame<'_, B>,
    tab: &usize,
    tab_titles: &[&'static str; 2],
    channel_highlight: &usize,
    channels: &Vec<Channel>,
) {
    let size = frame.size();

    let block = Block::default().style(
        Style::default()
            .bg(theme.background.as_tui_colour())
            .fg(Color::White),
    );
    frame.render_widget(block, size);

    let chunks = Layout::default()
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
        .split(size);

    let background = Paragraph::new("")
        .block(Block::default().title(""))
        .style(Style::default().bg(theme.elevation(Elevation::Level1).as_tui_colour()));

    frame.render_widget(background, chunks[2]);

    let middle_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .horizontal_margin(HORIZONTAL_MARGIN)
        .vertical_margin(VERTICAL_MARGIN)
        .constraints([Constraint::Percentage(30), Constraint::Min(1)].as_ref())
        .split(chunks[2]);

    let menu = tab_titles
        .iter()
        .map(|t| Spans::from(vec![Span::raw(*t)]))
        .collect();

    let tabs = Tabs::new(menu)
        .select(*tab)
        .block(Block::default().title(""))
        .style(
            Style::default()
                .fg(Color::White)
                .bg(theme.elevation(Elevation::Level1).as_tui_colour()),
        )
        .highlight_style(
            Style::default()
                .fg(theme.primary.as_tui_colour())
                .add_modifier(Modifier::BOLD),
        )
        .divider(Span::raw(""));

    frame.render_widget(tabs, chunks[0]);

    let mut list_state: ListState = ListState::default();
    list_state.select(Some(*channel_highlight));

    match Tab::from(*tab) {
        Tab::Home => {
            let list_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
                .split(middle_chunks[0]);

            frame.render_widget(
                generate_title(
                    "Favourites",
                    theme.elevation(Elevation::Level2).as_tui_colour(),
                    (&theme.primary).as_tui_colour(),
                ),
                list_chunks[0],
            );

            frame.render_stateful_widget(
                generate_favourites_widget(theme, &channels, middle_chunks[0].width),
                list_chunks[1],
                &mut list_state,
            );
        }
        Tab::Follows => {
            frame.render_widget(
                generate_follows_widget(theme, &channels, middle_chunks[0].width),
                middle_chunks[0],
            );
        }
    }

    let info_text = vec![Spans::from(vec![Span::styled(
        format!("Twitch Launcher {} ", Local::now().year()),
        Style::default().add_modifier(Modifier::ITALIC),
    )])];

    let info = Paragraph::new(info_text)
        .style(Style::default().fg((&theme.primary).as_tui_colour()))
        .alignment(Alignment::Right)
        .block(
            Block::default()
                .style(Style::default().fg(Color::White))
                .title(""),
        );

    frame.render_widget(info, chunks[3]);
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
                ChannelStatus::Online => Style::default().fg((&theme.secondary).as_tui_colour()),
                ChannelStatus::Offline => Style::default()
                    .fg((&theme.text).as_tui_colour())
                    .add_modifier(Modifier::DIM),
                ChannelStatus::Unknown => Style::default().fg((&theme.text).as_tui_colour()),
            };

            ListItem::new(Spans::from(vec![
                Span::styled(
                    format!(
                        " {:text_width$}",
                        a.friendly_name.as_str(),
                        text_width = min((width as usize) - a.status.message().len() - 5, 25),
                    ),
                    Style::default().fg((&theme.text).as_tui_colour()),
                ),
                Span::styled(format!("{}", a.status.message()), status_style),
            ]))
        })
        .collect();

    List::new(items)
        .block(Block::default().style(Style::default().fg((&theme.text).as_tui_colour())))
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
                .style(Style::default().fg(Color::White))
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
