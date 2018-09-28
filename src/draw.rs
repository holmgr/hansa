use ggez::{
    graphics::{
        clear, draw_ex, get_drawable_size, present, set_background_color, spritebatch::SpriteBatch,
        Color, DrawParam, Image, Point2,
    },
    Context, GameResult,
};

use config::Config;

/// A drawable type.
pub trait Drawable<'a> {
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
        let mut param = item.draw(data);

        // Scale the param to so it matches grid size screen.
        let (window_width, _) = get_drawable_size(ctx);
        let cell_size = (config.scaling * window_width / config.grid_width) as f32;

        // TODO: Move magic constant here.
        param.scale = Point2::new(
            param.scale.x * cell_size / 64.,
            param.scale.y * cell_size / 64.,
        );

        // Scale to grid coordinates only if needed.
        if grid_scaling {
            param.dest = Point2::new(param.dest.x * cell_size, param.dest.y * cell_size);
        }
        self.0.add(param);
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
        present(ctx);

        Ok(())
    }
}
