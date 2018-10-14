use ggez::{
    event, graphics, mouse, timer, {Context, GameResult},
};
use rand::prelude::*;
use rand::{Rng, ThreadRng};
use std::{
    io::{BufRead, BufReader, Read},
    mem,
    time::Duration,
};

use animation::{Animation, AnimationType};
use config::Config;
use draw::{Drawable, SpriteDrawer};
use geometry::Position;
use port::{is_valid_arrangement, Port};
use route::{RouteBuilder, ShapeSelector, Waypoint};
use ship::ShipBuilder;
use tile::{Tile, TileKind};
use time::GameTimer;
use update::Updatable;
use world::World;

const TILESET_PATH: &str = "/tileset.png";
const MAP_PATH: &str = "/map.ppm";

/// Load world from image file, mapping RGB to tiles.
fn load_world<R: Rng>(ctx: &mut Context, config: &Config, color_sampler: &mut R) -> World {
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
                    ports.push(Port::new(position, color_sampler));
                    map.push(Tile::new(position, TileKind::Land));
                }
                _ => map.push(Tile::new(position, TileKind::Land)),
            }
        });
    World::new(map.into_iter(), ports.into_iter())
}

/// Time of a single game session: 5min.
static GAME_TIME_LENGTH: u64 = 60 * 5;

/// Handles and holds all game information.
pub struct GameState {
    font: graphics::Font,
    config: Config,
    frames: usize,
    sprite_drawer: SpriteDrawer,
    world: World,
    route_builder: Option<RouteBuilder>,
    ship_builder: Option<ShipBuilder>,
    shape_selector: ShapeSelector,
    rng: ThreadRng,
    game_timer: GameTimer,
    last_color_shuffle: Duration,
}

impl GameState {
    /// Creates a new game state in Play mode.
    pub fn new(ctx: &mut Context, config: Config) -> GameResult<Self> {
        let mut rng = thread_rng();
        // Load game world from file.
        let world = load_world(ctx, &config, &mut rng);

        // Load spritebatch for effective drawing of sprites.
        let image = graphics::Image::new(ctx, TILESET_PATH)?;
        let sprite_drawer = SpriteDrawer::new(image);

        let state = GameState {
            font: graphics::Font::new(ctx, "/RobotoMono-Regular.ttf", 48)
                .expect("Failed to load font"),
            config,
            frames: 0,
            sprite_drawer,
            world,
            route_builder: None,
            ship_builder: None,
            shape_selector: ShapeSelector::new(),
            rng: thread_rng(),
            last_color_shuffle: timer::get_time_since_start(ctx),
            game_timer: GameTimer::new(
                timer::get_time_since_start(ctx),
                Duration::from_secs(GAME_TIME_LENGTH),
            ),
        };
        Ok(state)
    }

    /// Updates the port colors by switching one port randomly until it is valid.
    fn update_port_colors(&mut self) {
        let ports = self.world.ports_mut();
        loop {
            // Extra brackes due to NLL not existing in stable Rust yet.
            {
                let port = self.rng.choose_mut(ports).unwrap();
                let (import, export) = Port::sample_colors(&mut self.rng);

                // Update if we got new colors.
                if port.import() != import || port.export() != export {
                    // Got valid colors
                    *port.import_mut() = import;
                    *port.export_mut() = export;

                    // Add animation.
                    *port.animation_mut() = Some(Animation::new(
                        Duration::new(1, 0),
                        AnimationType::PulseScale {
                            amplitude: 0.4,
                            rate: 1.,
                        },
                    ));
                } else {
                    continue;
                }
            }
            if is_valid_arrangement(ports) {
                break;
            }
        }
    }

    /// Ends the game session.
    /// TODO: Currently only dumps the final score and quits.
    fn end_game(&mut self, ctx: &mut Context) {
        println!("GAME OVER: Score {}", self.world.tally_mut().score());
        ctx.quit().expect("Failed to quit game");
    }
}

impl event::EventHandler for GameState {
    /// Updates the game state.
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        // Check if game time is up, end game in such case.
        if self.game_timer.has_game_ended(ctx) {
            self.end_game(ctx);
        }

