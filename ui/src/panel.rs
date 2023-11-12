pub trait Panel {
    #[must_use]
    fn left(&self) -> Self;
    #[must_use]
    fn right(&self) -> Self;
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Home {
    Favourites,
    Search,
}

impl Panel for Home {
    fn left(&self) -> Self {
        match *self {
            Self::Favourites | Self::Search => Self::Favourites,
        }
    }

    fn right(&self) -> Self {
        match *self {
            Self::Favourites | Self::Search => Self::Search,
        }
    }
}

impl Default for Home {
    #[inline]
    fn default() -> Self {
        Self::Favourites
    }
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Lists {
    Lists,
    ListContent,
}

impl Panel for Lists {
    fn left(&self) -> Self {
        match *self {
            Self::Lists | Self::ListContent => Self::Lists,
        }
    }

    fn right(&self) -> Self {
        match *self {
            Self::Lists | Self::ListContent => Self::ListContent,
        }
    }
}

impl Default for Lists {
    #[inline]
    fn default() -> Self {
        Self::Lists
    }
}
