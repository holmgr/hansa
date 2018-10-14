use ggez::{
    graphics::{
        clear, draw_ex, get_drawable_size, set_background_color, spritebatch::SpriteBatch, Color,
        DrawParam, Image, Point2,
    },
    Context, GameResult,
};

use animation::Animation;
use config::Config;

/// A drawable type.
pub trait Drawable<'a> {
    // TODO: Move magic constant here.
    const TILE_SIZE: f32 = 512. / 2048.;
    const TILE_OFFSET: f32 = 513. / 2048.;

    /// Environmental data needed to draw item.
    type Data;

    /// Returns the current animation, defaults to None for all types.
    fn animation(&self) -> Option<Animation> {
        None
    }

    /// Returns a drawparam (sprite) representing this type.
    fn draw(&self, data: &'a Self::Data) -> Vec<DrawParam>;
}

/// A cached drawer which keeps track of and draws indiviual drawawble items,
/// as a single call to the GPU.
pub struct SpriteDrawer(SpriteBatch);

impl SpriteDrawer {
    /// Creates a new SpriteDrawer for optimized drawing.
    pub fn new(image: Image) -> Self {
        SpriteDrawer(SpriteBatch::new(image))
    }

    /// Clears the sprite drawer of all sprites.
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Draws the given item.
    pub fn draw_item<'a, T: Drawable<'a>>(
        &mut self,
        ctx: &mut Context,
        config: &Config,
        item: &T,
        data: &'a T::Data,
        grid_scaling: bool,
    ) {
        let mut params = item.draw(data);
        // Animate if possible.
        if let Some(animation) = item.animation() {
            params = animation.animate(params);
        }
        for mut param in params {
            // Scale the param to so it matches grid size screen.
            let (window_width, _) = get_drawable_size(ctx);
            let cell_size = (config.scaling * window_width) as f32 / config.grid_width as f32;

            // TODO: Move magic constant here.
            // Get background shine through with tile size as normalization factor,
            // seems to be an issue with scaling in the engine.
            param.scale = Point2::new(
                param.scale.x * cell_size / 508.,
                param.scale.y * cell_size / 508.,
            );

            // Scale to grid coordinates only if needed.
            if grid_scaling {
                param.dest = Point2::new(param.dest.x * cell_size, param.dest.y * cell_size);
            } else {
                param.dest = Point2::new(
                    param.dest.x * config.scaling as f32,
                    param.dest.y * config.scaling as f32,
                );
            }
            self.0.add(param);
        }
    }

    /// Draws all sprites in the sprite drawer on the screen.
    pub fn paint(&self, ctx: &mut Context, _config: &Config) -> GameResult<()> {
        // Find correct cell with for scaling grid.
        clear(ctx);

        // TODO: Move magic constant here.
        set_background_color(ctx, Color::from((243, 243, 236)));
        draw_ex(
            ctx,
            &self.0,
            DrawParam {
                ..Default::default()
            },
        )?;

        Ok(())
    }
}
