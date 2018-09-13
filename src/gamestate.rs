use ggez::{
    event, graphics, timer, {Context, GameResult},
};

use world::World;

const TILESET_PATH: &str = "/tileset.png";

/// Handles and holds all game information.
pub enum GameState {
    Playing {
        frames: usize,
        batch: graphics::spritebatch::SpriteBatch,
        world: World,
    },
}

impl GameState {
    /// Creates a new game state in Play mode.
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        // Load spritebatch for effective drawing of sprites.
        let image = graphics::Image::new(ctx, TILESET_PATH)?;
        let batch = graphics::spritebatch::SpriteBatch::new(image);

        let state = GameState::Playing {
            frames: 0,
            batch,
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
            GameState::Playing {
                frames,
                world,
                batch,
            } => {
                graphics::clear(ctx);
                graphics::set_background_color(ctx, graphics::Color::from((216, 218, 223)));

                // TODO: Draw something more useful :P
                let param = graphics::DrawParam {
                    dest: graphics::Point2::new(0.0, 0.0),
                    ..Default::default()
                };
                batch.add(param);
                graphics::draw_ex(ctx, batch, param)?;
                batch.clear();

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
