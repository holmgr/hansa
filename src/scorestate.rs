use ggez::{
    event, graphics, timer, {Context, GameResult},
};

use color::Color;
use fonts::FontCache;
use std::cell::RefCell;
use tally::Tally;

/// Handles and displays the score board.
pub struct ScoreState<'a> {
    font_cache: FontCache,
    frames: usize,
    tally: &'a RefCell<Tally>,
}

impl<'a> ScoreState<'a> {
    /// Creates a new score board state.
    pub fn new(ctx: &mut Context, tally: &'a RefCell<Tally>) -> GameResult<Self> {
        let state = ScoreState {
            font_cache: FontCache::new(ctx),
            frames: 0,
            tally,
        };
        Ok(state)
    }
}

impl<'a> event::EventHandler for ScoreState<'a> {
    /// Updates the score board state.
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    /// Draws the current state to the screen with the given context.
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        if self.frames % 100 == 0 {
            println!("FPS: {:.1}", timer::get_fps(ctx));
        }
        graphics::clear(ctx);
        graphics::set_background_color(ctx, graphics::Color::from((243, 243, 236)));

        let (window_width, window_height) = graphics::get_drawable_size(ctx);

        // Set blackish color.
        graphics::set_color(ctx, graphics::Color::from_rgb(69, 55, 52))?;
        let title = graphics::Text::new(ctx, "Score.", self.font_cache.medium())?;
        let title_y_offset = window_height as f32 * 0.2;
        let title_x_offset = (window_width - title.width()) as f32 / 2.;

        graphics::draw(
            ctx,
            &title,
            graphics::Point2::new(title_x_offset, title_y_offset),
            0.,
        )?;

        let tally = self.tally.try_borrow().expect("Failed to read tally");
        let start_game = graphics::Text::new(
            ctx,
            &format!(
                "Score: {}\nRed: {}, Green: {}, Blue:{}",
                tally.score(),
                tally.get(Color::Red),
                tally.get(Color::Green),
                tally.get(Color::Blue)
            ),
            self.font_cache.medium(),
        )?;
        let start_game_y_offset =
            title_y_offset + (title_y_offset * 0.2).max(title.height() as f32 * 1.2);
        let start_game_x_offset = (window_width - start_game.width()) as f32 / 2.;

        graphics::draw(
            ctx,
            &start_game,
            graphics::Point2::new(start_game_x_offset, start_game_y_offset),
            0.,
        )?;

        // Reset color to default (white).
        graphics::set_color(ctx, graphics::Color::from_rgb(255, 255, 255))?;

        graphics::present(ctx);
        self.frames += 1;
        // And yield the timeslice
        // This tells the OS that we're done using the CPU but it should
        // get back to this program as soon as it can.
        // This ideally prevents the game from using 100% CPU all the time
        // even if vsync is off.
        // The actual behavior can be a little platform-specific.
        timer::yield_now();

        Ok(())
    }
}
