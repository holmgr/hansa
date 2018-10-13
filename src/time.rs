use ggez::{
    graphics::{draw, get_drawable_size, set_color, Color, Font, Point2, Text},
    timer::get_time_since_start,
    Context, GameResult,
};
use std::time::Duration;

use config::Config;

/// Keeps track of the time past since the game was started.
pub struct GameTimer {
    start_time: Duration,
    session_length: Duration,
}

impl GameTimer {
    /// Creates a new timer with the given start time and game session length.
    pub fn new(start_time: Duration, session_length: Duration) -> Self {
        GameTimer {
            start_time,
            session_length,
        }
    }

    /// Returns the remaining game time.
    pub fn time_remaining(&self, ctx: &Context) -> Duration {
        (self.start_time + self.session_length)
            .checked_sub(get_time_since_start(ctx))
            .unwrap_or(Duration::from_secs(0))
    }

    /// Returns true if there is no game time remaining.
    pub fn has_game_ended(&self, ctx: &Context) -> bool {
        self.time_remaining(ctx).as_secs() == 0
    }

    /// Draws the current remaining game time on screen.
    /// Does not implement Drawable since it is unable to be drawn using a
    /// spritebatch.
    pub fn paint(&self, font: &Font, ctx: &mut Context, config: &Config) -> GameResult<()> {
        // TODO: Properly ugly code with loads of magic constants.
        let (window_width, _) = get_drawable_size(ctx);
        let cell_size = (config.scaling * window_width) as f32 / config.grid_width as f32;

        let y_offset = (config.grid_height as f32 + 1.) as f32 * cell_size;

        let seconds_remaining = self.time_remaining(ctx).as_secs();

        // Draw remaining time.
        set_color(ctx, Color::from_rgb(69, 55, 52))?;
        let text = Text::new(
            ctx,
            &format!("{}:{:#02}", seconds_remaining / 60, seconds_remaining % 60),
            &font,
        )?;
        draw(
            ctx,
            &text,
            Point2::new(
                (window_width as f32 * 0.1).min(100.),
                y_offset - (text.height() as f32 / 2.),
            ),
            0.,
        )?;
        // Reset color to default (white).
        set_color(ctx, Color::from_rgb(255, 255, 255))?;

        Ok(())
    }
}
