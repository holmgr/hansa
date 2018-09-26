use ggez::{
    graphics::{
        clear, draw_ex, get_drawable_size, present, set_background_color,
        spritebatch::{SpriteBatch, SpriteIdx},
        Color, DrawParam, Image, Point2,
    },
    Context, GameResult,
};
use std::{
    collections::hash_map::{DefaultHasher, HashMap},
    hash::{Hash, Hasher},
};

use config::Config;

/// A drawable type.
pub trait Drawable<'a>: Hash {
    // TODO: Move magic constant here.
    const TILE_SIZE: f32 = 65. / 256.;
    const TILE_OFFSET: f32 = 64. / 256.;

    /// Environmental data needed to draw item.
    type Data;

    /// Returns a drawparam (sprite) representing this type.
    fn draw(&self, data: &'a Self::Data) -> DrawParam;
}

/// A cached drawer which keeps track of and draws indiviual drawawble items,
/// as a single call to the GPU.
pub struct SpriteDrawer {
    sprites: HashMap<u64, SpriteIdx>,
    batch: SpriteBatch,
    hasher: DefaultHasher,
}

impl SpriteDrawer {
    /// Creates a new SpriteDrawer for optimized drawing.
    pub fn new(image: Image) -> Self {
        SpriteDrawer {
            sprites: HashMap::new(),
            batch: SpriteBatch::new(image),
            hasher: DefaultHasher::new(),
        }
    }

    /// Clears the sprite drawer of all sprites.
    pub fn clear(&mut self) {
        self.batch.clear();
        self.sprites.clear();
    }

    /// Draws the given item, uses cache if already drawn.
    pub fn draw_item<'a, T: Drawable<'a>>(
        &mut self,
        ctx: &mut Context,
        config: &Config,
        item: &T,
        data: &'a T::Data,
    ) {
        item.hash(&mut self.hasher);
        let id = self.hasher.finish();
        let mut param = item.draw(data);

        // Scale the param to so it matches grid size screen.
        let (window_width, _) = get_drawable_size(ctx);
        let cell_size = (config.scaling * window_width / config.grid_width) as f32;

        // TODO: Move magic constant here.
        param.scale = Point2::new(
            param.scale.x * cell_size / 64.,
            param.scale.y * cell_size / 64.,
        );
        param.dest = Point2::new(param.dest.x * cell_size, param.dest.y * cell_size);

        let idx = match self.sprites.get(&id) {
            Some(idx) => {
                self.batch
                    .set(*idx, param)
                    .unwrap_or_else(|_| panic!("Failed to draw item {:?} on batch", idx));
                *idx
            }
            None => self.batch.add(param),
        };
        self.sprites.insert(id, idx);

        // Clear hasher internal state.
        self.hasher = DefaultHasher::new();
    }

    /// Draws all sprites in the sprite drawer on the screen.
    pub fn paint(&self, ctx: &mut Context, _config: &Config) -> GameResult<()> {
        // Find correct cell with for scaling grid.
        clear(ctx);

        // TODO: Move magic constant here.
        set_background_color(ctx, Color::from((243, 243, 236)));
        draw_ex(
            ctx,
            &self.batch,
            DrawParam {
                ..Default::default()
            },
        )?;
        present(ctx);

        Ok(())
    }
}