        while timer::check_update_time(ctx, 60) {
            // Update all ships.
            let tradings = self
                .world
                .ports()
                .iter()
                .map(|p| (p.position(), p.import(), p.export()))
                .collect::<Vec<_>>();
            let mut new_colors = vec![];

            for (_, route) in self.world.routes_mut() {
                let next_paths = route
                    .ships()
                    .map(|s| (s.reverse(), s.is_arriving(), s.next_waypoint().unwrap()))
                    .map(|(reverse, is_arriving, curr)| {
                        if is_arriving && !reverse {
                            route.next_path(Position::from(curr))
                        } else if is_arriving && reverse {
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

                for ship in route.ships_mut() {
                    // Remove all animations that have finished.
                    *ship.animation_mut() = match ship.animation_mut() {
                        Some(ref mut animation) => {
                            animation.update(ctx, ());
                            if animation.has_finished() {
                                None
                            } else {
                                Some(*animation)
                            }
                        }
                        None => None,
                    };

                    if ship.is_docked() {
                        let mut animation_length = 500;
                        // TODO: Quick fix, We have no cargo, extend loading animation.
                        if ship.cargo().is_none() {
                            animation_length *= 2;
                        }
                        let (_, import, export) = tradings
                            .iter()
                            .find(|(p, _, _)| *p == Position::from(ship.position()))
                            .expect("No port at ship dock");
                        if let Some(cargo) = ship.try_unload() {
                            // Unload ship cargo, adding to score if it matches port import.
                            if cargo == *import {
                                new_colors.push(cargo);
                            }
                            // TODO: Fix animation of throwing away cargo.
                            if ship.animation().is_none() {
                                *ship.animation_mut() = Some(Animation::new(
                                    Duration::from_millis(animation_length),
                                    AnimationType::ColorDrain {
                                        from: Some(cargo),
                                        to: None,
                                    },
                                ));
                            }
                        } else {
                            ship.try_load(*export);
                            // Add cargo loading animation if already not animated.
                            if ship.animation().is_none() {
                                *ship.animation_mut() = Some(Animation::new(
                                    Duration::from_millis(animation_length),
                                    AnimationType::ColorDrain {
                                        from: None,
                                        to: Some(*export),
                                    },
                                ));
                            }
                        }
                    }
                }
            }

            // Add score for all colors collected.
            new_colors
                .into_iter()
                .for_each(|c| self.world.tally_mut().update(c));

            // Update all port animations.
            for port in self.world.ports_mut() {
                *port.animation_mut() = match port.animation_mut() {
                    Some(ref mut animation) => {
                        animation.update(ctx, ());
                        if animation.has_finished() {
                            None
                        } else {
                            Some(*animation)
                        }
                    }
                    None => None,
                }
            }
        }

        // Swtich some port import/export every 15s.
        // TODO: Move magic constant.
        if (timer::get_time_since_start(ctx) - self.last_color_shuffle).as_secs() > 15 {
            self.last_color_shuffle = timer::get_time_since_start(ctx);
            self.update_port_colors();
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
        let cell_size = (self.config.scaling * window_width) as f32 / self.config.grid_width as f32;

        // Check that we start to draw from a port.
        let mouse_position = Position::new(
            self.config.scaling as i32 * x,
            self.config.scaling as i32 * y,
        );
        let mouse_position_scaled = Position::new(
            ((self.config.scaling as f32 * x as f32) / cell_size) as i32,
            ((self.config.scaling as f32 * y as f32) / cell_size) as i32,
        );

        // Check if some mouse button on some shape.
        let num_shapes = self.shape_selector.shapes().count() as u32;
        let shape_selector_x_offset =
            (self.config.grid_width as f32 / 2. - num_shapes as f32 + 1.) * cell_size;
        let shape_selector_y_offset = (self.config.grid_height as f32 + 1.) * cell_size;

        for index in 0..num_shapes {
            let shape_position = Position::new(
                (shape_selector_x_offset + ((2. * index as f32 + 0.5) * cell_size)) as i32,
                (shape_selector_y_offset + cell_size / 2.) as i32,
            );

            // Check eucidean distance.
            if shape_position.distance(mouse_position) <= cell_size as f32 {
                self.shape_selector.toggle(index as usize);
                println!("Toggling shape: {:?}", self.shape_selector.selected());
            }
        }

        // Keeps track of whether we have changed the selection.
        // I.e created a ship or route builder or destroyed one.
        // TODO: Not the prettiest solution but hey.
        let mut has_selection_changed = false;

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
                has_selection_changed = true;
                None
            }
            _ => {
                // Check if some mouse button on shipyard.
                // TODO: Duplicated position logic.
                let shipyard_x_offset = ((self.config.grid_width as f32 / 2.) * cell_size) as i32;
                let shipyard_y_offset = ((self.config.grid_height as f32 + 2.5) * cell_size) as i32;

                let shipyard_position = Position::new(shipyard_x_offset, shipyard_y_offset);
                // Check if player has any ship available.

                if shipyard_position.distance(mouse_position) <= cell_size as f32 {
                    has_selection_changed = true;
                    self.world.shipyard_mut().build()
                } else {
                    None
                }
            }
        };

        self.route_builder = match &self.route_builder {
            // Drawing already in progress, stop drawing.
            Some(rb) => {
                // Remove all port animations.
                for port in self.world.ports_mut() {
                    *port.animation_mut() = None;
                }

                has_selection_changed = true;
                match self.shape_selector.selected() {
                    Some(shape) if self.world.port(mouse_position_scaled).is_some() => {
                        let allowed_ends = self.world.allowed_ends(*rb.from(), shape);

                        if allowed_ends.iter().any(|p| *p == mouse_position_scaled) {
                            if let Some(path) = rb.path() {
                                self.world
                                    .add_route(shape, *rb.from(), *rb.to(), path.clone())
                            }
                        }
                        None
                    }
                    _ => None,
                }
            }
            // Start drawing a new path
            None => match self.shape_selector.selected() {
                Some(ref shape) if self.world.port(mouse_position_scaled).is_some() => {
                    if self
                        .world
                        .allowed_starts(*shape)
                        .iter()
                        .any(|p| *p == mouse_position_scaled)
                    {
                        // Add port animations to valid end_points
                        let allowed_ends = self.world.allowed_ends(mouse_position_scaled, *shape);
                        for port in self.world.ports_mut() {
                            if allowed_ends.iter().any(|p| *p == port.position()) {
                                *port.animation_mut() = Some(Animation::new(
                                    Duration::new(3600, 0),
                                    AnimationType::PulseScale {
                                        amplitude: 0.1,
                                        rate: 1.,
                                    },
                                ));
                            }
                        }
                        has_selection_changed = true;
                        Some(RouteBuilder::new(mouse_position_scaled))
                    } else {
                        None
                    }
                }
                _ => None,
            },
        };

        // If click on some waypoint, remove all routes through and return ships to shipyard.
        if !has_selection_changed && self.route_builder.is_none() && self.ship_builder.is_none() {
            let ships_removed = self
                .world
                .remove_routes_at(Waypoint::from(mouse_position_scaled));
            for ship in ships_removed {
                self.world.shipyard_mut().add_ship(ship);
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
        for (shape, route) in self.world.routes() {
            for waypoint in route.waypoints() {
                self.sprite_drawer
                    .draw_item(ctx, &self.config, waypoint, shape, true);
            }
        }

        // Draw route (id any) currently being created.
        if let Some(builder) = &self.route_builder {
            if let Some(waypoints) = builder.path() {
                let shape = self.shape_selector.selected().unwrap();
                for waypoint in waypoints {
                    self.sprite_drawer
                        .draw_item(ctx, &self.config, waypoint, &shape, true);
                }
            }
        }

        // Draw all ports.
        for port in self.world.ports() {
            self.sprite_drawer
                .draw_item(ctx, &self.config, port, &self.world, true);
        }

        // Draw all ships.
        for (_, route) in self.world.routes() {
            for ship in route.ships() {
                // TODO: Must handle waypoints ending, and returning ships back.
                self.sprite_drawer
                    .draw_item(ctx, &self.config, ship, &(), true);
            }
        }

        // Draw shape selector.
        for shape in self.shape_selector.shapes() {
            self.sprite_drawer.draw_item(
                ctx,
                &self.config,
                shape,
                &(&self.config, &self.shape_selector),
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

        // Draw tally.
        self.world
            .tally_mut()
            .paint(&self.font, ctx, &self.config)?;

        // Draw remaining game time.
        self.game_timer.paint(&self.font, ctx, &self.config)?;

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
