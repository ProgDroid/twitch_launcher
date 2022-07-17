pub trait Panel {
    #[must_use]
    fn left(&self) -> Self;
    #[must_use]
    fn right(&self) -> Self;
}

#[derive(PartialEq)]
#[repr(usize)]
pub enum Home {
    Favourites = 0,
    Search = 1,
}

#[allow(clippy::missing_inline_in_public_items)]
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
