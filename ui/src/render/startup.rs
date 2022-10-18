use crate::render::common::{animate_ellipsis, HORIZONTAL_MARGIN, VERTICAL_MARGIN};
use crate::theme::Theme;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    terminal::Frame,
    widgets::{Block, Paragraph},
};

#[allow(clippy::trivially_copy_pass_by_ref, clippy::indexing_slicing)]
pub fn starting<B: Backend>(theme: &Theme, frame: &mut Frame<'_, B>, timer: u64) {
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
pub fn account_missing<B: Backend>(theme: &Theme, frame: &mut Frame<'_, B>, timer: u64) {
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
            "Account not configured, please enter your details when prompted{}",
            animate_ellipsis(timer)
        ))
        .block(Block::default().style(Style::default()))
        .style(Style::default().fg(theme.text.as_tui_colour()))
        .alignment(Alignment::Left),
        chunks[1],
    );
}
