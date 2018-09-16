use ggez::{
    event, graphics, timer, {Context, GameResult},
};
use std::io::{BufRead, BufReader, Read};

use tile::Tile;
use world::World;
use Position;

const TILESET_PATH: &str = "/tileset.png";
const MAP_PATH: &str = "/map.ppm";

/// Width of the grid.
const GRID_WIDTH: u32 = 30;

/// Height of the grid.
const GRID_HEIGTH: u32 = 15;

/// FUCK Apple OpenGL implementation.
static MACBOOK_SCALING: u32 = 2;

/// Load world map from image file, mapping RGB to tiles.
fn load_map(ctx: &mut Context) -> Vec<(Position, Tile)> {
    let mut map_file = ctx.filesystem.open(MAP_PATH).unwrap();
    let mut header_buffer = [0; 58];
    map_file.read_exact(&mut header_buffer).unwrap();

    let map_file = BufReader::new(map_file);
    let buffer: Vec<u32> = map_file
        .lines()
        .map(|s| s.unwrap().parse::<u32>().unwrap())
        .collect();
    let result: Vec<(Position, Tile)> = buffer
        .as_slice()
        .chunks(3)
        .enumerate()
        .map(|(i, colors)| {
            let index = i as i32;
            let position = Position::new(index % GRID_WIDTH as i32, index / GRID_WIDTH as i32);
            match colors {
                [205, 232, 247] => (position, Tile::Water),
                _ => (position, Tile::Land),
            }
        }).collect();
    result
}

/// Setup a basic spritebatch for sprites that will not move.
fn configure_base_batch(
    ctx: &mut Context,
    batch: &mut graphics::spritebatch::SpriteBatch,
    world: &World,
) {
    // Find correct cell with for scaling grid.
    let (window_width, _) = graphics::get_drawable_size(ctx);
    let cell_size = MACBOOK_SCALING * window_width / GRID_WIDTH;

    for i in 0..GRID_WIDTH {
        for j in 0..GRID_HEIGTH {
            let curr_cell = world.tile(Position::new(i as i32, j as i32)).unwrap();
            let north_cell = world.tile(Position::new(i as i32, j as i32 - 1));
            let east_cell = world.tile(Position::new(i as i32 + 1, j as i32));
            let south_cell = world.tile(Position::new(i as i32, j as i32 + 1));
            let west_cell = world.tile(Position::new(i as i32 - 1, j as i32));

            // Tile size is handled a bit oddly in the game engine.
            let tile_size = 65. / 256.;
            let tile_offset = 64. / 256.;

            // Check neighbors to determine which actual sprite should be used.
            let src = match (curr_cell, north_cell, east_cell, south_cell, west_cell) {
                (Tile::Water, _, _, _, _) => graphics::Rect::new(0., 0., tile_size, tile_size),
                (
                    Tile::Land,
                    Some(Tile::Water),
                    Some(Tile::Water),
                    Some(Tile::Land),
                    Some(Tile::Land),
                ) => graphics::Rect::new(0.0, tile_offset, tile_size, tile_size),
                (
                    Tile::Land,
                    Some(Tile::Land),
                    Some(Tile::Water),
                    Some(Tile::Water),
                    Some(Tile::Land),
                ) => graphics::Rect::new(1. * tile_offset, tile_offset, tile_size, tile_size),
                (
                    Tile::Land,
                    Some(Tile::Land),
                    Some(Tile::Land),
                    Some(Tile::Water),
                    Some(Tile::Water),
                ) => graphics::Rect::new(2. * tile_offset, tile_offset, tile_size, tile_size),
                (
                    Tile::Land,
                    Some(Tile::Water),
                    Some(Tile::Land),
                    Some(Tile::Land),
                    Some(Tile::Water),
                ) => graphics::Rect::new(3. * tile_offset, tile_offset, tile_size, tile_size),
                (Tile::Land, _, _, _, _) => {
                    graphics::Rect::new(tile_offset, 0., tile_size, tile_size)
                }
            };

            let dest = graphics::Point2::new((i * cell_size) as f32, (j * cell_size) as f32);
            let param = graphics::DrawParam {
                src,
                dest,
                scale: graphics::Point2::new(cell_size as f32 / 64., cell_size as f32 / 64.),
                ..Default::default()
            };
            batch.add(param);
        }
    }
}

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
        // Load game world from file.
        let map = load_map(ctx);
        let world = World::new(map.into_iter());

        // Load spritebatch for effective drawing of sprites.
        let image = graphics::Image::new(ctx, TILESET_PATH)?;
        let mut batch = graphics::spritebatch::SpriteBatch::new(image);
        configure_base_batch(ctx, &mut batch, &world);

        let state = GameState::Playing {
            frames: 0,
            batch,
            world,
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
                if (*frames % 100) == 0 {
                    println!("FPS: {:.1}", timer::get_fps(ctx));
                    graphics::clear(ctx);
                    graphics::set_background_color(ctx, graphics::Color::from((243, 243, 236)));
                    // Draw base batch.
                    graphics::draw_ex(ctx, batch, graphics::DrawParam::default())?;

                    graphics::present(ctx);
                }
                *frames += 1;
                // And yield the timeslice
                // This tells the OS that we're done using the CPU but it should
                // get back to this program as soon as it can.
                // This ideally prevents the game from using 100% CPU all the time
                // even if vsync is off.
                // The actual behavior can be a little platform-specific.
                timer::yield_now();
            }
        }

        Ok(())
    }
}
