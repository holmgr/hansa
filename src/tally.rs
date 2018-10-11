use ggez::{
    graphics::{
        circle, draw, get_drawable_size, set_color, Color as ggezColor, DrawMode, Font, Point2,
        Text,
    },
    Context, GameResult,
};
use std::iter::FromIterator;

use color::Color;
use config::Config;

/// Keeps track of the amount of each color collected.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tally {
    collected: Vec<(Color, u32)>,
}

impl Tally {
    /// Creates a new tally.
    pub fn new() -> Self {
        Tally {
            collected: Vec::from_iter(Color::values().into_iter().map(|c| (c, 0))),
        }
    }

    /// Returns the current score.
    pub fn score(&self) -> u32 {
        self.collected
            .iter()
            .map(|(_, amount)| *amount)
            .min()
            .expect("No score found")
    }

    /// Returns the amount collected for the given color.
    pub fn get(&self, color: Color) -> u32 {
        self.collected
            .iter()
            .find(|(c, _)| *c == color)
            .map(|(_, amount)| *amount)
            .expect("Tally for color not found")
    }

    /// Increments the tally for the given color.
    pub fn update(&mut self, color: Color) {
        {
            let (_, amount) = self
                .collected
                .iter_mut()
                .find(|(c, _)| *c == color)
                .expect("Tally for color not found");
            *amount += 1;
        }

        // TODO: Remove this pln.
        println!("Tally is now: {:#?}", self.collected);
    }

    /// Draws the current tally on screen.
    /// Does not implement Drawable since it is unable to be drawn using a
    /// spritebatch.
    pub fn paint(&self, font: &Font, ctx: &mut Context, _config: &Config) -> GameResult<()> {
        // TODO: Properly ugly code with loads of magic constants.
        let color_size = 48.;
        let segment_size = 300.;

        let (window_width, _) = get_drawable_size(ctx);
        let x_offset = window_width as f32 / 2. - 1.3 * segment_size;
        let y_offset = 60.;

        // Draw score for each color.
        for (i, (color, amount)) in self.collected.iter().enumerate() {
            let (r, g, b) = color.rgb();
            set_color(ctx, ggezColor::from_rgb(r, g, b))?;
            circle(
                ctx,
                DrawMode::Fill,
                Point2::new(x_offset + i as f32 * segment_size, y_offset),
                color_size / 2.,
                0.1,
            )?;
            let text = Text::new(ctx, &format!("{:3?}", amount), &font)?;
            draw(
                ctx,
                &text,
                Point2::new(
                    x_offset + i as f32 * segment_size + color_size,
                    y_offset - text.height() as f32 / 2.,
                ),
                0.,
            )?;
        }
        // Reset color to default (white).
        set_color(ctx, ggezColor::from_rgb(255, 255, 255))?;

        Ok(())
    }
}
