use ggez::{
    event, graphics, mouse, timer, {Context, GameResult},
};
use std::{
    io::{BufRead, BufReader, Read},
    mem,
};

use color::ColorSelector;
use config::Config;
use draw::SpriteDrawer;
use geometry::Position;
use port::Port;
use route::RouteBuilder;
use ship::ShipBuilder;
use tile::{Tile, TileKind};
use update::Updatable;
use world::World;

const TILESET_PATH: &str = "/tileset.png";
const MAP_PATH: &str = "/map.ppm";

/// Load world from image file, mapping RGB to tiles.
fn load_world(ctx: &mut Context, config: &Config) -> World {
    let mut map_file = ctx.filesystem.open(MAP_PATH).unwrap();
    let mut header_buffer = [0; 58];
    map_file.read_exact(&mut header_buffer).unwrap();

    let map_file = BufReader::new(map_file);
    let buffer: Vec<i32> = map_file
        .lines()
        .map(|s| s.unwrap().parse::<i32>().unwrap())
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
pub struct GameState {
    config: Config,
    frames: usize,
    sprite_drawer: SpriteDrawer,
    world: World,
    route_builder: Option<RouteBuilder>,
    ship_builder: Option<ShipBuilder>,
    color_selector: ColorSelector,
}

impl GameState {
    /// Creates a new game state in Play mode.
    pub fn new(ctx: &mut Context, config: Config) -> GameResult<Self> {
        // Load game world from file.
        let world = load_world(ctx, &config);

        // Load spritebatch for effective drawing of sprites.
        let image = graphics::Image::new(ctx, TILESET_PATH)?;
        let sprite_drawer = SpriteDrawer::new(image);

        let state = GameState {
            config,
            frames: 0,
            sprite_drawer,
            world,
            route_builder: None,
            ship_builder: None,
            color_selector: ColorSelector::new(),
        };
        Ok(state)
    }
}

impl event::EventHandler for GameState {
    /// Updates the game state.
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        // Update all ships.
        for (_, route) in self.world.routes_mut() {
            let next_paths = route
                .ships()
                .map(|s| (s.reverse(), s.position(), s.next_waypoint()))
                .map(|(reverse, curr, next)| {
                    if next.is_none() && !reverse {
                        route.next_path(Position::from(curr))
                    } else if next.is_none() && reverse {
                        route.previous_path(Position::from(curr))
                    } else {
                        None
                    }
                }).collect::<Vec<_>>();
            next_paths
                .into_iter()
                .zip(route.ships_mut())
                .for_each(|(path, ship)| {
                    ship.update(ctx, path);
                });
        }
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
        let (window_width, _) = graphics::get_drawable_size(ctx);
        let cell_size = self.config.scaling * window_width / self.config.grid_width;

        // Check that we start to draw from a port.
        let mouse_position = Position::new(
            self.config.scaling as i32 * x,
            self.config.scaling as i32 * y,
        );
        let mouse_position_scaled = Position::new(
            self.config.scaling as i32 * x / cell_size as i32,
            self.config.scaling as i32 * y / cell_size as i32,
        );

        // Check if some mouse button on some color.
        let num_colors = self.color_selector.colors().count() as u32;
        let color_selector_x_offset =
            ((self.config.grid_width / 2 - num_colors + 1) * cell_size) as i32;
        let color_selector_y_offset = ((self.config.grid_height + 1) * cell_size) as i32;

        for index in 0..num_colors {
            let color_position = Position::new(
                color_selector_x_offset + (2 * index * cell_size + cell_size / 2) as i32,
                color_selector_y_offset + cell_size as i32 / 2,
            );

            // Check eucidean distance.
            if color_position.distance(&mouse_position) <= cell_size as f32 {
                self.color_selector.toggle(index as usize);
                println!("Toggling color: {:?}", self.color_selector.selected());
            }
        }

        // Not the prettiest code...
        let mut swap_builder = None;
        mem::swap(&mut self.ship_builder, &mut swap_builder);

        self.ship_builder = match swap_builder {
            Some(sb) => {
                // Try to add ship to route (if any) on the mouse position.
                // If it fails (and returns a ship), place it back on shipyard.
                if let Some(builder) = sb.try_place(mouse_position_scaled, &mut self.world) {
                    self.world.shipyard_mut().add_builder(builder);
                }
                None
            }
            _ => {
                // Check if some mouse button on shipyard.
                // TODO: Duplicated position logic.
                let shipyard_x_offset = ((self.config.grid_width / 2) * cell_size) as i32;
                let shipyard_y_offset = ((self.config.grid_height + 3) * cell_size) as i32;

                let shipyard_position = Position::new(shipyard_x_offset, shipyard_y_offset);
                // Check if player has any ship available.

                if shipyard_position.distance(&mouse_position) <= cell_size as f32 {
                    self.world.shipyard_mut().build()
                } else {
                    None
                }
            }
        };

        self.route_builder = match &self.route_builder {
            // Drawing already in progress, stop drawing.
            Some(rb) => {
                if self.world.port(mouse_position_scaled).is_some() {
                    // Add an actual route here if, ensure we have a color selected.
                    if let Some(color) = self.color_selector.selected() {
                        if let Some(path) = rb.path() {
                            self.world
                                .add_route(color, *rb.from(), *rb.to(), path.clone())
                        }
                    }
                }
                None
            }
            // Start drawing a new path
            None => {
                if self.world.port(mouse_position_scaled).is_some()
                    && self.color_selector.selected().is_some()
                {
                    Some(RouteBuilder::new(Position::new(
                        self.config.scaling as i32 * x / cell_size as i32,
                        self.config.scaling as i32 * y / cell_size as i32,
                    )))
                } else {
                    None
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
        let (window_width, _) = graphics::get_drawable_size(ctx);
        let cell_size = self.config.scaling * window_width / self.config.grid_width;
        if let Some(rb) = &mut self.route_builder {
            rb.update(
                Position::new(
                    self.config.scaling as i32 * x / cell_size as i32,
                    self.config.scaling as i32 * y / cell_size as i32,
                ),
                &self.world,
            );
        }
    }

    /// Draws the current state to the screen with the given context.
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        if self.frames % 100 == 0 {
            println!("FPS: {:.1}", timer::get_fps(ctx));
        }

        graphics::clear(ctx);
        self.sprite_drawer.clear();

        // Draw all base tiles.
        for tile in self.world.tiles() {
            self.sprite_drawer
                .draw_item(ctx, &self.config, tile, &self.world, true);
        }

        // Draw all routes.
        for (color, route) in self.world.routes() {
            for waypoint in route.waypoints() {
                self.sprite_drawer
                    .draw_item(ctx, &self.config, waypoint, color, true);
            }
        }

        // Draw all ships.
        for (_, route) in self.world.routes() {
            for ship in route.ships() {
                // TODO: Must handle waypoints ending, and returning ships back.
                self.sprite_drawer
                    .draw_item(ctx, &self.config, ship, &(), true);
            }
        }

        // Draw route (id any) currently being created.
        if let Some(builder) = &self.route_builder {
            if let Some(waypoints) = builder.path() {
                let color = self.color_selector.selected().unwrap();
                for waypoint in waypoints {
                    self.sprite_drawer
                        .draw_item(ctx, &self.config, waypoint, &color, true);
                }
            }
        }

        // Draw all ports.
        for port in self.world.ports() {
            self.sprite_drawer
                .draw_item(ctx, &self.config, port, &self.world, true);
        }

        // Draw color selector.
        for color in self.color_selector.colors() {
            self.sprite_drawer.draw_item(
                ctx,
                &self.config,
                color,
                &(&self.config, &self.color_selector),
                true,
            );
        }

        // Draw shipyard.
        self.sprite_drawer.draw_item(
            ctx,
            &self.config,
            self.world.shipyard_mut(),
            &self.config,
            true,
        );

        // Draw ship icon under mouse if being held by player.
        if let Some(sb) = &self.ship_builder {
            let mouse_position =
                mouse::get_position(ctx).expect("Could not retrive mouse position");
            self.sprite_drawer
                .draw_item(ctx, &self.config, sb, &mouse_position, false);
        }

        // Draw to screen.
        self.sprite_drawer.paint(ctx, &self.config)?;

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
