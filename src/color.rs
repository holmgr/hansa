/// A drawable color of the game's color schme.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    Blue,
    Green,
    Purple,
    Red,
    Yellow,
}

impl Color {
    /// Return a tuple of the color's components in the RGB scheme.
    pub fn rgb(&self) -> (u8, u8, u8) {
        match self {
            Color::Blue => (72, 133, 237),
            Color::Green => (60, 186, 88),
            Color::Purple => (88, 42, 114),
            Color::Red => (219, 50, 54),
            Color::Yellow => (244, 194, 15),
        }
    }

    /// Returns all color variants.
    pub fn values() -> Vec<Color> {
        vec![
            Color::Blue,
            Color::Green,
            Color::Purple,
            Color::Red,
            Color::Yellow,
        ]
    }
}

/// Handles selection, unselection of a series of colors.
pub struct ColorSelector {
    selected: Option<usize>,
    colors: Vec<Color>,
}

impl ColorSelector {
    /// Create a new color seclector.
    pub fn new() -> Self {
        ColorSelector {
            selected: None,
            colors: Color::values(),
        }
    }

    /// Returns the selected color, if any.
    pub fn selected(&self) -> Option<Color> {
        if let Some(index) = self.selected {
            Some(self.colors[index])
        } else {
            None
        }
    }

    /// Selects the color at the given index if not already selected,
    /// otherwise unselect.
    pub fn toggle(&mut self, index: usize) {
        self.selected = match self.selected {
            Some(current_index) if current_index == index => None,
            Some(_) => Some(index),
            None => Some(index),
        };
    }

    /// Returns an iterator over all colors available in this selector.
    pub fn colors(&self) -> impl Iterator<Item = &Color> {
        self.colors.iter()
    }
}
