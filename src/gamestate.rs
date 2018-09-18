use ggez::{
    event, graphics, timer, {Context, GameResult},
};
use std::io::{BufRead, BufReader, Read};

use tile::Tile;
use port::Port;
use world::World;
use config::Config;
use Position;
use path::PathDrawer;

const TILESET_PATH: &str = "/tileset.png";
const MAP_PATH: &str = "/map.ppm";

/// Width of the grid.
const GRID_WIDTH: u32 = 30;

/// Height of the grid.
const GRID_HEIGTH: u32 = 15;

/// Load world from image file, mapping RGB to tiles.
fn load_world(ctx: &mut Context) -> World {
    let mut map_file = ctx.filesystem.open(MAP_PATH).unwrap();
    let mut header_buffer = [0; 58];
    map_file.read_exact(&mut header_buffer).unwrap();

    let map_file = BufReader::new(map_file);
    let buffer: Vec<u32> = map_file
        .lines()
        .map(|s| s.unwrap().parse::<u32>().unwrap())
        .collect();

    let mut map = vec![];
    let mut ports = vec![];

    buffer
        .as_slice()
        .chunks(3)
        .enumerate()
        .for_each(|(i, colors)| {
            let index = i as i32;
            let position = Position::new(index % GRID_WIDTH as i32, index / GRID_WIDTH as i32);
            match colors {
                [205, 232, 247] => map.push((position, Tile::Water)),
                [0, 0, 0] => {
                    ports.push((position, Port::new()));
                    map.push((position, Tile::Land));
                },
                _ => map.push((position, Tile::Land)),
            }
        });
    let world = World::new(map.into_iter(), ports.into_iter());
    world
}

/// Setup a basic spritebatch for sprites that will not move.
fn configure_base_batch(
    ctx: &mut Context,
    config: &Config,
    batch: &mut graphics::spritebatch::SpriteBatch,
    world: &World,
    ) {
    // Find correct cell with for scaling grid.
    let (window_width, _) = graphics::get_drawable_size(ctx);
    let cell_size = config.scaling * window_width / GRID_WIDTH;

    // Add all tiles to spritebatch, check neighbors to get correct variant and rotation.
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
        config: Config,
        frames: usize,
        batch: graphics::spritebatch::SpriteBatch,
        world: World,
        drawer: Option<PathDrawer>
    },
}

impl GameState {
    /// Creates a new game state in Play mode.
    pub fn new(ctx: &mut Context, config: Config) -> GameResult<Self> {
        // Load game world from file.
        let world = load_world(ctx);

        // Load spritebatch for effective drawing of sprites.
        let image = graphics::Image::new(ctx, TILESET_PATH)?;
        let mut batch = graphics::spritebatch::SpriteBatch::new(image);
        configure_base_batch(ctx, &config, &mut batch, &world);

        let state = GameState::Playing {
            config,
            frames: 0,
            batch,
            world,
            drawer: None,
        };
        Ok(state)
    }
}

impl event::EventHandler for GameState {
    /// Updates the game state.
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    /// Handle mouse down events (drawing of paths etc.)
    fn mouse_button_down_event(&mut self,ctx: &mut Context, button: event::MouseButton, x: i32,y: i32) {
        match self {
            GameState::Playing { drawer, config, world, .. } => {
                *drawer = match drawer {
                    // Drawing already in progress, stop drawing.
                    // TODO: Create/extend an actual route if the path is valid.
                    Some(d) => {
                        None
                    }
                    // Start drawing a new path
                    // TODO: Only allow path drawing between ports.
                    None => {
                        let (window_width, _) = graphics::get_drawable_size(ctx);
                        let cell_size = (config.scaling * window_width / GRID_WIDTH) as i32;

                        // Check that we start to draw from a port.
                        // TODO: Check that this is allowed by further by the game rules.
                        let mouse_position = Position::new(config.scaling as i32 * x / cell_size, config.scaling as i32 * y / cell_size);
                        if world.port(mouse_position).is_some() {
                            Some(PathDrawer::new(Position::new(config.scaling as i32 * x / cell_size, config.scaling as i32 * y / cell_size)))
                        }
                        else {
                            None
                        }
                    }
                }
            }
        }
    }

    /// Handle mouse movement events (updating path drawing etc.)
    fn mouse_motion_event(&mut self,ctx: &mut Context, _button: event::MouseState, x: i32,y: i32, _xrel: i32, _yrel: i32) {
        match self {
            GameState::Playing { drawer, config, world, .. } => {
                let (window_width, _) = graphics::get_drawable_size(ctx);
                let cell_size = (config.scaling * window_width / GRID_WIDTH) as i32;
                if let Some(d) = drawer {
                    d.update(Position::new(config.scaling as i32 * x / cell_size, config.scaling as i32 * y / cell_size), &world);
                }
            }
        }
    }

    /// Draws the current state to the screen with the given context.
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        match self {
            GameState::Playing {
                frames,
                config,
                world,
                batch,
                drawer,
            } => {
                if *frames % 100 == 0 {
                    println!("FPS: {:.1}", timer::get_fps(ctx));
                }
                graphics::clear(ctx);
                graphics::set_background_color(ctx, graphics::Color::from((243, 243, 236)));
                // Draw base batch.
                graphics::draw_ex(ctx, batch, graphics::DrawParam::default())?;

                // Create an upper spritebatch for effective drawing of sprites.
                let image = graphics::Image::new(ctx, TILESET_PATH)?;
                let mut upper_batch = graphics::spritebatch::SpriteBatch::new(image);

                // Find correct cell with for scaling grid.
                let (window_width, _) = graphics::get_drawable_size(ctx);
                let cell_size = (config.scaling * window_width / GRID_WIDTH) as i32;

                // Tile size is handled a bit oddly in the game engine.
                let tile_size = 65. / 256.;
                let tile_offset = 64. / 256.;


                // Draw all ports.
                for (position, _) in world.ports() {
                    let param = graphics::DrawParam {
                        src: graphics::Rect::new(0., 2. * tile_offset, tile_size, tile_size),
                        dest: graphics::Point2::new((position.coords.x * cell_size) as f32, (position.coords.y * cell_size) as f32),
                        scale: graphics::Point2::new(cell_size as f32 / 64., cell_size as f32 / 64.),
                        ..Default::default()
                    };
                    upper_batch.add(param);
                }

                // Check if we have a path and draw upon tile.
                if let Some(d) = drawer {
                    if let Some((_, path)) = d.path() {
                        for position in path {
                            let param = graphics::DrawParam {
                                src: graphics::Rect::new(tile_offset, 2. * tile_offset, tile_size, tile_size),
                                dest: graphics::Point2::new((position.coords.x * cell_size) as f32, (position.coords.y * cell_size) as f32),
                                scale: graphics::Point2::new(cell_size as f32 / 64., cell_size as f32 / 64.),
                                ..Default::default()
                            };
                            upper_batch.add(param);

                        }
                    }
                }
                graphics::draw_ex(ctx, &mut upper_batch, graphics::DrawParam::default())?;
                graphics::present(ctx);

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
