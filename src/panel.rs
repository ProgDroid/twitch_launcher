pub trait Panel {
    fn left(&self) -> Self;
    fn right(&self) -> Self;
}

#[derive(PartialEq)]
#[repr(usize)]
pub enum HomePanel {
    Favourites = 0,
    Search = 1,
}

impl Panel for HomePanel {
    fn left(&self) -> Self {
        match self {
            HomePanel::Favourites => HomePanel::Favourites,
            HomePanel::Search => HomePanel::Favourites,
        }
    }

    fn right(&self) -> Self {
        match self {
            HomePanel::Favourites => HomePanel::Search,
            HomePanel::Search => HomePanel::Search,
        }
    }
}

impl Default for HomePanel {
    fn default() -> Self {
        HomePanel::Favourites
    }
}
