#![allow(clippy::pub_use)]

mod common;
mod home;
mod popup;
pub mod startup;

pub use home::home;
pub use popup::{choice, input, timed_info};
pub use startup::{account_missing, starting};

// #[repr(usize)]
// enum Tab {
//     Home = 0,
//     Follows = 1,
// }

// impl From<usize> for Tab {
//     fn from(input: usize) -> Self {
//         match input {
//             0 => Self::Home,
//             1 => Self::Follows,
//             _ => {
//                 eprintln!("Menu Item does not exist, going Home");
//                 Self::Home
//             }
//         }
//     }
// }

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
