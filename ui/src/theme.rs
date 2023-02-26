use num::clamp;
use tui::style::{Color, Modifier};

// TODO write unit(+integration?) tests
// TODO this is a strong argument for a different storage format (e.g. TOML/YAML)

#[derive(Clone)]
pub struct CustomColour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[repr(usize)]
pub enum Elevation {
    Level0 = 0,
    Level1 = 1,
    Level2 = 2,
    Level3 = 3,
    Level4 = 4,
    Level6 = 5,
    Level8 = 6,
    Level12 = 7,
    Level16 = 8,
    Level24 = 9,
}

// TODO implement theme loading/saving

impl CustomColour {
    #[must_use]
    #[allow(
        clippy::use_self,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    pub fn blend(&self, color: &CustomColour, factor: f32) -> Self {
        Self {
            r: clamp(
                f32::from(self.r).mul_add(1.0 - factor, f32::from(color.r) * factor),
                0.0,
                255.0,
            )
            .round() as u8,
            g: clamp(
                f32::from(self.g).mul_add(1.0 - factor, f32::from(color.g) * factor),
                0.0,
                255.0,
            )
            .round() as u8,
            b: clamp(
                f32::from(self.b).mul_add(1.0 - factor, f32::from(color.b) * factor),
                0.0,
                255.0,
            )
            .round() as u8,
        }
    }

    #[must_use]
    pub const fn from(colour: Color) -> Self {
        match colour {
            Color::Black => Self {
                r: 12,
                g: 12,
                b: 12,
            },
            Color::Blue => Self {
                r: 0,
                g: 55,
                b: 218,
            },
            Color::Cyan => Self {
                r: 58,
                g: 150,
                b: 221,
            },
            Color::Green => Self {
                r: 19,
                g: 161,
                b: 14,
            },
            Color::Magenta => Self {
                r: 136,
                g: 23,
                b: 152,
            },
            Color::Red => Self {
                r: 197,
                g: 15,
                b: 31,
            },
            Color::Gray => Self {
                r: 204,
                g: 204,
                b: 204,
            },
            Color::Yellow => Self {
                r: 193,
                g: 156,
                b: 0,
            },
            Color::DarkGray => Self {
                r: 118,
                g: 118,
                b: 118,
            },
            Color::White => Self {
                r: 242,
                g: 242,
                b: 242,
            },
            Color::LightBlue => Self {
                r: 59,
                g: 120,
                b: 255,
            },
            Color::LightCyan => Self {
                r: 97,
                g: 214,
                b: 214,
            },
            Color::LightGreen => Self {
                r: 22,
                g: 198,
                b: 12,
            },
            Color::LightMagenta => Self {
                r: 180,
                g: 0,
                b: 158,
            },
            Color::LightRed => Self {
                r: 231,
                g: 72,
                b: 86,
            },
            Color::LightYellow => Self {
                r: 249,
                g: 241,
                b: 165,
            },
            Color::Rgb(r, g, b) => Self { r, g, b },
            _ => BG_GREY,
        }
    }

    #[inline]
    #[must_use]
    pub const fn as_tui_colour(&self) -> Color {
        Color::Rgb(self.r, self.g, self.b)
    }
}

const BG_GREY: CustomColour = CustomColour {
    r: 18,
    g: 18,
    b: 18,
};

const WHITE: CustomColour = CustomColour {
    r: 255,
    g: 255,
    b: 255,
};

const BLACK: CustomColour = CustomColour { r: 0, g: 0, b: 0 };

const ELEVATION: [f32; 10] = [0.0, 0.05, 0.07, 0.08, 0.09, 0.11, 0.12, 0.14, 0.15, 0.16];

#[derive(Clone)]
pub struct Cursor {
    pub cursor: String,
    pub modifier: Modifier,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            cursor: String::from(" "),
            modifier: Modifier::BOLD | Modifier::RAPID_BLINK | Modifier::REVERSED,
        }
    }
}

#[derive(Clone)]
pub struct Theme {
    pub background: CustomColour,
    pub primary: CustomColour,
    pub secondary: CustomColour,
    pub text: CustomColour,
    pub text_dimmed: CustomColour,
    pub cursor: Cursor,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: BG_GREY,
            primary: CustomColour::from(Color::Yellow),
            secondary: CustomColour::from(Color::LightRed).blend(&BLACK, 0.1),
            text: WHITE,
            text_dimmed: WHITE.blend(&BLACK, 0.5),
            cursor: Cursor::default(),
        }
    }
}

impl Theme {
    #[allow(clippy::needless_arbitrary_self_type)]
    #[must_use]
    pub fn elevation(self: &Self, level: Elevation) -> CustomColour {
        self.background.blend(&WHITE, ELEVATION[level as usize])
    }
}
