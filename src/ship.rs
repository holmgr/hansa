use ggez::{nalgebra::normalize, Context, timer::get_delta, graphics::{Color as ggezColor, DrawParam, Point2, Rect}};
use std::time::Duration;

use config::Config;
use draw::Drawable;
use route::Waypoint;
use world::World;
use Position;
use update::Updatable;

/// A ship which transports resources between ports along a route.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ship {
    /// Duration since arriving at current position.
    duration: Duration, 
    position: Waypoint
}

impl Ship {
    const SPEED: f32 = 0.5;

    /// Creates a new ship.
    pub fn new(position: Waypoint) -> Self {
        Ship { 
            duration: Duration::new(0, 0),
            position 
        }
    }

    /// Returns a mutable reference to the ship's position.
    pub fn position(&self) -> Waypoint {
        self.position
    }
}

impl Updatable for Ship {
    type Data = Waypoint;

    /// Updates the internal duration since last waypoint, moving to new
    /// waypoint if enough time has past.
    fn update(&mut self, ctx: &Context, next_waypoint: &Waypoint) {
        let current_position = Position::from(self.position);
        let next_position = Position::from(*next_waypoint);
        let distance_to_next =
                    ((current_position.coords.x - next_position.coords.x).pow(2) as f32
                    + (current_position.coords.y - next_position.coords.y).pow(2) as f32).sqrt();
        let delta_since_move = get_delta(ctx) + self.duration;

        // Move to next ewaypoint.
        if (delta_since_move.as_secs() as u32 * 1000 + delta_since_move.subsec_millis()) as f32 * Self::SPEED / 1000. >= distance_to_next {
            self.position = *next_waypoint;
            self.duration = Duration::new(0, 0); // Does not consider if we over reach!!!
        }
        else {
            self.duration = delta_since_move;
        }
    }
}

impl<'a> Drawable<'a> for Ship {
    type Data = Waypoint; // Mouse position

    fn draw(&self, next_waypoint: &Waypoint) -> DrawParam {

        let next_position = Position::from(*next_waypoint);
        let current_position = Position::from(self.position);
        println!("Next position: {}, current: {}", next_position, current_position);
        let base_translation = Point2::new((next_position.coords.x - current_position.coords.x) as f32, (next_position.coords.y - current_position.coords.y) as f32);
        let magnitute = 
                    (base_translation.coords.x.powi(2)
                    + base_translation.coords.y.powi(2)).sqrt();
        let distance_traveled = (self.duration.as_secs() as u32 * 1000 + self.duration.subsec_millis()) as f32 * Self::SPEED / 1000.;
        //println!("Distance traveled: {}, magnitude: {}", distance_traveled, magnitute);
        let translate_factor = distance_traveled / magnitute;
        //println!("Translate factor: {}", translate_factor);
        
        let interpolated_position = Point2::new(current_position.coords.x as f32 + base_translation.coords.x * translate_factor,
                                                current_position.coords.y as f32 + base_translation.coords.y * translate_factor);
        //println!("Drawing ship on position: {}", interpolated_position);

        DrawParam {
            src: Rect::new(
                3. * Self::TILE_SIZE,
                2. * Self::TILE_SIZE,
                Self::TILE_SIZE,
                Self::TILE_SIZE,
            ),
            dest: interpolated_position,
            color: Some(ggezColor::from_rgb(69, 55, 52)),
            ..Default::default()
        }
    }
}

/// Holds all unplaced 'ships' and manages of drawing UI element for ship selection.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Shipyard {
    ships: usize
}

impl Shipyard {
    /// Create a new shipyard.
    pub fn new() -> Self {
        Shipyard {
            ships: 1 // TODO: Make this scale with progress.
        }
    }


    /// Returns the number of available ships.
    pub fn available(&self) -> usize {
        self.ships
    }

    /// Return if there are any ships available.
    pub fn is_available(&self) -> bool {
        self.ships > 0
    }

    /// Returns a shipbuilder if not capped on ships.
    pub fn build(&mut self) -> Option<ShipBuilder> {
        if self.is_available() {
            self.ships -= 1;
            Some(ShipBuilder::new())
        }
        else {
            None
        }
    }

    /// Returns a builder to the shipyard.
    pub fn add_builder(&mut self, builder: ShipBuilder) {
        self.ships += 1;
    }
}

impl<'a> Drawable<'a> for Shipyard {
    type Data = Config;

    fn draw(&self, data: &Config) -> DrawParam {
        let x_offset = (data.grid_width / 2) as f32;
        let y_offset = data.grid_height as f32 + 3.;
        DrawParam {
            src: Rect::new(
                3. * Self::TILE_SIZE,
                2. * Self::TILE_SIZE,
                Self::TILE_SIZE,
                Self::TILE_SIZE,
            ),
            dest: Point2::new(x_offset, y_offset),
            color: Some(ggezColor::from_rgb(69, 55, 52)),
            offset: Point2::new(0.5, 0.5),
            ..Default::default()
        }
    }
}

/// Manages the placement of ships on routes.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ShipBuilder;

impl ShipBuilder {
    /// Create a new ship builder.
    pub fn new() -> Self {
        ShipBuilder {}
    }

    /// Attempts to place a ship at the given position, consuming the ship builder.
    /// Returns the builder if it failed.
    pub fn try_place(self, position: Position, world: &mut World) -> Option<ShipBuilder> {
        // TODO: Implement placement on the given possition if it exists a
        // valid path there.

        let waypoint = Waypoint::from(position);
        if let Some((_, route)) = world.routes_mut().find(|(_, r)| r.contains(&waypoint)) {
            println!("Placed ship on route!");
            route.add_ship( Ship::new(waypoint) );
            None
        } else {
            Some(self)
        }
    }
}

impl<'a> Drawable<'a> for ShipBuilder {
    type Data = Point2; // Mouse position

    fn draw(&self, mouse: &Point2) -> DrawParam {
        DrawParam {
            src: Rect::new(
                3. * Self::TILE_SIZE,
                2. * Self::TILE_SIZE,
                Self::TILE_SIZE,
                Self::TILE_SIZE,
            ),
            dest: Point2::new(mouse.coords.x, mouse.coords.y),
            offset: Point2::new(0.5, 0.5),
            color: Some(ggezColor::from_rgb(69, 55, 52)),
            ..Default::default()
        }
    }
}
