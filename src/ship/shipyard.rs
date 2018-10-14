use super::*;

use ggez::{
    graphics::{draw, get_drawable_size, rectangle, set_color, DrawMode, Font, Text},
    GameResult,
};

/// Holds all unplaced 'ships' and manages of drawing UI element for ship selection.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Shipyard {
    ships: usize,
}

impl Shipyard {
    /// Create a new shipyard.
    pub fn new() -> Self {
        Shipyard {
            ships: 3, // TODO: Make this scale with progress.
        }
    }

    /// Returns the number of available ships.
    pub fn available(&self) -> usize {
        self.ships
    }

    /// Return if there are any ships available.
    pub fn is_available(&self) -> bool {
        self.ships > 0
    }

    /// Returns a shipbuilder if not capped on ships.
    pub fn build(&mut self) -> Option<ShipBuilder> {
        if self.is_available() {
            self.ships -= 1;
            Some(ShipBuilder::new())
        } else {
            None
        }
    }

    /// Returns a builder to the shipyard.
    pub fn add_builder(&mut self, builder: ShipBuilder) {
        self.ships += 1;
    }

    /// Returns a ship to the shipyard.
    pub fn add_ship(&mut self, ship: Ship) {
        self.ships += 1;
    }

    /// Draws the shipyard with ship count on screen.
    /// Does not implement Drawable since it is unable to be drawn using a
    /// spritebatch.
    pub fn paint(&self, font: &Font, ctx: &mut Context, config: &Config) -> GameResult<()> {
        let (window_width, _) = get_drawable_size(ctx);
        let cell_size = (config.scaling * window_width) as f32 / config.grid_width as f32;

        let text = Text::new(ctx, &format!("{:1?}: ", self.ships), &font)?;
        let text_dims = text.get_dimensions();

        let x_offset =
            cell_size * (config.grid_width / 2) as f32 - (text_dims.right() + 2. * cell_size) / 2.;
        let y_offset = cell_size * (config.grid_height as f32 + 2.);

        set_color(ctx, ggezColor::from_rgb(69, 55, 52))?;
        draw(
            ctx,
            &text,
            Point2::new(x_offset as f32, y_offset - text_dims.top() / 2.),
            0.,
        )?;
        rectangle(
            ctx,
            DrawMode::Fill,
            Rect::new(
                x_offset + text_dims.right(),
                y_offset,
                cell_size * 2.,
                cell_size,
            ),
        )?;

        // Reset color to default (white).
        set_color(ctx, ggezColor::from_rgb(255, 255, 255))?;

        Ok(())
    }
}
