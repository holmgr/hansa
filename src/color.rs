use ggez::graphics::{Color as ggezColor, DrawParam, Point2, Rect};

use config::Config;
use draw::Drawable;

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
    pub fn rgb(self) -> (u8, u8, u8) {
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

impl<'a> Drawable<'a> for Color {
    type Data = (&'a Config, &'a ColorSelector);

    fn draw(&self, data: &(&Config, &ColorSelector)) -> DrawParam {
        let (config, selector) = data;
        let index = selector.find(*self).unwrap();
        let (r, g, b) = self.rgb();

        // Add little scale factor to indicate do the user which color is selected.
        let scale_factor = match selector.selected() {
            Some(selected_color) if selected_color == *self => 1.3,
            _ => 1.,
        };
        let num_colors = selector.colors().count() as u32;
        let color_selector_x_offset = (config.grid_width / 2 - num_colors + 1) as f32;
        let color_selector_y_offset = (config.grid_height as f32 + 1.) as f32;
        DrawParam {
            src: Rect::new(
                2. * Self::TILE_SIZE,
                2. * Self::TILE_SIZE,
                Self::TILE_SIZE,
                Self::TILE_SIZE,
            ),
            dest: Point2::new(
                color_selector_x_offset + 2. * index as f32 as f32,
                color_selector_y_offset,
            ),
            scale: Point2::new(scale_factor, scale_factor),
            offset: Point2::new(0.5, 0.5),
            color: Some(ggezColor::from_rgb(r, g, b)),
            ..Default::default()
        }
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

    /// Returns the index of the given color.
    pub fn find(&self, color: Color) -> Option<usize> {
        self.colors.iter().position(|c| *c == color)
    }

    /// Returns an iterator over all colors available in this selector.
    pub fn colors(&self) -> impl Iterator<Item = &Color> {
        self.colors.iter()
    }
}
