use num::clamp;
use tui::style::{Color, Modifier};

#[derive(Debug)]
pub struct CustomColour {
    pub r: u8,
    pub b: u8,
    pub g: u8,
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
    pub fn blend(self: &Self, color: CustomColour, factor: f32) -> CustomColour {
        CustomColour {
            r: clamp(
                (self.r as f32) * (1.0 - factor) + (color.r as f32) * factor,
                0.0,
                255.0,
            )
            .round() as u8,
            g: clamp(
                (self.g as f32) * (1.0 - factor) + (color.g as f32) * factor,
                0.0,
                255.0,
            )
            .round() as u8,
            b: clamp(
                (self.b as f32) * (1.0 - factor) + (color.b as f32) * factor,
                0.0,
                255.0,
            )
            .round() as u8,
        }
    }

    pub fn from(colour: Color) -> CustomColour {
        match colour {
            Color::Black => CustomColour {
                r: 12,
                g: 12,
                b: 12,
            },
            Color::Blue => CustomColour {
                r: 0,
                g: 55,
                b: 218,
            },
            Color::Cyan => CustomColour {
                r: 58,
                g: 150,
                b: 221,
            },
            Color::Green => CustomColour {
                r: 19,
                g: 161,
                b: 14,
            },
            Color::Magenta => CustomColour {
                r: 136,
                g: 23,
                b: 152,
            },
            Color::Red => CustomColour {
                r: 197,
                g: 15,
                b: 31,
            },
            Color::Gray => CustomColour {
                r: 204,
                g: 204,
                b: 204,
            },
            Color::Yellow => CustomColour {
                r: 193,
                g: 156,
                b: 0,
            },
            Color::DarkGray => CustomColour {
                r: 118,
                g: 118,
                b: 118,
            },
            Color::White => CustomColour {
                r: 242,
                g: 242,
                b: 242,
            },
            Color::LightBlue => CustomColour {
                r: 59,
                g: 120,
                b: 255,
            },
            Color::LightCyan => CustomColour {
                r: 97,
                g: 214,
                b: 214,
            },
            Color::LightGreen => CustomColour {
                r: 22,
                g: 198,
                b: 12,
            },
            Color::LightMagenta => CustomColour {
                r: 180,
                g: 0,
                b: 158,
            },
            Color::LightRed => CustomColour {
                r: 231,
                g: 72,
                b: 86,
            },
            Color::LightYellow => CustomColour {
                r: 249,
                g: 241,
                b: 165,
            },
            Color::Rgb(r, g, b) => CustomColour { r: r, g: g, b: b },
            _ => BG_GREY,
        }
    }

    pub fn as_tui_colour(self: &Self) -> Color {
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

#[derive(Debug)]
pub struct Cursor {
    pub cursor: String,
    pub modifier: Modifier,
}

impl Default for Cursor {
    fn default() -> Self {
        Cursor {
            cursor: String::from(" "),
            modifier: Modifier::BOLD | Modifier::RAPID_BLINK | Modifier::REVERSED,
        }
    }
}

#[derive(Debug)]
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
        Theme {
            background: BG_GREY,
            primary: CustomColour::from(Color::Yellow),
            secondary: CustomColour::from(Color::LightRed).blend(BLACK, 0.1),
            text: WHITE,
            text_dimmed: WHITE.blend(BLACK, 0.5),
            cursor: Cursor::default(),
        }
    }
}

impl Theme {
    pub fn elevation(self: &Self, level: Elevation) -> CustomColour {
        self.background.blend(WHITE, ELEVATION[level as usize])
    }
}
