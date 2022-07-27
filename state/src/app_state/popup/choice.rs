pub struct Choice {
    pub selected: usize,
    pub options: Vec<String>,
}

// impl Choice {
//     fn as_bool(&self) -> bool {
//         match self.selected {
//             0 => false,
//             1 => true,
//             _ => {
//                 eprintln!("Cannot convert usize > 1 to bool, returning false...");
//                 false
//             }
//         }
//     }
// }
