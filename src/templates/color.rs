#[derive([SERDE]Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn casted<I>(self) -> I
    where
        Self: Into<I>,
    {
        self.into()
    }
}

impl From<u32> for Color {
    fn from(value: u32) -> Self {
        Self::new(
            (value >> 24 & 0xff) as u8,
            (value >> 16 & 0xff) as u8,
            (value >> 8 & 0xff) as u8,
            (value & 0xff) as u8,
        )
    }
}

impl From<Color> for u32 {
    fn from(value: Color) -> Self {
        (value.r as u32) << 24 | (value.g as u32) << 16 | (value.b as u32) << 8 | value.a as u32
    }
}

impl ColorImpl for Color {
    fn from_hex(hex: u32) -> Self {
        Self::from(hex)
    }
}

