use ggez::{
    event, graphics, timer, {Context, GameResult},
};

use fonts::FontCache;
use audio::{AudioHandler, SoundEffect};
use config::Config;

pub struct MenuState {
    font_cache: FontCache,
    audio_handler: AudioHandler,
    config: Config,
    frames: usize,
}

impl MenuState {
    /// Creates a new main menu state.
    pub fn new(ctx: &mut Context, config: Config) -> GameResult<Self> {
        let audio_handler = AudioHandler::new(ctx);
        audio_handler.start_music();

        let state = MenuState {
            font_cache: FontCache::new(ctx),
            audio_handler,
            config,
            frames: 0,
        };
        Ok(state)
    }
}

impl event::EventHandler for MenuState {

    /// Updates the menu state.
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    /// Handle mouse down events.
    fn mouse_button_down_event(
        &mut self,
        ctx: &mut Context,
        _button: event::MouseButton,
        x: i32,
        y: i32,
    ) {
        let (window_width, window_height) = graphics::get_drawable_size(ctx);

        // Set blackish color.
        let title = graphics::Text::new(ctx, "hansa.", self.font_cache.large()).unwrap();
        let title_y_offset = window_height as f32 * 0.2;

        let start_game = graphics::Text::new(ctx, "Start new game", self.font_cache.medium()).unwrap();
        let start_game_y_offset = title_y_offset + (title_y_offset * 0.2).max(title.height() as f32 * 1.2);
        let start_game_x_offset = (window_width - start_game.width()) as f32 / 2.;

        let start_button_rect = graphics::Rect::new(start_game_x_offset, start_game_y_offset, start_game.width() as f32, start_game.height() as f32);
        if start_button_rect.contains(graphics::Point2::new(self.config.scaling as f32 * x as f32, self.config.scaling as f32 * y as f32)) {
            self.audio_handler.play(SoundEffect::ClickUIButton);
            ctx.quit().expect("Failed to start game");
        }

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
        let title = graphics::Text::new(ctx, "hansa.", self.font_cache.large())?;
        let title_y_offset = window_height as f32 * 0.2;
        let title_x_offset = (window_width - title.width()) as f32 / 2.;

        graphics::draw(
            ctx,
            &title,
            graphics::Point2::new(
                title_x_offset,
                title_y_offset,
            ),
           0.,
        )?;

        let start_game = graphics::Text::new(ctx, "Start new game", self.font_cache.medium())?;
        let start_game_y_offset = title_y_offset + (title_y_offset * 0.2).max(title.height() as f32 * 1.2);
        let start_game_x_offset = (window_width - start_game.width()) as f32 / 2.;

        graphics::draw(
            ctx,
            &start_game,
            graphics::Point2::new(
                start_game_x_offset,
                start_game_y_offset
            ),
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
