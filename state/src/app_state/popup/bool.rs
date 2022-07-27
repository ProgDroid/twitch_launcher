// use std::fmt::{Display, Formatter, Result};

// #[repr(usize)]
// #[derive(PartialEq)]
// pub enum BoolChoice {
//     False = 0,
//     True = 1,
// }

// impl Display for Bool {
//     fn fmt(&self, f: Formatter) -> Result {
//         match self {
//             Self::False => write!(f, "No"),
//             Self::True => write!(f, "Yes"),
//         }
//     }
// }

// impl From<usize> for Bool {
//     fn from(input: usize) -> Self {
//         match input {
//             0 => Self::False,
//             1 => Self::True,
//             _ => {
//                 eprintln!("Provided Bool Choice does not exist");
//                 Self::False
//             }
//         }
//     }
// }

// impl Bool {
//     #[must_use]
//     pub fn is_true(&self) -> bool {
//         *self == Self::True
//     }

//     // #[must_use]
//     // pub const fn yes_no_display(&self) -> &str {
//     //     match *self {
//     //         Self::False => "No",
//     //         Self::True => "Yes",
//     //     }
//     // }
// }
