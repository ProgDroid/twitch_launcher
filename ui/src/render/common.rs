use crate::theme::{Elevation, Theme};
use tui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Tabs, Wrap},
};

pub const HORIZONTAL_MARGIN: u16 = 2;
pub const VERTICAL_MARGIN: u16 = 1;

pub const TAB_TITLES: [&str; 1] = ["Home"];

#[allow(clippy::integer_division)]
pub fn animate_ellipsis(timer: u64) -> String {
    (0..((timer / 2) % 4)).map(|_| ".").collect::<String>()
}

pub fn generate_background_widget<'a>(colour: Color) -> Block<'a> {
    Block::default()
        .title("")
        .style(Style::default().bg(colour))
}

pub fn generate_app_layout(area: Rect) -> Vec<Rect> {
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

pub fn generate_tabs_widget<'a>(tab: usize, theme: &Theme) -> Tabs<'a> {
    let menu = TAB_TITLES
        .iter()
        .map(|t| Spans::from(vec![Span::raw(*t)]))
        .collect();

    Tabs::new(menu)
        .select(tab)
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

pub fn generate_title<'a>(
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

#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn generate_search_box<'a>(
    theme: &Theme,
    search_input: &[char],
    typing: bool,
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
            if typing {
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

pub fn generate_keys_widget<'a>(theme: &Theme, keybinds: &[String]) -> Paragraph<'a> {
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
