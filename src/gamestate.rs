use ggez::{
    event, graphics, timer, {Context, GameResult},
};

use world::World;

/// Handles and holds all game information.
pub enum GameState {
    Playing { frames: usize, world: World },
}

impl GameState {
    /// Creates a new game state in Play mode.
    pub fn new(_ctx: &mut Context) -> GameResult<Self> {
        let state = GameState::Playing {
            frames: 0,
            world: World::default(),
        };
        Ok(state)
    }
}

impl event::EventHandler for GameState {
    /// Updates the game state.
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    /// Draws the current state to the screen with the given context.
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        match self {
            GameState::Playing { frames, world } => {
                graphics::clear(ctx);
                graphics::set_background_color(ctx, graphics::Color::from((216, 218, 223)));

                graphics::present(ctx);
                *frames += 1;
                if (*frames % 100) == 0 {
                    println!("FPS: {}", timer::get_fps(ctx));
                }
            }
        }

        Ok(())
    }
}
