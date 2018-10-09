/// A drawable color of the game's color scheme.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    Blue,
    Green,
    Red,
}

impl Color {
    /// Return a tuple of the color's components in the RGB scheme.
    pub fn rgb(self) -> (u8, u8, u8) {
        match self {
            Color::Blue => (46, 83, 161),
            Color::Green => (2, 170, 92),
            Color::Red => (232, 66, 54),
        }
    }

    /// Returns all color variants.
    pub fn values() -> Vec<Color> {
        vec![Color::Blue, Color::Green, Color::Red]
    }
}
