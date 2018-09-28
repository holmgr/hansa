use ggez::{
    event, graphics, timer, {Context, GameResult},
};
use std::io::{BufRead, BufReader, Read};

use color::ColorSelector;
use config::Config;
use draw::SpriteDrawer;
use port::Port;
use route::RouteBuilder;
use tile::{Tile, TileKind};
use world::World;
use Position;

const TILESET_PATH: &str = "/tileset.png";
const MAP_PATH: &str = "/map.ppm";

/// Load world from image file, mapping RGB to tiles.
fn load_world(ctx: &mut Context, config: &Config) -> World {
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
            let position = Position::new(
                index % config.grid_width as i32,
                index / config.grid_width as i32,
            );
            match colors {
                [0, 0, 255] => map.push(Tile::new(position, TileKind::Water)),
                [0, 0, 0] => {
                    ports.push(Port::new(position));
                    map.push(Tile::new(position, TileKind::Land));
                }
                _ => map.push(Tile::new(position, TileKind::Land)),
            }
        });
    World::new(map.into_iter(), ports.into_iter())
}

/// Handles and holds all game information.
pub enum GameState {
    Playing {
        config: Config,
        frames: usize,
        sprite_drawer: SpriteDrawer,
        world: World,
        route_builder: Option<RouteBuilder>,
        color_selector: ColorSelector,
    },
}

impl GameState {
    /// Creates a new game state in Play mode.
    pub fn new(ctx: &mut Context, config: Config) -> GameResult<Self> {
        // Load game world from file.
        let world = load_world(ctx, &config);

        // Load spritebatch for effective drawing of sprites.
        let image = graphics::Image::new(ctx, TILESET_PATH)?;
        let sprite_drawer = SpriteDrawer::new(image);

        let state = GameState::Playing {
            config,
            frames: 0,
            sprite_drawer,
            world,
            route_builder: None,
            color_selector: ColorSelector::new(),
        };
        Ok(state)
    }
}

impl event::EventHandler for GameState {
    /// Updates the game state.
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        match self {
            // Clear sprite drawer to remove old sprites.
            GameState::Playing { sprite_drawer, .. } => sprite_drawer.clear(),
        };
        Ok(())
    }

    /// Handle mouse down events (drawing of paths etc.)
    fn mouse_button_down_event(
        &mut self,
        ctx: &mut Context,
        _button: event::MouseButton,
        x: i32,
        y: i32,
    ) {
        match self {
            GameState::Playing {
                route_builder,
                config,
                world,
                color_selector,
                ..
            } => {
                let (window_width, _) = graphics::get_drawable_size(ctx);
                let cell_size = (config.scaling * window_width / config.grid_width) as i32;

                // Check that we start to draw from a port.
                let mouse_position =
                    Position::new(config.scaling as i32 * x, config.scaling as i32 * y);
                let mouse_position_scaled = Position::new(
                    config.scaling as i32 * x / cell_size,
                    config.scaling as i32 * y / cell_size,
                );

                // Check if some mouse button on some color.
                let num_colors = color_selector.colors().count() as u32;
                let color_selector_x_offset =
                    (config.grid_width / 2 - num_colors + 1) as i32 * cell_size;
                let color_selector_y_offset = (config.grid_height + 1) as i32 * cell_size;

                for index in 0..num_colors {
                    let color_position = Position::new(
                        color_selector_x_offset + (2 * index as i32 * cell_size + cell_size / 2),
                        color_selector_y_offset + cell_size / 2,
                    );

                    // Check eucidean distance.
                    if (color_position.coords.x - mouse_position.coords.x).pow(2)
                        + (color_position.coords.y - mouse_position.coords.y).pow(2)
                        < cell_size.pow(2)
                    {
                        color_selector.toggle(index as usize);
                        println!("Toggling color: {:?}", color_selector.selected());
                    }
                }

                *route_builder = match route_builder {
                    // Drawing already in progress, stop drawing.
                    Some(rb) => {
                        if world.port(mouse_position_scaled).is_some() {
                            // Add an actual route here if, ensure we have a color selected.
                            if let Some(color) = color_selector.selected() {
                                if let Some(path) = rb.path() {
                                    world.add_route(color, *rb.from(), *rb.to(), path.clone())
                                }
                            }
                        }
                        None
                    }
                    // Start drawing a new path
                    None => {
                        if world.port(mouse_position_scaled).is_some()
                            && color_selector.selected().is_some()
                        {
                            Some(RouteBuilder::new(Position::new(
                                config.scaling as i32 * x / cell_size,
                                config.scaling as i32 * y / cell_size,
                            )))
                        } else {
                            None
                        }
                    }
                }
            }
        }
    }

    /// Handle mouse movement events (updating path drawing etc.)
    fn mouse_motion_event(
        &mut self,
        ctx: &mut Context,
        _button: event::MouseState,
        x: i32,
        y: i32,
        _xrel: i32,
        _yrel: i32,
    ) {
        match self {
            GameState::Playing {
                route_builder,
                config,
                world,
                ..
            } => {
                let (window_width, _) = graphics::get_drawable_size(ctx);
                let cell_size = (config.scaling * window_width / config.grid_width) as i32;
                if let Some(rb) = route_builder {
                    rb.update(
                        Position::new(
                            config.scaling as i32 * x / cell_size,
                            config.scaling as i32 * y / cell_size,
                        ),
                        &world,
                    );
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
                sprite_drawer,
                route_builder,
                color_selector,
            } => {
                if *frames % 100 == 0 {
                    println!("FPS: {:.1}", timer::get_fps(ctx));
                }

                graphics::clear(ctx);
                sprite_drawer.clear();

                // Draw all base tiles.
                for tile in world.tiles() {
                    sprite_drawer.draw_item(ctx, config, tile, world, true);
                }

                // Draw all routes.
                for (color, route) in world.routes() {
                    for waypoint in route.waypoints() {
                        sprite_drawer.draw_item(ctx, config, waypoint, color, true);
                    }
                }

                // Draw route (id any) currently being created.
                if let Some(builder) = route_builder {
                    if let Some(waypoints) = builder.path() {
                        let color = color_selector.selected().unwrap();
                        for waypoint in waypoints {
                            sprite_drawer.draw_item(ctx, config, waypoint, &color, true);
                        }
                    }
                }

                // Draw all ports.
                for port in world.ports() {
                    sprite_drawer.draw_item(ctx, config, port, world, true);
                }

                // Draw color selector.
                for color in color_selector.colors() {
                    sprite_drawer.draw_item(ctx, config, color, &(config, &color_selector), true);
                }

                // Draw to screen.
                sprite_drawer.paint(ctx, config)?;

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
